# `developer-sdk` µservice — SDK Engineer FAQ

20 real questions raised against the µservice that owns Oyatie's canonical multi-language SDK surface.

---

**Q1. Why generate SDKs from OpenAPI 3.2.0 instead of just using openapi-generator?**

We use openapi-generator under the hood **but** layer custom Rust + TS + Go preludes that bring Oyatie idioms (typed `Tenant`,
`OyatieError` enum, automatic `tracing::instrument`, Cedar bundle embedding). Pure openapi-generator output is unergonomic — it
matches the spec literally but doesn't feel hand-written. The custom prelude closes that gap.

---

**Q2. How do you keep 13 languages in sync?**

Three answers:
1. Single source of truth — `crates/oya-openapi-canonical-v<vintage>/spec/v1.yaml`. SDKs cannot drift unless the spec drifts.
2. Snapshot tests per language — any template change must update the snapshot, which is reviewed.
3. Canary apps per language — each language has a "hello world + 5 typical calls" canary running in CI; a regen that breaks any
   canary blocks promotion.

---

**Q3. What's the difference between an SDK release channel `dev` and `stable`?**

`dev` is auto-published on every green merge to `dev` branch — version is `0.0.0-dev-<git-sha>`.
`stable` is a manual (or reviewer-agent-gated automatic) promotion of a `dev` build that has been clean for ≥ 24 h (tenant_class paid) /
≥ 72 h (tenant_class paid) / ≥ 7 d (tenant_class demo_trial).

---

**Q4. Why is Rust the reference template?**

Because it's the most strictly-typed of the supported languages — any abstraction expressible in Rust generally translates cleanly
to Go/TS/Python/Swift/etc. Reference templates start in Rust and are then ported to other languages with snapshot tests as the contract.

---

**Q5. Is gRPC required, or is REST sufficient?**

REST (HTTP+JSON) is mandatory for every API. gRPC is mandatory for any API marked `streaming: true` in the canonical spec.
Per-API teams choose; the generator emits both transports when both are declared.

---

**Q6. How does HTTP/3 fallback work?**

The Rust SDK uses `hyper` with `h3` and prefers QUIC; falls back to HTTP/2 if Alt-Svc isn't advertised or QUIC handshake fails.
The Go SDK uses `quic-go/quic-http3` (Go 1.23+); the TS SDK uses `undici` (Node 22+). The choice is observed in the
`x-oya-protocol` response header for debugging.

---

**Q7. How does the Cedar bundle work client-side?**

Each SDK ships a compiled Cedar policy bundle as a Rust `&'static [u8]` (or equivalent). The bundle is keyed by tenant + tier and
loaded into a client-side evaluator. Before sending a request, the SDK runs a local Cedar permit check; failure short-circuits
with `OyatieError::PermitDenied` instead of round-tripping. The server still re-evaluates Cedar authoritatively — the client check
is an optimization, not the gate.

---

**Q8. What about WASM SDKs?**

Available at tenant_class paid via `oya-canonical-sdk` compiled to WebAssembly with `wasm-bindgen`. The WASM build deliberately drops
gRPC streaming (browser limitation) and falls back to long-polling. Service Worker integration patterns are in
`microservices/developer-sdk/tutorials/wasm-sdk-in-service-worker.md`.

---

**Q9. How is the OpenAPI spec authored?**

Per-µservice teams maintain their slice of the spec in their repo (`crates/<msvc>-api/openapi/v1.yaml`). The `developer-sdk` µservice's
spec aggregator combines all slices into the canonical bundle, runs spec lints, then publishes the bundle. The aggregator runs on
every merge to `dev`.

---

**Q10. What's the LTS policy?**

Per ADR-0220 §C-3:
- tenant_class demo_trial SDKs: 12 mo LTS per minor.
- tenant_class paid: 18 mo.
- tenant_class paid: 24 mo.
- compliance_pack-bound paid: 36 mo LTS branch with security backports.

Past LTS: SDK no longer compiles against current API but old artifacts remain on registries forever.

---

**Q11. How do we handle breaking spec changes?**

ADR amendment + 6-mo deprecation window. The SDK emits `Deprecated`-tagged calls with a `removal_at: <date>` and a migration link.
The lean-a10 lane enforces this — a breaking change without ADR is blocked.

---

**Q12. Can a tenant ship a custom SDK fork?**

compliance_pack-bound paid only. The tenant provides their KMS-backed signing key; the generator forks the canonical spec with the tenant's pack
overlay applied, generates SDKs, signs with the tenant's key, and publishes to the tenant's private registry. The fork rebases
nightly; non-trivial drift is paged.

---

**Q13. How is auth handled across SDKs?**

A single `Credentials` abstraction with adapters:
- `Credentials::api_key(...)` — long-lived bearer.
- `Credentials::oauth2(...)` — OAuth 2.0 + PKCE flow.
- `Credentials::workload(...)` — auto-detect K8s SA / AWS IAM / GCP SA / Azure MI.
- `Credentials::mtls(cert, key)` — mTLS only.
- `Credentials::fido2(challenge)` — hardware token assertion (tenant_class paid).
- `Credentials::mls_group_member(...)` — MLS group membership (messenger µservice integration).

Every SDK exposes the same shape.

---

**Q14. How is retry done?**

Default: idempotent `GET`/`HEAD`/`PUT`/`DELETE` calls retry up to 3 times with exponential backoff + jitter; `POST`/`PATCH` retry
only if the response has `x-oya-idempotent: safe` and an `Idempotency-Key` header was set. Custom retry policies via `RetryPolicy::custom(...)`.

---

**Q15. Where do telemetry counters live?**

Each SDK has a `Metrics` impl that emits Prometheus-shaped counters on every call:
- `oya_sdk_calls_total{method, route, outcome, tenant}`
- `oya_sdk_latency_seconds{method, route, tenant}` (histogram)
- `oya_sdk_retries_total{method, route, tenant}`
- `oya_sdk_permit_denied_total{action, tenant}`

Tenant cardinality is high — only emit `tenant` to a tenant-scoped collector.

---

**Q16. Does each SDK ship a CLI?**

Yes — every SDK ships a thin CLI (`oya` for Rust, `oyactl` for Go, etc) that wraps the SDK. Same Cedar permits; same retry behavior;
same auth surface. The CLI is the most-used surface in dev environments.

---

**Q17. What about WebSockets?**

For streaming we prefer gRPC; for legacy WebSocket needs (e.g. browser chat) the SDK exposes a typed wrapper that uses
WebTransport (HTTP/3) where supported and WebSocket-over-HTTP/2 elsewhere.

---

**Q18. How are SDK breaking bugs handled in published versions?**

Yank + republish patch. We have an SDK security incident runbook (`microservices/developer-sdk/runbooks/yank-republish.md`). The
publish hook can yank within ≤ 10 min of detection. Yank also propagates to the public attestations site.

---

**Q19. How do we handle the "long tail" language?**

tenant_class demo_trial covers Rust/TS/Python; tenant_class paid adds Go/Ruby/Java/Kotlin; tenant_class paid adds Swift/ObjC/C#/C++/Dart/Flutter/PHP; compliance_pack-bound paid adds
Elixir/Erlang/Clojure/Scala/Haskell/OCaml/Zig/Crystal/Nim. Beyond that, we use the gRPC protobuf as the source and let the customer
hand-build. No promises for languages outside the matrix.

---

**Q20. How are SDK examples kept in sync with API behavior?**

Every example in `microservices/<msvc>/reference-implementations/` is compiled + run as part of the canary suite. Examples that
fail compile or fail runtime block the promote-to-stable gate.
