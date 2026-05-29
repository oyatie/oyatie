---
doc_class: Tutorial
microservice: drive
persona: drive-engineer + storage-platform-engineer
related_adrs: [ADR-DRIVE-001, ADR-DRIVE-0002, ADR-DRIVE-0003]
date: 2026-05-20
doc_status: published
---

# Tutorial — Upload an encrypted file, rotate the tenant KEK, and verify lazy rewrap

You will: upload a file with per-file DEK + KEK + CMK envelope, verify the envelope row, rotate the tenant KEK, trigger lazy rewrap, verify the rewrap preserves payload + AAD, share via signed share-link, and verify audit-chain emissions. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant on paid tier (`tenant-class-adoption/tenant-class-adoption-record.md`).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `drive_admin` Cedar role for KEK rotation.
- A drive_member principal for file operations.
- OpenBao with FIPS 140-3 L2 HSM auto-unseal (per ADR-DRIVE-001).

## Step 1 — Verify tenant CMK + initial KEK state (≤ 5 min)

Inspect the tenant CMK:

```sh
oya drive tenant cmk-show --tenant acme-corp
# Output:
#   tenant_id: acme-corp
#   cmk_id: cmk_acme_corp_001
#   provider: openbao
#   openbao_ref: secret/acme-corp/drive/cmk/cmk_acme_corp_001
#   state: active
#   home_cell: prod-us-east-1
#   created_at: 2025-12-15T08:32:17Z
```

Inspect the initial KEK epoch:

```sh
oya drive tenant kek-list --tenant acme-corp
# Output:
#   - kek_epoch: 1
#     cmk_id: cmk_acme_corp_001
#     state: active
#     activates_at: 2025-12-15T08:32:17Z
#     algorithm: AES-256-GCM-Keywrap
```

## Step 2 — Upload a file (≤ 10 min)

Generate a sample file with substantial content (10 MB):

```sh
dd if=/dev/urandom of=./large-doc.pdf bs=1M count=10
```

Upload with full envelope encryption:

```sh
oya drive file upload \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --folder /Reports/Q2-2026 \
    --file-path ./large-doc.pdf \
    --name "Q2 2026 Financial Report.pdf" \
    --content-type application/pdf \
    --tags confidential,financial \
    --data-class PII_FINANCIAL_SENSITIVE \
    --retention-class audit_7y
# Behind the scenes:
#   1. Cedar drive::file::upload ✓
#   2. DLP + virus scan (pre-encryption) ✓
#   3. Client splits into content-defined chunks (FastCDC, target 4 MiB)
#   4. For each chunk: random DEK; wrap DEK with active KEK (epoch 1); encrypt chunk with DEK + AAD
#   5. AAD = tenant_id || file_id || version_id || object_digest || retention_class || data_class
#   6. Chunks uploaded to SeaweedFS
#   7. FileVersionEnvelope row inserted per chunk
#   8. File manifest references chunk hashes
# Output:
#   file_id: f_acme_q2_report_001
#   version_id: v_acme_q2_report_001_1
#   chunks: 3 (4.0 MiB + 4.0 MiB + 2.0 MiB)
#   total_size: 10 485 760 bytes
#   kek_epoch: 1
#   cmk_id: cmk_acme_corp_001
#   retention_class: audit_7y
#   audit_event_id: ae_drive_file_uploaded_001
```

Verify the envelope rows:

```sh
oya drive file envelope-show \
    --tenant acme-corp \
    --file f_acme_q2_report_001 \
    --version v_acme_q2_report_001_1
# Output (one envelope row per chunk):
#   [
#     {
#       "chunk_index": 0,
#       "object_ref": "seaweedfs://prod-us-east-1/3,01637037d6",
#       "dek_ciphertext_b64": "<base64>",
#       "kek_epoch": 1,
#       "aad_hash": "blake3:7c4a2b8e9f...",
#       "algorithm": "AES-256-GCM"
#     },
#     ... (3 chunks total)
#   ]
```

## Step 3 — Download + verify (≤ 5 min)

```sh
oya drive file download \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --file f_acme_q2_report_001 \
    --output ./downloaded.pdf
# Cedar drive::file::decrypt ✓
# OpenBao unwraps KEK epoch 1 (≤ 60s lease)
# Per chunk: unwrap DEK, decrypt chunk with AAD verification
# Output:
#   downloaded_bytes: 10 485 760
#   verification: passed (all 3 chunks AAD-verified)
#   audit_event_id: ae_drive_file_downloaded_001

# Verify byte-identical
sha256sum ./large-doc.pdf ./downloaded.pdf
# Both hashes match
```

## Step 4 — Rotate KEK to epoch 2 (≤ 5 min)

```sh
oya drive kek rotate \
    --tenant acme-corp \
    --new-epoch 2 \
    --reason scheduled \
    --requested-by u-drive-admin@acme-corp.com
# Cedar evaluates:
#   - drive::key::rotate_kek ✓
#   - Requester has admin step-up at acr ≥ aal3_hardware_bound ✓
#   - No active incident freeze ✓
# Output:
#   from_epoch: 1 → to_epoch: 2
#   new_kek_state: active
#   old_kek_state: retiring (still valid for unwraps until rewrap complete + 7d soak)
#   rewrap_eligible_versions: 1 (3 chunks under file f_acme_q2_report_001)
#   audit_event_id: ae_drive_kek_rotated_001
```

The new KEK epoch is active for NEW uploads. Existing files still reference epoch 1 in their envelope rows.

## Step 5 — Trigger lazy rewrap (≤ 10 min)

```sh
oya drive rewrap-job start \
    --tenant acme-corp \
    --from-epoch 1 \
    --to-epoch 2 \
    --priority hot-and-high-risk \
    --max-rate-per-sec 2000
# Cedar drive::key::rewrap ✓
# Output:
#   job_id: rj_acme_001
#   eligible_versions: 1 (3 chunks)
#   priority: hot-and-high-risk
#   max_rate: 2000 versions/sec
#   estimated_duration: 2s

oya drive rewrap-job watch --job rj_acme_001
# (streamed output)
#   rj_acme_001: rewrapped chunk 1/3 (kek_epoch 1 → 2)
#   rj_acme_001: rewrapped chunk 2/3
#   rj_acme_001: rewrapped chunk 3/3
#   rj_acme_001: completed
#   duration: 1.8s
#   audit_event_id: ae_drive_rewrap_completed_001
```

## Step 6 — Verify rewrap preserved payload + AAD (≤ 5 min)

```sh
oya drive file envelope-show \
    --tenant acme-corp \
    --file f_acme_q2_report_001 \
    --version v_acme_q2_report_001_1
# Output (compare against Step 2):
#   chunk_index: 0
#     kek_epoch: 2   # was 1; rewrapped
#     object_ref: seaweedfs://prod-us-east-1/3,01637037d6   # UNCHANGED
#     aad_hash: blake3:7c4a2b8e9f...   # UNCHANGED
#     algorithm: AES-256-GCM           # UNCHANGED
#   ... (same for chunks 1, 2)
```

Per ADR-DRIVE-001 § Implementation Notes: rewrap is idempotent because object payload bytes + version_id are stable. Only `dek_ciphertext` + `kek_epoch` change.

Verify download still works:

```sh
oya drive file download \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --file f_acme_q2_report_001 \
    --output ./downloaded-post-rewrap.pdf
# OpenBao unwraps KEK epoch 2 (the new active epoch)
# Output: downloaded_bytes: 10 485 760; verification: passed

sha256sum ./large-doc.pdf ./downloaded-post-rewrap.pdf
# Hashes match (payload unchanged)
```

## Step 7 — Issue a share-link (paid feature) (≤ 5 min)

```sh
oya drive share-link create \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --file f_acme_q2_report_001 \
    --permissions viewer \
    --expires-at 2026-08-20T00:00:00Z \
    --max-views 5 \
    --watermark-policy email-tagged \
    --require-email-verification true
# Cedar drive::share::create ✓
# Output:
#   share_link_id: sl_acme_001
#   share_link_url: https://acme.oyatie.local/s/<base64-encoded-ed25519-jws>
#   expires_at: 2026-08-20T00:00:00Z
#   max_views: 5
#   current_views: 0
#   audit_event_id: ae_drive_share_link_created_001

# Receiver clicks the link
curl -X GET "https://acme.oyatie.local/s/<token>?email=bob@external.example"
# Response:
#   - Server verifies Ed25519 signature ✓
#   - Checks expiration ✓
#   - Checks current_views < max_views ✓
#   - Records view; increments current_views
#   - Generates watermarked preview tagged with bob@external.example
#   - Returns the preview stream (or full file per permissions)
```

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "drive.*" --since 60m
```

Expected events for our flow:

- `drive.tenant.kek.initialized.v1`
- `drive.file.uploaded.v1`
- `drive.file.dek.wrapped.v1` (× 3; one per chunk)
- `drive.file.downloaded.v1` (× 2; pre + post rewrap)
- `drive.kek.rotated.v1`
- `drive.rewrap.started.v1`
- `drive.rewrap.completed.v1`
- `drive.file.dek.wrapped.v1` (× 3; rewrapped chunks under epoch 2)
- `drive.share-link.created.v1`
- `drive.share-link.viewed.v1`

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 9 — Cleanup (optional)

Retire the old KEK epoch (after 7-day soak in production; instant in drill):

```sh
oya drive kek retire \
    --tenant acme-corp \
    --kek-epoch 1 \
    --confirm-rewrap-complete true \
    --skip-soak-check true   # drill only
# Cedar drive::key::retire ✓
# Output:
#   kek_epoch: 1
#   state: retired
#   rewrap_backlog_versions: 0 (all versions migrated to epoch 2)
#   audit_event_id: ae_drive_kek_retired_001
```

## What you've learned

- File upload with full CMK / KEK / DEK envelope (per ADR-DRIVE-001).
- Content-defined chunking with per-chunk envelope (per ADR-DRIVE-0002).
- KEK rotation triggering lazy rewrap.
- Rewrap idempotency + payload + AAD preservation.
- Ed25519-signed share-link capabilities (per ADR-DRIVE-0003).
- Audit-chain verification of the full envelope-lifecycle flow.

Next tutorial: `tutorials/cross-tenant-deal-room-without-cmk-copy.md` — set up a cross-tenant deal room where the recipient tenant gets bounded access to ciphertext WITHOUT copying the originating tenant's CMK (paid tier).
