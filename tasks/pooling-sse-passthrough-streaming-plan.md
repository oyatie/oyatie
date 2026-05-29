# Plan: pooling-sse-passthrough-streaming

## Objective

Implement true byte-passthrough streaming dispatch (`dispatch_stream`) in the
`HyperProviderInvocationTransport`, replacing the existing honest-boundary stub.
The stream MUST never buffer/parse/log the SSE body — bytes flow from the
upstream TCP socket directly to the caller.

## Edge Cases

1. **Terminal SSE event (message_stop / response.completed + usage frame)**: The
   upstream closes the connection after the final usage frame. hyper-util's
   legacy client accumulates a "leftover buffer" between the last content chunk
   and the FIN. Without explicit EOF-flush the terminal frame is dropped.
   Mitigation: consume the stream until `None` before signaling end-of-stream,
   emitting any leftover bytes as a final chunk.

2. **First-byte failure (non-2xx status before any data)**: If the upstream
   returns a non-2xx status, the dispatch loop must be able to decide failover
   vs short-circuit. The hyper response header (status line) is available before
   body bytes arrive. Classify at the header level (same as `do_dispatch`):
   5xx → `Err(Retryable)` as first item, 4xx → `Err(NonRetryable)`,
   2xx → stream body bytes.

3. **Mid-stream 5xx / connection reset**: After ≥1 body chunk has been yielded,
   a connection error surfaces as `Err(Retryable)` mid-stream. The dispatch loop
   MUST NOT retry (partial body already delivered).

4. **SeatLease (health record) lifetime**: The lease must be held for the full
   stream lifetime. If the caller drops the stream early (or the stream ends
   with an error), `record_success` is called only on clean end; `record_failure`
   is called on any error item yielded from the stream.

5. **5xx status with empty body**: Retryable error, no body bytes emitted.

6. **Empty SSE stream (status 200, no body)**: Yield zero chunks. Clean end.

7. **Credential injection**: The credential is resolved before the stream is
   opened, same as `do_dispatch`. The stream transport signature already accepts
   `ProviderCredential` as a parameter.

## Acceptance Criteria

- `HyperProviderInvocationTransport::dispatch_stream` performs a real hyper POST
  to `upstream_url(provider)` with auth headers from `credential` (no OpenBao
  resolution — credential is pre-resolved by the caller).
- Upstream 5xx before any body → first item is `Err(Retryable { .. })`.
- Upstream 4xx (not 429) before any body → first item is `Err(NonRetryable { .. })`.
- Upstream 2xx → body bytes are yielded chunk-by-chunk, byte-exact, never
  buffered, never parsed.
- Terminal SSE event is never dropped: leftover-buffer EOF flush emits any
  remaining bytes as a final chunk before stream end.
- Stream is `Send + 'static` (no borrows from `&self` across the yield boundary).
- SeatLease semantics: the dispatch loop's `dispatch_to_pool_stream` already
  manages `record_success` / `record_failure` from the yielded stream items.
  The transport itself is not responsible for health recording.
- HERMETIC TESTS: all tests use a local `127.0.0.1:0` in-process HTTP/1.1 mock
  server (same pattern as existing `spawn_test_server` in `src/lib.rs`).
  No real upstream calls in any test.

## Subtasks (ordered)

1. Write the `tasks/pooling-sse-passthrough-streaming-plan.md` (this file).
2. Write `docs/specs/task-pooling-sse-passthrough-streaming.md`.
3. Write hermetic tests in `src/lib.rs` (under `#[cfg(test)] mod tests`):
   - `hyper_transport_stream_200_passthrough_byte_exact`
   - `hyper_transport_stream_5xx_first_byte_retryable`
   - `hyper_transport_stream_4xx_first_byte_non_retryable`
   - `hyper_transport_stream_terminal_event_not_dropped` (leftover-buffer flush)
   - `hyper_transport_stream_empty_body_clean_end`
   Confirm they FAIL (transport is still the stub).
4. Implement `do_dispatch_stream` in `HyperProviderInvocationTransport`:
   - Build the hyper request identically to `do_dispatch` (POST, auth headers,
     content-type: application/json).
   - Await the response headers (status line available before body streaming).
   - On non-2xx: classify status → first item `Err(Retryable|NonRetryable)`,
     then stream ends.
   - On 2xx: yield body chunks from `hyper::body::Incoming` frame-by-frame.
   - EOF flush: after `poll_frame` returns `None`, emit any buffered-but-not-yet-
     yielded bytes as a final chunk (leftover-buffer pattern).
   - Wrap in `Pin<Box<dyn Stream<Item=Result<Bytes,TransportError>> + Send>>`.
5. Wire `dispatch_stream` trait method to `do_dispatch_stream`.
6. Run `cargo check -p oya-intelligence-provider-pool-app --all-targets` → green.
7. Run `cargo nextest run -p oya-intelligence-provider-pool-app` → green.
8. Self-review: correctness / security / perf / cloud-native-readiness.
9. Simplify: guard clauses, dead code removal, naming.
10. Final green nextest run + commit.

## K8s / Cloud-native implications

- The stream is stateless — no per-stream state outside the `HyperProviderInvocationTransport`
  struct. Pod restarts are safe.
- The process-wide `OnceLock<hyper_util::client::legacy::Client>` is unchanged.
  Connection pooling is inherited.
- No new external deps — uses existing `hyper`, `hyper-util`, `http-body-util`,
  `bytes`, `futures-util` workspace deps.
- SLO impact: the SSE latency (TTFT) SLO in the intelligence microservice's
  `slos/ttft.openslo.yaml` is now on a real byte-passthrough path, not a stub.
