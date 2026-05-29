---
doc_class: Implementation-Plan
doc_id: IP-ADR-0339-Shared-IaC-Modules
microservice: sites
status: PROPOSED
date: 2026-05-21
owner_team: axis-sites
bounded_context: sites
implementation_phase: documentation-and-contracts-only
rust_code_status: not-authored-in-this-wave
source_adrs: ADR-0339, ADR-0322, ADR-0181, ADR-0248, ADR-0215, ADR-0218, ADR-0244, ADR-0251, ADR-0338, ADR-0340, ADR-0343, ADR-0344
lifecycle_rule: PROPOSED until the microservice wrappers invoke signed shared OpenTofu modules; ACCEPTED only after implementation evidence lands
---
# IP-ADR-0339-Shared-IaC-Modules: Sites Shared OpenTofu Module Adoption

## 1. Lifecycle, Boundary, And Stop Condition
SCOPE-001: This IP binds `sites` to ADR-0339 shared IaC module doctrine without authoring Rust, changing crates, or applying infrastructure.
SCOPE-002: Lifecycle state is PROPOSED for `sites` until the service-owned wrapper files under `microservices/sites/iac/<context>/main.tf` invoke signed cloud-iac modules and implementation evidence is reviewed.
SCOPE-003: ACCEPTED status requires a later service implementation change, not this document-stage propagation.
SCOPE-004: The only implementation authority created here is documentation intent plus manifest `iac_module_invocations` alignment for `sites`.
SCOPE-005: The stop condition for this IP is a reviewable doctrine packet: IP present, manifest field populated, PRD adoption section appended, ARCH integration section appended, and ADR citations validated.
SCOPE-006: ADR-0339 keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `sites` keeps only thin invocation wrappers and service-specific parameters.
SCOPE-007: The wrapper body for `sites` must stay at or below 80 logical lines per context after comments and blank lines are removed.
SCOPE-008: No `resource` block belongs in a `sites` per-context wrapper after migration; resource bodies belong to cloud-iac shared modules.
SCOPE-009: OpenTofu remains the IaC engine per ADR-0218; HashiCorp Terraform SaaS registry coupling is outside this design.
SCOPE-010: Tenant scoping parameters `tenant_id`, `tenant_class`, `cell_id`, and compliance-pack labels are first-class wrapper inputs for `sites`.
SCOPE-011: This IP cites ADR-0339 purpose and enforced_by lanes so downstream reviewers can validate the same doctrine without rereading the full ADR.
SCOPE-012: This IP avoids module-body authoring; Wave 15Q owns shared module bodies and catalog details under cloud-iac.

## 2. Service-Specific Dossier
DOSSIER-001: Microservice `sites` is classified here as `business application surface`.
DOSSIER-002: Owner team is `axis-sites`; wrapper ownership stays with this owner while reusable primitive ownership stays with axis-cloud-iac.
DOSSIER-003: Manifest version is `0.1.0` and schema version is `1.0`.
DOSSIER-004: Capacity scaling dimension is `per_request`; wrapper sizing must not infer a different primary load axis.
DOSSIER-005: ADR-0248 cell placement class is `Tier-3`; `sites` wrappers pass this as placement intent rather than open-coding nodepool choices.
DOSSIER-006: Baseline per-tenant CPU is `0.1` vCPU, RAM is `256` MiB, and storage is `10.0` GiB.
DOSSIER-007: Declared connection budget per tenant is valkey=2, postgres=2, outbound_http=6.
DOSSIER-008: Capacity notes: Public site render, CMS query, publish, image optimization, search, and domain-binding paths scale with request volume and publish jobs.
DOSSIER-009: DR target is RTO p99 `1800` seconds and RPO p99 `300` seconds.
DOSSIER-010: DR replication shape is `active-active-multi-az-cross-region-warm` with backup substrates `postgres_wal_g, object_storage_versioned, valkey`.
DOSSIER-011: Regulatory packs declared: kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa.
DOSSIER-012: Data classes processed: manifest-not-populated.
DOSSIER-BC-001: Bounded context `block` states: Bounded context 'block' within sites (data plane); crate count=1.
DOSSIER-BC-002: Bounded context `cdn-delivery` states: Bounded context 'cdn-delivery' within sites (control plane); crate count=5.
DOSSIER-BC-003: Bounded context `cms-collection` states: Bounded context 'cms-collection' within sites (data plane); crate count=1.
DOSSIER-BC-004: Bounded context `domain-binding` states: Bounded context 'domain-binding' within sites (control plane); crate count=3.
DOSSIER-BC-005: Bounded context `page` states: Bounded context 'page' within sites (data plane); crate count=3.
DOSSIER-BC-006: Bounded context `search` states: Bounded context 'search' within sites (data plane); crate count=1.
DOSSIER-CRATE-001: Existing crate `oya-sites-block-adapter-loro` remains untouched by this document-stage IP.
DOSSIER-CRATE-002: Existing crate `oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub` remains untouched by this document-stage IP.
DOSSIER-CRATE-003: Existing crate `oya-sites-cdn-delivery-adapter-libvips` remains untouched by this document-stage IP.
DOSSIER-CRATE-004: Existing crate `oya-sites-cdn-delivery-adapter-pandoc` remains untouched by this document-stage IP.
DOSSIER-CRATE-005: Existing crate `oya-sites-cdn-delivery-adapter-s3` remains untouched by this document-stage IP.
DOSSIER-CRATE-006: Existing crate `oya-sites-cdn-delivery-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-007: Existing crate `oya-sites-cms-collection-adapter-postgres` remains untouched by this document-stage IP.
DOSSIER-CRATE-008: Existing crate `oya-sites-domain-binding-adapter-acme` remains untouched by this document-stage IP.
DOSSIER-CRATE-009: Existing crate `oya-sites-domain-binding-adapter-cert-manager` remains untouched by this document-stage IP.
DOSSIER-CRATE-010: Existing crate `oya-sites-domain-binding-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-011: Existing crate `oya-sites-page-adapter-postgres` remains untouched by this document-stage IP.
DOSSIER-CRATE-012: Existing crate `oya-sites-page-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-013: Existing crate `oya-sites-page-kernel` remains untouched by this document-stage IP.
DOSSIER-CRATE-014: Existing crate `oya-sites-search-adapter-meilisearch` remains untouched by this document-stage IP.
DOSSIER-CRATE-015: Existing crate `oya-sites-site-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-016: Existing crate `oya-sites-site-kernel` remains untouched by this document-stage IP.
DOSSIER-CONTRACT-001: OpenAPI 3.2.0: microservices/sites/contracts/openapi/sites.yaml.
DOSSIER-CONTRACT-002: AsyncAPI 3.1.0: microservices/sites/contracts/asyncapi/sites-events.yaml.
DOSSIER-CONTRACT-003: proto3: microservices/sites/contracts/proto/sites.proto.
DOSSIER-CAPABILITY-001: T1 T0-suggest risk=minimal file=microservices/sites/capabilities/T0-suggest.yaml.
DOSSIER-CAPABILITY-002: T1 T1-assist risk=minimal file=microservices/sites/capabilities/T1-assist.yaml.
DOSSIER-CAPABILITY-003: T1 T2-auto risk=minimal file=microservices/sites/capabilities/T2-auto.yaml.

## 3. ADR-0339 Doctrine Binding
ADR0339-001: Purpose binding: collapse 385 per-service from-scratch module directories into roughly 50 shared OpenTofu primitives plus thin wrappers.
ADR0339-002: Purpose binding: reusable module bodies live under `microservices/cloud-iac/modules/<context>/<primitive>/` with catalog and signature evidence.
ADR0339-003: Purpose binding: `sites` owns primitive selection, tenant-class scope, sizing parameters, and service-specific blast-radius analysis.
ADR0339-004: Purpose binding: cloud-iac owns primitive implementation, provider constraints, input/output contracts, catalog entries, and module release signatures.
ADR0339-005: Purpose binding: wrapper files must remain OpenTofu-native and avoid provider-proprietary Terraform Cloud behavior.
ADR0339-006: Purpose binding: every consumed module pin must be explicit, versioned, and reviewable.
ADR0339-007: Purpose binding: every shared module release must be signed under ADR-0181 supply-chain discipline.
ADR0339-LANE-001: Enforced_by lane `oya-check-iac-shared-module-usage` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-002: Enforced_by lane `oya-check-iac-module-path-canonical` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-003: Enforced_by lane `oya-check-iac-module-signature-cosign` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-004: Enforced_by lane `oya-check-iac-module-pin` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-005: Enforced_by lane `oya-check-iac-opentofu-only` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-006: Enforced_by lane `oya-check-iac-thin-wrapper-line-floor` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-LANE-007: Enforced_by lane `oya-check-iac-module-catalog-discoverability` applies to `sites` once its migration bucket enters blocker mode.
ADR0339-015: ADR-0322 substance bar applies to this IP; the content below is service-specific and intentionally connects module doctrine to manifest facts.
ADR0339-016: ADR-0248 cellular topology applies because wrapper choices determine where this service lands by cell and tenant class.
ADR0339-017: ADR-0338 pod runtime tier applies because shared modules choose Kata or runc nodepool topology from manifest tier data.
ADR0339-018: ADR-0340 capacity data applies because module sizing must use declared per-tenant CPU, RAM, storage, and connection budgets.
ADR0339-019: ADR-0343 DR data applies because backup primitives and failover topology must satisfy service RTO/RPO floors.
ADR0339-020: ADR-0344 sustainability and FinOps apply because every module pin changes watts, carbon, and monthly cost envelopes.

## 4. Manifest Module Invocation Plan
MODULE-000: `sites` declares 4 shared module invocation(s) in `manifest.json#iac_module_invocations`.
MODULE-001: Contexts represented: oyatie-as-cloud-provider.
MODULE-002: Primitive names represented: dns, k8s-namespace-bootstrap, secrets-bootstrap, vpc.
MODULE-003: `oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1[both]` is selected for `sites`; blast radius is limited to that primitive release plus the `sites` wrapper variables that feed it.
MODULE-003A: `k8s-namespace-bootstrap` in context `oyatie-as-cloud-provider` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-3`, and compliance-pack labels from the wrapper.
MODULE-004: `oyatie-as-cloud-provider/secrets-bootstrap@v1[both]` is selected for `sites`; blast radius is limited to that primitive release plus the `sites` wrapper variables that feed it.
MODULE-004A: `secrets-bootstrap` in context `oyatie-as-cloud-provider` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-3`, and compliance-pack labels from the wrapper.
MODULE-005: `oyatie-as-cloud-provider/dns@v1[both]` is selected for `sites`; blast radius is limited to that primitive release plus the `sites` wrapper variables that feed it.
MODULE-005A: `dns` in context `oyatie-as-cloud-provider` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-3`, and compliance-pack labels from the wrapper.
MODULE-006: `oyatie-as-cloud-provider/vpc@v1[both]` is selected for `sites`; blast radius is limited to that primitive release plus the `sites` wrapper variables that feed it.
MODULE-006A: `vpc` in context `oyatie-as-cloud-provider` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-3`, and compliance-pack labels from the wrapper.
MODULE-FUTURE-012: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-013: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-014: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-015: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-016: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-017: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.
MODULE-FUTURE-018: Future primitive additions for `sites` require manifest amendment, catalog lookup, version pin review, and ADR-0181 signature evidence before wrapper use.

## 5. Per-Context Thin Wrapper Specifications
CTX-001-001: `aws-guest` wrapper stance for `sites`: No invocation declared yet; future use requires manifest amendment and a signed module pin.
CTX-001-002: `aws-guest` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-001-003: `aws-guest` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-001-004: `aws-guest` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-001-005: `aws-guest` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-001-006: `aws-guest` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-001-007: `aws-guest` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-001-008: `aws-guest` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-002-001: `oci-guest` wrapper stance for `sites`: No invocation declared yet; future use requires manifest amendment and a signed module pin.
CTX-002-002: `oci-guest` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-002-003: `oci-guest` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-002-004: `oci-guest` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-002-005: `oci-guest` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-002-006: `oci-guest` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-002-007: `oci-guest` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-002-008: `oci-guest` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-003-001: `oci-guest/always-free` wrapper stance for `sites`: No invocation declared yet; future use requires manifest amendment and a signed module pin.
CTX-003-002: `oci-guest/always-free` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-003-003: `oci-guest/always-free` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-003-004: `oci-guest/always-free` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-003-005: `oci-guest/always-free` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-003-006: `oci-guest/always-free` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-003-007: `oci-guest/always-free` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-003-008: `oci-guest/always-free` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-004-001: `on-prem` wrapper stance for `sites`: No invocation declared yet; future use requires manifest amendment and a signed module pin.
CTX-004-002: `on-prem` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-004-003: `on-prem` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-004-004: `on-prem` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-004-005: `on-prem` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-004-006: `on-prem` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-004-007: `on-prem` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-004-008: `on-prem` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-005-001: `colo` wrapper stance for `sites`: No invocation declared yet; future use requires manifest amendment and a signed module pin.
CTX-005-002: `colo` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-005-003: `colo` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-005-004: `colo` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-005-005: `colo` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-005-006: `colo` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-005-007: `colo` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-005-008: `colo` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-006-001: `oyatie-as-cloud-provider` wrapper stance for `sites`: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1[both], oyatie-as-cloud-provider/secrets-bootstrap@v1[both], oyatie-as-cloud-provider/dns@v1[both], oyatie-as-cloud-provider/vpc@v1[both]
CTX-006-002: `oyatie-as-cloud-provider` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-006-003: `oyatie-as-cloud-provider` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-006-004: `oyatie-as-cloud-provider` wrapper must pass `Tier-3` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-006-005: `oyatie-as-cloud-provider` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-006-006: `oyatie-as-cloud-provider` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-006-007: `oyatie-as-cloud-provider` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-006-008: `oyatie-as-cloud-provider` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.

## 6. Version Pinning, Signing, And Blast Radius
PIN-001: `sites` treats `version_pin` as a production contract, not a convenience string.
PIN-002: Major-version movement for `sites` requires an explicit wrapper review because input variables, outputs, and blast-radius assumptions can change.
PIN-003: Minor-version movement for `sites` is allowed during the quarterly module upgrade window when catalog release notes prove backward compatibility.
PIN-004: Patch-version movement for `sites` can occur for CVE, provider, or correctness repair when cosign evidence and validation pass.
PIN-005: The sunset path for a primitive replacement is: add successor module, dual-run wrapper plan, emit audit-chain evidence, update manifest pin, then remove the old invocation after one successful quarter.
PIN-006: `sites` never consumes `main`, a local unversioned path, or a registry path without an ADR-0181 signature chain.
PIN-007: Cosign attestation must cover module source digest, provider lockfile digest, catalog entry digest, and release tag.
PIN-008: Blast-radius review for `sites` starts at primitive granularity: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1[both], oyatie-as-cloud-provider/secrets-bootstrap@v1[both], oyatie-as-cloud-provider/dns@v1[both], oyatie-as-cloud-provider/vpc@v1[both].
PIN-009: If `sites` needs a non-catalog primitive, the change starts in cloud-iac with a catalog addition IP before any service wrapper uses it.
PIN-010: Supply-chain evidence is stored outside the service wrapper but referenced by `cosign_attestation_digest` when releases are signed.
PIN-011: ADR-0181 means module signing is a release prerequisite, not an after-the-fact audit note.
PIN-012: Wrapper review includes provider lock drift, state backend drift, tenant-class validation, and compliance-pack matrix impact.

## 7. Hyperscaler Precedents Cited
PRECEDENT-001: AWS Solutions Constructs precedent: reusable constructs encode common VPC, IAM, KMS, queue, and storage wiring behind typed inputs; `sites` draws that design choice by expressing only primitive selection and tenant parameters.
PRECEDENT-002: Google Cloud Foundation Toolkit precedent: foundation modules centralize network, IAM, logging, and project primitives with opinionated guardrails; `sites` draws that design choice by relying on cloud-iac modules for provider-specific guardrails.
PRECEDENT-003: Azure Verified Modules precedent: resource modules publish consistent interfaces, examples, and versioned releases; `sites` draws that design choice by pinning module versions and requiring catalog-backed input/output contracts.
PRECEDENT-004: AWS cellular architecture precedent: services isolate blast radius by cell and shuffle-shard; `sites` draws that design choice through `Tier-3` placement and per-cell wrapper variables.
PRECEDENT-005: Microsoft secure supply-chain precedent: signed build artifacts and repeatable pipelines are treated as deploy prerequisites; `sites` draws that design choice by tying OpenTofu module release to ADR-0181 cosign evidence.
PRECEDENT-006: Stripe API-change discipline precedent: versioned public contracts prevent silent tenant breakage; `sites` draws that design choice by treating module pins as service contracts with sunset windows.

## 8. Twenty-Four Month Maintainability Outlook
MAINT-24M-001: Month 01: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-002: Month 02: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-003: Month 03: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-004: Month 04: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-005: Month 05: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-006: Month 06: `sites` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-007: Month 07: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-008: Month 08: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-009: Month 09: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-010: Month 10: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-011: Month 11: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-012: Month 12: `sites` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-013: Month 13: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-014: Month 14: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-015: Month 15: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-016: Month 16: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-017: Month 17: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-018: Month 18: `sites` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-019: Month 19: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-020: Month 20: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-021: Month 21: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-022: Month 22: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-023: Month 23: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-024: Month 24: `sites` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.

## 9. Horizontal Scalability Path
SCALE-001: Scaling dimension for `sites` is `per_request`, so 10x means ten times that unit before the wrapper should ask for larger primitives.
SCALE-002: At 10x, `sites` keeps one cell family when p99, queue depth, and storage fill stay below ADR-0340 thresholds.
SCALE-003: At 100x, `sites` expects multiple cells in the same residency boundary, with tenant placement resolved by ADR-0248 and oya-shuffle-sharding.
SCALE-004: At 1000x, `sites` expects regional cell families, per-cell module pins, and explicit compliance-pack overlays to avoid one global blast radius.
SCALE-005: CPU limit dimension: baseline `0.1` vCPU per tenant becomes 1.00 at 10 tenants, 10.00 at 100, and 100.00 at 1000.
SCALE-006: RAM limit dimension: baseline `256` MiB per tenant becomes 2560 MiB at 10 tenants, 25600 MiB at 100, and 256000 MiB at 1000.
SCALE-007: Storage limit dimension: baseline `10.0` GiB per tenant becomes 100.00 GiB at 10 tenants, 1000.00 GiB at 100, and 10000.00 GiB at 1000.
SCALE-008: Connection count limit dimension: valkey=2, postgres=2, outbound_http=6 per tenant; wrapper modules must size pools from these facts.
SCALE-009: Cell placement strategy for `sites` is `Tier-3`; promotion or demotion follows ADR-0341 gate evidence rather than manual placement.
SCALE-010: Per-cell sharding strategy uses autosharding `control_plane_driven`, auto_rebalance enabled=false, dynamic_sharding enabled=false.
SCALE-011: Hot-split threshold p99 is `50` ms and utilization threshold is `80` percent.
SCALE-012: Cold-merge threshold is `20` percent after `24` quiet hours.
SCALE-013: At scale tier 1, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-014: At scale tier 2, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-015: At scale tier 3, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-016: At scale tier 4, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-017: At scale tier 5, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-018: At scale tier 6, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-019: At scale tier 7, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-020: At scale tier 8, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-021: At scale tier 9, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-022: At scale tier 10, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-023: At scale tier 11, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-024: At scale tier 12, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-025: At scale tier 13, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-026: At scale tier 14, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-027: At scale tier 15, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-028: At scale tier 16, `sites` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-029: At scale tier 17, `sites` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-030: At scale tier 18, `sites` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.

## 10. Five-Year Cost, Carbon, And Watt-Hour Outlook
COST-001: `sites` uses a planning proxy, not a billing quote: 7 W per allocated vCPU, 0.35 W per GiB RAM, 0.03 W per GiB durable storage, 0.35 kgCO2e/kWh, and 0.12 USD/kWh plus 0.023 USD/GiB-month storage.
COST-002: 10x planning envelope: 1.00 vCPU, 2.50 GiB RAM, 100.00 GiB storage, 10.88 W steady proxy, 7.94 kWh/month, 2.78 kgCO2e/month at 0.35 kg/kWh, 3.25 USD/month proxy before managed-service premiums.
COST-003: 100x planning envelope: 10.00 vCPU, 25.00 GiB RAM, 1000.00 GiB storage, 108.75 W steady proxy, 79.39 kWh/month, 27.79 kgCO2e/month at 0.35 kg/kWh, 32.53 USD/month proxy before managed-service premiums.
COST-004: 1000x planning envelope: 100.00 vCPU, 250.00 GiB RAM, 10000.00 GiB storage, 1087.50 W steady proxy, 793.88 kWh/month, 277.86 kgCO2e/month at 0.35 kg/kWh, 325.26 USD/month proxy before managed-service premiums.
COST-005: Five-year invariant for `sites`: cost labels carry tenant_id, cell_id, primitive, context, and version_pin so FinOps can attribute drift to the exact module release.
COST-006: Five-year invariant for `sites`: carbon accounting follows ADR-0344 and never hides provider-specific electricity mix behind a service-local average.
COST-007: Five-year invariant for `sites`: paid tenants can buy larger cells; demo_trial tenants remain bounded by OCI Always Free or equivalent cap modules.
COST-008: Five-year change path for `sites`: if a primitive becomes less efficient, cloud-iac ships the replacement module and `sites` re-pins through the sunset path.
COST-009: Five-year control for `sites`: wrapper variables include workload class and compliance-pack labels so high-regulation cells are costed separately from generic cells.
COST-010: Five-year risk for `sites`: storage growth of `10.0` GiB per tenant can dominate compute if retention is not tied to regulatory pack and DR policy.
COST-011: Five-year mitigation for `sites`: object and database primitives must expose retention, compaction, lifecycle, and snapshot knobs in the shared module input contract.
COST-012: Five-year operator signal for `sites`: module pin lag, module cost delta, module carbon delta, and wrapper drift count become review-board metrics.
COST-013: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-014: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-015: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-016: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-017: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-018: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-019: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-020: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-021: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-022: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-023: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-024: `sites` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.

## 11. Industry-Leading Comparison And Differentiation
LEADER-001: Comparison anchor: Linear product-surface discipline: fast tenant interactions, narrow contracts, and simple operator recovery paths.
LEADER-002: `sites` differentiates by making infrastructure primitive selection machine-readable in the manifest rather than burying deployment intent in per-service HCL.
LEADER-003: `sites` differentiates by combining ADR-0248 cells, ADR-0244 tenant scoping, ADR-0181 signatures, and ADR-0344 carbon labels in one wrapper contract.
LEADER-004: At leader scale, `sites` should look like a service-owned contract over cloud-iac primitives, not a service-owned infrastructure implementation fork.
LEADER-005: `sites` must preserve public contracts while module pins change; OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 surfaces stay versioned independently from IaC module versions.
LEADER-006: `sites` must surface module-driven deploy risk to operators before apply, matching hyperscaler change-management norms for shared foundations.
LEADER-007: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-008: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-009: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-010: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-011: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-012: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-013: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-014: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-015: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-016: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-017: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-018: `sites` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.

## 12. API And Contract Documentation Impact
API-001: `sites` does not change REST, event, or proto payloads in this document-stage wave.
API-002: OpenAPI 3.2.0 references for `sites` remain: OpenAPI 3.2.0: microservices/sites/contracts/openapi/sites.yaml
API-003: AsyncAPI 3.1.0 references for `sites` remain: AsyncAPI 3.1.0: microservices/sites/contracts/asyncapi/sites-events.yaml
API-004: proto3 references for `sites` remain: proto3: microservices/sites/contracts/proto/sites.proto
API-005: If a future wrapper migration exposes deployment preview APIs, the public boundary must carry ADR-0342 date-version carriers separately from module semantic versions.
API-006: If a future wrapper migration changes async deployment events, the AsyncAPI channel must identify module context, primitive, version_pin, tenant_class_scope, and cell_id.
API-007: If a future wrapper migration changes proto deployment receipts, proto3 reserved tags must prevent silent field reuse.
API-008: Contract docs must explain module pin behavior to SDK consumers when `sites` owns a tenant-facing deployment or admin surface.

## 13. Non-Obvious Gotchas
GOTCHA-001: `sites` wrapper simplicity can hide security complexity; reviewers must inspect cloud-iac primitive release notes and cosign signatures.
GOTCHA-002: `sites` capacity variables can look small per tenant but become large when cell placement concentrates regulated tenants.
GOTCHA-003: `sites` must not treat `tenant_class_scope` as billing-only; it also controls Always Free limits, BYOK availability, compliance packs, and provider quota behavior.
GOTCHA-004: `sites` must not use a generic module pin when a compliance pack requires a stricter backup, retention, encryption, or locality primitive.
GOTCHA-005: `sites` must not let wrapper drift bypass ADR-0341 cell promotion evidence or ADR-0348 sharding automation evidence.
GOTCHA-006: `sites` must not copy module source into the service tree during an incident; emergency changes still flow through signed module release or a documented break-glass audit event.
GOTCHA-007: `sites` must not rely on provider defaults for encryption, tags, network egress, or retention; cloud-iac modules expose explicit inputs for those decisions.
GOTCHA-008: `sites` must not let docs claim hyperscaler maturity until module pins, wrappers, signatures, catalog entries, and validation evidence all exist.

## 14. Alternatives Considered For This Microservice
ALT-001: Keep `sites` from-scratch HCL per context; rejected because the service would own provider plumbing that ADR-0339 assigns to cloud-iac.
ALT-002: Move all `sites` deployment decisions into cloud-iac; rejected because primitive selection, capacity variables, and tenant-facing blast-radius remain service-owned substance.
ALT-003: Use an external registry as the primary module source; rejected because Oyatie needs in-repo provenance, OpenTofu-native compatibility, and ADR-0181 signing.
ALT-004: Delay `sites` manifest declaration until implementation; rejected because doc-stage propagation needs an early reviewable contract for downstream agents.
ALT-005: Allow unpinned local module paths during migration; rejected because the exact path would work locally while hiding supply-chain and reproducibility risk.

## 15. Acceptance And Verification
VERIFY-001: Static read confirms this file exists at `microservices/sites/IPs/IP-ADR-0339-Shared-IaC-Modules.md`.
VERIFY-002: Static read confirms ADR-0339 is cited by exact ID.
VERIFY-003: Static read confirms ADR-0322 is cited by exact ID.
VERIFY-004: Static read confirms ADR-0181 is cited by exact ID.
VERIFY-005: Static read confirms ADR-0248 is cited by exact ID.
VERIFY-006: Static read confirms all ADR-0339 enforced_by lanes are named.
VERIFY-007: Static read confirms `sites` manifest has `iac_module_invocations` present.
VERIFY-008: Static read confirms `sites` PRD has an `ADR-0339 adoption` section.
VERIFY-009: Static read confirms `sites` ARCH has an `ADR-0339 integration` section.
VERIFY-010: Static read confirms no Rust source or crate metadata is changed by this wave.
VERIFY-011: Static read confirms no OpenTofu module body is authored in this service path.
VERIFY-012: Static read confirms the IP has at least 300 lines of service-specific content.
VERIFY-013: `cargo run -q -p oya-dev-cli -- gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions` must pass before commit.
VERIFY-014: `cargo run -q -p oya-dev-cli -- gate validate cohesion` must pass before commit.
VERIFY-015: `cargo run -q -p oya-dev-cli -- doc inventory --write` must refresh machine-readable inventory before commit.
ACCEPT-016: `sites` accepts doc-stage ADR-0339 propagation only after the verification commands pass or blockers are explicitly reported.
ACCEPT-017: `sites` implementation remains future work under a separate wrapper migration change and is not implied complete by this PROPOSED IP.
ACCEPT-018: `sites` module pins remain service-owned review inputs and cloud-iac module releases remain cloud-iac-owned implementation artifacts.
ACCEPT-019: `sites` reviewers can validate lifecycle, doctrine, scale path, 24-month maintainability, five-year economics, and supply-chain stance from this single IP.
