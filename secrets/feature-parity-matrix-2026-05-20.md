# cloud-secrets feature parity matrix - 2026-05-20

Audit owner: Wave 2 Batch 2.1 sole-owner audit for `cloud-secrets`.
Counterpart set: AWS Secrets Manager, Google Secret Manager, HashiCorp Vault Secrets.
Parity standard: union coverage, not lowest-common-denominator coverage.
Method: official counterpart docs, current service docs, local inventory, and canonical Oyatie constraints.

Citation anchor 1: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2495` for multi-context and OpenTofu constraints.
Citation anchor 2: `specs/master-plan-sequencing.json:704-866` for deployment contexts, IaC substrate, OS, Rust, and OCI Always Free.
Citation anchor 3: `secrets/PRD.md:20-331` for Oyatie product purpose and planned features.
Citation anchor 4: `secrets/ARCHITECTURE.md:3-704` for architecture and component evidence.
Citation anchor 5: `docs/standards/documentation-rigor.md:1-220` for intern-buildability and hyperscaler-grade documentation requirements.
AWS source 1: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html`.
AWS source 2: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/reference_limits.html`.
AWS source 3: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/rotating-secrets.html`.
AWS source 4: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/replicate-secrets.html`.
AWS source 5: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html`.
Google source 1: `https://docs.cloud.google.com/secret-manager/docs/overview`.
Google source 2: `https://docs.cloud.google.com/secret-manager/quotas`.
Google source 3: `https://docs.cloud.google.com/secret-manager/docs/locations`.
Google source 4: `https://docs.cloud.google.com/secret-manager/docs/secret-manager-secrets-comparison`.
HashiCorp source 1: `https://developer.hashicorp.com/vault/docs`.
HashiCorp source 2: `https://developer.hashicorp.com/vault/docs/secrets`.
HashiCorp source 3: `https://developer.hashicorp.com/hcp/docs/vault-secrets/dynamic-secrets`.
HashiCorp source 4: `https://developer.hashicorp.com/vault/docs/concepts/lease`.
HashiCorp source 5: `https://developer.hashicorp.com/vault/docs/audit`.
HashiCorp source 6: `https://developer.hashicorp.com/vault/docs/enterprise/namespaces`.
HashiCorp source 7: `https://developer.hashicorp.com/vault/docs/enterprise/replication`.
HashiCorp source 8: `https://developer.hashicorp.com/vault/docs/sync`.
HashiCorp source 9: `https://developer.hashicorp.com/vault/docs/deploy/kubernetes/vso/sources/hvs`.

## §1 Counterpart 1 - AWS Secrets Manager capability surface

1. AWS-01: Store database credentials, application credentials, OAuth tokens, API keys, and arbitrary secret material.
2. AWS-02: Retrieve secrets dynamically at runtime instead of hard-coding credentials.
3. AWS-03: Encrypt secrets at rest using AWS KMS keys.
4. AWS-04: Use the free AWS managed key `aws/secretsmanager`.
5. AWS-05: Use customer-managed KMS keys for cross-account access and explicit key policies.
6. AWS-06: Decrypt and transmit secret values over TLS on retrieval.
7. AWS-07: Integrate with IAM identity policies.
8. AWS-08: Integrate with resource-based policies attached to secrets.
9. AWS-09: Support ABAC/tag-based policy controls.
10. AWS-10: Support `BlockPublicPolicy` analysis for broad-access resource policies.
11. AWS-11: Support VPC endpoint conditions for private-network access controls.
12. AWS-12: Support interface VPC endpoints for private access.
13. AWS-13: Support automatic rotation schedules.
14. AWS-14: Support managed rotation for managed secrets.
15. AWS-15: Support managed external secrets partner rotation.
16. AWS-16: Support rotation by Lambda function for custom backends.
17. AWS-17: Support single-user database rotation strategy.
18. AWS-18: Support alternating-users database rotation strategy.
19. AWS-19: Support rotation as often as every four hours during a configured window.
20. AWS-20: Support secret versions with staging labels such as current and previous.
21. AWS-21: Support up to 100 versions per secret by default quota.
22. AWS-22: Support 65,536-byte encrypted secret values by default quota.
23. AWS-23: Support up to 500,000 secrets per Region/account by default quota.
24. AWS-24: Support 10,000 `GetSecretValue` requests per second per Region default quota.
25. AWS-25: Support 100 `BatchGetSecretValue` requests per second per Region default quota.
26. AWS-26: Support 40,000 `DescribeSecret` requests per second per Region default quota.
27. AWS-27: Support 50 mutating/control-plane requests per second per Region for many APIs.
28. AWS-28: Support multi-Region replication.
29. AWS-29: Replicate encrypted secret data and metadata including tags and resource policies.
30. AWS-30: Propagate primary rotation values to replica secrets.
31. AWS-31: Promote replica secrets to standalone secrets.
32. AWS-32: Restrict replication to allowed Regions with IAM conditions.
33. AWS-33: Generate CloudTrail logs for create, replication, and management events.
34. AWS-34: Integrate with CloudWatch monitoring.
35. AWS-35: Integrate with AWS Config compliance checks.
36. AWS-36: Integrate with GuardDuty threat detection.
37. AWS-37: Support cost allocation tags.
38. AWS-38: Provide client-side caching libraries, including Rust support.
39. AWS-39: Provide AWS Parameters and Secrets Lambda Extension.
40. AWS-40: Provide AWS Secrets Manager Agent for standardized consumption.
41. AWS-41: Integrate with EKS for Kubernetes secret consumption.
42. AWS-42: Provide AWS SDK and CLI APIs.
43. AWS-43: Support secret names, descriptions, ARNs, tags, KMS key ARNs, and resource policies.
44. AWS-44: Support cross-account secret access with policy and KMS constraints.
45. AWS-45: Provide explicit quota and retry guidance.
46. AWS-46: Provide deletion and recovery windows for secret lifecycle.
47. AWS-47: Provide random password generation.
48. AWS-48: Support managed service integrations with RDS, Redshift, DocumentDB, and more.
49. AWS-49: Provide documentation for avoiding shell-history secret exposure.
50. AWS-50: Provide pricing per secret and per API usage, including replicated secrets.

## §2 Counterpart 2 - Google Secret Manager capability surface

1. GCP-01: Store and manage API keys, usernames, passwords, certificates, and sensitive data.
2. GCP-02: Model a secret as a global resource with metadata and secret versions.
3. GCP-03: Store actual secret data in secret versions.
4. GCP-04: Support version IDs or timestamps.
5. GCP-05: Support rollback to previous known-good versions.
6. GCP-06: Support emergency recovery through version history.
7. GCP-07: Support auditing through version history.
8. GCP-08: Support version aliases for easier access.
9. GCP-09: Support pinning workloads to specific versions.
10. GCP-10: Support disabling secret versions.
11. GCP-11: Support destroying secret versions.
12. GCP-12: Encrypt secrets in transit.
13. GCP-13: Encrypt secrets at rest.
14. GCP-14: Use AES-256 at-rest encryption.
15. GCP-15: Support customer-managed encryption keys through Cloud KMS CMEK.
16. GCP-16: Support imported encryption keys through Cloud KMS.
17. GCP-17: Support fine-grained IAM roles and permissions.
18. GCP-18: Support IAM conditions.
19. GCP-19: Segregate duties for accessing, managing, auditing, and rotating secrets.
20. GCP-20: Support automatic replication.
21. GCP-21: Support user-managed replication.
22. GCP-22: Charge automatic replication as one location.
23. GCP-23: Allow user-managed replication to chosen geographic locations.
24. GCP-24: Support high availability through replication.
25. GCP-25: Support disaster recovery through replication.
26. GCP-26: Support automatic rotation schedules.
27. GCP-27: Support regional secrets for data residency.
28. GCP-28: Support regional endpoints.
29. GCP-29: Support regional endpoint enforcement for data-at-rest and in-transit locality.
30. GCP-30: Support global service default endpoint.
31. GCP-31: Support regional service for strict data sovereignty.
32. GCP-32: Explicitly document that regional secret access is restricted to its region.
33. GCP-33: Explicitly document that global secrets can be accessed across Google Cloud.
34. GCP-34: Support access requests quota of 90,000 per minute per project.
35. GCP-35: Support read request quota of 600 per minute per project.
36. GCP-36: Support write request quota of 600 per minute per project.
37. GCP-37: Support quota increase requests through support.
38. GCP-38: Support labels and annotations in secret metadata.
39. GCP-39: Integrate with Parameter Manager.
40. GCP-40: Allow Parameter Manager to reference Secret Manager secrets.
41. GCP-41: Distinguish secret management from key management.
42. GCP-42: Provide regional location availability tables.
43. GCP-43: Support Cloud Audit Logs integration through Google Cloud audit surfaces.
44. GCP-44: Support Kubernetes secret synchronization guidance.
45. GCP-45: Support best-practice controls for replication policy.
46. GCP-46: Support organization policies for resource locations.
47. GCP-47: Support secret metadata listing/read operations separately from access operations.
48. GCP-48: Support service-level agreement references for availability.
49. GCP-49: Provide SDK, language, and tooling documentation.
50. GCP-50: Provide explicit distinction between global, regional, and Parameter Manager location models.

## §3 Counterpart 3 - HashiCorp Vault Secrets capability surface

1. HCV-01: Centralize secret management.
2. HCV-02: Rotate old credentials.
3. HCV-03: Generate credentials on demand.
4. HCV-04: Audit client interactions.
5. HCV-05: Support regulatory compliance.
6. HCV-06: Manage static key-value secrets.
7. HCV-07: Store, rotate, and encrypt arbitrary strings.
8. HCV-08: Manage certificates through PKI/KMIP integration.
9. HCV-09: Manage identities and authentication.
10. HCV-10: Support managed entities.
11. HCV-11: Support identity tokens.
12. HCV-12: Support OIDC workflows.
13. HCV-13: Support workload identity federation.
14. HCV-14: Manage third-party dynamic secrets.
15. HCV-15: Generate and revoke on-demand cloud credentials.
16. HCV-16: Control access to external encryption keys and cloud credentials.
17. HCV-17: Encrypt sensitive data in transit through Transit.
18. HCV-18: Tokenize sensitive data through transform/tokenization surfaces.
19. HCV-19: Support HSM/FIPS/PKCS11 architectures.
20. HCV-20: Provide flexible secrets engines.
21. HCV-21: Enable secrets engines at mount paths.
22. HCV-22: Tune secrets engines, including TTL.
23. HCV-23: Move secrets engines and revoke leases tied to paths.
24. HCV-24: Isolate secrets engines through barrier views.
25. HCV-25: Support dynamic credentials with TTL.
26. HCV-26: Support dynamic secret blueprints.
27. HCV-27: Support provider integrations for dynamic credentials.
28. HCV-28: Support principals associated with dynamic secrets.
29. HCV-29: Support lease IDs for dynamic secrets.
30. HCV-30: Support lease renewal.
31. HCV-31: Support lease revocation.
32. HCV-32: Support prefix-based lease revocation.
33. HCV-33: Support automatic revocation on expiration.
34. HCV-34: Support audit devices for request/response logs.
35. HCV-35: Support multiple audit devices.
36. HCV-36: Fail closed when all audit devices are unavailable.
37. HCV-37: HMAC sensitive string audit values.
38. HCV-38: Support eliding large list response bodies in audit.
39. HCV-39: Support namespaces and secure multi-tenancy in Enterprise/Dedicated.
40. HCV-40: Support isolated namespace login paths and tenant environments.
41. HCV-41: Support namespace APIs and namespace headers.
42. HCV-42: Support performance replication.
43. HCV-43: Support disaster recovery replication.
44. HCV-44: Support local vs shared mounts under replication.
45. HCV-45: Support path filters for replication.
46. HCV-46: Support response-wrapped replication activation material.
47. HCV-47: Support secrets sync to external destinations.
48. HCV-48: Support reconciliation scans and retries for sync destinations.
49. HCV-49: Support Vault Secrets Operator sync to Kubernetes Secrets.
50. HCV-50: Support static, auto-rotating, and dynamic HCP Vault Secrets sync to Kubernetes.
51. HCV-51: Support Transit signing, verification, HMAC, hashing, random bytes, and data key generation.
52. HCV-52: Support integrated storage limits and advisory lease counts.
53. HCV-53: Support CLI, API, and plugin extension surfaces.
54. HCV-54: Support cloud-hosted HCP Vault services and self-managed Vault.
55. HCV-55: Support operational telemetry for secrets metrics.

## §4 UNION-coverage matrix

| capability | AWS | Google | HashiCorp | UNION required | Oyatie cloud-secrets has | Gap classification |
|---|---|---|---|---|---|---|
| Store arbitrary secrets | yes | yes | yes | yes | yes, via OpenBao intent | present |
| Runtime secret retrieval | yes | yes | yes | yes | yes, via SecretReference intent | partial |
| Secret versions | yes | yes | yes via KV/versioning | yes | partial, version appears in ADR grammar | partial |
| Version aliases/staging labels | yes | yes | partial | yes | no stable contract | missing |
| Version disable/destroy | partial | yes | yes | yes | not specified | missing |
| Rollback to prior version | partial | yes | yes | yes | not specified | missing |
| Secret size limit | 65,536 bytes | not from audited excerpt | Vault request/storage limits | yes | not specified | missing |
| Secret count quota | 500,000 per Region/account | not from audited excerpt | no simple fixed max | yes | not specified | missing |
| Read/access quota | 10,000 GetSecretValue rps | 90,000 access rpm/project | deployment-dependent | yes | capacity model only | partial |
| Write quota | 50 rps class | 600 rpm/project | deployment-dependent | yes | capacity model only | partial |
| Describe/list quota | yes | yes | yes | yes | not specified | missing |
| Rotation schedule | yes | yes | yes | yes | yes, planned | partial |
| Rotation as frequently as four hours | yes | not audited | configurable TTL/rotation | yes | not specified | missing |
| Managed rotation | yes | no direct equivalent | yes through engines | yes | not specified | missing |
| External managed secret rotation | yes | partial | yes through providers | yes | not specified | missing |
| Custom rotation function | yes Lambda | partial cloud automation | yes plugins/engines | yes | planned scheduler only | partial |
| Single-user rotation | yes | not named | engine-dependent | yes | not specified | missing |
| Alternating-user rotation | yes | not named | engine-dependent | yes | not specified | missing |
| Dynamic credentials | no core, partner/managed limited | no core | yes | yes | not specified as lease API | missing |
| Dynamic credential TTL | no core | no core | yes | yes | not specified | missing |
| Lease renewal | no core | no core | yes | yes | not specified | missing |
| Lease revocation | no core | no core | yes | yes | not specified | missing |
| Prefix-based revocation | no | no | yes | yes | not specified | missing |
| Secret reference contract | ARN/name | resource name/version | path/mount/namespace | yes | yes, but conflicting forms | partial |
| IAM identity policy | yes | yes | Vault policy/auth | yes | Cedar/OpenBao intent | partial |
| Resource-based policy | yes | no direct | Vault policy path | yes | partial policy docs | partial |
| ABAC/tags | yes | labels/IAM conditions | metadata/policy | yes | not complete | partial |
| Block broad access analysis | yes | no direct | policy review | yes | not specified | missing |
| Namespace/tenant isolation | account/ARN/policy | project/resource/IAM | namespaces | yes | yes, planned | partial |
| Namespace public API | no | no | yes | yes | no stable API | missing |
| Encryption at rest | KMS | AES-256/CMEK | barrier/storage/seal | yes | yes, via OpenBao/HSM intent | partial |
| Customer managed key | KMS CMK | CMEK | seal/HSM/transit | yes | HSM/BYOK planned | partial |
| Imported key support | KMS import | Cloud KMS import | HSM/PKCS11 | yes | not specified | missing |
| HSM/FIPS support | KMS/CloudHSM | Cloud KMS/HSM adjacent | yes | yes | planned HSM | partial |
| TLS in transit | yes | yes | yes | yes | implied | partial |
| Private network endpoint | VPC endpoint | regional/private Google controls | self-host/private | yes | not specified | missing |
| Regional replication | yes | yes | yes | yes | multi-region narrative | partial |
| Replica promotion | yes | no direct | DR promotion | yes | not specified | missing |
| Automatic replication | no, explicit add regions | yes | performance/DR replication | yes | not specified | missing |
| User-managed replication | yes | yes | path filters/local/shared | yes | partial residency docs | partial |
| Regional endpoint | service endpoints | yes | deployment endpoint | yes | not specified | missing |
| Strict data residency region | conditional | yes | namespaces/path filters/deployments | yes | policy docs present | partial |
| Cross-region access policy | yes | yes | yes via replication | yes | not specified | missing |
| Audit logging | CloudTrail | audit logs | audit devices | yes | audit chain planned | partial |
| Multiple audit devices | no direct | no direct | yes | yes | not specified | missing |
| Audit fail-closed behavior | no direct | no direct | yes | yes | conflicting docs | partial |
| Audit HMAC/hash | CloudTrail redaction varies | audit logs vary | yes | yes | Merkle/Ed25519 intent | partial |
| Compliance monitoring | Config/CloudTrail | IAM/audit/cloud controls | compliance support | yes | compliance doc present | partial |
| Cost allocation tags | yes | labels/billing | tags/destination | yes | cost-budget present | partial |
| Secret scanning integration | CodeGuru/Amazon Q | best-practice scanning | Vault Radar product | yes | leak-detection CI planned | partial |
| Client-side caching | yes | SDK-dependent | agents/cache patterns | yes | p99 cache target only | partial |
| Agent/sidecar consumption | AWS agent/extension | Kubernetes/SDK guides | Vault Agent/VSO | yes | not specified | missing |
| Kubernetes sync | EKS integration | docs/guidance | VSO/HVS | yes | Helm/OpenBao only | missing |
| Drift remediation for K8s sync | no direct | partial | yes | yes | not specified | missing |
| Parameter/config manager | no direct | Parameter Manager | KV/config | yes | not specified | missing |
| API and CLI | yes | yes | yes | yes | OpenAPI/proto docs only | partial |
| SDK | yes | yes | yes | yes | Rust planned, non-Rust drift | partial |
| Rust SDK | cache library | SDK support not audited | community/clients | yes | planned | partial |
| Python SDK | yes | yes | yes | optional | planned but conflicts | incoherent |
| TypeScript/JS SDK | yes | yes | yes | optional | planned but conflicts | incoherent |
| Go SDK | yes | yes | yes | optional | mentioned but conflicts | incoherent |
| Java SDK | yes | yes | yes | optional | mentioned but conflicts | incoherent |
| OpenAPI contract | AWS API model | Google API | Vault API | yes | yes | partial |
| gRPC/proto contract | no direct | no direct | no direct | optional | yes | additive |
| Event stream | CloudTrail/EventBridge | audit/logs | audit/telemetry | yes | AsyncAPI present | partial |
| Secret deletion recovery | yes | version destroy | lease/revoke/delete | yes | not specified | missing |
| Random password generation | yes | no audited excerpt | yes through engines | yes | secret generation crate planned | partial |
| Credentials for databases | yes | yes generic | dynamic DB engine | yes | rotation planned | partial |
| Credentials for cloud IAM | AWS roles | IAM resources | dynamic cloud creds | yes | not specified | missing |
| OAuth/API-token handling | yes | yes | yes | yes | intended | partial |
| Certificate management | no core | certificate secret storage | PKI engine | yes | not specified | missing |
| PKI issuance | no core | no core | yes | yes | absent | missing |
| Transit encryption API | KMS adjacent | KMS adjacent | yes | yes if Vault union | absent or owned by KMS | missing |
| Tokenization/transform | no core | no core | yes | yes if Vault union | absent | missing |
| HSM unseal/seal runbooks | no direct | no direct | yes | yes | runbooks partial | partial |
| Disaster recovery replication | yes | yes | yes | yes | DR restore plan partial | partial |
| Backup/restore | deletion recovery | versioning | snapshots/storage | yes | restore runbook present | partial |
| Migration from Vault | no | no | no | Oyatie-specific | yes | additive |
| Raw-secret linting | best practice | best practice | best practice | yes | explicit doctrine | additive-present |
| Secret<T> typed wrapper | no | no | no | Oyatie-specific | planned | additive-partial |
| Merkle/Ed25519 audit chain | no visible equivalent | no visible equivalent | audit hash only | Oyatie-specific | planned | additive-partial |
| Six-context deployment | no, AWS only | no, GCP only | self/HCP varies | Oyatie-specific | intended, not artifact-backed | additive-partial |
| OpenTofu deploy substrate | no | Terraform docs exist | Terraform provider exists | Oyatie canonical | missing | missing |
| OCI Always Free demo_trial | no | no | no | Oyatie canonical | missing | missing |
| OS support matrix | SaaS/service | SaaS/service | self-host OS concerns | Oyatie canonical | missing | missing |
| Sigstore module attestation | no visible equivalent | no visible equivalent | plugin signing concepts | Oyatie canonical | missing | missing |
| Context-specific state backend | AWS managed | GCP managed | Vault storage | Oyatie canonical | missing | missing |
| Tenant namespace provisioning | partial account/policy | partial project/IAM | yes | yes | planned | partial |
| Tenant namespace drift detection | no direct | no direct | partial | yes | runbook partial | partial |
| Secret residency proof | partial | yes | partial | yes | policy partial | partial |
| Sealed audit row per request | CloudTrail eventual | audit logs | audit devices | yes | SLO present, semantics conflict | partial |
| Secret leak incident runbook | best practices | best practices | operational docs | yes | present | present |
| BYOK onboarding runbook | KMS docs | CMEK docs | HSM/seal docs | yes | present | partial |
| Key escrow/recovery | KMS/account | KMS/IAM | seal/recovery keys | yes | planned | partial |
| Cost budget by tenant_class | pricing | pricing | pricing | yes | present but not Always Free | partial |
| Capability tenant_class matrix | pricing/tier docs | pricing/tier docs | HCP tiers/Enterprise | yes | present | partial |
| Multi-tenant delegated admin | IAM/resource policy | IAM | namespaces | yes | planned | partial |
| Quota increase process | service quotas | support | deployment-dependent | yes | not specified | missing |
| SLA surface | AWS service SLA | Google SLA | HCP/support/deploy | yes | SLOs present | partial |
| Monitoring dashboard | CloudWatch | Cloud Monitoring | telemetry | yes | dashboards present | partial |
| Threat model | best practices | best practices | security model | yes | present | present |
| DPIA/privacy doc | compliance docs | compliance docs | compliance docs | yes | present | present |
| Compliance export | AWS reports | Google reports | compliance support | yes | planned | partial |
| Cross-service handoffs | AWS integrations | GCP integrations | integrations | yes | present but broken refs | partial |
| Onboarding guide | tutorials | tutorials | tutorials | yes | present | partial |
| Tutorial | yes | yes | yes | yes | present | partial |
| FAQ | yes | yes | yes | yes | present | partial |
| Benchmarks | quotas/pricing | quotas | limits | yes | unverified measured claims | partial |
| Control-plane read APIs | yes | yes | yes | yes | OpenAPI present | partial |
| Secret write APIs | yes | yes | yes | yes | OpenAPI present | partial |
| Resolve API | GetSecretValue | accessSecretVersion | read path | yes | proto/OpenAPI conflict | partial |
| Tenant provision API | no direct | no direct | namespace API | yes | proto planned | partial |
| Audit query API | CloudTrail/Athena | Cloud Logging | audit devices | yes | architecture planned | partial |
| Policy compiler | IAM/Zelkova | IAM Conditions | Vault policy | yes | catalog planned | partial |
| Cedar policy | no | no | no | Oyatie-specific | present | additive-present |
| OpenBao operator | no | no | no | Oyatie-specific | Helm plan present | additive-partial |
| Provider-agnostic secret plane | no | no | partial self-host | yes | intended | partial |
| Cloud-provider direct API isolation | no | no | partial | Oyatie canonical | not enforceable yet | missing |
| Context-aware observability labels | CloudWatch dims | Cloud Monitoring labels | telemetry labels | yes | not explicit | missing |
| Context-aware billing labels | cost tags | labels | tags | yes | cost partial | partial |
| Compliance pack overlays | AWS Artifact | Google compliance | Vault compliance | yes | journey IPs present | partial |
| Generated SDK provenance | SDK docs | SDK docs | SDK docs | Oyatie canonical | absent | missing |

## §5 Capability families summary table

| family | union required count | Oyatie present | Oyatie partial | Oyatie missing/incoherent | headline |
|---|---:|---:|---:|---:|---|
| Core storage and retrieval | 13 | 1 | 8 | 4 | Strong intent, unstable contract. |
| Versioning and lifecycle | 10 | 0 | 3 | 7 | Version aliases, delete/recover, rollback need specification. |
| Rotation and dynamic secrets | 14 | 0 | 5 | 9 | Scheduler exists as plan, leases and dynamic credentials are missing. |
| IAM, policy, and tenancy | 16 | 1 | 9 | 6 | Cedar/OpenBao direction is credible but not complete. |
| Encryption, HSM, and BYOK | 13 | 0 | 8 | 5 | HSM/BYOK planned; imported keys and transit split unresolved. |
| Replication and residency | 13 | 0 | 7 | 6 | Residency docs exist; endpoint and replica semantics missing. |
| Audit and compliance | 16 | 2 | 10 | 4 | Audit ambition is strong; fail-closed semantics conflict. |
| SDK/API/consumption | 18 | 0 | 10 | 8 | Contracts exist; SDK language policy conflicts. |
| Kubernetes/operator integration | 8 | 0 | 2 | 6 | Helm exists; sync/drift/remediation missing. |
| Deployment and portability | 13 | 0 | 2 | 11 | This is the largest canonical gap. |
| Operations and runbooks | 13 | 3 | 7 | 3 | Good runbook breadth; broken references remain. |
| Cost, tiering, and quotas | 11 | 0 | 5 | 6 | Tiers exist, but quotas and OCI Always Free are missing. |

## §6 Headline gap analysis - top 15 missing capabilities

1. Gap 01: One canonical SecretReference grammar is missing.
2. Evidence: `secrets/PRD.md:20-28`, `secrets/contracts/openapi/cloud-secrets.yaml:80-90`, `secrets/contracts/proto/cloud-secrets.proto:46-48`, and `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:56-70`.
3. Hook: make ADR-MS-001 the grammar authority or revise it, then regenerate OpenAPI/proto/examples/lint rules.
4. Gap 02: Dynamic leases are missing.
5. Evidence: HashiCorp lease docs and absence of lease API in OpenAPI/proto.
6. Hook: add lease issue/renew/revoke/prefix-revoke APIs or explicitly delegate to another service.
7. Gap 03: Secret version aliases/staging labels are missing.
8. Evidence: AWS and Google support staging/aliases; Oyatie docs only imply version in ADR grammar.
9. Hook: add version alias model to SQL migration, OpenAPI, proto, and SLOs.
10. Gap 04: Quotas and hard limits are missing.
11. Evidence: AWS/GCP publish quotas; Oyatie has capacity model but no contract limits.
12. Hook: add limits table for secret size, versions, tenants, requests, rotation cadence, and audit backlog per tier.
13. Gap 05: Multi-Region replica semantics are missing.
14. Evidence: AWS replicate/promote and Google replication/regional docs; Oyatie has `multi-region.md` but not public replica states.
15. Hook: define primary, replica, promotion, lag, consistency, and residency constraints.
16. Gap 06: Regional endpoint behavior is missing.
17. Evidence: Google regional endpoint docs; Oyatie residency docs do not define endpoints.
18. Hook: add context/region endpoint matrix and enforcement behavior.
19. Gap 07: Private network endpoint semantics are missing.
20. Evidence: AWS VPC endpoint guidance; Oyatie docs do not map VPC/private link equivalents.
21. Hook: add per-context private endpoint or mesh ingress requirements.
22. Gap 08: Audit fail-closed semantics conflict.
23. Evidence: `failure-modes.md:83` versus `audit-log-completeness.openslo.yaml:18-43`.
24. Hook: define strict, degraded, and emergency modes with explicit secret-resolution behavior.
25. Gap 09: Kubernetes secret sync/drift remediation is missing.
26. Evidence: HashiCorp VSO supports sync, rotation, and drift remediation; Oyatie only has Helm/OpenBao deploy assets.
27. Hook: add optional sync controller or explicitly reject sync in favor of runtime references.
28. Gap 10: OpenTofu context deployment is missing.
29. Evidence: service `iac/` only has Helm/Kustomize directories.
30. Hook: add signed per-context OpenTofu modules wrapping Helm/Kustomize where useful.
31. Gap 11: OCI Always Free demo_trial is missing.
32. Evidence: `retired tenant_class adoption artifact:11-27`; no `iac/oci-guest/always-free/`.
33. Hook: define demo_trial limits for A1 4 OCPU/24GB, 200GB block, LB/network, and no paid dependencies.
34. Gap 12: OS support matrix is missing.
35. Evidence: no `supported-oses.json`.
36. Hook: add Tier-1/Tier-2/out-of-scope manifest with package/CI lanes.
37. Gap 13: SDK language strategy is incoherent.
38. Evidence: `IP-008-sdk-ts-python-bindings.md:14-58` and Rust-strict policy.
39. Hook: delete non-Rust SDK work or create generated SDK exception with provenance.
40. Gap 14: Secret scanning integration is not concrete.
41. Evidence: leak-detection CI is planned, but tests and CI lane are absent.
42. Hook: add Rust-based detector crate tests, CI gate, and evidence output.
43. Gap 15: Transit/PKI ownership is unresolved.
44. Evidence: HashiCorp Vault union includes transit/PKI; Oyatie docs focus on secrets/HSM references.
45. Hook: decide whether cloud-kms owns transit/PKI, and write an explicit non-goal if not in cloud-secrets.

## §7 Additive surface

1. Additive 01: Provider-agnostic six-context secret plane, broader than AWS-only or GCP-only service scope.
2. Additive 02: OpenBao-first substrate ownership rather than proprietary provider binding.
3. Additive 03: SecretReference doctrine integrated across all Oyatie microservices.
4. Additive 04: Raw-secret prohibition across repo, chat, checkpoints, CI logs, and non-secret stores.
5. Additive 05: `Secret<T>` typed wrapper aspiration for compile-time secret discipline.
6. Additive 06: Merkle/Ed25519 audit-chain target for tamper-evident evidence.
7. Additive 07: Cedar policy surface for local authorization logic.
8. Additive 08: Compliance journey IPs for country/regulator overlays.
9. Additive 09: HashiCorp Vault migration playbook as a first-party migration path.
10. Additive 10: BYOK/HSM onboarding runbook tuned to Oyatie tenant lifecycle.
11. Additive 11: Data residency policy tied to packs/tenants rather than one public cloud.
12. Additive 12: Cross-microservice handoff document that treats secrets as a substrate dependency.
13. Additive 13: SLO coverage for audit completeness and vault seal recovery, not just read latency.
14. Additive 14: Leak incident runbook coupled to source and CI doctrine.
15. Additive 15: Capability tenant_class matrix with secret-plane-specific axes.
16. Additive 16: Rust-strict implementation doctrine for a security-sensitive service.
17. Additive 17: OCI Always Free demo_trial requirement as a cost-sovereignty differentiator, once implemented.
18. Additive 18: OpenTofu-only IaC and sigstore attestation doctrine, once implemented.
19. Additive 19: Documentation-rigor/intern-buildability standard, once gaps are fixed.
20. Additive verdict: the additive surface is valuable, but parity cannot be claimed until foundational missing counterpart capabilities are specified and canonical constraints are implemented.
