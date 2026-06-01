---
id: ADR-0094
status: Accepted
doc_status: published
---

# ADR-0094: `Handler` trait with associated `Error` type

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-architecture`
> **Related:** ADR-0090, ADR-0092

## Status

Accepted (2026-05-14).

## Context

The existing handler signature is `Arc<dyn Fn(HttpRequest) -> HttpResponse +
Send + Sync>` (alias `SyncHandler` in `oya-http-runtime-hyper-adapter`).
This forces handlers to:

- construct an `HttpResponse` for every error path inside the handler body,
- have no typed error vocabulary,
- have no shared error-rendering policy.

Real handlers want to return structured errors (`Err(NotFound)`,
`Err(PermissionDenied)`, `Err(ValidationError(detail))`) and have those
render uniformly to HTTP responses at the framework boundary.

## Decision

Add a typed `Handler` trait in `oya-http-middleware-kernel`:

```rust
pub trait Handler: Send + Sync {
    type Error: Into<HttpResponse>;
    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>;
}

pub fn call_into_response<H: Handler>(handler: &H, req: HttpRequest)
    -> HttpResponse
{
    match handler.call(req) {
        Ok(r) => r,
        Err(e) => e.into(),
    }
}
```

Plus a bridge helper in the adapter that wraps a typed `Handler` into the
closure-shaped `SyncHandler` the router holds today:

```rust
pub fn handler_to_sync<H>(handler: H) -> SyncHandler
where
    H: Handler + 'static,
{
    let handler = Arc::new(handler);
    Arc::new(move |req| call_into_response(handler.as_ref(), req))
}
```

The change is ADDITIVE — existing closure-based handlers continue to work
through the unchanged `SyncHandler` alias. Migration is per-handler at
each author's pace.

### Why no blanket impl?

A blanket `impl<F: Fn(HttpRequest) -> HttpResponse> Handler for F` would
conflict with a future blanket `impl<F: Fn(HttpRequest) -> Result<...>>`
under Rust's coherence rules. Explicit wrappers (`handler_to_sync`) are
clearer and avoid trait-ambiguity errors.

### Why `Into<HttpResponse>` and not a fixed `Box<dyn std::error::Error>`?

`Into<HttpResponse>` lets each handler's error vocabulary control its own
rendering. Authors implement `From<MyError> for HttpResponse` once per
error type; the conversion is type-driven. A boxed-error variant would
lose this type-control benefit and force runtime downcasting.

## Adversarial fixtures (F3)

```text
oya_http_middleware_kernel::tests::handler_ok_path_returns_response
oya_http_middleware_kernel::tests::handler_err_path_renders_via_into_response_not_found
oya_http_middleware_kernel::tests::handler_err_path_renders_via_into_response_bad_input
oya_http_middleware_kernel::tests::handler_err_path_renders_via_into_response_internal
oya_http_middleware_kernel::tests::handler_err_variants_render_to_distinct_responses
oya_http_runtime_hyper_adapter::tests::handler_to_sync_routes_ok_and_err_paths
```

The last test routes a typed `Handler` through the router via
`handler_to_sync` and asserts the rendered response on both Ok and Err
paths.

## Future: async variant

`F-HANDLER-ASYNC`: introduce `trait AsyncHandler { type Future:
Future<Output = Result<HttpResponse, Self::Error>>; type Error:
Into<HttpResponse>; fn call(&self, req) -> Self::Future; }` once the
async-chain refactor (`F-ASYNCCHAIN-1`) lands. Until then the sync
variant covers existing handlers.

## Consequences

### Positive

- Typed error vocabulary; one place to render each error class.
- Additive; no breaking change.
- `From<HyperRuntimeError> for HttpResponse` (added in Phase 8) uses the
  same `Into<HttpResponse>` pattern — consistency across error types.

### Negative

- Two ways to define a handler (closure + Handler trait); style guide
  needs to point one.

## References

- ADR-0092 (D11 covers this within the seam policy)
- `docs/standards/multispectrum-review.md` F-MULTI-Q4 quality finding
- FixupTask F-HANDLER-ASYNC
