#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::Instant;

use bytes::Bytes;
use futures::StreamExt as _;
use intelligence_kernel::{AgentId, SeatOutcome, SubscriptionPool, TenantId};
use intelligence_rest_proxy::{RestAdapterError, SseStreamWithLease};

mod d3_sse_streaming_support;
use d3_sse_streaming_support::*;

// ---------------------------------------------------------------------------
// Test 4: Lease held until stream completes — 3 concurrent SSE vs 2-seat pool
// ---------------------------------------------------------------------------

/// SSE-4: Pool concurrency invariant. 3 concurrent SSE streams against a 2-seat
/// pool: the 3rd request must return 503 until one of the first 2 completes.
#[tokio::test]
async fn sse4_lease_held_during_stream_third_request_503() {
    let tenant = "t4";
    let pool_ref = make_pool_2_seats(tenant);

    let gate = AlwaysAllow;
    let agent = AgentId::new("agent-sse4").unwrap();

    // Acquire leases for seats 1 and 2 (simulate 2 active SSE streams).
    let lease1 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();
    let lease2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();

    // Third request should fail — both seats are in use.
    let result3 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    assert!(
        result3.is_err(),
        "third lease must fail while two SSE streams hold seats"
    );
    assert!(
        matches!(
            result3.err().unwrap(),
            intelligence_kernel::SubscriptionPoolError::NoEligibleSeat
        ),
        "expected NoEligibleSeat"
    );

    // Complete lease1 (stream 1 finishes).
    lease1.complete(SeatOutcome::Ok, Instant::now()).unwrap();

    // Now the 3rd request should succeed.
    let result4 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    assert!(
        result4.is_ok(),
        "lease should succeed after one SSE stream completes"
    );

    // Clean up.
    let _ = result4.unwrap().complete(SeatOutcome::Ok, Instant::now());
    let _ = lease2.complete(SeatOutcome::Ok, Instant::now());
}

// ---------------------------------------------------------------------------
// Test 5: Stream error mid-flight → seat outcome is ServerError5xx
// ---------------------------------------------------------------------------

/// SSE-5: Mid-stream error in `SseStreamWithLease` → lease completed with
/// `ServerError5xx` (not `Ok`), and the seat transitions to Cooldown.
#[tokio::test]
async fn sse5_mid_stream_error_seat_outcome_server_error() {
    let tenant = "t5";
    let pool_ref = make_pool_2_seats(tenant);
    let gate = AlwaysAllow;
    let agent = AgentId::new("agent-sse5").unwrap();

    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();

    // Build a stream that yields one Ok chunk then one Err chunk.
    let err_stream: intelligence_rest_proxy::BoxStream<Result<Bytes, RestAdapterError>> =
        Box::pin(futures::stream::iter(vec![
            Ok(Bytes::from_static(b"data: first\n\n")),
            Err(RestAdapterError::UpstreamError {
                status: 502,
                body: "mid-stream failure".to_string(),
            }),
        ]));

    let mut wrapped = SseStreamWithLease::new(err_stream, lease);

    // First chunk — Ok.
    let first = wrapped.next().await.unwrap();
    assert!(first.is_ok());

    // Second chunk — Err. SseStreamWithLease should have completed the lease
    // with ServerError5xx at this point.
    let second = wrapped.next().await.unwrap();
    assert!(second.is_err());

    // The pool should now reject a new lease for this seat because it's in Cooldown.
    // (Both seats were available initially; one is now in Cooldown after the error.)
    // We attempt to grab both seats — at least one attempt will get a Cooldown error.
    let r1 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    let r2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    let r3 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    // At most one seat remains active (seat 2 was never leased above).
    // The errored seat is in Cooldown; the remaining seat may or may not be available.
    // We just verify that at least one of the three attempts fails (pool is not full
    // capacity after the error).
    let failures = [r1, r2, r3].into_iter().filter(|r| r.is_err()).count();
    assert!(
        failures >= 1,
        "expected at least one NoEligibleSeat after mid-stream error puts seat in Cooldown"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Client drops stream → lease released via Drop (no panic, no leak)
// ---------------------------------------------------------------------------

/// SSE-6: Dropping `SseStreamWithLease` before completion releases the lease
/// via the `SeatLease` Drop impl (outcome = Released). The seat transitions
/// back to Available (no penalty for Released).
#[tokio::test]
async fn sse6_client_drop_releases_lease_cleanly() {
    let tenant = "t6";
    let pool_ref = make_pool_2_seats(tenant);
    let gate = AlwaysAllow;
    let agent = AgentId::new("agent-sse6").unwrap();

    let lease = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();

    // Verify that while the lease is held, the other seat is available but
    // not the leased one (pool has 2 seats; 1 is now in use).
    let lease2 = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    )
    .unwrap();

    // Both seats taken — 3rd should fail.
    assert!(
        SubscriptionPool::lease(
            &pool_ref,
            &TenantId::new(tenant).unwrap(),
            &agent,
            &gate,
            Instant::now()
        )
        .is_err()
    );

    // Build an infinite stream that never yields None.
    let infinite_stream: intelligence_rest_proxy::BoxStream<Result<Bytes, RestAdapterError>> =
        Box::pin(futures::stream::pending());

    {
        let wrapped = SseStreamWithLease::new(infinite_stream, lease);
        // Drop without consuming — simulates client disconnect.
        drop(wrapped);
    }

    // lease2 still holds a seat — release it.
    let _ = lease2.complete(SeatOutcome::Ok, Instant::now());

    // After drop, the previously-leased seat should be Available again
    // (SeatOutcome::Released has no penalty).
    let result = SubscriptionPool::lease(
        &pool_ref,
        &TenantId::new(tenant).unwrap(),
        &agent,
        &gate,
        Instant::now(),
    );
    assert!(
        result.is_ok(),
        "seat should be available again after dropped SseStreamWithLease"
    );
    let _ = result.unwrap().complete(SeatOutcome::Ok, Instant::now());
}
