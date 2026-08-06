---
id: ADR-AUD-001
title: Per-Cell Hash Tree versus Multi-Region Merkle Strategy
status: Proposed
date: 2026-05-20
microservice: audit-chain
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-audit-chain
---

# ADR-AUD-001: Per-Cell Hash Tree versus Multi-Region Merkle Strategy

## Context

- Audit-chain is the cryptographic evidence substrate for every state-changing event emitted by other Oyatie microservices.
- The service PRD inherits Bominal audit-chain posture: Merkle tree plus Ed25519 signature, HSM-backed signing, and regulator-ready proof export.
- ADR-0003 makes evidence emission a platform primitive; this local ADR decides how evidence roots are partitioned across cells and regions.
- ADR-0009 makes tenant-per-region cell boundaries load-bearing for residency and failure isolation.
- ADR-0240 allows sovereign packs to restrict replication; audit roots must respect those restrictions even when evidence is cryptographic.
- The local manifest already separates emission, query, retention-cascade, sealing, and verification bounded contexts.
- Local SLOs include seal write latency, seal write availability, seal storage availability, Merkle verification latency, evidence export freshness, and chain-of-custody correctness.
- The audit-chain service must prove inclusion, ordering within a bucket, signature authority, retention handling, and verification outcome without trusting a single mutable database.
- Global multi-region Merkle trees make a single canonical root attractive, but they couple residency, latency, and partition recovery across cells.
- Per-cell trees reduce blast radius and latency but require a federation layer for portfolio-wide audits.
- Audit records include `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `data_class`, `audit_event_class`, and source microservice.
- High-volume microservices such as observability, payments, workflow-engine, and intelligence emit audit events at very different rates.
- Emergency-services, healthcare, financial, and whistleblower flows require seal evidence even during partial regional failure.
- Some packs require local HSM partitions and forbid root material from crossing a jurisdiction boundary.
- Some auditors need a tenant-scoped time-range proof, not a global Oyatie proof.
- Some internal investigations need fleet-wide root comparison to detect forks or storage corruption.
- The strategy must make chain forks visible without forcing every tenant event into one global consensus structure.
- The strategy must preserve append-only semantics while allowing retention cascade and DSR redaction markers.
- The strategy must support regulator exports where the verifier has no production access.
- The strategy must support recursive self-audit: sealing, verification, retention, and export actions are themselves auditable.
- The strategy must not require two-phase commit; ADR-0145 bans cross-service distributed transactions.
- The strategy must tolerate storage backlog, HSM partition latency, and delayed regional replication.
- The strategy must give each cell a clean rollback and recovery boundary.
- The strategy must keep proof sizes bounded for tenant exports.
- The strategy must let Oya VCS and promotion gates verify evidence fast enough for CI.
- The strategy must let observability correlate seal latency and verification failure metrics per cell.
- The strategy must allow future public transparency witnesses without exposing tenant event payloads.
- The strategy must avoid treating blockchain consensus as a default requirement; Oyatie needs tamper evidence, not cryptocurrency consensus.
- The strategy must document whether a cross-region root is authoritative or merely a summary of cell roots.
- The strategy must give future implementers exact data shapes and endpoints.

## Decision

- Adopt per-cell append-only hash trees as the authoritative audit-chain structure.
- Create one Merkle-compatible hash tree per `(tenant_id, home_cell, period_start, event_class_bucket)`.
- Use one-second periods by default for high-throughput tenants and five-second periods for low-throughput tenants.
- Hash each leaf as `H(domain_separator || canonical_event_bytes || prev_event_hash || event_metadata_hash)`.
- Use SHA-256 for RFC 6962 compatible inclusion proof shape unless a pack requires SHA-384.
- Use explicit domain separators for leaf, node, cell-root, region-root, retention-marker, and witness-anchor hashes.
- Sign each cell-period root with the cell's HSM-backed Ed25519 signing key.
- Store each signed cell-period root in WORM object storage and in the audit-chain root index.
- Publish a region summary root every minute that hashes the ordered list of signed cell-period roots available in that region.
- Treat region summary roots as detectability and export accelerators, not as the authoritative event ledger.
- Publish a daily fleet witness manifest that hashes region summary roots plus missing-root attestations.
- Treat the fleet witness manifest as fork detection, not as permission to replicate tenant evidence across forbidden regions.
- Keep inclusion proofs cell-local: `{event_leaf, sibling_path, cell_period_root, cell_signature, signer_cert_chain}`.
- Keep region proofs optional: `{cell_period_root, region_summary_path, region_summary_signature}`.
- Keep fleet proofs optional: `{region_summary_root, daily_witness_path, witness_signature}`.
- Use HLC timestamp and monotonic per-cell sequence number to order events inside a period bucket.
- Reject caller-supplied sequence numbers; the emission service assigns sequence ids.
- Accept out-of-order arrival into a bounded pending set for 10 seconds before sealing a late marker.
- Emit `EVT-AUD-LATE-EVENT-MARKER` when an event arrives after its period root is sealed.
- Emit `EVT-AUD-CELL-ROOT-SIGNED` for every signed cell root.
- Emit `EVT-AUD-REGION-SUMMARY-SIGNED` for every regional summary root.
- Emit `EVT-AUD-WITNESS-MANIFEST-SIGNED` for every daily witness manifest.
- Use retention cascade by appending redaction or tombstone markers, not by modifying historical leaves.
- Use per-pack signing keys under OpenBao/HSM paths scoped by tenant, pack, cell, and purpose.
- Rotate cell signing keys every 90 days with a 7-day overlap and explicit chain-of-trust transition record.
- Support emergency key rotation that signs a `key_compromise_boundary` record before the new key starts.
- Keep verification pure-read: proof verification never mutates audit state.
- Keep query APIs separate from verification APIs; a verifier can validate a proof without being allowed to query tenant history.
- Avoid a single global Merkle tree because it would couple sovereign cells, create cross-region write dependencies, and expand blast radius.

## Alternatives Considered

### One global multi-region Merkle tree

- Pros: one canonical root is simple to explain to auditors.
- Pros: global fork detection is inherent if every event is included in one tree.
- Pros: proof construction can be standardized across all tenants.
- Cons: cross-region write availability becomes part of every state-changing event path.
- Cons: sovereign packs may forbid the data needed to build the global tree.
- Cons: a delayed or partitioned region blocks or complicates root publication for unrelated tenants.
- Rejected because ADR-0009 cell boundaries and ADR-0240 sovereign packs are stronger constraints than root simplicity.

### Per-tenant global tree spanning all cells

- Pros: tenant export is easy because each tenant has one tree.
- Pros: lower cross-tenant blast radius than a fleet-wide tree.
- Pros: tenant auditors can verify all tenant events from one root stream.
- Cons: large tenants with multiple regulated regions still create cross-region coupling.
- Cons: tenant relocation and DR transitions complicate signer authority.
- Cons: local-cell outage can block tenant roots in healthy cells.
- Rejected because tenant scope alone is not enough; cell and residency scope are required.

### Append-only SQL ledger without Merkle proofs

- Pros: easiest query implementation.
- Pros: low write latency and familiar operational model.
- Pros: retention markers are straightforward.
- Cons: external auditors must trust database controls instead of verifying cryptographic inclusion.
- Cons: tamper evidence depends on backups and database logs rather than proof artifacts.
- Cons: fork detection is weak without independent roots.
- Rejected because audit-chain is the non-repudiation substrate, not only an audit table.

### Blockchain or external distributed ledger

- Pros: public anchoring and consensus narratives are familiar in some audit markets.
- Pros: external observers can detect tampering without trusting Oyatie infrastructure.
- Pros: append-only semantics are built into the ledger abstraction.
- Cons: consensus overhead is unnecessary for Oyatie's tenant-scoped evidence goals.
- Cons: payload privacy and residency constraints become harder to reason about.
- Cons: cost, throughput, and legal posture are worse than per-cell hash trees.
- Rejected because Oyatie needs verifiable tamper evidence and scoped witnesses, not external monetary consensus.

## Consequences

- Positive: event sealing remains local to the tenant home cell and can meet low-latency SLOs.
- Positive: sovereign packs can keep roots, keys, and event payloads inside approved cells.
- Positive: proof export stays compact for tenant-scoped audits.
- Positive: regional and fleet summaries still provide fork detection across cells.
- Positive: cell failure does not block unrelated cells from sealing their own roots.
- Positive: retention cascade can append markers without rewriting historical tree leaves.
- Positive: HSM key rotation has a cell-local blast radius.
- Negative: auditors may need to understand a two-level proof shape for region or fleet-wide questions.
- Negative: fleet-wide incident review needs summary manifests in addition to inclusion proofs.
- Negative: late events require explicit markers, increasing proof interpretation complexity.
- Negative: a tenant moving cells needs a transition proof from old cell signer to new cell signer.
- Negative: independent cell roots can temporarily diverge in publication cadence during partitions.
- Neutral: region summaries are useful for detection but are not the source of authority.
- Neutral: public witnesses can be added later by anchoring only summary root hashes.
- Neutral: per-cell root strategy works with Postgres, S3 WORM, and HSM adapters already in the manifest.
- Neutral: proof verification stays offline-capable because it needs roots, signatures, and paths, not database access.
- Follow-up: add a proof schema under `contracts/proto/audit-chain.proto`.
- Follow-up: add runbook section for cell signer compromise and root transition.
- Follow-up: add dashboard for missing cell roots and region summary lag.
- Follow-up: add auditor FAQ explaining authoritative cell roots versus summary roots.
- Follow-up: add property tests for late-event marker proofs.

## Implementation Notes

- Data shape `AuditLeaf`: `{event_id, tenant_id, home_cell, event_class, event_ts_hlc, sequence_no, canonical_event_hash, prev_event_hash}`.
- Data shape `CellPeriodTree`: `{tenant_id, home_cell, period_start, period_end, event_class_bucket, tree_alg, leaf_count, root_hash}`.
- Data shape `SignedCellRoot`: `{cell_root_id, cell_period_tree_ref, root_hash, signer_key_id, signature_alg, signature, signed_at}`.
- Data shape `RegionSummaryRoot`: `{region_id, summary_period_start, summary_period_end, included_cell_root_ids, missing_cell_roots, root_hash, signature}`.
- Data shape `FleetWitnessManifest`: `{witness_date, region_summary_roots, missing_region_summaries, manifest_hash, signature, publication_ref}`.
- Data shape `InclusionProof`: `{event_id, leaf_hash, sibling_path, root_hash, cell_signature, signer_cert_chain, proof_version}`.
- Data shape `RetentionMarker`: `{marker_id, event_id, marker_type, reason_code, actor_principal_id, appended_at, marker_leaf_hash}`.
- REST endpoint `POST /v1/audit/events` accepts canonical event envelopes and returns `{event_id, period_ref, receipt_hash}`.
- REST endpoint `GET /v1/audit/events/{event_id}/proof` returns a cell-local inclusion proof.
- REST endpoint `POST /v1/audit/proofs/verify` verifies a proof without tenant query privileges.
- REST endpoint `GET /v1/audit/roots/cell/{cell_root_id}` returns signed root metadata.
- REST endpoint `GET /v1/audit/roots/region/{region_id}/{period}` returns region summary metadata.
- REST endpoint `GET /v1/audit/witness/{date}` returns the daily fleet witness manifest.
- REST endpoint `POST /v1/audit/retention/markers` appends DSR, retention, legal-hold, or correction markers.
- Async event `audit_chain.cell_root.signed.v1` is emitted after every HSM signature.
- Async event `audit_chain.region_summary.signed.v1` is emitted after every regional summary.
- Async event `audit_chain.proof.verification_failed.v1` is emitted for malformed or invalid proof attempts.
- Cedar permit `audit_chain::event::emit` requires SPIFFE caller identity, tenant match, and event class authorization.
- Cedar permit `audit_chain::proof::read` requires tenant scope or auditor engagement scope.
- Cedar permit `audit_chain::proof::verify` allows public verification of a supplied proof with no query expansion.
- Cedar forbid `audit_chain::root::cross_region_replicate` blocks root payload replication when pack residency forbids it.
- Cedar permit `audit_chain::retention_marker::append` requires DSR, retention, or legal-hold authority.
- SLO target `seal_write_latency`: p99 below 50 ms for receipt, p99 below 1 second for signed root completion.
- SLO target `merkle_chain_verification_latency`: p95 below 200 ms for supplied proof verification.
- SLO target `chain_of_custody_integrity_correctness`: 100 percent valid proof verification for uncorrupted generated fixtures.
- SLO target `evidence_export_freshness`: 95 percent of requested evidence bundles generated within 300 seconds.
- Storage path `s3://audit-chain/<home_cell>/<tenant_id>/<period>/events.parquet` stores canonical event batches.
- Storage path `s3://audit-chain/<home_cell>/<tenant_id>/<period>/roots.jsonl` stores signed cell roots.
- HSM key path `hsm://<pack>/<home_cell>/audit-chain/root-signing/<key_epoch>` signs cell roots.
- OpenBao reference `secret/<tenant_id>/audit-chain/export/<cell_id>/<purpose>` stores export credentials only.
- Verification library must operate from a portable bundle containing roots, signatures, leaves, sibling paths, and signer certificate chain.
- Late event handling appends a new marker leaf in the next open period and links to the original event timestamp.
- Cell relocation emits `EVT-AUD-CELL-AUTHORITY-TRANSFER` signed by both old and new cell keys when possible.
- DR recovery replays event batches, recomputes roots, and compares signed root hashes before promoting a recovered cell.
- Backpressure behavior refuses high-risk event-producing operations when receipt assignment cannot be durably written.
- Backpressure behavior allows read-only verification because verification is pure and does not depend on new sealing.

## Verification

- Unit test `audit_leaf_hash_uses_domain_separator` prevents leaf/node hash confusion.
- Unit test `caller_sequence_number_ignored` proves emission assigns sequence ids.
- Unit test `retention_marker_does_not_mutate_original_leaf` protects append-only semantics.
- Unit test `proof_verify_requires_matching_cell_signature` rejects roots signed by the wrong key.
- Unit test `region_summary_not_authoritative_for_event_inclusion` documents proof semantics.
- Property test `cell_tree_inclusion_roundtrip` generates random event batches and verifies proofs.
- Property test `late_event_marker_links_to_original_timestamp` covers delayed arrivals.
- Property test `root_recompute_matches_signed_root` validates canonical event encoding.
- Fuzz test `malformed_sibling_path_rejected` covers proof parser safety.
- Integration test `emit_to_signed_root_under_one_second` validates receipt and seal latency.
- Integration test `tenant_export_bundle_verifies_offline` validates regulator bundle portability.
- Integration test `sovereign_pack_blocks_cross_region_root_payload` validates residency policy.
- Integration test `cell_relocation_dual_signed_transition` validates tenant move proof.
- Integration test `hsm_key_rotation_overlap_preserves_verification` validates 7-day overlap.
- Load test `fifty_thousand_events_per_second_per_cluster` validates PRD throughput target.
- Load test `proof_verify_p95_under_200ms` validates verifier SLO.
- Chaos test `hsm_partition_delay_emits_missing_root_summary` validates region summary gaps.
- Chaos test `object_store_corruption_detected_by_root_recompute` validates storage tamper detection.
- Chaos test `audit_backpressure_blocks_high_risk_mutation` validates fail-closed dependency behavior.
- Dashboard check `seal-latency.json` shows p50, p99, p999 by cell and event class.
- Dashboard check `verification-failure-rate.json` shows invalid proof reason codes.
- Dashboard check `emission-rate.json` shows event volume by source service and retention class.
- Static check every mutating endpoint declares an audit event class.
- Static check every signer key id includes pack, cell, and epoch.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- RFC 6962, Certificate Transparency: https://www.rfc-editor.org/rfc/rfc6962.html
- RFC 9162, Certificate Transparency Version 2.0: https://www.rfc-editor.org/rfc/rfc9162.html
- NIST IR 8202, Blockchain Technology Overview.
- NIST Dictionary of Algorithms and Data Structures, Merkle tree definition.
- Ralph C. Merkle, "A Digital Signature Based on a Conventional Encryption Function."
- Crosby and Wallach, "Efficient Data Structures for Tamper-Evident Logging."
- Certificate Transparency architecture and Merkle audit proof model.
- Cedar Policy Language authorization and schema documentation: https://docs.cedarpolicy.com/
- ADR-0003, ADR-0009, ADR-0145, ADR-0240, ADR-0243, ADR-0244, and ADR-0263.
- Local audit-chain PRD, manifest, sealing runbooks, and SLO manifests.
