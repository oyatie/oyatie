---
doc_class: MigrationPlaybook
microservice: drive
vendor: Google Drive (Personal + Workspace Business/Enterprise)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Google Drive → oyatie drive

Audience: a team running Google Drive (Workspace Business or Enterprise) for organization file storage. Drivers: per-tenant CMK with per-file DEK envelope + sovereign-pack residency + cryptographic audit-chain + WORM compliance + 6× TCO reduction at 50k+ seat scale + tenant-controlled DLP enclave.

## Why this migration matters

Google Drive Business/Enterprise is excellent at:

- Best-in-class collaboration (Docs, Sheets, Slides real-time).
- Massive integration ecosystem (Google Workspace Marketplace).
- Mobile-first clients (Android native; iOS first-class).
- AI-assisted features (Smart Compose, search, summarize).
- Generous storage tiers.

oyatie drive adds:

- **Per-tenant CMK with rolling KEK and per-file DEK envelope** (per ADR-DRIVE-001) — Google's CMEK gives per-tenant key but doesn't expose KEK rotation or per-file DEK granularity.
- **FIPS 140-3 L3 HSM custody at compliance_pack** (Google has Cloud HSM but not in all Workspace tiers).
- **Cryptographic audit-chain** (Google's audit log is server-mutable).
- **Tenant-controlled DLP scanning enclave** (Google scans server-side).
- **Cross-tenant deal rooms without copying CMK** (Google sharing copies metadata between orgs).
- **Sovereign-pack residency** (KR-PIPA, EU-GDPR Art 9, US-HIPAA PHI, FedRAMP-High, CN-PIPL).
- **WORM immutability tier** (per ADR-DRIVE-0006) with SEC 17a-4 + HIPAA 6 y + SOX 7 y compliance.
- **6× TCO reduction** at 50k seats (oyatie paid ~ $1.1M vs Google Workspace Business ~ $7.4M annual).

The trade-off: Google Docs/Sheets/Slides real-time collaboration is hard to match at launch. oyatie's collaboration story uses the `design-collaboration` µservice (CRDT-based real-time editing) but the editor maturity is currently lower than Google Docs. Plan for parallel use during migration.

## Step 1 — Inventory the Google Drive estate (≤ 1-2 weeks)

```bash
# Google Workspace Admin → Drive Audit Log → Export to BigQuery
# Or use Google Workspace Migrate (free for Workspace Enterprise)
gwm-cli export drive \
    --organization acme-corp.example \
    --since 2020-01-01 \
    --output ./gdrive-export/

# Or use Google Drive API for programmatic export
python3 -m google_drive_export \
    --service-account-key ./sa-key.json \
    --customer-id C03dasdf12 \
    --output ./gdrive-export/
```

Document:

- User count + Google Workspace tier (Business Starter / Standard / Plus / Enterprise).
- Total drive volume (typical: 10 TB-10 PB).
- Shared Drives count + per-drive permissions.
- Google Workspace Marketplace apps installed (e.g., DocuSign, Adobe Sign, Lucidchart).
- Active Vault retention policies + legal holds.
- Domain-wide Delegation grants (third-party apps with Drive access).
- DLP rules + Sensitivity Labels (if using Workspace Plus+).
- External sharing patterns (which domains, how often).
- Custom Drive Labels (Workspace Plus+).
- Google Docs / Sheets / Slides count (Google's native format vs uploaded files).

Typical mid-size: 1k-50k users, 50 TB-10 PB, 100-10k Shared Drives, 50-500 Marketplace apps.

## Step 2 — Map Google Drive concepts to oyatie drive (≤ 1 week)

| Google Drive concept | oyatie drive equivalent |
|---|---|
| User Drive (My Drive) | Per-user mailbox folder + Cedar ownership |
| Shared Drive | Tenant folder with Cedar role-mapped permissions |
| Domain (organization) | Tenant |
| File | File (per-version envelope) |
| Folder | Folder with materialized-path tree |
| Sharing link | Ed25519-signed share-link capability (per ADR-DRIVE-0003) |
| Permissions (Owner/Editor/Commenter/Viewer) | Cedar permission roles |
| Vault retention | `compliance` µservice pack retention class |
| Vault legal hold | Legal-hold lock on file (per ADR-DRIVE-0006) |
| Sensitivity Labels | data_class + retention_class on file metadata |
| Google Docs/Sheets/Slides (native) | Convert to MS Office formats during import; future: native CRDT editor via `design-collaboration` µservice |
| Domain-wide Delegation | Per-app Cedar role grant |
| Backup & Sync (mobile) | oyatie drive sync client |
| Drive File Stream | oyatie drive sync (offline + on-demand) |
| Marketplace app | oyatie plugin SDK |

## Step 3 — Data migration (≤ 4-16 weeks per PB)

```sh
oya drive migrate import-google-drive \
    --tenant acme-corp \
    --gdrive-export-dir ./gdrive-export/ \
    --convert-google-docs-to docx \
    --convert-google-sheets-to xlsx \
    --convert-google-slides-to pptx \
    --map-shared-drives-to-folders /Shared/{drive_name} \
    --preserve-sharing-links false \
    --preserve-modification-times true \
    --include-vault-archive true \
    --throttle-rate 500-mb-per-sec
```

The migration:

1. Creates oyatie tenants from Google Workspace domains.
2. Creates oyatie principals from Google users (preserve email + display name).
3. Imports Shared Drives → `/Shared/{drive_name}` folders.
4. Imports user My Drives → `/Users/{user}` folders.
5. Converts Google Docs/Sheets/Slides to MS Office formats (using LibreOffice headless) during import.
6. **Re-encrypts ALL files under oyatie envelope** during import (Google can't export CMEK-wrapped keys).
7. Imports sharing permissions → Cedar roles (preserve owner/editor/viewer mapping).
8. Imports file versions (Google retains last 100 versions; oyatie imports all).
9. Imports Vault archives → oyatie's WORM immutability tier.
10. **Does NOT preserve sharing-link URLs** (Google's signed URLs differ from oyatie's Ed25519 capability; new share-links must be issued).

Backfill rate ~ 500 MB/sec at paid. 1 PB → ~ 23 days.

Verify post-import counts:

```sh
oya drive tenant stats --tenant acme-corp
# Output:
#   total_files: 8 421 932
#   total_size: 1.04 PB
#   folder_count: 234 821
#   shared_drives_mapped: 1 240
#   vault_archived: 142 893 (WORM)
#   imported_from: google-drive
```

Cross-check against Google Workspace audit export's per-user / per-shared-drive counts.

## Step 4 — User identity migration (≤ 1-2 weeks)

```sh
# Configure identity µservice OIDC federation with Google Workspace IdP
oya identity oidc-federation configure \
    --tenant acme-corp \
    --idp google-workspace \
    --google-workspace-customer-id C03dasdf12 \
    --client-id <google-oauth-client-id> \
    --client-secret-bao-ref secret/acme-corp/identity/google-oauth-client-secret

# Or migrate to a primary IdP (Okta, Entra ID, etc.) if planning to decommission Google Workspace
oya identity oidc-federation configure \
    --tenant acme-corp \
    --idp okta-acme-corp \
    ...

# Enable passkey-primary
oya identity tenant update \
    --tenant acme-corp \
    --auth-policy passkey-primary
```

## Step 5 — DLP + sensitivity label migration (≤ 2-4 weeks)

Google Workspace Plus+ has DLP rules + Sensitivity Labels. Map to oyatie:

```sh
oya drive migrate dlp-rules-import \
    --tenant acme-corp \
    --google-dlp-export ./gdrive-export/dlp-rules.json
# Output:
#   imported_rules: 47
#   oyatie_dlp_pack_overlay: dom_acme_dlp_v1
#   sensitivity_label_map:
#     - "Confidential" → data_class=PII_SENSITIVE
#     - "Highly Confidential" → data_class=PII_FINANCIAL_SENSITIVE
#     - "Public" → data_class=PUBLIC
```

Configure tenant-controlled DLP enclave (paid):

```sh
oya drive dlp-enclave configure \
    --tenant acme-corp \
    --enclave-cluster cloud-hypervisor://acme-dlp-enclave-us-east-1 \
    --scan-pre-encryption true \
    --classifier-version 2026q2
```

## Step 6 — Shadow run + cutover (≤ 4-12 weeks)

Run BOTH Google Drive + oyatie in parallel:

Phase 1 (weeks 1-4): Read-only Google Drive; new files go to oyatie. Users use Google Drive client + oyatie web for new content.
Phase 2 (weeks 5-8): Migrate Shared Drives one team at a time.
Phase 3 (weeks 9-12): Migrate user My Drives.

After phase 3 begins cutover:

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.drive_substrate.cut_over \
    --payload '{"from":"google-drive","to":"oyatie","cutover_at":"2026-08-15T14:00:00Z"}'
```

## Step 7 — Google Drive decommission (≤ 180-365 d post-cutover)

After ≥ 180 d:

- Export final Vault state for compliance retention.
- Cancel Google Workspace contract.
- Retain Vault archive read-only access for legal-hold duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Google Docs/Sheets/Slides real-time collaboration not 1:1 mapped | High | Use design-collaboration µservice CRDT editor; parallel use of Google Docs during transition; user training |
| Format conversion (Google Docs → docx) may lose formatting | Medium | Spot-check converted files; provide format-tweak service; some advanced features (Comments, Suggestion mode) preserved 1:1 |
| Marketplace app gap | High | Pre-audit; port top-20 apps to plugin SDK before cutover |
| Sharing-link URL changes (Google → oyatie) | High | Cannot preserve Google's signed URLs; bulk re-issue oyatie share-links; users notified to use new URLs |
| Domain-wide Delegation third-party apps | Medium | Re-grant via Cedar roles; some apps may not have oyatie integration yet |
| Backup & Sync mobile client transition | Medium | oyatie sync client side-by-side; rolling cutover by device |
| Vault retention conversion | Medium | Map via `compliance` µservice pack retention class; legal-hold preserved |
| Google Drive Stream (on-demand sync) | Medium | oyatie sync supports on-demand; same UX |
| Drive File Stream (NTFS-like) on macOS/Windows | Low | oyatie sync provides equivalent FUSE-based on macOS, ProjectedFS on Windows |
| File size limits (Google: 5 TiB/file) | Low | oyatie supports up to 50 TiB/file (paid tier) |
| Permission model differences | Low | Cedar role mapping is finer-grained than Google's; some manual review for edge cases |
| Custom Drive Labels (Plus+) | Medium | Map to oyatie data_class + custom-tag system |
| External sharing patterns (auto-share-to-domain) | Medium | Re-implement as tenant-pair federation grants + cross-tenant deal rooms |
| Comments + Suggestions on Google Docs | Medium | Preserved in conversion; re-rendered in oyatie's editor |
| Mobile offline cache | Low | oyatie mobile sync supports offline; cache encrypted under device-DEK |
| Sensitivity Labels (Workspace Plus+) | Medium | Map to data_class via DLP-rule-import; existing labeled files re-classified |
| Google Workspace Compliance Reports | Low | oyatie compliance dashboards + audit-chain query |
| API integrations (third-party services calling Google Drive API) | High | Port to oyatie drive REST/JMAP API; SDK shim provided for major languages |
| File ownership transfers during user offboarding | Medium | Map to oyatie ownership transfer ceremony per ADR-DRIVE-001 |
| Encrypted file content (Google can decrypt under CMEK) | Medium | oyatie cannot import CMEK-wrapped keys; ALL files re-encrypted during import — server has brief plaintext window during migration (mitigated by network isolation) |
| Cloud HSM compatibility (Google's Cloud HSM differs from OpenBao + Thales Luna) | Low | oyatie supports BYOK via AWS/GCP/Azure KMS; tenants can keep CMK in Google Cloud HSM if desired |
| Workspace Marketplace billing for users | Low | Direct billing tenant after Google decommission |
