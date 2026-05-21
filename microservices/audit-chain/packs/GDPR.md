---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: audit-chain
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# audit-chain GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR audit-chain personal-data evidence overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered audit-chain surface: event classes, Merkle leaves, erasure tombstones, restriction events, portability manifests, transfer evidence, breach evidence, and retention.
- Pack activation means audit-chain must preserve accountability while avoiding unlawful personal-data retention in event payloads.
- The overlay stores hashes, tombstones, and purpose metadata instead of raw personal data.
- Data classes include `AUDIT_CHAIN_GDPR_EVIDENCE`, `AUDIT_CHAIN_GDPR_DSAR_PROOF`, and `AUDIT_CHAIN_GDPR_BREACH_PROOF`.
- GDPR erasure may require cryptographic tombstones and aggregation proof rather than deleting audit integrity lineage.
- ADR-0064 keeps GDPR rights behavior in a pack overlay.
- ADR-0251 supplies breach workflow, retention, and pack signature.
- ADR-0263 supplies downstream event emission linkage.
- PCI-DSS is omitted because audit-chain stores tokenized payment evidence unless PCI scope is active elsewhere.
- Personal-data and PCI obligations may coexist but this document governs GDPR evidence.

## Data Model Deltas
- Add `audit_event.eu_personal_data_signal` as enum `none|personal|special_category`.
- Add `audit_event.lawful_basis`.
- Add `audit_event.processing_purpose_id`.
- Add `audit_event.data_subject_ids_hash`.
- Add `audit_event.personal_payload_hash`.
- Add `audit_event.redaction_manifest_hash`.
- Add `audit_event.erasure_state` as enum `active|restricted|tombstoned|anonymized`.
- Add `audit_event.restriction_reason`.
- Add `audit_event.transfer_mechanism`.
- Add `merkle_leaf.gdpr_pack_version`.
- Add `merkle_leaf.tombstone_replacement_hash`.
- Add `seal_batch.eu_residency_cell`.
- Add `retention_rule.gdpr_schedule_version`.
- Add `export_job.gdpr_evidence_manifest_hash`.
- Add `export_job.dpo_approval_id`.
- Add `query_session.dpo_visible_reason`.
- Add `breach_case.article33_evidence_bundle_hash`.
- Add `dsar_case.audit_chain_manifest_hash`.
- Add `aggregation.anonymity_proof_hash`.
- Add `support_session.dpo_visible_case_id`.
- Add `signature.eu_key_attestation_ref`.
- Add `audit_shadow.audit_chain_gdpr_event_id`.
- Add `tenant_audit_chain_config.eu_dpa_version`.
- Add `tenant_audit_chain_config.gdpr_retention_profile_version`.

## Cedar Policy Deltas
- Policy `GDPR-audit-chain-ingest-01`: require lawful basis for personal-data audit event.
- Policy `GDPR-audit-chain-ingest-02`: forbid raw special-category payload.
- Policy `GDPR-audit-chain-query-01`: require purpose filter for personal-data event query.
- Policy `GDPR-audit-chain-query-02`: restrict DPO query to DPO-visible reason.
- Policy `GDPR-audit-chain-export-01`: require DPO approval for evidence export.
- Policy `GDPR-audit-chain-export-02`: require manifest hash before export release.
- Policy `GDPR-audit-chain-erasure-01`: permit tombstone when lineage can be preserved.
- Policy `GDPR-audit-chain-erasure-02`: require anonymization proof when tombstone impossible.
- Policy `GDPR-audit-chain-restrict-01`: block normal query for restricted event.
- Policy `GDPR-audit-chain-transfer-01`: require transfer mechanism for non-EEA evidence route.
- Policy `GDPR-audit-chain-retention-01`: forbid indefinite retention without basis.
- Policy `GDPR-audit-chain-breach-01`: create Article 33 evidence bundle on confirmed incident.
- Policy `GDPR-audit-chain-route-01`: require EU cell unless transfer mechanism permits.
- Policy `GDPR-audit-chain-key-01`: require key attestation for EU evidence signatures.
- Policy `GDPR-audit-chain-replay-01`: require redaction manifest during event replay.
- Policy `GDPR-audit-chain-support-01`: require DPO-visible support case for personal-data query.
- Policy `GDPR-audit-chain-admin-01`: require elevated ACR for retention profile changes.
- Policy `GDPR-audit-chain-webhook-01`: require DPA proof for external evidence sink.
- Policy `GDPR-audit-chain-compaction-01`: preserve erasure tombstone lineage.
- Policy `GDPR-audit-chain-backfill-01`: require personal-data classifier before backfill.
- Policy `GDPR-audit-chain-index-01`: require index rebuild after tombstone.
- Policy `GDPR-audit-chain-pack-01`: defer deactivation while DSAR or breach cases remain open.
- Policy `GDPR-audit-chain-audit-01`: require self-audit seal for policy change.
- Policy `GDPR-audit-chain-portability-01`: include event metadata manifest in Article 20 export.

## API Contract Deltas
- `POST /events` requires lawful basis when personal-data signal is set.
- `POST /events` rejects raw special-category payload markers.
- `POST /events` accepts redaction manifest hash.
- `GET /events` requires purpose filter for personal-data queries.
- `POST /dsar/export` creates audit-chain metadata manifest.
- `POST /dsar/erasure` starts tombstone or anonymization workflow.
- `POST /dsar/restrict` restricts event query serving.
- `POST /exports` requires DPO approval id.
- `GET /exports/{id}` returns GDPR manifest hash.
- `POST /breach-evidence` creates Article 33 evidence bundle.
- `POST /transfers` records evidence transfer mechanism.
- `POST /replication/plan` validates EU route or transfer proof.
- `POST /keys/attest` stores EU key attestation ref.
- `POST /replay` requires redaction manifest.
- `POST /retention-profiles` requires elevated ACR.
- `POST /webhooks` requires DPA destination proof.
- `POST /backfill` requires personal-data classifier version.
- `POST /search/rebuild` requires tombstone reason.
- `PATCH /tenant-audit-chain-config` records GDPR profile version.
- `POST /pack/deactivate` refuses open DSAR or breach cases.

## Workflow Deltas
- Event ingest workflow validates lawful basis and redaction manifest.
- Special-category detector rejects unsafe payloads.
- Query workflow filters by purpose and role.
- DSAR export workflow builds audit metadata manifest.
- Erasure workflow replaces personal event content with tombstone hash.
- Anonymization workflow creates proof when tombstone cannot remove aggregate lineage.
- Restriction workflow blocks normal event query.
- Search index rebuild runs after tombstone.
- Breach workflow assembles Article 33 evidence bundle.
- Transfer workflow validates EU route or transfer mechanism.
- Retention workflow rejects indefinite schedules.
- Key attestation workflow verifies signing key before use.
- Replay workflow applies redaction manifest.
- Webhook workflow validates DPA destination.
- Compaction workflow preserves tombstone lineage.
- Backfill workflow classifies historical events.
- Retention profile change workflow requires elevated approval.
- Pack activation scans existing exports and webhooks.
- Pack deactivation waits for open DSAR and breach cases.
- Self-audit workflow seals every policy and retention change.

## SLO Deltas
- Personal-data event ingest validation p99 must stay <= 100 ms.
- Special-category rejection p99 must complete <= 30 seconds.
- Purpose-filter query authorization p99 must stay <= 100 ms.
- DSAR audit manifest generation target is <= 4 hours.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Anonymization proof p99 target is <= 24 hours.
- Restriction activation p99 must complete <= 15 minutes.
- Index rebuild target is <= 24 hours after tombstone.
- Article 33 evidence bundle creation p99 target is <= 10 minutes.
- Breach regulator-readiness p99 target is <= 60 hours.
- Transfer mechanism validation p99 must stay <= 200 ms.
- Retention conflict response p99 must stay <= 300 ms.
- Replay redaction throughput target is >= 10k events per minute.
- EU route validation p99 must stay <= 100 ms.
- GDPR audit-chain dashboard lag target is <= 5 minutes.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `AuditChainGdprEventIngested` records event class and basis.
- `AuditChainGdprSpecialCategoryRejected` records detector verdict.
- `AuditChainGdprPurposeQueryApplied` records filter id.
- `AuditChainGdprDsarManifestCreated` records manifest hash.
- `AuditChainGdprEventTombstoned` records replacement hash.
- `AuditChainGdprEventAnonymized` records proof hash.
- `AuditChainGdprRestrictionApplied` records reason.
- `AuditChainGdprRestrictionReleased` records reviewer.
- `AuditChainGdprIndexRebuilt` records shard id.
- `AuditChainGdprBreachEvidenceBundled` records bundle hash.
- `AuditChainGdprTransferMechanismRecorded` records mechanism.
- `AuditChainGdprPurgeRefused` records retention reason.
- `AuditChainGdprKeyAttested` records key ref.
- `AuditChainGdprReplayRedacted` records replay id.
- `AuditChainGdprReplicationBlocked` records target cell.
- `AuditChainGdprWebhookRefused` records destination id.
- `AuditChainGdprCompactionLineagePreserved` records segment id.
- `AuditChainGdprBackfillClassified` records classifier version.
- `AuditChainGdprPolicyChanged` records policy bundle.
- `AuditChainGdprPackDeactivationDeferred` records open cases.

## Failure Modes specific to this pack
- Raw personal data submitted in event body; recovery is reject and quarantine.
- Lawful basis missing; recovery is reject ingest.
- Redaction manifest missing; recovery is reject event.
- Erasure tombstone breaks Merkle lineage; recovery is use replacement hash and preserve chain proof.
- Anonymization proof cannot be generated; recovery is restrict query and page DPO.
- Search index still serves tombstoned event; recovery is remove shard and rebuild.
- Query lacks purpose filter; recovery is deny query.
- Export manifest mismatch appears; recovery is revoke export and rebuild.
- Breach bundle misses event segment; recovery is rebuild from Merkle range.
- Transfer mechanism expires; recovery is block non-EEA route.
- Retention purge requested without basis; recovery is refuse purge.
- Key attestation expires; recovery is pause EU evidence signing and rotate key.
- Replay bypasses redaction; recovery is halt replay.
- Webhook lacks DPA; recovery is disable sink.
- Compaction removes tombstone lineage; recovery is block compaction.
- Backfill classifier missing; recovery is block backfill.
- Support query lacks DPO-visible case; recovery is deny access.
- Pack deactivation requested with open cases; recovery is defer.
- Audit-chain backpressure appears; recovery is fail-closed for personal-data event writes.
- Data subject hash collision suspicion appears; recovery is recompute with stronger salt.

## Cross-µservice coordination
- `tenancy` provides EU cell placement and active pack roster.
- `identity` provides DPO and data-subject verification roles.
- `compliance` provides DSAR, transfer, and breach workflow state.
- `observability` emits personal-data-safe seal and verification metrics.
- `policy-engine` loads all `GDPR-audit-chain-*` fragments.
- `workflow-engine` runs DSAR, erasure, breach, and export workflows.
- `mail` emits GDPR mail event classes into audit-chain.
- `drive` emits GDPR file event classes into audit-chain.
- `calendar` emits GDPR schedule event classes into audit-chain.
- `storage` provides EU evidence backend proof.
- `cloud-kms` or OpenBao provides key attestation.
- `incident-response` consumes Article 33 evidence bundles.
- `admin-console` renders GDPR evidence status.
- `legal` defines redaction and transfer rules.
- `support` uses DPO-visible query path.
- `data-warehouse` receives aggregate audit health metrics.
- `notification` routes DPO deadline alerts.
- `release-engine` gates event schema changes.
- `search` rebuilds tombstone indexes.
- `pack-registry` signs this GDPR audit-chain overlay.
