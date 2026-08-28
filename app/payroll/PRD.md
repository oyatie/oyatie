---
doc_class: Owner-PRD
owner: app/payroll
status: Accepted
date: 2026-08-27
---

# Payroll product requirements

This PRD defines full-target behavior. The landed-scope section is the only
claim about reviewed implementation; every other section is acceptance for the
remaining work in [PLAN.md](PLAN.md).

<landed_scope>

## Landed foundation

Pure Rust tests currently cover validation and evidence for:

- a caller-supplied wage ledger and trial-close candidate;
- entity-close group rollup and redacted detachment history;
- statutory-export evidence and official-source manifests;
- balanced payroll journal drafts and group posting batches;
- rollback-first close promotion;
- HR leave-impact intake;
- payee variance flags and retro delta arithmetic; and
- volatile metadata duplicate-key behavior.

This foundation does not calculate gross-to-net pay, rates, brackets,
deductions, statutory withholding, employer contributions, or net pay from raw
inputs. It has no typed anomaly-resolution command, durable lifecycle,
production close, pack-install lookup, authorization-evidence adapter,
encrypted SQLite records, downstream delivery, generated Connect process,
deployment, or measured production SLO.

</landed_scope>

<personas_and_workflows>

## Required workflows

| Actor | Workflow | Successful outcome | Refused outcome |
|---|---|---|---|
| Payroll operator | Calculate a trial run | Deterministic gross-to-net results for every eligible payee under one certified overlay | Invalid inputs, missing employment evidence, unsupported overlay, overflow, or unavailable authority yields no mutation |
| Payroll reviewer | Review variance and retro anomalies | Typed findings are resolved, waived, or corrected with principal and evidence | Free-form bypass, missing evidence, conflicting resolution, or unauthorized actor leaves the gate closed |
| Legal-entity payroll operator | Close an entity run | Trial results and resolutions become an immutable entity close with durable audit/accounting intents | Any open anomaly, unbalanced journal, stale overlay, idempotency conflict, or storage fault refuses the transition |
| Group payroll operator | Close a payroll group | One group result references every eligible entity close in deterministic order | Missing, duplicate, foreign-tenant, stale, or incompatible entity closes refuse the whole group |
| Authorized payroll approver | Production-close a run or group | Final immutable outcome is acknowledged after authorization evidence and atomic durability | Missing step-up, stale policy, unresolved anomaly, uncommitted record, or intent failure refuses success |
| HR application | Submit approved leave impact | One durable idempotent Payroll intake linked to HR evidence | Wrong tenant, period, identity, evidence, overlay, or semantic duplicate conflict is refused |
| Accounting application | Consume Payroll journal intent | Balanced evidence-linked intent arrives through Accounting's agreed contract | Unbalanced or unauthorized intent never leaves Payroll; retries keep one operation identity |
| Payroll specialist | Certify jurisdiction content | Versioned owner-local calculation and statutory overlay becomes eligible for its installed pack-id | Blanket EU default, uncertified source, expired rules, or pack mismatch cannot calculate or close |
| On-call operator | Recover after crash or reply loss | A fresh process reopens SQLite and returns the committed outcome without a second effect | Corruption or ambiguous/unavailable authority withdraws readiness, burns budget, and fails closed |

</personas_and_workflows>

<calculation_requirements>

## Deterministic gross-to-net and statutory calculation

1. Calculation accepts bounded typed earnings, time/leave impacts, benefits and
   deductions, tax-profile references, pay period, currency, legal entity,
   payee class, and the selected overlay identity.
2. Money uses checked fixed-point minor units. Rates and thresholds use bounded
   rational/fixed-point values; binary floating point is not part of the
   calculation contract.
3. Each overlay defines effective windows, source evidence, rounding points,
   bracket ordering, employee withholding, employer contribution, net-pay
   constraints, and filing evidence requirements.
4. The result exposes gross earnings, pre-tax deductions, taxable bases,
   employee statutory deductions, employer contributions, post-tax
   deductions, net pay, rounding adjustments, overlay version, and evidence
   digest for each payee and aggregate.
5. Calculation is deterministic under input permutation and replay. Overflow,
   currency mismatch, ambiguous effective date, negative-net prohibition, or
   missing certified rule refuses the complete calculation.
6. United States federal, Korea, Japan, and each supported EU member
   jurisdiction are independent certified overlay versions. “EU” alone is not
   a country calculation default.

</calculation_requirements>

<lifecycle_requirements>

## Gating, close, and correction

1. Trial calculation precedes variance evaluation; supplied-ledger validation
   alone cannot produce a calculated or closed state.
2. Variance and retro findings have stable anomaly ids, typed reasons,
   severity, affected payees/lines, and evidence. Close remains blocked until
   every blocking id has one durable authorized resolution.
3. A resolution is `Corrected`, `AcceptedWithEvidence`, or `Rejected`;
   client text cannot create a new semantic resolution kind.
4. Entity close binds the exact calculated outcome, overlay version, anomaly
   resolution set, accounting draft, and authorization evidence.
5. Group close references immutable entity-close digests, rejects mixed
   tenant/currency/period/overlay eligibility, and cannot synthesize a missing
   entity close.
6. Production close is a distinct step-up-authorized transition. It atomically
   records the final result plus audit and accounting intents before success.
7. Retroactivity creates a new adjustment linked to the original close. It
   never mutates or erases the original outcome and must pass the same
   calculation, variance, resolution, authorization, and close rules.
8. Each accepted mutation records canonical request, outcome, idempotency
   result, authorization evidence, and durable outbound intents in one local
   transaction.

</lifecycle_requirements>

<integration_requirements>

## Ports, overlays, and external owners

- One installed pack-id selects exactly one certified Payroll overlay.
- Policy/IAM returns durable authorization evidence through a Payroll-owned
  port; unavailable, stale, malformed, or denied decisions fail closed.
- Audit, Accounting, HR, Packs, record encryption, and records storage are
  accessed only through agreed or owner-local draft ports and selected
  adapters.
- Protected wage, tax, leave, party, calculation, resolution, and close fields
  are ciphertext at rest; no process-local production key or plaintext
  fallback exists.
- Delivery is asynchronous from committed intents, stable under duplicate
  attempts, and never extends the SQLite transaction across a network call.
- First-party and external callers use the same generated protobuf/Connect
  contract and ordinary principals.

</integration_requirements>

<security_and_privacy>

## Security and data handling

- Every mutation is default-deny and tenant-bound before business-core or
  records invocation.
- Production close and anomaly acceptance require the configured step-up
  action; authorization evidence is part of the atomic record and privileged
  audit-before-ack path.
- Logs and generated errors expose stable redacted reasons and correlation ids,
  never request bodies, wage amounts, party/tax references, plaintext records,
  credentials, or provider keys.
- Evidence references reject credentials, traversal, unbounded text, and
  caller-selected storage locations.
- Retention/deletion follows the certified owner-local overlay and provider
  contract; a client flag cannot weaken it.

</security_and_privacy>

<success_failure_slo>

## Success, failure, and service objectives

Promotion requires the full calculation and close lifecycle, encrypted SQLite
restart/replay proof, selected policy/pack/records/encryption adapters, a real
generated-Connect process, route activation, and production telemetry. Until
then these values are objectives, not landed claims.

| Objective | Target | Population |
|---|---:|---|
| Authorized mutation availability | at least 99.95% per rolling 30 days | All syntactically valid, tenant-eligible requests presenting otherwise eligible authentication material during scheduled service, including dependency outages and readiness withdrawal |
| Mutation completion latency | p99 at most 500 ms for bounded intake/resolution calls | All population calls above through terminal success or server refusal; client cancellation before admission is excluded |
| Trial calculation latency | p99 at most 30 s | Offered calculations with at most 1,000 payees and 10,000 input lines on the published reference cell, including server/dependency failures |
| Acknowledged mutation durability | RPO 0 | Every success returned by the process |
| Local crash recovery | RTO at most 5 min | Every active-process loss during scheduled service with an intact SQLite volume; readiness withdrawal does not stop the clock |
| Durable outbound-intent delivery | at least 99.9% within 60 s | Every committed intent during scheduled service, including destination-adapter outages |

Only predeclared non-service windows, traffic proven invalid or lacking
authentication material, and client cancellation before admission are
excluded. Failure to validate otherwise eligible material because IAM or
another required dependency is unavailable remains included. Records, encryption,
Packs, IAM/Policy, Audit, Accounting, network, saturation, and process outages
remain in the denominator and burn the applicable budget.

Success means a typed result is returned only after authorization evidence and
atomic durability, then replay returns the same result without a second
semantic effect. Failure means a typed refusal, no partial mutation, no
unauthorized core call, and telemetry naming the failed dependency or
invariant.

</success_failure_slo>

<fault_acceptance>

## Required fault evidence

Promotion tests inject faults at identity, policy decision/evidence, pack
selection, overlay certification, calculation boundary, anomaly resolution,
idempotency lookup, encryption, SQLite mutation and intent writes, commit,
reply, delivery, process death, and reopen. Each durability test uses a real
temporary SQLite file, hard-closes all connections at the selected boundary,
constructs a fresh adapter, and asserts state plus replay.

Separate vectors cover malformed protobuf, generated Connect error framing,
JSON product-body and protobuf-JSON rejection, bound-plus-one fields, money
overflow, rounding boundaries, input permutation, deadline/cancellation,
queue saturation, cross-tenant identity, stale/denied policy, unsupported or
expired overlay, unresolved anomaly, missing entity close, duplicate
delivery, dependency outage, and readiness withdrawal.

</fault_acceptance>

<non_goals>

## Non-goals

Payroll does not own HR employment records, Accounting posting, payment
execution, regulator networks, Audit storage, pack installation, IAM/Policy,
Gateway, cloud Data/Storage engines, or Workflow. REST/JSON product
compatibility, protobuf JSON mapping, gRPC, trusted first-party mode,
dual-write storage, synchronous downstream delivery, blanket EU rules, and a
fake facade process are outside the product. Generated Connect error framing
is part of the selected Connect protocol, not a second product payload.

</non_goals>
