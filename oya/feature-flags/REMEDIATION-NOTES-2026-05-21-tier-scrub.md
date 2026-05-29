# Feature-flags tier-vocabulary remediation notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-06 scrubbed `microservices/feature-flags` for the retired Bronze/Silver/Gold/Platinum capability-tier vocabulary and adopted ADR-0330 `tenant_class` language.

## Files modified (line counts)

- `ARCHITECTURE.md` (1212)
- `README.md` (202)
- `benchmarks/openfeature-server-providers.md` (88)
- `coherence-audit-2026-05-20.md` (614)
- `faqs/engineer-faq.md` (48)
- `feature-parity-matrix-2026-05-20.md` (436)
- `manifest.json` (248)
- `migration-playbooks/from-launchdarkly.md` (114)
- `onboarding/engineer-first-week.md` (63)
- `performance-benchmark-numbers-2026-05-20.md` (423)
- `tutorials/cohort-rollout-with-analytics.md` (103)

## Deletions

- `capability-tiers/` deleted: Y

## Replacement count

Rough direct vocabulary replacements: ~70, including color-tier names, capability-tier fields, tier-gated prose, and performance references that pointed at retired tier matrices.

## Design decisions

- Replaced the manifest capability-tier array with `tenant_class_eligibility: ["demo_trial", "paid"]` and `paid_billing_components_emitted: ["per_usage"]` because flag evaluation volume can feed usage billing.
- Reframed second-approver and audit-required flag behavior as `compliance_pack` and paid-tenant policy requirements.
- Replaced tier-matrix benchmark citations with universal target language plus tenant_class and billing_components context.
- Updated README service-role rows so feature-flags is described as substrate and control-plane cell topology, not as a product tier.

## Outstanding follow-ups

None for BUCKET-06 scope.

## Wave 15-IP-substance scrub (2026-05-21)

Scope was limited to `microservices/feature-flags`.

Inventory:
- IP files inventoried: 37 (`IP-001` through `IP-027`, plus 10 `IP-journey-*` files).
- Stamped/thin IP shells detected: 10 journey IPs with exact 400-line generated bodies, repeated acceptance rows, repeated event/task/failure rows, generic "journey schemas directory" contract text, and broad counterpart invariant rows.
- Rewritten in place: 10 journey IPs.
- Deleted: 0 files.
- Preserved as already substantive: 27 non-journey IPs (`IP-001` through `IP-027`), because they already contain service-specific crate, policy, contract, SLO, SDK, or IaC acceptance details.

Rewritten files:
- `IP-journey-j91-us-msb-mtl-overlay.md`
- `IP-journey-j92-br-lgpd-us-parent-dsar.md`
- `IP-journey-j93-in-dpdpa-rbi-overlay.md`
- `IP-journey-j94-sox404-public-company-controls.md`
- `IP-journey-j95-iso27001-soc2-annual-audit.md`
- `IP-journey-j96-ksa-uae-mena-onboarding.md`
- `IP-journey-j97-sg-pdpa-mas-tenant.md`
- `IP-journey-j98-au-privacy-apra-cps234.md`
- `IP-journey-j99-multi-pack-conflict-resolution.md`
- `IP-journey-j100-pack-rollout-first-action.md`

Substance rules applied:
- Each rewritten journey IP cites real feature-flags contracts, policies, capabilities, runbooks, SLOs, dashboards, IaC files, or service docs that exist in this checkout.
- Journey counterpart refs were kept only where the counterpart files exist. The j92 file records that no repo-local journey counterpart exists and keeps only feature-flags service-local anchors.
- Generic generated row floods were removed instead of mechanically preserving unverifiable event/task/failure inventories.
