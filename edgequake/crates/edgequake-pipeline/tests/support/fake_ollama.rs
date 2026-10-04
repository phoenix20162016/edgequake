//! A fake Ollama server for decision-mode tests (SPEC-160).
//!
//! It serves the three routes the backend uses and records what it saw.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use serde_json::Value;

/// What a route answers.
pub type Reply = (StatusCode, String);

/// A System One route body handler.
pub type Handler = Arc<dyn Fn(&Value) -> Reply + Send + Sync>;

/// Route behavior. Tests replace the closures they care about.
#[derive(Clone)]
pub struct FakeOllama {
    pub version: Arc<Mutex<Reply>>,
    pub show: Arc<Mutex<Reply>>,
    pub tags: Arc<Mutex<Reply>>,
    pub systemone: Handler,
    pub seen_auth: Arc<Mutex<Vec<Option<String>>>>,
    pub systemone_calls: Arc<Mutex<Vec<Value>>>,
}

impl FakeOllama {
    pub fn healthy(handler: Handler) -> Self {
        Self {
            version: Arc::new(Mutex::new((
                StatusCode::OK,
                r#"{"version":"0.35.1"}"#.into(),
            ))),
            show: Arc::new(Mutex::new((
                StatusCode::OK,
                r#"{"capabilities":["decision"]}"#.into(),
            ))),
            tags: Arc::new(Mutex::new((
                StatusCode::OK,
                r#"{"models":[{"name":"tev1:0.8b"},{"name":"gemma4:latest"}]}"#.into(),
            ))),
            systemone: handler,
            seen_auth: Arc::default(),
            systemone_calls: Arc::default(),
        }
    }

    /// Start on a free port. Returns the base URL and keeps serving until the test ends.
    pub async fn serve(self) -> String {
        let app = Router::new()
            .route("/api/version", get(version))
            .route("/api/show", post(show))
            .route("/api/tags", get(tags))
            .route("/v1/systemone", post(systemone))
            .with_state(self);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr: SocketAddr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }
}

fn auth(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

async fn version(State(s): State<FakeOllama>, headers: HeaderMap) -> Reply {
    s.seen_auth.lock().unwrap().push(auth(&headers));
    s.version.lock().unwrap().clone()
}

async fn show(State(s): State<FakeOllama>) -> Reply {
    s.show.lock().unwrap().clone()
}

async fn tags(State(s): State<FakeOllama>) -> Reply {
    s.tags.lock().unwrap().clone()
}

async fn systemone(State(s): State<FakeOllama>, headers: HeaderMap, body: Bytes) -> Reply {
    s.seen_auth.lock().unwrap().push(auth(&headers));
    let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    s.systemone_calls.lock().unwrap().push(parsed.clone());
    (s.systemone)(&parsed)
}

/// Wrap a rule handler as a route that answers with JSON.
pub fn json_reply(
    handler: edgextract::testing::HandlerFn,
) -> Arc<dyn Fn(&Value) -> Reply + Send + Sync> {
    Arc::new(move |body| match handler(body.clone()) {
        Ok(v) => (StatusCode::OK, v.to_string()),
        Err(e) => (StatusCode::BAD_REQUEST, e),
    })
}
