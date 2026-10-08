//! Fetch views over cached retrieval bundles (SPEC-152).

use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::handlers::context_types::ContentGranularity;
use crate::handlers::context_types::ContextRetrievalResponse;
use crate::services::query_context::{fetch_context_by_id, FetchContextOptions};

use super::budget::{apply_budget, apply_chunk_cursor, BudgetClass};
use super::cursor::{Cursor, CursorObject};
use super::entity_ref::agent_id_for_node;
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use super::ids::{is_artifact_type, map_entity_type, title_case_label, truncate_chars};
use super::scores::scores_from_hits;

pub async fn eq_fetch(args: &Value, workspace: &str) -> ApiResult<Value> {
    let retrieval_id = args
        .get("retrieval_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("retrieval_id required".into()))?;
    if !retrieval_id.starts_with("ret_") {
        return Ok(eq_error(ErrorCode::InvalidId, "Invalid retrieval_id", None));
    }

    let budget = BudgetClass::parse(args.get("budget").and_then(|v| v.as_str()));
    let view = args.get("view").and_then(|v| v.as_str()).unwrap_or("toc");
    let include_subgraph = args
        .get("include_subgraph")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let include_artifacts = args
        .get("include_artifacts")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let include_weak_edges = args
        .get("include_weak_edges")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let ids: Option<Vec<String>> = args.get("ids").and_then(|v| v.as_array()).map(|a| {
        a.iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    });
    let cursor = args.get("cursor").and_then(|v| v.as_str());
    if let Some(raw) = cursor.filter(|s| !s.is_empty()) {
        let expected = match view {
            "chunks" => Some(CursorObject::Chunks),
            "entities" => Some(CursorObject::Entities),
            _ => None,
        };
        if let Some(exp) = expected {
            if let Err(err) = Cursor::require_object(Some(raw), exp) {
                return Ok(err);
            }
        } else if Cursor::parse(raw).is_err() {
            return Ok(eq_error(
                ErrorCode::InvalidId,
                format!("invalid cursor: {raw}"),
                None,
            ));
        }
    }

    let resp = match fetch_context_by_id(
        retrieval_id,
        FetchContextOptions {
            granularity: ContentGranularity::Agent,
            include_subgraph: include_subgraph || matches!(view, "entities" | "full" | "toc"),
        },
    ) {
        Ok(r) => r,
        Err(ApiError::NotFound(_)) | Err(ApiError::Gone(_)) => {
            return Ok(eq_error(
                ErrorCode::NotFound,
                "Retrieval expired or missing — re-run eq_search",
                None,
            ));
        }
        Err(e) => return Err(e),
    };

    Ok(project_fetch(
        &resp,
        workspace,
        budget,
        view,
        ids.as_deref(),
        include_artifacts,
        include_weak_edges,
        include_subgraph,
        cursor,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn project_fetch(
    resp: &ContextRetrievalResponse,
    workspace: &str,
    budget: BudgetClass,
    view: &str,
    ids: Option<&[String]>,
    include_artifacts: bool,
    include_weak_edges: bool,
    include_subgraph: bool,
    cursor: Option<&str>,
) -> Value {
    let mut documents = Vec::new();
    let mut chunks = Vec::new();
    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let mut citations = Vec::new();

    for d in &resp.bundle.documents {
        documents.push(json!({
            "id": d.document_id,
            "title": d.title,
            "chunk_count": d.chunk_count_in_bundle,
            "entity_count": d.entity_count_in_bundle,
        }));
    }

    for c in &resp.bundle.chunks {
        if let Some(filter) = ids {
            if !filter.is_empty() && !filter.iter().any(|id| id == &c.id) {
                continue;
            }
        }
        let doc_id = c
            .lineage
            .as_ref()
            .and_then(|l| l.document_id.clone())
            .unwrap_or_default();
        let file_name = c
            .lineage
            .as_ref()
            .and_then(|l| l.file_path.clone())
            .unwrap_or_default();
        chunks.push(json!({
            "id": c.id,
            "document_id": doc_id,
            "file_name": file_name,
            "start_line": c.lineage.as_ref().and_then(|l| l.start_line),
            "end_line": c.lineage.as_ref().and_then(|l| l.end_line),
            "score": c.score,
            "text": c.content,
        }));
        citations.push(json!({
            "quote": truncate_chars(&c.content, 280),
            "document_id": doc_id,
            "chunk_id": c.id,
            "score": c.score,
        }));
    }

    let want_entities = matches!(view, "toc" | "entities" | "full") || include_subgraph;
    if want_entities {
        for e in &resp.bundle.subgraph.entities {
            if !include_artifacts && is_artifact_type(&e.entity_type) {
                continue;
            }
            let storage_id = if !e.graph_node_id.is_empty() {
                e.graph_node_id.as_str()
            } else {
                e.name.as_str()
            };
            let id = agent_id_for_node(workspace, storage_id);
            if let Some(filter) = ids {
                if !filter.is_empty()
                    && !filter
                        .iter()
                        .any(|fid| fid == &id || fid == &e.id || fid == &e.name)
                {
                    continue;
                }
            }
            let mut doc_ids = e.source_document_ids.clone();
            if doc_ids.is_empty() {
                if let Some(ref lin) = e.lineage {
                    doc_ids = lin.source_document_ids.clone();
                    if doc_ids.is_empty() {
                        if let Some(ref singular) = lin.source_document_id {
                            doc_ids.push(singular.clone());
                        }
                    }
                }
            }
            entities.push(json!({
                "id": id,
                "name": title_case_label(edgequake_storage::EntityId::bare_name_from_graph_node_id(storage_id)),
                "slug": super::ids::resolve_entity_lookup(storage_id),
                "type": map_entity_type(&e.entity_type),
                "one_liner": truncate_chars(&e.description, budget.max_one_liner()),
                "degree": e.degree,
                "score": e.score,
                "document_ids": doc_ids,
            }));
        }

        for r in &resp.bundle.subgraph.relationships {
            if !include_weak_edges && r.relation_type.eq_ignore_ascii_case("RELATED_TO") {
                continue;
            }
            let src = if r.source_label.is_empty() {
                &r.source
            } else {
                &r.source_label
            };
            let tgt = if r.target_label.is_empty() {
                &r.target
            } else {
                &r.target_label
            };
            relationships.push(json!({
                "id": r.id,
                "type": r.relation_type,
                "source": agent_id_for_node(workspace, src),
                "target": agent_id_for_node(workspace, tgt),
                "score": r.score,
            }));
        }
    }

    // View shaping
    match view {
        "toc" => {
            entities.truncate(8);
            // Keep only 3 chunk snippets
            for c in chunks.iter_mut().take(3) {
                if let Some(Value::String(t)) = c.get_mut("text") {
                    *t = truncate_chars(t, 240);
                }
            }
            chunks.truncate(3);
            citations.clear();
            if !include_subgraph {
                relationships.clear();
            }
        }
        "chunks" => {
            entities.clear();
            relationships.clear();
            citations.clear();
        }
        "entities" => {
            chunks.clear();
            citations.clear();
        }
        "citations" => {
            entities.clear();
            relationships.clear();
            chunks.clear();
        }
        "full" => {
            citations.clear();
        }
        _ => {}
    }

    if matches!(view, "chunks" | "full" | "toc") {
        chunks = apply_chunk_cursor(&mut chunks, cursor);
    }

    let hit_scores: Vec<Value> = chunks.iter().chain(entities.iter()).cloned().collect();
    let score_type = scores_from_hits(&hit_scores);

    let env = EnvelopeBuilder::new(view, budget)
        .insert("retrieval_id", json!(resp.retrieval_id))
        .insert("documents", json!(documents))
        .insert("chunks", json!(chunks))
        .insert("entities", json!(entities))
        .insert("relationships", json!(relationships))
        .insert("citations", json!(citations))
        .insert("score_type", json!(score_type.as_str()))
        .insert(
            "stats",
            json!({
                "cached": true,
                "fingerprint": resp.retrieval_fingerprint,
            }),
        )
        .build();

    let (capped, over) = apply_budget(env, budget);
    if over {
        return eq_error(
            ErrorCode::TruncateInvalid,
            "structuredContent exceeded budget after truncation",
            None,
        );
    }
    capped
}
