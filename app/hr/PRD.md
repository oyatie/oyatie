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
  Trusted interval time and correlation-safe telemetry likewise require the
  production `hr-runtime-context-oyatie-draft` adapter implementing
  `hr-runtime-context-draft`; a process/system clock or facade-local emitter is
  not a production implementation.
- Encrypt durable sensitive fields through an HR-owned record-encryption port
  and an accepted authenticated-encryption/key-service adapter. Persist no
  employee, person, evidence, lifecycle, request, outcome, or outbox plaintext;
  persist no unkeyed canonical-request fingerprint; and prevent secrets, keys,
  credentials, raw sensitive values, or policy proofs from entering database
  pages, backups, logs, and metrics. A mutation is acknowledged only after its
  SQLite commit authorization is linearly ordered with key-generation
  rotation/revocation and resolved committed by the key authority.
- Derive replay equality only from the HR-owned canonical-request format and
bind commit authorization only to the HR-owned staged-write descriptor. A
transport/provider field order, unknown field, omitted effect, implicit
default, or provider-local normalization MUST NOT change either protected
preimage. New format writers remain disabled until every cohort reader admits
N/N+1 and stored byte-golden compatibility.
- During normal rotation, resolve replay only through a provider-authenticated
  set of at most active plus immediately prior draining generations; derive all
  generation-scoped candidates before one SQLite writer lookup, compare only
  encrypted canonical bytes, and fail closed on collision, stale lease, source
  loss, or provider loss. This preserves original-outcome replay without a
  stable cross-generation equality token or a second business effect.

## Durability and portability

- Persist runtime state through HR-owned ports; no employee, install, or
  idempotency database lives in git.
- Ship a SQLite v1 adapter with atomic mutation/idempotency/outbox semantics and
  format migrations. It requires the selected record-encryption port before it
  opens a production database. A tenant uses one active adapter per port.
- Complete normal key rotation through a bounded, durable repository rekey job:
  scan old-generation references, re-encrypt and generation-reindex them with
  compare-and-swap, checkpoint each committed page, resume after a hard close,
  and prove zero references plus zero unresolved earlier authorizations before
  revocation. Provider or repository outage, stale epoch/cursor, exhausted CAS
  contention, corrupt envelope, missing key, and nonzero references are typed
  fail-closed states, never reasons to skip a row or fall back to plaintext.
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
  admitted, installed-pack and encryption/key authorities are usable, and
  required policy/audit paths satisfy the operation's fail-closed contract.
- Bound one rekey page and one reconciler step by item count, ciphertext bytes,
  provider calls, pages, and CAS attempts. A new cohort is unready while a
  rotation job is incomplete; an already-routed cohort may continue normal
  reads during a healthy normal drain only while the source generation remains
  readable, new writes use the target generation, and durable progress remains
  within its declared SLO. Emergency drain or a non-progressing/corrupt job
  withdraws the affected cohort.
- Bound queues and in-flight work; reject retryably before unbounded memory or
  lock contention. Background delivery may not starve foreground reads/writes.
- Evaluate expiry/effective-window boundaries from a trusted interval. If the
  interval straddles a policy, overlay, key-authorization, or legal-effective
  boundary, refuse rather than selecting a favorable point estimate.

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
- idempotent replay: **100%** of same-key/same-canonical-request retries return
  the original committed outcome without a second business effect;
- while a healthy normal rotation has work, p99 one bounded rekey step is
  **5 seconds** and the durable-checkpoint age is at most **60 seconds**; no
  step exceeds its item/byte/page/provider-call/CAS ceilings and foreground
  read/commit objectives remain satisfied;
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
  Audit/outbox, authenticated record-encryption/key service, SQLite, and
  generated-Connect adapters plus the accepted trusted-time/telemetry runtime-
  context adapter; each provider outage is proven before route activation and
  before a non-empty tenant cohort.
- Real-file and backup inspection finds none of the injected sensitive
  sentinels; fresh-process reopen, key rotation/re-encryption, planned and
  emergency revocation, commit-authorization recovery, and replay preserve the
  declared durability contract. Canonical-request equality is represented only
  by its tenant/key-scoped blind index or authenticated ciphertext. Fresh boot
  fences the prior repository epoch and resolves every bounded provider-side
  pending receipt before readiness.
- Canonical-request and staged-write-descriptor goldens are identical through
Cargo and Buck and across N/N+1 readers. A same semantic request with reordered
transport fields or explicit/default-equivalent optionals replays; a changed
semantic field conflicts. Every committed employee, lifecycle,
idempotency/outcome, and audit/outbox effect appears exactly once in the
authenticated staged descriptor.
- Independent Cargo and Buck encoders assert exact complete blind-index and
  commit-binding preimages, including domains, purpose, component count, tags,
  lengths, fixed widths, omission/reordering, and exact/limit-plus-one bounds.
  Replay scheduled before/during/after page CAS, response loss, hard close,
  source drain/loss, revocation, and N/N+1 restart returns the original outcome
  or a typed refusal and never commits a second effect.
- A normal rotation hard-closes at every scan/open/seal/reindex/page-CAS/
  checkpoint/zero-count/revoke boundary and a fresh process deterministically
  resumes from the last committed page. Old-generation ciphertext and blind-
  index references reach zero before the provider reports `Revoked`; page and
  whole-step work remain inside the declared hard limits.
- Success and error responses, stored replay outcomes, returned strings, and
  repeated fields stay within exact hard ceilings under checked accounting;
  oversized state produces no partial response or sensitive fallback error.

## Failure

- An acknowledged employee mutation disappears or becomes partially visible
  after interruption or reopen.
- The same idempotency key creates two effects, or a different canonical request
  reuses it without conflict.
- A stale/forged PDP or overlay proof, cross-tenant body, missing legal basis,
  or audit outage reaches mutation or sensitive disclosure.
- HR core requires a Data/Gateway/cloud crate, adapter-specific schema, or
  trusted-tenant branch.
- A cloud IAM package imports any HR package, or an HR facade accepts gRPC,
  trailer-dependent, malformed, unbounded, second-codec, or HR-handwritten
  Connect traffic.
- A routed cohort uses an in-memory/test authority, omits a required production
  provider adapter—including runtime context—emits a partial oversized response,
  truncates a collection, or interpolates sensitive state into an error.
- SQLite or its backup contains a sensitive plaintext sentinel, reuses a nonce,
  accepts a caller/process-local fallback key, acknowledges with an unsealed
  field or unresolved commit authorization, persists an unkeyed request
  fingerprint, declares revocation complete with an outstanding earlier
  authorization, or stays ready after its required key generation is
  unavailable/revoked or its trusted runtime context is unavailable.
- Two admitted binaries produce different protected preimages for one semantic
  request, accept an unknown same-version field, omit/reorder a staged effect
  without a commit-binding mismatch, skip a rekey row, advance a checkpoint
  after a failed page CAS, loop without a work bound, or revoke with any old-
  generation repository reference.
- A health endpoint claims durability, delivery, or SLO qualification absent
  corresponding evidence.

## Named fault campaigns

- Interrupt after transaction begin, idempotency reservation, employee write,
  lifecycle/audit-outbox write, and immediately before/after commit; reopen the
  same database after each point.
- Lose the successful response after commit and replay the request; then replay
  the key with a changed canonical request.
- Inject corrupt/old schema versions, full disk, busy/locked database, expired
  pack/PDP proof, cross-tenant input, downstream timeout, and outbox redelivery.
- Kill the adapter during migration and prove it reopens at either the prior
  admitted version or the fully committed next version, never a hybrid.
- Inject malformed/truncated protobuf, a gRPC five-byte prefix, wrong Connect
  headers/content type, trailer metadata, request/response body and in-flight
  saturation, exact and limit-plus-one output/repeated-field/error sizes, and
  encode failure before headers; every case fails before repository mutation
  or partial response.
- Independently remove Packs/install, Policy/IAM, Audit, and encryption/key-
  service reachability plus trusted time and telemetry before process boot and
  during an admitted request; readiness drops, required operations fail closed,
  reservations drain, and no cohort is routed on a fake or system-clock
  fallback. Inject interval uncertainty that straddles every effective/expiry
  boundary and require the stable refusal.
- Crash before and after ciphertext persistence and each key-rotation CAS;
  reopen from the same database and backup with a fresh process, replay the
  request, race provider `authorize_commit` and idempotent `resolve_commit`
  against SQLite commit, normal rotation, emergency drain, and crash recovery,
  revoke the old generation at zero and nonzero reference/authorization counts,
  inject a stale recovery epoch plus duplicate/missing/reordered and exact/
  limit-plus-one unresolved-receipt pages, and prove either one authenticated
  value or a typed fail-closed result—never plaintext, an unkeyed equality token,
  mixed generations without metadata, a completed revocation with pending
  authorization, or an implicit fallback key.
- Exercise canonical-request and descriptor byte goldens with transport-field
  reordering, absent versus explicit defaults, changed fields, same-version
  unknowns, descriptor effect omission/duplication/reordering, N/N+1 readers,
  and every exact/limit-plus-one byte/count ceiling. During rotation inject a
  stale repository epoch, stale/corrupt/non-progressing cursor, page CAS race,
  retry exhaustion, source/target-key loss, database busy/full, provider outage,
  crash after page commit and after provider revoke, and a terminal nonzero
  reference; fresh-process recovery either advances one durable checkpoint or
  returns the specified typed refusal without acknowledgement or revocation.

</acceptance>
