---
doc_class: Runbook
title: Policy Store Unavailable (Workload-Identity Authorization)
status: Proposed
date: 2026-05-26
microservice: identity
bounded_context: workload-identity
severity: sev1
audience: security-engineer
owner_team: axis-identity + ops-security
related_adrs: [ADR-0002, ADR-0162, ADR-0183]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Runbook: Policy Store Unavailable

## Operator Contract
- Runbook id: `identity-workload-policy-store-unavailable`.
- Service namespace: `identity`; bounded context `workload-identity`.
- Owning rotation: PagerDuty `identity-primary`; `ops-security` secondary.
- Incident channel: `#inc-identity-security`.
- Audit event class: `EVT-IDENTITY-WORKLOAD-POLICY_STORE-INCIDENT` (ADR-0162
  fields `incident_id`, `tenant_id`, `trust_domain`, `cell_id`, `runbook_id`,
  `decision_id`, `evidence_hash`, `operator_id`).
- Stop condition: authorize traffic served from the authoritative policy store
  again, decision-correctness SLO green for 30 minutes, and no tenant stuck in
  blanket default-deny.
- Safety invariant: **never** convert the fail-closed default-deny into a
  fail-open "allow while the store is down." A store outage is a default-deny by
  design (brief §10); restore the store, do not bypass it.

## Background (why default-deny, not allow)

Per the cited brief (§10): when the policy store is unavailable, the embedded
in-process Cedar renders a decision using the last-loaded policy set; if even
that cannot, the answer is **default-deny** (failure mode F8 in
`design/failure-modes.md`). Cedar's formally-proven default-deny (arXiv
2403.04651) means the *absence* of a reachable permit is a denial. The blast
radius of a store outage is therefore "legitimate workloads are temporarily
denied," never "everything is allowed."

## Trigger Conditions
- Page on `IdentityWorkloadPolicyStoreUnavailable` when
  `sum(rate(identity_workload_authorize_request_total{failure="policy-store-unavailable"}[5m])) > 0`
  for 5 minutes.
- Page on `IdentityWorkloadDecisionCorrectnessBurn` when the
  `identity-workload-decision-correctness` golden-set match drops below
  target (a correctness regression is treated as sev0 — a wrong decision is a
  security event).
- Sev0 if the golden corpus replay shows ANY decision flip (e.g. a previously-DENY
  tuple returning ALLOW) — this indicates a partial/stale policy load, which is
  worse than a clean outage.

## Symptoms
- PEPs see `503` from `/authorize` (and embedded-Cedar callers see local
  default-deny); dependent workloads report authorization denials.
- Metric `identity_workload_policy_store_reachable` reads 0 for the affected
  partition; `identity_workload_embedded_cedar_fallback_total` climbs.
- Log signature `decision=deny reason=policy-store-unavailable` is the correct
  fail-closed path; a `decision=allow` during an outage is an INCIDENT-WORSENING
  bug and must be escalated immediately.

## Diagnostic Steps
1. Set vars: `export INCIDENT_ID=INC-identity-workload-policystore-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-eu-frankfurt-1; export TD=acme.oyatie.dev`.
2. Identify affected partitions: query
   `identity_workload_authorize_request_total{failure="policy-store-unavailable"}` by `trust_domain`.
3. Check store reachability: `identity_workload_policy_store_reachable{trust_domain="$TD"}`.
4. Check embedded-Cedar fallback usage: `identity_workload_embedded_cedar_fallback_total{trust_domain="$TD"}`.
5. Run the golden corpus replay for the affected partition and compare against
   expected decisions — confirm NO decision flipped (especially no DENY→ALLOW).
6. Determine whether the embedded last-loaded policy set is current or stale
   (compare its content hash to the authoritative store's expected hash).
7. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice identity --runbook identity-workload-policy-store-unavailable`.

### Decision tree
```text
1. Is any decision flipping DENY->ALLOW during the outage?
   |-- yes: SEV0 immediately; halt embedded fallback for that partition (force clean default-deny); page council-architecture.
   |-- no: continue (clean fail-closed).
2. Is the embedded last-loaded policy set current?
   |-- yes: keep serving from embedded Cedar while restoring the store (bounded blast radius).
   |-- no (stale): prefer clean default-deny over a stale permit set; restore the store before re-enabling permits.
3. Is the store outage cell-local or fleet-wide?
   |-- cell-local: contain to the cell; other cells serve from their own store/embedded set.
   |-- fleet-wide: SEV0; engage store owner + architecture.
```

## Mitigation Steps
1. Acknowledge; open the bridge.
2. If the embedded last-loaded set is **current**, allow embedded-Cedar fallback
   to keep serving (legitimate workloads keep working; blast radius minimized).
3. If the embedded set is **stale**, force clean default-deny for that partition
   rather than risk a stale permit — denial is safe, a stale permit is not.
4. Restore the authoritative policy store (storage, network, or control-plane
   sync). Do NOT bypass it or hand-edit decisions.
5. **Never** flip to fail-open. If anyone proposes "allow while we fix it,"
   refuse and escalate to architecture.
6. Emit mitigation audit: `oya audit-chain emit --event-class EVT-IDENTITY-WORKLOAD-POLICY_STORE-INCIDENT --incident $INCIDENT_ID --field mitigation=active`.

## Resolution Steps
1. Confirm `identity_workload_policy_store_reachable` is 1 for all partitions.
2. Confirm `identity_workload_authorize_request_total{failure="policy-store-unavailable"}`
   returns to zero for 3 consecutive 10-minute windows.
3. Re-run the golden corpus; confirm 100% match and the decision-correctness SLO
   is back at target.
4. If the cause was stale-policy load, add a regression asserting the embedded
   set's content hash equals the authoritative hash before fallback is trusted.
5. Seal resolution and verify.

## Verification Checklist
- Policy store reachable for all partitions; embedded fallback usage back to zero.
- Golden corpus replay = 100% match; no decision flips remain.
- Fail-closed posture preserved throughout (no fail-open ever enabled).
- `EVT-IDENTITY-WORKLOAD-POLICY_STORE-INCIDENT` has sealed mitigation + resolution rows.

## Escalation Path
- Primary `identity-primary`; security secondary `ops-security-primary`.
- Policy store / control-plane owner: engage for the store outage root cause.
- Architecture: page `council-architecture-reviewer` for any DENY→ALLOW flip or
  any fail-open proposal (which must be refused).

## References
Brief §10 (policy-store-unavailable → embedded Cedar default-deny; Cedar
default-deny + forbid-overrides-permit); ADR-0183; `design/failure-modes.md` F8–F9;
`design/operational-boundaries.md`; `slos/decision-correctness.openslo.yaml`.
