//! In-memory [`DecisionStore`]. Same semantics as the Postgres adapter.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use serde_json::Value;

use super::{DecisionScope, DecisionStore, ReviewRow};
use crate::error::StorageError;

struct CacheEntry {
    answer: Value,
    last_used: SystemTime,
}

#[derive(Default)]
struct Inner {
    /// `(workspace_id, key)` to entry.
    cache: HashMap<(String, String), CacheEntry>,
    /// `(workspace_id, document_id)` to rows.
    review: HashMap<(String, String), Vec<ReviewRow>>,
}

/// In-memory decision store.
#[derive(Default)]
pub struct MemoryDecisionStore {
    inner: Mutex<Inner>,
    /// Number of `put_answer` calls. Tests assert cache behavior with it.
    puts: std::sync::atomic::AtomicU64,
}

impl MemoryDecisionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of answers written since creation.
    pub fn put_count(&self) -> u64 {
        self.puts.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Test helper: age every cache entry of a workspace.
    pub fn age_cache(&self, workspace_id: &str, by: Duration) {
        let mut g = self.lock();
        for ((ws, _), e) in g.cache.iter_mut() {
            if ws == workspace_id {
                e.last_used = e
                    .last_used
                    .checked_sub(by)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
            }
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[async_trait]
impl DecisionStore for MemoryDecisionStore {
    async fn get_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
    ) -> Result<Option<Value>, StorageError> {
        let mut g = self.lock();
        Ok(g.cache
            .get_mut(&(scope.workspace_id.clone(), key.to_string()))
            .map(|e| {
                e.last_used = SystemTime::now();
                e.answer.clone()
            }))
    }

    async fn put_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
        _model: &str,
        _contract: &str,
        answer: &Value,
    ) -> Result<(), StorageError> {
        self.puts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.lock().cache.insert(
            (scope.workspace_id.clone(), key.to_string()),
            CacheEntry {
                answer: answer.clone(),
                last_used: SystemTime::now(),
            },
        );
        Ok(())
    }

    async fn replace_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
        rows: &[ReviewRow],
    ) -> Result<(), StorageError> {
        let k = (scope.workspace_id.clone(), document_id.to_string());
        let mut g = self.lock();
        if rows.is_empty() {
            g.review.remove(&k);
        } else {
            g.review.insert(k, rows.to_vec());
        }
        Ok(())
    }

    async fn list_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<Vec<ReviewRow>, StorageError> {
        let mut rows = self
            .lock()
            .review
            .get(&(scope.workspace_id.clone(), document_id.to_string()))
            .cloned()
            .unwrap_or_default();
        rows.sort_by(|a, b| b.score.total_cmp(&a.score));
        Ok(rows)
    }

    async fn delete_document(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<u64, StorageError> {
        Ok(self
            .lock()
            .review
            .remove(&(scope.workspace_id.clone(), document_id.to_string()))
            .map(|r| r.len() as u64)
            .unwrap_or(0))
    }

    async fn delete_workspace(&self, scope: &DecisionScope) -> Result<u64, StorageError> {
        let ws = &scope.workspace_id;
        let mut g = self.lock();
        let before = g.cache.len() as u64 + g.review.values().map(|r| r.len() as u64).sum::<u64>();
        g.cache.retain(|(w, _), _| w != ws);
        g.review.retain(|(w, _), _| w != ws);
        let after = g.cache.len() as u64 + g.review.values().map(|r| r.len() as u64).sum::<u64>();
        Ok(before - after)
    }

    async fn sweep_cache(
        &self,
        scope: &DecisionScope,
        ttl_days: u32,
        max_rows: u64,
    ) -> Result<u64, StorageError> {
        let ws = &scope.workspace_id;
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(u64::from(ttl_days) * 86_400))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mut g = self.lock();
        let before = g.cache.len();
        g.cache.retain(|(w, _), e| w != ws || e.last_used >= cutoff);
        let mut mine: Vec<((String, String), SystemTime)> = g
            .cache
            .iter()
            .filter(|((w, _), _)| w == ws)
            .map(|(k, e)| (k.clone(), e.last_used))
            .collect();
        if mine.len() as u64 > max_rows {
            mine.sort_by_key(|(_, t)| *t);
            let excess = mine.len() - max_rows as usize;
            for (k, _) in mine.into_iter().take(excess) {
                g.cache.remove(&k);
            }
        }
        Ok((before - g.cache.len()) as u64)
    }

    async fn cache_len(&self, scope: &DecisionScope) -> Result<u64, StorageError> {
        Ok(self
            .lock()
            .cache
            .keys()
            .filter(|(w, _)| w == &scope.workspace_id)
            .count() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ReviewKind;
    use super::*;
    use serde_json::json;

    fn scope(ws: &str) -> DecisionScope {
        DecisionScope::new(None, ws)
    }

    fn row(score: f32) -> ReviewRow {
        ReviewRow {
            chunk_id: "c1".into(),
            kind: ReviewKind::Entity,
            subject: "Acme".into(),
            label: "ORGANIZATION".into(),
            object: None,
            score,
            reason: None,
            sentence: "Acme grew.".into(),
            model: "m".into(),
            contract: "k".into(),
        }
    }

    // T-160-S01 — an answer round-trips and is scoped to its workspace.
    #[tokio::test]
    async fn answers_are_workspace_scoped() {
        let s = MemoryDecisionStore::new();
        s.put_answer(&scope("a"), "k", "m", "c", &json!({"x": 1}))
            .await
            .unwrap();
        assert_eq!(
            s.get_answer(&scope("a"), "k").await.unwrap(),
            Some(json!({"x": 1}))
        );
        assert_eq!(s.get_answer(&scope("b"), "k").await.unwrap(), None);
    }

    // T-160-S02 — replace_review replaces, never stacks.
    #[tokio::test]
    async fn replace_review_does_not_stack() {
        let s = MemoryDecisionStore::new();
        s.replace_review(&scope("a"), "d", &[row(0.5), row(0.6)])
            .await
            .unwrap();
        s.replace_review(&scope("a"), "d", &[row(0.9)])
            .await
            .unwrap();
        let rows = s.list_review(&scope("a"), "d").await.unwrap();
        assert_eq!(rows.len(), 1);
        s.replace_review(&scope("a"), "d", &[]).await.unwrap();
        assert!(s.list_review(&scope("a"), "d").await.unwrap().is_empty());
    }

    // T-160-S03 — review rows list best score first.
    #[tokio::test]
    async fn review_lists_best_first() {
        let s = MemoryDecisionStore::new();
        s.replace_review(&scope("a"), "d", &[row(0.45), row(0.65), row(0.55)])
            .await
            .unwrap();
        let scores: Vec<f32> = s
            .list_review(&scope("a"), "d")
            .await
            .unwrap()
            .iter()
            .map(|r| r.score)
            .collect();
        assert_eq!(scores, vec![0.65, 0.55, 0.45]);
    }

    // T-160-S04 — delete_document and delete_workspace remove the right rows.
    #[tokio::test]
    async fn deletes_are_scoped() {
        let s = MemoryDecisionStore::new();
        s.replace_review(&scope("a"), "d1", &[row(0.5)])
            .await
            .unwrap();
        s.replace_review(&scope("a"), "d2", &[row(0.5)])
            .await
            .unwrap();
        s.replace_review(&scope("b"), "d1", &[row(0.5)])
            .await
            .unwrap();
        s.put_answer(&scope("a"), "k", "m", "c", &json!(1))
            .await
            .unwrap();
        assert_eq!(s.delete_document(&scope("a"), "d1").await.unwrap(), 1);
        assert_eq!(s.list_review(&scope("a"), "d2").await.unwrap().len(), 1);
        assert_eq!(s.list_review(&scope("b"), "d1").await.unwrap().len(), 1);
        assert_eq!(s.delete_workspace(&scope("a")).await.unwrap(), 2);
        assert_eq!(s.cache_len(&scope("a")).await.unwrap(), 0);
        assert_eq!(s.list_review(&scope("b"), "d1").await.unwrap().len(), 1);
    }

    // T-160-S05 — sweep enforces TTL and the row cap, oldest first.
    #[tokio::test]
    async fn sweep_enforces_ttl_and_cap() {
        let s = MemoryDecisionStore::new();
        for i in 0..5 {
            s.put_answer(&scope("a"), &format!("k{i}"), "m", "c", &json!(i))
                .await
                .unwrap();
        }
        s.age_cache("a", Duration::from_secs(40 * 86_400));
        s.put_answer(&scope("a"), "fresh", "m", "c", &json!(9))
            .await
            .unwrap();
        assert_eq!(s.sweep_cache(&scope("a"), 30, 100).await.unwrap(), 5);
        assert_eq!(s.cache_len(&scope("a")).await.unwrap(), 1);
        for i in 0..4 {
            s.put_answer(&scope("a"), &format!("n{i}"), "m", "c", &json!(i))
                .await
                .unwrap();
        }
        assert_eq!(s.sweep_cache(&scope("a"), 30, 2).await.unwrap(), 3);
        assert_eq!(s.cache_len(&scope("a")).await.unwrap(), 2);
    }

    // T-160-S06 — clip_sentence cuts on a char boundary.
    #[test]
    fn clip_sentence_is_char_safe() {
        let long = "é".repeat(2000);
        assert_eq!(ReviewRow::clip_sentence(&long).chars().count(), 1000);
    }
}
