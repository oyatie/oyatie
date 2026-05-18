---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: axis-docs + ops-sre-reliability
methodology: STAMP + FMEA + Google SRE
related_adrs: [ADR-0139, ADR-0131, ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0006]
doc_status: published
---

# Failure Modes — docs µservice

## Purpose

Enumerate failure modes, blast radius, detection signals, automated recovery, and operational runbooks. Each failure mode has at least one runbook in `runbooks/` and at least one SLO + alert in `dashboards/`.

## Failure-mode catalog

### FM-01 — CRDT silent loss (op accepted but not reflected in final state)

- **Cause:** Loro adapter regression, op-serialisation bug, or Redis spool eviction before persistence ack.
- **Blast radius:** Sev-1; tenant trust impact; AC-06 invariant breach.
- **Detection:** `docs_collab_silent_loss_attempt_total > 0`.
- **Automated recovery:** halt save-paths for affected (tenant, doc); reconstruct from seal-deltas.
- **Runbook:** `runbooks/collab-conflict-resolution.md` (Path C — Sev-1 escalation).
- **Mitigation hardening:** AC-06 property test in CI; pinned Loro version; Redis AOF every-sec.

### FM-02 — CRDT conflict explosion (high conflict rate on a document)

- **Cause:** Two reviewers concurrently editing same node's required field; legitimate organisational disagreement; or coordinated DoS.
- **Blast radius:** Sev-3 if normal; Sev-2 if engine bug.
- **Detection:** `docs_collab_conflict_surfaced_total` rate > 0.5/s for ≥ 5min on a single (tenant, doc).
- **Automated recovery:** none; conflict UI surfaces; user reconciles.
- **Runbook:** `runbooks/collab-conflict-resolution.md` Path B.

### FM-03 — Document version corruption (snapshot mismatches op log)

- **Cause:** version-history adapter bug; partial-rotation of tenant-DEK; cold-storage corruption.
- **Blast radius:** affected document; version-revert unavailable; AC-13 (legal-hold) at risk if hold targets corrupted version.
- **Detection:** `docs_version_integrity_mismatch_total > 0` (Merkle root mismatch on version-restore attempt).
- **Automated recovery:** restore from prior version snapshot; if none, restore from S3 backup.
- **Runbook:** `runbooks/doc-version-restore-corruption.md`.

### FM-04 — Export pipeline failure (Pandoc / WeasyPrint / Chromium crash)

- **Cause:** Pandoc upgrade introduces unsupported feature; oversized doc exceeds memory; gVisor seccomp violation.
- **Blast radius:** affected export job; per-tenant export rate may drop.
- **Detection:** `docs_export_pipeline_failure_rate > 10%` on rolling 5min window.
- **Automated recovery:** auto-retry with fallback backend (WeasyPrint → Chromium for PDF if WeasyPrint fails); roll back Pandoc version if upgrade-correlated.
- **Runbook:** `runbooks/export-pipeline-failure-pandoc-rollback.md`.

### FM-05 — Attachment storage restore needed (S3 object corruption or accidental delete)

- **Cause:** disk corruption; S3-side delete (should be Object-Lock-blocked but possible on non-held); accidental tenant action.
- **Blast radius:** affected attachments unavailable.
- **Detection:** `docs_attachment_integrity_failure_total > 0`.
- **Automated recovery:** restore from S3 version history (Object Lock retains); from cold-tier backup if version exhausted.
- **Runbook:** `runbooks/attachment-restore.md`.

### FM-06 — Share ACL drift (private doc accidentally public)

- **Cause:** tenant misconfiguration; race condition between share-grant write + cache invalidation.
- **Blast radius:** information disclosure risk; per-share scope.
- **Detection:** `oya-check-public-collection-drift` LEAN scan + audit-chain query weekly.
- **Automated recovery:** revert to private on drift; emit `ShareGrantDriftDetected` audit event.
- **Runbook:** `runbooks/share-acl-drift.md`.

### FM-07 — Editor-session storm (tenant opens 10k+ sessions; WS gateway lease pressure)

- **Cause:** malicious tenant; misbehaving SDK retry loop; legitimate high-volume use.
- **Blast radius:** WS gateway pod saturation; new sessions throttled.
- **Detection:** WS gateway `lease_count` > 80% of cell max; `docs_ws_lease_acquisition_p99 > 1s`.
- **Automated recovery:** per-tenant rate limit; existing leases continue; new sessions queued.
- **Runbook:** `runbooks/editor-session-storm-throttle.md`.

### FM-08 — Embed-source stale (workflow-studio / sheets / slides snapshot diverged from source)

- **Cause:** source µservice changed but Workflow event lost; cross-pack mesh partition; source-side ACL revoked.
- **Blast radius:** embedded content shows stale data; tenant operational confusion.
- **Detection:** `docs_embed_source_staleness_seconds > 600` (10min).
- **Automated recovery:** force re-fetch on access; if source unavailable, surface "stale snapshot" banner; if source-side ACL revoked, replace with redacted placeholder.
- **Runbook:** `runbooks/embed-source-stale-detection.md`.

### FM-09 — Per-block ACL latency spike (cache miss storm)

- **Cause:** Cedar policy reload across many tenants concurrently; per-block ACL projection cache cold.
- **Blast radius:** doc-read p99 spikes; possible 5xx cascade.
- **Detection:** `docs_per_block_acl_check_p99 > 50ms`.
- **Automated recovery:** single-flight per (doc_id, principal_id); pre-warm cache on policy-reload.
- **Runbook:** `runbooks/share-acl-drift.md` (Section C — latency mitigation).

### FM-10 — Audit-chain emission failure (silent)

- **Cause:** audit-chain µservice ingest endpoint returns 5xx; emission ack times out.
- **Blast radius:** Audit-chain seal missing; SOC 2 / ISO 27001 audit-coverage gap.
- **Detection:** `docs_audit_emission_ack_lag_seconds > 30`.
- **Automated recovery:** Doc write blocks (fail-closed) when emission ack > 30s; user sees "operation pending"; ack-or-fail.
- **Runbook:** see `incident-response.md` §"Audit-chain emission gap".
- **Mitigation hardening:** Fail-closed prevents missing-seal gap.

### FM-11 — OOXML import fidelity below threshold (DOCX import lost > 5% features)

- **Cause:** unsupported OOXML feature in Pandoc; vendor-specific extensions.
- **Blast radius:** affected import job; user sees fidelity warning.
- **Detection:** `docs_ooxml_import_fidelity_ratio < 0.95`.
- **Automated recovery:** import proceeds with fidelity warning surfaced to user; per-feature support added to ADR-DOCS-0006 named edge-case matrix.
- **Runbook:** see `runbooks/export-pipeline-failure-pandoc-rollback.md` Section D (import-side).

### FM-12 — gVisor sandbox escape attempt detected

- **Cause:** malicious input designed to escape gVisor (CVE-class).
- **Blast radius:** Sev-1 potential; bounded by sandbox; tmpfs-only and no-network-egress contain.
- **Detection:** gVisor seccomp violation count > 0.
- **Automated recovery:** quarantine pod; drain pool; replay payload in forensic sandbox.
- **Runbook:** see `incident-response.md` §"Export pipeline gVisor escape".

### FM-13 — Postgres connection pool exhaustion

- **Cause:** burst of doc-opens + edits + ACL checks exhausts max-connections.
- **Blast radius:** new requests queued; some time out; cascading 5xx.
- **Detection:** `pg_connection_pool_utilisation > 85%`.
- **Automated recovery:** HPA scales rest pods; short-term rate-limit at REST.
- **Runbook:** `runbooks/editor-session-storm-throttle.md` (Section D — DB headroom).

### FM-14 — Cross-pack mesh partition during embed fetch

- **Cause:** Mesh partition between pack-kr and pack-eu; cross-pack embed query times out.
- **Blast radius:** Cross-pack embeds degrade; doc still opens.
- **Detection:** `docs_cross_pack_embed_timeout_rate > 5%`.
- **Automated recovery:** Cross-pack timeout = 5s; on timeout, return prior cached snapshot.
- **Runbook:** `runbooks/embed-source-stale-detection.md`.

### FM-15 — Workflow event loss (docs → audit-chain / mail / messenger)

- **Cause:** Workflow event bus loses a `DocumentEdited` event; downstream consumer never fires.
- **Blast radius:** audit-chain emission gap; share-via-mail not sent; mention-notification missed.
- **Detection:** outbox-replay metric; per-event delivery-ack mismatch.
- **Automated recovery:** Outbox pattern: events written to Postgres + relayed by sidecar; relay retries until ack.
- **Runbook:** `incident-response.md` §"Audit-chain emission gap" Section B (outbox replay).

### FM-16 — Tenant-DEK rotation failure leaves documents unreadable

- **Cause:** DEK rotation event partially applied; some blobs re-encrypted, others not; read fails.
- **Blast radius:** affected tenant cannot read recent docs until rotation completes / rolls back.
- **Detection:** `docs_dek_rotation_in_flight=true` + reads failing with `dek_mismatch` error.
- **Automated recovery:** Rotation is transactional + idempotent; partial-rotation state recovers on next worker run; old DEK retained read-only until ack.
- **Runbook:** see `incident-response.md` §"Tenant-DEK compromise" (rotation procedure equivalent).

### FM-17 — Recursive embed loop (Doc-A embeds Doc-B which embeds Doc-A)

- **Cause:** tenant accidentally creates cyclic embeds.
- **Blast radius:** doc-open blocks; resolver loops.
- **Detection:** embed depth > 3 in resolver call stack; `EmbedLoopDetected` audit event.
- **Automated recovery:** resolver rejects depth > 3; surfaces "embed-loop detected" to author.
- **Runbook:** `runbooks/embed-source-stale-detection.md` Section E.

## Failure-mode aggregation gates

- `oya gate validate failure-mode-coverage --microservice docs`: refuses build if any new code path lacks at least one failure-mode entry.
- Quarterly failure-mode review.
- Annual game-day: simulate FM-01, FM-03, FM-09, FM-12 end-to-end.

## References

- ADR-0139: SLO-gated promotion.
- ADR-0131: per-microservice layout.
- ADR-DOCS-0001 (Loro CRDT); ADR-DOCS-0003 (export backend); ADR-DOCS-0004 (per-block ACL); ADR-DOCS-0006 (DOCX fidelity).
- `runbooks/*.md` (one per failure mode).
- Google SRE Workbook ch. 6 (managing risk) + ch. 11 (managing incidents).
- NASA-STD-8729.1 (STAMP).
