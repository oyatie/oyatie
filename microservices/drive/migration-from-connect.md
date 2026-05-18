---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: drive
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-DRIVE accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
related_specs: [/specs/microservices/drive.json, /specs/microservices/workspace/drive.json]
owner_team: axis-drive
date: 2026-05-17
doc_status: published
---

# Migration: `oya-connect-drive-*` → `oya-drive-*`

This document applies the Strangler Pattern from the agent-skills `deprecation-and-migration` skill to the **drive** µservice. It is the consumer-facing companion to ADR-0134 (cross-µservice migration policy) and ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-proven in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-drive-*` crate family under `microservices/drive/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-DRIVE accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger) |
| Reason | ADR-0132 no-suite forward-policy + ADR-0130 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 11-pack-overlay program (per ADR-0133) is only addressable at µservice granularity, not at suite granularity |
| Migration owner (Churn Rule) | axis-drive |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 11 bounded-contexts of the `drive` µservice live under `microservices/drive/src/crates/` per ADR-0131. The legacy `oya-connect-drive-domain` crate (a single domain-layer crate that bundled all of file storage + folder hierarchy + permissions + sharing into one) splits per BC into the new flat layout.

### Crate import-path map

| Legacy `oya-connect-drive-*` path | New `oya-drive-*` path |
|---|---|
| `oya-connect-drive-domain` | split per BC; see note below |
| (planned) `oya-connect-drive-file-kernel` | `oya-drive-file-store-kernel` |
| (planned) `oya-connect-drive-file-domain` | `oya-drive-file-store-domain` |
| (planned) `oya-connect-drive-file-usecase` | `oya-drive-file-store-usecase` |
| (planned) `oya-connect-drive-file-api` | `oya-drive-file-store-api` |
| (planned) `oya-connect-drive-file-adapter` | `oya-drive-file-store-adapter` |
| (planned) `oya-connect-drive-file-adapter-postgres` | `oya-drive-file-store-adapter-postgres` |
| (planned) `oya-connect-drive-file-adapter-s3` | `oya-drive-file-store-adapter-s3` |
| (planned) `oya-connect-drive-file-rest` | `oya-drive-file-store-rest` |
| (planned) `oya-connect-drive-file-worker` | `oya-drive-file-store-worker` |
| (planned) `oya-connect-drive-file-sdk` | `oya-drive-file-store-sdk` |
| (planned) `oya-connect-drive-file-app` | `oya-drive-file-store-app` |
| (planned) `oya-connect-drive-folder-kernel` | `oya-drive-folder-hierarchy-kernel` |
| (planned) `oya-connect-drive-folder-*` | `oya-drive-folder-hierarchy-*` |
| (planned) `oya-connect-drive-upload-*` | `oya-drive-upload-*` |
| (planned) `oya-connect-drive-download-*` | `oya-drive-download-*` |
| (planned) `oya-connect-drive-sync-*` | `oya-drive-sync-*` |
| (planned) `oya-connect-drive-share-*` | `oya-drive-share-link-*` |
| (planned) `oya-connect-drive-permissions-*` | `oya-drive-permissions-*` |
| (planned) `oya-connect-drive-search-*` | `oya-drive-search-index-*` |
| (planned) `oya-connect-drive-preview-*` | `oya-drive-preview-*` |
| (planned) `oya-connect-drive-dlp-*` | `oya-drive-dlp-virus-scan-*` |
| (planned) `oya-connect-drive-retention-*` | `oya-drive-immutability-tier-*` |

> **`oya-connect-drive-domain` split.** The legacy bundled crate bundled file + folder + upload + download + sync + share-link + permissions + search + preview + scan + retention into a single domain-layer crate. Per ADR-0131 + ADR-0105 (13-layer enum), the new layout splits the domain layer per bounded context. Migration imports from the legacy bundled `oya-connect-drive-domain` must each pick the specific replacement BC; a one-line wholesale `use oya_drive::*` import is not supported.

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in `oya-connect-drive-*`. They are therefore not part of the migration surface — they are clean replacement-boundary features:

- **`oya-drive-file-store-adapter-garage`** + **`-adapter-seaweedfs`** — secondary backend-qualified adapters per ADR-DRIVE-0001 (the legacy `oya-connect-drive-domain` only spoke to S3-style stores via a single un-qualified adapter; new layout admits Garage edge-distributed + SeaweedFS archive tier directly).
- **`oya-drive-sync-*` (FastCDC + LBFS delta-sync)** — content-defined-chunking delta-sync per ADR-DRIVE-0002; the legacy surface uploaded whole files.
- **`oya-drive-immutability-tier-*` (WORM)** — per ADR-DRIVE-0006; the legacy surface had only soft-delete + retention, no WORM / object-lock semantics.
- **Client-side E2E (libsodium secretstream)** — opt-in for Personal pillar per ADR-DRIVE-0004; the legacy surface had only server-side envelope encryption.
- **Cross-tenant "Shared with me" with Cedar-policy + audit-chain gating** — PRD FR-18 differentiator; legacy surface had no cross-tenant resolver.
- **Dual-context (Personal / Professional) structural isolation** — inherited from Bominal ADR-0208 but enforced in code (Cedar `policy/dual-context-isolation.md`); the legacy surface only had policy-layer isolation, not code-layer.
- **DLP scan + virus-scan pipeline integrated at write-path** — per ADR-DRIVE-0005 + ADR-DRIVE-0006 referencing OPSWAT + ClamAV + foundry-runtime handoff; legacy surface scanned only on-demand.
- **Office preview in gVisor sandbox** — per ADR-DRIVE-0005; legacy surface ran LibreOffice in a shared pod without sandbox.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_drive_domain::{File, Folder, ShareLink, Permission};
use oya_connect_drive_domain::usecases::{upload_file, mint_share_link};

// AFTER
use oya_drive_file_store_kernel::{File, FileVersion};
use oya_drive_folder_hierarchy_kernel::Folder;
use oya_drive_share_link_kernel::ShareLink;
use oya_drive_permissions_kernel::Permission;
use oya_drive_upload_usecase::upload_file;
use oya_drive_share_link_usecase::mint_share_link;
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-connect-drive-domain = { workspace = true }

# AFTER
[dependencies]
oya-drive-file-store-kernel        = { workspace = true }
oya-drive-folder-hierarchy-kernel  = { workspace = true }
oya-drive-share-link-kernel        = { workspace = true }
oya-drive-permissions-kernel       = { workspace = true }
oya-drive-upload-usecase           = { workspace = true }
oya-drive-share-link-usecase       = { workspace = true }
```

## Reason

The legacy `oya-connect-drive-*` family was authored before the following ADRs crystallised; each ADR makes the legacy shape non-conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle membership at the architecture layer; bundle membership is a brand-layer concept and must not appear in crate names.
2. **ADR-0130 — per-µservice SLO authority.** Drive needs independent SLO targets per surface (file-list latency, upload throughput, download first-byte latency, search latency, sync delta latency, share-link generation latency, preview render latency, DLP scan correctness, WORM correctness, virus-scan correctness). A `connect-*` umbrella SLO cannot honour those.
3. **ADR-0131 — per-µservice flat layout.** Drive's IaC, runbooks, threat-model, DPIA, compliance, capacity-model, cost-budget, incident-response, failure-modes, multi-region all need to live under one folder (`microservices/drive/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA + KR-FSS), pack-eu (GDPR + EU AI Act), pack-us (SEC 17a-4(f) + FINRA 4511), pack-us-healthcare (HIPAA), pack-jp (APPI), pack-sg (PDPA), pack-au (Privacy Act), pack-in (DPDPA), pack-br (LGPD), pack-ae (UAE PDPL), pack-ksa (KSA PDPL) — each lives as `microservices/drive/policy/pack-<region>/`. They cannot share a folder root with mail / calendar / messenger.
5. **ADR-DRIVE-0001 → ADR-DRIVE-0006** — drive-specific decisions (object-storage backend pick, CDC algorithm, share-link security model, encryption-at-rest + E2E, preview sandboxing, WORM policy) need to live at per-µservice ADR granularity, not at the Connect / Workspace suite level.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-connect-drive-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
# Use this command per file as a guided rewrite (review every hit; manual
# disambiguation needed for the `oya-connect-drive-domain` split case):
rg -l "oya_connect_drive_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
# Inside your consumer crate:
cargo nextest run --features connect-drive-strangler-canary
```

Run with the feature flag enabled to route through the new µservice; run without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p99 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- log-line format (preserved verbatim during the canary; may be tightened in a successor-IP `feedback_no_silent_regression`-conforming ADR).
- chunked-upload checksum stability (per ADR-DRIVE-0002 — FastCDC may emit DIFFERENT chunk boundaries than the legacy fixed-size chunker; see Hyrum-bound surface #1 below).
- share-link TTL semantics (per ADR-DRIVE-0003 — new strict-TTL refused after expiry; legacy was more permissive).
- sync conflict tie-break (per ADR-DRIVE-0002 — new last-writer-wins deterministic; legacy was non-deterministic on equal wall-clock).
- virus-scan timing observable (per ADR-DRIVE-0005 — scan happens at write-path now; legacy scanned on-demand).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports AND the drive µservice's Phase 3 canary reaches 100% traffic (per ADR-0134), remove the legacy dependency from your `Cargo.toml`:

```toml
# Remove this line:
oya-connect-drive-domain = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
# Per ADR-0134 Phase 4 verification:
cargo tree -e normal -p your-crate | grep oya-connect-drive   # expect empty
rg "use oya_connect_drive_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.drive.*` | `drive.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/drive/slos/*.openslo.yaml` (per-µservice, 9 files) |
| Helm chart values key | `.Values.connect.drive.*` | `.Values.drive.*` |
| K8s namespace | `connect` | `drive` |
| Cedar policy fragment path | `policy/connect/drive/*.cedar` | `microservices/drive/policy/*.cedar` |
| pack-kr overlay path | `policy/connect/drive/pack-kr/*` | `microservices/drive/iac/kustomize/overlays/pack-kr/*` |
| Workflow event prefix | `connect.drive.*` | `drive.*` (e.g., `drive.file.lifecycle.v1`, `drive.share.v1`) |
| Ontology type prefix | `Connect.Drive.*` | `Drive.*` (e.g., `Drive.File`, `Drive.Folder`, `Drive.ShareLink`, `Drive.Permission`, `Drive.LegalHold`, `Drive.ImmutabilityRecord`) |
| Telemetry metric prefix | `oya_connect_drive_*` | `oya_drive_*` |
| Tracing span attribute namespace | `connect.drive.*` | `drive.*` |
| Object-store backend | `aws.s3` (single hard-coded) | `garage` (primary) + `minio` (secondary) + `seaweedfs` (archive tier) per ADR-DRIVE-0001 |
| CDC engine choice | `fixed-8MiB` (legacy) | `fastcdc` per ADR-DRIVE-0002 |
| Share-link KDF | `pbkdf2` (legacy) | `argon2id` per ADR-DRIVE-0003 |
| Preview sandbox | `shared-pod-libreoffice` (legacy) | `gvisor-libreoffice` per ADR-DRIVE-0005 |

## Dual-context isolation invariant (preserved + strengthened)

The Personal ↔ Professional context isolation invariant is preserved verbatim in `oya-drive-file-store-kernel` and strengthened with a Cedar-layer enforcement:

- The `FileContextBoundaryGuard` port trait keeps the same method signatures.
- Cross-context attempts (Professional → Personal file read) emit the same 403 + same audit-chain event variant (`DriveCrossContextRefused`).
- The kernel-layer refusal (not adapter-layer) invariant is preserved.
- **Strengthened**: cross-context attempts are also refused at the Cedar policy layer per `policy/dual-context-isolation.md`; the kernel refusal is the defence-in-depth backup.

This means downstream consumers that wrap the boundary guard via the legacy import path will see identical refusal behaviour after migration; no test rewrite needed for the isolation surface.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes Removal Hard", these are the legacy drive surfaces with observable behaviour that may be depended on. Each is preserved verbatim during the canary; consumers must re-test after Phase 5 removal in case they had a long-tail dependency:

1. **Chunk boundary stability for content-address derivation.** Legacy used fixed-size 8MiB chunking; `fastcdc` (per ADR-DRIVE-0002) uses content-defined boundaries. Consumers that depended on chunk-id stability across uploads of the same file will observe DIFFERENT chunk ids; the file-level content-address remains stable. Documented in ADR-DRIVE-0002 §Hyrum.
2. **Share-link signing-blob format.** Legacy used HMAC-SHA256 over a fixed-field serialisation. New µservice uses Ed25519 over a canonicalised JSON serialisation per ADR-DRIVE-0003. Consumers that pattern-matched on the legacy blob shape see a different shape; HTTP-level GET URL is the public contract, not the blob.
3. **Share-link TTL boundary inclusivity.** Legacy returned `200 OK` for one second past expiry due to clock-skew tolerance; new µservice enforces strict expiry. Consumers with timing-dependent tests may need a ≤ 1s tolerance.
4. **Sync conflict tie-break.** Legacy used non-deterministic ordering on equal wall-clock; new µservice uses deterministic last-writer-wins with `(timestamp, actor_id)` as tie-breaker per ADR-DRIVE-0002. Consumers expecting non-determinism may see flakey-tests-fixed behaviour.
5. **Virus-scan timing.** Legacy scanned on first download; new µservice scans on upload (per ADR-DRIVE-0005). Consumers that uploaded EICAR signatures and depended on first-download-trigger behaviour will see upload-time-rejection.
6. **Recurrence horizon for retention sweep.** Legacy was unbounded soft-delete; new µservice rejects > 99y retention floor (within object-lock spec). Consumers that requested very-long retention will be capped at 99y per ADR-DRIVE-0006.
7. **Object-store key shape.** Legacy used `tenant_id/file_id` shape; new µservice uses `tenant_id/<file_id_prefix_4>/file_id` shape for shard locality per PRD §"Sharding". External S3-API consumers see different keys; oya-native API is shape-agnostic.

## Runbook continuity table

| Legacy runbook (under `policy/connect/drive/runbooks/`) | New runbook (under `microservices/drive/runbooks/`) | Status |
|---|---|---|
| `file-storage-restore.md` | folded into `object-storage-degraded.md` + `backfill-replay.md` | refactored |
| `share-link-takeover.md` | `share-link-takeover-incident.md` | preserved + expanded |
| (no legacy counterpart) | `upload-multipart-stuck.md` | NEW per ADR-DRIVE-0002 |
| (no legacy counterpart) | `sync-conflict-resolution.md` | NEW per ADR-DRIVE-0002 |
| (no legacy counterpart) | `dlp-quarantine-release.md` | NEW per ADR-DRIVE-0005 |
| (no legacy counterpart) | `object-storage-degraded.md` | NEW per ADR-DRIVE-0001 |
| (no legacy counterpart) | `virus-scan-rollback.md` | NEW per ADR-DRIVE-0005 |
| (no legacy counterpart) | `immutability-tier-violation.md` | NEW per ADR-DRIVE-0006 |

## Phases (per ADR-0134)

| Phase | Description | Status (drive) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-DRIVE passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-connect-drive-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crates + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_drive_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

Per the deprecation-and-migration skill, every deprecation closeout must satisfy these checks. Each is gated by a concrete command:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice drive
  # expect: HG-DRIVE accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/drive/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-connect-drive-domain --invert    | grep -v 'oya-connect-drive-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_drive_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connect-drive-*" | wc -l   # expect 0
  test ! -f /specs/microservices/drive.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase**:
  ```bash
  rg "oya_connect_drive" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/golden/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (per Phase 5):
  ```bash
  test ! -f microservices/drive/deprecation-notice.md          # expect file absent
  test ! -f microservices/drive/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4 for the core symbol surface: the adapter preserves the legacy symbol surface verbatim, including error variant ordering and timing characteristics within the +5% canary tolerance.

**There ARE behavioural strengthenings** that may visibly differ from legacy and are NOT preserved by the adapter (per `feedback_no_silent_regression`):

1. **FastCDC chunk boundaries** (per ADR-DRIVE-0002). Consumers depending on legacy fixed-size chunking will observe different chunk-ids. The file-level content-address remains stable. Deliberate strengthening.
2. **Share-link strict-TTL** (per ADR-DRIVE-0003). Consumers depending on legacy 1s clock-skew tolerance will see strict refusal at expiry boundary. Deliberate strengthening; the legacy behaviour was a small information-disclosure surface.
3. **Virus-scan at upload time** (per ADR-DRIVE-0005). Consumers depending on legacy first-download-scan will see upload-time rejection of malicious content. Deliberate strengthening.
4. **WORM tier strictness** (per ADR-DRIVE-0006). Files elected into WORM cannot be purged by tenant-root before retention floor expires. Deliberate strengthening; the legacy soft-delete + retention was insufficient for SEC 17a-4(f) / FINRA 4511 / HIPAA §164.316 compliance.

Phase 5 (code removal) **IS a breaking change** for any consumer that did not migrate during the 5-month adapter+canary window. Per `feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's `deprecation_date` (2026-05-17), so a target advisory removal date of **2026-11-17** (subject to the HG-DRIVE retirement trigger gating).
- Owning axis (axis-drive) ships migration ChangeSets for every internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/drive.json` + `/specs/microservices/workspace/drive.json`) receive a 6-month sunset window from this notice; the spec file's `deprecated: true` + `replacement_path: /specs/microservices/drive/drive.json` fields render in the agent-coordination dashboard.

## References

- ADR-0135: Connect super-app expansion into 8 flat µservices (precedent topology).
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: Connect dissolution Strangler migration (operational policy).
- ADR-DRIVE-0001: Object-storage substrate selection.
- ADR-DRIVE-0002: Content-defined-chunking + delta-sync.
- ADR-DRIVE-0003: Share-link security model.
- ADR-DRIVE-0004: Encryption-at-rest + E2E.
- ADR-DRIVE-0005: Preview pipeline sandboxing.
- ADR-DRIVE-0006: Immutability + WORM policy.
- AWS S3 SigV4 spec; tus.io 1.0; RFC 4918 WebDAV; RFC 9110 HTTP; RFC 9106 Argon2.
- `microservices/drive/PRD.md` — full target-state product definition.
- `microservices/drive/PHASE-01-DRIVE-FOUNDATION.md` — phase plan.
- `microservices/drive/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
