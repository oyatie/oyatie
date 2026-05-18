# Runbook: data-residency-enforcement

- Severity: P3 routine (P1 if violation found; P0 if regulator-flagged)
- Trigger:
  - Quarterly residency report scheduled task.
  - Pack overlay rule change deployed (replay required).
  - Regulator request for residency report.
  - Tenant changes default_residency_region.

## Step 1 — Generate residency report

`oya consent-graph residency-report --window 90d --output evidence/residency-<ts>.json`:

For each agreement:
- Class A row location (Postgres region).
- Class B projection topic Pulsar region.
- Class C projection cache locations (grantee regions).
- Class D audit-chain entries per party + region.
- Class F compiled policy region.
- Class G OpenBao secrets region.

Output JSON: matches `data-residency.md` §2 schema. Sealed in audit-chain.

## Step 2 — Validate (≤30min)

Run validator: `oya consent-graph residency-validate <report>`.

Checks:
1. Every Class A row is in grantor.sovereignty.grantor_region.
2. Every Class B topic.region == grantor.sovereignty.grantor_region.
3. Every Class C cache region ∈ agreement.sovereignty.permitted_grantee_regions.
4. Cross-border-permitted agreements have lawful_basis recorded.
5. Pack overlay rules satisfied per `iac/kustomize/overlays/<pack>/sovereignty-rules.yaml`.

Output: pass / fail per check + list of violations.

## Step 3 — Triage violations (if any)

For each violation:
- Identify class (storage drift, pack rule change, agreement misconfigured).
- Severity:
  - P0 if any data physically resides in forbidden jurisdiction.
  - P1 if pack overlay rule change requires retroactive update.
  - P2 if classifier reclassified a field (review needed).

For P0: escalate to `regional-sovereignty-violation.md`.

For P1: re-run pack overlay replay (`backfill-replay.md` §8).

## Step 4 — Pack overlay rule change (P1 path)

If a pack overlay residency rule changes (e.g., new EU adequacy decision, KR PIPA tightening):

1. Update `iac/kustomize/overlays/<pack>/sovereignty-rules.yaml`.
2. Deploy via Helm; verify rules loaded.
3. Run replay: `oya consent-graph replay pack-overlay --pack <pack> --rule <rule-id>`.
4. Affected agreements re-evaluated against new rules; non-conformant agreements:
   - Emit `pack-overlay-noncompliance` warning event.
   - 14-day grace period for grantor + grantee re-acknowledgement.
   - Auto-suspend after grace.
5. Audit-chain emission for each affected agreement.

## Step 5 — Tenant default_residency_region change

If a tenant changes their residency declaration:
1. consent-graph re-evaluates all their agreements (as grantor + as grantee).
2. For agreements that no longer satisfy sovereignty: warning event + 14d grace.
3. New agreements default to new region; old agreements grandfathered with explicit annotation.

## Step 6 — Regulator request

If a regulator requests residency proof:
1. Generate report scoped to the regulator's jurisdiction.
2. Privacy officer reviews + signs.
3. Deliver via privacy-portal authenticated channel.
4. Audit-chain emission `regulator-disclosure`.

## Verification

- Quarterly report shows zero violations.
- Validator runs in CI as part of pre-deployment gate (P0 violation = blocked deployment).

## Audit evidence

- Quarterly residency reports retained 7y in audit-chain.
- Regulator disclosure events sealed.
- All pack overlay replay actions sealed with `replay_session_id`.

## Cross-references

- data-residency.md §6 right-to-erasure cascade.
- regional-sovereignty-violation.md for P0 escalation.
- compliance.md per-pack rules.
- ADR-SVC-CG-004 grantor-region authority.
