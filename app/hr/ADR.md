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
  L2i.0g.1 implements the selected key-service adapter, L2i.0g.2 only then
  implements repository/SQLite behavior and its real composition target, and
  L2i.0h implements bounded
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
  replay, seal, and commit authorization until atomic membership removal. The
  matching HR adapter remains draft while the port is draft. Plaintext fallback,
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
  rotation state, zero references, and zero unresolved authorizations. Tests
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
  file scan under that durable fence produces a terminal observation of every
  live-generation locator/ciphertext reference and unresolved authorization.
  `IssueDecommissionProofV1` authenticates that observation, including
  repository/member/epoch, membership snapshot/version, rotation state, live-
  generation digest, admission-fence id/epoch, and terminal write sequence.
  `RemoveKeyringRepositoryV1` atomically rechecks all of them before membership
  change and leaves the old member fenced. Response loss,
  crash, partition, abort, resume, and rejoin are typed state transitions; no
  later durable write can commit after the terminal observation or removal.

  A later format is non-dispatchable until a separately accepted codec,
  writer-authority, reader-cohort, admission/retirement, migration, and
  independent-encoder decision supplies its exact file/build/test closure; V1
  claims no V2 write or retry. Format discovery never changes locator lookup.
  Normal rotation has a global per-keyring no-overlap invariant: G+2 MUST NOT
  activate while G is draining, has durable ciphertext or locator references,
  has an unresolved authorization or incomplete rekey, or lacks its terminal
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
  structure, file-slot admission, port freeze, provider behavior,
  repository/SQLite behavior plus its dev-only real composition target, and
  rekey remain separate L2i.0d.1, L2i.0f, L2i.0g.0, L2i.0g.1, L2i.0g.2, and
  L2i.0h changes.
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
  scans reject a library/runtime adapter-to-repository edge. Decommission tests
  race authorization/commit before and after local intent, provider fence, each
  scan page, terminal observation, proof issuance, and removal; response loss,
  crash, partition, stale membership/live-generation state, concurrent rotation,
  abort/resume, and rejoin all preserve the fenced member or return a typed
  refusal.
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
