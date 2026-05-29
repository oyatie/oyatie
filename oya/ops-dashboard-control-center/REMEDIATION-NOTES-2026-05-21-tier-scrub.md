# Wave 15J-batch-4 Tier Scrub Remediation Notes

Bucket: BUCKET-09
Microservice: ops-dashboard-control-center
Date: 2026-05-21

## Files Modified

- ARCHITECTURE.md: 1238 lines
- README.md: 89 lines
- benchmarks/odcc-vs-pagerduty-vs-statuspage-vs-incident-io-vs-firehydrant.md: 143 lines
- capabilities/cluster-health-query.yaml: 12 lines
- capabilities/deployment-approve.yaml: 12 lines
- capabilities/evidence-pack-export.yaml: 12 lines
- capabilities/incident-declare.yaml: 12 lines
- capabilities/incident-remediation-approve.yaml: 12 lines
- capabilities/rollback-execute.yaml: 12 lines
- capabilities/step-up-auth-challenge.yaml: 33 lines
- capabilities/tenant-isolation-posture-query.yaml: 12 lines
- coherence-audit-2026-05-20.md: retained as scrubbed historical audit text
- faqs/sre-on-call-faq.md: 125 lines
- feature-parity-matrix-2026-05-20.md: 415 lines
- manifest.json: 551 lines
- performance-benchmark-numbers-2026-05-20.md: 320 lines
- tutorials/declare-incident-rollback-and-export-signed-evidence-pack.md: 297 lines

## Retirement Actions

- capability-tiers/ directory deleted: Y
- Vocabulary replacement count: ~55
- README updated: Y, with ADR-0330 tenant_class + billing_components adoption note.

## Design Decisions

- Replaced capability manifest `tier` fields with `availability` so operator actions describe paid always-on posture and demo_trial caps/read-only limits.
- Reworded benchmark and FAQ ladder language to tenant_class, compliance-pack, and step-up authorization gates.
- Preserved infrastructure criticality vocabulary where it describes cell topology rather than customer capability tiers.

## Outstanding Follow-ups

- none

## Wave 15-IP-substance scrub (2026-05-21)

- Rewritten in place as bespoke substance: `IP-001-control-plane-manifest-and-contracts.md` through `IP-016-on-call-handoff-bc.md`.
- Preserved as already substantive with explicit counterpart verification note where needed: journey IP files.
- Deleted as duplicative: none. Operator-evidence-console journey files share a generator frame but differ by journey/regulator object and require separate implementation PRs.
- Counterpart anchors added: AWS internal console, Stripe Internal Admin, Backstage, OpsLevel, Port, PagerDuty, ServiceNow, GitHub, Datadog/Grafana-style observability surfaces.
