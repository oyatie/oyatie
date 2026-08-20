---
doc_class: Runbook
title: Regulator export reissue — bundle defect, late-signal, or scope correction
microservice: foundry-evidence
severity: Sev-2 (Sev-1 if external regulator has already received the defective bundle)
status: Accepted
owner_team: council-privacy + axis-foundry-evidence
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-07, FM-08)
  - microservices/intelligence/policy/regulator-export-scope.cedar
  - microservices/intelligence/runbooks/evidence-pack-rebuild.md
  - microservices/intelligence/incident-response.md
doc_status: published
---

# Runbook: Regulator export reissue

## Purpose

Reissue procedure for a previously-delivered regulator export bundle when:
- FM-07: bundle was assembled with a defective framework profile (e.g., missing EU AI Act Art. 18 fields).
- FM-08: bundle scope was wrong (off-by-one on time-range, wrong tenant filter, wrong framework).
- A pack in the bundle has been superseded via `runbooks/evidence-pack-rebuild.md` and the regulator needs the corrected pack.

## Trigger

- Regulator-reported defect via engagement channel.
- Internal QA finding before regulator receives.
- Post-rebuild propagation requirement.
- Framework-profile drift detection on CI lane `regulator-profile-drill`.

## Severity

- **Sev-1** if the regulator has already received the defective bundle.
- **Sev-2** otherwise.

## Procedure

### Phase 1: Engage (≤ 15 min)

1. Declare severity; open `#inc-<id>`.
2. Engage IC: council-privacy chair (lead) + axis-foundry-evidence on-call + ops-security + legal-counsel.
3. Identify whether regulator has received:
   ```
   oya foundry-evidence regulator-export status \
     --bundle-id <id> \
     --include-delivery-receipts
   ```

### Phase 2: Defect characterisation (≤ 1 h)

1. Pull the original bundle from cold storage:
   ```
   oya foundry-evidence regulator-export retrieve \
     --bundle-id <id> --output ./original-bundle.tar
   ```
2. Verify bundle signature:
   ```
   oya foundry-evidence regulator-export verify \
     --bundle ./original-bundle.tar
   ```
   Expect: verification succeeds (defect is content, not signature).
3. Characterise the defect:
   - Field-completeness drift vs framework profile.
   - Time-range scope.
   - Tenant scope.
   - Superseded-pack inclusion.
4. Record findings + classification in incident channel.

### Phase 3: 2-person rule + Cedar permit (≤ 30 min)

1. Reissue request envelope assembled by IC: `{original_bundle_id, defect_class, corrected_scope, justification}`.
2. Distinct approver countersigns.
3. Cedar `regulator-export-scope.cedar` PERMIT evaluated; reissue treated as a fresh export (carries its own bundle_id).

### Phase 4: Reissue assembly (≤ 30 min – 2 h)

1. Trigger fresh assembly:
   ```
   oya foundry-evidence regulator-export reissue \
     --supersedes-bundle <original_bundle_id> \
     --framework <framework> \
     --tenant-id <tenant_id> \
     --time-range-start <ts> --time-range-end <ts> \
     --approver <approver-spiffe> \
     --justification-file justification.txt \
     --receiving-bucket-scc-id <scc_id>
   ```
2. Verify reissued bundle:
   ```
   oya foundry-evidence regulator-export verify --bundle ./reissue.tar
   ```
   Expect: signature valid; field-completeness against framework profile passes.
3. Diff against original bundle; archive the diff alongside the incident artifacts.

### Phase 5: Delivery (≤ 1 h)

1. If regulator has NOT received original yet:
   - Withdraw original from delivery queue (`oya foundry-evidence regulator-export withdraw --bundle-id <id>`).
   - Deliver reissue via standard mechanism.
2. If regulator HAS received original:
   - council-privacy chair (or legal-counsel) notifies regulator engagement lead in writing.
   - Deliver reissue with `supersedes_bundle_id=<original>` in the bundle envelope so regulator can index.
   - Track regulator acknowledgement.

### Phase 6: Post-reissue (≤ 5 business days)

- audit-emit `foundry.evidence.regulator_export.reissued.v1` linking original → reissue.
- Postmortem.
- If defect was field-completeness drift, file a P0 fix on the framework profile + add regression test to `regulator-profile-drill` lane.
- If defect was superseded-pack propagation, file a successor-IP to make pack-rebuild trigger automatic regulator-export notification.

## Halt conditions

- Original bundle signature verification fails on retrieval → escalate to Sev-1 (substrate integrity event); engage axis-audit-chain.
- Two-person rule cannot be honoured → defer; surface in council-privacy chair queue.
- Cedar permit refuses (typically: receiving bucket SCC has expired) → engage tenancy + cloud-secrets to refresh SCC.
- Reissue would expose data outside the engagement scope → halt; reassess engagement contract with legal-counsel.

## Verification (post-reissue)

- Reissued bundle delivered to regulator (or queued for tenant-mediated bridge).
- audit-chain carries `regulator_export.reissued.v1` event linking originals.
- Postmortem published.
- If applicable, regression test added to CI.

## References

- `microservices/intelligence/policy/regulator-export-scope.cedar`.
- `microservices/intelligence/failure-modes.md` FM-07 + FM-08.
- `microservices/intelligence/runbooks/evidence-pack-rebuild.md`.
- ADR-0133 (claim honesty: any bundle field-completeness drift is itself a claim-matrix update).
