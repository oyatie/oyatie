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
content_digest, descriptor_digest, signature/key_generation, validity_interval
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
| Canonical privileged-request frame | n/a | 4 MiB |
| Opaque target provider receipt | n/a | 64 KiB |
| Alias bytes / aliases per registry entry | n/a | 64 / 32 |
| Registry entries per immutable admitted descriptor generation | n/a | 4,096 |
| Projection targets per binding | 4 | 16 |
| List page entries | 100 | 1,000 |
| V1 durable page-handle bytes | n/a | exactly 32 |
| Idempotency-key bytes | n/a | 128 |
| Parser in-flight memory per request | n/a | 16 MiB |
| Admitted queued operations per tenant/cell | 100 / 10,000 | 100 / 10,000 |
| Concurrent export jobs per tenant | 2 | 8 |
| Queued export jobs per tenant/cell | 100 / 10,000 | 100 / 10,000 |
| Evidence references per export / export bytes | n/a | 100,000 / 10 GiB |

The request fingerprint, descriptor digest, and SHA-256 digest are exactly 32
bytes; an Ed25519
signature is exactly 64 bytes and verification key is exactly 32 bytes.
Idempotency outcomes remain addressable for 30 days after terminal completion;
active binding and export generations remain durable for their full lifetime. A
page session lives for exactly 900,000 ms and retains its immutable snapshot
until safe expiry collection. Overflow uses checked arithmetic. Limit-plus-one,
allocation overflow, invalid UTF-8, and a configured value above a hard maximum
return a stable typed error before candidate, queue, or binding mutation.

</hard_limits>

<canonical_identity_and_pagination>

## Descriptor and request identity

`content_digest` is exactly the Pack v1 `payload_digest`.
`descriptor_digest` is exactly the 32-byte `verified_preimage_digest` from
`<pack_admission>`: SHA-256 over the complete canonical signed Pack v1 preimage,
including the `oyatie.compliance.pack.v1` domain. Admission copies that verified
value into immutable catalog state and the signer receipt; it never re-encodes a
catalog struct or accepts a caller digest. Catalog, registry, Policy, Audit, and
Secrets fences compare those exact bytes.

Every privileged request fingerprint is computed by the server after parsing,
authorization-context resolution, canonical validation, sorting, and authority
lookups. The public semantic request contains the idempotency key but no trusted
fingerprint. The frame begins with the exact 28 ASCII bytes
`oyatie.compliance.request.v1`, one `0x00`, `frame_version:u8 = 0x01`, an
operation code, and `field_count:u8`. Required fields follow once each in
strictly increasing tag order as
`tag:u8 || type:u8 || length:u32_be || value[length]`. Type codes are
`ASCII=0x01`, `U64_BE=0x02`, `I64_BE=0x03`, `DIGEST32=0x04`,
`ENUM16_BE=0x05`, `ASCII_SET=0x06`, and `U64_SET=0x07`. Integers have exact
8-/2-byte widths. ASCII uses the Pack v1 printable/canonical rules. A set is
`count:u16_be` followed by `length:u16_be || value` elements, strictly sorted
by raw encoded bytes with no duplicate; U64 elements have length 8. Empty sets
are exactly `0x0000`. Unknown/missing/duplicate/out-of-order tags, wrong type or
width, length disagreement, alternate normalization, and trailing bytes are
invalid.

All operations require these common fields:

| Tag | Field | Type |
|---:|---|---|
| `0x01` | authority scope, exactly `tenant/<id>` or `pack-namespace/<id>` | ASCII |
| `0x02` | verified principal subject | ASCII |
| `0x03` | idempotency key | ASCII |
| `0x04` | request schema revision, greater than zero | U64_BE |

The remaining tag registry is global:

| Tag | Field | Type |
|---:|---|---|
| `0x10` | pack id | ASCII |
| `0x11` | descriptor digest | DIGEST32 |
| `0x12` | predecessor descriptor digest | DIGEST32 |
| `0x13` | successor descriptor digest | DIGEST32 |
| `0x14` | reason code | ENUM16_BE |
| `0x20` | registry id | ASCII |
| `0x21` | canonical DataClassification label | ASCII |
| `0x22` | prepared/active entry digest | DIGEST32 |
| `0x23` | successor entry digest | DIGEST32 |
| `0x24` | canonical label | ASCII |
| `0x25` | aliases | ASCII_SET |
| `0x26` | applicability selectors | ASCII_SET |
| `0x27` | evidence obligations | ASCII_SET |
| `0x28` / `0x29` | inclusive lower / exclusive upper Unix ms | I64_BE |
| `0x2a` / `0x2b` | expected entry / registry generation | U64_BE |
| `0x2c` | observed current pack-head generation | U64_BE |
| `0x30` | binding id | ASCII |
| `0x31` | registry generation | U64_BE |
| `0x32` | scope selectors | ASCII_SET |
| `0x33` | target owners | ASCII_SET |
| `0x34` | expected binding generation | U64_BE |
| `0x40` | projection payload digest | DIGEST32 |
| `0x41` | expected projection generation | U64_BE |
| `0x42` | target owner | ASCII |
| `0x43` | verified target-receipt digest | DIGEST32 |
| `0x44` | expected target-acknowledgement generation | U64_BE |
| `0x45` | source projection-publication fingerprint | DIGEST32 |
| `0x50` | export job id | ASCII |
| `0x51` | manifest digest | DIGEST32 |
| `0x52` / `0x53` | manifest / catalog generation | U64_BE |
| `0x54` | binding generation | U64_BE |
| `0x55` | projection generations | U64_SET |
| `0x56` | expected export-job generation | U64_BE |

Operation codes and their exact additional required fields are:

| Code | Operation | Field count | Required tags after common fields |
|---:|---|---:|---|
| `0x01` | pack admit | `0x07` | `0x10,0x11,0x53` |
| `0x02` | pack revoke | `0x08` | `0x10,0x11,0x14,0x53` |
| `0x03` | pack supersede | `0x08` | `0x10,0x12,0x13,0x53` |
| `0x10` | registry prepare | `0x10` | `0x11,0x20,0x21,0x24,0x25,0x26,0x27,0x28,0x29,0x2a,0x2b,0x2c` |
| `0x11` | registry activate | `0x0a` | `0x11,0x20,0x22,0x2a,0x2b,0x2c` |
| `0x12` | registry supersede | `0x0b` | `0x11,0x20,0x22,0x23,0x2a,0x2b,0x2c` |
| `0x13` | registry revoke | `0x0b` | `0x11,0x14,0x20,0x22,0x2a,0x2b,0x2c` |
| `0x20` | binding activate | `0x0d` | `0x11,0x28,0x29,0x30,0x31,0x32,0x33,0x34,0x53` |
| `0x21` | projection publish | `0x0b` | `0x30,0x31,0x33,0x40,0x41,0x53,0x54` |
| `0x22` | binding supersede | `0x0e` | `0x12,0x13,0x28,0x29,0x30,0x31,0x32,0x33,0x34,0x53` |
| `0x23` | binding revoke | `0x0a` | `0x11,0x14,0x30,0x31,0x34,0x53` |
| `0x24` | target acknowledge | `0x0e` | `0x30,0x31,0x40,0x41,0x42,0x43,0x44,0x45,0x53,0x54` |
| `0x30` | export admit | `0x0c` | `0x31,0x50,0x51,0x52,0x53,0x54,0x55,0x56` |

Reason codes are `administrative=0x0001`, `security=0x0002`,
`superseded=0x0003`, `source_revoked=0x0004`, and `legal=0x0005`; free text is
not fingerprint input. Binding activate resolves tag `0x11` from the selected
immutable `ADMITTED` descriptor and verifies request preconditions `0x31`,
`0x34`, and `0x53` against current registry, binding, and catalog state.
Binding supersede resolves tag `0x12` from the current active binding and tag
`0x13` from the selected immutable successor descriptor; binding revoke
resolves tag `0x11` from the current active binding. Scope, target, and interval
values are bounded request semantics, but they become fingerprint input only
after canonical normalization. Supersede and revoke each require a new
idempotency key and fresh Policy/Audit receipts; neither inherits the activate
identity. Projection publish resolves tags `0x31`, `0x53`, and `0x54` from the
binding's immutable registry, catalog, and binding generations and resolves
its target set, payload digest, and expected projection generation from the
candidate and request precondition before CAS.

A target acknowledgement is operation `0x24`, never an internal continuation
of projection publication. It authenticates the target service principal and
passes the opaque provider receipt through the owner-local `projection-target`
port. Only a verified result may supply this canonical record:

```text
VerifiedTargetAcknowledgement {
  tenant_id, authenticated_target_principal, target_owner,
  binding_id, binding_generation,
  projection_generation, projection_payload_digest,
  provider_receipt_digest,
}
```

The server resolves tags `0x30`, `0x31`, `0x40`, `0x41`, `0x42`, `0x43`,
`0x53`, and `0x54` from that result plus immutable binding/projection state and
requires exact equality between them. Tags `0x31` and `0x53` are the registry
and catalog generations frozen into the binding that produced the projection.
Tag `0x44` is the request's compare-and-swap precondition against current
acknowledgement state. Tag `0x45` is loaded from the stored successful `0x21`
outcome which published that exact projection; it is causal input, not
authority for `0x24`. The verified principal must be authorized for the
resolved target owner and tenant. A caller cannot supply a trusted target
owner, receipt digest, payload digest, publication fingerprint, or generation
by copying fields around an unverified receipt.

These thirteen codes are the exhaustive v1 privileged-mutation set: pack
admit/revoke/supersede; registry prepare/activate/supersede/revoke; binding
activate/supersede/revoke; projection publish; target acknowledge; and export
admit. Catalog `CANDIDATE`, binding `PREPARED`, reconciliation enqueue,
manifest computation, export worker progression, and pagination collection
are not independently callable privileged transitions and have no request
operation code. An internal step may only reference, never adopt as fresh
authority, its committed initiating operation through:

```text
InheritedTransitionIdentity {
  initiating_operation_code, initiating_request_fingerprint,
  authority_scope, initiating_principal,
  record_kind, record_id, expected_record_generation,
  authenticated_internal_actor,
}
```

The step must be fixed by the initiating state machine, retain the immutable
semantic input, and compare the stored authority, fingerprint, record, and
generation before CAS. Binding prepare/commit is one unacknowledged internal
phase of its binding operation; catalog-to-registry reconciliation enqueue is
atomic with the triggering catalog transition and requires a fresh registry
supersede/revoke operation to resolve; export workers are fenced by the durable
export-admit job and expected job generation. None may change tenant, resource,
operation, or semantic input, invoke another privileged edge, or use the
initiating fingerprint as a bearer token. Stable failures are
`InheritedTransitionError::{InitiatingOutcomeNotCommitted,AuthorityMismatch,
ActorNotAllowed,FingerprintMismatch,StaleGeneration,ReceiptBindingMismatch}`
and mutate no authoritative state.

Digest fields are resolved from immutable authoritative records or a verified
provider result, not copied from requests. The server computes
`request_fingerprint = SHA-256(frame)`. Including authority, principal, and
idempotency identity confines equality to that replay slot. The value is never
a telemetry label. Policy, Audit, Secrets commit authorization, compare-and-
swap, and durable idempotency outcomes all consume the same computed bytes.
Another operation code, normalized semantic value, expected generation, or
schema revision must conflict. A v2 request uses another domain/version rather
than ignoring N+1 fields under v1.

The invocation order is fixed: authenticate; perform bounded parsing; resolve
authority and immutable fields (and verify a target receipt for `0x24`);
normalize and derive the frame; obtain a current default-deny Policy decision
bound to the computed fingerprint; inspect or reserve the idempotency slot;
obtain a transition-bound durable pre-ACK Audit receipt; then atomically
recheck generations and persist the state change, both receipts, and outcome.
No response discloses authoritative state before authentication and Policy.
An exact committed retry must reauthenticate and reauthorize, then returns the
stored outcome without another CAS; its stored Audit receipt remains the
pre-ACK transition evidence. An exact pending retry resumes only the same
frame. A retryable dependency failure may leave that bounded pending slot but
no product mutation. A fresh semantic transition, including supersede,
revoke, or acknowledgement after publication, uses a fresh idempotency key.

Each idempotency slot durably stores the authority scope, principal,
idempotency key, frame version, operation code, bounded canonical frame bytes,
computed fingerprint, `PENDING | COMMITTED` state, exact Policy/Audit receipt
bindings, and terminal outcome when committed. This permits byte equality to
be checked independently from digest equality and survives reopen/restore.
The common stable identity failures are
`TransitionIdentityError::{OperationMismatch,IdempotencyConflict,
FingerprintCollision}`. Same authority/principal/key/code/fingerprint is a
lawful replay only when the canonical frame bytes also match; the same slot
under another code is `OperationMismatch`, the same code with changed frame is
`IdempotencyConflict`, and unequal stored/incoming frames with an equal digest
fail as `FingerprintCollision`. Binding operations add
`BindingTransitionError::{StaleCatalogGeneration,StaleRegistryGeneration,
StaleBindingGeneration,BindingStateConflict,DescriptorFenceChanged}`. Target
acknowledgement adds
`TargetAcknowledgementError::{DuplicateReceipt,StaleBindingGeneration,
StaleProjectionGeneration,StaleAcknowledgementGeneration,ReceiptNotVerified,
ReceiptBindingMismatch,TargetAuthorityMismatch,
PublicationFingerprintMismatch}`. An exact same-key acknowledgement replay
returns the stored result; the same verified provider receipt under another
key is `DuplicateReceipt` and cannot advance acknowledgement generation.
Malformed or forged provider evidence is the non-oracular
`ReceiptNotVerified`; cross-tenant, wrong-owner/principal, wrong payload or
binding, stale/reordered generation, and publication-identity swaps map to the
specific remaining variants before mutation. Policy/Audit failures remain
their separately typed port outcomes.

## Durable pagination session

The public v1 page token is exactly a 32-byte opaque `PageHandle`. It carries no
self-asserted tenant, cursor, filter, or signature. On the first list request,
the server opens one immutable catalog-store snapshot and atomically persists:

```text
PaginationSession {
  page_handle,
  tenant_id, verified_principal, list_kind,
  snapshot_commit_ordinal, catalog_generation, registry_generation,
  normalized_filter_frame, exclusive_cursor, fixed_page_size,
  schema_revision, created_cell_interval, expires_at_unix_ms,
}
```

The handle is minted by the production `pagination-session-data` adapter from
the accepted CSPRNG, never from tenant/filter/cursor bytes. It retries a random
collision at most three times, then returns
`PageTokenError::CollisionExhausted`; a CSPRNG read failure returns
`EntropyUnavailable`. No page is returned before the record and snapshot lease
are durable. `expires_at_unix_ms` is checked signed-Unix-ms addition of
`interval.latest + 900_000`; conversion/addition overflow returns
`TimeUncertain`. A subsequent request is valid only when the complete fresh Cell
interval remains before expiry. Subsequent requests supply only the handle;
tenant/principal come from verified context and filter/page size remain those in
the record. The store uses constant-time handle comparison and one redacted
`NotFoundOrForeign` result for malformed, unknown, forged, and foreign-tenant/
principal handles.

The stable failures are `PageTokenError::{NotFoundOrForeign,Expired,
TimeUncertain,SnapshotUnavailable,StoreUnavailable,EntropyUnavailable,
CollisionExhausted}`. A 31-/33-byte, unknown, forged, or foreign handle maps to
the same public `NotFoundOrForeign` result. A
session never advances to a different snapshot after process death or restore.
Its record and retained snapshot ordinal are part of the complete durable root,
point-in-time restore, N/N+1 migration, and garbage-collection proof. Collection
starts only after a fresh Cell interval proves expiry and no admitted page call
is in flight. V1 has no pagination signing key or rotation path; replacing the
durable handle with a stateless token requires a new protocol version and an
accepted Secrets generation/rotation/revocation/outage contract.

Production/reference encoders for descriptor and request identity share only
typed inputs. Golden, permutation, field/value corruption, operation-swap,
normalization, and N/N+1 vectors must match exactly for all thirteen operation
codes. The binding vectors cover activate/supersede/revoke with exact catalog,
registry, binding, descriptor, scope, target, and interval changes. Target-
acknowledgement vectors cover exact replay, another-key duplicate, receipt
corruption, target/principal/tenant swap, publication-fingerprint swap, and
stale binding/projection/acknowledgement generations. Internal-step vectors
swap initiator, actor, record, fingerprint, and generation and must return the
exact `InheritedTransitionError`. Page tests cover exact 32-byte and 31/33-byte
handles, collision exhaustion, bit corruption, tenant/principal swap, snapshot/
filter/page drift, expiry uncertainty, Cell loss, process death, snapshot
restore, and uniform redacted errors.

</canonical_identity_and_pagination>

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

This is the L3 semantic predicate, not proof of a production time source. The
current `cell_clock_api::Clock::now() -> Interval` and static-uncertainty
`NtpClock` cannot report chrony/source loss, measurement age, rollback, or an
unmeasured bound and therefore cannot satisfy production composition. Before
L4 process boot, Cell must accept a D-29 provider contract whose fallible read
result carries the exact interval plus measured-bound freshness/health
provenance and whose typed error refuses source loss, staleness, rollback, or an
unacceptable bound; its concrete adapter is selected by cell IR. The accepted
maximum age, source-generation semantics, typed source loss, and rollback
response are Cell types; Compliance does not wrap them. Any failed/stale read
maps to `TimeValidityError::TrustedClockUnavailable`, removes
readiness, and admits no new validity-dependent mutation. There is no process-
clock fallback.

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
  preimage_digest (= catalog descriptor_digest),
  payload_digest, request_fingerprint,
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

1. The facade authenticates the principal, resolves authority without
   disclosure, and applies the fixed `<canonical_identity_and_pagination>`
   Policy/Audit/idempotency order before admitting tenant work.
2. It resolves an immutable admitted descriptor and exact current catalog,
   registry, and binding generations. An activate is legal only from absent or
   its exact unacknowledged `PREPARED` attempt. A supersede is legal only from
   the named current `ACTIVE` generation and appends an immutable
   `SUPERSEDED` predecessor plus a higher active successor. A revoke is legal
   only from the named current `ACTIVE` generation and appends a higher
   `REVOKED` generation with no active successor.
3. Each activate/supersede/revoke validates its operation-table fields, scope,
   cell/capability compatibility, exact Cell interval under `<trusted_time>`,
   fresh transition identity, and requested target set. A hidden `PREPARED`
   record is part of that one operation's pending idempotency state; it is
   neither visible nor authority for another operation.
4. It derives immutable projection candidates deterministically from the
   descriptor and binding generation. The same inputs produce byte-identical
   canonical payload digests. A separate `projection publish` operation with
   its own Policy/Audit receipts and fresh idempotency identity makes that exact
   candidate target-ready; it cannot publish under a binding-operation receipt.
5. Every binding operation and projection publication obtains a durable Audit
   receipt bound to tenant, actor, exact operation code, pack descriptor,
   catalog/registry/binding/projection generations, Policy authorization, and
   fingerprint before its CAS and acknowledgement. State, receipts, and the
   durable idempotency outcome commit atomically.
6. A committed binding remains valid while projection publication or target
   application converges. Partial or absent target acknowledgement does not
   roll back the binding or masquerade as complete convergence. Revocation and
   supersession publish higher, explicit projection work; they never rewrite a
   prior applied generation.
7. Target adapters apply by compare-and-swap and return only a provider receipt
   which `projection-target` can verify. Recording that result is a fresh
   authenticated `target acknowledge` operation under the exact `0x24` frame,
   target principal, Policy decision, durable Audit receipt, expected binding/
   projection/acknowledgement generations, stored publication fingerprint, and
   verified receipt digest. Exact same-key retries return the stored outcome;
   same-receipt/different-key duplicates, stale, skipped, reordered, forged,
   wrong-owner, or foreign-tenant results return the stable typed errors above,
   advance no acknowledgement generation, and remain reconciliation telemetry.

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
activate/supersede/revoke; binding activate/supersede/revoke; projection
publish; authenticated target acknowledge; and export admit are exactly the
thirteen operations in the v1 table. Each requires an operation/generation-
bound Policy decision and durable pre-ACK Audit receipt in the frozen order.
One transition's operation, identity, or receipt cannot authorize another.
Internal substeps obey `InheritedTransitionIdentity` and cannot create a
fourteenth caller operation. Tenant zero uses the same contract. Internal
traffic uses mTLS; secret and key references are tenant-scoped and rotatable.

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
bindings, projections, publication fingerprints and target acknowledgements
evidence manifests and source cursors
export admission, job, publication and idempotency outcomes
catalog-to-registry reconciliation work and terminal outcomes
live pagination-session records and their retained snapshot ordinals
Policy, pre-ACK Audit, signer and Storage receipt bindings for those records
pending/terminal transition slots and inherited internal-step identities
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
and registry history, current-head references, all thirteen operation codes,
pending/terminal fingerprints, publication-to-acknowledgement receipt bindings,
inherited-step identities, export-job/output receipts, and cross-generation
referential integrity before serving. It also restores every
unexpired pagination record against the exact retained snapshot ordinal; a
missing or mismatched snapshot invalidates the session rather than moving its
cursor. It resumes accepted export jobs by immutable idempotency identity and
completes or blocks catalog-to-registry reconciliation before affected state is
available. Ambiguous or corrupt state is quarantined; it is never repaired by
accepting a lower pack, registry, binding, job, or reconciliation generation.

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
production-clock adapter/plant receipt, pagination-session adapter, and signer-
fence checks pass. Activation additionally requires the same join and its fail-
closed outage evidence. No in-memory store, static-uncertainty clock, or fake
dependency satisfies either gate, and process existence is not route
eligibility.

</persistence_and_recovery>

<process_lifecycle>

## CaS process boot and drain

```text
UNCOMPOSED -> RECOVERING -> READY -> DRAINING -> STOPPED
       \-----------> FAILED <-----------/
```

`compliance-cas-app` starts from declarative cell configuration and accepts no
CLI authority. `RECOVERING` validates config/version, constructs the accepted
durable store and every Pack/Secrets, Policy, Audit, projection, export,
pagination-session, Cell, and Connect adapter, completes restore, verifies a
fresh Cell measured-bound/health receipt, and proves the signer commit-fence
operation. It binds the internal listener and publishes readiness only after
all checks pass. Gateway registration remains independently disabled until
L4a-R.

Stable boot refusals are:

```text
ProcessBootError::Uncomposed
ProcessBootError::MalformedConfiguration
ProcessBootError::MissingAdapter
ProcessBootError::RestoreUnready
ProcessBootError::SignerFenceUnavailable
ProcessBootError::TrustedClockUnavailable
ProcessBootError::PaginationSessionUnavailable
ProcessBootError::ListenerBind
ProcessBootError::DependencyLost
ProcessBootError::DrainTimeout
```

Before readiness they produce no listener and a nonzero process result. After
readiness, mandatory dependency loss atomically withdraws readiness, stops new
admission, drains bounded in-flight work, and exits; it never switches to a fake
or memory authority. Clock-source loss or stale measurement performs that
withdrawal before another validity-dependent operation. Restart replays durable
catalog and registry history, idempotency/receipt state, accepted export jobs,
pagination sessions/snapshot leases, and catalog-to-registry reconciliation
before readiness. A second instance cannot bypass the catalog/store authority
epoch.

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
- Binding activate/supersede/revoke operation swaps, same-key changed frames,
  supersede/revoke reuse of activation identity, target acknowledgement reuse
  of publication identity, another-key duplicate target receipts, forged or
  wrong-target receipts, publication-fingerprint swaps, stale catalog/
  registry/binding/projection/acknowledgement generations, and inherited-step
  initiator/actor/record/generation swaps.
- Forged/expired or transition-mismatched Policy/Audit evidence, cross-tenant
  identifiers, revoked keys, Secrets fence outage/staleness/replay, Cell
  NTP/chrony loss, stale measured bounds, source-generation change, clock
  rollback/widening, checked Unix-ms overflow, intervals before/on/across
  inclusive/exclusive validity boundaries, Audit outage, and noisy tenants.
- Descriptor/request independent golden, permutation, corruption, operation-
  swap, and N/N+1 vectors; page-handle 31/32/33-byte, entropy collision, bit
  corruption, tenant/principal swap, expiry uncertainty, snapshot drift, and
  restore campaigns with uniform redacted foreign/forged outcomes.
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
