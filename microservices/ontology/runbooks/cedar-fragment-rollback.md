---
doc_class: Runbook
title: Cedar fragment rollback (deploy-time + runtime rollback of policy fragments)
microservice: ontology
severity: "Sev-1 (Cedar engine pathological / forged permit) / Sev-2 (operational rollback)"
status: Accepted
owner_team: axis-ontology + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-06 Cedar runaway, FM-13 audit chain tampering)
  - microservices/ontology/policy/*.cedar
doc_status: published
---

# Runbook: Cedar fragment rollback

## Trigger

Any of:
- Cedar engine evaluation latency p99 > 100 ms for ≥ 1 min (FM-06).
- Cedar fragment deployed that grants unintended permit (security incident).
- Audit reveals Cedar fragment edit bypassed PR review.

## Severity

- Sev-1 if forged permit / unintended grant active in production (data exposure risk).
- Sev-2 if Cedar engine pathological — engine 10 ms hard timeout is the fail-safe; Sev-2 from operational degradation, not breach.

## Pre-checks

1. Identify the offending fragment: `git log --oneline microservices/ontology/policy/*.cedar | head -5`.
2. Confirm the prior fragment SHA: `git show HEAD~1:microservices/ontology/policy/<fragment>.cedar`.
3. Confirm the prior SHA is itself signed + linear + present in history.
4. Capture the rollback reason in a structured form.

## Steps

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>` Slack; declare severity; assign IC | ≤ 5 min |
| 2 | If forged permit: engage ops-security; freeze affected Action path via Cedar evaluator hot-reload override | ≤ 2 min |
| 3 | Identify the change: `git diff HEAD~1 HEAD -- microservices/ontology/policy/<fragment>.cedar` | ≤ 2 min |
| 4 | Git revert: `git revert <bad-sha> -m <merge-context> --no-edit` | ≤ 1 min |
| 5 | Push revert PR + bypass branch-protection for emergency (2-person rule; OpenBao JIT): `gh pr create --title "REVERT: Cedar fragment <name>" --body "<incident-id>"` | ≤ 5 min |
| 6 | Merge revert; ArgoCD picks up; Cedar evaluator hot-reloads via the cedar-fragment-coverage worker | ≤ 5 min |
| 7 | Verify: `oya-ontology-sdk cedar-validate --fragment <name>` returns the prior fragment's SHA-256 | ≤ 1 min |
| 8 | Validate runtime: synthetic permit/deny tests for the affected Action Type return the expected verdict | ≤ 5 min |
| 9 | Audit-chain emit `CedarFragmentRollback{fragment, prev_sha, new_sha, reason, executed_at}` | automatic |
| 10 | Postmortem within 5 business days | – |

## Forged-permit incident path

If the forged fragment granted unauthorized access:

| Step | Action |
|---|---|
| A | Identify principals that exercised the forged permit (audit-chain queries by `cedar_fragment_sha`). |
| B | For each: revoke the grant (if not yet expired); audit-chain emit `ForgedPermitRevoked`. |
| C | Engage council-privacy: was personal data accessed under the forged permit? if yes, breach-notification chain (GDPR Art. 33 72h; HIPAA §164.404; KR PIPA Art. 34). |
| D | Forensic trace: how did the fragment bypass PR review? ops-security + axis-ontology. |
| E | Action items: tighten branch-protection on `policy/*.cedar` files; require 2-person CODEOWNERS sign-off; LEAN check on fragment hash compared to expected. |

## Runtime hot-reload validation

Cedar fragments are hot-reloaded by the `cedar-fragment-coverage` adapter via the schema-propagation-worker. To force a reload:

```bash
oya-ontology-sdk cedar-reload --fragment <name> --target-sha <sha>
```

Validation:
- `cedar_fragment_loaded_sha{fragment="<name>"}` Postgres metric matches the target SHA.
- Synthetic permit/deny probe returns expected.

## Verification

After rollback:
- `oya gate validate cedar-coverage --microservice ontology` — exit 0.
- All Action Types still have permit + default-deny.
- Cedar evaluation p99 < 10 ms.
- `oya_ontology_cedar_fragment_hash{fragment="<name>"}` matches prior version SHA.
- Audit chain seal for the rollback present.

## Post-incident updates

- Postmortem.
- If forged permit: ADR successor-IP tightening Cedar fragment authoring (e.g., 2-person CODEOWNERS sign-off; mandatory ops-security review for any new `permit` clause).
- If recurrence ≥ 2: consider Cedar fragment versioning with checksum-pinned references.

## References

- `microservices/ontology/failure-modes.md` FM-06.
- `microservices/ontology/policy/*.cedar`.
- Cedar v4 reference — `cedarpolicy.com`.
- ADR-0140 (Cedar policy enforcement).
