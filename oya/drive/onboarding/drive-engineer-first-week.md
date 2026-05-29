---
doc_class: Onboarding
microservice: drive
persona: drive-engineer + storage-platform-engineer + encryption-engineer
related_adrs: [ADR-DRIVE-001, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
date: 2026-05-20
doc_status: published
---

# Drive Engineer onboarding — first 5 working days on `drive`

Audience: a new drive engineer, storage platform engineer, or encryption engineer joining the `drive` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, uploaded their first encrypted file (CMK/KEK/DEK envelope), verified preview generation in sandbox, exercised KEK rotation + lazy rewrap, walked a cryptoshred ceremony dry-run, and shadowed cross-tenant share-link issuance.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the five-vendor displacement + envelope-encryption doctrine.
2. Read `ARCHITECTURE.md` § file-store + § envelope-encryption + § preview-sandbox + § cross-tenant-deal-room (∼ 60 min).
3. Read `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md` end-to-end (∼ 50 min). The binding architecture.
4. Read `decisions/ADR-DRIVE-0001..0006` (object storage substrate, chunking, share-link, encryption at rest, preview sandboxing, immutability) (∼ 40 min total).
5. Read `feedback_oyatie_is_a_tenant_doctrine` + `tenant_as_universal_scoping_primitive` (per project memory).
6. Open the Grafana folder `drive`. Reference boards: `drive-envelope-unwrap-latency`, `drive-rewrap-backlog-versions`, `drive-cryptoshred-blocked-total`, `drive-envelope-corruption-total`, `drive-upload-throughput`, `drive-preview-generation-latency`, `drive-share-link-issuance-rate`.
7. Walk `runbooks/README.md`. The on-call runbooks: `object-storage-degraded.md`, `upload-multipart-stuck.md`, `share-link-takeover-incident.md`, `immutability-tier-violation.md`, `kek-rotation-stuck.md`, `cryptoshred-blocked.md`, `preview-sandbox-crash.md`, `dlp-enclave-degraded.md`.
8. Sit in on the Wednesday drive-substrate handoff. Watch outgoing rotation review the past-week envelope unwrap p95 + rewrap backlog + cryptoshred drill.

Acceptance: you can sketch the upload path: client → drive-api → DLP scan (pre-encryption) → virus scan (pre-encryption) → client generates per-file DEK → DEK wrapped under tenant KEK (active epoch) → KEK wrapped under tenant CMK in OpenBao → object bytes encrypted with DEK + AAD → SeaweedFS write → `FileVersionEnvelope` insert → audit-chain `EVT-DRIVE-DEK-WRAPPED`. And the download path: Cedar `drive::file::decrypt` → fetch envelope row → OpenBao unwrap KEK (≤ 60 s lease) → unwrap DEK → SeaweedFS read → decrypt with DEK + AAD → stream to client → audit-chain emission.

## Day 2 — demo_trial cell bootstrap + first encrypted upload

```sh
cargo run -p oya-dev-cli -- drive bootstrap \
    --profile demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/drive \
    --seaweed-endpoint http://drill-seaweed-syd-1:8333 \
    --seaweed-master-endpoint http://drill-seaweed-syd-1:9333 \
    --opensearch-endpoint http://drill-opensearch-syd-1:9200 \
    --openbao-endpoint https://drill-openbao-syd-1:8200 \
    --openbao-mount drive \
    --clamav-endpoint http://drill-clamav-syd-1:3310 \
    --preview-sandbox-cluster cloud-hypervisor://drill-preview-syd-1 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 18 min. Verify:

```sh
oya drive health --cell drill-syd-1
# Expected:
#   postgres.file_metadata: up (lag_ms=14)
#   postgres.file_version_envelope: up
#   seaweedfs.volume-servers: up (5 volumes; EC 8+3)
#   opensearch.drive-index: up
#   openbao.cmk: up (active_tenants=0)
#   clamav: up (signatures_last_updated=2026-05-20T08:00:00Z)
#   preview-sandbox: up (cloud-hypervisor; idle_pods=4)
#   audit-chain.emit: up
```

Create a tenant + CMK:

```sh
oya drive tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Drive" \
    --pack-set default \
    --cmk-provider openbao   # default; or "aws-kms" / "gcp-kms" / "azure-key-vault" at paid
# Output:
#   tenant_id: drill-acme
#   cmk_id: cmk_drill_acme_001
#   cmk_state: active
#   home_cell: drill-syd-1

oya drive tenant kek-init \
    --tenant drill-acme \
    --kek-epoch 1 \
    --algorithm AES-256-GCM
# Output:
#   kek_epoch: 1
#   kek_state: active
#   activates_at: 2026-05-20T14:32:17Z
#   audit_event_id: ae_drive_kek_init_001
```

Upload your first encrypted file:

```sh
echo "Hello, encrypted world!" > ./test.txt
oya drive file upload \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --folder / \
    --file-path ./test.txt \
    --name test.txt \
    --content-type text/plain
# Output:
#   file_id: f_drill_001
#   version_id: v_drill_001_1
#   object_ref: seaweedfs://drill-syd-1/3,01637037d6
#   kek_epoch: 1
#   cmk_id: cmk_drill_acme_001
#   dek_wrap_algorithm: AES-256-GCM-Keywrap
#   aad_hash: blake3:7c4a2b8e9f...
#   audit_event_id: ae_drive_dek_wrapped_001
```

Verify the envelope row (no plaintext key material visible):

```sh
oya drive file envelope-show --tenant drill-acme --file f_drill_001 --version v_drill_001_1
# Output:
#   {
#     "tenant_id": "drill-acme",
#     "file_id": "f_drill_001",
#     "version_id": "v_drill_001_1",
#     "object_ref": "seaweedfs://drill-syd-1/3,01637037d6",
#     "dek_ciphertext_b64": "...<base64 wrapped DEK>...",  # only ciphertext, never plaintext
#     "kek_epoch": 1,
#     "cmk_id": "cmk_drill_acme_001",
#     "aad_hash": "blake3:7c4a2b8e9f...",
#     "algorithm": "AES-256-GCM",
#     "created_at": "2026-05-20T14:32:17Z"
#   }
#   dek_plaintext: NEVER (per ADR-DRIVE-001 § Static check)
```

Download + decrypt:

```sh
oya drive file download \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --file f_drill_001 \
    --output ./downloaded.txt
# Cedar evaluates drive::file::decrypt
# OpenBao unwraps KEK via 60s lease
# Server decrypts DEK + payload with AAD verification
# Output:
#   downloaded_bytes: 24
#   verification: passed (AAD hash matches)
#   audit_event_id: ae_drive_file_downloaded_001

cat ./downloaded.txt
# Output: Hello, encrypted world!
```

Acceptance: cell bootstrap; tenant + CMK + KEK + first file round-trip; envelope verified.

## Day 3 — Preview generation in Cloud Hypervisor sandbox + folder hierarchy

Upload a PDF (or larger document):

```sh
oya drive file upload \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --folder / \
    --file-path ./sample.pdf \
    --name "Sample report.pdf" \
    --content-type application/pdf
# Output: file_id=f_drill_002, version_id=v_drill_002_1
```

Generate preview (per ADR-DRIVE-0005 sandbox path):

```sh
oya drive file preview-generate \
    --tenant drill-acme \
    --file f_drill_002 \
    --preview-type thumbnail \
    --size 256x256
# Behind the scenes:
#   1. Cedar drive::file::preview ✓
#   2. Server unwraps DEK + downloads payload from SeaweedFS
#   3. Server spawns Cloud Hypervisor sandbox pod with the plaintext
#   4. LibreOffice + ImageMagick render thumbnail
#   5. Sandbox writes thumbnail to encrypted preview cache (same envelope model)
#   6. Sandbox pod terminated; memory zeroed
# Output:
#   preview_id: pv_drill_002_thumb_001
#   preview_version_id: v_drill_002_pv_thumb_1
#   preview_envelope: same kek_epoch + cmk_id as parent
#   sandbox_session_duration: 1.2s
#   audit_event_id: ae_drive_preview_generated_001
```

The preview is itself encrypted under the same envelope model (per ADR-DRIVE-001 Constraint DRIVE-C14 — preview must not be weaker encryption than originals).

Build a folder hierarchy + move files:

```sh
oya drive folder create --tenant drill-acme --user u-alice@drill.test --path /Reports/Q2-2026
oya drive folder create --tenant drill-acme --user u-alice@drill.test --path /Reports/Q3-2026

oya drive file move \
    --tenant drill-acme \
    --file f_drill_002 \
    --to-folder /Reports/Q2-2026
# Output:
#   moved: f_drill_002 → /Reports/Q2-2026/Sample report.pdf
#   audit_event_id: ae_drive_file_moved_001
```

Acceptance: preview generated in sandbox; preview encrypted under same envelope; folder hierarchy built.

## Day 4 — KEK rotation + lazy rewrap

Trigger KEK rotation (drill simulating 30-day cadence):

```sh
oya drive kek rotate \
    --tenant drill-acme \
    --new-epoch 2 \
    --reason scheduled
# Cedar requires tenant admin step-up + no active incident freeze
# Output:
#   from_epoch: 1
#   to_epoch: 2
#   new_kek_state: active
#   old_kek_state: retiring
#   rewrap_eligible_versions: 2 (f_drill_001 + f_drill_002 + preview)
#   audit_event_id: ae_drive_kek_rotated_001
```

Start lazy rewrap (priority: high-risk + hot files first):

```sh
oya drive rewrap-job start \
    --tenant drill-acme \
    --from-epoch 1 \
    --to-epoch 2 \
    --priority hot-and-high-risk
# Output:
#   job_id: rj_drill_001
#   eligible_versions: 2
#   estimated_duration: 4s
#   priority: hot-and-high-risk

# Wait for completion
oya drive rewrap-job watch --job rj_drill_001
# Output (streamed):
#   rj_drill_001: rewrapped 1/2 (50%)
#   rj_drill_001: rewrapped 2/2 (100%)
#   rj_drill_001: completed
#   duration: 3.8s
#   audit_event_id: ae_drive_rewrap_completed_001
```

Verify version envelope updated:

```sh
oya drive file envelope-show --tenant drill-acme --file f_drill_001 --version v_drill_001_1
# Output: kek_epoch: 2 (rewrapped under new KEK)
# Note: version_id + object_ref + aad_hash UNCHANGED (per ADR-DRIVE-001 § Implementation Notes: object payload bytes are stable)
```

Walk the kek-rotation-stuck runbook. Read `runbooks/kek-rotation-stuck.md`. Scenario: rewrap-job hangs because OpenBao briefly unavailable. Runbook covers:

1. Identify from `drive-rewrap-backlog-versions` panel (stalled count).
2. Check OpenBao health.
3. If OpenBao recovers: rewrap auto-resumes (per ADR-DRIVE-001 § Implementation Notes rollback path: idempotent).
4. If OpenBao persistently down: KEK rotation paused; old KEK pointer remains active.

Acceptance: KEK rotation completed; lazy rewrap completed; runbook walked.

## Day 5 — Share-link issuance + cryptoshred ceremony dry-run

Issue a signed share-link capability (paid feature; shadowed at demo_trial):

```sh
oya drive share-link create \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --file f_drill_002 \
    --permissions viewer \
    --expires-at 2026-06-20T00:00:00Z \
    --max-views 10 \
    --watermark-policy email-tagged
# Output:
#   share_link_id: sl_drill_001
#   share_link_url: https://drill.oyatie.local/s/EyJ0ZW5hbnRfaWQiOiJkcmlsbC1hY21lIiwiZmlsZV9pZCI6ImZfZHJpbGxfMDAyIiwicGVybSI6InZpZXdlciIsImV4cCI6IjIwMjYtMDYtMjBUMDA6MDA6MDBaIiwic2lnIjoiZWQyNTUxOTo3YzRhMmI4ZTlmLi4uIn0
#   token_algorithm: ed25519-jws
#   audit_event_id: ae_drive_share_link_created_001
```

The share-link URL is an Ed25519-signed JWT containing scope, expiration, max-views, and watermark policy. Receivers don't need an oyatie account to view (per ADR-DRIVE-0003).

Cryptoshred ceremony dry-run (compliance_pack feature; shadowed at demo_trial):

```sh
# Tenant offboarding triggers cryptoshred plan
oya drive cryptoshred plan create \
    --tenant drill-acme \
    --reason "tenant offboarding test" \
    --legal-hold-clearance-ref lhc_drill_001 \
    --retention-clearance-ref rc_drill_001 \
    --scheduled-destroy-at 2026-06-20T00:00:00Z
# Cedar evaluates:
#   - drive::tenant::cryptoshred ✓
#   - legal-hold clearance present ✓
#   - retention clearance present ✓
#   - no active legal-hold on tenant ✓
# Output:
#   cryptoshred_plan_id: csp_drill_001
#   tenant_id: drill-acme
#   cmk_id: cmk_drill_acme_001
#   scheduled_destroy_at: 2026-06-20T00:00:00Z
#   audit_event_id: ae_drive_cryptoshred_scheduled_001

# Verify the plan
oya drive cryptoshred plan show --plan csp_drill_001
# Output: state=scheduled; blockers=none

# The actual destroy happens at the scheduled time; before then, plan can be revoked
oya drive cryptoshred plan revoke --plan csp_drill_001 --reason "drill complete"
```

Acceptance: share-link issued + signed; cryptoshred plan created + revoked; audit-chain verified.

## What you've learned

- demo_trial bootstrap + CMK + KEK + DEK envelope per ADR-DRIVE-001.
- File upload + download with envelope verification.
- Preview generation in Cloud Hypervisor sandbox (per ADR-DRIVE-0005).
- KEK rotation + lazy rewrap with idempotent jobs.
- Share-link signed capabilities (per ADR-DRIVE-0003).
- Cryptoshred plan with clearance gates.

Next week: paid promotion (content-defined chunking + delta sync + signed share-link capabilities at scale), paid tour (WORM immutability + evidence-vault export + cross-tenant deal rooms + transit signing), compliance_pack tour (FIPS 140-3 L3 HSM + per-pack residency + cryptoshred ceremony rehearsal), and your first production shadow on tenant CMK rotation approval.
