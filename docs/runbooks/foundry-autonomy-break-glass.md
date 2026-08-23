---
purpose: Oyatie Runbook — Foundry Autonomy Break Glass
doc_status: published
---

# Oyatie Runbook — Foundry Autonomy Break Glass

> **Status:** Active W-Foundation ledger procedure; runtime override execution remains gated by ADR-0022 follow-up implementation
> **Owner:** `axis-foundry + ops-security + council-privacy`
> **Severity scope:** Sev 1
> **Authored from:** [`templates/runbook-template.md`](../templates/runbook-template.md)
> **Last verified:** 2026-05-11 (validated against `presubmit` (retired CLI `gate validate foundation-bypass`) autonomy break-glass ledger records)

## Symptom
An otherwise blocked capability invocation must proceed during an emergency, or an operator requests a temporary autonomy ceiling override for a tenant/capability pair.

Break-glass is not a bypass around the autonomy ceiling. It is an audited, time-boxed, M-of-N approved exception record in the foundation-bypass ledger, per ADR-0022 and ADR-0025.

## Detection
- Source signal: Sev incident bridge request, policy denial evidence, autonomy-ceiling breach alert, or customer/regulator-impacting outage.
- Confirm the affected `tenant_id`, `capability_id`, requested autonomy tier, current denial evidence, and data classes before any approval is gathered.
- For regulated, catastrophic, or T4 requests, page `axis-foundry`, `ops-security`, and `council-privacy` immediately.
- Incident commanders may downgrade non-regulated T2/T3 emergency overrides to Sev 2 only after confirming no regulated data class, T4 execution, tenant isolation, or safety impact is present.

## First-response checklist
1. Acknowledge the page and declare the incident in `#incident-bridge`.
2. Capture the audit-chain segment for the denial and impact window per ADR-0003.
3. Verify the capability is registered, evaluated, and still under the same tenant/capability binding.
4. Decide quorum class:
   - `two-of-three` for standard emergency override.
   - `three-of-five` for catastrophic-class, regulated, or T4 override.
5. Collect approvals from distinct `usr_` or `svc_` principals; the requesting actor cannot self-approve.
6. Add one `entry_class: autonomy-break-glass` YAML record under `registry/foundation-bypasses/`.
7. Run `presubmit` (retired CLI `gate validate foundation-bypass --ledger registry/foundation-bypasses`) before proceeding.

## Ledger record shape

```yaml
entry_class: autonomy-break-glass
id: abg_0001
tenant_id: ten_example
capability_id: cap.example.emergency
requested_tier: T4AutoExecute
permitted_tier: T4AutoExecute
requesting_actor: usr_operator
approving_actors: usr_security,usr_privacy
approval_quorum: two-of-three
rationale: patient safety emergency with explicit expiry
created_at_epoch_days: 20585
expires_at_epoch_days: 20586
```

Allowed tiers are `T1ViewOnly`, `T2Advisory`, `T3ExecuteWithApproval`, and `T4AutoExecute` (short forms `T1`-`T4` are accepted by the validator). Use `approval_quorum: three-of-five` for catastrophic-class overrides; the validator requires at least three distinct approvers.

## Containment
- Keep the override window as short as possible; set `expires_at_epoch_days` to the earliest defensible day boundary.
- Do not weaken capability policy, data-use boundaries, or tenant bindings to force an invocation through.
- If approvals cannot meet quorum, leave the denial in place and escalate to incident command.

## Diagnosis
- Determine whether the emergency came from tenant configuration, regional/vertical pack cap, subject-class cap, capability required tier, or policy rule drift.
- Compare the denial evidence fields against ADR-0022 inputs: tenant configured ceiling, capability required tier, vertical-pack cap, subject-class cap, and agentic-ads cap.
- If a policy or catalog defect caused the request, open a prevention issue before the incident closes.

## Recovery
- Remove the operational need for the override, then record revocation by adding `revoked_at_epoch_days` to the ledger record.
- Revocation must happen on or before `expires_at_epoch_days`; otherwise `presubmit` (retired CLI `gate validate foundation-bypass`) fails closed with `ExpiredBypass`.
- Restore the normal autonomy ceiling path and verify the original invocation would now be allowed or correctly denied without break-glass.

## Verify-recovery
- Run `presubmit` (retired CLI `gate validate foundation-bypass --ledger registry/foundation-bypasses`) and confirm zero expired open break-glass records.
- Confirm audit-chain integrity per ADR-0003 and preserve approval/override/revocation evidence in the incident record.
- Run the affected capability invocation test or policy fitness lane.
- File a MISTAKES-LEDGER row and prevention ticket if existing gates did not catch the defect.

## Sources
[INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../SLO-CATALOG.md), [`standards/prevention-doctrine.md`](../standards/prevention-doctrine.md), [`templates/runbook-template.md`](../templates/runbook-template.md), ADR-0003, ADR-0022, ADR-0025.
