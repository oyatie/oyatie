---
id: ADR-0276
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal
  - council-compliance
  - ops-compliance
  - ops-sre-reliability
  - ops-security
  - axis-tenancy
  - axis-audit-chain
  - axis-data-portability
  - axis-ontology
  - axis-cloud
  - axis-policy-engine
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0002-tenant-and-identity-kernel.md
  - ADR-0003-audit-chain-and-evidence-emission.md
  - ADR-0006-ontology-typed-entity-layer.md
  - ADR-0008-data-use-boundary.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0011-cross-microservice-contract-registry.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0034-per-microservice-data-class-overrides.md
  - ADR-0037-public-api-stability-tiers-and-deprecation.md
  - ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md
  - ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0180-stateful-disaster-recovery.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/data-portability-format.json
  - /specs/microservices/portability-export.json
  - /specs/microservices/portability-import.json
  - /specs/microservices/audit-chain.json
  - /specs/ontology-export-schema.json
  - /specs/manifest-schema.json
  - /specs/dsar-cascade-protocol.json
  - /specs/byok-credential-model.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_canonical_base_localization
  - feedback_doc_coverage_enforced
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_flat_product_catalog
  - feedback_bominal_inheritance_precedence
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-tier-1-lockdown
keystone_position: tier-1-portability
purpose: >
  Establish the canonical backup + portability format that satisfies
  GDPR Article 20 right to data portability and equivalent rights in
  KR-PIPA Article 35-2, CCPA, LGPD, PIPEDA, and POPIA. Define a single
  Tier-1 export format — JSON-LD bundled in tar.gz with a JSON Schema
  manifest, Ed25519 + cosign dual signatures, and per-µservice schemas
  resolved by URI — that lets every tenant of `oyatie` exit at any
  time, take every byte of their data with them in a structured,
  commonly-used, machine-readable form, and (in the limit) re-import
  into a different oyatie tenant. Tier-1 lockdown semantics apply:
  the format choice is hard to reverse once tenants depend on it for
  exit, so it is selected with the full reversibility-cost discount
  baked in. Establish full vs incremental modes, encryption choice,
  ontology export, audit chain export with Merkle proof, re-import
  compatibility for a 5+ year sunset window, and schema evolution
  rules for the format itself.
enforcement_status: advisory-until-portability-substrate-lands
enforced_by:
  - oya gate validate portability-format-coherence
  - oya gate validate manifest-schema-conformance
  - oya gate validate per-microservice-export-coverage
  - oya gate validate ontology-export-completeness
  - oya gate validate audit-chain-merkle-proof
  - oya gate validate dual-signature-attestation
  - oya gate validate import-roundtrip-fidelity
  - oya gate validate schema-evolution-backward-compat
---

> **Disposition light-edit (2026-08-06):** GDPR Art.20 portability format

# ADR-0276: Backup + Portability Format (GDPR Article 20)

## Status

Proposed — 2026-05-20.

This ADR is part of the **Tier-1 lockdown** bundle: decisions whose
reversibility cost is asymmetrically high because external parties
(regulators, tenants, third-party importers) consume the chosen
interface and develop dependencies on it. Once a tenant exports their
data under format `v1.0.0` and that export is the basis of a regulator
audit, of a contractual exit deliverable, or of a re-import into a
peer system, the format becomes a public contract with all the
sunsetting weight that ADR-0037 (public API stability tiers) attaches
to a Tier-1 contract.

Enforcement is `advisory-until-portability-substrate-lands`. CI lanes
that enforce this ADR promote to BLOCKER once:

1. `microservices/portability-export/` is scaffolded under the
   per-microservice flat layout (ADR-0131) with `src/` as the
   canonical code root, the `oya-portability-export-domain` crate
   declaring the manifest + bundle types, and the
   `oya-portability-export-app` crate orchestrating per-µservice
   contributions.
2. `microservices/portability-import/` is scaffolded as the peer
   import µservice with per-µservice import adapters per §D-13.
3. Each in-scope µservice (Mail, Messenger, Drive, Calendar, HR,
   Workflow Studio, Ontology, Audit Chain, IAM, Billing, Connect,
   Search, Intelligence, Foundry-self-modification-records, etc.)
   exposes its contributing JSON-LD schema at the canonical URI
   `https://contracts.oyatie.dev/portability/v1/<microservice>/<class>.jsonld`
   and a contract test demonstrating round-trip fidelity.
4. The `oya-gate-validate portability-format-coherence` lane verifies
   the full pipeline against the §B worked-example tenant fixture.
5. The Sigstore cosign trust root for the oyatie signing identity is
   published per ADR-0039 and accessible from the air-gap-aware
   verification CLI in `oya port verify`.
6. Schema-evolution gates per §D-14 (backward-compat field rules,
   versioned namespace, deprecation-warning manifest field) are
   wired into the contract-registry lane (ADR-0011).

Until those six items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER and any change to
the format goes through the ADR-0037 Tier-1 sunset path (12-month
deprecation, dual-write window, regulator notification).

## Date

2026-05-20.

## Context

### What Article 20 actually requires

Regulation (EU) 2016/679 Article 20 ("Right to data portability") is
the canonical right: a data subject has the right to receive personal
data concerning them, which they have provided to a controller, in a
**structured, commonly used and machine-readable format**, and to
transmit those data to another controller without hindrance. The
text further requires that, where technically feasible, the data
subject can have the personal data transmitted **directly from one
controller to another**. The 30-day fulfillment SLA inherits from
Article 12(3): the controller shall provide information on action
taken on a request without undue delay and in any event within one
month of receipt.

Three structural obligations follow from that text:

1. **Structured.** The format must have an inspectable schema. A
   blob of opaque bytes is non-conformant. JSON-LD with per-class
   schemas referenced by URI satisfies this directly because the
   `@context` field declares the schema inline and the schema is
   itself a structured artifact.
2. **Commonly used and machine-readable.** The format must be one a
   peer controller could plausibly consume. Proprietary binary
   formats whose only reader is the original controller fail this
   test. JSON + tar + gzip + Ed25519 + JSON Schema are each W3C, IETF,
   or de-facto-universal standards with widely available reader
   implementations across every major language ecosystem.
3. **Transmittable to another controller.** The export must be
   self-contained — every reference resolvable, every type
   self-describing, every secret either omitted or re-encrypted under
   a key the destination controller can be given.

KR-PIPA Article 35-2 (data portability, added 2023 amendment, in
force 2024-03-15) imposes a parallel right with similar structural
obligations and an additional **Korean-language documentation
requirement** for the exporting controller's portability declaration.
CCPA §1798.130(a)(2) and §1798.100(d) impose a parallel right for
California residents with a 45-day fulfillment SLA. LGPD Article 18,
VI, imposes the parallel right for Brazilian residents. PIPEDA
Principle 9 and POPIA §23 cover Canada and South Africa respectively.

### Why this is a Tier-1 lockdown

ADR-0037 establishes three public API stability tiers. Tier-1 is the
strictest: 12-month deprecation, dual-write windows, regulator
notification before sunset. The portability format meets every
Tier-1 trigger:

- **External-party consumption.** Regulators, peer controllers, and
  tenants themselves consume the format. None of those parties is
  inside the deployment perimeter where we control the upgrade
  cadence.
- **Re-import compatibility expectation.** A tenant exports in 2026
  and may re-import in 2031. The format must survive five years of
  evolution without breaking their bundle.
- **Asymmetric reversal cost.** Changing the format choice (e.g.,
  swapping JSON-LD for protobuf) is not a one-side migration: every
  tenant who exported under the old format would have their
  re-import contract broken, and every peer controller who built an
  importer would need to rebuild.
- **Contractual exit deliverable.** Tenant exit agreements reference
  the format by name. Changing it changes a contract clause.

Tier-1 lockdown means: select the format with the full reversibility
discount applied. Bias hard toward open, widely-implemented standards
even if a proprietary alternative is marginally more efficient.

### Why JSON-LD specifically (vs raw JSON, vs RDF, vs protobuf)

The candidates considered for the wire format are:

- **Raw JSON.** Structured and machine-readable, but the schema
  declaration is out-of-band. A peer controller reading the export
  must download a separate schema artifact. JSON-LD inlines the
  `@context` so the document is self-describing.
- **JSON-LD 1.1 (W3C Recommendation 2020-07-16).** Structured,
  machine-readable, schema-inline via `@context`, transformable to
  RDF triples for semantic-web tooling, polyglot-readable (every
  JSON reader can ignore `@context` and read it as plain JSON).
- **RDF/Turtle.** Strictly more expressive but requires a Turtle
  parser. Fewer peer controllers will have one.
- **N-Quads.** Same RDF-graph expressiveness, line-oriented for
  streaming, but the same parser-availability problem.
- **Protobuf or Avro.** Compact and schema-strict, but the schema is
  a binary contract; reading the export without the matching
  `.proto` or `.avsc` is impossible.
- **CSV/TSV.** Fails the "structured" test for nested data and the
  "self-describing" test for any non-trivial schema.

JSON-LD 1.1 satisfies all three Article 20 obligations *and* lets us
treat the export as RDF where downstream tooling wants RDF (the
Ontology layer per ADR-0006 already produces RDF-shaped triples;
JSON-LD-as-RDF is a zero-loss transform).

### Why per-µservice schemas (vs one monolithic schema)

The flat product catalog (feedback_flat_product_catalog) declares one
µservice per concern. Every µservice owns its data classes. A
monolithic schema would re-couple µservices through a single shared
type registry, violating ADR-0145 (inter-microservice communication
reform). The export schema *follows* the µservice boundary: each
µservice publishes its export schema at
`https://contracts.oyatie.dev/portability/v1/<microservice>/<class>.jsonld`,
and the manifest's `data_classes` array enumerates which µservice
contributed which class to a given bundle.

This also means: when a new µservice ships (per ADR-0131 single-
concern flat layout), it brings its own export schema. The
portability-export µservice does not need to be modified.

### Why Sigstore cosign + Ed25519 dual signatures

Single-signature exports are forgeable in one of two directions:

- **Forged by an attacker who breaches the controller.** A controller
  whose signing key is compromised can sign a bundle that
  misrepresents what the tenant's data actually was at export time.
- **Forged by a tenant claiming the controller did wrong.** A
  tenant whose copy is the only copy can mutate the bundle and
  claim the controller's signature applied to the mutated state.

Dual signatures (Ed25519 tenant key + Sigstore cosign oyatie
identity) close both directions. The tenant signs to attest "this is
the bundle I received and accepted." Oyatie signs to attest "this is
the bundle we generated at timestamp T from tenant state S." Either
party can verify both signatures against widely-deployed verifiers.
Sigstore's transparency log (Rekor) provides an independent third-
party attestation that the oyatie signature existed at the claimed
time, addressing the "controller backdates" attack.

ADR-0039 already establishes Sigstore cosign as the supply-chain
signing primitive for build artifacts; reusing the same trust root
for portability bundles keeps the verification toolchain to one.

### Why incremental + full modes

Two distinct use cases drive two modes:

- **Full export.** Tenant exit, regulator audit, contractual
  deliverable, periodic backup snapshot. The bundle contains every
  byte of tenant state at the requested timestamp.
- **Incremental export.** Continuous-export use cases: weekly
  privacy-rights syncs, tenant-to-third-party-backup-vault streams,
  re-import after a partial-loss event. The bundle contains only
  state that changed since the last cursor (RFC 3339 timestamp +
  per-µservice opaque cursor).

Without incremental, large tenants face hours-long full-export jobs
for low-change-rate data classes. With incremental, the same tenant
syncs deltas in minutes.

### Why cross-tenant restore

The B2C-to-B2B migration story (a personal user upgrades to a
business plan; their personal-tenant data should land in their new
business-tenant) and the agency-handoff story (an agency manages a
client's tenant for a year, then hands the tenant back to the
client) both require **importing a bundle exported under tenant A
into tenant B**, with Cedar gates verifying the destination tenant
has accepted the import and the source tenant has authorized the
export. Without cross-tenant restore, both stories require manual
data-engineering work that violates the no-silent-regression
doctrine and the autonomous-implementation goal.

### Why 5+ year re-import compatibility

The 5-year window comes from three converging constraints:

- **Audit retention regulations.** SOX (7 years), HIPAA (6 years),
  GDPR-derived national-law tax retention (typically 10 years in
  the EU, 5 years in KR per PIPA general statute). A tenant who
  exits and is audited 4 years later must be able to read their
  export.
- **ADR-0037 Tier-1 deprecation cycle.** A 12-month deprecation
  window starts on a *Tier-1 surface change*, but format-level
  compatibility extends beyond a single deprecation: the v1 reader
  must work against v1.0, v1.1, v1.2 bundles indefinitely; the v2
  reader must read v1 and v2 bundles for at least the 5-year
  sunset window before v1 reads are retired.
- **Industry convention.** Google Takeout has maintained a stable
  takeout-archive format across multiple Google product
  rearchitectures (2011-2026, 14 years). Apple Data + Privacy has
  similarly stable format across iOS 13 → iOS 17 (2019-2024). The
  industry bar for portability formats is "decade-stable."

5 years is the *minimum* commitment; in practice we plan for 10.

### What this ADR does not decide

- **Encryption at rest for the data inside the bundle.** §D-7 picks
  the encryption *option set* (no encryption, oyatie default key,
  tenant encryption-BYOK) and the algorithm constraints (AES-256-GCM for
  symmetric, X25519 for key wrap), but the operational rotation
  cadence for the oyatie default key belongs in ADR-0043 (secrets
  management).
- **The DSAR cascade itself.** ADR-0038 (trust framework + DSAR
  cascade + proof of erasure) already decides the DSAR protocol.
  This ADR consumes ADR-0038's cascade output for the portability
  use case; it does not redefine DSAR.
- **Storage of generated bundles.** Where bundles live (object
  store, signed URL TTL, retention period) belongs in a separate
  ADR. This ADR specifies the format only.
- **Billing for export.** GDPR mandates one free export per data
  subject per reasonable interval; commercial billing for bulk
  full-exports beyond the regulatory minimum is a billing-policy
  decision outside this ADR's scope.

## Decision

### D-1: Format — JSON-LD 1.1 with per-µservice schemas referenced by URI

The canonical wire format is **JSON-LD 1.1** per the W3C
Recommendation of 2020-07-16. Every exported document is a JSON-LD
node with a mandatory `@context` field referencing the per-µservice
schema URI:

```json
{
  "@context": "https://contracts.oyatie.dev/portability/v1/mail/message.jsonld",
  "@type": "MailMessage",
  "@id": "urn:oyatie:tenant:t-7421:mail:message:0193af2c-...",
  "tenantId": "t-7421",
  "subject": "Q1 plan",
  "body": {
    "@type": "MailMessageBody",
    "contentType": "text/markdown",
    "content": "..."
  },
  "sentAt": "2026-04-12T09:11:14Z",
  "from": {"@id": "urn:oyatie:tenant:t-7421:identity:user:u-882"},
  "to": [{"@id": "urn:oyatie:tenant:t-7421:identity:user:u-901"}]
}
```

Schema URIs follow the pattern
`https://contracts.oyatie.dev/portability/v1/<microservice>/<class>.jsonld`
where `<microservice>` is the µservice's flat-catalog name (per
ADR-0131) and `<class>` is the lowercased data-class noun (`message`,
`conversation`, `file`, `event`, `principal`, etc.).

Each schema URI resolves to a JSON-LD context document plus an
accompanying JSON Schema (Draft 2020-12) document at
`https://contracts.oyatie.dev/portability/v1/<microservice>/<class>.schema.json`.
The JSON-LD `@context` provides the semantic mapping; the JSON Schema
provides the validation constraints. The two are co-versioned: a
breaking change in either bumps the path-prefix major version
(`/portability/v1/` → `/portability/v2/`).

Constraints on the JSON-LD:

- Every node carries `@type` and `@id`.
- `@id` follows the URN pattern
  `urn:oyatie:tenant:<tenant-id>:<microservice>:<class>:<uuid>`.
- Cross-µservice references use `@id` URN references — never inline
  expansion — so the dependency graph is explicit and importers can
  resolve references after all classes are loaded.
- Timestamps are RFC 3339 with explicit UTC offset (`Z` or
  `±HH:MM`); no naive timestamps.
- Binary payloads (file contents, attachment bodies, image
  fragments) are stored as separate files in the tar archive and
  referenced by relative path inside a `"contentRef"` field, never
  inlined as base64 inside the JSON-LD document.

### D-2: Per-tenant export bundle — tar.gz with manifest + signature

The export bundle is a single **gzipped tar archive** (`.tar.gz`)
with the following layout:

```
<tenant-id>-export-<utc-timestamp>.tar.gz
  └── manifest.json                  (JSON Schema Draft 2020-12, signed)
  └── manifest.json.sig              (Ed25519 detached tenant signature)
  └── manifest.json.cosign.bundle    (Sigstore cosign attestation, oyatie)
  └── data/
      └── <microservice>/
          └── <class>/
              └── <shard>.jsonld     (JSON-LD documents, one per shard)
              └── blobs/
                  └── <sha256>.bin   (binary payloads referenced by contentRef)
  └── audit/
      └── chain.jsonl                (audit events in canonical order)
      └── merkle-root.json           (Merkle root + inclusion proofs)
      └── merkle-root.json.sig       (Ed25519 audit-chain signature)
  └── ontology/
      └── types/<type>.jsonld        (Ontology Object Types per ADR-0006)
      └── instances/<type>/<shard>.jsonld
  └── README.md                      (human-readable navigation, multilingual)
  └── LICENSE-data-export.txt        (tenant-controlled re-use license)
```

Tar (POSIX ustar / pax extension headers) is chosen over zip because:

- It preserves arbitrary file metadata via pax headers (creation
  timestamp, owner, permissions) without forcing a Windows-attribute
  model.
- It streams without seeking, so generation can run as a streaming
  pipeline against the µservice fan-out without buffering the
  whole bundle in memory.
- It composes cleanly with gzip (single-pipe streaming) for
  compression.
- Every Unix-derived OS, every container runtime, and every major
  language ecosystem includes a tar reader. JSON ecosystems may
  not include zip readers (Node's stdlib historically did not).

Gzip is chosen over zstd because gzip readers are universal in
2026; zstd, while more efficient, is not yet in every long-tail
target language. A future v2 of the format MAY adopt zstd.

### D-3: Manifest schema — JSON Schema declaring data classes + counts + signatures

The `manifest.json` document conforms to JSON Schema Draft 2020-12
and declares:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.oyatie.dev/portability/v1/manifest.schema.json",
  "formatVersion": "1.0.0",
  "bundleId": "urn:oyatie:export:t-7421:2026-05-20T10:11:12Z",
  "tenantId": "t-7421",
  "exportMode": "full",
  "exportRangeStart": null,
  "exportRangeEnd": "2026-05-20T10:11:12Z",
  "generatedAt": "2026-05-20T10:11:34Z",
  "generatedBy": {
    "principal": "oyatie.portability-export.api",
    "deploymentCell": "eu-central-1-pack-eu",
    "softwareVersion": "portability-export@1.4.7"
  },
  "regionalPack": "eu",
  "complianceContext": ["gdpr-article-20", "eu-ai-act-tier-2"],
  "languageHints": ["en", "ko", "de"],
  "encryption": {
    "mode": "tenant-byok",
    "kmsKeyRef": "urn:oyatie:tenant:t-7421:kms:keys:k-aa11",
    "dekWrapAlgorithm": "X25519-HKDF-SHA256-AES-256-GCM"
  },
  "dataClasses": [
    {
      "microservice": "mail",
      "class": "message",
      "schemaUri": "https://contracts.oyatie.dev/portability/v1/mail/message.jsonld",
      "instanceCount": 18342,
      "shardCount": 4,
      "shards": [
        {"path": "data/mail/message/0000.jsonld", "instanceCount": 4586, "sha256": "..."},
        {"path": "data/mail/message/0001.jsonld", "instanceCount": 4585, "sha256": "..."},
        {"path": "data/mail/message/0002.jsonld", "instanceCount": 4586, "sha256": "..."},
        {"path": "data/mail/message/0003.jsonld", "instanceCount": 4585, "sha256": "..."}
      ],
      "blobCount": 942,
      "blobBytes": 1843921007
    }
  ],
  "ontology": {
    "typeCount": 47,
    "instanceCount": 91204,
    "typesPath": "ontology/types/",
    "instancesPath": "ontology/instances/"
  },
  "audit": {
    "eventCount": 4112847,
    "chainPath": "audit/chain.jsonl",
    "merkleRoot": "0x...",
    "merkleProofPath": "audit/merkle-root.json"
  },
  "signatures": {
    "tenant": {
      "algorithm": "Ed25519",
      "publicKey": "...",
      "signaturePath": "manifest.json.sig",
      "signedAt": "2026-05-20T10:12:00Z"
    },
    "oyatie": {
      "algorithm": "sigstore-cosign",
      "transparencyLog": "https://rekor.sigstore.dev",
      "logIndex": 184729114,
      "bundlePath": "manifest.json.cosign.bundle",
      "signedAt": "2026-05-20T10:12:01Z"
    }
  },
  "schemaVersionsResolvedAt": "2026-05-20T10:11:12Z",
  "deprecationWarnings": []
}
```

Mandatory invariants enforced by `oya gate validate
manifest-schema-conformance`:

- `formatVersion` is semver and matches the bundle path prefix.
- `tenantId` appears in every `@id` URN inside the bundle.
- `dataClasses[*].schemaUri` resolves to a live schema at validation
  time *or* the bundle includes the schema document inline under
  `schemas/<microservice>/<class>.jsonld` (for offline / air-gapped
  re-import per ADR-0240 sovereign-cloud + air-gap).
- `dataClasses[*].instanceCount` equals the sum of
  `dataClasses[*].shards[*].instanceCount`.
- Every shard's `sha256` matches the hash of its file contents.
- `signatures.tenant` is present and verifies; `signatures.oyatie`
  is present and verifies against the Sigstore trust root.

### D-4: Per-µservice contribution to export

Each µservice contributes its own data classes via a per-µservice
**export adapter** living in
`microservices/<microservice>/exporters/portability/`. The adapter
implements the `oya-portability-export-port` trait declared in
`oya-portability-export-domain`:

```rust
#[async_trait]
pub trait PortabilityExporter: Send + Sync {
    fn microservice_name(&self) -> &'static str;
    fn data_classes(&self) -> &'static [DataClassDescriptor];
    async fn export_full(
        &self,
        ctx: &ExportContext,
        sink: &mut dyn ShardSink,
    ) -> Result<DataClassExportSummary, ExportError>;
    async fn export_incremental(
        &self,
        ctx: &ExportContext,
        cursor: &IncrementalCursor,
        sink: &mut dyn ShardSink,
    ) -> Result<DataClassExportSummary, ExportError>;
    fn schema_uri(&self, class: &str) -> &'static str;
}
```

The full list of in-scope contributors at v1.0 (every µservice that
holds tenant-attributable data per the data-class registry,
ADR-0099):

- `mail` — mailboxes, threads, messages, attachments, labels,
  filter rules, signatures.
- `messenger` — conversations, messages, reactions, message edits,
  message deletions (tombstones preserved per ADR-0038), presence
  history (opt-in).
- `drive` — files, folders, share grants, version history, comments,
  trash entries.
- `calendar` — calendars, events, attendees, recurrence rules, RSVP
  history, free/busy aggregates.
- `hr` — employee records, org-chart edges (subject's role and direct
  relationships only), leave balances, time-off requests.
- `workflow-studio` — workflow definitions, run histories, variable
  state, custom node libraries.
- `ontology` — Ontology Object Types and instances per §D-5.
- `audit-chain` — audit events scoped to the tenant per §D-6.
- `iam` — principals, groups, role bindings, MFA factors (public
  metadata only, not secrets), API keys (revoked snapshots only —
  active key material is excluded per §D-7 boundary).
- `billing` — invoices, line items, payment-method metadata
  (PCI-scope fields excluded; reference tokens only), credit notes.
- `connector` — social-graph edges scoped to the tenant, post history,
  reaction history.
- `search` — saved searches, search-history opt-in records.
- `intelligence` — prompt history (opt-in per ADR-0220 successor
  ADR-0255), saved chats, custom-instruction state.
- `foundry-self-modification-records` — for `oyatie`-tenant exports
  only (ADR-0247 self-hosting): autonomous-workflow decision
  records, ADR drafts, eval runs.
- `marketplace` — purchased plugins, plugin configuration state,
  publisher records for tenants who also publish.
- `compute`, `network`, `kms`, `dns`, `iam-bridge` — where the
  tenant has provisioned cloud resources, the metadata describing
  those resources (not the runtime state of the resources
  themselves — that lives in the customer's own infrastructure for
  key-custody-BYOK cloud per ADR-0044).

Any µservice that **does not** export tenant data declares
`exports_portability_data: false` in its µservice manifest; the
`oya gate validate per-microservice-export-coverage` lane verifies
that every µservice either ships an exporter or carries the
explicit opt-out.

### D-5: Ontology Object Type instances exported in full

Per ADR-0006 (typed entity layer) and feedback_glossary_ontology_not_object_graph,
the Ontology is the tenant-owned semantic graph. The export
includes:

- **All Object Types** the tenant has authored, customized, or
  inherited from regional packs, under `ontology/types/<type>.jsonld`.
  Inherited-but-uncustomized types from canonical packs are
  referenced by URI rather than copied (the destination should
  resolve them from the same canonical-pack distribution); inline
  copies are emitted if the manifest's `regionalPack` does not
  match the destination's pack OR if the source tenant has
  modified the type.
- **All instances** of every Object Type that the tenant owns,
  under `ontology/instances/<type>/<shard>.jsonld`. Sharding follows
  the 64-MiB-uncompressed soft cap per shard.
- **All edges** (typed relationships between instances), serialized
  with the relationship's `@type` and both endpoint `@id` URNs.
- **All property histories** where the Object Type has versioned
  properties enabled, serialized as JSON-LD nodes with `@type:
  PropertyVersion`.

The Ontology export is the **functional equivalent of a Palantir
Foundry / Palantir AIP Ontology export** in scope; the data model
follows ADR-0006 directly.

### D-6: Audit chain export with Merkle proof

Per ADR-0003 (audit chain), every tenant's audit events form a
content-addressed chain. The export includes:

- `audit/chain.jsonl` — newline-delimited JSON, one event per line,
  in canonical chain order. Each event carries its `eventId`
  (content hash), `prevEventId`, `timestamp`, `principal`, `tenant`,
  `microservice`, `action`, `targetRef`, `outcome`, and any
  µservice-specific payload.
- `audit/merkle-root.json` — the Merkle root over all exported
  events, the inclusion proof for the first and last events, and
  every 1024th event as an anchor (for log-time random-access
  verification).
- `audit/merkle-root.json.sig` — Ed25519 detached signature over
  the Merkle root by the per-cell audit-chain signing key. This
  signature is independent of the manifest signature so an importer
  can verify the audit chain even if the rest of the bundle is
  corrupted.

The Merkle structure follows RFC 9162 (Certificate Transparency
v2) Merkle tree construction: SHA-256 leaf hashes prefixed with
`0x00`, internal-node hashes prefixed with `0x01`, the root is the
hash of the topmost internal node.

An importer who only wants to attest that a given audit event
existed at export time can verify a single inclusion proof against
the signed Merkle root without parsing the rest of the bundle.

### D-7: Encryption — tenant's choice

The exporter offers three encryption modes, selected by the tenant
at export-request time and recorded in `manifest.json` under
`encryption.mode`:

- **`none`** — no encryption applied at the bundle layer. The
  tenant is responsible for protecting the bundle at rest after
  download. Permitted only when the request is fulfilled over an
  end-to-end TLS channel (signed URL with short TTL) and the
  destination is explicitly opted-in. Some regulators (notably
  certain financial-services regimes) disallow this mode; the
  Cedar policy enforced at export-request admission blocks `none`
  in those compliance contexts.
- **`oyatie-default-key`** — bundle encrypted with a per-export
  AES-256-GCM data encryption key (DEK), the DEK wrapped with the
  oyatie regional-pack default KMS key. The tenant receives the
  wrapped DEK in a sidecar artifact and the unwrap operation
  requires a tenant-authenticated call to the oyatie KMS at
  decrypt time. This mode is the path of least operational
  friction.
- **`tenant-byok`** — bundle encrypted with a per-export DEK,
  the DEK wrapped with the tenant's encryption-BYOK key (X25519 public key
  recipient or AWS-KMS/Azure-Key-Vault/GCP-KMS key reference).
  Oyatie never sees the unwrapped DEK; only the tenant or their
  KMS can decrypt. This mode is the path tenants exiting to a
  peer controller will typically choose.

Algorithm choices are locked at v1.0:

- Symmetric: **AES-256-GCM** with a 96-bit random IV and 128-bit
  authentication tag. IV is stored alongside the ciphertext in
  the encrypted-shard envelope.
- Asymmetric (recipient): **X25519** for public-key wrapping (RFC
  7748) composed with HKDF-SHA-256 key derivation and AES-256-GCM
  data encryption (the `X25519-HKDF-SHA256-AES-256-GCM` AEAD).
- KMS wrap: per-provider KMS key references resolved at export
  time; the manifest records the key reference (not the key
  material).

Encryption is at the **shard** layer (each `data/<microservice>/<class>/<shard>.jsonld`
and each blob file is independently encrypted), not at the
archive layer. This means: a tenant can selectively decrypt only
the classes they need without unwrapping the whole bundle, and a
corrupted shard does not poison the rest of the bundle.

### D-8: Signed export bundle — Ed25519 + Sigstore cosign attestation

Every bundle carries **two independent signatures** per §D-2:

- **Tenant Ed25519 detached signature** at `manifest.json.sig`. The
  tenant signs the manifest with their portability signing key
  registered at export-request time. This signature attests that
  the tenant received and accepted this bundle as their
  authoritative export.
- **Oyatie Sigstore cosign attestation** at
  `manifest.json.cosign.bundle`. The oyatie portability-export
  µservice signs the manifest with its workload identity using
  Sigstore's keyless flow (per ADR-0039), producing a
  cosign-attestation bundle that includes the certificate, the
  Rekor inclusion proof, and the Fulcio-issued ephemeral
  certificate. This signature attests that the oyatie controller
  generated this bundle from tenant state at the manifest's
  `exportRangeEnd` timestamp.

Verification is performed by `oya port verify <bundle>`. The CLI
verifies both signatures, the Merkle root signature, every shard
hash against the manifest, and (if cosign is reachable) the Rekor
transparency-log inclusion. Air-gapped verification is supported
via a `--offline-trust-bundle` flag that consumes a pre-fetched
trust bundle.

Both signatures sign **the manifest, not the bundle contents
directly**. The manifest contains the sha256 of every shard and
blob, so signing the manifest transitively binds the entire
bundle. This shrinks the signing surface from gigabytes (raw
bundle) to kilobytes (manifest) without weakening the integrity
guarantee.

### D-9: Full export vs incremental

Two modes, selected per request:

- **Full.** `exportMode: "full"`, `exportRangeStart: null`. The
  bundle contains every tenant-owned record at
  `exportRangeEnd`. This is the GDPR Article 20 fulfillment mode
  and the contractual-exit deliverable mode.
- **Incremental.** `exportMode: "incremental"`,
  `exportRangeStart: <RFC-3339-timestamp>`,
  `exportRangeEnd: <RFC-3339-timestamp>`. The bundle contains
  every record changed (created, updated, soft-deleted, restored)
  between the two timestamps, plus a tombstone entry for hard-
  deleted records. Each µservice's exporter consumes the previous
  bundle's `incrementalCursor.<microservice>` opaque-string
  cursor and emits a fresh cursor in the new bundle.

Incremental bundles cannot stand alone — they only make sense
applied to a prior full bundle. The manifest records
`baseBundleId` referencing the prior full bundle's `bundleId`.
Importers refuse to apply an incremental bundle without the named
base bundle being already imported.

A **synthesizer** µservice operation (`oya port synthesize
<base-full> <incremental-1> <incremental-2> ...`) produces a fresh
full bundle from a base + a sequence of incrementals. This
operation is used during long re-import workflows to flatten a
chain of incrementals into a single self-contained bundle.

### D-10: Cross-tenant restore

The import µservice accepts a bundle exported under tenant `A`
and imports it into tenant `B` under the following constraints,
enforced at admission by Cedar (per ADR-0243):

- **Source consent.** Tenant `A`'s export request must carry a
  Cedar-evaluated `crossTenantRestoreAuthorization` claim listing
  the permitted destination tenants. This claim travels inside the
  manifest under `crossTenantAuthorization.destinations`.
- **Destination acceptance.** Tenant `B` must issue a Cedar-
  evaluated `crossTenantRestoreAcceptance` decision at import
  request time, referencing the source bundle's `bundleId`.
- **Principal mapping.** The import adapter applies a
  principal-mapping table (provided in the import request) that
  rewrites every `@id` URN of pattern
  `urn:oyatie:tenant:A:identity:user:<u>` to
  `urn:oyatie:tenant:B:identity:user:<u'>` where `<u'>` is the
  corresponding principal under tenant `B`. Unmapped principals
  cause the import to halt with a `principal-mapping-incomplete`
  error.
- **Data-class scope.** The import request may scope to a subset
  of data classes (e.g., "import only `drive.file` and
  `calendar.event`"). Unscoped classes are skipped without error.
- **Audit-chain disposition.** The source bundle's audit chain is
  imported as a *referenced external chain* — it is not merged
  into tenant `B`'s audit chain. The audit records become
  read-only historical artifacts attached to the imported
  entities, preserving the source-controller attestation.

Cross-tenant restore powers:

- B2C-to-B2B migration (personal user upgrades to business plan;
  personal-tenant data lands in the new business-tenant).
- Agency handoff (agency-tenant data lands in client-tenant).
- M&A (acquired-company tenant data lands in acquirer tenant).
- Multi-tenant consolidation (small subsidiary tenants land in a
  parent tenant during corporate reorganization).
- DR-style restore where a tenant was destroyed and re-provisioned
  under a different ID (ADR-0241 DR portfolio).

### D-11: GDPR 30-day SLA for fulfillment

The portability-export µservice exposes the export-request API
with the following SLAs, encoded in the public contract and
enforced by error budgets (per the observability substrate, ADR-0130):

- **Acknowledgment.** 200 OK on the request endpoint within 5
  seconds. The response carries an opaque `exportJobId`.
- **Job-state visibility.** The status endpoint reports `queued`,
  `running`, `ready`, `failed`, `expired` within 60 seconds of
  the state transition. The 99th percentile state-transition-to-
  visibility latency target is 30 seconds.
- **Bundle availability.** The bundle is available for download
  within **30 calendar days** of the request, per GDPR Article
  12(3). The internal target is 7 calendar days for tenants under
  the median tenant-data-volume; tenants above the 99th
  percentile may take longer but never more than 30 days.
- **Download window.** The bundle download URL is valid for at
  least 30 calendar days after `ready`. Tenants may request a
  refresh URL during that window.
- **Retention after download.** The bundle is retained at the
  oyatie storage substrate for a tenant-configurable period
  (default 90 days) after `ready`. Tenants may request earlier
  deletion via the standard data-deletion API.

Failure to meet the 30-day SLA escalates per ADR-0038's DSAR
cascade as a compliance incident. Failure to meet the 7-day
internal target raises an SRE incident at the per-cell
portability-export error budget burner.

KR-PIPA Article 35-2 imposes the same 30-day envelope; CCPA's
45-day window is satisfied by the stricter GDPR window.

### D-12: Re-import compatibility guarantee — 5+ year sunset

The format carries an explicit compatibility commitment encoded
in the contract registry (ADR-0011):

- **Format v1.0** readers MUST read v1.0, v1.1, v1.2, ..., v1.N
  bundles. Minor versions add optional fields and new schema URIs
  under `/portability/v1/` only; they never remove fields or
  rename them in-place.
- **Format v2.0** readers MUST read v2.0 bundles natively. They
  MUST ALSO read v1.N bundles for at least **5 years after v2.0
  GA**. The recommended target is 10 years to align with
  industry-standard portability formats (Google Takeout, Apple
  Data + Privacy).
- **Deprecation cycle for format-level changes.** A breaking
  format change (major-version bump) follows the ADR-0037 Tier-1
  sunset path: 12-month deprecation notice, dual-write window
  (both v1 and v2 bundles emitted for every export request during
  the dual-write window), regulator notification to the relevant
  EU DPAs and KR PIPC at notice time, and explicit per-tenant
  notification through the data-subject portal.
- **Per-class schema deprecation.** Within a major version, a
  per-class schema MAY be deprecated. Deprecated schemas remain
  resolvable for the duration of the major version. The
  `deprecationWarnings` array in the manifest enumerates any
  deprecated schemas the bundle references, including the
  deprecation date and the replacement schema URI.

The 5-year minimum is enforced by the format-evolution gate:
deprecation of a v1.N schema requires an ADR amendment to ADR-
0276 with explicit sunset-date reasoning, and the gate refuses
the change if the sunset is earlier than 5 years after v1.0 GA.

### D-13: Per-µservice import adapters

The import µservice (`microservices/portability-import/`) mirrors
the export µservice's per-µservice adapter pattern. Each µservice
provides an importer in
`microservices/<microservice>/importers/portability/` implementing:

```rust
#[async_trait]
pub trait PortabilityImporter: Send + Sync {
    fn microservice_name(&self) -> &'static str;
    fn accepted_data_classes(&self) -> &'static [DataClassDescriptor];
    async fn import_full(
        &self,
        ctx: &ImportContext,
        source: &mut dyn ShardSource,
    ) -> Result<DataClassImportSummary, ImportError>;
    async fn import_incremental(
        &self,
        ctx: &ImportContext,
        cursor: &IncrementalCursor,
        source: &mut dyn ShardSource,
    ) -> Result<DataClassImportSummary, ImportError>;
    async fn validate(
        &self,
        ctx: &ImportContext,
        source: &mut dyn ShardSource,
    ) -> Result<DataClassValidationSummary, ImportError>;
}
```

The importer first runs in `validate` mode (no writes, only
schema-validation + reference-resolution + Cedar-precheck) and
emits a per-class validation report. The tenant then approves
the import, after which `import_full` or `import_incremental`
runs against the µservice's write path.

Cross-µservice references are resolved in two passes:

- **Pass 1.** Every µservice imports its own records with
  cross-µservice references **deferred** (placeholder rows with
  `import_state: pending_reference`).
- **Pass 2.** Every µservice resolves its deferred references.
  Any unresolved reference at the end of pass 2 raises an
  `import-reference-unresolved` error with the full reference
  graph for the operator.

This two-pass model handles every cross-µservice reference shape
without requiring a topologically-ordered import sequence (which
would re-couple µservices through an external ordering
authority).

### D-14: Schema evolution for the export format itself — versioned + backward-compatible

The export format is a Tier-1 public contract. Evolution follows
the rules below, enforced by the contract registry (ADR-0011)
and the `oya gate validate schema-evolution-backward-compat`
lane:

- **Within a major version (v1 → v1.N).** Additive changes only.
  Permitted: new optional fields with documented defaults; new
  schema URIs for new data classes; new optional manifest fields;
  new optional encryption modes; new optional signature
  algorithms in *parallel to* existing ones (never replacing).
  Forbidden: removing fields; renaming fields; changing field
  types; changing required-ness; removing schema URIs; changing
  the path of the manifest or any data shard inside the tar.
- **Across major versions (v1 → v2).** Anything is permitted in
  v2 within the format-design constraints of this ADR (JSON-LD,
  tar.gz, dual signatures). Compatibility is achieved by
  emitting **both** v1 and v2 bundles during the dual-write
  window (see §D-12) and by maintaining v1 readers for the 5-year
  sunset window.
- **Schema version registry.** Every schema URI carries an
  `oya:schemaVersion` annotation in its JSON-LD `@context`. The
  contract registry mirrors every schema version with a
  fingerprint hash; CI lanes diff schema versions on every PR
  that touches `microservices/*/exporters/portability/`.
- **Deprecation surface.** The manifest's `deprecationWarnings`
  array lists every deprecated schema referenced by the bundle.
  Importers MAY emit operator warnings on deprecation but MUST
  succeed if the schema is still within its support window.
- **No silent regression.** Per feedback_no_silent_regression,
  any format-breaking change requires an ADR (amendment to ADR-
  0276 or a successor), a version bump, a sunset window, and
  regulator notification. The format-evolution lane refuses
  silent removals or renames.
- **Schema validation strictness.** v1.0 readers operate in
  "permissive" mode: unknown fields in known schemas are ignored
  (forward-compatibility within the major version), but missing
  required fields fail validation. v2.0 readers reading v1.N
  bundles operate in "translating" mode: a registered v1-to-v2
  upgrade transform converts the bundle in memory before the v2
  reader proceeds.

## Alternatives considered

### Alternative A: One monolithic JSON Schema for the entire export

A single schema describing every data class in the entire oyatie
catalog, versioned as one artifact.

- **Pros.** One schema URL to remember. One version number. One
  place to look up every class.
- **Cons.** Re-couples every µservice through a shared schema
  artifact, violating ADR-0145 (inter-microservice communication
  reform). A new µservice can't ship its export without
  modifying the monolith. The monolith grows unboundedly. A
  single breaking change forces every µservice's exporter to
  re-version even when their own schema is unchanged. Fails the
  flat-product-catalog doctrine (feedback_flat_product_catalog).
- **Verdict.** Rejected.

### Alternative B: Protobuf wire format

Use Protocol Buffers (binary) with `.proto` schema files
distributed alongside the bundle.

- **Pros.** Compact. Schema-strict. Excellent code-generation
  story across many languages.
- **Cons.** Schema-without-the-schema is opaque bytes. Fails the
  Article 20 "commonly used" test for peer controllers who may
  not have a protobuf toolchain — they would need to install
  protoc and generate readers before reading anything. JSON-LD,
  by contrast, opens in any text editor and is plain JSON to
  every JSON reader.
- **Verdict.** Rejected for the wire format. (Protobuf MAY be
  used internally between µservices per ADR-0145; this ADR is
  specifically about the *external* format.)

### Alternative C: SQL dump

PostgreSQL `pg_dump` SQL output, one file per µservice, or one
file per tenant.

- **Pros.** Industry-standard for database snapshots. Direct
  re-import into a peer Postgres.
- **Cons.** Locks the destination to PostgreSQL (or a Postgres-
  compatible engine). Fails Article 20's "transmit to another
  controller" test for any controller running a different DBMS.
  Exposes oyatie's internal schema choices to the world as a
  public contract, violating ADR-0145's "no internal schema is
  external contract" rule. Doesn't carry cross-µservice
  reference semantics at the right level (foreign-key
  references inside SQL are tied to oyatie's specific schema).
- **Verdict.** Rejected.

### Alternative D: ActivityPub-formatted JSON

Use ActivityStreams 2.0 / ActivityPub Object types for every data
class.

- **Pros.** Already a W3C Recommendation; widely implemented by
  Mastodon, the Fediverse, and peer social platforms.
- **Cons.** Designed for social-activity streams; doesn't model
  files, calendars, mailboxes, HR records, or any of the broader
  µservice surface. Would force shoehorning every data class into
  Activity types, losing semantic precision.
- **Verdict.** Rejected as the general format. ActivityPub MAY
  appear as a profile-specific export option for the `connector`
  µservice's social-graph data (a future v1.N addition).

### Alternative E: Proprietary binary container with vendor-specific reader

A single proprietary `.oyaback` archive with a vendor-supplied
reader CLI.

- **Pros.** Maximum flexibility. Could carry any internal
  representation directly.
- **Cons.** Fails every Article 20 obligation. The entire point
  of Article 20 is that the tenant should not need the original
  controller's tooling to read the export.
- **Verdict.** Rejected categorically.

### Alternative F: SCIM 2.0 + per-µservice extensions

Use the SCIM (System for Cross-domain Identity Management,
RFC 7643/7644) JSON schema as the base envelope with per-µservice
extensions.

- **Pros.** Industry standard for identity portability. Existing
  importers in Okta, Microsoft Entra, Google Workspace.
- **Cons.** Scoped to identity; doesn't model the broader product
  surface. JSON-LD is a strict superset (a SCIM JSON document is
  trivially expressible as JSON-LD with a SCIM `@context`).
- **Verdict.** Rejected as the base format; SCIM-shaped extension
  for `iam` is a v1.N addition.

### Alternative G: One bundle per µservice (no aggregator)

Each µservice produces and signs its own bundle independently;
there is no per-tenant aggregator bundle.

- **Pros.** Tighter coupling between µservice and its export.
  Lower aggregation cost.
- **Cons.** Tenants must download N bundles, verify N
  signatures, resolve cross-µservice references across N
  archives. Importers must orchestrate N adapter pipelines
  without a manifest. Audit chain becomes scoped to per-µservice
  events losing the cross-µservice causal links. Cross-tenant
  restore loses the cohesion guarantee that the source tenant
  state was a coherent snapshot.
- **Verdict.** Rejected. The per-tenant aggregator bundle is the
  primary unit of portability; per-µservice bundling is an
  internal-tooling detail.

### Alternative H: Streaming HTTP API (no bundle artifact)

Expose a streaming HTTP API; tenants pipe the stream directly
into peer controllers without producing a file artifact.

- **Pros.** No retention; no signed-URL TTL; no bundle storage
  cost.
- **Cons.** Article 20 explicitly contemplates a transmittable
  artifact. Tenants frequently need to verify-then-import in
  separate operations (separated by days or weeks). Audits
  reference the bundle. Re-import after 5 years requires that
  the bundle existed.
- **Verdict.** Rejected as the sole mode. The streaming API MAY
  be offered as a supplementary delivery mode for tenants who
  want it (a future v1.N addition), but the bundle is always
  available.

## Consequences

### Positive

- **Article 20 compliance.** Every Article 20 right is fulfillable
  through a single API call producing a structured, commonly-used,
  machine-readable bundle.
- **Parallel-regulation compliance.** KR-PIPA 35-2, CCPA, LGPD,
  PIPEDA, POPIA, and other parallel rights are fulfilled by the
  same mechanism.
- **Tenant exit is real.** A tenant can decide to leave oyatie at
  any time and walk away with every byte of their data.
- **Tenant re-entry is real.** A tenant who left can re-enter
  oyatie or hand their bundle to a successor controller, with the
  dual-signature attestation establishing that the bundle is
  authentic.
- **Self-hosting consistency.** The `oyatie` tenant exports under
  the same path as customer tenants (per ADR-0247 self-hosting
  doctrine). Backup of oyatie itself is the same operation as
  backup of any tenant.
- **Audit-chain portability.** Tenants leaving carry their audit
  chain with cryptographic attestation, preserving the regulator-
  visible evidence chain even after exit.
- **Ontology portability.** Tenants carry their Ontology
  customizations, satisfying the Palantir-parallel claim of the
  product positioning (feedback_workflow_objectgraph_adapter_layer).
- **Schema co-evolution governance.** The contract registry
  (ADR-0011) gains a Tier-1 surface for portability schemas with
  the same governance as inter-µservice contracts.

### Negative / costs

- **Aggregation cost.** Generating a full bundle for a large
  tenant is expensive. The exporter must be optimized to stream,
  shard, and parallelize across µservices.
- **Storage cost during retention.** Bundles live for 90 days
  default after generation. For large tenants this is significant.
  ADR-0049 (cross-region replication + residency) determines
  where; this ADR determines how long.
- **Schema-evolution discipline.** Every µservice must maintain
  its portability schema for at least 5 years. Schema deprecation
  becomes a multispectrum-reviewed event.
- **Cross-tenant restore complexity.** Principal mapping, Cedar
  acceptance, audit-chain disposition — each is a non-trivial
  operational surface.
- **Key-management complexity.** Three encryption modes triple
  the test surface compared to a single mode.

### Neutral / explicit non-decisions

- This ADR does not decide where the exporter runs (per-cell,
  per-pack, central). ADR-0028 cloud-microservice-architecture +
  ADR-0254 deployment-model-spectrum decide deployment topology.
- This ADR does not decide the billing model for export beyond
  the regulatory-minimum-free guarantee.
- This ADR does not decide the storage backend for completed
  bundles (object store, signed-URL TTL).

## Implementation surface

### New crates

- `oya-portability-export-domain` — manifest types, bundle types,
  `PortabilityExporter` trait, error taxonomy. Layer-1 (domain).
- `oya-portability-export-kernel` — exporter orchestrator,
  per-µservice fan-out, sharding, encryption pipeline,
  signature emission. Layer-2 (kernel).
- `oya-portability-export-app` — request lifecycle, job state,
  status API, download URL signing. Layer-3 (app).
- `oya-portability-export-api` — HTTP API surface
  (`/v1/exports`, `/v1/exports/{id}`, `/v1/exports/{id}/download`).
  Layer-4 (api).
- `oya-portability-import-domain` — `PortabilityImporter` trait,
  import-context types, validation-report types.
- `oya-portability-import-kernel` — import orchestrator,
  two-pass reference resolver, principal-mapper.
- `oya-portability-import-app` — request lifecycle, validation
  API, approval gate, import-execution API.
- `oya-portability-import-api` — HTTP API surface.
- `oya-portability-manifest-schema` — manifest JSON Schema
  + validator + types shared between export and import.
- `oya-portability-jsonld-context` — JSON-LD context document
  registry + URI-to-schema resolver.
- `oya-portability-merkle` — RFC 9162 Merkle tree builder and
  proof generator/verifier for the audit-chain export.
- `oya-portability-cli` — `oya port export`, `oya port verify`,
  `oya port import`, `oya port synthesize` subcommands.
- `oya-portability-cosign-attestation` — Sigstore cosign
  attestation generator + verifier for manifests.

### Per-µservice additions

Each in-scope µservice (per §D-4) ships:

- `microservices/<ms>/exporters/portability/Cargo.toml` —
  exporter crate depending on `oya-portability-export-domain`.
- `microservices/<ms>/exporters/portability/src/lib.rs` —
  implements `PortabilityExporter`.
- `microservices/<ms>/importers/portability/Cargo.toml` —
  importer crate.
- `microservices/<ms>/importers/portability/src/lib.rs` —
  implements `PortabilityImporter`.
- `microservices/<ms>/schemas/portability/v1/<class>.jsonld` —
  JSON-LD context for each data class.
- `microservices/<ms>/schemas/portability/v1/<class>.schema.json`
  — JSON Schema validator for each data class.
- `microservices/<ms>/tests/portability_roundtrip.rs` — golden
  roundtrip test: export → verify → import → diff against
  source.

### Specs

- `/specs/data-portability-format.json` — canonical declaration
  of the format invariants, schema URI patterns, manifest
  required fields.
- `/specs/microservices/portability-export.json` — µservice
  spec.
- `/specs/microservices/portability-import.json` — µservice
  spec.
- `/specs/ontology-export-schema.json` — ontology-specific
  export schema declarations.
- `/specs/manifest-schema.json` — manifest schema doc.

### CI lanes (advisory-until-substrate-lands → BLOCKER)

- `oya gate validate portability-format-coherence` — top-level
  lane that orchestrates the others.
- `oya gate validate manifest-schema-conformance` — manifest
  validates against `manifest.schema.json` and all referenced
  schemas resolve.
- `oya gate validate per-microservice-export-coverage` — every
  µservice ships exporter+importer or carries explicit opt-out.
- `oya gate validate ontology-export-completeness` — exported
  ontology matches source ontology types + instances.
- `oya gate validate audit-chain-merkle-proof` — exported
  Merkle root verifies against source audit chain.
- `oya gate validate dual-signature-attestation` — both
  signatures verify and cosign Rekor inclusion is present.
- `oya gate validate import-roundtrip-fidelity` — golden
  roundtrip per §B fixture produces bit-identical entities.
- `oya gate validate schema-evolution-backward-compat` — every
  schema change is additive within a major version.

### Migration / sequencing

1. **Wave 1 (week 0).** Land `oya-portability-export-domain` +
   `oya-portability-export-kernel` + manifest schema. Smoke-test
   with a single µservice exporter (`mail`).
2. **Wave 2 (week 4).** Expand to `messenger`, `drive`,
   `calendar`, `iam`. Land import substrate. Land cross-tenant
   restore admission gate.
3. **Wave 3 (week 8).** Expand to `workflow-studio`, `ontology`,
   `audit-chain`, `hr`, `billing`, `connector`, `search`,
   `intelligence`.
4. **Wave 4 (week 12).** Expand to `marketplace`,
   `foundry-self-modification-records`, `compute`, `network`,
   `kms`, `dns`. Promote CI lanes from advisory to BLOCKER.
5. **Wave 5 (week 16).** Public-contract registration; regulator
   notification of v1.0 GA; tenant-facing documentation;
   developer-portal landing for peer-importer authors.

### Cell + regional pack interaction

The exporter runs per-cell (each tenant lives in exactly one
cell per ADR-0009). Cross-cell aggregation is unnecessary; a
tenant's bundle is generated entirely inside the cell that owns
that tenant. Regional pack settings (per ADR-0010) influence:

- The `regionalPack` field in the manifest.
- The default oyatie KMS key for `oyatie-default-key` encryption
  mode.
- The Korean-language documentation requirement for KR-pack
  exports (a `README.ko.md` companion to the bundle's
  `README.md`).
- The `complianceContext` array (e.g., `["gdpr-article-20",
  "kr-pipa-35-2", "eu-ai-act-tier-2"]`).

### Sovereign-cloud + air-gap

Per ADR-0240, sovereign-cloud deployments host the exporter
inside the sovereign envelope. Air-gap deployments include:

- The full schema corpus inlined under `schemas/` inside the
  bundle (not just URI references).
- A bundled trust-bundle for cosign offline verification (per
  Sigstore air-gap pattern).
- A manifest field `offlineMode: true` indicating the inline
  resolution path.

### Performance + sustainability

Export jobs are expected to dominate per-tenant batch-compute
spend at the upper tail. Optimizations baked into v1.0:

- Streaming exporter: no whole-bundle buffering. Memory ceiling
  per job is bounded by max shard size (64 MiB uncompressed) ×
  concurrent µservices.
- Per-class parallelization: every µservice runs concurrently
  with bounded concurrency per cell.
- Shard-level deduplication for large blobs: identical blobs
  across multiple references share a single `blobs/<sha256>.bin`
  entry.
- Incremental mode default: scheduled (non-exit) exports default
  to incremental after the first full bundle.

## Verification

### Roundtrip fidelity

The primary correctness signal is bit-identical roundtrip:

1. Generate a synthetic tenant state per the §B fixture.
2. Run `oya port export --tenant t-fixture --mode full`.
3. Verify the bundle with `oya port verify`.
4. Run `oya port import --bundle <path> --tenant t-fixture-restore`.
5. Diff the destination tenant state against the source.

The diff must be empty modulo the deliberate identity changes
(new tenant ID, new principal IDs, new audit-chain head). The
CI lane `import-roundtrip-fidelity` runs this against the
fixture on every PR touching portability-export, portability-
import, or any µservice's portability adapter.

### Signature verification

`oya port verify <bundle>` performs:

- Tenant Ed25519 signature verification against the registered
  tenant portability public key.
- Sigstore cosign attestation verification: ephemeral
  certificate chain to Fulcio root, Rekor inclusion proof, log
  signature.
- Per-shard sha256 verification against manifest hashes.
- Audit-chain Merkle-root signature verification.
- Manifest schema conformance.

The CLI exits with non-zero on any failure, listing every
defect.

### Cross-tenant restore policy

`oya port import --source-tenant A --destination-tenant B` is
gated by Cedar at admission. The policy fragment:

```cedar
permit (
  principal in TenantAdmin::<destination-tenant>,
  action == Action::"portability:import",
  resource == Bundle::<bundle-id>
)
when {
  resource.crossTenantAuthorization.destinations.contains(<destination-tenant>) &&
  context.crossTenantRestoreAcceptance == "issued" &&
  context.principalMappingComplete == true
};
```

The verification lane `cross-tenant-restore-policy` runs Cedar
policy tests against the §B fixture.

### Schema evolution

Every PR that touches a `microservices/*/schemas/portability/`
file runs the schema-diff lane. The lane refuses removals,
renames, type changes, or required-ness changes within a major
version. The lane accepts additive changes silently and emits
deprecation warnings for fields marked `oya:deprecated: true`.

### Performance budgets

Per ADR-0128 (hyperscaler architecture invariants) and
feedback_quality_performance_scalability_bar:

- A median-tenant full export (≈ 10 GiB uncompressed) completes
  in ≤ 30 minutes (P50) / ≤ 2 hours (P99).
- The exporter saturates at ≥ 60% of available cell-portability
  CPU; below saturation, throughput scales linearly with
  µservice fan-out concurrency.
- Re-import of the same bundle completes in ≤ 90 minutes (P50)
  / ≤ 4 hours (P99) — slower because importer two-pass
  reference resolution is sequential per pass.
- Incremental exports for a 1%-daily-change tenant complete in
  ≤ 5 minutes (P50).

### SLO authoring

Per ADR-0130 (agentic SLO-gated promotion), `microservices/portability-export/slos/*.openslo.yaml`
declares:

- `portability_export_request_acknowledgment_latency_p99 ≤ 5s`
- `portability_export_job_completion_within_7d ≥ 0.999`
- `portability_export_job_completion_within_30d ≥ 0.99999`
- `portability_bundle_verification_failure_rate ≤ 0.0001`

Per-µservice exporter crates declare contribution SLOs for the
fan-out.

## References

### Normative — regulation

- **Regulation (EU) 2016/679 (GDPR), Article 20** — "Right to
  data portability." Text:
  https://eur-lex.europa.eu/eli/reg/2016/679/oj#Article_20
- **GDPR Article 12(3)** — fulfillment deadline (one month
  baseline; extendable by two months for complex requests with
  notice).
- **Article 29 Working Party Guidelines on the right to data
  portability** (WP242 rev.01, 2017-04-05) — interpretive
  guidance, including the "commonly used machine-readable
  format" gloss.
- **European Data Protection Board endorsement of WP242** —
  carried forward into the GDPR-era EDPB position.
- **KR-PIPA (Personal Information Protection Act), Article 35-2**
  — Korean data-portability right, in force 2024-03-15 per the
  2023 amendment.
- **KR PIPC implementing decree** — Korean-language declaration
  requirement for portability surfaces.
- **CCPA §1798.130(a)(2)** — California right to a copy of
  personal information.
- **CCPA §1798.100(d)** — portability format requirements.
- **CPRA §1798.135** — successor amendments to CCPA.
- **LGPD (Brazil) Article 18, VI** — Brazilian data-portability
  right.
- **PIPEDA Principle 9** — Canadian individual-access principle
  inclusive of portability.
- **POPIA §23 (South Africa)** — data-subject-access right
  inclusive of portability.

### Normative — technical standards

- **W3C JSON-LD 1.1** — Recommendation 2020-07-16. URL:
  https://www.w3.org/TR/json-ld11/
- **JSON Schema Draft 2020-12** — URL:
  https://json-schema.org/draft/2020-12/release-notes.html
- **RFC 8259** — JSON specification.
- **RFC 3339** — Date and Time on the Internet timestamps.
- **RFC 9162** — Certificate Transparency Version 2.0 (Merkle
  tree construction).
- **RFC 7748** — Elliptic curves for security (X25519).
- **RFC 8032** — Edwards-curve Digital Signature Algorithm
  (Ed25519).
- **RFC 5869** — HKDF (HMAC-based Key Derivation Function).
- **NIST SP 800-38D** — AES-GCM specification.
- **POSIX tar / pax interchange format** — IEEE Std 1003.1.
- **RFC 1952** — gzip file format.

### Normative — supply-chain + signing

- **Sigstore cosign** — keyless signing with Fulcio + Rekor.
  URL: https://docs.sigstore.dev/
- **Sigstore Rekor** — transparency log specification.
- **Sigstore Fulcio** — short-lived-certificate issuer.
- **in-toto attestation spec** — companion attestation format
  Sigstore-attestations.

### Industry pattern

- **Google Takeout** — multi-product portability bundle pattern;
  in production since 2011 across ~50 Google products. URL:
  https://takeout.google.com/
- **Apple Data and Privacy export** — multi-product portability
  bundle for Apple ID. URL: https://privacy.apple.com/
- **Microsoft Privacy Dashboard** — analogous portability surface.
- **Meta Download Your Information** — Facebook/Instagram
  portability bundles.
- **Data Transfer Project** — cross-controller portability
  protocol prototype (Google + Microsoft + Apple + Meta + Twitter
  + SmugMug, 2018-2023).
- **SCIM 2.0** — RFC 7643/7644 cross-controller identity
  schema; influence on `iam` exporter.
- **ActivityPub** — W3C Recommendation 2018-01-23; influence
  on `connector` social-graph exporter (future profile).

### Internal — oyatie ADRs

- **ADR-0002** — tenant + identity kernel; tenant ID is the
  scoping primitive of every URN.
- **ADR-0003** — audit chain + evidence emission; per-tenant
  chain that this ADR exports.
- **ADR-0006** — ontology / typed entity layer; the substrate
  whose instances §D-5 exports.
- **ADR-0011** — cross-µservice contract registry; the registry
  this ADR's schemas inhabit.
- **ADR-0037** — public API stability tiers; the Tier-1 cycle
  this ADR's format inherits.
- **ADR-0038** — trust framework + DSAR cascade + proof of
  erasure; the parallel ADR for the delete/erase right.
- **ADR-0039** — supply-chain security (Sigstore cosign reuse).
- **ADR-0049** — cross-region replication and residency.
- **ADR-0099** — data-class registry; the source of truth for
  per-µservice exported data classes.
- **ADR-0105** — 13-layer canonical enum (which layers the new
  crates inhabit).
- **ADR-0128** — hyperscaler architecture invariants
  (performance bars).
- **ADR-0130** — agentic SLO-gated promotion.
- **ADR-0131** — per-µservice flat layout.
- **ADR-0145** — inter-µservice communication reform.
- **ADR-0150** — Cedar policy engine.
- **ADR-0180** — stateful disaster recovery.
- **ADR-0211** — in-house tech-stack preference.
- **ADR-0218** — tenant granular control surface.
- **ADR-0240** — sovereign cloud per regional pack.
- **ADR-0241** — DR + business continuity portfolio policy.
- **ADR-0242** — `oyatie` is a tenant doctrine; explains why
  oyatie itself exports under the same path.
- **ADR-0243** — Cedar as universal gate; gates this ADR's
  admission.
- **ADR-0244** — tenant as universal scoping primitive.
- **ADR-0245** — substrate vs product layering.
- **ADR-0247** — self-hosting / self-modification doctrine.

## Appendix A — Pattern attribution

This ADR's design borrows from multiple prior-art patterns. The
attribution table below records what was borrowed and where the
pattern was tightened or differs.

| Pattern | Source | Borrowed | Where we differ |
|---|---|---|---|
| Multi-product portability bundle | Google Takeout (2011-2026) | Per-product subdirectories under a single archive; manifest enumerating products | We use JSON-LD with schema URIs (Takeout uses opaque per-product formats); we ship dual signatures (Takeout is unsigned); we publish the schema as a Tier-1 contract (Takeout's per-product formats are not contractual) |
| User-controlled key wrapping | Signal protocol; AWS S3 SSE-C; Tink envelope encryption | Per-bundle DEK wrapped under tenant or platform key; algorithm choices (AES-256-GCM, X25519) | We require dual signatures over the manifest (not just the body); we explicitly support three modes (most prior art picks one) |
| Manifest-signs-the-bundle | Sigstore cosign attestation pattern; in-toto attestations | Signing the manifest transitively signs the bundle via per-shard hashes | Direct adoption |
| Merkle-tree audit-chain export | Certificate Transparency (RFC 9162); Trillian-backed transparency logs | RFC 9162 Merkle construction; signed root + inclusion proofs | We anchor every 1024th event for log-time random-access verification; CT anchors more sparsely |
| Cross-controller portability protocol | Data Transfer Project (2018-2023) | Per-service adapter pattern; principal mapping at handoff | We do not require destination-controller pre-registration with a central directory (DTP did); we use Cedar at admission instead |
| JSON-LD as the wire format for portability | W3C Verifiable Credentials data model; Solid project portability profile | JSON-LD 1.1 with `@context` referencing canonical schemas | Direct adoption |
| Per-class schema URI registry | OCF (Open Container Initiative) image-spec mediaType registry; HL7 FHIR profile URL pattern | URI-as-schema-pointer; versioned path prefix | Direct adoption |
| Two-pass reference resolution on import | Database bulk-import pattern (PostgreSQL `COPY` with deferred FKs); LDIF imports | Defer cross-µservice refs to pass 2; halt on unresolved at end of pass 2 | Direct adoption |
| Incremental + base bundle relationship | rsync delta-transfer; PostgreSQL WAL shipping; git pack-file delta encoding | Cursor-based per-µservice incremental | We do not use binary deltas at the JSON-LD layer (records are reissued in full per change); shard-level dedup recovers most of the efficiency |
| Tar.gz as the archive format | Linux distribution package format (.tar.gz, .tar.xz); Docker image layers (tarballs); macOS .tar.gz exports | Universal-reader archive with streaming generation | Direct adoption |
| Per-cell exporter (no cross-cell aggregation) | AWS Backup per-Region service model; GCP regional buckets | Tenant lives in one cell; export is cell-local | Direct adoption |
| Korean-language documentation requirement | KR PIPC implementing decree on portability declarations | Bundle includes `README.ko.md` for KR-pack exports | Direct adoption |
| 30-day SLA | GDPR Article 12(3) | Acknowledgment immediate; bundle within 30 days | We add an internal 7-day target tracked by SRE error budgets |
| Format-version-bumps trigger Tier-1 sunset | ADR-0037 Tier-1 stability tier | 12-month sunset for breaking format changes | We add explicit regulator notification at notice time |
| Self-tenant exports (oyatie of oyatie) | ADR-0247 self-hosting / self-modification doctrine | The oyatie tenant uses the same export path as customer tenants | Direct adoption |

## Appendix B — Worked example: tenant exits and re-imports later

### B.1 Setup

`Acme Ltd.` is a B2B oyatie tenant (`tenant-id: t-acme-7421`) in
the EU regional pack. They have used oyatie for 18 months across
six µservices (`mail`, `drive`, `calendar`, `messenger`,
`workflow-studio`, `ontology`). They have approximately:

- Mail: 184,000 messages across 142 mailboxes, 89 GiB of attachment
  blobs.
- Drive: 47,000 files, 412 GiB of file blobs.
- Calendar: 28,000 events across 87 calendars.
- Messenger: 1.4 million messages across 6,200 conversations.
- Workflow Studio: 412 workflow definitions, 184,000 run records.
- Ontology: 47 custom Object Types, 91,000 instances.
- Audit chain: 4.1 million events scoped to t-acme-7421.

Acme decides to migrate to a peer ecosystem. They invoke their
Article 20 right.

### B.2 Day 0 — request

The tenant admin (`principal: tenant-acme-7421.admin.u-alice`)
calls the export API:

```http
POST /v1/exports
Authorization: Bearer <token>
Content-Type: application/json

{
  "tenantId": "t-acme-7421",
  "mode": "full",
  "encryption": {
    "mode": "tenant-byok",
    "recipientPublicKey": "X25519:<base64...>"
  },
  "complianceContext": ["gdpr-article-20"],
  "languageHints": ["en", "ko"]
}
```

Response within 3 seconds:

```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "exportJobId": "urn:oyatie:export-job:t-acme-7421:0193b1...",
  "status": "queued",
  "statusEndpoint": "/v1/exports/urn:oyatie:export-job:t-acme-7421:0193b1...",
  "estimatedReadyAt": "2026-05-23T10:11:34Z"
}
```

Cedar admission gate evaluates:
- `principal.role == TenantAdmin`: pass.
- `tenant.allowExport == true` (Cedar default): pass.
- `compliance.gdpr-article-20.allowed-this-quarter`: pass.
- `encryption.mode == tenant-byok` accepted: pass.

The job is admitted to the per-cell exporter queue.

### B.3 Day 0 → Day 3 — generation

The exporter fans out across six µservice exporters in parallel
(`mail`, `drive`, `calendar`, `messenger`, `workflow-studio`,
`ontology`) plus the audit-chain exporter and the IAM exporter.

Each per-µservice exporter:

- Reads records from its source-of-truth store in canonical
  order.
- Serializes each record as a JSON-LD document conforming to the
  per-class schema.
- Writes to its shard sink. The sink shards at 64 MiB
  uncompressed and writes `data/<microservice>/<class>/<NNNN>.jsonld`.
- Streams binary payloads to `data/<microservice>/<class>/blobs/<sha256>.bin`.
- Computes per-shard sha256 as it writes.

The audit-chain exporter walks the per-tenant chain, emits
`audit/chain.jsonl`, computes the Merkle root anchoring every
1024th event, writes `audit/merkle-root.json`, and signs it with
the per-cell audit-chain key.

The ontology exporter emits `ontology/types/<type>.jsonld` for
every Object Type Acme has authored or customized, and
`ontology/instances/<type>/<NNNN>.jsonld` for every instance.

When all per-µservice exporters complete (Day 3 09:14 UTC), the
aggregator:

- Writes the manifest with every shard reference, instance count,
  sha256, blob count, ontology summary, audit summary, and
  compliance context.
- Encrypts each shard with a per-export DEK (AES-256-GCM, fresh
  IV per shard).
- Wraps the DEK under the tenant's X25519 recipient key
  (X25519-HKDF-SHA256-AES-256-GCM AEAD).
- Computes the tenant-signature placeholder: the manifest is
  signed by the oyatie portability-export workload identity first
  (Sigstore cosign keyless flow producing
  `manifest.json.cosign.bundle`).
- Generates a tenant-signing URL the tenant uses to apply their
  Ed25519 signature to the manifest. The tenant signs offline and
  uploads the signature; the aggregator writes
  `manifest.json.sig`.
- Tars + gzips the archive into `t-acme-7421-export-2026-05-23T10-11-34Z.tar.gz`.
- Uploads to the per-tenant download bucket with a 30-day signed
  URL TTL.

### B.4 Day 3 — download + verify

The tenant downloads the bundle (412 GiB compressed) and verifies:

```bash
oya port verify t-acme-7421-export-2026-05-23T10-11-34Z.tar.gz \
  --tenant-public-key acme-portability-pubkey.pem
```

The verifier:

1. Extracts and validates `manifest.json` against the manifest
   JSON Schema. PASS.
2. Verifies the tenant Ed25519 signature against the supplied
   tenant public key. PASS.
3. Verifies the oyatie cosign attestation: chain to Fulcio root
   PASS; Rekor inclusion proof PASS (logIndex 184,729,114, log
   leaf hash matches).
4. Iterates every shard and verifies its sha256 against the
   manifest. 4,212 shards verified PASS.
5. Iterates every blob and verifies its sha256-as-filename. 71,294
   blobs verified PASS.
6. Verifies the audit-chain Merkle root signature against the
   per-cell audit-chain public key. PASS.
7. Verifies a random sample of 100 audit events against the
   Merkle inclusion proofs. PASS.

`oya port verify` exits 0. Acme writes the bundle to their own
long-term retention archive (cold object store, 7-year retention
under their financial-services retention policy).

### B.5 Day 3 → Day 90 — Acme uses the bundle in a peer ecosystem

Acme builds a per-class importer for their peer ecosystem. They
reference `https://contracts.oyatie.dev/portability/v1/mail/message.jsonld`
to understand the structure. The peer importer reads JSON-LD
plain (without JSON-LD-specific tooling), validating against the
JSON Schema mirror at
`https://contracts.oyatie.dev/portability/v1/mail/message.schema.json`.

Acme's import to the peer ecosystem completes in 6 weeks. Their
audit chain is attached to the peer system as a read-only
historical artifact preserving the oyatie attestation.

### B.6 Year 4 — regulator audit

In 2030, a regulator opens a compliance audit against Acme's
2026 data-handling practices. The regulator requests Acme produce
their data-state-as-of-2026.

Acme produces the original `t-acme-7421-export-2026-05-23T10-11-34Z.tar.gz`
bundle from cold storage. The regulator runs `oya port verify`
against the v1.0 reader (still supported per §D-12's 5+ year
sunset). All signatures verify; the cosign Rekor inclusion proof
re-verifies against the still-online Rekor log (Sigstore's log
is durable beyond the 5-year window). The audit-chain Merkle
proofs verify. The regulator accepts the bundle as authoritative
evidence of Acme's 2026 data state, including the oyatie
attestation that the bundle reflects state at the manifest
timestamp.

### B.7 Year 5 — Acme returns

In 2031, Acme returns to oyatie. They open a new B2B tenant
(`tenant-id: t-acme-returning-9912`).

The tenant admin calls the import API:

```http
POST /v1/imports
Authorization: Bearer <token>
Content-Type: application/json

{
  "destinationTenantId": "t-acme-returning-9912",
  "bundlePath": "<signed-upload-url>",
  "sourceTenantId": "t-acme-7421",
  "principalMapping": {
    "urn:oyatie:tenant:t-acme-7421:identity:user:u-alice": "urn:oyatie:tenant:t-acme-returning-9912:identity:user:u-alice2",
    "urn:oyatie:tenant:t-acme-7421:identity:user:u-bob": "urn:oyatie:tenant:t-acme-returning-9912:identity:user:u-bob2",
    "...": "..."
  },
  "scopeDataClasses": [
    "mail.message", "mail.mailbox",
    "drive.file", "drive.folder",
    "calendar.event", "calendar.calendar",
    "workflow-studio.workflow",
    "ontology.type", "ontology.instance"
  ]
}
```

The importer:

1. Validates the bundle (decrypts with Acme's still-archived
   recipient key; verifies signatures against the still-published
   trust roots).
2. Confirms the manifest declares format version `1.0.0`. The
   2031 reader (now at format v1.4) reads v1.0 in permissive
   mode.
3. Runs the validation pass: schema-validates every record;
   resolves every cross-µservice reference; runs Cedar
   admission ("can this destination tenant accept this bundle?").
4. The Cedar gate evaluates `crossTenantAuthorization.destinations`
   in the source manifest. Acme's 2026 export carried
   `crossTenantAuthorization.destinations: ["*"]` (Acme
   authorized any future destination), so the gate passes.
5. The destination admin issues `crossTenantRestoreAcceptance`.
6. The two-pass importer runs: pass 1 imports records with
   placeholder references; pass 2 resolves references with the
   principal-mapping table.
7. Audit chain is imported as a read-only historical artifact.
8. Ontology types + instances are imported; type customizations
   from 2026 are preserved.

The import completes in 14 hours (412 GiB of payload, two-pass
import). Acme is operational on `t-acme-returning-9912` with
their 2026 data restored.

The audit chain attached to imported entities shows the original
2026 oyatie attestation alongside the 2031 oyatie re-import
attestation, preserving the full evidentiary trail.

### B.8 Year 5+ — schema evolution observed

By 2031, the format has evolved to v1.4. Changes since v1.0:

- v1.1 (2027-Q1) — added `workflow-studio.workflow-version`
  schema for finer-grained workflow versioning.
- v1.2 (2028-Q3) — added `intelligence.saved-chat` schema for
  chat-history export.
- v1.3 (2029-Q4) — added optional `accessibility.preferences`
  field across user-level schemas.
- v1.4 (2030-Q4) — added `marketplace.publisher-record` for
  tenants who publish plugins.

None of those changes broke v1.0 compatibility. The 2026 bundle
read by the v1.4 reader produces identical entities as it would
have produced under the v1.0 reader. The `deprecationWarnings`
array in the 2026 bundle remains empty (v1.0's surface had no
deprecations at v1.4 time).

When v2.0 ships (planned 2032), the dual-write window opens:
every export request between 2032-01 and 2033-01 produces both a
v1.N bundle and a v2.0 bundle. After 2033-01, v2.0 is the
default; v1.N remains supported as a request flag through at
least 2037 (5-year sunset from v2.0 GA per §D-12).

Acme's 2026 bundle remains readable by every v1.x reader
through at least 2037 and likely beyond, satisfying the
decade-stable industry bar.

## Appendix C — JSON Schema fragment: manifest.schema.json (excerpt)

The full manifest schema lives at
`oya-portability-manifest-schema/schemas/manifest.schema.json`.
The excerpt below shows the top-level required structure.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.oyatie.dev/portability/v1/manifest.schema.json",
  "title": "Oyatie Portability Bundle Manifest v1",
  "type": "object",
  "required": [
    "formatVersion",
    "bundleId",
    "tenantId",
    "exportMode",
    "exportRangeEnd",
    "generatedAt",
    "generatedBy",
    "regionalPack",
    "complianceContext",
    "encryption",
    "dataClasses",
    "audit",
    "signatures"
  ],
  "properties": {
    "formatVersion": {
      "type": "string",
      "pattern": "^1\\.[0-9]+\\.[0-9]+$"
    },
    "bundleId": {
      "type": "string",
      "pattern": "^urn:oyatie:export:[a-z0-9-]+:[0-9TZ:.-]+$"
    },
    "tenantId": {
      "type": "string",
      "minLength": 1
    },
    "exportMode": {
      "type": "string",
      "enum": ["full", "incremental"]
    },
    "exportRangeStart": {
      "oneOf": [{"type": "null"}, {"type": "string", "format": "date-time"}]
    },
    "exportRangeEnd": {
      "type": "string",
      "format": "date-time"
    },
    "baseBundleId": {
      "type": ["string", "null"],
      "description": "Required when exportMode == incremental"
    },
    "incrementalCursor": {
      "type": ["object", "null"],
      "additionalProperties": {"type": "string"}
    },
    "generatedAt": {"type": "string", "format": "date-time"},
    "generatedBy": {
      "type": "object",
      "required": ["principal", "deploymentCell", "softwareVersion"],
      "properties": {
        "principal": {"type": "string"},
        "deploymentCell": {"type": "string"},
        "softwareVersion": {"type": "string"}
      }
    },
    "regionalPack": {"type": "string"},
    "complianceContext": {
      "type": "array",
      "items": {"type": "string"}
    },
    "languageHints": {
      "type": "array",
      "items": {"type": "string"}
    },
    "encryption": {
      "type": "object",
      "required": ["mode"],
      "properties": {
        "mode": {
          "type": "string",
          "enum": ["none", "oyatie-default-key", "tenant-byok"]
        },
        "kmsKeyRef": {"type": "string"},
        "dekWrapAlgorithm": {"type": "string"},
        "recipientPublicKey": {"type": "string"}
      }
    },
    "crossTenantAuthorization": {
      "type": "object",
      "properties": {
        "destinations": {
          "type": "array",
          "items": {"type": "string"}
        }
      }
    },
    "dataClasses": {
      "type": "array",
      "items": {"$ref": "#/$defs/dataClass"}
    },
    "ontology": {"$ref": "#/$defs/ontology"},
    "audit": {"$ref": "#/$defs/audit"},
    "signatures": {"$ref": "#/$defs/signatures"},
    "deprecationWarnings": {
      "type": "array",
      "items": {"$ref": "#/$defs/deprecationWarning"}
    }
  },
  "$defs": {
    "dataClass": {
      "type": "object",
      "required": [
        "microservice", "class", "schemaUri",
        "instanceCount", "shardCount", "shards"
      ],
      "properties": {
        "microservice": {"type": "string"},
        "class": {"type": "string"},
        "schemaUri": {"type": "string", "format": "uri"},
        "instanceCount": {"type": "integer", "minimum": 0},
        "shardCount": {"type": "integer", "minimum": 0},
        "shards": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["path", "instanceCount", "sha256"],
            "properties": {
              "path": {"type": "string"},
              "instanceCount": {"type": "integer", "minimum": 0},
              "sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"}
            }
          }
        },
        "blobCount": {"type": "integer", "minimum": 0},
        "blobBytes": {"type": "integer", "minimum": 0}
      }
    },
    "ontology": {
      "type": "object",
      "properties": {
        "typeCount": {"type": "integer", "minimum": 0},
        "instanceCount": {"type": "integer", "minimum": 0},
        "typesPath": {"type": "string"},
        "instancesPath": {"type": "string"}
      }
    },
    "audit": {
      "type": "object",
      "required": ["eventCount", "chainPath", "merkleRoot", "merkleProofPath"],
      "properties": {
        "eventCount": {"type": "integer", "minimum": 0},
        "chainPath": {"type": "string"},
        "merkleRoot": {"type": "string", "pattern": "^0x[0-9a-f]{64}$"},
        "merkleProofPath": {"type": "string"}
      }
    },
    "signatures": {
      "type": "object",
      "required": ["tenant", "oyatie"],
      "properties": {
        "tenant": {
          "type": "object",
          "required": ["algorithm", "publicKey", "signaturePath", "signedAt"],
          "properties": {
            "algorithm": {"type": "string", "enum": ["Ed25519"]},
            "publicKey": {"type": "string"},
            "signaturePath": {"type": "string"},
            "signedAt": {"type": "string", "format": "date-time"}
          }
        },
        "oyatie": {
          "type": "object",
          "required": ["algorithm", "transparencyLog", "bundlePath", "signedAt"],
          "properties": {
            "algorithm": {"type": "string", "enum": ["sigstore-cosign"]},
            "transparencyLog": {"type": "string", "format": "uri"},
            "logIndex": {"type": "integer"},
            "bundlePath": {"type": "string"},
            "signedAt": {"type": "string", "format": "date-time"}
          }
        }
      }
    },
    "deprecationWarning": {
      "type": "object",
      "required": ["schemaUri", "deprecatedAt", "replacementSchemaUri"],
      "properties": {
        "schemaUri": {"type": "string", "format": "uri"},
        "deprecatedAt": {"type": "string", "format": "date"},
        "replacementSchemaUri": {"type": "string", "format": "uri"},
        "supportEndsAt": {"type": "string", "format": "date"}
      }
    }
  }
}
```

## Appendix D — Open questions

The following questions are explicitly deferred from this ADR's
scope and are tracked as follow-up items:

- **D.1 Per-class JSON-LD `@context` registry hosting.** Where do
  the `https://contracts.oyatie.dev/portability/v1/*` URIs
  resolve to? The contract registry (ADR-0011) currently hosts
  internal contracts; extending it for public-facing schema
  hosting requires a separate ADR on the public contracts
  surface.
- **D.2 Streaming HTTP delivery mode.** Should we offer the
  bundle as a streaming HTTP response in addition to a stored
  artifact? Several enterprise tenants have asked for this for
  weekly continuous-export pipelines. Plausibly a v1.1 addition.
- **D.3 Webhook-driven import-completion notification.** Where
  do completion notifications land? ADR-0112 (webhook-driven
  Foundry-agent invocation) suggests a webhook substrate; this
  ADR could consume it once landed.
- **D.4 SBOM-style "what produced this bundle" attestation.**
  Beyond the cosign attestation, do we want an in-toto-style
  attestation listing the exact µservice versions, the schema
  versions, and the data-class registry version? Plausibly
  yes; deferred to a follow-on ADR.
- **D.5 ActivityPub profile for `connector`.** The social-graph
  export currently uses an oyatie-specific JSON-LD context. An
  ActivityPub-compatible profile would let tenants migrate to
  Fediverse-shaped destinations natively. Deferred.
- **D.6 SCIM 2.0 profile for `iam`.** The identity export
  currently uses an oyatie-specific schema. A SCIM-compatible
  profile would let tenants migrate to Okta/Entra natively.
  Deferred.
- **D.7 Differential-privacy guarantees for aggregate exports.**
  Some compliance frames (e.g., research-data sharing) want
  differentially-private aggregate exports rather than
  identifiable records. Out of scope for the GDPR Article 20
  format; tracked under a separate analytics ADR.
- **D.8 Export-rate-limit policy.** GDPR mandates one free
  export per data subject per reasonable interval. What is
  "reasonable"? Annual? Quarterly? Per-request? Tracked
  separately as a Cedar-policy decision.
- **D.9 Tenant-side preview UI for export contents.** Before
  download, tenants likely want to preview what their bundle
  will contain. Deferred to a UX-surface ADR.
- **D.10 Bulk export for `oyatie`-tenant audit purposes.**
  When the `oyatie` tenant exports itself (per ADR-0247
  self-hosting), the bundle is much larger than any customer
  tenant. Are there special-case optimizations? Deferred.

---

End of ADR-0276.
