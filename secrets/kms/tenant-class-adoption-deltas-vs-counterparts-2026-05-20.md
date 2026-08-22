# cloud-kms tenant_class model Deltas vs Counterparts - 2026-05-20

doc_class: capability-tenant_class-delta-report
microservice: cloud-kms
status: landed
date: 2026-05-20
counterpart_set: AWS KMS / Google Cloud KMS / HashiCorp Vault self-hosted

## Citation Anchor Block

1. Canonical audit/tier/cross-context rules: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2235`, `docs/decisions/ADR-0700-ci-admission-live-apex.md:3756-4153`.
2. Machine-readable deployment, IaC, OS, language, and OCI constraints: `specs/master-plan-sequencing.json:704-868`.
3. Local tenant_class source: `microservices/cloud-kms/retired tenant_class adoption artifact:13-83`.
4. Runtime contract/source source: `contracts/openapi/cloud/cloud-kms-v1.yaml:1-420`, `crates/cloud-kms-domain/src/lib.rs:71-220`.
5. Documentation rigor source: `docs/standards/documentation-rigor.md:133-190`, `docs/standards/brief-template.md:666-1304`.

## Section 1 - Oyatie Tier Definitions for cloud-kms

demo_trial definition 01: demo_trial is the low-cost and development-safe KMS tier.
demo_trial definition 02: Current local demo_trial uses software/libsodium custody, no FIPS claim, AES-256-GCM, Ed25519/X25519, HMAC-SHA-256, and up to 5 CMKs per tenant.
demo_trial definition 03: Current local demo_trial sets 90-day rotation and manual cryptoshred within 24 hours.
demo_trial definition 04: Current local demo_trial targets 100/sec sustained and 500/sec burst DEK issuance.
demo_trial definition 05: Current local demo_trial targets p95 DEK <=18 ms and sign <=22 ms.
demo_trial definition 06: Current local demo_trial says approximate cost is `$40/month`, which conflicts with OCI demo_trial tenant_class Always Free.
demo_trial definition 07: Corrected demo_trial should split generic demo_trial from OCI demo_trial tenant_class Always Free.
demo_trial definition 08: demo_trial should support encrypt/decrypt authorization receipts, AAD fingerprint validation, tenant binding, and data-class validation.
demo_trial definition 09: demo_trial should not claim regulated HSM assurance unless backed by an explicit context module.
demo_trial definition 10: demo_trial should include a denial path, idempotency path, and audit receipt path before expansion.

paid definition 01: paid is the paid baseline production tier.
paid definition 02: Current local paid uses AWS CloudHSM or Marvell LiquidSecurity 2 and FIPS 140-3 Level 2.
paid definition 03: Current local paid allows 50 CMKs per tenant and 30-day rotation.
paid definition 04: Current local paid targets 1,500/sec sustained and 6,000/sec burst DEK issuance.
paid definition 05: Current local paid targets p95 DEK <=8 ms and sign <=12 ms.
paid definition 06: Current local paid targets cryptoshred within 30 minutes.
paid definition 07: paid should include BYOK import, basic HSM attestation, key CRUD, and OpenSLO coverage.
paid definition 08: paid should cover paid AWS/GCP/OCI backing when Always Free is insufficient.
paid definition 09: paid should include production audit-chain integration.
paid definition 10: paid should include local OpenTofu modules for every supported context.

paid definition 01: paid is the regulated production scale tier.
paid definition 02: Current local paid uses Thales Luna 7 plus AWS CloudHSM with FIPS 140-3 Level 3.
paid definition 03: Current local paid includes PQC ML-KEM/ML-DSA migration readiness.
paid definition 04: Current local paid allows 500 CMKs per tenant and 14-day rotation.
paid definition 05: Current local paid targets 12,000/sec sustained and 50,000/sec burst DEK issuance.
paid definition 06: Current local paid targets p95 DEK <=4 ms and sign <=6 ms.
paid definition 07: Current local paid targets cryptoshred within 5 minutes.
paid definition 08: paid should include multi-region key replica design, HSM sharding, strict residency, and import/reimport workflows.
paid definition 09: paid should include regulated compliance evidence for KCMVP/FIPS/Common Criteria/Pci where claimed.
paid definition 10: paid should include cross-service integration tests with cloud-iam, cloud-storage, cloud-data, cloud-iac, and audit-chain.

paid definition 01: paid is the target single-tenant/high-scale tier; hyperscaler-class positioning remains target_non_claim until benchmark, failover, compliance, and cross-service evidence pass.
paid definition 02: Current local paid uses dedicated Utimaco CP5, Thales Luna 7, and Vault Enterprise HSM seal.
paid definition 03: Current local paid claims FIPS 140-3 Level 3 and Common Criteria EAL4+.
paid definition 04: Current local paid says unlimited CMKs by contract and 7-day rotation.
paid definition 05: Current local paid targets up to 200,000/sec per partition.
paid definition 06: Current local paid targets p95 DEK <=2 ms and sign <=3 ms.
paid definition 07: Current local paid targets cryptoshred within 60 seconds.
paid definition 08: paid should include dedicated partition or tenant-isolated HSM cells.
paid definition 09: paid should include measured proof, not only tenant_class claims.
paid definition 10: paid should not rely on external vendor managed-KMS ceilings unless explicitly sharded and documented.

## Section 2 - Counterpart Tier Mapping

AWS tenant_class map 01: AWS standard KMS key store maps closest to Oyatie demo_trial/paid depending on quota and assurance needs.
AWS tenant_class map 02: AWS imported key material maps to Oyatie paid BYOK when import, expiration, and reimport are implemented.
AWS tenant_class map 03: AWS CloudHSM custom key store maps to Oyatie paid/paid depending on partitioning and FIPS target.
AWS tenant_class map 04: AWS External Key Store maps to Oyatie paid/paid HYOK when external key manager reliability is proven.
AWS tenant_class map 05: AWS multi-Region keys map to Oyatie paid multi-region continuity when residency permits.
AWS tenant_class map 06: AWS key policies/grants/aliases map to Oyatie Cedar policy/alias/grant equivalents, currently missing locally.
AWS tenant_class map 07: AWS symmetric high regional quotas map to Oyatie paid throughput but not custom-store HSM throughput.
AWS tenant_class map 08: AWS custom key store 1,800/sec quota maps to Oyatie paid unless sharded.
AWS tenant_class map 09: AWS standard service integration maps to Oyatie cloud product integration through cloud-storage/cloud-data/mail and others.
AWS tenant_class map 10: AWS does not have an OCI Always Free equivalent.

GCP tenant_class map 01: Google software Cloud KMS maps to Oyatie demo_trial/paid.
GCP tenant_class map 02: Google multi-tenant Cloud HSM maps to Oyatie paid/paid.
GCP tenant_class map 03: Google Single-tenant Cloud HSM maps to Oyatie paid/paid.
GCP tenant_class map 04: Google Cloud EKM maps to Oyatie paid/paid HYOK.
GCP tenant_class map 05: Google Autokey maps to an Oyatie missing autoprovision capability.
GCP tenant_class map 06: Google Key Access Justifications maps to an Oyatie missing justification receipt capability.
GCP tenant_class map 07: Google key version lifecycle maps to an Oyatie missing public state transition contract.
GCP tenant_class map 08: Google CMEK integrations map to Oyatie cross-service storage/data/mail integrations.
GCP tenant_class map 09: Google HSM/external quotas map to paid/paid depending on quota adjuster and single-tenant HSM.
GCP tenant_class map 10: Google does not have an OCI Always Free equivalent for Oyatie guest context.

Vault tenant_class map 01: Vault Community self-hosted transit maps to Oyatie demo_trial/paid when HA and HSM are not required.
Vault tenant_class map 02: Vault self-hosted with integrated storage and HA maps to Oyatie paid.
Vault tenant_class map 03: Vault self-hosted with HSM/PKCS#11 auto-unseal maps to Oyatie paid.
Vault tenant_class map 04: Vault Enterprise performance standbys map to Oyatie paid read scale.
Vault tenant_class map 05: Vault Enterprise performance replication maps to Oyatie paid/paid multi-region scale.
Vault tenant_class map 06: Vault Enterprise DR replication maps to Oyatie paid/paid DR.
Vault tenant_class map 07: Vault namespaces map to Oyatie tenancy/cell isolation.
Vault tenant_class map 08: Vault transit rewrap/datakey/derived/convergent capabilities are missing or only narrative in Oyatie.
Vault tenant_class map 09: Vault seal/unseal ceremony maps to Oyatie key-material quorum runbooks.
Vault tenant_class map 10: Vault self-hosted cost maps well to on-prem and colo contexts but not to managed public KMS quotas.

## Section 3 - Per-Oyatie-Tier Delta Tables

### demo_trial Tier Table

| feature | Oyatie demo_trial | AWS equivalent | GCP equivalent | Vault self-hosted equivalent | gap classification |
|---|---|---|---|---|---|
| Encrypt/decrypt authorization | repo API supports | standard KMS encrypt/decrypt | software Cloud KMS encrypt/decrypt | transit encrypt/decrypt | parity-present |
| Key CRUD | not local | full CreateKey/Disable/Delete | CryptoKey/CryptoKeyVersion APIs | transit key create/config | catch-up |
| Data-key issuance | docs only | GenerateDataKey | envelope via client/KMS patterns | transit datakey | catch-up |
| Software custody | libsodium docs | AWS standard HSM service | software protection level | transit software/self-host | partial |
| HSM assurance | no demo_trial FIPS | standard AWS managed HSM service | software or HSM option | optional HSM | acceptable-lower-tenant_class |
| BYOK | docs only | imported key material | key import | transit import | catch-up |
| HYOK | not demo_trial | XKS higher tenant_class | EKM higher tenant_class | self-hosted native | acceptable-defer |
| AAD/encryption context | AAD fingerprint | encryption context | AAD/internal metadata | context/derived | ahead |
| Data-class binding | yes in API/domain | no native | no native | policy possible | ahead |
| Purpose binding | yes in domain/API | no native | no native | policy possible | ahead |
| Idempotency | API tests | SDK/client idempotency | client/API patterns | client policy | ahead |
| SLO manifest | absent | service SLA docs | service SLA docs | operator defined | catch-up |
| OCI Always Free | conflict `$40/month` | none | none | self-host possible | catch-up-canonical |
| OpenTofu context | absent | CloudFormation/Terraform ecosystem outside scope | Deployment Manager/Terraform ecosystem outside scope | Helm/terraform ecosystem outside scope | catch-up-canonical |
| OS support matrix | absent | managed service | managed service | self-host docs | catch-up-canonical |
| Audit receipt | receipt model | CloudTrail | Cloud Audit Logs | audit devices | partial |
| Cryptoshred proof | docs/domain | delete key/material | destroy key version | rotate/delete/audit | partial |
| Autokey | absent | no direct | Autokey | automation possible | catch-up |
| Key Access Justification | absent | no direct | KAJ | policy/audit possible | catch-up |
| Handoff contract | absent | AWS service integration | CMEK integration | app integration | catch-up |

### paid Tier Table

| feature | Oyatie paid | AWS equivalent | GCP equivalent | Vault self-hosted equivalent | gap classification |
|---|---|---|---|---|---|
| HSM backing | CloudHSM/Marvell docs | CloudHSM custom key store | Cloud HSM | HSM/PKCS#11 seal | partial-doc |
| FIPS level | FIPS 140-3 L2 docs | AWS FIPS-backed KMS/HSM | Cloud HSM FIPS posture | FIPS architecture | partial-doc |
| 50 CMKs per tenant | docs | 100k CMK quota account/region | no key count quota | self-host capacity | ahead-for-tenant-limit |
| 30-day rotation | docs | automatic/on-demand eligible | rotation schedule | rotate/config | partial |
| 1,500/sec sustained | docs | custom store 1,800/sec | HSM quota dependent | substrate dependent | parity-target |
| 6,000/sec burst | docs | standard KMS yes, custom store no | quota adjuster/single-tenant HSM needed | substrate dependent | catch-up-evidence |
| Key CRUD | absent local | full | full | full | catch-up |
| BYOK import | docs only | imported key material | key import | transit import | catch-up |
| Basic HYOK | docs only | XKS | EKM | self-hosted | catch-up |
| Audit-chain integration | docs only | CloudTrail | Cloud Audit Logs | audit devices | catch-up |
| Cedar policy | described | key policies/IAM | IAM | ACL/policy | catch-up-policy |
| SLO OpenSLO | absent | service quotas/SLA | service quotas/SLA | operator SLO | catch-up |
| Context IaC | absent | AWS resources | GCP resources | self-host deploy | catch-up-canonical |
| OS manifest | absent | managed service | managed service | self-host matrix | catch-up-canonical |
| HSM attestation | FAQ | CloudHSM | Cloud HSM | HSM docs | catch-up-evidence |
| Quota/throttle model | tenant_class docs | rich quotas | rich quotas | operator telemetry | catch-up |
| Failure runbooks | present | AWS docs | GCP docs | Vault runbooks | parity-doc |
| Integration tests | absent local | service tests internal | service tests internal | operator tests | catch-up |
| Migration from AWS | present | N/A | N/A | N/A | present |
| Migration from GCP | absent | N/A | N/A | N/A | catch-up-doc |

### paid Tier Table

| feature | Oyatie paid | AWS equivalent | GCP equivalent | Vault self-hosted equivalent | gap classification |
|---|---|---|---|---|---|
| FIPS 140-3 L3 | docs | CloudHSM custom store | Cloud HSM/single-tenant | HSM/PKCS#11 | partial-evidence |
| Thales/AWS HSM | docs | CloudHSM | Cloud HSM | external HSM | partial-doc |
| PQC readiness | ML-KEM/ML-DSA docs | ML-DSA quota appears | PQC decapsulate appears in quota ops | plugin/crypto dependent | ahead-target |
| 500 CMKs/tenant | docs | 100k account/region | no count quota | substrate capacity | present-doc |
| 14-day rotation | docs | on-demand/automatic eligible | rotation schedule | rotate/config | partial |
| 12k/sec sustained | docs | standard KMS yes, custom store needs sharding | HSM quotas need tuning | substrate dependent | catch-up-evidence |
| 50k/sec burst | docs | standard high-quota regions yes | software quota possible; HSM needs single-tenant/tuning | substrate dependent | catch-up-evidence |
| Multi-region key replicas | FAQ only | multi-Region keys | locations/versions | replication | catch-up-design |
| Residency enforcement | domain model | region/IAM | location/IAM | namespace/storage | partial |
| External key store | docs only | XKS | EKM | native self-hosted | catch-up |
| Rewrap | absent | ReEncrypt | raw/re-encrypt patterns | transit rewrap | catch-up |
| Sign/verify | docs only | asymmetric sign/verify | asymmetricSign/getPublicKey | transit sign/verify | catch-up-api |
| MAC/HMAC | absent | HMAC keys | MAC keys | transit HMAC | catch-up-api |
| Random generation | absent | GenerateRandom | generateRandomBytes | transit random | catch-up-api |
| Key Access Justification | absent | no native | KAJ | policy/audit | catch-up |
| HSM attestation evidence | docs | CloudHSM evidence | Cloud HSM | HSM docs | catch-up-evidence |
| Compliance matrix | absent | service compliance docs | service compliance docs | FIPS/HSM docs | catch-up |
| Cross-service integration | missing scenario | service integrations | CMEK integrations | app integrations | catch-up |
| OpenTofu per context | absent | not required by AWS | not required by GCP | self-host automation | catch-up-canonical |
| Measured benchmark | absent evidence | quotas public | quotas public | substrate measured | catch-up-evidence |

### paid Tier Table

| feature | Oyatie paid | AWS equivalent | GCP equivalent | Vault self-hosted equivalent | gap classification |
|---|---|---|---|---|---|
| Dedicated HSM partition | docs | CloudHSM cluster | Single-tenant Cloud HSM | dedicated HSM | partial-doc |
| Dual-vendor HSM | docs | AWS plus external possible | GCP plus external possible | any self-host design | ahead-target |
| Vault HSM seal | docs | no direct | no direct | native HSM/cloud seal | parity-doc |
| Unlimited CMKs by contract | docs | 100k quota per region/account | no key count quota | substrate contract | partial |
| 7-day rotation | docs | possible on-demand | possible schedule | possible config | partial |
| 200k/sec per partition | docs | standard high quota maybe; custom store no | software tokens maybe; HSM no default | substrate dependent | catch-up-evidence |
| p95 <=2 ms | docs | no public latency guarantee | no public latency guarantee | substrate dependent | catch-up-evidence |
| cryptoshred <=60 sec | docs | delete material/key state not equivalent | destroy version staged | operator dependent | ahead-target |
| external/HYOK | docs | XKS | EKM | self-host native | catch-up-contract |
| per-tenant single-tenant | docs | CloudHSM | Single-tenant HSM | namespaces/dedicated cluster | partial |
| audited destruction proof | domain/docs | CloudTrail | Cloud Audit Logs | audit devices | ahead-target |
| KAJ equivalent | absent | no direct | KAJ | policy/audit | catch-up |
| autoprovision keys | absent | AWS service-created keys | Autokey | automation | catch-up |
| import/reimport | absent local | supported | supported | supported | catch-up |
| cross-region active/active | FAQ only | multi-Region keys | multi-location design | replication | catch-up-design |
| disaster recovery | runbooks partial | service durability | service durability | DR replication | catch-up-design |
| formal SLOs | absent | service SLA/quota | service SLA/quota | operator SLO | catch-up |
| six-context deployment | absent | single provider | single provider | self-host flexible | catch-up-canonical |
| security review state | unreviewed catalog | mature service | mature service | mature product | catch-up-governance |
| measured proof | absent | public quota docs | public quota docs | operator benchmark | catch-up-evidence |

## Section 4 - OCI demo_trial tenant_class = Always Free Reconciliation

OCI reconciliation 01: Canonical direction says guest-on-OCI has an Always Free sub-profile.
OCI reconciliation 02: Canonical direction says per-microservice Always Free modules belong under `iac/oci-guest/always-free/`.
OCI reconciliation 03: Canonical direction says demo_trial tenant_class in OCI context should use the Always Free envelope.
OCI reconciliation 04: Current cloud-kms demo_trial cost says about `$40/month`, so the local tenant_class matrix is not OCI demo_trial tenant_class compliant.
OCI reconciliation 05: Corrected OCI demo_trial tenant_class should use OCI Ampere A1 budget up to 4 OCPU/24 GB across the guest stack.
OCI reconciliation 06: Corrected OCI demo_trial tenant_class should avoid paid HSM or paid external KMS features.
OCI reconciliation 07: Corrected OCI demo_trial tenant_class should limit throughput to a lower target such as 50/sec sustained and 200/sec burst until measured.
OCI reconciliation 08: Corrected OCI demo_trial tenant_class should treat OCI Vault paid or capacity-exceeding features as paid tenant_class.
OCI reconciliation 09: Corrected OCI demo_trial tenant_class should store only enough key metadata and receipts to remain within free storage limits.
OCI reconciliation 10: Corrected OCI demo_trial tenant_class should avoid replication patterns that consume paid egress.
OCI reconciliation 11: Corrected OCI demo_trial tenant_class should use OpenTofu Always Free guards, not prose-only claims.
OCI reconciliation 12: Corrected OCI demo_trial tenant_class should still preserve Oyatie authority over key policy and receipts even if OCI Vault backs a key.
OCI reconciliation 13: Corrected OCI demo_trial tenant_class should explicitly declare unsupported features: dedicated HSM, external HYOK proxy, high-throughput bursts, single-tenant HSM, and cross-region active/active.
OCI reconciliation 14: Corrected OCI demo_trial tenant_class should include a fail-closed degradation path when Always Free capacity is exhausted.
OCI reconciliation 15: Corrected OCI demo_trial tenant_class should emit cloud-billing zero-cost events for visibility, even when spend is zero.
OCI reconciliation 16: Corrected OCI demo_trial tenant_class should state upgrade triggers to paid: >5 CMKs, sustained >50/sec, regulated HSM requirement, import/HYOK, replication, or cryptoshred SLA <24h.
OCI reconciliation 17: Corrected OCI demo_trial tenant_class should map package/runtime to Oracle Linux 9+ or containerized Ampere targets once OS manifest lands.
OCI reconciliation 18: Corrected OCI demo_trial tenant_class should include a capacity test in CI to prove it fits within Always Free.
OCI reconciliation 19: Corrected OCI demo_trial tenant_class should not claim parity with AWS/GCP managed KMS quotas.
OCI reconciliation 20: Corrected OCI demo_trial tenant_class should become a sub-profile inside `retired tenant_class adoption artifact`, not a separate unwritten assumption.

## Section 5 - Findings by Tier

demo_trial finding 01: demo_trial is ahead on data-class and purpose-bound receipts.
demo_trial finding 02: demo_trial is behind on OCI Always Free reconciliation.
demo_trial finding 03: demo_trial is behind on local PRD/architecture/contract/SLO/IaC ownership.
demo_trial finding 04: demo_trial is at parity for encrypt/decrypt concept, because repo-level API supports those operations.
demo_trial finding 05: demo_trial is behind AWS/GCP/Vault on key CRUD and import.
demo_trial finding 06: demo_trial should not claim regulated HSM maturity.
demo_trial finding 07: demo_trial can be credible after context manifest, OpenSLO, and Always Free module land.
demo_trial classification: catch-up on canonical ownership, ahead on data-boundary semantics.

paid finding 01: paid is close to AWS custom key store quota but needs measured sharding and context modules.
paid finding 02: paid is behind Google Cloud HSM on documented quotas and key lifecycle.
paid finding 03: paid is behind Vault on rewrap/datakey and self-hosted operator ceremony.
paid finding 04: paid is ahead only where Oyatie data-class/purpose/Cedar receipt semantics are enforced.
paid finding 05: paid lacks policy files, import workflows, and HSM attestation evidence.
paid finding 06: paid can become credible with OpenTofu, SLO, BYOK, key CRUD, and integration tests.
paid classification: partial parity in target numbers, catch-up in implementation envelope.

paid finding 01: paid target throughput exceeds default AWS custom key store and many Google HSM baselines unless partitioned.
paid finding 02: paid is ahead-target on PQC readiness, but lacks contract/API support.
paid finding 03: paid is behind AWS multi-Region keys and Vault replication because local replication design is only FAQ prose.
paid finding 04: paid is behind counterpart maturity on signing, MAC, random, import, and rewrap APIs.
paid finding 05: paid needs compliance evidence before KCMVP/FIPS/Common Criteria claims are operationally credible.
paid finding 06: paid can be credible only after measured HSM benchmarks and cross-service integration tests.
paid classification: ambitious target, catch-up evidence and API surface.

paid finding 01: paid is hyperscaler-aspirational and not currently supportable by local artifacts.
paid finding 02: paid target p95 <=2 ms and 200k/sec per partition require dedicated measured substrate.
paid finding 03: paid is behind AWS/GCP managed-service maturity for public operational proof.
paid finding 04: paid is behind Vault Enterprise on replication/seal documentation if Vault remains part of the tier.
paid finding 05: paid is ahead-target on dual-vendor HSM and cryptoshred proof semantics.
paid finding 06: paid should be gated behind security review, SLOs, IaC, OS support, HSM attestation, and benchmark evidence.
paid classification: catch-up for evidence, ahead-target for custody ambition.

## Section 6 - Remediation Sequence

Remediation 01: Add `PRD.md` and `ARCHITECTURE.md` before expanding tenant_class promises.
Remediation 02: Add `supported-oses.json` and context manifest so every tenant_class is deployment-scoped.
Remediation 03: Add `iac/oci-guest/always-free/` and update demo_trial to include OCI Always Free sub-profile.
Remediation 04: Add `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
Remediation 05: Add local contract ownership pointers for existing OpenAPI and planned key lifecycle operations.
Remediation 06: Add OpenSLO files for each tenant_class and the encrypt/decrypt authorization surfaces.
Remediation 07: Add policy/Cedar artifacts for KMS encrypt/decrypt/key lifecycle decisions.
Remediation 08: Add key CRUD/import/rewrap/sign/MAC/random decision: implement, delegate, or reject with ADR.
Remediation 09: Add self-hosted Vault parity note distinct from Vault Enterprise.
Remediation 10: Add measured benchmark evidence and demote unverified performance claims until evidence exists.

## Section 7 - Tier Delta Bottom Line

demo_trial can become credible quickly if it is narrowed and OCI Always Free is made explicit.
paid is the first serious production tenant_class and needs IaC, SLO, BYOK, and HSM attestation artifacts.
paid is the regulated scale tenant_class and needs multi-region, partition sharding, compliance evidence, and cross-service tests.
paid is an ambition, not a claim the current artifact set can support.
Across all tiers, Oyatie's differentiator is policy-bound evidence, not raw cryptographic novelty.
Across all tiers, the blocker is artifact ownership: the path must own or point to machine-readable contracts, deployment modules, OS support, policies, and evidence.

## Section 8 - Evidence-Backed Delta Notes

Evidence delta 01: The local tenant_class matrix defines demo_trial and paid tenant_class, but it does not bind those tenant_classes to the six canonical deployment contexts.
Evidence delta 02: The local tenant_class matrix gives throughput and latency numbers, but it does not name the workload shape used to derive them.
Evidence delta 03: The local benchmark document compares against Azure Key Vault, but the assigned counterpart set for this audit requires Google Cloud KMS instead.
Evidence delta 04: The local benchmark document names an evidence directory that is absent from the repository snapshot.
Evidence delta 05: The FAQ says cloud-kms remains the policy engine when Vault is used, which is an important additive surface versus plain Vault transit usage.
Evidence delta 06: The FAQ's BYOK/HYOK answer is directionally strong, but it is not backed by a local contract, import manifest, attestation schema, or runbook.
Evidence delta 07: The migration playbook contains useful AWS and Vault inventory steps, but it does not yet map those discovered fields to Oyatie key metadata.
Evidence delta 08: The reference implementation proves envelope encryption ergonomics, but it is an SDK guide rather than a service runtime implementation.
Evidence delta 09: The runbooks prove operational imagination, but they depend on microservice handoffs without a local `cross-microservice-handoffs.md`.
Evidence delta 10: The repo-level OpenAPI proves encrypt/decrypt request and receipt shapes, but cloud-kms path does not locally own or cite that contract.
Evidence delta 11: The repo-level Rust API crate proves authorization and idempotency behavior, but cloud-kms docs do not tell a new maintainer where that crate lives.
Evidence delta 12: The repo-level domain crate proves key purpose, HSM validation, receipt, and error enums, but local docs do not tie those enums to tenant_class gates.
Evidence delta 13: AWS KMS has explicit custom and external key store concepts; cloud-kms has similar language but lacks a deployment binding.
Evidence delta 14: Google Cloud KMS has Autokey and EKM surfaces; cloud-kms has tenant-aware policy language but lacks equivalent automation and EKM contracts.
Evidence delta 15: Vault transit has rewrap, datakey, convergent encryption, and self-hosted operator ceremonies; cloud-kms has envelope examples but no rewrap surface.
Evidence delta 16: demo_trial should avoid presenting itself as competitor-equivalent to AWS/GCP managed KMS because its OCI sub-profile must fit free-resource limits.
Evidence delta 17: paid should become the baseline for regulated paid deployments because it can justify HSM backing and measured capacity.
Evidence delta 18: paid should be the first tenant_class allowed to claim cross-region active/active once replication and key-state convergence are designed.
Evidence delta 19: paid should be reserved for dedicated tenancy, measured HSM partitions, and independent compliance evidence.
Evidence delta 20: Every tenant_class should carry a demotion rule: if IaC, SLO, OS support, or policy evidence is missing, the tenant_class cannot be advertised externally.

## Section 9 - Tier-Specific Architecture Hooks

demo_trial hook 01: Add `tier = "demo_trial"` to a local service manifest so docs, IaC, and CI can share one tenant_class spelling.
demo_trial hook 02: Add `context = "guest-on-oci"` plus `sub_profile = "always_free"` for the OCI demo_trial tenant_class deployment.
demo_trial hook 03: Add a demo_trial key-count guard that rejects paid-scale posture before provisioning.
demo_trial hook 04: Add a demo_trial throughput guard that emits upgrade guidance when sustained load exceeds the free envelope.
demo_trial hook 05: Add demo_trial receipt retention settings that fit zero-cost storage assumptions.
demo_trial hook 06: Add demo_trial OpenSLO objectives with honest free-tenant_class latency and capacity, not paid HSM aspirations.
demo_trial hook 07: Add demo_trial package/runtime declaration for Ampere-compatible Linux targets.
demo_trial hook 08: Add demo_trial failure semantics for capacity exhaustion, including fail-closed key creation and continued decrypt where policy allows.
demo_trial hook 09: Add demo_trial test coverage for encrypt/decrypt/auth/idempotency using the repo-level Rust API crate.
demo_trial hook 10: Add demo_trial documentation that explicitly excludes HYOK, dedicated HSM, cross-region active/active, and sub-minute cryptoshred.

paid hook 01: Add paid OpenTofu modules for paid AWS, paid OCI, on-prem, colo, public-cloud, and provider-hosted contexts.
paid hook 02: Add paid HSM adapter contract for CloudHSM, OCI Vault, SoftHSM test mode, and self-hosted Vault transit.
paid hook 03: Add paid import-key workflow with custody proof, wrapping key, attestation, and rollback semantics.
paid hook 04: Add paid rotation scheduler semantics that distinguish manual, automatic, emergency, and compliance-driven rotation.
paid hook 05: Add paid rewrap API so imported AWS/Vault keys can be moved without application plaintext exposure.
paid hook 06: Add paid SLOs for p50, p95, p99, error budget, and cryptoshred confirmation.
paid hook 07: Add paid OS package matrix so Linux, Talos, Flatcar, Windows client, and Apple Silicon claims are testable.
paid hook 08: Add paid audit event schema binding for every key lifecycle operation.
paid hook 09: Add paid policy pack that maps tenant, data class, purpose, region, and residency into authorization decisions.
paid hook 10: Add paid runbook evidence for HSM quorum loss, adapter outage, rotation drift, and receipt replay.

paid hook 01: Add paid partitioning design for tenant, region, key family, and HSM pool boundaries.
paid hook 02: Add paid replication design that defines key-state convergence and cryptoshred propagation.
paid hook 03: Add paid multi-region API behavior for encrypt, decrypt, sign, verify, rotate, disable, schedule delete, and restore.
paid hook 04: Add paid compliance mapping for KCMVP, FIPS 140-3, audit retention, and chain-of-custody proof.
paid hook 05: Add paid benchmark harness that measures HSM-backed encrypt, decrypt, generate data key, sign, random, and rewrap.
paid hook 06: Add paid degraded-mode semantics for read-only decrypt, key create freeze, and emergency breakglass.
paid hook 07: Add paid cross-service integration tests with cloud-iam, cloud-network, cloud-storage, cloud-data, cloud-audit-chain, and cloud-billing.
paid hook 08: Add paid customer-held external key manager design if HYOK remains in scope.
paid hook 09: Add paid shard evacuation and rotation storm controls.
paid hook 10: Add paid evidence bundle output for auditor review.

paid hook 01: Add paid dedicated-tenant control plane with isolated HSM partition ownership.
paid hook 02: Add paid dual-vendor HSM quorum and failover design.
paid hook 03: Add paid external key custody proof that can survive customer disconnection.
paid hook 04: Add paid latency budget with measured numbers per context, per OS, and per HSM adapter.
paid hook 05: Add paid active/active replication proof with conflict handling for rotation and destroy operations.
paid hook 06: Add paid service-level policy for customer-managed root-of-trust ceremonies.
paid hook 07: Add paid chaos test set for HSM partition loss, provider outage, clock drift, KMS denial, and audit-chain backpressure.
paid hook 08: Add paid signed OpenTofu module provenance and state backend controls for every context.
paid hook 09: Add paid supply-chain attestation for Rust binaries, generated SDKs, schemas, and policy bundles.
paid hook 10: Add paid launch gate that blocks external maturity claims until benchmarks, compliance, and cross-context failover pass.

## Section 10 - Final Tier Gate Ledger

Gate ledger 01: demo_trial passes only after OCI Always Free is codified and local ownership artifacts exist.
Gate ledger 02: demo_trial should be advertised as developer and small-tenant safe, not as hyperscaler equivalent.
Gate ledger 03: paid passes only after paid-context IaC, HSM adapter contracts, and import/rotation/rewrap flows exist.
Gate ledger 04: paid should be advertised as production baseline after measured SLOs are attached.
Gate ledger 05: paid passes only after regulated multi-region evidence exists.
Gate ledger 06: paid should be advertised as regulated scale after cross-service tests and compliance packs exist.
Gate ledger 07: paid passes only after dedicated-tenant and dual-vendor HSM evidence exists.
Gate ledger 08: paid should be advertised as single-tenant hyperscaler-class only after measured failover and benchmark evidence exists.
Gate ledger 09: Any tenant_class missing OpenTofu context modules remains deployment-incomplete.
Gate ledger 10: Any tenant_class missing supported OS declarations remains packaging-incomplete.
Gate ledger 11: Any tenant_class missing Rust build/test invocation remains implementation-incomplete.
Gate ledger 12: Any tenant_class missing policy artifacts remains authorization-incomplete.
Gate ledger 13: Any tenant_class missing OpenSLO files remains reliability-incomplete.
Gate ledger 14: Any tenant_class missing benchmark evidence remains performance-target-only.
Gate ledger 15: Any tenant_class missing audit-chain integration remains evidence-incomplete.
Gate ledger 16: Any tenant_class missing billing/cost hooks remains commercial-control-incomplete.
Gate ledger 17: Any tenant_class missing incident runbooks remains operations-incomplete.
Gate ledger 18: Any tenant_class missing migration playbook mappings remains adoption-incomplete.
Gate ledger 19: Any tenant_class missing counterpart parity decision remains product-surface-incomplete.
Gate ledger 20: Current cloud-kms is best classified as a strong conceptual documentation pack with incomplete deployable ownership.
