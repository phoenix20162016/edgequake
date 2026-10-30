//! Tenant brand color (`primary_color`) — API contract.
//!
//! The color is tenant-scoped: create, get, list and update must all round-trip the
//! same canonical `#rrggbb` value, `none` must clear it, and malformed input must be
//! rejected before anything is written.
//!
//! Run:
//!   cargo test -p edgequake-api --test contract_tenant_primary_color

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use common::{create_test_app, extract_json};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json_body) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(json_body.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    (status, extract_json(response).await)
}

async fn create_tenant(app: &Router, extra: Value) -> (StatusCode, Value) {
    let slug = format!("brand-{}", Uuid::new_v4());
    let mut body = json!({ "name": format!("Brand {slug}"), "slug": slug, "plan": "pro" });
    body.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    send(app, "POST", "/api/v1/tenants", Some(body)).await
}

#[tokio::test]
async fn primary_color_create_get_list_round_trip() {
    let app = create_test_app();
    let (status, created) = create_tenant(&app, json!({ "primary_color": "#4B5BD6" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["primary_color"], "#4b5bd6");
    let id = created["id"].as_str().unwrap().to_string();

    let (status, fetched) = send(&app, "GET", &format!("/api/v1/tenants/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["primary_color"], "#4b5bd6");

    let (status, listed) = send(&app, "GET", "/api/v1/tenants?limit=100", None).await;
    assert_eq!(status, StatusCode::OK);
    let row = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == id.as_str())
        .expect("created tenant is listed");
    assert_eq!(row["primary_color"], "#4b5bd6");
}

#[tokio::test]
async fn tenant_without_color_serializes_null_default() {
    let app = create_test_app();
    let (status, created) = create_tenant(&app, json!({})).await;
    assert_eq!(status, StatusCode::CREATED);
    // The key is always present so clients can distinguish "default" from "unknown".
    assert!(created.get("primary_color").is_some());
    assert!(created["primary_color"].is_null());
}

#[tokio::test]
async fn update_sets_clears_and_preserves_primary_color() {
    let app = create_test_app();
    let (_, created) = create_tenant(&app, json!({})).await;
    let uri = format!("/api/v1/tenants/{}", created["id"].as_str().unwrap());

    let (status, updated) = send(&app, "PUT", &uri, Some(json!({ "primary_color": "#0F0" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["primary_color"], "#00ff00");

    // Omitting the field must leave the color untouched.
    let (status, renamed) = send(&app, "PUT", &uri, Some(json!({ "name": "Renamed" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["primary_color"], "#00ff00");
    assert_eq!(renamed["name"], "Renamed");

    // `none` clears back to the EdgeQuake default.
    let (status, cleared) = send(&app, "PUT", &uri, Some(json!({ "primary_color": "none" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(cleared["primary_color"].is_null());
}

#[tokio::test]
async fn invalid_primary_color_is_rejected_without_writing() {
    let app = create_test_app();
    let (status, _) = create_tenant(&app, json!({ "primary_color": "not-a-color" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, created) = create_tenant(&app, json!({ "primary_color": "#112233" })).await;
    let uri = format!("/api/v1/tenants/{}", created["id"].as_str().unwrap());
    let (status, body) = send(
        &app,
        "PUT",
        &uri,
        Some(json!({ "primary_color": "#gg0000" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("primary_color"));

    // The stored value is unchanged after the rejected write.
    let (_, fetched) = send(&app, "GET", &uri, None).await;
    assert_eq!(fetched["primary_color"], "#112233");
}
