---
id: ADR-0241
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - ops-dr-capacity
  - ops-sre-reliability
  - ops-compliance
supersedes: []
superseded_by: [ADR-704]
amended_by: [ADR-0343]
related:
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
doc_class: Architecture-Decision-Record
purpose: >
  Four DR tiers per µservice (T1 < 5 min RTO + 0 RPO through T4 < 24 h
  RTO + < 1 h RPO). Drill cadence quarterly for T1/T2, semi-annual for
  T3/T4. Every µservice manifest declares its tier; observability
  visualizes per-µservice last-drill-success-timestamp.
enforcement_status: advisory-until-per-microservice-tier-declared
enforced_by: oya gate validate dr-business-continuity
---

# ADR-0241: DR + business-continuity portfolio policy

## Status

Accepted — 2026-05-18. Enforcement is advisory until every µservice
manifest declares its `dr_tier`, and every T1/T2 µservice has emitted
at least one successful drill receipt to the audit chain.

## Context

ADR-0049 (cross-region replication + residency) covers data replication
at the cell level. ADR-0009 (cell architecture) defines blast-radius
isolation. ADR-0040 (progressive delivery) covers deployment rollback.
But the portfolio has no ADR establishing:

1. The **DR tier** per µservice (RTO + RPO targets).
2. The **drill cadence** to prove the DR plan actually works.
3. The **regulator-evidence cadence** for business continuity.
4. The **per-µservice declaration** that ties the manifest to a tier.

The hyperscaler invariants spec (ADR-0128) references DR as a category
but defers the tier model. Individual µservices (foundry, cloud,
workflow) have informal DR notes in their PRDs but no portfolio binding.

Regulators (CSAP, K-ISMS-P, SOC 2, ISO 22301, FedRAMP, GDPR Art. 32)
require documented RTO/RPO and provable drill history. Without an
explicit ADR, the portfolio cannot produce that evidence pack.

## Decision

### D-1. Four DR tiers

| Tier | RTO | RPO | Replication shape | Drill cadence | Typical µservices |
| --- | --- | --- | --- | --- | --- |
| **T1** | < 5 min | 0 (zero data loss) | Active-active multi-AZ + cross-region warm standby | Quarterly + ad-hoc on every release | Foundry runtime (capability invocation), audit chain, observability, identity kernel, payment, ops-portal |
| **T2** | < 1 h | < 1 min | Active-passive cross-region with continuous replication | Quarterly | Workflow engine, ontology, search, messenger, mail, drive, sites, calendar |
| **T3** | < 4 h | < 15 min | Backup + restore + cross-region warm | Semi-annual | Analytics, ads, sheets, slides, forms, notes, recordings, social, community, shorts, calendar, tasks |
| **T4** | < 24 h | < 1 h | Backup + restore + cold standby | Semi-annual | Internal-only µservices (the cloud-iac control plane is itself T2; this row is for low-volume internal staff tools) |

`Active-active` means simultaneous traffic from multiple AZs/regions;
`Active-passive` means standby instance that takes over on primary
failure.

### D-2. Per-µservice declaration

Each µservice manifest at `microservices/<ms>/manifest.json` carries:

```json
{
  "dr_tier": "T1" | "T2" | "T3" | "T4",
  "dr_owner_team": "ops-sre-reliability" | "ops-dr-capacity" | "<axis-team>",
  "rto_minutes": 5,
  "rpo_seconds": 0,
  "replication_shape": "active-active-multi-az-cross-region-warm",
  "last_drill_evidence_id": "<audit-chain-event-id>",
  "next_drill_due_at": "2026-08-15"
}
```

### D-3. Drill mechanics

A drill is a controlled failover from primary to DR substrate, validated
against:

- **Functional**: every critical user journey succeeds against the DR
  substrate.
- **Performance**: SLOs met within 2× steady-state targets for T1/T2;
  within 5× for T3/T4.
- **Data**: per-tenant data integrity sample verified post-failover.
- **Audit**: drill emits to the audit chain with class
  `DrDrillReceipt` carrying {microservice_id, tier, started_at,
  completed_at, success: bool, findings[]}.

Drill orchestration lives in
`microservices/cloud-iac/src/dr_drill_orchestrator.rs` (planned;
backlog tracked at `registry/dr/orchestrator-backlog.tsv`).

### D-4. Replication shape per tier

| Replication shape | Substrate | Latency cost | Storage cost |
| --- | --- | --- | --- |
| `active-active-multi-az-cross-region-warm` | Synchronous multi-AZ + asynchronous cross-region | ~2 ms write tail | 2× storage |
| `active-passive-cross-region-continuous` | Asynchronous cross-region replication (< 1s lag) | ~0 ms write tail | 2× storage |
| `backup-restore-cross-region-warm` | Periodic backup (hourly) + warm instance in DR region | 0 ms write tail | 1.1× storage |
| `backup-restore-cold` | Periodic backup (daily); restore-on-demand | 0 ms | 1.05× storage |

The cloud-iac layer (per ADR-0240 sovereign cloud overlay) implements
each shape against the pack's primary + secondary providers.

### D-5. Cross-tier coordination

A T1 µservice depending on a T2 µservice MUST tolerate the T2's RTO.
This is the **degraded-dependency rule**: a T1 µservice's drill plan
includes scenarios where its T2 dependency is in DR (and thus may have
< 1h lag). The brown-out signal (ADR-0176) is the channel: when a T2
dependency is in DR, the T1 µservice transitions to `degraded` until
the dependency recovers.

### D-6. Business continuity (broader than DR)

The DR tier covers infrastructure failure. Business continuity covers
broader scenarios:

| Scenario | Plan owner | Drill cadence |
| --- | --- | --- |
| Single-AZ failure | ops-sre-reliability | Quarterly (covered by T1/T2 drill) |
| Region failure | ops-dr-capacity | Quarterly (T1) / semi-annual (T2) |
| Provider failure (per ADR-0240) | ops-dr-capacity + ops-compliance | Semi-annual |
| Sovereign-pack regulator suspension | ops-compliance | Tabletop annually |
| Cyber-attack / ransomware | ops-security + ops-dr-capacity | Tabletop annually |
| Pandemic / staff unavailability | council-architecture | Tabletop annually |
| Multi-cell catastrophic loss | ops-dr-capacity | Tabletop semi-annual |

Tabletop drills produce a `BusinessContinuityTabletop` audit row but
do not actually fail over.

### D-7. Observability

Per-µservice DR tier + last-drill-success-timestamp visualized on
`microservices/observability/dashboards/dr-business-continuity.md`.

Anomaly: per-µservice `last_drill_success_at` older than 2× cadence
pages ops-dr-capacity (SEV-3). Per-µservice failed-drill rate > 5% pages
council-architecture (SEV-2).

### D-8. Regulator evidence cadence

`ops-compliance` emits a quarterly DR + BC evidence packet (audit-chain
class `DrBcQuarterlyEvidence`) containing:

- Per-µservice tier declaration.
- Per-µservice drill history within the quarter.
- Per-µservice findings + remediation timeline.
- Cross-provider failover proof (if any).

Regulators (CSAP, ISO 22301, SOC 2) consume the packet.

## Alternatives considered

### Alt-1. Single global RTO/RPO target

Use one RTO + RPO across the whole portfolio. **Rejected.** Forces T3/T4
µservices to over-engineer (huge replication cost) OR forces T1
µservices to under-engineer (regulator + customer-experience failure).
Tier discipline is industry standard.

### Alt-2. Per-µservice DR plan, no portfolio tier model

Let each µservice define its DR plan freely. **Rejected.** Defeats the
cross-tier coordination rule (D-5); makes regulator-evidence packets
heterogeneous; obscures cross-µservice dependency analysis.

### Alt-3. Active-active everywhere

Make every µservice T1. **Rejected.** Cost-prohibitive (~2× substrate
spend for the entire portfolio); operationally complex; provides
diminishing-returns RTO/RPO for low-volume µservices.

## Consequences

### C-1. Positive

- **Regulator evidence is automatic.** Quarterly packet enumerates
  drill history; no manual evidence extraction.
- **Cross-tier coordination is explicit.** Brown-out signal carries
  the degraded-dependency signal.
- **Drill discipline is enforceable.** Validator flags µservices whose
  last drill is older than 2× cadence.
- **Per-cell drill is bounded.** A drill is per cell + per µservice;
  it does not span the entire portfolio.
- **Hyperscaler-grade.** Matches AWS multi-AZ + multi-region DR
  guidance; Google SRE DR tier model; Microsoft Azure Site Recovery
  RPO/RTO contract.

### C-2. Negative

- **T1 replication doubles substrate spend.** Mitigation: T1 list is
  intentionally short (foundry, audit chain, identity, observability,
  payment, ops-portal); FinOps (ADR-0174) tracks the spend per
  microservice.
- **Drill orchestration is non-trivial.** Mitigation: orchestrator
  implementation backlogged but covered by the cloud-iac µservice's
  IP catalog.
- **Cross-tier dependency reasoning is harder than single-tier.**
  Mitigation: brown-out signal (ADR-0176) carries the degraded
  state; static-stability hook (ADR-0009) is the canonical caller-side
  pattern.

### C-3. Sustainability

- Cold standby (T4) consumes ~5% of primary substrate. Warm standby
  (T3) consumes ~10%. Active-active (T1) consumes ~100% additional.
  Carbon cost per tier is visible in the FinOps + sustainability tag
  (ADR-0174).

## Implementation surface

- `specs/dr-business-continuity.json` — canonical tier enum + per-tier
  schema.
- `docs/standards/dr-business-continuity.md` — full standards doc with
  worked drill plan per tier.
- `registry/dr/per-microservice-tier.yaml` — per-µservice declared
  tier (initial assignments per D-1 table).
- `microservices/observability/dashboards/dr-business-continuity.md`
  — dashboard schema.
- New validator lane `dr-business-continuity` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).
- Orchestrator: `microservices/cloud-iac/src/dr_drill_orchestrator.rs`
  (backlog tracked at `registry/dr/orchestrator-backlog.tsv`).

## References

- AWS Well-Architected — *Reliability pillar: DR strategy* (4 tiers).
- Google SRE — *Risk-aligned DR tiering* (ch. 4 of *Site Reliability
  Engineering Workbook*).
- Microsoft Cloud Adoption Framework — *Business continuity and
  disaster recovery* (BCDR).
- ISO 22301:2019 — *Security and resilience — Business continuity
  management systems*.
- AWS — *re:Invent 2023 DR demos* (multi-region + cross-account
  drills).
- ADR-0049 (this portfolio) — cross-region replication + residency.
- ADR-0009 (this portfolio) — cell architecture (blast-radius
  isolation).
- ADR-0240 (this portfolio) — sovereign cloud per regional pack
  (provides the substrate set for cross-provider failover).
- ADR-0176 (this portfolio) — brown-out + degradation signal API.
