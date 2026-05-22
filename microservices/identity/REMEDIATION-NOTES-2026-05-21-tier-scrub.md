# Wave 15J-batch-4 Tier Scrub — identity

## Scope

- Bucket: BUCKET-10.
- Service: `identity`.
- Doctrine: ADR-0329, ADR-0330, ADR-0331.

## Files Modified

- `ARCHITECTURE.md` — 880 lines.
- `README.md` — 24 lines.
- `benchmarks/okta-auth0-entra-vs-oyatie.md` — 119 lines.
- `tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md` — 366 lines.
- `capacity-model.md` — 158 lines.
- `coherence-audit-2026-05-20.md` — 690 lines.
- `faqs/identity-engineer-faq.md` — 179 lines.
- `feature-parity-matrix-2026-05-20.md` — 416 lines.
- `iac/oci-guest/always-free/main.tf` — 182 lines.
- `iac/oyatie-public-cloud/main.tf` — 215 lines.
- `manifest.json` — 470 lines.
- `migration-playbooks/from-okta.md` — 192 lines.
- `onboarding/identity-engineer-first-week.md` — 332 lines.
- `performance-benchmark-numbers-2026-05-20.md` — 307 lines.
- `policy/tenant-class.cedar` — 206 lines.
- `reference-implementations/webauthn-passkey-flow-rust-sdk.md` — 327 lines.
- `scorecards/aws-well-architected.json` — 22 lines.
- `scorecards/google-sre-prr.json` — 34 lines.
- `supported-oses.json` — 207 lines.
- `test-plans/integration-test-strategy.md` — 190 lines.
- `tutorials/register-passkey-and-recovery-envelope.md` — 327 lines.

## Retirement Actions

- `capability-tiers/` deleted: Y.
- Vocabulary replacement count: roughly 250 service-local replacements.
- Former capability-tier delta artifact renamed to tenant-class adoption language.
- README created with ADR-0330 tenant_class and billing_components guidance.

## Design Decisions

- Principal and token examples now use `tenant_class` and paid billing_components instead of customer capability tiers.
- The service retains non-customer criticality and regulatory vocabulary where it is not the retired Bronze/Silver/Gold/Platinum model.
- Dashboard wording was adjusted from the substring-conflicting "golden signals" phrase to "core signals" so the hard vocabulary gate has zero false positives.

## Follow-ups

- None for the Wave 15J hard vocabulary gate.
