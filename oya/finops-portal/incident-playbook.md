---
doc_id: finops-portal/incident-playbook
authored: 2026-05-18
status: ready
authority: ADR-0130 SLO-gated promotion + incident-management µservice integration
classification: internal
---

# Incident playbook — finops-portal

This playbook coordinates incident response for `finops-portal` SLO
breaches + data-integrity events. It complements the per-alert
runbooks under `runbooks/`.

## Severity matrix

| SEV   | Definition                                                    | Page              | Customer comms |
|-------|---------------------------------------------------------------|-------------------|----------------|
| SEV-1 | Critical: cross-tenant data leak; signing-key compromise      | exec + leadership | yes, < 1 h     |
| SEV-2 | High: quarterly-emit miss; widespread budget-exhaustion       | on-call manager   | yes, < 4 h     |
| SEV-3 | Medium: dashboard outage; isolated tenant impact              | on-call rotation  | optional       |
| SEV-4 | Low: cosmetic UI bug; slow drill-down on one query            | next-business-day | no             |

## Roles

- **Incident commander (IC)** — owns the response; usually
  ops-finops on-call.
- **Communications lead** — drafts status-page + tenant-facing
  comms.
- **Subject-matter expert (SME)** — engineer with the relevant
  context; rotated on a per-bounded-context basis.
- **Scribe** — records the timeline; produces the post-mortem
  draft.

## Lifecycle

1. **Detect** — alert fires OR human report.
2. **Acknowledge** — within 5 min for SEV-1/2; 30 min for SEV-3.
3. **Triage** — apply the relevant runbook (`runbooks/*.md`).
4. **Mitigate** — restore service / contain blast radius.
5. **Resolve** — confirm alert clears; tenant impact ceased.
6. **Post-mortem** — within 5 business days for SEV-1/2; 10 days
   for SEV-3.

## Runbook routing

| Alert / event                              | Runbook                                                              |
|--------------------------------------------|----------------------------------------------------------------------|
| `TenantCostAnomalySpike`                   | `runbooks/tenant-cost-anomaly-spike.md`                              |
| `TenantBudgetHeadroomLow`                  | `runbooks/tenant-budget-headroom-low.md`                             |
| `TenantBudgetExhausted`                    | `runbooks/tenant-budget-exhausted.md`                                |
| `FocusExportFailureRate`                   | `runbooks/focus-export-failure.md`                                   |
| `QuarterlyRegulatorEmitMiss`               | `runbooks/quarterly-regulator-emit-miss.md`                          |
| `CreditApplicationSealMiss`                | `runbooks/credit-application-reconciliation.md`                      |
| `CostAllocationPolicyChangedAlert`         | `runbooks/cost-allocation-policy-rollback.md`                        |
| `FinopsPortalSloBudgetBurnFast`            | `runbooks/finops-portal-deploy-rollback.md`                          |
| Cross-tenant leak (manual report)          | THIS playbook §Cross-tenant leak                                     |
| Signing-key compromise                     | THIS playbook §Key compromise                                        |

## SEV-1 specific: Cross-tenant leak

1. **Immediate**: page exec + leadership.
2. **Contain**: roll back to the previous deploy (which presumably
   did not leak). If the leak is via Cedar policy mis-author,
   redeploy with the previous policy bundle.
3. **Assess scope**: which tenants were exposed to which other
   tenants' data? Use audit-chain query.
4. **Notify**: affected tenants within 24 h; regulator within 72 h
   (GDPR Art. 33) if EU pack.
5. **Remediate**: fix the policy / code; re-deploy via emergency
   path.
6. **Audit**: full code + policy review of the changes in the last
   30 days; gate every future Cedar PR with the cross-tenant test.
7. **Post-mortem**: blameless; published internally; external
   summary published if regulatorily required.

## SEV-1 specific: Signing-key compromise

1. **Immediate**: revoke the active quarterly Ed25519 key in the
   HSM.
2. **Publish**: a `FinOpsQuarterlyKeyRevoked` event to
   audit-chain.
3. **Re-key**: generate a new key in the HSM; publish via
   `FinOpsQuarterlyKeyPublished`.
4. **Re-sign**: every envelope signed by the compromised key gets
   a re-signed counterpart sealed to audit-chain.
5. **Notify**: regulators receive the re-keyed envelopes; chain of
   custody documented.
6. **Post-mortem**: focus on how the key was compromised.

## Drill schedule

- Quarter-end rehearsal: practice the IP-015 emit a week before
  the quarter close (in a staging environment).
- Rollback rehearsal: quarterly; uses
  `runbooks/finops-portal-deploy-rollback.md`.
- Cross-tenant-leak game day: annually.
- Key-rotation rehearsal: every key rotation (quarterly).

## Status page integration

The platform status page consumes a JSON manifest published by the
incident-management µservice. `finops-portal` posts impact
declarations via the standard channel; format documented in the
`incident-management` µservice README.

## Customer communication templates

Templates live at `evidence/incident-comms/finops-portal/`:

- `template-cross-tenant-leak.md`
- `template-quarterly-emit-miss.md`
- `template-budget-exhausted.md`
- `template-focus-export-degraded.md`

Each template includes the placeholder slots for incident commander,
detection time, impact summary, mitigation taken, and next steps.

## Post-mortem template

Lives at `evidence/post-mortems/finops-portal/`; follows the SRE
blameless template:

- What happened?
- When did we detect it?
- What was the impact?
- What was the root cause?
- What action items result?
- What did we learn?

## References

- ADR-0130 SLO-gated promotion.
- ADR-0162 audit-log integrity.
- `runbooks/*.md`.
- `failure-modes.md`.
