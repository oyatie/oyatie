# Managed K8s SLA Observability — Audit Evidence Emission

## Status: Target-only until live ingestion Review/fix

SLA evidence emission is not implemented by this source-authority cleanup. The
current deterministic foundation can compute summaries from snapshots, but it
does not claim sealed production audit events, live collector evidence, or public
SLA proof.

## Future Evidence Events

When the live ingestion lane is implemented and reviewed, it should emit sealed
evidence for:

- accepted SLA observation snapshots, including source type, window bounds,
  freshness deadline, collector identity, and tenant/cluster scope;
- summary reads and evidence reads, including authorization scope and result
  freshness without leaking tenant secrets;
- burn-rate verdicts (`page`, `ticket`, or `none`) derived from the kernel alert
  verdict rather than recomputed in a web layer;
- stale/missing/disagreeing evidence holds used for rollout or rollback policy.

Until that lane lands, this file is a target contract and must not be cited as
measured production evidence.
