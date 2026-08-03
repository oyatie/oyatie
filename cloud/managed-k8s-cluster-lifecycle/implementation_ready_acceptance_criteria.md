# managed-k8s-cluster-lifecycle — Implementation-Ready Acceptance Criteria

ADR-0376 design-spec-maturity surface (authored 2026-05-28 with the managed-k8s 4-split).

## Acceptance criteria
- The deterministic kernel + port + in-memory adapter surface for this microservice is implementation-ready under ADR-0376.
- Live-integration legs (Kamaji / kube-rs / billing / SLA emission, as applicable) are honest-deferred per ADR-0376-D3 and registered in `registry/placeholder-debt/adr-follow-ups.yaml`.
- Cross-microservice ports remain narrow + typed; no out-of-band integration paths.

## t_c3231b0f implementation evidence

- `oya-managed-k8s-cluster-lifecycle-api` now exposes a service-local `InMemoryOperationLedger` plus create/update/scale/delete operation request/record contracts carrying tenant, account, project, region, cell, resource-group, ORN, idempotency key, quota decision, control-plane action, audit id/events, and deterministic SLO-evidence fields.
- Create operation semantics record a pending ledger entry before quota/backend work, call `ControlPlaneProvisioning::provision` only after quota allow, and record deny/not-found/persistence failures without invoking the backend.
- Delete operation semantics map to `ControlPlaneProvisioning::teardown`, replay duplicate idempotency keys without a second teardown, and preserve create/delete ledger records rather than erasing registry/ledger state on request receipt.
- Update and scale operation semantics are honest-deferred/held where the current `ControlPlaneProvisioning` port has no live update/scale method; scale-up checks quota and scale-down enforces drain floors/stale-observation hold before any release semantics.
- Claim ceiling remains deterministic dogfood/design foundation only: no live Kubernetes/provider action, no measured production SLO, no billing/DPIA/public-SLA/external-GA claim, no broad tenant-quota implementation, and no generated JSON hand edit.
