---
doc_class: FailureModesAnalysis
template_id: TPL-FAILURE-MODES
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + ops-sre-reliability
related_adrs: [ADR-0114, ADR-0117, ADR-0139, ADR-DRIVE-0001, ADR-DRIVE-0005, ADR-DRIVE-0006]
doc_status: published
---

# Failure Modes — drive µservice

## Purpose

Enumerate plausible failure modes, expected behaviour under each, detection mechanisms, and graceful-degradation guarantees.

## Failure mode catalog

### FM-01 — Object-store single-cell loss

- **Trigger**: Garage / MinIO / SeaweedFS cell crash, disk failure, network partition.
- **Expected behaviour**: replication-factor 3 absorbs; reads route to healthy cells; writes still durable.
- **Detection**: per-cell health metric; replication backlog metric.
- **Graceful degradation**: zero customer impact; rebuild in background.
- **Runbook**: `object-storage-degraded.md`.

### FM-02 — Object-store dual-cell loss (same pack)

- **Trigger**: simultaneous cell failure (e.g., AZ outage); replication-factor 3 leaves one healthy copy.
- **Expected behaviour**: reads still serve from sole healthy cell; writes refused until rebuild starts.
- **Detection**: critical alert on replication degradation.
- **Graceful degradation**: read-only mode for affected tenants; writes queue.
- **Runbook**: `object-storage-degraded.md` Sev-1 path.

### FM-03 — Postgres metadata failure

- **Trigger**: primary Postgres crash.
- **Expected behaviour**: sync replica promotes; DNS swing; ≤ 5 min RTO.
- **Detection**: Patroni health.
- **Graceful degradation**: write path briefly unavailable; read serves from replica.
- **Runbook**: `cloud-iac` runbooks.

### FM-04 — Redis cluster failure

- **Trigger**: Redis cluster node loss.
- **Expected behaviour**: in-flight upload sessions tolerated lost; client must re-init multipart. Sync sessions tolerated lost; client must re-init.
- **Detection**: per-node health.
- **Graceful degradation**: re-init from client. Upload session has client-side resume marker; sync session re-syncs from manifest.

### FM-05 — Meilisearch index loss

- **Trigger**: index corruption / disk failure.
- **Expected behaviour**: re-index from Postgres metadata + Tika extract over ≤ 4h per 1M files.
- **Detection**: index-version drift metric.
- **Graceful degradation**: search returns "indexing in progress" with fallback to filename-only matching from Postgres `LIKE`.

### FM-06 — ClamAV worker pool exhaustion

- **Trigger**: per-tenant flood OR upstream signature update failure.
- **Expected behaviour**: scan queue grows; uploads land in pending-scan state; files in pending-scan cannot be shared/downloaded by non-uploader.
- **Detection**: queue depth > 60s.
- **Graceful degradation**: tenant sees "Uploaded; scan pending"; auto-resume when capacity returns.
- **Runbook**: `virus-scan-rollback.md`.

### FM-07 — OPSWAT outage

- **Trigger**: OPSWAT MetaDefender cloud unavailability.
- **Expected behaviour**: pack-us-healthcare + pack-eu falls back to ClamAV-only verdict; healthcare tenants flagged for re-scan when OPSWAT returns.
- **Detection**: OPSWAT API health check.
- **Graceful degradation**: BAA tenants surface "single-engine scan only" notice; multi-engine re-scan when OPSWAT recovers.

### FM-08 — Preview renderer crash (LibreOffice in gVisor)

- **Trigger**: malformed Office file triggers LibreOffice crash; gVisor isolates.
- **Expected behaviour**: per-render timeout (60s) fires; container teardown; user sees "preview unavailable; download to view".
- **Detection**: per-render error rate.
- **Graceful degradation**: download path still works; preview only is degraded.

### FM-09 — Preview renderer sandbox escape (gVisor CVE)

- **Trigger**: gVisor CVE actively exploited.
- **Expected behaviour**: incident-response Sev-1; disable Office preview pack-wide until patched.
- **Detection**: gVisor CVE feed; per-preview-worker chaos exercise.
- **Graceful degradation**: image / PDF / video preview still works (no LibreOffice involvement); Office preview returns "service temporarily unavailable".
- **Runbook**: `runbooks/object-storage-degraded.md` (general containment) + `incident-response.md`.

### FM-10 — Share-link signing-key compromise

- **Trigger**: per-tenant signing key suspected compromised.
- **Expected behaviour**: incident-response Sev-1; rotate signing key + revoke all extant links; affected tenant comms within 1h.
- **Detection**: anomaly detection on share-link verification patterns + audit-chain forensic replay.
- **Graceful degradation**: existing links invalidated; tenant re-mints from UI.
- **Runbook**: `share-link-takeover-incident.md`.

### FM-11 — Tenant-DEK rotation failure

- **Trigger**: OpenBao Transit unavailable or key-rotation orchestration drift.
- **Expected behaviour**: writes refuse with "encryption-at-rest unavailable"; reads continue using prior DEK.
- **Detection**: rotation-success metric; per-tenant DEK staleness alarm.
- **Graceful degradation**: read-only mode for tenant until DEK rotation completes.

### FM-12 — CDN regional outage

- **Trigger**: CDN provider regional unavailability.
- **Expected behaviour**: fallback to direct-from-cell download; latency degrades; bandwidth cost spikes.
- **Detection**: CDN health probe.
- **Graceful degradation**: cost-budget alarms fire; cost-budget cap activates throttling for non-critical tenants.

### FM-13 — Sync conflict mass-resolution failure

- **Trigger**: concurrent edits exceed conflict-resolver capacity.
- **Expected behaviour**: conflict surfaces to user; user resolves via UI; no auto-merge of byte content.
- **Detection**: per-tenant conflict rate.
- **Graceful degradation**: drive remains usable; conflicting versions both preserved.
- **Runbook**: `sync-conflict-resolution.md`.

### FM-14 — WORM scan + retention worker failure

- **Trigger**: worker pod crashes or DB connection failure during WORM scan.
- **Expected behaviour**: worker idempotent; resumes from checkpoint; never violates WORM under failure.
- **Detection**: per-worker health + checkpoint-lag metric.
- **Graceful degradation**: retention sweep scheduled-for-distinct-tracked-work; WORM tier invariants preserved.

### FM-15 — Audit-chain emission failure

- **Trigger**: audit-chain µservice unavailable.
- **Expected behaviour**: per Bominal ADR-0028, writes blocked or queued depending on tenant tier; "held" SLO state via observability.
- **Detection**: emission-ack metric.
- **Graceful degradation**: writes queue for emission; reads continue.

### FM-16 — Foundry-runtime OCR / auto-tag handoff failure

- **Trigger**: foundry-runtime unavailable.
- **Expected behaviour**: file persists; OCR / auto-tag retry queue; T1 capability degrades to T0 (suggest mode).
- **Detection**: workflow event ack.
- **Graceful degradation**: file storage / sharing / search-by-filename still works; full-text search misses content until OCR completes.

### FM-17 — Mail attachment-bridge token leak

- **Trigger**: short-lived bridge token leaked via outbound mail intercept.
- **Expected behaviour**: token TTL ≤ 5 min limits blast radius; one-time-use enforced.
- **Detection**: per-token use-count > 1 = alarm.
- **Graceful degradation**: tenant comms + bridge token rotation.

### FM-18 — Cross-pack route leak

- **Trigger**: misconfigured ingress sends pack-eu tenant request to pack-kr cell.
- **Expected behaviour**: pack-pinning at Cedar layer + LEAN check refuses; request returns 403.
- **Detection**: pack-mismatch counter > 0 = Sev-1.
- **Graceful degradation**: request refused; tenant notified.

### FM-19 — Tika extractor crash on malformed file

- **Trigger**: malformed PDF / Office triggers Tika exception.
- **Expected behaviour**: per-job timeout (60s); skip extraction; file is still uploaded + virus-scanned + previewable.
- **Detection**: per-job error rate.
- **Graceful degradation**: full-text search misses the file; filename + metadata search still works.

### FM-20 — DLP false-positive flood

- **Trigger**: DLP rule update tags benign files as flagged.
- **Expected behaviour**: tenant share-out blocked; tenant policy team notified.
- **Detection**: per-tenant flag rate > 5% rolling 24h.
- **Graceful degradation**: quarantine release runbook executed.
- **Runbook**: `dlp-quarantine-release.md`.

## References

- ADR-0114 — canary observability + rollback.
- ADR-0117 — cloud-native infrastructure.
- ADR-0139 — SLO-gated promotion.
- ADR-DRIVE-0001 — object-storage substrate selection.
- ADR-DRIVE-0005 — preview pipeline sandboxing.
- ADR-DRIVE-0006 — immutability + WORM policy.
- `microservices/drive/runbooks/*.md`.
- `microservices/drive/incident-response.md`.
- `microservices/drive/threat-model.md`.
