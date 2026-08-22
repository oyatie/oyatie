---
doc_class: Runbook
title: JWKS Fetch Failure (Workload-Identity Validation)
status: Proposed
date: 2026-05-26
microservice: identity
bounded_context: workload-identity
severity: sev1
audience: security-engineer
owner_team: axis-identity + ops-security
related_adrs: [ADR-0002, ADR-0162]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Runbook: JWKS Fetch Failure

## Operator Contract
- Runbook id: `identity-workload-jwks-fetch-failure`.
- Service namespace: `identity`; bounded context `workload-identity`.
- Owning rotation: PagerDuty `identity-primary`; `ops-security` secondary.
- Incident channel: `#inc-identity-security`.
- Audit event class: `EVT-IDENTITY-WORKLOAD-JWKS_FETCH-INCIDENT` (ADR-0162 fields
  `incident_id`, `tenant_id`, `trust_domain`, `cell_id`, `runbook_id`,
  `decision_id`, `evidence_hash`, `operator_id`).
- Stop condition: validation availability green for 30 minutes, last-known-good
  cache repopulated from a fresh fetch, and no tenant is in hard-deny.
- Safety invariant: **never** relax the fail-closed posture to "skip signature
  verification" or "trust expired keys" to restore availability. A JWKS outage is
  a hard-deny by design (brief §10); availability is restored by fixing the fetch,
  not by weakening validation.

## Background (why this is fail-closed)

Per the cited brief (§10): on JWKS fetch failure with a **valid cache present**,
the validator serves from the in-memory last-known-good keys — no incident. The
incident is the **empty/expired cache** case: validation hard-denies
(`503 jwks-unavailable`, failure mode F2 in `design/failure-modes.md`). This
burns the `validation-availability` SLO **intentionally** so the outage is
visible, not hidden.

## Trigger Conditions
- Page on `IdentityWorkloadJwksUnavailable` when
  `sum(rate(identity_workload_validate_request_total{failure="jwks-unavailable"}[5m])) > 0`
  for 5 minutes in any production cell.
- Page on `IdentityWorkloadValidationAvailBurn` when the
  `identity-workload-validation-availability` SLO burn rate ≥ 14.4x for 1h.
- Sev0 if more than one trust domain reports `jwks-unavailable` simultaneously
  (fleet-wide issuer or network failure).

## Symptoms
- PEPs see `503` from `/tokens/validate` and `/authorize-with-token`; downstream
  calls fail closed (denied), so dependent µservices report authz denials, not
  data corruption.
- Metric pattern: `identity_workload_jwks_cache_age_seconds` exceeds the
  cache TTL while `identity_workload_jwks_fetch_error_total` climbs.
- Log signature `decision=deny reason=jwks-unavailable` with `trust_domain` set
  means the fail-closed path is working as designed — confirm cache state before
  assuming a bug.

## Diagnostic Steps
1. Set incident vars: `export INCIDENT_ID=INC-identity-workload-jwks-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-eu-frankfurt-1; export TD=acme.oyatie.dev`.
2. Confirm which trust domains are affected: query
   `identity_workload_validate_request_total{failure="jwks-unavailable"}` by `trust_domain`.
3. Check cache age vs TTL: query `identity_workload_jwks_cache_age_seconds{trust_domain="$TD"}`.
4. Check fetch errors: query `identity_workload_jwks_fetch_error_total{trust_domain="$TD"}`.
5. Verify the upstream issuer JWKS endpoint is reachable from the cell network
   (DNS, TLS, HTTP status) for the affected trust domain.
6. Confirm whether the failure is fetch (network/DNS/TLS) or cache-expiry
   (cache aged out and no successful refresh) — they have different fixes.
7. Verify the `jku`/`x5u` allowlist did not recently change to exclude the issuer
   (a misconfigured allowlist looks like a fetch failure).
8. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice identity --runbook identity-workload-jwks-fetch-failure`.

### Decision tree
```text
1. Is the cache age below TTL (valid cache present)?
   |-- yes: this is NOT the incident path; validation should be serving. Investigate why hard-deny fired (bug).
   |-- no: cache empty/expired -> hard-deny is correct; restore the fetch (continue).
2. Is the upstream issuer JWKS endpoint reachable from the cell?
   |-- no: network/DNS/TLS branch (A).
   |-- yes: allowlist or issuer-content branch (B).
3. Did the jku/x5u allowlist or trust-domain->JWKS map change recently?
   |-- yes: config-regression branch (C) -> roll back the map.
   |-- no: escalate to upstream issuer owner.
```

## Mitigation Steps
1. Acknowledge page; open the bridge in `#inc-identity-security`.
2. Branch A (network/DNS/TLS): restore connectivity to the issuer JWKS endpoint;
   confirm a fresh fetch repopulates the last-known-good cache.
3. Branch B (issuer content / key rotation): confirm the issuer published a valid
   key set; if the issuer rotated keys without overlap, coordinate with the
   issuer owner — do NOT disable verification.
4. Branch C (config regression): roll back the trust-domain→JWKS map / `jku`
   allowlist change that excluded the issuer.
5. Do NOT widen the algorithm allowlist, disable skew, or accept `alg:none` to
   "get traffic flowing" — these reintroduce the #1 vuln class (brief §5).
6. If a single tenant/trust-domain is affected, quarantine its blast radius;
   other tenants keep serving from their own caches.
7. Emit mitigation audit: `oya audit-chain emit --event-class EVT-IDENTITY-WORKLOAD-JWKS_FETCH-INCIDENT --incident $INCIDENT_ID --field mitigation=active`.

## Resolution Steps
1. Confirm a successful JWKS fetch repopulated the cache for every affected
   trust domain (`identity_workload_jwks_cache_age_seconds` resets to ~0).
2. Confirm `identity_workload_validate_request_total{failure="jwks-unavailable"}`
   returns to zero for 3 consecutive 10-minute windows.
3. Confirm the validation-availability SLO burn rate is back below 1x.
4. If config-driven, add a regression test asserting the trust-domain→JWKS map
   includes the issuer and the proactive-refresh schedule is shorter than the
   cache TTL (so the empty-cache state is not reachable in steady state).
5. Seal resolution: `oya audit-chain emit ... --field resolution=complete`; verify the seal.

## Verification Checklist
- No trust domain is in `jwks-unavailable` hard-deny.
- Last-known-good cache age < TTL for all affected trust domains.
- Proactive refresh interval < cache TTL (steady-state cannot reach empty cache).
- Fail-closed posture was preserved throughout (no verification weakened).
- `EVT-IDENTITY-WORKLOAD-JWKS_FETCH-INCIDENT` has sealed mitigation + resolution rows.

## Escalation Path
- Primary: `identity-primary`; security secondary `ops-security-primary`.
- Upstream issuer owner: engage when the issuer JWKS endpoint or key rotation is
  the root cause and local network is proven healthy.
- Architecture: page `council-architecture-reviewer` before any proposal to relax
  the fail-closed posture (such a proposal should be refused).

## References
Brief §10 (JWKS resilience, fail-closed, key-set cap); RFC 8725 §3.1, §3.10;
`design/failure-modes.md` F1–F4, F13; `design/operational-boundaries.md`.
