# ADR-0090: Hyper canonical HTTP backbone (LTS-exception for hyper 1.x)

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-architecture`
> **Supersedes:** —
> **Superseded-by:** —
> **Related:** ADR-0064

---

## Status

Accepted (2026-05-14).

## Context

The user issued a workspace-wide directive on 2026-05-14:

> "Hyper for framework. Let's switch from axum to hyper everywhere — that's
> our backbone. Fits our 'support everything ourselves with 0 to minimal
> dependency' principle. We build our libraries as we build. Make sure to
> plan it out as building blocks are foundations."

Prior REST scaffolds had referenced axum/router; that wording is retired.
This ADR captures the chosen HTTP backbone + the LTS-exception that the
LTS-dependency-enforcement directive (2026-05-12) requires for any new
direct dependency.

## Decision

Hyper 1.x is the canonical HTTP backbone for every µservice in the
workspace. Direct deps for the runtime composition layer:

| Crate            | Version pin | Purpose                                |
| ---------------- | ----------- | -------------------------------------- |
| `hyper`          | `1`         | bare HTTP I/O protocol (h1/h2)         |
| `hyper-util`     | `0.1`       | tokio + auto-builder connection helper |
| `tokio`          | `1`         | async runtime (rt-multi-thread + net)  |
| `http-body-util` | `0.1`       | `Full<Bytes>` body collector           |
| `bytes`          | `1`         | byte buffer type used by hyper bodies  |

These five crates are the ONLY external dependencies introduced by the
hyper backbone. Their transitive cost is consciously accepted; alternative
HTTP stacks (axum, actix, poem) bring substantially larger trees.

### Building-block layering (Foundation)

```
Layer 5: oya-http-runtime-hyper-adapter   ← the ONLY crate importing hyper
Layer 4: oya-http-{cedar,tenant,telemetry,deadline}-middleware-domain
Layer 3: oya-http-sse-domain               ← std-only
Layer 2: oya-http-middleware-kernel        ← std-only
Layer 1: oya-http-router-kernel            ← std-only
Layer 0: hyper + tokio + hyper-util + http-body-util + bytes
```

Layers 1–4 stay std-only; only Layer 5 binds to hyper. Per-cell binaries
(Layer 6, e.g. `oya-ops-workspace-shell-runtime`) consume Layer 5 + the
BC-specific rest/application crates.

### Version-pin verification

These pins are caret-major-minor placeholders. Before each merge that
bumps hyper/tokio/hyper-util/http-body-util/bytes, the LTS-pin verifier
(`oya-foundry-fitness-lts-dependency` lane, ADR-0064 §5) cross-checks
against the current upstream LTS major.minor.

## Drivers

1. **0-to-minimal-dependency** — hyper is the bare HTTP I/O protocol; we
   build router, middleware, SSE framing, and cedar gating ourselves
   instead of pulling axum + tower + tower-http.
2. **Support everything ourselves** — every behavior above the I/O layer
   lives in a crate we author, can audit, and can change without upstream
   coordination.
3. **Single backbone** — exactly one HTTP framework workspace-wide makes
   security audits, perf tuning, and observability instrumentation
   tractable.
4. **No axum** — axum (and the tower-http ecosystem) brings a
   substantially larger transitive tree (tower, http, http-body, hyper,
   pin-project-lite, …) and a programming model that competes with our
   own router-kernel + middleware-kernel; axum stays banned per the
   user-issued directive.

## Alternatives Considered

- **axum 0.8** — most idiomatic for kernel/adapter pattern; mature SSE;
  but `tower-http` transitives violate 0-to-minimal-deps. Rejected.
- **poem** — has first-class OpenAPI codegen but bundles a tracing /
  metrics stack we'd rather own. Rejected.
- **Pure stdlib `std::net::TcpListener` + hand-rolled HTTP parser** —
  reinvents hyper without any of its perf or HTTP/2 support. Rejected
  as over-engineering of the "support ourselves" principle.

## Consequences

### Positive

- Single HTTP backbone everywhere; no per-µservice framework drift.
- Layers 1–3 (router, middleware, sse) stay std-only, ship before any
  hyper dep lands, and are trivially testable.
- Layer 5 is the only "hot zone" for hyper version churn.

### Negative

- Adopting hyper means accepting tokio as the async runtime — there is
  no realistic alternative for an async HTTP server in Rust today.
- We own all middleware (cedar, tenant, telemetry, deadline, …). That's
  more code to maintain than a tower-http ecosystem dep would supply.

## Follow-ups

1. Author `oya-http-runtime-hyper-adapter` (Layer 5) as the only
   hyper-touching crate. (slice K'.4 in progress)
2. Author per-cell binaries (e.g. `oya-ops-workspace-shell-runtime`).
   (slice L)
3. Author Phase B middlewares (cedar / tenant / telemetry / deadline) on
   Layer 4. (slice K'')
4. Wire `oya-foundry-fitness-lts-dependency` to cross-check hyper /
   tokio / hyper-util pins on every merge.
