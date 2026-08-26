---
doc_class: Owner-SPEC
owner: compliance
status: Target
date: 2026-08-26
---

# Compliance behavior and contract

<maturity>

The behavior below is the normative destination. Current retention and
off-charter Rust models, Markdown/YAML packs, Cedar text, Helm/Terraform, and
OpenSLO files do not implement it. A stage becomes available only when its
engine, contract, fault evidence, SLO signals, and independent promotion review
land.

</maturity>

<topology>

## Control and evidence planes

```text
signed root packs data ---- verified pack-source adapter
                                      |
                                      v
Connect CaS facade -> catalog/binding/projection engine -> Policy compiler port
          |                   |                 |
          |                   |                 +-> Audit/Data/Storage projections
          |                   +-> Audit evidence references
          +-> bounded export jobs -> Storage export bytes
```

- `compliance/facade/cas-app` is the D-8 process. Its `src/main.rs` composes the
  versioned Connect service with accepted adapters, while its library keeps
  handlers testable. It authenticates, authorizes, meters, admits, and
  translates one semantic contract; it is a control-plane surface, never a
  serving authorization shortcut or in-process Gateway plugin.
- The engine owns immutable catalog history/current pack heads, immutable
  registry history/current generation, tenant bindings, projection generations,
  evidence coverage/manifests, target acknowledgement state, durable export
  jobs/outcomes, catalog-to-registry reconciliation, and idempotency.
- A pack-source adapter supplies bounded envelope and payload bytes. The engine
  invokes the owner-local `pack-auth` port for trusted-key resolution and
  verification; callers cannot construct or assert a verification receipt.
  The engine never reads root `packs/` from a facade request.
- Policy remains the only evaluator and compiler of serving snapshots. CaS can
  preview attachment/projection but cannot return permit/forbid.
- Audit remains event authority; Data, Storage, and Audit remain authorities for
  their own retention application; Storage may hold immutable export bytes.

</topology>

<identity_and_versions>

## Canonical records

A catalog descriptor contains at least:

```text
pack_id: namespace/instrument
semantic_version, schema_revision, plane
projection_dimensions: principal/action/resource/context attributes
content_digest, signature/key_generation, validity_interval
policy and pre-ACK Audit receipt bindings
key-use authorization ordinal and receipt digest when admitted
state: CANDIDATE | ADMITTED | REVOKED | SUPERSEDED
catalog_generation
```

A tenant binding contains:

```text
tenant_id, binding_id, pack_id, admitted digest/version
scope selector, expected prior generation, binding generation
policy/schema/classification-registry revisions
effective interval, idempotency key/fingerprint
authorization and pre-ACK audit receipt bindings
state: PREPARED | ACTIVE | SUPERSEDED | REVOKED
```

A projection contains:

```text
tenant_id, target owner, projection kind and generation
pack/binding id, version, digest, schema revision
classification selector, obligations, effective interval
predecessor digest, payload digest, publication state
target acknowledgement generation/status
```

An evidence manifest contains:

```text
tenant_id, pack/binding/projection generations
required evidence classes and intervals
Audit source cursor/range and verified item digests
gaps, exclusions and reasons
manifest/export generation and digest
state: BUILDING | INCOMPLETE | COMPLETE | EXPORTED
```

A classification-registry entry contains:

```text
registry_id, exact DataClassification value
canonical label and bounded aliases
applicability and evidence obligations
source pack id/version/content digest, descriptor digest and schema revision
admitted catalog generation and observed current pack-head generation
validity interval, predecessor and entry generation
registry generation
policy and pre-ACK Audit receipt bindings
state: PREPARED | ACTIVE | SUPERSEDED | REVOKED
```

An export job contains:

```text
tenant_id, export_job_id, manifest generation and digest
catalog, registry, binding and projection generations
idempotency key/fingerprint and immutable outcome
policy and pre-ACK Audit receipt bindings
expected job generation and bounded attempt state
state: QUEUED | WRITING | PUBLISHED | FAILED_RETRYABLE | CANCELLED
Storage receipt and output digest when published
```

Identifiers are tenant-scoped. Versions never decrement. Retrying the same
idempotency key and fingerprint returns the same outcome; changing the
fingerprint conflicts. Rebinding or rollback publishes a higher generation and
never rewrites an admitted descriptor or prior evidence manifest.

</identity_and_versions>

<hard_limits>

## V1 protocol ceilings

These are hard interoperability and allocation ceilings, not production
capacity claims. A cell profile may lower them but cannot raise them without a
new protocol version.

| Surface | Default | Hard maximum |
|---|---:|---:|
| Encoded pack envelope | n/a | 4 MiB |
| Raw Cedar+IR payload | n/a | 3 MiB |
| Decode nesting depth | n/a | 32 |
| Decoded fields/nodes | n/a | 16,384 |
| Pack fragments or obligations | n/a | 4,096 each |
| Projection dimensions | n/a | 64 |
| Canonical id/key/namespace bytes | n/a | 128 each |
| Semantic-version bytes | n/a | 64 |
| Alias bytes / aliases per registry entry | n/a | 64 / 32 |
| Registry entries per immutable admitted descriptor generation | n/a | 4,096 |
| Projection targets per binding | 4 | 16 |
| List page entries | 100 | 1,000 |
| Pagination-token bytes | n/a | 4 KiB |
| Idempotency-key bytes | n/a | 128 |
| Parser in-flight memory per request | n/a | 16 MiB |
| Admitted queued operations per tenant/cell | 100 / 10,000 | 100 / 10,000 |
| Concurrent export jobs per tenant | 2 | 8 |
| Queued export jobs per tenant/cell | 100 / 10,000 | 100 / 10,000 |
| Evidence references per export / export bytes | n/a | 100,000 / 10 GiB |

The request fingerprint and SHA-256 digest are exactly 32 bytes; an Ed25519
signature is exactly 64 bytes and verification key is exactly 32 bytes.
Idempotency outcomes remain addressable for 30 days after terminal completion;
active binding and export generations remain durable for their full lifetime.
Overflow uses checked arithmetic. Limit-plus-one, allocation overflow, invalid
UTF-8, and a configured value above a hard maximum return a stable typed error
before candidate, queue, or binding mutation.

</hard_limits>

<pack_admission>

## Bounded fail-closed admission

1. The adapter enforces the v1 byte, nesting, item-count, identifier, and memory
   ceilings before materializing the envelope.
2. Identifiers are canonical printable ASCII; versions are canonical SemVer
   2.0 without leading-zero or equivalent alternate spellings. Lists whose
   semantics are sets are lexically sorted and duplicates are refused.
3. `payload_digest` is SHA-256 over the exact raw Cedar+IR payload bytes. Its
   external label is `sha256:` plus 64 lowercase hexadecimal digits.
4. The signing preimage is the following complete v1 binary grammar. The fixed
   prefix is the exact 25 ASCII bytes `oyatie.compliance.pack.v1`, one `0x00`
   byte, `frame_version:u8 = 0x01`, and `field_count:u8 = 0x0b`. Exactly eleven
   required fields follow once each in strictly increasing tag order. Every
   field is `tag:u8 || type:u8 || length:u32_be || value[length]`; v1 has no
   optional field. Type codes are `ASCII=0x01`, `U64_BE=0x02`,
   `I64_BE=0x03`, `ENUM8=0x04`, `DIMENSION_SET=0x05`, and
   `DIGEST32=0x06`.

   | Tag | Type and exact value |
   |---:|---|
   | `0x01` | `ASCII`: namespace, 1..128 printable-ASCII bytes |
   | `0x02` | `ASCII`: instrument, 1..128 printable-ASCII bytes |
   | `0x03` | `ASCII`: canonical SemVer, 1..64 printable-ASCII bytes |
   | `0x04` | `U64_BE`: schema revision, length 8, value greater than zero |
   | `0x05` | `ENUM8`: plane, length 1; `serving=0x01`, `control=0x02` |
   | `0x06` | `DIMENSION_SET`: collection grammar below |
   | `0x07` | `I64_BE`: inclusive `not_before_unix_ms`, length 8 |
   | `0x08` | `I64_BE`: exclusive `not_after_unix_ms`, length 8 and greater than tag `0x07` |
   | `0x09` | `ASCII`: signer key id, 1..128 printable-ASCII bytes |
   | `0x0a` | `U64_BE`: signer key generation, length 8, value greater than zero |
   | `0x0b` | `DIGEST32`: payload digest, length 32 |

   Printable ASCII is byte range `0x21..=0x7e`; NUL, whitespace, Unicode, and
   alternate text normalization are invalid. `I64_BE` is signed two's-
   complement Unix milliseconds. The dimension-set value is `count:u16_be`
   followed by exactly `count` elements, where `0 <= count <= 64`; zero is the
   exact two-byte value `0x0000` with no element bytes. Each element is
   `kind:u8 || name_length:u16_be || name[name_length]`.
   Kind codes are `principal=0x01`, `action=0x02`, `resource=0x03`, and
   `context=0x04`; each name is 1..128 printable-ASCII bytes. Elements are
   strictly sorted by `(kind_code, raw_name_bytes)` and duplicates are invalid.
   The outer `length` covers the count and every element byte.

   Wrong prefix/version/count, missing/unknown/duplicate/out-of-order tags,
   wrong type/width, count/length disagreement, noncanonical collection order,
   and trailing bytes are invalid. `payload_digest` is SHA-256 over the exact
   raw Cedar+IR payload bytes. No protobuf, JSON, YAML, Markdown, integer text,
   locale, host endianness, or re-encoding participates in the preimage.
5. The detached signature uses RFC 8032 pure Ed25519, not Ed25519ph or
   Ed25519ctx, and covers the complete canonical preimage bytes directly.
   `verified_preimage_digest` is SHA-256 over those same bytes.
   `key_digest` is SHA-256 over the exact 32 ASCII bytes
   `oyatie.compliance.ed25519.key.v1`, one `0x00` byte, and the raw 32-byte
   Ed25519 public key. Golden preimage, payload digest, key digest, preimage
   digest, public key, and signature bytes are part of the v1 contract.
6. The engine invokes `PackAuthenticator` through the owner-local `pack-auth`
   port. That port resolves a trusted 32-byte public key by namespace, key id,
   and key generation and returns verification evidence bound to the request,
   both digests, key validity/revocation generation, and the exact
   `cell_clock_api::Interval` obtained from composition's injected
   `cell_clock_api::Clock`. The engine never accepts caller-constructed time or
   evidence; production composition supplies the clock, crypto, and key
   adapters. Verification evidence is not catalog-commit authority, and a key
   carried by the envelope is never a trust source.
7. It accepts only v1 namespaces `us`, `eu`, `jp`, and `kr`; a package id is one
   namespace plus one granular instrument, never a combinatoric jurisdiction.
8. `plane` is `serving` or `control`. Every projection dimension maps to a
   versioned Cedar Principal/Action/Resource/Context attribute; unknown free
   strings are invalid.
9. A serving fragment must be consumable by the agreed Policy compiler port.
   CaS records the compiler receipt but does not evaluate the policy.
10. Before admit, revoke, or supersede, the engine validates a default-deny
   Policy decision and obtains a durable Audit receipt bound to actor or pack-
   namespace authority, exact transition, pack digest/version, expected catalog
   generation, request fingerprint, and Policy receipt digest. Denial, outage,
   forgery, expiry, or binding mismatch mutates nothing.
11. Immediately before an admit compare-and-swap, the engine executes the
   `<signer_revocation>` commit-authorization operation. The catalog mutation
   requires and atomically persists its exact receipt. A lower version, reused
   version with another digest, stale expected generation, revoked signer,
   expired interval, or unsupported schema fails before `CANDIDATE` is durable.
12. Revoke and supersede use the same Policy/Audit gate but do not mint new key-
   use authority. They append a higher immutable catalog generation and never
   rewrite the admitted receipt or descriptor.
13. Only an `ADMITTED` immutable descriptor can be bound. Candidate cleanup may
   leak bounded work after failure; it cannot make the pack visible.

Malformed includes truncation, trailing/duplicate canonical fields, oversized
collections, invalid UTF-8 where text is required, digest/signature mismatch,
unknown enum values, missing projection schema, and cross-namespace references.
No parser fallback interprets the current Markdown/YAML scaffold as admitted
CaC.

</pack_admission>

<trusted_time>

## Exact Cell interval and validity

The only trusted time input is the exact Rust type:

```text
cell_clock_api::Interval {
  earliest: SystemTime,
  latest: SystemTime,
  logical: u64,
}
```

It is returned by an injected `cell_clock_api::Clock`. Compliance defines no
clock trait, instant wrapper, interval DTO, midpoint, or caller request field
for trusted time. The same exact Rust type crosses pack-auth, core, and facade
composition; `logical` never turns the uncertain interval into a point
timestamp.

Signed `i64` Unix-millisecond endpoints convert to `SystemTime` by checked
addition or subtraction from `UNIX_EPOCH`. The negative magnitude, duration,
and `SystemTime` operation are checked; overflow is a typed refusal. The Cell
interval is invalid when `earliest > latest`.

The stable refusal identities are `TimeValidityError::EndpointOverflow`,
`ReversedInterval`, `NotYetValid`, `Expired`, `UncertainAcrossLowerBound`, and
`UncertainAcrossUpperBound`. Pack, key, binding, and registry callers preserve
these identities rather than collapsing them into a boolean or transport
status.

For every pack, key, binding, or registry validity window
`[not_before_unix_ms, not_after_unix_ms)`, acceptance is exactly:

```text
not_before_instant <= interval.earliest
interval.latest < not_after_instant
```

Equality at the inclusive lower bound is valid. Equality at the exclusive
upper bound is invalid. An interval partly before the lower bound, partly at or
after the upper bound, wholly before/after the window, reversed by clock
behavior, or widened across either boundary fails before candidate, binding,
registry, projection, or key-use mutation. Implementations cannot floor/
truncate endpoints to milliseconds, choose the midpoint/latest value, or
discard uncertainty to manufacture acceptance.

</trusted_time>

<signer_revocation>

## Linearizable signer commit authorization

Resolution returns a `ResolvedKeyFence` containing namespace, key id and
generation, key digest, observed revocation generation, verified preimage and
payload digests, request fingerprint, key validity window, and the exact Cell
interval used for verification. It proves what was checked; it does not fence a
later revoke.

After parsing, cryptographic verification, schema/Policy compilation checks,
default-deny Policy authorization, and durable pre-ACK Audit, Compliance calls:

```text
TrustedKeyResolver::authorize_catalog_commit(
  resolved_key_fence,
  expected_catalog_generation,
  policy_receipt_digest,
  audit_receipt_digest,
  current_cell_interval,
)
```

The production Secrets provider linearizes this operation and key revocation in
one per-key authoritative order. Success durably returns:

```text
KeyUseCommitReceipt {
  namespace, key_id, key_generation, key_digest,
  revocation_generation, key_use_ordinal,
  preimage_digest, payload_digest, request_fingerprint,
  expected_catalog_generation,
  policy_receipt_digest, audit_receipt_digest,
  authorized_interval,
}
```

The receipt is usable only for that exact idempotent catalog compare-and-swap
while the current Cell interval remains within both pack and key windows.
Catalog state persists the complete receipt binding atomically with admission.

The stable losing outcomes are:

```text
SignerCommitError::RevokedBeforeCommit
SignerCommitError::StaleFence
SignerCommitError::FenceExpired
SignerCommitError::BindingMismatch
SignerCommitError::ReceiptReplayMismatch
SignerCommitError::AuthorityUnavailable
```

Each carries the key generation and observed/current revocation generation when
available. If revocation linearizes first, authorization returns
`RevokedBeforeCommit` and no catalog mutation is legal. If authorization
linearizes first, the receipt is the admission operation's key-use
linearization point; a later key revocation does not rewrite that immutable
descriptor and uses a separately authorized/audited catalog revoke transition.
A failed catalog CAS may retry the receipt only with the same fingerprint and
expected generation; any changed binding is `ReceiptReplayMismatch`.

Deterministic barriers exercise both orders: resolve → revoke → authorize must
refuse, while resolve → authorize → revoke may commit only the receipt-bound
generation. Process death before/after authorization, duplicate delivery,
stale-generation retry, Cell-window expiry, forged receipt, and Secrets outage
leave no ambiguous catalog state. L3 fakes model this total order; production
evidence is unavailable until the L4 Secrets adapter proves the provider
linearization contract.

</signer_revocation>

<classification_registry>

## Exact-type registry state machine

The registry consumes `DataClassification` from the agreed
`data-classification` port. It does not define another enum, parser, error, or
wrapper.

```text
ABSENT -> PREPARED -> ACTIVE -> SUPERSEDED
                         \----> REVOKED
```

1. Request source fields are selectors, not authority. Before `PREPARE`, the
   engine resolves this immutable catalog fence from its current catalog root:

   ```text
   ResolvedAdmittedCatalogFence {
     pack_id, semantic_version, schema_revision,
     content_digest, descriptor_digest,
     admitted_catalog_generation,
     observed_pack_head_generation,
     state: ADMITTED
   }
   ```

   Pack id, version, schema revision, content digest, and descriptor digest must
   equal the caller-selected source exactly. The entry, default-deny Policy
   decision, and durable Audit receipt bind every fence field plus actor,
   tenant/scope, `PREPARE`, exact classification, expected entry/registry
   generations, and idempotency fingerprint. The engine then validates the
   exact classification value, canonical label, aliases, applicability,
   evidence obligations, trusted interval, and all hard limits.
2. Canonical labels and aliases are unique within the registry snapshot. An
   alias cannot identify two values or shadow another canonical label. The
   4,096-entry ceiling is counted atomically per immutable admitted descriptor
   generation, never across a caller-provided pack label.
3. Activation resolves the same current `ADMITTED` descriptor again and
   requires exact equality with the fence stored by `PREPARE`. It obtains
   separate Policy and durable Audit receipts bound to `ACTIVATE`, the prepared
   digest/generation, and that catalog fence. One compare-and-swap verifies both
   the expected registry generation and that the current catalog pack head
   still equals the observed admitted fence, persists the receipts, publishes
   the immutable entry, and advances the registry generation. The same
   idempotency key/fingerprint returns the same result; changed fingerprints
   conflict.
4. Catalog and registry mutations have a deterministic order. If catalog revoke
   or supersede changes the pack head before the registry CAS, prepare/activate
   loses without registry mutation. If activation commits first, the later
   catalog transition durably enqueues catalog-to-registry reconciliation in
   the same catalog transaction. The affected active registry generation is
   unavailable to new bind, projection, manifest, or export work until a
   registry-specific Policy-authorized, pre-ACK-Audited supersede or revoke
   publishes a terminal successor. History is immutable in either order.
5. Registry supersession or revocation independently obtains transition-bound
   Policy and durable Audit receipts, then publishes a higher entry and
   registry generation. It never edits or deletes a prior entry and cannot
   activate a lower/equal conflicting source version.
6. Bindings, projections, manifests, and exports name and revalidate the exact
   registry generation and its admitted source fence. A stale, missing, or
   reconciliation-blocked generation fails before publication; old evidence
   remains verifiable against immutable history.

Source resolution returns only these stable source-fence outcomes:
`RegistrySourceError::CatalogEntryNotFound`, `CatalogEntryNotAdmitted`,
`SourceIdentityMismatch`, `StaleCatalogGeneration`, `SourceRevoked`,
`SourceSuperseded`, or `FenceChangedBeforeCommit`. Unknown classification
values, duplicate/conflicting aliases, stale expected registry generations,
invalid intervals, unsupported schemas, cross-tenant scope, Policy denial/
forgery/expiry, Audit outage, and a receipt bound to another transition remain
separate typed failures. Every failure mutates neither registry history nor its
current generation; a catalog transition may only create its required durable
reconciliation record.

</classification_registry>

<binding_and_projection>

## Bind and publish

1. The facade verifies identity and Policy provenance and admits tenant work.
2. It resolves an immutable admitted descriptor and current binding generation.
3. The engine validates expected generation, scope, cell/capability
   compatibility, the exact Cell interval under `<trusted_time>`, idempotency
   fingerprint, and requested target set before writing `PREPARED`.
4. It derives projections deterministically from the descriptor and binding.
   The same inputs produce byte-identical canonical payload digests.
5. A privileged bind/publish obtains a durable Audit receipt bound to tenant,
   actor, pack digest/version, binding/projection generations, authorization,
   and fingerprint before acknowledgement.
6. One atomic publication makes the binding `ACTIVE`, projections target-ready,
   and the idempotent result visible. Partial target acknowledgement does not
   roll back the binding or masquerade as complete convergence.
7. Target adapters apply by compare-and-swap. Duplicate receipts converge;
   stale, skipped, reordered, or foreign-tenant generations fail and remain in
   reconciliation telemetry.

Retention projections describe obligations. They do not delete records/bytes,
create holds, or mutate Audit. Each target owner decides and records application
under its own authority and returns a generation-bound receipt.

`Preview` runs the same deterministic derivation against an immutable snapshot
but persists no binding/projection and returns no permit/forbid. It names the
catalog generation used so callers can detect staleness.

</binding_and_projection>

<evidence_and_export>

## Coverage and export

1. The engine freezes a manifest input generation: tenant binding/projections,
   required evidence classes/intervals, Audit cursor/range, schema, and
   classification registry.
2. It reads verified Audit references through an agreed port, checks tenant,
   event class, interval, source generation, digest, and ordering, and records
   explicit gaps.
3. `COMPLETE` is reachable only when every requirement has verified coverage.
   Exclusions are versioned obligations with reasons; they are not silent gap
   suppression.
4. Export admission durably records the bounded job defined under
   `<identity_and_versions>` and its idempotency outcome before queue
   acknowledgement. The frozen input binds the exact catalog, registry,
   binding, projection, and manifest generations plus Policy/Audit receipt
   digests. Admission and every state transition compare the expected job
   generation; queue capacity and the job record commit atomically.
5. The worker writes immutable output through the Storage facade, verifies its
   digest, obtains required Audit evidence, and atomically publishes `EXPORTED`
   with the Storage receipt. Process death before queue commit returns no
   acknowledgement; death after it resumes the same durable job. Death around
   Storage publication reconciles the immutable receipt before retry, so one
   idempotency identity cannot publish two outcomes.
6. Retry resumes the same generation. Cleanup failure leaks unreachable export
   bytes instead of publishing an incomplete or mismatched manifest.

An export is evidence packaging, not a trust portal or eDiscovery search
product. Audit records remain authoritative even after export.

</evidence_and_export>

<security_and_admission>

## Fail-closed context

Every disclosure or mutation binds:

```text
tenant and verified principal
operation and resource scope
policy issuer/revision/audience/expiry
request, correlation and idempotency identity
catalog/current pack-head, registry, binding, projection and schema generations
pack content/descriptor digests and signer/key generation
exact `cell_clock_api::Interval` from injected `Clock` where validity matters
```

Missing, forged, expired, cross-tenant, stale, or ambiguous context fails before
state change or response data. Pack admit/revoke/supersede; registry prepare/
activate/supersede/revoke; bind; projection publish; and export admission each
require an operation/generation-bound Policy decision and durable pre-ACK Audit
receipt. One transition's receipt cannot authorize another. Tenant zero uses
the same contract. Internal traffic uses mTLS; secret and key references are
tenant-scoped and rotatable.

Admission is bounded by cell, tenant, operation, requests, bytes, concurrency,
in-flight memory, parser work, projection fan-out, and export jobs. Catalog
reads, mutations, reconciliation, evidence scans, and export work have separate
queues. Overload rejects early with typed retry information.

</security_and_admission>

<persistence_and_recovery>

## Authority and restore

Compliance's complete durable root set is:

```text
immutable catalog history and current pack heads
immutable registry history and current registry generation
bindings, projections and target acknowledgements
evidence manifests and source cursors
export admission, job, publication and idempotency outcomes
catalog-to-registry reconciliation work and terminal outcomes
Policy, pre-ACK Audit, signer and Storage receipt bindings for those records
```

That root set must use Data's accepted engine-neutral `data-records` provider
through Compliance's narrower owner-local `catalog-store` port and
`catalog-store-data` adapter. Compliance does not fork the generic records
contract or import a Data core. The provider must expose commit ordinals and
consensus-backed authority inside bounded cells. Compliance consumes trusted
Cell intervals for validity and lease decisions; wall time is not a generation.

Snapshots bind the last commit ordinal, schema versions, and every root above;
snapshot creation cannot omit queued/running export jobs or reconciliation.
Recovery validates framing, checksums, monotonic generations, immutable catalog
and registry history, current-head references, export-job/output receipts, and
cross-generation referential integrity before serving. It resumes accepted
export jobs by immutable idempotency identity and completes or blocks catalog-
to-registry reconciliation before affected state is available. Ambiguous or
corrupt state is quarantined; it is never repaired by accepting a lower pack,
registry, binding, job, or reconciliation generation.

Until that persistence port is agreed and implemented, L3d remains an unrouted
in-memory/deterministic oracle and cannot claim acknowledged durability,
availability, snapshot/restore, or production CaS service. Corrupt-WAL,
snapshot, quorum-loss, process-death durability, and recovery evidence first
become executable in the later persistence slice; they are not acceptance
evidence for an in-memory lane.

An early Gateway service registration is structurally disabled and cannot
receive traffic. The CaS process may publish readiness or bind its internal
listener only after declarative composition, durable restore, production pack/
key, Policy, Audit source/sink, projection-target, export-store, exact Cell
clock, and signer-fence checks pass. Activation additionally requires the same
join and its fail-closed outage evidence. No in-memory store or fake dependency
satisfies either gate, and process existence is not route eligibility.

</persistence_and_recovery>

<process_lifecycle>

## CaS process boot and drain

```text
UNCOMPOSED -> RECOVERING -> READY -> DRAINING -> STOPPED
       \-----------> FAILED <-----------/
```

`compliance-cas-app` starts from declarative cell configuration and accepts no
CLI authority. `RECOVERING` validates config/version, constructs the accepted
durable store and every Pack/Secrets, Policy, Audit, projection, export, Cell,
and Connect adapter, completes restore, and proves the signer commit-fence
operation. It binds the internal listener and publishes readiness only after all
checks pass. Gateway registration remains independently disabled until L4a-R.

Stable boot refusals are:

```text
ProcessBootError::Uncomposed
ProcessBootError::MalformedConfiguration
ProcessBootError::MissingAdapter
ProcessBootError::RestoreUnready
ProcessBootError::SignerFenceUnavailable
ProcessBootError::ListenerBind
ProcessBootError::DependencyLost
ProcessBootError::DrainTimeout
```

Before readiness they produce no listener and a nonzero process result. After
readiness, mandatory dependency loss atomically withdraws readiness, stops new
admission, drains bounded in-flight work, and exits; it never switches to a fake
or memory authority. Restart replays durable catalog and registry history,
idempotency/receipt state, accepted export jobs, and catalog-to-registry
reconciliation before readiness. A second instance cannot bypass the catalog/
store authority epoch.

Contract tests cover cold start, every missing adapter, corrupt configuration,
failed restore, signer-fence refusal, bind conflict, dependency loss at each
request phase, cancellation, bounded drain timeout, process death before/after
listener bind and mutation commit, restart replay, and stale instance fencing.

</process_lifecycle>

<observability>

## Required signals

Telemetry exposes request/correlation ids, admission and queue time, catalog and
binding generation, stale/malformed refusal class, signer/schema failures,
projection publication/ack lag, missing/reordered target receipts, evidence
coverage/gaps, Audit cursor lag, export age/bytes, recovery state, per-tenant
work, and unit cost.

The numeric objectives are in `PRD.md`. Their sole handwritten Rust IR lives in
the accepted `compliance/ports/slo` package. The IR package has no materializer
dependency; the accepted Observability materializer consumes it through the
one-way Cargo/Buck edge frozen in `PLAN.md`. Only materializer-produced
`*.generated.openslo.yaml` files may live under
`compliance/observability/slos`. Aggregate throughput or current unit tests
cannot substitute for tail latency, gap detection, isolation, restore, and
fault evidence.

</observability>

<fault_model>

## Required campaigns

Campaigns are gated by the subsystem that makes them executable. Contract,
admission, registry, binding, and facade oracles run bounded input,
authorization, generation, replay, and receipt campaigns first. Process-death,
durable-barrier, corrupt-snapshot, quorum, network, restore, and mixed-version
campaigns are future promotion evidence after the corresponding persistence,
transport, and adapter decision lands.

- Truncated, oversized, deeply nested, duplicate-field, unknown-schema,
  unknown-plane/dimension, digest-mismatched, unsigned, revoked, expired, and
  cross-namespace packs.
- Lower versions, equal versions with different digests, stale expected
  catalog/binding generations, rollback attempts, concurrent bind/revoke, and
  both signer resolve/revoke/commit linearization orders.
- Registry prepare/activate interleaved immediately before and after catalog
  revoke/supersede, including stale source fences, exact typed loser outcomes,
  durable reconciliation, and blocked downstream use.
- Process death before and after candidate, registry prepare/activate, Audit
  receipt, active publication, projection target acknowledgement, manifest
  completion, export-job/idempotency commit, queue acknowledgement, export
  write, and export publication.
- Lost, duplicated, delayed, and reordered Policy compiler, Audit source,
  projection target, and Storage export receipts.
- Forged/expired or transition-mismatched Policy/Audit evidence, cross-tenant
  identifiers, revoked keys, Secrets fence outage/staleness/replay, Cell
  clock rollback/widening, checked Unix-ms overflow, intervals before/on/across
  inclusive/exclusive validity boundaries, Audit outage, and noisy tenants.
- Corrupt snapshots, missing registry history/current generation, missing
  accepted export jobs or reconciliation work, cross-generation reference
  corruption, missing Audit ranges, schema N/N+1 and downgrade barriers,
  quorum loss, cell loss, interrupted restore, repeated replay, malformed
  process composition, cold-start dependency outage, listener bind failure,
  drain, cancellation, and process death.

Any malformed/stale admission, permit/forbid decision emitted by CaS,
cross-tenant access, complete manifest with a gap, acknowledged binding loss,
or target application under the wrong generation is a hard failure regardless
of latency or availability.

</fault_model>
