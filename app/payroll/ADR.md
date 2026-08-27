---
doc_class: Owner-ADR
owner: app/payroll
status: Accepted
date: 2026-08-27
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Payroll decisions in force

This file specializes ADR-0719 for `app/payroll/`. It defines the full
destination and migration constraints; it does not promote planned behavior to
landed truth.

<current_state>

## Current implementation truth

| Surface | Evidence at `8489b29b` | ADR-0719 destination |
|---|---|---|
| `core/run-domain` | Pure validation and evidence functions for supplied wage ledgers, HR leave intake, entity/group close shape, statutory-source manifests, journal drafts, rollback gates, variance, and retro deltas | **Portable core:** preserve proved domain behavior, detach Data, split to the file budget, and rename `core/run` / `payroll-run`. It is not yet gross-to-net payroll |
| `facade/run-app` | Three wrappers call one domain function each and construct string-topic envelopes | **Delete:** the wrapper functions, topics, envelopes, wrapper outcomes, app error, crate, and facade identity. None invokes a port or adds a real use case |
| `ports/run-api` | Serde JSON DTOs, JSON errors, and domain conversions | **Delete:** required semantic fields return in reviewed protobuf; no DTO is retained as a port |
| `contracts/openapi-v1.yaml` and `contracts/openapi-v1.meta.yaml` | Preview OpenAPI product contract and metadata | **Delete after external references retire:** no compatibility codec or transcode |
| `adapters/run-infrastructure` | In-process REST/JSON router, static bearer verifier, tenant comparison, app-owned PDP traits, limits, and health values; imports Gateway internals | **Delete:** preserve only verified ordering and bounded-work lessons in the later genuine process and selected adapters |
| `adapters/run-storage-inmemory` | Adapter-owned metadata trait, volatile stores, capability flags, and duplicate-key tests | **Delete:** reintroduce a port-owned records/replay contract, a conformance-only in-memory adapter, and an independently built SQLite v1 adapter |
| External bindings | IAM's 39-crate tenant-rbac cone contains every IAM import or name of Payroll; root Cargo config and pipeline admission bind the OpenAPI file | **External-owner retirements:** ADR-0710 amendment plus atomic IAM cone deletion, then root/pipeline binding deletion. Payroll does not edit those paths |

The current source explicitly disclaims tax-rate calculation, disbursement,
regulator transmission, workflow execution, durability, and deployment. Its
trial close validates a caller-supplied wage ledger; it does not calculate
gross-to-net pay.

</current_state>

<product_boundary>

## Decision: Payroll owns the complete pay-run decision lifecycle

- **achieves:** one owner calculates and closes payroll without absorbing HR,
  Accounting, Payments, Audit, Packs, IAM/Policy, or storage engines.
- **origin:** the landed validation foundation can be mistaken for a payroll
  product even though it consumes already-calculated ledger lines.
- **rule:** Payroll MUST own deterministic gross-to-net and statutory
  calculation, trial calculation, variance and retro gates, typed anomaly
  resolution, entity and group close, production close, balanced accounting
  intent, statutory evidence, and its jurisdiction overlay content. It MUST
  NOT own employment truth, ledger posting, payment execution, regulator
  transport, audit storage, pack installation, IAM/PDP, Gateway, or cloud
  storage.
- **ensure:** promotion evidence starts from raw bounded pay inputs and a
  selected certified overlay, produces reproducible payee/run totals, and
  proves every close transition and downstream intent without a copied
  provider engine.
- **overturn_when:** a founder-accepted boundary decision reallocates a named
  behavior and amends each affected owner in the same wave.

</product_boundary>

<portable_architecture>

## Decision: one portable core behind app-owned ports

- **achieves:** Payroll continues on SQLite or commodity providers if an
  Oyatie cloud SKU is absent or retired.
- **origin:** current core and facade code import Data classification directly,
  while the REST adapter imports Gateway internals; ADR-0719 D-23/D-25 makes
  first-party apps ordinary tenants.
- **rule:** domain rules and real port-orchestrating use cases MUST live in
  `core/run`; every IO and cross-owner interaction MUST cross a typed port;
  adapters MUST translate ports to SQLite, commodity systems, or sold Connect
  facades. Core MUST NOT import another app, a cloud core/port, transport,
  serde, SQL, SQLite, or adapter types. First-party execution MUST NOT add a
  trusted mode, skip-PDP path, or in-process cloud shortcut.
- **ensure:** dependency checks reject forbidden edges, and identical core
  suites run with deterministic fakes, in-memory records, and SQLite without
  changing core source.
- **overturn_when:** a five-field architecture decision proves a narrower
  design preserves portability, default-deny policy, and adapter parity with
  less coupling.

</portable_architecture>

<core_and_facade>

## Decision: retain domain behavior, delete facade-shaped no-ops

- **achieves:** useful invariants survive without preserving misleading names
  or pretending a library is a process.
- **origin:** `close_trial_run`, `prepare_accounting_dispatch`, and
  `prepare_hr_leave_impact_intake` only call existing domain functions and
  wrap their results in topic envelopes.
- **rule:** proved pure domain functions and values MUST remain in portable
  core. `PAYROLL_*_TOPIC`, the three envelope structs, the three wrapper
  outcome structs, `PayrollAppError`, and all three wrapper functions MUST be
  deleted. New use cases MUST be introduced only after their records,
  authorization-evidence, pack-install, audit, and destination contracts exist.
  `facade/run-app` MUST remain absent until a real bounded generated-Connect
  process lands; an empty, inert, or always-not-ready `main.rs` MUST NOT count.
- **ensure:** a symbol-by-symbol deletion review finds no wrapper alias or
  string topic, while regression tests preserve every real domain outcome.
- **overturn_when:** ADR-0719's facade definition is explicitly amended and the
  replacement still separates portable use cases from a runnable surface.

</core_and_facade>

<calculation_and_close>

## Decision: calculation and close are distinct deterministic transitions

- **achieves:** a valid input cannot jump from supplied ledger validation to a
  claimed production close.
- **origin:** the current `TrialClosed` state is produced from caller-supplied
  wage lines, with no statutory calculation or durable anomaly resolution.
- **rule:** a run MUST progress through typed trial calculation, variance/retro
  gating, entity close, group close where applicable, and production close.
  Gross, employee deductions, statutory withholding, employer contributions,
  and net pay MUST be derived with checked fixed-point arithmetic, explicit
  rounding, one certified overlay version, and deterministic ordering.
  Unresolved anomalies MUST block close; resolution MUST be typed, authorized,
  evidenced, durable, and non-destructive. Retro changes MUST create adjustment
  records rather than rewrite a closed outcome.
- **ensure:** golden jurisdiction vectors, property tests, permutation tests,
  overflow/rounding faults, state-transition tests, and replay tests prove the
  same canonical input and overlay produce the same byte-stable outcome.
- **overturn_when:** a five-field Payroll decision provides equivalent
  deterministic calculation, immutable close, and auditable correction
  semantics with a smaller state machine.

</calculation_and_close>

<records_and_encryption>

## Decision: SQLite v1 records are atomic, replayable, and ciphertext-only

- **achieves:** every acknowledged mutation survives restart without duplicate
  close or downstream intent, while protected payroll fields are not stored in
  plaintext.
- **origin:** the current in-memory adapter separates reservation from record
  insertion, owns its trait, and loses all state on restart.
- **rule:** the records port MUST atomically bind canonical request,
  idempotency outcome, typed mutation, authorization evidence, and durable
  outbound intents. SQLite v1 MUST commit before acknowledgement, use one
  active adapter per tenant, and MUST NOT dual-write. Protected fields MUST
  cross a separately reviewed record-encryption port and selected provider
  adapter before SQLite encrypted behavior lands; process-local production
  keys and plaintext fallback MUST NOT exist.
- **ensure:** conformance kills the process before/during/after commit, reopens
  a real file with a fresh adapter, and proves pre-commit absence, post-commit
  replay, key/crypto outage refusal, full/busy/corrupt failure, and no partial
  acknowledgement.
- **overturn_when:** a replacement v1 backend and protection provider prove
  equal offline packaging, atomicity, restart recovery, migration, key
  separation, and commodity/cloud portability.

</records_and_encryption>

<sold_contract>

## Decision: one typed protobuf contract through generated Connect

- **achieves:** every client receives one generated contract covering the full
  run lifecycle and bounded semantic failures.
- **origin:** current DTOs, OpenAPI, REST paths, and JSON errors duplicate an
  incomplete domain; the repository has not yet accepted a generated Connect
  toolchain.
- **rule:** the sold IDL MUST live at
  `facade/proto/payroll/run/v1/payroll_run_service.proto`, package
  `payroll.run.v1`, and expose the lifecycle methods named in SPEC. The
  genuine facade MUST use the repository-accepted generated Connect runtime.
  REST product routes, JSON product payloads, protobuf JSON mapping, OpenAPI,
  gRPC, handwritten framing, and standing transcode MUST NOT remain. Only
  error framing emitted by that generated Connect runtime is permitted; it
  MUST NOT become a hand-authored second error schema.
- **ensure:** API review and wire tests cover protobuf success, generated
  Connect errors, malformed/oversized input, deadline, cancellation,
  saturation, and rejection of JSON product bodies, protobuf JSON mapping,
  REST routes, and gRPC trailers/content types.
- **overturn_when:** an accepted protocol decision replaces ADR-0719 and a
  same-wave migration preserves one IDL, equivalent fault evidence, and no
  standing second SDK.

</sold_contract>

<authorization>

## Decision: authorization returns durable evidence through a Payroll port

- **achieves:** no money mutation reaches core or records without a verified,
  tenant-bound, fail-closed decision whose evidence can be audited.
- **origin:** the current HTTP adapter has useful ordering but static bearer
  authn and app-owned PDP traits; it cannot be reused without importing cloud
  internals or preserving REST.
- **rule:** Payroll MUST own an `authorization-evidence` port whose input is a
  verified principal, typed action, tenant resource, canonical request digest,
  and context, and whose success is a durable evidence reference. A selected
  Policy/IAM adapter MUST call only agreed sold contracts and deny on absent,
  refused, stale, timeout, malformed, or faulted decisions. Facade and core
  MUST NOT import Policy/IAM core or port crates, and request tenant identity
  MUST equal the verified tenant.
- **ensure:** contract and process tests prove every refusal makes zero
  business-core calls and zero records writes; approved requests carry the
  returned authorization evidence into the atomic mutation and privileged
  audit-before-ack path.
- **overturn_when:** a five-field zero-trust decision supplies an equally
  fail-closed, durable evidence boundary without a second product API.

</authorization>

<jurisdiction_overlay>

## Decision: Payroll owns overlay content; Packs owns the install

- **achieves:** jurisdictions evolve independently without a second installer
  or tenant/jurisdiction split-brain.
- **origin:** current statutory source manifests prove provenance but can be
  mistaken for calculation rules or a Payroll-owned pack authority.
- **rule:** Payroll calculation, rounding, filing, retention, and evidence
  overlay content MUST live with this owner and be selected by the single
  installed pack-id returned through a Packs adapter. Each supported
  jurisdiction/version MUST be explicit and certified; EU MUST use member
  overlays rather than a blanket country default. `app/payroll/packs/`, a
  second reconciler, combinatoric pack ids, and serving-path policy fetches
  MUST NOT exist.
- **ensure:** tests bind one installed pack-id to one certified owner-local
  overlay and reject missing, unsupported, mismatched, expired, or
  uncertified content before calculation or mutation.
- **overturn_when:** ADR-0719 D-24 is explicitly replaced with a model that
  still preserves one install fact, owner-local content, and fail-closed
  selection.

</jurisdiction_overlay>

<service_objectives>

## Decision: dependency outages remain in the service denominator

- **achieves:** readiness withdrawal cannot make Payroll availability look
  healthy by erasing failed offered work.
- **origin:** the first PRD draft counted only periods when required
  dependencies declared ready, excluding the outages the SLO must expose.
- **rule:** availability and latency populations MUST include all eligible
  offered requests during scheduled service, including requests refused or
  delayed by records, encryption, Packs, IAM/Policy, Audit, or destination
  outages and periods when readiness is withdrawn. Readiness MUST protect
  callers and load balancers, not edit the denominator. Only documented
  non-service windows and traffic proven invalid or lacking authentication
  material may be excluded; inability to validate otherwise eligible material
  because a required dependency failed remains included.
- **ensure:** SLO tests inject each dependency outage and readiness withdrawal
  and prove both burn the applicable budget while emitting dependency-specific
  telemetry.
- **overturn_when:** an accepted SLO policy supplies a stricter offered-load
  population that cannot hide dependency or readiness failures.

</service_objectives>

<alignment>

## ADR alignment

These decisions specialize ADR-0719 D-8, D-23 through D-30, D-33, and D-35;
none amends, diverts from, or overturns ADR-0719. The external prerequisite in
PLAN amends ADR-0710 only to preserve its abstract VAP/CEL/RBAC+PSA invariant
while removing a false concrete crate binding.

</alignment>
