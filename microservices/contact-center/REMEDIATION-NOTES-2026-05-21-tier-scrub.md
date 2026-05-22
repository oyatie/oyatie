# Wave 15J-batch-4 tier scrub notes — contact-center

## Summary
- capability-tiers/ dir deleted: Y
- Vocabulary replacement count: ~90
- Replacement doctrine: ADR-0330 tenant_class (`demo_trial`, `paid`) plus paid `billing_components` (`per_seat`, `per_usage` for contact-center).
- Verification: required Bronze/Silver/Gold/Platinum and capability_tier/max_tier/tier_threshold scans return zero matches outside this note.

## Files Modified With Current Line Counts
- README.md — 223 lines
- manifest.json — 141 lines
- PRD.md — 400 lines
- ARCHITECTURE.md — 902 lines
- benchmarks/genesys-vs-five9-vs-aws-connect-vs-oyatie.md — 119 lines
- faqs/contact-center-admin-faq.md — 68 lines
- migration-playbooks/from-genesys.md — 199 lines
- onboarding/contact-center-admin-first-week.md — 123 lines
- tutorials/build-ivr-flow-with-pci-suppression.md — 196 lines
- coherence-audit-2026-05-20.md — 633 lines
- feature-parity-matrix-2026-05-20.md — 454 lines
- performance-benchmark-numbers-2026-05-20.md — 685 lines
- IP-001-tenant-scope-kernel.md — 113 lines
- IP-002-cedar-default-deny.md — 111 lines
- IP-003-ontology-projection.md — 104 lines
- IP-004-workflow-template-library.md — 108 lines
- IP-005-rest-contract-surface.md — 114 lines
- IP-026-omnichannel-routing-policy-engine.md — 104 lines
- IP-027-recording-consent-redaction-vault.md — 105 lines
- IP-028-workforce-adherence-stream.md — 104 lines
- IP-029-agent-assist-escalation-guardrail.md — 103 lines
- IP-030-callback-and-sla-rescheduler.md — 104 lines

## Design Decisions
- Replaced service manifest tier fields with `tenant_class_eligibility`, `paid_billing_components_emitted`, `tenant_class_doctrine`, and cell-topology eligibility.
- Rewrote contact-center benchmark posture from named paid tiers into paid tenant_class deployment-context language.
- Preserved compliance-pack distinctions as compliance-pack gating, not pricing tiers.

## Outstanding Follow-ups
- None for the assigned scrub checks.
