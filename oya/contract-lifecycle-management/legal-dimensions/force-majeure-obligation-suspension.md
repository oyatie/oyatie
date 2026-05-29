---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-020
authoritative_source: ICC Force Majeure Clause 2020 + UNIDROIT Principles + civil-law analogues
related_packs: [sox-404]
date: 2026-05-21
---

# Force-Majeure Obligation Suspension

Force-majeure clauses suspend obligations during qualifying events outside the parties' control. The obligation tracker (IP-027) must recognize force-majeure suspension to avoid false breach alerts.

## Force-majeure schema

```
ForceMajeureClause {
  clause_id: ClauseId,
  contract_id: ContractId,
  source_span: ClauseSourceSpan,
  triggering_events: [ForceMajeureEventClass],
  excluded_events: [ForceMajeureEventClass],     // events explicitly carved out
  notification_requirement: NotificationRequirement,
  mitigation_obligation: MitigationObligation,
  duration_cap: Option<DurationCap>,             // max suspension before termination right
  affected_obligation_scope: ObligationScope,
  documentation_requirement: DocumentationRequirement,
}

enum ForceMajeureEventClass {
  NaturalDisaster,                              // earthquake, flood, hurricane
  WarOrArmedConflict,
  Insurrection,
  Terrorism,
  Pandemic,                                     // post-2020 inclusion
  Epidemic,
  GovernmentAction,                             // including sanctions, embargoes
  StrikeOrLabourAction,
  CyberAttackOrInfrastructureFailure,
  EnergyOrFuelShortage,
  RawMaterialShortage,
  SupplyChainDisruption,
  ActOfGod,                                     // catch-all civil-law term
  Other { description: String },
}

struct NotificationRequirement {
  notify_within_days: u32,                      // typically 3-30 days
  notify_methods: [NoticeMethod],
  notify_addresses: [Address],
  failure_to_notify_consequence: NotifyFailure,
}

enum NotifyFailure {
  ForfeitSuspensionRight,                       // missing notice => no force-majeure protection
  ReducedSuspensionPeriod,
  RebuttablePresumption,
}

struct MitigationObligation {
  reasonable_efforts_required: bool,
  alternative_performance_required: bool,
  cost_allocation: CostAllocation,
}

struct DurationCap {
  max_suspension_days: u32,                     // typically 90-180 days
  termination_right_on_expiry: bool,
}

enum ObligationScope {
  AllAffectedObligations,
  PerformanceObligationsOnly,
  PaymentObligationsExcluded,                   // common — payment obligations not suspended
  EnumeratedObligations { obligation_ids: [ObligationId] },
}
```

## Force-majeure event state machine

```
       force_majeure_event_declared(event_class, evidence)
                |
                v
       +------------------------+
       | NOTIFICATION_PENDING   |
       +------------------------+
                |
                | notify_counterparty()
                v
       +------------------------+
       | NOTIFIED               |
       +------------------------+
                |
                | acknowledge() OR no_response_within_period()
                v
       +------------------------+
       | SUSPENSION_ACTIVE      |
       +------------------------+
                |
                +-- event_resolved -----------> +-----------+
                |                              | RESUMED   |
                |                              +-----------+
                |
                +-- duration_cap_exceeded ----> +-----------+
                                                | TERMINATION_RIGHT_AVAILABLE |
                                                +-----------+
```

## Extraction

The obligation-extraction pipeline detects force-majeure clauses by:

- Lexical patterns: "force majeure", "act of God", "fuerza mayor", "höhere Gewalt", "불가항력", "不可抗力", "ekstraordinære forhold".
- Contextual patterns: clauses in "Force Majeure", "Excused Performance", "Suspension of Obligations", "Termination" sections.

Each detected force-majeure clause generates a `ForceMajeureClause` record. The triggering events are extracted from enumerated lists (or signaled as "ALL_REASONABLY_FORESEEABLE" for catch-all formulations).

## Suspension impact on obligation tracker

When a force-majeure event is declared and acknowledged (or no response within the contractual response period):

1. All obligations within `affected_obligation_scope` are marked `force_majeure_suspended`.
2. The obligation due date is **paused** (not extended yet; pause clock running).
3. When event resolves, obligation resumes with the residual time = original due date - event declaration time.
4. If `duration_cap` exceeded, the suspending party gains a termination right.

## Payment obligation default

By default in most commercial contracts, payment obligations are **not** suspended by force majeure (the other party did not cause the event; counterparty should not bear the cost). The µservice's default is `payment_obligations_excluded = true` unless the clause text explicitly suspends payment.

## Pandemic-specific handling

Post-2020 contracts increasingly enumerate `Pandemic` and `Epidemic` separately. Pre-2020 contracts often relied on `Act of God` or `Government Action` catch-alls. The µservice's clause-extraction recognizes both modern (explicit pandemic) and legacy (Act of God) formulations.

For COVID-19-era contracts (effective 2020-2023), the µservice's training data includes case law on whether COVID-19 qualified as force majeure under various clause formulations — this is exposed as confidence-band metadata on the extraction.

## Documentation required during force-majeure period

The party declaring force majeure typically must document:

- Date and nature of the event.
- Evidence of the event (news reports, government declarations, expert attestation).
- Specific obligations affected.
- Mitigation efforts undertaken.
- Updated status reports at agreed intervals.

The µservice produces a force-majeure log per event.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ObligationBreachAlert",
  resource is Obligation
) when {
  resource.force_majeure_suspended == true
};

forbid (
  principal,
  action == Action::"ContractTerminate",
  resource is Contract
) when {
  resource.has_force_majeure_clause == true &&
  resource.force_majeure_state matches "SUSPENSION_ACTIVE" &&
  resource.force_majeure_clause.duration_cap == null
};

permit (
  principal,
  action == Action::"ContractTerminate",
  resource is Contract
) when {
  resource.force_majeure_state matches "TERMINATION_RIGHT_AVAILABLE"
};
```

## Audit events

- `oya.contract.lifecycle.management.force_majeure.event_declared`
- `oya.contract.lifecycle.management.force_majeure.notification_sent`
- `oya.contract.lifecycle.management.force_majeure.suspension_activated`
- `oya.contract.lifecycle.management.force_majeure.obligation_paused`
- `oya.contract.lifecycle.management.force_majeure.duration_cap_exceeded`
- `oya.contract.lifecycle.management.force_majeure.event_resolved`
- `oya.contract.lifecycle.management.force_majeure.obligation_resumed`

## Standards references

- ICC Force Majeure Clause 2020 (publication 1136-0 ENG).
- UNIDROIT Principles of International Commercial Contracts (2016) Article 7.1.7.
- UN Convention on Contracts for the International Sale of Goods (CISG) Article 79.
- Restatement (Second) of Contracts § 261 (impracticability).
- French Civil Code Article 1218 (force majeure).
- German BGB § 313 (Wegfall der Geschäftsgrundlage).
- Japanese Civil Code Article 415 + 419.
- Korean Civil Code Article 390 + 537.
