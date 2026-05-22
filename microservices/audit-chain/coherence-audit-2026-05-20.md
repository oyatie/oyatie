# audit-chain coherence audit — 2026-05-20

## Header anchor block
1. Canonical sequence anchor: `/Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2125` and `:3900-4235` define D-15 through D-20, including the six deployment contexts, OpenTofu-only IaC, OS matrix, Rust-strict policy, OCI Always Free sub-profile, severity rules, and completion criteria.
2. Master-plan machine anchor: `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json:704-868` defines `deployment_contexts`, `iac_substrate`, `supported_oses`, `language_policy`, and `oci_always_free`.
3. Service PRD anchor: `/Users/jasonlee/oyatie/microservices/audit-chain/PRD.md:18-361` was read for purpose, functional requirements, data residency, performance targets, capacity model, competitor claims, and open questions.
4. Service architecture anchor: `/Users/jasonlee/oyatie/microservices/audit-chain/ARCHITECTURE.md:1-754` was read for principals, Cedar gates, substrate binding, policy evaluation, deployment shape, observability, abuse defense, edge cases, and credential isolation.
5. Documentation rigor anchor: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md:133-224` plus `/Users/jasonlee/oyatie/docs/standards/brief-template.md:666-1185` and `:1520-1775` define intern-buildability, graph-traversability, µservice audit headers, and the anti-pattern ban on scaffold or line-count-only completion.

## Executive verdict
The audit-chain µservice has a coherent product core: cryptographic, tenant-scoped, append-only audit evidence with Merkle proofs, Ed25519 signatures, sealed roots, retention, export, verification, and incident response.
The product core is evidenced by `PRD.md:18-26`, `PRD.md:38-49`, `contracts/openapi/audit-chain.yaml:72-201`, `contracts/proto/audit-chain.proto:20-33`, and `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:11-13`.
The service is not coherent with ADR-0328 D-15 through D-19 for deployability.
The live service path lacks all six required per-context OpenTofu module directories.
The manifest lacks `deployment_contexts`, `supported_oses`, and an OCI Always Free declaration.
The IaC path is Helm and Kustomize only, with a current implementation plan pointing to a missing Terraform path.
No forbidden backend-language source files were found inside `microservices/audit-chain/`.
The highest-risk findings are P1, not P0, because audit-chain is Phase 1 foundation and not HR/Payroll, ERP, or CRM under ADR-0328 severity lines `D-20.111` through `D-20.117`.
The key remediation shape is not product reinvention; it is deployment-contract repair: manifest context rows, OpenTofu modules, OS manifest, OCI demo_trial Always Free tenant_class envelope, and provider-neutral HSM/storage abstractions.

## §1 µservice purpose summary
`audit-chain` is the evidence backbone for Oyatie.
Its PRD states that every state-changing event must receive non-repudiation evidence via hash-chain receipts, Merkle roots, Ed25519 signatures, and independently verifiable proof APIs (`PRD.md:18-26`).
The core functional requirements are emit, seal, verify, query, export, retention, key rotation, self-observability, and cross-microservice emission adapters (`PRD.md:38-49`).
The architecture decomposes that product into emission, sealing, verification, query, retention, adapters, and policy layers (`ARCHITECTURE.md:197-208`, `manifest.json:6-50`).
The REST contract exposes `/emit`, proof, verify, query, export, signed-root, and public-key surfaces (`contracts/openapi/audit-chain.yaml:215-220`, `contracts/proto/audit-chain.proto:20-33`).
The eventing contract publishes durable storage, seal minting, verification failure, retention application, and key rotation events (`contracts/asyncapi/audit-events.yaml:30-81`).
The service is tenant and pack scoped: OpenAPI requires `X-Scope-OrgID`, tenant id patterns, and pack variables (`contracts/openapi/audit-chain.yaml:20-27`, `:36-44`, `:72-91`).
Its failure posture treats chain-integrity faults as Sev-1 because this service carries compliance evidence for other services (`failure-modes.md:24-27`, `incident-response.md:24-35`).
Its tenant_class model distinguishes key custody, retention class, availability, and capacity while preserving cryptographic correctness across demo_trial and paid tenant_class (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:11-13`, `:114-120`).
Its industry posture is stronger than basic log collection because it adds tenant-visible verification and signed Merkle roots, but it still lacks parity docs against the currently assigned counterpart set because the existing competitor table targets Splunk, Datadog, and CloudTrail rather than CloudTrail, Google Cloud Audit Logs, and Microsoft Purview Audit (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137`, `PRD.md:243-260`).

## §2 Inventory snapshot
Total files seen before writing this audit batch: 251.
Total current lines audited before writing this audit batch: 62,817.
Inventory method: `find microservices/audit-chain -type f | sort` plus `wc -l`.
The table uses line count as the size field because the audit scope is documentation and source-read completeness.

| File | Size | Role | Coherent with purpose? |
|---|---:|---|---|
| `ARCHITECTURE.md` | 754 lines | service architecture | partial: strong product mechanics but current deployment shape is Helm/Kustomize, not six-context OpenTofu |
| `AUDIT-FINDINGS-2026-05-18.json` | 115 lines | prior audit evidence | partial: historical evidence, not current ADR-0328 wave result |
| `IP-001-storage-backend-iac.md` | 68 lines | IaC implementation plan | no: references missing `iac/terraform/oci-cloud-hsm-partition.tf` and Terraform-managed HSM |
| `IP-002-self-slo-manifest.md` | 151 lines | SLO implementation plan | yes: supports observability claim |
| `IP-003-emission-kernel.md` | 134 lines | emission kernel plan | yes: aligns to emit receipt purpose |
| `IP-004-emission-domain.md` | 41 lines | emission domain plan | partial: useful but thin |
| `IP-005-emission-usecase-and-adapter.md` | 97 lines | emission usecase plan | yes: aligns to inbound service emitters |
| `IP-006-sealing-kernel.md` | 71 lines | sealing kernel plan | yes: aligns to Merkle sealing |
| `IP-007-sealing-domain-merkle.md` | 73 lines | Merkle domain plan | yes: aligns to proof surface |
| `IP-008-sealing-adapter-hsm.md` | 81 lines | HSM adapter plan | partial: needs provider-neutral context expansion |
| `IP-009-sealing-adapter-postgres-s3.md` | 96 lines | storage adapter plan | partial: S3 wording needs cloud-storage abstraction for non-AWS contexts |
| `IP-010-sealing-worker-app.md` | 77 lines | worker app plan | yes: aligns to seal-cycle purpose |
| `IP-011-verification-stack.md` | 76 lines | verification plan | yes: aligns to proof verification |
| `IP-012-query-stack.md` | 75 lines | query plan | yes: aligns to auditor query |
| `IP-013-retention-cascade.md` | 95 lines | retention plan | yes: aligns to retention and DSR |
| `IP-014-cross-microservice-emission-adapter.md` | 56 lines | cross-service adapter plan | yes: aligns to shared substrate use |
| `IP-015-self-observability-slo-wiring.md` | 75 lines | observability plan | yes: aligns to SLOs |
| `IP-journey-j01-emergency-911-dispatch-emergency-classes.md` | 876 lines | journey-specific seal plan | yes: purpose-specific audit classes |
| `IP-journey-j02-healthcare-code-blue-ehr-break-glass-classes.md` | 817 lines | journey-specific seal plan | yes |
| `IP-journey-j03-minor-safety-chain-of-custody.md` | 803 lines | journey-specific seal plan | yes |
| `IP-journey-j05-anonymous-chain-of-custody.md` | 802 lines | journey-specific seal plan | yes |
| `IP-journey-j06-publisher-only-custody-seal.md` | 802 lines | journey-specific seal plan | yes |
| `IP-journey-j07-inheritance-seal.md` | 804 lines | journey-specific seal plan | yes |
| `IP-journey-j100-pack-rollout-first-action.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j101-dual-seal-events.md` | 865 lines | journey-specific seal plan | yes |
| `IP-journey-j102-dual-seal-events.md` | 862 lines | journey-specific seal plan | yes |
| `IP-journey-j103-dual-seal-events.md` | 862 lines | journey-specific seal plan | yes |
| `IP-journey-j104-dual-seal-events.md` | 863 lines | journey-specific seal plan | yes |
| `IP-journey-j105-dual-seal-events.md` | 863 lines | journey-specific seal plan | yes |
| `IP-journey-j106-dual-seal-events.md` | 860 lines | journey-specific seal plan | yes |
| `IP-journey-j107-dual-seal-events.md` | 862 lines | journey-specific seal plan | yes |
| `IP-journey-j118-dual-tenant-read-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j119-auction-award-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j12-surge-bypass-accountability.md` | 802 lines | journey-specific seal plan | yes |
| `IP-journey-j124-bypass-and-reason-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j125-dual-history-preservation.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j126-dual-tenant-emission-classes.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j129-warrant-query-emission.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j130-whistleblower-evidence-seal.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j131-region-local-seal.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j135-investigation-merkle-seal.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j138-corporate-audit-investigation-evidence-trail.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j139-internal-audit-cedar-permit-misuse-pattern-evidence.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j14-agent-action-seal.md` | 803 lines | journey-specific seal plan | yes |
| `IP-journey-j140-internal-audit-dlp-egress-evidence-trail.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j141-internal-audit-personal-tenant-boundary-deny-trail.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j143-epistemic-source-tagging.md` | 425 lines | journey-specific seal plan | yes |
| `IP-journey-j148-chain-of-custody-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j15-disclosure-custody-seal.md` | 801 lines | journey-specific seal plan | yes |
| `IP-journey-j18-ncmec-chain-of-custody.md` | 803 lines | journey-specific seal plan | yes |
| `IP-journey-j19-shamir-reconstitution-seal.md` | 802 lines | journey-specific seal plan | yes |
| `IP-journey-j32-anonymous-proof-seal.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j33-admin-action-seals.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j38-regulator-seal.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j43-hipaa-seal.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j44-consult-seal.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j45-record-correction-seal.md` | 420 lines | journey-specific seal plan | yes |
| `IP-journey-j51-procure-to-pay-classes.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j55-dispute-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j59-termination-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j60-promotion-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j61-hipaa-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j62-prescription-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j63-irb-hipaa-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j64-baa-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j65-dsar-proof.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j66-tax-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j67-chain-of-custody.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j68-seal-service.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j70-ai-human-override-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j71-fraud-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j73-slsa-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j75-revocation-seal.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j76-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j77-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j78-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j79-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j80-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j81-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j82-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j83-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j84-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j85-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j86-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j87-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j88-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j89-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j90-sealed-evidence-chain.md` | 430 lines | journey-specific seal plan | yes |
| `IP-journey-j91-us-msb-mtl-overlay.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j94-sox404-public-company-controls.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j98-au-privacy-apra-cps234.md` | 400 lines | journey-specific seal plan | yes |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | 400 lines | journey-specific seal plan | yes |
| `PHASE-01-AUDIT-CHAIN-SUBSTRATE.md` | 169 lines | phase plan | yes |
| `PRD.md` | 361 lines | product requirements | partial: product core strong, but PRD competitor and provider assumptions drift from current brief |
| `audit-event-class-registry.json` | 218 lines | event registry | yes |
| `backfill-replay.md` | 174 lines | replay operations | yes |
| `benchmarks/splunk-vs-datadog-vs-cloudtrail-vs-oyatie.md` | 98 lines | old benchmark comparison | partial: useful numbers, wrong top-3 union set |
| `capabilities/audit-emit.yaml` | 105 lines | capability record | yes |
| `capabilities/seal-mint.yaml` | 105 lines | capability record | yes |
| `capabilities/verify-merkle.yaml` | 95 lines | capability record | yes |
| `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model` | 137 lines | tenant_class model | partial: strong tenant_class envelopes, no OCI demo_trial Always Free reconciliation |
| `capacity-model.md` | 202 lines | capacity model | partial: useful math, OCI-centric assumptions |
| `catalog/oya-audit-chain-emission-adapter.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-api.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-app.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-domain.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-kernel.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-rest.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-sdk.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-emission-usecase.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-adapter-postgres.yaml` | 16 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-adapter.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-api.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-domain.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-kernel.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-rest.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-sdk.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-query-usecase.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-adapter.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-api.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-domain.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-kernel.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-usecase.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-retention-cascade-worker.yaml` | 15 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-adapter-hsm.yaml` | 22 lines | catalog record | partial: HSM adapter needs context-neutral provider mapping |
| `catalog/oya-audit-chain-sealing-adapter-postgres.yaml` | 21 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-adapter-s3.yaml` | 21 lines | catalog record | partial: S3 adapter needs cloud-storage abstraction language |
| `catalog/oya-audit-chain-sealing-adapter.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-api.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-app.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-domain.yaml` | 20 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-kernel.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-usecase.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-sealing-worker.yaml` | 20 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-adapter.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-api.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-domain.yaml` | 20 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-kernel.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-rest.yaml` | 17 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-sdk.yaml` | 20 lines | catalog record | yes |
| `catalog/oya-audit-chain-verification-usecase.yaml` | 17 lines | catalog record | yes |
| `coherence-audit-2026-05-20.md` | 2655 lines | stale prior audit doc | no: stale inventory, stale scope, and missing current four-doc wave shape |
| `competitor-parity-matrix.md` | 133 lines | old competitor matrix | partial: useful, but not current top-3 union coverage |
| `compliance.md` | 1061 lines | compliance posture | partial: rich compliance, but on-prem and OpenTofu claims need per-context evidence |
| `contracts/asyncapi/audit-events.yaml` | 197 lines | AsyncAPI contract | yes |
| `contracts/openapi/audit-chain.yaml` | 388 lines | REST contract | yes |
| `contracts/proto/audit-chain.proto` | 220 lines | gRPC contract | yes |
| `cost-budget.md` | 133 lines | FinOps model | partial: OCI-dominant costing, no Always Free demo_trial tenant_class reconciliation |
| `cross-microservice-handoffs.md` | 259 lines | handoff matrix | yes |
| `dashboards/emission-rate.json` | 128 lines | Grafana dashboard | yes |
| `dashboards/seal-latency.json` | 115 lines | Grafana dashboard | yes |
| `dashboards/verification-failure-rate.json` | 109 lines | Grafana dashboard | yes |
| `decisions/ADR-AUD-001-per-cell-hash-tree-vs-multi-region-merkle-strategy.md` | 229 lines | service ADR | yes |
| `dpia.md` | 238 lines | privacy impact assessment | yes |
| `failure-modes.md` | 283 lines | failure catalog | yes |
| `faqs/compliance-officer-faq.md` | 114 lines | FAQ | yes |
| `iac/helm/audit-storage/Chart.yaml` | 10 lines | Helm chart | partial: deployment helper, not OpenTofu context module |
| `iac/helm/audit-storage/templates/deployment.yaml` | 78 lines | Kubernetes deployment | partial: deployment helper, not OpenTofu context module |
| `iac/helm/audit-storage/templates/networkpolicy.yaml` | 75 lines | network policy | partial: useful runtime resource, not context IaC |
| `iac/helm/audit-storage/values.yaml` | 58 lines | Helm values | no: points to `iac/terraform/` for bucket creation |
| `iac/helm/hsm-operator/Chart.yaml` | 10 lines | Helm chart | partial: deployment helper, not OpenTofu context module |
| `iac/helm/hsm-operator/values.yaml` | 53 lines | HSM Helm values | partial: OCI HSM specific without all context alternatives |
| `iac/helm/postgres/Chart.yaml` | 14 lines | Helm chart | partial: deployment helper, not OpenTofu context module |
| `iac/helm/postgres/values.yaml` | 55 lines | Postgres Helm values | partial |
| `iac/kustomize/base/kustomization.yaml` | 34 lines | Kustomize base | partial |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | 45 lines | Kustomize overlay | partial |
| `incident-response.md` | 270 lines | incident response | yes |
| `manifest.json` | 418 lines | service manifest | partial: strong component/dependency roster, missing ADR-0328 context and OS fields |
| `migration-playbooks/from-splunk-audit.md` | 184 lines | migration playbook | yes |
| `multi-region.md` | 232 lines | multi-region model | partial: pack model rich, OCI-region wording needs provider-neutral review |
| `onboarding/compliance-officer-first-week.md` | 214 lines | onboarding | yes |
| `packs/EU-AI-Act.md` | 206 lines | compliance pack | yes |
| `packs/GDPR.md` | 207 lines | compliance pack | yes |
| `packs/HIPAA.md` | 207 lines | compliance pack | yes |
| `packs/KR-PIPA.md` | 207 lines | compliance pack | yes |
| `packs/SOC2.md` | 207 lines | compliance pack | yes |
| `policy/auditor-scope.cedar` | 203 lines | Cedar policy | yes |
| `policy/cedar-canonical-imports.cedar` | 9 lines | Cedar import file | yes |
| `policy/ci-scope.cedar` | 216 lines | Cedar policy | yes |
| `policy/data-residency-enforcement.cedar` | 32 lines | Cedar policy | yes |
| `policy/data-residency.md` | 177 lines | data residency policy | partial: rich pack policy, OCI-region assumptions need context mapping |
| `policy/dual-tenant-emit.cedar` | 24 lines | Cedar policy | yes |
| `policy/j51-procure-to-pay-classes.cedar` | 25 lines | Cedar policy | yes |
| `policy/j55-dispute-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j59-termination-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j60-promotion-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j61-hipaa-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j62-prescription-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j63-irb-hipaa-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j64-baa-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j65-dsar-proof.cedar` | 25 lines | Cedar policy | yes |
| `policy/j66-tax-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j67-chain-of-custody.cedar` | 25 lines | Cedar policy | yes |
| `policy/j68-seal-service.cedar` | 25 lines | Cedar policy | yes |
| `policy/j70-ai-human-override-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j71-fraud-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j73-slsa-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j75-revocation-seal.cedar` | 25 lines | Cedar policy | yes |
| `policy/j76-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j77-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j78-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j79-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j80-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j81-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j82-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j83-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j84-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j85-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j86-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j87-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j88-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j89-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/j90-sealed-evidence-chain.cedar` | 25 lines | Cedar policy | yes |
| `policy/public-read.cedar` | 132 lines | Cedar policy | yes |
| `policy/region-local-pi-read.cedar` | 17 lines | Cedar policy | yes |
| `policy/retention-matrix.yaml` | 61 lines | retention config | yes |
| `policy/seal-integrity.md` | 272 lines | integrity policy | yes |
| `policy/tenant-scope.cedar` | 165 lines | Cedar policy | yes |
| `policy/warrant-emit.cedar` | 16 lines | Cedar policy | yes |
| `policy/whistleblower-seal.cedar` | 17 lines | Cedar policy | yes |
| `reference-implementations/emit-and-verify-rust-sdk.md` | 277 lines | Rust SDK reference | yes |
| `runbooks/audit-chain-restart.md` | 281 lines | runbook | yes |
| `runbooks/audit-export.md` | 281 lines | runbook | yes |
| `runbooks/chain-replay-from-snapshot-protocol.md` | 281 lines | runbook | yes |
| `runbooks/hsm-key-rotation.md` | 281 lines | runbook | yes |
| `runbooks/merkle-root-discrepancy-investigation.md` | 281 lines | runbook | yes |
| `runbooks/merkle-seal-recovery.md` | 281 lines | runbook | yes |
| `runbooks/regulator-evidence-export-failure.md` | 281 lines | runbook | yes |
| `runbooks/retention-cascade.md` | 281 lines | runbook | yes |
| `runbooks/signature-verification-failure.md` | 281 lines | runbook | yes |
| `scorecards/overrides.json` | 45 lines | scorecard config | yes |
| `sdk-plan.md` | 124 lines | SDK plan | partial: must preserve generated-only non-Rust language policy |
| `security/threat-model.md` | 426 lines | security threat model | yes |
| `slos/chain-of-custody-integrity-correctness.openslo.yaml` | 52 lines | OpenSLO | yes |
| `slos/evidence-export-freshness.openslo.yaml` | 48 lines | OpenSLO | yes |
| `slos/merkle-chain-verification-latency.openslo.yaml` | 48 lines | OpenSLO | yes |
| `slos/seal-cycle-latency.openslo.yaml` | 50 lines | OpenSLO | yes |
| `slos/seal-storage-availability.openslo.yaml` | 48 lines | OpenSLO | yes |
| `slos/seal-write-availability.openslo.yaml` | 53 lines | OpenSLO | yes |
| `slos/seal-write-latency.openslo.yaml` | 49 lines | OpenSLO | yes |
| `test-plans/contract-test-strategy.md` | 311 lines | test plan | yes |
| `test-plans/integration-test-strategy.md` | 312 lines | test plan | yes |
| `test-plans/unit-test-strategy.md` | 347 lines | test plan | yes |
| `threat-model.md` | 629 lines | top-level threat model | partial: has current Terraform-managed wording |
| `tutorials/regulator-evidence-export.md` | 220 lines | tutorial | yes |

## §3 9-dimension audit

### §3.1 Dimension 1 — internal coherence
Dimension verdict: FINDING.
Severity rollup: P1 for deployment contradictions; P2 for stale/scaffold documentation; P3 for cosmetic naming drifts.
Purpose coherence passes: the PRD, contracts, tenant_class model, failure catalog, and runbooks all describe one service boundary: emit receipts, seal roots, verify proofs, query events, export evidence, and manage retention.
Internal cross-reference: `PRD.md:178-190` names ports that map to emission, sealing, verification, query, retention, and SDK catalog records; those catalog records exist under `catalog/`.
Internal cross-reference: `PRD.md:211-227` names produced and consumed events that map to AsyncAPI channels in `contracts/asyncapi/audit-events.yaml:30-81`.
Internal cross-reference: `cross-microservice-handoffs.md:12-18` cites the OpenAPI, AsyncAPI, proto, and Cedar policy paths; all cited local contract and policy paths exist.
Internal cross-reference: `capacity-model.md:11-15` cites cost budget, multi-region, data residency, and `/specs/audit-chain-merkle-ed25519.json`; the three service-local docs exist, but the root spec was not changed or validated in this audit.
Internal cross-reference: `failure-modes.md:11-17` cites threat-model, dpia, seal-integrity, data-residency, incident-response, and runbooks; all cited service-local targets exist.
Internal cross-reference: `incident-response.md:11-17` cites threat-model, dpia, compliance, failure-modes, multi-region, and runbooks; all cited service-local targets exist.
Internal cross-reference: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:116-120` cites contracts and kernels; contracts exist, and Rust crates exist under the workspace `crates/` path rather than under the µservice path.
Internal cross-reference: `manifest.json:402-413` declares dependencies on identity, docs, tenancy, observability, cloud-secrets, network, ontology, detection, cell, and cloud-iac; all are service names in the broader repo, but this manifest does not declare deployment context seams for those dependencies.
Broken or wrong-direction reference: `IP-001-storage-backend-iac.md:38` names `microservices/audit-chain/iac/terraform/oci-cloud-hsm-partition.tf`; no such file exists and Terraform is forbidden by ADR-0328 D-20 lines `3900-3908`.
Broken or wrong-direction reference: `iac/helm/audit-storage/values.yaml:4` says actual bucket creation happens via OpenTofu under `iac/terraform/`; that directory is absent and its name is wrong under OpenTofu-only doctrine.
Contradiction probe 1: PRD claims adapter-hsm to OCI Cloud-HSM (`PRD.md:67`, `PRD.md:154`, `PRD.md:182`), while ADR-0328 requires all six contexts and provider adapters not product leakage (`ADR-0328:2038-2050`, `:2079-2096`).
Contradiction probe 2: Architecture says current IaC manifests are Helm/Kustomize (`ARCHITECTURE.md:445-456`), while ADR-0328 requires OpenTofu modules per supported context (`ADR-0328:3908-3939`, `:4214-4224`).
Contradiction probe 3: Manifest declares component and dependency details but no `deployment_contexts` array despite ADR-0328 D-15.102 requiring one (`manifest.json:1-418`, `ADR-0328:2079-2084`).
Contradiction probe 4: Manifest has tenant_class policy `T0`, `T2`, `T3` (`manifest.json:343-347`), while the service tenant_class model uses demo_trial and paid tenant_class (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-110`).
Contradiction probe 5: The existing old competitor posture compares Splunk and Datadog (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137`, `benchmarks/splunk-vs-datadog-vs-cloudtrail-vs-oyatie.md:1-98`), but the current audit brief assigns AWS CloudTrail, Google Cloud Audit Logs, and Microsoft Purview Audit from chat line `8f603fc7...jsonl:15698`.
Contradiction probe 6: `capacity-model.md:99` assumes OCI Cloud-HSM partition baseline, but `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:49` names AWS CloudHSM or Thales Luna for paid tenant_class; the service lacks a unifying provider-neutral HSM profile.
Contradiction probe 7: `cost-budget.md:22-32` uses OCI pricing as the cost substrate, while ADR-0328 D-15 expects separate billing and state behavior per context, including AWS guest, on-prem, colo, and Oyatie provider.
Contradiction probe 8: `ARCHITECTURE.md:3` says the file is a stub placeholder from a Wave-3-C anchor sweep, yet the file also asserts production-grade mechanics throughout; this is a P2 documentation-quality defect because readers cannot tell which sections are normative.
Contradiction probe 9: `PRD.md:47` allows future TS/Python bindings, while ADR-0328 D-20.107 requires distinguishing generated SDK clients from authored application logic and D-20.92/D-20.94 forbid Python/TypeScript application logic.
Contradiction probe 10: `manifest.json:414` describes the failure domain as "north-south edge admission and routing plane"; audit-chain's real failure domain is evidence integrity and sealing, not edge routing.
Internal coherence summary: product semantics are coherent; deployment, tenant_class vocabulary, and provider abstraction are not yet coherent.

### §3.2 Dimension 2 — outbound cross-references
Dimension verdict: FINDING.
Outbound references to service-local documents mostly resolve.
Outbound references to canonical ADRs are numerous and generally point to existing root ADR identifiers, but this audit did not edit root ADRs.
Outbound reference to `/specs/audit-chain-merkle-ed25519.json` appears in `capacity-model.md:15`; the service uses it as a capacity source.
Outbound reference to `docs/standards/documentation-rigor.md` is current for this audit because that standard applies retroactively to µservice docs at lines `40-56`.
Outbound reference to `docs/decisions/ADR-0328...` is mandatory because current chat instruction names D-15 through D-20.
Outbound references from `cross-microservice-handoffs.md:20-55` to `api-gateway`, `application`, `cell`, `cloud-iac`, `cloud-secrets`, `developer-sdk`, `payments`, `compliance`, `ops-dashboard-control-center`, `observability`, and `identity` are domain-correct because audit-chain is a shared substrate.
Outbound references from `manifest.json:402-413` overlap that same dependency graph and add docs, tenancy, ontology, detection, network, and cloud-iac.
Reference to other µservices from docs search: `docs/standards/logging-tracing.md:73` requires per-tenant audit-chain integration.
Reference to this service from root specs search: `specs/multi-region-disposition-canonical.json:14` lists audit-chain among globally relevant multi-region examples.
Reference to this service from quality lanes: `registry/quality/lanes.yaml:682-689` defines an audit-chain seal coverage lane.
Reference to this service from placeholder debt: `registry/placeholder-debt/adr-follow-ups.yaml:247-258` tracks production implementation and seal-coverage promotion debt for the shared audit client kernel.
Reference to this service from other µservices: `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md` and `microservices/tenancy/IP-011-audit-chain-integration.md` exist in search results, confirming inbound dependence.
Reference to this service from compliance: `microservices/compliance/IP-005-audit-chain-seal-coverage.md` exists in search results, confirming audit-chain is a compliance evidence dependency.
Reference to this service from ontology: `microservices/ontology/IP-010-audit-chain-merkle-ed25519.md` exists in search results, confirming ontology integration.
Reference to this service from plugin-app-store: `microservices/plugin-app-store/decisions/ADR-PAS-0007-per-plugin-action-audit-trail-seals-via-audit-chain-µservice.md` exists in search results, confirming product-surface dependence.
Orphan-risk reference: root user journeys reference audit-chain heavily, but the audit-chain service manifest does not include a reverse-index of inbound callers beyond dependencies and handoff docs.
Missing reverse reference: `docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/story.md:119-171` references audit-chain obligations, but the audit-chain path has only journey IPs, not a manifest-level journey map.
Missing reverse reference: `docs/user-journeys/j101-multi-tier-supply-chain-formation/story.md:1998-2003` defines service closure expectations, but `manifest.json` does not list that user journey as an inbound contract.
External chat provenance: `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13449` shows a previous audit-chain ownership prompt; this explains why a stale audit file already existed.
External chat provenance: `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:14069` says the later wave is audit-only and requires four docs; this matches the current assignment and supersedes the older remediation-oriented prompt.
External chat provenance: `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15698` confirms the top-3 counterpart set: AWS CloudTrail, Google Cloud Audit Logs, and Microsoft Purview Audit.
Outbound cross-reference summary: graph density is high, but reverse-reference indexing and current-wave counterpart alignment are incomplete.

### §3.3 Dimension 3 — substance bar
Dimension verdict: FINDING.
The service can be understood by a cold intern as a product: emit an event, persist it, assign a period, seal with Merkle roots, sign roots, verify proofs, query, export, retain, and rotate keys.
The service cannot yet be built end-to-end by a cold intern for all six deployment contexts.
Strong buildability evidence: `contracts/openapi/audit-chain.yaml:72-201` defines event, receipt, proof, signed root, verify, query, export, and error shapes.
Strong buildability evidence: `contracts/proto/audit-chain.proto:20-33` gives a parallel gRPC service with the same operations.
Strong buildability evidence: `cross-microservice-handoffs.md:20-55` names inbound and outbound callers with API names and data shapes.
Strong buildability evidence: `failure-modes.md:30-47` indexes failure modes and severity.
Strong buildability evidence: `incident-response.md:78-91` gives a staged lifecycle.
Strong buildability evidence: `capacity-model.md:28-36` provides capacity inputs and formulas.
Strong buildability evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-110` gives tenant_class hardware, retention, and SLO postures.
Weak section: `ARCHITECTURE.md:3` explicitly marks the architecture as an anchor-sweep stub requiring expansion, which undermines normative confidence.
Weak section: `PRD.md:323-331` lists open questions, including an S-tier capacity-model trigger; open questions are acceptable only if they do not block buildability, but this service uses capacity as a production gate.
Weak section: `IP-001-storage-backend-iac.md:24-38` gives IaC shape but points to a missing Terraform path; an intern following it builds the wrong substrate.
Weak section: `iac/helm/audit-storage/values.yaml:4-47` delegates actual bucket creation to OpenTofu under `iac/terraform/`, but the module is absent.
Weak section: `manifest.json:343-347` uses tenant_class names incompatible with demo_trial/paid tenant_class.
Weak section: `manifest.json:414` gives an edge-routing failure domain that does not match the audit evidence domain.
Missing API detail: the OpenAPI contract does not declare all rate limits, search job limits, export bundle pagination, retention lock legal hold, or admin policy operations needed for counterpart parity.
Missing data model: there is no service-local SQL migration or explicit table schema under `microservices/audit-chain/`, even though Postgres is central in `capacity-model.md:57-69` and `failure-modes.md:100-111`.
Missing failure semantics: the service has failure modes, but no per-context difference for on-prem disconnected mode, colo remote-hands delay, AWS guest KMS/CloudHSM fallback, or OCI Always Free capacity breach.
Missing CI lane spec: `PRD.md:198-205` names CI lanes, but there is no service-local supported OS manifest or context lane matrix.
Missing tenant onboarding: ADR-0328 requires `tofu init -> tofu plan -> tofu apply` per context, but the service path has no context modules and no per-context README.
Missing benchmark provenance: old benchmarks compare Splunk/Datadog/CloudTrail and a paid tenant_class on-prem setup, but the current audit needs CloudTrail, Google Cloud Audit Logs, and Purview Audit.
Substance-bar summary: buildability is good for logical service behavior and insufficient for ADR-0328 deployment, OS, and parity obligations.

### §3.4 Dimension 4 — canonical-direction alignment
Dimension verdict: FINDING.
Multi-context alignment: drifted-fixable.
Multi-context evidence: ADR-0328 D-15 requires all six contexts for Phase 1 by default (`ADR-0328:2116-2119`), and the master plan lists the six context ids (`specs/master-plan-sequencing.json:704-745`).
Multi-context service evidence: `manifest.json:1-418` contains no `deployment_contexts` array.
Multi-context service evidence: `find microservices/audit-chain/iac` shows Helm and Kustomize only.
OpenTofu alignment: incoherent.
OpenTofu evidence: master plan requires engine `OpenTofu`, forbids Terraform/Pulumi/CloudFormation as primary, requires module signing, state backends, and tenant onboarding (`specs/master-plan-sequencing.json:747-775`).
OpenTofu service evidence: `IP-001-storage-backend-iac.md:38` names a Terraform-managed HSM partition path.
OpenTofu service evidence: `threat-model.md:466` says Postgres role grants are Terraform-managed.
OpenTofu service evidence: `iac/helm/audit-storage/values.yaml:4` points to `iac/terraform/`.
OS support alignment: drifted-fixable.
OS evidence: master plan requires Tier-1, Tier-2 test-only, out-of-scope explicit, architecture matrix, and CI lane policy (`specs/master-plan-sequencing.json:777-815`).
OS service evidence: no `supported-oses.json` was found under `microservices/audit-chain/`.
Rust-strict alignment: aligned for current files.
Rust evidence: forbidden-source scan for `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, and `.fs` returned no files under `microservices/audit-chain/`.
Rust drift evidence: `PRD.md:47` and `sdk-plan.md` must be constrained to generated SDKs only, because ADR-0328 D-20.107 requires generated-client provenance.
OCI Always Free alignment: drifted-fixable.
OCI evidence: master plan requires `iac/oci-guest/always-free/` and maps demo_trial tenant_class OCI to Always Free (`specs/master-plan-sequencing.json:857-867`).
OCI service evidence: no `iac/oci-guest/always-free/` path exists.
OCI service evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` defines demo_trial tenant_class as 3 ingest nodes, 2 sealer nodes, Postgres, and SeaweedFS; this exceeds OCI Always Free and does not reconcile which features downshift on OCI demo_trial tenant_class.
Canonical-direction summary: service logic aligns with audit-chain doctrine; infrastructure and portability doctrine do not.

### §3.5 Dimension 5 — industry-counterpart parity
Dimension verdict: PARTIAL.
Headline finding: audit-chain is partial against the union coverage of AWS CloudTrail, Google Cloud Audit Logs, and Microsoft Purview Audit.
Counterpart 1, AWS CloudTrail, covers management events, data events, network activity events, event history, organization trails, CloudTrail Lake, Insights, integrity validation, S3 delivery, CloudWatch/EventBridge integration, KMS encryption, advanced event selectors, delegated administration, and audit event export.
Counterpart 2, Google Cloud Audit Logs, covers Admin Activity, Data Access, System Event, Policy Denied audit logs, log routing, log buckets, log views, CMEK, sinks, Log Analytics, BigQuery linkage, access transparency adjacency, retention, IAM controls, and organization/folder/project scope.
Counterpart 3, Microsoft Purview Audit, covers standard and premium audit, broad Microsoft 365 activity sources, audit search, audit retention policies, high-value user priority, long-term retention, export, Audit Search API, Management Activity API integration, forensic investigation workflows, and role-based access.
Audit-chain present: cryptographic verification, Merkle proof, Ed25519 signed roots, tenant-visible verify API, retention cascade, export bundles, public key endpoint, failure incident posture, self-observability, and many compliance pack overlays.
Audit-chain missing: first-class admin console search job semantics, saved searches, delegated admin UX, built-in anomaly insight detection comparable to CloudTrail Insights, turnkey ingestion from all platform control planes, default retention parity with Purview Premium, explicit activity taxonomy parity for Microsoft 365-style workloads, Access Transparency-like human support access logs, and Google-style log bucket/view governance.
Audit-chain additive: tenant-visible Merkle proof API and per-tenant signing keys at paid tenant_class are stronger than counterpart audit logs that primarily store and query provider-collected activity.
Existing service competitor evidence is stale because `PRD.md:243-260` and `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137` do not include Microsoft Purview Audit or Google Cloud Audit Logs as equal union counterparts.
Industry parity summary: audit-chain can exceed counterparts on verifiability but lacks broad operational audit-log feature surface and current counterpart mapping.

### §3.6 Dimension 6 — multi-context deployment support
Dimension verdict: FINDING.
Severity: P1 because audit-chain is an in-scope Phase 1 foundation service and the missing contexts would make deployment claims false.
Required contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` per `specs/master-plan-sequencing.json:704-745`.
Per-context status: `oyatie-public-cloud` is not supported by service-local OpenTofu evidence; `iac/oyatie-public-cloud/` is absent.
Per-context status: `guest-on-aws` is not supported by service-local OpenTofu evidence; `iac/guest-on-aws/` is absent.
Per-context status: `guest-on-oci` is not supported by service-local OpenTofu evidence; `iac/oci-guest/` is absent.
Per-context status: `on-prem` is not supported by service-local OpenTofu evidence; `iac/on-prem/` is absent.
Per-context status: `colo` is not supported by service-local OpenTofu evidence; `iac/colo/` is absent.
Per-context status: `oyatie-as-cloud-provider` is not supported by service-local OpenTofu evidence; the ADR names `iac/oyatie-iaas/`, while the user brief names `oyatie-as-cloud-provider`; no local module exists for either spelling.
Correctly N/A contexts: none documented.
Missing N/A reason fields: no context has `reason`, `missing_primitives`, `customer_impact`, `remediation_owner`, or `target_revisit_gate` as required by ADR-0328 D-15.103 (`ADR-0328:2082-2084`).
Provider API leakage: current PRD and capacity docs name OCI Cloud-HSM and OCI Object Storage directly (`PRD.md:67`, `capacity-model.md:99`, `cost-budget.md:22-32`).
Provider API leakage: tenant_class model names AWS CloudHSM as paid tenant_class option but lacks AWS guest module evidence (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:49`).
Tenant onboarding gap: no service-local text proves `tofu init -> tofu plan -> tofu apply` for every context, despite ADR-0328 D-15.107 and D-20.157.
Data residency gap: `policy/data-residency.md` and `multi-region.md` model packs, but they do not map storage state backends per six deployment contexts.
Network seam gap: no per-context ingress, DNS, certificate, WAF, rate limit, or tenant-isolation matrix exists in this service path.
IAM seam gap: HSM/KMS identity is not expressed through a per-context `cloud-iam`/`cloud-kms` adapter map.
Observability seam gap: SLOs exist, but no per-context observability export path is declared.
Billing seam gap: `cost-budget.md` is OCI-oriented and does not map AWS CUR, on-prem allocation, colo rack/power, or Oyatie provider metering.
Dimension 6 remediation hint: add service-local context manifest rows, per-context deployment assumptions, and six OpenTofu module directories or explicit N/A rows.

### §3.7 Dimension 7 — OpenTofu IaC coverage
Dimension verdict: FINDING.
Severity: P1.
IaC inventory: `iac/helm/audit-storage/Chart.yaml`, `templates/deployment.yaml`, `templates/networkpolicy.yaml`, and `values.yaml`.
IaC inventory: `iac/helm/hsm-operator/Chart.yaml` and `values.yaml`.
IaC inventory: `iac/helm/postgres/Chart.yaml` and `values.yaml`.
IaC inventory: `iac/kustomize/base/kustomization.yaml` and `iac/kustomize/overlays/pack-kr/kustomization.yaml`.
Missing OpenTofu directory: `iac/oyatie-public-cloud/`.
Missing OpenTofu directory: `iac/guest-on-aws/`.
Missing OpenTofu directory: `iac/oci-guest/`.
Missing OpenTofu directory: `iac/oci-guest/always-free/`.
Missing OpenTofu directory: `iac/on-prem/`.
Missing OpenTofu directory: `iac/colo/`.
Missing OpenTofu directory: `iac/oyatie-as-cloud-provider/` or canonical `iac/oyatie-iaas/`.
Required files absent for every context: `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md`.
Version pinning evidence: absent.
Provider pinning evidence: absent.
Sigstore and cosign module signing evidence: absent.
State backend mapping: absent at service-local module level; master plan expected S3+DynamoDB, OCI Object Storage+Autonomous DB lock, MinIO+lock table, internal OCI, and internal cloud-storage.
Forbidden engine reference: `IP-001-storage-backend-iac.md:38` says Terraform-managed HSM partition lifecycle.
Forbidden engine reference: `threat-model.md:466` says Postgres role grants are Terraform-managed.
Forbidden path reference: `iac/helm/audit-storage/values.yaml:4` says bucket creation happens via OpenTofu under `iac/terraform/`.
Forbidden pattern scan: no `null_resource`, `local-exec`, hand-edited `tfstate`, or SSH provisioner pattern was found in live files, but the absence is weak because no OpenTofu modules exist.
CloudFormation/Pulumi scan: no current CloudFormation or Pulumi references were found in service-local scope.
Tenant onboarding command evidence: absent in service-local IaC docs.
`cloud-iac` orchestration evidence: dependency exists in `manifest.json:412`, but no plan/apply integration contract exists for audit-chain modules.
Dimension 7 remediation hint: replace Terraform references with OpenTofu, create per-context module directories, pin versions/providers, add state backends, add sigstore/cosign evidence, and keep Helm/Kustomize as rendered runtime artifacts rather than provisioning source of truth.

### §3.8 Dimension 8 — OS support matrix
Dimension verdict: FINDING.
Severity: P1 because the service claims deployability but has no service-local OS manifest.
Manifest evidence: no `microservices/audit-chain/supported-oses.json` exists.
Manifest field evidence: no `supported_oses` field exists in `manifest.json:1-418`.
Tier-1 Talos status: missing row.
Tier-1 RHEL 9.x+ status: missing row.
Tier-1 Oracle Linux 9.x+ status: missing row.
Tier-1 SLES 15 SP6+ status: missing row.
Tier-1 Ubuntu 24.04 LTS+ status: missing row.
Tier-1 Debian 13+ status: missing row.
Tier-1 Rocky 9.x+ status: missing row.
Tier-1 AlmaLinux 9.x+ status: missing row.
Tier-1 CentOS Stream 10+ status: missing row.
Tier-1 Amazon Linux 2023+ status: missing row.
Tier-1 Flatcar status: missing row.
Tier-1 Photon 5.x+ status: missing row.
Tier-1 macOS Apple Silicon M5+ status: missing row and likely needs local tooling decision.
Tier-2 `linux-ppc64le` status: missing test-only row.
Tier-2 `linux-s390x` status: missing test-only row.
Out-of-scope `macos-intel`: no explicit exclusion.
Out-of-scope `macos-apple-silicon-pre-m5`: no explicit exclusion.
Out-of-scope `freebsd`: no explicit exclusion.
Out-of-scope `openbsd`: no explicit exclusion.
Out-of-scope `windows-server`: no explicit exclusion.
Out-of-scope `solaris`: no explicit exclusion.
Architecture matrix: absent for `linux/amd64`, `linux/arm64`, `darwin/arm64-m5+`, `linux/ppc64le-test-only`, and `linux/s390x-test-only`.
Package formats: no RPM mapping for RHEL/Oracle Linux/SLES/Rocky/Alma/CentOS Stream/Amazon Linux/Photon.
Package formats: no DEB mapping for Ubuntu/Debian.
Package formats: container images are implied by Kubernetes/Helm but not declared per OS.
Package formats: no Talos extension decision.
Package formats: no Flatcar ignition or extension decision.
Package formats: no macOS `.pkg` or Homebrew decision for local tools.
CI lane policy: no blocking Tier-1 lanes or soft Tier-2 lanes declared.
Python runtime dependency: no forbidden Python source was found; this is aligned.
Rust portability evidence: workspace crates exist, but no service-local OS portability manifest or static-linking posture exists.
Dimension 8 remediation hint: add `supported-oses.json` with all Tier-1 and Tier-2 rows, explicit exclusions, package formats, architecture matrix, and CI lane names.

### §3.9 Dimension 9 — Rust-strict language coverage
Dimension verdict: PASS-WITH-P2-DOC-GAP.
Forbidden-language scan result: no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files were found under `microservices/audit-chain/`.
Authorized non-Rust files present: `.md` documentation, `.json` manifests/dashboards, `.yaml` contracts/config/catalog, `.proto` contract, `.cedar` policy, and `.openslo.yaml` SLOs.
No backend JavaScript application logic was found.
No backend TypeScript application logic was found.
No Python application logic or scripts were found.
No Ruby, Go, Java, Scala, Groovy, PHP, or F# files were found.
No frontend directory exists under `microservices/audit-chain/`; therefore Swift/Kotlin/WinUI3/Leptos frontend allowlist is not exercised.
Workspace Rust implementation evidence exists outside the µservice path in `crates/oya-audit-chain-domain`, `crates/oya-audit-chain-file-adapter`, `crates/oya-audit-chain-usecase`, and `crates/oya-shared-audit-chain-client-kernel`.
Representative Rust source inventory from search: `crates/oya-audit-chain-domain/src/lib.rs`, `crates/oya-audit-chain-domain/src/merkle_tree.rs`, `crates/oya-audit-chain-usecase/src/lib.rs`, and `crates/oya-audit-chain-file-adapter/src/lib.rs`.
Canonical build invocation required by ADR-0328 is `cargo build --workspace --release --all-features --locked`.
Service-local build docs do not consistently state that canonical invocation.
SDK wording risk: `PRD.md:47` says future TS/Python bindings; this must be rewritten in follow-up to "generated SDK output only with provenance" or backed by a per-µservice exception ADR.
SDK plan risk: if non-Rust SDKs are emitted, generated-client provenance must be recorded and no backend runtime may depend on them.
Rust-strict summary: current service-local files do not violate the file-extension ban, but docs still need generated-SDK boundary language and canonical build invocation.

## §4 Findings summary
| Severity | Dimension | Short description | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | D6 | Missing `deployment_contexts` manifest for all six required contexts | `manifest.json:1-418`; `ADR-0328:2079-2084` | Add six context rows or complete N/A rows |
| P1 | D6 | No per-context IaC directories for any required context | `specs/master-plan-sequencing.json:704-745`; file inventory | Add `iac/<context>/` modules |
| P1 | D6 | OCI HSM/storage assumptions leak into provider-agnostic deployment doctrine | `PRD.md:67`; `capacity-model.md:99`; `cost-budget.md:22-32` | Define provider-neutral HSM/storage ports per context |
| P1 | D7 | Terraform-managed HSM partition path is prescribed | `IP-001-storage-backend-iac.md:38`; `ADR-0328:3900-3939` | Replace with OpenTofu module path and language |
| P1 | D7 | Helm values point to absent `iac/terraform/` bucket creation path | `iac/helm/audit-storage/values.yaml:4`; `:47` | Move provisioning docs to OpenTofu context module |
| P1 | D7 | Terraform-managed Postgres grant wording remains in threat model | `threat-model.md:466` | Rewrite as OpenTofu-managed or cloud-iac-managed |
| P1 | D7 | Missing OpenTofu version/provider pins and state backend evidence | `specs/master-plan-sequencing.json:747-775`; service inventory | Add `versions.tf` and backend README per context |
| P1 | D7 | Missing sigstore/cosign module signing evidence | `ADR-0328:3922-3926` | Add module signing provenance |
| P1 | D8 | No `supported-oses.json` exists | file inventory; `ADR-0328:3950-4000` | Add OS support manifest |
| P1 | D8 | No Tier-1 OS rows or blocking CI lanes | `specs/master-plan-sequencing.json:777-815` | Declare per-OS package and CI lane |
| P1 | D8 | No Tier-2 test-only rows or out-of-scope explicit rows | `ADR-0328:3981-3991` | Add ppc64le/s390x and exclusions |
| P1 | D19 | No `iac/oci-guest/always-free/` module exists | `specs/master-plan-sequencing.json:857-867`; file inventory | Add Always Free module or explicit capacity floor |
| P1 | D19 | demo_trial tenant_class exceeds OCI Always Free capacity and lacks reconciliation | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42`; `specs/master-plan-sequencing.json:857-867` | Split OCI demo_trial tenant_class from standard demo_trial tenant_class or downscope |
| P2 | D1 | Architecture begins with stub-disclaimer language | `ARCHITECTURE.md:3`; `documentation-rigor.md:133-156` | Remove stub label after normative expansion |
| P2 | D1 | Manifest failure domain names edge routing instead of evidence integrity | `manifest.json:414` | Correct failure-domain description |
| P2 | D1 | Tenant_class vocabulary mismatch between `T0/T2/T3` and demo_trial/paid tenant_class | `manifest.json:343-347`; `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-110` | Normalize manifest and tenant_class docs |
| P2 | D3 | No service-local SQL/schema for Postgres-backed tables | `capacity-model.md:57-69`; service inventory | Add schema or link canonical migrations |
| P2 | D3 | CI lanes named but not backed by OS/context matrix | `PRD.md:198-205`; file inventory | Add context and OS lane evidence |
| P2 | D5 | Existing competitor docs do not use current top-3 counterpart set | `PRD.md:243-260`; `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137`; chat `:15698` | Supersede with current parity matrix |
| P2 | D9 | Future TS/Python SDK wording lacks generated-only provenance | `PRD.md:47`; `ADR-0328:4050-4083` | Rewrite as generated SDK only with provenance |
| P3 | D2 | Reverse-reference index for inbound user journeys is incomplete | `docs/user-journeys/j89...:119-171`; `manifest.json:402-413` | Add journey reverse-index section |
| P3 | D2 | Old stale coherence audit existed with stale inventory and old scope | previous `coherence-audit-2026-05-20.md:1-2655` | Superseded by this report |

Severity counts in this audit document: P0=0, P1=13, P2=7, P3=2.

## §5 Open questions for Wave 14 aggregation
1. Should the canonical context directory for context 6 remain `iac/oyatie-iaas/` from ADR-0328 or use the prompt spelling `iac/oyatie-as-cloud-provider/` for all Phase 1 foundation services?
2. Should audit-chain demo_trial tenant_class be split into `demo_trial-OCI-AF` and `demo_trial tenant_class-standard`, or should the global demo_trial tenant_class floor be reduced to the OCI Always Free envelope?
3. Should the service-local HSM abstraction be owned by audit-chain, cloud-secrets, cloud-kms, or a shared cloud-iac provider module?
4. Should non-Rust SDK output live under service-local generated directories, a shared SDK repo, or language-specific package directories with generated-client provenance manifests?
5. Should the stale `competitor-parity-matrix.md` remain historical or be superseded after the four current-wave deliverables land?
6. Should workspace Rust crates be mirrored in service-local catalog manifests with line-level source references, or is workspace-level crate discovery enough for intern-buildability?
7. Should Microsoft Purview Audit parity be mapped as audit-log product parity only, or should eDiscovery/Purview compliance surfaces be delegated to the compliance µservice?
8. Should Google Access Transparency-style support-access logging belong to audit-chain or observability, with audit-chain sealing the resulting events?
9. Should current service runbooks be expanded with per-context variants before or after OpenTofu modules are authored?
10. Should Wave 14 classify missing six-context support in Phase 1 foundations as a release blocker for FD-001, even when the product surface is otherwise coherent?

## New Constraint Dimensions
Dimension 6 Multi-context deployment support: FINDING, P1, no complete six-context service-local support evidence.
Dimension 7 OpenTofu IaC coverage: FINDING, P1, Helm/Kustomize only plus Terraform references.
Dimension 8 OS support matrix: FINDING, P1, no supported-oses manifest.
Dimension 9 Rust-strict language coverage: PASS-WITH-P2-DOC-GAP, no forbidden files found, but SDK/generated-client boundary wording needs repair.

## §6 Evidence Annex for Aggregation
1. Product purpose evidence: `PRD.md:18-26` defines audit-chain as a non-repudiation and evidence-ledger service, so storage, query, and retention docs must serve evidence integrity rather than generic logging.
2. Product surface evidence: `PRD.md:38-49` lists append, seal, verify, query, retention, and SDK-related functional requirements, which map directly to CloudTrail, Google Audit Logs, and Purview Audit parity families.
3. Performance evidence: `PRD.md:55-63` states emit/seal/verify latency targets, but this audit found target statements rather than measured benchmark artifacts.
4. Infrastructure evidence: `PRD.md:67` names OCI Cloud-HSM and Object Lock style storage, which is a paid-infrastructure assumption that conflicts with an unqualified OCI Always Free demo_trial claim.
5. Compliance evidence: `PRD.md:77-84` names compliance objectives; those objectives require stronger retention, export, and proof language than the current context-IaC documentation provides.
6. SLO evidence: `PRD.md:88-91` asserts availability and recoverability, but `failure-modes.md:243-261` is the place where those objectives need context-specific RTO/RPO backing.
7. Bounded-context evidence: `PRD.md:101-109` defines the service boundary, and this audit treats logging analytics outside evidence integrity as parity surface rather than core ownership.
8. Layering evidence: `PRD.md:162-170` and `ARCHITECTURE.md:197-208` support the finding that domain/usecase/adapters are conceptually separated even though service-local source files are not present.
9. Crate-location evidence: `PRD.md:172` points at workspace crates, so intern-buildability requires either service-local links or a manifest that maps docs to crate paths.
10. Port evidence: `PRD.md:178-190` names port traits; the absence of provider-context OpenTofu makes those ports architectural intent, not deployment proof.
11. CI evidence: `PRD.md:198-205` names CI lanes, but the missing `supported-oses.json` means those lanes are not mapped to the 13 Tier-1 OS rows required by ADR-0328.
12. Event evidence: `PRD.md:211-227` names emitted events; those events support service purpose and cross-service handoff coherence.
13. Counterpart drift evidence: `PRD.md:243-260` uses an older counterpart set and therefore needs replacement by the current AWS CloudTrail / Google Cloud Audit Logs / Microsoft Purview Audit bar from chat line `15698`.
14. Capacity evidence: `PRD.md:284-306` and `capacity-model.md:139-147` provide capacity targets that are useful but not yet benchmark measurements.
15. Acceptance evidence: `PRD.md:312-321` contains acceptance criteria, but those criteria do not currently include six-context OpenTofu proof, OS support proof, or OCI Always Free demo_trial reconciliation.
16. Stub evidence: `ARCHITECTURE.md:3` says the architecture was scaffolded by a wave anchor sweep, making it a P2 substance-risk signal even though the rest of the file is substantial.
17. Principal evidence: `ARCHITECTURE.md:9-20` anchors tenant, actor, and evidence-chain principals used in the product-purpose finding.
18. Authorization evidence: `ARCHITECTURE.md:71-82` and `:261-270` define Cedar policy evaluation and justify treating authorization evidence as in-scope for audit-chain.
19. Cell evidence: `ARCHITECTURE.md:321-332` defines cell eligibility, but no service-local context modules prove actual six-context cell deployment.
20. Runtime evidence: `ARCHITECTURE.md:445-456` names Kubernetes pods and Cloud Hypervisor/Kata, while ADR-0328 requires OpenTofu context substrate; both must be reconciled.
21. Observability evidence: `ARCHITECTURE.md:507-518` supports product observability, but benchmark and SLO evidence remain target-level.
22. Abuse-defense evidence: `ARCHITECTURE.md:569-580` supports threat posture but does not replace IaC, OS, or language-policy evidence.
23. Edge-case evidence: `ARCHITECTURE.md:631-642` is substantive enough for design review, but it does not close deployment support gaps.
24. Credential evidence: `ARCHITECTURE.md:693-704` supports strong credential isolation claims, but context-specific key/HSM modules are still missing.
25. Manifest evidence: `manifest.json:6-50` lists workspace crates, supporting the conclusion that implementation code exists outside the µservice directory.
26. Manifest evidence: `manifest.json:64-73` lists contract paths, supporting the conclusion that API/async/proto contract coverage exists.
27. Manifest evidence: `manifest.json:75-93` lists capabilities, but capability labels need normalization against demo_trial/paid tenant_class.
28. Manifest evidence: `manifest.json:95-137` lists SLOs, but those SLOs need context and OS dimensions to pass ADR-0328 D-20.
29. Manifest evidence: `manifest.json:343-347` lists `T0/T2/T3`, which conflicts with `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-112`.
30. Manifest evidence: `manifest.json:402-413` lists dependencies, providing a basis for outbound cross-reference verification.
31. Manifest evidence: `manifest.json:414` names an edge-routing failure domain, which is wrong-direction for an audit evidence-chain service.
32. Tier evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` defines demo_trial tenant_class but does not name OCI Always Free as the demo_trial tenant_class envelope.
33. Tier evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:44-66` defines paid tenant_class and introduces paid-scale expectations that fit OCI paid context, not Always Free.
34. Tier evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:68-92` defines paid tenant_class and implies regulated scale, which requires stronger evidence than current context-IaC docs provide.
35. Tier evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:94-112` defines paid tenant_class and should explicitly exclude OCI Always Free as a deployment envelope.
36. Tier evidence: `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137` uses an older vendor displacement set, creating a P2 counterpart drift against the current prompt.
37. Cost evidence: `cost-budget.md:22-32` prices OCI primitives and Cloud-HSM; this is useful for paid tenant_class and risky for demo_trial-OCI-AF unless split.
38. Cost evidence: `cost-budget.md:62` says HSM cost dominates, supporting the OCI Always Free reconciliation finding.
39. Cost evidence: `cost-budget.md:67` treats HSM as mandatory, which needs a constrained demo_trial-OCI-AF exception or a paid-only floor.
40. Capacity evidence: `capacity-model.md:57-69` models Postgres storage but no service-local schema file was found.
41. Capacity evidence: `capacity-model.md:93-109` models HSM capacity, including OCI Cloud-HSM, so provider-neutral signer ports need explicit per-context binding.
42. Handoff evidence: `cross-microservice-handoffs.md:20-55` defines inbound/outbound flows and supports dependency coherence.
43. Handoff evidence: `cross-microservice-handoffs.md:72-87` defines emissions and supports audit-chain as a producer/consumer of domain evidence.
44. Handoff evidence: `cross-microservice-handoffs.md:109-124` defines failure cascade behavior and supports incident/failure-mode coherence.
45. Handoff evidence: `cross-microservice-handoffs.md:151-164` defines Cedar guard-ledger handoffs, supporting authorization-evidence ownership.
46. Contract evidence: `contracts/openapi/audit-chain.yaml:1-220` shows REST API contract substance, but hard limits still need explicit publication.
47. Contract evidence: `contracts/asyncapi/audit-events.yaml:1-197` shows async event contract substance, supporting cross-service event surface.
48. Contract evidence: `contracts/proto/audit-chain.proto:1-220` shows proto contract substance, but generated SDK provenance remains under-specified.
49. Failure evidence: `failure-modes.md:30-47` has a failure index; this is substantial but not context-specific enough for all six deployment contexts.
50. Failure evidence: `failure-modes.md:48-241` has detailed failure paths; those should become test cases after context modules exist.
51. Incident evidence: `incident-response.md:24-35` defines severity levels, supporting operational maturity.
52. Incident evidence: `incident-response.md:78-91` defines lifecycle state, supporting incident flow substance.
53. Incident evidence: `incident-response.md:150-218` defines notifications, supporting handoff maturity.
54. Incident evidence: `incident-response.md:254-258` defines verification commands, but not per-context or per-OS release gates.
55. OpenTofu evidence: `IP-001-storage-backend-iac.md:38` names an absent Terraform path, making the P1 OpenTofu finding direct rather than inferred.
56. OpenTofu evidence: `iac/helm/audit-storage/values.yaml:4` points to `iac/terraform/`, creating a second direct IaC drift citation.
57. Threat-model evidence: `threat-model.md:466` says Postgres role grants are Terraform-managed, which conflicts with the OpenTofu-only doctrine.
58. Canonical evidence: `ADR-0328:3900-3939` forbids Terraform, Pulumi, CloudFormation, null_resource, local-exec, SSH provisioners, hand-edited state, and unsigned modules.
59. Canonical evidence: `ADR-0328:3950-4000` requires OS manifest fields and per-OS CI/package coverage.
60. Canonical evidence: `ADR-0328:4011-4083` permits `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, `.openapi.yaml`, `.asyncapi.yaml`, `.openslo.yaml`, `.sql`, and `.md` while forbidding non-Rust backend source.
61. Constraint memory evidence: `feedback_multi_context_provider_agnostic_2026_05_20.md:28-38` says audit-chain must abstract storage/compute/network across contexts.
62. Constraint memory evidence: `feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` says missing context directories and manual/provisioner patterns are P1 issues.
63. Constraint memory evidence: `feedback_os_support_matrix_2026_05_20.md:56-76` defines the expected OS manifest shape.
64. Constraint memory evidence: `feedback_rust_strict_only_no_python_2026_05_20.md:38-64` defines the authorized and forbidden language boundary.
65. Constraint memory evidence: `feedback_oci_always_free_maximization_2026_05_20.md:65-82` says every µservice needs an OCI Always Free path and demo_trial tenant_class note.
66. Chat evidence: chat line `13449` shows prior audit-chain ownership-coherence assignment history.
67. Chat evidence: chat line `14069` shows the four-document audit shape in the current wave family.
68. Chat evidence: chat line `15698` confirms the current top-three counterparts for audit-chain.
69. Chat evidence: chat line `15739` shows audit-chain was part of a later in-flight batch, so this report supersedes stale partial work.
70. Inventory evidence: the pre-write inventory found 251 files and 62,817 lines under `microservices/audit-chain/`, proving the audit covered more than top-level summaries.
<!-- ORCHESTRATOR REPORT
  µservice: audit-chain
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/audit-chain/coherence-audit-2026-05-20.md (646 lines)
    - /Users/jasonlee/oyatie/microservices/audit-chain/feature-parity-matrix-2026-05-20.md (402 lines)
    - /Users/jasonlee/oyatie/microservices/audit-chain/performance-benchmark-numbers-2026-05-20.md (521 lines)
    - /Users/jasonlee/oyatie/microservices/audit-chain/tenant-class-remediation-report-2026-05-20.md (398 lines)
  inventory_files_seen: 251
  inventory_lines_read: 62817
  chat_history_matches_processed: 4
  findings_p0: 0
  findings_p1: 13
  findings_p2: 7
  findings_p3: 2
  top_3_counterparts_confirmed: AWS CloudTrail / Google Cloud Audit Logs / Microsoft Purview Audit
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1967
-->
