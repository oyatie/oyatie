# cloud-kms Feature Parity Matrix - 2026-05-20

doc_class: feature-parity-matrix
microservice: cloud-kms
status: landed
date: 2026-05-20
counterpart_set: AWS KMS / Google Cloud KMS / HashiCorp Vault self-hosted

## Citation Anchor Block

1. Canonical audit mandate: `docs/decisions/ADR-0700-ci-admission-live-apex.md:3756-4153`.
2. Canonical machine-readable constraints: `specs/master-plan-sequencing.json:704-868`.
3. Local microservice purpose: `microservices/cloud-kms/retired tenant_class adoption artifact:7-11`, `microservices/cloud-kms/faqs/kms-engineer-faq.md:7-12`.
4. Runtime/contract anchor: `contracts/openapi/cloud/cloud-kms-v1.yaml:1-168`, `crates/cloud-kms-domain/src/lib.rs:1-113`.
5. Documentation rigor: `docs/standards/documentation-rigor.md:133-190`, `docs/standards/brief-template.md:1720-1854`.

## External Source Set

AWS source 1: https://docs.aws.amazon.com/kms/latest/developerguide/key-store-overview.html
AWS source 2: https://docs.aws.amazon.com/kms/latest/developerguide/keystore-external.html
AWS source 3: https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html
AWS source 4: https://docs.aws.amazon.com/kms/latest/developerguide/symm-asymm-compare.html
AWS source 5: https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html
AWS source 6: https://docs.aws.amazon.com/general/latest/gr/kms.html
Google source 1: https://docs.cloud.google.com/kms/docs/key-management-service
Google source 2: https://docs.cloud.google.com/kms/docs/quotas
Google source 3: https://docs.cloud.google.com/kms/docs/ekm
Google source 4: https://docs.cloud.google.com/kms/docs/kms-autokey
Google source 5: https://docs.cloud.google.com/kms/docs/algorithms
Google source 6: https://docs.cloud.google.com/kms/docs/key-states
Vault source 1: https://developer.hashicorp.com/vault/docs/secrets/transit
Vault source 2: https://developer.hashicorp.com/vault/docs/internals/rotation
Vault source 3: https://developer.hashicorp.com/vault/docs/configuration/seal/seal-best-practices
Vault source 4: https://developer.hashicorp.com/vault/docs/enterprise/replication
Vault source 5: https://developer.hashicorp.com/vault/tutorials/enterprise/performance-standbys
Vault source 6: https://docs.hashicorp.com/vault/docs/about-vault/how-vault-works

## Section 1 - Counterpart 1: AWS KMS Capability Surface

AWS-01: Standard regional KMS key store for service-managed HSM-backed customer managed keys.
AWS-02: Customer managed KMS keys with symmetric encryption operations.
AWS-03: AWS managed keys for integrated AWS services.
AWS-04: AWS owned keys controlled by service teams.
AWS-05: Imported key material for symmetric, RSA, ECC, and HMAC key specs.
AWS-06: DeleteImportedKeyMaterial and key-material expiration behavior.
AWS-07: CloudHSM custom key store backed by a single-tenant CloudHSM cluster.
AWS-08: External Key Store for HYOK through an XKS proxy.
AWS-09: External key manager outside AWS retains key material and operations.
AWS-10: XKS connectivity and external key store proxy abstraction.
AWS-11: Multi-Region primary keys.
AWS-12: Multi-Region replica keys.
AWS-13: Shared multi-Region properties: key ID, key material, key spec, key usage, and rotation.
AWS-14: Symmetric encrypt, decrypt, re-encrypt, GenerateDataKey, GenerateDataKeyWithoutPlaintext.
AWS-15: GenerateMac and VerifyMac for HMAC key types.
AWS-16: RSA asymmetric encrypt/decrypt/sign/verify.
AWS-17: ECC asymmetric sign/verify.
AWS-18: ML-DSA sign/verify quota surface in current KMS quota docs.
AWS-19: GenerateDataKeyPair and GenerateDataKeyPairWithoutPlaintext.
AWS-20: GetPublicKey.
AWS-21: Automatic key rotation for eligible AWS-generated symmetric customer managed keys.
AWS-22: On-demand rotation for eligible symmetric keys, including imported symmetric material.
AWS-23: Manual rotation pattern for asymmetric, HMAC, and custom key store keys.
AWS-24: Key deletion scheduling and cancellation.
AWS-25: Aliases and alias lookup.
AWS-26: Key policies.
AWS-27: Grants for delegated usage.
AWS-28: IAM authorization integration.
AWS-29: CloudTrail audit log integration.
AWS-30: CloudWatch metrics for external key store throttling.
AWS-31: Per-account/per-region cryptographic request quotas.
AWS-32: Custom key store quota of 1,800 cryptographic ops per second per custom store.
AWS-33: Custom key store count quota per account and region.
AWS-34: Customer managed key count quota.
AWS-35: DescribeKey and key metadata inspection.
AWS-36: ListKeys, ListAliases, ListGrants, ListKeyPolicies, ListKeyRotations.
AWS-37: EnableKey, DisableKey, EnableKeyRotation, DisableKeyRotation.
AWS-38: ReplicateKey for multi-Region workflows.
AWS-39: Imported multi-Region key material responsibilities.
AWS-40: AWS service integrations through customer managed keys.
AWS-41: KMS encryption context as authenticated metadata.
AWS-42: Origin tracking for AWS_KMS, EXTERNAL, AWS_CLOUDHSM, and EXTERNAL_KEY_STORE patterns.
AWS-43: Key usage separation between encrypt/decrypt, sign/verify, and generate/verify MAC.
AWS-44: FIPS-backed regional service baseline.
AWS-45: Availability and durability managed by AWS for standard keys.
AWS-46: External key store trust model shifts availability, latency, durability, and operation security to customer.
AWS-47: CloudHSM key store single-tenant HSM control.
AWS-48: Unsupported feature matrix for custom/external key stores, including lack of multi-Region and automatic rotation.
AWS-49: Request throttling and Service Quotas adjustability for many quotas.
AWS-50: Pricing model outside this matrix includes key storage and API requests; capability hook is metering-aware usage.

## Section 2 - Counterpart 2: Google Cloud KMS Capability Surface

GCP-01: Centralized service for creating, importing, managing, and using cryptographic keys.
GCP-02: Software protection-level keys.
GCP-03: Multi-tenant Cloud HSM protection-level keys.
GCP-04: Single-tenant Cloud HSM instances and keys.
GCP-05: Cloud External Key Manager keys with EXTERNAL and EXTERNAL_VPC protection levels.
GCP-06: Coordinated external keys through Cloud KMS-managed EKM connection.
GCP-07: External key material never exposed to Google for Cloud EKM.
GCP-08: Key rings organized by project and location.
GCP-09: CryptoKeys as key containers.
GCP-10: CryptoKeyVersions as individual key material versions.
GCP-11: Key version states including pending generation, enabled, disabled, scheduled for destruction, and destroyed.
GCP-12: Restore from scheduled destruction into disabled state.
GCP-13: Default scheduled destruction soft-delete period.
GCP-14: Logical deletion from active systems after destruction timeline.
GCP-15: Primary version selection for symmetric keys.
GCP-16: Encrypt/decrypt for symmetric keys.
GCP-17: Asymmetric decrypt.
GCP-18: Asymmetric signing.
GCP-19: MAC sign.
GCP-20: MAC verify.
GCP-21: Get public key.
GCP-22: Generate random bytes for HSM.
GCP-23: GOOGLE_SYMMETRIC_ENCRYPTION algorithm using AES-256-GCM with internal metadata.
GCP-24: Key purposes including ENCRYPT_DECRYPT, ASYMMETRIC_SIGN, ASYMMETRIC_DECRYPT, and MAC.
GCP-25: Key import into Cloud KMS or Cloud HSM.
GCP-26: Import jobs and wrapping-key workflow.
GCP-27: Rotation schedule for symmetric keys.
GCP-28: IAM permissions and roles on key rings/keys.
GCP-29: CMEK integrations with Google Cloud services.
GCP-30: Cloud HSM for Google Workspace client-side encryption support.
GCP-31: Cloud KMS Autokey for automatic CMEK provisioning and assignment.
GCP-32: Autokey service account/IAM role creation.
GCP-33: Autokey resource-specific key generation.
GCP-34: Key Access Justifications support when enrolled and region/program allow it.
GCP-35: Audit logging through Google Cloud audit stack.
GCP-36: No quota on number of KeyRing, CryptoKey, or CryptoKeyVersion resources, only operations.
GCP-37: Cryptographic operation quotas by calling or hosting project.
GCP-38: HSM symmetric cryptographic quota.
GCP-39: HSM asymmetric cryptographic quota.
GCP-40: HSM generate random quota.
GCP-41: External cryptographic quota.
GCP-42: Single-tenant Cloud HSM exemption from some multi-tenant HSM quotas.
GCP-43: External key access must be granted in the external key manager.
GCP-44: External key access can be revoked outside Google.
GCP-45: Centralized key policy across cloud and on-premises EKM.
GCP-46: Cloud KMS API resource hierarchy and regionality.
GCP-47: Destroy/restore APIs for key versions.
GCP-48: Disable/enable key versions.
GCP-49: Google service CMEK usage that does not always count against calling-project quota.
GCP-50: Single-tenant Cloud HSM as dedicated capacity option.

## Section 3 - Counterpart 3: HashiCorp Vault Self-Hosted Capability Surface

Vault-01: Self-hosted identity-based secrets and encryption management.
Vault-02: Auth methods for users, machines, and workloads.
Vault-03: ACL policies around secrets and cryptographic operations.
Vault-04: Audit devices that record detailed access logs.
Vault-05: Secrets engines for modular capabilities.
Vault-06: Transit secrets engine as cryptography-as-a-service.
Vault-07: Transit encrypt operation.
Vault-08: Transit decrypt operation.
Vault-09: Transit rewrap operation without exposing plaintext.
Vault-10: Transit datakey generation for envelope encryption.
Vault-11: Transit sign operation.
Vault-12: Transit verify operation.
Vault-13: Transit HMAC generation.
Vault-14: Transit hash generation.
Vault-15: Transit random bytes generation.
Vault-16: Transit key derivation.
Vault-17: Transit convergent encryption option.
Vault-18: Transit key versioned ciphertext prefix.
Vault-19: Transit named key model per application.
Vault-20: AES-128-GCM key type.
Vault-21: AES-256-GCM key type.
Vault-22: RSA key types for transit operations.
Vault-23: ECDSA/Ed25519-style signing key support in current transit surface.
Vault-24: HMAC key material generated alongside transit key types.
Vault-25: NIST rotation guidance before approximately 2^32 AES-GCM encryptions per key version.
Vault-26: Root key, internal encryption key, unseal key, and upgrade key model.
Vault-27: Shamir seal default.
Vault-28: Cloud KMS auto-unseal.
Vault-29: HSM/PKCS#11 seal configuration.
Vault-30: Transit auto-unseal pattern.
Vault-31: Recovery keys for privileged actions when auto-unseal/HSM is used.
Vault-32: Seal migration procedures.
Vault-33: Integrated Storage.
Vault-34: Consul storage option.
Vault-35: HA active/standby model.
Vault-36: Performance standby nodes for read scaling in Enterprise.
Vault-37: Non-voter Integrated Storage nodes in Enterprise.
Vault-38: Performance Replication in Enterprise for horizontal read/write locality.
Vault-39: Disaster Recovery Replication in Enterprise.
Vault-40: Replication tokens and mutually authenticated TLS between clusters.
Vault-41: Namespaces in Enterprise for administrative isolation.
Vault-42: Local secret engines and auth methods that do not replicate.
Vault-43: HTTP API for replication management.
Vault-44: Plugin ecosystem.
Vault-45: FIPS/HSM architecture support.
Vault-46: PKI/certificate lifecycle management.
Vault-47: KV/static secrets management.
Vault-48: Dynamic credential generation and revocation.
Vault-49: Token lifecycle and lease management.
Vault-50: Operator ceremonies for unseal, rekey, rotate, and root token recovery.

## Section 4 - UNION-Coverage Matrix

| capability | AWS KMS | Google Cloud KMS | Vault self-hosted | union required | Oyatie cloud-kms has | gap classification |
|---|---|---|---|---|---|---|
| Tenant-scoped CMK identity | yes | yes | partial | yes | yes in docs/domain | present |
| Per-tenant key policy | yes | yes | yes | yes | partial via Cedar intent | gap-policy-file |
| Encrypt operation | yes | yes | yes | yes | yes repo API | present |
| Decrypt operation | yes | yes | yes | yes | yes repo API | present |
| Re-encrypt operation | yes | no direct | rewrap | yes | no local API | gap-api |
| Rewrap without plaintext | partial | no direct | yes | yes | no local API | gap-api |
| Generate data key | yes | no direct standard | yes | yes | docs mention DEK issuance | gap-contract |
| Generate data key without plaintext | yes | no | partial | yes | no | gap-api |
| Generate data key pair | yes | no | no | yes | no | gap-api |
| Asymmetric decrypt | yes | yes | partial | yes | no local API | gap-api |
| Asymmetric sign | yes | yes | yes | yes | docs mention HSM signing | gap-contract |
| Verify signature | yes | yes | yes | yes | no local API | gap-api |
| HMAC/MAC sign | yes | yes | yes | yes | no local API | gap-api |
| HMAC/MAC verify | yes | yes | yes | yes | no local API | gap-api |
| Hash generation | no | no direct | yes | yes | no | optional-gap |
| Generate random bytes | yes | yes HSM | yes | yes | no local API | gap-api |
| Key create | yes | yes | yes | yes | domain model only | gap-api |
| Key disable | yes | yes | partial | yes | domain state only | gap-api |
| Key enable | yes | yes | partial | yes | domain state only | gap-api |
| Key deletion schedule | yes | yes | partial | yes | destruction model only | gap-api |
| Key restore/cancel deletion | yes | yes | partial | yes | no local API | gap-api |
| Key destruction receipt | partial | partial | audit only | yes | yes domain/docs | present-additive |
| Alias management | yes | no direct analog | path names | yes | no | gap-api |
| Grant management | yes | IAM | policy tokens | yes | no | gap-api |
| IAM integration | yes | yes | auth/policy | yes | partial cloud-iam | gap-handoff |
| Cedar integration | no | no | no | no vendor | yes intended | additive |
| Key rings/project locations | no | yes | mount/path | yes | no local model | gap-data-model |
| Multi-region key replication | yes | location keys | replication | yes | docs only | gap-design |
| External key store/HYOK | yes XKS | yes EKM | yes self-hosted | yes | docs only | gap-adapter |
| CloudHSM/HSM backing | yes | yes | yes | yes | docs only | gap-iac |
| Single-tenant HSM option | yes CloudHSM | yes single-tenant HSM | self-hosted HSM | yes | paid/paid docs | gap-iac |
| Imported key material | yes | yes | yes | yes | docs only | gap-api |
| Import expiration/reimport | yes | partial | yes | yes | no | gap-api |
| BYOK | yes | yes | yes | yes | docs/domain origins | gap-workflow |
| HYOK | yes XKS | yes EKM | yes self-hosted | yes | docs/domain origins | gap-workflow |
| Automatic rotation | yes eligible | yes symmetric | yes internal/config | yes | docs only | gap-scheduler |
| On-demand rotation | yes eligible | manual/API | yes | yes | tutorial/docs only | gap-api |
| Manual rotation | yes | yes | yes | yes | docs only | gap-runbook-api |
| Rotation cadence drift detection | CloudWatch/custom | Cloud Monitoring/custom | telemetry | yes | runbook exists | present-doc |
| Versioned ciphertext | implicit key versions | yes | yes prefix | yes | receipt key_version | present-partial |
| Encryption context/AAD | yes encryption context | associated data | context/derived | yes | AAD fingerprint | present-additive |
| Data-class binding | no | no | policy possible | no vendor | yes domain/API | additive |
| Purpose binding | no | no | policy possible | no vendor | yes domain/API | additive |
| Audit logs | CloudTrail | Cloud Audit Logs | audit devices | yes | docs/receipt | gap-event-schema |
| Immutable audit chain | no native | no native | audit device | no vendor | intended audit-chain | additive-gap |
| Cost metering | pricing/API requests | pricing/quota | self-hosted cost | yes | local cost tenant_classes | gap-context |
| Operation quotas | yes | yes | self-hosted tuned | yes | tenant_class matrix docs | gap-slo |
| Custom store quota | yes | N/A | self-hosted capacity | yes | no | gap-capacity-model |
| External key quota | yes | yes | self-hosted capacity | yes | no | gap-capacity-model |
| Key count quota | yes | no resource count quota | self-hosted capacity | yes | docs tenant_class limits | present-doc |
| Request throttling metric | yes | yes | telemetry | yes | no local SLO | gap-observability |
| Key Access Justifications | no | yes | policy/audit possible | yes | no | gap-policy |
| Autokey/autoprovision | no | yes | automation possible | yes | no | gap-control-plane |
| CMEK service integrations | yes | yes | app integration | yes | storage/data docs depend on KMS | gap-handoff |
| Certificate/CA signing | KMS sign | CAS separate | PKI engine | yes | cloud-network depends on it | gap-api |
| PKI lifecycle | no | CAS separate | yes | yes | no local API | optional-gap |
| Seal/unseal ceremony | no | no | yes | yes | HSM bootstrap docs only | gap-runbook-design |
| Shamir quorum | no | no | yes | yes | key-material quorum runbook | present-doc |
| HSM attestation | CloudHSM | Cloud HSM | HSM docs | yes | FAQ/runbooks | gap-evidence |
| Compliance pack mapping | FIPS | FIPS/Assured | FIPS/HSM | yes | docs/domain validation | gap-compliance |
| OpenTofu IaC | not vendor | not vendor | not vendor | Oyatie required | no | gap-canonical |
| Six deployment contexts | no vendor | no vendor | self-host flexible | Oyatie required | no | gap-canonical |
| OS support matrix | service hosted | service hosted | self-hosted | Oyatie required | no | gap-canonical |
| OCI Always Free demo_trial | no | no | possible self-host | Oyatie required | no | gap-canonical |
| Rust-only implementation | no | no | Go upstream | Oyatie required | repo Rust yes | present-repo |
| Local microservice source | N/A | N/A | N/A | Oyatie required | no | gap-ownership |
| Local PRD/architecture | N/A | N/A | N/A | Oyatie required | no | gap-ownership |
| Local SLO OpenSLO | N/A | N/A | N/A | Oyatie required | no | gap-ownership |
| Local contract pointer | N/A | N/A | N/A | Oyatie required | absent local | gap-ownership |
| Idempotency | SDK/retries | API semantics | client/policy | yes | API tests | present |
| Conflict handling | API errors | API errors | API errors | yes | API tests | present |
| Tenant drift rejection | IAM/policy | IAM/policy | namespaces/policy | yes | API tests | present-additive |
| AAD digest validation | encryption context | AAD/internal | context | yes | API tests | present-additive |
| Data class validation | no | no | policy possible | no vendor | API tests | additive |
| Per-cell HSM partition | CloudHSM cluster | HSM location | cluster/mount | yes | domain model | gap-iac |
| Residency binding | region/location | location | storage/namespace | yes | domain model | gap-context |
| Cryptoshred proof | delete key/material | destroy version | destroy/rotate/audit | yes | domain/docs | gap-api |
| Provider adapter matrix | AWS native | GCP native | self | Oyatie required | docs only | gap-adapter |
| Migration from AWS | N/A | N/A | N/A | useful | yes | present-doc |
| Migration from GCP | N/A | N/A | N/A | useful | no | gap-doc |
| Migration from Vault self-hosted | N/A | N/A | N/A | useful | partial Enterprise | gap-doc |

## Section 5 - Capability Families Summary

| family | union required count | Oyatie present count | present basis | main gap |
|---|---:|---:|---|---|
| Core crypto operations | 16 | 4 | encrypt/decrypt/AAD/receipt | sign, verify, MAC, random, rewrap, data key APIs |
| Key lifecycle | 14 | 4 | domain states, rotation docs, destruction model | local CRUD/restore/import/version APIs |
| Key custody and HSM | 13 | 5 | HSM tiers, BYOK/HYOK docs, domain origins | IaC, adapter contracts, attestation evidence |
| Authorization and policy | 10 | 5 | cloud-iam intent, Cedar intent, idempotency tests | policy files, grants, alias IAM mapping |
| Audit and evidence | 9 | 5 | use receipts, audit docs, destruction receipts | event schema, immutable evidence, benchmark evidence |
| Deployment and operations | 15 | 2 | runbooks and tenant_class docs | six contexts, OpenTofu, OS manifest, SLOs |
| Quota and performance | 10 | 3 | tenant_class matrix and benchmark docs | measured evidence, per-context capacity model |
| Migration and interoperability | 11 | 4 | AWS/Vault playbook, adapters described | GCP migration, XKS/EKM protocol, import workflow |
| Self-hosted Vault parity | 12 | 4 | Shamir/quorum/HSM/bootstrap references | transit rewrap/batch/derive/convergent, seal migration |
| Oyatie additive controls | 8 | 6 | data-class, purpose, AAD digest, Cedar, residency | policy files and end-to-end enforcement |

## Section 6 - Headline Gap Analysis: Top 15 Missing Capabilities

Gap 01: Local key CRUD API.
Evidence: repo-level OpenAPI only has encrypt/decrypt authorization endpoints at `contracts/openapi/cloud/cloud-kms-v1.yaml:9-168`.
Implementation hook: add key lifecycle endpoints or declare that key CRUD is owned by another cloud-kms crate/service surface.

Gap 02: Local import/BYOK workflow.
Evidence: local FAQ describes PKCS#11 import and AWS XKS at `faqs/kms-engineer-faq.md:48-56`, but no OpenAPI/IaC workflow exists.
Implementation hook: add `ImportKeyMaterial`, wrapping-key, receipt, expiration, and reimport contracts.

Gap 03: External key store/EKM/XKS protocol.
Evidence: AWS and Google both expose external key patterns; local docs name XKS but no protocol.
Implementation hook: define provider-neutral external-key adapter contract for AWS XKS, Google EKM, OCI Vault, and self-hosted HSM.

Gap 04: Multi-region key replication.
Evidence: FAQ claims cross-region replication at `faqs/kms-engineer-faq.md:114-120`; no local design.
Implementation hook: add key replica state machine, import-material behavior, region failover, and residency checks.

Gap 05: Signing/verification API.
Evidence: tenant_class matrix names HSM signing at `retired tenant_class adoption artifact:7-11`; OpenAPI lacks sign/verify.
Implementation hook: add sign/verify authorization receipts and key usage model.

Gap 06: MAC/HMAC API.
Evidence: AWS/GCP/Vault all cover MAC/HMAC surfaces; local OpenAPI lacks them.
Implementation hook: add MAC key type and purpose-specific authorization receipts.

Gap 07: Random generation.
Evidence: AWS KMS, Google Cloud HSM, and Vault transit all expose random generation.
Implementation hook: decide whether cloud-kms or cloud-secrets owns random bytes; document explicit owner.

Gap 08: Rewrap operation.
Evidence: Vault transit rewrap is a major self-hosted counterpart capability; local cryptoshred/rotation docs imply but do not expose it.
Implementation hook: add `rewrap` endpoint with no-plaintext proof and audit receipt.

Gap 09: Key version state matrix.
Evidence: Google Cloud KMS has explicit key version states; local domain has states but no user-visible matrix.
Implementation hook: map PendingImport/Enabled/Disabled/PendingDeletion/Destroyed to API transitions and failures.

Gap 10: Autokey/resource-specific key provisioning.
Evidence: Google Cloud KMS Autokey provisions CMEKs on demand; Oyatie storage/data docs depend on cloud-kms but no auto-provisioning contract exists.
Implementation hook: add cloud-resource-driven CMK provisioning flow through cloud-iac/cloud-iam/cloud-kms.

Gap 11: Key Access Justification equivalent.
Evidence: Google KAJ is a distinct compliance capability; Oyatie has audit-chain and data-class semantics but no justification contract.
Implementation hook: add `justification_code`, `policy_basis`, and `actor_reason` fields to use receipt schema.

Gap 12: Alias and grant management.
Evidence: AWS aliases/grants are common operator ergonomics; local docs do not cover them.
Implementation hook: decide whether Cedar replaces grants and whether aliases are allowed under tenant boundaries.

Gap 13: OpenTofu deployment modules.
Evidence: no local `iac/` exists; canonical direction requires OpenTofu only.
Implementation hook: create context modules with HSM/provider resources and state backends.

Gap 14: OS and package support.
Evidence: no local `supported-oses.json` exists; KMS adapters are OS-sensitive.
Implementation hook: define Tier-1 package/runtime support and HSM/TPM/Secure Enclave compatibility.

Gap 15: Measured benchmark evidence.
Evidence: benchmark doc claims measured dates but evidence path is absent.
Implementation hook: re-run benchmark harness, store immutable evidence, and separate target numbers from measurements.

## Section 7 - Additive Surface

Additive 01: Data-class-aware KMS receipts.
Rationale: vendor KMS products provide encryption context and audit logs, but Oyatie links KMS receipt data to data boundary classes in API and domain types.
Additive 02: Purpose-bound KMS use.
Rationale: Oyatie enum values bind key use to cloud object storage, block storage, workspace recordings, secret provider, cross-region replication, and database backup.
Additive 03: AAD fingerprint as a first-class contract field.
Rationale: vendors support AAD/encryption context, but Oyatie stores the fingerprint in request validation and receipt semantics.
Additive 04: Cedar-gated cryptographic authorization.
Rationale: AWS/GCP/IAM/Vault policy are not Cedar; Oyatie can express tenant, data-class, purpose, actor, and residency in one policy plane.
Additive 05: Per-pack HSM validation.
Rationale: the domain model names KCMVP, FIPS 140-3, Cryptrec, Common Criteria EAL4, and PCI HSM as explicit validation options.
Additive 06: Cryptoshredding proof ref.
Rationale: key deletion is common, but Oyatie models proof refs and destruction SLA as product evidence.
Additive 07: Cross-service data-boundary handoffs.
Rationale: cloud-storage, cloud-data, mail, audit-chain, and cloud-network docs all depend on KMS semantics, making KMS a shared data-boundary service.
Additive 08: OCI Always Free demo_trial requirement.
Rationale: not a vendor KMS capability, but canonical Oyatie direction requires KMS to fit a zero-cost guest-on-OCI demo_trial tenant_class profile.
Additive 09: Rust-only backend and generated contract expectations.
Rationale: none of the three counterparts impose Oyatie's Rust-strict policy, but the repo does.
Additive 10: Six-context deployability.
Rationale: AWS and Google are provider-specific, Vault is self-hosted; Oyatie must span public cloud, guest cloud, on-prem, colo, and Oyatie-as-provider.

## Bottom Line

The current cloud-kms artifact set is a partial match against the union coverage of AWS KMS, Google Cloud KMS, and HashiCorp Vault self-hosted.
It is strongest in tenant-bound encrypt/decrypt authorization receipts, AAD/data-class purpose binding, HSM-tenant_class thinking, incident runbooks, and cryptoshredding narrative.
It is weakest in local ownership of key lifecycle APIs, import/external-key-store contracts, sign/MAC/random/rewrap operations, deployment/IaC/OS/SLO evidence, and self-hosted Vault parity.
Wave 14 should treat this service as a real Rust nucleus with a missing KMS control-plane envelope.

## Appendix A - Capability Classification Notes

Classification note 01: "present" means the current repo has either local docs plus repo-level code/contract evidence, not merely aspirational prose.
Classification note 02: "present-doc" means the local microservice path documents the capability but this audit did not find a local machine-readable contract or implementation pointer.
Classification note 03: "present-repo" means the capability exists in `contracts/` or `crates/` but is not represented inside `microservices/cloud-kms/`.
Classification note 04: "present-additive" means Oyatie has a stronger or more specific product concept than the counterparts, but may still need enforcement artifacts.
Classification note 05: "gap-api" means the capability needs an OpenAPI/gRPC/event contract or an explicit non-goal decision.
Classification note 06: "gap-iac" means the capability depends on deployable substrate modules under OpenTofu context directories.
Classification note 07: "gap-context" means the service has not mapped the capability across six deployment contexts.
Classification note 08: "gap-doc" means the capability likely belongs in the doc set but the local document is absent or counterpart coverage is incomplete.
Classification note 09: "gap-policy" means Cedar/IAM/policy behavior is described but no policy artifact exists in the microservice path.
Classification note 10: "gap-handoff" means the capability requires another microservice and lacks a reciprocal ownership contract.
Classification note 11: "gap-capacity-model" means the capability needs quotas, throttle behavior, HSM sizing, and tenancy math.
Classification note 12: "optional-gap" means the capability exists in one counterpart but should be accepted or rejected by product decision before implementation.
Classification note 13: AWS KMS heavily weights managed regional service integration, key policies, grants, custom key stores, XKS, multi-Region keys, and request quotas.
Classification note 14: Google Cloud KMS heavily weights resource hierarchy, Cloud HSM, EKM, Autokey, key version lifecycle, and CMEK integrations.
Classification note 15: Vault self-hosted heavily weights transit cryptography, policy/auth/audit, seal/unseal, operator ceremony, and self-controlled replication.
Classification note 16: Oyatie cloud-kms should not blindly clone all three counterparts; union coverage means each capability must be implemented, explicitly delegated, or explicitly rejected.
Classification note 17: The strongest Oyatie differentiator is not raw cryptography; it is policy-bound, data-class-aware evidence that connects KMS use to tenant purpose and audit-chain proof.
Classification note 18: The largest implementation risk is exposing broad key lifecycle operations before the six-context custody/IaC/OS matrix is settled.
Classification note 19: The largest documentation risk is keeping HSM/BYOK/HYOK claims in prose while registry/catalog state still says security review is unreviewed.
Classification note 20: The highest-value next artifact is a local cloud-kms architecture document that aligns repo-level Rust crates, OpenAPI, IaC, policy, and counterpart parity in one ownership map.
