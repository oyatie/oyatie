---
contract: dr-business-continuity
authored: 2026-05-18
canonical_authority: ADR-0180
related_specs:
  - /specs/dr-business-continuity.json
related_adrs:
  - ADR-0009
  - ADR-0040
  - ADR-0049
  - ADR-0128
  - ADR-0176
  - ADR-0179
  - ADR-0180
status: canonical-base
authorities_cited:
  - AWS Well-Architected Reliability pillar 4-tier DR
  - Google SRE Workbook risk-aligned DR tiering (ch. 4)
  - Microsoft Cloud Adoption Framework BCDR
  - ISO 22301:2019 business continuity management
---

# DR + business-continuity standards

## Four DR tiers

| Tier | RTO | RPO | Replication | Drill cadence | Typical µservices |
| --- | --- | --- | --- | --- | --- |
| T1 | < 5 min | 0 | active-active multi-AZ + cross-region warm | quarterly + ad-hoc per release | foundry runtime, audit chain, observability, identity kernel, payment, ops-portal |
| T2 | < 1 h | < 1 min | active-passive cross-region continuous | quarterly | workflow engine, ontology, search, messenger, mail, drive, sites, calendar |
| T3 | < 4 h | < 15 min | backup + restore + cross-region warm | semi-annual | analytics, ads, sheets, slides, forms, notes, recordings, social, community, shorts, tasks |
| T4 | < 24 h | < 1 h | backup + restore + cold | semi-annual | low-volume internal staff tools |

## Per-µservice declaration

In `microservices/<ms>/manifest.json`:

```yaml
dr_tier: T1
dr_owner_team: ops-sre-reliability
rto_minutes: 5
rpo_seconds: 0
replication_shape: active-active-multi-az-cross-region-warm
last_drill_evidence_id: <audit-chain event id>
next_drill_due_at: 2026-08-15
```

## Drill mechanics

A drill = controlled failover from primary to DR substrate. Validated
against:

| Axis | T1/T2 acceptance | T3/T4 acceptance |
| --- | --- | --- |
| Functional | every critical user journey succeeds against DR | every critical user journey succeeds |
| Performance | SLOs met within 2× steady-state | SLOs met within 5× |
| Data | per-tenant integrity sample verified post-failover | sample verified |
| Audit | drill emits `DrDrillReceipt` row | same |

Orchestrator: `microservices/cloud-iac/src/dr_drill_orchestrator.rs`
(backlog at `registry/dr/orchestrator-backlog.tsv`).

## Replication shapes

| Shape | Substrate | Write latency cost | Storage cost |
| --- | --- | --- | --- |
| active-active multi-AZ + cross-region warm | synchronous multi-AZ + async cross-region | ~2 ms p99 tail | 2× |
| active-passive cross-region continuous | async cross-region < 1s lag | ~0 ms | 2× |
| backup + restore + cross-region warm | hourly backup + warm DR instance | 0 | 1.1× |
| backup + restore + cold | daily backup + restore-on-demand | 0 | 1.05× |

## Cross-tier coordination

A T1 µservice MUST tolerate a T2 dependency in DR (up to 1 hour RTO).
Implementation: caller-side static-stability hook per ADR-0009 +
brown-out signal class transition per ADR-0176 (`degraded` while T2
dependency is in DR).

## Business continuity (broader than DR)

| Scenario | Owner | Cadence |
| --- | --- | --- |
| Single-AZ failure | ops-sre-reliability | quarterly drill (covered by T1/T2) |
| Region failure | ops-dr-capacity | quarterly (T1) / semi-annual (T2) |
| Provider failure | ops-dr-capacity + ops-compliance | semi-annual |
| Sovereign-pack regulator suspension | ops-compliance | annual tabletop |
| Cyber-attack / ransomware | ops-security + ops-dr-capacity | annual tabletop |
| Pandemic / staff unavailability | council-architecture | annual tabletop |
| Multi-cell catastrophic loss | ops-dr-capacity | semi-annual tabletop |

Tabletops emit `BusinessContinuityTabletop` audit rows but do not
actually fail over.

## Observability

Dashboard:
`microservices/observability/dashboards/dr-business-continuity.md`
shows per-µservice tier + last-drill timestamp + next-drill due.

Alerts:

- Last-drill-success older than 2× cadence → SEV-3 to ops-dr-capacity.
- Per-µservice failed-drill rate > 5% → SEV-2 to council-architecture.

## Regulator evidence

`ops-compliance` emits quarterly `DrBcQuarterlyEvidence` audit row
containing:

- Per-µservice tier declaration.
- Per-µservice drill history within the quarter.
- Findings + remediation timelines.
- Cross-provider failover proof (if any).

Regulators (CSAP, ISO 22301, SOC 2) consume via cloud-iac audit-export.

## Cost dimension

Per-tier substrate spend multiplier:

| Tier | Substrate cost over primary |
| --- | --- |
| T1 | +100% (active-active doubles substrate) |
| T2 | +20% (warm standby) |
| T3 | +10% (warm DR + hourly backup storage) |
| T4 | +5% (cold backup storage) |

FinOps tracks the per-µservice DR substrate spend via the
`sustainability_class` tag from ADR-0174.

## Coverage tracker

Per-µservice tier declaration rollout in
`registry/dr/per-microservice-tier-declaration-tracker.tsv`. Validator
lane `dr-business-continuity` is advisory until coverage reaches 100%.
