# Wave 15J-batch-4 Tier Scrub Remediation Notes

Bucket: BUCKET-09
Microservice: healthcare-integration
Date: 2026-05-21

## Files Modified

- ARCHITECTURE.md: 902 lines
- IP-019-sdk-client-generation.md: 203 lines
- PRD.md: 400 lines
- README.md: 224 lines
- benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-oyatie.md: 117 lines
- coherence-audit-2026-05-20.md: 1118 lines
- faqs/clinical-integrator-faq.md: 104 lines
- feature-parity-matrix-2026-05-20.md: 551 lines
- manifest.json: 140 lines
- migration-playbooks/from-redox.md: 141 lines
- onboarding/clinical-integrator-first-week.md: 139 lines
- performance-benchmark-numbers-2026-05-20.md: 543 lines
- tutorials/ingest-hl7-orm-and-publish-fhir-servicerequest.md: 222 lines

## Retirement Actions

- capability-tiers/ directory deleted: Y
- Vocabulary replacement count: ~165
- README updated: Y, with ADR-0330 tenant_class + billing_components adoption note.

## Design Decisions

- Replaced clinical capacity and SLO ladder language with tenant_class, compliance_pack, and cell_topology framing.
- Preserved healthcare compliance distinctions as compliance-pack gates rather than customer capability tiers.
- Replaced manifest tier-classification fields with classification/product_subtype/product_classification and `tenant_class_adoption`.

## Outstanding Follow-ups

- none
