//! Contract: typed ANN / registry production paths must not read
//! `EDGEQUAKE_EMBEDDING_MODEL` directly — use [`embedding_model_key_from_env`].
//!
//! Provider construction (which embedder to load) may still read the env; ANN
//! registry keys must go through the SSOT so empty Compose `=` cannot diverge.

use std::path::PathBuf;

fn crate_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn walk_rs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Paths allowed to call `env::var("EDGEQUAKE_EMBEDDING_MODEL")` / `EMBEDDING_MODEL_ENV`.
fn is_allowlisted(path: &std::path::Path) -> bool {
    let s = path.to_string_lossy().replace('\\', "/");
    s.ends_with("/vector_backend.rs")
        || s.contains("/tests/")
        || s.contains("/bin/")
}

#[test]
fn contract_typed_ann_registry_env_reads_use_ssot() {
    let mut files = Vec::new();
    walk_rs(&crate_src(), &mut files);

    let needles = [
        "env::var(\"EDGEQUAKE_EMBEDDING_MODEL\")",
        "env::var('EDGEQUAKE_EMBEDDING_MODEL')",
        "std::env::var(\"EDGEQUAKE_EMBEDDING_MODEL\")",
        "std::env::var(EMBEDDING_MODEL_ENV)",
        "env::var(EMBEDDING_MODEL_ENV)",
    ];

    let mut offenders = Vec::new();
    for path in &files {
        if is_allowlisted(path) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        for needle in needles {
            if src.contains(needle) {
                offenders.push(format!("{} contains `{needle}`", path.display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "typed ANN / registry production code must use embedding_model_key_from_env(); offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn contract_query_filtered_uses_serving_candidates() {
    let storage_impl = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    assert!(
        storage_impl.contains("serving_embedding_model_candidates"),
        "query_filtered typed ANN must prefer workspace model via serving_embedding_model_candidates"
    );
    assert!(
        !storage_impl
            .split("async fn query_filtered")
            .nth(1)
            .unwrap_or("")
            .split("async fn ")
            .next()
            .unwrap_or("")
            .contains("env::var(\"EDGEQUAKE_EMBEDDING_MODEL\")"),
        "query_filtered body must not read EDGEQUAKE_EMBEDDING_MODEL directly"
    );
}
