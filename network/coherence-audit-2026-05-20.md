# cloud-network ownership-coherence audit — 2026-05-20

## Header anchor block

1. Canonical sequence anchor: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-4146`, especially §D-15 multi-context, §D-16 OpenTofu, §D-17 OS matrix, §D-18 Rust-strict, §D-19 OCI Always Free, and §D-20 audit decision tree.
2. Machine control anchor: `specs/master-plan-sequencing.json:704-868` for `deployment_contexts`, `iac_substrate`, `supported_oses`, `language_policy`, and `oci_always_free`.
3. µservice PRD anchor: `microservices/cloud-network/PRD.md` is absent from the complete path inventory; equivalent product-purpose evidence was read from `microservices/cloud-network/retired tenant_class adoption artifact:7-10` and `docs/products/cloud/PRD.md:138-176`.
4. µservice architecture anchor: `microservices/cloud-network/ARCHITECTURE.md` is absent from the complete path inventory; equivalent implementation-shape evidence was read from `crates/cloud-network-domain/src/lib.rs:1-7`, `crates/cloud-network-vpc-api/src/lib.rs:1-17`, `crates/cloud-network-lb-api/src/lib.rs:1-16`, and `crates/cloud-network-dns-api/src/lib.rs:1-16`.
5. Documentation rigor anchor: `docs/standards/documentation-rigor.md:40-83`, `docs/standards/documentation-rigor.md:175-190`, and `docs/standards/documentation-rigor.md:222-260`.

## Evidence basis

- Target µservice path audited: `microservices/cloud-network/`.
- Total files seen before this audit: 10.
- Total pre-existing lines audited in target path: 1,942.
- Chat history source searched: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
- Chat history matches processed: 52 `cloud-network` / `cloud.network` / `cloud-network` matches.
- Chat history line `552` names `cloud-network` among infrastructure substrates.
- Chat history line `686` places `cloud-network` in a bootstrap-minimum candidate with cloud-iac, cloud-k8s, cloud-secrets, and observability.
- Chat history line `15218` shows Wave 2 cloud-network audit dispatch evidence.
- Chat history line `15231` records Wave 2 Batch 2.1 as an active one-per-µservice audit task.
- Counterpart research sources used: Amazon VPC official docs, Google Cloud VPC official docs, Microsoft Azure Virtual Network official docs.
- AWS source: https://docs.aws.amazon.com/vpc/ and https://docs.aws.amazon.com/vpc/latest/userguide/amazon-vpc-limits.html.
- AWS source: https://docs.aws.amazon.com/en_us/directconnect/latest/UserGuide/connection-classic.html.
- AWS source: https://docs.aws.amazon.com/vpc/latest/reachability/how-reachability-analyzer-works.html.
- Google source: https://docs.cloud.google.com/vpc/docs/vpc.
- Google source: https://docs.cloud.google.com/vpc/docs/quota.
- Google source: https://docs.cloud.google.com/vpc/docs/flow-logs.
- Azure source: https://learn.microsoft.com/en-us/azure/virtual-network/concepts-and-best-practices.
- Azure source: https://learn.microsoft.com/en-us/azure/virtual-network/virtual-network-peering-overview.
- Azure source: https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/azure-subscription-service-limits.
- Azure source: https://learn.microsoft.com/en-us/azure/network-watcher/network-watcher-overview.

## §1 µservice purpose summary

- `cloud-network` is intended to be Oyatie's network substrate, not a small SDK example or a single cloud-provider wrapper.
- The local tenant_class matrix states that it owns per-tenant network isolation, VPC-equivalent semantics, NAT/proxy topology, per-cell subnets, mTLS enforcement, Cilium/eBPF/Envoy policy points, and the packet-classification path for every legitimate Oyatie flow (`microservices/cloud-network/retired tenant_class adoption artifact:7-10`).
- The cloud product PRD binds the broader cloud product to `cloud-network-kernel`, `cloud-network-adapter`, `cloud-network-vpc-api`, `cloud-network-lb-api`, `cloud-network-dns-api`, and a network REST API (`docs/products/cloud/PRD.md:138-143`).
- The product PRD also names the VPC, load balancer, and DNS OpenAPI contracts as the VPC / Network API with a p99 control-mutation target (`docs/products/cloud/PRD.md:176-176`).
- The Rust domain crate matches that broad purpose: it says the crate owns VPC, subnet, load balancer, DNS zone, CDN, interconnect, DDoS, and mesh invariants for `cloud.network.*` surfaces (`crates/cloud-network-domain/src/lib.rs:1-7`).
- The VPC API crate owns tenant/header/path/body normalization, idempotency, and authenticated API projection before handing typed requests to the kernel (`crates/cloud-network-vpc-api/src/lib.rs:1-17`).
- The LB API crate provides the same control-boundary shape for load balancer creation (`crates/cloud-network-lb-api/src/lib.rs:1-16`).
- The DNS API crate provides the same control-boundary shape for DNS zone creation (`crates/cloud-network-dns-api/src/lib.rs:1-16`).
- The OpenAPI VPC contract exposes a tenant-scoped VPC creation surface at `/v1/cloud/network/vpcs/{vpc_id}` (`contracts/openapi/cloud/cloud-network-vpc-v1.yaml:1-12`).
- The current µservice folder itself does not contain the PRD, architecture, OpenAPI contracts, SLOs, source tree, test tree, manifest, OS manifest, or IaC tree that would let a cold reader traverse from the ownership folder to those implementation artifacts.
- Product purpose is therefore coherent in the broader repo, but only partially coherent inside the µservice ownership path.
- The audit treats the ownership folder as the deliverable boundary because the user assigned `/Users/jasonlee/oyatie/microservices/cloud-network/` and required every file under that path to be inventoried.
- The highest-risk gap is not that network semantics are absent from the repo.
- The highest-risk gap is that the µservice ownership path does not own or link the authoritative product, architecture, OpenTofu, OS, and contract artifacts needed to build and govern those semantics.
- In current form, `cloud-network` reads like a documentation supplement folder plus three runbooks, not a complete µservice ownership package.
- The folder has real operational knowledge: DDoS, cross-cell routing stalls, and mTLS cascade runbooks are detailed.
- The folder has real user-training knowledge: onboarding, tutorial, migration, FAQ, and Rust SDK reference are present.
- The folder has useful tenant_class vocabulary: demo_trial and paid tenant_class are described across isolation, mTLS, BGP, DDoS, compliance, and throughput axes.
- The folder has a benchmark-style comparison against AWS VPC, Google Cloud VPC, Azure VNet, and Cilium Mesh.
- However, the ownership surface lacks the canonical control documents required by documentation-rigor and ADR-0328.
- It also uses older provider-wrapping language in some docs, especially Crossplane and AWS/GCP/Azure-specific substrate phrases, which drift from the post-ADR-0328 direction.
- The intended product should be expressed as portable VPC-equivalent network semantics deployed across six contexts with OpenTofu and Rust-owned control logic.
- Current local docs still often express it as an AWS/GCP/Azure wrapper plus Cilium/Envoy substrate.
- That is fixable, but it is a real coherence gap.

## §2 Inventory snapshot

| File | Lines | Bytes | Role | coherent_with_purpose? |
|---|---:|---:|---|---|
| `microservices/cloud-network/benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md` | 101 | 5,813 | Claimed measured benchmark comparison and reproducibility note. | partial |
| `microservices/cloud-network/retired tenant_class adoption artifact` | 95 | 6,672 | Tier definitions and product-purpose capsule. | partial |
| `microservices/cloud-network/faqs/network-engineer-faq.md` | 175 | 7,759 | Network engineer FAQ covering architecture, mTLS, BGP, logs, DDoS, private endpoints, IPv6, and FIPS. | partial |
| `microservices/cloud-network/migration-playbooks/from-aws-vpc-and-istio.md` | 166 | 6,478 | AWS VPC + Istio migration playbook. | partial |
| `microservices/cloud-network/onboarding/network-engineer-first-week.md` | 181 | 6,736 | First-week onboarding path with local lab exercises. | partial |
| `microservices/cloud-network/reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md` | 199 | 6,573 | Rust SDK reference implementation for VPC, service deployment, mTLS ingress, Cedar, and flow logs. | yes |
| `microservices/cloud-network/runbooks/cross-cell-routing-stall.md` | 270 | 16,981 | Incident runbook for cross-cell routing stalls. | yes |
| `microservices/cloud-network/runbooks/ddos-mitigation-engagement.md` | 268 | 16,512 | Incident runbook for DDoS mitigation. | yes |
| `microservices/cloud-network/runbooks/mtls-handshake-failure-cascade.md` | 271 | 16,718 | Incident runbook for mTLS cascade failures. | yes |
| `microservices/cloud-network/tutorials/provision-vpc-mtls-and-cedar-policy.md` | 216 | 6,843 | Tutorial for provisioning VPC, mTLS ingress, Cedar policy, and BGP demonstration. | partial |

- Total files seen before audit deliverables: 10.
- Total lines audited before audit deliverables: 1,942.
- Required but absent: `PRD.md`.
- Required but absent: `ARCHITECTURE.md`.
- Required but absent: `README.md`.
- Required but absent: `manifest.json`.
- Required but absent: `supported-oses.json`.
- Required but absent: `decisions/ADR-MS-*.md`.
- Required but absent: `implementation-plans/IP-*.md`.
- Required but absent inside ownership path: `contracts/*.{yaml,json,proto}`.
- Required but absent: `slos/*.openslo.yaml`.
- Required but absent: `cross-microservice-handoffs.md`.
- Required but absent: `capacity-model.md`.
- Required but absent: `failure-modes.md`.
- Required but absent: `incident-response.md`.
- Required but absent: `cost-budget.md`.
- Required but absent: `dpia.md`.
- Required but absent: `compliance.md`.
- Required but absent: `iac/oyatie-public-cloud/`.
- Required but absent: `iac/guest-on-aws/`.
- Required but absent: `iac/guest-on-oci/`.
- Required but absent: `iac/guest-on-oci/always-free/`.
- Required but absent: `iac/on-prem/`.
- Required but absent: `iac/colo/`.
- Required but absent: `iac/oyatie-as-cloud-provider/`.
- Required but absent: `tests/`.
- Required but absent: `src/`.
- No forbidden language files were found in the target path for `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, `*.fs`, or `*.fsx`.
- No target-path IaC files were found.
- No target-path OpenTofu backend files were found.
- No target-path sigstore/cosign module-signing files were found.
- No target-path Terraform, Pulumi, or CloudFormation files were found.
- No target-path `null_resource`, `local-exec`, `remote-exec`, provisioner, or hand-edited state pattern was found.
- The absence of forbidden IaC is not sufficient because D-16 requires OpenTofu coverage; an empty IaC directory set is a coverage failure, not a clean pass.

## §3 9-dimension audit

### §3.1 Dimension 1 — internal coherence within the µservice path

- D1-01 Purpose statement resolves: tenant_class matrix states the network substrate owns per-tenant isolation and every legitimate flow (`retired tenant_class adoption artifact:7-10`).
- D1-02 The FAQ resolves to the tenant_class matrix for SLO-like values by naming demo_trial/paid tenant_class latency (`faqs/network-engineer-faq.md:117-120`).
- D1-03 The DDoS runbook resolves to the FAQ by naming it as canonical FAQ (`runbooks/ddos-mitigation-engagement.md:32-32`).
- D1-04 The DDoS runbook resolves to its own microservice ownership by frontmatter and operator contract (`runbooks/ddos-mitigation-engagement.md:1-18`).
- D1-05 The mTLS runbook resolves to cloud-kms and cloud-iam as dependencies (`runbooks/mtls-handshake-failure-cascade.md:15-37`).
- D1-06 The cross-cell runbook resolves to private-service endpoint and rollback operations through its operator contract (`runbooks/cross-cell-routing-stall.md:15-35`).
- D1-07 The tutorial resolves to the Rust SDK example at the concept level, because both provision VPC, mTLS ingress, Cedar policy, and flow logs (`tutorials/provision-vpc-mtls-and-cedar-policy.md:11-192`; `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:54-150`).
- D1-08 The migration playbook resolves to the tenant_class matrix for provider-neutral target concepts only partially, because it maps AWS VPC/Istio concepts into cloud-network but still uses AWS/Crossplane vocabulary (`migration-playbooks/from-aws-vpc-and-istio.md:44-60`).
- D1-09 The benchmark doc resolves to the tenant_class matrix only partially: both discuss paid/paid throughput, but the benchmark claims measured values without local evidence (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:3-17`).
- D1-10 The onboarding doc resolves to the tutorial partially: both start a dev cell and create a tenant, but both rely on `make` rather than the D-18 Cargo/OpenTofu canonical build path (`onboarding/network-engineer-first-week.md:21-26`; `tutorials/provision-vpc-mtls-and-cedar-policy.md:6-9`).
- D1-11 Broken internal reference: DDoS runbook cites `microservices/cloud-network/faqs/network-engineer-faq.md`, and the file exists (`runbooks/ddos-mitigation-engagement.md:32-32`).
- D1-12 Broken internal reference not found: no target-path doc links to `PRD.md`; the absence is a missing anchor rather than a broken link.
- D1-13 Broken internal reference not found: no target-path doc links to `ARCHITECTURE.md`; the absence is a missing anchor rather than a broken link.
- D1-14 Wrong-direction reference: docs explain APIs and runbooks before the ownership folder has a product PRD, forcing readers to infer product boundaries from downstream artifacts.
- D1-15 Contradiction probe 1: tenant_class matrix demo_trial says shared region VPC on AWS/GCP (`retired tenant_class adoption artifact:12-27`), while ADR-0328 D-19 says OCI demo_trial tenant_class for guest-on-oci must mean Always Free, not just a shared AWS/GCP network profile (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3418-3438`).
- D1-16 Contradiction probe 2: FAQ says cloud-network wraps and unifies AWS/GCP/Azure network constructs through Crossplane/cloud-iac (`faqs/network-engineer-faq.md:7-11`), while D-15 says cloud-network owns portable VPC-equivalent semantics, not provider route-table vocabulary (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2058-2059`).
- D1-17 Contradiction probe 3: migration playbook says provisioning creates the underlying AWS VPC through cloud-iac + Crossplane (`migration-playbooks/from-aws-vpc-and-istio.md:44-60`), while D-16 mandates OpenTofu and forbids substituting other IaC engines (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2243-2249`).
- D1-18 Contradiction probe 4: benchmark reproducibility says `make benchmarks.cloud-network.run` (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-99`), while D-18 canonical backend invocation is Cargo over Rust crates, not Make as a release/build control surface (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3215-3247`).
- D1-19 Contradiction probe 5: onboarding uses `make dev-cell.up` and `make dev-tenant.create` (`onboarding/network-engineer-first-week.md:21-26`), which is not aligned with D-18 canonical build and release invocation (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3215-3247`).
- D1-20 Contradiction probe 6: tutorial uses `make dev-cell.up` and `kubectl` as the first setup lane (`tutorials/provision-vpc-mtls-and-cedar-policy.md:6-9`), but D-16 requires deployment plan/apply to be mediated through OpenTofu and cloud-iac (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2436-2456`).
- D1-21 Contradiction probe 7: FAQ promises VPC Flow Log style visibility and audit projections (`faqs/network-engineer-faq.md:96-105`), but the target path has no SLO, event contract, dashboard, or observability schema file.
- D1-22 Contradiction probe 8: FAQ gives p99 tenant_class latency values (`faqs/network-engineer-faq.md:117-120`), but the target path has no `slos/*.openslo.yaml`.
- D1-23 Contradiction probe 9: tenant_class matrix says every flow is illegitimate unless it traverses cloud-network (`retired tenant_class adoption artifact:7-10`), but there is no `failure-modes.md` defining bypass detection or enforcement response.
- D1-24 Contradiction probe 10: the runbooks are operationally rich (`runbooks/ddos-mitigation-engagement.md:80-186`), yet there is no `incident-response.md` or `cross-microservice-handoffs.md` consolidating cross-service paging and ownership.
- D1-25 The internal docs are not hollow; they contain real operational details and example commands.
- D1-26 The internal docs are not ownership-complete; they lack authoritative product and architecture entry points.
- D1-27 Severity for D1 missing PRD/ARCH in this substrate is P1 because the broader implementation exists but the ownership folder cannot orient a cold builder.
- D1-28 Severity for Crossplane/OpenTofu drift is P1 because it directly conflicts with D-16.
- D1-29 Severity for benchmark evidence absence is P2 because it can mislead planning but does not itself change runtime behavior.
- D1-30 Severity for Make-based tutorial/onboarding is P2 unless those commands are used as release gates; then it becomes P1.
- D1-31 The Rust SDK reference is coherent with Rust-strict policy because it uses Rust/Cargo and no forbidden application language (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:6-24`, `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:156-191`).
- D1-32 The runbooks are coherent with the service purpose because they cover network-specific incidents, not generic on-call prose.
- D1-33 The migration playbook is useful but too AWS-centered for the new all-context bar.
- D1-34 The onboarding doc is useful but lacks OS matrix, OpenTofu, and six-context paths.
- D1-35 The tutorial is useful but it teaches a loopback/Kubernetes path before teaching OpenTofu context deployment.
- D1-36 The FAQ is useful but has the strongest canonical drift because it frames the product as a wrapper over AWS/GCP/Azure.
- D1-37 The benchmark doc has useful comparison categories but overclaims measured status.
- D1-38 The tenant_class matrix has useful tenant_class axes but does not reconcile OCI demo_trial tenant_class equals Always Free.
- D1-39 The folder currently favors operator experience over product governance.
- D1-40 The folder currently favors examples over contracts.
- D1-41 The folder currently favors runbooks over architecture.
- D1-42 The folder currently favors cloud-provider analogies over the portable-semantics doctrine.
- D1-43 Internal coherence rating: partial.
- D1-44 Internal contradiction count sampled: 10.
- D1-45 Internal hard contradiction count: 3.
- D1-46 Internal missing-anchor count: at least 14 required artifacts.
- D1-47 Most urgent internal repair: add PRD, ARCHITECTURE, manifest, supported-oses, SLO, contracts pointer, IaC context matrix, and cross-microservice handoff files.
- D1-48 Highest-quality existing internal artifacts: the three runbooks and the Rust SDK reference.
- D1-49 Lowest-confidence existing internal artifact: benchmark doc because the cited evidence path was not present in the working tree during audit.
- D1-50 Dimension verdict: P1 drifted-fixable.

### §3.2 Dimension 2 — outbound cross-references

- D2-01 Outbound reference to ADR-0242 appears in tenant_class matrix frontmatter (`retired tenant_class adoption artifact:4-5`); target ADR existence was not required to modify, but the line creates an outgoing governance edge.
- D2-02 Outbound reference to ADR-0243 appears in tenant_class matrix frontmatter (`retired tenant_class adoption artifact:4-5`).
- D2-03 Outbound reference to ADR-0244 appears in tenant_class matrix frontmatter (`retired tenant_class adoption artifact:4-5`).
- D2-04 Outbound reference to ADR-0245 appears in tenant_class matrix frontmatter (`retired tenant_class adoption artifact:4-5`).
- D2-05 Outbound reference to ADR-0248 appears in tenant_class matrix frontmatter (`retired tenant_class adoption artifact:4-5`).
- D2-06 Outbound reference to ADR-0253 appears in tenant_class matrix frontmatter and FAQ HTTP/3 answer (`retired tenant_class adoption artifact:4-5`; `faqs/network-engineer-faq.md:23-30`).
- D2-07 Outbound reference to ADR-0254 appears in tenant_class matrix and FAQ Cilium answer (`retired tenant_class adoption artifact:4-5`; `faqs/network-engineer-faq.md:15-19`).
- D2-08 Outbound reference to ADR-0263 appears in tenant_class matrix and flow-log FAQ (`retired tenant_class adoption artifact:4-5`; `faqs/network-engineer-faq.md:96-105`).
- D2-09 Outbound reference to Cilium appears in tenant_class matrix and FAQ (`retired tenant_class adoption artifact:7-10`; `faqs/network-engineer-faq.md:15-19`).
- D2-10 Outbound reference to Envoy appears in tenant_class matrix and runbooks (`retired tenant_class adoption artifact:7-10`; `runbooks/mtls-handshake-failure-cascade.md:146-150`).
- D2-11 Outbound reference to Cedar appears in FAQ, tutorial, reference implementation, and runbooks (`faqs/network-engineer-faq.md:81-85`; `tutorials/provision-vpc-mtls-and-cedar-policy.md:116-137`; `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:115-129`).
- D2-12 Outbound reference to cloud-kms appears in mTLS runbook and DDoS/cross-cell coordination sections (`runbooks/mtls-handshake-failure-cascade.md:114-115`; `runbooks/ddos-mitigation-engagement.md:249-262`).
- D2-13 Outbound reference to cloud-iam appears in mTLS and DDoS runbook coordination (`runbooks/mtls-handshake-failure-cascade.md:252-264`; `runbooks/ddos-mitigation-engagement.md:249-262`).
- D2-14 Outbound reference to cloud-network-dns appears indirectly through DNS and private endpoint discussion, but the target path does not include a formal handoff file.
- D2-15 Outbound reference to Foundry appears in FAQ (`faqs/network-engineer-faq.md:156-160`).
- D2-16 Outbound reference to AWS VPC appears in benchmark and migration playbook (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:1-17`; `migration-playbooks/from-aws-vpc-and-istio.md:1-18`).
- D2-17 Outbound reference to Google Cloud VPC appears in benchmark and tenant_class matrix (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:1-17`; `retired tenant_class adoption artifact:34-51`).
- D2-18 Outbound reference to Azure VNet appears in benchmark and tenant_class matrix (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:1-17`; `retired tenant_class adoption artifact:34-40`).
- D2-19 Outbound reference to Istio appears in migration playbook (`migration-playbooks/from-aws-vpc-and-istio.md:1-42`).
- D2-20 Outbound reference to Crossplane appears in FAQ and migration playbook (`faqs/network-engineer-faq.md:7-11`; `migration-playbooks/from-aws-vpc-and-istio.md:44-60`).
- D2-21 Crossplane reference is wrong-direction under D-16 because OpenTofu is the canonical substrate for cloud deployment modules.
- D2-22 Inbound repo reference: `registry/microservices.json:112` lists `cloud-network`.
- D2-23 Inbound repo reference: `specs/master-plan-sequencing.json:410-411` lists `cloud-network` and `cloud-network-dns` in the network seam.
- D2-24 Inbound repo reference: `docs/products/cloud/PRD.md:138-143` lists cloud-network kernel, adapter, VPC API, LB API, DNS API, and network API.
- D2-25 Inbound repo reference: `docs/products/cloud/PRD.md:176-176` binds VPC/LB/DNS contracts to the cloud network API surface.
- D2-26 Inbound repo reference: `docs/machine-readable/contracts.json:659-692` binds cloud-network contract IDs to OpenAPI files and runtime crates.
- D2-27 Inbound repo reference: `registry/openapi/runtime-bindings.tsv:15-17` maps VPC, DNS, and LB operations to runtime crates and tests.
- D2-28 Inbound repo reference: `registry/openapi/schema-bindings.tsv:88-116` maps many cloud-network schemas to runtime structs.
- D2-29 Inbound repo reference: `docs/DOC-COVERAGE.md:139` marks `cloud-network` as a stub awaiting M02-P18, which matches this audit's ownership-folder gap.
- D2-30 Inbound repo reference: `microservices/cloud-kms/runbooks/hsm-cluster-failover.md:242-252` pages cloud-network for tenant CA and mTLS signing impact.
- D2-31 Inbound repo reference: `microservices/cloud-iam/runbooks/federated-identity-provider-stall.md:240-250` calls cloud-network for callback ingress, WAF, mTLS, and DNS reachability.
- D2-32 Inbound repo reference: `docs/advanced-cicd/progressive-delivery/playbook-cloud.md:33` references `cloud-network-*` canary behavior.
- D2-33 Orphan reference: the ownership folder refers to cloud-iac and Crossplane but does not include an IaC handoff or cloud-iac dependency contract.
- D2-34 Orphan reference: the ownership folder refers to mTLS tenant CA issuance but does not include a `cross-microservice-handoffs.md` binding to cloud-kms.
- D2-35 Orphan reference: the ownership folder refers to Cedar policy but has no local `policy/*.cedar` or policy-gate matrix.
- D2-36 Orphan reference: the ownership folder refers to flow logs and audit events but has no local event schema or dashboard artifact.
- D2-37 Orphan reference: benchmark cites `.foundry/evidence/benchmarks/cloud-network/2026-05-11T19:42:08Z/` (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-101`), and that evidence path was not present during audit.
- D2-38 Missing reverse reference: external OpenAPI contracts and runtime crates are not linked from a local README or architecture file in the ownership folder.
- D2-39 Missing reverse reference: docs/products/cloud PRD names cloud-network runtime crates, but the target folder does not point back to that PRD.
- D2-40 Missing reverse reference: registry bindings know runtime crates, but local onboarding/tutorial docs do not tell readers those bindings are canonical.
- D2-41 The graph is connected from global docs to implementation, but not through the µservice folder itself.
- D2-42 Documentation-rigor requires graph traversal and bidirectional citations (`docs/standards/documentation-rigor.md:192-220`).
- D2-43 The current ownership path fails that graph product standard because it lacks PRD/ARCH frontmatter and catalog reverse-index pointers.
- D2-44 Positive: runbook-to-FAQ reference is local and resolvable.
- D2-45 Positive: runbooks name operational owner rotation and dashboards.
- D2-46 Positive: code registry references outside the folder are concrete.
- D2-47 Negative: code registry references are invisible to a cold reader starting inside the folder.
- D2-48 Severity for outbound graph gaps: P1 for absent ownership anchors, P2 for individual orphan references.
- D2-49 Remediation: add local `README.md`, `PRD.md`, and `ARCHITECTURE.md` that link product PRD, contracts, crates, registry bindings, runbooks, and IaC directories.
- D2-50 Dimension verdict: P1 drifted-fixable.

### §3.3 Dimension 3 — substance bar and intern-buildability

- D3-01 A cold intern can understand that the service is a VPC-equivalent network substrate from the tenant_class matrix (`retired tenant_class adoption artifact:7-10`).
- D3-02 A cold intern can run through a conceptual VPC/mTLS/Cedar tutorial (`tutorials/provision-vpc-mtls-and-cedar-policy.md:11-192`).
- D3-03 A cold intern can read a Rust SDK example and learn request flow shape (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:26-154`).
- D3-04 A cold intern can see operational incident procedures in three runbooks.
- D3-05 A cold intern cannot find a local PRD with target users, product surfaces, functional requirements, non-functional requirements, compliance impact, success metrics, or open decisions.
- D3-06 A cold intern cannot find a local architecture file with module boundaries, runtime placement, data model, threat model, Cedar gates, provider credential mode, or substrate/product binding.
- D3-07 A cold intern cannot find the authoritative local API contracts because `contracts/` is absent under the µservice path.
- D3-08 A cold intern cannot find a local SLO document despite p99 targets in the FAQ (`faqs/network-engineer-faq.md:117-120`).
- D3-09 A cold intern cannot find local IaC modules for any context.
- D3-10 A cold intern cannot find a local OS support manifest.
- D3-11 A cold intern cannot find local tests.
- D3-12 A cold intern cannot find a local `src/` tree.
- D3-13 A cold intern cannot find a local capacity model despite bandwidth claims in tenant_class matrix (`retired tenant_class adoption artifact:23-80`).
- D3-14 A cold intern cannot find a local cost budget despite benchmark TCO claims (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:60-71`).
- D3-15 A cold intern cannot verify benchmark measurements because the named evidence path is absent (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-101`).
- D3-16 A cold intern cannot derive the complete data model from local docs.
- D3-17 Data model gap: the VPC contract has `resource_id`, `tenant_id`, `region`, CIDRs, flow logs, route table, security groups, residency, and data class (`contracts/openapi/cloud/cloud-network-vpc-v1.yaml:102-170`), but local docs do not point to it.
- D3-18 Data model gap: Rust domain has VPC, route, route table, security group, BGP, interconnect, mesh, WAF, and certificate reference types (`crates/cloud-network-domain/src/lib.rs:21-240`), but local docs do not index them.
- D3-19 API gap: local docs show CLI/API examples but do not list OpenAPI paths, status codes, or error bodies.
- D3-20 Failure semantics gap: runbooks cover incidents but no design doc defines failure modes, retry policy, idempotency, or consistency guarantees.
- D3-21 CI lane gap: local docs mention tests in runbook resolution (`runbooks/ddos-mitigation-engagement.md:167-169`; `runbooks/mtls-handshake-failure-cascade.md:169-171`; `runbooks/cross-cell-routing-stall.md:168-171`) but no local CI lane spec exists.
- D3-22 Security gap: local docs mention Cedar and mTLS, but no local policy inventory or default-deny baseline exists.
- D3-23 Compliance gap: tenant_class matrix names FIPS and packet capture retention at paid/paid (`retired tenant_class adoption artifact:55-80`), but no `compliance.md` or `dpia.md` exists.
- D3-24 Operations gap: runbooks are detailed, but there is no central `incident-response.md` tying severity, paging, evidence, customer communication, and postmortem duties.
- D3-25 Onboarding gap: documentation-rigor requires onboarding to reach at least 1,000 lines and include Day 0, Day 1, Week 1, Month 1, glossary, pitfalls, and escalation channels (`docs/standards/documentation-rigor.md:187-187`); current onboarding is 181 lines.
- D3-26 Runbook quality: each current runbook exceeds the 250-line floor in documentation-rigor (`docs/standards/documentation-rigor.md:185-185`).
- D3-27 Runbook quality: the runbooks include commands, verification, and cross-service coordination.
- D3-28 Runbook risk: many commands depend on external operational tools and dashboards that are not locally defined.
- D3-29 Tutorial quality: tutorial gives expected JSON outputs and assertions, which helps intern execution.
- D3-30 Tutorial risk: tutorial uses `make` setup before canonical OpenTofu/Cargo context paths.
- D3-31 Reference implementation quality: Rust SDK example is coherent and source-language aligned.
- D3-32 Reference implementation risk: it depends on `cloud-network-sdk = "0.42.0"` but local ownership folder does not tell a builder where that SDK is built or published (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:6-24`).
- D3-33 Migration quality: AWS/Istio migration playbook gives concrete extraction and dual-run steps.
- D3-34 Migration risk: it covers AWS/Istio only; no equivalent GCP, Azure, on-prem, colo, or Oyatie-provider migration path exists.
- D3-35 FAQ quality: FAQ covers real network engineer questions and tradeoffs.
- D3-36 FAQ risk: FAQ has canonical drift around wrapping AWS/GCP/Azure through Crossplane (`faqs/network-engineer-faq.md:7-11`).
- D3-37 Benchmark quality: benchmark categories are useful for parity thinking.
- D3-38 Benchmark risk: measured claim lacks in-repo evidence and has no OS/arch/deployment-context disclosure.
- D3-39 Documentation-rigor says PRDs require 1,500 lines with personas, 40 stories, NFRs, UX flows, success metrics, compliance, open questions, and out-of-scope (`docs/standards/documentation-rigor.md:183-183`).
- D3-40 No local PRD means intern-buildability cannot meet that bar.
- D3-41 Documentation-rigor says architecture deep dives require 1,500 lines with flow traces, layer diagrams, examples, confusions, and reading map (`docs/standards/documentation-rigor.md:189-189`).
- D3-42 No local architecture means intern-buildability cannot meet that bar.
- D3-43 The service has real substance fragments.
- D3-44 The service does not have an ownership-complete build path.
- D3-45 Intern-buildability answer: no, not from current target-path docs alone.
- D3-46 Intern-buildability answer with broader repo lookup: partial, because contracts and Rust crates exist elsewhere.
- D3-47 Required remediation: add local docs that connect target folder to contracts, crates, tests, IaC, OS manifest, and CI lanes.
- D3-48 Required remediation: convert Make-led developer flows into canonical Cargo/OpenTofu/Oya VCS steps or clearly classify them as non-release local convenience.
- D3-49 Required remediation: add capacity and cost models with explicit assumptions and benchmark provenance.
- D3-50 Dimension verdict: P1 for ownership folder buildability.

### §3.4 Dimension 4 — canonical-direction alignment

- D4-01 Multi-context constraint source: ADR-0328 D-15 requires all six contexts for every µservice unless N/A is explicit (`docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1736`).
- D4-02 Multi-context status: drifted-fixable because no local context matrix or `iac/<context>/` directory exists.
- D4-03 Public-cloud context: missing local IaC and missing support declaration.
- D4-04 Guest-on-AWS context: missing local IaC and missing support declaration.
- D4-05 Guest-on-OCI context: missing local IaC, missing support declaration, and missing Always Free sub-profile.
- D4-06 On-prem context: missing local IaC and missing support declaration.
- D4-07 Colo context: missing local IaC and missing support declaration.
- D4-08 Oyatie-as-cloud-provider context: missing local IaC and missing support declaration.
- D4-09 OpenTofu constraint source: ADR-0328 D-16 says OpenTofu, not Terraform, Pulumi, or CloudFormation (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2243-2249`).
- D4-10 OpenTofu status: drifted-fixable because the folder contains no OpenTofu modules and uses Crossplane language in FAQ/migration.
- D4-11 OpenTofu positive: no Terraform/Pulumi/CloudFormation files or forbidden patterns were found in the target path.
- D4-12 OpenTofu negative: absence of forbidden files does not satisfy required per-context modules.
- D4-13 OpenTofu negative: FAQ says Crossplane/cloud-iac wrapper (`faqs/network-engineer-faq.md:7-11`).
- D4-14 OpenTofu negative: migration says underlying AWS VPC via cloud-iac + Crossplane (`migration-playbooks/from-aws-vpc-and-istio.md:44-60`).
- D4-15 OS support source: ADR-0328 D-17 requires supported OS manifests for binaries, daemons, controllers, and native bundles (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2646-3044`).
- D4-16 OS support status: drifted-fixable because no `supported-oses.json` or equivalent manifest exists.
- D4-17 OS positive: no out-of-scope OS claims were found in the target path.
- D4-18 OS negative: no Tier-1 OS package/test/CI coverage is declared.
- D4-19 Rust-strict source: ADR-0328 D-18 requires Rust backend/runtime/CLI/validation/codegen/scripting/CI, with narrow non-Rust exceptions (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3045-3247`).
- D4-20 Rust-strict status: aligned for files present in target path because only Markdown exists.
- D4-21 Rust-strict positive: Rust SDK reference uses Rust and Cargo (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:6-24`; `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:156-191`).
- D4-22 Rust-strict concern: durable docs still use Make as primary setup/benchmark commands.
- D4-23 OCI Always Free source: ADR-0328 D-19 says guest-on-oci demo_trial means Always Free, not a generic free-trial abstraction (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3418-3438`).
- D4-24 OCI Always Free status: drifted-fixable because no `iac/guest-on-oci/always-free/` exists and demo_trial tenant_class does not mention OCI Always Free.
- D4-25 OCI positive: no incompatible parallel "free tier" naming was found.
- D4-26 OCI negative: demo_trial is described as shared AWS/GCP region VPC with 100 Mbps and 5 GB/day, not as an OCI Always Free constrained deployment (`retired tenant_class adoption artifact:12-27`).
- D4-27 D-20 audit decision tree source: ADR-0328 D-20 requires language scan, OpenTofu module evidence, OS manifest evidence, tenant onboarding evidence, and context support (`docs/decisions/ADR-0700-ci-admission-live-apex.md:4120-4146`).
- D4-28 D-20 status: current ownership folder can supply tenant onboarding evidence but not language build, OpenTofu, or OS manifest evidence.
- D4-29 Documentation-rigor source: every µservice must answer ADR-adherence checklist questions in `ARCHITECTURE.md` or `compliance.md` (`docs/standards/documentation-rigor.md:222-260`).
- D4-30 Documentation-rigor status: incoherent inside ownership path because neither file exists.
- D4-31 Canonical positive: actual Rust domain crate is adapter-free and typed (`crates/cloud-network-domain/src/lib.rs:1-7`).
- D4-32 Canonical positive: actual API crates include tenant, idempotency, principal, authorization, and data-class fields (`crates/cloud-network-vpc-api/src/lib.rs:102-174`; `crates/cloud-network-lb-api/src/lib.rs:95-163`; `crates/cloud-network-dns-api/src/lib.rs:93-134`).
- D4-33 Canonical negative: target ownership folder does not surface those facts.
- D4-34 Canonical negative: target ownership folder still uses older provider wrappers and Crossplane in user-facing docs.
- D4-35 Classification for multi-context: drifted-fixable.
- D4-36 Classification for OpenTofu: drifted-fixable.
- D4-37 Classification for OS support: drifted-fixable.
- D4-38 Classification for Rust-strict: aligned for file inventory, drifted-fixable for commands.
- D4-39 Classification for OCI Always Free: drifted-fixable.
- D4-40 Classification for ownership coherence: drifted-fixable but urgent.
- D4-41 P1 trigger: missing all six context IaC dirs for a network substrate.
- D4-42 P1 trigger: no supported OS manifest.
- D4-43 P1 trigger: no OCI Always Free demo_trial reconciliation.
- D4-44 P1 trigger: Crossplane/OpenTofu drift in canonical docs.
- D4-45 P2 trigger: Make-first tutorials.
- D4-46 P2 trigger: measured benchmark evidence not found.
- D4-47 P3 positive: no forbidden language source files in target path.
- D4-48 Remediation should happen in the ownership folder first, then point to external crates/contracts.
- D4-49 Remediation should not move external contracts without a planned ownership migration.
- D4-50 Dimension verdict: P1 canonical drift.

### §3.5 Dimension 5 — industry-counterpart parity

- D5-01 Counterpart bar is AWS VPC, Google Cloud VPC, and Azure Virtual Network.
- D5-02 AWS VPC official docs list VPC creation/configuration, peering, traffic mirroring, VPC API, IPAM, PrivateLink, Transit Gateway, Network Manager, and Cloud WAN (https://docs.aws.amazon.com/vpc/).
- D5-03 AWS VPC quotas include route tables, routes per route table, route servers, security groups, rules, security groups per ENI, subnet sharing, and Network Address Usage (`docs.aws.amazon.com/vpc/latest/userguide/amazon-vpc-limits.html`).
- D5-04 AWS Direct supports dedicated connection port speeds of 1, 10, 100, and 400 Gbps (https://docs.aws.amazon.com/en_us/directconnect/latest/UserGuide/connection-classic.html).
- D5-05 AWS Reachability Analyzer models paths through VPC resources and reports reachability without sending packets (https://docs.aws.amazon.com/vpc/latest/reachability/how-reachability-analyzer-works.html).
- D5-06 Google Cloud VPC official docs describe global VPC networks, regional subnets, routes, dynamic routing through Cloud Router, firewall policies, internal communication, Cloud NAT, Cloud VPN, Cloud Interconnect, VPC Peering, and Private Service (https://docs.cloud.google.com/vpc/docs/vpc).
- D5-07 Google VPC quotas include per-network instances, aliases, subnet ranges, peering, static routes, dynamic routes, Private Service Connect, MTU, alias IPs, and flow-related limits (https://docs.cloud.google.com/vpc/docs/quota).
- D5-08 Google VPC Flow Logs cover VM, GKE, Cloud Run direct VPC egress, Cloud Interconnect, and Cloud VPN traffic samples, plus monitoring, cost, and forensics use cases (https://docs.cloud.google.com/vpc/docs/flow-logs).
- D5-09 Azure Virtual Network official docs include address spaces, subnets, NSGs, regions, subscriptions, and best practices (https://learn.microsoft.com/en-us/azure/virtual-network/concepts-and-best-practices).
- D5-10 Azure VNet peering supports same-region/cross-region/cross-subscription/cross-tenant connectivity, service chaining, gateway transit, UDRs, VPN, and ExpressRoute paths (https://learn.microsoft.com/en-us/azure/virtual-network/virtual-network-peering-overview).
- D5-11 Azure limits include 100 virtual networks, 20 DNS servers per VNet, 4,096 private IPs per VNet, 500,000 concurrent flows per NIC, 200 NSGs, 1,000 NSG rules maximum, 200 route tables, and 600 UDRs per route table (https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/azure-subscription-service-limits).
- D5-12 Azure Network Watcher provides monitoring, diagnostics, metrics, and flow logs for VMs, VNets, application gateways, and load balancers (https://learn.microsoft.com/en-us/azure/network-watcher/network-watcher-overview).
- D5-13 Oyatie present: VPC-equivalent isolation.
- D5-14 Oyatie present: per-cell subnets.
- D5-15 Oyatie present: route tables and route next-hop kinds in Rust domain (`crates/cloud-network-domain/src/lib.rs:161-200`).
- D5-16 Oyatie present: security groups and security rules in Rust domain (`crates/cloud-network-domain/src/lib.rs:202-235`).
- D5-17 Oyatie present: load balancer API and mTLS config (`crates/cloud-network-lb-api/src/lib.rs:116-153`).
- D5-18 Oyatie present: DNS zone API (`crates/cloud-network-dns-api/src/lib.rs:114-125`).
- D5-19 Oyatie present: DDoS runbook and mitigation path (`runbooks/ddos-mitigation-engagement.md:80-186`).
- D5-20 Oyatie present: mTLS incident path (`runbooks/mtls-handshake-failure-cascade.md:82-188`).
- D5-21 Oyatie present: BGP and route correction path (`runbooks/cross-cell-routing-stall.md:81-186`).
- D5-22 Oyatie present: Cedar policy integration in examples (`tutorials/provision-vpc-mtls-and-cedar-policy.md:116-137`).
- D5-23 Oyatie present: flow logs in FAQ and examples (`faqs/network-engineer-faq.md:96-105`; `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:131-150`).
- D5-24 Gap: no local IPAM equivalent plan despite AWS IPAM being part of the VPC family.
- D5-25 Gap: no local PrivateLink/Private Service Connect/Private Endpoint parity matrix.
- D5-26 Gap: no local Transit Gateway / Network Connectivity Center / Virtual WAN equivalent.
- D5-27 Gap: no local Reachability Analyzer / Connectivity Tests / Network Watcher diagnostic surface.
- D5-28 Gap: no local traffic mirroring / packet capture contract despite paid packet capture claims.
- D5-29 Gap: no local Network Firewall / Cloud NGFW / Azure Firewall equivalent.
- D5-30 Gap: no local Network Manager / Virtual Network Manager / global network topology manager.
- D5-31 Gap: no local route-propagation quota model.
- D5-32 Gap: no local Shared VPC / subnet sharing / cross-account sharing model.
- D5-33 Gap: no local customer BYOIP or IP prefix advertisement model.
- D5-34 Gap: no local service endpoint/private DNS integration detail.
- D5-35 Gap: no local direct interconnect SKU and SLA model beyond tenant_class matrix prose.
- D5-36 Gap: no local NAT gateway capacity and failure model.
- D5-37 Gap: no local egress-only IPv6 gateway equivalent.
- D5-38 Gap: no local firewall hierarchy / org-level policy model.
- D5-39 Gap: no local Network Access Analyzer style formal verification surface.
- D5-40 Gap: no local quotas by tenant/tier/context.
- D5-41 Additive Oyatie surface: Cedar-gated network policy integration is stronger than a basic hyperscaler VPC control plane when fully implemented.
- D5-42 Additive Oyatie surface: mTLS as a first-class load balancer axis is more explicit than base VPC products.
- D5-43 Additive Oyatie surface: tenant packet tags and audit-chain flow-log projection are product-specific advantages.
- D5-44 Additive Oyatie surface: cell-aware routing and platform-wide illegitimate-flow doctrine are stronger substrate claims than ordinary VPC docs.
- D5-45 Headline parity answer: partial.
- D5-46 Oyatie matches core VPC primitives in implementation model.
- D5-47 Oyatie does not match the union coverage of AWS VPC, Google Cloud VPC, and Azure VNet in the ownership folder.
- D5-48 It also does not document enough quotas, analyzers, managed private-service endpoints, and global network management to claim hyperscaler parity.
- D5-49 Remediation: create a feature parity backlog tied to Rust domain/API structs, OpenTofu modules, and SLOs.
- D5-50 Dimension verdict: P1 parity gap.

### §3.6 Dimension 6 — multi-context deployment support

- D6-01 ADR-0328 D-15 requires the six context IDs `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` (`docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1736`).
- D6-02 Master plan repeats those six context IDs in machine-readable form (`specs/master-plan-sequencing.json:704-745`).
- D6-03 `oyatie-public-cloud`: not documented in target path.
- D6-04 `oyatie-public-cloud`: missing `iac/oyatie-public-cloud/`.
- D6-05 `oyatie-public-cloud`: no state backend, variables, outputs, or README.
- D6-06 `guest-on-aws`: partially implied by AWS-specific docs and migration playbook.
- D6-07 `guest-on-aws`: missing `iac/guest-on-aws/`.
- D6-08 `guest-on-aws`: no S3+DynamoDB state backend declaration despite brief-template example for cloud-network (`docs/standards/brief-template.md:947-947`).
- D6-09 `guest-on-oci`: not documented in target path.
- D6-10 `guest-on-oci`: missing `iac/guest-on-oci/`.
- D6-11 `guest-on-oci`: missing `iac/guest-on-oci/always-free/`.
- D6-12 `guest-on-oci`: demo_trial tenant_class does not map to OCI Always Free.
- D6-13 `on-prem`: implied by FAQ air-gapped support (`faqs/network-engineer-faq.md:89-93`).
- D6-14 `on-prem`: missing `iac/on-prem/`.
- D6-15 `on-prem`: no documented substrate primitives for physical switch/BGP/MetalLB/FRR or air-gapped registry.
- D6-16 `colo`: implied by BGP/interconnect tenant_class language (`retired tenant_class adoption artifact:55-80`).
- D6-17 `colo`: missing `iac/colo/`.
- D6-18 `colo`: no documented provider-specific N/A reason.
- D6-19 `oyatie-as-cloud-provider`: implied by product PRD cloud-network kernel/API (`docs/products/cloud/PRD.md:138-176`).
- D6-20 `oyatie-as-cloud-provider`: missing `iac/oyatie-as-cloud-provider/`.
- D6-21 `oyatie-as-cloud-provider`: no native network product API IaC wiring in the target path.
- D6-22 No context is correctly marked N/A.
- D6-23 No context is explicitly unsupported.
- D6-24 No context has a local manifest declaring support.
- D6-25 No context has local plan/apply verification evidence.
- D6-26 No context has a tenant onboarding workflow tied to OpenTofu variables.
- D6-27 No context has a per-context rollback path.
- D6-28 No context has per-context secret/backend handling.
- D6-29 No context has per-context CI lane naming.
- D6-30 No context has per-context OS/package mapping.
- D6-31 Forbidden pattern: cloud vendor APIs in docs are mostly migration/operator examples, not business logic code.
- D6-32 Forbidden pattern risk: FAQ frames cloud-network as wrapping AWS/GCP/Azure network constructs, which encourages provider vocabulary leakage (`faqs/network-engineer-faq.md:7-11`).
- D6-33 Forbidden pattern risk: migration playbook provisions underlying AWS VPC through Crossplane (`migration-playbooks/from-aws-vpc-and-istio.md:44-60`).
- D6-34 D-15 specifically says cloud-network should own portable VPC-equivalent semantics, not provider route-table vocabulary (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2058-2059`).
- D6-35 Current artifacts do not prove multi-context deployability.
- D6-36 Current artifacts prove only provider-adjacent training and operations substance.
- D6-37 The runbooks are operationally context-light: they use cells, tenants, dashboards, and Kubernetes namespace, but not the six canonical contexts.
- D6-38 The tutorial is loopback-focused, not six-context focused.
- D6-39 The onboarding is loopback-focused, not six-context focused.
- D6-40 The benchmark comparison is hyperscaler-focused, not context-deployability focused.
- D6-41 P1 finding: all six context IaC directories are absent.
- D6-42 P1 finding: no supported-context manifest exists.
- D6-43 P1 finding: no OCI Always Free sub-profile exists.
- D6-44 P2 finding: on-prem and colo support are hinted but not buildable.
- D6-45 P2 finding: provider language leaks into product semantics.
- D6-46 Required repair: add `manifest.json` or equivalent with six context statuses and N/A proofs.
- D6-47 Required repair: add OpenTofu modules per context or explicit exception records.
- D6-48 Required repair: add per-context tenant onboarding, state backend, and rollback docs.
- D6-49 Required repair: update FAQ/migration language to portable semantics plus context adapters.
- D6-50 Dimension verdict: P1 missing multi-context support.

### §3.7 Dimension 7 — OpenTofu IaC coverage

- D7-01 ADR-0328 D-16 mandates OpenTofu as the only IaC substrate for in-scope cloud deployment modules (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2249`).
- D7-02 ADR-0328 D-16 requires per-service/per-context directories and required files such as `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and README (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2275-2309`).
- D7-03 Directory inventory: no `iac/` directory exists under `microservices/cloud-network/`.
- D7-04 `iac/oyatie-public-cloud/`: absent.
- D7-05 `iac/guest-on-aws/`: absent.
- D7-06 `iac/guest-on-oci/`: absent.
- D7-07 `iac/guest-on-oci/always-free/`: absent.
- D7-08 `iac/on-prem/`: absent.
- D7-09 `iac/colo/`: absent.
- D7-10 `iac/oyatie-as-cloud-provider/`: absent.
- D7-11 Required `main.tf`: absent in every context.
- D7-12 Required `variables.tf`: absent in every context.
- D7-13 Required `outputs.tf`: absent in every context.
- D7-14 Required `versions.tf`: absent in every context.
- D7-15 Required per-context README: absent in every context.
- D7-16 State backend for public cloud: absent.
- D7-17 State backend for AWS guest: absent.
- D7-18 State backend for OCI guest: absent.
- D7-19 State backend for OCI Always Free: absent.
- D7-20 State backend for on-prem: absent.
- D7-21 State backend for colo: absent.
- D7-22 State backend for Oyatie-as-provider: absent.
- D7-23 Sigstore signing wiring: absent.
- D7-24 Cosign verification wiring: absent.
- D7-25 Module provenance wiring: absent.
- D7-26 Plan/apply audit events: absent.
- D7-27 Tenant variable examples: absent.
- D7-28 Context-specific outputs: absent.
- D7-29 Network-specific resources expected but absent: VPC/VCN/network namespace.
- D7-30 Network-specific resources expected but absent: subnets.
- D7-31 Network-specific resources expected but absent: route tables.
- D7-32 Network-specific resources expected but absent: security groups/network ACLs/firewall policies.
- D7-33 Network-specific resources expected but absent: NAT/egress gateway.
- D7-34 Network-specific resources expected but absent: private endpoints/private service connectivity.
- D7-35 Network-specific resources expected but absent: load balancer scaffolding.
- D7-36 Network-specific resources expected but absent: flow-log sink wiring.
- D7-37 Network-specific resources expected but absent: DDoS/WAF integration.
- D7-38 Network-specific resources expected but absent: BGP/interconnect primitives.
- D7-39 No Terraform references were found in the target path.
- D7-40 No Pulumi references were found in the target path.
- D7-41 No CloudFormation references were found in the target path.
- D7-42 No `null_resource` pattern was found.
- D7-43 No `local-exec` pattern was found.
- D7-44 No `remote-exec` pattern was found.
- D7-45 No provisioner pattern was found.
- D7-46 No `tfstate` pattern was found.
- D7-47 Crossplane appears in FAQ and migration docs and should be removed or reframed (`faqs/network-engineer-faq.md:7-11`; `migration-playbooks/from-aws-vpc-and-istio.md:44-60`).
- D7-48 D-16 classifies missing OpenTofu modules as P1/P2 depending scope; for this infrastructure substrate it is P1 (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2577-2581`).
- D7-49 Remediation: create per-context OpenTofu modules and make cloud-iac own orchestration while cloud-network owns portable inputs/outputs.
- D7-50 Dimension verdict: P1 missing OpenTofu coverage.

### §3.8 Dimension 8 — OS support matrix

- D8-01 ADR-0328 D-17 requires Tier-1/Tier-2 OS support declarations and test/package lanes (`docs/decisions/ADR-0700-ci-admission-live-apex.md:2646-3044`).
- D8-02 `supported-oses.json`: absent.
- D8-03 Equivalent `supported_oses` manifest field: absent in target path.
- D8-04 Manifest format: absent.
- D8-05 Tier-1 Talos Linux v1.10+: no local status.
- D8-06 Tier-1 RHEL 9.x: no local status.
- D8-07 Tier-1 Oracle Linux 9.x: no local status.
- D8-08 Tier-1 SUSE Linux Enterprise Server 15 SP6+: no local status.
- D8-09 Tier-1 Ubuntu 24.04 LTS: no local status.
- D8-10 Tier-1 Debian 12: no local status.
- D8-11 Tier-1 Rocky Linux 9.x: no local status.
- D8-12 Tier-1 AlmaLinux 9.x: no local status.
- D8-13 Tier-1 CentOS Stream 9: no local status.
- D8-14 Tier-1 Amazon Linux 2023: no local status.
- D8-15 Tier-1 Flatcar Container Linux stable: no local status.
- D8-16 Tier-1 VMware Photon OS 5: no local status.
- D8-17 Tier-1 macOS 15+ on Apple Silicon M5+: no local status.
- D8-18 Tier-2 Linux ppc64le: no local test-only declaration.
- D8-19 Tier-2 Linux s390x: no local test-only declaration.
- D8-20 Out-of-scope Intel macOS: not claimed.
- D8-21 Out-of-scope pre-M5 Apple Silicon: not claimed.
- D8-22 Out-of-scope FreeBSD: not claimed.
- D8-23 Out-of-scope OpenBSD: not claimed.
- D8-24 Out-of-scope Windows Server: not claimed.
- D8-25 Out-of-scope Solaris/illumos: not claimed.
- D8-26 Package format RPM: no local declaration.
- D8-27 Package format DEB: no local declaration.
- D8-28 Package format `.pkg`: no local declaration.
- D8-29 Package format Homebrew: no local declaration.
- D8-30 Talos system extension: no local declaration.
- D8-31 Flatcar extension: no local declaration.
- D8-32 Container image support: implied by Kubernetes runbook commands but not declared.
- D8-33 CI lane for Talos: absent.
- D8-34 CI lane for RHEL: absent.
- D8-35 CI lane for Oracle Linux: absent.
- D8-36 CI lane for SLES: absent.
- D8-37 CI lane for Ubuntu: absent.
- D8-38 CI lane for Debian: absent.
- D8-39 CI lane for Rocky: absent.
- D8-40 CI lane for AlmaLinux: absent.
- D8-41 CI lane for CentOS Stream: absent.
- D8-42 CI lane for Amazon Linux: absent.
- D8-43 CI lane for Flatcar: absent.
- D8-44 CI lane for Photon: absent.
- D8-45 CI lane for macOS M5+: absent.
- D8-46 Existing docs use `kubectl` and operational probes, but those do not substitute for OS support manifest entries.
- D8-47 No forbidden out-of-scope OS claim is present, which avoids one failure class.
- D8-48 Missing OS support is still P1 because D-17 says Tier-1 support is blocking for covered runtime surfaces.
- D8-49 Remediation: add `supported-oses.json` with per-OS package, architecture, test lane, exception, and context applicability.
- D8-50 Dimension verdict: P1 missing OS matrix.

### §3.9 Dimension 9 — Rust-strict language coverage

- D9-01 ADR-0328 D-18 requires Rust for backend, runtime, CLI, validation, codegen, scripting, and CI control surfaces (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3045-3247`).
- D9-02 Target-path forbidden language scan found no `*.py`.
- D9-03 Target-path forbidden language scan found no `*.js`.
- D9-04 Target-path forbidden language scan found no `*.ts`.
- D9-05 Target-path forbidden language scan found no `*.rb`.
- D9-06 Target-path forbidden language scan found no `*.go`.
- D9-07 Target-path forbidden language scan found no `*.java`.
- D9-08 Target-path forbidden language scan found no `*.scala`.
- D9-09 Target-path forbidden language scan found no `*.groovy`.
- D9-10 Target-path forbidden language scan found no `*.php`.
- D9-11 Target-path forbidden language scan found no `*.fs`.
- D9-12 Target-path forbidden language scan found no `*.fsx`.
- D9-13 Target-path file type inventory is Markdown only before this audit.
- D9-14 Markdown is an authorized non-Rust artifact type under D-18.
- D9-15 No generated SDK output is present in target path.
- D9-16 No frontend code exists under `frontend/<platform>/`.
- D9-17 No Swift frontend exists.
- D9-18 No Kotlin frontend exists.
- D9-19 No WinUI3 frontend exists.
- D9-20 No unauthorized frontend code exists.
- D9-21 Reference implementation uses Rust and `tokio` (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:6-24`).
- D9-22 Reference implementation invocation is `cargo run --release` (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:156-160`).
- D9-23 Reference implementation test command is `cargo test --features hermetic` (`reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:184-191`).
- D9-24 Runbooks use Cargo commands for resolution gates (`runbooks/ddos-mitigation-engagement.md:167-169`; `runbooks/mtls-handshake-failure-cascade.md:169-171`; `runbooks/cross-cell-routing-stall.md:168-171`).
- D9-25 Rust domain crate exists outside target path and is in Rust (`crates/cloud-network-domain/src/lib.rs:1-7`).
- D9-26 Rust API crates exist outside target path for VPC, LB, and DNS (`crates/cloud-network-vpc-api/src/lib.rs:1-17`; `crates/cloud-network-lb-api/src/lib.rs:1-16`; `crates/cloud-network-dns-api/src/lib.rs:1-16`).
- D9-27 Concern: benchmark reproducibility uses Make (`benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-99`).
- D9-28 Concern: onboarding setup uses Make (`onboarding/network-engineer-first-week.md:21-26`).
- D9-29 Concern: tutorial setup uses Make (`tutorials/provision-vpc-mtls-and-cedar-policy.md:6-9`).
- D9-30 Concern: migration playbook uses shell loops and AWS CLI extraction (`migration-playbooks/from-aws-vpc-and-istio.md:10-28`); as migration operator prose it is less severe than a repo-owned script, but it should not become canonical automation.
- D9-31 Concern: runbooks use shell tools, `kubectl`, `rg`, `curl`, and dashboards; these may be acceptable operational commands but should not replace Rust-owned automation.
- D9-32 Build invocation per D-20 should cite Cargo workspace commands, OpenTofu plan/apply via cloud-iac, and Oya VCS claim/verify/done/promote gates.
- D9-33 Target path lacks a `build.md` or `README.md` that states canonical build invocation.
- D9-34 Target path lacks a `tests/` directory with Rust integration tests.
- D9-35 Target path lacks CI lane docs that prove no forbidden application language can be introduced later.
- D9-36 No `package.json` was present.
- D9-37 No `requirements.txt` was present.
- D9-38 No `go.mod` was present.
- D9-39 No `pom.xml` was present.
- D9-40 No `build.gradle` was present.
- D9-41 No `Gemfile` was present.
- D9-42 No forbidden source file count: zero.
- D9-43 Authorized source file count in target path before audit: ten Markdown docs.
- D9-44 Rust-strict file coverage: aligned.
- D9-45 Rust-strict command coverage: partial.
- D9-46 Rust-strict architecture coverage: partial because Rust crates exist but are not indexed locally.
- D9-47 Severity for forbidden source files: P3 positive, no issue.
- D9-48 Severity for Make-first command drift: P2.
- D9-49 Required remediation: replace Make-first docs with Cargo/OpenTofu/Oya command surfaces or explicitly classify Make snippets as historical/non-release convenience.
- D9-50 Dimension verdict: aligned files, P2 command drift.

## §4 Findings summary

| Severity | Dimension | Short description | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | D1/D3 | Ownership path lacks `PRD.md`, so product requirements cannot be built from the target folder. | `docs/standards/documentation-rigor.md:183-183`; §2 inventory | Add a 1,500+ line cloud-network PRD or equivalent machine-owned product artifact with target users, requirements, metrics, compliance, and scope. |
| P1 | D1/D3 | Ownership path lacks `ARCHITECTURE.md`, so module boundaries, Cedar gates, tenant scoping, deployment shape, and observability are not locally answered. | `docs/standards/documentation-rigor.md:189-189`; `docs/standards/documentation-rigor.md:222-260`; §2 inventory | Add architecture deep dive linking domain crate, VPC/LB/DNS API crates, OpenAPI contracts, IaC, and runbooks. |
| P1 | D6/D7 | No `iac/<context>/` directories exist for any of six required contexts. | `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1736`; `docs/decisions/ADR-0700-ci-admission-live-apex.md:2275-2309`; §2 inventory | Add OpenTofu modules for all supported contexts or explicit N/A records with D-15-compliant reasons. |
| P1 | D7 | Crossplane language conflicts with OpenTofu-only doctrine. | `faqs/network-engineer-faq.md:7-11`; `migration-playbooks/from-aws-vpc-and-istio.md:44-60`; `docs/decisions/ADR-0700-ci-admission-live-apex.md:2243-2249` | Rewrite provider-wrapper sections as portable cloud-network semantics plus OpenTofu context adapters. |
| P1 | D8 | No supported OS manifest exists. | `docs/decisions/ADR-0700-ci-admission-live-apex.md:2646-3044`; §2 inventory | Add `supported-oses.json` with Tier-1/Tier-2 rows, package formats, arch, CI lanes, and exclusions. |
| P1 | D4/D6 | OCI demo_trial tenant_class does not reconcile to Always Free and no `iac/guest-on-oci/always-free/` exists. | `retired tenant_class adoption artifact:12-27`; `docs/decisions/ADR-0700-ci-admission-live-apex.md:3418-3438`; §2 inventory | Define OCI demo_trial tenant_class as Always Free and split paid OCI capacity into paid tenant_class. |
| P1 | D5 | Counterpart union coverage is partial; IPAM, private endpoint, transit hub, diagnostics, traffic mirroring, firewall hierarchy, and quota models are missing locally. | AWS/GCP/Azure official sources listed in header; `crates/cloud-network-domain/src/lib.rs:1-7`; §3.5 | Create a parity backlog and map each missing capability to Rust API, OpenTofu, SLO, and runbook owners. |
| P1 | D2 | External contracts and runtime crates exist, but the ownership folder does not link them through README/PRD/ARCH. | `docs/machine-readable/contracts.json:659-692`; `registry/openapi/runtime-bindings.tsv:15-17`; §2 inventory | Add a local artifact map with canonical inbound/outbound edges. |
| P2 | D1/D3 | Benchmark claims measured results but named evidence path was not present during audit. | `benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:3-17`; `benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-101` | Reclassify as target/projection or add signed benchmark evidence with OS/arch/context/tenant disclosure. |
| P2 | D9 | Make-first setup and benchmark commands drift from Rust-strict canonical invocation. | `benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-99`; `onboarding/network-engineer-first-week.md:21-26`; `tutorials/provision-vpc-mtls-and-cedar-policy.md:6-9`; `docs/decisions/ADR-0700-ci-admission-live-apex.md:3215-3247` | Replace or subordinate Make commands behind Cargo/OpenTofu/Oya command surfaces. |
| P2 | D3 | No local SLO artifact exists despite p99 claims. | `faqs/network-engineer-faq.md:117-120`; §2 inventory | Add `slos/cloud-network.openslo.yaml` or equivalent governed SLO artifact. |
| P2 | D3 | No capacity model exists despite throughput and BGP claims. | `retired tenant_class adoption artifact:23-80`; §2 inventory | Add capacity model with bandwidth, route scale, flow scale, tenant limits, and context overlays. |
| P2 | D3 | No cost budget exists despite benchmark TCO claims. | `benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:60-71`; §2 inventory | Add per-context cost budget and OCI Always Free budget guardrails. |
| P2 | D2/D3 | Cross-service handoffs are operationally referenced but not governed in `cross-microservice-handoffs.md`. | `runbooks/ddos-mitigation-engagement.md:249-262`; `runbooks/mtls-handshake-failure-cascade.md:252-264`; §2 inventory | Add handoff doc for cloud-kms, cloud-iam, cloud-network-dns, cloud-iac, observability, and audit-chain. |
| P2 | D6 | On-prem and colo are hinted but not buildable. | `faqs/network-engineer-faq.md:89-93`; `retired tenant_class adoption artifact:55-80`; §2 inventory | Add per-context deployment matrix and IaC modules. |
| P2 | D1 | FAQ frames product as provider wrapper, which conflicts with portable semantics. | `faqs/network-engineer-faq.md:7-11`; `docs/decisions/ADR-0700-ci-admission-live-apex.md:2058-2059` | Rewrite as portable VPC-equivalent API with context-specific adapters. |
| P3 | D9 | No forbidden source-language files were present in target path. | §2 inventory; `docs/decisions/ADR-0700-ci-admission-live-apex.md:3085-3107` | Keep the path Markdown/Rust/OpenTofu-only and add a scan lane. |
| P3 | D3 | Runbooks are substantive and exceed line floor. | `runbooks/cross-cell-routing-stall.md:1-270`; `runbooks/ddos-mitigation-engagement.md:1-268`; `runbooks/mtls-handshake-failure-cascade.md:1-271`; `docs/standards/documentation-rigor.md:185-185` | Preserve runbook quality while wiring them to PRD/ARCH/SLO artifacts. |
| P3 | D3/D9 | Rust SDK reference aligns with Rust-strict direction. | `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:6-24`; `reference-implementations/provision-vpc-and-mtls-ingress-rust-sdk.md:156-191` | Link it from architecture as an example, not the canonical contract. |

- P0 count: 0.
- P1 count: 8.
- P2 count: 8.
- P3 count: 3.
- Total findings listed: 19.
- Highest-risk finding family: missing context/OpenTofu/OS/OCI ownership artifacts.
- Most misleading single statement: Crossplane/provider-wrapper framing in FAQ and migration docs.
- Most useful existing surface: incident runbooks.
- Most useful implementation evidence outside target path: Rust domain/API crates and OpenAPI bindings.

## §5 Open questions for Wave 14 aggregation

- OQ-01 Should `cloud-network-dns` remain a separate µservice while `cloud-network` owns DNS zone creation in its Rust/API surface, or should DNS ownership be split more sharply between network-private DNS and authoritative public DNS?
- OQ-02 Should `cloud-network` own CDN semantics, since the Rust domain crate claims CDN invariants, or should CDN become a separate µservice under the cloud edge family?
- OQ-03 Should the migration playbook keep AWS/Istio as the only source migration path, or should Wave 14 require GCP VPC, Azure VNet, on-prem, and colo migration playbooks for parity?
- OQ-04 Should `cloud-network` own IPAM directly, or should IPAM be a shared cloud-resource/region substrate consumed by network?
- OQ-05 Should `cloud-network` own Network Access Analyzer style formal reachability analysis, or should that live in observability/security with cloud-network as the graph source?
- OQ-06 Should traffic mirroring / packet capture live in `cloud-network`, `observability`, or `security`?
- OQ-07 Should DDoS mitigation be owned by `cloud-network` or by an edge/security µservice with cloud-network enforcement hooks?
- OQ-08 Should OCI Always Free demo_trial for network include the OCI load balancer's 10 Mbps Always Free allowance, or should load balancing be deferred to paid tenant_class except minimal ingress?
- OQ-09 Should demo_trial cross-context semantics allow shared VPC/network namespace outside OCI, or should demo_trial always mean context-minimal/no-dedicated-network?
- OQ-10 Should the OpenTofu modules be physically stored under each µservice or owned centrally by `cloud-iac` with per-µservice input schemas?
- OQ-11 Should Make snippets be globally retired from existing docs or preserved only as prose historical examples?
- OQ-12 Should benchmark docs be demoted to target benchmarks until signed evidence exists?
- OQ-13 Should route scale and flow scale quotas be globally governed by tenant tier, cell tier, or deployment context?
- OQ-14 Should mTLS be part of base `cloud-network` or part of the load balancer sub-surface only?
- OQ-15 Should Cilium remain named in product docs, or should local docs shift to capability descriptions to avoid implementation lock-in?
- OQ-16 Should BGP/FRR be documented as colo/on-prem implementation details only, or as part of all paid contexts?
- OQ-17 Should `oyatie-as-cloud-provider` use the same API contracts as guest deployments, or should it expose additional provider-facing APIs?
- OQ-18 Should the `cloud.network.dns.zone.create` surface move to `cloud-network-dns` registry ownership?
- OQ-19 Should the VPC/LB/DNS OpenAPI contracts be duplicated into the µservice folder or linked via manifest-only to avoid source-of-truth drift?
- OQ-20 Should Wave 14 aggregate all cloud-infra µservice docs into a single cloud substrate control-surface spec?

## Completion note

- Five constraint dimensions evaluated: yes.
- Audit result: `cloud-network` has real substance in runbooks, examples, and external Rust/OpenAPI implementation, but the ownership folder is missing core governance, deployment, OS, and IaC artifacts required by ADR-0328.
- The correct next action is not to delete current docs.
- The correct next action is to add ownership anchors and rewrite provider-wrapper sections under portable OpenTofu/Rust/six-context doctrine.

<!-- ORCHESTRATOR REPORT
  µservice: cloud-network
  deliverables_landed: microservices/cloud-network/coherence-audit-2026-05-20.md (668 lines); microservices/cloud-network/feature-parity-matrix-2026-05-20.md (411 lines); microservices/cloud-network/performance-benchmark-numbers-2026-05-20.md (422 lines); microservices/cloud-network/capability-tenant_class-deltas-vs-counterparts-2026-05-20.md (355 lines)
  inventory_files_seen: 10
  inventory_lines_read: 1942
  chat_history_matches_processed: 52
  findings_p0: 0
  findings_p1: 8
  findings_p2: 8
  findings_p3: 3
  top_3_counterparts_confirmed: AWS VPC / Google Cloud VPC / Azure Virtual Network
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1856
-->
