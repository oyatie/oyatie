# cloud-secrets capability tenant_class deltas vs counterparts - 2026-05-20

Audit owner: Wave 2 Batch 2.1 sole-owner audit for `cloud-secrets`.
Tier model under audit: demo_trial and paid tenant_class.
Counterpart set: AWS Secrets Manager, Google Secret Manager, HashiCorp Vault Secrets.
Delta rule: compare Oyatie tenant_class intent against closest counterpart tier/usage mode, then mark ahead, parity, catch-up, incoherent, or not-applicable.

Citation anchor 1: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2495` for deployment and OpenTofu constraints.
Citation anchor 2: `specs/master-plan-sequencing.json:704-866` for contexts, OpenTofu, OS, Rust, and OCI Always Free.
Citation anchor 3: `microservices/cloud-secrets/retired tenant_class adoption artifact:1-98` for current Oyatie tenant_class matrix.
Citation anchor 4: `microservices/cloud-secrets/PRD.md:20-331` for product purpose and acceptance criteria.
Citation anchor 5: `microservices/cloud-secrets/ARCHITECTURE.md:3-704` for architecture and credential isolation.
Citation anchor 6: `docs/standards/documentation-rigor.md:1-220` for substance and intern-buildability.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html`.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/reference_limits.html`.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html`.
Google source: `https://docs.cloud.google.com/secret-manager/docs/overview`.
Google source: `https://docs.cloud.google.com/secret-manager/quotas`.
Google source: `https://docs.cloud.google.com/secret-manager/docs/secret-manager-secrets-comparison`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs`.
HashiCorp source: `https://developer.hashicorp.com/hcp/docs/vault-secrets/dynamic-secrets`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/audit`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/enterprise/namespaces`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/enterprise/replication`.

## §1 Tier definitions in Oyatie

1. demo_trial current evidence: `retired tenant_class adoption artifact:11-27`.
2. demo_trial current doc says shared CockroachDB, shared OpenBao, shared HSM partition, 200 secrets, p95 read <=30ms, and 50 rps.
3. demo_trial current doc says price is approximately USD 45/month.
4. demo_trial current doc does not map guest-on-oci demo_trial to OCI Always Free.
5. demo_trial corrected definition should mean minimal production-capable secret reference service.
6. demo_trial corrected definition should include zero raw-secret enforcement.
7. demo_trial corrected definition should include one OpenBao-backed tenant namespace.
8. demo_trial corrected definition should include sealed audit rows for all mutating operations.
9. demo_trial corrected definition should include cache-hit p99 target no stronger than the demo_trial context can prove.
10. demo_trial corrected definition should include rotation for basic application credentials.
11. demo_trial corrected definition should include at-rest encryption and TLS.
12. demo_trial corrected definition should include no paid OCI primitives when deployed as guest-on-oci demo_trial.
13. demo_trial corrected definition should not claim multi-region HA unless the context module proves it.
14. demo_trial corrected definition should not claim dynamic leases unless implemented.
15. demo_trial corrected definition should be a constrained baseline, not a marketing tier.
16. paid current evidence: `retired tenant_class adoption artifact:29-45`.
17. paid current doc says dedicated OpenBao namespace, optional AWS KMS key, 2,000 secrets, and 500 rps.
18. paid current doc uses AWS KMS naming that needs provider-neutral abstraction.
19. paid corrected definition should mean paid baseline with better quotas and managed rotation.
20. paid corrected definition should include per-tenant namespace lifecycle APIs.
21. paid corrected definition should include version aliases or labels.
22. paid corrected definition should include defined secret size and version retention limits.
23. paid corrected definition should include private endpoint or mesh access per context.
24. paid corrected definition should include basic regional replication only when context supports it.
25. paid corrected definition should include dashboard and alert coverage.
26. paid corrected definition should include OpenTofu module evidence for every supported context.
27. paid current evidence: `retired tenant_class adoption artifact:47-63`.
28. paid current doc says dedicated HSM partition, dedicated OpenBao cluster, dual-region replication, 50,000 secrets, and 5,000 rps.
29. paid current doc names AWS KMS CloudHSM, Azure Dedicated HSM, and YubiHSM as examples.
30. paid corrected definition should mean production scale with regional HA.
31. paid corrected definition should include strict audit fail-closed mode.
32. paid corrected definition should include regional data-residency endpoints where applicable.
33. paid corrected definition should include replication lag SLO.
34. paid corrected definition should include tenant namespace drift detection.
35. paid corrected definition should include compliance export and regulator-ready evidence.
36. paid corrected definition should include HSM/BYOK onboarding and rekey workflows.
37. paid corrected definition should include measured performance evidence, not target-only numbers.
38. paid current evidence: `retired tenant_class adoption artifact:65-81`.
39. paid current doc says FIPS HSM, dedicated cell, multi-region active-active, 500,000 secrets, and 50,000 rps.
40. paid corrected definition should mean hyperscaler-grade, single-tenant capable service.
41. paid corrected definition should include dedicated OpenBao/control plane cell.
42. paid corrected definition should include dedicated audit devices and independent audit durability.
43. paid corrected definition should include active-active or explicitly documented active/passive topology.
44. paid corrected definition should include single-tenant cryptographic boundary.
45. paid corrected definition should include disaster recovery promotion semantics.
46. paid corrected definition should include measured p99 under fail-closed audit.
47. paid corrected definition should include workload isolation and cell evacuation procedures.
48. Cross-tenant_class axis: maximum secrets per tenant.
49. Cross-tenant_class axis: read and write throughput.
50. Cross-tenant_class axis: rotation cadence and rotation throughput.
51. Cross-tenant_class axis: HSM/BYOK custody.
52. Cross-tenant_class axis: region and context count.
53. Cross-tenant_class axis: audit durability and fail-closed posture.
54. Cross-tenant_class axis: version retention and rollback.
55. Cross-tenant_class axis: lease/dynamic-secret support.
56. Cross-tenant_class axis: Kubernetes sync or deliberate runtime-reference-only stance.
57. Cross-tenant_class axis: OpenTofu module maturity.
58. Cross-tenant_class axis: OS support and package support.
59. Cross-tenant_class axis: compliance evidence export.
60. Cross-tenant_class verdict: the named tenant_classes are useful, but they need canonical OCI, context, OS, quota, and counterpart semantics before they can govern implementation.

## §2 Counterpart tenant_class mapping

1. AWS Secrets Manager mapping is usage/quota-based rather than named demo_trial/paid tenant_class tiers.
2. AWS free/low-use equivalent: low secret count, low request volume, AWS managed key, default quotas, no replica Regions.
3. AWS standard equivalent: default regional Secrets Manager with KMS, IAM/resource policies, rotation, CloudTrail, and default quotas.
4. AWS advanced equivalent: multi-Region replication, customer-managed KMS keys, VPC endpoints, ABAC, CloudWatch/Config/GuardDuty monitoring, and high request volumes.
5. AWS dedicated-equivalent limitation: AWS Secrets Manager is regional/account service, not a customer-dedicated cell in the same way Oyatie paid proposes.
6. AWS emphasized axes: request quotas, Region/account scale, KMS encryption, IAM policy, managed rotation, replication, and logging.
7. AWS gap against Oyatie: no provider-agnostic six-context deployment because AWS is the provider context.
8. AWS gap against Oyatie: no OCI Always Free demo_trial concept.
9. AWS gap against Oyatie: no OpenBao/OpenTofu/Rust-strict platform doctrine.
10. AWS advantage over Oyatie: mature quotas, service integration, private endpoint, and rotation semantics.
11. Google Secret Manager mapping is usage/location/quota-based rather than named demo_trial/paid tenant_class tiers.
12. Google free/low-use equivalent: low project usage under default quotas with global service and default encryption.
13. Google standard equivalent: global Secret Manager with IAM, versions, aliases, rotation, CMEK optional, and access quota.
14. Google advanced equivalent: regional service, regional endpoints, user-managed replication, CMEK, organization policies, and explicit data residency.
15. Google dedicated-equivalent limitation: regional service enforces locality but is not the same as Oyatie single-tenant cell.
16. Google emphasized axes: versioning, IAM, CMEK, replication policy, regional endpoints, data residency, and project quotas.
17. Google gap against Oyatie: no six-context provider-agnostic deployment.
18. Google gap against Oyatie: no OpenBao-first self-host path.
19. Google gap against Oyatie: no Merkle/Ed25519 audit-chain doctrine in product surface.
20. Google advantage over Oyatie: regional endpoint semantics and published access/read/write quotas.
21. HashiCorp Vault Secrets mapping spans HCP Vault Secrets tiers, Vault Enterprise/Dedicated, and self-managed Vault deployment modes.
22. HashiCorp low-use equivalent: static KV secrets with basic auth/policy and low traffic.
23. HashiCorp standard equivalent: HCP Vault Secrets Standard features such as auto-rotating and dynamic secrets.
24. HashiCorp enterprise equivalent: namespaces, performance replication, DR replication, audit devices, sync, HSM/FIPS, PKI, and advanced auth.
25. HashiCorp dedicated equivalent: HCP Vault Dedicated or self-managed dedicated clusters.
26. HashiCorp emphasized axes: static secrets, dynamic secrets, leases, revocation, audit devices, namespaces, replication, and secrets engines.
27. HashiCorp gap against Oyatie: no native Oyatie six-context/OpenTofu/OCI demo_trial tenant_class doctrine.
28. HashiCorp gap against Oyatie: Vault Enterprise namespaces may be license-gated, while Oyatie aims for built-in tenant namespace control.
29. HashiCorp advantage over Oyatie: mature leases, dynamic secrets, audit-device semantics, replication categories, and Kubernetes sync.
30. Counterpart tenant_class warning: AWS/GCP do not map cleanly to service tiers; mapping is by feature/capacity axis.
31. Counterpart tenant_class warning: HashiCorp maps more naturally to demo_trial/paid tenant_class because deployment topology can vary.
32. Counterpart tenant_class warning: Oyatie's paid is a target architecture, not a current implementation claim.
33. Counterpart tenant_class warning: Oyatie demo_trial on OCI is more constrained than AWS/GCP low-use modes because it must fit Always Free.
34. Counterpart tenant_class warning: vendor quotas are provider-side constraints, while Oyatie targets are service-owned planning constraints.
35. Counterpart tenant_class warning: official public docs rarely publish p99 latency; quota and limit numbers should not be treated as latency benchmarks.
36. Mapping conclusion for AWS demo_trial: closest is low-use single-Region secret with AWS managed key.
37. Mapping conclusion for AWS paid: closest is standard service with rotation, policies, logs, and KMS.
38. Mapping conclusion for AWS paid: closest is replicated, private-endpoint, customer-managed KMS usage at high volume.
39. Mapping conclusion for AWS paid: no exact equivalent; use high-volume multi-Region AWS plus dedicated account controls.
40. Mapping conclusion for Google demo_trial: closest is low-use global service with default encryption.
41. Mapping conclusion for Google paid: closest is global service with IAM/CMEK/version/rotation.
42. Mapping conclusion for Google paid: closest is regional service with data residency and user-managed replication.
43. Mapping conclusion for Google paid: no exact equivalent; use strict regional service with dedicated projects and org policy.
44. Mapping conclusion for HashiCorp demo_trial: closest is static KV/OpenBao/Vault with basic policies.
45. Mapping conclusion for HashiCorp paid: closest is HCP Vault Secrets Standard dynamic/rotating secret usage.
46. Mapping conclusion for HashiCorp paid: closest is Enterprise namespaces, audit, HSM, and replication.
47. Mapping conclusion for HashiCorp paid: closest is dedicated/self-managed Enterprise or HCP Dedicated with isolated clusters.
48. Tier-delta baseline: use AWS/GCP as cloud-managed secret-manager baselines.
49. Tier-delta baseline: use HashiCorp as advanced secret-platform baseline.
50. Tier-delta baseline: use Oyatie canonical constraints as non-negotiable local differentiators.

## §3 Per-Oyatie-tenant_class delta tables

### demo_trial tenant_class table

| feature | Oyatie demo_trial | AWS equivalent | Google equivalent | HashiCorp equivalent | Gap classification |
|---|---|---|---|---|---|
| Basic secret storage | planned shared OpenBao | yes | yes | yes | parity target |
| Runtime read | planned SecretReference | yes | yes | yes | partial |
| Public grammar | conflicting | ARN/name | resource/version | path/mount | catch-up |
| Versioning | unclear | yes | yes | yes | catch-up |
| Version aliases | absent | staging labels | aliases | paths/versions | catch-up |
| Rotation | planned | yes | yes | yes | partial |
| Dynamic leases | absent | no core | no core | yes | catch-up to HashiCorp |
| Audit | planned sealed audit | CloudTrail | Cloud Audit Logs | audit devices | partial |
| Audit fail-closed | conflicting | no direct | no direct | yes | catch-up |
| IAM policy | Cedar/OpenBao planned | IAM/resource | IAM | Vault policy | partial |
| Tenant namespace | planned | account/policy | project/IAM | namespace | partial |
| Secret size limit | absent | 65,536 bytes | not in audited excerpt | Vault storage limits | catch-up |
| Read quota | target 50-200 rps by context | 10,000 rps | 1,500 rps access | deployment-bound | catch-up |
| Write quota | target 10-25 rps | 50 rps class | 10 rps | deployment-bound | partial |
| Region replication | absent for demo_trial | optional | automatic/user | possible | catch-up |
| Private endpoint | absent | VPC endpoint | regional/private controls | self-host network | catch-up |
| OCI Always Free | required, missing | not applicable | not applicable | not applicable | incoherent |
| OpenTofu module | missing | not applicable | not applicable | Terraform provider exists | incoherent |
| OS manifest | missing | SaaS | SaaS | self-host docs | incoherent |
| Rust strict | doctrine present | SDKs many langs | SDKs many langs | SDKs many langs | ahead if enforced |
| K8s sync | absent | EKS guidance | K8s guidance | VSO/HVS | catch-up |
| Data residency | policy partial | Region/replica | regional service | namespaces/replication | partial |
| HSM/BYOK | shared HSM partition | KMS | CMEK | HSM/seal | partial |
| Cost model | current USD 45/mo | usage priced | usage priced | product/deploy priced | incoherent for OCI |
| Status | not build-ready | mature | mature | mature | catch-up |

### paid tenant_class table

| feature | Oyatie paid | AWS equivalent | Google equivalent | HashiCorp equivalent | Gap classification |
|---|---|---|---|---|---|
| Secret count | 2,000 current doc | 500,000 account/Region max | quota/project model | deployment-bound | catch-up |
| Read throughput | 500 rps current doc | 10,000 rps GetSecretValue | 1,500 rps access | deployment-bound | partial |
| Dedicated namespace | yes in doc | policy/account isolation | project/resource IAM | namespace | partial |
| Optional KMS | AWS-named in doc | KMS CMK | CMEK | HSM/seal/transit | provider-leak |
| Managed rotation | planned | yes | yes | yes | partial |
| Version retention | absent | 100 versions | versions | KV versions | catch-up |
| Version alias | absent | staging labels | aliases | path/version patterns | catch-up |
| Quota contract | absent | published | published | limits/advisory | catch-up |
| Regional endpoint | absent | endpoints | regional endpoints | deployment endpoint | catch-up |
| Private endpoint | absent | VPC endpoint | private/regional controls | self-host | catch-up |
| Audit query | planned | CloudTrail query | Cloud Logging | audit devices | partial |
| Multiple audit sinks | absent | service logs | service logs | yes | catch-up |
| Namespace API | partial proto | no exact | no exact | yes | partial |
| HSM integration | optional | KMS/CloudHSM | KMS/HSM | HSM/FIPS | partial |
| OpenTofu | missing | not applicable | not applicable | provider tooling | incoherent |
| OS manifest | missing | SaaS | SaaS | self-host | incoherent |
| SDK | Rust plus forbidden planned | many SDKs | many SDKs | many SDKs | incoherent |
| K8s consumption | not specified | EKS integration | K8s guidance | VSO | catch-up |
| Replication | not clear | multi-Region | automatic/user | performance/DR | catch-up |
| Secret scanning | planned | CodeGuru/Q | best practices | Radar | partial |
| Cost | USD 280 current doc | usage priced | usage priced | product/deploy priced | partial |
| Compliance | doc present | compliance validation | compliance controls | compliance support | partial |
| Data residency | policy partial | regional choices | regional service | namespaces/filters | partial |
| CI evidence | missing | provider service | provider service | operator dependent | catch-up |
| Status | paid baseline target | mature | mature | mature | catch-up |

### paid tenant_class table

| feature | Oyatie paid | AWS equivalent | Google equivalent | HashiCorp equivalent | Gap classification |
|---|---|---|---|---|---|
| Dedicated cluster | current doc says yes | no exact | no exact | dedicated/self-managed | partial |
| Dedicated HSM partition | current doc says yes | CloudHSM/KMS | Cloud KMS/HSM | HSM/FIPS | partial |
| Dual-region replication | current doc says yes | replica Regions | user-managed replication | performance/DR replication | partial |
| Read throughput | 5,000 rps current doc | 10,000 rps GetSecretValue | 1,500 rps access | deployment-bound | parity/catch-up |
| Write throughput | not explicit in tenant_class doc | 50 rps class | 10 rps default | deployment-bound | missing |
| Secret count | 50,000 | 500,000 max | project model | deployment-bound | partial |
| Audit fail-closed | not consistently specified | no direct | no direct | yes | catch-up |
| Regional endpoint | not specified | regional service endpoint | regional endpoint | deployment endpoint | catch-up |
| Replica promotion | absent | yes | no direct | DR promotion | catch-up |
| Path filters | absent | IAM conditions | location policy | yes | catch-up |
| Dynamic leases | absent | no core | no core | yes | catch-up |
| Prefix revoke | absent | no | no | yes | catch-up |
| Transit crypto | not scoped | KMS adjacent | KMS adjacent | yes | ownership gap |
| PKI | not scoped | ACM adjacent | CAS adjacent | yes | ownership gap |
| K8s sync | absent | EKS usage | K8s guidance | VSO | catch-up |
| Drift remediation | absent | no direct | no direct | VSO | catch-up |
| Compliance export | planned | AWS Artifact/Config | compliance/audit | compliance | partial |
| Multi-context IaC | missing | AWS only | GCP only | self/HCP | incoherent |
| OS support | missing | SaaS | SaaS | self-host concern | incoherent |
| Sigstore attestation | missing | no direct | no direct | plugin signing adjacent | incoherent |
| Context labels | absent | CloudWatch dims | Cloud Monitoring labels | telemetry labels | catch-up |
| Bench evidence | unverified | quotas mature | quotas mature | limits mature | catch-up |
| Cost | USD 2400 current doc | usage priced | usage priced | product/deploy priced | partial |
| Provider-neutral HSM | examples leak providers | AWS-native | GCP-native | provider-flexible | partial |
| Status | production-scale target | mature | mature | mature | catch-up |

### paid tenant_class table

| feature | Oyatie paid | AWS equivalent | Google equivalent | HashiCorp equivalent | Gap classification |
|---|---|---|---|---|---|
| Dedicated cell | current doc says yes | no exact | no exact | Dedicated/self-managed | partial target |
| FIPS HSM | current doc says yes | CloudHSM/KMS FIPS options | Cloud HSM/KMS | HSM/FIPS | partial |
| Multi-region active-active | current doc says yes | multi-Region replicas | global/replication choices | performance replication | partial |
| Single-tenant capable | current doc says yes | account isolation | project/org isolation | dedicated clusters/namespaces | partial |
| Read throughput | 50,000 rps current doc | 10,000 rps GetSecretValue default | 1,500 rps access default | deployment-bound | ahead target |
| Secret count | 500,000 | 500,000 AWS max | project model | deployment-bound | parity target |
| Write scale | not enough detail | 50 rps class | 10 rps default | deployment-bound | missing |
| Disaster promotion | absent | replica promotion | regional behavior | DR promotion | catch-up |
| Response wrapping | absent | no direct | no direct | replication activation tokens | catch-up |
| Multiple audit devices | absent | service logs | service logs | yes | catch-up |
| Audit HMAC | Merkle target | service logs | service logs | HMAC audit values | partial/ahead target |
| Dynamic secrets | absent | limited/partners | no core | yes | catch-up |
| Lease advisory scale | absent | no core | no core | 256,000 advisory | catch-up |
| Transit | not scoped | KMS adjacent | KMS adjacent | yes | ownership gap |
| Transform/tokenize | absent | no core | no core | yes | catch-up if in scope |
| PKI/cert lifecycle | absent | ACM adjacent | CAS adjacent | yes | catch-up if in scope |
| K8s VSO equivalent | absent | EKS integration | K8s guidance | VSO | catch-up |
| OpenTofu all contexts | missing | no | no | tooling | incoherent |
| OS all Tier-1 | missing | SaaS | SaaS | self-host | incoherent |
| OCI Always Free | not paid issue | no | no | no | not-applicable |
| Provider API isolation | not proven | AWS-native | GCP-native | self/HCP | partial |
| Measured evidence | absent | mature service | mature service | mature platform | catch-up |
| Compliance posture | broad docs | compliance validation | compliance controls | compliance/FIPS | partial |
| Cost | USD 11000 current doc | usage priced | usage priced | product/deploy priced | partial |
| Status | hyperscaler-bar target | mature managed | mature managed | mature platform | catch-up |

## §4 OCI demo_trial tenant_class = Always Free reconciliation

1. Canonical rule: guest-on-oci demo_trial must map to OCI Always Free.
2. Canonical source: `specs/master-plan-sequencing.json:856-866`.
3. Current gap: `retired tenant_class adoption artifact:11-27` prices demo_trial at about USD 45/month.
4. Current gap: no `iac/oci-guest/always-free/` directory exists.
5. Current gap: `cost-budget.md:22-30` names paid OCI resources rather than a strict Always Free envelope.
6. Corrected OCI demo_trial tenant_class compute budget should fit Ampere A1 4 OCPU and 24GB memory aggregate.
7. Corrected OCI demo_trial tenant_class fallback compute should treat AMD micro as optional low-capacity support, not a paid dependency.
8. Corrected OCI demo_trial tenant_class storage budget should fit 200GB block volume aggregate.
9. Corrected OCI demo_trial tenant_class object storage budget should fit 10GB object and 10GB archive where used.
10. Corrected OCI demo_trial tenant_class database use should fit the Always Free autonomous database allowance only if architecture explicitly uses it.
11. Corrected OCI demo_trial tenant_class load balancer target should fit Always Free bandwidth constraints.
12. Corrected OCI demo_trial tenant_class egress planning should respect the Always Free egress allowance.
13. Corrected OCI demo_trial tenant_class should use OpenBao in the available compute envelope.
14. Corrected OCI demo_trial tenant_class should avoid paid HSM unless explicitly upgraded to paid tenant_class.
15. Corrected OCI demo_trial tenant_class should avoid paid managed database unless explicitly upgraded to paid tenant_class.
16. Corrected OCI demo_trial tenant_class should avoid paid observability exports unless within free service allowances.
17. Corrected OCI demo_trial tenant_class should reduce secret count target below current generic demo_trial if necessary.
18. Corrected OCI demo_trial tenant_class should reduce rotation throughput below generic demo_trial if necessary.
19. Corrected OCI demo_trial tenant_class should reduce audit retention window or use compact audit batching if necessary.
20. Corrected OCI demo_trial tenant_class should never silently spill to paid OCI resources.
21. Feature requiring paid tenant_class on OCI: dedicated HSM partition.
22. Feature requiring paid tenant_class on OCI: high-write rotation bursts beyond Always Free compute envelope.
23. Feature requiring paid tenant_class on OCI: multi-region active-active replication with paid network/storage.
24. Feature requiring paid tenant_class on OCI: large tenant namespace counts beyond free memory/storage.
25. Feature requiring paid tenant_class on OCI: large audit retention beyond free object/archive storage.
26. Feature requiring paid tenant_class on OCI: dedicated paid load balancer.
27. Feature requiring paid tenant_class on OCI: managed paid database.
28. Feature requiring paid tenant_class on OCI: compliance export storage exceeding free allowance.
29. Feature requiring paid tenant_class on OCI: dedicated customer cell.
30. Feature requiring paid tenant_class on OCI: strict paid p99 targets.
31. demo_trial reconciliation finding: current docs are incoherent with canonical OCI policy.
32. demo_trial reconciliation remediation: add service-local `iac/oci-guest/always-free/` with resource budget assertions.
33. demo_trial reconciliation remediation: update tenant_class matrix to split generic demo_trial from OCI demo_trial tenant_class constraints.
34. demo_trial reconciliation remediation: update performance targets to use OCI demo_trial tenant_class row from the performance target doc.
35. demo_trial reconciliation remediation: add cost-budget guardrail that rejects paid resources in demo_trial guest-on-oci.
36. demo_trial reconciliation remediation: add deployment validation that fails if plan contains paid OCI resources.
37. demo_trial reconciliation remediation: add documentation saying which features require paid tenant_class on OCI.
38. demo_trial reconciliation remediation: attach audit evidence path for the Always Free plan.
39. demo_trial reconciliation remediation: include OpenTofu state backend and module attestation.
40. demo_trial reconciliation status: not satisfied as of this audit.

## §5 Findings by tier

1. demo_trial classification: catch-up overall.
2. demo_trial ahead area: zero-raw-secret doctrine and typed secret aspiration.
3. demo_trial parity area: basic secret storage and runtime retrieval intent.
4. demo_trial catch-up area: versioning, aliases, quotas, endpoint behavior, and audit semantics.
5. demo_trial incoherent area: OCI Always Free not represented despite canonical mandate.
6. demo_trial action: fix OCI demo_trial tenant_class before claiming deployable guest-on-oci.
7. paid classification: catch-up with partial parity.
8. paid ahead area: provider-agnostic goal if OpenTofu and context modules land.
9. paid parity area: planned rotation, namespace, audit, and dashboard scope.
10. paid catch-up area: public quotas, version retention, private endpoints, and generated SDK strategy.
11. paid incoherent area: AWS KMS naming leaks provider-specific implementation into a provider-agnostic tier.
12. paid action: abstract KMS/HSM adapters and add OpenTofu evidence.
13. paid classification: partial parity target, current catch-up.
14. paid ahead area: potential audit-chain and multi-context portability.
15. paid parity area: high read throughput target can be comparable with AWS default read quota in controlled contexts.
16. paid catch-up area: replica promotion, regional endpoints, dynamic leases, K8s sync, and strict audit behavior.
17. paid incoherent area: dual-region replication is claimed without context OpenTofu modules.
18. paid action: tie replication to stateful OpenTofu modules and measured SLOs.
19. paid classification: hyperscaler target, not a current claim.
20. paid ahead area: target read throughput and provider-owned cells can exceed public default quotas if measured.
21. paid parity area: FIPS HSM, dedicated cell, and single-tenant control align with high-end Vault Dedicated patterns.
22. paid catch-up area: mature leases, audit devices, replication activation, transit/PKI ownership, and DR promotion.
23. paid incoherent area: no measured evidence, no OS manifest, no OpenTofu modules, and no exact SDK strategy.
24. paid action: defer public paid claim until build evidence exists.
25. Cross-tenant_class P1: deployment context evidence missing for every tier.
26. Cross-tenant_class P1: OpenTofu coverage missing for every tier.
27. Cross-tenant_class P1: OS support matrix missing for every tier.
28. Cross-tenant_class P1: Rust-strict SDK plan unresolved for every tier.
29. Cross-tenant_class P1: SecretReference grammar unresolved for every tier.
30. Cross-tenant_class P1: OCI demo_trial tenant_class Always Free missing for the one tier/context combination where it is mandatory.
31. Cross-tenant_class P2: benchmark evidence should be downgraded to target/provenance until measured build evidence exists.
32. Cross-tenant_class P2: cost numbers should be tied to context-specific resource plans.
33. Cross-tenant_class P2: tenant_class names should include exact quotas and SLOs.
34. Cross-tenant_class P2: provider-specific examples should become adapter mappings.
35. Cross-tenant_class conclusion: the tenant_class model is directionally useful but cannot govern implementation until canonical constraints and counterpart gaps are reconciled.
36. Aggregation handoff: demo_trial remediation should be sequenced before any guest-on-oci deployability claim.
37. Aggregation handoff: paid remediation should focus on stable quota, version, and endpoint contracts.
38. Aggregation handoff: paid remediation should focus on replication semantics, fail-closed audit, and measured SLO evidence.
39. Aggregation handoff: paid remediation should remain gated behind measured dedicated-cell proof.
40. Aggregation handoff: all tenant_classes need one SecretReference grammar before any SDK or tutorial update.
41. Aggregation handoff: all tenant_classes need a generated-SDK decision before non-Rust client claims continue.
42. Aggregation handoff: all tenant_classes need context OpenTofu modules before cost or capacity figures are authoritative.
43. Aggregation handoff: all tenant_classes need `supported-oses.json` before package and CI claims are credible.
44. Aggregation handoff: all tenant_classes need an audit-mode decision before latency numbers can be interpreted.
45. Final tenant_class verdict: no tenant_class is currently clean enough to serve as an implementation gate without remediation.
