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

- The facade authenticates, authorizes, meters, admits, and translates one
  versioned semantic contract. It is a control-plane surface, never a serving
  authorization shortcut.
- The engine owns catalog descriptors, tenant bindings, projection generations,
  evidence coverage/manifests, target acknowledgement state, and idempotency.
- A pack-source adapter supplies bounded bytes and signature provenance. The
  engine never reads root `packs/` from a facade request.
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
source pack id/version/digest and schema revision
validity interval, predecessor and entry generation
registry generation
state: PREPARED | ACTIVE | SUPERSEDED | REVOKED
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
| Registry entries per admitted pack | n/a | 4,096 |
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
4. The signing preimage is a fixed binary frame: ASCII domain
   `oyatie.compliance.pack.v1` plus NUL, followed by field-number-ordered
   fields encoded as a `u32` big-endian byte length and then the exact bytes
   for pack id, canonical version,
   schema revision, plane, sorted dimensions, validity interval, signer key id,
   key generation, and the 32-byte payload digest. Integers are fixed-width
   big-endian; unknown/duplicate fields and trailing bytes are invalid.
5. The detached Ed25519 signature covers that preimage, not protobuf, JSON,
   YAML, Markdown, or re-encoded payload bytes. Golden preimage bytes are part
   of the contract.
6. Verification resolves a trusted 32-byte public key by namespace, key id,
   and key generation through the key-resolution port. The receipt binds key
   digest, validity/revocation generation, verified preimage digest, and trusted
   Cell interval. A key carried by the envelope is never a trust source.
7. It accepts only v1 namespaces `us`, `eu`, `jp`, and `kr`; a package id is one
   namespace plus one granular instrument, never a combinatoric jurisdiction.
8. `plane` is `serving` or `control`. Every projection dimension maps to a
   versioned Cedar Principal/Action/Resource/Context attribute; unknown free
   strings are invalid.
9. A serving fragment must be consumable by the agreed Policy compiler port.
   CaS records the compiler receipt but does not evaluate the policy.
10. Admission compare-and-swaps the catalog generation. A lower version, reused
   version with another digest, stale expected generation, revoked signer,
   expired interval, or unsupported schema fails before `CANDIDATE` is durable.
11. Only an `ADMITTED` immutable descriptor can be bound. Candidate cleanup may
   leak bounded work after failure; it cannot make the pack visible.

Malformed includes truncation, trailing/duplicate canonical fields, oversized
collections, invalid UTF-8 where text is required, digest/signature mismatch,
unknown enum values, missing projection schema, and cross-namespace references.
No parser fallback interprets the current Markdown/YAML scaffold as admitted
CaC.

</pack_admission>

<classification_registry>

## Exact-type registry state machine

The registry consumes `DataClassification` from the agreed
`data-classification` port. It does not define another enum, parser, error, or
wrapper.

```text
ABSENT -> PREPARED -> ACTIVE -> SUPERSEDED
                         \----> REVOKED
```

1. Preparation validates the exact classification value, canonical label,
   aliases, applicability, evidence obligations, source pack/schema/digest,
   trusted interval, expected prior entry/registry generations, and the hard
   limits above.
2. Canonical labels and aliases are unique within the registry snapshot. An
   alias cannot identify two values or shadow another canonical label.
3. One compare-and-swap publishes the immutable entry and advances the registry
   generation. The same idempotency key/fingerprint returns the same result;
   changed fingerprints conflict.
4. Supersession or revocation publishes a higher entry and registry generation.
   It never edits or deletes a prior entry and cannot activate a lower/equal
   conflicting source version.
5. Bindings, projections, manifests, and exports name the exact registry
   generation used. A stale or missing generation fails before publication;
   old evidence remains verifiable against immutable history.

Unknown classification values, duplicate/conflicting aliases, stale expected
generations, invalid intervals, unsupported schemas, source-digest mismatch,
and cross-tenant scope are typed failures with no mutation.

</classification_registry>

<binding_and_projection>

## Bind and publish

1. The facade verifies identity and Policy provenance and admits tenant work.
2. It resolves an immutable admitted descriptor and current binding generation.
3. The engine validates expected generation, scope, cell/capability
   compatibility, trusted interval, idempotency fingerprint, and requested
   target set before writing `PREPARED`.
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
4. Export admission durably records a bounded job and idempotency result. The
   worker writes immutable output through the Storage facade, verifies its
   digest, obtains required Audit evidence, and atomically publishes
   `EXPORTED` with the Storage receipt.
5. Retry resumes the same generation. Cleanup failure leaks unreachable export
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
catalog, binding, projection and schema generations
pack digest and signer/key generation
trusted Cell interval where validity matters
```

Missing, forged, expired, cross-tenant, stale, or ambiguous context fails before
state change or response data. Tenant zero uses the same contract. Internal
traffic uses mTLS; secret and key references are tenant-scoped and rotatable.

Admission is bounded by cell, tenant, operation, requests, bytes, concurrency,
in-flight memory, parser work, projection fan-out, and export jobs. Catalog
reads, mutations, reconciliation, evidence scans, and export work have separate
queues. Overload rejects early with typed retry information.

</security_and_admission>

<persistence_and_recovery>

## Authority and restore

Catalog, binding, projection, manifest, and idempotency state must use an owned
durable records contract with engine commit ordinals and consensus-backed
authority inside bounded cells. Compliance consumes trusted Cell intervals for
validity and lease decisions; wall time is not a generation.

Snapshots bind the last commit ordinal, schema versions, catalog/binding roots,
projection acknowledgements, evidence cursors, and manifest digests. Recovery
validates framing, checksums, monotonic generations, and referential integrity
before serving. Ambiguous or corrupt state is quarantined; it is never repaired
by accepting a lower pack or binding version.

Until that persistence port is agreed and implemented, L3d remains an unrouted
in-memory/deterministic oracle and cannot claim acknowledged durability,
availability, snapshot/restore, or production CaS service. Corrupt-WAL,
snapshot, quorum-loss, process-death durability, and recovery evidence first
become executable in the later persistence slice; they are not acceptance
evidence for an in-memory lane.

</persistence_and_recovery>

<observability>

## Required signals

Telemetry exposes request/correlation ids, admission and queue time, catalog and
binding generation, stale/malformed refusal class, signer/schema failures,
projection publication/ack lag, missing/reordered target receipts, evidence
coverage/gaps, Audit cursor lag, export age/bytes, recovery state, per-tenant
work, and unit cost.

The numeric objectives are in `PRD.md`. Aggregate throughput or current unit
tests cannot substitute for tail latency, gap detection, isolation, restore,
and fault evidence.

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
  catalog/binding generations, rollback attempts, and concurrent bind/revoke.
- Process death before and after candidate, prepare, Audit receipt, active
  publication, projection target acknowledgement, manifest completion, export
  write, and export publication.
- Lost, duplicated, delayed, and reordered Policy compiler, Audit source,
  projection target, and Storage export receipts.
- Forged/expired authorization, cross-tenant identifiers, revoked keys, Cell
  clock rollback/widening, Audit outage, and noisy tenants.
- Corrupt snapshots, missing Audit ranges, schema N/N+1 and downgrade barriers,
  quorum loss, cell loss, restore, and repeated replay.

Any malformed/stale admission, permit/forbid decision emitted by CaS,
cross-tenant access, complete manifest with a gap, acknowledged binding loss,
or target application under the wrong generation is a hard failure regardless
of latency or availability.

</fault_model>
