# Spec: pooling-sse-passthrough-streaming

**Status**: in-progress  
**Crate**: `intelligence-provider-pool-app`  
**Lane**: pooling / priority: high / effort: L  
**ADR references**: ADR-0090 (http-stack), ADR-0083 (panic-free Tier 3), ADR-0105 (flat clean arch), ADR-0509 (single-crate per service)

---

## Objective

Implement `HyperProviderInvocationTransport::dispatch_stream` as a TRUE byte-passthrough
of the upstream SSE/chunked response body. The body bytes MUST:

- Flow chunk-by-chunk from the upstream TCP connection to the caller stream.
- NEVER be buffered in a `Vec<u8>` or collected in full before yielding.
- NEVER be parsed (no SSE framing, no JSON decode, no usage extraction).
- NEVER be logged (data_class: INTERNAL_ONLY; the body may contain PII/prompt content).

The prior implementation was an honest-boundary stub (single `Err(NonRetryable)`).
This slice replaces it end-to-end.

---

## Contract

### `ProviderInvocationTransport::dispatch_stream` (existing trait, unchanged)

```rust
fn dispatch_stream(
    &self,
    account_id: ProviderAccountId,
    provider: ProviderFamily,
    credential: ProviderCredential,   // pre-resolved by caller
    body: Bytes,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send + '_>>;
```

Stream item semantics (already specified in the trait doc, reproduced here for clarity):

| Position | Value | Meaning |
|----------|-------|---------|
| First item | `Err(Retryable)` | First-byte failure; dispatch loop MAY walk fallback chain |
| First item | `Err(NonRetryable)` | Terminal failure; short-circuit |
| First item | `Ok(chunk)` | Stream live; subsequent items forwarded verbatim |
| Later item | `Err(Retryable)` | Mid-stream failure; dispatch loop MUST NOT walk chain |
| Later item | `Err(NonRetryable)` | Mid-stream terminal; surface to caller |
| `None` (stream end) | — | Clean EOF; all bytes delivered |

---

## Mod layout (flat clean arch per ADR-0105 / ADR-0509)

All changes live in `src/lib.rs`. No new files, no new modules.

New private async function added to `HyperProviderInvocationTransport`:

```
impl HyperProviderInvocationTransport {
    async fn do_dispatch(...)           // existing — unchanged
    async fn do_dispatch_stream(...)    // new — SSE passthrough
}
```

`dispatch_stream` trait impl delegates to `do_dispatch_stream`.

---

## Implementation sketch

```
do_dispatch_stream(account_id, provider, credential, body):
  1. upstream_url(provider)?
  2. auth_headers(provider, &credential)?   // same as do_dispatch
  3. Build hyper POST request with body + auth headers + content-type
  4. client.request(req).await  → on network error: yield Err(Retryable), stream ends
  5. status = response.status()
     if 5xx → yield Err(Retryable { "upstream returned {status}" }), return
     if 4xx (≠ 429) → yield Err(NonRetryable { "upstream returned {status}" }), return
     if 429 → yield Err(Retryable { "upstream returned 429 (rate-limited)" }), return
     if 2xx → continue to body streaming
  6. body = response.into_body()   // hyper::body::Incoming
  7. Loop: poll body.frame().await
     - Frame::data(chunk) → yield Ok(chunk)
     - None (EOF) → flush any leftover bytes as final Ok(chunk) if non-empty, then end stream
     - transport error → yield Err(Retryable { .. }), end stream
```

The stream is wrapped in a `futures_util::stream::unfold` or manually pinned
async generator via `async_stream`-style `Box::pin(async_stream::stream! { ... })`.
Since `async_stream` is not a workspace dep, use `futures_util::stream::unfold`
or a hand-written `Stream` impl via `Pin<Box<dyn Stream<...> + Send>>` wrapping
an async block that yields into a channel / mpsc, OR use the simpler approach:
collect frames into a `Vec` and return `stream::iter` — BUT that would buffer the
full body (forbidden by spec).

Correct approach: use `async_stream::stream!` — but it's not a dep.

Use `futures_util::stream::try_unfold` or a custom pin-project struct, OR:
use `tokio::sync::mpsc` channel: spawn a task that pumps body frames into the
channel, return the receiver as a stream. This avoids the self-referential
async closure problem cleanly.

**Chosen approach**: `tokio::sync::mpsc::channel` + `tokio::spawn`. The spawned
task owns the `hyper::body::Incoming` and pumps frames; the channel receiver
implements `Stream<Item=Result<Bytes,TransportError>>` via `tokio_stream::wrappers::ReceiverStream`
— but `tokio_stream` may not be a dep. Alternative: use `futures_util::stream::unfold`
with state `(body, leftover)` and an async closure; this works because `unfold`
takes an `async fn(State) -> Option<(Item, State)>` which is `Send` when State is `Send`.

**Final chosen approach**: `futures_util::stream::unfold` with state
`Option<hyper::body::Incoming>`. Each call to the unfold closure polls one frame.
State transitions: `Some(body)` → `None` (EOF). Leftover-buffer flush is handled
by detecting EOF and emitting any accumulated leftover before returning `None`
from unfold.

Since `hyper::body::Incoming: !Unpin`, we need `Box::pin` the body before
passing to unfold, or use `pin_utils::pin_mut!`. Use `Box::pin` on the body.

State: `enum StreamState { Body(Pin<Box<Incoming>>), Done }` — avoids
wrapping in Option which loses the done signal.

---

## Testing strategy

All tests are hermetic: in-process HTTP/1.1 server on `127.0.0.1:0`.
Uses existing `spawn_test_server` helper (already in `src/lib.rs`) or a new
`spawn_sse_server` helper that sends chunked SSE data.

### Tests added to `src/lib.rs` `#[cfg(test)] mod tests`:

1. `hyper_transport_stream_200_passthrough_byte_exact`  
   Server sends three SSE frames as HTTP/1.1 chunked body. Assert collected bytes
   match exactly (no reordering, no parsing).

2. `hyper_transport_stream_5xx_first_byte_retryable`  
   Server returns 500 with no body. Assert first stream item is `Err(Retryable)`.

3. `hyper_transport_stream_4xx_first_byte_non_retryable`  
   Server returns 422 with JSON error body. Assert first item is `Err(NonRetryable)`.

4. `hyper_transport_stream_terminal_event_not_dropped`  
   Server sends a multi-chunk SSE body where the final chunk is a `usage` frame.
   Assert all bytes including the final chunk arrive in the collected stream output.

5. `hyper_transport_stream_empty_body_clean_end`  
   Server returns 200 with empty body. Assert stream yields zero `Ok` items and
   terminates cleanly (no error).

---

## Observability / SLO

The `microservices/intelligence/` microservice does not yet have dedicated SSE
streaming SLOs. The TTFT SLO (`slos/` not present under `microservices/intelligence/`)
is a follow-up (ADR-0130 mandates SLO authoring before promotion past dev). This
slice operates within the existing no-SLO-yet dev posture for the intelligence ms.

No new OTel spans are introduced in this slice (the existing `MetricsSink` port
records `Attempt`/`Success`/`Failure` at the `dispatch_to_pool_stream` level).

---

## Crate boundary

Changes are strictly inside:
- `intelligence/core/provider-pool-app/src/lib.rs`

No new workspace members. No edits to `Cargo.toml` (no new deps needed —
`hyper`, `hyper-util`, `http-body-util`, `bytes`, `futures-util`, `tokio` are
already declared). No edits to root `Cargo.toml`.

---

## Acceptance evidence

`cargo nextest run -p intelligence-provider-pool-app` — all tests green including
the five new hermetic stream tests.
