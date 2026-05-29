---
doc_class: ImplementationPlan
ip_id: IP-020-catalog-layer-registration
microservice: marketing-automation
bounded_contexts: [campaign-journey, segment, suppression, attribution, consent-audience, marketplace-audience-license]
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + council-architecture
tenant_class_aware: true
---

# IP-020: Catalog Layer Registration

## A. Problem

Marketing Automation has many catalog YAMLs under `catalog/`, but the stamped IP did not say which layer registrations must exist or how they align with ADR-0105. Without catalog registration, the service cannot prove layer completeness, ownership, capability publication, or dependency boundaries for Big-8 parity.

## B. Approach

Register the current campaign-journey crate family and capability surfaces through service-local catalog files. Catalog entries must match `src/lib.rs` scaffold declarations, `manifest.json` bounded contexts, capability YAMLs, and contract files. This IP does not create new product behavior; it makes the architecture discoverable and gateable.

## C. Deliverables

| Artifact | Change |
|---|---|
| `catalog/oya-marketing-automation-campaign-journey-*.yaml` | Ensure each layer entry points to the correct source, contract, and owner. |
| `manifest.json` | Keep bounded context and dependency lists aligned with catalog registrations. |
| `src/lib.rs` | Ensure `scaffold()` exposes the same contracts and layer count expected by tests. |
| `capabilities/*.yaml` | Cross-reference capabilities to catalog entries where implementation exists. |
| `tests/integration.rs` | Keep `scaffold_declares_adr_0105_layers` and add catalog fixture validation. |

## D. Implementation

1. Inventory catalog files for app, adapter, api, cli, domain, kernel, rest, sdk, test, usecase, and worker layers.
2. Compare each catalog id to ADR-0105 layer enum and `src/domain/mod.rs` `LAYERS`.
3. Add missing references to `contracts/openapi-v1.yaml`, `contracts/asyncapi-v1.yaml`, and `contracts/marketing-automation-v1.proto`.
4. Bind each capability YAML to either a current implementation layer or a future IP slice; do not claim implemented code where only docs exist.
5. Add catalog validation that catches stale paths and duplicate layer registrations.
6. Ensure dependency list names real µservices from `manifest.json`: iam, policy-engine, workflow-engine, ontology, consent-graph, mail, messenger, analytics, intelligence, audit-chain, data-boundary, tenancy, finops, cloud-billing, crm, sites, social, contact-center, calendar.
7. Record unresolved implementation gaps as follow-ups, not fake catalog entries.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app scaffold_declares_adr_0105_layers`
- `cargo run -p oya-dev-cli -- gate validate catalog-layer-registration --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice marketing-automation`
- Manual evidence: every catalog path exists or is explicitly marked planned.

## F. Evidence

- Local catalog: `catalog/oya-marketing-automation-campaign-journey-*.yaml`.
- Local source: `src/lib.rs`, `src/domain/mod.rs`.
- Local manifest: `manifest.json`.
- Local tests: `tests/integration.rs`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Marketing APIs and workflow capabilities become discoverable as platform catalog surfaces. |
| Adobe Marketo Engage | Campaign/journey internals are registered by layer instead of hidden in one service blob. |
| Mailchimp | Audience and campaign capabilities can be traced from catalog to API and events. |

## H. Local Traceability

- Catalog glob: `catalog/oya-marketing-automation-campaign-journey-*.yaml`.
- Source file: `src/lib.rs`.
- Source file: `src/domain/mod.rs`.
- Manifest file: `manifest.json`.
- Test file: `tests/integration.rs`.
- Layer: kernel.
- Layer: domain.
- Layer: usecase.
- Layer: app.
- Layer: adapter.
- Layer: rest.
- Layer: grpc.
- Layer: worker.
- Layer: sdk.
- Layer: api.
- Dependency: `workflow-engine`.
- Dependency: `policy-engine`.
- Failure state: catalog claims a path that does not exist.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `asyncapi`, `.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-020-catalog-layer-registration.md` matched [`attribution`, `finops`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-020-catalog-layer-registration.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
