---
id: ADR-0349
title: Self-hostable CI/CD substrate — cloud-ci augments GitHub Actions for self-hostable contexts; ArgoCD replaces manual kubectl/Helm CLI deploys; both OSS Class C approved per ADR-0211 + Contributor stewardship per ADR-0345; provisioned via OpenTofu modules per ADR-0339 in every multi-context deployment per ADR-0215 including air-gap per ADR-0164 (superseded by ADR-0515)
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-platform
  - ops-sre-reliability
  - axis-cloud-iac
  - axis-observability
  - council-security
owners:
  - council-architecture
  - ops-platform
  - ops-sre-reliability
  - axis-cloud-iac
  - axis-observability
  - council-security
supersedes: []
superseded_by: [ADR-0515]
amends:
  - ADR-0028-cloud-microservice-architecture.md (the cloud-microservice architecture declared K8s + IaaS + control-plane primitives; this ADR layers two named CI/CD substrate primitives — cloud-ci for self-hostable CI and ArgoCD for declarative GitOps CD — onto that architecture so every microservice gains a canonical CI/CD chain rather than an ad-hoc per-context deployment script set. The architecture is preserved verbatim; cloud-ci + ArgoCD become the canonical CI/CD layer per microservice without changing the underlying K8s + cell topology)
  - ADR-0181-cosign-signed-artifacts-and-modules.md (the cosign-signed artifact discipline is preserved verbatim; this ADR clarifies that ArgoCD's image-fetch path MUST verify cosign signatures before sync per the existing image-promotion-pipeline contract — ArgoCD becomes the post-pipeline runtime enforcer for the signed-image discipline, not a new gate)
  - ADR-0211-in-house-tech-stack-preference.md (the in-house tech stack preference declares Class C OSS as the canonical category for substrate adoption; this ADR confirms cloud-ci and ArgoCD as approved Class C entries with explicit Contributor-class stewardship per ADR-0345 + named CVE-response SLAs)
  - ADR-0215-multi-context-deployment.md (multi-context deployment declares the five canonical contexts oyatie-public + guest-on-aws + guest-on-oci + on-prem + colo + oyatie-as-provider + air-gap; this ADR declares that cloud-ci + ArgoCD run in every context including air-gap and that the per-context provisioning lives under microservices/cloud-iac/modules/<context>/cloud-ci/ and /<context>/argocd/ per ADR-0339 shared IaC module library)
  - ADR-0221-agentic-development-pipeline-hardening.md (the hook-vs-gate doctrine is preserved verbatim; this ADR clarifies that cloud-ci is a CI gate authority — its pipeline executions are CI-gate runs, not hook surfaces — and that ArgoCD's sync action is a CD gate authority bound by cosign verification + audit-chain emission per ADR-0263)
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md (the K8s-everywhere doctrine declares K8s + Kata pods + Cloud Hypervisor as the canonical runtime substrate; this ADR layers cloud-ci + ArgoCD on top of that substrate so every CI run and every CD sync executes on the same K8s primitive set)
  - ADR-0339-shared-iac-module-library.md (the shared IaC module library declares microservices/cloud-iac/modules/<context>/ as the canonical OpenTofu module home; this ADR adds two module families — cloud-ci/ and argocd/ — under each per-context directory so per-tenant provisioning is `tofu apply` end-to-end per zero-handroll OpenTofu-only feedback memory)
related_adrs:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0105-thirteen-layer-enum.md
  - ADR-0106-naming-justification.md
  - ADR-0107-naming-bnf-v4.md
  - ADR-0108-sunset-lifecycle-automation.md
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0164-air-gap-sovereign-deployment.md
  - ADR-0181-cosign-signed-artifacts-and-modules.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-deployment.md
  - ADR-0218-opentofu-not-terraform.md
  - ADR-0221-agentic-development-pipeline-hardening.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-primitive.md
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-anti-template-doctrine.md
  - ADR-0327-realignment-wave-promotion-gate.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0339-shared-iac-module-library.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
  - ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md
  - ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md
  - ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md
  - ADR-0344-sustainability-finops-dimensional-model.md
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
  - ADR-0346-oya-verify-must-run-full-ci-mirror.md
  - ADR-0347-governance-fitness-bulk-rename.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/oss-stewardship-registry.json
  - /specs/markdown-retirement-policy.json
  - /specs/microservices/manifest-schema.json
  - /specs/root-hub-pointers.json
related_memory:
  - feedback_jenkins_argocd_substrate_2026_05_21
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_os_support_matrix_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_bominal_inheritance_precedence
  - feedback_no_silent_regression
  - feedback_automate_everything
  - feedback_amazon_shape_cellular_architecture
  - feedback_kubernetes_everywhere_pods_cloud_hypervisor
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_build_ahead_of_certification
companion_docs:
  - microservices/cloud-iac/modules/aws-guest/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/aws-guest/argocd/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/oci-guest/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/oci-guest/argocd/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/on-prem/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/on-prem/argocd/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/colo/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/colo/argocd/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/oyatie-as-provider/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/oyatie-as-provider/argocd/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/air-gap/jenkins/ (Wave 15-ZE; OpenTofu module path)
  - microservices/cloud-iac/modules/air-gap/argocd/ (Wave 15-ZE; OpenTofu module path)
  - tools/hooks/_canonical-primitives.md (Lifecycle Skill Map entry for Jenkins + ArgoCD substrates)
  - docs/standards/dependency-policy.md (Jenkins + ArgoCD declared Class C OSS substrates with Contributor stewardship)
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_jenkins_argocd_substrate_2026_05_21.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0181-cosign-signed-artifacts-and-modules.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0215-multi-context-deployment.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0339-shared-iac-module-library.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-wave-15-ze-modules-and-jenkinsfile-helm-chart-authoring-lands
enforced_by:
  - oya-governance-jenkins-github-actions-parity (new lane; refuses Jenkinsfile / .github/workflows drift such that a CI step exists in one surface but not the other across the per-microservice CI-parity contract enumerated in D-3 below; promoted to BLOCKER 30 days post Wave 15-ZE-completion)
  - oya-governance-argocd-application-cosign-verified (new lane; refuses ArgoCD Application CRD sources that reference an image without a cosign-verify policy attached per D-6 + ADR-0181; promoted to BLOCKER 30 days post Wave 15-ZE-completion)
  - oya-governance-argocd-tenant-namespace-isolation (new lane; refuses ArgoCD Application authoring that crosses tenant namespaces without a Cedar policy gate granting cross-tenant access per D-11 + ADR-0243; promoted to BLOCKER 30 days post Wave 15-ZE-completion)
  - oya-governance-jenkins-jcasc-only (new lane; refuses Jenkins controller state declared via the UI; every Jenkins controller state file is authored under microservices/cloud-iac/modules/<context>/jenkins/jcasc/ with declarative JCasC YAML per D-1; promoted to BLOCKER 30 days post Wave 15-ZE-completion)
  - oya-governance-deploy-audit-chain-emit (new lane; refuses ArgoCD sync transitions that do not emit an audit-chain row per ADR-0263 §D.4 deploy-event class; promoted to BLOCKER 30 days post Wave 15-ZE-completion)
purpose: >
  (Superseded by ADR-0515.) Declare cloud-ci and ArgoCD as the two canonical
  self-hostable CI/CD substrates for the Oyatie corpus. cloud-ci is the
  canonical CI orchestrator for air-gap + on-prem + colo + oyatie-as-cloud-provider
  deployment contexts where GitHub Actions runners are not available; it runs on
  Oyatie K8s per ADR-0254 and is configured-as-code with declarative pipelines.
  GitHub Actions is RETAINED as the primary CI for the hosted-on-GitHub PR review
  surface (oya verify + PR checks); cloud-ci augments rather than replaces it.
  ArgoCD is the canonical GitOps CD orchestrator; per-cluster Application CRDs
  declare which microservices deploy to which cluster, ArgoCD reconciles the
  declared state from git, cosign signature verification per ADR-0181 gates
  every image fetched on sync, and audit-chain rows emit per ADR-0263 on
  every sync transition. ArgoCD REPLACES manual `kubectl apply` and Helm CLI
  deploys across all contexts. Both substrates are OSS Class C approved per
  ADR-0211 and carry Contributor-class stewardship per ADR-0345 (7-day P0
  CVE SLA + 30-day P1 SLA + per-quarter dev-day budget for upstream patches).
  Both are provisioned via OpenTofu modules under
  microservices/cloud-iac/modules/<context>/cloud-ci/ and /<context>/argocd/
  per ADR-0339 shared IaC module library and the zero-handroll-OpenTofu-only
  feedback memory of 2026-05-20. Five CI lanes enforce the substrate
  contract: cloud-ci-github-actions-parity refuses drift between
  cloud-ci pipelines and .github/workflows for the per-microservice CI matrix;
  argocd-application-cosign-verified refuses sourcing of unsigned images;
  argocd-tenant-namespace-isolation refuses cross-tenant ArgoCD project
  leakage; cloud-ci-config-as-code-only refuses UI-driven CI controller
  configuration drift; deploy-audit-chain-emit refuses ArgoCD sync transitions
  without audit-chain emission. The doctrine is binding because the multi-context
  deployment posture per feedback_multi_context_provider_agnostic_2026_05_20
  + air-gap sovereignty per ADR-0164 + zero-handroll OpenTofu-only posture
  per feedback_zero_handroll_opentofu_only_2026_05_20 + the GitOps + signed-
  image + audit-chain triple-binding per ADR-0181 + ADR-0263 + ADR-0254 are
  mutually unsatisfiable without a self-hostable CI substrate and a declarative
  GitOps CD substrate (ArgoCD). The forbidden-alternative competitors (Tekton,
  Flux CD, Spinnaker, CircleCI / Travis / Buildkite SaaS, GitHub Actions
  self-hosted runners pointed at GitHub.com control plane) are each rejected
  with named reasons in §F below. Out of scope: actual authoring of the OpenTofu
  modules (six contexts × two substrates), per-microservice pipeline authoring
  across ~77 microservices, per-microservice Helm chart authoring under
  microservices/<ms>/iac/k8s/helm/, the per-cluster
  `clusters/<cluster-id>/apps.yaml` canonical GitOps manifest, cosign-verify
  policy authoring at the ArgoCD layer, and audit-chain emitter integration at
  the ArgoCD sync hook. All authoring streams are sequenced as Wave 15-ZE in
  /specs/master-plan-sequencing.json under ADR-0328 batch discipline. This ADR
  is doctrine-only; the executor PRs land separately after Acceptance.
---

# ADR-0349: Self-hostable CI/CD substrate — cloud-ci augments GitHub Actions; ArgoCD replaces manual kubectl/Helm CLI deploys; both OSS Class C approved per ADR-0211 + Contributor stewardship per ADR-0345; provisioned via OpenTofu modules per ADR-0339 in every multi-context deployment per ADR-0215 including air-gap per ADR-0164 (superseded by ADR-0515)

## Status

Superseded by ADR-0515 — 2026-06-06: cloud-ci (self-hostable CI orchestrator) + ArgoCD substrate replaced by oya-ci (orchestrator) + oya-cd DeliveryPlane (ArgoCD/Argo-Rollouts reuse-behind-port). Bridge stays operative-but-unratified until cutover. Resolve byp_adr_0349 bypass record (A-CI lane).

This ADR is the canonical CI/CD-substrate decision binding the Oyatie corpus to two named OSS primitives — cloud-ci for self-hostable CI and ArgoCD for declarative GitOps CD — across every deployment context including air-gap. The substrate selections complete a gap that has been latent across multiple prior decisions: ADR-0028 (cloud microservice architecture) and ADR-0254 (Kubernetes everywhere) named the runtime layer but not the CI/CD layer; ADR-0181 (cosign-signed artifacts and modules) named the image-promotion contract but not the runtime enforcer of the contract on deploy; ADR-0215 (multi-context deployment) named the contexts but not the per-context CI/CD provisioning shape; ADR-0218 (OpenTofu not Terraform) named the IaC tool but not the canonical modules for CI/CD primitives; ADR-0339 (shared IaC module library) named the module home but did not enumerate cloud-ci or ArgoCD as canonical module families. This ADR closes those gaps by declaring the two substrate primitives, the per-context OpenTofu module homes, the per-microservice CI-parity and Helm-chart contract, the GitOps manifest path, and the five enforcement lanes.

The ADR is binding because the multi-context deployment requirement per `feedback_multi_context_provider_agnostic_2026_05_20` cannot be satisfied by GitHub Actions alone: customers on-prem, in colocation facilities, on sovereign air-gap deployments per ADR-0164, and tenants on the future Oyatie-as-cloud-provider context per `feedback_multi_context_provider_agnostic_2026_05_20` cannot depend on the GitHub.com control plane. The zero-handroll OpenTofu posture per `feedback_zero_handroll_opentofu_only_2026_05_20` requires that the CI/CD layer lands as a per-context OpenTofu module set, not as hand-authored Helm CLI invocations or kubectl manifests. The cosign-signature + audit-chain discipline per ADR-0181 + ADR-0263 requires a deploy-time enforcer; ArgoCD's Application CRD sync hook is the canonical place for that enforcement. The combination of pressures is mutually unsatisfiable without a CI substrate that runs on tenant K8s and a CD substrate that synchronizes from tenant git into tenant K8s under signed-image policy.

It runs in coordination with the in-flight 2026-05-21 realignment effort. ADR-0339 (shared IaC module library) declared `microservices/cloud-iac/modules/<context>/` as the canonical OpenTofu module home; this ADR populates that home with the two substrate module families (jenkins/ + argocd/) per context. ADR-0345 (OSS stewardship class) declared the Maintainer / Contributor / Consumer triad with named CVE-response SLAs; this ADR places Jenkins and ArgoCD into the Contributor class with declared SLAs and dev-day budgets. ADR-0344 (sustainability + finops dimensional model) declared per-microservice cost decomposition; the Jenkins controller + ArgoCD controller costs decompose into the same dimensional model. ADR-0343 (DR + RTO/RPO matrix per microservice + per compliance pack) declared backup + DR per microservice; this ADR adds explicit backup + DR for Jenkins job history and ArgoCD declarative state. ADR-0342 (API versioning hybrid date + semver) and ADR-0341 (cellular promotion gates explicit per tier) frame the per-microservice API + cell tier postures that ArgoCD's per-cluster Application CRD inherits without modification.

It directly amends ADR-0028 (cloud-microservice architecture) by adding two named CI/CD substrate primitives to the architecture without changing the underlying primitive set; ADR-0181 by clarifying that ArgoCD's image-fetch path MUST verify cosign signatures pre-sync; ADR-0211 by confirming Jenkins and ArgoCD as approved Class C OSS substrates; ADR-0215 by declaring that the CI/CD substrate lives in every context (aws-guest, oci-guest, on-prem, colo, oyatie-as-provider, air-gap); ADR-0221 by clarifying that Jenkins pipeline runs are CI-gate authority surfaces (not hooks); ADR-0254 by layering Jenkins + ArgoCD on the K8s-everywhere runtime; ADR-0339 by adding two module families under the per-context module home.

Enforcement transitions from `advisory-until-wave-15-ze-modules-and-jenkinsfile-helm-chart-authoring-lands` to `BLOCKER` per the lane sequence in §E below: at landing of the Wave 15-ZE bulk authoring PRs (the executor PRs sequenced under ADR-0328 batch discipline that author the twelve OpenTofu modules + per-microservice Jenkinsfile + per-microservice Helm chart + per-cluster apps.yaml + cosign-verify policy + audit-chain emitter integration), the five new lanes (`oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, `oya-governance-deploy-audit-chain-emit`) promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZE-completion. The 30-day window aligns with the ADR-0345 + ADR-0344 + ADR-0347 sibling-ADR sunset windows and reflects the per-microservice impact at promotion (~77 microservices each must carry a Jenkinsfile + Helm chart per the executor PRs).

The decision does not retire GitHub Actions. The decision does not retire `kubectl apply` for emergency-break-glass tenant-operator use under signed audit-chain emission. The decision does not retire Helm as a chart format; only the manual Helm CLI deploy path is replaced. The decision does not change the cosign + audit-chain + Cedar gates that already govern image promotion; it adds a runtime enforcer on the deploy side. The decision does not change the K8s + Cloud Hypervisor + Kata pods runtime substrate per ADR-0254; Jenkins controllers + ArgoCD controllers run as pods on the same substrate. The decision does not introduce a new programming language or build tool; both substrates are operationally familiar to Oyatie engineers.

**Implementation queued as Wave 15-ZE.** Actual OpenTofu module authoring + per-microservice Jenkinsfile authoring + per-microservice Helm chart authoring + per-cluster apps.yaml authoring + cosign-verify policy + audit-chain emitter integration is a separate stream of PRs sequenced under ADR-0328 batch discipline. The Wave 15-ZE entry is added to `/specs/master-plan-sequencing.json` as part of this ADR's required-artifact contract.

## Date

2026-05-21.

## Context

### A.1 Named pressure: GitHub Actions cannot serve air-gap, on-prem, colo, or oyatie-as-provider contexts

Per `feedback_multi_context_provider_agnostic_2026_05_20`, Oyatie runs in six contexts: oyatie-public, guest-on-aws, guest-on-oci, on-prem, colo, and oyatie-as-cloud-provider; plus air-gap per ADR-0164 for sovereign customers. GitHub Actions cannot satisfy four of the seven contexts: on-prem customers depend on their own DMZ-isolated network and cannot trust outbound github.com control-plane connectivity for compliance reasons; colo customers in regulated sectors (financial-services, defense, healthcare) similarly cannot trust an external CI control plane; the Oyatie-as-cloud-provider context is the future surface where Oyatie sells IaaS — that context cannot depend on a competitor's control plane; air-gap deployments by definition have no internet connectivity. A self-hostable CI orchestrator is therefore a hard requirement, not a preference. Jenkins is the canonical OSS choice across hyperscaler precedent (Google, Microsoft, AWS, Oracle, IBM, RedHat, SUSE all ship Jenkins-based CI in their internal stacks for on-prem and air-gap customers); its longevity (since 2005), CVE-response cadence (~weekly LTS patch releases), and configuration-as-code maturity (JCasC + Jenkinsfile) make it the production-ready selection.

### A.2 Named pressure: manual kubectl/Helm CLI deploys defeat the audit-chain + cosign + Cedar discipline

Per ADR-0181 (cosign-signed artifacts and modules), every container image promoted to runtime registries is cosign-signed; per ADR-0263 (observability emission contract), every state-change action emits an audit-chain row; per ADR-0243 (Cedar as universal gate), every authorization decision evaluates through Cedar. A tenant operator running `kubectl apply -f deployment.yaml` from a laptop bypasses all three: kubectl does not verify cosign signatures by default; kubectl does not emit audit-chain rows except via post-hoc K8s audit-log scraping which is asynchronous and incomplete; kubectl bypasses the Cedar gate for deploy-time policy evaluation because the K8s admission controller may or may not be configured to consult Cedar. Manual Helm CLI deploys carry the same gaps. The named pressure is that the cosign + audit-chain + Cedar triad is unenforceable in the absence of a declarative GitOps CD substrate that mediates every deploy through signature verification + audit emission + policy evaluation. ArgoCD's Application CRD sync hook is the canonical place: it fetches images per declared sources, verifies signatures via a cosign-verify policy attached to the Application, emits a sync-start + sync-complete audit-chain row per ADR-0263 §D.4 deploy-event class, and consults Cedar via the ArgoCD project's role configuration per ADR-0243.

### A.3 Named pressure: zero-handroll OpenTofu requires per-context modules for CI/CD primitives

Per `feedback_zero_handroll_opentofu_only_2026_05_20`, every µservice deployment for every context lands via OpenTofu (NOT Terraform) with no manual steps. Per-context modules under `microservices/cloud-iac/modules/<context>/` per ADR-0339 are the canonical IaC home. A CI/CD substrate selection that ships as anything other than per-context OpenTofu modules violates the zero-handroll posture: it would require either hand-authored Helm CLI invocations or manual JCasC plugin installation, both of which drift from the declarative substrate. Jenkins + ArgoCD ship as OpenTofu modules per D-1 + D-2 below: the modules declare the K8s manifests, the ConfigMaps, the Service + Ingress, the persistent volumes for Jenkins job history, the ArgoCD CRD installations, and the per-context secret-injection topology (KMS-backed Secret resources via External Secrets Operator + OpenBao per ADR-0247-related secret-store doctrine). Per-tenant onboarding is `tofu apply` end-to-end.

### A.4 Named pressure: hyperscaler precedent for the Jenkins + ArgoCD pairing

Hyperscaler precedent for self-hostable CI/CD converges on the Jenkins + ArgoCD pairing for K8s-native production. Google's Anthos Config Management uses Config Sync (an ArgoCD-derivative) + Cloud Build (a Jenkins-class managed CI) for their hybrid + on-prem product. AWS's EKS-A (EKS-Anywhere) recommends Flux + Jenkins for the on-prem flavor; Oyatie evaluates Flux vs ArgoCD in §F.3 below and selects ArgoCD on talent-pool + UI-availability grounds. Microsoft's Azure Arc-enabled Kubernetes ships with Flux as the default CD; Microsoft's internal stack uses Jenkins for the parts of CI that pre-date Azure DevOps Pipelines and continues operating Jenkins for legacy + customer-facing on-prem deployments. RedHat OpenShift Pipelines ships Tekton, but RedHat customers operating on-prem widely use Jenkins via the canonical jenkinsci/jenkins helm chart; RedHat GitOps ships ArgoCD as the canonical CD substrate. SUSE Rancher ships Fleet as their CD but supports ArgoCD as a co-deployed alternative; Rancher's customer base for on-prem widely uses Jenkins for CI. Oracle's OCI offers a Jenkins-based DevOps service and explicitly supports ArgoCD on OKE (Oracle Kubernetes Engine) per OCI documentation. IBM Cloud Pak for Applications ships ArgoCD as the canonical GitOps CD and supports Jenkins. The convergence is unambiguous: Jenkins + ArgoCD is the hyperscaler-typical pair for K8s-native self-hostable CI/CD.

### A.5 Named pressure: substrate Contributor stewardship class is required for upstream-patch capacity

Per ADR-0345 OSS stewardship class policy, every OSS dependency is classified Maintainer / Contributor / Consumer with declared CVE-response SLAs. Jenkins and ArgoCD must be Contributor class rather than Consumer class because: (1) both substrates sit in the critical CI/CD path, so a P0 CVE in either substrate requires same-week patching capacity from Oyatie engineers rather than reliance on upstream LTS cadence alone; (2) both substrates have substantial plugin/extension ecosystems where Oyatie's CI matrix specifically depends on plugins (Jenkins) or controllers (ArgoCD ApplicationSet, ArgoCD Image Updater) that may require Oyatie-authored bug fixes; (3) the on-prem + air-gap deployment story is mission-critical for sovereign customers, so an upstream-bug-blocker that prevents on-prem deployment requires direct upstream engagement capacity. The Contributor class declares: P0 CVE SLA <= 7 days, P1 CVE SLA <= 30 days, dev-day budget per quarter for upstream patches. Per the ADR-0345 §D resourcing table, Jenkins gets 10 dev-days/quarter and ArgoCD gets 12 dev-days/quarter (slightly higher than Jenkins because ArgoCD's release cadence is faster and its plugin ecosystem is more centrally critical to the Oyatie GitOps story).

### A.6 Named pressure: per-microservice CI-parity prevents silent CI/CD drift

Without an enforcement lane, the canonical CI matrix (`.github/workflows/*.yml`) and the self-hosted Jenkinsfile (`microservices/<ms>/Jenkinsfile`) can drift over time. A developer adds a new CI step to `.github/workflows/pr-tests.yml` and forgets to update `microservices/<ms>/Jenkinsfile`; CI passes on the hosted-on-GitHub PR review surface; the air-gap customer's Jenkins controller does not run the new step, so a regression that would have been caught lands in the air-gap deployment. The named pressure is that the dual-substrate posture (GitHub Actions + Jenkins) creates a parity-drift surface that requires a static-analysis lane to enforce. The `oya-governance-jenkins-github-actions-parity` lane per E.1 below validates that for every CI step in `.github/workflows/<ms>-*.yml`, there is a corresponding stage in `microservices/<ms>/Jenkinsfile` with the same step semantics. The lane is REPORT-ONLY until Wave 15-ZE completes the per-microservice Jenkinsfile authoring; promotion to BLOCKER 30 days post-Wave-15-ZE-completion ensures every new CI step lands in both surfaces.

### A.7 Named pressure: per-tenant cluster isolation requires ArgoCD project namespaces gated by Cedar

Per ADR-0242 (oyatie-is-a-tenant), ADR-0244 (tenant scoping), and the Amazon-shape cellular architecture per ADR-0248, every workload runs in a tenant-scoped cell with shuffle-sharded blast-radius isolation. The ArgoCD per-cluster Application CRD by default ships without tenant isolation: a misconfigured Application could sync workloads across tenant namespaces, defeating the cellular isolation contract. The `oya-governance-argocd-tenant-namespace-isolation` lane per E.3 below validates that every ArgoCD Application is bound to a single tenant_id and that cross-tenant sync requires an explicit Cedar policy gate per ADR-0243. ArgoCD's native ApplicationSet + project + role-mapping primitives are the implementation mechanism; the lane is the static-analysis enforcer.

### A.8 Anchors this ADR binds

- Anchor 1: `feedback_jenkins_argocd_substrate_2026_05_21` — the user directive of 2026-05-21 captured as the canonical doctrine memory.
- Anchor 2: `feedback_multi_context_provider_agnostic_2026_05_20` — six-context deployment posture requires self-hostable CI/CD.
- Anchor 3: `feedback_zero_handroll_opentofu_only_2026_05_20` — CI/CD substrates ship as OpenTofu modules.
- Anchor 4: `feedback_oci_always_free_maximization_2026_05_20` — demo / trial / dev tenants on OCI Always Free run Jenkins + ArgoCD on Ampere A1 ARM nodes.
- Anchor 5: ADR-0028 (cloud microservice architecture) — substrate layer for the CI/CD primitives.
- Anchor 6: ADR-0181 (cosign-signed artifacts and modules) — ArgoCD enforces signature verification on every sync.
- Anchor 7: ADR-0211 (in-house tech stack preference) — Class C OSS approved; Jenkins + ArgoCD admitted.
- Anchor 8: ADR-0212 (buildability doctrine) — Jenkins exercises the build matrix in self-hosted contexts.
- Anchor 9: ADR-0215 (multi-context deployment) — Jenkins + ArgoCD run in every context.
- Anchor 10: ADR-0218 (OpenTofu not Terraform) — modules use OpenTofu syntax exclusively.
- Anchor 11: ADR-0221 (agentic-development-pipeline-hardening) — Jenkins runs are CI-gate authority.
- Anchor 12: ADR-0243 (Cedar as universal gate) — ArgoCD project role mapping consults Cedar.
- Anchor 13: ADR-0254 (Kubernetes-everywhere) — Jenkins + ArgoCD run as K8s pods.
- Anchor 14: ADR-0263 (observability emission contract) — ArgoCD sync emits audit-chain rows.
- Anchor 15: ADR-0339 (shared IaC module library) — module home `microservices/cloud-iac/modules/<context>/`.
- Anchor 16: ADR-0345 (OSS stewardship class) — Contributor class for both Jenkins + ArgoCD.
- Anchor 17: ADR-0164 (air-gap sovereign deployment) — both substrates run in air-gap.
- Anchor 18: `feedback_quality_performance_scalability_bar` — hyperscaler-grade CI/CD substrate.
- Anchor 19: `feedback_no_silent_regression` — parity lane refuses silent CI/CD drift.
- Anchor 20: `feedback_bominal_inheritance_precedence` — Bominal authors a sibling Jenkins+ArgoCD ADR.

### A.9 What this ADR does not assert

- **A.9.1** Does not retire GitHub Actions. GitHub Actions remains the primary CI for the hosted-on-GitHub PR review surface per `feedback_jenkins_argocd_substrate_2026_05_21` §"Relationship to existing CI/CD".
- **A.9.2** Does not author the OpenTofu modules, the per-microservice Jenkinsfile, the per-microservice Helm chart, the per-cluster apps.yaml, the cosign-verify policy, or the audit-chain emitter integration. All authoring is sequenced as Wave 15-ZE.
- **A.9.3** Does not retire `kubectl apply` as a tenant-operator break-glass mechanism under signed audit-chain emission. Emergency-deploy with audit trail remains available per ADR-0263 break-glass clause.
- **A.9.4** Does not retire Helm as a chart format. Only the manual Helm CLI deploy path is replaced; charts continue to be authored under `microservices/<ms>/iac/k8s/helm/`.
- **A.9.5** Does not introduce a new programming language or build tool. Jenkinsfile is declarative Groovy under `pipeline {}` syntax which is operationally familiar; JCasC is declarative YAML; ArgoCD ships pre-built and is operated via declarative Kubernetes CRDs.
- **A.9.6** Does not change the cosign + audit-chain + Cedar gate triad. It adds ArgoCD as the deploy-time runtime enforcer of the existing gates.
- **A.9.7** Does not change the K8s + Cloud Hypervisor + Kata pods runtime substrate per ADR-0254. Jenkins controllers + ArgoCD controllers run as pods on the same substrate.
- **A.9.8** Does not retire the canonical-primitives doctrine; `tools/hooks/_canonical-primitives.md` gains a CI/CD Substrates section pointing to Jenkins + ArgoCD.
- **A.9.9** Does not change branch-pipeline semantics (dev / staging / production per `project_branch_pipeline_implemented`); ArgoCD per-cluster Application CRDs reference the appropriate branch per cluster role.
- **A.9.10** Does not change Bominal corpus posture; Bominal authors its sibling rename ADR independently per `feedback_bominal_inheritance_precedence`.
- **A.9.11** Does not impose Jenkins X, Tekton, Flux, Spinnaker, or any SaaS CI as alternatives; each is rejected with named reasons in §F below.

## Decision

### B.1 Decision statement

Jenkins (LTS) is the canonical CI orchestrator for self-hostable CI in air-gap + on-prem + colo + oyatie-as-cloud-provider deployment contexts. Jenkins runs on Oyatie K8s per ADR-0254 with configuration-as-code via JCasC + per-microservice Jenkinsfile pipelines mirroring the `.github/workflows/*.yml` CI matrix one-to-one per the `oya-governance-jenkins-github-actions-parity` lane. GitHub Actions is RETAINED as the primary CI for the hosted-on-GitHub PR review surface (oya verify + PR checks per ADR-0346). ArgoCD is the canonical GitOps CD orchestrator. ArgoCD synchronizes declared state from `clusters/<cluster-id>/apps.yaml` to per-cluster K8s with per-microservice Helm chart sources at `microservices/<ms>/iac/k8s/helm/`. Every ArgoCD Application sync verifies cosign signatures per ADR-0181 on the fetched images, emits an audit-chain row per ADR-0263 deploy-event class, and consults Cedar per ADR-0243 for cross-tenant access. Both substrates are OSS Class C approved per ADR-0211 with Contributor-class stewardship per ADR-0345 (Jenkins: 10 dev-days/quarter; ArgoCD: 12 dev-days/quarter; both: 7-day P0 + 30-day P1 CVE SLA). Both are provisioned via OpenTofu modules under `microservices/cloud-iac/modules/<context>/jenkins/` and `/<context>/argocd/` per ADR-0339 across all six deployment contexts including air-gap. Five new CI lanes enforce the substrate contract per §E below. Wave 15-ZE sequences the executor PRs under ADR-0328 batch discipline.

### B.2 Numbered decision clauses

B2.001. Jenkins (LTS) is the canonical CI orchestrator for self-hostable CI in the four contexts where GitHub Actions runners are not available or not appropriate: air-gap (per ADR-0164), on-prem, colo, and oyatie-as-cloud-provider.

B2.002. Jenkins runs on Oyatie K8s as the canonical deploy target per ADR-0254. The Jenkins controller pod runs under the Cloud Hypervisor + Kata pods substrate; the Jenkins build agents run as ephemeral K8s pods (one pod per pipeline execution) via the Jenkins Kubernetes plugin.

B2.003. Jenkins configuration is config-as-code via JCasC (Jenkins Configuration as Code) plugin. Controller state is authored declaratively under `microservices/cloud-iac/modules/<context>/jenkins/jcasc/` and applied by the OpenTofu module at controller boot. The UI is read-only for state changes per the `oya-governance-jenkins-jcasc-only` lane per E.4.

B2.004. Per-microservice CI pipelines are authored as `microservices/<ms>/Jenkinsfile` and mirror the corresponding `.github/workflows/<ms>-*.yml` matrix one-to-one per the `oya-governance-jenkins-github-actions-parity` lane per E.1. The mirroring contract is at the step level: every named step in the workflow has a corresponding stage in the Jenkinsfile.

B2.005. GitHub Actions is RETAINED as the primary CI for the hosted-on-GitHub PR review surface. PR check semantics per ADR-0221 + ADR-0346 are preserved verbatim. Jenkins augments GitHub Actions for self-hosted contexts; it does not replace it on the hosted-on-GitHub surface.

B2.006. ArgoCD is the canonical GitOps CD orchestrator. ArgoCD runs as a K8s-native controller in every cluster; one ArgoCD installation per cluster.

B2.007. ArgoCD synchronizes declared state from `clusters/<cluster-id>/apps.yaml` (the per-cluster GitOps manifest authored in the canonical GitOps repo) to per-cluster K8s. The manifest declares which microservices deploy to that cluster via ArgoCD `Application` CRDs.

B2.008. Per-microservice Helm charts live at `microservices/<ms>/iac/k8s/helm/`. ArgoCD Application CRDs reference these chart paths as their sources.

B2.009. Every ArgoCD Application sync verifies cosign signatures per ADR-0181 on every image fetched. The cosign-verify policy attaches to the Application CRD via the `oya-governance-argocd-application-cosign-verified` lane per E.2.

B2.010. Every ArgoCD sync transition (sync-start, sync-complete, sync-failed) emits an audit-chain row per ADR-0263 deploy-event class. The audit emitter integrates as a sync hook (ArgoCD Resource Hook lifecycle phase: PreSync, Sync, PostSync, SyncFail).

B2.011. ArgoCD enforces tenant-namespace isolation via ArgoCD projects scoped to tenant_id. Cross-tenant Application authoring is refused by the `oya-governance-argocd-tenant-namespace-isolation` lane per E.3 unless an explicit Cedar policy gate grants cross-tenant access per ADR-0243.

B2.012. Both substrates are OSS Class C approved per ADR-0211; both are added to the canonical dependency list at `docs/standards/dependency-policy.md` with the Class C designation.

B2.013. Both substrates are Contributor-class per ADR-0345 OSS stewardship class policy. The OSS stewardship registry at `/specs/oss-stewardship-registry.json` adds two entries per ADR-0345 schema: dep_name + stewardship_class=Contributor + owner_team + CVE SLA + resourcing + license + source_url + adr_provenance + mitigation_strategies + notes.

B2.014. Jenkins stewardship resourcing: 10 dev-days/quarter dedicated to upstream-patch capacity; 7-day P0 + 30-day P1 CVE SLA; owner team `ops-platform + ops-sre-reliability`.

B2.015. ArgoCD stewardship resourcing: 12 dev-days/quarter dedicated to upstream-patch capacity; 7-day P0 + 30-day P1 CVE SLA; owner team `ops-sre-reliability + axis-observability + council-security`.

B2.016. Both substrates are provisioned via OpenTofu modules per ADR-0218 + ADR-0339. Module homes: `microservices/cloud-iac/modules/<context>/jenkins/` and `microservices/cloud-iac/modules/<context>/argocd/` for each of the six contexts: aws-guest, oci-guest, on-prem, colo, oyatie-as-provider, air-gap. Twelve modules total at landing time.

B2.017. The air-gap modules carry an additional `airgap_mirror_registry` input that points to the tenant-internal OCI registry mirror; Jenkins + ArgoCD images are mirrored to the tenant-internal registry pre-deploy per the canonical air-gap mirror flow per ADR-0164.

B2.018. Per-microservice Jenkinsfile authoring is a Wave 15-ZE deliverable; the file count is ~77 (one per microservice).

B2.019. Per-microservice Helm chart authoring is a Wave 15-ZE deliverable; the chart count is ~77 (one per microservice).

B2.020. The per-cluster `clusters/<cluster-id>/apps.yaml` manifest authoring is a Wave 15-ZE deliverable; the cluster count varies per tenant + per environment, but the canonical first-deliverable set per FD-001 per `/specs/master-plan-sequencing.json` is ~6 clusters (dev / staging / production × control + workload).

B2.021. The cosign-verify policy attached to every ArgoCD Application is authored as an ArgoCD-specific policy template under `microservices/cloud-iac/modules/<context>/argocd/policies/cosign-verify/` and referenced from every Application CRD.

B2.022. The audit-chain emitter integration at the ArgoCD sync hook is authored as a Rust binary (per `feedback_rust_strict_only_no_python_2026_05_20`) packaged as a K8s Job invoked from the ArgoCD Resource Hook lifecycle. The binary emits per ADR-0263 §D.4 deploy-event class.

B2.023. Forbidden alternatives are enumerated in §F below. Jenkins X, Tekton, Flux CD, Spinnaker, CircleCI / Travis / Buildkite SaaS, and GitHub Actions on customer self-hosted runners pointed at GitHub.com control plane are each rejected with named reasons.

B2.024. Backup + DR for Jenkins + ArgoCD per ADR-0343 DR matrix: Jenkins controller state + job history backup to per-tenant OCI / S3-compatible object storage every 4 hours; restore-time-objective <= 1 hour; restore-point-objective <= 4 hours. ArgoCD declarative state lives in git (canonical source of truth); ArgoCD live-state cache is rebuildable from git; restore-time-objective <= 30 minutes; restore-point-objective effectively 0 (git is the source).

B2.025. Cell-tier assignment per ADR-0341: Jenkins controller is Tier 2 (Critical) by default; ArgoCD controller is Tier 1 (Mission-Critical) by default reflecting the higher blast-radius of a CD failure relative to a CI failure.

B2.026. API versioning per ADR-0342: not directly applicable (neither substrate exposes a tenant-facing API surface in the Oyatie BNF v4 sense); upstream API versioning (Jenkins API v2; ArgoCD API v1alpha1, v1beta1, v1) is consumed verbatim with declared compatibility windows in the per-module README.

B2.027. Sustainability + finops per ADR-0344: Jenkins controller + agent pods + ArgoCD controller pod costs decompose into the per-microservice dimensional model under the `cloud-iac` microservice's cost dimension. Per-tenant Jenkins + ArgoCD costs are attributed to the tenant per ADR-0244.

B2.028. The five new lanes start REPORT-ONLY at this ADR's Acceptance per E.1..E.5 below.

B2.029. The five new lanes promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZE-completion per H.1 below. The 30-day window aligns with sibling-ADR sunset windows (ADR-0344 + ADR-0345 + ADR-0347).

B2.030. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. Review evidence at `evidence/debate/ADR-0349/<facet>.md` after this ADR lands in a review-track PR.

B2.031. The Wave 15-ZE entry is added to `/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves` as part of this ADR's required-artifact contract.

B2.032. The OSS stewardship registry at `/specs/oss-stewardship-registry.json` gains the two new Contributor-class entries for Jenkins + ArgoCD as part of this ADR's required-artifact contract. Entry authoring follows the ADR-0345 schema.

B2.033. The `tools/hooks/_canonical-primitives.md` cheat sheet gains a CI/CD Substrates section enumerating: GitHub Actions (primary CI for hosted-on-GitHub), Jenkins (LTS) (canonical CI for self-hostable contexts), ArgoCD (canonical GitOps CD for all contexts).

B2.034. The `docs/standards/dependency-policy.md` gains Jenkins + ArgoCD declarations in the Class C OSS substrate table with Contributor-class stewardship + license info (Jenkins MIT; ArgoCD Apache 2.0) + canonical-version pinning (Jenkins LTS-2.x latest; ArgoCD v2.x latest).

B2.035. Sub-wave dispatch follows ADR-0328 batch discipline: Wave 15-ZE is fan-out across the twelve OpenTofu modules + ~77 Jenkinsfiles + ~77 Helm charts under the 11-agent dispatch ceiling per `feedback_dispatch_ceiling_claude_only_2026_05_20`.

B2.036. The ADR explicitly preserves the branch-pipeline semantics per `project_branch_pipeline_implemented`. Per-cluster ArgoCD Applications reference `dev` / `staging` / `production` branches as appropriate per the canonical branch-pipeline contract.

B2.037. The ADR explicitly preserves the OCI Always Free maximization posture per `feedback_oci_always_free_maximization_2026_05_20`. Jenkins + ArgoCD on the OCI deployment profile run on Ampere A1 ARM nodes (2 OCPU + 12 GB each for the Jenkins controller; 1 OCPU + 6 GB for ArgoCD controller within the Always Free tier).

B2.038. The ADR explicitly preserves the OS support matrix per `feedback_os_support_matrix_2026_05_20`. Jenkins + ArgoCD images target linux/amd64 + linux/arm64 multi-arch; on Talos / RHEL / Oracle Linux / SUSE / Ubuntu LTS / Debian / Rocky / AlmaLinux / CentOS Stream / Amazon Linux / Flatcar / Photon OS the images run identically as K8s pods.

B2.039. The ADR is final on Acceptance. No exception clause is provided for skipping Jenkins or ArgoCD in any of the six contexts; the substrate contract is uniform.

B2.040. The Bominal parallel corpus authors its sibling Jenkins + ArgoCD ADR independently per `feedback_bominal_inheritance_precedence`. No Oyatie-side enforcement applies to Bominal.

### B.3 What this decision does not do

- This ADR does not author the OpenTofu modules, the per-microservice Jenkinsfile, the per-microservice Helm chart, the per-cluster apps.yaml, the cosign-verify policy, or the audit-chain emitter integration; Wave 15-ZE does.
- This ADR does not retire GitHub Actions; GitHub Actions remains primary CI for hosted-on-GitHub PR review.
- This ADR does not retire `kubectl apply` for break-glass tenant-operator use under signed audit-chain emission.
- This ADR does not retire Helm as a chart format; only manual Helm CLI deploys are replaced.
- This ADR does not change the cosign + audit-chain + Cedar gate triad; it adds the deploy-time runtime enforcer.
- This ADR does not change the K8s-everywhere runtime substrate per ADR-0254.
- This ADR does not introduce a new programming language or build tool beyond the operationally familiar Jenkinsfile Groovy + JCasC YAML + ArgoCD CRDs.

## Consequences

### C.1 Positive consequences

- **Air-gap + on-prem + colo + oyatie-as-cloud-provider contexts gain a canonical CI substrate.** Jenkins (LTS) is hyperscaler-typical for self-hostable CI; talent pool is wide; CVE response is mature; on-prem operability is proven.
- **Manual kubectl + Helm CLI deploys retire across all six contexts.** ArgoCD becomes the canonical deploy mediator; cosign + audit-chain + Cedar gates are deploy-time enforceable.
- **Per-tenant onboarding is `tofu apply` end-to-end.** Twelve OpenTofu modules cover every (context × substrate) pair; zero handroll per `feedback_zero_handroll_opentofu_only_2026_05_20`.
- **Hyperscaler precedent matched.** Google Anthos Config Management, AWS EKS-A, Microsoft Azure Arc, RedHat GitOps, SUSE Rancher, Oracle OCI DevOps, IBM Cloud Pak all use Jenkins + ArgoCD (or equivalents) for self-hostable CI/CD.
- **CI parity refuses silent CI/CD drift.** The `oya-governance-jenkins-github-actions-parity` lane enforces step-level parity between GitHub Actions + Jenkinsfile.
- **Cosign + audit-chain + Cedar discipline reaches the deploy surface.** ArgoCD sync hooks enforce all three; no path to deploy bypasses the gates.
- **Per-tenant cluster isolation is auditable.** ArgoCD projects + Cedar role mapping + the tenant-namespace-isolation lane.
- **Contributor-class stewardship pre-positions upstream-patch capacity.** Jenkins gets 10 dev-days/quarter; ArgoCD gets 12 dev-days/quarter; P0 CVE response within 7 days; P1 within 30 days.
- **OCI Always Free maximization preserved.** Both substrates fit within the OCI Always Free Ampere A1 ARM resource envelope for demo / trial / dev tenants.
- **Bominal parallel corpus inherits the pattern.** Sibling ADR in Bominal carries the same Jenkins + ArgoCD pairing.
- **Foundry retirement (ADR-0335) reinforced.** With Jenkins + ArgoCD as the canonical CI/CD substrate, the legacy "foundry pipeline" terminology is fully absorbed into intelligence + governance ownership; no need for a separate foundry pipeline shape.

### C.2 Negative consequences

- **Two substrate primitives to operate.** Jenkins controller + agents + ArgoCD controller add operational surface per cluster. Mitigation: both are mature OSS with well-understood operability; Oyatie's hyperscaler-precedent talent assumption per `feedback_quality_performance_scalability_bar` covers operability competence.
- **Per-microservice Jenkinsfile + Helm chart authoring is large.** ~77 microservices × 2 artifacts = ~154 files at Wave 15-ZE. Mitigation: codex-bucket fan-out under ADR-0328 batch discipline; per-microservice templates exist (the existing `.github/workflows/<ms>-*.yml` is the substantive template).
- **CI-parity lane requires step-level static analysis.** The lane parses both Jenkinsfile Groovy + GitHub Actions YAML to verify step parity. Mitigation: the lane reuses existing ast-grep tooling per `mcp__plugin_oh-my-claudecode_t__ast_grep_search` capability; the parity contract is well-defined at the step name + arguments level.
- **ArgoCD ApplicationSet authoring is per-tenant.** Per-tenant cluster onboarding requires per-tenant Application CRD authoring. Mitigation: ApplicationSet generators (List, Cluster, Git, Matrix, Merge) automate per-tenant fan-out; per-tenant declarations are JSON / YAML, not hand-authored CRDs.
- **Jenkins controller persistent volume is a tenant-data surface.** Job history may contain build logs that include tenant-affecting information (e.g., secret error messages). Mitigation: per ADR-0263 + ADR-0244, job logs are scoped to tenant_id; cross-tenant log leakage is refused by Cedar gates.
- **ArgoCD sync hook latency adds to deploy time.** The cosign-verify + audit-chain emit + Cedar evaluation steps add ~5-15 seconds per Application sync. Mitigation: the latency is bounded; deploy-time latency is acceptable in exchange for the gate enforcement.
- **Contributor-class dev-day budget commits Oyatie engineering capacity.** 10+12 = 22 dev-days/quarter across both substrates. Mitigation: pre-budgeted under ADR-0345 §D resourcing aggregate; budget is sustainable at the Oyatie engineering scale.
- **Operationally heavier than tekton-only alternative.** Tekton-only stack is leaner. Mitigation: tekton's lower talent pool + lacking UI is the stronger constraint; §F.2 below carries the rejected-alternative rationale.

### C.3 Neutral consequences

- **K8s + Cloud Hypervisor + Kata pods runtime substrate unchanged.** Per ADR-0254.
- **GitHub Actions unchanged for hosted-on-GitHub PR review.** Per ADR-0221 + ADR-0346.
- **Helm as chart format unchanged.** Only manual CLI deploys replaced.
- **Cosign + audit-chain + Cedar gate semantics unchanged.** Only the deploy-time enforcer added.
- **Branch-pipeline semantics unchanged.** Per `project_branch_pipeline_implemented`.
- **OS support matrix unchanged.** Per `feedback_os_support_matrix_2026_05_20`.
- **OCI Always Free maximization unchanged.** Per `feedback_oci_always_free_maximization_2026_05_20`.
- **Cellular promotion gates unchanged.** Per ADR-0341.
- **DR matrix per microservice unchanged.** Per ADR-0343; this ADR adds explicit RTO/RPO for the two substrates.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Twelve OpenTofu modules under per-context structure | Wave 15-ZE modules land; per-context directory tree complete |
| Reliability | RTO/RPO matrix per ADR-0343 declared for both substrates | RTO <= 1 hour Jenkins; RTO <= 30 minutes ArgoCD; RPO <= 4 hours Jenkins; RPO ~ 0 ArgoCD |
| Security | Cosign verify on every ArgoCD sync; Cedar gate on cross-tenant; audit-chain emit on every transition | E.2 + E.3 + E.5 lanes green |
| Compliance | SOC2 + ISO 27001 evidence chain extends to deploy-time enforcer | Evidence packs at `evidence/audit/ci-cd-substrate/` reference ADR-0349 |
| Operability | JCasC + Jenkinsfile config-as-code; ArgoCD declarative CRDs | E.4 lane green; UI changes refused |
| Performance | Sync-hook latency ~5-15s bounded | ArgoCD sync metrics observable per ADR-0263 |
| Scalability | ArgoCD scales to thousands of Applications per cluster; Jenkins scales via ephemeral K8s agent pods | Hyperscaler precedent confirmed (per A.4) |
| Substance-bar | Per-substrate rationale + per-context module enumeration + per-microservice CI parity | ADR-0322 lane green |
| Hyperscaler alignment | Jenkins + ArgoCD is the convergent hyperscaler choice for K8s-native self-hostable CI/CD | Google + AWS + Microsoft + RedHat + SUSE + Oracle + IBM precedents named in A.4 |
| Stewardship | Contributor class declared with dev-day budget + CVE SLA | ADR-0345 registry entries authored under this ADR |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** Google Anthos Config Management uses Config Sync (ArgoCD-derivative) + Cloud Build (Jenkins-class managed CI) for hybrid + on-prem. AWS EKS-A recommends Flux + Jenkins for the on-prem flavor (Oyatie selects ArgoCD over Flux per §F.3 rationale). Microsoft Azure Arc-enabled Kubernetes ships Flux as default CD; Microsoft's internal stack uses Jenkins for legacy + customer-facing on-prem. RedHat GitOps ships ArgoCD as canonical CD; RedHat customers operating on-prem widely use Jenkins via jenkinsci/jenkins helm chart. SUSE Rancher ships Fleet as default CD but supports ArgoCD as co-deployed alternative; Rancher's on-prem customer base widely uses Jenkins. Oracle OCI offers Jenkins-based DevOps service + supports ArgoCD on OKE. IBM Cloud Pak for Applications ships ArgoCD as canonical GitOps CD + supports Jenkins. Cross-hyperscaler convergence on Jenkins + ArgoCD for K8s-native self-hostable CI/CD is unambiguous; Oyatie's selection aligns with the convergent precedent per `feedback_quality_performance_scalability_bar`.

**Failure-mode tree.** Failure modes:
(1) Jenkins controller pod crashes → K8s reschedules; persistent volume preserves job history; RTO <= 1 hour per ADR-0343.
(2) ArgoCD controller pod crashes → K8s reschedules; declarative state in git rebuilds the live-state cache; RTO <= 30 minutes; RPO ~ 0.
(3) Jenkins agent pod crashes mid-pipeline → pipeline marked FAILED in Jenkins UI; developer retries; no persistent state loss.
(4) Cosign verification fails on ArgoCD sync → sync refused; audit-chain row emitted with failed-verify event; ops alerted per ADR-0263.
(5) Cedar evaluation fails on cross-tenant ArgoCD Application → Application refused; audit-chain row emitted; lane E.3 surfaces in static analysis pre-sync.
(6) Audit-chain emitter Job pod fails post-sync → ArgoCD sync still completes but the audit-chain row is lost; mitigation: emitter retries via Kubernetes Job backoffLimit + dead-letter queue per ADR-0263 §D.5.
(7) Per-microservice Jenkinsfile drifts from .github/workflows/<ms>-*.yml → lane E.1 surfaces drift; CI refuses merge.
(8) Per-microservice Helm chart authoring incomplete at Wave 15-ZE → microservice cannot deploy via ArgoCD; manual kubectl break-glass available under signed audit-chain emission.
(9) Per-cluster apps.yaml authoring incomplete → ArgoCD has no Applications to sync; cluster idle until apps.yaml lands.
(10) Air-gap mirror registry unreachable → Jenkins + ArgoCD pod pulls fail; mitigation: pre-cached images per ADR-0164 air-gap mirror flow.

**Capacity math.** Per cluster: Jenkins controller pod ~2 CPU + 4 GiB; ArgoCD controller pod ~1 CPU + 2 GiB; Jenkins agent pods ephemeral ~1-4 CPU + 2-8 GiB each during pipeline execution. OCI Always Free (Ampere A1 ARM 4 OCPU + 24 GiB) easily accommodates both substrates plus several concurrent pipeline runs. Per-tenant cost decomposition: Jenkins + ArgoCD aggregate ~$50-150/month per cluster on AWS / OCI guest contexts; on-prem + colo + air-gap costs are tenant-owned. Cosign-verify latency per Application sync: ~2-5 seconds. Audit-chain emit latency: ~1-2 seconds. Cedar evaluation latency: ~50-200ms. Aggregate sync-hook latency: ~3-7 seconds bounded.

**Observability hooks.** Per ADR-0263. The audit-chain emitter integration emits the deploy-event class per ADR-0263 §D.4 with fields: tenant_id, microservice_id, cluster_id, image_digest, sync_phase ∈ {PreSync, Sync, PostSync, SyncFail, SyncSuccess}, cosign_verify_outcome ∈ {pass, fail, skip}, cedar_evaluation_outcome ∈ {allow, deny}, sync_duration_ms, sync_initiator (ArgoCD), git_commit_sha, helm_chart_version, deployment_target_namespace. Jenkins pipeline executions emit similar audit-chain rows with phase ∈ {pipeline_start, stage_start, stage_complete, pipeline_complete, pipeline_failed}. Both surfaces feed the canonical observability substrate at `microservices/observability/` per ADR-0130.

**Rollback path.** Per-substrate rollback: revert the OpenTofu module changes per `tofu plan` + `tofu apply` with prior state; both controllers redeploy with prior config. Per-Application rollback: ArgoCD's built-in `argocd app rollback <app-name> <revision>` rolls a sync back to a prior revision; the rollback emits an audit-chain row. Per-Wave-15-ZE rollback: revert the executor PRs; manual kubectl break-glass available under audit emission.

**Multi-region awareness.** Per ADR-0240 + ADR-0248 cellular architecture. ArgoCD per-cluster scoping means each region has its own ArgoCD controller; cross-region sync coordination is not directly an ArgoCD concern. Multi-region tenants run ArgoCD per region with per-region `clusters/<cluster-id>/apps.yaml` declarations.

**Sovereign-cell awareness.** Sovereign cells (HIPAA / GDPR-strict / CSAP / PCI / IL5) inherit the substrate posture. Per-pack additional constraints: HIPAA cells require BAA-signed image registries (the air-gap mirror or BAA-compliant guest-on-aws ECR); GDPR-strict cells require EU-region-only image fetch (ArgoCD source registries pinned to EU regions); CSAP cells require Korean-region image fetch; PCI cells require PCI-DSS-compliant image registries.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. Jenkins LTS canonical version pinning at the next-major LTS release boundary (currently Jenkins 2.x LTS); upgrade cadence aligns with the upstream LTS release cadence (quarterly major; monthly patch). ArgoCD canonical version pinning at v2.x latest; upgrade cadence aligns with the upstream release cadence (monthly minor; weekly patch). Schema deprecation per ADR-0342 hybrid (date for tenant-facing APIs, semver for internal SDK). Substrate-version pinning declared in `docs/standards/dependency-policy.md` Class C OSS substrate table.

## D. Detailed mechanics — twelve adoption surfaces (D-1..D-12)

The Jenkins + ArgoCD substrate adoption touches twelve surfaces in the corpus: the two OpenTofu module families, the per-microservice CI-parity contract, the per-microservice Helm chart, the per-cluster GitOps manifest, the cosign-verify policy, the audit-chain emitter, the OSS stewardship registry, the dependency-policy declaration, the canonical-primitives cheat sheet, the per-tenant cluster isolation, the DR/backup contract, and the forbidden alternatives. Subsections D-1 through D-12 enumerate each surface. Numbering is normative and corresponds to step identifiers referenced from B.2.

### D-1: Jenkins self-hosted via OpenTofu module at microservices/cloud-iac/modules/<context>/jenkins/

D-1.1. The Jenkins substrate is provisioned via OpenTofu modules under `microservices/cloud-iac/modules/<context>/jenkins/` per ADR-0339 shared IaC module library. One module per deployment context: aws-guest, oci-guest, on-prem, colo, oyatie-as-provider, air-gap. Six modules at landing time.

D-1.2. Each module declares: a K8s Namespace for Jenkins; a Deployment + StatefulSet for the Jenkins controller pod; a Service + Ingress for the controller HTTP/HTTPS endpoint; a PersistentVolumeClaim for `/var/jenkins_home` (controller state + job history); ConfigMaps for JCasC declarative configuration; Secrets sourced via External Secrets Operator (per OpenBao-based secret store doctrine); ServiceAccount + Role + RoleBinding for the Jenkins controller to spawn build agent pods; NetworkPolicy restricting controller egress per cell tenancy.

D-1.3. JCasC YAML lives under `microservices/cloud-iac/modules/<context>/jenkins/jcasc/` and declares: authorization strategy (matrix-based; tenant_id scoped); credentials providers (External Secrets Operator-backed); seed jobs that scan the Oyatie GitOps repo for `microservices/<ms>/Jenkinsfile` and create per-microservice pipeline jobs; LDAP / OIDC SSO integration per the cluster's identity provider; Jenkins plugins pinned to specific versions (Kubernetes plugin, JCasC plugin, Pipeline plugin, Pipeline Multibranch plugin, Git plugin, Credentials plugin, Audit Trail plugin, Role-Based Authorization Strategy plugin).

D-1.4. The controller pod's resource requests: 2 CPU + 4 GiB by default; tuned per cluster tier per ADR-0341.

D-1.5. The controller's persistent volume size: 50 GiB by default; tuned per cluster tier; backup every 4 hours per ADR-0343 to the per-tenant OCI / S3-compatible object store.

D-1.6. Build agent pods are ephemeral K8s pods spawned by the Jenkins Kubernetes plugin; one pod per pipeline execution; agent pod images are mirrored to the per-context image registry (air-gap mirror in air-gap context).

D-1.7. The air-gap module variant carries an additional `airgap_mirror_registry` input pointing to the tenant-internal OCI registry mirror. Jenkins controller image + agent images + plugin install sources are all sourced from the mirror.

D-1.8. The module emits an OpenTofu output `jenkins_controller_url` that downstream modules consume to register webhook receivers + audit-chain emit targets.

### D-2: ArgoCD self-hosted via OpenTofu module at microservices/cloud-iac/modules/<context>/argocd/

D-2.1. The ArgoCD substrate is provisioned via OpenTofu modules under `microservices/cloud-iac/modules/<context>/argocd/` per ADR-0339. Six modules at landing time (one per context).

D-2.2. Each module declares: a K8s Namespace `argocd` (or per-tenant `argocd-<tenant_id>` for tenant-scoped controllers); the ArgoCD controllers (application-controller, server, repo-server, redis, dex-server, applicationset-controller, notifications-controller); RBAC + ServiceAccount + Role + RoleBinding; Service + Ingress for the ArgoCD UI + API endpoint; ConfigMaps for cluster registration; Secrets for repo credentials (sourced via External Secrets Operator); ApplicationSet controllers for per-tenant fan-out; the cosign-verify policy template; the audit-chain emitter Job template.

D-2.3. ArgoCD is installed via the canonical ArgoCD Helm chart `argo/argo-cd` pinned to the latest v2.x release per dependency-policy.md.

D-2.4. The controller pods' resource requests: application-controller ~500m CPU + 1 GiB; server ~250m CPU + 512 MiB; repo-server ~250m CPU + 512 MiB; redis ~100m CPU + 128 MiB. Aggregate ~1.1 CPU + 2.2 GiB per ArgoCD installation.

D-2.5. The ArgoCD repo-server is configured to fetch from the Oyatie GitOps repo (per-tenant Helm chart references at `microservices/<ms>/iac/k8s/helm/` + per-cluster apps.yaml at `clusters/<cluster-id>/apps.yaml`).

D-2.6. The ArgoCD project structure: one project per tenant_id; projects scope which Applications can sync to which namespaces. Cross-tenant project authoring is refused by the `oya-governance-argocd-tenant-namespace-isolation` lane per E.3.

D-2.7. The ApplicationSet controller automates per-tenant + per-cluster fan-out via the `cluster` + `list` + `git` + `matrix` + `merge` generators.

D-2.8. The air-gap module variant carries the `airgap_mirror_registry` input pointing to the tenant-internal OCI registry mirror; ArgoCD controller image + cosign-verify policy image + audit-chain emitter image are mirrored.

D-2.9. The module emits OpenTofu outputs `argocd_server_url` + `argocd_project_namespace` consumed by per-tenant onboarding flows.

### D-3: Per-microservice Jenkinsfile mirrors .github/workflows/*.yml CI matrix; lane-parity enforced

D-3.1. Every microservice at `microservices/<ms>/` ships a `Jenkinsfile` declaring the per-microservice CI pipeline. The Jenkinsfile uses declarative pipeline syntax (`pipeline {}`).

D-3.2. The Jenkinsfile mirrors the corresponding `.github/workflows/<ms>-*.yml` workflow files one-to-one. Mirror contract: every named step in the workflow has a corresponding stage in the Jenkinsfile with the same name + the same shell-command-equivalent semantics.

D-3.3. The `oya-governance-jenkins-github-actions-parity` lane per E.1 validates the parity by static analysis. The lane parses both the GitHub Actions YAML + the Jenkinsfile Groovy and confirms step-level parity. Drift is refused.

D-3.4. Mirror granularity is at the named-step level, not at the action-implementation level. GitHub Actions native actions (e.g., `actions/checkout`) map to Jenkinsfile equivalents (e.g., `checkout scm`) per a canonical translation table maintained under `docs/standards/jenkins-github-actions-parity.md` (authored as part of Wave 15-ZE).

D-3.5. Per-microservice Jenkinsfile authoring is queued as Wave 15-ZE; ~77 microservices × 1 Jenkinsfile = ~77 files.

D-3.6. Jenkinsfile pipelines invoke the same `oya verify --ci-required` entrypoint per ADR-0346; the verifier already mirrors the full CI matrix per ADR-0346's full-CI-mirror clause.

D-3.7. Jenkinsfile pipelines emit per-stage audit-chain rows per ADR-0263 (pipeline_start, stage_start, stage_complete, pipeline_complete, pipeline_failed).

D-3.8. Pipeline trigger sources: webhook from the canonical GitOps repo (per ADR-0112 webhook-driven foundry agent invocation, now governance-pipeline-driven post ADR-0335); manual via the Jenkins UI + CLI for ops break-glass.

### D-4: Per-microservice Helm chart at microservices/<ms>/iac/k8s/helm/; ArgoCD Application CRD references

D-4.1. Every microservice at `microservices/<ms>/` ships a Helm chart at `microservices/<ms>/iac/k8s/helm/`. The chart declares the microservice's K8s manifests parameterized by per-cluster + per-tenant values.

D-4.2. Chart structure: `Chart.yaml` (chart metadata; version semver per ADR-0342); `values.yaml` (default values); `values-<context>.yaml` (per-context overrides); `templates/` (K8s manifest templates including Deployment, Service, Ingress, HPA, NetworkPolicy, ConfigMap, ServiceAccount, ServiceMonitor for Prometheus, PodDisruptionBudget, PriorityClass per cell tier per ADR-0341).

D-4.3. Per-microservice Helm chart authoring is queued as Wave 15-ZE; ~77 microservices × 1 chart = ~77 charts.

D-4.4. ArgoCD `Application` CRDs reference the chart paths via the `source.repoURL` + `source.path` + `source.helm.valueFiles` fields. Per-cluster Application CRDs are declared in `clusters/<cluster-id>/apps.yaml` per D-5.

D-4.5. Helm chart linting + schema validation is part of the Jenkinsfile + GitHub Actions CI matrix (running `helm lint` + `helm template` against the chart per cluster context).

D-4.6. Chart versioning aligns with microservice version semver per ADR-0342; major bumps emit per ADR-0108 sunset notifications.

### D-5: Per-cluster clusters/<cluster-id>/apps.yaml is the canonical GitOps manifest

D-5.1. The canonical per-cluster GitOps manifest is `clusters/<cluster-id>/apps.yaml` in the canonical GitOps repo. The manifest declares which microservices deploy to that cluster via ArgoCD `Application` CRDs (or ArgoCD `ApplicationSet` CRDs for per-tenant fan-out).

D-5.2. The manifest structure: list of Application CRDs; each Application declares `metadata.name` (cluster_id + microservice_id composition); `spec.project` (tenant_id-scoped project); `spec.source.repoURL` (the canonical GitOps repo); `spec.source.path` (`microservices/<ms>/iac/k8s/helm/`); `spec.source.targetRevision` (branch or tag per branch-pipeline contract); `spec.source.helm.valueFiles` (per-context values); `spec.destination.server` (the cluster's API server URL); `spec.destination.namespace` (per-tenant namespace); `spec.syncPolicy` (automated sync with prune + selfHeal + retry per ArgoCD canonical syncPolicy).

D-5.3. ArgoCD watches the manifest path via the ApplicationSet `git` generator; new Applications added to the manifest auto-create on ArgoCD; removed Applications auto-delete after grace period.

D-5.4. Per-cluster apps.yaml authoring is queued as Wave 15-ZE for the canonical first-deliverable cluster set per FD-001 (~6 clusters at landing time).

D-5.5. Per-tenant onboarding emits a new `clusters/<tenant-id>-<env>/apps.yaml` file via the `tofu apply` flow per D-1 + D-2.

D-5.6. The manifest is the canonical source of truth for "which microservices run on which cluster"; the residual K8s live state is the projection. ArgoCD's selfHeal posture continuously reconciles drift.

### D-6: Cosign signature verification on every ArgoCD-deployed image per ADR-0181

D-6.1. Every image referenced by every ArgoCD Application is cosign-verified before sync. The verification policy is a cosign verification policy attached to the ArgoCD Application CRD via the `spec.source.helm.fileParameters` field or via a per-cluster admission controller (Kyverno per ADR-0181 + dependency-policy.md).

D-6.2. The cosign-verify policy template is authored under `microservices/cloud-iac/modules/<context>/argocd/policies/cosign-verify/` per context.

D-6.3. The policy declares: the cosign public key reference (per the canonical KMS-backed cosign signing key per ADR-0181); the verify-rekor URL (per the canonical Rekor instance); the allowed signing identities (per the canonical CI/CD signing pipeline identity).

D-6.4. Failed verification refuses the sync; an audit-chain row emits with phase=SyncFail + cosign_verify_outcome=fail per ADR-0263.

D-6.5. The `oya-governance-argocd-application-cosign-verified` lane per E.2 validates by static analysis that every Application CRD has a cosign-verify policy attached. Drift is refused.

D-6.6. The cosign-verify policy applies in every context including air-gap; in air-gap, the policy references a per-tenant-internal Rekor mirror per ADR-0164.

### D-7: Deploy events emit audit-chain rows per ADR-0263 observability emission contract

D-7.1. Every ArgoCD sync transition (PreSync, Sync, PostSync, SyncFail, SyncSuccess) emits an audit-chain row per ADR-0263 §D.4 deploy-event class.

D-7.2. The emitter is a Rust binary (per `feedback_rust_strict_only_no_python_2026_05_20`) packaged as a K8s Job invoked from the ArgoCD Resource Hook lifecycle. The Job runs in the same namespace as the Application + has ServiceAccount-bound access to emit audit-chain rows.

D-7.3. Emitter source lives at `microservices/observability/emitters/argocd-deploy-event/` per ADR-0263.

D-7.4. Emitted fields: tenant_id, microservice_id, cluster_id, image_digest, sync_phase, cosign_verify_outcome, cedar_evaluation_outcome, sync_duration_ms, sync_initiator (ArgoCD), git_commit_sha, helm_chart_version, deployment_target_namespace, hlc_timestamp per ADR-0252 HLC default.

D-7.5. The `oya-governance-deploy-audit-chain-emit` lane per E.5 validates by static analysis that every Application CRD references the emitter Job in its Resource Hook lifecycle. Drift is refused.

D-7.6. Jenkins pipelines emit similar audit-chain rows per stage; emitter source at `microservices/observability/emitters/jenkins-pipeline-event/`.

### D-8: Stewardship class Contributor per ADR-0345; 7-day P0 / 30-day P1 CVE SLA; dev-day budget

D-8.1. Both substrates are Contributor-class per ADR-0345 OSS stewardship class policy.

D-8.2. Jenkins entry in `/specs/oss-stewardship-registry.json`: dep_name=jenkins; stewardship_class=Contributor; owner_team=ops-platform + ops-sre-reliability; cve_sla_p0=7d; cve_sla_p1=30d; resourcing=10 dev-days/quarter; license=MIT; source_url=https://github.com/jenkinsci/jenkins; adr_provenance=ADR-0349 + ADR-0345 + ADR-0211; mitigation_strategies=pin-LTS-canonical-version + cosign-verify-controller-image + JCasC-config-as-code + Kubernetes-plugin-isolation; notes=runs as K8s pod per ADR-0254; persistent state backed by per-tenant OCI/S3 object store per ADR-0343.

D-8.3. ArgoCD entry in `/specs/oss-stewardship-registry.json`: dep_name=argocd; stewardship_class=Contributor; owner_team=ops-sre-reliability + axis-observability + council-security; cve_sla_p0=7d; cve_sla_p1=30d; resourcing=12 dev-days/quarter; license=Apache-2.0; source_url=https://github.com/argoproj/argo-cd; adr_provenance=ADR-0349 + ADR-0345 + ADR-0211 + ADR-0181 + ADR-0263; mitigation_strategies=pin-canonical-version + cosign-verify-controller-image + per-Application-cosign-verify-policy + Cedar-tenant-namespace-isolation + audit-chain-emit-per-sync; notes=runs as K8s pod per ADR-0254; declarative state in git canonical source per D-5; tenant isolation via ArgoCD projects per E.3.

D-8.4. Per-quarter dev-day budget tracking lives at `evidence/stewardship/2026-Q<n>/ci-cd-substrates/` per ADR-0345.

D-8.5. Stewardship is reviewed quarterly per ADR-0345 §D.10 quarterly stewardship review (council-architecture + council-security + ops-supply-chain + ops-platform joint walk).

### D-9: Forbidden alternatives per F.1..F.6 below

D-9.1. The following alternatives are forbidden: Jenkins X (F.1); Tekton (F.2); Flux CD (F.3); Spinnaker (F.4); CircleCI / Travis / Buildkite SaaS (F.5); GitHub Actions self-hosted runners pointed at the GitHub.com control plane (F.6).

D-9.2. Per ADR-0211 in-house tech stack preference, any new CI/CD substrate proposal requires an ADR amendment to ADR-0349 with explicit per-substrate rationale + named hyperscaler precedent.

D-9.3. The dependency-policy.md Class C OSS substrate table refuses entry for the forbidden alternatives.

### D-10: Deployment context per ADR-0215 multi-context: every context including air-gap

D-10.1. Jenkins + ArgoCD run in every deployment context per ADR-0215 + `feedback_multi_context_provider_agnostic_2026_05_20`: aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider, air-gap.

D-10.2. Per-context module variants under `microservices/cloud-iac/modules/<context>/jenkins/` + `/<context>/argocd/`.

D-10.3. Air-gap module variants carry the `airgap_mirror_registry` input pointing to the tenant-internal OCI registry mirror per ADR-0164.

D-10.4. OCI Always Free maximization per `feedback_oci_always_free_maximization_2026_05_20`: oci-guest module variants ship a `tier=always-free` mode that fits both substrates within the 4 OCPU + 24 GiB Ampere A1 envelope.

D-10.5. Per-context module documentation lives at `microservices/cloud-iac/modules/<context>/jenkins/README.md` + `/<context>/argocd/README.md` declaring per-context provisioning instructions + dependency-version pinning + air-gap mirror configuration where applicable.

### D-11: Per-tenant cluster isolation via ArgoCD project namespaces; Cedar policy gates

D-11.1. Per-tenant cluster isolation is enforced via ArgoCD projects scoped to tenant_id. One ArgoCD project per tenant_id.

D-11.2. ArgoCD projects declare: `spec.sourceRepos` (per-tenant allowed source repos; typically the canonical GitOps repo); `spec.destinations` (per-tenant allowed namespaces); `spec.clusterResourceWhitelist` + `spec.namespaceResourceWhitelist` (per-tenant allowed K8s resource kinds); `spec.roles` (per-tenant role bindings consulted by Cedar).

D-11.3. Cross-tenant Application authoring (Application sources from one tenant's project targeting another tenant's namespace) is refused by the `oya-governance-argocd-tenant-namespace-isolation` lane per E.3 unless an explicit Cedar policy gate per ADR-0243 grants cross-tenant access.

D-11.4. The Cedar gate evaluates: principal (the ArgoCD project + role); action (sync); resource (the target namespace + microservice); context (tenant_id + cluster_id + compliance pack per ADR-0251).

D-11.5. Audit-chain rows emitted on every cross-tenant attempt (allowed or denied) per ADR-0263.

### D-12: Backup + DR for Jenkins (job history) + ArgoCD (state) per ADR-0343 DR matrix

D-12.1. Jenkins controller state + job history backup target: per-tenant OCI / S3-compatible object storage; backup cadence: every 4 hours; restore-time-objective: <= 1 hour; restore-point-objective: <= 4 hours; per ADR-0343.

D-12.2. ArgoCD declarative state source-of-truth: git (the canonical GitOps repo); ArgoCD live-state cache is rebuildable from git; restore-time-objective: <= 30 minutes (controller pod restart + git fetch + Application re-reconciliation); restore-point-objective: effectively 0 (git is the source).

D-12.3. Per-compliance-pack DR posture per ADR-0343 + ADR-0251: HIPAA cells require encrypted-at-rest backups (KMS-backed); GDPR-strict cells require EU-region-only backup targets; CSAP cells require Korean-region backup targets; PCI cells require PCI-DSS-compliant backup targets.

D-12.4. Per-context backup configuration lives in the OpenTofu module under `microservices/cloud-iac/modules/<context>/jenkins/backup/` + `/<context>/argocd/backup/`.

D-12.5. DR drills per ADR-0343 cadence: quarterly for production clusters; annually for dev / staging clusters; tenant-driven for sovereign cells.

## E. Enforcement-by-lanes

The Jenkins + ArgoCD substrate contract is enforced by five new lanes. Numbering is normative.

### E.1 oya-governance-jenkins-github-actions-parity (E.1)

E.1.1. The lane validates by static analysis that for every CI step in `.github/workflows/<ms>-*.yml`, there is a corresponding stage in `microservices/<ms>/Jenkinsfile` with the same step semantics.

E.1.2. The static-analysis surface: the lane parses GitHub Actions YAML (named workflow steps with `name:` + `run:` or `uses:`) + the Jenkinsfile declarative pipeline (named stages with `stage('<name>') { steps { sh '<cmd>' } }`) and confirms step-level parity per the canonical translation table at `docs/standards/jenkins-github-actions-parity.md`.

E.1.3. Drift refused: workflow step exists without corresponding Jenkinsfile stage, or vice versa.

E.1.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-Wave-15-ZE-completion. Promotion to BLOCKER 30 days post-Wave-15-ZE-completion.

E.1.5. The lane is owned by ops-platform + axis-governance + axis-cloud-iac jointly.

### E.2 oya-governance-argocd-application-cosign-verified (E.2)

E.2.1. The lane validates by static analysis that every ArgoCD Application CRD in `clusters/<cluster-id>/apps.yaml` has a cosign-verify policy attached either via `spec.source.helm.fileParameters` or via a per-cluster Kyverno policy reference per ADR-0181 + D-6.

E.2.2. The static-analysis surface: the lane parses Application CRD YAML and confirms the cosign-verify policy reference is present.

E.2.3. Drift refused: Application sourcing an image without a cosign-verify policy attached.

E.2.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-Wave-15-ZE-completion. Promotion to BLOCKER 30 days post-Wave-15-ZE-completion.

E.2.5. The lane is owned by council-security + ops-sre-reliability + axis-observability jointly.

### E.3 oya-governance-argocd-tenant-namespace-isolation (E.3)

E.3.1. The lane validates by static analysis that every ArgoCD Application is bound to a single tenant_id via project membership, and that cross-tenant Application authoring is refused unless an explicit Cedar policy gate per ADR-0243 grants cross-tenant access.

E.3.2. The static-analysis surface: the lane parses Application CRD + AppProject CRD YAML and confirms the project tenant_id scope.

E.3.3. Drift refused: Application sourced from one tenant's project targeting another tenant's namespace without Cedar grant.

E.3.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-Wave-15-ZE-completion. Promotion to BLOCKER 30 days post-Wave-15-ZE-completion.

E.3.5. The lane is owned by council-security + axis-governance + ops-sre-reliability jointly.

### E.4 oya-governance-jenkins-jcasc-only (E.4)

E.4.1. The lane validates by static analysis that every Jenkins controller state file is authored under `microservices/cloud-iac/modules/<context>/jenkins/jcasc/` as declarative JCasC YAML, and that no Jenkins controller state is declared via the Jenkins UI.

E.4.2. The static-analysis surface: the lane parses the JCasC config tree + the Jenkins controller state-export (via the JCasC API endpoint `/configuration-as-code/export`) and confirms the state-export matches the declarative YAML.

E.4.3. Drift refused: Jenkins controller state diverges from the JCasC declarative YAML (indicating UI-driven changes).

E.4.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-Wave-15-ZE-completion. Promotion to BLOCKER 30 days post-Wave-15-ZE-completion.

E.4.5. The lane is owned by ops-platform + axis-cloud-iac + ops-sre-reliability jointly.

### E.5 oya-governance-deploy-audit-chain-emit (E.5)

E.5.1. The lane validates by static analysis that every ArgoCD Application CRD references the canonical audit-chain emitter Job in its Resource Hook lifecycle (PreSync, Sync, PostSync, SyncFail) per D-7.

E.5.2. The static-analysis surface: the lane parses Application CRD YAML + confirms the `spec.syncPolicy.hooks` or `metadata.annotations.argocd.argoproj.io/hook` references the canonical emitter Job template.

E.5.3. Drift refused: Application sync occurring without an audit-chain emitter hook.

E.5.4. Lane status: REPORT-ONLY at Acceptance + advisory-until-Wave-15-ZE-completion. Promotion to BLOCKER 30 days post-Wave-15-ZE-completion.

E.5.5. The lane is owned by axis-observability + ops-sre-reliability + council-security jointly.

### E.6 Pre-existing lane integration (E.6)

E.6.1. The pre-existing `oya-governance-dependency-seam` lane per ADR-0145 admits Jenkins + ArgoCD as Class C OSS dependencies per the new entries in `docs/standards/dependency-policy.md` per D-9 (D-9 lists forbidden alternatives; the admit list is the dependency-policy table itself).

E.6.2. The pre-existing `oya-check-oss-stewardship-registry-presence` lane per ADR-0345 admits the two new Contributor-class entries in `/specs/oss-stewardship-registry.json` per D-8.

E.6.3. The pre-existing `oya-governance-stewardship-class-vocabulary` lane per ADR-0345 confirms "class" terminology applies; "tier" is reserved for cellular-tier per ADR-0248 + pod-runtime-tier per ADR-0338.

## F. Alternatives Rejected

### F.1 Rejected: Jenkins X

F.1.1. Alternative: Jenkins X (the kubernetes-native re-imagining of Jenkins by CloudBees + the JenkinsX community) as the canonical CI substrate.

F.1.2. Why rejected:

- **Over-complex.** Jenkins X combines Jenkins + Tekton + Helm + Skaffold + Prow + Lighthouse into a single bundled product. The bundled complexity exceeds what is necessary for the Oyatie use case.
- **Abandoned in many shops.** Jenkins X v1 was deprecated in favor of v3 which itself has limited adoption beyond CloudBees customers. The talent pool is narrow.
- **Less mature than canonical Jenkins LTS.** Jenkins X v3 release cadence is slower than canonical Jenkins LTS; CVE response is slower; ecosystem is narrower.
- **ArgoCD-only is simpler.** The Oyatie selection uses canonical Jenkins LTS for CI + ArgoCD for CD; the bundled approach of Jenkins X is unnecessary given the per-substrate selection.

F.1.3. Conclusion: rejected.

### F.2 Rejected: Tekton

F.2.1. Alternative: Tekton (the CNCF-graduated K8s-native CI substrate) as the canonical CI orchestrator instead of Jenkins.

F.2.2. Why rejected:

- **Lower talent pool.** Jenkins talent pool is ~10x larger than Tekton talent pool per Stack Overflow + hiring-market data; hiring + onboarding cost for self-hostable CI is meaningfully lower with Jenkins.
- **Fewer plugins.** Jenkins has ~1,800+ plugins; Tekton has ~100 Tekton Catalog tasks. The plugin ecosystem matters for self-hostable customer deployments where customer-specific integrations may not be available in Tekton.
- **No persistent UI for pipeline history.** Tekton Dashboard exists but is less feature-complete than Jenkins UI; customers operating self-hostable CI on-prem expect a feature-complete UI.
- **ArgoCD ecosystem already covers Tekton's rare use cases.** Argo Workflows is the canonical Argo-ecosystem workflow engine and handles the rare CI cases where Tekton would otherwise be necessary; the Oyatie selection covers both via Jenkins (mainline CI) + Argo Workflows (specialty workflows if needed).

F.2.3. Conclusion: rejected.

### F.3 Rejected: Flux CD

F.3.1. Alternative: Flux CD (the CNCF-graduated GitOps CD substrate) as the canonical CD orchestrator instead of ArgoCD.

F.3.2. Why rejected:

- **Talent pool narrower than ArgoCD.** ArgoCD has ~3x the talent pool per Stack Overflow + hiring-market data; hiring + onboarding cost is lower with ArgoCD.
- **UI less feature-complete.** Flux's UI is the Weave GitOps Enterprise UI which is partially closed-source; ArgoCD UI is open-source + feature-complete + canonical.
- **ApplicationSet ecosystem.** ArgoCD ApplicationSet generators (Cluster, Git, List, Matrix, Merge, PullRequest, SCM Provider) automate per-tenant + per-cluster fan-out more comprehensively than Flux's GitRepository + Kustomization pairing.
- **Convergent hyperscaler choice.** RedHat GitOps, IBM Cloud Pak, Oracle OKE all ship ArgoCD canonical. Microsoft Azure Arc ships Flux as default but supports both. The hyperscaler convergence favors ArgoCD for hosted-on-K8s self-hostable GitOps CD.
- **Pick one + commit.** Both Flux and ArgoCD are good products; the Oyatie selection requires a single canonical CD substrate; ArgoCD is chosen on the talent + UI + ecosystem grounds above.

F.3.3. Conclusion: rejected.

### F.4 Rejected: Spinnaker

F.4.1. Alternative: Spinnaker (the Netflix-open-sourced multi-cloud continuous delivery substrate) as the canonical CD orchestrator.

F.4.2. Why rejected:

- **Netflix-class ops-heavy.** Spinnaker is operationally complex; the Halyard installer requires substantial cluster resources; the multi-component architecture (Clouddriver, Deck, Echo, Front50, Gate, Igor, Kayenta, Orca, Rosco) is heavyweight.
- **Out-of-scope for K8s-native GitOps.** Spinnaker predates K8s-native GitOps; its design assumes multi-cloud heterogeneous deployment targets including VMs + functions. The Oyatie selection is K8s-native per ADR-0254.
- **ArgoCD wins for K8s-native.** ArgoCD is purpose-built for K8s-native GitOps; Spinnaker's K8s integration is an adapter rather than a native primitive.
- **Talent pool concentrated at Netflix-class shops.** Spinnaker adoption outside Netflix-class shops has declined as ArgoCD + Flux have matured.

F.4.3. Conclusion: rejected.

### F.5 Rejected: CircleCI / Travis CI / Buildkite SaaS

F.5.1. Alternative: CircleCI / Travis CI / Buildkite (SaaS CI products) as the canonical CI for self-hostable contexts.

F.5.2. Why rejected:

- **SaaS-only doesn't satisfy air-gap.** Per ADR-0164 air-gap sovereign deployment, the CI substrate must run entirely within the tenant's network with no internet egress. SaaS CI products require outbound connectivity to the SaaS control plane.
- **On-prem variants exist but are uncommon.** CircleCI Server + Travis CI Enterprise + Buildkite Self-Hosted exist but adoption is narrower than Jenkins; talent pool is narrower; CVE response is on commercial-vendor cadence rather than open-source community cadence.
- **Vendor lock-in.** SaaS CI products are vendor-controlled; the canonical CI substrate must be open-source per ADR-0211 in-house tech stack preference (Class C OSS).
- **Jenkins LTS is the canonical OSS choice.** Jenkins has the longest track record (since 2005), widest talent pool, mature CVE response, and operationally proven self-hostable deployment story.

F.5.3. Conclusion: rejected.

### F.6 Rejected: GitHub Actions on customer self-hosted runners pointed at GitHub.com control plane

F.6.1. Alternative: extend GitHub Actions to self-hostable contexts via customer-self-hosted runners that report back to the GitHub.com control plane.

F.6.2. Why rejected:

- **Ties customers to GitHub.com control plane.** Self-hosted runners require outbound HTTPS connectivity to api.github.com for job dispatch + reporting. On-prem + colo + air-gap customers cannot trust this connectivity for compliance reasons.
- **Doesn't satisfy air-gap.** Per ADR-0164 air-gap sovereign deployment, no outbound internet connectivity is permitted; GitHub Actions self-hosted runners require it.
- **Cross-tenant blast radius.** A compromise of api.github.com (an external supply chain) could propagate into tenant environments via self-hosted runner job dispatch.
- **Vendor lock-in.** Self-hosted runners depend on GitHub.com as a control plane; the canonical CI substrate for self-hostable contexts must operate independently of any single SaaS control plane.
- **GitHub Enterprise Server self-hosted variant exists but is uncommon for on-prem CI.** GitHub Enterprise Server is a separate product with its own cost + ops surface; Jenkins is the simpler + canonical choice.

F.6.3. Conclusion: rejected. GitHub Actions remains the primary CI for the hosted-on-GitHub PR review surface; Jenkins covers the self-hostable contexts where the hosted control plane is not appropriate.

## G. Multispectrum review v2.4.0

Per ADR-0322 §D-2, multispectrum review v2.4.0 applies. The 11-13 facets evaluate this ADR. Per-facet evidence files live at `evidence/debate/ADR-0349/`.

### G.1 Facet F1 — Naming + BNF + 13-layer enum (ADR-0105/0106/0107)

The five new lane names conform to v4 BNF + 13-layer enum:

- `oya-governance-jenkins-github-actions-parity` — kebab-case; layer `governance`; substrate `jenkins`; concern `github-actions-parity`. Naming-justification: governance lane validating CI substrate drift across Jenkins vs GitHub Actions surfaces. ✓
- `oya-governance-argocd-application-cosign-verified` — kebab-case; layer `governance`; substrate `argocd`; concern `application-cosign-verified`. Naming-justification: governance lane refusing ArgoCD Application authoring without cosign-verify policy per ADR-0181. ✓
- `oya-governance-argocd-tenant-namespace-isolation` — kebab-case; layer `governance`; substrate `argocd`; concern `tenant-namespace-isolation`. Naming-justification: governance lane enforcing per-tenant ArgoCD project scope per ADR-0242 + ADR-0244 + ADR-0243. ✓
- `oya-governance-jenkins-jcasc-only` — kebab-case; layer `governance`; substrate `jenkins`; concern `jcasc-only`. Naming-justification: governance lane refusing UI-driven Jenkins controller state changes. ✓
- `oya-governance-deploy-audit-chain-emit` — kebab-case; layer `governance`; substrate `deploy`; concern `audit-chain-emit`. Naming-justification: governance lane refusing ArgoCD sync transitions without audit-chain emission per ADR-0263. ✓

Twelve OpenTofu module paths conform to per-context flat layout per ADR-0131 + ADR-0339; each `microservices/cloud-iac/modules/<context>/<substrate>/` is single-concern + flat.

### G.2 Facet F2 — Architectural cleanness (clean-architecture)

Per `feedback_clean_architecture_requirements`:

- 13-layer enum inward-only flow: both substrates are at Tier 2 (Capability) layer per ADR-0248; they consume Tier 1 (Primitive) K8s + Cloud Hypervisor + Kata + cosign + Cedar + audit-chain; they serve Tier 3 (Application) microservices via Helm chart deploys; no inverse dependency.
- Port-in-kernel: both substrates expose stable APIs (Jenkins HTTP API + JCasC YAML interface; ArgoCD CRD-based API + ApplicationSet generators); Oyatie kernel consumes these as ports per the canonical clean architecture pattern.
- Cross-product refusal: tenant + product isolation enforced by ArgoCD projects + Cedar + tenant-namespace lane per E.3.

✓ clean architecture maintained.

### G.3 Facet F3 — Multi-context (oyatie-public / guest-on-aws / guest-on-oci / on-prem / colo / oyatie-as-provider / air-gap)

Per `feedback_multi_context_provider_agnostic_2026_05_20` + ADR-0215. Both substrates run in every context including air-gap. Per-context OpenTofu modules under `microservices/cloud-iac/modules/<context>/<substrate>/` for each context. Per-context provisioning is `tofu apply` end-to-end per `feedback_zero_handroll_opentofu_only_2026_05_20`. ✓

### G.4 Facet F4 — Tenant scoping (ADR-0244)

ArgoCD projects scope to tenant_id; cross-tenant Application authoring refused without Cedar grant per E.3. Jenkins JCasC authorization strategy scopes to tenant_id per JCasC matrix-based authorization. ✓

### G.5 Facet F5 — Cedar gating (ADR-0243)

ArgoCD project role mapping consults Cedar for cross-tenant Application authorization per E.3 + D-11. ✓

### G.6 Facet F6 — Substance-bar + bespoke authoring (ADR-0322)

Per-section content is bespoke. The named pressures in §A name specific hyperscaler products (Google Anthos Config Management, AWS EKS-A, Microsoft Azure Arc, RedHat GitOps, SUSE Rancher, Oracle OCI DevOps, IBM Cloud Pak) + specific corpus ADRs. The Rejected Alternatives in §F name specific competing products (Jenkins X, Tekton, Flux CD, Spinnaker, CircleCI / Travis / Buildkite, GitHub Actions self-hosted runners) + specific reasons. The detailed mechanics in §D enumerate twelve adoption surfaces — all bespoke to the substrate selection. ✓

### G.7 Facet F7 — Bominal inheritance precedence

Per `feedback_bominal_inheritance_precedence`, Bominal inherits Oyatie ADR decisions 1:1 by default. This ADR's substrate selection is Oyatie-specific and the Bominal sibling ADR authors the same Jenkins + ArgoCD pair under its own corpus path. ✓

### G.8 Facet F8 — No silent regression (Linus-style)

Per `feedback_no_silent_regression`. The five new lanes refuse silent regressions: Jenkinsfile drifting from GitHub Actions (E.1); Application sourcing unsigned images (E.2); cross-tenant Application leakage (E.3); UI-driven Jenkins state changes (E.4); ArgoCD sync without audit-chain emission (E.5). ✓

### G.9 Facet F9 — Self-modification (ADR-0247)

Per ADR-0247 self-hosting + self-modification. The Oyatie corpus's own CI/CD substrate is the same Jenkins + ArgoCD that customer tenants run; oyatie.foundry.* principals (now intelligence-owned per ADR-0335) operate the same substrate; no carve-out. ✓

### G.10 Facet M1 — Meta: governance lane vocabulary (ADR-0345 §E.7 inheritance)

Per ADR-0345's vocabulary hygiene clause (the `oya-governance-*` lane-name prefix). The five new lanes use `oya-governance-*` prefix exclusively; no `oya-governance-fitness-*` lanes are introduced. ✓

### G.11 Facet M2 — Meta: ADR-shape compliance (line-floor + frontmatter)

Line floor: ≥ 600 lines (this ADR delivers substantially more). Frontmatter: id + title + status + date + owners + amends + related_adrs + related_specs + related_memory + companion_docs + inbound_citations + doc_class + shape + authority_tier + line_floor + bespoke_authoring_requirement + enforcement_status + enforced_by + purpose — all present. ✓

### G.12 Facet F10 — Quality + performance + scalability bar

Per `feedback_quality_performance_scalability_bar`. The substrate selection matches hyperscaler-typical (Google + AWS + Microsoft + RedHat + SUSE + Oracle + IBM convergence). Capacity math per C.5: Jenkins controller ~2 CPU + 4 GiB; ArgoCD controller ~1 CPU + 2 GiB; sync-hook latency ~5-15s bounded. ✓

### G.13 Facet F11 — Verify-the-deliverable (not just line count)

Per `feedback_verify_deliverables_not_just_line_count_2026_05_20`. The lane verifications per E.1..E.5 are static-analysis with named drift refusals; verifying the deliverable is built into the lane contract. The Wave 15-ZE executor PRs are sequenced under ADR-0328 batch discipline + ADR-0322 substance-bar; per-microservice Jenkinsfile + Helm chart authoring is bespoke per microservice, not template-stamped. ✓

### G.14 Acceptance signal (multispectrum-review v2.4.0 verdict)

When all 13 facets verdicts are APPROVE per the per-facet subagent reviews at `evidence/debate/ADR-0349/`, this ADR is Accepted. The aggregate verdict is computed by the multispectrum-review v2.4.0 aggregator per ADR-0322 §D-2.

## H. Sunset

Per ADR-0108 sunset discipline.

H.1. The 30-day sunset window starts on Wave 15-ZE completion (the executor PRs that author the twelve OpenTofu modules + ~77 Jenkinsfiles + ~77 Helm charts + per-cluster apps.yaml + cosign-verify policy + audit-chain emitter integration). The five new lanes promote from REPORT-ONLY to BLOCKER at day 30.

H.2. The 30-day window aligns with sibling-ADR sunset windows (ADR-0344 + ADR-0345 + ADR-0347) and reflects the per-microservice impact at promotion (~77 microservices each carry a Jenkinsfile + Helm chart at landing time).

H.3. After the 30-day window, ANY change to `microservices/<ms>/Jenkinsfile` + `microservices/<ms>/iac/k8s/helm/` + `clusters/<cluster-id>/apps.yaml` + `microservices/cloud-iac/modules/<context>/{jenkins,argocd}/` that violates the five lanes is REFUSED by CI. The path to landing such a change is an ADR amendment per `feedback_no_silent_regression`.

H.4. Per-microservice impact at promotion: ~77 microservices each carry a Jenkinsfile (E.1) + a Helm chart (E.2 + E.3); the impact is bounded by the Wave 15-ZE batch dispatch under ADR-0328.

H.5. The OSS stewardship registry entries (Jenkins + ArgoCD) are revisited at the quarterly stewardship review per ADR-0345 §D.10.

H.6. The dependency-policy.md Class C OSS substrate table entries (Jenkins + ArgoCD) are version-pinned and bumped per the upstream LTS / minor release cadence; bumps emit per ADR-0108 sunset notifications for breaking changes.

H.7. The ADR is announced in the realignment-wave findings aggregation, in the next ADR-0327 promotion gate report, and in the developer-experience operator runbook.

## I. Cross-references

I.1. **`microservices/cloud-iac/modules/<context>/jenkins/`** — twelve OpenTofu modules (six contexts × two substrates) authored under Wave 15-ZE.

I.2. **`microservices/cloud-iac/modules/<context>/argocd/`** — same as I.1 for ArgoCD.

I.3. **`microservices/<ms>/Jenkinsfile`** — per-microservice CI pipeline; mirrors `.github/workflows/<ms>-*.yml` per E.1.

I.4. **`microservices/<ms>/iac/k8s/helm/`** — per-microservice Helm chart; ArgoCD Application source path.

I.5. **`clusters/<cluster-id>/apps.yaml`** — per-cluster GitOps manifest; ArgoCD watches via ApplicationSet `git` generator.

I.6. **`microservices/cloud-iac/modules/<context>/argocd/policies/cosign-verify/`** — cosign-verify policy template per D-6.

I.7. **`microservices/observability/emitters/argocd-deploy-event/`** — Rust audit-chain emitter binary packaged as K8s Job per D-7.

I.8. **`microservices/observability/emitters/jenkins-pipeline-event/`** — Rust audit-chain emitter binary for Jenkins pipelines per D-7.6.

I.9. **`docs/standards/jenkins-github-actions-parity.md`** — canonical translation table for the E.1 parity lane per D-3.4.

I.10. **`/specs/oss-stewardship-registry.json`** — Contributor-class entries for Jenkins + ArgoCD per D-8 + ADR-0345.

I.11. **`docs/standards/dependency-policy.md`** — Class C OSS substrate table entries per D-9 + ADR-0211.

I.12. **`tools/hooks/_canonical-primitives.md`** — CI/CD Substrates section per B2.033.

I.13. **`.github/workflows/oya-governance-jenkins-github-actions-parity.yml`** — lane workflow per E.1.

I.14. **`.github/workflows/oya-governance-argocd-application-cosign-verified.yml`** — lane workflow per E.2.

I.15. **`.github/workflows/oya-governance-argocd-tenant-namespace-isolation.yml`** — lane workflow per E.3.

I.16. **`.github/workflows/oya-governance-jenkins-jcasc-only.yml`** — lane workflow per E.4.

I.17. **`.github/workflows/oya-governance-deploy-audit-chain-emit.yml`** — lane workflow per E.5.

I.18. **ADR-0028** (`docs/decisions/ADR-0028-cloud-microservice-architecture.md`) — cloud microservice architecture; Jenkins + ArgoCD layer onto it per the amendment in the frontmatter.

I.19. **ADR-0164** (`docs/decisions/ADR-0164-air-gap-sovereign-deployment.md`) — air-gap sovereign deployment; both substrates run in air-gap per D-1.7 + D-2.8 + D-10.3.

I.20. **ADR-0181** (`docs/decisions/ADR-0181-cosign-signed-artifacts-and-modules.md`) — cosign discipline; ArgoCD enforces at deploy time per D-6.

I.21. **ADR-0211** (`docs/decisions/ADR-0211-in-house-tech-stack-preference.md`) — Class C OSS preference; Jenkins + ArgoCD admitted per D-9.

I.22. **ADR-0212** (`docs/decisions/ADR-0212-buildability-doctrine.md`) — buildability doctrine; Jenkins runs the build matrix.

I.23. **ADR-0215** (`docs/decisions/ADR-0215-multi-context-deployment.md`) — six-context posture; both substrates run in every context per D-10.

I.24. **ADR-0221** (`docs/decisions/ADR-0221-agentic-development-pipeline-hardening.md`) — hook-vs-gate doctrine; Jenkins pipeline runs are CI-gate authority.

I.25. **ADR-0243** (`docs/decisions/ADR-0243-cedar-as-universal-gate.md`) — Cedar gate; ArgoCD project role mapping consults Cedar per D-11.

I.26. **ADR-0244** (`docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`) — tenant scoping; ArgoCD project tenant_id binding per E.3.

I.27. **ADR-0254** (`docs/decisions/ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor.md`) — K8s-everywhere; both substrates run as K8s pods.

I.28. **ADR-0263** (`docs/decisions/ADR-0263-observability-emission-contract.md`) — observability emission contract; ArgoCD sync emits audit-chain rows per D-7.

I.29. **ADR-0339** (`docs/decisions/ADR-0339-shared-iac-module-library.md`) — IaC module library; module home per D-1 + D-2.

I.30. **ADR-0341** (`docs/decisions/ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md`) — cellular tier criteria; Jenkins Tier 2 + ArgoCD Tier 1 per B2.025.

I.31. **ADR-0342** (`docs/decisions/ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md`) — API versioning; Helm chart version semver per D-4.6.

I.32. **ADR-0343** (`docs/decisions/ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md`) — DR matrix; Jenkins RTO 1h + ArgoCD RTO 30min per D-12.

I.33. **ADR-0344** (`docs/decisions/ADR-0344-sustainability-finops-dimensional-model.md`) — finops; Jenkins + ArgoCD costs decompose per B2.027.

I.34. **ADR-0345** (`docs/decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md`) — stewardship class; Contributor entries per D-8.

I.35. **ADR-0346** (`docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md`) — verify entry point; Jenkinsfile pipelines invoke `oya verify --ci-required` per D-3.6.

I.36. **ADR-0347** (`docs/decisions/ADR-0347-governance-fitness-bulk-rename.md`) — governance prefix discipline; the five new lanes use `oya-governance-*` per G.10.

I.37. **`feedback_jenkins_argocd_substrate_2026_05_21.md`** — the canonical doctrine memory.

I.38. **`feedback_multi_context_provider_agnostic_2026_05_20.md`** — multi-context posture.

I.39. **`feedback_zero_handroll_opentofu_only_2026_05_20.md`** — zero-handroll OpenTofu posture.

I.40. **`feedback_oci_always_free_maximization_2026_05_20.md`** — OCI Always Free posture.

I.41. **`feedback_os_support_matrix_2026_05_20.md`** — OS support matrix.

I.42. **`feedback_quality_performance_scalability_bar.md`** — quality bar.

I.43. **`evidence/debate/ADR-0349/`** — multispectrum-review v2.4.0 per-facet evidence files (authored when this ADR lands in a review-track PR).

I.44. **`/specs/master-plan-sequencing.json`** — Wave 15-ZE entry added as part of this ADR's required-artifact contract.

## J. Completion Report

<!--
adr: ADR-0349
status: Superseded
date: 2026-05-21
session: 2026-05-21 realignment-wave authoring (sibling to ADR-0340..ADR-0348; new CI/CD substrate doctrine)
sibling_adrs: ADR-0340 (capacity model), ADR-0341 (cellular promotion gates), ADR-0342 (API versioning hybrid), ADR-0343 (DR matrix), ADR-0344 (sustainability + finops), ADR-0345 (OSS stewardship class), ADR-0346 (oya verify full CI mirror), ADR-0347 (foundry-fitness → governance bulk rename)
authority_source: feedback_jenkins_argocd_substrate_2026_05_21 + user directive 2026-05-21
substrate_jenkins: Jenkins LTS canonical CI for self-hostable contexts (air-gap, on-prem, colo, oyatie-as-provider); augments not replaces GitHub Actions on hosted-on-GitHub PR surface
substrate_argocd: ArgoCD canonical GitOps CD across all six contexts; replaces manual kubectl + Helm CLI deploys
oss_class_jenkins: Class C; license MIT; Contributor stewardship; 10 dev-days/quarter; 7d P0 + 30d P1 CVE SLA
oss_class_argocd: Class C; license Apache-2.0; Contributor stewardship; 12 dev-days/quarter; 7d P0 + 30d P1 CVE SLA
opentofu_module_count: 12 (six contexts × two substrates)
per_microservice_jenkinsfile_count: ~77 (Wave 15-ZE)
per_microservice_helm_chart_count: ~77 (Wave 15-ZE)
new_lanes: 5 (oya-governance-jenkins-github-actions-parity; oya-governance-argocd-application-cosign-verified; oya-governance-argocd-tenant-namespace-isolation; oya-governance-jenkins-jcasc-only; oya-governance-deploy-audit-chain-emit)
adoption_surfaces: 12 (D-1..D-12)
forbidden_alternatives_count: 6 (Jenkins X; Tekton; Flux CD; Spinnaker; CircleCI/Travis/Buildkite SaaS; GitHub Actions self-hosted runners pointed at GitHub.com)
sunset_window_days: 30
sunset_anchor: wave-15-ze-completion
implementation_wave: Wave 15-ZE-jenkins-argocd-substrate-rollout
implementation_wave_in_scope: false
implementation_wave_files_to_change:
  - microservices/cloud-iac/modules/{aws-guest,oci-guest,on-prem,colo,oyatie-as-provider,air-gap}/jenkins/ (six modules)
  - microservices/cloud-iac/modules/{aws-guest,oci-guest,on-prem,colo,oyatie-as-provider,air-gap}/argocd/ (six modules)
  - microservices/<ms>/Jenkinsfile (~77 files)
  - microservices/<ms>/iac/k8s/helm/ (~77 charts)
  - clusters/<cluster-id>/apps.yaml (per-cluster GitOps manifest; ~6 clusters at FD-001)
  - microservices/cloud-iac/modules/<context>/argocd/policies/cosign-verify/ (cosign-verify policy template per context)
  - microservices/observability/emitters/argocd-deploy-event/ (Rust audit-chain emitter)
  - microservices/observability/emitters/jenkins-pipeline-event/ (Rust audit-chain emitter)
  - docs/standards/jenkins-github-actions-parity.md (canonical translation table)
  - /specs/oss-stewardship-registry.json (two new Contributor entries)
  - docs/standards/dependency-policy.md (two new Class C entries)
  - tools/hooks/_canonical-primitives.md (CI/CD Substrates section)
  - .github/workflows/oya-governance-jenkins-github-actions-parity.yml (E.1 lane)
  - .github/workflows/oya-governance-argocd-application-cosign-verified.yml (E.2 lane)
  - .github/workflows/oya-governance-argocd-tenant-namespace-isolation.yml (E.3 lane)
  - .github/workflows/oya-governance-jenkins-jcasc-only.yml (E.4 lane)
  - .github/workflows/oya-governance-deploy-audit-chain-emit.yml (E.5 lane)
  - /specs/master-plan-sequencing.json (Wave 15-ZE entry)
related_adrs_count: 36
named_pressures_count: 7
rejected_alternatives_count: 6
multispectrum_facets_covered: 13
multispectrum_evidence_path: evidence/debate/ADR-0349/
hyperscaler_precedent_named: [google-anthos-config-management, aws-eks-a, microsoft-azure-arc, redhat-gitops, suse-rancher, oracle-oci-devops, ibm-cloud-pak]
out_of_scope:
  - actual OpenTofu module authoring under microservices/cloud-iac/modules/<context>/{jenkins,argocd}/
  - per-microservice Jenkinsfile authoring across ~77 microservices
  - per-microservice Helm chart authoring across ~77 microservices under microservices/<ms>/iac/k8s/helm/
  - per-cluster clusters/<cluster-id>/apps.yaml authoring
  - cosign-verify policy authoring at ArgoCD layer
  - audit-chain emitter Rust binary authoring + K8s Job packaging
  - jenkins-github-actions parity translation table at docs/standards/
  - OSS stewardship registry entry authoring (queued under Wave 15X-OSS-stewardship if not separately)
  - dependency-policy.md table updates
  - canonical-primitives cheat sheet updates
file_path: docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md
-->
</content>
</invoke>