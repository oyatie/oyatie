---
doc_class: Onboarding
microservice: recordings
persona: compliance-officer + records-management
related_adrs: [ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Compliance Officer onboarding — first 5 working days on `recordings`

Audience: a new compliance officer or records-management engineer joining the `recordings` rotation. By Day-5 they will have: walked the retention-policy enforcement substrate, engaged + released a legal hold, processed an eDiscovery export, exercised the GDPR right-to-erasure flow against a held recording, and shadowed a SEC 17a-4(f) audit prep.

## Day 1 — Tour the substrate

1. Read `PRD.md` § retention + legal hold (∼ 45 min) + `decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md` + `decisions/ADR-RECORDINGS-0003-redaction-and-pii-policy.md` (∼ 90 min).
2. Open the Grafana folder `recordings`. Identify the boards: `recordings-retention-aging`, `recordings-legal-hold-active`, `recordings-redaction-coverage`, `recordings-ediscovery-queue`, `recordings-deletion-vs-hold-conflicts`.
3. Walk the runbook index. The on-call runbooks: `legal-hold-engaged.md`, `legal-hold-released.md`, `dsar-request-on-held-recording.md`, `sec-17a-4-worm-verification.md`, `ediscovery-export-corruption.md`, `retention-policy-update.md`, `cross-pack-evidence-transfer.md`.
4. Sit in on Wed's records-management handoff. Watch how the outgoing rotation reviews the past-week legal-hold engagements + eDiscovery throughput + DSAR-on-held-recording conflicts.

Acceptance: you can sketch the legal-hold lifecycle: engage → WORM-lock → all deletion attempts denied + audited → released → retention-policy resumes.

## Day 2 — Retention policy + tier behaviour

Read `decisions/ADR-RECORDINGS-0002`. The retention model:

- Per-tenant DEFAULT policy: 90 d hot tier + 7 y cold tier + auto-delete at 7 y.
- Per-pack OVERRIDE: SEC 17a-4(f) → 7 y WORM Glacier-Vault-Lock; HIPAA → 6 y default; KR 전자문서법 → 10 y default; EU-GDPR right-to-erasure interrupt-driven.
- Per-recording OVERRIDE via Cedar: `recordings::policy::override` action; the override must justify against the tenant's retention-policy ADR.

Walk a synthetic retention scenario:

```sh
oya recordings drill retention-aging \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --pack us-financial \
    --recording-ages 30d,89d,90d,91d,2557d  # 30d, 89d, 90d, 91d, 7y
```

The drill creates 5 synthetic recordings at those ages. Expected behaviour:

- 30 d: hot tier, fully accessible.
- 89 d: hot tier, fully accessible.
- 90 d: tier-transition triggered; this is the boundary case — recording is in flight to cold.
- 91 d: cold tier (Glacier-Vault-Lock WORM); read latency 3-5 h for restore.
- 2557 d (7 y - 1 d): cold tier, queued for deletion.
- 2557 d + 1 d: deleted; proof-of-erasure emitted.

Verify the deletion proof:

```sh
oya audit query --tenant drill-acme --since 1h --event-class recording_deleted
```

Acceptance: you can articulate the per-pack retention defaults from memory + the boundary cases.

## Day 3 — Legal hold engage + release

Read `runbooks/legal-hold-engaged.md`.

Engage a hold:

```sh
oya recordings legal-hold engage \
    --tenant drill-acme \
    --recording-id rec-456abc \
    --order-id order-2026-litigation-12 \
    --justification "Smith v. Acme Corp; recording responsive to RFP-7 of plaintiff's discovery requests" \
    --hold-until 2027-12-31
```

The engage step:

1. Cedar gate `recordings::legal_hold::engage` evaluated; allowed only for principals in the `records-officer` role.
2. Recording's deletion-policy WORM-locked; any subsequent deletion attempt is denied + audited.
3. The recording's retention policy is suspended; the auto-delete clock does not advance during the hold.
4. Audit event `legal_hold_engaged` emitted.

Try to delete the held recording (should fail):

```sh
oya recordings delete --tenant drill-acme --recording-id rec-456abc
```

Expected error: `legal_hold_active: order-2026-litigation-12 expires 2027-12-31; cannot delete`.

Audit event `legal_hold_deletion_denied` emitted.

Now release the hold:

```sh
oya recordings legal-hold release \
    --tenant drill-acme \
    --recording-id rec-456abc \
    --order-id order-2026-litigation-12 \
    --release-reason "litigation concluded; preservation no longer required" \
    --release-evidence ./settlement-agreement.pdf
```

Audit event `legal_hold_released` + the retention policy resumes.

Acceptance: hold engaged + verified to block deletion + released; you can articulate why the hold-until is required (defense-of-eternal-hold; hold-without-end-date is operationally suspect).

## Day 4 — eDiscovery export + GDPR-on-hold conflict

eDiscovery export (per IP-012):

```sh
oya recordings ediscovery export \
    --tenant drill-acme \
    --case-id case-smith-vs-acme \
    --custodian-user drill-user-z \
    --window 2024-01-01..2025-06-30 \
    --classes meet-recording,messenger-huddle-recording,manual-upload \
    --redaction-spec spec-attorney-eyes-only.yaml \
    --output ./case-smith-acme-export/
```

The export pipeline:

1. Resolves the custodian's recording corpus (per Cedar + ontology).
2. Applies the per-class filter.
3. Applies the redaction spec at the playback-overlay manifest layer (the underlying media is NOT redacted; the manifest applies the overlay).
4. Generates EDRM-XML-compliant manifest + per-recording Bates numbering.
5. Emits `ediscovery_export_completed` audit event.

Verify:

```sh
ls -lh ./case-smith-acme-export/
```

Expected contents: `EDRM.xml`, per-recording subdirectory with the recording bytes + `transcript.vtt` + `redaction-overlay.json` + a per-recording PDF cover-sheet with Bates ranges.

Now walk the GDPR-on-hold conflict drill. A tenant's user files a GDPR Art. 17 right-to-erasure for their recording, but the recording is on legal hold:

```sh
oya recordings drill dsar-on-held-recording \
    --tenant drill-acme \
    --user drill-user-z \
    --recording-id rec-456abc \
    --dsar-source gdpr-art-17
```

The system response per ADR-RECORDINGS-0002 § "Conflict resolution":

1. DSAR received → `dsar_received` audit event.
2. System checks legal-hold status → `legal_hold_active` for the recording.
3. DSAR cannot proceed to deletion; per GDPR Art. 17(3)(b) "for establishment, exercise or defence of legal claims", the legal hold legitimately overrides erasure.
4. System emits `dsar_blocked_by_legal_hold` audit event + notifies the data subject + the tenant's DPO.
5. The DSAR is queued for re-evaluation upon hold release.

Acceptance: you can articulate why GDPR Art. 17(3)(b) takes precedence over Art. 17(1) in this scenario + you understand what evidence the DPO must retain to defend this choice in a regulator audit.

## Day 5 — SEC 17a-4(f) audit prep + retention vault verification

Read `decisions/ADR-RECORDINGS-0002` § "SEC 17a-4(f) overlay" + 17 CFR § 240.17a-4(f) (the actual rule text; ∼ 30 min).

Walk the SEC audit prep:

```sh
oya recordings sec-17a-4 audit-prep \
    --tenant drill-acme \
    --pack us-financial \
    --window 2024-01-01..2025-12-31 \
    --output ./sec-17a-4-audit-2026.json
```

The audit-prep:

1. Enumerates all recordings classified as `class=communications-of-record` per the tenant's pack-us-financial classification.
2. Verifies WORM-lock invariant: each recording is in Glacier-Vault-Lock with the lock-policy + immutability flag set.
3. Verifies the audit chain: every `recording_ingested` event has a corresponding `worm_locked` event within ≤ 5 min.
4. Verifies the access log: every `recording_read` event has a corresponding Cedar decision + principal attribution.
5. Verifies the third-party-readability requirement (17 CFR § 240.17a-4(f)(3)(vi)): includes a SHA-256 hash chain readable by a non-affiliated auditor.

Expected output: a structured audit-evidence file the tenant can submit to FINRA examiners.

Acceptance: you can articulate why SEC 17a-4(f) requires WORM (not just retention) + you can explain the third-party-readability requirement + you can identify the audit-chain events that demonstrate compliance.

## What you've learned

- The retention policy lifecycle + per-pack overrides.
- The legal-hold engage + release flow + WORM enforcement.
- The eDiscovery export + EDRM-XML pipeline.
- The GDPR Art. 17 vs legal-hold conflict + the Art. 17(3)(b) escape clause.
- The SEC 17a-4(f) WORM + audit-trail + third-party-readability invariants.

Next week: cross-pack evidence transfer drill, redaction-override review, HIPAA BAA tenant onboarding shadow, KR 전자문서법 evidence preservation shadow.
