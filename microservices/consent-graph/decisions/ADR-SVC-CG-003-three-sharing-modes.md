# ADR-SVC-CG-003: Three sharing modes — Projection / Aggregate / AttestedQuery

- Status: Accepted
- Scope: service
- Date: 2026-05-18
- Authority: ADR-0214 §2.2, IP-009/010/011.

## Context

Cross-tenant visibility use cases span: real-time row-level (manufacturer→retailer inventory), privacy-
preserving (marketplace cohort stats), and on-demand attested (bank balance check). One mode does not
cover all three.

## Decision

Support three modes, selectable per `SharingTerms.mode`:

1. **Projection** — row-level real-time event stream. Pulsar topic per (grantor, grantee, entity).
   Field-level redaction. p99 ≤500ms latency. Default mode.
2. **Aggregate** — pre-aggregated buckets with k-anonymity (k≥5 default; ≥10 sensitive) and optional
   DP noise. Suppresses below-k buckets. p99 ≤window_size + 60s. For analytics use cases.
3. **AttestedQuery** — request/response; grantor evaluates query locally, returns signed answer.
   No projection stream. p99 ≤5s. For low-frequency, on-demand reads (e.g., bank balance).

Each mode has distinct Cedar action (`project.read`, `aggregate.read`, `attested.query`).

## Alternatives

- Only Projection (rejected: doesn't fit B2C aggregate analytics, doesn't fit attested queries).
- Add a fourth "PushNotification" mode (rejected: collapses into Projection with grantee-side filter).
- Per-vertical mode (rejected: cross-cutting; should be per-agreement).

## Consequences

- Three pipelines = three sets of tests + three sets of latency budgets.
- Aggregator requires DP noise generator (cryptographic PRNG per agreement).
- AttestedQuery requires a separate worker on grantor side.
- Five starter templates (per IP-002 §7) span the three modes.

## Verification

- IP-011 tests cover all three modes.
- E2E latency tests per mode against SLO.
- DP budget tests for Aggregate mode.
