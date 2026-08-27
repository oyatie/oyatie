---
doc_class: Owner-SPEC
owner: app/payroll
status: Accepted
date: 2026-08-27
---

# Payroll behavior and contract

This specification separates reviewed behavior from the full target. Planned
paths, types, states, and RPCs are not landed claims.

<current_contract>

## Reviewed behavior

The payroll-run-domain package currently exposes validation and pure evidence
functions: trial_close, ingest_hr_leave_impact, close_group_rollup,
statutory_export_evidence, build_statutory_rulepack_manifest,
build_payroll_journal, build_group_gl_posting, evaluate_close_promotion,
evaluate_payroll_variance, and evaluate_retro_adjustment. The functions
classify values, validate identifiers/evidence, balance supplied ledger lines,
and derive evidence metadata. They do not derive wage lines or statutory
amounts from raw payroll inputs.

The payroll-run-app package adds no independent behavior. Its public API and
private topic constants map as follows:

| Existing symbol | Exact disposition |
|---|---|
| PAYROLL_CLOSE_TOPIC, PAYROLL_ACCOUNTING_TOPIC, PAYROLL_HR_LEAVE_IMPACT_TOPIC | Delete string routing |
| PayrollAuditEnvelope, PayrollAccountingDispatchEnvelope, PayrollHrLeaveImpactEnvelope | Delete transport-shaped metadata envelopes |
| TrialCloseOutcome, AccountingBridgeOutcome, HrLeaveImpactIntakeOutcome | Delete wrapper outcomes; later use cases define new typed outcomes from real port contracts |
| PayrollAppError::Domain | Delete one-variant wrapper error |
| close_trial_run, prepare_accounting_dispatch, prepare_hr_leave_impact_intake | Delete no-op wrappers; do not preserve aliases |
| Underlying domain functions and values | Retain in portable core, subject to honest naming and full-target extensions |

The current API/infrastructure crates are JSON/OpenAPI and an in-process HTTP
router. The current storage crate owns its trait and stores volatile metadata.
None is the target wire, process, use-case, or durable contract.

</current_contract>

<target_core>

## Portable calculation and lifecycle core

The primary package is core/run / payroll-run. Its modules have these
responsibilities:

| Module | Contract |
|---|---|
| identity and money | Bounded tenant/entity/run/group/payee/employee/period/evidence/idempotency types; checked fixed-point money and rate arithmetic |
| pay inputs | Earnings, time/leave impacts, benefits/deductions, tax-profile references, payee class, and canonical ordering |
| jurisdiction overlay | Typed effective window, brackets/rates, rounding rules, withholding/contribution rules, filing/retention evidence, and certification identity |
| calculation | Deterministic per-payee and aggregate gross, taxable bases, deductions, withholding, employer contributions, rounding adjustments, and net |
| variance and retro | Stable anomaly ids, variance/retro findings, correction deltas, and blocking status |
| anomaly resolution | Typed Corrected, AcceptedWithEvidence, and Rejected decisions bound to principal/evidence; no free-form bypass |
| entity/group close | Immutable entity-close digest and deterministic group aggregation over eligible entity closes |
| production close | Step-up-authorized final transition plus balanced accounting and privileged audit intents |
| HR intake | Validate an authenticated HR leave-impact command and translate it to Payroll-owned input |
| use cases | Invoke ports in the specified order and return transport-neutral committed outcomes |

Core accepts explicit inputs and traits. It does not read time, environment,
network, SQL, SQLite, serde, generated transport, or another owner directly.

</target_core>

<calculation_contract>

## Deterministic gross-to-net

CalculateTrialRun canonicalizes payees and input lines by typed identity,
selects one certified overlay version, and calculates with checked fixed-point
minor units and rates. Each calculation records:

- source input digest and overlay id/version/effective window;
- gross earnings and each taxable basis;
- pre-tax, statutory, and post-tax employee deductions;
- employee withholding and employer contributions by typed rule id;
- explicit rounding adjustments at overlay-defined boundaries;
- net pay and aggregate debit/credit basis;
- source and certification evidence; and
- a calculation-algorithm version.

The same canonical input and overlay produces byte-equivalent semantic output
independent of input order. Overflow, unsupported currency, ambiguous
effective window, duplicate identity, missing rule, invalid negative net, or
uncertified overlay returns a typed refusal for the whole calculation.

Owner-local overlay content is selected by one Packs-installed id. Supported
content is versioned independently for us/federal, kr, jp, and explicit EU
member namespaces such as eu/de; eu alone cannot calculate country payroll. A
request never fetches or interprets git content.

</calculation_contract>

<state_machine>

## Run lifecycle and anomaly gates

    Draft
      -> TrialCalculated
      -> ReviewBlocked | ReviewClear
    ReviewBlocked
      -> ReviewBlocked | ReviewClear       (typed durable resolutions only)
    ReviewClear
      -> EntityClosed
    EntityClosed[*]
      -> GroupClosed                       (when a group applies)
    EntityClosed | GroupClosed
      -> ProductionClosed                  (step-up authorization)
    ProductionClosed
      -> RetroAdjustmentDraft              (new linked record, never mutation)
    RetroAdjustmentDraft
      -> TrialCalculated                   (same calculation/gate path)

Every transition compares an expected prior version and stores a new immutable
outcome. ReviewClear requires every blocking anomaly id to have exactly one
current authorized resolution. Corrected calculation invalidates resolutions
whose input digest changed. Entity close binds calculation, overlay, anomaly
set, resolutions, accounting draft, and evidence. Group close rejects absent,
duplicate, foreign-tenant, wrong-period/currency, or ineligible entity closes.
Production close records authorization evidence and audit/accounting intents
atomically. Retroactivity links to the closed outcome and never rewrites it.

</state_machine>

<port_contracts>

## Port boundaries

| Need | Payroll contract | Required semantics |
|---|---|---|
| Records/replay | ports/draft/records / payroll-records-draft until agreed | canonical request lookup, expected-version mutation, outcome, authorization evidence, and outbound intents in one atomic commit |
| Installed pack | ports/draft/pack-install until provider settlement | resolve one tenant/cell pack-id and version; fail closed on stale, absent, or mismatch |
| Authorization evidence | ports/draft/authorization-evidence until Policy/IAM settlement | verified principal + action/resource/context + request digest to durable allow evidence or typed refusal |
| Audit intent | ports/draft/audit until Audit settlement | idempotent privileged Payroll decision evidence before acknowledgement |
| Accounting intent | ports/draft/accounting until Accounting settlement | idempotent balanced posting request and operation lookup |
| Record encryption | provider/path unresolved until the required owner/architecture decision | seal/open bounded records with purpose, tenant, schema, generation, and authenticated context; provider keys never enter Payroll |

Draft ports remain owner-local and cannot be imported by another owner. A
second consumer or provider binding triggers ADR-0719 D-28/D-29 settlement,
promotion to the provider-owned agreed port/proto, and cross-owner review.
Selected adapters import only sold provider contracts, never provider core or
internal ports.

</port_contracts>

<mutation_protocol>

## Atomic mutation and idempotent replay

Every mutating RPC supplies a bounded idempotency key. After identity,
authorization evidence, pack selection, and overlay certification, the use case
derives a versioned canonical request from verified tenant, method, semantic
input, expected state version, and overlay version. The records port returns:

- Absent: calculate/transition, encrypt protected record values, then
  atomically persist canonical request, typed outcome, authorization evidence,
  and outbound intents;
- CommittedSameRequest: return the stored outcome without reevaluation or a
  second intent; or
- CommittedDifferentRequest: return IdempotencyConflict without mutation.

SQLite v1 uses one writer transaction. Success follows commit. Pre-commit crash
leaves no visible result; reply loss after commit resolves by reopening and
lookup. Delivery workers claim committed intents, send a stable operation id,
and persist delivery status; at-least-once delivery is expected. An ambiguous
commit is never converted to success without resolving durable state.

</mutation_protocol>

<use_case_contract>

## Port-orchestrating use cases

These use cases are new behavior; none aliases a deleted run-app wrapper:

| Use case | Required preconditions | Committed result |
|---|---|---|
| CalculateTrialRun | authorization evidence, installed pack, certified overlay, valid raw pay inputs | calculated trial plus variance/retro findings |
| ResolveRunAnomaly | step-up action where configured, current anomaly/input digest, typed resolution evidence | new resolution set and resulting gate state |
| CloseEntityRun | review clear, balanced journal, expected version | immutable entity close plus audit/accounting intents |
| CloseGroupRun | all listed entity closes eligible and compatible | immutable group close plus group accounting intent |
| CloseProductionRun | eligible entity/group close and production-close authorization evidence | immutable production close plus privileged audit intent |
| CreateRetroAdjustment | production-close reference and corrected raw inputs | linked adjustment draft entering the calculation/gate path |
| PrepareAccountingJournal | calculated outcome and resolved gate | balanced transport-neutral journal intent |
| IntakeHrLeaveImpact | authenticated HR evidence, current run/period, installed overlay | durable Payroll-owned leave-impact input |

</use_case_contract>

<proto_contract>

## Sold Payroll run service

Source: facade/proto/payroll/run/v1/payroll_run_service.proto

Package: payroll.run.v1

Unary RPCs:

- CalculateTrialRun(CalculateTrialRunRequest) -> CalculateTrialRunResponse;
- ResolveRunAnomaly(ResolveRunAnomalyRequest) -> ResolveRunAnomalyResponse;
- CloseEntityRun(CloseEntityRunRequest) -> CloseEntityRunResponse;
- CloseGroupRun(CloseGroupRunRequest) -> CloseGroupRunResponse;
- CloseProductionRun(CloseProductionRunRequest) -> CloseProductionRunResponse;
- CreateRetroAdjustment(CreateRetroAdjustmentRequest) -> CreateRetroAdjustmentResponse;
- PrepareAccountingJournal(PrepareAccountingJournalRequest) -> PrepareAccountingJournalResponse;
- IntakeHrLeaveImpact(IntakeHrLeaveImpactRequest) -> IntakeHrLeaveImpactResponse.

Messages use bounded nested money, rate, input-line, calculation, anomaly,
resolution, close, evidence, and outcome values. They do not expose Rust
Classified<T>, string topics, adapter/storage names, or internal flags. Every
mutation includes tenant, idempotency key, and expected version where
applicable; request tenant equals the verified principal's tenant.

Stable semantic error details include Unauthenticated, PermissionDenied,
TenantMismatch, InvalidArgument, IdempotencyConflict, VersionConflict,
OverlayUnavailable, OverlayUncertified, CalculationOverflow,
AnomalyGateClosed, ResolutionConflict, UnbalancedJournal,
RecordsUnavailable, EncryptionUnavailable, Saturated, DeadlineExceeded, and
InternalRefusal.

The product accepts protobuf Connect requests/responses only. REST paths, JSON
product payloads, protobuf JSON mapping, gRPC content types/trailers,
streaming, and handwritten framing are rejected. The selected generated
Connect runtime may emit its protocol-defined error framing; Payroll does not
hand-author, extend, or expose it as a second schema.

</proto_contract>

<process_contract>

## Genuine process and authorization order

facade/run-app is promoted only with a real binary, generated handlers,
bounded admission, composition root, readiness, telemetry, shutdown, and
activated platform route. Boot selects exactly one records, encryption,
pack-install, authorization-evidence, audit, and accounting adapter. Missing
or incompatible required configuration refuses boot or withdraws readiness;
readiness withdrawal still burns scheduled-service SLO budget.

For each mutation:

1. accept a bounded generated Connect request and correlation id;
2. obtain the platform-verified ordinary principal;
3. bind typed Payroll action, verified tenant resource, request digest, and
   context;
4. obtain durable authorization evidence through the selected Policy/IAM
   adapter;
5. verify request tenant equals verified tenant;
6. resolve one installed pack-id and certified owner-local overlay;
7. resolve idempotency and invoke the named core use case;
8. encrypt protected values and atomically commit outcome, authorization
   evidence, and intents; and
9. return the committed generated response while delivery proceeds from
   durable intents.

No business-core or records lookup runs before steps 2–6 succeed. Health and
readiness are platform operational surfaces, not Payroll REST/JSON methods.
Shutdown stops admission, drains bounded work, leaves committed intents
recoverable, and closes adapters; graceful close is not recovery evidence.

</process_contract>

<observability_and_faults>

## Telemetry and conformance

Metrics cover eligible offered, accepted, refused, and completed calls;
latency by semantic RPC; calculation/rounding/anomaly outcomes; authorization
and overlay refusal; replay/conflict; encryption and SQLite commit/recovery;
pending-intent age; delivery retry; saturation; dependency state; and
readiness. SLO accounting includes dependency outage and readiness withdrawal.

Logs carry opaque correlation/operation/tenant identifiers, semantic reason,
overlay version, and adapter identity while excluding credentials, request
bodies, amounts, party/tax references, plaintext, and keys. Traces connect
generated facade, authorization evidence, calculation/use case, records
transaction, and delivery through opaque operation ids.

Conformance includes pure calculation goldens and properties, state-machine
tests, every port fake, in-memory reference parity, real-file SQLite
restart/replay, encryption-provider faults, protobuf byte/limit goldens,
generated Connect errors, authorization ordering, adapter parity, cross-owner
contracts, dependency outage, saturation, readiness withdrawal, and every PRD
fault boundary.

</observability_and_faults>
