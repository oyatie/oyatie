# REMEDIATION-NOTES-2026-05-21-tier-scrub

Service: plugin-app-store

Files modified with current line counts:
- `README.md` — 5 lines
- `manifest.json` — 339 lines
- `PRD.md` — 205 lines
- `capabilities/plugin-install.yaml` — 67 lines
- `capabilities/plugin-revoke.yaml` — 62 lines
- `capabilities/plugin-vetting-decide.yaml` — 64 lines
- `contracts/openapi/plugin-app-store.yaml` — 366 lines
- `contracts/proto/plugin-app-store.proto` — 200 lines
- `policy/tenant-scope.cedar` — 65 lines
- `cost-budget.md` — 40 lines
- `compliance.md` — 971 lines
- `decisions/ADR-PAS-0004-vetting-trust-verdict-determined.md` — renamed from the retired badge-ladder ADR path.

capability-tiers/ dir deleted: Y

Vocabulary replacement count: ~110 direct and derived replacements.

Design decisions:
- Replaced plugin `vetting_badge` ladder with `trust_verdict` in OpenAPI, proto, and Cedar policy.
- Replaced subscription-tier policy action with `change_billing_components`.
- Gated paid marketplace mutations on `principal.tenant_class == "paid"` plus `billing_components contains "per_seat"`.
- Preserved publisher trust as a security/compliance signal rather than a pricing ladder.

Outstanding follow-ups: none for assigned forbidden vocabulary.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 21.
Trigger A matches: 8.
Trigger B matches: 13.
Trigger C matches: 16.
Trigger D matches: 21.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- `microservices/plugin-app-store/IP-journey-j100-pack-rollout-first-action.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j115-api-capability-entitlement.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j116-publish-install-catalog.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j119-marketplace-auction-surface.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j148-marketplace-return-flow.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j150-creator-brand-marketplace.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j40-vendor-subscription.md`: Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j49-marketplace-case-context.md`: Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j73-catalog-publication.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j74-install-flow.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j75-quarantine.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j90-marketplace-app-surface.md`: Trigger A -> API Versioning; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j91-us-msb-mtl-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j92-br-lgpd-us-parent-dsar.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j93-in-dpdpa-rbi-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j94-sox404-public-company-controls.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j95-iso27001-soc2-annual-audit.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j96-ksa-uae-mena-onboarding.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j97-sg-pdpa-mas-tenant.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j98-au-privacy-apra-cps234.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/plugin-app-store/IP-journey-j99-multi-pack-conflict-resolution.md`: Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.

Unmatched IPs:
- none.

Follow-ups:
- Reconcile `manifest.json#dr` numeric service targets when the D-2 manifest DR fields land for this service.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/plugin-app-store/PRD.md`. Related ADRs added: ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 900s, RPO p99 60s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, EU-AI high-risk floor exceeded. Alternative rejected: treating catalog replicas as the only DR or keeping a looser 1800s/300s PRD target. Cost: write idempotency and queue recovery during promotion.
- Capacity model (ADR-0340): values: manifest 0.25 vCPU, 512 MiB RAM, 2 GB storage, valkey=4, postgres=3, outbound_http=8, `per_capability` scaling, Tier-2 placement plus pod_runtime_tier=0 sandbox runtime, catalog/install 3-100 and vetting 2-50. Alternative rejected: catalog-only marketplace sizing. Cost: runtime engine pools and per-installation concurrency controls.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on catalog, install, revoke, vetting, runtime, and billing rows. Alternative rejected: plugin cost only through subscription invoices. Cost: third-party plugin activity must carry tenant and plugin dimensions.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for manifests/install/grants, ADR-0145 internal mesh exemption. Alternative rejected: plugin manifest schema as the only version. Cost: manifest and install-flow compatibility tests.
