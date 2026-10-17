//! Request coalescing — dedupe concurrent identical upstream requests
//! (PR7).
//!
//! When N requests with the same `CoalesceKey` arrive simultaneously, only
//! the first runs the `primary` future; the other N-1 wait for its
//! outcome and piggyback on it. This mitigates the "thundering herd"
//! problem on high-QPS read paths where every client request would
//! otherwise trigger the same expensive upstream call.
//!
//! # Comparison with caching (PR6)
//!
//! - **Cache (PR6)** collapses SEQUENTIAL duplicates: the second
//!   identical request hits the in-memory entry stored by the first.
//! - **Coalesce (PR7)** collapses CONCURRENT duplicates: the second
//!   identical request, arriving while the first is still upstream,
//!   waits for the first's response instead of issuing a parallel call.
//!
//! Together, PR6 + PR7 give a clean caching layer for read-heavy routes:
//! cache handles misses via LRU+ETag, coalesce handles bursts via
//! single-flight.
//!
//! # Wire-up
//!
//! The tracker is generic over the future returned by `primary`. It
//! stores a `CoalesceOutcome` (Clone) shared between primary and
//! waiters. To piggyback on an HTTP response, build the outcome from
//! the upstream's `(status, headers, body)` triple; to piggyback on an
//! error, use `CoalesceOutcome::err(msg)`.

use bytes::Bytes;
use dashmap::DashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Coalesce key — typically `(route, method, sorted_query)` so that
/// requests for different routes don't accidentally share an in-flight
/// slot. Use [`crate::services::cache::compute_cache_key`] to derive a
/// key consistent with the cache layer.
pub type CoalesceKey = String;

/// Outcome shared between the primary and all waiters. `Clone` is
/// required so waiters can receive a copy after the primary has
/// finished; `Bytes` is cheap to clone (refcount bump), `Vec<(String,
/// String)>` is small for typical upstream responses.
#[derive(Clone, Debug)]
pub enum CoalesceOutcome {
    /// Successful upstream response: status, headers, body.
    Response {
        status: u16,
        headers: Vec<(String, String)>,
        body: Bytes,
    },
    /// Upstream error. Stringified (no `Clone` on `GatewayError`) so
    /// waiters can receive it without lifetime gymnastics.
    Error(String),
}

impl CoalesceOutcome {
    /// Build a successful outcome from a response's components.
    pub fn ok(
        status: u16,
        headers: Vec<(String, String)>,
        body: Bytes,
    ) -> Self {
        CoalesceOutcome::Response {
            status,
            headers,
            body,
        }
    }

    /// Build an error outcome from a display-able error.
    pub fn err(msg: impl Into<String>) -> Self {
        CoalesceOutcome::Error(msg.into())
    }
}

struct InflightSlot {
    notify: Notify,
    state: Mutex<SlotState>,
}

enum SlotState {
    Pending,
    Done(CoalesceOutcome),
}

/// In-flight request coalescer.
///
/// One instance per gateway. Routes that opt in (see
/// `CacheConfig::coalesce`) use this tracker via the proxy handler to
/// collapse concurrent identical upstream requests.
///
/// # Counters
///
/// `primary_calls` and `waiters` are exposed so the metrics layer can
/// sync them into the Prometheus-style collector on demand (mirrors
/// the PR6 `sync_cache_stats` pattern).
pub struct InFlightTracker {
    inflight: DashMap<CoalesceKey, Arc<InflightSlot>>,
    primary_calls: Arc<AtomicU64>,
    waiters: Arc<AtomicU64>,
}

impl InFlightTracker {
    pub fn new() -> Self {
        Self {
            inflight: DashMap::new(),
            primary_calls: Arc::new(AtomicU64::new(0)),
            waiters: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Run `primary` for `key`.
    ///
    /// - If no request with `key` is in flight, this call is the
    ///   **primary**: it runs `primary`, broadcasts the outcome to any
    ///   future waiters, then deregisters itself.
    /// - If another request is already running `key`, this call is a
    ///   **waiter**: it blocks on `notify.notified()` until the primary
    ///   finishes, then returns the primary's outcome.
    ///
    /// `primary` is `FnOnce` to enforce it runs exactly once (only the
    /// leader ever invokes it).
    ///
    /// # Race-free slot lookup
    ///
    /// A waiter holds a local `Arc<InflightSlot>` so the slot outlives
    /// the DashMap entry — even if the primary finishes and removes
    /// the entry while the waiter is still pending, the waiter's Arc
    /// keeps the slot state and Notify alive until it exits the loop.
    pub async fn coalesce<F, Fut>(
        &self,
        key: CoalesceKey,
        primary: F,
    ) -> CoalesceOutcome
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = CoalesceOutcome>,
    {
        let slot = Arc::new(InflightSlot {
            notify: Notify::new(),
            state: Mutex::new(SlotState::Pending),
        });

        // Try to claim the key. Vacant -> we're primary. Occupied ->
        // someone else is already primary; we'll piggyback.
        let we_are_primary = match self.inflight.entry(key.clone()) {
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(slot.clone());
                true
            }
            dashmap::mapref::entry::Entry::Occupied(_) => false,
        };

        if we_are_primary {
            self.primary_calls.fetch_add(1, Ordering::Relaxed);
            let outcome = primary().await;

            // Broadcast the outcome to any waiters. notify_waiters
            // wakes every task currently parked on this Notify; new
            // waiters arriving after this call will see `state = Done`
            // on their first lock and return immediately.
            {
                let mut state = slot.state.lock().unwrap();
                *state = SlotState::Done(outcome.clone());
            }
            slot.notify.notify_waiters();

            // Deregister. Any future waiter that hits this key will
            // see Vacant and become a new primary — correct behaviour
            // for cache-cold reads after a burst subsides.
            self.inflight.remove(&key);

            outcome
        } else {
            self.waiters.fetch_add(1, Ordering::Relaxed);

            // Clone the slot Arc so we can release the DashMap borrow.
            // The Arc keeps the slot alive even if the primary removes
            // the entry while we're inside the loop.
            let primary_slot = self
                .inflight
                .get(&key)
                .map(|entry| entry.value().clone())
                .expect("if we are not primary, someone else's slot must be in the map");

            loop {
                // Check state first — the primary may have finished
                // between our get() and now (notify_waiters is
                // fire-and-forget; we still need to read the state).
                {
                    let state = primary_slot.state.lock().unwrap();
                    if let SlotState::Done(r) = &*state {
                        return r.clone();
                    }
                }
                primary_slot.notify.notified().await;
            }
        }
    }

    /// Total times this tracker was selected as primary (i.e. the
    /// upstream future was actually executed).
    pub fn primary_calls(&self) -> u64 {
        self.primary_calls.load(Ordering::Relaxed)
    }

    /// Total times a request piggybacked on an existing primary.
    pub fn waiters(&self) -> u64 {
        self.waiters.load(Ordering::Relaxed)
    }
}

impl Default for InFlightTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    #[tokio::test]
    async fn single_request_runs_as_primary() {
        let tracker = InFlightTracker::new();
        let outcome = tracker
            .coalesce("k1".to_string(), || async {
                CoalesceOutcome::ok(200, vec![], Bytes::from("body"))
            })
            .await;
        assert!(matches!(outcome, CoalesceOutcome::Response { status: 200, .. }));
        assert_eq!(tracker.primary_calls(), 1);
        assert_eq!(tracker.waiters(), 0);
    }

    #[tokio::test]
    async fn concurrent_requests_coalesce_to_one_primary() {
        let tracker = Arc::new(InFlightTracker::new());
        let upstream_calls = Arc::new(AtomicUsize::new(0));

        // 50 concurrent identical requests.
        let mut handles = Vec::new();
        for _ in 0..50 {
            let tracker = tracker.clone();
            let upstream_calls = upstream_calls.clone();
            handles.push(tokio::spawn(async move {
                tracker
                    .coalesce("hot-key".to_string(), || {
                        let upstream_calls = upstream_calls.clone();
                        async move {
                            upstream_calls.fetch_add(1, Ordering::Relaxed);
                            // Simulate upstream latency.
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            CoalesceOutcome::ok(
                                200,
                                vec![("content-type".to_string(), "application/json".to_string())],
                                Bytes::from_static(b"{\"id\":42}"),
                            )
                        }
                    })
                    .await
            }));
        }

        let outcomes: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|h| h.expect("task panicked"))
            .collect();

        // All 50 requests must have received the response.
        assert_eq!(outcomes.len(), 50);
        for o in &outcomes {
            assert!(matches!(o, CoalesceOutcome::Response { status: 200, .. }));
        }

        // But only ONE upstream call should have happened.
        assert_eq!(
            upstream_calls.load(Ordering::Relaxed),
            1,
            "expected exactly 1 upstream call, got {}",
            upstream_calls.load(Ordering::Relaxed)
        );

        // 1 primary + 49 waiters.
        assert_eq!(tracker.primary_calls(), 1);
        assert_eq!(tracker.waiters(), 49);
    }

    #[tokio::test]
    async fn different_keys_do_not_coalesce() {
        let tracker = Arc::new(InFlightTracker::new());
        let upstream_calls = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for key in ["a", "b", "c", "d", "e"] {
            let tracker = tracker.clone();
            let upstream_calls = upstream_calls.clone();
            let key = key.to_string();
            handles.push(tokio::spawn(async move {
                tracker
                    .coalesce(key, || {
                        let upstream_calls = upstream_calls.clone();
                        async move {
                            upstream_calls.fetch_add(1, Ordering::Relaxed);
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            CoalesceOutcome::ok(200, vec![], Bytes::from("ok"))
                        }
                    })
                    .await
            }));
        }

        let _ = futures::future::join_all(handles).await;
        // 5 distinct keys → 5 distinct upstream calls.
        assert_eq!(upstream_calls.load(Ordering::Relaxed), 5);
        assert_eq!(tracker.primary_calls(), 5);
        assert_eq!(tracker.waiters(), 0);
    }

    #[tokio::test]
    async fn error_propagates_to_all_waiters() {
        let tracker = Arc::new(InFlightTracker::new());

        let mut handles = Vec::new();
        for _ in 0..10 {
            let tracker = tracker.clone();
            handles.push(tokio::spawn(async move {
                tracker
                    .coalesce("failing-key".to_string(), || async {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        CoalesceOutcome::err("upstream returned 500")
                    })
                    .await
            }));
        }

        let outcomes: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|h| h.expect("task panicked"))
            .collect();

        // All 10 (1 primary + 9 waiters) get the same error.
        assert_eq!(outcomes.len(), 10);
        for o in &outcomes {
            match o {
                CoalesceOutcome::Error(msg) => {
                    assert_eq!(msg, "upstream returned 500");
                }
                other => panic!("expected Error, got {:?}", other),
            }
        }
        assert_eq!(tracker.primary_calls(), 1);
        assert_eq!(tracker.waiters(), 9);
    }

    #[tokio::test]
    async fn late_request_after_completion_becomes_new_primary() {
        let tracker = InFlightTracker::new();
        // First wave.
        for _ in 0..2 {
            tracker
                .coalesce("seq-key".to_string(), || async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    CoalesceOutcome::ok(200, vec![], Bytes::from("v1"))
                })
                .await;
        }
        // By now the first wave's entry has been deregistered.
        let before = tracker.primary_calls();
        // Second wave — same key, sequential.
        for _ in 0..2 {
            tracker
                .coalesce("seq-key".to_string(), || async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    CoalesceOutcome::ok(200, vec![], Bytes::from("v2"))
                })
                .await;
        }
        // Second wave must have produced 2 NEW primaries (no entries
        // were cached).
        assert_eq!(tracker.primary_calls() - before, 2);
    }
}