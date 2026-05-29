---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: audit-chain
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# audit-chain HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification audit-chain ePHI evidence overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered audit-chain surface: audit event classes, Merkle leaves, seal cadence, retention, export manifests, signature verification, disclosure accounting, and breach evidence.
- Pack activation means audit-chain stores PHI-related evidence without raw PHI and retains HIPAA audit proof.
- The overlay treats event payload, reason text, actor metadata, and export filters as possible PHI leak surfaces.
- Data classes include `AUDIT_CHAIN_PHI_EVIDENCE`, `AUDIT_CHAIN_HIPAA_DISCLOSURE`, and `AUDIT_CHAIN_HIPAA_BREACH_PROOF`.
- Minimum necessary applies to event query, export, and support access.
- ADR-0064 keeps audit-chain canonical while this pack adds HIPAA event and retention deltas.
- ADR-0251 supplies retention, breach workflow, and pack activation semantics.
- ADR-0263 supplies event emission linkage from producing services.
- This overlay excludes PCI-DSS because audit-chain stores tokenized payment evidence unless a payment service activates PCI scope.
- Cardholder-data evidence is hashed and routed to PCI pack owners if detected.

## Data Model Deltas
- Add `audit_event.phi_signal` as enum `none|possible|confirmed`.
- Add `audit_event.phi_event_class` nullable.
- Add `audit_event.phi_payload_hash` for raw-PHI-free proof.
- Add `audit_event.minimum_necessary_scope`.
- Add `audit_event.patient_context_id_hash`.
- Add `audit_event.disclosure_accounting_id`.
- Add `audit_event.break_glass_reason_id`.
- Add `audit_event.hipaa_purpose` as enum `treatment|payment|operations|patient_request|audit`.
- Add `merkle_leaf.hipaa_pack_version`.
- Add `merkle_leaf.phi_redaction_manifest_hash`.
- Add `seal_batch.hipaa_certified_cell_id`.
- Add `seal_batch.tamper_detection_due_at`.
- Add `retention_rule.hipaa_floor_iso8601` default `P6Y`.
- Add `export_job.hipaa_evidence_manifest_hash`.
- Add `export_job.privacy_officer_approval_id`.
- Add `query_session.minimum_necessary_filter`.
- Add `support_session.break_glass_reason_id`.
- Add `signature.hipaa_key_attestation_ref`.
- Add `breach_case.audit_chain_evidence_bundle_hash`.
- Add `disclosure_accounting.recipient_org_hash`.
- Add `disclosure_accounting.purpose`.
- Add `audit_shadow.audit_chain_hipaa_event_id`.
- Add `tenant_audit_chain_config.hipaa_cell_certification`.
- Add `tenant_audit_chain_config.hipaa_retention_profile_version`.

## Cedar Policy Deltas
- Policy `HIPAA-audit-chain-ingest-01`: require PHI payload hash and redaction manifest for PHI events.
- Policy `HIPAA-audit-chain-ingest-02`: forbid raw PHI payload in audit event body.
- Policy `HIPAA-audit-chain-seal-01`: require HIPAA-certified cell for PHI seal batch.
- Policy `HIPAA-audit-chain-query-01`: require minimum necessary filter for PHI event query.
- Policy `HIPAA-audit-chain-query-02`: permit privacy officer disclosure query by patient context hash.
- Policy `HIPAA-audit-chain-export-01`: require privacy-office approval for HIPAA evidence export.
- Policy `HIPAA-audit-chain-export-02`: require evidence manifest before export release.
- Policy `HIPAA-audit-chain-retention-01`: forbid purge before six-year floor.
- Policy `HIPAA-audit-chain-breakglass-01`: permit emergency query only with reason id and TTL <= 1h.
- Policy `HIPAA-audit-chain-disclosure-01`: require purpose for disclosure accounting event.
- Policy `HIPAA-audit-chain-breach-01`: create breach evidence bundle for confirmed PHI incident.
- Policy `HIPAA-audit-chain-route-01`: forbid replication outside HIPAA-certified cell.
- Policy `HIPAA-audit-chain-key-01`: require key attestation for HIPAA seal signatures.
- Policy `HIPAA-audit-chain-verify-01`: require tamper detection verification on HIPAA cadence.
- Policy `HIPAA-audit-chain-replay-01`: require redaction manifest during event replay.
- Policy `HIPAA-audit-chain-support-01`: require covered workforce role for support query.
- Policy `HIPAA-audit-chain-admin-01`: require elevated ACR for retention profile changes.
- Policy `HIPAA-audit-chain-webhook-01`: require BAA proof for external evidence sink.
- Policy `HIPAA-audit-chain-compaction-01`: forbid compaction that removes PHI evidence lineage.
- Policy `HIPAA-audit-chain-backfill-01`: require historical PHI classifier before backfill.
- Policy `HIPAA-audit-chain-quarantine-01`: quarantine event if raw PHI detector fires.
- Policy `HIPAA-audit-chain-pack-01`: defer deactivation while HIPAA evidence remains retained.
- Policy `HIPAA-audit-chain-audit-01`: require self-audit seal for policy change.
- Policy `HIPAA-audit-chain-minimum-01`: restrict event fields by purpose and role.

## API Contract Deltas
- `POST /events` requires `phi_payload_hash` when `phi_signal=confirmed`.
- `POST /events` rejects raw PHI payload markers.
- `POST /events` requires `hipaa_purpose`.
- `POST /seal-batches` requires HIPAA-certified cell id.
- `POST /disclosures` requires purpose and recipient organization hash.
- `GET /events` requires minimum necessary filter for PHI queries.
- `POST /exports` requires privacy-office approval id.
- `GET /exports/{id}` returns HIPAA evidence manifest hash.
- `DELETE /events/{id}` returns retention conflict before six-year floor.
- `POST /support/break-glass` requires reason id and TTL.
- `POST /breach-evidence` creates evidence bundle hash.
- `POST /replication/plan` rejects uncertified target cell.
- `POST /keys/attest` stores HIPAA key attestation ref.
- `POST /verify/tamper` records HIPAA verification result.
- `POST /replay` requires redaction manifest.
- `POST /retention-profiles` requires elevated ACR.
- `POST /webhooks` requires BAA destination proof.
- `POST /backfill` requires historical PHI classifier version.
- `PATCH /tenant-audit-chain-config` requires retention profile version.
- `POST /pack/deactivate` returns retained evidence count.

## Workflow Deltas
- Event ingest workflow validates PHI-free payload and hash.
- Raw PHI detector quarantines unsafe events before seal.
- Seal workflow routes HIPAA leaves to certified cell.
- Tamper verification workflow runs on HIPAA cadence.
- Disclosure accounting workflow writes recipient and purpose event.
- Query workflow filters event fields by minimum necessary.
- Break-glass workflow grants one-hour emergency query.
- Export workflow builds redacted HIPAA evidence manifest.
- Breach workflow assembles sealed evidence bundle.
- Retention workflow blocks purge before six-year floor.
- Key attestation workflow verifies signing key before use.
- Replay workflow applies redaction manifest before re-emission.
- Replication workflow validates certified target cell.
- Webhook workflow validates BAA destination.
- Compaction workflow preserves PHI evidence lineage.
- Backfill workflow classifies historical events before promotion.
- Retention profile change workflow requires elevated approval.
- Pack activation workflow scans existing exports and webhooks.
- Pack deactivation waits for retained evidence inventory.
- Self-audit workflow seals every policy and retention change.

## SLO Deltas
- PHI event ingest validation p99 must stay <= 100 ms.
- Raw PHI quarantine p99 must complete <= 30 seconds.
- HIPAA seal batch p99 must complete <= 1 second after batch close.
- Tamper verification cadence target is <= 1 hour.
- Disclosure accounting write p99 must complete <= 1 second.
- Minimum necessary query authorization p99 must stay <= 100 ms.
- Break-glass workflow start p99 target is <= 2 minutes.
- Export manifest generation p99 target is <= 30 minutes.
- Breach evidence bundle creation p99 target is <= 10 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- Key attestation lookup p99 must stay <= 200 ms.
- Replay redaction throughput target is >= 10k events per minute.
- Replication route validation p99 must stay <= 100 ms.
- Backfill classifier validation p99 target is <= 15 minutes per batch.
- HIPAA audit-chain dashboard lag target is <= 5 minutes.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `AuditChainHipaaEventIngested` records event class and hash.
- `AuditChainHipaaRawPhiRejected` records detector verdict.
- `AuditChainHipaaSealBatchCreated` records batch id.
- `AuditChainHipaaSealBatchVerified` records verification id.
- `AuditChainHipaaDisclosureRecorded` records recipient hash.
- `AuditChainHipaaMinimumQueryApplied` records filter id.
- `AuditChainHipaaBreakGlassStarted` records reason id.
- `AuditChainHipaaExportManifestCreated` records manifest hash.
- `AuditChainHipaaBreachEvidenceBundled` records bundle hash.
- `AuditChainHipaaPurgeRefused` records retention floor.
- `AuditChainHipaaKeyAttested` records key ref.
- `AuditChainHipaaReplayRedacted` records replay id.
- `AuditChainHipaaReplicationBlocked` records target cell.
- `AuditChainHipaaWebhookRefused` records destination id.
- `AuditChainHipaaCompactionLineagePreserved` records segment id.
- `AuditChainHipaaBackfillClassified` records classifier version.
- `AuditChainHipaaRetentionProfileChanged` records profile version.
- `AuditChainHipaaPolicyChanged` records policy bundle.
- `AuditChainHipaaQuarantined` records quarantine id.
- `AuditChainHipaaPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- Raw PHI is submitted in event body; recovery is quarantine and reject seal.
- PHI payload hash missing; recovery is reject ingest.
- HIPAA-certified cell unavailable; recovery is buffer or reject PHI events.
- Seal batch misses PHI pack version; recovery is rebuild batch before publication.
- Tamper verification overdue; recovery is page audit-chain owner.
- Disclosure accounting missing purpose; recovery is reject disclosure event.
- Query lacks minimum necessary filter; recovery is deny query.
- Export manifest mismatch appears; recovery is revoke export and rebuild.
- Breach bundle misses event segment; recovery is rebuild from Merkle range.
- Retention purge requested early; recovery is refuse purge.
- Key attestation expires; recovery is pause HIPAA seal signing and rotate key.
- Replay bypasses redaction; recovery is halt replay.
- Replication target uncertified; recovery is block plan.
- Webhook lacks BAA; recovery is disable sink.
- Compaction would remove lineage; recovery is block compaction.
- Backfill classifier missing; recovery is block backfill.
- Support break-glass lacks reason; recovery is deny access.
- Pack deactivation requested with retained evidence; recovery is defer.
- Audit-chain backpressure appears; recovery is fail-closed for PHI event writes.
- Payload hash collision suspicion appears; recovery is rehash with stronger digest and preserve both proofs.

## Cross-µservice coordination
- `tenancy` provides HIPAA cell placement and active pack roster.
- `identity` provides privacy officer and covered workforce roles.
- `compliance` provides BAA proof, disclosure accounting, and breach workflows.
- `observability` emits PHI-free seal and verification metrics.
- `policy-engine` loads all `HIPAA-audit-chain-*` fragments.
- `workflow-engine` runs breach, export, break-glass, and retention workflows.
- `mail` emits HIPAA mail event classes into audit-chain.
- `drive` emits HIPAA file event classes into audit-chain.
- `calendar` emits HIPAA schedule event classes into audit-chain.
- `storage` provides HIPAA-certified immutable segment storage.
- `cloud-kms` or OpenBao provides signing key attestation.
- `incident-response` consumes HIPAA breach evidence bundles.
- `admin-console` renders HIPAA evidence status.
- `legal` defines export redaction profiles.
- `support` uses break-glass query path.
- `data-warehouse` receives only aggregate audit health metrics.
- `notification` routes tamper and deadline alerts.
- `release-engine` gates event schema changes.
- `dlp-virus-scan` screens backfill payloads for raw PHI.
- `pack-registry` signs this HIPAA audit-chain overlay.
