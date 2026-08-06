---
id: ADR-0216
status: Superseded
superseded_by: [ADR-709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0216: Open Integration and Migration-Out Policy

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, axis-platform, axis-developer-experience, council-security
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0211, ADR-0212, ADR-0213, ADR-0217, ADR-0221
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0216`
- **Task:** #E substrate doctrines follow-up

## Context

Oyatie's product thesis depends on deep integration across many business surfaces. That depth can look like lock-in if customers cannot see how they would leave, import existing data, export a complete history, or integrate third-party tools.

The PR #143 checkpoint locked the opposite doctrine: lock-in is forbidden. Trust through openness is the moat. Customers start faster when they know they can leave; they adopt more only if the platform keeps earning its place.

This doctrine is also a practical engineering constraint. Every microservice that owns customer data must expose clear contracts, import/export adapters, and plugin seams early enough that product teams cannot hide data behind bespoke UI-only workflows.

## Decision

Every customer-facing microservice that owns portable business data must ship an explicit open-integration surface:

1. first-party importer from the top three competitors or incumbent systems for that product surface;
2. first-party exporter to the top three competitors or a neutral standards-based archive;
3. OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts as the canonical integration surface where applicable;
4. plugin extension points via Wasmtime sandbox and plugin app store governance per ADR-0213;
5. open-standard compatibility where the domain has one, such as OIDC, SCIM 2.0, SAML, FHIR, ISO 20022, XBRL, OpenAPI, OpenSLO, OpenTofu, or FOCUS.

No product may present "contact support for export" as the only migration-out story. Support may assist, but the platform must own deterministic, tested export and import mechanics.

### Product PRD requirement

Each B2B SaaS product PRD must include an `Import/export adapter list` section with:

- top three import sources and why they were chosen;
- top three export targets or open archival formats;
- data classes included and excluded;
- idempotency and cursor semantics;
- audit-chain events emitted during import/export;
- tenant-admin controls and role requirements;
- known lossy fields and how the UI discloses them.

### Contract requirement

Contracts must be enough to generate working clients. New REST surfaces use OpenAPI 3.2.0. Event surfaces use AsyncAPI 3.1.0. Cross-language internal or partner interfaces use proto3 when a schema-first binary contract is justified.

## In-house roadmap

This policy is Class C doctrine per ADR-0211. The import/export adapters for each product are product-owned and in-house by default because migration trust is part of the Oyatie product promise.

Phase 1: require adapter lists in PRDs and manifests. Phase 2: add CI coverage that fails customer-facing microservices missing importer/exporter declarations. Phase 3: expose tenant-facing migration dry-runs and evidence bundles in Tenant Admin Console. Phase 4: publish external developer docs from the same contract sources.

## Alternatives considered

### Alternative 1 - Closed platform, retention through friction

**Rejected because** exit friction delays adoption and erodes trust. Customers with regulated data, board oversight, or procurement review need to know the exit plan before they approve entry.

### Alternative 2 - API-only openness

**Rejected because** an API without importer/exporter tooling still leaves customers to reverse-engineer mappings, pagination, idempotency, and audit evidence. API-only openness is useful for developers but insufficient for migration.

### Alternative 3 - Export-only openness

**Rejected because** customers also need to arrive safely. Importing from incumbent systems is where first value is proven. Export-only support would satisfy legal minimums but fail adoption.

### Alternative 4 - Per-product discretion

**Rejected because** lock-in posture is a platform-level trust property. If one product traps data, the whole Oyatie brand inherits that risk.

## Consequences

### Positive

- Customers can start with lower perceived risk because migration-out is designed from day one.
- Partner and developer ecosystems can build against stable contracts instead of UI scraping or private APIs.
- Competitive migration becomes measurable and testable rather than a services-only project.
- Open contracts support ADR-0221 citation and version-pin gates.

### Negative

- Product teams must fund import/export work before GA, not after the first enterprise asks for it.
- Some competitor APIs are incomplete or hostile, so importers may need lossy-field disclosure and manual remediation paths.
- Maintaining top-three competitor adapters is ongoing work as competitors change APIs.

### Operational

- Product manifests must declare importer/exporter coverage before GA promotion.
- Contract generation must validate OpenAPI 3.2.0 and AsyncAPI 3.1.0 versions.
- Import/export runs emit audit-chain entries with actor, source/target, counts, hashes, and failure summaries.
- Tenant admins must be able to run dry-runs before destructive or large-scale migrations.

## Named industry sources

- AWS open APIs and partner ecosystem: broad integration depth lowers adoption risk even while AWS remains sticky.
- Stripe Connect: strong APIs and migration-friendly integration patterns increase platform adoption.
- Apple ecosystem: app and data portability pressure coexists with deep product integration.
- Google Workspace APIs: organizational data surfaces expose admin and developer integration paths.
- Shopify app ecosystem: merchant trust depends on third-party extension and migration paths.

## References

- ADR-0211: In-house tech stack policy; openness and phase-out seams are part of the same lock-in discipline.
- ADR-0212: Buildability doctrine; contracts and PRDs must be implementable by cold readers.
- ADR-0213: Plugin App Store and Developer SDK provide the extension substrate.
- ADR-0217: Vertical rollout order determines when product-specific adapter depth lands.
- ADR-0221: Version-pin and buildability gates enforce the contract discipline.
