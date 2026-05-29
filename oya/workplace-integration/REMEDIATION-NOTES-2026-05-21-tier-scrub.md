# Wave 15J-batch-4 Tier Scrub — workplace-integration

## Scope

- Bucket: BUCKET-10.
- Service: `workplace-integration`.
- Doctrine: ADR-0329, ADR-0330, ADR-0331.

## Files Modified

- `README.md` — 65 lines.
- `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md` — 112 lines.
- `coherence-audit-2026-05-20.md` — 651 lines.
- `faqs/hris-engineer-faq.md` — 184 lines.
- `feature-parity-matrix-2026-05-20.md` — 421 lines.
- `migration-playbooks/from-rippling-and-gusto.md` — 177 lines.
- `onboarding/hris-engineer-first-week.md` — 127 lines.
- `performance-benchmark-numbers-2026-05-20.md` — 311 lines.
- `tutorials/hire-onboard-clock-in-payroll-cycle.md` — 193 lines.

## Retirement Actions

- `capability-tiers/` deleted: Y.
- Vocabulary replacement count: roughly 70 service-local replacements.
- README updated with ADR-0330 tenant_class and billing_components guidance.

## Design Decisions

- Payroll breadth, e-sign ceremony choice, work-permit verification, and regulated workflow activation were reworded as tenant_class caps, billing_component contract terms, compliance_pack activation, or jurisdiction policy.
- The old capability tier matrix directory was removed instead of recreated under a synonym.

## Follow-ups

- None for the Wave 15J hard vocabulary gate.

## Wave 15-ADR-0105-RED-remediation (2026-05-21)

- Old array contents removed from `src/lib.rs`: `api`, `rest`, `application`, `usecase`, `domain`, `kernel`, `adapter`, `worker`, `sdk`, `iac`, `policy`, `observability`.
- New canonical declaration: `domain::LAYERS` uses `Layer::Kernel`, `Layer::Domain`, `Layer::Usecase`, `Layer::App`, `Layer::Adapter`, `Layer::Infrastructure`, `Layer::Rest`, `Layer::Grpc`, `Layer::Graphql`, `Layer::Worker`, `Layer::Cli`, `Layer::Sdk`, `Layer::Api`.
- Files modified (97): `src/lib.rs`; `REMEDIATION-NOTES-2026-05-21-tier-scrub.md`; plus service-local legacy enum references grouped as root files (18), `ip/` (25), `catalog/` (13), `iac/` (12), `capabilities/` (6), `policies/` (6), `slos/` (6), `dashboards/` (5), `contracts/` (3), and `scorecards/` (1).
- Cargo check status: PASS — `cargo check -p 'oya-workplace-integration-*'` from `microservices/workplace-integration`.
- Test status: PASS — `cargo test -p 'oya-workplace-integration-*'` from `microservices/workplace-integration`.
- Legacy signature grep status: PASS — no obsolete enum constant, legacy enum label, or old 12-value comma sequence remains under `microservices/workplace-integration`.
- Catalog layer-field status: PASS — service catalog `layer:` values now use canonical ADR-0105/0106 values only.
