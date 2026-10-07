//! Typed ANN registry key: workspace model hits; wrong name at same dim is empty.
//!
//! Reproduces the Ask/mistral-embed gap: process env may be
//! `text-embedding-3-small` while vectors are registered under `mistral-embed@1024`.
//! `MetadataFilter.embedding_model` must select the correct registry row; a miss
//! must not search another model's space.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres \
//!       --test e2e_typed_ann_model_name_hit_vs_miss -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::adapters::postgres::{PgVectorStorage, PostgresPool};
use edgequake_storage::traits::domain::{EmbeddingIndex, EmbeddingRow, ModelId, WorkspaceId};
use edgequake_storage::traits::{MetadataFilter, VectorStorage};
use edgequake_storage::VECTOR_BACKEND_ENV;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use uuid::Uuid;

const DIM: usize = 1024;
const HIT_MODEL: &str = "mistral-embed";
const MISS_MODEL: &str = "text-embedding-3-small";

async fn table_exists(pool: &sqlx::PgPool, table: &str) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = $1)",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_typed_ann_model_name_hit_vs_miss() {
    let Some(cfg) = require_or_skip_postgres("typed_ann_model_hit_miss") else {
        return;
    };
    let _g = w3::w3_env_guard().await;
    let pool = contract_pg_pool(&cfg).await;

    if !table_exists(&pool, "chunk_embeddings").await {
        eprintln!("skip: chunk_embeddings missing — run migrations");
        return;
    }

    let ns = format!("mhit_{}", Uuid::new_v4().as_simple());
    std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");
    // Process default points at the wrong name — preferred filter must still hit.
    std::env::set_var("EDGEQUAKE_EMBEDDING_MODEL", MISS_MODEL);

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns;
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        DIM,
    );
    storage.initialize().await.expect("typed initialize");

    let ws = w3::seed_workspace(&pool, "model-hit").await;
    let doc = w3::seed_document(&pool, ws).await;
    let chunk_id = w3::seed_chunk(&pool, doc, ws, 0, "mistral-only row").await;
    let emb = w3::make_embedding(DIM, 7);

    let index = edgequake_storage::PgChunkEmbeddingIndex::new(pool.clone(), HIT_MODEL);
    index
        .upsert_batch(
            ModelId(Uuid::nil()),
            &[EmbeddingRow {
                chunk_id: chunk_id.into(),
                workspace_id: WorkspaceId::new(ws),
                dimensions: DIM as i32,
                embedding: emb.clone(),
            }],
        )
        .await
        .expect("upsert mistral-embed@1024");
    sqlx::query(
        "INSERT INTO public.chunk_serving_state (chunk_id, state) VALUES ($1, 'ready') \
         ON CONFLICT (chunk_id) DO UPDATE SET state = EXCLUDED.state",
    )
    .bind(chunk_id)
    .execute(&pool)
    .await
    .expect("serving fence");

    let hit_mf = MetadataFilter {
        workspace_id: Some(ws.to_string()),
        vector_type: Some("chunk".into()),
        embedding_model: Some(HIT_MODEL.into()),
        ..Default::default()
    };
    let hits = storage
        .query_filtered(&emb, 5, None, Some(&hit_mf))
        .await
        .expect("hit query");
    assert!(
        !hits.is_empty(),
        "mistral-embed@1024 preferred filter must return typed hits; got {hits:?}"
    );

    // Seed a *different* registry space under the env model name so fallthrough
    // would falsely hit if preferred→env candidates were still active.
    let distractor = w3::make_embedding(DIM, 99);
    let miss_index = edgequake_storage::PgChunkEmbeddingIndex::new(pool.clone(), MISS_MODEL);
    let distractor_chunk = w3::seed_chunk(&pool, doc, ws, 1, "env-model distractor").await;
    miss_index
        .upsert_batch(
            ModelId(Uuid::nil()),
            &[EmbeddingRow {
                chunk_id: distractor_chunk.into(),
                workspace_id: WorkspaceId::new(ws),
                dimensions: DIM as i32,
                embedding: distractor,
            }],
        )
        .await
        .expect("upsert env-model distractor");
    sqlx::query(
        "INSERT INTO public.chunk_serving_state (chunk_id, state) VALUES ($1, 'ready') \
         ON CONFLICT (chunk_id) DO UPDATE SET state = EXCLUDED.state",
    )
    .bind(distractor_chunk)
    .execute(&pool)
    .await
    .expect("serving fence distractor");

    // Preferred wrong name (or unregistered preferred) must not search env space.
    let miss_mf = MetadataFilter {
        workspace_id: Some(ws.to_string()),
        vector_type: Some("chunk".into()),
        embedding_model: Some("not-a-registered-model".into()),
        ..Default::default()
    };
    let miss = storage
        .query_filtered(&emb, 5, None, Some(&miss_mf))
        .await
        .expect("miss query");
    assert!(
        miss.is_empty(),
        "unregistered preferred must return empty even when env model has rows; got {miss:?}"
    );

    let wrong_name_mf = MetadataFilter {
        workspace_id: Some(ws.to_string()),
        vector_type: Some("chunk".into()),
        embedding_model: Some(MISS_MODEL.into()),
        ..Default::default()
    };
    let wrong = storage
        .query_filtered(&emb, 5, None, Some(&wrong_name_mf))
        .await
        .expect("wrong-name query");
    // Query vector matches HIT_MODEL row geometry; MISS_MODEL has distractor only —
    // still searching MISS space is OK (preferred=MISS). Must not return HIT rows.
    assert!(
        wrong.iter().all(|r| r.id != hits[0].id),
        "preferred env-model search must not leak mistral rows; wrong={wrong:?} hit={hits:?}"
    );

    let _ = sqlx::query("DELETE FROM chunk_embeddings WHERE workspace_id = $1")
        .bind(ws)
        .execute(&pool)
        .await;
    w3::cleanup_workspace(&pool, ws).await;
    std::env::remove_var(VECTOR_BACKEND_ENV);
    std::env::remove_var("EDGEQUAKE_EMBEDDING_MODEL");
}
