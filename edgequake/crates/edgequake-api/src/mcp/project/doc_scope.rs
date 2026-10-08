//! Hard document scope for MCP search / entity_search (SPEC-162 R6–R8).

use serde_json::Value;

use crate::error::ApiResult;
use crate::middleware::TenantContext;
use crate::state::AppState;

#[derive(Debug, Clone, Default)]
pub struct DocumentScope {
    pub allowed_ids: Vec<String>,
    pub unknown_ids: Vec<String>,
    pub pattern_matched: bool,
    pub filter_result: Option<&'static str>,
    pub message: Option<String>,
}

impl DocumentScope {
    /// Validate explicit ids exist in the workspace (no pattern). Used by entity_search.
    pub async fn try_resolve_ids_only(
        state: &AppState,
        tenant_ctx: &TenantContext,
        ids: &[String],
    ) -> ApiResult<Option<Self>> {
        if ids.is_empty() {
            return Ok(None);
        }
        let mut unknown = Vec::new();
        let mut allowed = Vec::new();
        for id in ids {
            match crate::services::load_staging_first_metadata(
                state.storage.kv_storage.as_ref(),
                id,
            )
            .await
            {
                Ok(Some((_, meta)))
                    if crate::workspace_scope::metadata_matches_tenant_context(
                        &meta, tenant_ctx,
                    ) =>
                {
                    allowed.push(id.clone());
                }
                _ => unknown.push(id.clone()),
            }
        }
        Ok(Some(Self {
            allowed_ids: allowed,
            unknown_ids: unknown,
            pattern_matched: true,
            filter_result: None,
            message: None,
        }))
    }

    /// Full MCP hard scope: ids ∩ pattern (file_name + title).
    pub async fn resolve(
        state: &AppState,
        tenant_ctx: &TenantContext,
        args: &Value,
    ) -> ApiResult<Option<Self>> {
        let ids: Vec<String> = args
            .get("document_ids")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let nested = args.get("document_filter");
        let pattern = args
            .get("document_pattern")
            .or_else(|| nested.and_then(|f| f.get("document_pattern")))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let nested_ids: Vec<String> = nested
            .and_then(|f| f.get("document_ids"))
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let mut explicit = if ids.is_empty() { nested_ids } else { ids };
        explicit.sort();
        explicit.dedup();

        if explicit.is_empty() && pattern.as_ref().is_none_or(|p| p.trim().is_empty()) {
            return Ok(None);
        }

        let mut unknown = Vec::new();
        let mut id_allowed = Vec::new();
        for id in &explicit {
            match crate::services::load_staging_first_metadata(
                state.storage.kv_storage.as_ref(),
                id,
            )
            .await
            {
                Ok(Some((_, meta)))
                    if crate::workspace_scope::metadata_matches_tenant_context(
                        &meta, tenant_ctx,
                    ) =>
                {
                    id_allowed.push(id.clone());
                }
                _ => unknown.push(id.clone()),
            }
        }

        if !unknown.is_empty() && !explicit.is_empty() {
            return Ok(Some(Self {
                allowed_ids: Vec::new(),
                unknown_ids: unknown,
                pattern_matched: false,
                filter_result: None,
                message: None,
            }));
        }

        let patterns = parse_patterns(pattern.as_deref());
        let mut pattern_ids = Vec::new();
        let mut pattern_matched = patterns.is_empty();

        if !patterns.is_empty() {
            let pool = state.optional_pg_pool();
            let entries =
                crate::services::document_metadata_scan::load_scoped_document_metadata_entries(
                    state.storage.kv_storage.as_ref(),
                    pool,
                    tenant_ctx,
                )
                .await?;
            for (doc_id, meta) in entries {
                let title = meta
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let file_name = meta
                    .get("file_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if patterns
                    .iter()
                    .any(|p| title.contains(p.as_str()) || file_name.contains(p.as_str()))
                {
                    pattern_ids.push(doc_id);
                }
            }
            pattern_matched = !pattern_ids.is_empty();
            if !pattern_matched {
                return Ok(Some(Self {
                    allowed_ids: Vec::new(),
                    unknown_ids: Vec::new(),
                    pattern_matched: false,
                    filter_result: Some("no_match"),
                    message: Some("No document matches the pattern.".into()),
                }));
            }
        }

        // Intersection when both present; else whichever is set.
        let allowed = if !explicit.is_empty() && !patterns.is_empty() {
            id_allowed
                .into_iter()
                .filter(|id| pattern_ids.iter().any(|p| p == id))
                .collect()
        } else if !explicit.is_empty() {
            id_allowed
        } else {
            pattern_ids
        };

        let filter_result = if allowed.is_empty() {
            Some("empty")
        } else {
            None
        };

        Ok(Some(Self {
            allowed_ids: allowed,
            unknown_ids: Vec::new(),
            pattern_matched,
            filter_result,
            message: None,
        }))
    }
}

fn parse_patterns(pattern: Option<&str>) -> Vec<String> {
    pattern
        .map(|p| {
            p.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}
