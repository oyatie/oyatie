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
- Derive replay semantic equality only by authenticated open and constant-time
  comparison of the HR-owned canonical-request format; derive its opaque lookup
  locator only from the logical idempotency slot. Bind commit authorization only
  to the HR-owned staged-write descriptor. A
transport/provider field order, unknown field, omitted effect, implicit
  default, or provider-local normalization MUST NOT change either protected
  preimage. The executable baseline writes CanonicalRequestV1 only. A new-format
  writer remains disabled until a separately accepted codec/lifecycle decision
  supplies active-writer authority, reader-cohort admission, repository migration,
  retirement evidence, and independent byte-oracle compatibility; this owner law
  does not claim V2 behavior.
- During normal rotation, resolve replay only through the record-encryption
  port's provider-authenticated repository/epoch/fence/membership-bound active-
  plus-draining generation set. Before one SQLite writer lookup, use every
  returned generation-scoped opaque authority to derive the V1 idempotency
  locator for exactly `(repository, tenant, operation kind, idempotency key)`.
  The locator excludes request schema, canonical format, and mutable canonical
  request bytes, changes with its generation, and is never cleartext or a stable
  cross-generation equality token. Thus the same logical key locates its row
  even when a semantic request field changes; the bounded lookup
  authenticates/decrypts the one located ciphertext envelope and constant-time
  compares canonical plaintext in memory to choose replay or
  `IdempotencyConflict`. It never uses randomized ciphertext equality or
  persists plaintext. Matrix/locator divergence, collision, stale lease, source
  loss, tampering, or provider loss fail closed without a reservation or second
  business effect.

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
  Globally for one keyring, normal G+2 activation is refused until G has no
  durable ciphertext or idempotency-locator reference (and no remaining
  non-replay field-index reference), no incomplete rekey or earlier
  unresolved authorization, and a provider retirement receipt marks G revoked.
  The retirement receipt is bound to the immutable keyring-membership snapshot
  captured with the rotation fence and includes every enrolled repository
  instance's terminal zero-reference receipt. A repository can enroll, remove,
  or rejoin only through provider-versioned CAS. The repository-port's
  `ProduceDecommissionProofV1` first durably closes a repository write-admission
  epoch, then CASes the matching provider decommission fence. Its fenced SQLite
  scan must authenticate a complete bounded checkpoint and zero ciphertext,
  locator, and non-replay field-index references across every live generation,
  plus zero unresolved authorizations, before the provider issues one explicit
  `Issued { issuance: DecommissionProofIssuanceV1 }` value. The provider-ledger
  issuance is immutable and response-loss Get/replay returns byte-identical
  issuance bytes, including the authenticated proof and reference; SQLite
  persists it and never mints a reference. Proof, reference, and issuance use
  canonical kinds `0x07`, `0x08`, and `0x09`; provider signing encodes the body
  without authenticator, signs that body, appends the authenticator to the final
  wire, and only then calculates an external proof/reference digest. Their
  maxima are respectively 1,805, 1,265, and 3,092 bytes. Missing, mismatch,
  corrupt, and authenticator-invalid outcomes are separately typed and fail
  closed.
  The exact proof and reference authentication literals are
  `hr.decommission.proof-authenticator.v1` and
  `hr.decommission.proof-reference-authenticator.v1`; their published external
  digest domains remain unchanged. Provider-operation bytes are fixed in the
  existing order: `RegisterKeyringRepository=0x01`,
  `BeginRepositoryDecommission=0x02`, `IssueDecommissionProof=0x03`,
  `AbortRepositoryDecommission=0x04`, `RemoveKeyringRepository=0x05`,
  `BeginKeyringRetirement=0x06`, `CompleteKeyringRetirement=0x07`, and
  `BeginNormalRotation=0x08`.

  The auxiliary canonical kinds add to, and never renumber, the published
  `0x01..=0x0c` map: `KeyringRetirementHandoff = 0x0d`,
  `KeyringRetirementFence = 0x0e`, `DecommissionRemovalReceipt = 0x0f`, and
  `KeyringRetirementReceipt = 0x10`. `0x00` and unassigned `0x11..=0xff` are
  invalid. The global decoder recognizes `0x01..=0x10`; plan decoders still
  accept only `0x01..=0x05` and distinguish known-wrong `0x06..=0x10`
  from unknown `0x00`/`0x11..=0xff` as separate typed refusals. Every authenticated
  record uses the existing 16-byte header and TLV rules: encode the body header
  and ascending non-authenticator tags, sign the literal-domain/zero/body
  preimage, append the authenticator last in a same-kind final wire, and only
  then derive an external digest. External digests are never tags. Stored and
  replayed authority is the exact final byte sequence, not a decoded
  reserialization.

  | kind | exact body tags | final tag | authentication / external digest domain | body/final maximum |
  | --- | --- | --- | --- | --- |
  | `KeyringRetirementHandoff = 0x0d` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 live_generation_digest`; `10 removal_plan_digest` | `11 authenticator` | `hr.decommission.keyring-retirement-handoff-authenticator.v1` / `hr.decommission.keyring-retirement-handoff.v1` | `926` / `1,441` |
  | `KeyringRetirementFence = 0x0e` | `1 exact final handoff wire`; `2 retirement_fence_id`; `3 retirement_begin_operation_id`; `4 begin_plan_digest` | `5 authenticator` | `hr.decommission.keyring-retirement-fence-authenticator.v1` / `hr.decommission.keyring-retirement-fence.v1` | `1,758` / `2,273` |
  | `DecommissionRemovalReceipt = 0x0f` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 prior_membership_snapshot_id`; `7 prior_membership_version`; `8 successor_membership_snapshot_id`; `9 successor_membership_version`; `10 removal_operation_id`; `11 removal_plan_digest` | `12 authenticator` | `hr.decommission.removal-receipt-authenticator.v1` / terminal domain below | `1,034` / `1,549` |
  | `KeyringRetirementReceipt = 0x10` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 retirement_fence_id`; `10 removal_plan_digest`; `11 retirement_begin_operation_id`; `12 retirement_complete_operation_id`; `13 all_generation_digest`; `14 scan_checkpoint_digest`; `15 durable_ciphertext_references`; `16 durable_locator_references`; `17 durable_non_replay_index_references`; `18 unresolved_authorizations`; `19 state` | `20 authenticator` | `hr.decommission.keyring-retirement-receipt-authenticator.v1` / terminal domain below | `1,404` / `1,919` |

  The four counters at tags 15 through 18 are exact `u64be(0)` and
  `Retired=0x01` is the sole accepted state byte. The
  `ProviderDecommissionTerminalReceiptV1` logical sum accepts only a final
  kind-`0x0f` `Removed` wire or a final kind-`0x10` `KeyringRetired` wire. It
  has no outer wire; the signed header kind is the variant discriminator. Both
  variants derive `provider_terminal_receipt_digest` only from the final bytes
  under `hr.decommission.provider-terminal-receipt.v1`.

  The fence, Begin plan, and Begin request nest exact final kind-`0x0d` bytes;
  the Complete plan nests exact final kind-`0x0e` bytes; disposition nests
  exact final kind-`0x0f` or `0x10` bytes. Wrong kind/header/schema, body/final
  confusion, terminal substitution, count/tag/order/length/domain/authenticator/
  digest mutation, invalid state byte, or nested reserialization is a typed
  refusal. The contract-freeze, provider, and repository lanes each own
  independent body/final min, max, and max-plus-one goldens for all four rows
  plus response-loss and fresh-process byte-identical replay vectors.

  The independently checked size ledger retains plan maxima
  `3,096/1,793/3,832/2,179/253`, request maxima
  `2,267/1,758/2,184/292/288`, storage receipt/binding `870/1,246`, the
  `4,096` ceiling, and `264` bytes of headroom. Both
  `KeyringMembershipError` and `RepositoryDecommissionRemovalError` expose the
  same named quartets `RetirementHandoff{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`,
  `KeyringRetirementFence{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`, and
  `ProviderTerminalReceipt{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`.
  Missing is absent retained state; Corrupt is structural wire/state failure;
  Mismatch is wrong canonical identity/parent/digest/status/terminal branch;
  AuthenticatorInvalid is a present envelope with an invalid key/signature.
  Fence-stale and retirement-precondition errors stay separate, and contract-
  freeze plus recovery evidence preserve the exact branches without a catch-
  all. Receipt fields and
  maxima do not expand: assurance for omitted history relies on retained
  provider state and byte-identical exact-operation replay, not a claim that
  omitted fields are directly authenticated.
  Binding tag 8 follows only from the verified terminal header: kind `0x0f`
  supplies `removal_operation_id` with provider-operation kind `0x05`, and kind
  `0x10` supplies `retirement_complete_operation_id` with kind `0x07`. A caller
  cannot select it; mismatch is typed. Since the retired receipt omits
  `begin_plan_digest`, `complete_plan_digest`, and `retirement_fence_digest`,
  verification composes the retained exact operation cells/plans and handoff/
  fence ledgers. Exact auxiliary final wires, cells/plans, proof issuance, and
  the provider-envelope-selected verification-key epoch survive Get/replay/
  recovery together until bounded atomic local-terminal GC; neither response
  nor caller chooses the epoch, and no linked record is collected early.
  Before its first provider removal side effect,
  `RemoveRepositoryDecommissionV1` writes a durable immutable pre-Remove plan
  that binds that proof reference/fence, `Quarantine` or `Delete` local
  disposition and manifest, preallocated retirement fence, and distinct scoped
  provider/local ids. Each plan has a fixed domain-separated canonical
  tagged-field preimage and digest that excludes itself and every subsequently
  derived request byte/digest; a sibling exact-request journal is atomically
  derived and persisted only after the plan digest, before that side effect.
  Its immutable header kinds are Removal `0x01`, Begin `0x02`, Complete `0x03`,
  LocalDisposition `0x04`, and LocalCompletion `0x05`; a kind substitution,
  receipt kind `0x06`, or every other byte is rejected before digest, journal,
  or side effect. Fixed port/key-service/memory/SQLite vectors cover all five
  plan/request kinds, min/max/+1, and kind/tag/parent/id mutation.
  The plan/journal pair is the sole source of values on retry or recovery; a
  changed id reuse is a typed `MembershipOperationConflict`, not a new attempt.
  A sole-member Remove
  returns provider `RetirementHandoffReady`, an authenticated, queryable handoff;
  SQLite first durably
  stores that handoff, then a separate immutable Begin plan and exact Begin
  journal containing the handoff bytes/authenticator, Begin id, fence id, and
  already-derived Begin request digest before Begin. The typed Begin request,
  provider idempotency record, and returned signed retirement fence bind that
  same `begin_plan_digest`; a changed reuse is conflict. It persists corresponding
  post-Begin Complete, post-terminal disposition, and post-storage completion
  plan/journal pairs before the later local side effects.

  The earlier durable decommission intent likewise preallocates distinct Begin,
  Issue-proof, and abort provider-operation ids before the Begin call. Its
  terminal fenced scan atomically records a `ProofIssuePlanned` observation and
  exact Issue request digest before Issue-proof. Abort and recovery response ids
  identify only their local response records; they cannot select, replace, or
  become any provider side-effect id.
  Provider side-effect ids are kind-scoped, and their canonical digest binds the
  kind plus its operation-specific authority tuple. The implementation baseline freezes named,
  exhaustive provider status/Abort/membership-mutation results and repository
  Abort/Remove/Complete results; every provider status/error is explicitly
  matched by port, adapter, and SQLite tests. `DecommissionObservationStale` is
  a provider error and stays that exact typed error in proof/removal paths.
  The provider retains the proof and its authenticated immutable
  `DecommissionProofReferenceV1` as one ledger issuance through exact-operation
  replay and terminal-receipt GC; Remove and Complete resolve that bounded
  reference and reject named Missing, Mismatch, Corrupt, or
  AuthenticatorInvalid variants without a membership transition. The 2,273-byte signed Begin fence carries the plan
  digest, making the dependent Complete-plan maximum 3,832 bytes. Provider Begin atomically commits
  its idempotency cell, Decommissioning member state, and signed `Fenced` value,
  so provider status has no `IntentPending` variant. SQLite `IntentPending` is
  exclusively a write-closed local pre-Begin state: Get/Begin response loss
  converges to `NotStarted`, signed `Aborted`, or signed closed status, with
  only the persisted Abort CAS permitted from `NotStarted`.

  The repository then uses typed `AbortRepositoryDecommissionIntentV1`,
  `RemoveRepositoryDecommissionV1`, `CompleteRepositoryDecommissionV1`,
  `GetRepositoryDecommissionStatusV1`, and `RecoverRepositoryDecommissionV1`
  operations. Provider removal atomically rechecks proof, plan digest,
  snapshot/version, repository/member instance/epoch, and admission fence and
  returns a signed proof-and-plan-bound receipt. The local state is observable
  as proof-issue-planned, planned, handoff-persisted, Begin-planned, `Retiring`,
  Complete-planned, provider-terminal-pending-disposition,
  disposition-planned/in-progress/applied, completion-planned, or receipt-
  carrying terminal state. `CompleteRepositoryDecommissionV1` consumes only the
  stored plans and terminal receipt, applies the bound local disposition, then
  records `Removed` or `KeyringRetired` by matching atomic completion CAS. A lost response, crash,
  partition, or local drain/delete/quarantine failure is a plan/receipt-bound
  recovery state with readiness withdrawn, never a path that can omit a
  referenced member, invent an id/disposition, or reopen its old epoch.
  `LocalDecommissionStorageReceiptV1` remains a separately retained 870-byte
  SQLite receipt addressed by its canonical key and 32-byte external digest;
  completion and recovery never use its sole `storage_receipt_digest` plan field
  until they have also resolved the trusted 1,246-byte
  `LocalDecommissionStorageReceiptBindingV1`. In the same atomic transition that
  publishes `LocalDispositionApplied`, that binding commits the receipt lookup
  key, expected digest, all repository/keyring/member/epoch and parent fields,
  terminal/local operation ids, disposition/manifest, admission epoch, and
  key-id/epoch plus detached metadata-commit authenticator. The signer encodes
  its body before appending that authenticator, is rotation-aware, and fails
  closed on unavailability, unknown key epoch, or invalid signature. Fresh
  recovery verifies receipt bytes/digest and binding before it derives the
  253-byte plan; missing, duplicate, corrupt, changed, or unauthenticated
  receipt/binding inputs are `LocalDispositionReceiptInvalid` and keep the
  member closed.
  Recovery replays only plan-bound steps after pre-Remove plan, provider
  handoff, handoff persistence, Begin-plan, Begin, `Retiring`, Complete-plan,
  Complete, provider terminal receipt, disposition-plan/disposition,
  completion-plan/completion. A delayed begin cannot resurrect after abort because the provider
  atomically persists a begin-operation tombstone and SQLite CASes a strictly
  greater reopened admission epoch before local admission reopens. Recovery
  treats provider `NotStarted` for a persisted intent only as input to that
  stored Begin/abort-tuple tombstone CAS; it never opens from the observation
  alone or uses a recovery-response id for the provider abort.
  Removing a sole member returns a typed, provider-queryable retirement handoff rather than
  an empty snapshot; provider retirement fences all writers, reports
  `Retiring`, verifies the proof's all-generation zero ciphertext/locator/
  non-replay-index checkpoint before revocation, and ends in a separate
  `Retired` keyring state with no rejoin. Removal is refused during a
  drain. Emergency drain, source loss, or partition withdraws readiness and
  blocks normal rotation; it is never a route around this invariant.
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
  required policy/audit paths satisfy the operation's fail-closed contract. A
  required-authority outage is a correct fail-closed refusal but an availability
  failure for eligible traffic until recovery or acknowledged cohort withdrawal;
  a readiness flip alone does not erase that budget burn.
- Bound one rekey page and one reconciler step by item count, ciphertext bytes,
  provider calls, pages, and CAS attempts. A new cohort is unready while a
  rotation job is incomplete; an already-routed cohort may continue normal
  reads during a healthy normal drain only while the source generation remains
  readable, new writes use the target generation, and durable progress remains
  within its declared SLO. Emergency drain or a non-progressing/corrupt job
  withdraws the affected cohort.
- Bound the executable V1 replay path to two generations, one format (V1) per
  generation, two returned-authority idempotency-locator derivations, five
  locator-row reads, and one authenticated open. A future format lifecycle may
  add separately bounded reader work but does not add locator derivations, because
  the locator is format-independent; no V2/V3 writer is admitted by this plan.
- Bound queues and in-flight work; reject retryably before unbounded memory or
  lock contention. Background delivery may not starve foreground reads/writes.
- Evaluate expiry/effective-window boundaries from a trusted interval. If the
  interval straddles a policy, overlay, key-authorization, or legal-effective
  boundary, refuse rather than selecting a favorable point estimate.

</requirements>

<slo_objective>

## Promotion objective (not currently measured or achieved)

For an admitted home-cell tenant at no more than 70% declared capacity:

- monthly facade availability objective: **99.95%**. Its denominator is every
  syntactically valid, capacity-admitted facade request unless an available
  authority determines the request is caller-caused validation,
  unauthenticated, or forbidden traffic; its numerator is those requests that
  complete successfully within the declared operation objective. Required Packs,
  Policy/IAM/PDP, Audit, encryption/key-service, durable-adapter, or runtime-
  context authority unavailability counts as an eligible failure and consumes
  error budget. It is never retroactively excluded after recovery. The burn
  interval ends only at provider recovery or an observed router acknowledgement
  that withdrew the affected cohort, not when readiness merely changes;
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
  generation. They expose the eligible-request denominator, successful numerator,
  required-authority failures, error-budget burn, readiness transition, and
  router-withdrawal acknowledgement without sensitive labels; an authority
  failure cannot be classified as excluded availability traffic.
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
  declared durability contract. The generation-scoped opaque idempotency locator
  only locates the logical key slot; equality authenticates/decrypts its
  ciphertext then constant-time compares matching canonical plaintext in memory.
  Ciphertext equality is forbidden, and a changed request under the same key
  conflicts rather than reserving a second effect.
  Fresh boot fences the prior repository epoch and resolves every bounded
  provider-side pending receipt before readiness.
- Canonical-request and staged-write-descriptor goldens are identical through
Cargo and Buck and across schema N/N+1 readers while the executable canonical
format remains V1. A same semantic request with reordered
transport fields or explicit/default-equivalent optionals replays; a changed
semantic field conflicts. Every committed employee, lifecycle,
idempotency/outcome, and audit/outbox effect appears exactly once in the
authenticated staged descriptor.
- Independent Cargo and Buck encoders assert exact complete idempotency-locator
  and commit-binding preimages, including domains, purpose, component count,
  tags, lengths, fixed widths, omission/reordering, and exact/limit-plus-one
  bounds. Replay scheduled before/during/after page CAS, response loss, hard
  close, source drain/loss, revocation, and schema N/N+1 restart returns the
  original outcome or a typed refusal; a changed request under the same logical
  key returns conflict and never commits a second effect.
- A normal rotation hard-closes at every scan/open/seal/reindex/page-CAS/
  checkpoint/zero-count/revoke boundary and a fresh process deterministically
  resumes from the last committed page. Old-generation ciphertext and
  idempotency-locator references (and any non-replay field-index references)
  reach zero before the provider reports `Revoked`; page and
  whole-step work remain inside the declared hard limits.
- Keyring membership evidence registers two repositories, freezes their exact
  membership instances in a G-to-G+1 fence, refuses a concurrent
  enroll/remove/rejoin or G+2 request, and permits revocation only after both
  snapshot-bound terminal zero-reference receipts and provider unresolved counts
  reach zero. Decommission evidence persists an intent and write fence before
  provider fencing; persists a known-only pre-Remove plan; races authorization/
  commit/proof/plan/provider-removal/provider-handoff/handoff-persistence/
  Begin-plan/Begin/`Retiring`/Complete-plan/Complete/terminal/
  disposition-plan/disposition/completion-plan/local-completion/recovery at every
  boundary; and proves that an
  observed, provider-removed, or locally terminal member cannot commit a durable
  reference. A lost response or local storage drain/delete/quarantine fault
  resumes only from the signed proof-and-plan-bound terminal receipt and stored
  exact plan/id for the next step. It exercises the begin-tombstone race so a delayed begin
  cannot resurrect after abort. A sole-member removal returns a typed retirement
  handoff, persists an exact Begin plan, and reaches a no-member `Retired` state only after every generation is
  revoked with all-zero ciphertext/locator/non-replay-index/unresolved evidence. A
  partitioned or omitted repository has no receipt and therefore cannot be
  removed or let G advance to G+2.
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
- A decommission proof is produced without a current provider/local admission
  fence, does not bind the repository/member/epoch/snapshot/rotation state and
  terminal write sequence, permits a durable write after its observation,
  provider removal, or local terminal completion, treats a bare `Removed` as
  recoverable truth, or reopens after a terminal-local-storage fault.
- A health endpoint claims durability, delivery, or SLO qualification absent
  corresponding evidence.
- An eligible request is removed from the availability denominator because a
  mandatory provider was unavailable, because readiness changed without a router
  withdrawal acknowledgement, or after the provider later recovers.

## Named fault campaigns

- Interrupt after transaction begin, idempotency reservation, employee write,
  lifecycle/audit-outbox write, and immediately before/after commit; reopen the
  same database after each point.
- Lose the successful response after commit and replay the request; then replay
  the same logical key with a changed canonical request under active-only,
  active-plus-draining, post-page-rekey, hard-close, and fresh-process schedules.
  Every schedule derives both bounded generation locators before lookup and
  proves exactly one stored effect: same plaintext replays and changed plaintext
  conflicts.
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
  fallback. For each outage, establish syntactically valid capacity-admitted
  traffic, assert its typed refusal has no mutation/disclosure, increments the
  availability denominator and required-authority-failure/error-budget counters,
  then prove the burn ends only on recovery or an observed router-withdrawal
  acknowledgement. Inject interval uncertainty that straddles every effective/
  expiry boundary and require the stable refusal.
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
- Exercise canonical-request and descriptor V1 byte goldens with transport-field
  reordering, absent versus explicit defaults, changed fields, same-version
  unknowns, descriptor effect omission/duplication/reordering, unadmitted-format
  refusal, and every exact/limit-plus-one byte/count ceiling. During rotation inject a
  stale repository epoch, stale/corrupt/non-progressing cursor, page CAS race,
  retry exhaustion, source/target-key loss, database busy/full, provider outage,
  crash after page commit and after provider revoke, and a terminal nonzero
  reference; fresh-process recovery either advances one durable checkpoint or
  returns the specified typed refusal without acknowledgement or revocation.
- Register duplicate/missing repositories, race enrollment/removal/rotation,
  partition one enrolled repository, retry after crash, and attempt a rejoin with
  a stale membership version or decommission proof. For decommission, race an
  authorization immediately before and after the durable intent, provider fence,
  each bounded SQLite ciphertext/locator/non-replay-index scan page, terminal
  zero observation, proof issuance, pre-provider removal-plan persistence,
  provider removal, provider retirement handoff, handoff persistence,
  post-handoff Begin-plan persistence, Begin, `Retiring`, post-Begin
  Complete-plan persistence, Complete, provider terminal receipt,
  post-terminal disposition-plan persistence, local drain/disposition,
  post-storage completion-plan persistence, local completion, and recovery;
  inject response loss before/after each provider/local boundary, provider
  partition, process crash, database busy/full/I/O/commit/quarantine/delete
  faults, stale live-generation digest, concurrent rotation, plan/receipt/count
  mismatch, and changed-operation-id replay.
  Exercise `NotStarted` plus abort-tombstone plus delayed-begin delivery and
  prove the delayed begin cannot mutate membership after local reopen. For all
  five plan kinds, freeze minimum and maximum canonical byte/digest vectors,
  assert the exact tag order/domain/u16 accounting, mutate every field/id/parent
  and reject max-plus-one, then compare an independent encoder against the
  port, key-adapter, and SQLite encoders. Crash before/after plan persistence,
  journal persistence, and the side effect; recovery must replay only the
  matching journal. Exercise provider Begin/Abort serialization and response
  loss in a fresh process: no provider IntentPending may be returned, and local
  IntentPending may only abort a persisted NotStarted tuple or install the exact
  signed Fenced/Aborted/closed status. Exercise
  both a multi-member removal and a sole-member typed retirement handoff, then
  fail its all-generation-zero ciphertext/locator/non-replay-index reference or
  unresolved preconditions. A resumed operation either returns its identical
  plan-and-proof-bound receipt/result or a typed refusal; the old member remains
  write-closed and cannot rejoin until a fresh registration on a new active
  keyring. Each schedule preserves the frozen fence
  snapshot; a zero-reference, unresolved-authorization, decommission, removal,
  local-completion, or retirement receipt from another snapshot, member instance,
  generation set, fence/epoch, or unresolved count is rejected.
- First accept standalone Cargo/Buck key-adapter tests for real envelope
  open/seal, authorization/resolution, and decommission-fence behavior. Only
  then execute the dev-only key-service composition target against a real SQLite
  file and the accepted provider contract server. It must exercise
  `AcquireReplayGenerationSetV1`, returned-authority locator derivation,
  authenticated open/tamper behavior, decommission-versus-durable-write,
  typed stale/replayed/changed-request/provider-loss outcomes, and the
  reverse-edge scan that permits repository/SQLite dependencies only in that
  test target.

</acceptance>
