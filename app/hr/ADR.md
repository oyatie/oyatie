---
doc_class: Owner-ADR
owner: app/hr
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# HR decisions in force

This file specializes ADR-0719 D-23 through D-25 for `app/hr/`. It describes
the destination and the migration constraints; it does not claim that the
destination implementation has landed.

<current_state>

## Evidence at L2a

| Surface | What exists | Maturity |
|---|---|---|
| `core/employment-domain` | Typed employee and legal-entity identities; employment lifecycle; Korea-first labor thresholds; leave/payroll-impact, balance, and carryover projections; sensitive-read policy; onboarding readiness; statutory rulepack manifests | Tested domain foundation, but its 1,600-line source plus 440-, 383-, and 360-line tests exceed the budget, and its direct `data-boundary-kernel` dependency violates the target |
| `facade/employment-app` | Pure functions compose onboarding outcomes plus audit, workflow, payroll-impact, and sensitive-read envelopes | In-process composition only; no transaction boundary or durable acknowledgement |
| `ports/employment-api` | Serde request/response DTOs and conversions to domain/facade inputs | A 484-line JSON-shaped compatibility surface, not a sold versioned facade; it depends inward on both facade and domain |
| `adapters/employment-storage-inmemory` | Volatile record metadata keyed by idempotency key, with reserve/put/get/list behavior | Reference fixture only; the storage trait is incorrectly owned by this adapter, reservation and write are not one durable transaction, and restart loses all state |
| `adapters/employment-infrastructure` | In-process HTTP route, bearer verification, tenant-match, PDP-decision, and health behavior | Its 448-line authorization source, 372-line crate root, and 512-line runtime test exceed the budget; it imports Gateway core/runtime crates directly and has no deployment or network-listener promotion evidence |
| SLO and operations | Capability flags honestly report that no durable backend, workflow call, payroll call, sensitive retrieval, or audit-chain emission is attached | No HR-owned SLO source, persistent recovery proof, or production serving evidence has landed |

The current package and public type names remain compatibility facts until a
versioned structural migration changes them. The direct Data/Gateway edges and
over-budget files are debt, not precedent.

</current_state>

<product_boundary>

## Decision: HR owns people and employment rules, not substrate engines

- **achieves:** one portable owner for employee/employment behavior while
  payroll, workflow, audit, identity, storage, and transport retain one truth.
- **origin:** the landed slice creates correct HR projections but also carries
  cross-product envelopes and runtime adapters that can be mistaken for owning
  the downstream engines.
- **rule:** `app/hr/` MUST own employee and employment records, onboarding
  readiness, legal-entity organization relations, leave decisions and balances,
  labor-compliance detection, sensitive-HR disclosure policy, and HR evidence
  references. It MUST NOT execute gross-to-net payroll, workflow, audit-chain,
  IAM/PDP, relational storage, blob storage, gateway, or notification engines.
- **ensure:** cross-owner effects leave HR through typed app-owned ports; HR
  records contain references and intent, not a copied downstream ledger or
  engine; dependency and review gates reject foreign core imports.
- **overturn_when:** a founder-accepted owner-boundary decision reallocates a
  named behavior and amends both owners in the same change.

</product_boundary>

<portable_architecture>

## Decision: portable core, app-owned ports, replaceable adapters

- **achieves:** HR keeps working when an Oyatie cloud SKU is unavailable or
  retired, without rewriting business rules.
- **origin:** ADR-0719 D-23/D-25 makes apps tenants of sold cloud facades; the
  current HR core/facade imports Data core and the transport adapter imports
  Gateway core/runtime packages directly.
- **rule:** HR business rules and use cases MUST live in portable `core/` crates;
  every I/O or between-app interaction MUST cross a trait owned under
  `app/hr/ports/`; `adapters/` MUST translate those ports to SQLite, commodity,
  or sold Oyatie facades; `facade/` MUST expose the stable application API. HR
  core MUST NOT import SQLite, HTTP, IAM, Storage, Data, Gateway, or another app
  core/port, and no trusted-tenant or in-process cloud shortcut may exist. The
  reverse boundary is equally strict: no cloud capability package, including
  IAM, may import an `app/hr` package; a sold HR facade is a network contract,
  not permission to hide a cloud-to-app edge behind an HR client crate.
- **ensure:** the same parameterized HR contract suite runs against the
  in-memory reference, SQLite v1, and each promoted commodity/cloud adapter;
  dependency tests reject `app/hr/core` foreign-engine edges and app-to-cloud
  core/port edges; first-party calls use ordinary principals and the sold
  facade.
- **overturn_when:** a five-field architecture decision proves a narrower
  boundary preserves portability, default-deny policy, and adapter parity with
  less coupling.

</portable_architecture>

<sold_people_boundary>

## Decision: one semantic protobuf contract through generated Connect

- **achieves:** the sold People surface uses the platform's one generated
  Connect contract rather than an HR-specific parser, envelope, or second SDK.
- **origin:** the reviewed workspace has protobuf and tonic tooling but no
  accepted Connect generator/runtime target. Message-only `prost-build` plus
  handwritten request, response, and error framing would violate ADR-0719 D-4
  even if byte examples happened to resemble Connect.
- **rule:** the reserved People IDL identity is
  `app/hr/facade/proto/hr/people/v1/people_service.proto`, package
  `hr.people.v1`, with unary `OnboardEmployee` and `GetEmployee` methods.
  The IDL, codegen packages, and behavior are non-dispatchable until L2f.0a
  records an architecture/Build/protocol-owner accepted Connect generator and
  runtime with exact Cargo/Buck targets, versions/features, generated-output
  contract, dependency/removal policy, and fault suite. After that gate, the
  owner-local `hr-transport-draft` port is implemented by the mechanically
  matching provider adapter `hr-transport-connect-draft`, and `hr-people-app`
  is the only sold HR facade process. HR code consumes generated service
  bindings; it MUST NOT handwrite Connect HTTP parsing, protobuf framing,
  Connect JSON error framing, or trailer behavior. It also MUST NOT generate or
  link a tonic client/server, accept a gRPC content type, emit `grpc-status` or
  `grpc-message`, use HTTP trailers, or advertise streaming. No other owner may
  import either draft crate; a future Rust consumer requires a separate D-28
  promotion and a consumer-owned client adapter.
- **ensure:** the L2f gate fails closed while no accepted target exists. Its
  acceptance receipt and amended plan prove generated Connect service symbols,
  Cargo/Buck input/output parity, no tonic/gRPC runtime, and byte/fault vectors
  for exact paths, bare protobuf success, bounded success/error output,
  generated Connect errors, malformed protobuf, framing, request/response
  saturation, cancellation, and trailer rejection.
- **overturn_when:** an accepted protocol decision replaces ADR-0719 D-4 and a
  same-wave migration preserves one IDL, equivalent bounded wire evidence, and
  no standing second protocol.

</sold_people_boundary>

<runtime_state>

## Decision: SQLite v1 is durable state, never the domain model

- **achieves:** a self-contained v1 with crash-safe HR records and a direct path
  to Data, Postgres, or on-premises adapters.
- **origin:** ADR-0719 D-24 forbids runtime state in git and requires ports plus
  SQLite v1; the current in-memory map loses acknowledged records on restart.
- **rule:** each durable HR port MUST have a SQLite v1 adapter whose schema is
  private to that adapter. One tenant selects one active adapter per port; HR
  MUST NOT dual-write. A mutation, its idempotency outcome, and its durable
  audit/outbox intent MUST commit atomically before acknowledgement.
- **ensure:** backend conformance interrupts every transaction boundary,
  reopens the database, and replays the same idempotency key; pre-commit faults
  expose no mutation, post-commit reply loss returns the stored result, and the
  same key with a different canonical request fails closed.
- **overturn_when:** a replacement local adapter demonstrates equal offline
  packaging, atomicity, recovery, migration, and commodity/cloud parity, and a
  same-wave decision records its exit path.

</runtime_state>

<data_at_rest>

## Decision: sensitive HR records are encrypted before persistence

- **achieves:** a copied SQLite file, backup, or page image does not disclose
  employee, person, evidence, lifecycle, request, outcome, or outbox payloads,
  while HR remains portable across key providers.
- **origin:** the PRD requires encrypted durable sensitive fields, but the
  initial SQLite plan paired the repository with an unkeyed SHA-256 request
  fingerprint and only three encryption operations. That shape leaks equality
  across sensitive canonical requests and cannot order a separate SQLite
  commit against provider rotation or revocation.
- **rule:** SQLite and every promoted durable repository MUST pass sensitive
  values through the owner-local `hr-record-encryption-draft` port before
  persistence. The port owns bounded ciphertext-envelope, generation-scoped
  opaque idempotency-locator, key-generation, associated-data, opaque
  commit-binding, provider-serialized commit-authorization, replay-generation
  authority, decommission-admission-fence, and keyring-membership values; it
  does not own a cipher or KMS. Replay lookup MUST persist only the keyed
  locator for `(repository, tenant, operation, idempotency key, generation)`;
  the locator excludes schema, format, and canonical-request bytes, and an
  unkeyed digest is forbidden. Locator derivation MUST consume only an opaque
  generation authority returned by the provider-authenticated replay-generation
  set; a repository MUST NOT choose a raw generation, format, or PRF input.
  Canonical-request equality remains only inside the authenticated ciphertext:
  after slot lookup the repository opens that one row and constant-time compares
  its canonical plaintext. The canonical request and staged-write descriptor are
  HR-owned, versioned byte contracts; a key provider authenticates those bytes
  but MUST NOT choose, reorder, omit, or normalize their semantic fields.
  Production use is non-dispatchable until
  L2i.0d accepts one authenticated-encryption
  implementation and one commodity or sold key-service facade with exact
  dependencies, key custody, nonce, rotation, revocation, outage, exact
  `AcquireReplayGenerationSetV1`/idempotency-locator-derivation semantics,
  exact decommission fence/proof/removal semantics, and
  `authorize_commit`/idempotent `resolve_commit` semantics, L2i.0d.1 first
  admits the key-service adapter graph, L2i.0f prepares the closed protocol/
  repository/SQLite/adapter file set, L2i.0g.0 freezes the HR port contracts,
  L2i.0g.1 implements selected replay/membership/decommission provider
  behavior, L2i.0g.1a implements and reviews the minimal concrete open/seal,
  commit-authorization/resolution, and decommission-fence behavior, L2i.0g.2
  only then implements repository/SQLite behavior and its real composition
  target, and L2i.0h implements bounded
  repository rekey/recovery. Provider authorization, repository-
  epoch fencing, keyring repository enrollment/removal, and generation
  transitions MUST share one linearizable order: a transition denies new
  authorizations immediately but cannot become `Revoked` until every earlier
  authorization is durably resolved as committed or aborted. SQLite stores the
  opaque authorization receipt in the same transaction and no mutation is
  acknowledged before the provider resolves its committed receipt. A new
  exclusive repository epoch fences older writers, then enumerates bounded
  provider-side pending receipts so a crash before the local receipt becomes
  durable resolves aborted and a durable exact receipt resolves committed;
  absence is never classified before fencing. A decommission intent is first
  durable in the repository's SQLite writer, so all later write/authorization
  attempts see its admission epoch; the matching provider CAS then fences new
  replay, seal, and commit authorization. The bounded proof authenticates a
  complete all-live-generation scan and separate zero ciphertext, locator, and
  non-replay-index counts. Before Remove, the repository persists an immutable
  known-input plan carrying proof/fences, disposition/manifest, and stable
  provider/local ids. Every plan digest is a domain-separated canonical
  tagged-field preimage that excludes itself and all later request bytes/digests;
  SQLite then derives and atomically persists a sibling exact-request journal
  before that plan's side effect. A sole-member provider result durably exposes
  its authenticated `RetirementHandoffReady` value; SQLite persists that exact
  handoff, then a separate immutable post-handoff/pre-Begin plan and journal
  before Begin. It likewise writes a post-Begin Complete plan/journal, then
  post-terminal disposition and post-storage completion plan/journal pairs
  before each later side effect. It performs typed provider removal, local
  completion, status, and recovery only from those records: the provider
  non-last-member removal CAS or sole-member terminal-retirement CAS is the
  global linearization point and returns a signed proof-and-plan-bound terminal
  receipt, while SQLite remains fenced through
  plan/handoff/Begin-plan/`Retiring`/Complete-plan/disposition/completion and the
  matching local `BEGIN IMMEDIATE` CAS. This is not narrated as a distributed
  transaction. A local drain/delete/quarantine fault leaves a plan-and-receipt-
  bound recoverable state rather than reopening admission.
  The matching HR adapter remains draft while the port is draft. Plaintext fallback,
  caller-provided keys,
  process-local production keys, provider-internal imports, and logging key/
  plaintext material are forbidden.
- **ensure:** SQLite stores only approved nonsensitive schema metadata, opaque
  identifiers, generation-scoped idempotency locators, and authenticated
  ciphertext envelopes. Contract
  and real-file tests scan database and backup bytes for injected sentinels,
  hard-close/reopen with a fresh adapter, rotate and re-encrypt with idempotent
  restart, revoke old generations, remove the key provider before boot and
  mid-transaction, and race authorization, local receipt durability, commit,
  resolution, repository-epoch takeover, rotation, normal/emergency drain, and
  crash recovery in every order. The repository receives active/draining
  generation, rotation-fence, immutable membership snapshot, V1 active-writer
  format, and generation-scoped PRF authority only through the record-encryption
  port and selected key-service adapter; it never manufactures provider truth or
  gives that adapter a reverse repository edge. The only adapter-to-repository
  relationship is a named dev-only composition target, which Cargo and Buck
  execute after both implementations exist and whose reverse scans prove no
  library/runtime edge. Bounded pending-page exact/limit-plus-one,
  duplicate, missing, reordered, stale-epoch, and non-progressing cases fail
  closed. Tests prove no unkeyed equality token, acknowledgement without a
  resolved receipt, disclosure, partial plaintext, nonce reuse, fallback key,
  or completed revocation with an unresolved earlier authorization. The provider
  rejects an unregistered repository, freezes an immutable membership snapshot
  with a normal-rotation fence, and cannot accept a terminal receipt from a
  repository instance outside that snapshot. A repository removal is possible
  only after durable local intent/write fencing, provider admission fencing, a
  bounded real-file all-live-generation scan, and a provider-verified proof
  binding that fence, terminal write sequence, member/epoch, snapshot/version,
  rotation state, a complete scan checkpoint with zero ciphertext, locator, and
  non-replay-index references, and zero unresolved authorizations. Tests
  race authorization and commit before/after every fence/proof/removal boundary;
  outage, partition, stale snapshot, duplicate enrollment, or rejoin is a typed
  refusal.
- **overturn_when:** an independently reviewed encrypted-SQLite binding or
  storage adapter proves equivalent full-file confidentiality, key separation,
  rotation/revocation, crash recovery, portability, and provider-outage failure
  behavior and replaces the port/adapter sequence in the same wave.

</data_at_rest>

<protected_preimages_and_rekey>

## Decision: HR owns protected preimages and bounded rekey completion

- **achieves:** retries and authenticated commits retain one meaning across
  rolling versions, while normal rotation can actually remove every old-key
  repository reference before revocation.
- **origin:** a request-dependent keyed lookup can miss a changed request that
  reuses one logical idempotency key, an unnamed staged-write descriptor can
  omit an employee, lifecycle, idempotency, or outbox effect, an unfenced
  decommission observation can be overtaken by a durable write, a key adapter
  with no executable repository composition test can only narrate traversal,
  an unnamed registered-repository set can strand a source generation, and
  V1-only bytes cannot truthfully promise an undefined V2 replay path.
- **rule:** HR MUST own the exact versioned canonical-request and staged-write-
  descriptor encodings in its repository port. Their semantic fields, order,
  normalization, bounds, domain tags, optional/default behavior, and upgrade
  window are fixed in `SPEC.md`; provider code receives bounded bytes and
  authenticates them without reinterpretation. `IdempotencyLocatorV1` is the
  sole fixed purpose/domain for the generation-scoped lookup PRF. Its preimage
  binds only repository, tenant, operation kind, idempotency key, and key
  generation—not schema, canonical format, or canonical-request bytes—so a
  changed request reaches the same logical slot while each generation has a
  different opaque locator. No cleartext or stable cross-generation equality
  token exists. The executable baseline admits CanonicalRequestV1 only. Replay
  first obtains the provider-authenticated repository/epoch/fence/membership-
  bound generation set through `AcquireReplayGenerationSetV1`, then passes every
  returned opaque generation authority to `DeriveIdempotencyLocatorV1` before
  SQLite lookup. One bounded lookup locates the slot; it authenticates/opens one
  row and constant-time compares canonical plaintext under the SQLite writer to
  choose the stored outcome or `IdempotencyConflict`. It MUST NOT compare
  randomized ciphertext for equality, and zero match reserves only with the
  active locator.

  Decommission is a provider-neutral, two-sided fence. The repository-port's
  `ProduceDecommissionProofV1` producer first commits an intent that closes its
  shared write-admission epoch, then the provider CASes a matching admission
  fence that denies new replay, seal, and commit authorization. A bounded real-
  file scan under that durable fence enumerates every live-generation ciphertext,
  idempotency-locator, and admitted non-replay field-index reference in stable
  order, then authenticates zero counts, its checkpoint, and unresolved
  authorizations in `DecommissionObservationV1`/`DecommissionProofV1`.
  `IssueDecommissionProofV1` authenticates repository/member/epoch, membership
  snapshot/version, rotation state, live-generation digest, admission-fence
  id/epoch, terminal write sequence, scan checkpoint, and all three counts.
  Its exhaustive success sum is exactly `Issued { issuance:
  DecommissionProofIssuanceV1 }`; the provider ledger writes that one immutable
  issuance under the reference's canonical lookup key, and exact Issue replay
  or `ProofIssued` status returns byte-identical issuance bytes, including the
  nested proof/reference bytes and authenticators. SQLite persists that provider
  result and cannot mint or reconstruct a reference. `DecommissionProofV1`,
  `DecommissionProofReferenceV1`, and their issuance have canonical kinds
  `0x07`, `0x08`, and `0x09`: each authenticated value is encoded as a body
  without its authenticator, signed over `domain || 0x00 || body`, emitted with
  the authenticator appended as its final tag, then externally digested over
  that final wire. A proof/reference digest never includes itself. The issuance
  nests those exact final wires and is valid only when the reference binds the
  proof digest. Their frozen maxima are 1,805, 1,265, and 3,092 bytes;
  Missing/Mismatch/Corrupt/AuthenticatorInvalid are named fail-closed results.

  After that proof but before any provider side effect,
  `RemoveRepositoryDecommissionV1` durably CASes an immutable
  `RepositoryDecommissionRemovalPlanV1`. That pre-Remove plan binds the
  proof/fences, `Quarantine` or `Delete` disposition and manifest, a
  preallocated retirement-fence id, and distinct scoped provider and local
  operation ids. Its exact canonical Removal digest contains only those stable
  inputs; only after it exists may the sibling Remove request journal bind
  materialized request bytes/digest. A
  later cardinality result cannot choose or change an id or disposition, but it
  can return a provider-authenticated handoff whose bytes did not exist before
  Remove. SQLite must persist that handoff and an immutable
  `RepositoryDecommissionRetirementBeginPlanV1` plus its journal before Begin,
  then persist an immutable `RepositoryDecommissionRetirementCompletePlanV1`
  plus its journal before Complete; later provider-terminal and storage receipts
  similarly precede immutable local disposition and completion plan/journal
  pairs. The provider-owned durable proof ledger returns one issuance containing
  `DecommissionProofReferenceV1` and its immutable full proof; Remove and
  Complete resolve and reauthenticate that bounded reference rather than
  serialize an unbounded full proof. The ledger retains both exact canonical
  values through Issue/Remove/Complete/Get/recovery replay and terminal-receipt
  GC; Missing, Mismatch, Corrupt, and AuthenticatorInvalid remain distinct typed
  fail-closed results for proof and reference verification.
  The typed Begin request, exact request journal/idempotency cell, and signed
  `KeyringRetirementFenceV1` carry the same `begin_plan_digest`; a changed
  digest is `MembershipOperationConflict`, while Complete must match the
  authenticated fence binding. The fence maximum is 2,273 bytes and the
  dependent Complete plan maximum is 3,832 bytes.
  The canonical plan codecs have immutable kind bytes `Removal=0x01`,
  `BeginRetirement=0x02`, `CompleteRetirement=0x03`,
  `LocalDisposition=0x04`, and `LocalCompletion=0x05`, fixed domain tags,
  ascending required field tags, u16 lengths, and a 4,096-byte body ceiling whose maximum typed
  Removal/Begin/Complete/Disposition/Completion plans and request journals are
  proven by fixed minimum/maximum vectors, plus-one, kind-substitution/unknown-
  kind, mutation, parent, and independent-rederivation tests. Plan decoders
  accept only `0x01..=0x05`; every other byte is rejected before a plan digest,
  journal, or effect.

  The canonical auxiliary-wire freeze leaves the published `0x01..=0x0c`
  assignments unchanged and adds `KeyringRetirementHandoff = 0x0d`,
  `KeyringRetirementFence = 0x0e`, `DecommissionRemovalReceipt = 0x0f`, and
  `KeyringRetirementReceipt = 0x10`; `0x00` and unassigned `0x11..=0xff` fail
  closed. The global decoder recognizes `0x01..=0x10`; a plan decoder accepts
  only `0x01..=0x05` and reports recognized `0x06..=0x10` as typed known-wrong-
  kind, distinct from unknown `0x00`/`0x11..=0xff`. The exact kind-scoped provider-operation order is
  `RegisterKeyringRepository=0x01`, `BeginRepositoryDecommission=0x02`,
  `IssueDecommissionProof=0x03`, `AbortRepositoryDecommission=0x04`,
  `RemoveKeyringRepository=0x05`, `BeginKeyringRetirement=0x06`,
  `CompleteKeyringRetirement=0x07`, and `BeginNormalRotation=0x08`.
  `Retired=0x01` is the only legal tag-19 state byte; `0x00` and every other
  state byte are corrupt.

  Every auxiliary record reuses the exact published 16-byte header and TLV
  rules. The body carries only the ascending non-authenticator tags; the
  provider signs `ASCII(literal_authentication_domain) || 0x00 || body_wire`;
  the final same-kind wire appends the authenticator as its last tag; only then
  is an external digest derived. External digests are never tags, and retained
  exact final bytes—not decoded reserialization—are the replay authority. The
  frozen schemas and independently checked ledgers are:

  | kind | exact body tags | final tag | literal authentication / external digest domain | body/final maximum |
  | --- | --- | --- | --- | --- |
  | `KeyringRetirementHandoff = 0x0d` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 live_generation_digest`; `10 removal_plan_digest` | `11 authenticator` | `hr.decommission.keyring-retirement-handoff-authenticator.v1` / `hr.decommission.keyring-retirement-handoff.v1` | `926` / `1,441` |
  | `KeyringRetirementFence = 0x0e` | `1 exact final handoff wire`; `2 retirement_fence_id`; `3 retirement_begin_operation_id`; `4 begin_plan_digest` | `5 authenticator` | `hr.decommission.keyring-retirement-fence-authenticator.v1` / `hr.decommission.keyring-retirement-fence.v1` | `1,758` / `2,273` |
  | `DecommissionRemovalReceipt = 0x0f` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 prior_membership_snapshot_id`; `7 prior_membership_version`; `8 successor_membership_snapshot_id`; `9 successor_membership_version`; `10 removal_operation_id`; `11 removal_plan_digest` | `12 authenticator` | `hr.decommission.removal-receipt-authenticator.v1` / terminal domain below | `1,034` / `1,549` |
  | `KeyringRetirementReceipt = 0x10` | `1 keyring_id`; `2 repository_id`; `3 member_instance_id`; `4 repository_epoch`; `5 decommission_proof_digest`; `6 membership_snapshot_id`; `7 membership_version`; `8 rotation_fence_id`; `9 retirement_fence_id`; `10 removal_plan_digest`; `11 retirement_begin_operation_id`; `12 retirement_complete_operation_id`; `13 all_generation_digest`; `14 scan_checkpoint_digest`; `15 durable_ciphertext_references`; `16 durable_locator_references`; `17 durable_non_replay_index_references`; `18 unresolved_authorizations`; `19 state` | `20 authenticator` | `hr.decommission.keyring-retirement-receipt-authenticator.v1` / terminal domain below | `1,404` / `1,919` |

  The proof and reference signing domains are the explicit literals
  `hr.decommission.proof-authenticator.v1` and
  `hr.decommission.proof-reference-authenticator.v1`; their existing external
  domains remain unchanged. `ProviderDecommissionTerminalReceiptV1` is the
  closed logical sum of final kind `0x0f` (`Removed`) or final kind `0x10`
  (`KeyringRetired`), with no outer wire; the signed header kind is the variant discriminator.
  Both variants derive `provider_terminal_receipt_digest` only after the final
  wire under `hr.decommission.provider-terminal-receipt.v1`.

  The fence, Begin plan, and Begin request nest exact final kind-`0x0d` bytes;
  the Complete plan nests exact final kind-`0x0e` bytes; disposition nests
  exact final kind-`0x0f` or `0x10` bytes. Wrong kind/header/schema, body/final
  confusion, variant substitution, count/tag/order/length/domain/authenticator/
  digest mutation, or nested reserialization is a typed refusal. g.0, g.1, and
  g.2 independently freeze body/final min, max, and max-plus-one vectors for all
  four rows, including response loss and fresh-process byte-identical replay.
  The published plan maxima `3,096/1,793/3,832/2,179/253`, request maxima
  `2,267/1,758/2,184/292/288`, storage receipt/binding `870/1,246`, and
  `4,096 - 3,832 = 264` bytes of headroom remain unchanged.

  Both `KeyringMembershipError` and `RepositoryDecommissionRemovalError`
  contain the same explicit quartets
  `RetirementHandoff{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`,
  `KeyringRetirementFence{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`, and
  `ProviderTerminalReceipt{Missing,Mismatch,Corrupt,AuthenticatorInvalid}`.
  Absence is Missing; bad framing/kind/count/tag/order/length/bound/state byte/
  nested kind or missing authenticator tag is Corrupt; wrong canonical identity,
  parent, digest, status, or expected terminal branch is Mismatch; and a present
  envelope with an invalid key or signature is AuthenticatorInvalid.
  `KeyringRetirementFenceStale` and `KeyringRetirementPreconditionFailed` remain
  distinct semantic branches. g.0 and recovery preserve the same names, with no
  wildcard or generic terminal-invalid branch. The existing receipt fields and
  maxima do not expand: chain assurance for omitted historical fields relies on
  retained provider state and byte-identical exact-operation replay, not a claim
  that those omitted fields are directly authenticated.
  Accordingly, binding tag 8 is deterministic after terminal verification:
  kind `0x0f` supplies `removal_operation_id` of operation kind `0x05`, while
  kind `0x10` supplies `retirement_complete_operation_id` of operation kind
  `0x07`. It is never caller-selected; a branch/id-kind disagreement is the
  corresponding typed Mismatch. Because the retired receipt omits
  `begin_plan_digest`, `complete_plan_digest`, and `retirement_fence_digest`,
  verification composes the retained exact operation cells/plans and handoff/
  fence ledgers. Exact auxiliary final wires, operation cells/plans, proof
  issuance, and the provider-envelope-selected verification-key epoch survive
  Get/replay/recovery through the bounded local-terminal GC horizon and are
  collected only by the atomic terminal cleanup. Neither caller nor response
  selects the epoch, and no linked record has an independent early-GC path.
  `RemoveKeyringRepositoryV1` rechecks its plan digest together with that proof
  before membership change. A non-last removal yields a proof-and-plan-bound
  receipt; a sole member first receives provider `RetirementHandoffReady`,
  durably records that exact handoff and Begin plan, then provider `Retiring`,
  then a Complete plan and terminal retirement receipt. `CompleteKeyringRetirementV1`
  verifies the same all-generation zero ciphertext/locator/non-replay-index
  checkpoint and counts before it revokes every generation.

  The local status type has `ProofIssuePlanned`, `RemovalPlanned`,
  `RetirementHandoffPersisted`, `RetirementBeginPlanned`, `Retiring`,
  `RetirementCompletePlanned`, provider-terminal-pending-disposition,
  disposition-planned/in-progress/applied, completion-planned, and
  receipt-carrying `Removed`/`KeyringRetired` variants.
  `CompleteRepositoryDecommissionV1` accepts only the stored plan digest and
  stored local-completion id: it never accepts a caller-provided receipt or new
  disposition. `GetRepositoryDecommissionStatusV1` validates the corresponding
  provider state before reporting it. `RecoverRepositoryDecommissionV1` reads
  the plans and repeats only its stored Remove, Begin, Complete, disposition, or
  completion step; before each later side effect it writes that step's exact
  request plan, and its response id cannot become a side-effect id. Thus every
  crash/lost-response edge after plan write, provider handoff, handoff
  persistence, Begin-plan persistence, Begin, `Retiring`, Complete-plan
  persistence, Complete, terminal receipt, disposition/completion planning and
  execution, or local CAS converges without inventing an id, re-registering, or
  reopening the old epoch. The provider CAS/terminal
  receipt is the global removal linearization point; local plan/intermediate/
  terminal CASes are independently atomic and deliberately fenced across the
  remote/local gap.
  `LocalDecommissionStorageReceiptV1` is separately retained by canonical
  local-receipt key and 32-byte external digest; the only completion-plan field
  is that digest. No later than the atomic `LocalDispositionApplied` transition,
  SQLite stores `LocalDecommissionStorageReceiptBindingV1`: a 1,246-byte,
  signed kind-`0x0c` record that names the receipt lookup key, expected receipt
  digest, repository/keyring/member/epoch, removal/disposition/terminal parents,
  terminal and local operation ids, disposition/manifest, admission epoch, and
  metadata key id/epoch. Its signer input is the sixteen-tag binding body and
  its detached authenticator is appended only afterward. Fresh recovery resolves
  the receipt and binding, verifies the receipt digest, parent fields, key epoch
  and signature, then derives the 253-byte plan. A missing, duplicate, corrupt,
  mismatched, or bad-authenticator receipt/binding is the named
  `LocalDispositionReceiptInvalid` branch, not a way to substitute its 870-byte
  wire. The repository metadata-commit signer/verify port is g.0-owned,
  key-service implemented, rotation-aware, fail-closed on unavailable/unknown
  key/signature failure, and has no reverse adapter-to-repository edge.
  Provider Begin and Abort serialize through one provider transaction: Begin
  commits its idempotency cell, member Decommissioning state, and signed Fenced
  value atomically, so provider status is never IntentPending. Before that
  commit Get is NotStarted; after it, Get and exact Begin replay are Fenced.
  Abort-first writes the signed Aborted tombstone and tombstones delayed Begin;
  Begin-first makes Abort return the exact closed status. Repository-local
  IntentPending is only the persisted pre-Begin SQLite state and remains
  write-closed until one of those signed outcomes is installed.
  Each durable intent preallocates distinct Begin, Issue-proof, and abort
  provider-operation ids before its first provider call. The terminal fenced
  scan then durably records a `ProofIssuePlanned` observation plus the exact
  Issue request digest before it invokes Issue-proof. An
  abort—including recovery that observes provider `NotStarted` for a persisted
  local intent—sends only that stored Begin/abort tuple through the provider
  begin-operation tombstone CAS and then records a strictly greater local
  reopened-admission epoch before local admission can reopen. A recovery-response
  id is never a provider-operation id, so a delayed original begin cannot
  resurrect that `NotStarted` race.
  Provider operation ids are explicitly kind-scoped and their canonical request
  digests include that kind and its operation-specific authority tuple; local
  response/disposition/completion ids live in separate namespaces. g.0 freezes named exhaustive
  `ProviderDecommissionStatusV1`, provider Abort and membership-mutation result
  sums, and repository Abort/Remove/Complete result sums. Port, key-adapter, and
  SQLite implementations match every status/error branch without a wildcard;
  `DecommissionObservationStale` is a named provider error and maps unchanged
  into the proof/removal paths that can observe it.
  A non-last remove yields the successor snapshot. A last remove instead yields
  a typed retirement handoff; `BeginKeyringRetirementV1` fences all writers and
  exposes `Retiring`, while `CompleteKeyringRetirementV1` produces the distinct
  no-member `Retired` state only after the authenticated all-generation proof
  has zero ciphertext, locator, non-replay-index, and unresolved counts. The
  shared `RepositoryDecommissionRemovalError` includes
  `MembershipOperationConflict`, plan/status/receipt mismatches, and every local
  drain/disposition failure, so a changed id reuse is not translated into an
  untyped local conflict. Response loss, crash, partition, abort, resume, and
  rejoin are typed state transitions; no later durable write can commit after
  the terminal observation, provider removal, or retirement.

  A later format is non-dispatchable until a separately accepted codec,
  writer-authority, reader-cohort, admission/retirement, migration, and
  independent-encoder decision supplies its exact file/build/test closure; V1
  claims no V2 write or retry. Format discovery never changes locator lookup.
  Normal rotation has a global per-keyring no-overlap invariant: G+2 MUST NOT
  activate while G is draining, has durable ciphertext, locator, or non-replay
  field-index references, has an unresolved authorization or incomplete rekey,
  or lacks its terminal
  provider retirement receipt. The retirement receipt is bound to the immutable,
  versioned keyring membership snapshot captured by the rotation CAS and
  contains a terminal zero-reference receipt for every enrolled repository
  instance. A membership mutation is refused during a drain; only after G is
  zero-reference and revoked may a new normal rotation start. Normal rotation
  MUST execute through a provider-neutral HR rekey contract and the selected
  repository: after the provider drain fence, a bounded cursor scan opens
  old-generation envelopes, seals under the active generation, recomputes
  generation-scoped locators, and atomically CAS-replaces each observed record
  plus a durable checkpoint. Revocation requires a terminal zero-reference
  receipt, zero earlier unresolved commit authorizations, and fresh-process
  recovery.

  The matrix is capped at two generations, two locator derivations, five
  locator-row reads, and one authenticated open per replay. A malformed, stale,
  oversized, colliding, or locator-divergent matrix is a typed refusal, never
  zero-match creation. The current V1 set has exactly one canonical format; a
  future format may not expand locator work and cannot be admitted until the
  oldest format has a durable compatibility-retirement receipt. Adapter
  structure, file-slot admission, port freeze, provider behavior, minimal
  concrete adapter behavior, repository/SQLite behavior plus its dev-only real
  composition target, and rekey remain separate L2i.0d.1, L2i.0f, L2i.0g.0,
  L2i.0g.1, L2i.0g.1a, L2i.0g.2, and L2i.0h changes. The g.2 target may execute
  real open/seal/authorization/decommission behavior only after g.1a accepts it;
  L2i.2d does not backfill that prerequisite.
- **ensure:** independent Cargo and Buck encoders share typed inputs but not an
  encoder, and assert exact idempotency-locator and `CommitBinding` preimages,
  semantic-equivalence and changed-field vectors, unknown/omitted/reordered
  descriptor fields, and every exact/limit-plus-one bound. Real-file SQLite
  tests interrupt locator reservation, scan, open, seal, reindex, page CAS,
  checkpoint, zero-count, provider revoke, and completion recording; a fresh
  process resumes from the last committed checkpoint, and stale epochs, CAS
  exhaustion, missing keys, provider/repository outages, corrupt cursors, and
  nonzero references produce closed typed outcomes with no key fallback or
  premature readiness. They prove V1 replay across active-only and
  active-plus-draining response loss, rotation/page-CAS, hard close, rekey, and
  fresh restart: same logical key/same plaintext returns the original outcome,
  while changed plaintext conflicts without a second effect. Equal requests
  sealed under different nonce/generation authenticate/open and compare only
  canonical plaintext, while ciphertext/tag/associated-data tampering refuses.
  They attempt G+2 during G drain, emergency drain, source loss, stale
  matrix/lease, and concurrent replay/rekey schedules.

  The key-service composition target executes the real SQLite repository through
  the port and selected adapter, including `AcquireReplayGenerationSetV1`,
  returned-authority locator derivation, and stale/replayed/provider-loss errors;
  its matching Cargo/Buck test target is the only dev edge to SQLite and reverse
  scans reject a library/runtime adapter-to-repository edge. That target is
  enabled only after the g.1a adapter open/seal, authorization/resolution, and
  decommission-fence tests are accepted. Decommission tests race
  authorization/commit before and after local intent, provider fence, each scan
  page, terminal observation, proof issuance, provider removal, local completion,
  and recovery; response loss, crash, partition, local drain/delete/quarantine
  fault, stale membership/live-generation state, concurrent rotation,
  tombstoned-begin abort/resume, last-member retirement, and rejoin all preserve
  the fenced member or return a typed refusal.
- **overturn_when:** an independently reviewed repository or encrypted-SQLite
  design proves equivalent cross-version replay identity, complete staged-
  effect authentication, bounded crash-resumable re-encryption, and
  linearizable zero-reference revocation with a smaller owner-local contract.

</protected_preimages_and_rekey>

<availability_accounting>

## Decision: required-authority refusal is correct but not available

- **achieves:** fail-closed security never creates a false-green product SLO.
- **origin:** excluding unavailable authority from availability can report a
  perfect month while every valid HR request is refused for a durable repository,
  key, PDP, Audit, Packs, or runtime-context outage.
- **rule:** the availability denominator MUST include every syntactically valid,
  capacity-admitted facade request unless an available authority determines it is
  caller-caused validation, unauthenticated, or forbidden traffic. A required
  authority or selected durable-adapter outage, timeout, stale proof, or readiness precondition is a typed
  fail-closed correctness result and an availability failure for eligible traffic;
  it cannot be retroactively excluded when the authority recovers. The outage
  burns the product error budget from first failed eligible request until service
  recovers or the router acknowledges the cohort withdrawal. Readiness alone is
  not the withdrawal acknowledgement.
- **ensure:** facade telemetry emits bounded eligible, good, required-authority-
  failure, error-budget, readiness-transition, and routing-withdrawal counters;
  fault campaigns independently remove every mandatory provider and the durable
  adapter before and during
  an eligible request and assert no mutation/disclosure, denominator inclusion,
  budget burn, and recovery or acknowledged withdrawal.
- **overturn_when:** an accepted product SLO decision supplies an equally
  auditable denominator that cannot hide required-authority unavailability.

</availability_accounting>

<policy_and_privacy>

## Decision: installed pack authority and fail-closed sensitive access

- **achieves:** HR law follows the tenant's admitted jurisdiction while
  sensitive employee data is never disclosed on caller assertions alone.
- **origin:** the domain currently accepts rulepack references and produces
  sensitive-read decisions, while D-24 makes `packs/` the thin install authority
  and owner overlays the policy content.
- **rule:** HR MUST resolve the installed pack-id through a port and evaluate a
  signed, content-addressed HR overlay owned by this app. Sensitive reads and
  mutations MUST require verified principal, tenant binding, unexpired PDP
  decision provenance, purpose/legal basis, and required audit evidence before
  disclosure or commit; missing, stale, or conflicting authority fails closed.
- **ensure:** receipts bind tenant, pack-id, overlay digest/generation, policy
  revision, principal, purpose, and correlation/idempotency key; negative tests
  cover forged proof, cross-tenant bodies, missing basis, stale overlays, and
  audit unavailability with no mutation or disclosure.
- **overturn_when:** central pack selection or IAM proof changes through an
  accepted owner decision that still provides one runtime authority and equal
  fail-closed evidence.

</policy_and_privacy>

<migration_discipline>

## Decision: structure before behavior

- **achieves:** reviewable changes whose regressions can be attributed and
  reverted without mixing package movement with new employment semantics.
- **origin:** HR has eight hand-written Rust files above ADR-0719's 300-line cap:
  `core/employment-domain/src/lib.rs`, its `leave_balance`,
  `leave_carryover_forfeiture`, and `onboarding` tests,
  `ports/employment-api/src/lib.rs`, and the infrastructure adapter's
  `src/authz.rs`, `src/lib.rs`, and `tests/runtime.rs`; it also has misplaced
  port/transport responsibilities and illegal dependency direction. No SQLite
  dependency or adapter package exists, so admitting that graph and implementing
  its durability protocol are distinct ADR-0719 D-33 change classes. The IAM
  workload-manifest source is also 618 lines and must be structurally prepared
  before the IAM content retirement touches it.
- **rule:** the migration MUST first repair the stale Buck labels in the exact
  HR plus reverse-consumer build closure, then proceed as L2b file-budget
  splits; L2c structural face admission followed by content-only role
  separation; L2d structural draft-port/adapter admission, content-only
  dependency inversion, then structural removal of direct Data/Gateway edges;
  serialized SQLite dependency and adapter-face admission, including the
  owner-local record-encryption port; content-only encrypted SQLite parity and
  crash proof; a fail-closed Connect generator/runtime decision
  gate; structural empty codegen/package/build admission; a separate schema-only
  People contract; a content-only fail-closed `Unrouted` process state; and then
  a content-only generated-Connect onboarding slice. The temporary JSON/Serde
  translation MUST move into `transport-employment-compat`; no new
  `*-app` library-only compatibility facade may be created.
  A mandatory D-29 IAM consumer sequence MUST first install D-35/D-41 stable
  membership in its oversized tenant-workload-manifest crate, then delete
  IAM-local HR composition and remove every
  IAM Cargo/Buck/Rust edge into `app/hr` without substituting an HR client,
  after which a separate HR structural lane MUST retire the compatibility
  surfaces. Production serving then requires decision-gated Packs/install,
  Policy/IAM authorization-evidence, Audit/outbox, authenticated record-
  encryption/key-service, and trusted runtime-context provider contracts; an
  intervening D-33 structural commit/rekey file-slot lane, content-only
  canonical commit-fence protocol, and separate bounded repository rekey/
  recovery lane; D-28/D-30-correct draft adapter faces while their HR ports
  remain draft; content-only adapter
  behavior; structural composition edges; content-only composition; and a
  separately gated main/route activation before any tenant cohort. No live
  route or production-readiness promotion may precede the zero-inverse-edge
  proof or those production authorities. Structural lanes MUST preserve public
  behavior and make no durability, network, or readiness claim.
- **ensure:** each lane has an exact changed-path envelope, Cargo and Buck build
  closure, reviewer jurisdiction, rollback, before/after tests, no generated
  hand edits, and a protected PR. SQLite behavior begins only after the pinned
  binding, workspace/lock, port, adapter face, and Cargo/Buck membership
  prerequisites are green and frozen; routing begins only after an inverse scan
  of the whole IAM cone proves no manifest, Buck label, or Rust import reaches
  any `app/hr` package and outage tests prove that unavailable Packs, Policy/IAM,
  Audit, or encryption/key authority prevents routing, mutation, plaintext
  persistence, and sensitive disclosure as required by the operation class.
- **overturn_when:** independently reviewed evidence shows two adjacent lanes
  cannot be separated safely and a replacement plan preserves the same rollback
  boundary and proof strength.

</migration_discipline>

<stable_item_membership>

## Decision: L2b installs owned stable compile-time indexes

- **achieves:** later HR work adds one uniquely named item without editing a
  shared crate or test index, while Cargo and Buck compile the same membership.
- **origin:** L2b turns three monolithic crate roots and four oversized tests
  into multi-file surfaces. ADR-0719 D-41 forbids replacing those monoliths with
  tracked generated indexes or hand-maintained `mod` lists, and Storage already
  demonstrates the owned sorted `build.rs` plus `OUT_DIR` pattern.
- **rule:** each L2b package MUST install its own crate-root `build.rs`. The
  script MUST enumerate the package's declared `src/items/*.rs` and split-test
  item directories, retain Rust sources, sort their paths deterministically,
  and emit named membership files only under `OUT_DIR`. Each affected crate or
  test root MUST hold a stable `include!(concat!(env!("OUT_DIR"),
  "/<name>.generated.rs"))` index. No generated membership file may be tracked,
  and no parent may carry a manual per-item `mod` inventory.
- **ensure:** Cargo's auto-detected build script and Buck's `buildscript_run`
  MUST execute the same scanner over the same globbed directory sets; their
  generated source order and contents must match. L2b evidence adds, renames,
  and removes a uniquely named item without an index edit, builds it through
  both graphs, and rejects tracked/generated or manual membership inventories.
- **overturn_when:** rustc provides deterministic directory membership without
  a generated index, or a five-field owner decision supplies a smaller owned
  mechanism that preserves stable parents and Cargo/Buck parity.

</stable_item_membership>

## Rejected destinations

- HR core importing cloud capability core/ports or another app's internals.
- A JSON/HTTP adapter treated as the semantic API source of truth.
- Git, an in-memory map, or caller-owned state presented as durable HR storage.
- Dual-write migration between SQLite and cloud/commodity adapters.
- A first-party/trusted mode that bypasses normal IAM, PDP, audit, quota, or
  metering.
- New People behavior mixed into file moves, crate-graph changes, or SQLite
  introduction.
- SQLite dependency, package/face, Cargo/Buck, root/lock, or scanner admission
  mixed with transaction, migration, replay, or recovery behavior.
- A tracked generated module index, hand-maintained per-item `mod` list, or
  Cargo-only item scan that leaves Buck with different membership.
- A tonic/gRPC client, server, service stub, content type, status trailer,
  hand-written Connect parser/framer/error envelope, or fake transport presented
  as the generated Connect-class People boundary.
- Any terminal Cargo, Buck, or Rust edge from `iam/**` into `app/hr/**`, even
  through an HR-owned client adapter.
- A non-draft adapter path or package implementing an HR `ports/draft/*`
  contract without a preceding D-28/D-29 port promotion.
- A newly created `facade/*-app` with no compiler-only `src/main.rs`, or a
  facade process used as an in-process JSON/compatibility translation library.
- An unkeyed digest of canonical request material, an adapter-only key fence,
  or revocation declared complete while an earlier commit authorization remains
  unresolved.
- A production People composition that supplies runtime time/telemetry from a
  test fake, the facade, or process/system clocks instead of the selected
  `hr-runtime-context-oyatie-draft` provider adapter.
- Plaintext sensitive HR values in SQLite, backups, page images, logs, or
  fallback process-local keys when the selected encryption/key provider fails.
