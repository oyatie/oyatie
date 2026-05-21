---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: audit-chain
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# audit-chain KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act audit-chain evidence overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered audit-chain surface: event classes, Merkle leaves, consent ledgers, retention ledgers, transfer proof, processor delegation proof, breach evidence, exports, and query access.
- Pack activation means audit-chain seals Korean privacy evidence while preventing raw Korean personal information in event payloads.
- The overlay stores hashes and Korean ledger references, not raw resident registration numbers or sensitive PI.
- Data classes include `AUDIT_CHAIN_KR_PIPA_EVIDENCE`, `AUDIT_CHAIN_KR_CONSENT_PROOF`, `AUDIT_CHAIN_KR_RETENTION_PROOF`, and `AUDIT_CHAIN_KR_BREACH_PROOF`.
- Korean-language notice version is preserved in evidence manifests.
- ADR-0064 keeps Korean evidence behavior in a pack overlay.
- ADR-0251 supplies retention, breach workflow, and pack signature.
- ADR-0263 supplies event emission linkage.
- PCI-DSS is omitted because audit-chain is not payment authorization.
- PAN-like evidence is hashed and routed to the payment compliance owner if detected.

## Data Model Deltas
- Add `audit_event.kr_pi_signal` as enum `none|personal|sensitive|rrn`.
- Add `audit_event.kr_event_class`.
- Add `audit_event.kr_payload_hash`.
- Add `audit_event.kr_consent_id`.
- Add `audit_event.kr_retention_basis_id`.
- Add `audit_event.kr_cross_border_transfer_id`.
- Add `audit_event.kr_processor_delegation_id`.
- Add `audit_event.kr_notice_version`.
- Add `audit_event.kr_subject_hash`.
- Add `audit_event.kr_breach_candidate_id`.
- Add `merkle_leaf.kr_pipa_pack_version`.
- Add `merkle_leaf.kr_redaction_manifest_hash`.
- Add `seal_batch.kr_cell_id`.
- Add `retention_rule.kr_floor_or_expiry_ref`.
- Add `export_job.kr_evidence_manifest_hash`.
- Add `export_job.kr_dpo_approval_id`.
- Add `query_session.kr_purpose_filter`.
- Add `support_session.kr_dpo_visible_case_id`.
- Add `signature.kr_key_attestation_ref`.
- Add `breach_case.kr_evidence_bundle_hash`.
- Add `consent_ledger_event.kr_consent_text_hash`.
- Add `audit_shadow.audit_chain_kr_pipa_event_id`.
- Add `tenant_audit_chain_config.kr_retention_profile_version`.
- Add `tenant_audit_chain_config.kr_notice_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-audit-chain-ingest-01`: require Korean payload hash for Korean PI events.
- Policy `KRPIPA-audit-chain-ingest-02`: forbid raw RRN or sensitive PI in event body.
- Policy `KRPIPA-audit-chain-consent-01`: require 동의 ledger id for consent-based event.
- Policy `KRPIPA-audit-chain-retention-01`: require 보존 basis for retained event.
- Policy `KRPIPA-audit-chain-transfer-01`: require 국외이전 id for cross-border evidence route.
- Policy `KRPIPA-audit-chain-processor-01`: require 처리위탁 id for processor evidence sink.
- Policy `KRPIPA-audit-chain-query-01`: require purpose filter for Korean PI event query.
- Policy `KRPIPA-audit-chain-query-02`: require DPO-visible case for support query.
- Policy `KRPIPA-audit-chain-export-01`: require KR DPO approval for evidence export.
- Policy `KRPIPA-audit-chain-export-02`: require Korean evidence manifest before release.
- Policy `KRPIPA-audit-chain-erasure-01`: permit tombstone when retention basis expires.
- Policy `KRPIPA-audit-chain-erasure-02`: preserve ledger lineage during tombstone.
- Policy `KRPIPA-audit-chain-breach-01`: create KR breach evidence bundle on confirmed leak.
- Policy `KRPIPA-audit-chain-route-01`: require KR cell for Korean resident evidence.
- Policy `KRPIPA-audit-chain-key-01`: require key attestation for KR evidence signatures.
- Policy `KRPIPA-audit-chain-replay-01`: require redaction manifest during replay.
- Policy `KRPIPA-audit-chain-admin-01`: require elevated ACR for retention profile changes.
- Policy `KRPIPA-audit-chain-webhook-01`: require processor delegation for external evidence sink.
- Policy `KRPIPA-audit-chain-compaction-01`: preserve Korean ledger lineage.
- Policy `KRPIPA-audit-chain-backfill-01`: require RRN classifier before backfill.
- Policy `KRPIPA-audit-chain-notice-01`: require Korean notice version for subject-facing export.
- Policy `KRPIPA-audit-chain-pack-01`: defer deactivation while Korean ledgers are open.
- Policy `KRPIPA-audit-chain-audit-01`: require self-audit seal for policy change.
- Policy `KRPIPA-audit-chain-minimum-01`: restrict event fields by purpose and role.

## API Contract Deltas
- `POST /events` requires Korean payload hash when KR PI signal is set.
- `POST /events` rejects raw RRN markers.
- `POST /events` accepts consent, retention, transfer, and processor ledger ids.
- `POST /seal-batches` requires KR cell id.
- `GET /events` requires purpose filter for Korean PI queries.
- `POST /exports` requires KR DPO approval id.
- `GET /exports/{id}` returns Korean evidence manifest hash.
- `POST /dsr/erasure` starts Korean tombstone workflow.
- `POST /breach-evidence` creates KR evidence bundle hash.
- `POST /replication/plan` rejects non-KR target for resident evidence.
- `POST /keys/attest` stores KR key attestation ref.
- `POST /replay` requires redaction manifest.
- `POST /retention-profiles` requires elevated ACR.
- `POST /webhooks` requires processor delegation id.
- `POST /backfill` requires RRN classifier version.
- `POST /notices` records Korean notice version.
- `DELETE /events/{id}` returns retention conflict.
- `PATCH /tenant-audit-chain-config` records notice and retention profile versions.
- `POST /support/case-view` requires DPO-visible case id.
- `POST /pack/deactivate` returns open Korean ledger count.

## Workflow Deltas
- Event ingest workflow validates Korean payload hash.
- Raw RRN detector quarantines unsafe events before seal.
- Consent ledger workflow seals 동의 event references.
- Retention ledger workflow seals 보존 event references.
- Transfer workflow seals 국외이전 evidence.
- Processor workflow seals 처리위탁 evidence.
- Query workflow filters by purpose and role.
- Export workflow builds Korean evidence manifest with notice version.
- Tombstone workflow preserves ledger lineage.
- Breach workflow assembles Korean evidence bundle.
- Replication workflow validates KR target cell.
- Key attestation workflow verifies signing key before use.
- Replay workflow applies redaction manifest.
- Webhook workflow validates processor delegation.
- Compaction workflow preserves Korean ledger lineage.
- Backfill workflow classifies historical events for RRN.
- Retention profile change workflow requires elevated approval.
- Pack activation workflow scans existing exports and webhooks.
- Pack deactivation waits for open Korean ledgers.
- Self-audit workflow seals every policy and retention change.

## SLO Deltas
- Korean PI event ingest validation p99 must stay <= 100 ms.
- Raw RRN quarantine p99 must complete <= 30 seconds.
- KR seal batch p99 must complete <= 1 second after batch close.
- Consent ledger event seal p99 must complete <= 1 second.
- Retention ledger event seal p99 must complete <= 1 second.
- Transfer evidence seal p99 must complete <= 1 second.
- Processor evidence seal p99 must complete <= 1 second.
- Purpose-filter query authorization p99 must stay <= 100 ms.
- Export manifest generation p99 target is <= 30 minutes.
- KR breach evidence bundle creation p99 target is <= 10 minutes.
- KR route validation p99 must stay <= 100 ms.
- Key attestation lookup p99 must stay <= 200 ms.
- Replay redaction throughput target is >= 10k events per minute.
- Backfill classifier validation p99 target is <= 15 minutes per batch.
- Korean audit-chain dashboard lag target is <= 5 minutes.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `AuditChainKrPipaEventIngested` records event class and hash.
- `AuditChainKrPipaRawRrnRejected` records detector verdict.
- `AuditChainKrPipaConsentEventSealed` records consent id.
- `AuditChainKrPipaRetentionEventSealed` records 보존 basis.
- `AuditChainKrPipaTransferEventSealed` records transfer id.
- `AuditChainKrPipaProcessorEventSealed` records delegation id.
- `AuditChainKrPipaPurposeQueryApplied` records filter id.
- `AuditChainKrPipaExportManifestCreated` records manifest hash.
- `AuditChainKrPipaEventTombstoned` records replacement hash.
- `AuditChainKrPipaBreachEvidenceBundled` records bundle hash.
- `AuditChainKrPipaReplicationBlocked` records target cell.
- `AuditChainKrPipaKeyAttested` records key ref.
- `AuditChainKrPipaReplayRedacted` records replay id.
- `AuditChainKrPipaWebhookRefused` records sink id.
- `AuditChainKrPipaCompactionLineagePreserved` records segment id.
- `AuditChainKrPipaBackfillClassified` records classifier version.
- `AuditChainKrPipaNoticeVersionRecorded` records notice version.
- `AuditChainKrPipaRetentionProfileChanged` records profile version.
- `AuditChainKrPipaPolicyChanged` records policy bundle.
- `AuditChainKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- Raw RRN is submitted in event body; recovery is quarantine and reject seal.
- Korean payload hash missing; recovery is reject ingest.
- Consent ledger id missing for consent-based event; recovery is reject event.
- Retention basis missing; recovery is reject event.
- Transfer id missing for cross-border route; recovery is block route.
- Processor delegation id missing; recovery is disable evidence sink.
- KR cell unavailable; recovery is buffer or reject Korean resident evidence.
- Korean notice version missing for subject-facing export; recovery is reject export.
- Export manifest mismatch appears; recovery is revoke and rebuild.
- Tombstone would remove ledger lineage; recovery is use replacement hash.
- Breach bundle misses event segment; recovery is rebuild from Merkle range.
- Key attestation expires; recovery is pause KR evidence signing.
- Replay bypasses redaction; recovery is halt replay.
- Webhook lacks processor delegation; recovery is disable sink.
- Compaction removes Korean ledger lineage; recovery is block compaction.
- Backfill classifier missing; recovery is block backfill.
- Support query lacks DPO-visible case; recovery is deny access.
- Pack deactivation requested with open ledgers; recovery is defer.
- Audit-chain backpressure appears; recovery is fail-closed for Korean PI event writes.
- Subject hash collision suspicion appears; recovery is recompute with stronger salt.

## Cross-µservice coordination
- `tenancy` provides KR cell placement and active KR-PIPA roster.
- `identity` provides KR DPO and subject verification roles.
- `compliance` provides consent, retention, transfer, processor, and breach workflows.
- `observability` emits Korean PI-safe seal and verification metrics.
- `policy-engine` loads all `KRPIPA-audit-chain-*` fragments.
- `workflow-engine` runs subject-rights, breach, export, and retention workflows.
- `mail` emits KR-PIPA mail event classes into audit-chain.
- `drive` emits KR-PIPA file event classes into audit-chain.
- `calendar` emits KR-PIPA schedule event classes into audit-chain.
- `storage` provides KR evidence backend proof.
- `cloud-kms` or OpenBao provides key attestation.
- `incident-response` consumes Korean breach evidence bundles.
- `admin-console` renders KR evidence status.
- `legal` defines Korean export redaction and notice rules.
- `support` uses DPO-visible query path.
- `data-warehouse` receives aggregate audit health metrics.
- `notification` routes DPO deadline alerts.
- `release-engine` gates event schema changes.
- `dlp-virus-scan` screens backfill payloads for RRN.
- `pack-registry` signs this KR-PIPA audit-chain overlay.
