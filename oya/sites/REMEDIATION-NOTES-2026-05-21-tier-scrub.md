# Sites tier-vocabulary remediation notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-06 scrubbed `microservices/sites` for the retired Bronze/Silver/Gold/Platinum capability-tier vocabulary and adopted ADR-0330 `tenant_class` language.

## Files modified (line counts)

- `ARCHITECTURE.md` (880)
- `IP-003-page-bc-kernel.md` (49)
- `IP-013-contracts-and-capabilities.md` (87)
- `README.md` (19)
- `benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md` (111)
- `capabilities/T0-suggest.yaml` (143)
- `capabilities/T1-assist.yaml` (178)
- `capabilities/T2-auto.yaml` (113)
- `coherence-audit-2026-05-20.md` (1354)
- `feature-parity-matrix-2026-05-20.md` (574)
- `manifest.json` (437)
- `runbooks/page-export-corruption.md` (179)
- `runbooks/publish-pipeline-rollback.md` (182)
- `tutorials/launch-site-with-custom-domain-cms-and-accessibility.md` (307)

## Deletions

- `capability-tiers/` deleted: Y

## Replacement count

Rough direct vocabulary replacements: ~135, including color-tier names, tenant-tier-gated prose, capability-tier fields, and verification-blocking `golden` substrings changed to `reference`.

## Design decisions

- Replaced the manifest capability-tier array with `tenant_class_eligibility: ["demo_trial", "paid"]` and `paid_billing_components_emitted: ["per_usage"]` because publish, CDN, image, and search usage can feed usage billing.
- Reframed LLM-backed site actions as `tenant_class` and `billing_components` governed, with compliance-pack and cell-topology constraints where quality or jurisdiction matters.
- Collapsed benchmark rows into a single industry-leader target posture rather than service levels by color tier.
- Renamed service-local golden test references to reference test references so required verification returns zero matches.

## Outstanding follow-ups

None for BUCKET-06 scope.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/sites/PRD.md`. Related ADRs added: ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 1800s, RPO p99 300s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, HIPAA-2024 floor met. Alternative rejected: treating CDN stale cache as full DR or preserving the older 900s/60s PRD target against D-2. Cost: origin replicas, DNS drift drills, and publish pause semantics.
- Capacity model (ADR-0340): values: manifest 0.10 vCPU, 256 MiB RAM, 10 GB storage, valkey=2, postgres=2, outbound_http=6, `per_request` scaling, Tier-3 placement, with medium-tenant 1k page/100k visitor operating shape. Alternative rejected: only sizing by page count. Cost: separate cache-miss, editor, CMS, search, and publish admission budgets.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on render, CDN purge, image, search, publish, and AI build rows. Alternative rejected: using CDN invoice totals only. Cost: edge/provider attribution must survive cache-hit aggregation.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for editor/custom-domain/webhook automations, ADR-0145 internal mesh exemption. Alternative rejected: unversioned CMS and domain APIs. Cost: route and webhook compatibility matrix.
