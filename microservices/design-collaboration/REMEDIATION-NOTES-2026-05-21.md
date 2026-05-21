<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: design-collaboration
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 6
  prd_md_tier_references_scrubbed: 5
  architecture_md_tier_references_scrubbed: 14
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 15
  total_lines_changed: 232
  ADR_0316_citations_replaced_with_0329_0330_0331: 8
  cellular_tier_references_preserved: 33
  halt_cleanly: yes
-->

## Wave 15J-final-cleanup

- Bucket: F-BUCKET-3.
- Action: deleted stale 2026-05-20 coherence audit and feature parity artifacts; scrubbed remaining non-allowed observability precedent wording.
- Verification: tier-name grep and `capability_tier|max_tier|tier_threshold` grep both return 0 outside remediation notes.
- Follow-ups: none.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/design-collaboration/PRD.md`. Related ADRs added: ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 1800s, RPO p99 120s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, HIPAA-2024 floor exceeded. Alternative rejected: active-passive-only design-file authority after D-2 declared active-active. Cost: prototype links may pause while source authority reconciles.
- Capacity model (ADR-0340): values: manifest 0.18 vCPU, 512 MiB RAM, 20 GB storage, valkey=3, postgres=3, outbound_http=5, `per_user` scaling, Tier-3 placement, 2-40 collaboration pods and 2-30 render pods. Alternative rejected: one render queue for all creative tenants. Cost: per-capability admission and render queue isolation.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on design save, component publish, review comment, prototype render, brand-kit export, and migration replay rows. Alternative rejected: object-storage-only carbon accounting. Cost: creative compute must emit tenant/provider dimensions.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for design clients/prototype embeds/migration adapters, ADR-0145 internal mesh exemption. Alternative rejected: sharing one unpinned creative client protocol. Cost: client, prototype, and migration contract fixtures.
