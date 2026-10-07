//! Shared byte caps for MCP blob and image responses (SPEC-161).

pub fn blob_max_bytes() -> usize {
    std::env::var("EDGEQUAKE_MCP_BLOB_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &usize| *n > 0)
        .unwrap_or(4 * 1024 * 1024)
}

pub fn upload_ttl_secs() -> u64 {
    std::env::var("EDGEQUAKE_MCP_UPLOAD_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &u64| *n > 0)
        .unwrap_or(3600)
}

pub const EXTRA_CONTENT_KEY: &str = "_mcp_extra_content";
