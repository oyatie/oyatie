# Wave 15J-batch-4 tier scrub remediation notes: recordings

## Scope

- Service: `recordings`
- Doctrine: ADR-0329, ADR-0330, ADR-0331
- Deleted `capability-tiers/` directory: Y

## Files modified with line counts

- `microservices/recordings/README.md` - 25 lines
- `microservices/recordings/manifest.json` - 409 lines
- `microservices/recordings/benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md` - 105 lines
- `microservices/recordings/tutorials/legal-hold-engage-and-ediscovery-export.md` - 267 lines
- `microservices/recordings/faqs/compliance-officer-faq.md` - 106 lines
- `microservices/recordings/reference-implementations/ingest-and-search-rust-sdk.md` - 224 lines
- `microservices/recordings/coherence-audit-2026-05-20.md` - 622 lines
- `microservices/recordings/ARCHITECTURE.md` - 877 lines
- `microservices/recordings/backfill-replay.md` - 75 lines
- `microservices/recordings/sdk-plan.md` - 81 lines
- `microservices/recordings/migration-from-connect.md` - 405 lines
- `microservices/recordings/decisions/ADR-RECORDINGS-0001-transcription-and-diarization-pipeline.md` - 209 lines
- `microservices/recordings/capabilities/T0-suggest.yaml` - 75 lines
- `microservices/recordings/capabilities/T1-assist.yaml` - 118 lines
- `microservices/recordings/capabilities/T2-auto.yaml` - 116 lines

## Replacement count

Rough vocabulary replacements: ~160 lines across the active and untracked recordings service tree, plus the directory deletion.

## Design decisions

- Replaced recording throughput, ASR, eDiscovery, retention, and export ladder language with `tenant_class`, `billing_components`, compliance packs, and cell topology.
- Preserved storage hot/cold operational wording where it described data lifecycle rather than customer capability ladders.
- Reclassified sovereign/pack-bound ASR and retention behavior as compliance-pack gating.
- Replaced broad `golden` wording caught by the verification regex with baseline/reference wording for evaluation sets and query checks.
- Added README coverage for ADR-0330 because the service did not have a tracked README in the current tree.

## Outstanding follow-ups

None for the assigned zero-residue vocabulary gate.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/recordings/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/recordings/IP-001-iac-bootstrap.md`
- `microservices/recordings/IP-002-cargo-workspace-bootstrap.md`
- `microservices/recordings/IP-011-playback-share-link-watermark-bcs.md`
- `microservices/recordings/PRD.md`
- `microservices/recordings/capacity-model.md`
- `microservices/recordings/catalog/oya-recordings-share-link-adapter-valkey.yaml`
- `microservices/recordings/cost-budget.md`
- `microservices/recordings/iac/helm/recordings/Chart.yaml`
- `microservices/recordings/iac/helm/recordings/values.yaml`
- `microservices/recordings/manifest.json`
- `microservices/recordings/policy/data-residency.md`
- `microservices/recordings/threat-model.md`

Counterpart-fact preservations:

None.

Files renamed (git mv):

- `microservices/recordings/catalog/oya-recordings-share-link-adapter-redis.yaml` -> `microservices/recordings/catalog/oya-recordings-share-link-adapter-valkey.yaml`

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 3600s RTO / 300s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `seaweedfs_replicated`, `audit_chain_merkle_seal`). ADR: ADR-0343. Alternative considered: treating media and metadata as separate older PRD targets; rejected because D-2 manifest values are the current contract for this propagation. Cost: separate DR lanes for search/playback and immutable media still need drills.
- Capacity model: PRD now states manifest-aligned 0.3 vCPU / 768Mi / 50Gi storage, 2 Valkey, 3 Postgres, 6 outbound HTTP connections, `per_capability` scaling, Tier-3 placement, and queue-based worker scaling. ADR: ADR-0340. Alternative considered: single per-tenant archive quota or larger worker reservations in PRD; rejected because D-2 manifest values govern the PRD propagation. Cost: more HPA signals and queue metrics to maintain.
- Sustainability + cost attribution: PRD now requires per-row cost/emission/watt/provider/region fields and carbon routing for batch transcription, diarization, chaptering, transcode, and backfill only. ADR: ADR-0344. Alternative considered: greener route for eDiscovery and legal-hold export; rejected because court-order and retention deadlines are not optional. Cost: GPU/storage/CDN cost rollups must stay tenant- and pack-dimensional for long retention windows.
- API versioning: PRD now uses YYYY-MM-DD public contract carrier triplet, SDK semver, N=3 / 180d support, tenant pinning for court-validated exports, and ADR-0145 internal-mesh exemption. ADR: ADR-0342. Alternative considered: producer-service private versioning only; rejected because manual ingest, eDiscovery, and playback are public-facing. Cost: ingest/export schemas require long compatibility tests.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because recordings stores media/transcript archives rather than directly writing that path.
