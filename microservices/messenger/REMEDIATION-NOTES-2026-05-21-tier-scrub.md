# Messenger Tier Scrub Remediation Notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-03 scrub for `microservices/messenger`.

## Files Modified

- `README.md` — 25 lines
- `PRD.md` — 1740 lines
- `manifest.json` — 517 lines
- `coherence-audit-2026-05-20.md` — 627 lines
- `feature-parity-matrix-2026-05-20.md` — 460 lines
- `performance-benchmark-numbers-2026-05-20.md` — 401 lines
- `benchmarks/slack-teams-discord-vs-oyatie.md` — 127 lines
- `onboarding/messenger-engineer-first-week.md` — 303 lines
- `migration-playbooks/from-slack.md` — 209 lines
- `tutorials/configure-cross-tenant-cohort-channel.md` — 301 lines
- `faqs/messenger-engineer-faq.md` — 163 lines
- `reference-implementations/send-mls-message-rust-sdk.md` — 172 lines
- `test-plans/unit-test-strategy.md` — 245 lines
- `test-plans/integration-test-strategy.md` — 210 lines
- `test-plans/contract-test-strategy.md` — 374 lines
- `capabilities/T0-suggest.yaml` — 120 lines
- `capabilities/T1-assist.yaml` — 102 lines
- `capabilities/T2-auto.yaml` — 113 lines

## Deletion

`capability-tiers/` deleted: Y.

## Replacement Count

Rough replacement count: ~45 content replacements plus the directory deletion.

## Design Decisions

- Replaced the customer capability ladder with `tenant_class` language and
  preserved messenger's legitimate paid-only controls as compliance-pack or
  billing-component gates.
- Preserved MLS and huddle capability substance while removing Bronze/Silver/
  Gold/Platinum wording from onboarding, FAQs, tutorials, benchmarks, and
  reference examples.
- Replaced "golden" fixture and evaluation wording with reference fixture /
  reference-set wording.
- Added a top-level README because this service did not have one; the README
  now points at ADR-0330's `tenant_class` plus `billing_components` model.

## Outstanding Follow-Ups

None for the assigned Bronze/Silver/Gold/Platinum and `capability_tier` scrub.
