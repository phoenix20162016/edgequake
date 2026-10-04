//! SPEC-160 — Postgres [`DecisionStore`] over `decision_cache` and
//! `decision_review` (migration 166).
//!
//! Every query filters by `workspace_id` (EC-160-39). A non-UUID workspace or
//! document id cannot be represented: reads return `None`/empty and writes are
//! skipped. The extractor then runs without a cache, which is slower and
//! still correct (same rule as `ingestion_dedup`).

use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::decision::{DecisionScope, DecisionStore, ReviewKind, ReviewRow};
use crate::error::StorageError;

/// Postgres-backed decision store.
#[derive(Clone)]
pub struct PostgresDecisionStore {
    pool: PgPool,
}

impl PostgresDecisionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn uuid_of(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw).ok()
}

fn ws_of(scope: &DecisionScope) -> Option<Uuid> {
    uuid_of(&scope.workspace_id)
}

fn db_err(what: &str, e: sqlx::Error) -> StorageError {
    StorageError::Database(format!("{what} failed: {e}"))
}

#[async_trait]
impl DecisionStore for PostgresDecisionStore {
    async fn get_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
    ) -> Result<Option<Value>, StorageError> {
        let Some(ws) = ws_of(scope) else {
            return Ok(None);
        };
        let row: Option<(Value,)> = sqlx::query_as(
            "UPDATE public.decision_cache SET last_used_at = now() \
             WHERE workspace_id = $1 AND key_hash = $2 RETURNING answer",
        )
        .bind(ws)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| db_err("decision_cache get", e))?;
        Ok(row.map(|(v,)| v))
    }

    async fn put_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
        model: &str,
        contract: &str,
        answer: &Value,
    ) -> Result<(), StorageError> {
        let Some(ws) = ws_of(scope) else {
            return Ok(());
        };
        sqlx::query(
            "INSERT INTO public.decision_cache \
                 (workspace_id, key_hash, tenant_id, contract, model, answer) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (workspace_id, key_hash) DO UPDATE SET \
                 answer = EXCLUDED.answer, model = EXCLUDED.model, \
                 contract = EXCLUDED.contract, last_used_at = now()",
        )
        .bind(ws)
        .bind(key)
        .bind(scope.tenant_id.as_deref().and_then(uuid_of))
        .bind(contract)
        .bind(model)
        .bind(answer)
        .execute(&self.pool)
        .await
        .map_err(|e| db_err("decision_cache put", e))?;
        Ok(())
    }

    async fn replace_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
        rows: &[ReviewRow],
    ) -> Result<(), StorageError> {
        let (Some(ws), Some(doc)) = (ws_of(scope), uuid_of(document_id)) else {
            return Ok(());
        };
        let tenant = scope.tenant_id.as_deref().and_then(uuid_of);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| db_err("decision_review begin", e))?;
        sqlx::query(
            "DELETE FROM public.decision_review WHERE workspace_id = $1 AND document_id = $2",
        )
        .bind(ws)
        .bind(doc)
        .execute(&mut *tx)
        .await
        .map_err(|e| db_err("decision_review clear", e))?;
        for r in rows {
            sqlx::query(
                "INSERT INTO public.decision_review \
                     (workspace_id, document_id, tenant_id, chunk_id, kind, subject, label, \
                      object, score, reason, sentence, model, contract) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
            )
            .bind(ws)
            .bind(doc)
            .bind(tenant)
            .bind(&r.chunk_id)
            .bind(r.kind.as_str())
            .bind(&r.subject)
            .bind(&r.label)
            .bind(r.object.as_deref())
            .bind(r.score)
            .bind(r.reason.as_deref())
            .bind(&r.sentence)
            .bind(&r.model)
            .bind(&r.contract)
            .execute(&mut *tx)
            .await
            .map_err(|e| db_err("decision_review insert", e))?;
        }
        tx.commit()
            .await
            .map_err(|e| db_err("decision_review commit", e))
    }

    async fn list_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<Vec<ReviewRow>, StorageError> {
        let (Some(ws), Some(doc)) = (ws_of(scope), uuid_of(document_id)) else {
            return Ok(Vec::new());
        };
        type Row = (
            String,
            String,
            String,
            String,
            Option<String>,
            f32,
            Option<String>,
            String,
            String,
            String,
        );
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT chunk_id, kind, subject, label, object, score, reason, sentence, model, contract \
             FROM public.decision_review WHERE workspace_id = $1 AND document_id = $2 \
             ORDER BY score DESC, created_at ASC",
        )
        .bind(ws)
        .bind(doc)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| db_err("decision_review list", e))?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                Some(ReviewRow {
                    chunk_id: r.0,
                    kind: ReviewKind::parse(&r.1)?,
                    subject: r.2,
                    label: r.3,
                    object: r.4,
                    score: r.5,
                    reason: r.6,
                    sentence: r.7,
                    model: r.8,
                    contract: r.9,
                })
            })
            .collect())
    }

    async fn delete_document(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<u64, StorageError> {
        let (Some(ws), Some(doc)) = (ws_of(scope), uuid_of(document_id)) else {
            return Ok(0);
        };
        sqlx::query(
            "DELETE FROM public.decision_review WHERE workspace_id = $1 AND document_id = $2",
        )
        .bind(ws)
        .bind(doc)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| db_err("decision_review delete", e))
    }

    async fn delete_workspace(&self, scope: &DecisionScope) -> Result<u64, StorageError> {
        let Some(ws) = ws_of(scope) else {
            return Ok(0);
        };
        let mut total = 0;
        for table in ["decision_cache", "decision_review"] {
            let sql = format!("DELETE FROM public.{table} WHERE workspace_id = $1");
            total += sqlx::query(&sql)
                .bind(ws)
                .execute(&self.pool)
                .await
                .map_err(|e| db_err("decision workspace delete", e))?
                .rows_affected();
        }
        Ok(total)
    }

    async fn sweep_cache(
        &self,
        scope: &DecisionScope,
        ttl_days: u32,
        max_rows: u64,
    ) -> Result<u64, StorageError> {
        let Some(ws) = ws_of(scope) else {
            return Ok(0);
        };
        let aged = sqlx::query(
            "DELETE FROM public.decision_cache WHERE workspace_id = $1 \
             AND last_used_at < now() - make_interval(days => $2)",
        )
        .bind(ws)
        .bind(i32::try_from(ttl_days).unwrap_or(i32::MAX))
        .execute(&self.pool)
        .await
        .map_err(|e| db_err("decision_cache ttl sweep", e))?
        .rows_affected();
        let excess = sqlx::query(
            "DELETE FROM public.decision_cache WHERE workspace_id = $1 AND key_hash IN ( \
                 SELECT key_hash FROM public.decision_cache WHERE workspace_id = $1 \
                 ORDER BY last_used_at DESC, key_hash OFFSET $2)",
        )
        .bind(ws)
        .bind(i64::try_from(max_rows).unwrap_or(i64::MAX))
        .execute(&self.pool)
        .await
        .map_err(|e| db_err("decision_cache cap sweep", e))?
        .rows_affected();
        Ok(aged + excess)
    }

    async fn cache_len(&self, scope: &DecisionScope) -> Result<u64, StorageError> {
        let Some(ws) = ws_of(scope) else {
            return Ok(0);
        };
        let (n,): (i64,) =
            sqlx::query_as("SELECT count(*) FROM public.decision_cache WHERE workspace_id = $1")
                .bind(ws)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| db_err("decision_cache count", e))?;
        Ok(n.max(0) as u64)
    }
}
