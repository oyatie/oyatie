---
id: ADR-0350
adr_id: ADR-0350
title: UUIDv7 canonical ID primitive across Oyatie
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - council-security
  - axis-governance
  - axis-audit-chain
  - axis-workflow-engine
  - axis-tenancy
  - axis-identity
  - axis-observability
owners:
  - council-architecture
  - council-security
  - axis-governance
  - axis-audit-chain
  - axis-workflow-engine
  - axis-tenancy
  - axis-identity
  - axis-observability
authority_chain:
  - keystone: ADR-0244 (Tenant as universal scoping primitive)
  - keystone: ADR-0248 (Amazon-shape cellular architecture)
  - keystone: ADR-0252 (HLC default and TrueTime opt-in)
  - doctrine: ADR-0322 (Substance bar as doctrine and CI enforcement)
  - doctrine: ADR-0324 (Anti-template-stamping doctrine)
  - doctrine: ADR-0345 (OSS stewardship and vendor-lockin discipline)
  - amends: ADR-0003 (Audit chain and evidence emission)
  - amends: ADR-0005 (Eventing backbone and outbox pattern)
  - amends: ADR-0113 (VCS orchestrator end to end)
  - amends: ADR-0214 (Cross-tenant real-time visibility)
  - amends: ADR-0292 (Minor-user doctrine)
  - amends: ADR-0252 (Time, coordination, and distributed consistency)
supersedes: []
superseded_by: [ADR-709]
amends:
  - ADR-0003-audit-chain-and-evidence-emission.md
  - ADR-0005-eventing-backbone-outbox-pattern.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0214-cross-tenant-real-time-visibility.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0252-time-coordination-distributed-consistency.md
related_adrs:
  - ADR-0003-audit-chain-and-evidence-emission.md
  - ADR-0005-eventing-backbone-outbox-pattern.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-policy-and-flat-microservice-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0214-cross-tenant-real-time-visibility.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-http3-quic-default-protocol.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
  - ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md
  - ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md
  - ADR-0351-cell-rebalancer-and-cell-lifecycle-microservices.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/workflow.json
  - /specs/root-hub-pointers.json
  - /specs/markdown-retirement-policy.json
companion_docs:
  - tools/hooks/_canonical-primitives.md
  - docs/standards/dependency-policy.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 800
substance_bar: documentation-rigor-1.1-plus-ADR-0322
substance_bar_facet_binding: reviewer-substance-pending-wave-15-zh
enforcement_status: advisory-until-wave-15-zh-id-corpus-scrub-lands
enforced_by:
  - oya-governance-id-strategy-canonical
  - oya-governance-id-format-validation
  - oya-governance-no-ulid-imports
  - oya-governance-no-snowflake-imports
  - oya-governance-uuid-v7-feature-flag-pinned
  - oya-governance-uuidv7-newtype-validation
purpose: >
  Declare UUIDv7, as standardized by RFC 9562, as the single canonical ID
  primitive across Oyatie. UUIDv7 replaces ULID and every ad hoc UUID or
  string-ID convention for event_id, audit_chain_row_id, vcs_changeset_id,
  tenant_id, cell_id, principal_id, resource_id, request_id, idempotency_key,
  evidence_ref, and every other ID surface. The canonical Rust dependency is
  uuid v1.x with the v7 feature enabled at workspace level, generated through
  Uuid::now_v7(), serialized as lowercase hyphenated RFC 9562 UUID text, stored
  as native Postgres UUID where available and SQLite TEXT where needed, and
  validated through typed newtypes that parse Uuid and check version 7 bytes.
  ULID references in ADR-0003, ADR-0005, ADR-0113, ADR-0214, ADR-0292, and
  ADR-0252 are flagged for Wave 15-ZH rewrite. Snowflake-style IDs are
  explicitly rejected because central allocator and worker-ID coordination
  violate ADR-0248 cellular independence and ADR-0345 vendor-lockin discipline.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0350: UUIDv7 Canonical ID Primitive Across Oyatie

## Status

Accepted on 2026-05-21.

This ADR is doctrine, not the corpus rewrite itself.

The corpus rewrite is Wave 15-ZH.

Wave 15-ZH owns every existing ULID reference, every ad hoc ID format, and
every validator that currently assumes ULID, UUIDv4, Snowflake-style integers,
or opaque untyped strings.

The decision is immediately authoritative for new authoring.

New code, new specifications, new manifests, new OpenAPI 3.2.0 contracts, new
AsyncAPI 3.1.0 contracts, new proto3 files, new Cedar policies, and new ADRs
must use UUIDv7 language and must not introduce ULID as a canonical ID scheme.

Existing corpus references remain migration debt until Wave 15-ZH lands.

Existing corpus references are not grandfathered as doctrine.

They are only tolerated as known debt because their rewrite has a named wave,
named lanes, and named acceptance criteria in this ADR.

The enforcement status is `advisory-until-wave-15-zh-id-corpus-scrub-lands`.

After Wave 15-ZH lands, the lanes listed in the frontmatter promote to BLOCKER
for every pull request that introduces or preserves forbidden ID discipline.

This ADR amends ADR-0003, ADR-0005, ADR-0113, ADR-0214, ADR-0292, and
ADR-0252.

The amendment is narrow: replace their ULID or mixed-ID references with
UUIDv7 canonical doctrine.

This ADR does not change ADR-0252's HLC time primitive.

HLC remains the ordering and causality primitive.

UUIDv7 is the identifier primitive.

The timestamp bits in UUIDv7 are useful for locality and coarse sorting, but
they are not a substitute for HLC, audit-chain sequence checks, saga ordering,
or database transaction ordering.

## Context

### C-1: Current Mixed-Scheme State

The corpus currently uses several ID schemes.

ADR-0003 declares `AuditEvent.event_id` as ULID.

ADR-0005 declares `OutboxRow.event_id` as ULID.

ADR-0113 describes `oya vcs claim --changeset <id>` as producing a ULID.

ADR-0214 declares cross-tenant sharing agreement identifiers as ULID.

ADR-0292 uses `<ulid>` as the event_id placeholder in a compliance event
example.

ADR-0252 lists UUIDv7, crypto-random base32, and ULID as acceptable
idempotency-key body generation strategies.

The workspace also has a Rust crate named `oya-shared-ulid-id-kernel`.

The workspace also has an id-discipline check that still comments that ULID is
canonical per older doctrine.

The workflow microservice manifest already lists the `uuid` crate as a
kernel-grade dependency in `specs/microservices/workflow.json`.

Several OpenAPI and JSON Schema surfaces use `format: uuid`.

Those `format: uuid` declarations are not wrong by themselves, but they are
underspecified.

The missing decision is which UUID version is canonical.

A generic UUID declaration permits UUIDv4, UUIDv1, UUIDv6, UUIDv7, UUIDv8, or
implementation-defined values depending on the SDK generator.

That ambiguity is no longer acceptable.

An `event_id` generated as ULID and a `tenant_id` generated as UUIDv4 have
different parsing rules.

An `audit_chain_row_id` generated as ULID and a `vcs_changeset_id` generated
as an opaque string require different validators.

An `idempotency_key` body generated from ULID and a `request_id` generated from
UUIDv4 make trace joins harder.

Mixed schemes spread ID semantics into application code.

Every place that sees an ID must ask which family it belongs to.

That is exactly the kind of substrate drift ADR-0245 forbids.

The immediate mixed state is especially dangerous because old ADRs use ULID
for eventing and audit-chain, while newer contracts already lean on UUID
ecosystem support.

Without a decision, new code will copy whichever artifact it read last.

That creates silent divergence.

Silent divergence violates the no-silent-regression posture repeatedly cited
by the Wave 15 ADRs.

The correction must be a single ID primitive across the entire platform.

### C-2: Why A Single Canonical

Oyatie is a multi-context platform.

It runs in hosted, AWS-guest, OCI-guest, on-prem, colo, air-gap, and
Oyatie-as-cloud-provider contexts.

It also runs across cellular topology per ADR-0248.

Cells are intentionally independent.

Every ID scheme must therefore work without a global allocator.

Every ID scheme must work inside a cell that has no live connection to another
cell.

Every ID scheme must work in air-gap deployments.

Every ID scheme must work in local developer flows.

Every ID scheme must work through OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3.

Every ID scheme must work through Postgres and SQLite.

Every ID scheme must work in Rust and through generated SDKs.

A single canonical ID primitive gives the corpus one parser.

It gives Cedar policies one ID grammar.

It gives audit-chain one row shape.

It gives VCS changesets one changeset reference shape.

It gives observability one trace join shape.

It gives idempotency-key helpers one generation path.

It gives database migrations one storage rule.

It gives SDK generators one target type.

It gives future agents one doctrine to apply instead of re-deciding per
surface.

The platform already has enough typed identifiers at the domain layer.

`TenantId`, `CellId`, `PrincipalId`, `ResourceId`, `RequestId`,
`EvidenceRef`, and `ChangesetId` remain distinct types.

The single canonical primitive does not erase semantic typing.

It only standardizes the underlying serialization, generation, and validation
rules.

The type layer still prevents passing a tenant ID where a cell ID is expected.

The primitive layer prevents every type from inventing its own wire format.

This separation is the clean architecture answer.

Domain types name meaning.

UUIDv7 supplies the shared substrate representation.

### C-3: Why UUIDv7

UUIDv7 is standardized in RFC 9562.

UUIDv7 uses a Unix epoch millisecond timestamp in the high-order bits.

UUIDv7 keeps enough randomness for decentralized generation.

UUIDv7 sorts roughly by creation time when represented in canonical byte order.

UUIDv7 fits the existing UUID ecosystem.

UUIDv7 fits native Postgres UUID storage.

UUIDv7 fits SDKs that already know how to parse UUID text.

UUIDv7 fits OpenAPI `format: uuid`.

UUIDv7 fits proto3 as a string today and can later map to a shared message if
the platform needs binary UUID transport.

UUIDv7 avoids ULID's Crockford-specific parsing tax.

UUIDv7 avoids UUIDv4's lack of temporal locality.

UUIDv7 avoids Snowflake's central allocator and worker-ID coordination.

UUIDv7 avoids KSUID's smaller ecosystem and 27-character custom text form.

The timestamp bits do not grant distributed consistency.

They grant locality and coarse operational readability.

ADR-0252 still owns causal consistency.

The timestamp bits are useful because adjacent writes often cluster in indexes.

The timestamp bits are useful because support engineers can roughly understand
the creation period without joining a timestamp table.

The timestamp bits are useful because event streams and audit-chain rows often
arrive close to ID order.

The randomness bits are useful because every cell can generate IDs
independently.

No cell needs to ask a global service for the next number.

No worker needs a globally unique worker ID assignment.

No region needs a leased ID block.

No air-gap customer needs a callback to Oyatie control planes.

This matches ADR-0248 cellular independence.

It also matches ADR-0345 vendor-lockin discipline: no Twitter-pattern
allocator, no cloud-provider allocator, and no proprietary sequencing service.

### C-4: Hyperscaler Precedent

The precedent is not that every hyperscaler already mandates UUIDv7.

The precedent is that hyperscaler-grade systems expose and preserve UUID as a
first-class cross-service identifier type.

AWS RDS for PostgreSQL supports UUID generation through the `uuid-ossp`
extension and native Postgres UUID columns.

Postgres supports a native `UUID` type with compact 16-byte storage and index
support.

Google Cloud Spanner supports UUID-oriented application identifiers through
standard UUID string values and generated client types.

Cloud APIs across AWS, Google Cloud, Azure, Stripe-class APIs, GitHub-class
APIs, and Kubernetes-adjacent tooling routinely accept UUID-shaped identifiers.

The UUID ecosystem is the portable interoperability layer.

Oyatie should not force every external integration to learn ULID syntax.

Oyatie should not force every generated SDK to carry a custom Crockford parser
before it can validate a normal resource ID.

Oyatie should not force database schemas to store canonical IDs as arbitrary
text when the primary database has a UUID type.

The UUIDv7 decision composes a standard UUID surface with temporal locality.

That is the useful part of ULID without the custom string ecosystem.

That is the useful part of Snowflake without the allocator.

That is the useful part of UUIDv4 without pure-random index behavior.

The substrate pattern is simple:

- public contracts say UUID
- doctrine says version 7 only
- application generation uses `Uuid::now_v7()`
- domain types wrap `uuid::Uuid`
- Postgres stores `UUID`
- SQLite stores lowercase canonical text
- validators reject non-v7 values

This is ordinary enough for SDKs and strict enough for governance.

## Decision

### D-1: UUIDv7 Is The Single Canonical ID Scheme

UUIDv7 is the single canonical ID scheme for Oyatie.

This applies to every ID surface.

The rule covers event IDs.

The rule covers audit-chain row IDs.

The rule covers VCS changeset IDs.

The rule covers tenant IDs.

The rule covers cell IDs.

The rule covers principal IDs.

The rule covers resource IDs.

The rule covers request IDs.

The rule covers idempotency keys.

The rule covers evidence references.

The rule covers saga IDs.

The rule covers workflow run IDs.

The rule covers outbox row IDs.

The rule covers message IDs.

The rule covers command IDs.

The rule covers policy decision IDs.

The rule covers every future ID surface unless a later ADR explicitly amends
this one.

Domain-specific names remain.

`TenantId` remains a different type from `CellId`.

`PrincipalId` remains a different type from `ResourceId`.

`EvidenceRef` remains a different type from `RequestId`.

The difference is in the type name and semantic contract, not in the underlying
identifier primitive.

Every domain ID newtype wraps a UUIDv7 value.

Every public contract serializes the value in RFC 9562 lowercase hyphenated
text form.

Every internal persistence surface stores the same value in the storage shape
defined by D-6.

Every validator rejects IDs that parse as UUID but are not version 7.

Every validator rejects IDs that are ULID, KSUID, Snowflake integer, NanoID,
random base32, UUIDv4, UUIDv1, or opaque string unless that field is explicitly
not an ID.

Opaque strings remain allowed for names, labels, titles, slugs, and user
visible handles.

Opaque strings are not allowed for canonical identifiers.

The governing principle is: if a value is used to join, reference, deduplicate,
audit, authorize, route, or replay a resource, it is an ID and must be UUIDv7.

This rule is deliberately broad.

Narrow exceptions invite drift.

Drift is the current failure mode.

Wave 15-ZH will rename and rewrite old surfaces that still say ULID.

This ADR is the authority for that rewrite.

### D-2: Manifest Field `id_strategy: "uuidv7"`

Every microservice manifest gains an `id_strategy` field.

The only legal value is `"uuidv7"`.

The field is a closed enum with one value.

The field is intentionally not boolean.

The field is intentionally not optional after Wave 15-ZH.

The field is intentionally not extensible by ordinary manifest authors.

Adding another value requires a future ADR that supersedes or amends this ADR.

Canonical schema fragment:

```json
{
  "id_strategy": {
    "type": "string",
    "enum": ["uuidv7"],
    "description": "Canonical ID generation and validation strategy for every ID surface in this microservice."
  }
}
```

Per-microservice examples:

```json
{
  "microservice_id": "workflow-engine",
  "id_strategy": "uuidv7"
}
```

```json
{
  "microservice_id": "audit-chain",
  "id_strategy": "uuidv7"
}
```

```json
{
  "microservice_id": "tenancy",
  "id_strategy": "uuidv7"
}
```

The field exists because manifest review is where drift is easiest to stop.

If a microservice lacks `id_strategy`, reviewers cannot tell whether the
surface has been migrated.

If a microservice declares `"uuidv7"`, the governance lane can inspect its
contracts, schemas, migrations, and code for matching discipline.

The field is also useful for generated SDKs.

SDK generators can read a single manifest field and map every `format: uuid`
field to a UUIDv7-aware type where the target language supports it.

The field is also useful for documentation projection.

Human docs can show that a service has accepted the platform ID primitive
without reading every contract.

The field is also useful for Wave 15-ZH inventory.

Every service without it is incomplete.

Every service with a value other than `"uuidv7"` is invalid.

The enforcing lane is `oya-governance-id-strategy-canonical`.

### D-3: Crate Canonical - `uuid` v1.x With `v7`

The canonical Rust crate is `uuid` v1.x.

The workspace enables the `v7` feature at the workspace dependency level.

The crate is already present in the workspace dependency corpus through
`specs/microservices/workflow.json`.

The crate license posture is acceptable.

The crate is Apache-2.0 OR MIT.

Apache-2.0 is acceptable for new crates per the canonical primitives.

The workspace dependency should be declared once.

No microservice should pin a divergent `uuid` major version.

No microservice should import a second UUID implementation for canonical ID
generation.

No microservice should import an ULID crate for canonical ID generation.

No microservice should import a Snowflake ID crate.

No microservice should implement UUIDv7 bit assembly by hand.

The canonical workspace dependency shape is:

```toml
[workspace.dependencies]
uuid = { version = "1", features = ["v7", "serde"] }
```

Feature additions such as `fast-rng` or language-specific adapter features may
be reviewed later.

The minimum contract is v1.x plus `v7`.

The `serde` feature is allowed because IDs cross API, audit, and persistence
boundaries.

Every shared ID newtype crate depends on the workspace dependency.

Every microservice that needs direct ID generation depends on the shared
newtype crate rather than directly calling `uuid` from business logic.

Direct `uuid` usage remains allowed in low-level shared crates.

Domain code should prefer `TenantId::new()`, `EventId::new()`,
`RequestId::new()`, and similar constructors.

This keeps ID discipline inspectable.

The enforcing lane is `oya-governance-uuid-v7-feature-flag-pinned`.

### D-4: API Generation Pattern - `Uuid::now_v7()`

The canonical generation call is:

```rust
let id = uuid::Uuid::now_v7();
```

Shared newtype constructors wrap this call.

Example:

```rust
use uuid::{Uuid, Version};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn parse(input: &str) -> Result<Self, IdParseError> {
        let uuid = Uuid::parse_str(input)?;
        if uuid.get_version() != Some(Version::SortRand) {
            return Err(IdParseError::NotUuidV7);
        }
        Ok(Self(uuid))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}
```

The exact enum name used by the crate is checked by implementation.

The doctrine requirement is semantic: parse as UUID and verify version 7.

Business logic must not use `Uuid::new_v4()` for canonical IDs.

Business logic must not use `Uuid::nil()` as a placeholder ID.

Business logic must not use timestamp strings as IDs.

Business logic must not use database sequence integers as IDs.

Business logic must not use Snowflake-style worker IDs.

Business logic must not use ULID constructors.

Test fixtures may use fixed UUIDv7 literals.

Test fixtures must still be version 7.

The canonical fixture pattern is a deterministic version-7 literal with clear
purpose in the fixture name.

Example:

```rust
const FIXTURE_EVENT_ID: &str = "01910f3c-8a4b-7000-8000-000000000000";
```

Tests must not use random UUIDv4 values just because they are easy.

Random values in tests hide regression causes.

Fixed UUIDv7 values make snapshots stable while preserving ID discipline.

The enforcing lane is `oya-governance-id-format-validation`.

### D-5: Serialization

Canonical serialization is lowercase hyphenated hexadecimal UUID text per RFC
9562.

Example:

```text
01910f3c-8a4b-7000-8000-000000000000
```

The canonical string has 36 characters.

The canonical string has five groups.

The canonical string uses hyphens at positions 8, 13, 18, and 23.

The canonical string uses lowercase hexadecimal.

The canonical string carries the version nibble `7`.

The canonical string carries the RFC variant bits.

Uppercase UUID input may be accepted on read if a parser normalizes it.

Lowercase is emitted on write.

Hyphenless UUID input is not canonical.

Hyphenless UUID input may be accepted only at external boundaries if a
compatibility adapter explicitly documents normalization.

New internal contracts must reject hyphenless form.

Base32 ULID text is not accepted as a canonical ID.

Base62 Snowflake text is not accepted as a canonical ID.

Decimal integer Snowflake text is not accepted as a canonical ID.

`urn:uuid:<value>` is not the canonical storage or wire form.

The public API may document that a field is `type: string`,
`format: uuid`, and `x-oyatie-id-version: 7`.

OpenAPI 3.2.0 schema example:

```yaml
type: string
format: uuid
x-oyatie-id-version: 7
example: "01910f3c-8a4b-7000-8000-000000000000"
```

AsyncAPI 3.1.0 message schema uses the same extension.

Proto3 uses `string` until the platform introduces a shared UUID message type.

Proto3 example:

```proto
message AuditEvent {
  string event_id = 1; // UUIDv7 lowercase hyphenated RFC 9562 text.
}
```

The enforcing lane scans OpenAPI, AsyncAPI, proto3, JSON Schema, Rust
serializers, and documentation examples.

### D-6: Database Storage

Postgres stores canonical IDs in the native `UUID` type.

Postgres must not store canonical IDs as `TEXT` unless the column is a legacy
compatibility projection scheduled for removal.

Postgres must not store canonical IDs as `BYTEA` unless a later ADR chooses a
binary-only storage strategy.

Postgres must not store canonical IDs as `BIGINT`.

Postgres must not store canonical IDs as two integer columns.

Postgres must not use sequences for canonical IDs.

Canonical Postgres example:

```sql
CREATE TABLE audit_events (
    event_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    cell_id UUID NOT NULL,
    request_id UUID NOT NULL,
    evidence_ref UUID NOT NULL,
    event_class TEXT NOT NULL,
    created_hlc_physical_ms BIGINT NOT NULL,
    created_hlc_logical SMALLINT NOT NULL
);
```

SQLite stores canonical IDs as `TEXT`.

SQLite is used for local, embedded, or test surfaces where the native UUID type
is absent.

SQLite must enforce a UUIDv7 check through application validators.

SQLite may add a `CHECK` constraint for shape, but application validation is
the authoritative version check.

Canonical SQLite example:

```sql
CREATE TABLE audit_events (
    event_id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    cell_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    evidence_ref TEXT NOT NULL,
    event_class TEXT NOT NULL,
    created_hlc_physical_ms INTEGER NOT NULL,
    created_hlc_logical INTEGER NOT NULL,
    CHECK (length(event_id) = 36)
);
```

Database migrations must not silently rewrite ID values without an audit plan.

ULID to UUIDv7 migration is not a reversible text transform.

Existing rows that already have ULID values require a migration strategy that
preserves old external references.

Wave 15-ZH must choose per-table strategies:

- dual-write new UUIDv7 plus legacy ULID reference during transition
- build mapping table from old ULID to new UUIDv7
- preserve old ULID as a non-canonical legacy external reference field
- emit audit-chain migration rows
- update all foreign keys and references atomically

The database storage rule is simple for new tables.

The migration rule is careful for existing tables.

No table gets a new ULID primary key after this ADR.

### D-7: Validation

Validation uses custom domain newtypes.

Each newtype parses through `uuid::Uuid`.

Each newtype checks the UUID version.

Each newtype emits canonical lowercase hyphenated text.

Each newtype rejects nil UUID.

Each newtype rejects max UUID.

Each newtype rejects UUIDv4.

Each newtype rejects ULID strings.

Each newtype rejects Snowflake decimal strings.

Each newtype rejects whitespace-padded strings.

Each newtype rejects empty strings.

Each newtype rejects path traversal strings.

Each newtype rejects any format that cannot round-trip through canonical UUID
text.

The shared validation contract:

```rust
pub trait CanonicalUuidV7Id: Sized {
    fn from_uuid(uuid: uuid::Uuid) -> Result<Self, IdParseError>;
    fn parse_str(input: &str) -> Result<Self, IdParseError>;
    fn as_uuid(&self) -> uuid::Uuid;
    fn to_canonical_string(&self) -> String;
}
```

Every ID type has a distinct wrapper.

Do not use a single `Id` type everywhere.

The platform needs type distinction for Cedar, audit, and clean architecture.

Examples:

```rust
pub struct TenantId(uuid::Uuid);
pub struct CellId(uuid::Uuid);
pub struct PrincipalId(uuid::Uuid);
pub struct ResourceId(uuid::Uuid);
pub struct RequestId(uuid::Uuid);
pub struct EvidenceRef(uuid::Uuid);
pub struct ChangesetId(uuid::Uuid);
```

The parser error enum must be explicit.

Example:

```rust
pub enum IdParseError {
    Empty,
    InvalidUuid(uuid::Error),
    NotUuidV7,
    NilUuid,
    NonCanonicalText,
}
```

The version check is not optional.

`format: uuid` is not enough.

`Uuid::parse_str` is not enough.

A value can parse as UUID and still be the wrong version.

The enforcing lane is `oya-governance-uuidv7-newtype-validation`.

### D-8: Migration Plan For Existing ULID Corpus

Wave 15-ZH is the corpus scrub sub-wave.

Wave 15-ZH is separate from this ADR.

The sub-wave exists because the corpus has real ULID references in doctrine,
code, comments, and validators.

The scrub must not be a blind text replacement.

Every old ULID reference must be classified.

Classification A: normative doctrine that must be rewritten.

Classification B: historical reference that should remain but needs a note.

Classification C: external product name or unrelated phrase that is not an ID
primitive reference.

Classification D: test fixture or compatibility parser that remains during a
bounded migration window.

Known normative rewrites:

- ADR-0003 `AuditEvent.event_id` moves from ULID to UUIDv7.
- ADR-0005 `OutboxRow.event_id` moves from ULID to UUIDv7.
- ADR-0113 `changeset <id>` moves from ULID to UUIDv7.
- ADR-0214 `agreement_id` moves from ULID to UUIDv7.
- ADR-0292 event examples move from `<ulid>` to UUIDv7 examples.
- ADR-0252 idempotency-key generation removes ULID as a canonical generator.

Known code rewrites:

- retire `oya-shared-ulid-id-kernel` as canonical generation surface.
- update `oya-check-id-discipline` comments and logic from ULID canonical to
  UUIDv7 canonical.
- add or rename a shared UUIDv7 ID kernel crate.
- update workspace dependencies to pin `uuid` v1.x with `v7`.
- remove ULID crate imports unless a legacy compatibility parser remains.

Known spec rewrites:

- add `id_strategy: "uuidv7"` to every microservice manifest.
- add `x-oyatie-id-version: 7` to OpenAPI 3.2.0 UUID fields where the schema
  can carry extensions.
- add equivalent metadata to AsyncAPI 3.1.0 and proto3 comment conventions.
- update `specs/microservices/manifest-schema.json`.

Wave 15-ZH must emit an inventory before edits.

The inventory must list every ULID occurrence and every Snowflake-ID occurrence.

The inventory must separate Snowflake database/product references from
Snowflake ID algorithm references.

Snowflake database/product references in ADR-0214 remain valid as product
counterpart discussion.

Snowflake ID algorithm references are rejected and must not become canonical.

Wave 15-ZH must finish with zero canonical ULID references.

Wave 15-ZH must finish with zero canonical Snowflake-ID references.

Wave 15-ZH must finish with all new ID validators checking UUIDv7.

### D-9: Idempotency Key Consequence

ADR-0252 currently gives idempotency keys a custom `idem_<base32>` wire format.

This ADR amends that posture.

The canonical idempotency key primitive is UUIDv7.

The field name `idempotency_key` remains.

The HTTP header name `Idempotency-Key` remains.

The value becomes canonical UUIDv7 text unless a future API-specific standard
requires a decorated form.

The Stripe-style semantics remain.

The caller still supplies the key.

The server still scopes the key by tenant.

The server still stores request signature and cached response.

The server still rejects same key with different request signature.

The server still treats the key as opaque for business semantics.

The server does not decode UUIDv7 timestamp bits for authorization or replay.

The timestamp bits only improve locality and supportability.

ADR-0252's HLC remains the ordering source.

ADR-0252's saga replay semantics remain unchanged.

What changes is the value grammar:

```text
Idempotency-Key: 01910f3c-8a4b-7000-8000-000000000000
```

The old `idem_<32-base32>` examples become migration debt.

The old `ulid_then_base32` generator becomes migration debt.

The old `uuid_v7_then_base32` generator becomes migration debt because the
UUIDv7 itself is now the value, not an intermediate entropy source.

The new helper is:

```rust
pub struct IdempotencyKey(uuid::Uuid);

impl IdempotencyKey {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}
```

This simplification removes one custom parser from every SDK.

It also makes idempotency keys join naturally with request IDs and audit rows.

### D-10: Contract Surface Rules

OpenAPI 3.2.0 is the canonical REST contract language.

AsyncAPI 3.1.0 is the canonical event contract language.

Proto3 is the canonical binary RPC schema language.

Every contract language must express UUIDv7 discipline.

OpenAPI fields use:

```yaml
type: string
format: uuid
x-oyatie-id-version: 7
```

AsyncAPI schema fields use the same JSON Schema extension when applicable.

Proto3 fields use `string` plus a required comment convention until a shared
`UuidV7` message is adopted.

Proto3 comment convention:

```proto
// UUIDv7 lowercase hyphenated RFC 9562 text.
string tenant_id = 1;
```

Contract generators must not emit language-native random UUID constructors for
new ID values.

Generated SDK create calls must call the SDK's UUIDv7 helper or require the
caller to provide a UUIDv7 value.

Generated SDK parse calls must reject non-v7 UUIDs if the language runtime can
inspect version bits.

Generated SDK parse calls that cannot inspect version bits must call the shared
SDK validator.

SDK documentation must not show UUIDv4 examples.

SDK documentation must not show ULID examples.

SDK documentation must not show Snowflake integer examples.

SDK documentation may mention legacy ULID only in migration guides.

The `format: uuid` phrase alone is now insufficient for Oyatie contracts.

The extension or equivalent comment is required for new authoring.

### D-11: Audit, Evidence, And Observability Event Classes

This ADR adds governance audit event classes.

It does not change the audit-chain cryptographic hash algorithm.

It does change ID fields that audit-chain rows carry.

New audit classes:

```text
governance.id.uuidv7.validation_failed
governance.id.ulid_reference_detected
governance.id.snowflake_reference_detected
governance.id.manifest_strategy_missing
governance.id.manifest_strategy_invalid
governance.id.uuid_v7_feature_flag_missing
governance.id.legacy_id_mapping_created
governance.id.legacy_id_mapping_verified
```

`governance.id.uuidv7.validation_failed` emits when a validator sees an ID
that parses but is not version 7.

`governance.id.ulid_reference_detected` emits when the no-ULID lane finds a
canonical ULID reference.

`governance.id.snowflake_reference_detected` emits when the no-Snowflake-ID
lane finds a canonical Snowflake ID algorithm reference.

`governance.id.manifest_strategy_missing` emits when a manifest lacks
`id_strategy`.

`governance.id.manifest_strategy_invalid` emits when `id_strategy` is present
but not `"uuidv7"`.

`governance.id.uuid_v7_feature_flag_missing` emits when workspace dependency
configuration lacks the required `v7` feature.

`governance.id.legacy_id_mapping_created` emits when Wave 15-ZH creates a
mapping between old ULID and new UUIDv7 for a persistent table.

`governance.id.legacy_id_mapping_verified` emits when a migration verifies
referential integrity after mapping.

All event classes carry:

- `artifact_path`
- `surface_kind`
- `field_name`
- `tenant_id` when applicable
- `cell_id` when applicable
- `legacy_value_hash` when applicable
- `uuidv7_value` when applicable
- `detected_by_lane`
- `evidence_ref`

The `evidence_ref` is itself UUIDv7.

The audit event class additions satisfy ADR-0322 S-5.

### D-12: No Hidden Allocator

UUIDv7 generation is local.

There is no central ID service.

There is no global sequence.

There is no worker-ID registry.

There is no per-region allocator.

There is no lease block.

There is no Snowflake epoch.

There is no shard ID embedded in the canonical ID.

Cell identity belongs in `cell_id`.

Tenant identity belongs in `tenant_id`.

Residency belongs in residency fields.

Compliance pack identity belongs in compliance fields.

Embedding topology into the ID would couple routing to identifier creation.

That violates ADR-0248 cellular independence.

That also makes cell migration harder.

If an ID encodes its original cell, then moving a tenant makes every old ID
look like it belongs elsewhere.

UUIDv7 deliberately avoids that.

Routing is a lookup.

Authorization is a Cedar evaluation.

Ordering is HLC and database state.

Uniqueness is UUIDv7.

These concerns remain separate.

The enforcing lane `oya-governance-no-snowflake-imports` blocks any dependency
or code path that introduces central-worker ID allocation.

It must distinguish Snowflake database/product references from Snowflake ID
algorithm references.

ADR-0214 can keep Snowflake Secure Data Sharing as an external product
counterpart reference.

It cannot use Snowflake-style IDs as a platform primitive.

## Rationale

1. RFC standard.

UUIDv7 is standardized by RFC 9562.

Standards matter because Oyatie crosses languages, SDKs, deployment contexts,
and customers.

A standard UUID version is easier to explain than a custom or de facto format.

2. Ecosystem support.

UUID parsing exists in every target SDK language.

ULID support is uneven.

Snowflake support is usually library-specific and topology-specific.

UUID support is already assumed by OpenAPI `format: uuid`.

3. Timestamped.

UUIDv7 carries millisecond timestamp bits.

That gives log locality and index locality without making IDs an ordering
authority.

Operational debugging benefits from time-correlated IDs.

4. Sortable.

UUIDv7 sorts approximately by creation time in canonical byte order.

This improves common append-heavy table and event-stream behavior compared
with UUIDv4.

This does not replace explicit indexes on HLC or created_at fields.

5. Postgres native.

Postgres has a native UUID type.

Native UUID storage is smaller and clearer than storing ULID as text.

It also keeps schema intent visible.

6. No central allocator.

Every cell can generate UUIDv7 values locally.

No cell depends on a global allocator.

No outage in an ID service can stop write paths across the platform.

7. Cellular-compatible.

ADR-0248 requires per-cell independence.

UUIDv7 generation respects that.

Snowflake-style worker coordination does not.

8. Foundry-self-modification-friendly.

ADR-0247 self-modification workflows need IDs that agents can generate in
isolated worktrees, local tests, and air-gap contexts.

UUIDv7 needs no coordination surface.

9. Contract-friendly.

OpenAPI 3.2.0 and AsyncAPI 3.1.0 already understand UUID format.

Proto3 can carry UUIDv7 as a string with validation.

Every SDK generator can expose UUID strings safely.

10. Migration-friendly.

Moving from ULID to UUIDv7 is a controlled rewrite.

Moving from mixed schemes to "one UUIDv7 primitive" removes future choices.

It gives Wave 15-ZH a precise target.

11. Security-friendly.

UUIDv7 does not embed tenant, cell, shard, worker, region, or sequence
information.

It avoids leaking topology through identifiers.

12. Governance-friendly.

CI lanes can parse UUIDs, inspect version bits, and report deterministic
violations.

ULID-vs-UUID-vs-Snowflake policy becomes mechanically enforceable.

## Consequences

### Benefits

The corpus gets one canonical ID grammar.

The platform gets one generation API.

The database gets one primary-key storage rule.

The API layer gets one serialization form.

The SDK layer gets one validation target.

The audit-chain gets one row-ID format.

The VCS changeset system gets one changeset ID format.

The idempotency-key system loses a custom base32 parser.

The manifest schema gains a simple field that allows drift detection.

The governance lanes can be deterministic.

Postgres schemas become clearer.

SQLite schemas remain portable.

Air-gap deployments can generate IDs locally.

Cellular deployments can generate IDs without cross-cell coordination.

Developer fixtures become stable and standards-shaped.

Future ADRs no longer need to choose an ID family.

### Costs

ULID corpus references must be rewritten.

The `oya-shared-ulid-id-kernel` crate must be retired or converted to a legacy
compatibility surface.

Existing validators must change.

Existing docs that use `<ulid>` examples must be edited.

Existing idempotency-key examples in ADR-0252 must be amended.

Existing storage that has ULID primary keys needs migration plans.

Generated SDKs must learn UUIDv7 validation where language support is uneven.

Some languages may parse UUID but not expose version bits directly.

Those languages need shared validators.

Some tests currently relying on UUIDv4 helpers must be rewritten.

Some database fixtures must be rewritten.

Some external references may keep legacy IDs for compatibility.

Those references need explicit "legacy_external_id" naming.

The migration is broad.

That is why it is Wave 15-ZH, not hidden in this ADR commit.

## Alternatives Considered

### A-1: ULID

ULID is rejected as the canonical ID primitive.

ULID has useful properties.

It is timestamped.

It sorts lexicographically.

It is compact.

It has been used in the Oyatie corpus.

Those benefits are not enough.

ULID is not the RFC UUID standard.

ULID uses Crockford base32.

That requires custom parsing in every SDK.

ULID does not map to Postgres native UUID type.

ULID does not fit OpenAPI `format: uuid`.

ULID support is less universal than UUID support.

ULID remains useful as historical context.

ULID is no longer canonical.

### A-2: UUIDv4

UUIDv4 is rejected as the canonical ID primitive.

UUIDv4 has broad ecosystem support.

UUIDv4 is decentralized.

UUIDv4 maps to Postgres UUID.

UUIDv4 is not timestamped.

UUIDv4 is not sortable by creation time.

UUIDv4 creates worse locality for append-heavy tables and event streams.

UUIDv4 examples also encourage old helper usage that already exists in many
libraries.

Oyatie needs the UUID ecosystem and temporal locality.

UUIDv7 is the better fit.

### A-3: Snowflake

Snowflake-style IDs are rejected.

This means the Twitter-pattern ID algorithm, not the Snowflake database product
references in ADR-0214.

Snowflake IDs require worker-ID coordination.

Snowflake IDs require an epoch contract.

Snowflake IDs usually require central assignment of worker or shard identity.

That violates ADR-0248 cellular independence.

It also introduces a vendor-lockin and topology-lockin pattern rejected by
ADR-0345.

It leaks topology into the ID.

It makes cell migration harder.

It creates an allocator surface that must be operated.

Snowflake is explicitly rejected.

### A-4: KSUID

KSUID is rejected.

KSUID is timestamped.

KSUID is decentralized.

KSUID has operational precedent.

KSUID uses a 27-character custom text form.

KSUID has a smaller library ecosystem than UUID.

KSUID does not map to Postgres native UUID.

KSUID does not fit OpenAPI `format: uuid`.

The benefit over UUIDv7 is not enough.

### A-5: Hybrid ULID + UUID

A hybrid scheme is rejected.

The current corpus is already close to this failure mode.

Using ULID for events and UUID for resources creates two parsers.

Using UUID for public APIs and ULID for internal rows creates translation
pressure.

Using ULID for idempotency and UUID for request IDs creates trace join friction.

Using two schemes forces every validator to carry exceptions.

The cognitive cost is permanent.

The migration cost is temporary.

Oyatie chooses the temporary migration cost.

## Affected Surface

Affected doctrine:

- ADR-0003 audit-chain event IDs.
- ADR-0005 outbox event IDs.
- ADR-0113 VCS changeset IDs.
- ADR-0214 agreement IDs and Snowflake-product wording disambiguation.
- ADR-0292 event examples.
- ADR-0252 idempotency-key generation and examples.

Affected specs:

- `specs/microservices/manifest-schema.json`
- every `specs/microservices/<name>.json`
- OpenAPI 3.2.0 schemas using ID fields
- AsyncAPI 3.1.0 schemas using ID fields
- proto3 messages using ID fields

Affected code:

- `Cargo.toml` workspace dependency configuration
- `crates/oya-shared-ulid-id-kernel`
- future UUIDv7 shared ID kernel crate
- `crates/oya-check-id-discipline`
- validators that currently permit UUIDv4 or ULID
- test fixtures using random UUIDv4
- database migrations using text ULID primary keys

Affected CI lanes:

- `oya-governance-id-strategy-canonical`
- `oya-governance-id-format-validation`
- `oya-governance-no-ulid-imports`
- `oya-governance-no-snowflake-imports`
- `oya-governance-uuid-v7-feature-flag-pinned`
- `oya-governance-uuidv7-newtype-validation`

Affected runtime surfaces:

- audit-chain
- eventing outbox
- workflow-engine sagas
- VCS orchestrator
- tenancy
- identity
- api-gateway
- cell-rebalancer
- cell-lifecycle
- observability

Affected storage:

- Postgres primary keys
- Postgres foreign keys
- Postgres idempotency key tables
- SQLite local/test tables
- legacy mapping tables during Wave 15-ZH

Affected documentation:

- ADR examples
- README snippets that mention ULID
- standards docs that mention canonical IDs
- code comments that say ULID is canonical
- migration playbooks

## Cedar Policy Fragment

The PDP enforces ID discipline for governance actions that admit new artifacts.

```cedar
@id("governance.id.uuidv7.required")
permit (
  principal in Group::"oyatie.governance.validators",
  action == Action::"governance.id.validate",
  resource is Artifact
) when {
  context.required_id_strategy == "uuidv7" &&
  context.artifact_declares_id_strategy == "uuidv7" &&
  context.uuid_version == 7 &&
  context.id_text_is_canonical_lowercase_hyphenated == true
};

@id("governance.id.ulid.forbidden")
forbid (
  principal,
  action == Action::"governance.artifact.admit",
  resource is Artifact
) when {
  context.contains_canonical_ulid_reference == true &&
  context.wave != "15-ZH-legacy-compatibility-window"
};

@id("governance.id.snowflake.forbidden")
forbid (
  principal,
  action == Action::"governance.artifact.admit",
  resource is Artifact
) when {
  context.contains_snowflake_id_algorithm_reference == true
};

@id("governance.id.strategy.closed-enum")
forbid (
  principal,
  action == Action::"governance.manifest.admit",
  resource is MicroserviceManifest
) when {
  resource.id_strategy != "uuidv7"
};
```

The fragment is intentionally PDP-level.

Application code does not decide whether ULID is allowed.

Application code only exposes typed validators.

The governance lane evaluates corpus authoring.

The PDP grants or refuses artifact admission.

This matches ADR-0243 Cedar universal gate doctrine.

## CI Lanes

### L-1: `oya-governance-id-strategy-canonical`

This lane reads every microservice manifest.

It refuses missing `id_strategy` after Wave 15-ZH.

It refuses every value except `"uuidv7"`.

It emits `governance.id.manifest_strategy_missing` when absent.

It emits `governance.id.manifest_strategy_invalid` when wrong.

It is REPORT-ONLY at this ADR's acceptance.

It promotes to BLOCKER after Wave 15-ZH.

### L-2: `oya-governance-id-format-validation`

This lane scans schemas, examples, and fixtures.

It verifies UUID examples are version 7.

It verifies OpenAPI UUID fields carry `x-oyatie-id-version: 7`.

It verifies AsyncAPI UUID fields carry equivalent metadata.

It verifies proto3 ID fields carry the UUIDv7 comment convention.

It rejects UUIDv4 examples.

It rejects ULID examples outside legacy migration docs.

### L-3: `oya-governance-no-ulid-imports`

This lane scans Rust imports, Cargo manifests, documentation, and contracts.

It refuses ULID crates for canonical ID generation.

It allows bounded legacy compatibility code only when the path and module name
make legacy status explicit.

It requires an evidence_ref for every legacy exception.

It emits `governance.id.ulid_reference_detected`.

### L-4: `oya-governance-no-snowflake-imports`

This lane scans for Snowflake ID algorithm dependencies and code.

It refuses worker-ID allocator libraries.

It refuses custom bit-packing sequence generators.

It refuses database sequences used as canonical distributed IDs.

It does not refuse Snowflake database/product references in counterpart
analysis.

It emits `governance.id.snowflake_reference_detected`.

### L-5: `oya-governance-uuid-v7-feature-flag-pinned`

This lane reads workspace Cargo dependency configuration.

It requires `uuid` v1.x.

It requires the `v7` feature.

It allows `serde`.

It refuses divergent major versions.

It refuses local reimplementation of UUIDv7 generation.

It emits `governance.id.uuid_v7_feature_flag_missing`.

### L-6: `oya-governance-uuidv7-newtype-validation`

This lane scans shared ID newtypes.

It requires parse plus version check.

It requires nil rejection.

It requires canonical lowercase serialization.

It requires distinct domain wrappers.

It refuses a single untyped `Id` alias for every domain.

It verifies the validator is used at external boundaries.

## Acceptance Criteria Checklist

- [ ] ADR-0350 exists at `docs/decisions/ADR-0350-uuidv7-canonical-id-primitive.md`.
- [ ] The ADR is at least 800 lines.
- [ ] Frontmatter includes `adr_id`, `status: Accepted`, `date`, `authority_chain`, `enforced_by`, and `purpose`.
- [ ] Status section names Wave 15-ZH as the corpus scrub.
- [ ] Context includes the current mixed-scheme state.
- [ ] Context explains why a single canonical primitive is required.
- [ ] Context explains why UUIDv7 wins over ULID, Snowflake, and UUIDv4.
- [ ] Context names UUID hyperscaler/database precedent.
- [ ] Decision declares UUIDv7 canonical for every ID surface.
- [ ] Decision declares manifest field `id_strategy: "uuidv7"`.
- [ ] Decision declares `uuid` v1.x plus `v7` feature.
- [ ] Decision declares `Uuid::now_v7()` as generation pattern.
- [ ] Decision declares lowercase hyphenated RFC 9562 serialization.
- [ ] Decision declares Postgres `UUID` storage.
- [ ] Decision declares SQLite `TEXT` storage.
- [ ] Decision declares newtype parse plus version check validation.
- [ ] Decision declares Wave 15-ZH migration plan for ULID corpus.
- [ ] Rationale includes RFC standard.
- [ ] Rationale includes ecosystem support.
- [ ] Rationale includes timestamped IDs.
- [ ] Rationale includes sortability.
- [ ] Rationale includes Postgres native storage.
- [ ] Rationale includes no central allocator.
- [ ] Rationale includes cellular compatibility.
- [ ] Rationale includes self-modification compatibility.
- [ ] Consequences list benefits and costs.
- [ ] Alternatives reject ULID.
- [ ] Alternatives reject UUIDv4.
- [ ] Alternatives reject Snowflake ID algorithm.
- [ ] Alternatives reject KSUID.
- [ ] Alternatives reject hybrid ULID plus UUID.
- [ ] Affected surface lists docs, specs, code, runtime, storage, and CI.
- [ ] Cedar policy fragment exists.
- [ ] CI lanes include id-strategy-canonical.
- [ ] CI lanes include id-format-validation.
- [ ] CI lanes include no-ulid-imports.
- [ ] CI lanes include no-snowflake-imports.
- [ ] CI lanes include uuid-v7-feature-flag-pinned.
- [ ] Cross-references name amended ADRs 0003, 0005, 0113, 0214, 0292, and 0252.
- [ ] `cargo run -q -p oya-dev-cli -- doc adr-index --write` passes.
- [ ] `cargo run -q -p oya-dev-cli -- gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions` passes.
- [ ] `cargo run -q -p oya-dev-cli -- lint adr-shape` passes.

## Cross-References To Amended ADRs

### ADR-0003

ADR-0003 currently names ULID for `AuditEvent.event_id`.

Wave 15-ZH rewrites that field to UUIDv7.

The audit-chain hash, tenant shard, payload, and anchoring model remain
unchanged.

The field type changes from ULID-backed ID to UUIDv7-backed `EventId`.

### ADR-0005

ADR-0005 currently names ULID for `OutboxRow.event_id`.

Wave 15-ZH rewrites that field to UUIDv7.

CloudEvents `id` remains a string, but the string value is UUIDv7 canonical
text.

The outbox pattern and Kafka backbone remain unchanged.

### ADR-0113

ADR-0113 currently describes VCS changeset IDs as ULID.

Wave 15-ZH rewrites changeset IDs to UUIDv7.

The Oya VCS state machine remains unchanged.

The canonical command surface remains `oya vcs claim`, `oya vcs verify`,
`oya vcs done`, and `oya vcs promote`.

The VCS implementation must generate `ChangesetId` through UUIDv7 newtypes.

### ADR-0214

ADR-0214 currently names `agreement_id` as ULID.

Wave 15-ZH rewrites agreement IDs to UUIDv7.

ADR-0214 also references Snowflake Secure Data Sharing as an external product
counterpart.

That product reference may remain.

It must not be confused with Snowflake ID algorithms.

This ADR rejects the ID algorithm, not product counterpart analysis.

### ADR-0292

ADR-0292 currently includes `<ulid>` in an event example.

Wave 15-ZH rewrites the example to a UUIDv7 literal.

The minor-user doctrine remains unchanged.

The compliance event row shape changes only in ID grammar.

### ADR-0252

ADR-0252 currently allows UUIDv7, crypto-random base32, and ULID as
idempotency-key body generation strategies.

This ADR narrows that to UUIDv7 canonical text.

ADR-0252's HLC default remains authoritative.

ADR-0252's TrueTime opt-in remains authoritative.

ADR-0252's saga and idempotency semantics remain authoritative.

Only the ID grammar is amended.

## Completion Report

status: Accepted

date: 2026-05-21

line_floor: 800

new_lanes:

- oya-governance-id-strategy-canonical
- oya-governance-id-format-validation
- oya-governance-no-ulid-imports
- oya-governance-no-snowflake-imports
- oya-governance-uuid-v7-feature-flag-pinned
- oya-governance-uuidv7-newtype-validation

wave:

- Wave 15-ZH: UUIDv7 corpus scrub and ID strategy adoption

verification:

- ADR index generation is required before commit.
- ADR citation validation is required before commit.
- ADR shape lint is required before commit.

