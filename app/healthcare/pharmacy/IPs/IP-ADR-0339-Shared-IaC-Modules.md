---
doc_class: Implementation-Plan
doc_id: IP-ADR-0339-Shared-IaC-Modules
microservice: pharmacy
status: PROPOSED
date: 2026-05-21
owner_team: axis-pharmacy
bounded_context: pharmacy
implementation_phase: documentation-and-contracts-only
rust_code_status: not-authored-in-this-wave
source_adrs: ADR-0339, ADR-0322, ADR-0181, ADR-0248, ADR-0215, ADR-0218, ADR-0244, ADR-0251, ADR-0338, ADR-0340, ADR-0343, ADR-0344
lifecycle_rule: PROPOSED until the microservice wrappers invoke signed shared OpenTofu modules; ACCEPTED only after implementation evidence lands
---
# IP-ADR-0339-Shared-IaC-Modules: Pharmacy Shared OpenTofu Module Adoption

## 1. Lifecycle, Boundary, And Stop Condition
SCOPE-001: This IP binds `pharmacy` to ADR-0339 shared IaC module doctrine without authoring Rust, changing crates, or applying infrastructure.
SCOPE-002: Lifecycle state is PROPOSED for `pharmacy` until the service-owned wrapper files under `microservices/pharmacy/iac/<context>/main.tf` invoke signed cloud-iac modules and implementation evidence is reviewed.
SCOPE-003: ACCEPTED status requires a later service implementation change, not this document-stage propagation.
SCOPE-004: The only implementation authority created here is documentation intent plus manifest `iac_module_invocations` alignment for `pharmacy`.
SCOPE-005: The stop condition for this IP is a reviewable doctrine packet: IP present, manifest field populated, PRD adoption section appended, ARCH integration section appended, and ADR citations validated.
SCOPE-006: ADR-0339 keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `pharmacy` keeps only thin invocation wrappers and service-specific parameters.
SCOPE-007: The wrapper body for `pharmacy` must stay at or below 80 logical lines per context after comments and blank lines are removed.
SCOPE-008: No `resource` block belongs in a `pharmacy` per-context wrapper after migration; resource bodies belong to cloud-iac shared modules.
SCOPE-009: OpenTofu remains the IaC engine per ADR-0218; HashiCorp Terraform SaaS registry coupling is outside this design.
SCOPE-010: Tenant scoping parameters `tenant_id`, `tenant_class`, `cell_id`, and compliance-pack labels are first-class wrapper inputs for `pharmacy`.
SCOPE-011: This IP cites ADR-0339 purpose and enforced_by lanes so downstream reviewers can validate the same doctrine without rereading the full ADR.
SCOPE-012: This IP avoids module-body authoring; Wave 15Q owns shared module bodies and catalog details under cloud-iac.

## 2. Service-Specific Dossier
DOSSIER-001: Microservice `pharmacy` is classified here as `healthcare regulated workflow`.
DOSSIER-002: Owner team is `axis-pharmacy`; wrapper ownership stays with this owner while reusable primitive ownership stays with axis-cloud-iac.
DOSSIER-003: Manifest version is `0.1.0` and schema version is `1.0`.
DOSSIER-004: Capacity scaling dimension is `per_workflow_run`; wrapper sizing must not infer a different primary load axis.
DOSSIER-005: ADR-0248 cell placement class is `Tier-2`; `pharmacy` wrappers pass this as placement intent rather than open-coding nodepool choices.
DOSSIER-006: Baseline per-tenant CPU is `0.55` vCPU, RAM is `1280` MiB, and storage is `20.0` GiB.
DOSSIER-007: Declared connection budget per tenant is valkey=8, postgres=8, outbound_http=12.
DOSSIER-008: Capacity notes: Medication ordering, EPCS, BCMA, inventory, DSCSA, and reimbursement paths scale by dispense and verification workflow runs; Tier-2 is required because pod_runtime_tier=1 cannot co-vary with Tier-3.
DOSSIER-009: DR target is RTO p99 `900` seconds and RPO p99 `60` seconds.
DOSSIER-010: DR replication shape is `active-active-multi-az-cross-region-warm` with backup substrates `postgres_wal_g, valkey_cluster, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal`.
DOSSIER-011: Regulatory packs declared: kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa.
DOSSIER-012: Data classes processed: manifest-not-populated.
DOSSIER-BC-001: Bounded context `medication-catalog` states: NDC + RxNorm + GPI + ATC + UNII canonical identity; FDB/Multum/Medi-Span ingestion; A/B knowledge package switching.; crate count=12.
DOSSIER-BC-002: Bounded context `formulary` states: Per-tenant + per-cell formulary; P&T workflow; therapeutic interchange; prior-auth criteria.; crate count=9.
DOSSIER-BC-003: Bounded context `eprescribe` states: Surescripts + NCPDP SCRIPT 2017-071+ orchestration; EPCS signing for Schedule II–V.; crate count=11.
DOSSIER-BC-004: Bounded context `drug-interaction` states: DDI/DAI/DCI/DPI/DDxI/DLI/DFI/DDoseI evaluation; severity stratification; tenant suppression with audit.; crate count=7.
DOSSIER-BC-005: Bounded context `allergy-check` states: Patient allergy mirror from emr; exact + cross-class; severity-aware override capture.; crate count=8.
DOSSIER-BC-006: Bounded context `dose-check` states: Weight/BSA/renal(eGFR+CrCl)/hepatic(Child-Pugh)/age-band/cumulative caps.; crate count=7.
DOSSIER-CRATE-001: Existing crate `oya-pharmacy-medication-catalog-adapter` remains untouched by this document-stage IP.
DOSSIER-CRATE-002: Existing crate `oya-pharmacy-medication-catalog-adapter-fdb` remains untouched by this document-stage IP.
DOSSIER-CRATE-003: Existing crate `oya-pharmacy-medication-catalog-adapter-multum` remains untouched by this document-stage IP.
DOSSIER-CRATE-004: Existing crate `oya-pharmacy-medication-catalog-adapter-medi-span` remains untouched by this document-stage IP.
DOSSIER-CRATE-005: Existing crate `oya-pharmacy-medication-catalog-api` remains untouched by this document-stage IP.
DOSSIER-CRATE-006: Existing crate `oya-pharmacy-medication-catalog-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-007: Existing crate `oya-pharmacy-medication-catalog-domain` remains untouched by this document-stage IP.
DOSSIER-CRATE-008: Existing crate `oya-pharmacy-medication-catalog-kernel` remains untouched by this document-stage IP.
DOSSIER-CRATE-009: Existing crate `oya-pharmacy-medication-catalog-rest` remains untouched by this document-stage IP.
DOSSIER-CRATE-010: Existing crate `oya-pharmacy-medication-catalog-sdk` remains untouched by this document-stage IP.
DOSSIER-CRATE-011: Existing crate `oya-pharmacy-medication-catalog-usecase` remains untouched by this document-stage IP.
DOSSIER-CRATE-012: Existing crate `oya-pharmacy-medication-catalog-worker` remains untouched by this document-stage IP.
DOSSIER-CRATE-013: Existing crate `oya-pharmacy-formulary-adapter` remains untouched by this document-stage IP.
DOSSIER-CRATE-014: Existing crate `oya-pharmacy-formulary-api` remains untouched by this document-stage IP.
DOSSIER-CRATE-015: Existing crate `oya-pharmacy-formulary-app` remains untouched by this document-stage IP.
DOSSIER-CRATE-016: Existing crate `oya-pharmacy-formulary-domain` remains untouched by this document-stage IP.
DOSSIER-CRATE-017: Existing crate `oya-pharmacy-formulary-kernel` remains untouched by this document-stage IP.
DOSSIER-CRATE-018: Existing crate `oya-pharmacy-formulary-rest` remains untouched by this document-stage IP.
DOSSIER-CRATE-019: Existing crate `oya-pharmacy-formulary-sdk` remains untouched by this document-stage IP.
DOSSIER-CRATE-020: Existing crate `oya-pharmacy-formulary-usecase` remains untouched by this document-stage IP.
DOSSIER-CONTRACT-001: OpenAPI 3.2.0: microservices/pharmacy/contracts/openapi/pharmacy.yaml.
DOSSIER-CONTRACT-002: AsyncAPI 3.1.0: microservices/pharmacy/contracts/asyncapi/pharmacy-events.yaml.
DOSSIER-CONTRACT-003: proto3: microservices/pharmacy/contracts/proto/pharmacy.proto.
DOSSIER-CAPABILITY-001: T0 medication-catalog-read risk=none file=manifest-only.
DOSSIER-CAPABILITY-002: T0 formulary-read risk=none file=manifest-only.
DOSSIER-CAPABILITY-003: T1 drug-interaction-evaluate risk=limited file=manifest-only.
DOSSIER-CAPABILITY-004: T1 dose-check-evaluate risk=limited file=manifest-only.
DOSSIER-CAPABILITY-005: T1 bcma-verify risk=limited file=manifest-only.
DOSSIER-CAPABILITY-006: T2 order-queue-route risk=limited file=manifest-only.
DOSSIER-CAPABILITY-007: T2 refill-triage risk=limited file=manifest-only.
DOSSIER-CAPABILITY-008: T3 therapeutic-interchange risk=high file=manifest-only.
DOSSIER-CAPABILITY-009: T3 b340-mixed-use-classify risk=high file=manifest-only.
DOSSIER-CAPABILITY-010: T3 pbm-automated-resubmit risk=high file=manifest-only.

## 3. ADR-0339 Doctrine Binding
ADR0339-001: Purpose binding: collapse 385 per-service from-scratch module directories into roughly 50 shared OpenTofu primitives plus thin wrappers.
ADR0339-002: Purpose binding: reusable module bodies live under `microservices/cloud-iac/modules/<context>/<primitive>/` with catalog and signature evidence.
ADR0339-003: Purpose binding: `pharmacy` owns primitive selection, tenant-class scope, sizing parameters, and service-specific blast-radius analysis.
ADR0339-004: Purpose binding: cloud-iac owns primitive implementation, provider constraints, input/output contracts, catalog entries, and module release signatures.
ADR0339-005: Purpose binding: wrapper files must remain OpenTofu-native and avoid provider-proprietary Terraform Cloud behavior.
ADR0339-006: Purpose binding: every consumed module pin must be explicit, versioned, and reviewable.
ADR0339-007: Purpose binding: every shared module release must be signed under ADR-0181 supply-chain discipline.
ADR0339-LANE-001: Enforced_by lane `oya-check-iac-shared-module-usage` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-002: Enforced_by lane `oya-check-iac-module-path-canonical` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-003: Enforced_by lane `oya-check-iac-module-signature-cosign` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-004: Enforced_by lane `oya-check-iac-module-pin` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-005: Enforced_by lane `oya-check-iac-opentofu-only` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-006: Enforced_by lane `oya-check-iac-thin-wrapper-line-floor` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-LANE-007: Enforced_by lane `oya-check-iac-module-catalog-discoverability` applies to `pharmacy` once its migration bucket enters blocker mode.
ADR0339-015: ADR-0322 substance bar applies to this IP; the content below is service-specific and intentionally connects module doctrine to manifest facts.
ADR0339-016: ADR-0248 cellular topology applies because wrapper choices determine where this service lands by cell and tenant class.
ADR0339-017: ADR-0338 pod runtime tier applies because shared modules choose Kata or runc nodepool topology from manifest tier data.
ADR0339-018: ADR-0340 capacity data applies because module sizing must use declared per-tenant CPU, RAM, storage, and connection budgets.
ADR0339-019: ADR-0343 DR data applies because backup primitives and failover topology must satisfy service RTO/RPO floors.
ADR0339-020: ADR-0344 sustainability and FinOps apply because every module pin changes watts, carbon, and monthly cost envelopes.

## 4. Manifest Module Invocation Plan
MODULE-000: `pharmacy` declares 10 shared module invocation(s) in `manifest.json#iac_module_invocations`.
MODULE-001: Contexts represented: aws-guest, colo, oci-guest, oci-guest/always-free, on-prem, oyatie-as-cloud-provider.
MODULE-002: Primitive names represented: openbao-bindings, per-cell-nodepool-kata, postgres-wal-g, tenant-namespace, valkey-cluster.
MODULE-003: `aws-guest/tenant-namespace@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-003A: `tenant-namespace` in context `aws-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-004: `aws-guest/postgres-wal-g@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-004A: `postgres-wal-g` in context `aws-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-005: `aws-guest/valkey-cluster@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-005A: `valkey-cluster` in context `aws-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-006: `aws-guest/openbao-bindings@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-006A: `openbao-bindings` in context `aws-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-007: `oci-guest/tenant-namespace@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-007A: `tenant-namespace` in context `oci-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-008: `oci-guest/postgres-wal-g@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-008A: `postgres-wal-g` in context `oci-guest` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-009: `oci-guest/always-free/tenant-namespace@v1[demo_trial]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-009A: `tenant-namespace` in context `oci-guest/always-free` must receive tenant_id, tenant_class_scope `demo_trial`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-010: `on-prem/tenant-namespace@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-010A: `tenant-namespace` in context `on-prem` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-011: `colo/tenant-namespace@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-011A: `tenant-namespace` in context `colo` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.
MODULE-012: `oyatie-as-cloud-provider/per-cell-nodepool-kata@v1[both]` is selected for `pharmacy`; blast radius is limited to that primitive release plus the `pharmacy` wrapper variables that feed it.
MODULE-012A: `per-cell-nodepool-kata` in context `oyatie-as-cloud-provider` must receive tenant_id, tenant_class_scope `both`, cell placement `Tier-2`, and compliance-pack labels from the wrapper.

## 5. Per-Context Thin Wrapper Specifications
CTX-001-001: `aws-guest` wrapper stance for `pharmacy`: aws-guest/tenant-namespace@v1[both], aws-guest/postgres-wal-g@v1[both], aws-guest/valkey-cluster@v1[both], aws-guest/openbao-bindings@v1[both]
CTX-001-002: `aws-guest` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-001-003: `aws-guest` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-001-004: `aws-guest` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-001-005: `aws-guest` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-001-006: `aws-guest` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-001-007: `aws-guest` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-001-008: `aws-guest` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-002-001: `oci-guest` wrapper stance for `pharmacy`: oci-guest/tenant-namespace@v1[both], oci-guest/postgres-wal-g@v1[both]
CTX-002-002: `oci-guest` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-002-003: `oci-guest` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-002-004: `oci-guest` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-002-005: `oci-guest` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-002-006: `oci-guest` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-002-007: `oci-guest` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-002-008: `oci-guest` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-003-001: `oci-guest/always-free` wrapper stance for `pharmacy`: oci-guest/always-free/tenant-namespace@v1[demo_trial]
CTX-003-002: `oci-guest/always-free` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-003-003: `oci-guest/always-free` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-003-004: `oci-guest/always-free` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-003-005: `oci-guest/always-free` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-003-006: `oci-guest/always-free` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-003-007: `oci-guest/always-free` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-003-008: `oci-guest/always-free` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-004-001: `on-prem` wrapper stance for `pharmacy`: on-prem/tenant-namespace@v1[both]
CTX-004-002: `on-prem` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-004-003: `on-prem` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-004-004: `on-prem` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-004-005: `on-prem` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-004-006: `on-prem` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-004-007: `on-prem` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-004-008: `on-prem` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-005-001: `colo` wrapper stance for `pharmacy`: colo/tenant-namespace@v1[both]
CTX-005-002: `colo` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-005-003: `colo` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-005-004: `colo` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-005-005: `colo` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-005-006: `colo` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-005-007: `colo` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-005-008: `colo` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.
CTX-006-001: `oyatie-as-cloud-provider` wrapper stance for `pharmacy`: oyatie-as-cloud-provider/per-cell-nodepool-kata@v1[both]
CTX-006-002: `oyatie-as-cloud-provider` wrapper must expose only module blocks, variables, outputs, provider constraints, and backend state references.
CTX-006-003: `oyatie-as-cloud-provider` wrapper must pass tenant_class and compliance pack explicitly so demo_trial and paid tenants cannot share accidental defaults.
CTX-006-004: `oyatie-as-cloud-provider` wrapper must pass `Tier-2` placement intent and must not locally choose node labels outside ADR-0248 and ADR-0338.
CTX-006-005: `oyatie-as-cloud-provider` wrapper must pin every source with `?ref=v<major>.<minor>.<patch>` or the accepted major-only pin during the Wave 15Q transition.
CTX-006-006: `oyatie-as-cloud-provider` wrapper must include a cosign attestation digest once module releases carry ADR-0181 signatures.
CTX-006-007: `oyatie-as-cloud-provider` wrapper must keep state backend references tenant-scoped and cell-scoped to prevent cross-tenant plan leakage.
CTX-006-008: `oyatie-as-cloud-provider` wrapper must be reviewed as service substance, not as cloud-iac primitive implementation.

## 6. Version Pinning, Signing, And Blast Radius
PIN-001: `pharmacy` treats `version_pin` as a production contract, not a convenience string.
PIN-002: Major-version movement for `pharmacy` requires an explicit wrapper review because input variables, outputs, and blast-radius assumptions can change.
PIN-003: Minor-version movement for `pharmacy` is allowed during the quarterly module upgrade window when catalog release notes prove backward compatibility.
PIN-004: Patch-version movement for `pharmacy` can occur for CVE, provider, or correctness repair when cosign evidence and validation pass.
PIN-005: The sunset path for a primitive replacement is: add successor module, dual-run wrapper plan, emit audit-chain evidence, update manifest pin, then remove the old invocation after one successful quarter.
PIN-006: `pharmacy` never consumes `main`, a local unversioned path, or a registry path without an ADR-0181 signature chain.
PIN-007: Cosign attestation must cover module source digest, provider lockfile digest, catalog entry digest, and release tag.
PIN-008: Blast-radius review for `pharmacy` starts at primitive granularity: aws-guest/tenant-namespace@v1[both], aws-guest/postgres-wal-g@v1[both], aws-guest/valkey-cluster@v1[both], aws-guest/openbao-bindings@v1[both], oci-guest/tenant-namespace@v1[both], oci-guest/postgres-wal-g@v1[both], oci-guest/always-free/tenant-namespace@v1[demo_trial], on-prem/tenant-namespace@v1[both].
PIN-009: If `pharmacy` needs a non-catalog primitive, the change starts in cloud-iac with a catalog addition IP before any service wrapper uses it.
PIN-010: Supply-chain evidence is stored outside the service wrapper but referenced by `cosign_attestation_digest` when releases are signed.
PIN-011: ADR-0181 means module signing is a release prerequisite, not an after-the-fact audit note.
PIN-012: Wrapper review includes provider lock drift, state backend drift, tenant-class validation, and compliance-pack matrix impact.

## 7. Hyperscaler Precedents Cited
PRECEDENT-001: AWS Solutions Constructs precedent: reusable constructs encode common VPC, IAM, KMS, queue, and storage wiring behind typed inputs; `pharmacy` draws that design choice by expressing only primitive selection and tenant parameters.
PRECEDENT-002: Google Cloud Foundation Toolkit precedent: foundation modules centralize network, IAM, logging, and project primitives with opinionated guardrails; `pharmacy` draws that design choice by relying on cloud-iac modules for provider-specific guardrails.
PRECEDENT-003: Azure Verified Modules precedent: resource modules publish consistent interfaces, examples, and versioned releases; `pharmacy` draws that design choice by pinning module versions and requiring catalog-backed input/output contracts.
PRECEDENT-004: AWS cellular architecture precedent: services isolate blast radius by cell and shuffle-shard; `pharmacy` draws that design choice through `Tier-2` placement and per-cell wrapper variables.
PRECEDENT-005: Microsoft secure supply-chain precedent: signed build artifacts and repeatable pipelines are treated as deploy prerequisites; `pharmacy` draws that design choice by tying OpenTofu module release to ADR-0181 cosign evidence.
PRECEDENT-006: Stripe API-change discipline precedent: versioned public contracts prevent silent tenant breakage; `pharmacy` draws that design choice by treating module pins as service contracts with sunset windows.

## 8. Twenty-Four Month Maintainability Outlook
MAINT-24M-001: Month 01: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-002: Month 02: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-003: Month 03: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-004: Month 04: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-005: Month 05: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-006: Month 06: `pharmacy` remains PROPOSED while Wave 15Q module releases stabilize; wrapper work is limited to selecting signed primitives and validating no inline resources.
MAINT-24M-007: Month 07: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-008: Month 08: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-009: Month 09: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-010: Month 10: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-011: Month 11: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-012: Month 12: `pharmacy` moves high-churn primitives through quarterly upgrade windows; invariants are tenant scoping, explicit pins, signed modules, and ≤80-line wrappers.
MAINT-24M-013: Month 13: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-014: Month 14: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-015: Month 15: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-016: Month 16: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-017: Month 17: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-018: Month 18: `pharmacy` can absorb provider changes by re-pinning cloud-iac primitives while keeping service-owned capacity and compliance variables stable.
MAINT-24M-019: Month 19: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-020: Month 20: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-021: Month 21: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-022: Month 22: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-023: Month 23: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.
MAINT-24M-024: Month 24: `pharmacy` should have wrapper drift near zero; primitive replacement follows dual-run, audit evidence, manifest pin update, and one-quarter sunset path.

## 9. Horizontal Scalability Path
SCALE-001: Scaling dimension for `pharmacy` is `per_workflow_run`, so 10x means ten times that unit before the wrapper should ask for larger primitives.
SCALE-002: At 10x, `pharmacy` keeps one cell family when p99, queue depth, and storage fill stay below ADR-0340 thresholds.
SCALE-003: At 100x, `pharmacy` expects multiple cells in the same residency boundary, with tenant placement resolved by ADR-0248 and oya-shuffle-sharding.
SCALE-004: At 1000x, `pharmacy` expects regional cell families, per-cell module pins, and explicit compliance-pack overlays to avoid one global blast radius.
SCALE-005: CPU limit dimension: baseline `0.55` vCPU per tenant becomes 5.50 at 10 tenants, 55.00 at 100, and 550.00 at 1000.
SCALE-006: RAM limit dimension: baseline `1280` MiB per tenant becomes 12800 MiB at 10 tenants, 128000 MiB at 100, and 1280000 MiB at 1000.
SCALE-007: Storage limit dimension: baseline `20.0` GiB per tenant becomes 200.00 GiB at 10 tenants, 2000.00 GiB at 100, and 20000.00 GiB at 1000.
SCALE-008: Connection count limit dimension: valkey=8, postgres=8, outbound_http=12 per tenant; wrapper modules must size pools from these facts.
SCALE-009: Cell placement strategy for `pharmacy` is `Tier-2`; promotion or demotion follows ADR-0341 gate evidence rather than manual placement.
SCALE-010: Per-cell sharding strategy uses autosharding `control_plane_driven`, auto_rebalance enabled=false, dynamic_sharding enabled=false.
SCALE-011: Hot-split threshold p99 is `50` ms and utilization threshold is `80` percent.
SCALE-012: Cold-merge threshold is `20` percent after `24` quiet hours.
SCALE-013: At scale tier 1, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-014: At scale tier 2, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-015: At scale tier 3, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-016: At scale tier 4, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-017: At scale tier 5, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-018: At scale tier 6, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-019: At scale tier 7, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-020: At scale tier 8, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-021: At scale tier 9, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-022: At scale tier 10, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-023: At scale tier 11, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-024: At scale tier 12, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-025: At scale tier 13, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-026: At scale tier 14, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-027: At scale tier 15, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.
SCALE-028: At scale tier 16, `pharmacy` wrapper growth must add cells before adding bespoke module code, preserving the ADR-0339 O(1) primitive maintenance model.
SCALE-029: At scale tier 17, `pharmacy` service SLOs decide whether to add replicas, storage shards, or separate tenant cohorts; wrappers only pass the chosen variables.
SCALE-030: At scale tier 18, `pharmacy` rejects cross-cell shared mutable state unless ADR-0248 and ADR-0244 evidence proves tenant isolation remains intact.

## 10. Five-Year Cost, Carbon, And Watt-Hour Outlook
COST-001: `pharmacy` uses a planning proxy, not a billing quote: 7 W per allocated vCPU, 0.35 W per GiB RAM, 0.03 W per GiB durable storage, 0.35 kgCO2e/kWh, and 0.12 USD/kWh plus 0.023 USD/GiB-month storage.
COST-002: 10x planning envelope: 5.50 vCPU, 12.50 GiB RAM, 200.00 GiB storage, 48.88 W steady proxy, 35.68 kWh/month, 12.49 kgCO2e/month at 0.35 kg/kWh, 8.88 USD/month proxy before managed-service premiums.
COST-003: 100x planning envelope: 55.00 vCPU, 125.00 GiB RAM, 2000.00 GiB storage, 488.75 W steady proxy, 356.79 kWh/month, 124.88 kgCO2e/month at 0.35 kg/kWh, 88.81 USD/month proxy before managed-service premiums.
COST-004: 1000x planning envelope: 550.00 vCPU, 1250.00 GiB RAM, 20000.00 GiB storage, 4887.50 W steady proxy, 3567.88 kWh/month, 1248.76 kgCO2e/month at 0.35 kg/kWh, 888.14 USD/month proxy before managed-service premiums.
COST-005: Five-year invariant for `pharmacy`: cost labels carry tenant_id, cell_id, primitive, context, and version_pin so FinOps can attribute drift to the exact module release.
COST-006: Five-year invariant for `pharmacy`: carbon accounting follows ADR-0344 and never hides provider-specific electricity mix behind a service-local average.
COST-007: Five-year invariant for `pharmacy`: paid tenants can buy larger cells; demo_trial tenants remain bounded by OCI Always Free or equivalent cap modules.
COST-008: Five-year change path for `pharmacy`: if a primitive becomes less efficient, cloud-iac ships the replacement module and `pharmacy` re-pins through the sunset path.
COST-009: Five-year control for `pharmacy`: wrapper variables include workload class and compliance-pack labels so high-regulation cells are costed separately from generic cells.
COST-010: Five-year risk for `pharmacy`: storage growth of `20.0` GiB per tenant can dominate compute if retention is not tied to regulatory pack and DR policy.
COST-011: Five-year mitigation for `pharmacy`: object and database primitives must expose retention, compaction, lifecycle, and snapshot knobs in the shared module input contract.
COST-012: Five-year operator signal for `pharmacy`: module pin lag, module cost delta, module carbon delta, and wrapper drift count become review-board metrics.
COST-013: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-014: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-015: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-016: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-017: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-018: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-019: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-020: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-021: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-022: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-023: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.
COST-024: `pharmacy` planning horizon keeps USD, watt-hours, and CO2e visible at primitive granularity so a cheap local wrapper cannot hide an expensive shared-module release.

## 11. Industry-Leading Comparison And Differentiation
LEADER-001: Comparison anchor: Epic/Cerner regulated health-workflow discipline: PHI boundaries, auditability, and site-local continuity before feature speed.
LEADER-002: `pharmacy` differentiates by making infrastructure primitive selection machine-readable in the manifest rather than burying deployment intent in per-service HCL.
LEADER-003: `pharmacy` differentiates by combining ADR-0248 cells, ADR-0244 tenant scoping, ADR-0181 signatures, and ADR-0344 carbon labels in one wrapper contract.
LEADER-004: At leader scale, `pharmacy` should look like a service-owned contract over cloud-iac primitives, not a service-owned infrastructure implementation fork.
LEADER-005: `pharmacy` must preserve public contracts while module pins change; OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 surfaces stay versioned independently from IaC module versions.
LEADER-006: `pharmacy` must surface module-driven deploy risk to operators before apply, matching hyperscaler change-management norms for shared foundations.
LEADER-007: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-008: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-009: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-010: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-011: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-012: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-013: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-014: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-015: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-016: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-017: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.
LEADER-018: `pharmacy` leader-scale posture keeps primitive selection explicit, reviewable, and reversible while avoiding service-local provider logic.

## 12. API And Contract Documentation Impact
API-001: `pharmacy` does not change REST, event, or proto payloads in this document-stage wave.
API-002: OpenAPI 3.2.0 references for `pharmacy` remain: OpenAPI 3.2.0: microservices/pharmacy/contracts/openapi/pharmacy.yaml
API-003: AsyncAPI 3.1.0 references for `pharmacy` remain: AsyncAPI 3.1.0: microservices/pharmacy/contracts/asyncapi/pharmacy-events.yaml
API-004: proto3 references for `pharmacy` remain: proto3: microservices/pharmacy/contracts/proto/pharmacy.proto
API-005: If a future wrapper migration exposes deployment preview APIs, the public boundary must carry ADR-0342 date-version carriers separately from module semantic versions.
API-006: If a future wrapper migration changes async deployment events, the AsyncAPI channel must identify module context, primitive, version_pin, tenant_class_scope, and cell_id.
API-007: If a future wrapper migration changes proto deployment receipts, proto3 reserved tags must prevent silent field reuse.
API-008: Contract docs must explain module pin behavior to SDK consumers when `pharmacy` owns a tenant-facing deployment or admin surface.

## 13. Non-Obvious Gotchas
GOTCHA-001: `pharmacy` wrapper simplicity can hide security complexity; reviewers must inspect cloud-iac primitive release notes and cosign signatures.
GOTCHA-002: `pharmacy` capacity variables can look small per tenant but become large when cell placement concentrates regulated tenants.
GOTCHA-003: `pharmacy` must not treat `tenant_class_scope` as billing-only; it also controls Always Free limits, BYOK availability, compliance packs, and provider quota behavior.
GOTCHA-004: `pharmacy` must not use a generic module pin when a compliance pack requires a stricter backup, retention, encryption, or locality primitive.
GOTCHA-005: `pharmacy` must not let wrapper drift bypass ADR-0341 cell promotion evidence or ADR-0348 sharding automation evidence.
GOTCHA-006: `pharmacy` must not copy module source into the service tree during an incident; emergency changes still flow through signed module release or a documented break-glass audit event.
GOTCHA-007: `pharmacy` must not rely on provider defaults for encryption, tags, network egress, or retention; cloud-iac modules expose explicit inputs for those decisions.
GOTCHA-008: `pharmacy` must not let docs claim hyperscaler maturity until module pins, wrappers, signatures, catalog entries, and validation evidence all exist.

## 14. Alternatives Considered For This Microservice
ALT-001: Keep `pharmacy` from-scratch HCL per context; rejected because the service would own provider plumbing that ADR-0339 assigns to cloud-iac.
ALT-002: Move all `pharmacy` deployment decisions into cloud-iac; rejected because primitive selection, capacity variables, and tenant-facing blast-radius remain service-owned substance.
ALT-003: Use an external registry as the primary module source; rejected because Oyatie needs in-repo provenance, OpenTofu-native compatibility, and ADR-0181 signing.
ALT-004: Delay `pharmacy` manifest declaration until implementation; rejected because doc-stage propagation needs an early reviewable contract for downstream agents.
ALT-005: Allow unpinned local module paths during migration; rejected because the exact path would work locally while hiding supply-chain and reproducibility risk.

## 15. Acceptance And Verification
VERIFY-001: Static read confirms this file exists at `microservices/pharmacy/IPs/IP-ADR-0339-Shared-IaC-Modules.md`.
VERIFY-002: Static read confirms ADR-0339 is cited by exact ID.
VERIFY-003: Static read confirms ADR-0322 is cited by exact ID.
VERIFY-004: Static read confirms ADR-0181 is cited by exact ID.
VERIFY-005: Static read confirms ADR-0248 is cited by exact ID.
VERIFY-006: Static read confirms all ADR-0339 enforced_by lanes are named.
VERIFY-007: Static read confirms `pharmacy` manifest has `iac_module_invocations` present.
VERIFY-008: Static read confirms `pharmacy` PRD has an `ADR-0339 adoption` section.
VERIFY-009: Static read confirms `pharmacy` ARCH has an `ADR-0339 integration` section.
VERIFY-010: Static read confirms no Rust source or crate metadata is changed by this wave.
VERIFY-011: Static read confirms no OpenTofu module body is authored in this service path.
VERIFY-012: Static read confirms the IP has at least 300 lines of service-specific content.
VERIFY-013: `cargo run -q -p oya-dev-cli -- gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions` must pass before commit.
VERIFY-014: `cargo run -q -p oya-dev-cli -- gate validate cohesion` must pass before commit.
VERIFY-015: `cargo run -q -p oya-dev-cli -- doc inventory --write` must refresh machine-readable inventory before commit.
ACCEPT-016: `pharmacy` accepts doc-stage ADR-0339 propagation only after the verification commands pass or blockers are explicitly reported.
ACCEPT-017: `pharmacy` implementation remains future work under a separate wrapper migration change and is not implied complete by this PROPOSED IP.
ACCEPT-018: `pharmacy` module pins remain service-owned review inputs and cloud-iac module releases remain cloud-iac-owned implementation artifacts.
ACCEPT-019: `pharmacy` reviewers can validate lifecycle, doctrine, scale path, 24-month maintainability, five-year economics, and supply-chain stance from this single IP.
