---
doc_class: Tutorial
microservice: compliance
persona: compliance-engineer + privacy-engineer + dpo
related_adrs: [ADR-COMP-001, ADR-0304]
date: 2026-05-20
doc_status: published
---

# Tutorial — Resolve a multi-pack erasure conflict (GDPR Art 17 vs HIPAA retention) end-to-end

You will: activate GDPR + HIPAA packs on a tenant, submit a GDPR Art 17 erasure DSAR for a subject with PHI data, watch the 6-step precedence apply (hard-stop wins for HIPAA), generate a transparency report with legal basis, deliver the partial-erasure bundle to the subject, audit-chain-verify the trail. Total time ≤ 75 minutes.

## Pre-requisites

- A tenant on paid tenant_class (`capability-tiers/tier-matrix.md`).
- `oya-dev-cli` ≥ 1.42.0.
- Tenant principals: `u-compliance-admin@acme-corp.com` (compliance admin), `u-dpo@acme-corp.com` (DPO).
- HIPAA BAA evidence file (`./hipaa-baa-acme-corp.pdf`).
- A test subject `u-bob@acme-corp.com` with mixed data classes (regular + PHI).

## Step 1 — Activate GDPR + HIPAA packs (≤ 10 min)

```sh
# Activate GDPR
oya compliance tenant pack-activate \
    --tenant acme-corp \
    --pack-id gdpr \
    --version 2026.05.20 \
    --requesting-principal u-compliance-admin@acme-corp.com
# Output: pack_id=gdpr, version=2026.05.20, audit_event_id=ae_comp_gdpr_activated_001

# Activate HIPAA (requires BAA evidence)
oya compliance tenant pack-activate \
    --tenant acme-corp \
    --pack-id hipaa \
    --version 2026.05.20 \
    --requesting-principal u-compliance-admin@acme-corp.com \
    --baa-evidence ./hipaa-baa-acme-corp.pdf
# Output: pack_id=hipaa, version=2026.05.20, audit_event_id=ae_comp_hipaa_activated_001

# Wait soak period (60s per ADR-COMP-001)
sleep 60
```

Verify both active:

```sh
oya compliance tenant pack-list --tenant acme-corp
# Output:
#   active_packs:
#     - pack_id: gdpr
#       version: 2026.05.20
#       state: active
#     - pack_id: hipaa
#       version: 2026.05.20
#       state: active
#   pack_set_hash: psh_acme_001
```

## Step 2 — Bob has data in both regular + PHI classes (≤ 10 min)

Suppose Bob has 5 drive files (3 regular + 2 PHI):

```sh
oya drive file list --tenant acme-corp --user u-bob@acme-corp.com
# Output:
#   - f_bob_001: "Performance review.pdf" (data_class=PII_SENSITIVE; not PHI)
#   - f_bob_002: "1099-tax-form.pdf" (data_class=PII_FINANCIAL)
#   - f_bob_003: "Vacation itinerary.pdf" (data_class=PII_GENERAL)
#   - f_bob_004: "Lab report 2026-02.pdf" (data_class=PHI)   # subject to HIPAA
#   - f_bob_005: "Prescription history.pdf" (data_class=PHI) # subject to HIPAA
```

## Step 3 — Bob submits a GDPR Art 17 erasure DSAR (≤ 10 min)

```sh
oya compliance dsar create \
    --tenant acme-corp \
    --request-class gdpr_erasure_art17 \
    --subject-id u-bob@acme-corp.com \
    --subject-verified-via passkey-aal3 \
    --scope full \
    --justification "Subject withdrew consent" \
    --due-by 2026-06-19   # GDPR Art 12: 30 d max
# Output:
#   dsar_id: dsar_acme_001
#   state: collecting
#   audit_event_id: ae_comp_dsar_created_001
```

## Step 4 — Watch the multi-pack conflict resolution (≤ 15 min)

DSAR cascade fans out to product µservices. compliance evaluates per-data-class conflicts:

```sh
oya compliance dsar status --dsar dsar_acme_001 --verbose
# Output:
#   dsar_id: dsar_acme_001
#   state: conflict_resolution_in_progress
#   data_class_decisions:
#     PII_SENSITIVE:
#       erasure_decision: permit
#       winning_rule_id: rule_gdpr_art17_001
#       legal_basis: "GDPR Art 17 right to erasure; no overriding regulatory hold"
#       restriction_level: 5
#     PII_FINANCIAL:
#       erasure_decision: permit
#       winning_rule_id: rule_gdpr_art17_001
#     PII_GENERAL:
#       erasure_decision: permit
#       winning_rule_id: rule_gdpr_art17_001
#     PHI:
#       erasure_decision: DENY
#       winning_rule_id: rule_hipaa_530_minimum_retention
#       winning_pack: hipaa
#       winning_step: 1 (absolute hard-stop)
#       restriction_level: 10
#       legal_basis: "45 CFR § 164.530(j) HIPAA Privacy Rule 6-y retention minimum"
#       losing_rule_id: rule_gdpr_art17_001
#       transparency_report_ref: tr_acme_001
#   audit_event_id: ae_comp_dsar_conflict_resolved_001
```

Notice: PHI is denied (HIPAA wins via hard-stop), but other classes are permitted.

## Step 5 — Generate transparency report for the denial (≤ 5 min)

```sh
oya compliance transparency-report generate \
    --conflict-id tr_acme_001 \
    --include-legal-basis-all-packs true \
    --include-historical-precedent true \
    --include-subject-appeal-pathway true \
    --output ./transparency-report-tr_acme_001.json
# Output:
#   report:
#     conflict_id: tr_acme_001
#     dsar_id: dsar_acme_001
#     subject_id: u-bob@acme-corp.com
#     data_class: PHI
#     winning_rule_id: rule_hipaa_530_minimum_retention
#     winning_pack: hipaa
#     winning_step: 1 (absolute hard-stop)
#     winning_legal_basis: "45 CFR § 164.530(j) HIPAA Privacy Rule retention requirement"
#     losing_rule_id: rule_gdpr_art17_001
#     losing_pack: gdpr
#     losing_legal_basis: "GDPR Art 17 right to erasure"
#     resolution_authority: ADR-COMP-001 § Decision precedence-step-1 + ADR-0304 cross-jurisdiction-conflict
#     subject_notification_required: yes
#     subject_appeal_pathway:
#       - "Contact data protection officer (u-dpo@acme-corp.com)"
#       - "File complaint with EU Data Protection Authority (if EU subject)"
#       - "File complaint with HHS Office for Civil Rights (if US subject)"
#     historical_precedent:
#       - ADR-0304-cross-jurisdiction-conflict-resolution.md
#       - In re XYZ-2025: "HIPAA retention is regulatory floor that GDPR erasure cannot pierce without alternate legal basis"
```

## Step 6 — Finalize the DSAR + deliver to the subject (≤ 15 min)

```sh
oya compliance dsar finalize \
    --dsar dsar_acme_001 \
    --output ./dsar-acme_001-bundle.zip \
    --include-transparency-report true \
    --sign-with-tenant-issuer-key true
# Output:
#   dsar_id: dsar_acme_001
#   state: completed
#   bundle_size: 248 MB
#   bundle_path: ./dsar-acme_001-bundle.zip
#   bundle_signature_b64: Ed25519:7c4a2b8e...
#   included_data:
#     - f_bob_001 (PII_SENSITIVE; erased)
#     - f_bob_002 (PII_FINANCIAL; erased)
#     - f_bob_003 (PII_GENERAL; erased)
#   excluded_data:
#     - f_bob_004 (PHI; erasure denied; retained under HIPAA)
#     - f_bob_005 (PHI; erasure denied; retained under HIPAA)
#   transparency_report_included: yes
#   audit_event_id: ae_comp_dsar_completed_001
```

Subject Bob receives the bundle containing:

- His erased non-PHI data (deleted; copies provided per Art 15 in the bundle).
- A transparency report explaining why PHI was retained + his appeal pathway.

## Step 7 — Regulator audit (if challenged) (≤ 5 min)

Suppose a year later, EU DPA challenges the denial. Audit-chain replay:

```sh
oya audit query \
    --tenant acme-corp \
    --event-class "compliance.dsar.*" \
    --since 2026-05-01 \
    --include-precedent-version-snapshots true
# Output: full timeline of dsar_acme_001 + the pack version that was active when the decision was made

oya compliance pack-version-snapshot \
    --pack-id hipaa \
    --version 2026.05.20 \
    --output ./hipaa-version-snapshot-for-dpa-audit.zip
# Output: signed snapshot of the HIPAA pack rules active at the time of denial
```

The DPA can independently verify the decision was legally consistent.

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "compliance.*" --since 60m
```

Expected events:

- `compliance.pack.activated.v1` (× 2; gdpr + hipaa)
- `compliance.effective-policy.changed.v1` (× 1; on pack-set activation)
- `compliance.dsar.created.v1`
- `compliance.dsar.cascade.started.v1`
- `compliance.dsar.cascade.acknowledged.v1` (per µservice)
- `compliance.pack-conflict.detected.v1` (PHI conflict)
- `compliance.pack-conflict.resolved.v1`
- `compliance.transparency-report.generated.v1`
- `compliance.dsar.completed.v1`

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
```

## What you've learned

- Activate multiple competing packs.
- Trigger a real GDPR vs HIPAA erasure conflict.
- 6-step precedence application (step 1 hard-stop wins for HIPAA).
- Transparency report with legal basis + appeal pathway.
- DSAR finalization with partial-erasure bundle.
- Regulator audit replay against immutable pack version snapshot.
- Audit-chain verification of the full conflict-resolution flow.

Next tutorial: `tutorials/orchestrate-eu-dpia-with-cross-border-transfer.md` — initiate a DPIA for cross-border data transfer (PIPL Art 38 + GDPR Art 49) at paid on-prem-connected cell_topology.
