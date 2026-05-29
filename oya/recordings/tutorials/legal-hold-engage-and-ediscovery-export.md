---
doc_class: Tutorial
microservice: recordings
persona: compliance-officer + outside-counsel-integrator
date: 2026-05-20
doc_status: published
---

# Tutorial — Engage a legal hold + produce an EDRM-XML eDiscovery export

You will: identify a custodian's recording corpus, engage a litigation hold, prove that deletion is denied while held, generate an EDRM-XML export with attorney-eyes-only redaction, and verify the audit chain. Total time ≤ 60 minutes.

## Pre-requisites

- A paid-tier+ recordings cell.
- `oya-dev-cli` ≥ 1.42.0.
- A test tenant `drill-acme` + a synthetic custodian `drill-user-z` with > 5 recordings in the last 90 days.
- Cedar principal in the `records-officer` role for `drill-acme`.

## Step 1 — Identify the custodian's recording corpus (≤ 10 min)

```sh
oya recordings search \
    --tenant drill-acme \
    --custodian drill-user-z \
    --window 2024-01-01..2025-12-31 \
    --classes meet-recording,messenger-huddle-recording,manual-upload \
    --output corpus/drill-user-z-2024-2025.json
```

Output (truncated):

```json
{
  "custodian": "drill-user-z",
  "tenant": "drill-acme",
  "recordings": [
    {
      "recording_id": "rec-1a2b3c",
      "class": "meet-recording",
      "duration_seconds": 1842,
      "ingested_at": "2024-03-14T15:42:00Z",
      "retention_policy": "us-financial-7y-worm",
      "legal_hold_status": "none"
    },
    {
      "recording_id": "rec-4d5e6f",
      "class": "messenger-huddle-recording",
      "duration_seconds": 642,
      "ingested_at": "2024-07-22T09:15:00Z",
      "retention_policy": "us-financial-7y-worm",
      "legal_hold_status": "none"
    }
    // ... 8 more recordings
  ],
  "total_recordings": 10,
  "total_duration_seconds": 23_842
}
```

## Step 2 — Engage the legal hold (≤ 5 min)

```sh
oya recordings legal-hold engage \
    --tenant drill-acme \
    --corpus corpus/drill-user-z-2024-2025.json \
    --order-id order-2026-litigation-12 \
    --justification "Smith v. Acme Corp; corpus responsive to RFP-7 of plaintiff's discovery requests; engaged per outside-counsel-letter-of-2026-03-15.pdf" \
    --letter-attachment ./outside-counsel-letter-2026-03-15.pdf \
    --hold-until 2027-12-31 \
    --extending-trigger litigation-conclusion-OR-court-order
```

The engage step:

1. Cedar gate `recordings::legal_hold::engage` evaluates (allowed for principals in `records-officer` role).
2. Per recording in the corpus:
   - Sets `legal_hold_active=true`.
   - WORM-locks the recording (stored bytes immutable).
   - Suspends the retention policy clock.
   - Emits `legal_hold_engaged` audit event with the order_id + justification + letter-attachment hash.
3. The hold record is persisted to the legal-hold registry.

Expected output:

```
Legal hold engaged.
  Order ID: order-2026-litigation-12
  Recordings held: 10
  Hold expires: 2027-12-31 (or upon litigation conclusion / court order)
  Audit events emitted: 10 (one per recording)
  Letter attachment SHA-256: a1b2c3d4...
```

## Step 3 — Prove deletion is denied (≤ 3 min)

Attempt to delete one of the held recordings:

```sh
oya recordings delete --tenant drill-acme --recording-id rec-1a2b3c
```

Expected error:

```
Error: legal_hold_active
Recording: rec-1a2b3c
Hold order: order-2026-litigation-12
Hold expires: 2027-12-31
Justification: "Smith v. Acme Corp; corpus responsive to RFP-7..."

The delete operation was denied by the legal-hold gate.
An audit event (legal_hold_deletion_denied) has been emitted.
```

Verify the audit event:

```sh
oya audit query --tenant drill-acme --since 5m --recording-id rec-1a2b3c --event-class legal_hold_*
```

Expected: 1 `legal_hold_engaged` + 1 `legal_hold_deletion_denied`.

## Step 4 — Generate the EDRM-XML eDiscovery export (≤ 25 min)

Draft a redaction specification for "attorney-eyes-only" review:

```yaml
# spec-attorney-eyes-only.yaml
redaction_classes:
  pii_explicit:
    - ssn
    - credit_card
    - tax_id
    - drivers_license
  pii_implicit:
    - email
    - phone
    - physical_address
  phi:
    - hipaa_safe_harbor_18_identifiers
  attorney_eyes_only_overlay:
    apply: true
    overlay_kind: opaque-redaction-box
    transcript_token: "[REDACTED-AEO]"
  preserve_for_review:
    - speaker_id    # don't redact speaker labels; only PII within speech
    - timestamps
```

Generate the export:

```sh
oya recordings ediscovery export \
    --tenant drill-acme \
    --case-id case-smith-vs-acme \
    --corpus corpus/drill-user-z-2024-2025.json \
    --redaction-spec spec-attorney-eyes-only.yaml \
    --bates-prefix SMITH-ACME-DEPO \
    --bates-start 000001 \
    --output-format edrm-xml \
    --output ./export/case-smith-acme/
```

The export pipeline:

1. Cedar gate `recordings::ediscovery::export` evaluates.
2. For each recording, apply the redaction overlay (no media modification).
3. Generate per-recording WebVTT transcript with PII tokenised.
4. Assemble EDRM-XML manifest with per-recording entry, Bates range, redaction-spec hash.
5. Generate per-recording PDF cover-sheet with Bates ranges, custodian, duration, redaction summary.
6. Emit `ediscovery_export_completed` audit event with the export hash.

Expected wall-clock: ~ 25 min for 10 recordings (transcoding + redaction overlay generation; the export itself is dominated by transcript indexing).

Verify the output:

```sh
ls -lh ./export/case-smith-acme/
```

```
EDRM.xml                          (master manifest)
rec-1a2b3c/
    SMITH-ACME-DEPO-000001.mp4    (the recording bytes; overlay applied at player)
    SMITH-ACME-DEPO-000001.vtt    (redacted transcript)
    SMITH-ACME-DEPO-000001.pdf    (cover sheet; Bates ranges)
    SMITH-ACME-DEPO-000001.redaction-spec.json
rec-4d5e6f/
    SMITH-ACME-DEPO-000002.mp4
    SMITH-ACME-DEPO-000002.vtt
    SMITH-ACME-DEPO-000002.pdf
    SMITH-ACME-DEPO-000002.redaction-spec.json
...
SHA256SUMS                         (per-file hash; for the third-party-readability requirement)
```

## Step 5 — Verify EDRM-XML conformance (≤ 5 min)

```sh
oya recordings ediscovery validate --export ./export/case-smith-acme/
```

The validator checks against EDRM XML 1.2 schema + Bates uniqueness + per-recording file integrity.

Expected output:

```
EDRM-XML validation
  Schema: EDRM XML 1.2
  Recordings: 10
  Bates ranges: SMITH-ACME-DEPO-000001 … SMITH-ACME-DEPO-000010
  Bates uniqueness: PASS
  Per-recording integrity: PASS (10/10)
  Cross-reference manifest <-> per-recording: PASS
  Verdict: VALID

The export can be ingested by Relativity, Concordance, Ipro, or Disco.
```

## Step 6 — Hand the export to outside counsel (≤ 5 min)

Pack the export for transmission:

```sh
oya recordings ediscovery package \
    --export ./export/case-smith-acme/ \
    --encryption recipient-pgp:outside-counsel-pubkey.asc \
    --output case-smith-acme-export.tar.gz.pgp
```

The package:

1. Compresses to tar.gz.
2. PGP-encrypts to the recipient's public key (so only outside counsel can decrypt).
3. Computes SHA-256 of the package + signs with the tenant's records-officer key.
4. Generates a transmission receipt with the package hash, recipient identity, transmission timestamp.

Hand to outside counsel via your standard secure transmission channel.

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 1h --case-id case-smith-vs-acme
oya audit verify-chain --tenant drill-acme --since 1h
```

Expected events:

- `legal_hold_engaged` × 10 (Step 2).
- `legal_hold_deletion_denied` × 1 (Step 3).
- `redaction_overlay_generated` × 10 (Step 4).
- `ediscovery_export_started` × 1.
- `ediscovery_export_completed` × 1.
- `ediscovery_export_packaged` × 1 (Step 6).

All events Ed25519-signed; chain verified.

## What you've learned

- The legal-hold engage flow + WORM-lock enforcement.
- The defense-of-evidence invariant (held recordings cannot be deleted; attempts are audited).
- The EDRM-XML export shape + per-recording Bates numbering.
- The overlay-not-mutation redaction model.
- The cross-vendor portability of EDRM-XML.

Next tutorial: `tutorials/handle-dsar-on-recording-corpus.md` — process a GDPR right-to-erasure request against a recording corpus that includes held recordings.
