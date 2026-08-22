# cloud-kms Ownership-Coherence Audit - 2026-05-20

doc_class: microservice-ownership-coherence-audit
microservice: cloud-kms
status: landed
owner: solo-audit-agent
date: 2026-05-20
write_scope: /Users/jasonlee/oyatie/microservices/cloud-kms/
read_scope: canonical direction, cloud-kms artifacts, reverse references, chat history, official counterpart docs

## Citation Anchor Block

1. Canonical deployment/IaC/OS/language/OCI audit rules: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2235`, `docs/decisions/ADR-0700-ci-admission-live-apex.md:2241-2494`, `docs/decisions/ADR-0700-ci-admission-live-apex.md:3756-4153`.
2. Machine-readable canonical sequence: `specs/master-plan-sequencing.json:704-868`, including deployment contexts, OpenTofu substrate, OS matrix, Rust language policy, and OCI Always Free profile.
3. Microservice-owned product purpose: `microservices/cloud-kms/retired tenant_class adoption artifact:7-11`, `microservices/cloud-kms/faqs/kms-engineer-faq.md:7-12`, with the missing local `PRD.md` recorded as a finding.
4. Architecture equivalent read: `crates/cloud-kms-domain/src/lib.rs:1-7`, `contracts/openapi/cloud/cloud-kms-v1.yaml:1-12`, `docs/products/cloud/PRD.md:121-121`, because `microservices/cloud-kms/ARCHITECTURE.md` is absent.
5. Documentation rigor anchors: `docs/standards/documentation-rigor.md:133-190`, `docs/standards/brief-template.md:101-117`, `docs/standards/brief-template.md:666-1304`, `docs/standards/brief-template.md:1720-1854`.

## Executive Verdict

This audit did not find a source tree for a deployable cloud-kms service under `microservices/cloud-kms/`.
The local path contains ten documentation artifacts and no local PRD, architecture file, OpenAPI contract, OpenSLO file, OpenTofu context directory, supported OS manifest, Rust source, or local tests.
The repo does contain canonical cloud-kms runtime artifacts outside the microservice path: `contracts/openapi/cloud/cloud-kms-v1.yaml`, `crates/cloud-kms-domain`, `crates/cloud-kms-api`, and `crates/cloud-kms-api/tests/cloud_kms_api.rs`.
That split is the central coherence problem.
The product thesis is strong: key lifecycle, envelope encryption, tenant CMKs, rotation, BYOK/HYOK, HSM validation, cryptoshredding, audit receipts, and KMS-use evidence.
The ownership package is weak: the microservice folder documents high-value behavior but does not own enough machine-readable deployability or contract evidence for an intern-buildable Phase 0 cloud infrastructure service.
No P0 finding is assigned because cloud-kms is Phase 0 shared infrastructure, not one of the explicitly named P0 business-critical families in ADR-0328.
The audit assigns P1 to missing context coverage, missing OpenTofu IaC, missing OS matrix, missing local PRD/architecture, missing local SLO/contract surfaces, OCI demo_trial tenant_class drift, and runtime/workflow guidance that uses shell/make/provider CLIs without a Rust/OpenTofu control surface.
The audit assigns P2 to documentation-internal contradictions, stale reverse-reference expectations, benchmark provenance gaps, and counterpart-parity incompleteness.

## Section 1 - Microservice Purpose Summary

The local tenant_class matrix defines cloud-kms as the key management substrate for the cloud surface.
Its direct purpose is to own customer-managed keys, key-encryption keys, data-encryption-key issuance, envelope encryption, key rotation, cryptoshredding, and HSM signing.
The FAQ sharpens that purpose: AWS KMS, Google Cloud KMS, Azure Key Vault, Vault Enterprise, CloudHSM, Thales, and Utimaco are not the authoritative policy layer; they are downstream custody or backing adapters when a context requires them.
The domain crate confirms the runtime boundary more narrowly: it owns typed control/data invariants for `cloud.kms.encrypt` and `cloud.kms.decrypt`, including per-tenant keys, per-cell HSM partition binding, residency, pack-certified/FIPS validation, key-use receipts, and key-destruction evidence.
The domain crate explicitly does not perform cryptography or HSM I/O; adapter/runtime crates consume the invariants.
The OpenAPI contract confirms the public API surface currently consists of authorize-encrypt and authorize-decrypt receipt operations, not general-purpose key CRUD, aliasing, signing, random generation, import, external-key-store management, or HSM cluster lifecycle.
The cloud product PRD names `cloud-kms-api` as the KMS encrypt/decrypt authorization receipt REST API and names KMS/BYOK/HYOK as a cloud product capability.
That means the real product purpose is two-layered.
Layer one is currently implemented in Rust as policy-bound authorization and receipt recording for encrypt/decrypt use events.
Layer two is described in local docs as a broader KMS control plane for lifecycle, custody, HSM partitions, rotation, cryptoshredding, and external provider adapters.
The current artifacts do not yet reconcile those layers.
The local documentation reads like a mature key-management service.
The local machine-readable surface under `microservices/cloud-kms/` reads like a documentation-only pack.
The repo-level contract and crates prove there is implementation elsewhere, but they are not mirrored or owned in the microservice folder.
For Wave 14 aggregation, cloud-kms should be treated as a high-risk Phase 0 service with a real Rust nucleus and an incomplete ownership envelope.

## Section 2 - Inventory Snapshot

| file | lines | role | coherent_with_purpose |
|---|---:|---|---|
| `benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md` | 100 | vendor comparison and claimed measured benchmark notes | partial |
| `retired tenant_class adoption artifact` | 99 | demo_trial/paid tenant_class capability tenant_class specification | partial |
| `faqs/kms-engineer-faq.md` | 193 | engineer Q/A for key custody, BYOK, HSM, rotation, audit, and incident behavior | yes |
| `migration-playbooks/from-aws-kms-and-vault-enterprise.md` | 166 | migration path from AWS KMS and Vault Enterprise | partial |
| `onboarding/kms-engineer-first-week.md` | 161 | first-week walkthrough for KMS engineers | partial |
| `reference-implementations/envelope-encrypt-rust-sdk.md` | 208 | Rust SDK reference for envelope encrypt/rotate/cryptoshred | yes |
| `runbooks/hsm-cluster-failover.md` | 269 | failover runbook for HSM quorum degradation | partial |
| `runbooks/key-material-quorum-loss.md` | 269 | quorum-loss incident runbook | partial |
| `runbooks/rotation-cadence-drift-detection.md` | 267 | key rotation drift incident runbook | partial |
| `tutorials/envelope-encrypt-rotate-and-cryptoshred.md` | 210 | tutorial for envelope encryption lifecycle | partial |

Total files seen under the target path: 10.
Total lines in target-path files: 1,942.
Local `PRD.md`: absent.
Local `ARCHITECTURE.md`: absent.
Local `README.md`: absent.
Local `decisions/ADR-MS-*.md`: absent.
Local `implementation-plans/IP-*.md`: absent.
Local `contracts/*.{yaml,json,proto}`: absent.
Local `slos/*.openslo.yaml`: absent.
Local `supported-oses.json`: absent.
Local `iac/` directory: absent.
Local `tests/` directory: absent.
Local `src/` directory: absent.
Local `capacity-model.md`: absent.
Local `failure-modes.md`: absent.
Local `incident-response.md`: absent.
Local `cost-budget.md`: absent.
Local `dpia.md`: absent.
Local `compliance.md`: absent.
Representative repo-level contract read: `contracts/openapi/cloud/cloud-kms-v1.yaml:1-420`.
Representative repo-level code read: `crates/cloud-kms-domain/src/lib.rs:1-260`.
Representative repo-level tests read: `crates/cloud-kms-api/tests/cloud_kms_api.rs:1-352`.
Chat-history raw matches processed: 48.
Chat-history relevant clusters processed: Wave 2 Batch 2.1 dispatch at chat line 15245 and active task reminder at chat line 15231.
Counterpart research sources used: AWS KMS official docs, Google Cloud KMS official docs, HashiCorp Vault official docs.

## Section 3 - Nine-Dimension Audit

### Section 3.1 - Dimension 1: Internal Coherence Within Microservice Path

Dimension verdict: partial, with P1/P2 drift.
The local docs agree on the broad mission: cloud-kms provides tenant key custody, HSM partitions, BYOK/HYOK, envelope encryption, rotation, cryptoshredding, and audit evidence.
The local docs do not agree on the execution surface: some files describe SDK calls, some prescribe make targets, some use provider CLIs, and none points to a local contract, source tree, SLO file, or deployment manifest.
Internal reference 1: `retired tenant_class adoption artifact:7-11` defines CMK, KEK, DEK, envelope encryption, rotation, cryptoshredding, HSM signing, and no-roll-your-own-crypto enforcement. Target resolves conceptually to FAQ, tutorial, and SDK docs.
Internal reference 2: `retired tenant_class adoption artifact:24-25` declares demo_trial latency and availability SLOs. Target SLO file does not exist under `microservices/cloud-kms/slos/`.
Internal reference 3: `retired tenant_class adoption artifact:42-43` declares paid p95 and availability. Target OpenSLO file does not exist locally.
Internal reference 4: `retired tenant_class adoption artifact:60-61` declares paid p95 and availability. Target OpenSLO file does not exist locally.
Internal reference 5: `retired tenant_class adoption artifact:78-79` declares paid p95 and availability. Target OpenSLO file does not exist locally.
Internal reference 6: `faqs/kms-engineer-faq.md:7-12` says cloud-kms is the canonical policy and receipt surface and downstream providers are adapters. Target architecture file is absent, but repo-level domain crate supports the invariant at `crates/cloud-kms-domain/src/lib.rs:1-7`.
Internal reference 7: `faqs/kms-engineer-faq.md:60-68` describes cryptoshredding with destruction receipts and fail-closed behavior. Target local API contract for destruction receipts is absent; repo-level domain struct exists at `crates/cloud-kms-domain/src/lib.rs:203-220`.
Internal reference 8: `faqs/kms-engineer-faq.md:132-136` says cloud-iam performs principal/session policy and cloud-kms verifies action, AAD, data class, and residency. Target local Cedar policy file is absent.
Internal reference 9: `faqs/kms-engineer-faq.md:165-168` names a Vault adapter crate path. The local microservice path has no adapter inventory.
Internal reference 10: `reference-implementations/envelope-encrypt-rust-sdk.md:6-23` declares a Rust SDK dependency and example. Target SDK crate is not under the microservice path.
Internal reference 11: `reference-implementations/envelope-encrypt-rust-sdk.md:196-203` says tests use `cargo test --features hermetic` and SoftHSM 2.6.1. No local `tests/` path or CI lane file exists under this microservice.
Internal reference 12: `migration-playbooks/from-aws-kms-and-vault-enterprise.md:6-39` uses AWS CLI, Vault CLI, `jq`, and shell loops for inventory. This resolves as a playbook but conflicts with Rust-strict/OpenTofu direction for operational primitives.
Internal reference 13: `onboarding/kms-engineer-first-week.md:21-28` uses `make dev-cell.up` and `make dev-tenant.create`. That conflicts with ADR-0328's canonical Rust build invocation and OpenTofu onboarding shape.
Internal reference 14: `tutorials/envelope-encrypt-rotate-and-cryptoshred.md:7-9` requires `make`, `jq`, and `openssl` on PATH. That is operationally useful but not canonical under D-18/D-20.
Internal reference 15: `benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md:91-100` says reproducibility uses `make benchmarks.cloud-kms.run` and evidence should be under `.foundry/evidence/benchmarks/cloud-kms/2026-05-13T16:42:18Z/`; the referenced evidence directory is not present in the current workspace.
Contradiction probe 1: demo_trial cost in `retired tenant_class adoption artifact:29` says approximate cost is `$40/month`, while the canonical OCI profile requires demo_trial on OCI to be Always Free at `specs/master-plan-sequencing.json:856-864`.
Contradiction probe 2: benchmark target/context in `benchmarks/...md:1-4` compares AWS KMS, Google Cloud KMS, Azure Key Vault, and Vault Enterprise, while this audit's top-3 bar is AWS KMS, Google Cloud KMS, and HashiCorp Vault self-hosted. Azure is extra, and self-hosted Vault is not the same as Vault Enterprise as a managed/commercial tier.
Contradiction probe 3: `benchmarks/...md:3-4` claims measured dates, but the evidence path in `benchmarks/...md:100` is absent. That is a benchmark-provenance contradiction, not a product contradiction.
Contradiction probe 4: `faqs/kms-engineer-faq.md:114-120` describes cross-region replication with "Spanner-class TrueTime," but no local architecture or context matrix defines whether that dependency exists for on-prem, colo, or oyatie-as-cloud-provider.
Contradiction probe 5: `retired tenant_class adoption artifact:67-83` says paid includes Vault Enterprise HSM seal, while the product purpose says cloud-kms remains the canonical authority, and the top-3 counterpart for this audit is self-hosted Vault. The relationship is under-explained.
Contradiction probe 6: `reference-implementations/...md:25-164` is Rust-shaped and canonical-friendly, while onboarding/tutorial/benchmark docs use make and shell paths. This is an internal toolchain split.
Contradiction probe 7: runbooks repeatedly prescribe observability queries and breakers, but no local `observability/`, `slos/`, or `incident-response.md` file anchors the names.
Contradiction probe 8: local docs include FIPS/KCMVP/Common Criteria language, but no local compliance matrix or evidence manifest exists.
Contradiction probe 9: local docs mention Cedar decisions, but no local `policies/` or `cedar/` path exists.
Contradiction probe 10: local docs present high-cardinality product capabilities, but repo-level OpenAPI only exposes encrypt/decrypt authorization receipts.
Severity for Dimension 1: P1 for local ownership-envelope gaps and P2 for internal documentation/tooling inconsistencies.

### Section 3.2 - Dimension 2: Outbound Cross-References

Dimension verdict: drifted-fixable with missing reciprocal ownership evidence.
Outbound reference 1: `microservices/cloud-kms/runbooks/hsm-cluster-failover.md:250-262` names cloud-iam, cloud-network, foundry, audit-chain, tenancy, cloud-billing, observability, comms-email, security, compliance, support, and workflow-engine. These are plausible dependencies, but no local handoff file exists.
Outbound reference 2: `microservices/cloud-kms/runbooks/key-material-quorum-loss.md:250-262` repeats the same cross-service handoff set. Target services mostly exist, but reciprocal handoffs are not centralized.
Outbound reference 3: `microservices/cloud-kms/runbooks/rotation-cadence-drift-detection.md:248-260` repeats cross-service handoffs. The pattern is consistent but not linked to `cross-microservice-handoffs.md`, which is absent.
Outbound reference 4: `faqs/kms-engineer-faq.md:132-136` references cloud-iam. Reverse references exist in cloud-iam runbooks: `microservices/cloud-iam/runbooks/federated-identity-provider-stall.md:129`, `:162`, `:239`, `:249`.
Outbound reference 5: `faqs/kms-engineer-faq.md:172-176` references Foundry hooks and audit evidence. Reverse references appear in `docs/products/cloud/PRD.md:549` and runtime bindings, but no local event schema file exists.
Outbound reference 6: `migration-playbooks/from-aws-kms-and-vault-enterprise.md:11-38` references AWS KMS and Vault Enterprise. It does not include Google Cloud KMS despite Google being one of the top-3 counterparts for this audit.
Outbound reference 7: `benchmarks/...md:1` references AWS KMS, Google Cloud KMS, Azure Key Vault, and Vault Enterprise. That is useful but not aligned to this audit's self-hosted Vault counterpart.
Outbound reference 8: `reference-implementations/...md:14-20` references `cloud-kms-sdk`, `audit-chain-sdk`, and `cloud-iam-sdk`. The repo-level crates visible in this audit are `cloud-kms-domain` and `cloud-kms-api`, not a local SDK crate.
Outbound reference 9: `retired tenant_class adoption artifact:85-99` declares invariants including external provider indirection and receipts. Runtime proof exists in the API/domain crates, but not in local microservice docs.
Outbound reference 10: `tutorials/...md:11-210` walks lifecycle operations but does not link to the OpenAPI contract or runtime crate tests.
Reverse reference 1: `docs/SPEC.md:155` declares `cloud.kms.encrypt` and `cloud.kms.decrypt` stable, REST, KCMVP/FIPS, per-tenant key, OpenAPI source `contracts/openapi/cloud/cloud-kms-v1.yaml`, and crate `cloud-kms-api`.
Reverse reference 2: `registry/openapi/schema-bindings.tsv:21-28` binds CloudKms schemas to the OpenAPI contract and `crates/cloud-kms-api/src/lib.rs`.
Reverse reference 3: `registry/openapi/runtime-bindings.tsv:5-6` binds encrypt/decrypt operations to API functions and tests.
Reverse reference 4: `registry/catalog/cloud-kms-domain.yaml:1-9` classifies the catalog record as preview, internal-only, security-review unreviewed, and source-only supply chain.
Reverse reference 5: `docs/products/cloud/PRD.md:121` identifies `cloud-kms-api` as the KMS encrypt/decrypt authorization receipt REST API.
Reverse reference 6: `docs/products/cloud/PRD.md:173` expects KMS API p99 <=100 ms and KCMVP/FIPS binding.
Reverse reference 7: `docs/products/cloud/PRD.md:549` expects `oya.audit.cloud_kms_use` emission on decrypt.
Reverse reference 8: `docs/products/cloud/PRD.md:731` names KMS/BYOK/HYOK and indefinite KMS-use audit events.
Reverse reference 9: `docs/products/cloud/PRD.md:779` says HYOK integration evidence is still missing.
Reverse reference 10: `docs/standards/brief-template.md:1657-1662` uses cloud-kms SLO and runbook paths that are absent locally.
Reverse reference 11: `docs/architecture/wave-3-final-scorecard-2026-05-20.md:4763` says cloud-kms has no integration markers and has a cross-service integration scenario gap.
Reverse reference 12: `docs/architecture/wave-3-retrospective-2026-05-20.md:2332` says BYOK/HYOK confusion is a high-risk drift source for cloud-kms.
Reverse reference 13: `microservices/cloud-network/runbooks/mtls-handshake-failure-cascade.md:23-35` depends on cloud-kms tenant CA signing.
Reverse reference 14: `microservices/cloud-network/runbooks/mtls-handshake-failure-cascade.md:243-253` pages cloud-kms for signer failures.
Reverse reference 15: `microservices/cloud-storage/retired tenant_class adoption artifact:25` uses cloud-kms for single-tenant CMK envelope encryption.
Reverse reference 16: `microservices/cloud-data/faqs/data-engineer-faq.md:52-53` says data-layer cryptoshredding depends on cloud-kms.
Reverse reference 17: `microservices/cloud-billing/runbooks/invoice-generation-timeout.md:255` depends on cloud-kms invoice signing keys.
Reverse reference 18: `microservices/mail/PRD.md:927` depends on cloud-kms for mailbox-store/S3 encryption.
Reverse reference 19: `microservices/audit-chain/packs/SOC2.md:195` depends on cloud-kms or OpenBao key rotation proof.
Reverse reference 20: `docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2045-2046` explicitly assigns key lifecycle, rotation, custody, evidence, and policy to cloud-kms.
Orphan reference: local docs imply a `cross-microservice-handoffs.md` owner map, but that file is absent.
Orphan reference: `docs/standards/brief-template.md:1657` expects `microservices/cloud-kms/slos/control-plane.openslo.yaml`; absent.
Orphan reference: `docs/standards/brief-template.md:1659` expects `microservices/cloud-kms/runbooks/README.md`; absent.
Missing reverse reference: local runbooks name observability and support, but a single reciprocal handoff contract from those services to cloud-kms was not found in the microservice folder.
Severity for Dimension 2: P1 for missing local handoff and SLO anchors; P2 for stale reverse references and partial reciprocal mapping.

### Section 3.3 - Dimension 3: Substance Bar and Intern-Buildability

Dimension verdict: not intern-buildable from the microservice path alone.
The docs are substantive in their domain content, but they are not sufficient to let a cold intern build or deploy cloud-kms end to end from `microservices/cloud-kms/`.
The missing local PRD prevents a reader from seeing the service problem statement, personas, hard non-goals, product boundaries, and acceptance criteria in one place.
The missing local architecture document prevents a reader from understanding how the domain crate, API crate, adapters, HSMs, provider KMSes, OpenBao/Vault, cloud-iam, audit-chain, cloud-iac, and observability fit together.
The missing local OpenAPI contract is especially important because the repo-level contract is stable and real, yet the microservice folder does not point to it.
The missing local SLO manifest blocks promotion-gate verification for the latency/availability numbers already stated in the tenant_class matrix.
The missing local OS manifest blocks supported platform claims for HSM adapters, TPM, SoftHSM, macOS Secure Enclave, and package formats.
The missing local IaC blocks every deployment context claim.
The missing local tests block the reader from knowing whether examples in the docs reflect the current Rust API.
The missing local compliance/DPIA blocks a reader from connecting FIPS/KCMVP/Common Criteria/PCI/CSAP/HIPAA/PIPA claims to evidence.
The missing local failure-mode index means runbooks are standalone, not a complete failure tree.
Buildability gap 1: no `README.md` entrypoint. An intern cannot find a canonical start path under the folder.
Buildability gap 2: no `PRD.md`; cite local absence against `docs/standards/documentation-rigor.md:175-190` and brief-template header expectations at `docs/standards/brief-template.md:101-117`.
Buildability gap 3: no `ARCHITECTURE.md`; the closest architecture facts are outside the path at `crates/cloud-kms-domain/src/lib.rs:1-7`.
Buildability gap 4: no `contracts/` under the microservice despite `docs/SPEC.md:155` and `registry/openapi/runtime-bindings.tsv:5-6`.
Buildability gap 5: no `slos/` despite tenant_class SLO claims at `retired tenant_class adoption artifact:24-25`, `:42-43`, `:60-61`, and `:78-79`.
Buildability gap 6: no `iac/` despite six-context IaC requirements at ADR-0328 D-16 and `specs/master-plan-sequencing.json:704-776`.
Buildability gap 7: no `supported-oses.json` despite D-17 and `specs/master-plan-sequencing.json:777-815`.
Buildability gap 8: no local code sample beyond markdown; repo-level Rust crates exist, but their ownership is not represented here.
Buildability gap 9: no local CI lane spec; the scorecard already says integration markers are 0 at `docs/architecture/wave-3-final-scorecard-2026-05-20.md:4763`.
Buildability gap 10: no local event schema for `cloud.kms_key_used.v1`, although cloud PRD references it at `docs/products/cloud/PRD.md:526`.
Buildability gap 11: no local policy files for Cedar/KMS permits, although FAQ references Cedar authorization semantics at `faqs/kms-engineer-faq.md:132-136`.
Buildability gap 12: no local provider-adapter matrix for AWS KMS, Google Cloud KMS, OCI Vault, TPM, Apple Secure Enclave, SoftHSM, Thales, Utimaco, CloudHSM, and Vault.
Weak section 1: `onboarding/kms-engineer-first-week.md:21-28` jumps straight into make targets without explaining where those targets are defined.
Weak section 2: `tutorials/envelope-encrypt-rotate-and-cryptoshred.md:7-9` lists prerequisites but not Rust/OpenTofu canonical build.
Weak section 3: `migration-playbooks/from-aws-kms-and-vault-enterprise.md:6-39` uses shell inventory flows rather than an Oyatie import tool contract.
Weak section 4: `benchmarks/...md:91-100` gives a make command and absent evidence path rather than reproducible benchmark harness details.
Weak section 5: runbooks are deep and useful, but they assume existing `oya ops` surfaces, dashboards, and metrics that are not locally specified.
Positive evidence 1: `reference-implementations/envelope-encrypt-rust-sdk.md:25-164` gives a real Rust SDK flow with tenant, AAD, receipt, rotation, and cryptoshred semantics.
Positive evidence 2: `crates/cloud-kms-api/tests/cloud_kms_api.rs:114-352` proves API-level behavior for drift rejection, authorization denial, idempotency, conflicts, AAD validation, and data class validation.
Positive evidence 3: `crates/cloud-kms-domain/src/lib.rs:71-113` models origins, usages, HSM validation, states, purposes, and operations.
Positive evidence 4: `contracts/openapi/cloud/cloud-kms-v1.yaml:9-168` exposes the encrypt/decrypt authorization endpoints and required headers.
Positive evidence 5: `contracts/openapi/cloud/cloud-kms-v1.yaml:176-301` defines the request body fields and purpose enum.
Intern-buildability result: a strong senior engineer can triangulate the service from repo-wide sources; a cold intern cannot build it from the microservice folder alone.
Severity for Dimension 3: P1.

### Section 3.4 - Dimension 4: Canonical-Direction Alignment

Dimension verdict: mostly drifted-fixable, with one clean pass for absence of forbidden source files.
Constraint 1 - multi-context: drifted-fixable. ADR-0328 D-15 and `specs/master-plan-sequencing.json:704-746` require six contexts or explicit N/A records. The microservice has no `deployment_contexts` manifest and no `iac/` directories.
Constraint 2 - OpenTofu IaC: drifted-fixable. ADR-0328 D-16 and `specs/master-plan-sequencing.json:747-776` require OpenTofu-only context modules. The microservice has no `iac/` directory.
Constraint 3 - OS support: drifted-fixable. ADR-0328 D-17 and `specs/master-plan-sequencing.json:777-815` require Tier-1/Tier-2/out-of-scope declarations. The microservice has no `supported-oses.json`.
Constraint 4 - Rust strict: partially aligned. The target path contains no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.cs` source files. However, docs prescribe make, shell, `jq`, AWS CLI, and Vault CLI workflows at `tutorials/...md:7-9`, `onboarding/...md:21-28`, and `migration-playbooks/...md:6-39`.
Constraint 5 - OCI Always Free: drifted-fixable. `specs/master-plan-sequencing.json:856-864` requires per-microservice OCI Always Free modules and demo_trial tenant_class mapping. The microservice has no `iac/oci-guest/always-free/`, and demo_trial cost says `$40/month` at `retired tenant_class adoption artifact:29`.
Canonical direction also says cloud-kms remains the Oyatie authority even when OCI Vault backs a guest context at ADR-0328 D-19 lines 3518-3521; local docs agree in spirit but lack OCI-specific adapter/deployment proof.
Canonical direction says cloud-kms owns key lifecycle, rotation, custody, evidence, and policy at ADR-0328 D-15 lines 2045-2046; local docs cover those themes but repo-level API only exposes encrypt/decrypt authorization receipts.
Canonical direction requires context N/A justification when support is missing. The local path has no N/A entries for any of the six contexts.
Canonical direction forbids direct cloud-vendor APIs from business logic in guest contexts. The local migration playbook uses AWS CLI for inventory, but this is migration documentation, not application code; classify as P2 if kept strictly migration-only and P1 if it becomes runtime control logic.
Canonical direction requires OpenTofu and forbids Terraform/Pulumi/CloudFormation as engines. Existing local migration text names Terraform AWS provider as mature ecosystem tooling at `migration-playbooks/...md:165`; this is a P2 wording drift unless it is promoted as implementation guidance.
Canonical direction requires sigstore signing wiring for IaC modules. No local IaC modules exist, so signing is absent.
Canonical direction requires state backend per context. No local state backend exists.
Canonical direction requires build invocation `cargo build --workspace --release --all-features --locked`. Local reference implementation uses `cargo run --release` and `cargo test`, while docs use make for dev and benchmark entrypoints.
Canonical direction allows markdown, yaml, json, proto, HCL, Cedar, SQL, and OpenAPI. The existing local files are markdown only, so file extensions themselves are allowed.
Canonical direction requires frontend code only under platform-specific frontend folders. No frontend code exists.
Canonical direction requires OCI Always Free demo_trial for guest-on-OCI. The local demo_trial tenant_class is useful for generic low-cost deployment but not OCI Always Free compliant.
Canonical direction requires all cloud-* services to be non-wrapper IaaS control planes in the `oyatie-as-cloud-provider` context. The local FAQ supports this by stating providers are downstream adapters, but deployment proof is absent.
Classification summary: multi-context drifted-fixable; OpenTofu drifted-fixable; OS support drifted-fixable; Rust strict partial; OCI Always Free drifted-fixable.
Severity for Dimension 4: P1.

### Section 3.5 - Dimension 5: Industry-Counterpart Parity

Dimension verdict: partial.
Top-3 union counterparts for this audit: AWS KMS, Google Cloud KMS, HashiCorp Vault self-hosted.
AWS KMS has standard key stores, CloudHSM custom key stores, external key stores, imported key material, multi-Region keys, automatic/on-demand/manual rotation, symmetric/asymmetric/HMAC/ML-DSA key types, grants, aliases, key policies, CloudTrail audit, quotas, and custom key store limits.
Google Cloud KMS has software keys, Cloud HSM, Single-tenant Cloud HSM, Cloud EKM, Autokey, import, rotation, key version states, destroy/restore, IAM, CMEK integrations, Key Access Justifications, quotas, and key-ring/location organization.
HashiCorp Vault self-hosted has transit encryption-as-a-service, datakey generation, encrypt/decrypt/rewrap, sign/verify, HMAC/hash/random, key derivation, convergent encryption, Shamir seal, cloud/HSM auto-unseal, audit devices, auth methods, policy, namespaces in Enterprise, performance/DR replication in Enterprise, and self-hosted operational control.
Oyatie cloud-kms has clear local coverage for envelope encryption, AAD fingerprints, key-use receipts, BYOK/HYOK concepts, HSM tiers, cryptoshredding, rotation cadence, incident runbooks, and Rust SDK examples.
Oyatie cloud-kms has repo-level implementation coverage for encrypt/decrypt authorization receipts, idempotency, tenant/path/body drift rejection, AAD digest validation, data-class validation, key origins, HSM validation, key states, and destruction receipts.
Missing union capability 1: local key CRUD API documentation for create/update/disable/delete/schedule deletion/restore.
Missing union capability 2: local alias management comparable to AWS aliases.
Missing union capability 3: local grant/delegated access model comparable to AWS KMS grants.
Missing union capability 4: local key policy file and Cedar mapping.
Missing union capability 5: local CloudHSM/custom key store equivalent per context.
Missing union capability 6: local external key store/XKS/EKM proxy protocol.
Missing union capability 7: local imported key material import/reimport/expiration contract.
Missing union capability 8: local multi-region key replication contract.
Missing union capability 9: local signing and verification API contract, despite HSM signing being in the tenant_class matrix.
Missing union capability 10: local MAC/HMAC API contract.
Missing union capability 11: local random generation API contract.
Missing union capability 12: local Vault transit rewrap/batch/derived/convergent encryption parity.
Missing union capability 13: local Vault seal/unseal operational parity for self-hosted Vault.
Missing union capability 14: local key version state transition matrix comparable to Google Cloud KMS.
Missing union capability 15: local Autokey-style automatic resource-specific key provisioning.
Missing union capability 16: local Key Access Justification equivalent.
Missing union capability 17: local quota and rate-limit spec per operation.
Missing union capability 18: local pricing/cost budget by context, especially OCI Always Free.
Missing union capability 19: local audit log schema for every operation, not only decrypt use.
Missing union capability 20: local provider adapter contract for AWS, GCP, OCI, TPM, Secure Enclave, SoftHSM, Thales, Utimaco, and Vault.
Additive Oyatie capability 1: data-class-aware KMS receipts integrated with Oyatie data boundary labels.
Additive Oyatie capability 2: AAD fingerprint treated as a first-class receipt/control field.
Additive Oyatie capability 3: per-pack HSM validation language for KCMVP/FIPS/Common Criteria/PCI in the domain model.
Additive Oyatie capability 4: cryptoshredding framed as tenant-purpose evidence rather than simple key deletion.
Additive Oyatie capability 5: Cedar-gated purpose binding across storage, workspace, replication, secret provider, and database backup.
Headline parity finding: cloud-kms is conceptually ambitious but only partially matches the union coverage of AWS KMS, Google Cloud KMS, and Vault self-hosted.
Severity for Dimension 5: P2 because product scope is present but artifact coverage is incomplete.

### Section 3.6 - Dimension 6: Multi-Context Deployment Support

Dimension verdict: missing.
Canonical context 1 `oyatie-public-cloud`: not evidenced under the microservice path. Required path is `iac/oyatie-public-cloud/`; absent.
Canonical context 2 `guest-on-aws`: not evidenced under the microservice path. Required path is `iac/guest-on-aws/`; absent.
Canonical context 3 `guest-on-oci`: not evidenced under the microservice path. Required path is `iac/oci-guest/`; absent.
Canonical context 4 `on-prem`: not evidenced under the microservice path. Required path is `iac/on-prem/`; absent.
Canonical context 5 `colo`: not evidenced under the microservice path. Required path is `iac/colo/`; absent.
Canonical context 6 `oyatie-as-cloud-provider`: not evidenced under the microservice path. Required path is `iac/oyatie-iaas/`; absent.
The audit found no local manifest declaring supported contexts.
The audit found no local N/A table explaining missing primitives, customer impact, remediation owner, and revisit gate.
The audit found no local OCI-specific Always Free overlay.
The audit found no local AWS guest adapter module.
The audit found no local OCI Vault adapter module.
The audit found no local on-prem HSM/TPM/SoftHSM deployment module.
The audit found no local colo HSM cluster deployment module.
The audit found no local oyatie-as-provider HSM partition module.
The local FAQ says cloud providers are downstream adapters, which aligns with provider-agnostic doctrine in spirit.
The local migration playbook uses provider CLI inventory commands, but it does not make provider APIs the application business logic.
The local runbooks assume cells, regions, HSM partitions, and tenant IDs, but no context deployment matrix maps those fields.
The local tenant_class matrix does not say which tenant_classes are valid in which context.
The demo_trial tenant_class could plausibly run on software/SoftHSM in on-prem or dev contexts, but this is not declared.
paid and higher depend on HSM providers, but context-specific sourcing is not declared.
paid says dedicated Utimaco + Thales + Vault Enterprise HSM seal, which may be incompatible with OCI Always Free and some guest contexts.
For `oyatie-public-cloud`, cloud-kms should likely be mandatory because all cloud product encryption depends on it.
For `guest-on-aws`, cloud-kms should likely support AWS KMS/CloudHSM/XKS adapters, but no IaC or adapter contract is local.
For `guest-on-oci`, cloud-kms should support OCI Vault as backing while remaining Oyatie authority, but no module exists.
For `on-prem`, cloud-kms should support HSM/TPM/SoftHSM and manual custody ceremony without cloud-managed prerequisites, but no module exists.
For `colo`, cloud-kms should support dedicated HSM cluster deployment and remote-hands evidence, but no module exists.
For `oyatie-as-cloud-provider`, cloud-kms should expose the provider KMS surface, not wrap another provider, but only encrypt/decrypt authorization receipts are currently visible in repo-level OpenAPI.
Forbidden direct vendor API pattern: no runtime source under the microservice path, so no business logic vendor-call violation was proven.
Forbidden operational pattern: migration docs use direct AWS/Vault CLIs. Keep them as migration inventory aids only; do not promote them to runtime control paths.
Severity for Dimension 6: P1.

### Section 3.7 - Dimension 7: OpenTofu IaC Coverage

Dimension verdict: absent.
Required context directory `iac/oyatie-public-cloud/`: absent.
Required context directory `iac/guest-on-aws/`: absent.
Required context directory `iac/oci-guest/`: absent.
Required context directory `iac/oci-guest/always-free/`: absent.
Required context directory `iac/on-prem/`: absent.
Required context directory `iac/colo/`: absent.
Required context directory `iac/oyatie-iaas/`: absent.
Required file `main.tf` in each supported context: absent.
Required file `variables.tf` in each supported context: absent.
Required file `outputs.tf` in each supported context: absent.
Required file `versions.tf` in each supported context: absent.
Required file `README.md` in each supported context: absent.
Provider pinning evidence: absent.
OpenTofu version pinning evidence: absent.
Module source pinning evidence: absent.
Sigstore/cosign signing evidence: absent.
Remote state backend evidence for oyatie-public-cloud: absent.
Remote state backend evidence for guest-on-aws: absent.
Remote state backend evidence for guest-on-oci: absent.
Remote state backend evidence for on-prem: absent.
Remote state backend evidence for colo: absent.
Remote state backend evidence for oyatie-as-cloud-provider: absent.
State lock evidence: absent.
Drift detection hook evidence: absent.
Plan artifact retention evidence: absent.
OPA/Cedar admission hook evidence for IaC: absent.
Tenant onboarding path via `tofu init`: absent.
Tenant onboarding path via `tofu plan`: absent.
Tenant onboarding path via `tofu apply`: absent.
Forbidden `null_resource` in local path: not found.
Forbidden `local-exec` in local path: not found.
Forbidden `remote-exec` in local path: not found.
Forbidden SSH provisioners in local IaC: not applicable because no IaC exists.
Forbidden hand-edited tfstate in local path: not found.
Forbidden unsigned local modules: not applicable because no local modules exist.
Forbidden Pulumi references in local path: not found.
Forbidden CloudFormation references in local path: not found.
Forbidden ARM/Bicep references in local path: not found.
Terraform reference found: `migration-playbooks/from-aws-kms-and-vault-enterprise.md:165` mentions the Terraform AWS provider as mature ecosystem tooling.
That Terraform reference is not an IaC implementation, but it is directionally risky because ADR-0328 and the zero-handroll memory file require OpenTofu-only language.
OpenTofu reference found locally: none.
OpenTofu reference needed for AWS guest: module should declare AWS KMS/CloudHSM/XKS resources through OpenTofu, not AWS CLI orchestration.
OpenTofu reference needed for OCI guest: module should declare OCI Vault where used, plus Always Free constraints, through OpenTofu.
OpenTofu reference needed for on-prem: module should declare local substrate expectations and HSM inventory without SSH provisioners.
OpenTofu reference needed for colo: module should model remote-hands gates declaratively, not as a replacement for state.
OpenTofu reference needed for oyatie-as-provider: module should create Oyatie KMS provider primitives, not wrapper variables.
State backend expectation: `specs/master-plan-sequencing.json:747-776` says every context must define backend and locking.
Security expectation: ADR-0328 D-16 forbids README-only onboarding; current path has README-equivalent prose and no module.
Substance expectation: `feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` says every microservice should have context IaC directories and missing IaC is audit P1.
Cloud-kms-specific IaC gap: no declaration of HSM partition lifecycle.
Cloud-kms-specific IaC gap: no declaration of key material custody boundary.
Cloud-kms-specific IaC gap: no declaration of CloudHSM custom key store or XKS proxy resources for AWS.
Cloud-kms-specific IaC gap: no declaration of OCI Vault backing for guest-on-OCI.
Cloud-kms-specific IaC gap: no declaration of SoftHSM/TPM-only demo_trial substrate for Always Free.
Cloud-kms-specific IaC gap: no declaration of per-cell KMS partition wiring.
Cloud-kms-specific IaC gap: no declaration of key-rotation scheduler infrastructure.
Cloud-kms-specific IaC gap: no declaration of audit-chain event sink.
Cloud-kms-specific IaC gap: no declaration of HSM attestation material storage.
Cloud-kms-specific IaC gap: no declaration of cross-region replication topology.
Cloud-kms-specific IaC gap: no declaration of destructive-operation approval quorum.
Cloud-kms-specific IaC gap: no declaration of break-glass path.
Severity for Dimension 7: P1.

### Section 3.8 - Dimension 8: OS Support Matrix

Dimension verdict: absent.
Required local manifest `supported-oses.json`: absent.
Alternative supported_oses field in a local manifest: absent.
Tier-1 OS Talos: no local support declaration.
Tier-1 OS RHEL 9+: no local support declaration.
Tier-1 OS Oracle Linux 9+: no local support declaration.
Tier-1 OS SLES 15 SP6+: no local support declaration.
Tier-1 OS Ubuntu 24.04 LTS: no local support declaration.
Tier-1 OS Debian 13: no local support declaration.
Tier-1 OS Rocky Linux 9: no local support declaration.
Tier-1 OS AlmaLinux 9: no local support declaration.
Tier-1 OS CentOS Stream 10: no local support declaration.
Tier-1 OS Amazon Linux 2023: no local support declaration.
Tier-1 OS Flatcar Container Linux: no local support declaration.
Tier-1 OS VMware Photon OS 5: no local support declaration.
Tier-1 OS macOS Apple Silicon M5+ when applicable: no local support declaration.
Tier-2 ppc64le: no test-only declaration.
Tier-2 s390x: no test-only declaration.
Out-of-scope macOS Intel: no explicit exclusion.
Out-of-scope pre-M5 Apple Silicon: no explicit exclusion.
Out-of-scope FreeBSD: no explicit exclusion.
Out-of-scope OpenBSD: no explicit exclusion.
Out-of-scope Windows Server: no explicit exclusion.
Out-of-scope Solaris/illumos: no explicit exclusion.
Arch matrix x86_64: no local support declaration.
Arch matrix aarch64: no local support declaration.
Arch matrix riscv64: no local support declaration.
Arch matrix ppc64le: no test-only declaration.
Arch matrix s390x: no test-only declaration.
RPM packaging: no local declaration.
DEB packaging: no local declaration.
Container image packaging: no local declaration.
Talos extension packaging: no local declaration.
Flatcar extension packaging: no local declaration.
Photon packaging: no local declaration.
macOS `.pkg` packaging: no local declaration.
Homebrew tap packaging: no local declaration.
CI lane for Talos: absent locally.
CI lane for RHEL: absent locally.
CI lane for Oracle Linux: absent locally.
CI lane for SLES: absent locally.
CI lane for Ubuntu: absent locally.
CI lane for Debian: absent locally.
CI lane for Rocky: absent locally.
CI lane for Alma: absent locally.
CI lane for CentOS Stream: absent locally.
CI lane for Amazon Linux: absent locally.
CI lane for Flatcar: absent locally.
CI lane for Photon: absent locally.
CI lane for macOS M5+: absent locally.
KMS/TPM/Secure Enclave abstraction required by memory: `feedback_os_support_matrix_2026_05_20.md:45-54`.
Cloud-kms adapter surface named by memory: AWS KMS, OCI Vault, Azure Key Vault, Apple Secure Enclave, TPM, SoftHSM.
Local docs mention SoftHSM in test context at `reference-implementations/envelope-encrypt-rust-sdk.md:196-203`.
Local docs mention HSM products in tenant_class matrix at `retired tenant_class adoption artifact:31-83`.
Local docs do not map HSM support to OS package/runtime constraints.
Local docs do not explain whether macOS M5+ is only a developer signer path or a deployable service runtime.
Local docs do not explain whether Talos/Flatcar run only containerized KMS API nodes while HSM agents run externally.
Local docs do not explain whether ppc64le/s390x are test-only for FIPS/KMS compliance validation.
Local docs do not explain which OSes can use TPM-backed custody.
Local docs do not explain which OSes can use OCI Always Free Ampere Oracle Linux.
Severity for Dimension 8: P1.

### Section 3.9 - Dimension 9: Rust-Strict Language Coverage

Dimension verdict: source-file clean, workflow-doc drift.
Forbidden source scan result: no `.py` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.js` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.ts` or `.tsx` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.rb` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.go` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.java` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.scala` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.groovy` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.php` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.fs` or `.fsx` files under `microservices/cloud-kms/`.
Forbidden source scan result: no `.cs` files under `microservices/cloud-kms/`.
Authorized markdown: all ten local files are `.md`, allowed by ADR-0328 language policy.
Authorized YAML/JSON/OpenAPI/Proto files: none local.
Authorized Cedar files: none local.
Authorized SQL files: none local.
Generated SDK output: none local.
Frontend Swift/Kotlin/WinUI3: none local.
Backend Rust source under local `src/`: absent.
Repo-level Rust source exists in `crates/cloud-kms-domain/src/lib.rs` and `crates/cloud-kms-api/src/lib.rs`.
Repo-level Rust tests exist in `crates/cloud-kms-api/tests/cloud_kms_api.rs`.
Canonical build invocation required by ADR-0328: `cargo build --workspace --release --all-features --locked`.
Local docs do not name the canonical build invocation.
Local reference implementation names `cargo run --release` at `reference-implementations/envelope-encrypt-rust-sdk.md:166-170`.
Local reference implementation names `cargo test --features hermetic` at `reference-implementations/envelope-encrypt-rust-sdk.md:196-203`.
Local onboarding uses `make dev-cell.up` and `make dev-tenant.create` at `onboarding/kms-engineer-first-week.md:21-28`.
Local tutorial requires `make`, `jq`, and `openssl` at `tutorials/envelope-encrypt-rotate-and-cryptoshred.md:7-9`.
Local benchmark reproducibility uses `make benchmarks.cloud-kms.run` at `benchmarks/...md:91-98`.
Local migration playbook uses shell loops with AWS CLI and Vault CLI at `migration-playbooks/from-aws-kms-and-vault-enterprise.md:6-39`.
This workflow-doc drift should not be classified as forbidden source code.
This workflow-doc drift should be remediated because ADR-0328 forbids non-Rust backend build invocations and manual shell/bootstrap pathways as canonical.
The desired remediation is to keep shell snippets as illustrative operator transcripts only when paired with canonical Rust/Oya/OpenTofu commands.
The desired remediation is to replace make-first onboarding with `cargo build --workspace --release --all-features --locked`, `cargo test`, and OpenTofu module commands.
The desired remediation is to replace direct AWS/Vault CLI migration control with a Rust migration tool plus OpenTofu-managed provider resources.
The desired remediation is to document any exception as a local ADR if external CLI usage remains required for evidence-only import.
Severity for Dimension 9: P1 for canonical workflow drift; no P0/P1 forbidden source-file violation.

## Section 4 - Findings Summary

| severity | dimension | short description | citation | remediation hint |
|---|---|---|---|---|
| P1 | 1,3 | Local PRD is absent for a Phase 0 service with broad external dependencies. | `docs/standards/documentation-rigor.md:175-190`; inventory in Section 2 | Add `PRD.md` with purpose, personas, API scope, non-goals, and acceptance criteria. |
| P1 | 1,3 | Local architecture doc is absent; closest architecture is outside path. | `crates/cloud-kms-domain/src/lib.rs:1-7`; inventory in Section 2 | Add `ARCHITECTURE.md` mapping domain/API/adapters/HSM/audit/cloud-iac. |
| P1 | 2,3 | Local contract folder absent despite stable repo contract bindings. | `docs/SPEC.md:155`; `registry/openapi/runtime-bindings.tsv:5-6` | Add local contract pointer or mirrored ownership manifest under `microservices/cloud-kms/contracts/`. |
| P1 | 1,3 | SLO files absent despite four tenant_class SLO claims. | `retired tenant_class adoption artifact:24-25`; `:42-43`; `:60-61`; `:78-79` | Add OpenSLO specs per tenant_class and context. |
| P1 | 4,6,7 | Six deployment contexts have no local manifest or IaC directories. | `specs/master-plan-sequencing.json:704-746`; `docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2235` | Add context manifest and OpenTofu modules or explicit N/A records. |
| P1 | 7 | OpenTofu coverage absent for all required context paths. | `specs/master-plan-sequencing.json:747-776`; `feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` | Add `iac/<context>/` with pinned OpenTofu modules, state backend, signing, and README. |
| P1 | 8 | OS support manifest absent. | `specs/master-plan-sequencing.json:777-815`; `feedback_os_support_matrix_2026_05_20.md:10-54` | Add `supported-oses.json` with Tier-1/Tier-2/out-of-scope and package/CI details. |
| P1 | 4,6 | OCI demo_trial tenant_class does not reconcile with Always Free. | `specs/master-plan-sequencing.json:856-864`; `retired tenant_class adoption artifact:29` | Split generic demo_trial from OCI demo_trial tenant_class or make OCI demo_trial tenant_class explicitly Always Free. |
| P1 | 9 | Docs prescribe make/shell tooling as first-class paths. | `onboarding/kms-engineer-first-week.md:21-28`; `tutorials/envelope-encrypt-rotate-and-cryptoshred.md:7-9` | Replace with canonical Rust/OpenTofu/Oya commands; preserve shell only as evidence transcript if needed. |
| P1 | 5 | Union counterpart parity lacks local key CRUD, import, grants, aliases, signing, MAC, random, and EKM/XKS contracts. | `contracts/openapi/cloud/cloud-kms-v1.yaml:9-168`; AWS/GCP/Vault official docs cited in parity report | Expand product surface or explicitly declare non-goals. |
| P1 | 2 | Missing cross-microservice handoff file despite many outbound runbook references. | `runbooks/hsm-cluster-failover.md:250-262`; `runbooks/key-material-quorum-loss.md:250-262` | Add `cross-microservice-handoffs.md` with reciprocal service obligations. |
| P1 | 3 | Local CI/integration evidence missing; external scorecard says integration markers are zero. | `docs/architecture/wave-3-final-scorecard-2026-05-20.md:4763` | Add integration scenario plan and tests linking KMS, IAM, storage, audit-chain, and cloud-iac. |
| P2 | 1 | Benchmark claims measured evidence but referenced evidence directory is absent. | `benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md:3-4`; `:100` | Reclassify as target/provisional or restore immutable evidence. |
| P2 | 1,5 | Counterpart framing includes Azure and Vault Enterprise while this audit top-3 requires HashiCorp Vault self-hosted. | `benchmarks/...md:1`; `migration-playbooks/from-aws-kms-and-vault-enterprise.md:1-4` | Add Google Cloud KMS and self-hosted Vault migration/parity slices; keep Azure as optional fourth. |
| P2 | 4,7 | Terraform AWS provider is named positively in migration playbook. | `migration-playbooks/from-aws-kms-and-vault-enterprise.md:165`; `specs/master-plan-sequencing.json:747-776` | Reword as "legacy source ecosystem" and make OpenTofu the only target IaC engine. |
| P2 | 1 | paid depends on Vault Enterprise HSM seal without self-hosted Vault boundary explanation. | `retired tenant_class adoption artifact:67-83`; Vault seal docs | Clarify when Vault is internal backing substrate vs external counterpart. |
| P2 | 1 | Cross-region "Spanner-class TrueTime" claim lacks deployment-context proof. | `faqs/kms-engineer-faq.md:114-120`; `specs/master-plan-sequencing.json:704-746` | Add clock/causal-order design per context. |
| P2 | 2 | Brief-template reverse reference expects absent runbook/SLO paths. | `docs/standards/brief-template.md:1657-1662` | Either land those files or update template example after Wave 14. |
| P2 | 3 | Cedar authorization is described but local policy files are absent. | `faqs/kms-engineer-faq.md:132-136`; `crates/cloud-kms-api/src/lib.rs:101-106` | Add `policies/` or contract pointer for KMS surfaces and data-class gates. |
| P2 | 5 | Google Cloud KMS migration is absent from local migration playbooks. | `migration-playbooks/from-aws-kms-and-vault-enterprise.md:1-4`; Google Cloud KMS overview docs | Add GCP KMS/HSM/EKM migration plan. |
| P3 | 2 | Repo catalog says `security_review: unreviewed` while docs use high assurance language. | `registry/catalog/cloud-kms-domain.yaml:8-9`; `retired tenant_class adoption artifact:49-83` | Make security review state visible in local docs. |
| P3 | 1 | Runbooks are individually strong but not indexed. | `runbooks/hsm-cluster-failover.md:1-11`; `runbooks/key-material-quorum-loss.md:1-11`; `runbooks/rotation-cadence-drift-detection.md:1-11` | Add `runbooks/README.md` with trigger taxonomy. |

Finding totals:
P0: 0.
P1: 12.
P2: 9.
P3: 2.

## Section 5 - Open Questions for Wave 14 Aggregation

Open question 1: Should `microservices/cloud-kms/` own local copies of contracts and Rust crate pointers, or should the repo standard define a machine-readable microservice ownership manifest that points to `contracts/` and `crates/`?
Open question 2: Should cloud-kms expose full KMS control-plane APIs locally, or should it intentionally remain an authorization/receipt API while another service owns key CRUD?
Open question 3: Is Vault in Oyatie an internal backing substrate, an external counterpart, or both, and how should "Vault Enterprise" docs be reconciled with the audit requirement for self-hosted HashiCorp Vault?
Open question 4: Should Google Cloud KMS become a first-class migration playbook because it is in the top-3 counterpart set?
Open question 5: Does demo_trial mean the same thing across all contexts, or should OCI demo_trial tenant_class be a strict sub-profile with lower capability and explicit Always Free cost ceilings?
Open question 6: Are HSM signing, MAC/HMAC, random generation, and key import in scope for the cloud-kms API, or should they be explicit future tenant_class adoption model?
Open question 7: Which team owns the absent `cloud.kms_key_used.v1` event schema: cloud-kms, audit-chain, or contracts?
Open question 8: Should `cloud-kms` own tenant CA signing for cloud-network mTLS, or should that be a separate certificate-authority microservice?
Open question 9: Is the current repo-level `cloud-kms-domain` security review state acceptable for Phase 0, given registry status is unreviewed?
Open question 10: Should Wave 14 aggregate a cross-service "crypto custody" doctrine joining cloud-kms, cloud-secrets, audit-chain, identity, cloud-iam, cloud-storage, and cloud-data?

## Appendix A - Evidence Ledger for Wave 14

Evidence row 1: local purpose comes from `retired tenant_class adoption artifact:7-11`; it names CMKs, KEKs, DEKs, envelope encryption, rotation, cryptoshredding, HSM signing, and no-roll-your-own-crypto.
Evidence row 2: local demo_trial tenant_class says software libsodium, <=5 CMKs, 90-day rotation, 100/sec sustained, and approximate cost `$40/month` at `retired tenant_class adoption artifact:13-29`.
Evidence row 3: local paid tenant_class says AWS CloudHSM/Marvell LiquidSecurity 2 and FIPS 140-3 L2 at `retired tenant_class adoption artifact:31-47`.
Evidence row 4: local paid tenant_class says Thales Luna 7, AWS CloudHSM, PQC, 12k/sec sustained, and HSM custody at `retired tenant_class adoption artifact:49-65`.
Evidence row 5: local paid tenant_class says dedicated Utimaco, Thales, Vault Enterprise HSM seal, and 200k/sec partition target at `retired tenant_class adoption artifact:67-83`.
Evidence row 6: FAQ declares downstream KMS/HSM providers are adapters, not the canonical policy layer, at `faqs/kms-engineer-faq.md:7-12`.
Evidence row 7: FAQ declares BYOK paths through PKCS#11 import and AWS XKS at `faqs/kms-engineer-faq.md:48-56`.
Evidence row 8: FAQ declares cryptoshredding semantics and destruction receipt behavior at `faqs/kms-engineer-faq.md:60-68`.
Evidence row 9: FAQ declares AAD mandatory in v0.42 at `faqs/kms-engineer-faq.md:72-76`.
Evidence row 10: FAQ declares HSM attestation and certificate chain behavior at `faqs/kms-engineer-faq.md:80-84`.
Evidence row 11: FAQ declares cross-region replication semantics at `faqs/kms-engineer-faq.md:114-120`.
Evidence row 12: FAQ declares cloud-iam integration at `faqs/kms-engineer-faq.md:132-136`.
Evidence row 13: FAQ declares offline HSM bootstrap ceremony at `faqs/kms-engineer-faq.md:157-161`.
Evidence row 14: FAQ declares Vault Enterprise adapter crate intent at `faqs/kms-engineer-faq.md:165-168`.
Evidence row 15: FAQ declares Foundry hook/audit evidence expectations at `faqs/kms-engineer-faq.md:172-176`.
Evidence row 16: onboarding starts with make-based dev cell commands at `onboarding/kms-engineer-first-week.md:21-28`.
Evidence row 17: tutorial starts with make, jq, and openssl prerequisites at `tutorials/envelope-encrypt-rotate-and-cryptoshred.md:7-9`.
Evidence row 18: migration playbook uses AWS CLI and jq inventory at `migration-playbooks/from-aws-kms-and-vault-enterprise.md:11-21`.
Evidence row 19: migration playbook uses Vault CLI inventory at `migration-playbooks/from-aws-kms-and-vault-enterprise.md:27-38`.
Evidence row 20: migration playbook names Terraform AWS provider as mature ecosystem tooling at `migration-playbooks/from-aws-kms-and-vault-enterprise.md:165`.
Evidence row 21: benchmark file claims measured dates at `benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md:3-4`.
Evidence row 22: benchmark file says reproducibility uses make and a `.foundry/evidence` path at `benchmarks/...md:91-100`.
Evidence row 23: reference implementation gives a real Rust dependency block at `reference-implementations/envelope-encrypt-rust-sdk.md:6-23`.
Evidence row 24: reference implementation gives runnable Rust code at `reference-implementations/envelope-encrypt-rust-sdk.md:25-164`.
Evidence row 25: reference implementation uses `cargo run --release` at `reference-implementations/envelope-encrypt-rust-sdk.md:166-170`.
Evidence row 26: reference implementation uses hermetic cargo tests and SoftHSM 2.6.1 at `reference-implementations/envelope-encrypt-rust-sdk.md:196-203`.
Evidence row 27: HSM failover runbook frontmatter establishes incident runbook maturity at `runbooks/hsm-cluster-failover.md:1-11`.
Evidence row 28: HSM failover runbook has cross-service dependencies at `runbooks/hsm-cluster-failover.md:250-262`.
Evidence row 29: quorum-loss runbook has cross-service dependencies at `runbooks/key-material-quorum-loss.md:250-262`.
Evidence row 30: rotation-drift runbook has cross-service dependencies at `runbooks/rotation-cadence-drift-detection.md:248-260`.
Evidence row 31: repo-level OpenAPI title/description define encrypt/decrypt authorization receipt contract at `contracts/openapi/cloud/cloud-kms-v1.yaml:1-12`.
Evidence row 32: repo-level OpenAPI encrypt endpoint is at `contracts/openapi/cloud/cloud-kms-v1.yaml:9-88`.
Evidence row 33: repo-level OpenAPI decrypt endpoint is at `contracts/openapi/cloud/cloud-kms-v1.yaml:89-168`.
Evidence row 34: repo-level OpenAPI request fields and KMS purpose enum are at `contracts/openapi/cloud/cloud-kms-v1.yaml:176-301`.
Evidence row 35: repo-level OpenAPI receipt schema is at `contracts/openapi/cloud/cloud-kms-v1.yaml:316-393`.
Evidence row 36: API crate says it owns tenant/header/path/body normalization at `crates/cloud-kms-api/src/lib.rs:1-4`.
Evidence row 37: API crate exports encrypt/decrypt surfaces at `crates/cloud-kms-api/src/lib.rs:14-15`.
Evidence row 38: API crate status codes are at `crates/cloud-kms-api/src/lib.rs:17-37`.
Evidence row 39: API crate request models are at `crates/cloud-kms-api/src/lib.rs:108-150`.
Evidence row 40: API crate receipt model is at `crates/cloud-kms-api/src/lib.rs:210-225`.
Evidence row 41: domain crate declares it does not perform cryptography or HSM I/O at `crates/cloud-kms-domain/src/lib.rs:1-7`.
Evidence row 42: domain crate key origins include OyatieManaged, BYOK, and HYOK at `crates/cloud-kms-domain/src/lib.rs:71-76`.
Evidence row 43: domain crate HSM validation enum includes KCMVP/FIPS/Common Criteria/PCI at `crates/cloud-kms-domain/src/lib.rs:84-91`.
Evidence row 44: domain crate purpose enum includes storage, workspace, secret provider, replication, and backup at `crates/cloud-kms-domain/src/lib.rs:102-113`.
Evidence row 45: domain crate key create shape is at `crates/cloud-kms-domain/src/lib.rs:121-137`.
Evidence row 46: domain crate destruction receipt shape is at `crates/cloud-kms-domain/src/lib.rs:203-220`.
Evidence row 47: API tests reject path/body key drift at `crates/cloud-kms-api/tests/cloud_kms_api.rs:114-133`.
Evidence row 48: API tests reject header and tenant drift at `crates/cloud-kms-api/tests/cloud_kms_api.rs:135-157`.
Evidence row 49: API tests reject unauthorized same-tenant principal at `crates/cloud-kms-api/tests/cloud_kms_api.rs:159-178`.
Evidence row 50: API tests cover idempotent encrypt replay at `crates/cloud-kms-api/tests/cloud_kms_api.rs:203-232`.
Evidence row 51: API tests cover idempotent decrypt replay at `crates/cloud-kms-api/tests/cloud_kms_api.rs:234-253`.
Evidence row 52: API tests cover reused idempotency key drift at `crates/cloud-kms-api/tests/cloud_kms_api.rs:255-272`.
Evidence row 53: API tests cover duplicate event conflicts at `crates/cloud-kms-api/tests/cloud_kms_api.rs:274-304`.
Evidence row 54: API tests cover malformed AAD fingerprint at `crates/cloud-kms-api/tests/cloud_kms_api.rs:306-331`.
Evidence row 55: API tests cover invalid data class labels at `crates/cloud-kms-api/tests/cloud_kms_api.rs:334-352`.
Evidence row 56: cloud product PRD identifies `cloud-kms-api` at `docs/products/cloud/PRD.md:121`.
Evidence row 57: cloud product PRD expects KMS API p99 <=100 ms at `docs/products/cloud/PRD.md:173`.
Evidence row 58: cloud product PRD expects `cloud.kms_key_used.v1` event schema at `docs/products/cloud/PRD.md:526`.
Evidence row 59: cloud product PRD expects audit emission `oya.audit.cloud_kms_use` at `docs/products/cloud/PRD.md:549`.
Evidence row 60: cloud product PRD names KMS/BYOK/HYOK and indefinite audit retention at `docs/products/cloud/PRD.md:731`.
Evidence row 61: cloud product PRD says HYOK integration evidence is still missing at `docs/products/cloud/PRD.md:779`.
Evidence row 62: runtime binding registry maps encrypt/decrypt functions and tests at `registry/openapi/runtime-bindings.tsv:5-6`.
Evidence row 63: schema binding registry maps eight CloudKms schemas at `registry/openapi/schema-bindings.tsv:21-28`.
Evidence row 64: catalog says security review is unreviewed and supply chain is source-only at `registry/catalog/cloud-kms-domain.yaml:8-9`.
Evidence row 65: master plan lists cloud-kms in Phase 0 shared infrastructure at `specs/master-plan-sequencing.json:403-410`.
Evidence row 66: chat history line 15245 records Wave 2 Batch 2.1 dispatch and cloud-kms PID 9861 with four deliverables expected.
Evidence row 67: chat history line 15231 records the active task reminder for Wave 2 Batch 2.1, eight one-per-microservice audits, and four docs per service.
Evidence row 68: retrospective says BYOK/HYOK confusion is a high-risk drift source at `docs/architecture/wave-3-retrospective-2026-05-20.md:2332`.
Evidence row 69: scorecard says cloud-kms has zero integration markers and cross-service scenario gap at `docs/architecture/wave-3-final-scorecard-2026-05-20.md:4763`.
Evidence row 70: brief-template example expects cloud-kms SLO and runbook README paths that are absent at `docs/standards/brief-template.md:1657-1662`.

## Completion Summary

The audit found cloud-kms has a strong product narrative and useful runbook/tutorial/reference content, but weak ownership coherence.
The path is documentation-only and does not contain the deployability/control surfaces expected by ADR-0328 for a Phase 0 cloud infrastructure microservice.
The highest-priority remediation is not prose polish; it is to land or point to machine-readable ownership artifacts: PRD, architecture, context manifest, OpenTofu modules, supported OS manifest, SLOs, contract ownership, policy ownership, and CI/integration evidence.
Feature parity should be expanded against AWS KMS, Google Cloud KMS, and self-hosted Vault, with Azure retained only as an optional fourth comparison.
The Rust source scan is clean for forbidden local source files, but docs need to stop presenting make/shell/provider CLI paths as canonical build or control paths.

<!-- ORCHESTRATOR REPORT
  microservice: cloud-kms
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/cloud-kms/coherence-audit-2026-05-20.md (609 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-kms/feature-parity-matrix-2026-05-20.md (411 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-kms/performance-benchmark-numbers-2026-05-20.md (314 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-kms/capability-tenant_class-deltas-vs-counterparts-2026-05-20.md (370 lines)
  inventory_files_seen: 10
  inventory_lines_read: 1942
  chat_history_matches_processed: 48
  findings_p0: 0
  findings_p1: 12
  findings_p2: 9
  findings_p3: 2
  top_3_counterparts_confirmed: AWS KMS / Google Cloud KMS / HashiCorp Vault (self-hosted)
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1704
-->
