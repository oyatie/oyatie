---
doc_class: FailureModes
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + ops-sre-reliability
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/threat-model.md
  - microservices/foundry-evidence/policy/evidence-pack-integrity.md
  - microservices/foundry-evidence/runbooks/
doc_status: published
---

# foundry-evidence — failure modes catalogue (FM-01..FM-12)

## FM-01 — Late signal arrived after pack assembled

**Trigger**: a foundry-runtime / foundry-eval / foundry-guardrails / foundry-supervisor signal arrives after the assembly window closed.

**Detection**: `oya_foundry_evidence_late_signal_total` counter; emits `foundry.evidence.late_signal.v1` Workflow event.

**Honest representation**: late signal is captured + emitted; pack is NOT silently retro-modified. If `materially_significant=true`, `runbooks/evidence-pack-rebuild.md` issues a new pack with `supersedes_pack_ref` pointing to the original.

**Severity**: Sev-3 default; Sev-2 if rate exceeds 0.001/s sustained; Sev-1 if multiple materially-significant late signals concentrate (suggests upstream µservice malfunction).

**Owner runbook**: `runbooks/evidence-pack-rebuild.md`.

## FM-02 — Audit-chain substrate fully unavailable

**Trigger**: substrate returns 5xx or times out repeatedly.

**Detection**: `oya_foundry_evidence_audit_chain_emit_failure_rate` > 0.01 sustained.

**Behaviour**: `record_invocation` STILL returns receipt to caller with `sealed=false`, `audit_chain_emit_pending=true`. Packs queue in durable dead-letter store. Bridge worker retries under bounded back-off.

**Severity**: Sev-2 default; Sev-1 if substrate down > 10 min.

**Owner runbook**: `runbooks/audit-chain-backlog.md` + substrate's `microservices/audit-chain/runbooks/audit-chain-restart.md`.

## FM-03 — Audit-chain substrate degraded (slow)

**Trigger**: substrate accepts emits but seal latency exceeds SLO.

**Detection**: `oya_foundry_evidence_audit_chain_emit_backlog_depth_seconds` > 60.

**Behaviour**: bridge worker keeps draining; backlog grows; `sealed=false` packs visible in evidence-query.

**Severity**: Sev-2.

**Owner runbook**: `runbooks/audit-chain-backlog.md`.

## FM-04 — Missing signal at assembly time

**Trigger**: pack-assembly window closes (default 60 s) before one of the expected signals (eval verdict, guardrail decisions, autonomy-tier decision) arrives.

**Detection**: `oya_foundry_evidence_pack_assembled_partial_total{missing_source=...}`.

**Behaviour**: pack assembled with `partial=true` + `missing_sources=[...]`. Pack is sealed honestly with the partial flag; no fabrication.

**Severity**: Sev-3 individually; Sev-2 if partial-rate > 0.001 sustained on a stable source.

**Owner runbook**: `runbooks/pack-assembly-fail.md` §"Missing-signal".

## FM-05 — Schema drift OR builder crash

**Trigger**: source µservice publishes envelope with unrecognised schema_version OR pack-builder process crashes.

**Detection**: `oya_foundry_evidence_pack_assembly_failed_total{failure_class=schema_drift|builder_crash}`.

**Behaviour**: assembly fails; envelope quarantined in forensic bucket (Cedar-gated); audit-emit `foundry.evidence.pack.assembly_failed.v1` after retry budget exhausted.

**Severity**: Sev-2 (schema_drift on stable contract is automatic Sev-1).

**Owner runbook**: `runbooks/pack-assembly-fail.md` §"Schema drift" / §"Builder crash".

## FM-06 — Eval-evidence join correctness violation

**Trigger**: ADR-0024 invariant violated — the eval verdict joined into a pack was not the verdict current at invocation time.

**Detection**: nightly property-based drill on eval-evidence-aggregator; tenant-reported diff via support.

**Behaviour**: affected packs flagged for rebuild; eval-history table investigated for clock drift or verdict-ordering bug.

**Severity**: Sev-2; Sev-1 if drill detects > 0.001 of packs affected.

**Owner runbook**: `runbooks/evidence-pack-rebuild.md`.

## FM-07 — Regulator-export field-completeness drift

**Trigger**: bundle was produced with a framework-profile that lacks required fields (e.g., EU AI Act bundle missing Art. 18 fields).

**Detection**: `regulator-profile-drill` CI lane; council-privacy bundle review.

**Behaviour**: bundle marked `field_completeness_drift=true`; `runbooks/regulator-export-reissue.md` reissues with corrected profile.

**Severity**: Sev-1 if regulator has already received the defective bundle; Sev-2 otherwise.

**Owner runbook**: `runbooks/regulator-export-reissue.md`.

## FM-08 — Regulator-export wrong scope

**Trigger**: bundle assembled with off-by-one time-range, wrong tenant filter, or wrong framework.

**Detection**: pre-delivery QA; regulator feedback.

**Behaviour**: withdraw + reissue (if not yet delivered); reissue + notify (if already delivered).

**Severity**: Sev-1 if delivered; Sev-2 if pre-delivery.

**Owner runbook**: `runbooks/regulator-export-reissue.md`.

## FM-09 — Payload blob unreadable or corrupted

**Trigger**: read-side hash verify fails OR substrate WORM returns NotFound OR access path broken.

**Detection**: `oya_foundry_evidence_payload_blob_hash_verify_failure_total` > 0.

**Behaviour**: Sev-1 always; forensic capture; if hash mismatch persists, pack marked `integrity_compromised=true`.

**Severity**: Sev-1.

**Owner runbook**: `runbooks/blob-storage-restore.md`.

## FM-10 — Archive cascade lag

**Trigger**: hot→warm or warm→cold cascade fails or is slow.

**Detection**: `oya_foundry_evidence_archive_cascade_lag_hours` > 36.

**Behaviour**: cascade defers (substrate-interlocked); capacity headroom shrinks; alert escalates to Sev-2 if lag exceeds 36h.

**Severity**: Sev-3 default; Sev-2 if lag > 36h.

**Owner runbook**: `runbooks/evidence-archive-migration.md`.

## FM-11 — Pack-region migration mid-flight halt

**Trigger**: replication lag spikes mid-migration; Cedar denial; substrate unavailability.

**Detection**: migration script SLI; substrate health check.

**Behaviour**: revert to sending-region primary; receiving-region paused; ChangeRequest revisited.

**Severity**: Sev-2.

**Owner runbook**: `runbooks/evidence-archive-migration.md` §"Procedure B".

## FM-12 — DSR cascade race (DSR fires while pack assembly in flight)

**Trigger**: tenancy issues DSR for `subject_hash=X` while a pack containing X is being assembled.

**Detection**: pack-builder observes the cascade event mid-assembly.

**Behaviour**: pack assembly completes; substrate immediately applies `RetentionApplied{mode=redact_payload}` to the new pack; foundry-evidence index marks `subject_restricted=true`. Tenant DSR receipt updated to include the new pack.

**Severity**: Sev-3 (operational); audit-emitted recursively per Bominal ADR-0028.

**Owner runbook**: tenancy DSR cascade runbook + foundry-evidence index reflects substrate state via `RetentionApplied` consumer.

## Cross-reference matrix

| FM | Threat-model class | Primary runbook | SLI |
|---|---|---|---|
| FM-01 | T-T-04, T-T-06 | evidence-pack-rebuild | late_signal_total |
| FM-02 | T-D-03 | audit-chain-backlog | audit_chain_emit_backlog_depth_seconds |
| FM-03 | T-D-03 | audit-chain-backlog | audit_chain_emit_backlog_depth_seconds |
| FM-04 | — | pack-assembly-fail | pack_assembled_partial_total |
| FM-05 | T-T-04, T-D-02 | pack-assembly-fail | pack_assembly_failed_total |
| FM-06 | T-T-04 | evidence-pack-rebuild | eval_join_correctness_drill |
| FM-07 | T-T-05 | regulator-export-reissue | regulator_export_field_completeness_drift_total |
| FM-08 | T-I-06 | regulator-export-reissue | regulator_export_scope_violation_total |
| FM-09 | T-T-01, T-T-03 | blob-storage-restore | payload_blob_hash_verify_failure_total |
| FM-10 | T-D-04 | evidence-archive-migration | archive_cascade_lag_hours |
| FM-11 | T-D-04 | evidence-archive-migration | migration_replication_lag_seconds |
| FM-12 | T-R-01 | tenancy DSR cascade runbook | dsr_cascade_race_total |

## Honest-claim posture (ADR-0133)

Every FM-* entry carries a measured SLI + a runbook. No "we'll write the runbook later" entries. The `hyperscaler-maturity-claims` lane refuses commit if any FM in this catalogue lacks an SLI or a runbook pointer.
