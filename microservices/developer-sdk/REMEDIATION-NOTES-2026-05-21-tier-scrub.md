# REMEDIATION-NOTES-2026-05-21-tier-scrub

Service: developer-sdk

Files modified with current line counts:
- `README.md` — 5 lines
- `manifest.json` — 325 lines
- `capabilities/developer-onboard.yaml` — 65 lines
- `capabilities/developer-payout-settle.yaml` — 65 lines
- `capabilities/developer-sandbox-reset.yaml` — 58 lines
- `compliance.md` — 909 lines
- `cost-budget.md` — 40 lines
- `decisions/ADR-SDK-0001-ed25519-signing-keys-via-openbao-transit-engine-only;-privat.md` — 263 lines
- `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md` — 255 lines
- `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md` — modified; path contains non-ASCII and was included in scrub
- `decisions/ADR-SDK-0004-payout-substrate-uses-iso-20022-pain.001-for-sepa-and-nacha-.md` — 253 lines
- `decisions/ADR-SDK-0005-tax-form-emission-triggered-at-year-end-regenerated-on-deman.md` — 266 lines
- `decisions/ADR-SDK-0006-kyc-pipeline-in-house;-no-external-kyc-saas-(onfido-persona-.md` — 254 lines
- `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md` — 250 lines

capability-tiers/ dir deleted: Y

Vocabulary replacement count: ~70 direct and derived replacements, including Bronze/Silver/Gold/Platinum residues, capability_tiers manifest metadata, golden fixture wording, and support-window prose.

Design decisions:
- Replaced service manifest capability ladder metadata with `tenant_class_eligibility` and `paid_billing_components_emitted`.
- Converted SDK support and language-availability prose to tenant_class, billing_components, and launch-gate language instead of customer ladder labels.
- Preserved legitimate risk-class semantics in KYC language where they are not customer capability tiers.

Outstanding follow-ups: none for assigned forbidden vocabulary.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 11.
Trigger A matches: 0.
Trigger B matches: 4.
Trigger C matches: 10.
Trigger D matches: 1.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- `microservices/developer-sdk/IP-journey-j100-pack-rollout-first-action.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j41-sandbox-deploy.md`: Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/developer-sdk/IP-journey-j91-us-msb-mtl-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j92-br-lgpd-us-parent-dsar.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j93-in-dpdpa-rbi-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j94-sox404-public-company-controls.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j95-iso27001-soc2-annual-audit.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j96-ksa-uae-mena-onboarding.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j97-sg-pdpa-mas-tenant.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j98-au-privacy-apra-cps234.md`: Trigger C -> Sustainability emission.
- `microservices/developer-sdk/IP-journey-j99-multi-pack-conflict-resolution.md`: Trigger C -> Sustainability emission.

Unmatched IPs:
- none.

Follow-ups:
- Reconcile `manifest.json#dr` numeric service targets when the D-2 manifest DR fields land for this service.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/developer-sdk/PRD.md`. Related ADRs added: ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 900s, RPO p99 60s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, stricter than SOC2/ISO/KR-PIPA floors because keys and payouts are load-bearing. Alternative rejected: read-replica recovery for legal and bank state or keeping a looser 1800s/300s PRD target. Cost: payout and onboarding writes need active-active idempotency.
- Capacity model (ADR-0340): values: manifest 0.30 vCPU, 768 MiB RAM, 5 GB storage, valkey=2, postgres=3, outbound_http=10, `per_workflow_run` scaling, Tier-2 placement, pod_runtime_tier=2, portal 3-80 and sandbox 2-40. Alternative rejected: batching all developer operations behind codegen. Cost: separate queues for keys, sandbox, payout, and tax lanes.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on onboarding, KYC, signing, codegen, sandbox, payout, and tax-form rows. Alternative rejected: amortizing developer ecosystem costs only monthly. Cost: financial evidence rows must include energy and carbon fields.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for developer accounts/sandboxes/generated SDKs, ADR-0145 internal mesh exemption. Alternative rejected: package semver as public API governance. Cost: generated-client and contracts-registry compatibility lanes.
