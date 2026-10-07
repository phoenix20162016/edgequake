//! Default-vector retrieval — thin delegates to `vector_queries` (SPEC-017 QUERY-DRY-004).
//!
//! Single implementation lives in `vector_queries::*_with_vector_storage`; default-path
//! callers use `&self.vector_storage` so production and workspace paths share semantics
//! (hybrid includes local + global + naive; local fallback continues chunk collection).

use crate::context::QueryContext;
use crate::error::Result;
use crate::keywords::ExtractedKeywords;
use crate::mix_weights::MixWeightOverride;

use super::{QueryEmbeddings, QueryEngine};

impl QueryEngine {
    pub(super) async fn query_local(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        self.query_local_with_vector_storage(
            query_text,
            keywords,
            embeddings,
            tenant_id,
            workspace_id,
            None, // no document scope on default path
            &self.vector_storage,
            max_chunks,
        )
        .await
    }

    pub(super) async fn query_global(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        self.query_global_with_vector_storage(
            query_text,
            keywords,
            embeddings,
            tenant_id,
            workspace_id,
            None,
            &self.vector_storage,
            max_chunks,
        )
        .await
    }

    pub(super) async fn query_hybrid(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        self.query_hybrid_with_vector_storage(
            query_text,
            keywords,
            embeddings,
            tenant_id,
            workspace_id,
            None,
            &self.vector_storage,
            max_chunks,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn query_mix(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        mix_weights: Option<&MixWeightOverride>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        self.query_mix_with_vector_storage(
            query_text,
            keywords,
            embeddings,
            tenant_id,
            workspace_id,
            None,
            &self.vector_storage,
            mix_weights,
            max_chunks,
        )
        .await
    }

    pub(super) async fn query_naive(
        &self,
        query_text: &str,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        self.query_naive_with_vector_storage(
            query_text,
            embeddings,
            tenant_id,
            workspace_id,
            None,
            &self.vector_storage,
            max_chunks,
        )
        .await
    }
}

// ── Issue #208 regression tests (default path = vector_queries implementation) ──
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_impl::{QueryEngine, QueryEngineConfig};
    use edgequake_llm::MockProvider;
    use edgequake_storage::traits::VectorStorage;
    use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};
    use std::sync::Arc;

    fn make_engine(vector_storage: Arc<MemoryVectorStorage>) -> QueryEngine {
        let graph_storage = Arc::new(MemoryGraphStorage::new("test"));
        let embedding_provider: Arc<dyn crate::EmbeddingProvider> =
            Arc::new(MockProvider::default());
        let llm_provider: Arc<dyn crate::LLMProvider> = Arc::new(MockProvider::default());
        QueryEngine::new(
            QueryEngineConfig::default(),
            vector_storage as Arc<dyn VectorStorage>,
            graph_storage,
            embedding_provider,
            llm_provider,
        )
    }

    #[tokio::test]
    async fn test_query_naive_returns_chunks_when_entities_dominate() {
        let storage = Arc::new(MemoryVectorStorage::new("test", 4));

        for i in 0..10 {
            let score = 1.0 - (i as f32 * 0.01);
            storage
                .upsert(&[(
                    format!("entity_{i}"),
                    vec![score, 0.0, 0.0, 0.0],
                    serde_json::json!({"type": "entity"}),
                )])
                .await
                .unwrap();
        }
        for i in 0..3 {
            storage
                .upsert(&[(
                    format!("chunk_{i}"),
                    vec![0.85 - (i as f32 * 0.01), 0.01, 0.0, 0.0],
                    serde_json::json!({"type": "chunk", "content": format!("chunk content {i}")}),
                )])
                .await
                .unwrap();
        }

        let engine = make_engine(storage);
        let embeddings = QueryEmbeddings {
            query: vec![1.0, 0.0, 0.0, 0.0],
            high_level: vec![1.0, 0.0, 0.0, 0.0],
            low_level: vec![1.0, 0.0, 0.0, 0.0],
            model: String::new(),
        };

        let ctx = engine
            .query_naive("chunk content", &embeddings, None, None, 20)
            .await
            .expect("query_naive must not error");

        assert!(
            !ctx.chunks.is_empty(),
            "query_naive returned 0 chunks — issue #208 regression"
        );
        assert!(
            ctx.entities.is_empty(),
            "query_naive must not return entities"
        );
    }

    #[tokio::test]
    async fn test_query_naive_respects_tenant_isolation() {
        let storage = Arc::new(MemoryVectorStorage::new("test", 4));

        storage
            .upsert(&[
                (
                    "chunk_t1".to_string(),
                    vec![1.0, 0.0, 0.0, 0.0],
                    serde_json::json!({"type": "chunk", "tenant_id": "t1", "content": "t1 data"}),
                ),
                (
                    "chunk_t2".to_string(),
                    vec![0.99, 0.01, 0.0, 0.0],
                    serde_json::json!({"type": "chunk", "tenant_id": "t2", "content": "t2 data"}),
                ),
            ])
            .await
            .unwrap();

        let engine = make_engine(storage);
        let embeddings = QueryEmbeddings {
            query: vec![1.0, 0.0, 0.0, 0.0],
            high_level: vec![1.0, 0.0, 0.0, 0.0],
            low_level: vec![1.0, 0.0, 0.0, 0.0],
            model: String::new(),
        };

        let ctx = engine
            .query_naive("t1 data", &embeddings, Some("t1".into()), None, 20)
            .await
            .expect("query_naive must not error");

        assert_eq!(ctx.chunks.len(), 1, "must only return t1 chunks");
        assert_eq!(ctx.chunks[0].id, "chunk_t1");
    }

    /// MV-32: numeric/chart queries pre-filter `modality=chart` before vector rank.
    #[tokio::test]
    async fn test_query_naive_prefers_chart_modality_for_numeric_query() {
        std::env::set_var("EDGEQUAKE_CHART_MODALITY_FILTER", "true");
        let storage = Arc::new(MemoryVectorStorage::new("test", 4));

        storage
            .upsert(&[
                (
                    "prose-chunk".to_string(),
                    vec![0.99, 0.01, 0.0, 0.0],
                    serde_json::json!({
                        "type": "chunk",
                        "content": "The company overview discusses revenue trends in general terms."
                    }),
                ),
                (
                    "chart-chunk".to_string(),
                    vec![0.95, 0.05, 0.0, 0.0],
                    serde_json::json!({
                        "type": "chunk",
                        "modality": "chart",
                        "content": "Q4 Revenue: 42 million USD"
                    }),
                ),
            ])
            .await
            .unwrap();

        let engine = make_engine(storage);
        let embeddings = QueryEmbeddings {
            query: vec![1.0, 0.0, 0.0, 0.0],
            high_level: vec![1.0, 0.0, 0.0, 0.0],
            low_level: vec![1.0, 0.0, 0.0, 0.0],
            model: String::new(),
        };

        let ctx = engine
            .query_naive("What was Q4 revenue in USD?", &embeddings, None, None, 5)
            .await
            .expect("query_naive must not error");

        assert!(!ctx.chunks.is_empty());
        assert_eq!(ctx.chunks[0].id, "chart-chunk");
        assert!(ctx.chunks[0].content.contains("42"));
        assert_eq!(ctx.chunks[0].modality.as_deref(), Some("chart"));
        assert_eq!(
            ctx.metadata
                .get("chart_modality_filter")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            ctx.metadata
                .get("retrieved_chart_chunks")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        std::env::remove_var("EDGEQUAKE_CHART_MODALITY_FILTER");
    }

    #[tokio::test]
    async fn test_query_naive_chart_filter_fail_open_without_chart_chunks() {
        std::env::set_var("EDGEQUAKE_CHART_MODALITY_FILTER", "true");
        let storage = Arc::new(MemoryVectorStorage::new("test", 4));
        storage
            .upsert(&[(
                "prose-only".to_string(),
                vec![1.0, 0.0, 0.0, 0.0],
                serde_json::json!({
                    "type": "chunk",
                    "content": "Q4 revenue rose according to the annual report narrative."
                }),
            )])
            .await
            .unwrap();

        let engine = make_engine(storage);
        let embeddings = QueryEmbeddings {
            query: vec![1.0, 0.0, 0.0, 0.0],
            high_level: vec![1.0, 0.0, 0.0, 0.0],
            low_level: vec![1.0, 0.0, 0.0, 0.0],
            model: String::new(),
        };

        let ctx = engine
            .query_naive("What was Q4 revenue?", &embeddings, None, None, 5)
            .await
            .expect("query_naive must not error");

        assert_eq!(ctx.chunks.len(), 1);
        assert_eq!(ctx.chunks[0].id, "prose-only");
        std::env::remove_var("EDGEQUAKE_CHART_MODALITY_FILTER");
    }

    /// Default-path hybrid must use the same implementation as workspace override storage.
    #[tokio::test]
    async fn test_default_and_workspace_hybrid_use_same_retrieval() {
        let storage = Arc::new(MemoryVectorStorage::new("test", 4));
        storage
            .upsert(&[(
                "chunk_a".to_string(),
                vec![0.9, 0.1, 0.0, 0.0],
                serde_json::json!({"type": "chunk", "content": "alpha"}),
            )])
            .await
            .unwrap();

        let engine = make_engine(storage.clone());
        use crate::keywords::{ExtractedKeywords, QueryIntent};
        // Comparative keeps all three arms under default intent gating (OPS-P1).
        let keywords = ExtractedKeywords::new(vec![], vec![], QueryIntent::Comparative);
        let embeddings = QueryEmbeddings {
            query: vec![0.9, 0.1, 0.0, 0.0],
            high_level: vec![0.9, 0.1, 0.0, 0.0],
            low_level: vec![0.9, 0.1, 0.0, 0.0],
            model: String::new(),
        };

        let default_ctx = engine
            .query_hybrid("alpha", &keywords, &embeddings, None, None, 20)
            .await
            .unwrap();
        let workspace_ctx = engine
            .query_hybrid_with_vector_storage(
                "alpha",
                &keywords,
                &embeddings,
                None,
                None,
                None, // no doc scope
                &(storage as Arc<dyn VectorStorage>),
                20,
            )
            .await
            .unwrap();

        assert_eq!(
            default_ctx.chunks.len(),
            workspace_ctx.chunks.len(),
            "hybrid parity: default vs explicit vector storage"
        );
    }
}
