//! Shared byte caps for MCP blob and image responses (SPEC-161 / SPEC-162).

/// Hard cap per download call (SPEC-162 R10). Env may only lower this value.
pub const DOWNLOAD_HARD_CAP: usize = 65_536;

pub fn blob_max_bytes() -> usize {
    std::env::var("EDGEQUAKE_MCP_BLOB_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &usize| *n > 0)
        .map(|n| n.min(DOWNLOAD_HARD_CAP))
        .unwrap_or(DOWNLOAD_HARD_CAP)
}

pub fn upload_ttl_secs() -> u64 {
    std::env::var("EDGEQUAKE_MCP_UPLOAD_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &u64| *n > 0)
        .unwrap_or(3600)
}

pub const EXTRA_CONTENT_KEY: &str = "_mcp_extra_content";
