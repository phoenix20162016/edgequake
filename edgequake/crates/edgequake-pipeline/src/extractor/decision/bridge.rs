//! Sync-to-async bridge (SPEC-160, DIP).
//!
//! `edgextract` calls a blocking [`Transport`]. EdgeQuake owns an async
//! backend and an async answer cache. The bridge runs inside
//! `spawn_blocking` and uses the runtime handle to await both.
//!
//! One request takes this path:
//!
//! ```text
//!   Extractor ──post_json──▶ cancel? ──▶ cache hit? ──yes──▶ answer
//!                                            │ no
//!                                            ▼
//!                                   backend.systemone ──▶ validate ──▶ cache.put ──▶ answer
//! ```
//!
//! Only a valid answer reaches the cache. A cache fault is a miss, never a failure.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use edgequake_storage::decision::{DecisionScope, DecisionStore};
use edgextract::cache::request_key;
use edgextract::systemone::{validate_response, Transport};
use edgextract::{SystemOneError, DECISION_CONTRACT};
use serde_json::Value;
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

use super::backend::DecisionBackend;
use super::error::DecisionError;

/// The persistent answer cache and the scope it writes to.
#[derive(Clone)]
pub struct CacheLink {
    pub store: Arc<dyn DecisionStore>,
    pub scope: DecisionScope,
}

/// Counters the bridge keeps for one run.
#[derive(Debug, Default)]
pub struct BridgeCounters {
    backend_calls: AtomicU64,
    cache_hits: AtomicU64,
}

impl BridgeCounters {
    pub fn backend_calls(&self) -> u64 {
        self.backend_calls.load(Ordering::Relaxed)
    }

    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }
}

/// Transport that serves `edgextract` from an async backend and cache.
pub struct BridgeTransport {
    backend: Arc<dyn DecisionBackend>,
    handle: Handle,
    cache: Option<CacheLink>,
    cancel: CancellationToken,
    counters: Arc<BridgeCounters>,
    errors: Mutex<Vec<DecisionError>>,
}

impl BridgeTransport {
    pub fn new(
        backend: Arc<dyn DecisionBackend>,
        handle: Handle,
        cache: Option<CacheLink>,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            backend,
            handle,
            cache,
            cancel,
            counters: Arc::new(BridgeCounters::default()),
            errors: Mutex::new(Vec::new()),
        }
    }

    pub fn counters(&self) -> Arc<BridgeCounters> {
        Arc::clone(&self.counters)
    }

    /// Typed errors seen so far, oldest first.
    pub fn take_errors(&self) -> Vec<DecisionError> {
        self.errors
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }

    fn note(&self, err: &DecisionError) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(err.clone());
        }
    }

    fn cache_key(&self, body: &Value) -> String {
        let model = body.get("model").and_then(Value::as_str).unwrap_or("");
        let null = Value::Null;
        request_key(
            DECISION_CONTRACT,
            model,
            body.get("state").unwrap_or(&null),
            body.get("questions").unwrap_or(&null),
            body.get("images"),
        )
    }

    fn cache_get(&self, key: &str) -> Option<Value> {
        let link = self.cache.as_ref()?;
        match self
            .handle
            .block_on(link.store.get_answer(&link.scope, key))
        {
            Ok(hit) => hit,
            Err(err) => {
                tracing::warn!(error = %err, "decision cache read failed; treating as a miss");
                None
            }
        }
    }

    fn cache_put(&self, key: &str, model: &str, answer: &Value) {
        let Some(link) = self.cache.as_ref() else {
            return;
        };
        let put = link
            .store
            .put_answer(&link.scope, key, model, DECISION_CONTRACT, answer);
        if let Err(err) = self.handle.block_on(put) {
            tracing::warn!(error = %err, "decision cache write failed; answer kept for this run only");
        }
    }

    fn ask_backend(&self, body: &Value) -> Result<Value, DecisionError> {
        self.handle.block_on(async {
            tokio::select! {
                biased;
                _ = self.cancel.cancelled() => Err(DecisionError::Cancelled),
                answer = self.backend.systemone(body) => answer,
            }
        })
    }

    fn fail(&self, err: DecisionError) -> SystemOneError {
        self.note(&err);
        SystemOneError::new(err.to_string())
    }
}

impl Transport for BridgeTransport {
    fn post_json(&self, _path: &str, body: &Value) -> Result<Value, SystemOneError> {
        if self.cancel.is_cancelled() {
            return Err(self.fail(DecisionError::Cancelled));
        }
        let key = self.cache_key(body);
        if let Some(hit) = self.cache_get(&key) {
            self.counters.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(hit);
        }
        let answer = self.ask_backend(body).map_err(|e| self.fail(e))?;
        self.counters.backend_calls.fetch_add(1, Ordering::Relaxed);
        validate_response(&answer, None)
            .map_err(|e| self.fail(DecisionError::Contract(e.to_string())))?;
        let model = body.get("model").and_then(Value::as_str).unwrap_or("");
        self.cache_put(&key, model, &answer);
        Ok(answer)
    }
}

/// Lets the extractor own a transport while the caller keeps the counters.
pub struct SharedTransport(pub Arc<BridgeTransport>);

impl Transport for SharedTransport {
    fn post_json(&self, path: &str, body: &Value) -> Result<Value, SystemOneError> {
        self.0.post_json(path, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::decision::backend::{BackendDescriptor, BackendHealth, BackendKind};
    use async_trait::async_trait;
    use edgequake_storage::decision::MemoryDecisionStore;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;

    struct Scripted {
        calls: AtomicUsize,
        reply: Result<Value, DecisionError>,
    }

    #[async_trait]
    impl DecisionBackend for Scripted {
        fn descriptor(&self) -> BackendDescriptor {
            BackendDescriptor {
                kind: BackendKind::OllamaSystemOne,
                model: "m".into(),
                host: "h".into(),
            }
        }
        async fn health(&self) -> BackendHealth {
            BackendHealth::unreachable("n/a")
        }
        async fn systemone(&self, _r: &Value) -> Result<Value, DecisionError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.reply.clone()
        }
    }

    fn good() -> Value {
        json!({"model":"m","answers":{"q":{"type":"noul","noul":0.9}},"usage":{"input_tokens":3,"output_tokens":1}})
    }

    fn request() -> Value {
        json!({"model":"m","state":"s","questions":{"q":{"type":"noul","instructions":"i"}}})
    }

    fn link() -> CacheLink {
        CacheLink {
            store: Arc::new(MemoryDecisionStore::new()),
            scope: DecisionScope::new(None, "ws"),
        }
    }

    fn bridge(
        reply: Result<Value, DecisionError>,
        cache: Option<CacheLink>,
    ) -> (BridgeTransport, Arc<Scripted>) {
        let backend = Arc::new(Scripted {
            calls: AtomicUsize::new(0),
            reply,
        });
        let t = BridgeTransport::new(
            backend.clone(),
            Handle::current(),
            cache,
            CancellationToken::new(),
        );
        (t, backend)
    }

    async fn on_blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        tokio::task::spawn_blocking(f).await.unwrap()
    }

    // T-160-I22 — the second identical request is served from the cache.
    #[tokio::test(flavor = "multi_thread")]
    async fn second_request_hits_cache() {
        let (t, backend) = bridge(Ok(good()), Some(link()));
        let t = Arc::new(t);
        let t2 = t.clone();
        on_blocking(move || {
            t2.post_json("/v1/systemone", &request()).unwrap();
            t2.post_json("/v1/systemone", &request()).unwrap();
        })
        .await;
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            (t.counters().backend_calls(), t.counters().cache_hits()),
            (1, 1)
        );
    }

    // T-160-I23 — an invalid answer is an error and is never cached.
    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_answer_is_not_cached() {
        let bad = json!({"model":"m","answers":{"q":{"type":"noul","noul":7}}});
        let l = link();
        let (t, backend) = bridge(Ok(bad), Some(l.clone()));
        let t = Arc::new(t);
        let t2 = t.clone();
        let r = on_blocking(move || t2.post_json("/v1/systemone", &request())).await;
        assert!(r.is_err());
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        assert_eq!(l.store.cache_len(&l.scope).await.unwrap(), 0);
        assert!(matches!(
            t.take_errors().as_slice(),
            [DecisionError::Contract(_)]
        ));
    }

    // T-160-I24 — a backend error keeps its type for the caller.
    #[tokio::test(flavor = "multi_thread")]
    async fn backend_error_is_typed() {
        let (t, _) = bridge(Err(DecisionError::Unavailable("refused".into())), None);
        let t = Arc::new(t);
        let t2 = t.clone();
        assert!(
            on_blocking(move || t2.post_json("/v1/systemone", &request()))
                .await
                .is_err()
        );
        assert!(matches!(
            t.take_errors().as_slice(),
            [DecisionError::Unavailable(_)]
        ));
    }

    // T-160-I25 — a cancelled run asks the backend nothing.
    #[tokio::test(flavor = "multi_thread")]
    async fn cancelled_run_skips_backend() {
        let backend = Arc::new(Scripted {
            calls: AtomicUsize::new(0),
            reply: Ok(good()),
        });
        let cancel = CancellationToken::new();
        cancel.cancel();
        let t = Arc::new(BridgeTransport::new(
            backend.clone(),
            Handle::current(),
            None,
            cancel,
        ));
        let t2 = t.clone();
        assert!(
            on_blocking(move || t2.post_json("/v1/systemone", &request()))
                .await
                .is_err()
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
        assert!(matches!(
            t.take_errors().as_slice(),
            [DecisionError::Cancelled]
        ));
    }

    // T-160-I26 — different models never share a cache entry (EC-160-31).
    #[tokio::test(flavor = "multi_thread")]
    async fn model_is_part_of_the_key() {
        let (t, _) = bridge(Ok(good()), None);
        let mut other = request();
        other["model"] = json!("other");
        assert_ne!(t.cache_key(&request()), t.cache_key(&other));
    }
}
