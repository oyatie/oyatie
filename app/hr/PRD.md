---
doc_class: Owner-PRD
owner: app/hr
status: Active
date: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - app/hr/ADR.md
---

# HR product requirements

<product_boundary>

`app/hr/` is the portable People and employment application. It owns the
tenant's employee/employment record, organization and manager relationships,
onboarding readiness, leave policy projections, labor-compliance obligations,
sensitive-access policy, and the evidence references needed to explain those
decisions.

It does not own payroll calculation or payment, workflow execution, immutable
audit storage, identity or policy decision, the Data/Storage/Gateway engines,
notification delivery, or deployment infrastructure. Those are replaceable
effects behind HR-owned ports. HR is an ordinary tenant of each selected
adapter, including Oyatie cloud adapters.

</product_boundary>

<users>

- People operators create and maintain legal-entity-scoped employee records,
  determine onboarding readiness, and see explicit blockers and evidence.
- Managers and delegates make leave decisions through policy-bound routing.
- Compliance operators evaluate jurisdictional thresholds and trace every
  obligation to a versioned rulepack source and workflow intent.
- Payroll, workflow, and audit systems consume typed, idempotent intent without
  becoming co-owners of HR records.
- Employees and authorized operators access sensitive HR data only for an
  allowed purpose and legal basis with durable access evidence.
- Operators need deterministic recovery, adapter health, queue/transaction
  telemetry, schema migration, and tenant-scoped incident evidence.

</users>

<landed_scope>

## Current foundation

Landed tests cover employee construction and lifecycle events, Korea rules-of-
employment and labor-management-council thresholds, leave payroll-impact and
balance projections, carryover/forfeiture, onboarding readiness, statutory
source manifests, sensitive-read denial rules, DTO conversion, in-process
authorization, and volatile idempotency-key storage.

This is not yet a live People product. It has no durable HR repository, no
production facade, no adapter parity suite, no installed-pack resolver, no
workflow/payroll/audit delivery, no recovery campaign, and no measured SLO.

</landed_scope>

<requirements>

## People records and organization

- Create a tenant- and legal-entity-scoped employee from validated identifiers,
  person reference, manager relation, employment status, evidence reference,
  and schema version.
- Preserve explicit lifecycle events with stable event and idempotency identity;
  reject unsafe, blank, cross-tenant, or malformed references before mutation.
- Evaluate onboarding readiness from mandatory checklist items and evidence,
  returning a stable blocker set rather than silently activating an employee.
- Expose create/read and later update/list behavior through one versioned HR
  facade derived from the same use-case contract, not from adapter internals.
- Serve that facade as binary-protobuf unary Connect with bounded request work,
  meaningful HTTP/Connect errors, and no gRPC service, envelope, or trailers.
  Do not dispatch the surface until an accepted generated Connect target owns
  the protocol envelope; HR must not substitute hand-written framing.

## Leave, payroll intent, and compliance

- Evaluate leave routing, balance, carryover, forfeiture, and payroll-impact
  intent without calculating gross-to-net payroll or calling Payroll in core.
- Evaluate jurisdictional employment obligations only from an admitted HR
  overlay; each decision carries pack/source revision, effective date, evidence,
  and idempotency identity.
- Deliver workflow and payroll intent through retryable ports. Duplicate
  delivery is safe, and delivery failure does not falsify the committed HR
  record.

## Privacy and authority

- Default-deny all facade calls until principal, tenant, action, resource, PDP
  provenance, and request binding are verified.
- Require purpose, legal basis, policy reference, and evidence for sensitive
  reads. Consent-based reads require consent evidence; general browsing is not
  an allowed purpose.
- Never return sensitive payload merely because a caller supplied an allow
  field. Audit/evidence requirements complete before disclosure.
- Route no production request through a test fake or trait-only composition.
  Installed-pack resolution, Policy/IAM authorization evidence, and Audit/
  outbox delivery each require an accepted provider contract and a production
  HR-owned adapter whose outage behavior fails closed for the operation class.
- Encrypt durable sensitive fields through a selected adapter and prevent
  secrets, credentials, raw sensitive values, or policy proofs from entering
  logs and metrics.

## Durability and portability

- Persist runtime state through HR-owned ports; no employee, install, or
  idempotency database lives in git.
- Ship a SQLite v1 adapter with atomic mutation/idempotency/outbox semantics and
  format migrations. A tenant uses one active adapter per port.
- Run the same behavioral and fault contract against the in-memory reference,
  SQLite, and each promoted commodity or Oyatie-cloud adapter.
- Keep app core free of database, network, IAM, Data, Storage, Gateway, and
  provider types. Adapter replacement changes configuration and translation,
  not employment rules.

## Operability

- Emit correlation-safe metrics for admitted/rejected calls, domain failures,
  authorization denials, transaction latency, commit/reopen failure, replay,
  outbox lag, adapter health, migration revision, and installed overlay
  generation without employee identifiers or sensitive values.
- Provide readiness only when the selected adapter is open, migrations are
  admitted, installed-pack authority is usable, and required policy/audit paths
  satisfy the operation's fail-closed contract.
- Bound queues and in-flight work; reject retryably before unbounded memory or
  lock contention. Background delivery may not starve foreground reads/writes.

</requirements>

<slo_objective>

## Promotion objective (not currently measured or achieved)

For an admitted home-cell tenant at no more than 70% declared capacity:

- monthly facade availability objective: **99.95%**, excluding requests
  correctly rejected for invalid or unavailable authority;
- p99 single-employee read: **100 ms** and p99 single-record command commit:
  **250 ms**, measured at the facade and excluding asynchronous downstream
  workflow/payroll delivery;
- acknowledged mutation durability: **zero loss** within the selected adapter's
  declared durability profile after process restart;
- idempotent replay: **100%** of same-key/same-digest retries return the original
  committed outcome without a second business effect;
- unauthorized sensitive disclosure objective: **zero**.

These become advertised SLOs only after HR-owned telemetry, load envelopes,
reopen tests, and promotion gates exist. Until then the product reports the
objectives as unqualified rather than manufacturing availability evidence.

</slo_objective>

<acceptance>

## Success

- A valid authorized onboarding command commits one employee, lifecycle event,
  idempotency outcome, and audit/outbox intent; reopening and replay return the
  same outcome.
- A selected SQLite or commodity/cloud adapter passes the identical contract,
  isolation, migration, and fault suite without changing core source.
- Policy, overlay, and downstream failures have typed outcomes and leave no
  ambiguous partial HR mutation or sensitive disclosure.
- SLO signals and readiness truthfully identify the selected adapter and policy
  generation.
- Cargo/Buck and byte-golden tests prove the sold People proto, accepted
  generated Connect service/runtime, request/success/error mapping, and absence
  of HR-written framing, gRPC framing, or trailers before any listener
  promotion.
- The production process composes concrete Packs/install, Policy/IAM evidence,
  Audit/outbox, SQLite, and generated-Connect adapters; each provider outage is
  proven before route activation and before a non-empty tenant cohort.
- Success and error responses, stored replay outcomes, returned strings, and
  repeated fields stay within exact hard ceilings under checked accounting;
  oversized state produces no partial response or sensitive fallback error.

## Failure

- An acknowledged employee mutation disappears or becomes partially visible
  after interruption or reopen.
- The same idempotency key creates two effects, or a different payload reuses it
  without conflict.
- A stale/forged PDP or overlay proof, cross-tenant body, missing legal basis,
  or audit outage reaches mutation or sensitive disclosure.
- HR core requires a Data/Gateway/cloud crate, adapter-specific schema, or
  trusted-tenant branch.
- A cloud IAM package imports any HR package, or an HR facade accepts gRPC,
  trailer-dependent, malformed, unbounded, second-codec, or HR-handwritten
  Connect traffic.
- A routed cohort uses an in-memory/test authority, omits a required production
  provider adapter, emits a partial oversized response, truncates a collection,
  or interpolates sensitive state into an error.
- A health endpoint claims durability, delivery, or SLO qualification absent
  corresponding evidence.

## Named fault campaigns

- Interrupt after transaction begin, idempotency reservation, employee write,
  lifecycle/audit-outbox write, and immediately before/after commit; reopen the
  same database after each point.
- Lose the successful response after commit and replay the request; then replay
  the key with a changed request digest.
- Inject corrupt/old schema versions, full disk, busy/locked database, expired
  pack/PDP proof, cross-tenant input, downstream timeout, and outbox redelivery.
- Kill the adapter during migration and prove it reopens at either the prior
  admitted version or the fully committed next version, never a hybrid.
- Inject malformed/truncated protobuf, a gRPC five-byte prefix, wrong Connect
  headers/content type, trailer metadata, request/response body and in-flight
  saturation, exact and limit-plus-one output/repeated-field/error sizes, and
  encode failure before headers; every case fails before repository mutation
  or partial response.
- Independently remove Packs/install, Policy/IAM, and Audit service reachability
  before process boot and during an admitted request; readiness drops, required
  operations fail closed, reservations drain, and no cohort is routed on a fake.

</acceptance>
