# Community Tier Scrub Remediation Notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-03 scrub for `microservices/community`.

## Files Modified

- `README.md` — 25 lines
- `PRD.md` — 1525 lines
- `manifest.json` — 508 lines
- `coherence-audit-2026-05-20.md` — 615 lines
- `feature-parity-matrix-2026-05-20.md` — 417 lines
- `performance-benchmark-numbers-2026-05-20.md` — 326 lines
- `benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md` — 119 lines
- `migration-playbooks/from-discourse.md` — 196 lines
- `onboarding/community-engineer-first-week.md` — 290 lines
- `tutorials/configure-anonymous-board-and-moderation.md` — 324 lines
- `faqs/community-engineer-faq.md` — 159 lines

## Deletion

`capability-tiers/` deleted: Y.

## Replacement Count

Rough replacement count: ~35 content replacements plus the directory deletion.

## Design Decisions

- Replaced Bronze/Silver/Gold/Platinum ladder language with `demo_trial`,
  `paid`, compliance-pack, cell-topology, and tenant-class phrasing.
- Replaced exact "golden" evaluation/observability wording with reference-set
  or primary-SRE-signal wording so the literal retirement grep is clean.
- Kept service criticality and T0/T1/T2 AI-capability references where they are
  not customer capability tiers.
- Added a top-level README because this service did not have one; the README
  now points at ADR-0330's `tenant_class` plus `billing_components` model.

## Outstanding Follow-Ups

None for the assigned Bronze/Silver/Gold/Platinum and `capability_tier` scrub.
