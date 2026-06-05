---
doc_class: Onboarding
microservice: compliance
persona: compliance-engineer + privacy-engineer + dpo
related_adrs: [ADR-COMP-001, ADR-0304, ADR-0010, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Compliance Engineer onboarding — first 5 working days on `compliance`

Audience: a new compliance engineer, privacy engineer, or Data Protection Officer joining the `compliance` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, published a pack overlay, activated multiple packs on a tenant, exercised a multi-pack conflict + transparency report, run a DSAR fulfillment, walked the pack-hotfix runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the five-vendor displacement + 6-step precedence doctrine.
2. Read `ARCHITECTURE.md` § pack-registry + § effective-policy-projection + § conflict-resolution + § dsar-pipeline + § dpia-orchestration + § regulator-portal (∼ 90 min).
3. Read `decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md` end-to-end (∼ 60 min). The binding architecture.
4. Read `docs/decisions/ADR-0304-cross-jurisdiction-conflict-resolution.md` + `ADR-0010-regional-pack-architecture.md` + `ADR-0251-compliance-pack-primitive.md` (∼ 60 min). Critical authority.
5. Read IP series IP-001 through IP-017 (∼ 60 min total).
6. Open the Grafana folder `compliance`. Key boards: `compliance-effective-policy-compute-latency`, `compliance-pack-conflict-total`, `compliance-pack-projection-staleness`, `compliance-regulator-request-denied`, `compliance-dsar-fulfillment-time`, `compliance-pack-coverage-percent`.
7. Walk `runbooks/README.md`. The on-call runbooks: `pack-publish-soak-stuck.md`, `effective-policy-projection-stale.md`, `multi-pack-conflict-explosion.md`, `dsar-fulfillment-stuck.md`, `regulator-export-failed.md`, `pack-hotfix-emergency.md`, `cross-jurisdictional-transfer-blocked.md`, `dpia-orchestration-stuck.md`.
8. Sit in on the Wednesday compliance-substrate handoff.

Acceptance: you can sketch the effective-policy computation path: tenant subscribes to packs → server computes `EffectivePackPolicy` projection per (tenant_id, primitive, action, data_class, jurisdiction) → resolves conflicts via 6-step precedence → publishes via `compliance.effective-policy.changed.v1` → product µservices subscribe + use for runtime enforcement. And the DSAR path: user submits DSAR via tenant portal → compliance creates `DsarRequest` → cascade to drive (find files), messenger (find DMs), mail (find emails), calendar (find events), identity (find principals) → conflict resolution (e.g., GDPR erasure vs HIPAA retention) → produce response bundle → audit-chain seal → deliver to user within 30 d.

## Day 2 — demo_trial cell bootstrap + first pack overlay

```text
Native operation: compliance bootstrap
Route: cloud control-plane operation ledger (not local retired CLI/raw Cargo)
Required evidence:
- Buck2 target(s) for the changed contract/runtime
- Prow/Kubernetes-native `oya-ci-required` job URL
- operation ledger id and emitted audit-chain event ids
```

Expected runtime: ≤ 14 min. Verify:

```sh
oya compliance health --cell drill-syd-1
# Expected:
#   postgres.compliance_pack_overlay: up (lag_ms=12)
#   postgres.effective_pack_policy: up
#   clickhouse.compliance_evidence: up
#   kafka.compliance-events: connected
#   valkey.effective-policy-cache: up
#   audit-chain.emit: up
#   pack_schema_version: compliance-pack-overlay-v1
#   active_packs: 0
```

Publish the SOC 2 pack overlay (demo_trial ships with SOC 2 + ISO 27001 + basic GDPR + basic HIPAA):

```sh
oya compliance pack publish \
    --pack-id soc2 \
    --version 2026.05.20 \
    --jurisdiction global \
    --rules-file ./packs/soc2-rules.yaml \
    --cedar-policies-file ./packs/soc2-policies.cedar \
    --scorecard-refs ./packs/soc2-scorecards.yaml \
    --legal-basis "AICPA SOC 2 Type 2 Trust Service Criteria 2017"
# Cedar evaluates:
#   - compliance::pack::publish ✓
#   - signed pack schema ✓
# Output:
#   pack_id: soc2
#   version: 2026.05.20
#   rules_count: 247
#   audit_event_id: ae_comp_pack_published_001
```

Acceptance: cell bootstrap + SOC 2 pack published.

## Day 3 — Activate multiple packs + exercise multi-pack conflict

Activate packs on a tenant (paid dedicated-cloud feature; shadow at demo_trial):

```sh
oya compliance tenant pack-activate \
    --tenant drill-acme \
    --pack-id soc2 \
    --version 2026.05.20 \
    --requesting-principal u-compliance-admin@drill.test

oya compliance tenant pack-activate \
    --tenant drill-acme \
    --pack-id gdpr \
    --version 2026.05.20 \
    --requesting-principal u-compliance-admin@drill.test

oya compliance tenant pack-activate \
    --tenant drill-acme \
    --pack-id hipaa \
    --version 2026.05.20 \
    --requesting-principal u-compliance-admin@drill.test \
    --baa-evidence ./hipaa-baa-drill-acme.pdf

# Wait soak period (60s per ADR-COMP-001)
sleep 60

# Activations complete; effective policy projected
oya compliance effective-policy --tenant drill-acme
# Output:
#   pack_set_hash: psh_drill_acme_001
#   pack_set: [soc2, gdpr, hipaa]
#   effective_policy_projection_version: ep_v1
#   sample_decisions:
#     - primitive: data_retention
#       action: keep_audit_log
#       data_class: PII_FINANCIAL_SENSITIVE
#       winning_rule_id: rule_hipaa_001
#       restriction_level: 8
#       decision: retain_6y_minimum
#     - primitive: data_breach_notification
#       action: notify_dpa
#       data_class: PII_SENSITIVE
#       winning_rule_id: rule_gdpr_art33_001
#       restriction_level: 7
#       decision: notify_within_72_hours
```

Trigger an explicit conflict — request data erasure (GDPR Art 17) on PHI data (HIPAA 6-y retention):

```sh
oya compliance regulator-request evaluate \
    --tenant drill-acme \
    --request-class gdpr_erasure_art17 \
    --target-data-class PHI \
    --target-subject-id u-alice@drill.test
# Cedar evaluates the 6-step precedence:
#   Step 1: absolute hard-stop? HIPAA blocks medical record erasure during retention window → HARD STOP
# Output:
#   decision: deny
#   winning_rule_id: rule_hipaa_530_minimum_retention
#   pack: hipaa
#   reason_code: hard_stop_hipaa_retention
#   legal_basis: "45 CFR § 164.530(j) prohibits PHI erasure during 6-y retention window"
#   transparency_report_ref: tr_drill_001
#   audit_event_id: ae_comp_request_denied_001
```

Generate transparency report:

```sh
oya compliance transparency-report generate \
    --conflict-id cf_drill_001 \
    --include-legal-basis-all-packs true
# Output:
#   report:
#     conflict_id: cf_drill_001
#     winning_rule_id: rule_hipaa_530_minimum_retention
#     winning_pack: hipaa
#     winning_step: 1 (absolute hard-stop)
#     winning_legal_basis: "45 CFR § 164.530(j) HIPAA Privacy Rule retention requirement"
#     losing_rules:
#       - rule_id: rule_gdpr_art17
#         pack: gdpr
#         legal_basis: "GDPR Article 17 right to erasure"
#         losing_step: 1 (preempted by hard-stop)
#     resolution_authority: ADR-COMP-001 § Decision precedence-step-1 + ADR-0304 cross-jurisdiction-conflict
#     notification_required: yes (notify subject within 30d of denial with reason)
#     subject_appeal_pathway: regulator-complaint
```

Acceptance: multi-pack conflict + transparency report verified.

## Day 4 — DSAR fulfillment with multi-pack conflict resolution

A user submits a GDPR Art 15 DSAR (right of access):

```sh
oya compliance dsar create \
    --tenant drill-acme \
    --request-class gdpr_access_art15 \
    --subject-id u-alice@drill.test \
    --subject-verified-via passkey \
    --scope full \
    --due-by 2026-06-19   # GDPR Art 12: 30 d max
# Output:
#   dsar_id: dsar_drill_001
#   state: collecting
#   estimated_completion: 2026-06-19
#   audit_event_id: ae_comp_dsar_created_001
```

The DSAR cascade fans out to product µservices:

```sh
oya compliance dsar status --dsar dsar_drill_001
# Output:
#   dsar_id: dsar_drill_001
#   state: collecting
#   sub-tasks:
#     drive: 12 files found (4.2 GiB)
#     messenger: 47 conversations (alice party to)
#     mail: 1 248 messages
#     calendar: 24 events (alice attendee)
#     identity: 1 principal record + 3 credentials
#     ... (per cross-µservice fan-in per IP-011)
#   blocked_data_classes: [PHI] (HIPAA retention; partial export)
#   blocking_pack: hipaa
#   transparency_report_ref: tr_drill_002
```

Resolve the DSAR:

```sh
# Wait for cascade complete (in production: minutes to hours)
sleep 30

oya compliance dsar finalize --dsar dsar_drill_001 --output ./dsar-drill_001-bundle.zip
# Output:
#   dsar_id: dsar_drill_001
#   state: completed
#   bundle_size: 524 MB
#   bundle_path: ./dsar-drill_001-bundle.zip
#   bundle_signature_b64: Ed25519:7c4a2b8e...
#   included_subjects: u-alice@drill.test
#   excluded_due_to_pack_conflict:
#     - data_class: PHI
#       reason: hipaa_minimum_retention_active
#       transparency_report_ref: tr_drill_002
#       legal_basis: "45 CFR § 164.530(j)"
#   audit_event_id: ae_comp_dsar_completed_001
```

The user receives the bundle. The transparency report explains why PHI was excluded.

Acceptance: DSAR fulfillment with multi-pack conflict verified.

## Day 5 — Pack hotfix emergency + DPIA orchestration

Pack hotfix scenario (per ADR-COMP-001 § Decision):

Suppose the EU regulator publishes an emergency clarification of GDPR Art 32 (security of processing); pack must update immediately.

```sh
oya compliance pack hotfix \
    --pack-id gdpr \
    --base-version 2026.05.20 \
    --hotfix-version 2026.05.20-hotfix-1 \
    --rules-file ./packs/gdpr-hotfix-rules.yaml \
    --reason "EU Commission published clarifying guidance on Art 32" \
    --requesting-principal u-compliance-owner@drill.test \
    --skip-soak-check true   # only with emergency reason
# Cedar evaluates:
#   - compliance::pack::hotfix ✓
#   - emergency reason present ✓
# Output:
#   pack_id: gdpr
#   hotfix_version: 2026.05.20-hotfix-1
#   audit_event_id: ae_comp_pack_hotfix_001
#   previous_version_preserved_for_historical_decisions: true
```

Verify: historical DSARs decided against the previous version remain unchanged; new DSARs use the hotfix version.

DPIA orchestration:

```sh
# Tenant initiates a DPIA for a new high-risk processing activity
oya compliance dpia initiate \
    --tenant drill-acme \
    --processing-activity-name "ML-based credit scoring" \
    --data-classes "PII_FINANCIAL_SENSITIVE,PII_SOCIAL_BEHAVIORAL" \
    --intended-use "automated decision-making per GDPR Art 22"
# Output:
#   dpia_id: dpia_drill_001
#   risk_assessment:
#     - data_minimization: medium
#     - consent: high (Art 22 explicit consent required)
#     - automated_decision_making: high (Art 22 right of explanation)
#     - cross-border_transfer: medium (if model trained on EU data)
#   recommended_safeguards:
#     - "Document algorithm + provide right-of-explanation interface"
#     - "Use explicit consent flow"
#     - "Implement automated-decision review pathway"
#   audit_event_id: ae_comp_dpia_initiated_001
```

Acceptance: pack hotfix + DPIA orchestration verified.

## What you've learned

- demo_trial bootstrap + pack publishing.
- Multiple pack activation (SOC 2 + GDPR + HIPAA).
- Multi-pack conflict resolution via 6-step precedence.
- Transparency report generation.
- DSAR fulfillment with multi-pack conflict resolution.
- Pack hotfix emergency workflow.
- DPIA orchestration.

Next week: paid dedicated-cloud promotion (20-pack scope + DSAR automation at scale + DPIA templates), paid on-prem-connected tour (certification evidence pipeline + regulator export bundles + ADR-0304 cross-jurisdiction authority + EU AI Act Annex III pipeline + per-tenant DPO workspace), paid compliance_pack tour (30+ packs + per-pack residency + regulator-attested publishing + cross-jurisdictional transfer evidence), and your first production shadow on a regulator export approval.
