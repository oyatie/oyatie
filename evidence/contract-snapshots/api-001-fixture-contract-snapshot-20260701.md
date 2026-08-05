# API-001 fixture contract snapshot for spec/static UX descendants

Artifact: `API-001-FIXTURE-CONTRACT-SNAPSHOT-20260701`
Kanban task: `t_885677fc`
Generated UTC: 2026-07-01T09:03:39Z
JSON companion: `evidence/contract-snapshots/api-001-fixture-contract-snapshot-20260701.json`

## Claim boundary

This is a mock/fixture contract snapshot only. It lets spec and static UX descendant lanes proceed with explicit non-production data shapes while `t_9f82f89e` (`API-001: schema registry + public API versioning plane`) remains the serialized owner for production API implementation, schema registry publication, SDK/release, and integration work.

Explicit non-claims:

- No production API implementation or public API readiness is claimed.
- No Apicurio registry publication, schema-registry runtime, compatibility gate, or per-cell deployment is claimed.
- No Rust-native contract source, OpenAPI/proto/GraphQL generator, drift gate, resolver, or SDK release is implemented.
- No billing runtime, invoice ledger, provider-live resource CRUD, OpenTofu/Argo actuation, quota runtime, Cedar runtime, audit persistence, or customer-facing invoice publication is claimed.
- No generated SDK or generated face may be hand-edited from this snapshot.

## Sources inspected

- `specs/schema-registry-canonical.json:20-21,36-45,59-62,89-105` and `docs/decisions/ADR-0166-schema-registry.md:47-109` for Apicurio Registry 3.x, subject naming, semver compatibility levels, and publication/runtime-consumer assumptions.
- `docs/decisions/ADR-0258-api-versioning-model.md:56-84,86-133,135-224` for `X-Oyatie-API-Version`, internal `/vN` mesh URLs, deprecation/sunset behavior, and SDK generation assumptions.
- `docs/decisions/ADR-0257-ontology-object-type-versioning-deprecation-handshake.md:255-268,321-342,369-396` for object schema revision and ACTIVE -> DEPRECATED -> TOMBSTONED lifecycle expectations.
- `specs/api-contract-ssot-canonical.json:21-25,28-40,62-66` for the Rust-native contract SSOT target and non-claims.
- `specs/api-surface-separation.json:81-109` for public/internal surface defaults and route classification.
- `docs/decisions/ADR-0004-plane-separation-control-data-analytics.md:28-75,89-123` for plane assignments, cross-plane call declarations, review labels, and SLO expectations.
- `specs/cloud-control-plane-canonical.json:24-45,47-81,82-96,97-149`, `specs/cloud-resource-catalog-target.json:24-36,37-142,148-150`, and `specs/masterplan.json:144-257` for resource hierarchy, ORN, operation ledger, resource facets, developer portal provisioning, FOCUS/OpenCost, and non-claims.
- `specs/design-system/catalog.json:47-80` plus component specs for `audit-evidence-timeline`, `cloud-cell-topology-map`, `entity-action-policy-preview`, and `ops-deployment-status-panel` for static UX fixture assumptions.

## Shared fixture envelope

Descendant fixtures should carry these fields until API-001 ratifies exact production contract artifacts:

- `fixture_id`, `fixture_kind`, `fixture_scope`, `claim_boundary`
- `api_surface`: `public` or `internal`
- `stability_tier`
- `provisional_schema_subject`: follows `<schema-kind>.<microservice>.<surface>.<event-class-or-resource>` but is not a registry-published subject
- `schema_kind`
- `schema_revision`: SemVer for fixture/object shape evolution
- `public_api_generation`: mock date value only when `public_api_generation_is_mock=true`
- `mesh_major_version`: e.g. `v1` for internal fixture routes
- `route`, `method`, `operation_id`, `idempotency_key`
- `tenant_scope`: include organization/account/project/region/cell for cloud/control-plane resources
- `data_class`: use `INTERNAL_FIXTURE_ONLY` unless a stricter fixture class is defined
- `cedar_policy_ref`, `audit_chain_ref`
- `response_headers`: include `X-Oyatie-API-Version`, `X-Oyatie-Default-Version`, `X-Oyatie-Latest-Version`; include `Sunset`/`Deprecation` placeholders when modeling deprecated generations
- `source_refs`
- `reconciliation_after_api_001`

Recommended mock values:

- `public_api_generation = "2026-07-01"`
- `public_api_generation_is_mock = true`
- `mesh_major_version = "v1"`
- `data_class = "INTERNAL_FIXTURE_ONLY"`

## Route and schema assumptions

Public fixture APIs model ADR-0258 request-time pinning with `X-Oyatie-API-Version`; header absence must be represented as default-version resolution, not as no-version behavior. Internal fixture APIs model mesh routing with `/v1/` URL paths or `v1` package names. Fixture subjects are provisional and must be replaced or ratified by API-001 before registry publication. Default compatibility assumptions are BACKWARD for events and REST responses, FORWARD for REST requests, FULL for ontology/audit-chain, and NONE only with an explicit ADR carve-out.

Public surface defaults: `api_surface=public`, `hostname=api.oyatie.com`, `auth=oauth2-plus-key-signature`, `rate_limit_tier=per-public-key-per-ip`, `semver_discipline=mandatory`.

Internal surface defaults: `api_surface=internal`, `hostname=internal-api.oyatie.com`, `auth=mesh-mtls-spiffe-only`, `rate_limit_tier=per-microservice-internal`, `semver_discipline=optional`.

Every tenant-scoped fixture must include cross-tenant denial or zero-result behavior, Cedar/default-deny as a decision fixture only, support/admin redaction behavior, and no raw secrets, raw provider internals, credential values, or real customer invoice rows.

## Descendant safety matrix

### `t_69f99449` — PLANE-001

Safe to proceed from this snapshot for spec/manifest/gate fixture work only: plane assignments, explicit `cross_plane_calls`, reviewer labels, per-plane SLO catalog, and one invalid cross-plane fixture. Provisional subjects:

- `jsonschema.architecture.fixture.plane-assignment`
- `jsonschema.architecture.fixture.cross-plane-call-invalid`

Do not mutate production API/runtime code or claim ADR-0004 production enforcement beyond fixture/gate evidence. API-001 must replace provisional subject IDs with registry subjects before production publication.

### `t_9574c74d` — FINOPS-UX-001A

Safe to proceed from this snapshot for a static Cost Explorer budget/anomaly fixture for one tenant/account/project. Required mock fields include tenant/account/project, resource/resource_type/resource_group, cost_center, workload_class, compliance pack, region/cell, meter_dimension, usage_hour, quantity, currency, integer money/minor units, KR tax/VAT fields, price_book/rate_card reference placeholders, invoice_line_id, operation_id, audit_chain pointer, report_event_id/idempotency evidence, restatement/freeze status, and data_class. Provisional subjects:

- `jsonschema.finops.fixture.cost-explorer-budget-anomaly`
- `jsonschema.finops.fixture.invoice-line-evidence-pointer`

Do not invent customer-facing SKU names, real price numbers, real invoices, or founder-ratified pricing decisions. Do not implement billing aggregation, invoice ledger, price-book mutation, tax runtime, or external invoice publication.

### `t_c14f66bc` — CONTROLPLANE-UX-001A

Safe to proceed from this snapshot for one read-only Resource Explorer / Operation Center fixture. Required mock fields include Organization -> Account -> Project -> Region -> Cell -> Resource Group -> Resource, ORN, service/resource_type, lifecycle_state, owner, policy_state, quota_cost, billing_meters, audit_events, slo_tier, observability_ref, rollback_state, reconciliation_state, operation_id, client_idempotency_key, actor, requested_mutation, operation_state, timestamps, retry_classification, rollback/compensation state, audit_chain_ref, and error/remediation metadata. Provisional subjects:

- `jsonschema.cloud-control-plane.fixture.resource-explorer-resource`
- `jsonschema.cloud-control-plane.fixture.operation-timeline-entry`

Do not implement provider CRUD, resource-registry persistence, operation-ledger persistence, quota/billing/Cedar runtime, OpenTofu/Argo actuation, or production provisioning. Do not broaden beyond one representative resource type.

### `t_e43dadd0` — DEVPORTAL-001A

Safe to proceed from this snapshot for a fixture-backed Developer Portal approved-template provisioning flow. Required mock entities:

- `ServiceCatalogEntity`
- `ApprovedTemplate`
- `TemplateVersion`
- `TemplateParameter`
- `ProvisioningRequest`
- `ProvisioningOperation`
- `GeneratedArtifact`
- `PreviewEnvironment`
- `ResourceFacetEvidence`

Provisional subjects:

- `jsonschema.developer-portal.fixture.service-catalog-entity`
- `jsonschema.developer-portal.fixture.provisioning-request`
- `jsonschema.developer-portal.fixture.provisioning-operation`
- `jsonschema.developer-portal.fixture.generated-artifact`
- `jsonschema.developer-portal.fixture.resource-facet-evidence`

Do not run provider-live provisioning, OpenTofu/Kubernetes apply, Backstage destination work, generated JSON hand edits, or new operator CLI work.

## Reconciliation needs after API-001

1. Resolve the OpenAPI version mismatch: `schema-registry-canonical` still names OpenAPI 3.1 while `api-contract-ssot-canonical` and ADR-0258 require OpenAPI 3.2.0.
2. Ratify exact microservice, surface, and resource tokens for Apicurio subjects and replace all provisional fixture subjects before registry publication.
3. Create or ratify manifest fields for `public_api_generations`, `default_public_version`, `mesh_api_major_versions`, and `default_mesh_major_version` for any descendant that graduates from fixture to production contract.
4. Replace mock public API generation values with minted API generations and real changelog/audit-chain rows; keep fixture generation values out of production SDKs.
5. Generate or validate REST/OpenAPI, gRPC/proto3, GraphQL SDL/resolver contracts, and AsyncAPI/event contracts from the API-001 Rust-native SSOT before SDK/release work.
6. Wire compatibility checks and per-subject publication into the cloud-ci required context only after the owner lane implements them.
7. Bind ontology `schema_revision` and ACTIVE -> DEPRECATED -> TOMBSTONED lifecycle to any Resource/Template/Cost/Plane domain objects that become public contracts.
8. Reconcile child-card ADR clarification flags separately; this snapshot is non-mutating fixture evidence and does not elevate Proposed ADRs or authorize product/cloud runtime mutation.
9. Confirm release-governance impact, rollback/no-deploy rationale, and observation-harvest links when descendants move from fixture/spec to implementation/review.

## Verification expectations for descendants

- JSON/schema fixture validates with `python3 -m json.tool` or a stricter owner schema when available.
- Browser/user-story evidence is required for UI descendants; browser N/A is not acceptable for `t_9574c74d`, `t_c14f66bc`, or `t_e43dadd0`.
- Cross-tenant denial/zero-result fixture is present for any tenant-scoped data surface.
- Generated JSON, SDKs, faces, and production API artifacts are producer-owned and must not be hand-edited from this snapshot.
- Kanban handoff cites this snapshot and records the non-production fixture claim ceiling.
