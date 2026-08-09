---
id: ADR-REC-001
title: Recording Retention WORM vs Mutable vs Purge on Request
status: Proposed
date: 2026-05-20
microservice: recordings
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0703-cas-cache-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-recordings
---

# ADR-REC-001: Recording Retention WORM vs Mutable vs Purge on Request

## Context

- Recordings owns durable storage, transcript, redaction, retention, legal hold, eDiscovery, playback, and export for recording artifacts.
- The PRD says recordings is the central archive for meet, messenger huddles, live broadcast, manual uploads, and screen captures.
- Existing ADR-RECORDINGS-0002 names retention and legal-hold policy.
- Existing ADR-RECORDINGS-0003 names redaction and PII policy.
- This ADR chooses the storage-retention mode matrix for regulated and non-regulated recordings.
- Named pressure REC-P1: financial tenants can require SEC 17a-4(f) compliant electronic recordkeeping.
- Named pressure REC-P2: healthcare tenants can require HIPAA administrative, physical, and technical safeguards for ePHI.
- Named pressure REC-P3: GDPR and KR PIPA require storage limitation, purpose limitation, and deletion pathways.
- Named pressure REC-P4: legal hold and eDiscovery require preservation even when a user requests deletion.
- Named pressure REC-P5: redaction overlays must not mutate source media when source preservation is required.
- Named pressure REC-P6: tenant admins need retention policy control without bypassing statutory floors.
- Named pressure REC-P7: personal recordings need purge-on-request when no legal basis or hold exists.
- Named pressure REC-P8: audit exports must prove chain of custody without exposing raw keys.
- Named pressure REC-P9: transcript search must hide purged or redacted content immediately.
- Named pressure REC-P10: data residency must keep recordings in their pack-pinned cell.
- Constraint REC-C1: every hold, purge, restore, export, retention extension, and redaction emits audit-chain evidence per ADR-0003.
- Constraint REC-C2: data subject request cascade and proof of erasure follow ADR-0038.
- Constraint REC-C3: PHI, PII, financial records, and general recordings use per-microservice data classes per ADR-0034.
- Constraint REC-C4: encryption keys and purge tombstones use OpenBao and HSM-backed custody per ADR-0043.
- Constraint REC-C5: cross-region replication cannot violate residency per ADR-0049.
- Constraint REC-C6: RPO and RTO claims must follow ADR-0152.
- Constraint REC-C7: Cedar gates retention mutation, legal hold, purge, export, and reviewer access per ADR-0243.
- Constraint REC-C8: observability and dashboards follow ADR-0263.
- Constraint REC-C9: recordings remains flat under ADR-0131 and does not absorb compliance, audit-chain, or legal workflow ownership.
- Constraint REC-C10: WORM mode must support either non-rewriteable storage or audit-trail equivalence where regulation permits.
- Mutable source media is convenient for user edits.
- Mutable source media is not acceptable for legal hold or regulated financial records.
- Purge-on-request supports privacy rights.
- Purge-on-request cannot override legal hold, active investigation, statutory retention, or safety preservation.
- WORM storage protects record integrity.
- WORM storage creates deletion and correction tension under privacy law.
- The system needs an explicit mode matrix, not ad hoc per-tenant flags.

## Decision

- Adopt `RecordingRetentionModeMatrix v1`.
- Define retention modes `worm_record`, `mutable_record`, `purge_on_request`, and `redaction_overlay_only`.
- Use `worm_record` for SEC 17a-4(f) scoped broker-dealer communications, active legal hold, regulator export sets, and signed eDiscovery bundles.
- Use `mutable_record` for normal enterprise recordings that allow transcript correction, chapter updates, and admin-managed retention within pack bounds.
- Use `purge_on_request` for personal or non-regulated recordings when no hold, statutory floor, safety block, or counterparty interest exists.
- Use `redaction_overlay_only` when source media must be preserved but playback and transcript display must hide specific segments.
- Store source media as immutable once a recording crosses into `worm_record`.
- Store transcripts as versioned derived records with a source-media hash binding.
- Store redactions as overlay objects, never destructive edits to source media in WORM or legal-hold mode.
- Store mutable corrections as new transcript versions, not in-place mutation of historical transcript rows.
- Use key shredding only after Cedar confirms purge eligibility and hold absence.
- Write purge tombstones to audit-chain before deleting search index entries.
- Delete search index entries before object deletion returns success to callers.
- Keep `proof_of_erasure_ref` after content purge, with no recoverable media or transcript bytes.
- Represent SEC 17a-4(f) tenants as retention pack overlays, not one-off tenant code.
- Represent HIPAA tenants as six-year policy floors for applicable ePHI artifacts.
- Represent GDPR Article 17 and KR PIPA erasure requests as DSR workflows that can be lawfully denied or partially fulfilled.
- Separate legal basis from storage mode.
- Require two-person approval for `worm_record` release, retention override, and eDiscovery export.
- Require DSR officer approval for rejected purge requests.
- Require tenant admin approval for routine retention policy shortening.
- Require compliance officer approval for legal hold engagement and release.
- Emit `recordings.retention.mode.changed.v1` on every mode transition.
- Emit `recordings.purge.requested.v1` and `recordings.purge.completed.v1` for privacy requests.
- Emit `recordings.worm.locked.v1` when a record enters immutable mode.
- Emit `recordings.legal_hold.engaged.v1` and `recordings.legal_hold.released.v1`.
- Keep playback authorization independent of retention mode.
- Keep deletion eligibility computation in the recordings use case layer.
- Keep cross-service DSR orchestration outside recordings, but expose typed hooks for it.
- Make this ADR authoritative for retention mode selection, not for transcript model selection or diarization engine choice.

## Alternatives Considered

### WORM for Every Recording

- Pros: strongest preservation posture.
- Pros: simpler legal-hold semantics.
- Pros: easy chain-of-custody argument.
- Cons: conflicts with purge-on-request for ordinary personal data.
- Cons: increases storage cost and operational rigidity.
- Cons: makes user correction workflows harder.
- Rejected because universal WORM ignores GDPR, KR PIPA, and normal tenant retention flexibility.

### Mutable Storage for Every Recording

- Pros: simplest user edit path.
- Pros: easy transcript correction and media replacement.
- Pros: lowest object-lock complexity.
- Cons: fails regulated record integrity expectations.
- Cons: weakens eDiscovery chain of custody.
- Cons: legal hold can be bypassed by accidental mutation.
- Rejected because financial and legal-hold recordings need stronger preservation.

### Purge on Every User Request

- Pros: strongest apparent privacy responsiveness.
- Pros: lowers storage footprint.
- Pros: simple user-facing promise.
- Cons: unlawful when statutory retention or legal hold applies.
- Cons: can destroy evidence required for disputes, abuse reports, or regulators.
- Cons: undermines enterprise compliance features.
- Rejected because erasure rights are conditional and must be reconciled with legal obligations.

### External Archiving Vendor as System of Record

- Pros: mature SEC and FINRA-oriented retention features.
- Pros: compliance attestation may already exist.
- Pros: lower first-build complexity.
- Cons: conflicts with first-party archive ownership.
- Cons: complicates pack-pinned residency and per-tenant Cedar policy.
- Cons: makes audit-chain evidence dependent on vendor export fidelity.
- Rejected for core archive; export adapters can exist for tenant-approved downstream archives.

### Audit-Trail-Only Retention

- Pros: can satisfy amended SEC electronic recordkeeping alternatives where accepted.
- Pros: more flexible than physical WORM.
- Pros: easier cloud-native implementation.
- Cons: still needs strict immutability of audit events.
- Cons: regulators and tenants may demand WORM representations.
- Cons: audit-trail equivalence needs strong operational controls.
- Accepted only as a pack-configured equivalent, never as the universal default.

## Consequences

- Positive: retention behavior is explicit and testable per recording.
- Positive: SEC, HIPAA, GDPR, and KR PIPA tensions are handled through policy, not ad hoc exceptions.
- Positive: legal hold cannot be bypassed by user deletion requests.
- Positive: ordinary personal recordings can still be purged when eligible.
- Positive: redaction overlays preserve chain of custody while hiding sensitive segments.
- Positive: DSR responses can explain lawful denial, partial erasure, or completed purge.
- Positive: search index deletion happens before purge success, reducing stale disclosure risk.
- Negative: retention-mode computation becomes a load-bearing policy engine.
- Negative: WORM and overlay storage increase data model complexity.
- Negative: purge workflows must coordinate object storage, transcript store, search, cache, and audit-chain.
- Negative: partial erasure can be confusing to end users without careful status names.
- Neutral: immutable source media can coexist with mutable transcript corrections.
- Neutral: eDiscovery export bundles remain separate from routine playback.
- Neutral: a recording can move from mutable to WORM, but not from WORM to mutable without explicit release workflow.
- Neutral: HIPAA retention applies only when recording data class is ePHI or policy overlay says so.
- Follow-up work REC-F1: add retention-mode matrix fixtures under recordings tests.
- Follow-up work REC-F2: add purge tombstone schema to the DSR contract.
- Follow-up work REC-F3: add WORM lock evidence panel to compliance dashboard.
- Follow-up work REC-F4: add legal-hold release runbook.
- Follow-up work REC-F5: add SEC audit-trail-equivalence pack notes.

## Implementation Notes

- Data shape `RecordingObject`: `{tenant_id, recording_id, source_service, data_class, home_cell, source_media_ref, source_hash, created_at}`.
- Data shape `RetentionPolicy`: `{tenant_id, policy_id, pack_code, statutory_floor_days, tenant_retention_days, purge_mode, worm_required}`.
- Data shape `RetentionModeDecision`: `{recording_id, mode, legal_basis, policy_id, hold_ids, decided_at, permit_id}`.
- Data shape `LegalHold`: `{tenant_id, hold_id, scope_kind, scope_ref, reason_code, requested_by, approved_by, starts_at, released_at}`.
- Data shape `RedactionOverlay`: `{recording_id, overlay_id, segment_ranges, transcript_spans, reason_code, reviewer_id, audit_event_id}`.
- Data shape `PurgeRequest`: `{tenant_id, request_id, subject_id, recording_id, jurisdiction, requested_at, status, denial_reason}`.
- Data shape `PurgeTombstone`: `{tenant_id, recording_id, request_id, proof_of_erasure_ref, deleted_at, retained_metadata_fields}`.
- Data shape `WormLockEvidence`: `{recording_id, object_lock_ref, retention_until, hash, regulator_profile, audit_event_id}`.
- Postgres table `recording_object` stores source identity and immutable hashes.
- Postgres table `recording_retention_policy` stores tenant and pack retention rules.
- Postgres table `recording_retention_mode_decision` stores per-recording decisions.
- Postgres table `recording_legal_hold` stores hold scopes and release approvals.
- Postgres table `recording_redaction_overlay` stores non-destructive redactions.
- Postgres table `recording_purge_request` stores DSR state.
- Object path `recordings/{tenant_id}/{recording_id}/source` stores source media.
- Object path `recordings/{tenant_id}/{recording_id}/transcripts/{version}` stores transcript versions.
- Object path `recordings/{tenant_id}/{recording_id}/overlays/{overlay_id}` stores redaction overlays.
- Object lock profile `recordings-sec-17a4f` applies WORM or audit-trail equivalent controls.
- OpenBao path `secret/<tenant_id>/recordings/purge/<request_id>` stores purge workflow signing material.
- REST endpoint `GET /v1/recordings/{recording_id}/retention-mode` returns current mode and reason.
- REST endpoint `POST /v1/recordings/{recording_id}/legal-holds` engages hold.
- REST endpoint `POST /v1/recordings/{recording_id}/legal-holds/{hold_id}/release` releases hold with approval.
- REST endpoint `POST /v1/recordings/{recording_id}/purge-requests` starts DSR deletion review.
- REST endpoint `POST /v1/recordings/{recording_id}/redaction-overlays` creates overlay redaction.
- REST endpoint `POST /v1/recordings/{recording_id}/retention-policy/evaluate` recomputes mode after policy changes.
- REST endpoint `POST /v1/recordings/{recording_id}/ediscovery-export` creates export bundle.
- AsyncAPI channel `recordings.retention.mode.changed.v1` publishes mode changes.
- AsyncAPI channel `recordings.worm.locked.v1` publishes WORM lock evidence.
- AsyncAPI channel `recordings.legal_hold.engaged.v1` publishes hold engagement.
- AsyncAPI channel `recordings.purge.requested.v1` publishes DSR intake.
- AsyncAPI channel `recordings.purge.completed.v1` publishes proof-of-erasure reference.
- Cedar action `recordings::retention::change_policy` requires tenant admin and pack floor compliance.
- Cedar action `recordings::legal_hold::engage` requires compliance officer role.
- Cedar action `recordings::legal_hold::release` requires two-person approval.
- Cedar action `recordings::purge::approve` requires DSR officer and no active hold.
- Cedar action `recordings::ediscovery::export` requires legal basis and two-person approval.
- Cedar action `recordings::redaction::overlay_create` requires reviewer purpose and audit reason.
- SLO target `recordings_legal_hold_engage_correctness_ratio` is 1.0.
- SLO target `recordings_legal_hold_engage_p99_ms` is <=1000.
- SLO target `recordings_search_purge_visibility_p99_ms` is <=5000.
- SLO target `recordings_purge_completion_p95_hours` is <=24 for eligible purge.
- SLO target `recordings_worm_lock_evidence_p99_ms` is <=1000 after mode transition.

## Verification

- Unit test `worm_mode_refuses_source_mutation` proves immutable source enforcement.
- Unit test `purge_request_denied_when_legal_hold_active` proves hold precedence.
- Unit test `hipaa_policy_floor_blocks_shorter_tenant_retention` proves statutory floor handling.
- Unit test `gdpr_purge_generates_tombstone_without_media_ref` proves erasure proof shape.
- Unit test `kr_pipa_request_uses_pack_jurisdiction_rules` proves pack overlay selection.
- Unit test `redaction_overlay_does_not_mutate_source_hash` proves overlay-only redaction.
- Unit test `sec_17a4f_profile_requires_worm_or_audit_trail` proves regulated mode.
- Contract test `retention_mode_endpoint_explains_legal_basis` proves admin-visible state.
- Contract test `purge_completed_event_contains_proof_ref_only` proves no leaked object refs.
- Property test `retention_mode_is_monotonic_under_active_hold` proves WORM cannot silently downgrade.
- Replay test `purge_replay_is_idempotent_after_search_delete` proves retry safety.
- Integration test `search_index_hides_recording_before_purge_success` proves stale disclosure prevention.
- Integration test `ediscovery_export_contains_media_transcript_overlay_and_hash` proves chain-of-custody bundle.
- Failure test `object_store_delete_failure_keeps_purge_pending` proves no false success.
- Failure test `audit_chain_unavailable_blocks_worm_release` proves evidence-first posture.
- Security test `tenant_admin_cannot_release_legal_hold_alone` proves two-person Cedar gate.
- Security test `viewer_playback_does_not_reveal_redacted_segment` proves overlay enforcement.
- Metric `recordings_retention_mode_total` tracks mode distribution by pack.
- Metric `recordings_legal_hold_engage_latency_ms` tracks hold latency.
- Metric `recordings_purge_request_total` tracks requested, completed, denied, and partial outcomes.
- Metric `recordings_search_purge_visibility_lag_ms` tracks index purge latency.
- Metric `recordings_worm_lock_failure_total` tracks object-lock failures.
- Metric `recordings_redaction_overlay_apply_latency_ms` tracks playback overlay cost.
- Dashboard `recordings-retention-mode-matrix` shows mode distribution, statutory floors, and transition failures.
- Dashboard `recordings-dsr-purge` shows purge backlog, denials, proof generation, and stale index risk.
- Dashboard `recordings-legal-hold` shows hold engagement latency, active holds, and release approvals.
- Dashboard `recordings-worm-integrity` shows WORM lock evidence, hash mismatches, and export readiness.
- Alert `RecordingsLegalHoldCorrectnessViolation` fires on any missed hold enforcement.
- Alert `RecordingsPurgeVisibilityLag` fires when search purge p99 exceeds 5 seconds.
- Alert `RecordingsWormLockFailure` fires on any failed WORM transition.
- Alert `RecordingsPurgeFalseSuccess` fires if media bytes remain after completed purge.

## References

- Internal: microservices/recordings/PRD.md
- Internal: microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md
- Internal: microservices/recordings/decisions/ADR-RECORDINGS-0003-redaction-and-pii-policy.md
- Internal: docs/decisions/ADR-0703-cas-cache-live-apex.md
- Internal: docs/decisions/ADR-0700-ci-admission-live-apex.md
- SEC amendments to electronic recordkeeping requirements: https://www.sec.gov/investment/amendments-electronic-recordkeeping-requirements-broker-dealers
- SEC Rule 17a-4(f) electronic storage guidance: https://www.sec.gov/rules-regulations/2001/05/commission-guidance-broker-dealers-use-electronic-storage-media-under-electronic-signatures-global
- HHS HIPAA Security Rule: https://www.hhs.gov/hipaa/for-professionals/security/index.html
- HHS summary of HIPAA Security Rule: https://www.hhs.gov/hipaa/for-professionals/security/laws-regulations/
- GDPR Regulation 2016/679 Article 17: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
- Korea Personal Information Protection Act English text: https://www.law.go.kr/LSW/lsInfoP.do?lsiSeq=213857&urlMode=engLsInfoR&viewCls=engLsInfoR
- Korea PIPC English portal: https://www.pipc.go.kr/eng/
- NIST SP 800-88 Rev. 1 media sanitization: https://csrc.nist.gov/publications/detail/sp/800-88/rev-1/final
- ISO/IEC 27037 digital evidence guidance overview: https://www.iso.org/standard/44381.html
- AWS S3 Object Lock documentation: https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html
- Sedona Conference publications: https://thesedonaconference.org/publications
- FINRA Rule 4511 books and records: https://www.finra.org/rules-guidance/rulebooks/finra-rules/4511
