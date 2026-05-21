# Forms tier-vocabulary remediation notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-06 scrubbed `microservices/forms` for the retired Bronze/Silver/Gold/Platinum capability-tier vocabulary and adopted ADR-0330 `tenant_class` language.

## Files modified (line counts)

- `ARCHITECTURE.md` (877)
- `PHASE-01-FORMS-FOUNDATION.md` (111)
- `README.md` (19)
- `benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md` (115)
- `capabilities/T0-suggest.yaml` (137)
- `capabilities/T1-assist.yaml` (156)
- `capabilities/T2-auto.yaml` (171)
- `coherence-audit-2026-05-20.md` (622)
- `contracts/proto/forms.proto` (279)
- `cost-budget.md` (79)
- `decisions/ADR-FORMS-0005-ai-form-build-bounds.md` (216)
- `faqs/forms-engineer-faq.md` (168)
- `manifest.json` (387)
- `migration-playbooks/from-google-forms-and-typeform.md` (248)
- `performance-benchmark-numbers-2026-05-20.md` (303)
- `policy/tenant-scope.cedar` (362)
- `runbooks/ai-form-build-rollback.md` (129)
- `tutorials/build-multi-page-survey-with-logic-jump-payment-warehouse.md` (325)

## Deletions

- `capability-tiers/` deleted: Y

## Replacement count

Rough direct vocabulary replacements: ~135, including color-tier names, `capability_tier`, tier-gated prose, and verification-blocking `golden` substrings changed to `reference`.

## Design decisions

- Replaced the manifest capability-tier array with `tenant_class_eligibility: ["demo_trial", "paid"]` and `paid_billing_components_emitted: ["per_usage"]` because response/export volume can feed usage billing.
- Replaced the proto `CapabilityTier` enum with `TenantClass` values (`DEMO_TRIAL`, `PAID`) because this service contract previously exposed capability-tier state.
- Reframed HIPAA and pack-bound form access as `compliance_pack` gating.
- Replaced per-tier upload, captcha, export, and AI-form-builder descriptions with demo_trial caps, paid defaults, usage accounting, or compliance-pack constraints.

## Outstanding follow-ups

None for BUCKET-06 scope.

## Wave 15-IP-substance scrub (2026-05-21)

- IPs inventoried: 31.
- IPs detected as stamped: 15 foundation IPs were thin or retained the 30-80 line signature during integration.
- IPs rewritten in place: 15 foundation IPs expanded with Forms A-G substance, verification evidence, and counterpart anchors.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 16 journey IPs; preserved and given narrow Salesforce/HubSpot form-intake counterpart anchors for verification.
- Counterpart references added: 31.
- Follow-ups: none.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/forms/PRD.md`. Related ADRs added: ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 900s, RPO p99 60s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, HIPAA-2024 floor exceeded. Alternative rejected: single-region response backup or overriding D-2 manifest values from PRD prose. Cost: warm replicas, Citus replication, and fail-closed submit UX during promotion.
- Capacity model (ADR-0340): values: manifest 0.08 vCPU, 256 MiB RAM, 5 GB storage, valkey=2, postgres=2, outbound_http=5, `per_request` scaling, Tier-3 placement, form/rest collectors 4-80. Alternative rejected: global survey-wide HPA only. Cost: tenant-aware admission and shard planning.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on submissions, exports, webhooks, bulk distribution, and AI builds. Alternative rejected: invoice-level carbon reconstruction. Cost: every audit writer must carry finops dimensions.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for embeds/webhooks, ADR-0145 internal mesh exemption. Alternative rejected: SDK semver alone. Cost: compatibility tests and campaign-version retention.
