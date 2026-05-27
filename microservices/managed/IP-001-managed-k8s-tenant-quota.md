# IP-001: Managed K8s Tenant Quota

## Status: accepted

## Scope

Implements the per-tenant quota model and RBAC enforcement for managed K8s clusters
per ADR-0376. Crates: kernel, api, adapter-cedar, adapter-inmemory, app.

## Acceptance Criteria

- All 5 crates pass `cargo nextest run`.
- `evaluate()` is deterministic, total, panic-free (ADR-0083 Tier-3).
- Cedar RBAC default-deny enforced via `oya-identity-workload-authz-cedar-adapter`.
- REST API acceptance tests pass (35/35).
- Billing + audit chain emission: typed `Unimplemented` placeholders with
  `registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-billing-emission` entries.
