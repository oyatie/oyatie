---
doc_class: BackfillReplayPlan
template_id: TPL-BACKFILL-REPLAY
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + ops-sre-reliability + council-privacy
related_adrs: [ADR-0028, ADR-0114, ADR-0117, ADR-0130, ADR-DRIVE-0001, ADR-DRIVE-0006]
doc_status: published
---

# Backfill + Replay Plan — drive µservice

## Purpose

Define how drive backfills data from tenants' prior systems (migration ingest), replays events for re-indexing / re-scanning / re-applying-policy, and recovers state from durable storage. Includes per-pack residency + WORM-respecting semantics.

## Backfill scenarios

### B-01 — Tenant onboarding migration from competitor

- Source: Google Drive / Dropbox / OneDrive / Box / Nextcloud / S3 bucket / WebDAV server.
- Tooling: per-source connector at `microservices/drive/src/crates/oya-drive-migration-{gdrive,dropbox,onedrive,box,nextcloud,s3,webdav}-app`.
- Throughput: ≥ 1TB/day per migration job; HPA-bound.
- Manifest: every imported file emits content-address + metadata; staged in pending bucket; promoted after virus scan + DLP.
- Verification: per-file checksum comparison; report on missing / corrupt / unsupported.
- Tenant comms: migration progress dashboard.

### B-02 — Per-tenant restore from backup

- Source: oyatie cross-region backup at `s3://oyatie-backup-<pack>-<tenant>/`.
- Tooling: `oya-drive-file-store-worker --restore-from-backup --tenant <tenant_id>`.
- Resumability: checkpoint-bound; can resume on failure.
- WORM-respecting: restored objects preserve immutability metadata.

### B-03 — Per-tenant cross-pack restore (SCC-gated)

- Source: oyatie cross-pack backup (with tenant DPA + SCC).
- Tooling: as B-02 + cross-pack Cedar grant.
- Audit: every cross-pack restore emits to audit-chain with explicit grant pointer.

### B-04 — Whole-cell restore (disaster recovery)

- Source: cross-cell replication + per-cell backup.
- Tooling: cell-level orchestrator (`cloud-iac`).
- RTO: ≤ 4h for cell scale (5PB).
- RPO: ≤ 60s.

## Replay scenarios

### R-01 — Re-index full-text search after Meilisearch index loss

- Trigger: index corruption / cluster failure.
- Source: Postgres metadata + Tika extract cache.
- Throughput: ≥ 1M files/4h per tenant.
- Approach: `oya-drive-search-index-worker --rebuild-index --tenant <tenant_id>`.

### R-02 — Re-scan all files for virus (signature update / OPSWAT outage recovery)

- Trigger: ClamAV signature update on critical CVE; OPSWAT outage recovery.
- Source: object store enumeration.
- Throughput: bounded by scan-worker pool.
- Approach: `oya-drive-dlp-virus-scan-worker --rescan-all --tenant <tenant_id>`.

### R-03 — Re-scan all files for DLP (new rule pack)

- Trigger: new DLP rule pack deployed.
- Source: object store enumeration.
- Throughput: bounded by DLP-worker pool.
- Approach: `oya-drive-dlp-virus-scan-worker --rescan-dlp-all --tenant <tenant_id>`.

### R-04 — Replay audit-chain events to verify integrity

- Trigger: audit-chain forensic investigation.
- Source: audit-chain µservice event store.
- Approach: per-event Ed25519 + Merkle verification; report on any mismatch.

### R-05 — Re-apply retention policy after pack change

- Trigger: tenant changes pack (with regulatory pre-approval); per-pack retention floor differs.
- Source: file metadata.
- Approach: `oya-drive-immutability-tier-worker --reapply-retention --tenant <tenant_id>`.
- WORM-respecting: cannot reduce retention; can only extend.

### R-06 — Re-generate preview cache

- Trigger: preview cache eviction / corruption / format change.
- Source: object store enumeration.
- Throughput: bounded by preview-worker pool.

## Durability + checkpoint

- All backfill / replay jobs are checkpoint-bound; resumable.
- Per-file idempotency key prevents duplicate writes.
- Audit-chain emits per-batch progress; observability dashboard tracks per-tenant progress.

## Data residency

- Backfill / replay never crosses pack boundary except via explicit `cross-pack-replication-grant` Cedar policy.
- B-03 (cross-pack restore) requires tenant DPA + SCC + Cedar grant; LEAN check enforces.

## Verification

- per-job: file count + byte count + content-address checksum verification.
- per-tenant post-completion: random-sample 1% verification with content-address re-check.
- audit-chain receipt verification on every emitted event.

## Cost

- per-byte ingress cost included in cost-budget.md.
- backfill rate-limited per tenant to avoid cost surprises; tenant approves rate-cap during onboarding.

## References

- ADR-0028 (Bominal) — audit chain.
- ADR-0114 — canary observability + rollback.
- ADR-0117 — cloud-native infrastructure / data residency.
- ADR-0130 — SLO-gated promotion.
- ADR-DRIVE-0001 — object-storage substrate.
- ADR-DRIVE-0006 — immutability + WORM policy.
- `microservices/drive/runbooks/object-storage-degraded.md`.
- `microservices/drive/cost-budget.md`.
- `microservices/drive/multi-region.md`.
