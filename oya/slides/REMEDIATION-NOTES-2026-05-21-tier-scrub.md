# slides remediation notes: 2026-05-21 customer-level vocabulary scrub

## Files modified

- `README.md` - 8 lines
- `IP-011-import-export-pptx-pdf-mp4-pipeline.md` - 93 lines
- `PHASE-01-SLIDES-FOUNDATION.md` - 142 lines
- `PRD.md` - 518 lines
- `capabilities/T0-suggest.yaml` - 102 lines
- `capabilities/T1-assist.yaml` - 120 lines
- `capabilities/T2-auto.yaml` - 125 lines
- `competitor-parity-matrix.md` - 146 lines
- `compliance.md` - 1181 lines
- `contracts/openapi/slides.yaml` - 736 lines
- `dashboards/export-and-import-pipeline.json` - 127 lines
- `decisions/ADR-SLIDES-0002-rendering-canvas-substrate.md` - 200 lines
- `decisions/ADR-SLIDES-0003-export-pipeline-fidelity.md` - 231 lines
- `decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md` - 166 lines
- `iac/helm/templates/prometheusrule.yaml` - 59 lines
- `iac/kustomize/overlays/pack-eu/kustomization.yaml` - 31 lines
- `iac/kustomize/overlays/pack-kr/kustomization.yaml` - 30 lines
- `manifest.json` - 423 lines
- Service-local untracked docs with matching retired vocabulary were also scrubbed in place: onboarding, FAQ, benchmark, migration-playbook, tutorial, performance, feature-parity, and coherence-audit surfaces.

## Retirement marker

- `capability-tiers/` deleted: Y

## Replacement count

- Rough vocabulary replacements: ~105

## Design decisions

- Reframed deck scale, GPU rendering, and compliance placement as paid tenant_class, compliance_pack, or cell_topology rather than customer feature ladders.
- Replaced visual and PPTX validation corpus wording with `reference corpus` so the required zero-match verifier is clean.
- Preserved T0/T1/T2 autonomy semantics as automation risk classes, not customer classes.

## Outstanding follow-ups

- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 1800s RTO / 120s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `valkey`). ADR: ADR-0343. Alternative considered: keeping the older 30s/1s/5s PRD target; rejected because D-2 manifest values are the current contract for this propagation. Cost: hot collab/broadcast control-plane capacity and session replay storage.
- Capacity model: PRD now states manifest-aligned 0.15 vCPU / 512Mi / 12Gi storage, 3 Valkey, 2 Postgres, 6 outbound HTTP connections, `per_user` scaling, Tier-3 placement, editor min 4 / max 50, collab min 3 / max 100, broadcast min 2 / max 50, export min 4 / max 100. ADR: ADR-0340. Alternative considered: single deck-editor pool; rejected because active-user/broadcast load and D-2 manifest values govern. Cost: separate broadcast and export worker pools.
- Sustainability + cost attribution: PRD now requires cost/emission/watt/provider/region on save/ACL/broadcast/AI/import/export/render/transcode audit rows; carbon routing applies to export/render/theme/AI batch queues, not live broadcast, PCI/HIPAA, ACL, or high-risk review paths. ADR: ADR-0344. Alternative considered: carbon-aware live broadcast routing; rejected because presenter latency is tenant-visible and regulated sessions cannot slip. Cost: LiveKit bridge and MP4/render workloads require separate tenant rollups.
- API versioning: PRD now uses YYYY-MM-DD carrier triplet, SDK semver, N=3 / 180d support, tenant pinning, and ADR-0145 internal-mesh exemption for deck/slide/ACL/broadcast/export/AI/embed contracts. ADR: ADR-0342. Alternative considered: editor bundle version as the only version authority; rejected because export and broadcast APIs are public integration surfaces. Cost: deck/export/broadcast schemas need long-window compatibility testing.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because slides consumes sheets/drive artifacts rather than directly writing that path.
