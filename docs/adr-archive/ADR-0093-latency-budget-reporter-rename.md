---
id: ADR-0093
status: Superseded
superseded_by: [ADR-704]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0093: DeadlineMiddleware → LatencyBudgetReporter (honest naming)

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-architecture`
> **Related:** ADR-0092

## Status

Accepted (2026-05-14).

## Context

`oya-http-deadline-middleware-domain::DeadlineMiddleware` measured request
latency and replaced slow responses with HTTP 504. The name implied real
deadline enforcement (cancellation). The middleware-kernel chain is sync
and CANNOT cancel an in-flight handler; the slow work runs to completion
and the 504 is post-hoc.

This conflicts with the multispectrum F5 quality bar: type names that lie
about behavior. A reader sees "Deadline" and reasonably expects the work
to abort when the budget is exceeded. The doc-comment was honest; the name
was not.

## Decision

- Rename type `DeadlineMiddleware` → `LatencyBudgetReporter` in
  `oya-http-deadline-middleware-domain` (also renamed; see below).
- Rename crate `oya-http-deadline-middleware-domain` →
  `oya-http-latency-budget-middleware-infrastructure` (also picks up the
  middleware-infrastructure layer rename from ADR-0092 D3).
- Rename associated identifiers:
  - `DEADLINE_EXCEEDED_BODY_PREFIX` → `LATENCY_BUDGET_EXCEEDED_BODY_PREFIX`.
  - Headers `x-deadline-budget-ms` / `x-deadline-elapsed-ms` →
    `x-latency-budget-ms` / `x-latency-elapsed-ms`.
  - Error body string `"deadline-exceeded"` → `"latency-budget-exceeded"`.
- Add adversarial test
  `slow_handler_runs_before_504_overwrite_proves_post_hoc_semantics`
  proving the side-effects-still-happen behavior — the name now matches
  what the test asserts.

## Future async-chain variant

When `F-ASYNCCHAIN-1` lands (async middleware chain), introduce a separate
`DeadlineMiddleware` (real cancellation) ALONGSIDE the existing
`LatencyBudgetReporter`. Composition-root binaries pick whichever fits
their cell. Until then, calling something "Deadline" remains banned.

## Consequences

### Positive

- Reader trust: names match behavior.
- The honesty test (side effects run before 504) is documentation-as-test.
- No silent failure mode: a future async chain doesn't have to fight a
  stale "Deadline" name.

### Negative

- One rename in a downstream cell binary (none today; tracked).
- Extra ADR in the citation graph.

## References

- ADR-0092 (D5 latency-budget rename within the seam plan)
- FixupTask F-ASYNCCHAIN-1
