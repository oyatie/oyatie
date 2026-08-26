# HR

Owner: `app/hr`

Status: portable-app migration; domain foundation only

HR is the tenant-portable People and employment application. It owns employee
and employment records, organization/manager relations, onboarding readiness,
leave policy projections, labor-compliance decisions, sensitive-read policy,
and HR evidence references.

The landed Rust domain and in-process adapters are test foundations. They do
not yet constitute a durable service, sold network facade, installed-pack
integration, downstream delivery path, or measured SLO. The current direct
Data/Gateway dependencies and volatile storage are migration debt.

Canonical owner law:

- [ADR.md](ADR.md) — decisions and portability boundaries
- [PRD.md](PRD.md) — product requirements, acceptance, and SLO objectives
- [SPEC.md](SPEC.md) — current contract and target transaction/fault semantics
- [PLAN.md](PLAN.md) — L2a through L2k.3 implementation, retirement, and
  bounded-production sequence

Replay obtains its bounded, provider-authenticated generation matrix through the
record-encryption port, then uses only a returned generation-scoped opaque
authority to derive an idempotency locator for the repository, tenant,
operation, and idempotency key. The locator never contains mutable request
content and changes with the generation, so it is neither cleartext nor a stable
cross-generation equality token. It locates the one logical slot; the adapter
then authenticates/decrypts that row and constant-time compares canonical
plaintext in memory. A same key with changed semantic plaintext is a conflict,
not a second reservation. Ciphertext equality is never replay equality;
plaintext never persists outside authenticated envelopes.
The executable baseline is canonical-request V1 only: a second format cannot be
advertised, selected for writes, or required by replay/rekey until a separately
accepted format-lifecycle decision supplies its codec, authority, migration, and
independent-oracle closure. Keyring membership is provider-authoritative and
frozen into a rotation fence, so a missing repository cannot be omitted before
an old generation is revoked. Repository decommission first closes a durable
SQLite write-admission fence and the matching provider admission fence, then
produces a bounded, authenticated proof of zero ciphertext, locator, and
non-replay field-index references across every live generation plus zero
unresolved authorizations. `IssueDecommissionProofV1` returns one explicit
`Issued { issuance: DecommissionProofIssuanceV1 }` provider-ledger value;
`ProofIssued` Get recovery returns byte-identical issuance bytes, including the
same canonical proof/reference bytes and authenticators, not a proof-only value
or a repository-minted reference. Proof/reference/issuance have immutable
canonical kinds `0x07`/`0x08`/`0x09`: sign the body without an authenticator,
append that authenticator to the final proof/reference wire, then calculate the
external digest. Their maxima are 1,805/1,265/3,092 bytes, and missing,
mismatch, corrupt, or bad-authenticator values fail closed. Before Remove, SQLite durably records an immutable
known-input removal plan that binds a bounded authenticated proof
reference/fences, fixed `Quarantine` or `Delete` disposition and manifest,
preallocated retirement fence, and distinct scoped provider/local ids. Every
Removal/Begin/Complete/disposition/completion plan digest has a fixed
domain-separated tagged-field preimage that excludes itself and all later
request bytes/digests; SQLite atomically derives a sibling exact-request journal
only after that digest and before its side effect. A sole-member provider result
is `RetirementHandoffReady`, a durable, queryable signed handoff; SQLite
persists that handoff and a separate post-handoff Begin plan/journal with exact
handoff bytes/authenticator, Begin id, fence id, and request digest before it
calls Begin. The typed Begin request, its exact journal/idempotency record, and
the returned signed retirement fence all carry the same `begin_plan_digest`;
changed bytes or digest are a typed conflict. It similarly persists post-Begin Complete, post-terminal
disposition, and post-storage completion plan/journal pairs before each later
side effect. The repository performs typed provider
removal, local completion, status, and recovery only from those records. Its
local status includes planned, handoff-persisted, Begin-planned, `Retiring`,
Complete-planned, provider-terminal-pending-disposition,
disposition-planned/in-progress/applied, completion-planned, and receipt-
carrying terminals; the provider receipt also binds the parent plan. Response
loss, crash, local drain/delete/quarantine failure, or partition therefore
repeats only the stored step and converges without inventing bytes, an
id/disposition, reopening, or re-registering an old member.
The `Removed` status carries removal, storage, and local-completion receipts; it
is never a bare terminal label. A sole member receives a typed retirement
handoff rather than an impossible empty membership snapshot: it persists the
exact Begin plan, retirement fences all writers, reports `Retiring`, and revokes
every generation only after the
same authenticated all-generation zero-reference/unresolved proof, ending in a
separate no-member `Retired` keyring state. A begin operation has a provider-
side abort tombstone; the durable intent preallocates distinct Begin, Issue-
proof, and abort provider ids before Begin, then the terminal fenced scan writes
the exact Issue request digest in `ProofIssuePlanned` before it calls Issue.
Recovery sends only the persisted tuple through the tombstone CAS. A recovery
response id never becomes a provider side-effect id. Provider ids are
operation-kind-scoped, and g.0 freezes named exhaustive provider status/Abort/
membership-mutation and local Abort/Remove/Complete result sums; every status
and error branch is explicitly matched in port/adapter/SQLite tests, including
`DecommissionObservationStale`. Thus `NotStarted` is not
permission to reopen and a late begin cannot resurrect a locally aborted fence.
The five plans also have immutable header kinds: Removal `0x01`, Begin `0x02`,
Complete `0x03`, LocalDisposition `0x04`, and LocalCompletion `0x05`; a
substituted, receipt, or unknown kind is rejected before digest/journal/effect.
Port, key-service, memory, and SQLite vectors cover all plan/request kinds,
min/max/+1, and kind/tag/parent/id mutation. The full proof and its bounded authenticated `DecommissionProofReferenceV1` are
retained as one provider-issued ledger value through exact-operation replay and
terminal-receipt GC; Issue/Get response loss returns that exact value, while
named Missing, Mismatch, Corrupt, or AuthenticatorInvalid variants fail closed.
A separately retained
SQLite `LocalDecommissionStorageReceiptV1` is re-resolved by its canonical key
and 32-byte digest before completion/recovery. In the same atomic transition
that records `LocalDispositionApplied`, SQLite also persists a signed 1,246-byte
`LocalDecommissionStorageReceiptBindingV1` covering the receipt lookup/digest,
identity/parent/operation/disposition/manifest/admission fields, and metadata
signing key id/epoch. A fresh process verifies receipt and binding
bytes/digests/key/signature before it derives the 253-byte completion plan; a
missing, changed, corrupt, duplicate, or unauthenticated receipt/binding is
`LocalDispositionReceiptInvalid`, not an 870-byte completion-plan input. The
g.0-owned metadata-commit signer/verify port is implemented by the key-service
adapter with retained verification keys through receipt GC; unavailable, unknown
key, or bad-signature outcomes withdraw readiness and there is no reverse
adapter-to-repository runtime edge.
The fence wire is 2,273 bytes, the dependent Complete plan is 3,832 bytes, and
the digest-addressed completion plan is 253 bytes; the Begin request remains
its already-frozen 1,758-byte four-record wire. Provider Begin atomically
commits its replay cell, Decommissioning membership state, and signed `Fenced`
result, so provider status never exposes `IntentPending`. That name is solely
the write-closed SQLite pre-Begin state: response loss resolves by Get/exact
replay to `NotStarted`, signed `Aborted`, or a signed closed state, and only the
stored Abort tuple may act on `NotStarted`. g.0/g.1/g.2 tests freeze minimum and
maximum plan/request byte vectors, every field/id/parent/receipt mutation,
max-plus-one, independent rederivation, and crashes before/after plan, journal,
issuance, and side-effect persistence.
Minimal concrete
key-adapter open/seal, authorization/resolution, and decommission-fence behavior
is implemented and reviewed before the dev-only real
SQLite-to-record-encryption-to-key-service composition target runs; that target
has no adapter-to-repository runtime edge.
Required-authority outages fail closed and consume availability budget for
eligible traffic until recovery or acknowledged routing withdrawal.

HR does not own payroll calculation/disbursement, accounting, workflow
execution, audit-chain persistence, IAM/PDP, Data/Storage/Gateway engines,
notification delivery, or deployment infrastructure. Those effects cross
HR-owned ports and replaceable adapters.
