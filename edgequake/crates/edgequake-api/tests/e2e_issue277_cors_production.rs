//! GitHub #277 E2E — CORS + WebSocket in production auth mode (`EDGEQUAKE_DEV_MODE=false`).
//!
//! ```bash
//! cargo test -p edgequake-api --test e2e_issue277_cors_production
//! ```

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use edgequake_api::{AppState, Server, ServerConfig};

const ALLOWED_ORIGIN: &str = "https://app.example.com";
const API_KEY: &str = "issue-277-test-key";

fn production_cors_state() -> AppState {
    let mut state = AppState::test_state();
    state.auth.config.auth_enabled = true;
    state.auth.config.dev_mode = false;
    state.auth.config.api_keys = vec![API_KEY.to_string()];
    state.security.cors_origins = Some(vec![ALLOWED_ORIGIN.to_string()]);
    // SPEC-083 S-10: prod fail-closed CORS / WS Origin.
    state.security.cors_fail_closed = true;
    state
}

fn build_production_cors_app(state: AppState) -> axum::Router {
    Server::new(
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            enable_cors: true,
            enable_compression: false,
            enable_swagger: true,
        },
        state,
    )
    .build_router()
}

fn cors_header<'a>(response: &'a axum::response::Response, name: &str) -> Option<&'a str> {
    response.headers().get(name).and_then(|v| v.to_str().ok())
}

#[tokio::test]
async fn issue_277_options_preflight_returns_allowed_origin() {
    let app = build_production_cors_app(production_cors_state());

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/documents")
                .header("Origin", ALLOWED_ORIGIN)
                .header("Access-Control-Request-Method", "POST")
                .header(
                    "Access-Control-Request-Headers",
                    "authorization,content-type",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "preflight must not be blocked by auth middleware"
    );
    assert_eq!(
        cors_header(&response, "access-control-allow-origin"),
        Some(ALLOWED_ORIGIN)
    );
}

/// Headers a browser client of this API puts on a non-simple request.
const BROWSER_REQUEST_HEADERS: &[&str] = &[
    "authorization",
    "content-type",
    "if-match",
    "prefer",
    "range",
    "traceparent",
    "tracestate",
    "x-api-key",
    "x-correlation-id",
    "x-edgequake-app-id",
    "x-edgequake-app-name",
    "x-edgequake-app-url",
    "x-edgequake-client",
    "x-edgequake-confirm",
    "x-edgequake-request-id",
    "x-edgequake-setup-token",
    "x-edgequake-tenant-id",
    "x-filename",
    "x-request-id",
    "x-tenant-id",
    "x-user-id",
    "x-workspace-id",
];

/// Response headers the WebUI reads from a cross-origin response.
const BROWSER_EXPOSED_HEADERS: &[&str] = &[
    "accept-ranges",
    "content-encoding",
    "content-length",
    "content-range",
    "traceparent",
    "tracestate",
    "x-request-id",
    "x-retrieval-fingerprint",
    "x-retrieval-id",
];

/// WebUI and browser API clients send these. Fail-closed CORS must allow them
/// or the call fails when `EDGEQUAKE_DEV_MODE=false`.
#[tokio::test]
async fn issue_277_preflight_allows_webui_headers_outside_dev_mode() {
    let app = build_production_cors_app(production_cors_state());
    let requested = BROWSER_REQUEST_HEADERS.join(",");

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/documents")
                .header("Origin", ALLOWED_ORIGIN)
                .header("Access-Control-Request-Method", "POST")
                .header("Access-Control-Request-Headers", requested)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        cors_header(&response, "access-control-allow-origin"),
        Some(ALLOWED_ORIGIN),
        "preflight must succeed for the WebUI header set"
    );
    let allowed = cors_header(&response, "access-control-allow-headers")
        .unwrap_or("")
        .to_ascii_lowercase();
    for name in BROWSER_REQUEST_HEADERS {
        assert!(
            allowed.split(',').any(|h| h.trim() == *name),
            "fail-closed CORS dropped {name}; allow-headers={allowed}"
        );
    }
}

#[tokio::test]
async fn issue_277_exposes_headers_the_webui_reads() {
    let app = build_production_cors_app(production_cors_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header("Origin", ALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        cors_header(&response, "access-control-allow-origin"),
        Some(ALLOWED_ORIGIN)
    );
    let exposed = cors_header(&response, "access-control-expose-headers")
        .unwrap_or("")
        .to_ascii_lowercase();
    for name in BROWSER_EXPOSED_HEADERS {
        assert!(
            exposed.split(',').any(|h| h.trim() == *name),
            "cross-origin script cannot read {name}; expose-headers={exposed}"
        );
    }
    assert!(
        cors_header(&response, "x-request-id").is_some(),
        "health response must carry x-request-id for the WebUI error path"
    );
    assert!(
        cors_header(&response, "traceparent").is_some(),
        "health response must carry traceparent so the WebUI can adopt it"
    );
}

#[tokio::test]
async fn issue_277_openapi_json_includes_cors_for_allowed_origin() {
    let app = build_production_cors_app(production_cors_state());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api-docs/openapi.json")
                .header("Origin", ALLOWED_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        cors_header(&response, "access-control-allow-origin"),
        Some(ALLOWED_ORIGIN),
        "Swagger merge must be inside CORS layer (GitHub #277)"
    );
}

#[tokio::test]
async fn issue_277_websocket_auth_accepts_api_key_token_value() {
    let state = production_cors_state();
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("origin", ALLOWED_ORIGIN.parse().unwrap());

    assert!(edgequake_api::middleware::ws_validate_origin(&state, &headers).is_ok());
    assert!(
        edgequake_api::middleware::ws_validate_token(&state, Some(API_KEY))
            .await
            .is_some(),
        "production mode must accept API key via Authorization / X-API-Key (SPEC-154)"
    );
}

#[tokio::test]
async fn issue_277_websocket_auth_accepts_bearer_token_value() {
    let state = production_cors_state();
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("origin", ALLOWED_ORIGIN.parse().unwrap());

    assert!(edgequake_api::middleware::ws_validate_origin(&state, &headers).is_ok());
    assert!(
        edgequake_api::middleware::ws_validate_token(&state, Some(API_KEY))
            .await
            .is_some(),
        "same token validated via Authorization bearer header"
    );
}

#[tokio::test]
async fn issue_277_websocket_auth_rejects_missing_token() {
    let state = production_cors_state();
    assert!(
        edgequake_api::middleware::ws_validate_token(&state, None)
            .await
            .is_none(),
        "missing token must fail when auth enabled and dev_mode=false"
    );
}

#[tokio::test]
async fn issue_277_websocket_origin_rejects_disallowed() {
    let state = production_cors_state();
    let mut denied = axum::http::HeaderMap::new();
    denied.insert("origin", "https://evil.example.com".parse().unwrap());
    assert_eq!(
        edgequake_api::middleware::ws_validate_origin(&state, &denied),
        Err(StatusCode::FORBIDDEN)
    );
}

#[tokio::test]
async fn issue_277_websocket_origin_rejects_missing_in_prod() {
    let state = production_cors_state();
    let headers = axum::http::HeaderMap::new();
    assert_eq!(
        edgequake_api::middleware::ws_validate_origin(&state, &headers),
        Err(StatusCode::FORBIDDEN),
        "SPEC-083 S-10: missing Origin must fail closed in production"
    );
}

/// SPEC-083 matrix name (S-10) — alias of `issue_277_websocket_origin_rejects_missing_in_prod`.
#[tokio::test]
async fn e2e_ws_missing_origin_rejected_prod() {
    let state = production_cors_state();
    let headers = axum::http::HeaderMap::new();
    assert_eq!(
        edgequake_api::middleware::ws_validate_origin(&state, &headers),
        Err(StatusCode::FORBIDDEN),
        "SPEC-083 S-10: missing Origin must fail closed in production"
    );
}

#[test]
fn issue_277_ws_validate_origin_unit() {
    let state = production_cors_state();
    let mut allowed = axum::http::HeaderMap::new();
    allowed.insert("origin", ALLOWED_ORIGIN.parse().unwrap());
    assert!(edgequake_api::middleware::ws_validate_origin(&state, &allowed).is_ok());

    let mut denied = axum::http::HeaderMap::new();
    denied.insert("origin", "https://evil.example.com".parse().unwrap());
    assert_eq!(
        edgequake_api::middleware::ws_validate_origin(&state, &denied),
        Err(StatusCode::FORBIDDEN)
    );
}
