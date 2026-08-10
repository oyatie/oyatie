# cloud-secrets coherence audit - 2026-05-20

Audit owner: sole-agent Wave 2 Batch 2.1 microservice ownership-coherence audit.
Scope: `/Users/jasonlee/oyatie/microservices/cloud-secrets/` only.
Target microservice: `cloud-secrets`.
Deployable-context assumption under test: all six canonical contexts unless evidence proves a correctly documented N/A.
Counterpart bar: AWS Secrets Manager, Google Secret Manager, HashiCorp Vault Secrets.
Evidence standard: every finding below is tied to a local file line, canonical line, memory directive, chat-history line, or official counterpart source.

Citation anchor 1: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2225` for §D-15 multi-context deployment.
Citation anchor 1b: `docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2495` for §D-16 OpenTofu-only IaC and forbidden patterns.
Citation anchor 1c: `docs/decisions/ADR-0700-ci-admission-live-apex.md §D-17..§D-20` for OS matrix, Rust-strict policy, OCI Always Free, and audit-agent decision tree.
Citation anchor 2: `specs/master-plan-sequencing.json:704-866` for deployment contexts, OpenTofu substrate, supported OSes, language policy, and OCI Always Free.
Citation anchor 3: `secrets/PRD.md:20-331` read for purpose, requirements, SDK, SLO, benchmark, and acceptance criteria evidence.
Citation anchor 4: `secrets/ARCHITECTURE.md:3-704` read for architecture, dependency, tenant-scope, OpenBao, and credential-isolation evidence.
Citation anchor 5: `docs/standards/documentation-rigor.md:1-220` read for intern-buildability, substrate remediation priority, completeness invariants, and hyperscaler-grade documentation standard.
Citation anchor 6: `docs/standards/brief-template.md:666-740` read for multi-context, cloud-family, KMS/secrets, IaC, and citation-anchor expectations.
Constraint memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md`.
Constraint memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md`.
Constraint memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md`.
Constraint memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md`.
Constraint memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md`.
Ownership memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md`.
Verification memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md`.
Substance memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md`.
Chat-history source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`, searched for `cloud-secrets` and cross-cutting directives.

## §1 microservice purpose summary

1. `cloud-secrets` is intended to be Oyatie's canonical secret and credential plane, not a generic application vault wrapper.
2. The PRD says its purpose is OpenBao-backed secret reference resolution, rotation, HSM integration, audit emission, and per-tenant namespace control; citation: `secrets/PRD.md:20-28`.
3. The product contract forbids raw secrets in source, chat, checkpoint files, CI logs, and non-secret stores; citation: `secrets/PRD.md:20-28`.
4. The visible runtime primitive is a `SecretReference`, not a raw credential value; citation: `secrets/PRD.md:42-51`.
5. The microservice is substrate-class: every other service is supposed to consume it for credentials instead of embedding provider-specific secret storage; citation: `secrets/PRD.md:25-28`.
6. The architecture positions OpenBao, tenant namespaces, policy bundles, audit bundles, HSM envelopes, and rotation workflows as first-class components; citation: `secrets/ARCHITECTURE.md:9-70`.
7. The architecture also describes per-tenant scoping and audit-query surfaces; citation: `secrets/ARCHITECTURE.md:133-147`.
8. The immediate purpose is stronger than AWS Secrets Manager and Google Secret Manager in one respect: Oyatie wants a platform-local secret reference contract that other microservices can use without depending on provider-specific APIs.
9. The immediate purpose is weaker than HashiCorp Vault in another respect: current docs have no complete operator-grade implementation surface for leases, mount lifecycle, namespace operations, audit-device operation, or disaster-recovery bootstrap.
10. The service's strongest documented differentiators are zero-raw-secret doctrine, OpenBao-first implementation, Merkle/Ed25519 audit chain, tenant-specific residency, and cross-service secret reference linting.
11. The service's weakest documented area is deployment substrate coherence: it still has Helm/Kustomize evidence but lacks canonical per-context OpenTofu modules.
12. The second weak area is contract singularity: the PRD, OpenAPI, proto, and ADR-MS use incompatible SecretReference string formats.
13. The third weak area is implementation-readiness: many acceptance criteria cite tests and evidence paths that do not exist under the microservice path.
14. The fourth weak area is language-policy alignment: service docs prescribe TypeScript and Python SDK packages despite the Rust-strict backend/runtime/tooling doctrine.
15. The fifth weak area is OS support: no `supported-oses.json` was present in the inventory, and no Tier-1 package/CI matrix is defined.
16. The sixth weak area is OCI demo_trial tenant_class: the tenant_class matrix prices demo_trial at about USD 45/month and does not map demo_trial on OCI to Always Free.
17. The service is therefore purpose-coherent at the narrative level, but not yet ownership-coherent at the deployable artifact level.
18. Current docs can tell a senior platform engineer what the service should be.
19. Current docs cannot yet let a cold intern build, test, package, deploy, operate, and audit it across six contexts without external invention.
20. Overall classification: partial coherence, P1 remediation required before this can be called a canonical hyperscaler-grade secret plane.

## §2 Inventory snapshot

Total files seen under `microservices/cloud-secrets/`: 134.
Total service lines audited by inventory pass: 20,339.
Inventory source: recursive file listing plus `wc -l` over service files.

| file | size | role | coherent_with_purpose? |
|---|---:|---|---|
| `ARCHITECTURE.md` | 754 lines | Architecture narrative and component model | partial |
| `AUDIT-FINDINGS-2026-05-18.json` | 101 lines | Prior audit evidence | partial |
| `IP-001-openbao-operator.md` | 143 lines | Implementation plan | partial |
| `IP-002-secretreference-contract.md` | 66 lines | Implementation plan | partial |
| `IP-003-policy-engine.md` | 173 lines | Implementation plan | partial |
| `IP-004-tenant-namespace-controller.md` | 75 lines | Implementation plan | partial |
| `IP-005-hsm-key-envelopes.md` | 83 lines | Implementation plan | partial |
| `IP-006-rotation-scheduler.md` | 57 lines | Implementation plan | partial |
| `IP-007-audit-chain.md` | 77 lines | Implementation plan | partial |
| `IP-008-sdk-ts-python-bindings.md` | 75 lines | SDK implementation plan | no |
| `IP-009-leak-detection-ci.md` | 84 lines | CI implementation plan | partial |
| `IP-010-incident-response.md` | 68 lines | Incident implementation plan | partial |
| `IP-011-dr-restore.md` | 68 lines | Disaster recovery implementation plan | partial |
| `IP-012-key-escrow.md` | 57 lines | Key escrow implementation plan | partial |
| `IP-013-compliance-export.md` | 56 lines | Compliance export implementation plan | partial |
| `IP-014-slo-alerting.md` | 55 lines | SLO implementation plan | partial |
| `IP-015-cost-metering.md` | 100 lines | Cost metering implementation plan | partial |
| `IP-journey-j25-key-envelope.md` | 420 lines | Journey-specific implementation plan | partial |
| `IP-journey-j80-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j81-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j83-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j86-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j87-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j88-provider-and-encryption-byok.md` | 430 lines | Journey-specific BYOK plan | partial |
| `IP-journey-j91-us-msb-mtl-overlay.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j94-sox404-public-company-controls.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j98-au-privacy-apra-cps234.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | 400 lines | Regulatory journey plan | partial |
| `IP-journey-j100-pack-rollout-first-action.md` | 400 lines | Regulatory journey plan | partial |
| `PHASE-01-OPENBAO-SECRETREFERENCE-SUBSTRATE.md` | 133 lines | Phase plan | partial |
| `PRD.md` | 363 lines | Product requirements | partial |
| `backfill-replay.md` | 144 lines | Backfill/replay plan | partial |
| `benchmarks/cloud-secrets-vs-vault-vs-aws-sm-vs-azure-kv-vs-gcp-sm-vs-akeyless.md` | 112 lines | Benchmark narrative | partial |
| `retired tenant_class adoption artifact` | 98 lines | tenant_class adoption matrix | partial |
| `capacity-model.md` | 125 lines | Capacity model | partial |
| `catalog/audit-chain-writer.yaml` | 9 lines | Crate catalog | yes |
| `catalog/audit-exporter.yaml` | 9 lines | Crate catalog | yes |
| `catalog/audit-schema.yaml` | 9 lines | Crate catalog | yes |
| `catalog/backup-restore.yaml` | 9 lines | Crate catalog | yes |
| `catalog/cli.yaml` | 9 lines | Crate catalog | yes |
| `catalog/config.yaml` | 9 lines | Crate catalog | yes |
| `catalog/detectors.yaml` | 9 lines | Crate catalog | yes |
| `catalog/envelope.yaml` | 9 lines | Crate catalog | yes |
| `catalog/error.yaml` | 9 lines | Crate catalog | yes |
| `catalog/events.yaml` | 9 lines | Crate catalog | yes |
| `catalog/export.yaml` | 9 lines | Crate catalog | yes |
| `catalog/hsm.yaml` | 9 lines | Crate catalog | yes |
| `catalog/k8s-operator.yaml` | 9 lines | Crate catalog | yes |
| `catalog/key-rotation.yaml` | 9 lines | Crate catalog | yes |
| `catalog/lease.yaml` | 9 lines | Crate catalog | yes |
| `catalog/metrics.yaml` | 9 lines | Crate catalog | yes |
| `catalog/migration.yaml` | 9 lines | Crate catalog | yes |
| `catalog/namespace-controller.yaml` | 9 lines | Crate catalog | yes |
| `catalog/nonce.yaml` | 9 lines | Crate catalog | yes |
| `catalog/openbao-client.yaml` | 9 lines | Crate catalog | yes |
| `catalog/openbao-operator.yaml` | 9 lines | Crate catalog | yes |
| `catalog/policy-compiler.yaml` | 9 lines | Crate catalog | yes |
| `catalog/provisioning.yaml` | 9 lines | Crate catalog | yes |
| `catalog/recovery-key.yaml` | 9 lines | Crate catalog | yes |
| `catalog/reference-parser.yaml` | 9 lines | Crate catalog | yes |
| `catalog/residency.yaml` | 9 lines | Crate catalog | yes |
| `catalog/rotation-plan.yaml` | 9 lines | Crate catalog | yes |
| `catalog/sdk.yaml` | 9 lines | Crate catalog | yes |
| `catalog/secret-detector.yaml` | 9 lines | Crate catalog | yes |
| `catalog/secret-generation.yaml` | 9 lines | Crate catalog | yes |
| `catalog/seal.yaml` | 9 lines | Crate catalog | yes |
| `catalog/service.yaml` | 9 lines | Crate catalog | yes |
| `catalog/signing.yaml` | 9 lines | Crate catalog | yes |
| `catalog/sla.yaml` | 9 lines | Crate catalog | yes |
| `catalog/store.yaml` | 9 lines | Crate catalog | yes |
| `catalog/tenant-api.yaml` | 9 lines | Crate catalog | yes |
| `catalog/transport.yaml` | 9 lines | Crate catalog | yes |
| `catalog/types.yaml` | 9 lines | Crate catalog | yes |
| `competitor-parity-matrix.md` | 127 lines | Competitive surface | partial |
| `compliance.md` | 1163 lines | Compliance mapping | partial |
| `contracts/asyncapi/cloud-secrets-events.yaml` | 283 lines | Event contract | partial |
| `contracts/openapi/cloud-secrets.yaml` | 447 lines | REST contract | partial |
| `contracts/proto/cloud-secrets.proto` | 290 lines | gRPC/SDK contract | partial |
| `cost-budget.md` | 111 lines | Cost model | partial |
| `cross-microservice-handoffs.md` | 260 lines | Cross-service contract | partial |
| `dashboards/audit-lag-dashboard.json` | 110 lines | Observability dashboard | partial |
| `dashboards/openbao-health-dashboard.json` | 125 lines | Observability dashboard | partial |
| `dashboards/rotation-health-dashboard.json` | 138 lines | Observability dashboard | partial |
| `decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md` | 292 lines | Microservice ADR | partial |
| `dpia.md` | 230 lines | Privacy impact assessment | partial |
| `failure-modes.md` | 268 lines | Failure-mode catalog | partial |
| `faqs/security-engineer-faq.md` | 174 lines | Security FAQ | partial |
| `iac/helm/hsm-operator/values.yaml` | 31 lines | Helm configuration | partial |
| `iac/helm/openbao/Chart.yaml` | 11 lines | Helm chart metadata | partial |
| `iac/helm/openbao/templates/configmap.yaml` | 78 lines | Helm template | partial |
| `iac/helm/openbao/templates/service.yaml` | 45 lines | Helm template | partial |
| `iac/helm/openbao/values.yaml` | 81 lines | Helm values | partial |
| `iac/helm/postgres/values.yaml` | 41 lines | Helm values | partial |
| `iac/kustomize/base/kustomization.yaml` | 26 lines | Kustomize base | partial |
| `iac/kustomize/base/namespace.yaml` | 13 lines | Kubernetes namespace | partial |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | 20 lines | Kustomize overlay | partial |
| `incident-response.md` | 207 lines | Incident response | partial |
| `manifest.json` | 402 lines | Service manifest | partial |
| `migration-playbooks/from-hashicorp-vault.md` | 189 lines | Migration playbook | partial |
| `migrations/001_secret_reference_registry.sql` | 95 lines | SQL migration | yes |
| `multi-region.md` | 212 lines | Multi-region plan | partial |
| `onboarding/service-integration-guide.md` | 120 lines | Integrator onboarding | partial |
| `policy/data-residency.md` | 230 lines | Residency policy | partial |
| `policy/openbao-tenant-policy.hcl` | 57 lines | OpenBao policy | partial |
| `policy/secret-isolation.md` | 204 lines | Isolation policy | partial |
| `policy/tenant-scope.cedar` | 53 lines | Cedar policy | partial |
| `reference-implementations/rust-client.md` | 153 lines | Reference implementation | partial |
| `runbooks/audit-log-lag.md` | 157 lines | Runbook | partial |
| `runbooks/byok-onboarding-failed.md` | 203 lines | Runbook | partial |
| `runbooks/key-unseal-failed.md` | 235 lines | Runbook | partial |
| `runbooks/namespace-drift.md` | 170 lines | Runbook | partial |
| `runbooks/secret-leak-detected.md` | 248 lines | Runbook | partial |
| `runbooks/tenant-secret-restore.md` | 170 lines | Runbook | partial |
| `sdk-plan.md` | 219 lines | SDK plan | partial |
| `slos/audit-log-completeness.openslo.yaml` | 49 lines | OpenSLO | partial |
| `slos/key-rotation-correctness.openslo.yaml` | 49 lines | OpenSLO | partial |
| `slos/openbao-availability.openslo.yaml` | 50 lines | OpenSLO | partial |
| `slos/secret-resolve-latency.openslo.yaml` | 47 lines | OpenSLO | partial |
| `slos/tenant-namespace-provisioning.openslo.yaml` | 48 lines | OpenSLO | partial |
| `slos/vault-seal-recovery.openslo.yaml` | 49 lines | OpenSLO | partial |
| `threat-model.md` | 506 lines | Threat model | partial |
| `tutorials/integrate-service-with-cloud-secrets.md` | 152 lines | Tutorial | partial |

## §3 9-dimension audit

### §3.1 Dimension 1 - internal coherence

1. Internal coherence headline: partial, with multiple P1/P2 contradictions in core contracts.
2. `PRD.md` defines the visible reference form as `${openbao:secret/<path>}`; citation: `secrets/PRD.md:20-28`.
3. `manifest.json` repeats `${openbao:secret/<path>}` under the secrets substrate provider; citation: `secrets/manifest.json:318-321`.
4. `contracts/openapi/cloud-secrets.yaml` accepts `openbao:secret/.+` without the `${...}` wrapper; citation: `secrets/contracts/openapi/cloud-secrets.yaml:80-90`.
5. `contracts/proto/cloud-secrets.proto` describes `openbao:secret/<tenant>/<microservice>/<name>` in the resolver RPC; citation: `secrets/contracts/proto/cloud-secrets.proto:46-48`.
6. `decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md` decides `secretref:v1:{tenant_id}:{home_cell}:{microservice}:{purpose}:{secret_name}:{version}`; citation: `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:56-70`.
7. Contradiction probe 1: SecretReference grammar has at least four active shapes, so clients cannot implement one parser safely.
8. Severity for probe 1: P1, because it affects the service's public contract and every consuming microservice.
9. The PRD sets cache hit p99 <=10ms and cache miss p99 <=25ms; citation: `secrets/PRD.md:57-60`.
10. The `secret-resolve-latency` OpenSLO sets p99 <=100ms; citation: `secrets/observability/slos/cloud-secrets/secret-resolve-latency.openslo.yaml:18-42`.
11. ADR-MS-001 repeats p99 <=100ms for resolve; citation: `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:81-85`.
12. Contradiction probe 2: latency success can be either <=25ms or <=100ms depending on the file read.
13. Severity for probe 2: P1, because SLOs drive readiness gates and benchmark claims.
14. The PRD names expected SLO files `secret-resolution`, `rotation-completeness`, and `audit-emission-completeness`; citation: `secrets/PRD.md:84-87`.
15. Actual SLO files use `secret-resolve-latency`, `key-rotation-correctness`, and `audit-log-completeness`.
16. `IP-014-observability-slo-branch-protection-hg-cloud-secrets.md` repeats the PRD SLO filename expectation; citation: `secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md:28-30`.
17. Contradiction probe 3: SLO file names are not internally stable between PRD/IP and actual SLO directory.
18. Severity for probe 3: P2, because it breaks automation by path but not the product concept.
19. The PRD acceptance criteria cite `tests/bench/resolution-latency.rs`; citation: `secrets/PRD.md:310-313`.
20. No `tests/` directory was present in the 134-file inventory.
21. `ADR-MS-001` cites load, chaos, and property tests; citation: `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:230-252`.
22. Contradiction probe 4: validation criteria require tests that are not present under the service path.
23. Severity for probe 4: P2, because it prevents intern buildability and audit verification.
24. `ARCHITECTURE.md` line 3 explicitly says the file was created by an anchor sweep and needs content-pass expansion; citation: `secrets/ARCHITECTURE.md:3`.
25. The same architecture later claims runbook and IaC evidence exists; citation: `secrets/ARCHITECTURE.md:29-47`.
26. Contradiction probe 5: the architecture simultaneously marks itself as needing expansion and asserts evidence completeness.
27. Severity for probe 5: P2, because it weakens trust in completeness claims.
28. `PRD.md` says total crates introduced is 34; citation: `secrets/PRD.md:166`.
29. The catalog inventory contains 38 crate catalog YAML files, and `manifest.json` lists a broad catalog block; citation: `secrets/manifest.json:1-49`.
30. Contradiction probe 6: crate count differs between PRD and manifest/catalog.
31. Severity for probe 6: P3, because it is count drift but could mislead implementation planning.
32. `failure-modes.md` says secret-resolution can continue while audit lag breaches compliance; citation: `secrets/failure-modes.md:83`.
33. `audit-log-completeness.openslo.yaml` requires sealed audit within the same request lifetime and treats missing audit as Sev-2; citation: `secrets/observability/slos/cloud-secrets/audit-log-completeness.openslo.yaml:18-43`.
34. Contradiction probe 7: availability-vs-audit behavior is not resolved for strict audit mode.
35. Severity for probe 7: P1, because secrets without complete audit are a security substrate failure.
36. `cross-microservice-handoffs.md` references `contracts/proto/cloud_secrets.proto`; citation: `secrets/cross-microservice-handoffs.md:15-16`.
37. The actual file is `contracts/proto/cloud-secrets.proto`.
38. Contradiction probe 8: internal path reference is broken by underscore/dash drift.
39. Severity for probe 8: P2, because codegen and handoff readers can fail.
40. `cross-microservice-handoffs.md` references `developer-scope.cedar`; citation: `secrets/cross-microservice-handoffs.md:30` and `secrets/cross-microservice-handoffs.md:162`.
41. No `developer-scope.cedar` file exists in the service inventory.
42. Contradiction probe 9: policy handoff cites a missing Cedar artifact.
43. Severity for probe 9: P2.
44. `incident-response.md` references missing legal/regulator contacts; citation: `secrets/incident-response.md:76`, `secrets/incident-response.md:164`, and `secrets/incident-response.md:203`.
45. `compliance.md` references missing `legal/*` evidence; citation: `secrets/compliance.md:64-147`.
46. Contradiction probe 10: incident and compliance escalation paths cite absent legal artifacts.
47. Severity for probe 10: P2.
48. Internal cross-reference classification: SecretReference grammar is wrong-direction because ADR, PRD, OpenAPI, and proto cannot all be canonical.
49. Internal cross-reference classification: SLO files partly resolve but semantics contradict PRD targets.
50. Internal cross-reference classification: tests, legal, and Cedar references are broken.
51. Dimension 1 result: P1 due to public contract contradictions and audit/latency semantic contradictions.

### §3.2 Dimension 2 - outbound cross-references

1. Outbound coherence headline: partial, with broken service-local references and broad reverse references from docs/registry.
2. Outbound ADR reference to ADR-MS-001 resolves; citation: `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:1-20`.
3. Outbound reference to OpenBao resolves only as concept; service has Helm values but no complete OpenTofu module; citation: `secrets/iac/helm/openbao/Chart.yaml:1-11`.
4. Outbound reference to `cloud-auth` appears in bounded context and handoffs; citation: `secrets/PRD.md:95-104`.
5. Outbound reference to `audit-evidence` appears as audit event and evidence sink; citation: `secrets/cross-microservice-handoffs.md:58-88`.
6. Outbound reference to `messenger` appears in service docs and reverse docs; citation: `docs/standards/messenger-e2e-encryption-mls.md:125`.
7. Outbound reference to `cloud-iac` appears in compliance and FAQ as IaC disposal/control plane; citation: `secrets/compliance.md:982`.
8. Outbound reference to Terraform/Pulumi state appears in FAQ; citation: `secrets/faqs/security-engineer-faq.md:77`.
9. That FAQ reference is drifted after ADR-0328 because OpenTofu is canonical and Terraform/Pulumi are forbidden except as superseded/negative examples.
10. Outbound reference to `contracts/proto/cloud_secrets.proto` is broken; citation: `secrets/cross-microservice-handoffs.md:15-16`.
11. Outbound reference to `developer-scope.cedar` is broken; citation: `secrets/cross-microservice-handoffs.md:30`.
12. Outbound reference to missing legal contacts is broken; citation: `secrets/incident-response.md:76`.
13. Outbound reference to `tests/e2e/*` is broken under the service path; citation: `secrets/PRD.md:310-313`.
14. Outbound reference to `tests/bench/resolution-latency.rs` is broken under the service path; citation: `secrets/PRD.md:310-313`.
15. Outbound reference to `helm install` in acceptance criteria is not canonical deployment evidence; citation: `secrets/PRD.md:317`.
16. Reverse reference search across `docs`, `specs`, and `registry` produced a large cloud-secrets surface, including registry tenant_class mapping.
17. Reverse reference: `registry/tenant-class-adoption/microservice-tenant_class-mapping.yaml:625` maps `cloud-secrets`.
18. Reverse reference: `registry/tenant-class-adoption/vendor-tenant_class-mapping.yaml:847` and related nearby entries map AWS/GCP/HashiCorp vendor tenant_class equivalents.
19. Reverse reference: `registry/brownout/coverage-tracker.tsv:13` names `cloud-secrets` in coverage tracking.
20. Reverse reference: `registry/throttling/coverage-tracker.tsv:31` names `cloud-secrets` as default or N/A-internal.
21. Reverse reference: `registry/api-surface-classification/coverage-tracker.tsv:31` points to `secrets/contracts/openapi/cloud-secrets.yaml`.
22. Actual OpenAPI path is `secrets/contracts/openapi/cloud-secrets.yaml`.
23. Reverse reference classification: registry API path is broken and should be corrected during aggregation.
24. Reverse reference: `specs/microservices/manifests-index.json:127-128` maps the service manifest.
25. Reverse reference: `specs/platform-architecture.json:396` includes `cloud-secrets` in S0 leaves.
26. Reverse reference: `specs/platform-architecture.json:482` ties OpenBao seed material to platform architecture.
27. Reverse reference: `specs/platform-architecture.json:1110` references `cloud-secrets` as a component.
28. Reverse reference: `specs/platform-architecture.json:1455` ties tenant scoped encryption keys to cloud-secrets BYOK.
29. Reverse reference: `docs/standards/documentation-rigor.md:98` explicitly names `cloud-secrets` as a substrate microservice requiring remediation before product work.
30. Reverse reference: `docs/architecture/corpus-rigor-audit*` contains multiple historic `cloud-secrets` gap entries, some now stale because new files exist.
31. Reverse reference: `docs/runbooks/byok*` references service binaries that are not present in this microservice inventory.
32. Reverse reference: `docs/onboarding/doctrine-bootcamp-2026-05-21.md:550` says mail cannot ship until pipeline work is wired into cloud-secrets.
33. Chat-history search found prior cloud-secrets work assignments and anchor-sweep evidence near chat line 78 in the `cloud-secrets` match pass.
34. Chat-history directive search found progress and microservice surface set evidence near chat line 10553.
35. Chat-history evidence is useful as historical context, but local files remain the source of truth for this audit.
36. Orphan reference class 1: reverse references to `contracts/openapi.yaml` are wrong because actual OpenAPI file is nested and named `cloud-secrets.yaml`.
37. Orphan reference class 2: docs/runbooks references to service binaries are not backed by `bin/` or `src/` under this service.
38. Orphan reference class 3: legal/regulator contacts are cited but not present in service-local compliance scope.
39. Missing reverse reference class 1: no canonical per-context OpenTofu module is reverse-linked from this service, because no context module exists.
40. Missing reverse reference class 2: no `supported-oses.json` reverse anchor exists for OS support.
41. Missing reverse reference class 3: no generated-SDK exception ADR is reverse-linked for TypeScript/Python/Go/Java SDK claims.
42. Outbound cross-service handoffs are conceptually broad and useful, especially for audit, auth, KMS, and tenant registry.
43. Outbound handoffs are not yet buildable because they mix event names, missing policy files, and unstable proto paths.
44. ADR and standards references generally resolve.
45. Test references generally do not resolve.
46. Legal references generally do not resolve.
47. IaC references resolve only to Helm/Kustomize, not to canonical OpenTofu context modules.
48. Reverse references confirm `cloud-secrets` is a substrate dependency for multiple platform paths.
49. Reverse references therefore raise the severity of broken contracts because many other docs assume this service exists.
50. Dimension 2 result: P1 due to high fan-out and broken public API/IaC references.

### §3.3 Dimension 3 - substance bar and intern-buildability

1. Intern-buildability headline: not sufficient yet.
2. The documentation-rigor standard requires intern-buildability, meaning a cold implementer can build from docs without hidden context; citation: `docs/standards/documentation-rigor.md:133-141`.
3. The standard also names substrate microservices including `cloud-secrets` as early remediation targets; citation: `docs/standards/documentation-rigor.md:98-100`.
4. The PRD is substantial on product intent; citation: `secrets/PRD.md:20-130`.
5. The architecture is broad on components but begins with a content-expansion warning; citation: `secrets/ARCHITECTURE.md:3`.
6. A cold intern can identify the goal: no raw secrets, OpenBao-backed references, rotation, HSM, audit, namespaces.
7. A cold intern cannot identify a single canonical SecretReference grammar because four forms are present.
8. A cold intern cannot run the expected tests because the cited `tests/` tree is absent.
9. A cold intern cannot build Rust crates because no `Cargo.toml` or `src/` files were found under the service inventory.
10. A cold intern cannot build the TypeScript/Python SDK because that plan violates current language doctrine and the package tree is absent.
11. A cold intern cannot deploy with canonical OpenTofu because only Helm/Kustomize IaC directories exist.
12. A cold intern cannot target the six deployment contexts because no `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/guest-on-oci/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-as-cloud-provider/` directories exist.
13. A cold intern cannot package for Tier-1 OSes because `supported-oses.json` is absent.
14. A cold intern cannot decide demo_trial-on-OCI capacity because `retired tenant_class adoption artifact` prices demo_trial but does not map Always Free.
15. A cold intern cannot reconcile latency targets because PRD/SLO/ADR disagree.
16. A cold intern cannot rely on benchmark numbers because the benchmark doc claims measurements but local evidence artifacts were not present in the service inventory.
17. Weak section: `PRD.md:310-321` acceptance criteria name missing tests and Helm-based smoke install.
18. Weak section: `ARCHITECTURE.md:3` declares the architecture needs expansion.
19. Weak section: `manifest.json:132-222` lists implementation plans, including forbidden SDK plans, but no buildable crate workspace.
20. Weak section: `IP-008-sdk-ts-python-bindings.md:14-58` specifies TS/Python packages and non-Rust build commands.
21. Weak section: `retired tenant_class adoption artifact:11-27` defines demo_trial as paid and not OCI Always Free.
22. Weak section: `capacity-model.md:98-105` uses sandbox/trial/production tenant_classes instead of demo_trial/paid tenant_class.
23. Weak section: `cross-microservice-handoffs.md:15-16` cites a wrong proto path.
24. Weak section: `incident-response.md:76` cites missing legal contacts.
25. Weak section: `compliance.md:64-147` cites missing legal artifacts.
26. Buildability gap: no canonical development command beyond ADR-0328's `cargo build --workspace --release --all-features --locked`.
27. Buildability gap: no per-crate ownership map matching the 38 catalog entries to actual Rust package paths.
28. Buildability gap: no migration-run command for `migrations/001_secret_reference_registry.sql`.
29. Buildability gap: no OpenBao bootstrap command tied to OpenTofu state outputs.
30. Buildability gap: no sealed audit sink implementation path tied to `audit-evidence`.
31. Buildability gap: no test fixture for raw-secret linting acceptance.
32. Buildability gap: no CI lane spec per deployment context.
33. Buildability gap: no CI lane spec per Tier-1 OS.
34. Buildability gap: no package format matrix for RPM, DEB, `.pkg`, Homebrew, Talos extension, Flatcar extension, and container image.
35. Buildability gap: no data-residency enforcement proof attached to each context.
36. Buildability gap: no state backend mapping per context in service-local IaC.
37. Buildability gap: no module attestation or sigstore/cosign wiring in service-local IaC.
38. Buildability gap: no secret migration read-after-write safety proof across OpenBao namespace moves.
39. Buildability gap: no gRPC raw bytes exposure threat rationale despite proto returning secret bytes.
40. Buildability gap: no SDK provenance exception for non-Rust bindings.
41. Buildability gap: no capability-tenant_class conformance tests.
42. Buildability gap: no OCI Always Free load profile.
43. Buildability gap: no operator runbook for the runbooks that ADR-MS-001 says are missing; citation: `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:265-267`.
44. Positive substance: compliance, DPIA, threat model, runbooks, dashboards, and handoff docs are non-trivial.
45. Positive substance: event contract and REST/gRPC contracts are detailed enough to start API reconciliation.
46. Positive substance: capability-tenant_class matrix gives a first service-specific tenant_class frame.
47. Positive substance: failure modes identify real operational risks.
48. Positive substance: migration playbook from HashiCorp Vault is useful domain context.
49. Missing code means this audit is a documentation ownership audit, not an implementation verification.
50. Dimension 3 result: P1 for intern-buildability until grammar, build workspace, tests, OpenTofu, OS manifest, and tenant_class mappings are reconciled.

### §3.4 Dimension 4 - canonical-direction alignment

1. Canonical-direction headline: drifted-fixable, but not currently aligned.
2. Multi-context constraint classification: drifted-fixable.
3. ADR-0328 requires every microservice to name supported deployment contexts and justify any N/A; citation: `docs/decisions/ADR-0700-ci-admission-live-apex.md:2079-2087`.
4. Brief-template cloud-family guidance says cloud services generally need all six contexts, and KMS/secrets are mandatory for `oyatie-as-cloud-provider`; citation: `docs/standards/brief-template.md:666-740`.
5. `cloud-secrets` has no manifest-level `deployment_contexts` field in `manifest.json`.
6. `cloud-secrets` has no service-local N/A rationale with missing primitives, customer impact, remediation owner, and revisit gate.
7. Multi-context classification result: P1 drift.
8. OpenTofu constraint classification: drifted-fixable.
9. ADR-0328 requires OpenTofu as canonical and forbids Terraform/Pulumi/CloudFormation as implementation substrate; citation: `docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2495`.
10. The service has Helm/Kustomize IaC only, with no canonical context module directories.
11. `compliance.md:982` mentions Helm/Kustomize/OpenTofu, but no service-local OpenTofu module path exists.
12. `PRD.md:317` uses `helm install` acceptance evidence rather than OpenTofu `plan/apply` evidence.
13. OpenTofu classification result: P1 drift.
14. OS support constraint classification: drifted-fixable.
15. `specs/master-plan-sequencing.json:777-816` requires Tier-1/Tier-2/out-of-scope OS declarations and a manifest.
16. No `supported-oses.json` exists in the service inventory.
17. No package-format matrix exists in service-local docs.
18. No per-OS CI lane matrix exists in service-local docs.
19. OS classification result: P1 drift.
20. Rust-strict constraint classification: incoherent in docs, clean in actual file extensions.
21. The source scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.cs` source files under the service path.
22. Actual file extension inventory therefore has no forbidden non-Rust code implementation.
23. `IP-008-sdk-ts-python-bindings.md:14-58` prescribes TypeScript and Python binding packages and `npm`, `maturin`, and `pytest` commands.
24. `PRD.md:124-130` prescribes Rust SDK plus TS bindings through napi-rs and Python through pyo3.
25. `competitor-parity-matrix.md:70` and `competitor-parity-matrix.md:95` discuss Go and Java SDK parity.
26. No generated-SDK exception ADR/provenance file was found.
27. Rust-strict classification result: P1 documentation incoherence, P0 avoided because no forbidden source files were present.
28. OCI Always Free constraint classification: drifted-fixable.
29. ADR-0328 requires `iac/oci-guest/always-free/` and demo_trial-on-OCI as Always Free; citation: `docs/decisions/ADR-0700-ci-admission-live-apex.md §D-19`.
30. `specs/master-plan-sequencing.json:856-866` names the same OCI Always Free module path and forbids paid fallback in demo_trial.
31. `retired tenant_class adoption artifact:11-27` prices demo_trial around USD 45/month and does not name OCI Always Free.
32. `cost-budget.md:22-30` uses paid OCI cost surfaces rather than an Always Free demo_trial profile.
33. No `iac/oci-guest/always-free/` directory exists.
34. OCI classification result: P1 drift.
35. Documentation-rigor classification: partial.
36. The service has breadth across compliance, DPIA, runbooks, SLOs, threat model, and handoffs.
37. The service lacks enough canonical machine-readable deployment/OS/build/test artifacts to pass intern-buildability.
38. Brief-template §3.9 requires multi-context table and IaC anchors; the current docs have narrative but no context matrix.
39. Brief-template §3.10-§3.12 anchors are not fully reflected in service-local artifacts.
40. ADR-0328 §D-20 audit-agent dimensions are directly applicable because this audit is exactly a Wave 2 ownership-coherence pass.
41. The service's product direction is compatible with canonical doctrine: OpenBao, Rust, no raw secrets, provider-agnostic, multi-context.
42. The service's artifacts are not yet compatible with canonical doctrine: missing OpenTofu, missing OS manifest, non-Rust SDK planning, and missing OCI Always Free.
43. No issue is inherently unsalvageable.
44. Most drift can be remediated by consolidating contracts, deleting forbidden SDK plans, adding context OpenTofu modules, adding OS manifest, and adding OCI demo_trial tenant_class tenant_class mapping.
45. Remediation must happen before broad product fanout because docs-rigor names cloud-secrets as substrate-class.
46. If aggregation allows one near-term fix, fix SecretReference grammar first.
47. If aggregation allows two near-term fixes, add deployment context and IaC matrix second.
48. If aggregation allows three near-term fixes, add Rust/SDK exception decision or delete non-Rust SDK scope third.
49. Dimension 4 result: P1 across all five canonical constraints, with no P0 code-file violation observed.
50. Canonical alignment status: not ready for Wave 14 aggregation as green; ready as a clear remediation packet.

### §3.5 Dimension 5 - industry-counterpart parity

1. Union-coverage headline: partial, with strong audit/residency aspirations but missing mature secret-manager primitives.
2. AWS Secrets Manager official docs cover storage, retrieval, rotation, KMS encryption, IAM/resource policies, replication, CloudTrail/CloudWatch monitoring, VPC endpoints, caching, and quotas.
3. AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html`.
4. AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/reference_limits.html`.
5. AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/rotating-secrets.html`.
6. AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/replicate-secrets.html`.
7. AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html`.
8. Google Secret Manager official docs cover secret versions, IAM, CMEK, automatic/user-managed replication, regional secrets, regional endpoints, rotation, Parameter Manager, and quotas.
9. Google source: `https://docs.cloud.google.com/secret-manager/docs/overview`.
10. Google source: `https://docs.cloud.google.com/secret-manager/quotas`.
11. Google source: `https://docs.cloud.google.com/secret-manager/docs/locations`.
12. Google source: `https://docs.cloud.google.com/secret-manager/docs/secret-manager-secrets-comparison`.
13. HashiCorp Vault docs cover static secrets, dynamic secrets, leases, revocation, audit devices, transit crypto, namespaces, replication, sync, Kubernetes operator integration, HSM/FIPS, PKI, and identity.
14. HashiCorp source: `https://developer.hashicorp.com/vault/docs`.
15. HashiCorp source: `https://developer.hashicorp.com/vault/docs/secrets`.
16. HashiCorp source: `https://developer.hashicorp.com/hcp/docs/vault-secrets/dynamic-secrets`.
17. HashiCorp source: `https://developer.hashicorp.com/vault/docs/concepts/lease`.
18. HashiCorp source: `https://developer.hashicorp.com/vault/docs/audit`.
19. HashiCorp source: `https://developer.hashicorp.com/vault/docs/enterprise/namespaces`.
20. HashiCorp source: `https://developer.hashicorp.com/vault/docs/enterprise/replication`.
21. HashiCorp source: `https://developer.hashicorp.com/vault/docs/sync`.
22. HashiCorp source: `https://developer.hashicorp.com/vault/docs/deploy/kubernetes/vso/sources/hvs`.
23. Oyatie present: zero raw secret doctrine; citation: `secrets/PRD.md:20-28`.
24. Oyatie present: OpenBao-backed reference resolution; citation: `secrets/PRD.md:42-51`.
25. Oyatie present: audit event model; citation: `secrets/PRD.md:78-80`.
26. Oyatie present: tenant namespace controller; citation: `secrets/PRD.md:42-51`.
27. Oyatie present: HSM envelope concept; citation: `secrets/PRD.md:42-51`.
28. Oyatie present: data residency policy; citation: `secrets/policy/data-residency.md:1-80`.
29. Oyatie present: OpenSLO files for resolve, rotation, audit, availability, namespace provisioning, and seal recovery.
30. Oyatie missing: stable secret version aliases comparable to AWS labels and Google aliases.
31. Oyatie missing: public quota model comparable to AWS per-Region TPS and Google per-project RPM.
32. Oyatie missing: dynamic secret lease lifecycle comparable to HashiCorp leases.
33. Oyatie missing: lease renew/revoke API surface.
34. Oyatie missing: namespaces as an explicit public API comparable to Vault Enterprise namespace API.
35. Oyatie missing: automatic replication policy and replica promotion comparable to AWS.
36. Oyatie missing: regional endpoint behavior comparable to Google regional Secret Manager.
37. Oyatie missing: VPC/private endpoint or equivalent private connectivity policy comparable to AWS.
38. Oyatie missing: resource-based policy length, secret size, and version-count ceilings.
39. Oyatie missing: Parameter Manager equivalent, unless config/service-parameters are intentionally out of scope.
40. Oyatie missing: Kubernetes secret sync and drift remediation comparable to HashiCorp VSO/HVS.
41. Oyatie missing: multiple audit device failure semantics comparable to Vault's audit-device unavailability behavior.
42. Oyatie missing: transit cryptography as an explicit API comparable to Vault transit, although cloud-kms may own it.
43. Oyatie missing: PKI/certificate lifecycle if `cloud-secrets` claims Vault union coverage; this may belong to cloud-kms or identity services.
44. Oyatie missing: managed external secret partner rotation equivalent to AWS managed external secrets.
45. Oyatie missing: secret scanning integration comparable to AWS CodeGuru/Amazon Q in best practices, although leak detection CI is planned.
46. Oyatie additive: Merkle/Ed25519 audit-chain emphasis is stronger than the visible AWS/GCP product surface.
47. Oyatie additive: platform-wide Secret<T> and SecretReference lint doctrine is stronger than provider product docs.
48. Oyatie additive: provider-agnostic deployment across six contexts is broader than single-cloud AWS/GCP products.
49. Union parity result: partial.
50. Dimension 5 result: P1 because a cloud-secret plane cannot claim counterpart parity until leases, quotas, replication, endpoint, policy, and sync semantics are specified.

### §3.6 Dimension 6 - multi-context deployment support

1. Multi-context headline: unsupported as an artifact, intended as a product direction.
2. Canonical contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`; citation: `specs/master-plan-sequencing.json:704-746`.
3. `oyatie-public-cloud` status: product docs imply platform control, but no `iac/oyatie-public-cloud/` directory exists.
4. `oyatie-public-cloud` classification: missing IaC, not correctly N/A.
5. `guest-on-aws` status: product docs mention AWS KMS and AWS CloudHSM in tiers, but no `iac/guest-on-aws/` directory exists.
6. `guest-on-aws` classification: missing IaC, not correctly N/A.
7. `guest-on-oci` status: product docs mention OCI cost surfaces, but no `iac/guest-on-oci/` directory exists.
8. `guest-on-oci` classification: missing IaC, not correctly N/A.
9. `on-prem` status: product docs mention HSM and OpenBao but no `iac/on-prem/` directory exists.
10. `on-prem` classification: missing IaC, not correctly N/A.
11. `colo` status: product docs do not provide a colo-specific deployment overlay.
12. `colo` classification: missing IaC, not correctly N/A.
13. `oyatie-as-cloud-provider` status: brief-template says KMS/secrets are mandatory for this context; no `iac/oyatie-as-cloud-provider/` exists.
14. `oyatie-as-cloud-provider` classification: missing IaC, not correctly N/A.
15. Present IaC directories are `iac/helm`, `iac/helm/hsm-operator`, `iac/helm/openbao`, `iac/helm/postgres`, `iac/kustomize/base`, and `iac/kustomize/overlays/pack-kr`.
16. Helm and Kustomize may remain deployment components, but they do not satisfy ADR-0328's context OpenTofu requirement by themselves.
17. The manifest lacks a `deployment_contexts` object.
18. The manifest lacks data-residency state backend per context.
19. The manifest lacks observability labels per context.
20. The manifest lacks IAM/security perimeter per context.
21. The service lacks onboarding path per context.
22. The service lacks cost-metering path per context.
23. The service lacks rollout/drift-detection path per context.
24. The service lacks per-context CI lanes.
25. `PRD.md:317` acceptance via `helm install` is a single-cluster smoke path, not a six-context substrate gate.
26. `multi-region.md` helps with regions, but region support is not the same as six deployment-context support.
27. `policy/data-residency.md` helps with residency, but residency policy is not the same as deployability.
28. `retired tenant_class adoption artifact` includes provider examples, but provider examples do not prove deployable contexts.
29. Forbidden provider API pattern check: no direct source code was present, so no business-logic provider SDK calls were observed.
30. Documentation drift pattern: AWS KMS/Azure Dedicated HSM names in tenant_class matrix can leak provider-specific tenant_class semantics if not abstracted.
31. Documentation drift pattern: FAQ names Terraform/Pulumi state instead of OpenTofu state.
32. Documentation drift pattern: OCI cost model does not distinguish guest-on-oci demo_trial Always Free from paid tiers.
33. Correct N/A count: zero contexts.
34. Missing IaC count: six contexts.
35. Supported with evidence count: zero contexts.
36. Intended but unproven count: six contexts.
37. `oyatie-as-cloud-provider` has the highest urgency because brief-template treats KMS/secrets as mandatory in that context.
38. `guest-on-oci` has the highest cost-policy urgency because demo_trial must map to Always Free.
39. `guest-on-aws` has provider-specific leakage risk due to AWS examples.
40. `on-prem` and `colo` have HSM/operator requirements that current Helm charts do not fully state.
41. `oyatie-public-cloud` needs platform-control OpenTofu state and module attestation.
42. The service does not explicitly document whether OpenBao runs as a shared substrate or per-tenant/per-pack instance in each context.
43. The service does not document context-specific secret replication boundaries.
44. The service does not document context-specific root-of-trust custody.
45. The service does not document context-specific emergency unseal authority.
46. The service does not document context-specific audit sink durability.
47. The service does not document context-specific latency/capacity targets.
48. The service does not document context-specific onboarding commands.
49. Dimension 6 result: P1.
50. Remediation hint: add manifest deployment context matrix, per-context OpenTofu modules, and explicit N/A structures only where canonical evidence proves true N/A.

### §3.7 Dimension 7 - OpenTofu IaC coverage

1. OpenTofu headline: no service-local canonical OpenTofu coverage found.
2. ADR-0328 says OpenTofu is the canonical IaC engine and Terraform/Pulumi/CloudFormation are forbidden implementation substrates; citation: `docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2495`.
3. Required per-context module directories are absent.
4. Missing directory: `iac/oyatie-public-cloud/`.
5. Missing directory: `iac/guest-on-aws/`.
6. Missing directory: `iac/guest-on-oci/`.
7. Missing directory: `iac/on-prem/`.
8. Missing directory: `iac/colo/`.
9. Missing directory: `iac/oyatie-as-cloud-provider/`.
10. Missing directory: `iac/oci-guest/always-free/`.
11. Present IaC directory: `iac/helm/`.
12. Present IaC directory: `iac/helm/hsm-operator/`.
13. Present IaC directory: `iac/helm/openbao/`.
14. Present IaC directory: `iac/helm/openbao/templates/`.
15. Present IaC directory: `iac/helm/postgres/`.
16. Present IaC directory: `iac/kustomize/`.
17. Present IaC directory: `iac/kustomize/base/`.
18. Present IaC directory: `iac/kustomize/overlays/pack-kr/`.
19. Required `main.tf` files were not found in any context path.
20. Required `variables.tf` files were not found in any context path.
21. Required `outputs.tf` files were not found in any context path.
22. Required `versions.tf` files were not found in any context path.
23. Required context `README.md` files were not found in any context path.
24. Required variables such as `tenant_id`, `deployment_context`, `tenant_class`, `state_backend_ref`, and `module_attestation_ref` are not represented in service-local OpenTofu.
25. Required outputs such as `service_endpoint`, `secret_ref_prefix`, `audit_sink_ref`, and `policy_bundle_ref` are not represented in service-local OpenTofu.
26. Sigstore/cosign module signing wiring was not found in service-local IaC.
27. Module attestation references were not found in service-local IaC.
28. State backend mapping per context was not found in service-local IaC.
29. Forbidden `null_resource` pattern was not found in service-local text.
30. Forbidden `local-exec` pattern was not found in service-local text.
31. Forbidden `remote-exec` pattern was not found in service-local text.
32. Forbidden SSH provisioner pattern was not found.
33. The word `ssh` appears in migration domain context for Vault SSH secret engine mapping; citation: `secrets/migration-playbooks/from-hashicorp-vault.md:20`.
34. That `ssh` reference is not an IaC provisioner finding.
35. Terraform/Pulumi reference appears in FAQ; citation: `secrets/faqs/security-engineer-faq.md:77`.
36. That reference is a P2 wording drift because post-ADR-0328 docs should say OpenTofu state, and Terraform/Pulumi only as forbidden or superseded examples.
37. CloudFormation references were not observed in the service scan.
38. Hand-edited tfstate references were not observed.
39. Unsigned local module references were not observed because no module signing surface exists at all.
40. `compliance.md:982` names OpenTofu among IaC inventory, but the inventory path does not substantiate it.
41. `PRD.md:317` uses Helm installation as an acceptance criterion, which is insufficient for canonical IaC readiness.
42. Helm charts may still be implementation artifacts behind OpenTofu modules.
43. Kustomize overlays may still be rendered objects behind OpenTofu modules.
44. The missing layer is the audited, signed, stateful OpenTofu orchestration contract.
45. No state backend exists for oyatie-public-cloud.
46. No state backend exists for guest-on-aws.
47. No state backend exists for guest-on-oci.
48. No state backend exists for on-prem.
49. No state backend exists for colo.
50. No state backend exists for oyatie-as-cloud-provider.
51. Dimension 7 result: P1.

### §3.8 Dimension 8 - OS support matrix

1. OS support headline: no canonical service-local OS support manifest found.
2. Canonical Tier-1 OS set is defined in `specs/master-plan-sequencing.json:777-816`.
3. Required manifest path is service-local `supported-oses.json`.
4. `supported-oses.json` was absent in the inventory.
5. Tier-1 Talos Linux status: not declared.
6. Tier-1 RHEL 9+ status: not declared.
7. Tier-1 Oracle Linux 9+ status: not declared.
8. Tier-1 SLES 15 SP6+ status: not declared.
9. Tier-1 Ubuntu 24.04+ status: not declared.
10. Tier-1 Debian 13+ status: not declared.
11. Tier-1 Rocky Linux 9+ status: not declared.
12. Tier-1 AlmaLinux 9+ status: not declared.
13. Tier-1 CentOS Stream 10+ status: not declared.
14. Tier-1 Amazon Linux 2023+ status: not declared.
15. Tier-1 Flatcar Linux status: not declared.
16. Tier-1 VMware Photon OS 5+ status: not declared.
17. Tier-1 macOS Apple Silicon M5+ status: not declared.
18. Tier-2 ppc64le status: not declared as test-only.
19. Tier-2 s390x status: not declared as test-only.
20. Out-of-scope Intel macOS status: not explicitly declared out of scope.
21. Out-of-scope pre-M5 Apple Silicon status: not explicitly declared out of scope.
22. Out-of-scope FreeBSD status: not explicitly declared out of scope.
23. Out-of-scope OpenBSD status: not explicitly declared out of scope.
24. Out-of-scope Windows Server status: not explicitly declared out of scope.
25. Out-of-scope Solaris status: not explicitly declared out of scope.
26. RPM package format: not declared.
27. DEB package format: not declared.
28. macOS `.pkg` package format: not declared.
29. Homebrew formula/cask package format: not declared.
30. Talos extension format: not declared.
31. Flatcar extension format: not declared.
32. Container image format: implied by Kubernetes/OpenBao docs, but not declared as Tier-1 OS support.
33. CI lane for Talos: not declared.
34. CI lane for RHEL: not declared.
35. CI lane for Oracle Linux: not declared.
36. CI lane for SLES: not declared.
37. CI lane for Ubuntu: not declared.
38. CI lane for Debian: not declared.
39. CI lane for Rocky: not declared.
40. CI lane for AlmaLinux: not declared.
41. CI lane for CentOS Stream: not declared.
42. CI lane for Amazon Linux: not declared.
43. CI lane for Flatcar: not declared.
44. CI lane for Photon: not declared.
45. CI lane for macOS Apple Silicon M5+: not declared.
46. Service docs do include Kubernetes deployment artifacts.
47. Kubernetes artifacts do not replace OS support because canonical policy requires explicit OS matrix.
48. No out-of-scope OS false claims were found.
49. Absence of false claims does not satisfy the manifest requirement.
50. Dimension 8 result: P1.

### §3.9 Dimension 9 - Rust-strict language coverage

1. Rust-strict headline: actual source-file scan is clean; documentation/build plans are not clean.
2. Source scan included forbidden extensions `.py`, `.js`, `.ts`, `.tsx`, `.rb`, `.pl`, `.php`, `.java`, `.scala`, `.groovy`, `.go`, `.fs`, `.fsx`, `.cs`, plus package/build manifests.
3. Source scan found zero forbidden implementation files under `microservices/cloud-secrets/`.
4. This means no P0 forbidden code-file finding is warranted from current service inventory.
5. Whitelisted non-Rust files present include Markdown, YAML, JSON, Proto, OpenSLO YAML, SQL, HCL, and Cedar.
6. `.md` files are authorized documentation.
7. `.yaml` files are authorized structured configuration.
8. `.json` files are authorized structured data.
9. `.proto` files are authorized interface definition.
10. `.openslo.yaml` files are authorized SLO definitions.
11. `.sql` file is authorized database migration.
12. `.cedar` file is authorized policy language.
13. `.hcl` policy file is not in the explicit extension allowlist, but is OpenBao policy configuration rather than app logic.
14. `.hcl` should be covered by a policy/config exception note during canonical cleanup.
15. `IP-008-sdk-ts-python-bindings.md:14-18` explicitly names TypeScript and Python bindings.
16. `IP-008-sdk-ts-python-bindings.md:22-37` names package trees and tests for TypeScript/Python.
17. `IP-008-sdk-ts-python-bindings.md:41-47` contains TypeScript client example text.
18. `IP-008-sdk-ts-python-bindings.md:52-58` names `npm`, `maturin`, and `pytest`.
19. Those planned commands conflict with Rust-strict backend/runtime/tooling unless there is a generated-SDK exception.
20. `PRD.md:124-130` names TS bindings via napi-rs and Python bindings via pyo3.
21. No SDK exception ADR was present under `decisions/`.
22. No generated SDK provenance file was present.
23. No `frontend/<platform>/` Swift/Kotlin/WinUI3 subtree exists.
24. Therefore non-Rust frontend allowlist does not apply.
25. `contracts/proto/cloud-secrets.proto:22` includes a Go package option.
26. A proto `go_package` option is not generated Go code, but it is inconsistent with Rust-strict SDK direction unless codegen targets are constrained.
27. `competitor-parity-matrix.md:70` names Go SDK later.
28. `competitor-parity-matrix.md:95` names Java SDK later.
29. Those SDK parity claims should be deleted, scoped as external compatibility research, or covered by generated SDK exception.
30. No Python interpreter dependency was found in actual service files.
31. No JavaScript application dependency was found in actual service files.
32. No Ruby, Go, Java, Scala, Groovy, PHP, F#, or C# application dependency was found in actual service files.
33. Canonical build invocation is `cargo build --workspace --release --all-features --locked`; citation: `specs/master-plan-sequencing.json:817-855`.
34. Service docs do not show a service-local Rust workspace that can run that command.
35. Service docs also do not show a code-generation step that proves SDK outputs are generated from Rust-owned contracts.
36. If SDK bindings remain desired, the right remediation is generated client artifacts with provenance, not handwritten non-Rust implementation.
37. If SDK bindings are not required for Wave 2, delete IP-008 and remove non-Rust SDK acceptance criteria.
38. Rust-strict policy does not ban OpenAPI, AsyncAPI, Proto, SQL, Cedar, YAML, JSON, or Markdown.
39. Rust-strict policy does ban Python/JS/TS app/runtime/tooling in this microservice's backend surface.
40. The audit classifies actual code state separately from planned doc state to avoid overstating a P0.
41. Actual code state: aligned for forbidden file extensions.
42. Planned doc state: P1 drift.
43. Build state: incomplete because no Rust workspace was in the service inventory.
44. Frontend state: not applicable because no frontend subtree exists.
45. Generated SDK state: not established.
46. Exception state: not established.
47. Whitelist state: mostly clean, with HCL needing a config-language note.
48. Dimension 9 result: P1 due to prescribed non-Rust SDK/tooling and missing build workspace.
49. No P0 language file violation is recorded.
50. Remediation hint: resolve SDK strategy before generating any source files.

## §4 Findings summary

| severity | dimension | short description | citation | remediation hint |
|---|---|---|---|---|
| P1 | D1 | SecretReference grammar has four incompatible active forms | `secrets/PRD.md:20-28`; `secrets/contracts/openapi/cloud-secrets.yaml:80-90`; `secrets/contracts/proto/cloud-secrets.proto:46-48`; `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:56-70` | Choose one grammar and update PRD, OpenAPI, proto, ADR, examples, lint rules. |
| P1 | D1 | Resolve latency target conflicts between 10/25ms and 100ms | `secrets/PRD.md:57-60`; `secrets/observability/slos/cloud-secrets/secret-resolve-latency.openslo.yaml:18-42`; `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:81-85` | Split cache-hit, cache-miss, and backend SLOs or choose one tiered target model. |
| P1 | D1 | Audit completeness allows conflicting fail-open/fail-closed behavior | `secrets/failure-modes.md:83`; `secrets/observability/slos/cloud-secrets/audit-log-completeness.openslo.yaml:18-43` | Define strict/degraded mode and gate secret resolution accordingly. |
| P1 | D2 | Reverse references assume cloud-secrets as substrate while local paths are unstable | `docs/standards/documentation-rigor.md:98`; `registry/api-surface-classification/coverage-tracker.tsv:31` | Fix registry path and public contracts before downstream services depend on them. |
| P1 | D3 | Cold intern cannot build or validate from current docs | `secrets/PRD.md:310-321`; `secrets/ARCHITECTURE.md:3` | Add build workspace, tests, canonical commands, and contract reconciliation. |
| P1 | D4 | Five canonical constraints are not fully satisfied | `specs/master-plan-sequencing.json:704-866` | Add multi-context, OpenTofu, OS, Rust, and OCI Always Free artifacts. |
| P1 | D5 | Industry union parity is partial | official AWS/GCP/HashiCorp sources listed in §3.5 | Add leases, quotas, version aliases, replication, endpoint, sync, and policy semantics. |
| P1 | D6 | All six deployment contexts are missing canonical IaC | `specs/master-plan-sequencing.json:704-746`; `secrets/iac/helm/openbao/Chart.yaml:1-11` | Add per-context OpenTofu modules or complete N/A records. |
| P1 | D7 | No OpenTofu modules, state backends, or attestation wiring | `docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2495`; `secrets/PRD.md:317` | Wrap Helm/Kustomize behind signed OpenTofu modules. |
| P1 | D8 | No `supported-oses.json` or OS package/CI matrix | `specs/master-plan-sequencing.json:777-816` | Add manifest with Tier-1/Tier-2/out-of-scope, packages, and CI lanes. |
| P1 | D9 | Docs prescribe TypeScript/Python SDK/tooling without exception | `secrets/IP-008-sdk-ts-python-bindings.md:14-58`; `secrets/PRD.md:124-130` | Delete or convert to generated SDK exception with provenance. |
| P1 | D4 | OCI demo_trial tenant_class is not mapped to Always Free | `secrets/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md:11-27`; `secrets/cost-budget.md:22-30` | Add `iac/oci-guest/always-free/` and tenant_class limits. |
| P2 | D1 | SLO filenames in PRD/IP do not match actual SLO files | `secrets/PRD.md:84-87`; `secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md:28-30` | Rename or update references. |
| P2 | D1 | Test references are broken because service has no `tests/` tree | `secrets/PRD.md:310-313`; `secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:230-252` | Add tests or revise acceptance criteria. |
| P2 | D1 | Architecture file declares itself unfinished | `secrets/ARCHITECTURE.md:3` | Complete architecture or remove stale warning after review. |
| P2 | D1 | Proto path is wrong in handoff doc | `secrets/cross-microservice-handoffs.md:15-16` | Change underscore path to actual dashed path. |
| P2 | D1 | Missing `developer-scope.cedar` reference | `secrets/cross-microservice-handoffs.md:30`; `secrets/cross-microservice-handoffs.md:162` | Add policy file or update handoff. |
| P2 | D1 | Incident/compliance legal references are missing | `secrets/incident-response.md:76`; `secrets/compliance.md:64-147` | Add service-local legal contact appendix or canonical external pointer. |
| P2 | D7 | FAQ references Terraform/Pulumi state after OpenTofu-only doctrine | `secrets/faqs/security-engineer-faq.md:77` | Rewrite to OpenTofu state and forbidden-tool warning. |
| P2 | D5 | Benchmark doc claims measured numbers without service-local evidence | `secrets/benchmarks/cloud-secrets-vs-vault-vs-aws-sm-vs-azure-kv-vs-gcp-sm-vs-akeyless.md:1-15`; `secrets/benchmarks/cloud-secrets-vs-vault-vs-aws-sm-vs-azure-kv-vs-gcp-sm-vs-akeyless.md:95-105` | Mark prior numbers as imported/evidence-pending or attach evidence. |
| P2 | D6 | Provider-specific HSM/KMS examples leak into generic tenant_class model | `secrets/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md:29-81` | Replace with abstract adapters plus per-context overlays. |
| P3 | D1 | PRD crate count differs from catalog count | `secrets/PRD.md:166`; `secrets/manifest.json:1-49` | Reconcile catalog count and manifest. |
| P3 | D2 | Historic reverse-audit docs contain stale entries | `docs/architecture/corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md:5462-5469` | Let aggregation mark stale vs active. |
| P3 | D8 | No false out-of-scope OS claims were found, but explicit exclusions are absent | service inventory; `specs/master-plan-sequencing.json:777-816` | Add explicit out-of-scope block. |
| P3 | D9 | HCL policy extension needs a config-language note | `microservices/cloud-secrets/policy/openbao-tenant-policy.hcl:1-57` | Add allowlist note for OpenBao policy config. |

Severity totals:
P0: 0.
P1: 12.
P2: 9.
P3: 4.

## §5 Open questions for Wave 14 aggregation

1. Should `secretref:v1` from ADR-MS-001 supersede the OpenBao-path reference forms everywhere, or should OpenBao path remain the serialized public handle?
2. Should TypeScript/Python SDKs be fully deleted under Rust-strict doctrine, or should a generated-client exception be created for external consumers?
3. Should `cloud-secrets` own dynamic leases, or should lease/revocation semantics be split with `cloud-auth` or another substrate service?
4. Should Vault transit/PKI parity live in `cloud-secrets`, `cloud-kms`, or a separate certificate/key service?
5. Should benchmark files that claim measurements be reclassified as target/forecast until ADR-0212 build-phase measured evidence lands?
6. Should Helm/Kustomize charts remain service-local implementation assets behind OpenTofu modules, or move to a platform deployment package?
7. Should legal/regulator contact docs be service-local or canonical shared compliance paths with stable machine-readable pointers?
8. Should all six deployment contexts be mandatory immediately for `cloud-secrets`, or can any early context be delayed with a formal N/A record?
9. Should demo_trial on non-OCI contexts share the same capacity shape as OCI Always Free, or should demo_trial be context-specific?
10. Should `cloud-secrets` expose raw secret bytes over gRPC to trusted SDKs, or should it only return sealed references and require in-process resolver plumbing?

<!-- ORCHESTRATOR REPORT
  µservice: cloud-secrets
  deliverables_landed: secrets/coherence-audit-2026-05-20.md (731 lines); secrets/feature-parity-matrix-2026-05-20.md (409 lines); secrets/performance-benchmark-numbers-2026-05-20.md (437 lines); secrets/tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md (353 lines)
  inventory_files_seen: 134
  inventory_lines_read: 20339
  chat_history_matches_processed: 194
  findings_p0: 0
  findings_p1: 12
  findings_p2: 9
  findings_p3: 4
  top_3_counterparts_confirmed: AWS Secrets Manager / Google Secret Manager / HashiCorp Vault Secrets
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1930
-->
