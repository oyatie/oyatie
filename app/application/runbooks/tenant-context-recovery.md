---
doc_class: Runbook
title: Tenant-Context Recovery — restore tenant resolution
microservice: application
severity: "Sev-1 (availability)"
status: Accepted
owner_team: axis-application + tenancy axis + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-03)
  - microservices/application/incident-response.md
  - microservices/application/policy/route-isolation.md
doc_status: published
---

# Runbook: Tenant-Context Recovery

## Trigger

`oya_application_tenant_context_resolve_fail_total > 0` for ≥ 30 s, OR
downstream µservice (tenancy, ontology, workflow) reports tenant-claim
mismatch rejection at elevated rate.

## Severity

**Sev-1** — Application Shell fails closed (tenant unknown → refuse
serve); every product surface unreachable for affected pack.

## Pre-checks

1. Confirm scope of failure: `oya_application_tenant_context_resolve_fail_total` by `pack` label — is it one pack or all?
2. Identify root cause candidate:
   - OpenBao reachable from Application cluster? (curl health endpoint)
   - Tenancy µservice resolver reachable? (curl tenancy health)
   - tenant-context middleware container healthy? (kubectl logs)
   - JWT verification key (JWKS) rotated unexpectedly? (compare cached key id vs. current)
3. Identify magnitude: how many sign-in attempts failing / sec?

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC + axis-application SME + tenancy SME | ≤ 5 min |
| 2 | Pre-checks | ≤ 2 min |
| 3 | If OpenBao outage: engage cloud-secrets on-call; switch tenant-context middleware to read-only failover cache (last-known-good tenant map; serves stale data but allows existing sessions to function) | ≤ 5 min |
| 4 | If tenancy resolver outage: engage tenancy on-call; tenant-context falls back to OpenBao-side cache | ≤ 5 min |
| 5 | If JWKS rotation race: pin previous JWKS for grace period via `cargo run -p oya-dev-cli -- application auth jwks-pin --kid <prev-kid>` (10-min override) | ≤ 2 min |
| 6 | If middleware crashloop: kubectl rollout restart; verify deployment recovers | ≤ 5 min |
| 7 | Monitor `oya_application_tenant_context_resolve_fail_total` returning to 0 | ≤ 10 min |
| 8 | CommsLead: status page update; tenant comms | ≤ 30 min |
| 9 | Postmortem within 5 BDs | – |

## Fail-closed posture (the normal behavior)

When tenant cannot be resolved, the Application Shell:
- Returns 503 to the request.
- Does NOT serve a generic shell (no information leak).
- Does NOT make downstream calls (no risk of cross-tenant query).
- Emits audit event `TenantContextResolveFailure` with correlation_id.

This is correct fail-closed behavior. The recovery procedure restores the
ability to resolve, not the ability to bypass.

## Verification

- `oya_application_tenant_context_resolve_fail_total == 0` for ≥ 5 min.
- Sign-in synthetic probe succeeds end-to-end.
- Downstream µservices report normal tenant-claim acceptance rate.
- Audit-chain contains the recovery event.

## Post-incident updates

- Postmortem: usually identifies a single dependency single-point-of-failure
  → action item to either remove the SPOF or document accepted risk.
- Common findings: OpenBao single-region (we run per-pack OpenBao already);
  tenancy resolver cold-cache (action: pre-warm).

## References

- `failure-modes.md` FM-03.
- `policy/route-isolation.md` (default-deny posture).
- ADR-0123 (cross-product auth).
