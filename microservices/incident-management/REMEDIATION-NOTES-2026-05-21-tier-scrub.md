# Wave 15J-batch-4 Tier Scrub Remediation Notes

Bucket: BUCKET-09
Microservice: incident-management
Date: 2026-05-21

## Files Modified

- ARCHITECTURE.md: 902 lines
- PRD.md: 400 lines
- README.md: 224 lines
- benchmarks/pagerduty-vs-opsgenie-vs-incidentio-vs-oyatie.md: 123 lines
- coherence-audit-2026-05-20.md: 848 lines
- faqs/incident-commander-faq.md: 115 lines
- feature-parity-matrix-2026-05-20.md: 575 lines
- manifest.json: 137 lines
- migration-playbooks/from-pagerduty.md: 166 lines
- onboarding/incident-commander-first-week.md: 143 lines
- performance-benchmark-numbers-2026-05-20.md: 589 lines
- reference-implementations/trigger-and-ack-incident-rust-sdk.md: 277 lines
- src/domain/mod.rs: 581 lines
- tutorials/declare-sev1-incident-end-to-end.md: 215 lines

## Retirement Actions

- capability-tiers/ directory deleted: Y
- Vocabulary replacement count: ~85
- README updated: Y, with ADR-0330 tenant_class + billing_components adoption note.

## Design Decisions

- Renamed the Rust `CapabilityTier` model to `CapabilityAvailability` and changed descriptor fields from `tier` to `availability`.
- Collapsed tier-segmented incident posture language into tenant_class availability, compliance-pack gates, and SLO policy language.
- Replaced capability registry references with tenant_class adoption registry references; shipped migrations were not present in this service.

## Outstanding Follow-ups

- none

## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O conversion for `incident-management`.
- IPs rewritten or deepened in place: 25.
- Files: IP-006-async-event-surface.md, IP-007-grpc-internal-surface.md, IP-008-policy-eval-library-binding.md, IP-009-credential-sidecar-binding.md, IP-010-multi-region-cell-layout.md, IP-011-observability-audit-events.md, IP-012-abuse-defence-edge-waf.md, IP-013-emergency-services-bypass.md, IP-014-marketplace-dealset-settlement.md, IP-015-data-residency-pack-overlays.md, IP-016-backfill-replay-worker.md, IP-017-cost-budget-enforcer.md, IP-018-capacity-admission-control.md, IP-019-sdk-client-generation.md, IP-020-catalog-layer-registration.md, IP-021-slo-gated-promotion.md, IP-022-chaos-drill-pack.md, IP-023-dpia-evidence-packet.md, IP-024-threat-model-control-map.md, IP-025-audit-findings-closeout.md, IP-003-ontology-projection.md, IP-004-workflow-template-library.md, IP-005-rest-contract-surface.md, IP-028-victorops-splunk-on-call-routing-displacement.md, IP-030-incident-io-statuspage-stakeholder-displacement.md.
- Deleted as duplicative: 0; no 80% duplicate pair was removed during this pass.
- Preserved as already-substantive: existing non-stamped IPs outside the short/stamped set retained in place.
- Verification target: no assigned IP remains in the 31-79 line stamp-shell band; rewritten IPs carry real path references and counterpart anchors.

## Wave 15-IP-substance scrub update (2026-05-21)
- Additional rewrite: `IP-001-tenant-scope-kernel.md` and `IP-002-cedar-default-deny.md` were also converted after residual stamped bullet labels were detected.
