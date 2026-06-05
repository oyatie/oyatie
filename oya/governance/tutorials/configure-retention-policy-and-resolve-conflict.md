---
doc_class: Tutorial
microservice: governance
persona: governance-engineer + compliance-engineer
related_adrs: [ADR-GOV-001, ADR-COMP-001]
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure pack-overlay retention policies + resolve a cross-pack conflict

You will: configure a tenant with two competing pack retention policies (default + hipaa), trigger a real conflict, see the resolver pick the winner with higher-restriction-wins, generate a transparency report, dry-run a retention shortening with dual approval, walk the full audit-chain trail. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant with `tenant_class = paid` (ADR-0329 + ADR-0330 + ADR-0331).
- Governance control-plane access with tenant-admin scope; no local development CLI is authority.
- Two tenant-admin principals (for dual-approval flow): `u-compliance-admin@acme-corp.com` + `u-legal-counsel@acme-corp.com`.

## Step 1 — Subscribe tenant to multiple packs (≤ 5 min)

```sh
oya compliance tenant pack-activate \
    --tenant acme-corp \
    --pack-id default \
    --requesting-principal u-compliance-admin@acme-corp.com

oya compliance tenant pack-activate \
    --tenant acme-corp \
    --pack-id hipaa \
    --requesting-principal u-compliance-admin@acme-corp.com \
    --baa-evidence ./hipaa-baa-acme-corp.pdf
# Output:
#   activated_packs: [default, hipaa]
#   pack_set_hash: psh_acme_corp_001
```

## Step 2 — Configure retention policies per pack (≤ 10 min)

```sh
# Default pack: retain drive uploads for 1 y minimum
oya governance retention policy create \
    --tenant acme-corp \
    --pack-id default \
    --event-class "drive.file.uploaded.v1" \
    --data-class PII_SENSITIVE \
    --minimum-duration 1y \
    --max-duration 5y \
    --restriction-level 3 \
    --legal-basis "Default pack baseline retention" \
    --delete-approval-class dual_approval
# Output:
#   policy_id: rp_default_drive_001
#   pack_id: default
#   restriction_level: 3

# HIPAA pack: retain drive uploads for 6 y minimum (per 45 CFR § 164.530(j))
oya governance retention policy create \
    --tenant acme-corp \
    --pack-id hipaa \
    --event-class "drive.file.uploaded.v1" \
    --data-class PII_SENSITIVE \
    --minimum-duration 6y \
    --max-duration 6y \
    --restriction-level 8 \
    --legal-basis "45 CFR § 164.530(j); HIPAA Privacy Rule documentation retention requirement" \
    --delete-approval-class regulator_attested
# Output:
#   policy_id: rp_hipaa_drive_001
#   pack_id: hipaa
#   restriction_level: 8
```

## Step 3 — Trigger an event + evaluate retention (≤ 5 min)

Simulate a drive upload event being processed by governance:

```sh
# Drive emits an audit-chain event
oya drive file upload \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --file-path ./financial-report.pdf \
    --data-class PII_SENSITIVE \
    --tags confidential,financial
# Output: file_id=f_acme_001, audit_event_id=ae_drive_uploaded_001

# Governance ingests + evaluates retention
oya governance retention evaluate \
    --tenant acme-corp \
    --source-event-id ae_drive_uploaded_001 \
    --event-class drive.file.uploaded.v1 \
    --data-class PII_SENSITIVE
# Output:
#   source_event_id: ae_drive_uploaded_001
#   candidate_rules:
#     - rule_id: rp_default_drive_001 (pack: default; restriction_level: 3)
#     - rule_id: rp_hipaa_drive_001 (pack: hipaa; restriction_level: 8)
#   winning_rule_id: rp_hipaa_drive_001
#   effective_retention_until: 2032-05-20T14:32:17Z   # 6y from upload
#   reason_code: higher_restriction_wins
#   hold_refs: []
```

The HIPAA pack wins because `restriction_level=8 > 3`. The default pack's 1-y minimum is overridden by HIPAA's 6-y.

## Step 4 — Generate transparency report (≤ 5 min)

```sh
oya governance retention conflict transparency-report \
    --source-event-id ae_drive_uploaded_001 \
    --include-legal-basis true \
    --include-historical-precedent true
# Output:
#   source_event_id: ae_drive_uploaded_001
#   conflict_id: cf_acme_001
#   winning_rule_id: rp_hipaa_drive_001
#   losing_rule_id: rp_default_drive_001
#   reason_code: higher_restriction_wins
#   pack_winning: hipaa
#   restriction_level_winning: 8
#   legal_basis_winning: "45 CFR § 164.530(j); HIPAA Privacy Rule documentation retention requirement"
#   restriction_level_losing: 3
#   legal_basis_losing: "Default pack baseline retention"
#   decision_owner: axis-compliance
#   historical_precedent: "ADR-COMP-001 § Decision precedence-step-3-higher-restriction-wins + ADR-GOV-001 § Decision retention-policy-resolution"
#   adr_authority: "ADR-COMP-001 + ADR-GOV-001 + ADR-0304-cross-jurisdiction-conflict-resolution"
```

Save the report:

```sh
oya governance retention conflict transparency-report \
    --source-event-id ae_drive_uploaded_001 \
    --output ./transparency-report-cf_acme_001.json \
    --format json
```

Auditors + regulators can retrieve this via `oya governance evidence query`.

## Step 5 — Daily retention conflict report (≤ 5 min)

```sh
oya governance retention conflicts daily-report \
    --tenant acme-corp \
    --date 2026-05-20
# Output:
#   date: 2026-05-20
#   tenant: acme-corp
#   total_conflicts: 1
#   conflicts:
#     - conflict_id: cf_acme_001
#       event_class: drive.file.uploaded.v1
#       data_class: PII_SENSITIVE
#       winning_pack: hipaa
#       count: 1   # 1 event in this conflict pattern today
#   summary:
#     by_pack:
#       - pack: hipaa
#         won_conflicts: 1
#       - pack: default
#         lost_conflicts: 1
```

## Step 6 — Dry-run retention shortening (without dual approval) (≤ 5 min)

Try to shorten the HIPAA retention (DENIED):

```sh
oya governance retention policy shorten \
    --policy-id rp_hipaa_drive_001 \
    --new-minimum-duration 1y \
    --requesting-principal u-compliance-admin@acme-corp.com
# Cedar evaluates:
#   - governance::retention::shorten requires dual_approval (per ADR-GOV-001)
#   - only 1 approver present
# Output: 403 Forbidden
# Error: "retention_shorten_requires_dual_approval"
```

Now with dual approval:

```sh
oya governance retention policy shorten \
    --policy-id rp_hipaa_drive_001 \
    --new-minimum-duration 1y \
    --requesting-principal u-compliance-admin@acme-corp.com \
    --co-approver u-legal-counsel@acme-corp.com \
    --justification "Synthetic test data; not actual PHI per BAA scope review of 2026-05-15"
# Cedar evaluates:
#   - dual approvers present ✓
#   - both have governance::retention::shorten_co_approve permission ✓
#   - no active legal hold ✓
#   - projection freshness green ✓
# Output:
#   policy_id: rp_hipaa_drive_001
#   from_minimum_duration: 6y
#   to_minimum_duration: 1y
#   soak_period: 60s   # per ADR-GOV-001 § Decision
#   effective_at: 2026-05-20T14:33:17Z
#   audit_event_id: ae_gov_retention_shortened_001
```

The shortening is rare + heavily audited. Most retention changes are upward.

## Step 7 — Bypass grant for a temporarily-failing lane (≤ 5 min)

```sh
# A docs-coverage lane is failing due to a false positive; need to ship hotfix
oya governance bypass grant create \
    --tenant acme-corp \
    --lane-id docs-coverage \
    --action "skip-docs-coverage-check" \
    --reason "Hotfix shipping; doc-coverage check false-positive on auto-generated file" \
    --requested-by u-engineer@acme-corp.com \
    --expires-at 2026-05-21T14:32:17Z \
    --approver-1 u-team-lead@acme-corp.com \
    --approver-2 u-engineering-director@acme-corp.com
# Output:
#   grant_id: bg_acme_001
#   expires_at: 2026-05-21T14:32:17Z
#   audit_event_id: ae_gov_bypass_granted_001
```

After the engineer uses the bypass + the hotfix ships, the grant auto-expires.

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "governance.*" --since 60m
```

Expected events:

- `governance.retention.policy.created.v1` (× 2; default + hipaa)
- `governance.retention.decision.evaluated.v1` (each event triggers an evaluation; many)
- `governance.retention.conflict.detected.v1`
- `governance.retention.transparency-report.generated.v1`
- `governance.retention.policy.shortened.v1`
- `governance.bypass.granted.v1`
- `governance.bypass.expired.v1` (after expiration window)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
```

## What you've learned

- Tenant pack activation with multiple competing packs.
- Per-pack retention policy creation with `restriction_level`.
- Cross-pack conflict resolution via higher-restriction-wins.
- Transparency report generation with legal basis + ADR authority.
- Daily retention conflict report.
- Retention shortening with dual approval + soak period.
- Bypass grants with expiry + dual approval.
- Audit-chain verification of the full retention lifecycle.

Next tutorial: `tutorials/rebuild-projection-from-audit-chain.md` — execute a full projection rebuild after schema migration (paid tenant_class).
