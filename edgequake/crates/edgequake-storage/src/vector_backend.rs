//! SPEC-091 IW2 — vector backend flag (chunk + fleet embeddings).

/// Which store serves embeddings during the SPEC-091 cutover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorBackend {
    /// Legacy `eq_*_vectors` tables (explicit rollback only).
    LegacyTables,
    /// Typed `chunk_embeddings` + fleet tables (migration 108/130) — **default**.
    TypedEmbeddings,
}

pub const VECTOR_BACKEND_ENV: &str = "EDGEQUAKE_VECTOR_BACKEND";

/// Read `EDGEQUAKE_VECTOR_BACKEND`.
///
/// Unset / empty → [`VectorBackend::TypedEmbeddings`] (post dual-write soak).
/// Set `legacy_tables` / `legacy` for explicit rollback during ≤0.22 soak only.
/// Unknown values → **TypedEmbeddings** (SPEC-105 LAW-L2) — never silently
/// select LegacyTables.
///
/// **Authority:** [`VectorBackend::TypedEmbeddings`] is both **read and write**
/// authority — legacy `eq_*_vectors` upserts are write-stopped at the adapter
/// (typed `chunk_embeddings` / fleet tables are the SSOT). [`VectorBackend::LegacyTables`]
/// restores legacy INSERT/CREATE for soak rollback only while census > 0.
pub fn vector_backend_from_env() -> VectorBackend {
    match std::env::var(VECTOR_BACKEND_ENV)
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "typed_embeddings" | "chunk_embeddings" => VectorBackend::TypedEmbeddings,
        "legacy_tables" | "legacy" => VectorBackend::LegacyTables,
        _ => VectorBackend::TypedEmbeddings,
    }
}

pub fn vector_backend_reads_typed(mode: VectorBackend) -> bool {
    matches!(mode, VectorBackend::TypedEmbeddings)
}

/// Env key for the process-default embedding model name (typed ANN registry).
pub const EMBEDDING_MODEL_ENV: &str = "EDGEQUAKE_EMBEDDING_MODEL";

/// Default registry name when env is unset or empty (Compose `:-`).
pub const DEFAULT_EMBEDDING_MODEL_KEY: &str = "text-embedding-3-small";

/// Resolve the process-default embedding model key for typed ANN.
///
/// Compose often sets `EDGEQUAKE_EMBEDDING_MODEL=` (empty). Treat that as unset
/// so we do not look up `embedding_models.name = ''`.
pub fn embedding_model_key_from_env() -> String {
    std::env::var(EMBEDDING_MODEL_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_EMBEDDING_MODEL_KEY.to_string())
}

/// Ordered typed-ANN registry name candidates for a query/ingest vector.
///
/// When a workspace/lineage model is set, **only** that name is searched — a
/// miss must not fall through into another model's vector space (e.g. env
/// `text-embedding-3-small` while the embedder is `mistral-embed`). When
/// preferred is absent/empty, use [`embedding_model_key_from_env`].
pub fn serving_embedding_model_candidates(preferred: Option<&str>) -> Vec<String> {
    if let Some(p) = preferred.map(str::trim).filter(|s| !s.is_empty()) {
        return vec![p.to_string()];
    }
    vec![embedding_model_key_from_env()]
}

/// SPEC-091: when typed is authority, legacy `eq_*_vectors` **serving writes**
/// (INSERT / UPSERT / CREATE) must no-op.
///
/// Lifecycle **DELETE** / `clear_workspace` / `delete_by_document` still run when
/// the relation exists (wipe / document retract must not leave orphan fleet
/// rows that poison iw2 / provenance-stamp verify). Missing relation is skipped
/// **before** SQL (SPEC-383) so Postgres never logs 42P01; `map_legacy_mutate_err`
/// remains the TOCTOU fallback if the table is dropped between probe and execute.
pub fn legacy_vector_writes_stopped() -> bool {
    vector_backend_reads_typed(vector_backend_from_env())
}

impl VectorBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            VectorBackend::LegacyTables => "legacy_tables",
            VectorBackend::TypedEmbeddings => "typed_embeddings",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env_lock::test_env_lock;

    #[test]
    fn contract_spec091_vector_backend_default_typed() {
        let _g = test_env_lock();
        std::env::remove_var(VECTOR_BACKEND_ENV);
        assert_eq!(vector_backend_from_env(), VectorBackend::TypedEmbeddings);
    }

    #[test]
    fn contract_spec091_vector_backend_typed() {
        let _g = test_env_lock();
        std::env::set_var(VECTOR_BACKEND_ENV, "chunk_embeddings");
        assert_eq!(vector_backend_from_env(), VectorBackend::TypedEmbeddings);
        assert!(vector_backend_reads_typed(vector_backend_from_env()));
        std::env::remove_var(VECTOR_BACKEND_ENV);
    }

    #[test]
    fn contract_spec091_vector_backend_typed_alias() {
        let _g = test_env_lock();
        std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");
        assert_eq!(vector_backend_from_env(), VectorBackend::TypedEmbeddings);
        std::env::remove_var(VECTOR_BACKEND_ENV);
    }

    #[test]
    fn contract_spec091_vector_backend_explicit_legacy() {
        let _g = test_env_lock();
        std::env::set_var(VECTOR_BACKEND_ENV, "legacy_tables");
        assert_eq!(vector_backend_from_env(), VectorBackend::LegacyTables);
        std::env::remove_var(VECTOR_BACKEND_ENV);
    }

    #[test]
    fn contract_spec091_vector_backend_unknown_is_typed() {
        let _g = test_env_lock();
        std::env::set_var(VECTOR_BACKEND_ENV, "bogus");
        assert_eq!(vector_backend_from_env(), VectorBackend::TypedEmbeddings);
        std::env::remove_var(VECTOR_BACKEND_ENV);
    }

    #[test]
    fn e2e_105_01_unknown_vector_backend_typed() {
        let _g = test_env_lock();
        std::env::set_var(VECTOR_BACKEND_ENV, "not-a-backend");
        assert_eq!(vector_backend_from_env(), VectorBackend::TypedEmbeddings);
        std::env::remove_var(VECTOR_BACKEND_ENV);
    }

    #[test]
    fn embedding_model_key_empty_env_is_default() {
        let _g = test_env_lock();
        std::env::remove_var(EMBEDDING_MODEL_ENV);
        assert_eq!(embedding_model_key_from_env(), DEFAULT_EMBEDDING_MODEL_KEY);
        std::env::set_var(EMBEDDING_MODEL_ENV, "");
        assert_eq!(embedding_model_key_from_env(), DEFAULT_EMBEDDING_MODEL_KEY);
        std::env::set_var(EMBEDDING_MODEL_ENV, "  mistral-embed  ");
        assert_eq!(embedding_model_key_from_env(), "mistral-embed");
        std::env::remove_var(EMBEDDING_MODEL_ENV);
    }

    #[test]
    fn serving_candidates_preferred_only_no_env_fallthrough() {
        let _g = test_env_lock();
        std::env::set_var(EMBEDDING_MODEL_ENV, "text-embedding-3-small");
        assert_eq!(
            serving_embedding_model_candidates(Some("mistral-embed")),
            vec!["mistral-embed".to_string()]
        );
        assert_eq!(
            serving_embedding_model_candidates(Some("text-embedding-3-small")),
            vec!["text-embedding-3-small".to_string()]
        );
        assert_eq!(
            serving_embedding_model_candidates(None),
            vec!["text-embedding-3-small".to_string()]
        );
        assert_eq!(
            serving_embedding_model_candidates(Some("")),
            vec!["text-embedding-3-small".to_string()]
        );
        std::env::remove_var(EMBEDDING_MODEL_ENV);
    }
}
