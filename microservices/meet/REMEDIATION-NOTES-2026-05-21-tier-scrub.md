# Wave 15J-batch-4 tier scrub remediation notes: meet

## Scope

- Service: `meet`
- Doctrine: ADR-0329, ADR-0330, ADR-0331
- Deleted `capability-tiers/` directory: Y

## Files modified with line counts

- `microservices/meet/README.md` - 25 lines
- `microservices/meet/manifest.json` - 441 lines
- `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md` - 111 lines
- `microservices/meet/tutorials/host-100-person-webinar-with-recording-transcription-translation.md` - 228 lines
- `microservices/meet/migration-playbooks/from-zoom-and-google-meet.md` - 200 lines
- `microservices/meet/faqs/realtime-engineer-faq.md` - 152 lines
- `microservices/meet/reference-implementations/join-room-and-stream-rust-sdk.md` - 296 lines
- `microservices/meet/coherence-audit-2026-05-20.md` - 608 lines
- `microservices/meet/IP-009-transcription-pipeline.md` - 107 lines
- `microservices/meet/dashboards/ai-features-quality.json` - 112 lines
- `microservices/meet/capabilities/T0-suggest.yaml` - 117 lines
- `microservices/meet/capabilities/T1-assist.yaml` - 165 lines
- `microservices/meet/capabilities/T2-auto.yaml` - 114 lines
- `microservices/meet/slos/transcription-correctness-bound.openslo.yaml` - 43 lines
- `microservices/meet/failure-modes.md` - 249 lines
- `microservices/meet/runbooks/transcription-classifier-rollback.md` - 109 lines
- `microservices/meet/decisions/ADR-MEET-0006-ai-feature-bounds.md` - 187 lines

## Replacement count

Rough vocabulary replacements: ~75 lines across the active and untracked meet service tree, plus the directory deletion.

## Design decisions

- Replaced commercial meeting-size and media-quality ladder language with paid `tenant_class`, `billing_components`, and cell-topology profiles.
- Reclassified sovereign/FIPS/pack-bound meeting behavior as compliance-pack gating.
- Kept canonical realtime quality targets uniform across tenant classes.
- Replaced broad `golden` wording caught by the verification regex with baseline/reference wording for quality tests and dashboards.
- Added README coverage for ADR-0330 because the service did not have a tracked README in the current tree.

## Outstanding follow-ups

None for the assigned zero-residue vocabulary gate.

## Wave 15-IP-substance scrub (2026-05-21)

Scope: `microservices/meet` only.

Inventory: 36 meet IP files reviewed: 15 foundation IPs and 21 journey IPs.

Stamped/thin detected: 21 journey IP files contained generated rows, generic numbered implementation tasks, generic deliverables, or oversized repeated event/build matrices.

Rewritten: 21 journey IP files were rewritten in place to cite existing meet contract, policy, catalog, SLO, and counterpart boundary files. Foundation IPs were preserved because they already cite concrete meet paths and service-specific contracts/policies.

Deleted: 0 files. Stamped generated sections were removed from rewritten journey IPs; no paths outside `microservices/meet` were modified.

Preserved: `IP-001` through `IP-015`, including the already-substantive recording/transcription/root policy plans. Existing non-IP meet edits in the worktree were not reverted.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/meet/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/meet/IP-002-cargo-workspace-bootstrap.md`
- `microservices/meet/IP-006-participant-and-lobby.md`
- `microservices/meet/IP-journey-j100-pack-rollout-first-action.md`
- `microservices/meet/IP-journey-j132-interview-rooms.md`
- `microservices/meet/IP-journey-j142-layoff-room-and-hr-witness-badge.md`
- `microservices/meet/IP-journey-j145-cross-tenant-interview-room.md`
- `microservices/meet/IP-journey-j28-family-call-adaptation.md`
- `microservices/meet/IP-journey-j39-quarterly-review-room.md`
- `microservices/meet/IP-journey-j44-telemedicine-room.md`
- `microservices/meet/IP-journey-j56-interview-room.md`
- `microservices/meet/IP-journey-j57-orientation-session.md`
- `microservices/meet/IP-journey-j58-review-recording.md`
- `microservices/meet/IP-journey-j61-telehealth-consult.md`
- `microservices/meet/IP-journey-j72-live-translation.md`
- `microservices/meet/IP-journey-j91-us-msb-mtl-overlay.md`
- `microservices/meet/IP-journey-j92-br-lgpd-us-parent-dsar.md`
- `microservices/meet/IP-journey-j93-in-dpdpa-rbi-overlay.md`
- `microservices/meet/IP-journey-j94-sox404-public-company-controls.md`
- `microservices/meet/IP-journey-j95-iso27001-soc2-annual-audit.md`
- `microservices/meet/IP-journey-j96-ksa-uae-mena-onboarding.md`
- `microservices/meet/IP-journey-j97-sg-pdpa-mas-tenant.md`
- `microservices/meet/IP-journey-j98-au-privacy-apra-cps234.md`
- `microservices/meet/IP-journey-j99-multi-pack-conflict-resolution.md`
- `microservices/meet/PRD.md`
- `microservices/meet/capacity-model.md`
- `microservices/meet/catalog/oya-meet-participant-adapter-valkey.yaml`
- `microservices/meet/catalog/oya-meet-participant-rest.yaml`
- `microservices/meet/coherence-audit-2026-05-20.md`
- `microservices/meet/iac/helm/meet/templates/networkpolicy.yaml`
- `microservices/meet/iac/helm/meet/values.yaml`
- `microservices/meet/manifest.json`
- `microservices/meet/policy/data-residency.md`
- `microservices/meet/threat-model.md`

Counterpart-fact preservations:

None.

Files renamed (git mv):

- `microservices/meet/catalog/oya-meet-participant-adapter-redis.yaml` -> `microservices/meet/catalog/oya-meet-participant-adapter-valkey.yaml`

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 3600s RTO / 300s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `valkey_cluster`, `audit_chain_merkle_seal`). ADR: ADR-0343. Alternative considered: keeping the older 900s PRD target; rejected because D-2 manifest values are the current contract for this propagation. Cost: warm SFU/TURN capacity and replicated room metadata even when average utilization is low.
- Capacity model: PRD now states manifest-aligned 0.28 vCPU / 768Mi / 4Gi storage, 4 Valkey, 3 Postgres, 8 outbound HTTP connections, `per_user` scaling, Tier-3 cell placement, min 4 / max 100 room-control replicas, and min 6 / max 200 LiveKit adapter replicas. ADR: ADR-0340. Alternative considered: sizing by tenant account count or manifest-ignoring media pool values; rejected because participant concurrency and D-2 manifest values govern. Cost: higher reserved media and GPU headroom.
- Sustainability + cost attribution: PRD now requires `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` on audit rows; carbon routing applies only to async summary/transcription/archive jobs. ADR: ADR-0344. Alternative considered: carbon-aware routing for all paths; rejected for live media, HIPAA emergency, consent, and retention paths. Cost: FinOps dimensions increase audit payload and rollup cardinality.
- API versioning: PRD now uses YYYY-MM-DD public contract carrier triplet, SDK semver, N=3 / 180d support, tenant pinning, and ADR-0145 internal-mesh exemption. ADR: ADR-0342. Alternative considered: SDK semver only; rejected because meeting integrations need date-pinned public contract evidence. Cost: three public API versions must remain tested concurrently.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because meet does not directly write that path.
