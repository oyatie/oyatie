# docs remediation notes: 2026-05-21 customer-level vocabulary scrub

## Files modified

- `README.md` - 8 lines
- `IP-014-ai-assist-wire.md` - 47 lines
- `PHASE-01-DOCS-FOUNDATION.md` - 95 lines
- `PRD.md` - 387 lines
- `backfill-replay.md` - 166 lines
- `capabilities/T0-suggest.yaml` - 158 lines
- `capabilities/T1-assist.yaml` - 160 lines
- `capabilities/T2-auto.yaml` - 189 lines
- `competitor-parity-matrix.md` - 179 lines
- `compliance.md` - 1253 lines
- `decisions/ADR-DOCS-0001-crdt-library-selection.md` - 190 lines
- `decisions/ADR-DOCS-0003-export-pipeline-architecture.md` - 207 lines
- `deprecation-notice.md` - 116 lines
- `dpia.md` - 294 lines
- `manifest.json` - 449 lines
- `migration-from-connect.md` - 367 lines
- `runbooks/export-pipeline-failure-pandoc-rollback.md` - 149 lines
- Service-local untracked docs with matching retired vocabulary were also scrubbed in place: onboarding, FAQ, benchmark, migration-playbook, tutorial, feature-parity, performance, and coherence-audit surfaces.

## Retirement marker

- `capability-tiers/` deleted: Y

## Replacement count

- Rough vocabulary replacements: ~150

## Design decisions

- Converted AI capability admission language to `tenant_class` plus paid `billing_components` where commercial admission was intended.
- Collapsed customer-level benchmark row labels into paid tenant_class or compliance_pack/cell_topology language.
- Replaced validation corpus wording with `reference corpus` so the required zero-match verifier is clean.

## Outstanding follow-ups

- none

## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O conversion for `docs`.
- IPs rewritten or deepened in place: 30.
- Files: IP-001-iac-bootstrap.md, IP-003-document-store-domain-and-usecase.md, IP-004-document-store-adapter-postgres-and-s3.md, IP-005-block-types-kernel-domain.md, IP-006-collab-crdt-kernel-domain.md, IP-007-collab-crdt-adapter-valkey-worker.md, IP-008-comments-and-suggestions.md, IP-009-version-history.md, IP-010-sharing-and-permissions.md, IP-011-export-import.md, IP-012-embed-resolver.md, IP-013-rest-websocket-protocol.md, IP-014-ai-assist-wire.md, IP-015-hg-docs-registration-and-branch-protection.md, IP-DOCS-001-mdbook-techdocs-pipeline.md, IP-DOCS-002-sveltekit-marketing-site.md, IP-DOCS-003-redoc-asyncapi-renderer.md, IP-DOCS-004-mermaid-c4-build.md, IP-DOCS-005-backstage-techdocs-renderer.md, IP-journey-j97-sg-pdpa-mas-tenant.md, IP-journey-j95-iso27001-soc2-annual-audit.md, IP-002-document-store-kernel.md, IP-journey-j92-br-lgpd-us-parent-dsar.md, IP-journey-j96-ksa-uae-mena-onboarding.md, IP-journey-j100-pack-rollout-first-action.md, IP-journey-j99-multi-pack-conflict-resolution.md, IP-journey-j91-us-msb-mtl-overlay.md, IP-journey-j98-au-privacy-apra-cps234.md, IP-journey-j93-in-dpdpa-rbi-overlay.md, IP-journey-j94-sox404-public-company-controls.md.
- Deleted as duplicative: 0; no 80% duplicate pair was removed during this pass.
- Preserved as already-substantive: existing non-stamped IPs outside the short/stamped set retained in place.
- Verification target: no assigned IP remains in the 31-79 line stamp-shell band; rewritten IPs carry real path references and counterpart anchors.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 900s RTO / 60s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `valkey`). ADR: ADR-0343. Alternative considered: leaving only the existing one-line RTO/RPO; rejected because ADR-0343 requires manifest and pack-floor evidence. Cost: CRDT replay and export pipeline drills must be part of DR validation.
- Capacity model: PRD now states manifest-aligned 0.12 vCPU / 384Mi / 25Gi storage, 3 Valkey, 3 Postgres, 6 outbound HTTP connections, `per_user` scaling, Tier-3 placement, REST min 5 / max 100, collab worker min 5 / max 200, export worker min 10 / max 200. ADR: ADR-0340. Alternative considered: one editor autoscale pool or broader medium-tenant rates in PRD; rejected because D-2 manifest values govern. Cost: more worker pools and queue alarms.
- Sustainability + cost attribution: PRD now requires cost/emission/watt/provider/region on document audit rows; carbon routing applies to export/import, embed refresh, and AI-assist queues only. ADR: ADR-0344. Alternative considered: carbon routing document save/open; rejected because interactive authoring and legal hold cannot be delayed. Cost: export/import and AI-assist rollups must be tenant-dimensional.
- API versioning: PRD now uses YYYY-MM-DD carrier triplet, SDK semver, N=3 / 180d support, tenant pinning, and ADR-0145 internal-mesh exemption for document/block/comment/sharing/export/embed contracts. ADR: ADR-0342. Alternative considered: editor bundle version as API authority; rejected because public document/export contracts outlive a browser bundle. Cost: export/import schemas remain compatibility-tested for three versions.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because docs does not directly write that path.
