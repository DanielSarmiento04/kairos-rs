//! Integration tests for PR7 — request coalescing (single-flight).
//!
//! These tests verify the wire-up between `InFlightTracker` and
//! `RouteHandler`:
//!
//! - `RouteHandler::coalesce_stats()` returns `None` when no tracker
//!   is wired, and `Some((primary, waiters))` when one is.
//! - Counters increment correctly when `InFlightTracker::coalesce()`
//!   runs (via the `with_inflight_tracker` builder).
//! - 50 concurrent identical calls collapse to 1 primary + 49 waiters,
//!   demonstrating the "thundering herd" mitigation.
//!
//! The HTTP-level end-to-end (actix server + real upstream) is covered
//! by the gateway crate's own tests. Here we focus on the coalesce
//! primitives + the `RouteHandler` glue so a regression in the builder,
//! `coalesce_stats()` helper, or counter wiring is caught fast.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use kairos_rs::models::router::{CacheConfig, Protocol, Router};
use kairos_rs::services::inflight::InFlightTracker;
use kairos_rs::services::http::RouteHandler;

// ---------- helpers ----------

fn make_route(coalesce: bool, auth_required: bool) -> Router {
    Router {
        host: Some("http://backend".into()),
        port: Some(80),
        backends: None,
        protocol: Protocol::Http,
        load_balancing_strategy: Default::default(),
        external_path: "/api/test".into(),
        internal_path: "/test".into(),
        methods: vec!["GET".into()],
        auth_required,
        retry: None,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
        cache: Some(CacheConfig {
            enabled: true,
            ttl_secs: 60,
            max_size: 100,
            coalesce,
        }),
    }
}

fn handler_with_tracker() -> (RouteHandler, Arc<InFlightTracker>) {
    let tracker = Arc::new(InFlightTracker::new());
    let mut handler = RouteHandler::new(vec![make_route(true, false)], 30);
    handler = handler.with_inflight_tracker(Arc::clone(&tracker));
    (handler, tracker)
}

// ---------- coalesce_stats() returns the right shape ----------

#[test]
fn coalesce_stats_none_without_tracker() {
    // No `with_inflight_tracker` call → handler has no InFlightTracker.
    let handler = RouteHandler::new(vec![make_route(false, false)], 30);
    assert!(
        handler.coalesce_stats().is_none(),
        "must be None when no InFlightTracker is wired"
    );
}

#[test]
fn coalesce_stats_zero_zero_with_fresh_tracker() {
    let (handler, _tracker) = handler_with_tracker();
    let (primary, waiters) = handler
        .coalesce_stats()
        .expect("Some when InFlightTracker is wired");
    assert_eq!(primary, 0);
    assert_eq!(waiters, 0);
}

// ---------- counter wiring: 50 concurrent identical → 1 primary + 49 waiters ----------

#[tokio::test]
async fn fifty_concurrent_identical_calls_collapse_to_one_primary() {
    let (handler, tracker) = handler_with_tracker();

    let upstream_calls = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for _ in 0..50 {
        let tracker = Arc::clone(&tracker);
        let upstream_calls = Arc::clone(&upstream_calls);
        handles.push(tokio::spawn(async move {
            tracker
                .coalesce("hot-key".to_string(), || {
                    let upstream_calls = Arc::clone(&upstream_calls);
                    async move {
                        upstream_calls.fetch_add(1, Ordering::Relaxed);
                        // Simulate upstream latency so the 50 tasks
                        // genuinely overlap and exercise the waiter
                        // path (vs. a single-threaded serial run).
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        // The result type here is CoalesceOutcome, not
                        // HttpResponse — that's fine, the tracker
                        // doesn't care about the payload shape.
                        kairos_rs::services::inflight::CoalesceOutcome::ok(
                            200,
                            vec![],
                            bytes::Bytes::from_static(b"{\"id\":42}"),
                        )
                    }
                })
                .await
        }));
    }

    let _ = futures::future::join_all(handles).await;

    // Exactly 1 upstream call for 50 identical requests.
    assert_eq!(
        upstream_calls.load(Ordering::Relaxed),
        1,
        "expected exactly 1 upstream call, got {}",
        upstream_calls.load(Ordering::Relaxed)
    );

    // Tracker's internal counters: 1 primary, 49 waiters.
    assert_eq!(tracker.primary_calls(), 1);
    assert_eq!(tracker.waiters(), 49);

    // RouteHandler::coalesce_stats() mirrors the same numbers.
    let (primary, waiters) = handler.coalesce_stats().unwrap();
    assert_eq!(primary, 1);
    assert_eq!(waiters, 49);
}

#[tokio::test]
async fn different_keys_do_not_coalesce() {
    // 5 distinct keys → 5 distinct upstream calls, 0 waiters.
    let (handler, tracker) = handler_with_tracker();
    let upstream_calls = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for key in ["a", "b", "c", "d", "e"] {
        let tracker = Arc::clone(&tracker);
        let upstream_calls = Arc::clone(&upstream_calls);
        let key = key.to_string();
        handles.push(tokio::spawn(async move {
            tracker
                .coalesce(key, || {
                    let upstream_calls = Arc::clone(&upstream_calls);
                    async move {
                        upstream_calls.fetch_add(1, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        kairos_rs::services::inflight::CoalesceOutcome::ok(
                            200,
                            vec![],
                            bytes::Bytes::from_static(b"ok"),
                        )
                    }
                })
                .await
        }));
    }
    let _ = futures::future::join_all(handles).await;
    assert_eq!(upstream_calls.load(Ordering::Relaxed), 5);
    assert_eq!(tracker.primary_calls(), 5);
    assert_eq!(tracker.waiters(), 0);
    let (primary, waiters) = handler.coalesce_stats().unwrap();
    assert_eq!(primary, 5);
    assert_eq!(waiters, 0);
}

#[tokio::test]
async fn error_propagates_to_all_waiters() {
    // 10 concurrent identical calls where the primary errors. All 10
    // (1 primary + 9 waiters) receive the same `CoalesceOutcome::Error`.
    let (handler, tracker) = handler_with_tracker();
    let mut handles = Vec::new();
    for _ in 0..10 {
        let tracker = Arc::clone(&tracker);
        handles.push(tokio::spawn(async move {
            tracker
                .coalesce("failing-key".to_string(), || async {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    kairos_rs::services::inflight::CoalesceOutcome::err("upstream returned 500")
                })
                .await
        }));
    }
    let outcomes: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.expect("task panicked"))
        .collect();
    assert_eq!(outcomes.len(), 10);
    for o in &outcomes {
        assert!(matches!(o, kairos_rs::services::inflight::CoalesceOutcome::Error(_)));
    }
    assert_eq!(tracker.primary_calls(), 1);
    assert_eq!(tracker.waiters(), 9);
    let (primary, waiters) = handler.coalesce_stats().unwrap();
    assert_eq!(primary, 1);
    assert_eq!(waiters, 9);
}

#[tokio::test]
async fn late_request_after_completion_becomes_new_primary() {
    // Two waves of 2 CONCURRENT calls each. Within a wave, the 2nd
    // call is a waiter for the 1st (entry still in-flight). Between
    // waves, the 1st call of wave 2 is a NEW primary because wave 1
    // deregistered its entry on completion. So: wave 1 = 1 primary
    // + 1 waiter, wave 2 = 1 new primary + 1 waiter. Net: 1 new
    // primary + 1 waiter after wave 1.
    let (_handler, tracker) = handler_with_tracker();

    // Wave 1: 2 concurrent calls.
    let t1 = Arc::clone(&tracker);
    let t2 = Arc::clone(&tracker);
    let h1 = tokio::spawn(async move {
        t1.coalesce("seq-key".to_string(), || async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            kairos_rs::services::inflight::CoalesceOutcome::ok(
                200,
                vec![],
                bytes::Bytes::from_static(b"v1"),
            )
        })
        .await
    });
    let h2 = tokio::spawn(async move {
        t2.coalesce("seq-key".to_string(), || async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            kairos_rs::services::inflight::CoalesceOutcome::ok(
                200,
                vec![],
                bytes::Bytes::from_static(b"v1"),
            )
        })
        .await
    });
    let _ = tokio::join!(h1, h2);

    let after_wave1 = tracker.primary_calls();
    assert_eq!(
        after_wave1, 1,
        "wave 1 should have exactly 1 primary (the 2nd call is a waiter)"
    );
    assert_eq!(tracker.waiters(), 1, "wave 1 should have 1 waiter");

    // Wave 2: 2 concurrent calls. Entry was deregistered, so both
    // must become primaries. The 1st call is a new primary, the
    // 2nd call finds the 1st still in-flight and becomes its waiter.
    let t1 = Arc::clone(&tracker);
    let t2 = Arc::clone(&tracker);
    let h1 = tokio::spawn(async move {
        t1.coalesce("seq-key".to_string(), || async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            kairos_rs::services::inflight::CoalesceOutcome::ok(
                200,
                vec![],
                bytes::Bytes::from_static(b"v2"),
            )
        })
        .await
    });
    let h2 = tokio::spawn(async move {
        t2.coalesce("seq-key".to_string(), || async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            kairos_rs::services::inflight::CoalesceOutcome::ok(
                200,
                vec![],
                bytes::Bytes::from_static(b"v2"),
            )
        })
        .await
    });
    let _ = tokio::join!(h1, h2);

    // Wave 2 adds: 1 new primary (the 1st call of wave 2) + 1
    // waiter (the 2nd call of wave 2 piggybacks on it).
    assert_eq!(
        tracker.primary_calls() - after_wave1,
        1,
        "wave 2 should add exactly 1 new primary"
    );
    assert_eq!(
        tracker.waiters(),
        2,
        "total waiters should be 2 (1 from each wave)"
    );
}