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
unresolved authorizations. Before Remove, SQLite durably records an immutable
known-input removal plan that binds proof/fences, fixed `Quarantine` or `Delete`
disposition and manifest, preallocated retirement fence, distinct scoped
provider/local ids, and the exact Remove request. A sole-member provider result
is `RetirementHandoffReady`, a durable, queryable signed handoff; SQLite
persists that handoff and a separate post-handoff Begin plan with exact handoff
bytes/authenticator, Begin id, fence id, and request digest before it calls
Begin. It similarly persists
post-Begin Complete, post-terminal disposition, and post-storage completion
plans before each later side effect. The repository performs typed provider
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
