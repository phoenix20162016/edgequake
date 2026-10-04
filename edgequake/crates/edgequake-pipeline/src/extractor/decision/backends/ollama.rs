//! Ollama backend: `POST /v1/systemone` (Ollama 0.35 or later, Tev1 models).
//!
//! Health uses `GET /api/version` and `POST /api/show`. The error map below was
//! measured against Ollama 0.35.1:
//!
//! | Situation                   | Status | Body marker                       | Error          |
//! |-----------------------------|--------|-----------------------------------|----------------|
//! | Model not pulled            | 404    | `not found, try pulling it first` | `ModelMissing` |
//! | Route missing (old Ollama)  | 404    | other                             | `Unsupported`  |
//! | Model has no decision skill | 400    | `does not support decision`       | `Unsupported`  |
//! | Prompt over the token cap   | 400    | `tokens; expected`                | `Contract`     |

use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde_json::{json, Value};

use crate::extractor::decision::backend::{
    redact_host, BackendDescriptor, BackendHealth, BackendKind, DecisionBackend, ListedModel,
};
use crate::extractor::decision::error::DecisionError;

const SYSTEMONE_PATH: &str = "/v1/systemone";
const HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const SNIPPET_CHARS: usize = 400;
/// First Ollama release that serves `/v1/systemone`.
const MIN_VERSION: (u32, u32) = (0, 35);

/// Ollama System One backend.
pub struct OllamaBackend {
    client: Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
    /// Sent as `keep_alive` so the model stays loaded between requests (EC-160-19).
    keep_alive: Option<String>,
}

impl OllamaBackend {
    pub fn new(
        base_url: &str,
        model: &str,
        api_key: Option<String>,
        timeout_secs: u64,
    ) -> Result<Self, DecisionError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs.max(1)))
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(|e| DecisionError::Config(format!("cannot build the HTTP client: {e}")))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            api_key,
            keep_alive: None,
        })
    }

    /// Ask Ollama to keep the model loaded for this long after each request.
    pub fn with_keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Add `keep_alive` unless the caller set it.
    fn with_keep_alive_field(&self, request: &Value) -> Value {
        let mut body = request.clone();
        if let (Some(keep), Some(obj)) = (&self.keep_alive, body.as_object_mut()) {
            obj.entry("keep_alive").or_insert_with(|| json!(keep));
        }
        body
    }

    fn post(&self, path: &str) -> RequestBuilder {
        self.authorize(self.client.post(format!("{}{path}", self.base_url)))
    }

    fn authorize(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.api_key {
            Some(key) => req.bearer_auth(key),
            None => req,
        }
    }

    async fn probe_version(&self) -> Result<(String, u64), String> {
        let started = Instant::now();
        let url = format!("{}/api/version", self.base_url);
        let resp = self
            .authorize(self.client.get(url))
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await
            .map_err(transport_text)?;
        let latency = started.elapsed().as_millis() as u64;
        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("bad version body: {e}"))?;
        let version = body
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        Ok((version, latency))
    }

    /// `(model_present, decision_capable)` from `/api/show`.
    async fn probe_model(&self) -> Result<(bool, bool), String> {
        let resp = self
            .post("/api/show")
            .timeout(HEALTH_TIMEOUT)
            .json(&json!({ "model": self.model }))
            .send()
            .await
            .map_err(transport_text)?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok((false, false));
        }
        if !resp.status().is_success() {
            return Err(format!(
                "/api/show answered HTTP {}",
                resp.status().as_u16()
            ));
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("bad show body: {e}"))?;
        Ok((true, has_decision_capability(&body)))
    }

    /// Names from `GET /api/tags`, then a bounded `/api/show` for the decision skill.
    async fn list_models_inner(&self) -> Result<Vec<ListedModel>, DecisionError> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .authorize(self.client.get(url))
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await
            .map_err(map_send_error)?;
        if !resp.status().is_success() {
            return Err(DecisionError::Unavailable(format!(
                "/api/tags answered HTTP {}",
                resp.status().as_u16()
            )));
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DecisionError::Unavailable(format!("bad tags body: {e}")))?;
        let names: Vec<String> = body
            .get("models")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|m| {
                m.get("name")
                    .or_else(|| m.get("model"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .take(64)
            .collect();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            let capable = self.probe_named_model(&name).await.ok();
            out.push(ListedModel {
                name,
                decision_capable: capable,
            });
        }
        Ok(out)
    }

    async fn probe_named_model(&self, name: &str) -> Result<bool, String> {
        let resp = self
            .post("/api/show")
            .timeout(HEALTH_TIMEOUT)
            .json(&json!({ "model": name }))
            .send()
            .await
            .map_err(transport_text)?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(false);
        }
        if !resp.status().is_success() {
            return Err(format!(
                "/api/show answered HTTP {}",
                resp.status().as_u16()
            ));
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("bad show body: {e}"))?;
        Ok(has_decision_capability(&body))
    }
}

fn transport_text(err: reqwest::Error) -> String {
    if err.is_timeout() {
        "request timed out".into()
    } else if err.is_connect() {
        "connection refused or host unreachable".into()
    } else {
        // `without_url` keeps credentials and paths out of logs.
        err.without_url().to_string()
    }
}

/// True when `/api/show` lists the `decision` capability.
pub(crate) fn has_decision_capability(show: &Value) -> bool {
    show.get("capabilities")
        .and_then(Value::as_array)
        .is_some_and(|caps| caps.iter().any(|c| c.as_str() == Some("decision")))
}

/// `"0.35.1"` or `"0.35.1-rc2"` to `(0, 35)`. Unknown text is `None`.
pub(crate) fn parse_major_minor(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.trim().trim_start_matches('v').split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.split(['-', '+']).next()?.parse().ok()?;
    Some((major, minor))
}

fn snippet(body: &str) -> String {
    body.chars().take(SNIPPET_CHARS).collect()
}

/// Map a non-success System One answer to a typed error.
pub(crate) fn map_error_response(status: StatusCode, body: &str, model: &str) -> DecisionError {
    let lower = body.to_lowercase();
    let code = status.as_u16();
    if status == StatusCode::NOT_FOUND {
        return if lower.contains("not found, try pulling") {
            DecisionError::ModelMissing(model.to_string())
        } else {
            DecisionError::Unsupported(format!(
                "the server has no {SYSTEMONE_PATH} route. Use Ollama {}.{} or later.",
                MIN_VERSION.0, MIN_VERSION.1
            ))
        };
    }
    if lower.contains("does not support decision") {
        return DecisionError::Unsupported(format!("model '{model}' does not support decision"));
    }
    match code {
        401 | 403 => {
            DecisionError::Unavailable(format!("the server rejected the credentials (HTTP {code})"))
        }
        408 | 504 => DecisionError::Timeout(format!("http {code}")),
        500..=599 => DecisionError::Unavailable(format!("http {code}: {}", snippet(body))),
        _ => DecisionError::Contract(format!("http {code}: {}", snippet(body))),
    }
}

fn map_send_error(err: reqwest::Error) -> DecisionError {
    if err.is_timeout() {
        DecisionError::Timeout("no answer within the configured time".into())
    } else {
        DecisionError::Unavailable(transport_text(err))
    }
}

#[async_trait]
impl DecisionBackend for OllamaBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            kind: BackendKind::OllamaSystemOne,
            model: self.model.clone(),
            host: redact_host(&self.base_url),
        }
    }

    async fn health(&self) -> BackendHealth {
        let (version, latency) = match self.probe_version().await {
            Ok(v) => v,
            Err(e) => return BackendHealth::unreachable(e),
        };
        let mut health = BackendHealth::unreachable("");
        health.reachable = true;
        health.latency_ms = Some(latency);
        health.server_version = Some(version.clone());
        health.detail = None;
        if parse_major_minor(&version).is_some_and(|v| v < MIN_VERSION) {
            health.supported = false;
            health.detail = Some(format!(
                "Ollama {version} is too old. Use {}.{} or later.",
                MIN_VERSION.0, MIN_VERSION.1
            ));
            return health;
        }
        match self.probe_model().await {
            Ok((present, capable)) => {
                health.model_present = present;
                health.decision_capable = capable;
            }
            Err(e) => health.detail = Some(e),
        }
        health
    }

    async fn systemone(&self, request: &Value) -> Result<Value, DecisionError> {
        let body = self.with_keep_alive_field(request);
        let resp: Response = self
            .post(SYSTEMONE_PATH)
            .json(&body)
            .send()
            .await
            .map_err(map_send_error)?;
        let status = resp.status();
        let text = resp.text().await.map_err(map_send_error)?;
        if !status.is_success() {
            return Err(map_error_response(status, &text, &self.model));
        }
        let body: Value = serde_json::from_str(&text)
            .map_err(|e| DecisionError::Contract(format!("the answer is not JSON: {e}")))?;
        if body.is_object() {
            Ok(body)
        } else {
            Err(DecisionError::Contract(
                "the answer is not a JSON object".into(),
            ))
        }
    }

    async fn list_models(&self) -> Result<Vec<ListedModel>, DecisionError> {
        self.list_models_inner().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-160-U62 — keep_alive is added once and never overrides the caller.
    #[test]
    fn keep_alive_injection() {
        let plain = OllamaBackend::new("http://h:1", "m", None, 5).unwrap();
        assert!(plain
            .with_keep_alive_field(&json!({"a":1}))
            .get("keep_alive")
            .is_none());
        let kept = plain.with_keep_alive("30m");
        assert_eq!(
            kept.with_keep_alive_field(&json!({"a":1}))["keep_alive"],
            "30m"
        );
        assert_eq!(
            kept.with_keep_alive_field(&json!({"keep_alive":"1h"}))["keep_alive"],
            "1h"
        );
    }

    // T-160-U48 — version words parse; junk does not.
    #[test]
    fn version_parse() {
        assert_eq!(parse_major_minor("0.35.1"), Some((0, 35)));
        assert_eq!(parse_major_minor("v0.36.0-rc2"), Some((0, 36)));
        assert_eq!(parse_major_minor("1.0"), Some((1, 0)));
        assert_eq!(parse_major_minor("dev"), None);
        assert!((0, 34) < MIN_VERSION && (0, 35) >= MIN_VERSION);
    }

    // T-160-U49 — the measured Ollama errors map to the right types.
    #[test]
    fn error_map() {
        let nf = map_error_response(
            StatusCode::NOT_FOUND,
            r#"{"error":"model \"x\" not found, try pulling it first"}"#,
            "x",
        );
        assert_eq!(nf, DecisionError::ModelMissing("x".into()));
        let route = map_error_response(StatusCode::NOT_FOUND, "404 page not found", "x");
        assert!(matches!(route, DecisionError::Unsupported(_)));
        let nodec = map_error_response(
            StatusCode::BAD_REQUEST,
            r#"{"error":"\"llama3\" does not support decision"}"#,
            "llama3",
        );
        assert!(matches!(nodec, DecisionError::Unsupported(_)));
        let long = map_error_response(
            StatusCode::BAD_REQUEST,
            "prompt 0 has 2100 tokens; expected 1-2050",
            "m",
        );
        assert!(
            matches!(&long, DecisionError::Contract(m) if m.contains("http 400") && m.contains("tokens"))
        );
        assert!(matches!(
            map_error_response(StatusCode::UNAUTHORIZED, "", "m"),
            DecisionError::Unavailable(_)
        ));
        assert!(map_error_response(StatusCode::BAD_GATEWAY, "x", "m").is_retryable());
        assert!(matches!(
            map_error_response(StatusCode::GATEWAY_TIMEOUT, "", "m"),
            DecisionError::Timeout(_)
        ));
    }

    // T-160-U50 — capability detection reads `capabilities`.
    #[test]
    fn capability_detection() {
        assert!(has_decision_capability(
            &json!({"capabilities":["decision"]})
        ));
        assert!(!has_decision_capability(
            &json!({"capabilities":["completion"]})
        ));
        assert!(!has_decision_capability(&json!({})));
    }

    // T-160-U51 — an API key never shows in the descriptor.
    #[test]
    fn descriptor_hides_secrets() {
        let b = OllamaBackend::new(
            "https://u:pw@gpu.example.com:11434/",
            "tev1:0.8b",
            Some("sekret".into()),
            5,
        )
        .unwrap();
        let d = serde_json::to_string(&b.descriptor()).unwrap();
        assert!(d.contains("gpu.example.com:11434") && !d.contains("sekret") && !d.contains("pw@"));
    }
}
