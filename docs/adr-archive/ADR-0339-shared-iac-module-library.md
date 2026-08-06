---
id: ADR-0339
title: Shared IaC module library (`cloud/cloud-iac/modules/<context>/<primitive>/` is canonical; per-µservice `iac/<context>/main.tf` is a thin wrapper)
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-platform-iac
  - ops-sre-reliability
  - council-security
owners:
  - council-architecture
  - ops-platform-iac
  - ops-sre-reliability
  - council-security
supersedes: []
superseded_by: [ADR-709]
amends:
  - ADR-0211-in-house-tech-stack-preference.md (substrate-class allow-list: OpenTofu shared-module library is the canonical IaC reuse vehicle; per-µservice from-scratch IaC modules are non-canonical going forward)
  - ADR-0212-buildability-doctrine.md (per-µservice manifest `iac_module_invocations` field becomes the canonical declaration of the µservice's IaC primitive dependencies after the library lands; from-scratch module bodies become artifacts only inside cloud-iac, not per-µservice)
  - ADR-0215-multi-context-platform.md (the 5 deployment-contexts each gain a canonical module-library subdir; contexts no longer fork per-µservice)
  - ADR-0216-deployment-context-iac-layout.md (per-µservice `iac/<context>/main.tf` is a thin module-invocation wrapper, not a self-contained module body)
  - ADR-0218-opentofu-not-terraform.md (shared modules are OpenTofu only; HCL written for Terraform Cloud Run Tasks or other Terraform-Inc-proprietary surfaces are excluded)
  - ADR-0244-tenant-as-universal-scoping-primitive.md (every shared module accepts `tenant_id` + `tenant_class` per ADR-0331 §D-8 pattern; demo_trial / paid validation lives in the module, not duplicated per-µservice)
  - ADR-0248-amazon-shape-cellular-architecture.md (oyatie-as-cloud-provider modules in the library include `cell-zone`, `shard-cell`, and `tenant-namespace` primitives that materialize the cellular topology)
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md (on-prem and colo modules in the library encode kubeadm + Cilium + Istio-ambient + Envoy-gateway + Kata + Cloud-Hypervisor as canonical OSS substrate)
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (Wave 15Q-IaC-modules added as a sub-wave that authors the ~50 shared primitives; per-µservice migrations follow in their own canonical-sequence phase order)
  - ADR-0336-valkey-not-redis-substrate.md (the IaC modules at `modules/<context>/{valkey,cache-valkey,oci-cache-valkey,valkey-cluster}/` are the canonical authoring target for the Wave 15-Valkey iac-module-path surface per D-2 of that ADR)
related:
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0216-deployment-context-iac-layout.md
  - ADR-0218-opentofu-not-terraform.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md
  - ADR-0255-byok-everywhere-credentials.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-3.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/cloud-iac.json
  - /specs/microservices/manifest-schema.json
  - /specs/decision-principles.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_idea_refine_decisions_2026_05_21
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_bominal_inheritance_precedence
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_drift_too_big_2026_05_20
companion_docs:
  - docs/standards/dependency-policy.md
  - docs/standards/iac-module-catalog.md
  - cloud/cloud-iac/ARCHITECTURE.md
  - cloud/cloud-iac/manifest.json
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_idea_refine_decisions_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-module-library-lands
enforced_by:
  - oya-check-iac-shared-module-usage (new lane; advisory until crate lands; planned promotion to BLOCKER after Wave 15Q-IaC-modules lands)
  - oya-check-iac-module-path-canonical (new lane; advisory until crate lands; planned to refuse from-scratch per-µservice iac/<context>/ bodies that do not invoke a shared module)
  - oya-check-iac-module-signature-cosign (new lane; advisory until crate lands; planned to refuse module sources that lack ADR-0181 cosign attestation)
  - oya-check-iac-module-pin (new lane; advisory until crate lands; planned to refuse unpinned shared-module references; module source MUST include `?ref=v<MAJOR>.<MINOR>.<PATCH>` or equivalent)
  - oya-check-iac-opentofu-only (advisory until crate lands; planned to refuse HashiCorp Terraform syntax; preserved verbatim from `feedback_zero_handroll_opentofu_only_2026_05_20`)
  - oya-check-iac-thin-wrapper-line-floor (new lane; advisory until crate lands; planned per-µservice `iac/<context>/main.tf` must be ≤ 80 LOC excluding comments; substance is in the wrapper's primitive selection, not its plumbing)
  - oya-check-iac-module-catalog-discoverability (new lane; advisory until crate lands; planned to refuse additions of new primitives to the library without a corresponding entry in `docs/standards/iac-module-catalog.md`)
purpose: >
  Establish `cloud/cloud-iac/modules/<context>/<primitive>/` as the
  canonical home for reusable, signed, OpenTofu module primitives across
  Oyatie's five deployment-contexts (aws-guest, oci-guest [+ always-free
  sub-context], on-prem, colo, oyatie-as-cloud-provider). Establish that
  per-µservice `iac/<context>/main.tf` files are thin wrappers that invoke
  shared modules with tenant-scoped parameters, NOT from-scratch module
  bodies. Collapse the prior 77-µservice × 5-context = 385 from-scratch
  IaC module-dir blast-radius into ~50 shared-library primitives plus 385
  thin wrapper invocations. Specify the canonical primitive enumeration
  per context, the OpenTofu signed-module distribution path (ADR-0181
  cosign), the per-µservice migration path (factor reusable IaC into the
  library; keep µservice-specific IaC under µservice ownership with a
  declared `extends cloud-iac/modules/<context>/<base>` cross-reference),
  the module-versioning quarterly-upgrade-window pattern (matching
  network-substrate-versions discipline), the module catalog + discovery
  + documentation contract under `docs/standards/iac-module-catalog.md`,
  the OCI Always Free `oci-guest/always-free/` sub-context category for
  demo_trial-only modules (per `feedback_oci_always_free_maximization_2026_05_20`
  and ADR-0331 §D-8.2), and the seven new CI lanes that enforce the
  library boundary. Do NOT author the ~50 module bodies in this ADR;
  that authoring is sequenced as Wave 15Q-IaC-modules after this ADR is
  Accepted. Do NOT migrate any per-µservice iac/ directory in this ADR;
  per-µservice migration is sequenced per-µservice under ADR-0328
  canonical-build phase order.
---

> **Disposition light-edit (2026-08-06):** Shared IaC modules: path microservices/cloud-iac → cloud/cloud-iac

# ADR-0339: Shared IaC module library (`cloud/cloud-iac/modules/<context>/<primitive>/` is canonical; per-µservice `iac/<context>/main.tf` is a thin wrapper)

## Status

Proposed on 2026-05-21.

This ADR is the canonical IaC-substrate-shape decision establishing the shared OpenTofu module library at `cloud/cloud-iac/modules/<context>/<primitive>/` as the home for every reusable per-context infrastructure primitive Oyatie ships, and establishing the per-µservice `iac/<context>/main.tf` file as a thin invocation wrapper that selects primitives from the library and supplies tenant-scoped parameters.

IAC-001 materializes the first fixture slice under the de-branded `iac/modules/` capability home (the repo-current capability-first home for this ADR's `<context>/<primitive>/` library) while preserving this ADR's shared-module/thin-wrapper shape and non-runtime claim ceiling. The wrapper references the shared module by in-repo relative path (no Git ref pin) and the fixture asserts no cosign/SLSA attestation, since no signing generator exists to keep such digests fresh. The fixture-owned surfaces are `cloud/cloud-billing/iac/aws-guest/OWNERS`, `cloud/cloud-billing/iac/aws-guest/main.tofu`, `iac/modules/OWNERS`, `iac/modules/catalog.json`, `iac/modules/aws-guest/sg-baseline/README.md`, `iac/modules/aws-guest/sg-baseline/main.tofu`, `iac/modules/aws-guest/sg-baseline/outputs.tofu`, `iac/modules/aws-guest/sg-baseline/variables.tofu`, and `iac/modules/aws-guest/sg-baseline/versions.tofu`.

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0337 (Iceberg canonical OLAP) and ADR-0338 (Pod runtime tier 0..3) are the sibling decisions from the same `/idea-refine` session captured in `feedback_idea_refine_decisions_2026_05_21.md`; this ADR is the third in that triplet.

It directly amends ADR-0216 (deployment-context iac layout) so that the per-µservice context body is no longer a from-scratch module but a thin wrapper. It directly amends ADR-0212 (buildability doctrine) so that per-µservice manifest declarations describe **which library primitives are invoked**, not the inline body of HCL each µservice ships. It is binding on every µservice that declares an `iac/` directory.

Enforcement transitions from `advisory-until-module-library-lands` to `BLOCKER` per the lane sequence in §E below: at landing of Wave 15Q-IaC-modules (which authors the ~50 module bodies), the `oya-check-iac-module-path-canonical` lane promotes to BLOCKER for new authoring; per-µservice migrations of existing from-scratch bodies follow the µservice's canonical-build phase order under ADR-0328.

The decision does not delete any existing per-µservice IaC content. The decision does not change the OpenTofu-only constraint from `feedback_zero_handroll_opentofu_only_2026_05_20`. The decision does not change the OCI Always Free maximization rule from `feedback_oci_always_free_maximization_2026_05_20`. The decision does not change the multi-context provider-agnostic rule from `feedback_multi_context_provider_agnostic_2026_05_20`. The decision does not change which µservice owns which capability; it changes only the shape of the IaC authoring.

## Date

2026-05-21.

## Context

### A.1 Named pressure: 385-module-dir blast-radius today

Oyatie has 77 active µservices (47 baseline + 9 ERP + 13 B2B-leader + the in-flight 8 healthcare/marketing splits captured by the realignment effort). Each µservice ships under five canonical deployment-contexts per ADR-0215:

- `iac/aws-guest/` — AWS-hosted with Oyatie as the SaaS operator
- `iac/oci-guest/` — OCI-hosted with Oyatie as the SaaS operator (with a sub-context `always-free/` for demo_trial tenants per `feedback_oci_always_free_maximization_2026_05_20`)
- `iac/on-prem/` — customer-controlled bare-metal or VM cluster
- `iac/colo/` — colocated customer cluster (alias of on-prem topology)
- `iac/oyatie-as-cloud-provider/` — Oyatie running its own cellular topology per ADR-0248

That is 77 × 5 = 385 distinct `iac/<context>/` directories the corpus is heading toward if every µservice ships its own from-scratch HCL body.

Roughly 60-70% of the HCL content across those 385 directories is structurally identical from one µservice to the next. The shape "`aws_eks_cluster` + `aws_iam_role` + `aws_iam_role_policy_attachment` + `aws_security_group` + `aws_vpc` + `aws_subnet`" repeats across every µservice deployed to `aws-guest`. The shape "`oci_containerengine_cluster` + `oci_identity_dynamic_group` + `oci_core_vcn` + `oci_core_subnet`" repeats across every µservice deployed to `oci-guest`. The shape "`kubernetes_manifest` for kubeadm + Cilium + Istio-ambient + Envoy-gateway + Helm release per pod-runtime-tier per ADR-0338" repeats across every µservice deployed to `on-prem` and `colo`. The shape "`oya_cell_zone` + `oya_shard_cell` + `oya_tenant_namespace` + per-cell node pools" repeats across every µservice deployed to `oyatie-as-cloud-provider`.

The remaining ~30-40% per-µservice unique content is **which primitives the µservice depends on** (does it need a database, a cache, a queue, a stream, a vector store, an object-storage bucket, a KMS key, a TLS-certificate, a load-balancer terminating which routes, etc.) and **what tenant-scoped parameters are passed** (how large is the database; what cap shape applies for demo_trial; which compliance pack the cell carries; which BYOK credentials are wired in per ADR-0255).

Duplicating the 60-70% structural shape across 385 module dirs would create five concurrent regressions:

- **Drift risk.** Every change to AWS provider behavior, OCI provider behavior, kubeadm topology, or Cell topology requires editing 77 copies of the same HCL. The probability of two copies diverging accidentally during a quarterly upgrade window approaches certainty as the µservice count grows.
- **Duplication mass.** ~60-70% of 385 × (~500 LOC per HCL body) ≈ ~120,000 LOC of duplicated HCL across the corpus. That mass is repository bloat, review burden, and a constant authoring cost for every new µservice.
- **Inconsistent security posture.** A copy-paste of a `security-group-baseline` rule that diverges across µservices erodes the SOC2 / ISO27001 baseline. The shared-library shape lets every µservice inherit a single signed baseline.
- **OpenTofu module-cache miss.** OpenTofu reads modules from a registry-style cache by module-source URL. Distinct per-µservice paths defeat the cache; shared module sources hit cached entries.
- **Substance-bar regression.** Per ADR-0322 substance-bar doctrine, every per-µservice surface should carry **bespoke per-µservice substance**. If 60-70% of a µservice's IaC body is structural copy-paste, the µservice owner spends authoring effort on plumbing instead of on µservice-specific substance (which primitives, what cap shape, which tenant_class variants, what compliance pack).

### A.2 Named pressure: ADR-0331 §D-8 already specifies six contexts per µservice

ADR-0331 §D-8 (the cross-µservice tenant_class adoption template) requires every µservice's `iac/` directory to contain six deployment-context sub-directories. The six are: `oyatie-public-cloud/`, `aws-guest/`, `oci-guest/` (with `always-free/` and `paid/` sub-variants), `on-prem/`, `colo/`, and `oyatie-cloud-provider/`. The ADR-0331 §D-8 enumeration is six contexts; this ADR enumerates five (collapsing `oyatie-public-cloud/` and `oyatie-cloud-provider/` into the single `oyatie-as-cloud-provider/` context for the purposes of module-library taxonomy, because in both cases Oyatie operates the cellular control plane and the IaC primitive set is identical). For consistency this ADR aligns on the five-context taxonomy (per `feedback_multi_context_provider_agnostic_2026_05_20`) plus the `oci-guest/always-free/` sub-context for demo_trial tenants. ADR-0331 §D-8.1 is amended at landing time to use the five-context taxonomy; the substance of D-8.2..D-8.8 is preserved verbatim.

The point of the named pressure is: ADR-0331 already required every µservice to ship six (now five + Always-Free sub) IaC variant directories. If each variant directory ships from-scratch HCL, the 385-module-dir blast-radius is locked in by ADR-0331 itself. This ADR closes that loop by making 385 thin wrappers viable instead of 385 from-scratch bodies.

### A.3 Named pressure: substantive ops cost of N from-scratch modules

Per `feedback_quality_performance_scalability_bar`, Oyatie operates at hyperscaler-grade rigor. Hyperscaler IaC at scale uses **shared modules with signed releases and pinned versions** — AWS internally publishes "AWS Solutions Constructs" as shared CDK / CFN modules with versioned, signed releases; Google Cloud publishes "Cloud Foundation Toolkit" Terraform modules; Microsoft publishes "Azure Verified Modules" (AVM) terraform / bicep modules. The hyperscaler precedent is to operate a shared module library, not to have each consumer service ship from-scratch IaC.

The cost of N from-scratch modules at scale is well-named:

- Every quarterly upgrade window touches N modules instead of 1 + N thin wrappers.
- Every security patch (Cilium CVE, Envoy CVE, etcd CVE, Cloud Hypervisor CVE) requires N modules to consume the patch instead of 1.
- Every new feature (compliance pack, BYOK rotation, mTLS posture upgrade) requires N modules to learn the feature instead of 1.
- Every cargo-vet / cosign / SLSA audit examines N artifacts instead of ~50.

The shared library inverts this: one canonical primitive per context per concern, signed, versioned, consumed by N thin wrappers. The ops cost moves from O(N) per change to O(1) per change in the library + O(N) only for the wrappers that need new parameters.

### A.4 Named pressure: zero-handroll OpenTofu-only and signed-module discipline

Per `feedback_zero_handroll_opentofu_only_2026_05_20`, every Oyatie µservice deployment in every deployment-context lands via OpenTofu (not HashiCorp Terraform per ADR-0218) with no manual provisioning steps. Per ADR-0181 cosign discipline, every reusable artifact (containers, charts, OpenTofu modules) carries a cosign attestation signed by the canonical Oyatie root key.

The shared-library shape is the cleanest match to these constraints:

- A single OpenTofu module per primitive per context can be released, signed, and version-pinned once.
- A per-µservice from-scratch HCL body cannot be signed as a single artifact because it carries µservice-specific values inline; only its source can be signed at the µservice-repo level.
- Consumers of shared modules pin a version + verify cosign signature at the source pin: `source = "git::https://github.com/oyatie/oyatie//cloud/cloud-iac/modules/aws-guest/eks-cluster?ref=v1.2.3"` plus a `provider_meta` block referencing the cosign attestation.

### A.5 Named pressure: ADR-0336 Valkey migration just landed iac/<context>/valkey/ as a new path

ADR-0336 (Valkey not Redis substrate) D-2 requires every µservice that uses the in-memory KV substrate to ship `iac/<context>/valkey/` modules. Today, with no shared library, each µservice would author its own Valkey OpenTofu body per context.

Under this ADR, the canonical Valkey primitives live at:

- `cloud/cloud-iac/modules/aws-guest/elasticache-valkey/`
- `cloud/cloud-iac/modules/oci-guest/oci-cache-valkey/`
- `cloud/cloud-iac/modules/oci-guest/always-free/valkey-free/` (within Always-Free perpetual ceiling)
- `cloud/cloud-iac/modules/on-prem/valkey-cluster/`
- `cloud/cloud-iac/modules/colo/valkey-cluster/` (alias of on-prem)
- `cloud/cloud-iac/modules/oyatie-as-cloud-provider/valkey-cell-cluster/`

Per-µservice `iac/<context>/main.tf` invokes the canonical Valkey primitive per context with the µservice's tenant-scoped parameters (cluster size, TLS posture, AOF/RDB flavor, eviction policy, BYOK posture, observability label set). The thin wrapper for Valkey at any µservice is ~30 LOC of module invocation rather than ~300 LOC of from-scratch Valkey HCL.

The ADR-0336 D-2 surface lands cleanly on top of this ADR: instead of refusing `iac/*/redis/` paths in 77 from-scratch bodies, the `oya-check-iac-module-path-canonical` lane (new in §E below) refuses 77 from-scratch bodies altogether and routes them through the canonical Valkey library primitives.

### A.6 Named pressure: counterpart precedent at the hyperscalers

- **AWS Solutions Constructs.** AWS publishes a library of CDK / CFN constructs (vended at github.com/aws/aws-solutions-constructs) that internal services and external partners consume as versioned, signed releases. Internal AWS services do not author from-scratch CFN for an EKS cluster; they consume `aws-eks-fargate-cluster` construct with parameters.
- **Google Cloud Foundation Toolkit.** Google Cloud publishes terraform modules at github.com/GoogleCloudPlatform/cloud-foundation-toolkit covering project-factory, log-export, KMS, network, IAM. Internal Google teams and external customers consume the modules with version pins.
- **Azure Verified Modules (AVM).** Microsoft publishes AVM terraform and bicep modules at aka.ms/avm covering compute, storage, networking, observability. AVM is the canonical reuse path for Azure customers; from-scratch terraform is discouraged.
- **HashiCorp Terraform Registry.** The HashiCorp Terraform Registry is the canonical pattern of "shared modules registry" — published, versioned, source-pinned, public-or-private registries. (Note: ADR-0218 forbids HashiCorp Terraform itself; OpenTofu reads the same module-source URL grammar and can consume the same registry-style sources, but Oyatie hosts its own registry-style shape inside `cloud-iac/modules/` rather than depending on a SaaS registry.)
- **CNCF projects.** The Helm Charts repository, the kustomize-builtin transformers, and the Cluster API providers all instantiate the same "shared module library with version pins" pattern.

Every hyperscaler-grade and CNCF precedent operates a shared module library with version pins. Oyatie is in the wrong shape if it ships 385 from-scratch IaC dirs across 77 µservices.

### A.7 Anchors this ADR binds

- Anchor 1: the user directive of 2026-05-21 captured in `feedback_idea_refine_decisions_2026_05_21.md` Decision 3 — "shared IaC module library; per-context canonical reusable IaC primitives live at `cloud/cloud-iac/modules/<context>/<primitive>/`; per-µservice `iac/<context>/main.tf` is a thin invocation".
- Anchor 2: ADR-0215 (multi-context platform). The shared library serves all five contexts.
- Anchor 3: ADR-0216 (deployment-context IaC layout). This ADR amends ADR-0216 so that the per-µservice context body is a thin wrapper, not a self-contained body.
- Anchor 4: ADR-0218 (OpenTofu not Terraform). Shared modules are OpenTofu modules; HashiCorp Terraform syntax is forbidden everywhere including the shared library.
- Anchor 5: ADR-0244 (tenant scoping universal primitive). Every shared module accepts `tenant_id` + `tenant_class` parameters.
- Anchor 6: ADR-0248 (Amazon-shape cellular architecture). The `oyatie-as-cloud-provider/` module category materializes the cellular topology (cell-zone, shard-cell, tenant-namespace, per-cell node pools).
- Anchor 7: ADR-0254 (Kubernetes everywhere + Cloud Hypervisor + Kata). The `on-prem/` and `colo/` modules encode the kubeadm + Cilium + Istio-ambient + Envoy-gateway + Kata + Cloud Hypervisor stack as canonical OSS substrate.
- Anchor 8: ADR-0336 (Valkey not Redis substrate). The Valkey IaC primitives live in this library.
- Anchor 9: ADR-0181 (cosign-signed artifacts and modules). Every module in the library carries a cosign attestation.
- Anchor 10: ADR-0331 §D-8 (per-context iac/<context>/ tenant_class-aware module variants). The thin-wrapper pattern absorbs ADR-0331 §D-8 plumbing.
- Anchor 11: `feedback_zero_handroll_opentofu_only_2026_05_20`. The library is OpenTofu-only with no manual provisioning steps.
- Anchor 12: `feedback_oci_always_free_maximization_2026_05_20`. The `oci-guest/always-free/` sub-context is its own category in the library.

### A.8 What this ADR does not assert

- **A.8.1** Does not author the ~50 module bodies. That authoring is sequenced as Wave 15Q-IaC-modules under ADR-0328 batch discipline after this ADR is Accepted.
- **A.8.2** Does not migrate any existing per-µservice `iac/` directory. Per-µservice migration follows the canonical-build phase order under ADR-0328; each µservice files its own migration IP under `microservices/<name>/IPs/IP-iac-module-library-migration.md`.
- **A.8.3** Does not delete any existing per-µservice IaC content. Existing bodies remain compilable until each µservice's migration bucket lands.
- **A.8.4** Does not change which µservice owns which capability. The ownership shape is unchanged; only the shape of the IaC authoring changes.
- **A.8.5** Does not change the OpenTofu-only constraint. HashiCorp Terraform remains forbidden everywhere, including the shared library.
- **A.8.6** Does not change the cosign attestation pattern. The cosign attestation moves to the library primitive; per-µservice wrappers still cite the attestation pin.
- **A.8.7** Does not introduce a SaaS registry dependency. The library lives in-tree under `cloud/cloud-iac/modules/` and is consumed via Git-tag-pinned module sources, not a HashiCorp Cloud / Terraform Registry SaaS.
- **A.8.8** Does not retire any existing CI lane. New lanes are added (§E); existing lanes are preserved.
- **A.8.9** Does not relax the substance-bar for per-µservice IaC. Per ADR-0322, the per-µservice IaC wrapper substance is **which primitives are invoked + which tenant-scoped parameters are passed**, not lines of plumbing.
- **A.8.10** Does not assert "one true module per primitive" — for primitives that exist across multiple contexts (e.g., Valkey across aws-guest + oci-guest + on-prem + colo + oyatie-as-cloud-provider), each context has its own module body because the underlying provider differs (AWS provider, OCI provider, kubernetes provider, oya-cell provider). The "shared" axis is **across µservices within a context**, not across contexts within a primitive.

## Decision

### B.1 Decision statement

The canonical home for Oyatie reusable OpenTofu IaC primitives is `cloud/cloud-iac/modules/<context>/<primitive>/` where `<context>` is one of `{aws-guest, oci-guest, oci-guest/always-free, on-prem, colo, oyatie-as-cloud-provider}` and `<primitive>` is the canonical primitive name per §D-4 below. Every Oyatie µservice that declares an `iac/` directory ships **thin invocation wrappers** at `microservices/<name>/iac/<context>/main.tf` that consume the canonical primitives from the shared library, supplying tenant-scoped parameters per the µservice's `manifest.json#tenant_class_iac_variants` declaration and per ADR-0331 §D-8 cap-shape contract.

Per-µservice from-scratch HCL bodies are non-canonical going forward. Existing from-scratch bodies remain compilable until each µservice's migration bucket lands under ADR-0328 canonical-build phase order; new authoring after this ADR is Accepted MUST use the shared-library shape.

The shared library lives in-tree at `cloud/cloud-iac/modules/` and is governed by the cloud-iac µservice owner (axis-cloud-iac). Module releases are versioned with semantic-version tags scoped to the module path (e.g., `aws-guest/eks-cluster/v1.2.3`) and signed with cosign per ADR-0181. Module consumers pin a specific version per release-train cadence.

The library starts with ~50 canonical primitives enumerated in §D-4 below; primitive additions follow the discoverability + catalog contract in §D-8 below.

### B.2 Numbered decision clauses

B2.001. `cloud/cloud-iac/modules/<context>/<primitive>/` is the canonical path for Oyatie reusable OpenTofu IaC module primitives.

B2.002. Per-µservice `microservices/<name>/iac/<context>/main.tf` is the canonical thin-wrapper invocation site.

B2.003. The five canonical contexts are `aws-guest`, `oci-guest`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`. The sub-context `oci-guest/always-free` is reserved for demo_trial-tenant primitives within the OCI Always Free perpetual ceiling.

B2.004. ADR-0331 §D-8.1 six-context enumeration is amended to align with this ADR's five-context taxonomy at landing time; the `oyatie-public-cloud/` and `oyatie-cloud-provider/` contexts collapse into `oyatie-as-cloud-provider/` because the IaC primitive set is identical when Oyatie operates the control plane.

B2.005. Each context's module library is versioned per-primitive with semantic versioning. Tags are scoped to the path: `cloud-iac/modules/aws-guest/eks-cluster/v1.2.3` is the canonical release identifier.

B2.006. Module releases carry a cosign attestation per ADR-0181 signed by the canonical Oyatie root key.

B2.007. Module consumers (per-µservice wrappers) pin a specific version. Unpinned `?ref=main` consumption is forbidden by the `oya-check-iac-module-pin` lane (§E).

B2.008. The per-µservice wrapper at `iac/<context>/main.tf` is OpenTofu-only. HashiCorp Terraform syntax remains forbidden per ADR-0218 and the existing `oya-check-iac-opentofu-only` lane.

B2.009. The per-µservice wrapper at `iac/<context>/main.tf` MUST be ≤ 80 LOC excluding comments and blank lines. The line ceiling is planned to be enforced by `oya-check-iac-thin-wrapper-line-floor` (advisory until the crate lands per §E). Substance is in primitive selection + parameter passing, not in plumbing.

B2.010. Per-µservice wrappers MUST declare `tenant_id` + `tenant_class` parameters per ADR-0244 + ADR-0331 §D-8. The shared modules validate these parameters internally; per-µservice wrappers do not duplicate the validation.

B2.011. Per-µservice wrappers MUST NOT inline provider resource declarations (`resource "aws_eks_cluster"`, `resource "oci_containerengine_cluster"`, etc.). Inline resource declarations are P0 findings per the `oya-check-iac-module-path-canonical` lane (§E).

B2.012. Per-µservice wrappers MAY declare provider configuration (`provider "aws" { region = ... }`) at the wrapper level when the per-µservice deployment needs a specific provider configuration (e.g., a region pin different from the substrate default). Provider configuration is not a resource declaration.

B2.013. The shared library serves all five deployment contexts. Cross-context primitives (e.g., `valkey-cluster` exists as a body for `on-prem`, `colo`, and as `elasticache-valkey` for `aws-guest`, as `oci-cache-valkey` for `oci-guest`, etc.) live in separate per-context module directories; consumers select the per-context body via the wrapper.

B2.014. Module-library additions (new primitives) require a catalog entry in `docs/standards/iac-module-catalog.md` per §D-8 below. The `oya-check-iac-module-catalog-discoverability` lane (§E) refuses additions without catalog entries.

B2.015. Module-library changes (existing-primitive modifications) follow the network-substrate-versions quarterly upgrade window pattern. Breaking changes bump the major version; backward-compatible additions bump the minor; bug fixes bump the patch.

B2.016. Each µservice's `manifest.json` declares a top-level `iac_module_invocations` field listing the shared modules the µservice's wrappers invoke, by `<context>/<primitive>` path. The field is the canonical declaration of IaC primitive dependencies.

B2.017. Migration of existing per-µservice from-scratch IaC bodies follows the canonical-build phase order under ADR-0328. Each µservice files a per-µservice IP at `microservices/<name>/IPs/IP-iac-module-library-migration.md` documenting the modules invoked, the tenant-scoped parameters passed, and the cosign attestation pin per invocation.

B2.018. Existing per-µservice from-scratch IaC bodies remain compilable until each µservice's migration bucket lands. The lanes (§E) are advisory until Wave 15Q-IaC-modules ships the ~50 module bodies; the lanes promote to BLOCKER per-µservice as each µservice's migration bucket lands.

B2.019. The Wave 15Q-IaC-modules sub-wave authors the ~50 module bodies + the catalog page + the seven new lanes' stub implementations + a per-context example wrapper at one anchor µservice (e.g., the cloud-billing µservice as the canonical reference wrapper). The Wave 15Q-IaC-modules sub-wave does NOT migrate other µservices; per-µservice migration is sequenced under ADR-0328.

B2.020. The OCI Always Free sub-context `oci-guest/always-free/` is a first-class category in the module library. It is the canonical home for demo_trial-tenant-only primitives that fit within the OCI Always Free perpetual ceiling (per `feedback_oci_always_free_maximization_2026_05_20`). The `always-free` modules MUST validate `tenant_class == "demo_trial"` per ADR-0331 §D-8.4.

B2.021. The paid sub-variant of `oci-guest` lives at `cloud/cloud-iac/modules/oci-guest/<primitive>/` (no sub-context segment). The paid modules MUST validate `tenant_class == "paid"`.

B2.022. The `colo` context is an alias of `on-prem` for the purposes of the module library. `colo` primitives are symlinks-via-source-pin (`source = "../../on-prem/<primitive>"`) or full-path-copies — the choice is per-primitive based on whether colo-specific divergence exists (e.g., specific network-topology assumptions for a colocated rack).

B2.023. The `oyatie-as-cloud-provider` context modules materialize the cellular topology per ADR-0248. Canonical primitives include `cell-zone`, `shard-cell`, `tenant-namespace`, `per-cell-nodepool-kata`, `per-cell-nodepool-runc`, and the cross-cell observability + audit-chain wiring.

B2.024. Module consumption follows the OpenTofu module source URL grammar. Acceptable source pins:

  - In-tree: `source = "../../../cloud-iac/modules/<context>/<primitive>"` (relative path within the monorepo).
  - Git-tag-pinned for cross-repo or release-train consumption: `source = "git::https://github.com/oyatie/oyatie.git//cloud/cloud-iac/modules/<context>/<primitive>?ref=<context>/<primitive>/v<MAJOR>.<MINOR>.<PATCH>"`.

B2.025. The per-µservice wrapper MUST also reference the cosign attestation pin in a per-module `provider_meta` block or equivalent (the exact mechanism per the cosign + OpenTofu wiring tracked in ADR-0181).

B2.026. Modules MUST NOT directly access OpenBao credentials. Per ADR-0296, credential resolution is delegated to the per-µservice runtime; modules accept already-resolved credentials as input variables.

B2.027. Modules MUST emit observability labels per ADR-0263. Every resource created by a module carries `tenant_id`, `tenant_class`, `cell_id`, `compliance_pack`, and `module_version` labels.

B2.028. Modules MUST integrate with the audit-chain per ADR-0263. State-change events (resource created, resource modified, resource destroyed) emit `cloud-iac.module.<op>` audit events with the module path + version + tenant scope.

B2.029. The library catalog is published as a discoverable index at `docs/standards/iac-module-catalog.md` plus a machine-readable mirror at `cloud/cloud-iac/modules/catalog.json`. The catalog enumerates every primitive, its semantic version, its cosign attestation digest, its input variable contract, and its output contract.

B2.030. The library catalog page is the canonical discovery surface for module consumers. A new µservice owner reads the catalog to select primitives; they do not search the corpus for examples.

B2.031. Library primitives MAY compose other library primitives. Example: `oyatie-as-cloud-provider/shard-cell` may invoke `oyatie-as-cloud-provider/per-cell-nodepool-kata` + `oyatie-as-cloud-provider/per-cell-nodepool-runc` + `oyatie-as-cloud-provider/cell-observability-collector` as nested module references.

B2.032. Library primitive ownership defaults to axis-cloud-iac (per the existing manifest). Primitives that materialize a substrate decision owned by another axis (e.g., `aws-guest/elasticache-valkey` carries an axis-cloud-data substrate decision) cite the substrate-decision owner as a secondary owner in the catalog.

B2.033. Library primitive deprecation follows ADR-0108 sunset discipline. A deprecated primitive is marked in the catalog, retains its source for at least one quarterly cycle, and emits a deprecation warning to consumers via OpenTofu `tflog.Warn` per the canonical pattern.

B2.034. New µservices created after this ADR is Accepted MUST use the shared library from the first authoring step. New µservice scaffolding tooling (per ADR-0212 buildability scaffolder) consults the catalog and emits thin-wrapper scaffolds.

B2.035. Three Rejected Alternatives are recorded in §F below: (i) per-µservice from-scratch (the status-quo target), (ii) a single mega-module library spanning all contexts, (iii) a registry-based dependency on a SaaS registry like the HashiCorp Terraform Registry.

B2.036. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. The review evidence is at `evidence/debate/ADR-0339/` after this ADR lands in a review-track PR.

B2.037. The 30-day sunset window starts on Acceptance. The seven new lanes (§E) promote from REPORT-ONLY to BLOCKER for new authoring at day 30; per-µservice migration of existing bodies is sequenced under ADR-0328 and may extend the per-µservice-BLOCKER promotion until each µservice's migration bucket lands.

B2.038. The ADR is final on Acceptance. No exception clause is provided for any µservice's from-scratch authoring after Wave 15Q-IaC-modules lands the canonical bodies.

B2.039. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

B2.040. The ADR's enforcement and sunset run in coordination with ADR-0336 Wave 15-Valkey. The two waves share the iac-module-path enforcement surface: ADR-0336 lands the Valkey vocabulary; this ADR lands the shared library shape.

## Consequences

### C.1 Positive consequences

- **385 → ~50 + 385 thin wrappers.** The 385-from-scratch-module-dir blast-radius collapses to ~50 canonical primitives + 385 thin invocation wrappers, each ≤ 80 LOC. Aggregate IaC LOC across the corpus drops by ~120,000 LOC (roughly 60-70% of prior content).
- **Drift containment.** A change to AWS provider behavior, OCI provider behavior, kubeadm topology, Cilium release, Istio release, Envoy release, Kata release, Cloud Hypervisor release, or Cell topology touches one library primitive instead of N µservice copies.
- **Signed-module supply chain.** ADR-0181 cosign attestation flows cleanly: each library primitive is signed once; each consumer pins the signed version. cargo-vet / SLSA audit scope shrinks from N artifacts to ~50.
- **Substance-bar restoration.** Per-µservice IaC authoring effort shifts from plumbing to substance: which primitives, which cap shape, which tenant-class variants, which compliance pack, which BYOK posture. ADR-0322 substance-bar discipline is reinforced rather than fought.
- **OpenTofu module-cache hit.** Shared module sources hit OpenTofu's module cache; cold `init` time per µservice drops.
- **Hyperscaler-grade rigor.** Oyatie's IaC posture aligns with AWS Solutions Constructs, Google Cloud Foundation Toolkit, Azure Verified Modules, and the HashiCorp Terraform Registry pattern. `feedback_quality_performance_scalability_bar` is reinforced.
- **OCI Always Free first-class category.** The `oci-guest/always-free/` sub-context gives demo_trial-tenant provisioning a clean canonical home, aligned with `feedback_oci_always_free_maximization_2026_05_20`.
- **Cellular topology canonicalization.** The `oyatie-as-cloud-provider/` modules materialize ADR-0248 cellular topology in one place; per-µservice wrappers select tier and cell-class.
- **Tenant-class plumbing canonicalization.** ADR-0331 §D-8 tenant_class-aware module variants land in one place per primitive; demo_trial vs paid validation lives in the module, not duplicated per-µservice.
- **Library catalog discoverability.** `docs/standards/iac-module-catalog.md` becomes the canonical discovery surface for new µservice owners.
- **Versioning + quarterly upgrade window.** The network-substrate-versions quarterly pattern extends cleanly to IaC modules; the cadence is predictable and observable.

### C.2 Negative consequences

- **Wave 15Q-IaC-modules authoring cost.** The ~50 module bodies must be authored from-scratch in a coordinated wave. The work is ~50 × ~300 LOC ≈ 15,000 LOC of canonical HCL plus the catalog + lane implementations. Estimated 4-6 codex batches under ADR-0328 batch discipline.
- **Per-µservice migration cost.** 77 µservices × 5 contexts = 385 wrappers that need to be authored. Each wrapper is ≤ 80 LOC and bespoke per the µservice's primitive selection + tenant-scoped parameters; aggregate ~30,000 LOC of thin wrapper authoring under per-µservice canonical-build phase order.
- **Coupling between µservices and cloud-iac.** Every µservice now declares a dependency on cloud-iac module versions in its `manifest.json#iac_module_invocations`. A library version pin failure (e.g., a cosign attestation rotation) cascades across every µservice that pins the old digest.
- **Quarterly upgrade window discipline.** Every µservice must consume new module versions on the quarterly cadence; lag risks security-patch backlog accumulation.
- **Library governance overhead.** axis-cloud-iac becomes the gatekeeper for every primitive addition / modification. The throughput of the cloud-iac owner becomes a corpus-wide bottleneck unless the owner's bandwidth is sized for the load.
- **In-tree registry compromise.** The library lives in the monorepo at `cloud/cloud-iac/modules/`; consumers reference modules via relative paths or Git-tag-pinned URLs. There is no SaaS-registry intermediary, which simplifies provenance but means cross-repo consumers (e.g., a future external-tenant repo) must pin to specific Git tags rather than registry-cached versions.

### C.3 Neutral consequences

- **Runtime behavior unchanged.** The shape of the IaC authoring changes; the resulting infrastructure is identical to a from-scratch per-µservice body when the library primitive captures the same shape.
- **OpenTofu-only constraint unchanged.** ADR-0218 + `feedback_zero_handroll_opentofu_only_2026_05_20` are preserved verbatim.
- **Cosign attestation pattern unchanged.** ADR-0181 is preserved verbatim; the attestation pin shifts from per-µservice body to per-module-version, which is a tightening of the supply-chain posture, not a loosening.
- **Provider configuration unchanged.** Provider declarations at the wrapper level are permitted per B2.012.
- **Tenant scoping primitive unchanged.** ADR-0244's `tenant_id` + `tenant_class` are universal inputs.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | ~50 canonical primitives shared across 77 µservices; quarterly upgrade window | catalog + cosign attestations + version pins per consumer |
| Supply chain | cosign attestation on every primitive; per-consumer version pin | cargo-vet / SLSA audit ≤ ~50 artifacts |
| Substance-bar | per-µservice IaC substance is primitive selection + tenant-scoped parameters, not plumbing | thin-wrapper ≤ 80 LOC; substance-bar lane stays green |
| Hyperscaler alignment | AWS Solutions Constructs / GCFT / AVM precedent matched | shared library + version pins + signed releases |
| Performance | OpenTofu module cache hit on shared module sources; faster `init` per µservice | per-µservice `tofu init` p95 drops |
| Resilience | drift containment from N copies to 1 canonical body | quarterly security patch lands in 1 module + cascades to N consumers in one cycle |
| Compliance | per-pack compliance posture lives in one module; per-µservice wrapper attests pack via parameter | per-module compliance-pack attestation files at `cloud/cloud-iac/modules/<context>/<primitive>/compliance/` |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS Solutions Constructs (github.com/aws/aws-solutions-constructs), Google Cloud Foundation Toolkit (github.com/GoogleCloudPlatform/cloud-foundation-toolkit), Azure Verified Modules (aka.ms/avm), HashiCorp Terraform Registry (registry.terraform.io). The shared-library shape is the canonical hyperscaler pattern for IaC reuse at scale.

**Failure-mode tree.** Failure modes:
(1) library primitive bug breaks N consumers → roll back to prior version pin per-consumer in one quarterly cycle;
(2) cosign attestation rotation breaks N consumers → coordinated digest rotation per ADR-0181 + quarterly upgrade window;
(3) library version not pinned → `oya-check-iac-module-pin` lane refuses unpinned consumption (BLOCKER post-soak);
(4) per-µservice wrapper inlines a resource bypassing the library → `oya-check-iac-module-path-canonical` lane refuses (BLOCKER post-soak);
(5) primitive added without catalog entry → `oya-check-iac-module-catalog-discoverability` lane refuses (BLOCKER post-soak);
(6) HashiCorp Terraform syntax slipped into library or wrapper → `oya-check-iac-opentofu-only` lane refuses (existing, already BLOCKER);
(7) thin wrapper exceeds 80 LOC → `oya-check-iac-thin-wrapper-line-floor` lane refuses (BLOCKER post-soak);
(8) library primitive removed without sunset → ADR-0108 sunset discipline blocks the removal.

**Capacity math.** ~50 primitives × ~300 LOC each = ~15,000 LOC for the library + ~385 thin wrappers × ~80 LOC each = ~30,000 LOC for wrappers. Total new authoring under this ADR's downstream waves: ~45,000 LOC over ~6 batches × ~8 codex agents per ADR-0328 batch discipline.

**Observability hooks.** Every module emits `tenant_id`, `tenant_class`, `cell_id`, `compliance_pack`, `module_version` labels on every created resource. Every module emits `cloud-iac.module.<op>` audit events on state-change. Aggregate observability cost increases by the cardinality of the `module_version` label, bounded at ~50 primitives × ~5 version cohorts in flight at any time ≈ 250 distinct values.

**Rollback path.** Per-µservice rollback: consumer reverts its version pin to the previous tag; library content unaffected. Library rollback: maintainer reverts the failed module's tag; consumers pinned to the failed tag auto-re-resolve on next `init`. Aggregate corpus rollback is not provided; rollback is per-µservice or per-module.

**Multi-region awareness.** Each context's modules respect the multi-region topology of the underlying substrate. `aws-guest` modules accept `region` as input; `oci-guest` modules accept `region`; `on-prem` and `colo` modules accept `site-id`; `oyatie-as-cloud-provider` modules accept `cell-id`.

**Sovereign-cell awareness.** Modules accept `compliance_pack` as input and propagate to the underlying resources via the audit-chain emission and the per-resource policy attachment. Sovereign cells (HIPAA, GDPR-strict, CSAP, PCI, IL5) get the same module body with a different `compliance_pack` parameter.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. Major version bumps require a Migration Note in the catalog. Deprecated primitives retain their source for at least one quarterly cycle.

## D. Detailed mechanics — ten adoption surfaces

The shared-library shape touches ten adoption surfaces in cloud-iac itself and in every consumer µservice. Subsections D-1 through D-10 enumerate each surface. Numbering is normative.

### D-1: `cloud/cloud-iac/modules/<context>/<primitive>/` canonical path

D-1.1. The canonical path for every reusable OpenTofu IaC module primitive is `cloud/cloud-iac/modules/<context>/<primitive>/`.

D-1.2. The five canonical `<context>` values are `aws-guest`, `oci-guest`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`. The sub-context `oci-guest/always-free` is reserved for demo_trial-only primitives.

D-1.3. The `<primitive>` segment is the canonical primitive name per §D-4 below. Primitive names are kebab-case lowercase, scoped to the context (e.g., `eks-cluster` is `aws-guest`-specific; the OCI equivalent is `oke-cluster`).

D-1.4. Each `<primitive>` directory contains at minimum: `main.tf` (resource bodies), `variables.tf` (input contract), `outputs.tf` (output contract), `versions.tf` (provider version pins), `README.md` (per-primitive doc + catalog cross-reference), and `compliance/` subdirectory (per-pack attestation files per ADR-0251).

D-1.5. The `oya-check-iac-module-path-canonical` lane (§E) refuses any IaC body declared outside this canonical path (or inside a per-µservice `iac/<context>/main.tf` that inlines `resource` blocks instead of `module` blocks).

### D-2: per-µservice `iac/<context>/main.tf` thin-wrapper shape

D-2.1. Each µservice ships its IaC at `microservices/<name>/iac/<context>/main.tf` (per ADR-0331 §D-8.1 amended to five contexts per B2.004).

D-2.2. The wrapper file declares `module` blocks invoking shared-library primitives.

D-2.3. The wrapper file declares `variables.tf` for per-µservice inputs (typically `tenant_id`, `tenant_class`, optional µservice-specific overrides).

D-2.4. The wrapper file MUST NOT declare `resource` blocks. Inline resources are refused by `oya-check-iac-module-path-canonical`.

D-2.5. The wrapper file MUST be ≤ 80 LOC excluding comments and blank lines. Planned to be enforced by `oya-check-iac-thin-wrapper-line-floor` (advisory until the crate lands).

D-2.6. Reference example for a µservice's `iac/oci-guest/main.tf`:

```hcl
# microservices/cloud-billing/iac/oci-guest/main.tf
# Thin wrapper per ADR-0339 §D-2. Invokes shared-library primitives.
# tenant_class enforcement per ADR-0331 §D-8.6 (paid only on this context).

variable "tenant_id"    { type = string }
variable "tenant_class" {
  type = string
  validation {
    condition     = var.tenant_class == "paid"
    error_message = "oci-guest (paid) MUST be applied for tenant_class = paid; demo_trial uses oci-guest/always-free."
  }
}

module "billing_db" {
  source = "../../../cloud-iac/modules/oci-guest/autonomous-db?ref=oci-guest/autonomous-db/v1.4.0"
  tenant_id       = var.tenant_id
  tenant_class    = var.tenant_class
  workload_class  = "transactional"
  data_class      = "INTERNAL_ONLY,PII_QUASI"
  compliance_pack = ["soc2","kr-isms-p","gdpr"]
}

module "billing_cache" {
  source = "../../../cloud-iac/modules/oci-guest/oci-cache-valkey?ref=oci-guest/oci-cache-valkey/v1.0.2"
  tenant_id     = var.tenant_id
  tenant_class  = var.tenant_class
  cluster_size  = "medium"
  tls_posture   = "mtls-internal-only"
  eviction      = "allkeys-lfu"
}

module "billing_object_storage" {
  source = "../../../cloud-iac/modules/oci-guest/object-storage?ref=oci-guest/object-storage/v1.2.1"
  tenant_id    = var.tenant_id
  tenant_class = var.tenant_class
  bucket_name  = "billing-invoices"
  retention    = "7y"
}
```

D-2.7. The wrapper above is ~25 LOC of body. The same µservice without the shared library would author ~300 LOC of from-scratch HCL inlining the resource declarations.

### D-3: Versioning + pinning

D-3.1. Each module is independently versioned with semantic versioning scoped to its module-path. Tag form: `<context>/<primitive>/v<MAJOR>.<MINOR>.<PATCH>`.

D-3.2. MAJOR bumps for breaking changes to the input or output contract. MINOR bumps for backward-compatible additions (new optional input variable; new output value). PATCH bumps for bug fixes that do not change the contract.

D-3.3. Consumer wrappers MUST pin a specific tag per B2.007. The `oya-check-iac-module-pin` lane refuses unpinned `?ref=main` or `?ref=` consumption.

D-3.4. Consumers MAY pin to a major-only tag for opt-in auto-upgrade (`?ref=oci-guest/autonomous-db/v1`) at the cost of broader version exposure; the lane permits major-only tags but warns at REPORT-ONLY.

D-3.5. Module upgrades follow a quarterly upgrade window matching the network-substrate-versions cadence. The upgrade window opens for a 2-week soak; consumers update their pins during the window.

D-3.6. Library-side breaking changes require a Migration Note in `docs/standards/iac-module-catalog.md` documenting the upgrade path.

D-3.7. Library-side sunset of a primitive follows ADR-0108 sunset discipline: the primitive is marked deprecated in the catalog, retains its source for at least one quarterly cycle, and emits `tflog.Warn` to consumers.

### D-4: Canonical primitive enumeration per context (~50 total)

The library starts with the following ~50 canonical primitives. Each primitive is a self-contained module under `cloud/cloud-iac/modules/<context>/<primitive>/`. The enumeration is the binding minimum-set; additional primitives are added per the catalog discoverability contract in §D-8.

#### D-4.1 `aws-guest` context (10 primitives)

| Primitive | Purpose |
|---|---|
| `eks-cluster` | Managed Kubernetes control plane + node groups (kata-pool + runc-pool per ADR-0338) |
| `rds-postgres` | Managed PostgreSQL with tenant-class-aware sizing + backup retention |
| `elasticache-valkey` | Managed Valkey per ADR-0336 D-2 (AWS ElastiCache for Valkey engine) |
| `s3-bucket` | Object storage with tenant-class-aware retention + lifecycle + cross-region replication |
| `kms-key` | KMS CMK + alias + IAM grants per ADR-0244 tenant-scoped key hierarchy |
| `iam-role` | IAM role + trust policy + policy attachments per ADR-0244 service-linked-role pattern |
| `vpc` | VPC + subnets (public + private + isolated) + route tables + NAT + flow logs |
| `alb` | Application Load Balancer + listener + target groups + WAF + ACM cert |
| `route53-zone` | DNS zone + records + DNSSEC + health checks |
| `sg-baseline` | Security group baseline (allow-list canonical egress; deny-by-default ingress) |

#### D-4.2 `oci-guest` context (8 primitives, paid sub-context)

| Primitive | Purpose |
|---|---|
| `oke-cluster` | OCI Kubernetes Engine cluster + node pools + Kata + Cloud Hypervisor per ADR-0338 |
| `autonomous-db` | OCI Autonomous Database (transactional or analytics, per workload_class) |
| `oci-cache-valkey` | OCI Cache with Valkey engine per ADR-0336 D-2 |
| `object-storage` | OCI Object Storage with retention + lifecycle |
| `vault` | OCI Vault + master encryption keys + tenant-scoped key hierarchy |
| `vcn` | Virtual Cloud Network + subnets + route tables + NAT + flow logs |
| `lb` | OCI Load Balancer (Layer-7) + listener + backend set + WAF |
| `drg` | Dynamic Routing Gateway for cross-VCN + on-prem peering |

#### D-4.3 `oci-guest/always-free` sub-context (6 primitives, demo_trial only)

| Primitive | Purpose |
|---|---|
| `ampere-a1` | 2× Ampere A1 ARM 4 OCPU + 24 GB instance (perpetual free) |
| `e2-micro` | 2× E2 micro x86 instance (perpetual free) |
| `atp` | 2× Autonomous Transaction Processing (perpetual free; 20 GB cap) |
| `adw` | 2× Autonomous Data Warehouse (perpetual free; 20 GB cap) |
| `lb-free` | 1× Load Balancer 10 Mbps (perpetual free) |
| `vault-free` | Free-tier Vault (master key only; no virtual private vault) |

Notes per `feedback_oci_always_free_maximization_2026_05_20`: every Always-Free primitive MUST validate `tenant_class == "demo_trial"` per ADR-0331 §D-8.4 and MUST respect the OCI perpetual-free ceiling (200 GB block storage, 10 GB object storage, 10 TB egress/month, etc.) by clamping resource sizes inside the module body.

#### D-4.4 `on-prem` context (7 primitives)

| Primitive | Purpose |
|---|---|
| `kubeadm-cluster` | kubeadm + containerd + Kata + Cloud Hypervisor cluster bootstrap |
| `cilium-cni` | Cilium CNI install + BPF datapath + network policies |
| `istio-ambient` | Istio Ambient Mesh + ztunnel + waypoint (per ADR-0254) |
| `envoy-gateway` | Envoy Gateway (Kubernetes Gateway API) + listeners + TLS termination |
| `valkey-cluster` | Self-managed Valkey cluster (Helm release) per ADR-0336 D-2 |
| `pg-cluster` | Self-managed PostgreSQL cluster (CloudNativePG operator) |
| `milvus-cluster` | Self-managed Milvus vector cluster per ADR-0192 |

#### D-4.5 `colo` context

Alias of `on-prem`. The colo module library is symlinked-via-source-pin (`source = "../../on-prem/<primitive>"`) for every primitive that is colo-identical. Colo-specific primitives (e.g., a custom rack-aware topology assumption) MAY ship as a separate `colo/<primitive>` body; otherwise the alias suffices.

#### D-4.6 `oyatie-as-cloud-provider` context (~10 primitives)

| Primitive | Purpose |
|---|---|
| `cell-zone` | Cell zone (Tier 0..4 per ADR-0248) + cell-local control plane |
| `shard-cell` | Per-tenant shuffle-sharded cell assignment (per ADR-0248 + ADR-0244) |
| `tenant-namespace` | Per-tenant namespace within a cell + RBAC + NetworkPolicy + Cedar bindings |
| `per-cell-nodepool-kata` | Kata + Cloud Hypervisor nodepool for Tier 0+1 workloads per ADR-0338 |
| `per-cell-nodepool-runc` | runc nodepool for Tier 2+3 workloads per ADR-0338 |
| `cell-observability-collector` | Per-cell observability collector + audit-chain bridge |
| `cell-audit-chain-bridge` | Per-cell audit-chain bridge to the substrate audit-chain |
| `cross-cell-mesh-link` | Cross-cell ambient-mesh wiring per ADR-0254 |
| `cell-cedar-bundle-loader` | Per-cell Cedar policy bundle loader per ADR-0243 + ADR-0150 |
| `cell-compliance-pack-binding` | Per-cell compliance pack binding (HIPAA / GDPR-strict / CSAP / PCI / IL5 / etc.) per ADR-0251 |

Aggregate count: 10 + 8 + 6 + 7 + 10 = 41 primitives in the binding minimum-set. Additional cross-cutting primitives (e.g., a `prometheus-stack` for the observability µservice's own deployment, a `cert-manager` install for the cert pipeline) bring the total to approximately 50 at landing time.

### D-5: OpenTofu signed-modules per ADR-0181

D-5.1. Every module release carries a cosign attestation signed by the canonical Oyatie root key per ADR-0181.

D-5.2. The attestation is stored alongside the module body at `cloud/cloud-iac/modules/<context>/<primitive>/.cosign/<tag>/attestation.json`.

D-5.3. The attestation digest is published in the catalog (§D-8).

D-5.4. Consumer wrappers reference the attestation digest in a `provider_meta` block or equivalent per ADR-0181 wiring.

D-5.5. The `oya-check-iac-module-signature-cosign` lane verifies that every consumer's pinned module source has a corresponding attestation.

D-5.6. Attestation rotations (key rotation, root-key replacement) follow ADR-0181 rotation discipline and require coordinated consumer re-pin.

### D-6: Migration path for existing per-µservice iac/

D-6.1. Existing per-µservice `iac/<context>/` directories that contain from-scratch HCL bodies remain compilable until each µservice's migration bucket lands.

D-6.2. Each µservice owner files a per-µservice IP at `microservices/<name>/IPs/IP-iac-module-library-migration.md` documenting:

- Which primitives the µservice invokes (per context).
- Which tenant-scoped parameters are passed per invocation.
- Which existing µservice-specific resources factor into the shared library (and trigger a library-side primitive addition or extension).
- Which existing µservice-specific resources remain µservice-specific (and stay under µservice ownership with a declared `extends cloud-iac/modules/<context>/<base>` cross-reference for inheritance documentation).
- The cosign attestation pin per invocation.

D-6.3. The migration bucket is dispatched per the canonical-build phase order under ADR-0328:

- Phase 0 (cloud-* substrate µservices) migrate first.
- Phase 1 (foundations: identity, tenancy, audit-chain, observability, cloud-billing, cloud-billing-tax) follow.
- Phase 2 (capability substrate) follow.
- Phase 3 (communication + collaboration) follow.
- Phase 4A (Big 8 enterprise displacement) follow.
- Phase 4B (long-tail B2B SaaS / cloud-infra / developer tools) follow last.

D-6.4. Per-µservice migration is bespoke per ADR-0322 substance-bar discipline. Template stamping the same migration IP across µservices is a P0 finding per ADR-0324.

D-6.5. Each migration bucket's outcome is:

- The µservice's `iac/<context>/main.tf` files become thin wrappers ≤ 80 LOC each.
- The µservice's `manifest.json#iac_module_invocations` field lists the invoked primitives.
- The µservice's prior from-scratch HCL is deleted (or moved under `legacy/` if needed for historical reference).
- The µservice's `oya-check-iac-module-path-canonical` lane goes green.

D-6.6. Migration buckets file evidence at `evidence/iac-module-migration/<microservice>/<context>/` documenting the diff before/after.

### D-7: Module versioning + quarterly upgrade windows

D-7.1. Module versions follow semantic versioning per §D-3.

D-7.2. Quarterly upgrade windows align with the network-substrate-versions cadence:

- Quarter 1: Q1 upgrade window opens at the start of the quarter; 2-week soak; consumers re-pin during the soak.
- Quarter 2: Q2 upgrade window; consumers re-pin.
- And so on.

D-7.3. Major version bumps require a Migration Note in the catalog documenting the upgrade path.

D-7.4. Security patches (CVE fixes in OpenTofu providers, Cilium, Istio, Envoy, kubeadm components, Cloud Hypervisor, Kata) are released as patch versions at any time outside the quarterly window. Consumers re-pin per the same cosign attestation discipline.

D-7.5. Sunset of a primitive follows ADR-0108: marked deprecated in the catalog, retains source for at least one quarterly cycle, emits `tflog.Warn`. Hard removal happens at the end of the deprecation window.

D-7.6. The quarterly window is observable via cloud-iac's own observability emission (per ADR-0263): metric `cloud_iac_module_consumer_pins{module,version_pinned,version_current}` tracks consumer lag.

### D-8: Module catalog + discovery + docs

D-8.1. The canonical catalog page is `docs/standards/iac-module-catalog.md`. The catalog enumerates every primitive:

- Module path + name + purpose
- Current semantic version
- Cosign attestation digest
- Input variable contract (variable name + type + default + description)
- Output contract (output name + type + description)
- Compliance-pack support matrix
- Tenant-class support matrix (demo_trial / paid)
- Cell-tier support matrix (Tier 0..4 per ADR-0248)
- Pod-runtime-tier support matrix (Tier 0..3 per ADR-0338)
- Example invocation snippet
- Deprecation status (active | deprecated | sunset)
- Migration notes (if any)

D-8.2. The machine-readable mirror is `cloud/cloud-iac/modules/catalog.json` for automation consumers.

D-8.3. Module additions to the library require a corresponding catalog entry in the same change set. The `oya-check-iac-module-catalog-discoverability` lane refuses additions without catalog entries.

D-8.4. The catalog is the canonical discovery surface. New µservice owners read the catalog to select primitives; the catalog is the answer to "what's available", not a search across the corpus.

D-8.5. The catalog page is owned by axis-cloud-iac. Per-primitive substantive content is bespoke per ADR-0322 substance-bar discipline.

D-8.6. The catalog page cross-references this ADR (ADR-0339) as authority.

### D-9: New CI lane `oya-check-iac-shared-module-usage`

D-9.1. The new lane `oya-check-iac-shared-module-usage` scans every per-µservice `iac/<context>/main.tf` and verifies:

- File present at the expected path for every declared `tenant_class_iac_variants` entry in the µservice's manifest.
- File contains only `module` blocks, `variable` blocks, `output` blocks, `terraform` blocks, and `provider` blocks. No `resource` blocks.
- File is ≤ 80 LOC excluding comments and blank lines.
- Every `module` source pins a specific version (no `?ref=main`, no bare `?ref=`).
- Every `module` source resolves to a path under `cloud/cloud-iac/modules/<context>/<primitive>/` (or a Git-tag-pinned URL to the same path).
- Every consumed module has a cosign attestation per ADR-0181.
- Every consumed module is enumerated in the µservice's `manifest.json#iac_module_invocations` field.

D-9.2. The lane runs in CI under `oya-check-iac-shared-module-usage`. Operational from this ADR's Acceptance as REPORT-ONLY; promotes to BLOCKER per-µservice as each µservice's migration bucket lands.

D-9.3. The lane's evidence is filed at `evidence/iac-shared-module-usage/<microservice>/<context>/` per run.

### D-10: Always Free module category as sub-context

D-10.1. The sub-context `oci-guest/always-free/` is a first-class category in the module library, providing demo_trial-tenant-only primitives that fit within the OCI Always Free perpetual ceiling.

D-10.2. Every Always-Free primitive validates `tenant_class == "demo_trial"` per ADR-0331 §D-8.4. The validation lives in the module body, not in per-µservice wrappers.

D-10.3. Every Always-Free primitive clamps resource sizes to fit the perpetual-free ceiling. Examples: `ampere-a1` clamps to 2 instances × 4 OCPU × 24 GB; `atp` clamps to 2 instances × 20 GB; `lb-free` clamps to 10 Mbps; etc.

D-10.4. Demo_trial tenants on `oci-guest/always-free/` are pinned to a single Always-Free cell per ADR-0248 cellular architecture. The shared module provides the cell-binding parameter as a fixed value (`cell_class = "always-free"`).

D-10.5. Always-Free primitives MAY NOT activate compliance packs (per ADR-0251 + ADR-0331 §D-5 forbidden_features for demo_trial).

D-10.6. Always-Free primitives MAY NOT activate provider BYOK (per ADR-0255 §D-4 + ADR-0331 §D-5 forbidden_features for demo_trial).

D-10.7. Always-Free primitives MAY NOT activate sovereign-cell routing (per ADR-0331 §D-5 forbidden_features for demo_trial).

D-10.8. The Always-Free catalog page at `docs/standards/iac-module-catalog.md#always-free` enumerates the Always-Free primitives separately with explicit demo_trial-only labeling.

## E. Enforcement-by-lanes

E.1 `oya-check-iac-shared-module-usage` (new) — verifies per-µservice `iac/<context>/main.tf` uses shared library primitives. REPORT-ONLY at ADR Acceptance; BLOCKER per-µservice as each migration bucket lands.

E.2 `oya-check-iac-module-path-canonical` (new) — refuses inline `resource` declarations in per-µservice wrappers; refuses module paths outside `cloud/cloud-iac/modules/<context>/<primitive>/`. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance for new authoring; BLOCKER per-µservice as each migration bucket lands.

E.3 `oya-check-iac-module-signature-cosign` (new) — verifies every consumed module has a corresponding ADR-0181 cosign attestation. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.4 `oya-check-iac-module-pin` (new) — refuses unpinned `?ref=main`, `?ref=` (empty), or bare relative-path consumption without a version tag. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.5 `oya-check-iac-opentofu-only` (existing) — refuses HashiCorp Terraform syntax in the library or in per-µservice wrappers. Operational from this ADR's Acceptance (already BLOCKER under ADR-0218).

E.6 `oya-check-iac-thin-wrapper-line-floor` (new) — refuses per-µservice `iac/<context>/main.tf` exceeding 80 LOC excluding comments and blank lines. REPORT-ONLY at Acceptance; BLOCKER per-µservice as each migration bucket lands.

E.7 `oya-check-iac-module-catalog-discoverability` (new) — refuses module-library additions or modifications that do not include a corresponding catalog entry update in `docs/standards/iac-module-catalog.md`. REPORT-ONLY at Acceptance; BLOCKER 30 days post-Acceptance.

E.8 `cloud-iac` µservice manifest gains a top-level `module_library_scope` field documenting that the µservice's primary deliverable is now the shared module library + the IaC renderer/validator/applier crates. The field is informational; no lane enforces it directly, but the catalog page references it.

## F. Alternatives Rejected

F.1 **Per-µservice from-scratch IaC modules (status-quo target).** Every µservice authors its own from-scratch HCL body per context. Rejected because: 60-70% of content duplicates structurally across 77 µservices; aggregate ~120,000 LOC of duplicated HCL; drift risk; quarterly upgrade window touches N modules per change; substance-bar regression as µservice owners spend authoring effort on plumbing; cargo-vet / SLSA audit scope explodes to ~385 artifacts; no hyperscaler-grade precedent.

F.2 **Single mega-module library spanning all contexts.** One library per primitive, with the body containing branches for each context (`if context == "aws-guest" { aws_eks_cluster } else if context == "oci-guest" { oci_containerengine_cluster } ...`). Rejected because: OpenTofu provider configuration is per-context (AWS provider vs OCI provider vs kubernetes provider); a single body trying to satisfy all five contexts cannot use distinct provider blocks; the body becomes unreadable; testing is impossible without all five providers configured; the failure-mode of "OCI provider changes break the AWS path" is unacceptable; the substance-bar lane cannot distinguish per-context substance.

F.3 **Registry-based dependency on a SaaS registry like HashiCorp Terraform Registry.** Consume shared modules from registry.terraform.io or a private HashiCorp Cloud Terraform registry. Rejected because: ADR-0218 forbids HashiCorp Terraform; the modules would still be OpenTofu modules but the registry would be HashiCorp Inc. infrastructure, creating supply-chain dependency on a forbidden vendor; OpenTofu does not have a SaaS-registry analog in production; the in-tree path under `cloud/cloud-iac/modules/` provides better provenance (cosign + Git-tag pinning) than any SaaS-registry alternative; cross-repo consumers can still consume via Git-tag-pinned URLs.

F.4 **Move all per-µservice IaC into cloud-iac µservice tree (collapse all iac/ into cloud-iac).** Eliminate per-µservice `iac/` directories entirely; cloud-iac owns every µservice's deployment IaC. Rejected because: per-µservice ownership of "which primitives" remains with the µservice owner (cardinality + cap shape + tenant-class variants are µservice-specific decisions); the thin-wrapper model preserves per-µservice ownership of the **substance** while consolidating the **plumbing**; collapsing entirely would make cloud-iac the single point of failure for all 77 µservices' deployment authoring.

F.5 **Skip the shared library; keep status-quo but add lint rules.** Add CI lanes that lint per-µservice from-scratch HCL for inconsistencies. Rejected because: lint rules are detective, not preventive; the duplication mass remains; drift still accumulates between lint runs; the substance-bar regression remains; no quarterly-upgrade-window mechanism exists.

## G. Multispectrum Review v2.4.0

Per ADR-0322 §D-2 and ADR-0328 §D-4, this ADR is subject to multispectrum-review v2.4.0 evaluation across the F-family critique facets, M-family meta facets, and A-family own-policy-adherence facets. Evidence files land at `evidence/debate/ADR-0339/<facet>.md` after this ADR is opened in a review-track PR.

The expected critique surface:

- **F1 (correctness).** Are the ~50 enumerated primitives the right minimum-set? Are any obvious primitives missing (e.g., does the corpus need a `nats-cluster` primitive for the messaging substrate; does the corpus need a `clickhouse-cluster` primitive for analytics per ADR-0337)? Are the version-pinning semantics correctly described?
- **F2 (architecture).** Is the five-context taxonomy correct, or should `oyatie-public-cloud` be re-separated from `oyatie-as-cloud-provider`? Is the `colo` alias to `on-prem` correct?
- **F3 (security).** Is the cosign attestation pattern correctly bound to ADR-0181? Are the tenant-class validation responsibilities split correctly between module and wrapper?
- **F4 (performance).** Does the OpenTofu module-cache hit pattern actually deliver the claimed `init` time reduction?
- **F5 (operability).** Is the quarterly upgrade window mechanism correctly aligned with network-substrate-versions cadence? Is the per-cell observability emission correctly described?
- **F6 (compliance).** Does the per-pack compliance-pack-attestation-file pattern in `compliance/` subdirectory carry the correct compliance-pack semantics per ADR-0251?
- **F7 (cost).** Is the OCI Always Free clamping correctly described per `feedback_oci_always_free_maximization_2026_05_20`?
- **F8 (testability).** How are shared modules tested? Per-primitive `terratest`-equivalent under OpenTofu?
- **F9 (failure modes).** Is the failure-mode tree in C.5 complete?
- **M1 (counterpart-precedent calibration).** Are AWS Solutions Constructs, GCFT, AVM, Terraform Registry the right precedents?
- **M2 (substance bar).** Is the per-µservice substance correctly captured as "primitive selection + tenant-scoped parameters"?
- **A1..A7 (own-policy-adherence).** Does this ADR adhere to the naming BNF v4, documentation rigor 1.1, structural placement under `docs/decisions/`, architectural boundaries (cloud-iac owns the library), dependency policy (OpenTofu only, cosign required), schema (manifest field naming), and algorithmic invariants (per-µservice ownership of substance + cloud-iac ownership of plumbing)?

## H. Enforcement + Sunset

H.1 **Enforcement transition.** From ADR Acceptance, the seven new lanes (§E) start REPORT-ONLY. They promote per the schedule:

- E.2, E.3, E.4, E.7 promote to BLOCKER 30 days post-Acceptance (for new authoring).
- E.1 and E.6 promote to BLOCKER per-µservice as each µservice's migration bucket lands (per-µservice schedule under ADR-0328 canonical-build phase order).
- E.5 is already BLOCKER under ADR-0218 (no transition needed).

H.2 **Sunset window.** The 30-day post-Acceptance window is the sunset window for new authoring of from-scratch per-µservice IaC bodies. After day 30, new authoring MUST use the shared-library shape; existing per-µservice bodies remain compilable until their migration bucket lands.

H.3 **Wave 15Q-IaC-modules sub-wave.** The Wave 15Q-IaC-modules sub-wave (queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.wave_15.subwaves`) authors the ~50 module bodies + the catalog + the lane implementations + a reference wrapper at the cloud-billing µservice (as the canonical example). Sub-wave dispatch follows ADR-0328 batch discipline.

H.4 **Per-µservice migration sub-waves.** Each µservice's migration bucket is sequenced under ADR-0328 canonical-build phase order. Phase 0 cloud-* µservices migrate first; Phase 4B long-tail B2B SaaS µservices migrate last. The aggregate corpus-wide migration is expected to span multiple realignment waves.

H.5 **Exception clause.** None. No µservice may continue authoring from-scratch IaC bodies after the 30-day sunset window for new authoring (existing bodies are permitted until each migration bucket lands).

H.6 **Sunset of the prior shape.** From-scratch per-µservice IaC body authoring is forbidden after day 30 for new authoring. The retirement is recorded in `tools/hooks/_canonical-primitives.md` per the canonical-primitives cheat sheet pattern.

## I. Cross-references

I.1 Memory anchors:

- `feedback_idea_refine_decisions_2026_05_21` — user directive of 2026-05-21 capturing this decision as Decision 3.
- `feedback_zero_handroll_opentofu_only_2026_05_20` — OpenTofu-only constraint preserved verbatim.
- `feedback_multi_context_provider_agnostic_2026_05_20` — five-context taxonomy preserved.
- `feedback_oci_always_free_maximization_2026_05_20` — `oci-guest/always-free/` sub-context first-class category.
- `feedback_no_silent_regression` — substrate-shape change requires ADR + version bump + sunset.
- `feedback_quality_performance_scalability_bar` — hyperscaler-grade precedent (AWS / GCFT / AVM).
- `feedback_clean_architecture_requirements` — separation of substance (per-µservice ownership) from plumbing (cloud-iac shared library).
- `feedback_microservice_ownership_coherence_2026_05_20` — per-µservice owner remains accountable for which primitives the µservice invokes + which tenant-scoped parameters are passed.
- `feedback_rust_strict_only_no_python_2026_05_20` — the constraint applies to non-IaC code; IaC remains HCL/OpenTofu per ADR-0218.
- `feedback_bominal_inheritance_precedence` — Bominal corpus inherits the same shared-library pattern under its own migration plan.
- `feedback_docs_substance_not_scaffold_2026_05_20` — substance-bar applies to per-µservice IaC substance (primitive selection + parameters), not to plumbing.
- `feedback_drift_too_big_2026_05_20` — the 385-module-dir blast-radius is exactly the drift this ADR prevents.

I.2 ADR anchors:

- ADR-0181 (cosign-signed artifacts and modules)
- ADR-0211 (in-house tech stack preference)
- ADR-0212 (buildability doctrine)
- ADR-0215 (multi-context platform)
- ADR-0216 (deployment-context iac layout) — amended by this ADR
- ADR-0218 (OpenTofu not Terraform)
- ADR-0244 (tenant scoping universal primitive)
- ADR-0245 (substrate vs product layering)
- ADR-0247 (self-modification doctrine)
- ADR-0248 (Amazon-shape cellular architecture)
- ADR-0250 (build ahead of certification)
- ADR-0251 (compliance pack cell certification levels)
- ADR-0254 (Kubernetes everywhere + Cloud Hypervisor + Kata)
- ADR-0255 (BYOK everywhere credentials)
- ADR-0322 (substance bar as doctrine and CI enforcement)
- ADR-0324 (anti-script authoring doctrine)
- ADR-0328 (substance bar as canonical sequence and batch discipline)
- ADR-0329 (tier system retired; replaced by tenant_class)
- ADR-0330 (tenant_class demo_trial vs paid composable billing components)
- ADR-0331 (cross-microservice tenant_class adoption template)
- ADR-0335 (foundry retired; absorbed by intelligence)
- ADR-0336 (Valkey not Redis substrate)
- ADR-0337 (Iceberg canonical OLAP)
- ADR-0338 (Pod runtime tier 0..3)

I.3 Spec anchors:

- `/specs/master-plan-sequencing.json` — adds the Wave 15Q-IaC-modules sub-wave + queued ADR-0339 entry per H.3.
- `/specs/microservices/cloud-iac.json` — manifests-index pointer to the updated cloud-iac manifest with the module_library_scope expansion note.
- `/specs/microservices/manifest-schema.json` — admits the per-µservice `iac_module_invocations` field per B2.016.
- `/specs/decision-principles.json` — informational; this ADR does not change decision principles.
- `/specs/markdown-retirement-policy.json` — informational; this ADR does not retire any prior markdown.

I.4 Companion-doc anchors:

- `docs/standards/dependency-policy.md` — OpenTofu provider pins applied at the library level rather than per-µservice.
- `docs/standards/iac-module-catalog.md` (new) — canonical catalog page per §D-8.
- `cloud/cloud-iac/ARCHITECTURE.md` — gains the module-library scope expansion description at landing.
- `cloud/cloud-iac/manifest.json` — gains the `module_library_scope` field per E.8.
- `tools/hooks/_canonical-primitives.md` — gains a IaC Module Library entry naming `cloud/cloud-iac/modules/<context>/<primitive>/` as canonical.

## J. Completion Report

<!--
adr: ADR-0339
status: Proposed
date: 2026-05-21
session: 2026-05-21 /idea-refine triplet (Decision 3 of 3)
sibling_adrs: ADR-0337 (Iceberg canonical OLAP), ADR-0338 (Pod runtime tier 0..3)
authority_source: feedback_idea_refine_decisions_2026_05_21
canonical_path: cloud/cloud-iac/modules/<context>/<primitive>/
canonical_wrapper: microservices/<name>/iac/<context>/main.tf (≤80 LOC, thin invocation)
canonical_contexts: aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider (+ oci-guest/always-free sub-context)
primitive_count_minimum_set: ~50 (10 aws-guest + 8 oci-guest + 6 always-free + 7 on-prem + colo alias + 10 oyatie-as-cloud-provider)
blast_radius_collapse: 385 from-scratch module dirs → ~50 shared primitives + 385 thin wrappers (~120,000 LOC duplication eliminated)
new_lanes: 7 (oya-check-iac-shared-module-usage, -module-path-canonical, -module-signature-cosign, -module-pin, -thin-wrapper-line-floor, -module-catalog-discoverability) + 1 existing preserved (oya-check-iac-opentofu-only)
sunset_window: 30 days post-Acceptance for new authoring; per-µservice migration follows ADR-0328 canonical-build phase order
wave_queue: Wave 15Q-IaC-modules added to /specs/master-plan-sequencing.json#realignment_wave_sequence.wave_15.subwaves
manifest_expansion: cloud/cloud-iac/manifest.json gains module_library_scope expansion note
out_of_scope: authoring the ~50 module bodies (sequenced as Wave 15Q-IaC-modules); migrating per-µservice iac/ (sequenced per-µservice under ADR-0328)
hyperscaler_precedents: AWS Solutions Constructs; Google Cloud Foundation Toolkit; Azure Verified Modules; HashiCorp Terraform Registry pattern
commits: none required at this ADR's landing
-->
