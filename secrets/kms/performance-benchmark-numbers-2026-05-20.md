# cloud-kms Performance Benchmark Numbers - 2026-05-20

doc_class: target-performance-benchmark-report
microservice: cloud-kms
status: landed-targets-not-measured
date: 2026-05-20

## Citation Anchor Block

1. Canonical audit and benchmark disclosure rules: `docs/decisions/ADR-0700-ci-admission-live-apex.md:3756-4153`.
2. Machine-readable deployment/context constraints: `specs/master-plan-sequencing.json:704-868`.
3. Local microservice benchmark source: `secrets/kms/benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md:1-100`.
4. Local tenant_class target source: `microservices/cloud-kms/retired tenant_class adoption artifact:13-83`.
5. Runtime contract/code source: `contracts/openapi/cloud/cloud-kms-v1.yaml:1-168`, `crates/oya-cloud-kms-api/tests/cloud_kms_api.rs:203-352`.

## Methodology Disclosure

These are target numbers and public counterpart quota/provenance numbers.
They are not measured Oyatie benchmark results.
Measured benchmarks will be added in build phase per ADR-0212 and the benchmark evidence path must be restored or regenerated.
The local benchmark file claims measured dates at `benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md:3-4`.
The local benchmark file references evidence under `.foundry/evidence/benchmarks/cloud-kms/2026-05-13T16:42:18Z/` at line 100.
That evidence directory was not present in the current workspace inspection.
Therefore this document intentionally does not repeat local "measured" claims as verified facts.
Counterpart numbers are public service quotas or official operational limits where available.
Vault self-hosted numbers are target planning numbers because HashiCorp does not publish one universal transit throughput figure that applies to all HSMs, storage backends, network paths, and seal configurations.
Oyatie numbers below are design targets for demo_trial/paid tenant_class by context.
Targets are intentionally stricter than the current local artifact evidence can prove.
The stop condition for this deliverable is a target benchmark envelope ready for Wave 14 aggregation, not runtime proof.

## Section 1 - Methodology

Benchmark dimension 01: encrypt authorization receipt latency p50.
Benchmark dimension 02: encrypt authorization receipt latency p95.
Benchmark dimension 03: encrypt authorization receipt latency p99.
Benchmark dimension 04: decrypt authorization receipt latency p50.
Benchmark dimension 05: decrypt authorization receipt latency p95.
Benchmark dimension 06: decrypt authorization receipt latency p99.
Benchmark dimension 07: DEK issuance equivalent throughput.
Benchmark dimension 08: sign authorization equivalent throughput.
Benchmark dimension 09: key create/control-plane write throughput.
Benchmark dimension 10: key metadata read throughput.
Benchmark dimension 11: HSM-backed symmetric operation ceiling.
Benchmark dimension 12: external/HYOK operation ceiling.
Benchmark dimension 13: concurrent tenant ceiling.
Benchmark dimension 14: concurrent key version ceiling.
Benchmark dimension 15: rotation job p95 completion.
Benchmark dimension 16: cryptoshred proof p95 completion.
Benchmark dimension 17: audit receipt append p95.
Benchmark dimension 18: fail-closed denial p95.
Benchmark dimension 19: idempotency replay p95.
Benchmark dimension 20: cross-region receipt replication lag p95.
Workload 01: 1 KB envelope encrypt authorization with tenant, key, purpose, data class, AAD fingerprint, and idempotency key.
Workload 02: 1 KB decrypt authorization with tenant, key, purpose, data class, actor, and idempotency key.
Workload 03: 32-byte DEK issuance equivalent, receipt-only path, no bulk data encryption inside KMS.
Workload 04: asymmetric sign authorization receipt, HSM path, 256-byte digest input.
Workload 05: key create with policy attachment, residency binding, HSM partition selection, and audit receipt.
Workload 06: key rotation scheduling for 1,000 tenant keys.
Workload 07: cryptoshred proof for a single tenant CMK and dependent object references.
Workload 08: idempotent replay of prior encrypt authorization receipt.
Workload 09: fail-closed denial for unauthorized principal.
Workload 10: regional control-plane read of key metadata.
OS/arch disclosure: Tier-1 targets must include x86_64 and aarch64 once supported OS manifest lands.
OS/arch disclosure: ppc64le and s390x are target-only test lanes until a manifest declares them.
OS/arch disclosure: macOS M5+ is target-only for developer/Secure Enclave signing, not assumed server runtime.
Deployment disclosure: six contexts are evaluated: oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider.
Tenant class disclosure: demo_trial is small tenant or dev/test; paid is paid baseline; paid is production regulated tenant; paid is hyperscaler/single-tenant capable.
OCI disclosure: demo_trial in guest-on-OCI must fit Always Free when feature set is reduced to software/OCI-free resources.
Not measured disclosure: every Oyatie target below needs a future benchmark run with OS, arch, context, tenant class, HSM backing, and source revision.

## Section 2 - Counterpart Numbers

AWS-01: Symmetric cryptographic operation quota is 100,000 requests/sec in us-east-1/us-west-2/eu-west-1 per AWS general quota docs.
AWS-02: Symmetric cryptographic operation quota is 20,000 requests/sec in several large regions such as us-east-2 and eu-central-1 per AWS docs.
AWS-03: Symmetric cryptographic operation quota is 10,000 requests/sec in other supported regions per AWS docs.
AWS-04: RSA cryptographic operation request rate is 1,000/sec per supported region per AWS docs.
AWS-05: ML-DSA cryptographic operation request rate is 1,000/sec per supported region per AWS docs.
AWS-06: Custom key store request quota is 1,800 requests/sec per CloudHSM key store per AWS KMS request quota docs.
AWS-07: External key store request quota is 1,800 requests/sec per external key store per AWS KMS request quota docs.
AWS-08: Custom key stores quota is 10 per account per region per AWS general quota docs.
AWS-09: Customer managed KMS keys quota is 100,000 per account per region per AWS general quota docs.
AWS-10: DescribeKey quota is 2,000 requests/sec per supported region per AWS docs.
AWS-11: GetPublicKey quota is 2,000 requests/sec per supported region per AWS docs.
AWS-12: EnableKey and DisableKey quotas are 5 requests/sec per supported region per AWS docs.
AWS-13: EnableKeyRotation quota is 15 requests/sec per supported region per AWS docs.
AWS-14: ReplicateKey quota is 5 requests/sec per supported region per AWS docs.
AWS-15: GenerateDataKeyPair quota ranges from 100/sec for ECC P-256/P-384/P-521/P-256K1 to 1/sec for RSA-4096 per AWS docs.

GCP-01: Cloud KMS software usage quota after 2026-02-16 is 6,000,000 tokens/minute per project, soft enforcement.
GCP-02: Cloud KMS HSM usage quota after 2026-02-16 is 3,000,000 tokens/minute per project, soft enforcement.
GCP-03: Cloud KMS external KMS usage quota after 2026-02-16 is 10,000 transactions/sec, hard enforcement.
GCP-04: Read usage quota after 2026-02-16 is 600 tokens/minute.
GCP-05: Write usage quota after 2026-02-16 is 100 tokens/minute.
GCP-06: Software cryptographic operations consume 100 software tokens per operation.
GCP-07: External KMS cryptographic operations consume 100 external KMS tokens per operation.
GCP-08: Cloud HSM symmetric encryption/decryption, MAC, and getPublicKey consume 100 HSM tokens per operation.
GCP-09: Cloud HSM generateRandomBytes consumes 1,000 HSM tokens per operation.
GCP-10: Cloud HSM RSA-2048 asymmetric sign/decrypt consumes 1,500 HSM tokens per operation.
GCP-11: Cloud HSM RSA-3072 asymmetric sign/decrypt consumes 3,500 HSM tokens per operation.
GCP-12: Cloud HSM EC P-224/P-256/P-256K1 sign consumes 4,500 HSM tokens per operation.
GCP-13: Cloud HSM EC P-384/P-521 sign consumes 7,000 HSM tokens per operation.
GCP-14: Cloud HSM RSA-4096 sign/decrypt consumes 14,000 HSM tokens per operation.
GCP-15: Before 2026-02-16, cryptographic request quota was 60,000 QPM, HSM symmetric was 500 QPS, HSM asymmetric was 50 QPS, HSM random was 50 QPS, and external KMS was 100 QPS.

Vault-01: Vault transit HTTP API default maximum request size is 32 MB unless tuned per listener block.
Vault-02: Vault AES-GCM NIST rotation guidance recommends rotation before about 2^32 encryptions per key version.
Vault-03: Vault default Shamir split is 5 shares in the documented conceptual model.
Vault-04: Vault supports online rekey/rotate in HA deployments.
Vault-05: Vault encryption count persistence has a 1 second timeout by default, tunable with `VAULT_ENCRYPTION_COUNT_PERSIST_TIMEOUT`.
Vault-06: Vault Enterprise performance standby nodes scale read operations.
Vault-07: Vault Enterprise performance replication scales workloads horizontally across secondaries for local reads and selected operations.
Vault-08: Vault Enterprise DR secondaries do not serve read/write requests until promoted.
Vault-09: Vault replication uses mutually authenticated TLS for primary/secondary cluster traffic.
Vault-10: Vault HSM/cloud auto-unseal shifts unseal dependency to KMS/HSM availability.
Vault-11: Vault self-hosted transit throughput is substrate-dependent; this audit uses target classes rather than false universal numbers.
Vault-12: Vault self-hosted p99 latency targets should be measured per storage backend, seal method, HSM, network RTT, and audit-device configuration.

## Section 3 - Oyatie Target Numbers by Tier and Context

### demo_trial - oyatie-public-cloud

demo_trial public target 01: encrypt auth receipt p50 <= 12 ms.
demo_trial public target 02: encrypt auth receipt p95 <= 18 ms, matching local tenant_class target.
demo_trial public target 03: encrypt auth receipt p99 <= 35 ms.
demo_trial public target 04: decrypt auth receipt p50 <= 13 ms.
demo_trial public target 05: decrypt auth receipt p95 <= 20 ms.
demo_trial public target 06: decrypt auth receipt p99 <= 40 ms.
demo_trial public target 07: sustained DEK-equivalent issuance >= 100/sec per tenant.
demo_trial public target 08: burst DEK-equivalent issuance >= 500/sec per tenant for 60 seconds.
demo_trial public target 09: sign auth p95 <= 22 ms for software-only signatures where enabled.
demo_trial public target 10: key create p95 <= 750 ms for software-backed CMK.
demo_trial public target 11: rotation job for 100 keys p95 <= 30 minutes.
demo_trial public target 12: cryptoshred proof p95 <= 24 hours.

### demo_trial - guest-on-aws

demo_trial AWS target 01: encrypt p95 <= 22 ms using AWS KMS standard key backing.
demo_trial AWS target 02: decrypt p95 <= 24 ms using AWS KMS standard key backing.
demo_trial AWS target 03: sustained ops >= 100/sec per tenant, capped below account quota.
demo_trial AWS target 04: burst ops >= 500/sec, with account quota guardrail.
demo_trial AWS target 05: custom key store is not required in demo_trial.
demo_trial AWS target 06: XKS is not required in demo_trial.
demo_trial AWS target 07: key create p95 <= 1,000 ms.
demo_trial AWS target 08: key metadata read p95 <= 150 ms.
demo_trial AWS target 09: idempotency replay p95 <= 10 ms from local ledger.
demo_trial AWS target 10: fail-closed denial p95 <= 15 ms.
demo_trial AWS target 11: audit receipt append p95 <= 20 ms.
demo_trial AWS target 12: cryptoshred proof p95 <= 24 hours.

### demo_trial - guest-on-oci

demo_trial OCI target 01: must fit OCI Always Free profile.
demo_trial OCI target 02: Ampere budget <= 4 OCPU total for KMS plus local dependencies.
demo_trial OCI target 03: memory budget <= 24 GB total for KMS plus local dependencies.
demo_trial OCI target 04: encrypt p95 <= 30 ms for software/OCI-free backing.
demo_trial OCI target 05: decrypt p95 <= 32 ms for software/OCI-free backing.
demo_trial OCI target 06: sustained ops >= 50/sec per tenant under Always Free envelope.
demo_trial OCI target 07: burst ops >= 200/sec for 30 seconds.
demo_trial OCI target 08: key create p95 <= 1,500 ms.
demo_trial OCI target 09: rotation job for 100 keys p95 <= 60 minutes.
demo_trial OCI target 10: cryptoshred proof p95 <= 24 hours.
demo_trial OCI target 11: monthly incremental infrastructure cost target = 0 USD.
demo_trial OCI target 12: paid HSM, dedicated Vault, and external egress-heavy replication require paid tenant_class.

### demo_trial - on-prem

demo_trial on-prem target 01: encrypt p95 <= 25 ms with SoftHSM or TPM-only backing.
demo_trial on-prem target 02: decrypt p95 <= 28 ms.
demo_trial on-prem target 03: sustained ops >= 75/sec per tenant.
demo_trial on-prem target 04: burst ops >= 300/sec.
demo_trial on-prem target 05: key create p95 <= 2,000 ms.
demo_trial on-prem target 06: metadata read p95 <= 200 ms.
demo_trial on-prem target 07: fail-closed denial p95 <= 20 ms.
demo_trial on-prem target 08: idempotency replay p95 <= 10 ms.
demo_trial on-prem target 09: rotation job for 100 keys p95 <= 60 minutes.
demo_trial on-prem target 10: cryptoshred proof p95 <= 24 hours.
demo_trial on-prem target 11: operator quorum ceremony <= 4 hours.
demo_trial on-prem target 12: HSM attestation is optional at demo_trial.

### demo_trial - colo

demo_trial colo target 01: encrypt p95 <= 25 ms.
demo_trial colo target 02: decrypt p95 <= 28 ms.
demo_trial colo target 03: sustained ops >= 75/sec per tenant.
demo_trial colo target 04: burst ops >= 300/sec.
demo_trial colo target 05: cross-rack HSM dependency not required.
demo_trial colo target 06: key create p95 <= 2,000 ms.
demo_trial colo target 07: audit receipt append p95 <= 25 ms.
demo_trial colo target 08: fail-closed denial p95 <= 20 ms.
demo_trial colo target 09: rotation p95 for 100 keys <= 60 minutes.
demo_trial colo target 10: cryptoshred proof p95 <= 24 hours.
demo_trial colo target 11: remote-hands evidence capture <= 4 hours when needed.
demo_trial colo target 12: standby promotion RTO <= 2 hours.

### demo_trial - oyatie-as-cloud-provider

demo_trial provider target 01: encrypt p95 <= 18 ms.
demo_trial provider target 02: decrypt p95 <= 20 ms.
demo_trial provider target 03: sustained ops >= 100/sec per tenant.
demo_trial provider target 04: burst ops >= 500/sec.
demo_trial provider target 05: provider KMS API should not wrap AWS/GCP as canonical path.
demo_trial provider target 06: key create p95 <= 750 ms.
demo_trial provider target 07: metadata read p95 <= 100 ms.
demo_trial provider target 08: audit receipt append p95 <= 20 ms.
demo_trial provider target 09: fail-closed denial p95 <= 15 ms.
demo_trial provider target 10: idempotency replay p95 <= 8 ms.
demo_trial provider target 11: rotation job for 100 keys p95 <= 30 minutes.
demo_trial provider target 12: cryptoshred proof p95 <= 24 hours.

### paid - all contexts baseline

paid target 01: encrypt authorization receipt p50 <= 5 ms.
paid target 02: encrypt authorization receipt p95 <= 8 ms, matching local tenant_class target.
paid target 03: encrypt authorization receipt p99 <= 18 ms.
paid target 04: decrypt authorization receipt p50 <= 6 ms.
paid target 05: decrypt authorization receipt p95 <= 10 ms.
paid target 06: decrypt authorization receipt p99 <= 22 ms.
paid target 07: sustained DEK-equivalent issuance >= 1,500/sec per tenant.
paid target 08: burst DEK-equivalent issuance >= 6,000/sec per tenant.
paid target 09: sign auth p95 <= 12 ms.
paid target 10: key create p95 <= 500 ms.
paid target 11: rotation job for 1,000 keys p95 <= 30 minutes.
paid target 12: cryptoshred proof p95 <= 30 minutes.
paid target 13: audit receipt append p95 <= 8 ms.
paid target 14: fail-closed denial p95 <= 8 ms.
paid target 15: idempotency replay p95 <= 5 ms.

### paid - all contexts baseline

paid target 01: encrypt authorization receipt p50 <= 2 ms.
paid target 02: encrypt authorization receipt p95 <= 4 ms, matching local tenant_class target.
paid target 03: encrypt authorization receipt p99 <= 9 ms.
paid target 04: decrypt authorization receipt p50 <= 2.5 ms.
paid target 05: decrypt authorization receipt p95 <= 5 ms.
paid target 06: decrypt authorization receipt p99 <= 12 ms.
paid target 07: sustained DEK-equivalent issuance >= 12,000/sec per tenant.
paid target 08: burst DEK-equivalent issuance >= 50,000/sec per tenant.
paid target 09: sign auth p95 <= 6 ms.
paid target 10: key create p95 <= 300 ms.
paid target 11: rotation job for 10,000 keys p95 <= 5 minutes per shard.
paid target 12: cryptoshred proof p95 <= 5 minutes.
paid target 13: audit receipt append p95 <= 4 ms.
paid target 14: fail-closed denial p95 <= 5 ms.
paid target 15: cross-region receipt replication lag p95 <= 3 seconds.

### paid - all contexts baseline

paid target 01: encrypt authorization receipt p50 <= 1 ms.
paid target 02: encrypt authorization receipt p95 <= 2 ms, matching local tenant_class target.
paid target 03: encrypt authorization receipt p99 <= 5 ms.
paid target 04: decrypt authorization receipt p50 <= 1.2 ms.
paid target 05: decrypt authorization receipt p95 <= 2.5 ms.
paid target 06: decrypt authorization receipt p99 <= 6 ms.
paid target 07: sustained DEK-equivalent issuance >= 200,000/sec per partition.
paid target 08: burst DEK-equivalent issuance >= 500,000/sec per partition for 60 seconds.
paid target 09: sign auth p95 <= 3 ms.
paid target 10: key create p95 <= 150 ms.
paid target 11: rotation job for 100,000 keys p95 <= 5 minutes per partition.
paid target 12: cryptoshred proof p95 <= 60 seconds.
paid target 13: audit receipt append p95 <= 2 ms.
paid target 14: fail-closed denial p95 <= 3 ms.
paid target 15: cross-region receipt replication lag p95 <= 1 second.

## Section 4 - Per-Context Overlay

Overlay public-cloud 01: can use Oyatie-controlled cells and HSM pools; target numbers should match baseline.
Overlay public-cloud 02: paid can use dedicated partitions per tenant.
Overlay public-cloud 03: cross-region replication is in-scope with Oyatie-controlled regions.
Overlay AWS 01: demo_trial/paid may use AWS KMS standard keys and remain under AWS account quotas.
Overlay AWS 02: paid may require AWS CloudHSM custom key store and is capped by the 1,800/sec custom-store quota unless sharded.
Overlay AWS 03: paid should use dedicated CloudHSM/XKS topology or Oyatie-owned HSMs, with RTT target <=35 ms for XKS-like paths.
Overlay OCI 01: demo_trial is Always Free and therefore lower than generic demo_trial for throughput.
Overlay OCI 02: paid tenant_class may use paid OCI Vault/HSM resources and should no longer be called Always Free.
Overlay OCI 03: OCI demo_trial tenant_class should disable external replication that would exceed free egress.
Overlay on-prem 01: HSM vendor, TPM, SoftHSM, storage backend, and network locality dominate latency.
Overlay on-prem 02: p95 targets should be accepted only after local hardware attestation and audit-device measurement.
Overlay on-prem 03: paid requires dedicated HSM cluster and local audit chain sink.
Overlay colo 01: remote-hands and rack topology affect RTO more than crypto p50.
Overlay colo 02: cross-rack partition quorum must be tested before claiming paid tenant_class.
Overlay colo 03: paid requires dual-vendor HSM or explicit single-vendor risk acceptance.
Overlay provider 01: Oyatie-as-cloud-provider must not simply forward to AWS/GCP as product implementation.
Overlay provider 02: public API quotas should be explicit and may differ from internal HSM partition quotas.
Overlay provider 03: target numbers should become external SLOs only after OpenSLO and benchmark evidence exist.

## Section 5 - Comparison Narrative

Comparison 01: AWS standard symmetric quota of 100,000/sec in several regions is higher than Oyatie paid sustained target but lower than Oyatie paid per-partition target.
Comparison 02: AWS custom key store quota of 1,800/sec is close to Oyatie paid sustained target and well below paid/paid, so paid tenant_class requires partition sharding or non-AWS-HSM topology.
Comparison 03: AWS external key store quota of 1,800/sec makes HYOK throughput a paid-class default unless sharded.
Comparison 04: Google post-2026 software quota of 6,000,000 tokens/minute with 100 tokens/op is roughly 1,000 software crypto ops/sec before soft-overquota behavior, so high-throughput designs must request/earn quota or use service-integrated CMEK paths.
Comparison 05: Google HSM quota of 3,000,000 tokens/minute with 100 tokens/op is roughly 500 HSM symmetric ops/sec baseline before soft behavior, below Oyatie paid target unless quotas adjust or single-tenant HSM is used.
Comparison 06: Google external KMS 10,000 TPS with 100 tokens/op gives roughly 100 external crypto ops/sec equivalent under the token model, a demo_trial/paid external-key baseline unless adjusted by implementation specifics.
Comparison 07: Vault self-hosted can exceed or undershoot all targets depending on HSM, storage, audit devices, and replication; therefore Oyatie must measure per context.
Comparison 08: Oyatie demo_trial public-cloud target is intentionally modest and should be reachable with software custody but not enough for regulated production.
Comparison 09: Oyatie OCI demo_trial tenant_class target is lower because zero-cost is a hard constraint.
Comparison 10: Oyatie paid targets aim for paid baseline parity with managed KMS under moderate load.
Comparison 11: Oyatie paid targets aim to beat default custom-key-store throughput by sharding across partitions.
Comparison 12: Oyatie paid targets are hyperscaler aspirations and must not be claimed until measured.
Comparison 13: The current local tenant_class matrix latency numbers are aggressive but not supported by local benchmark evidence.
Comparison 14: The current local benchmark file should be converted into a measured-evidence report only after the evidence directory exists.
Comparison 15: Until SLO files exist, all latency and availability values remain target claims, not enforceable service objectives.

## Verification Requirements for Future Measured Report

Requirement 01: record source revision and build command.
Requirement 02: record OS, kernel, architecture, package/container image, and HSM provider.
Requirement 03: record deployment context and tenant class.
Requirement 04: record whether backing is software, TPM, SoftHSM, CloudHSM, OCI Vault, EKM/XKS, Thales, Utimaco, or Vault.
Requirement 05: record whether audit-chain append is synchronous or asynchronous.
Requirement 06: record whether cloud-iam/Cedar authorization is in the hot path.
Requirement 07: record whether receipt persistence is local, replicated, or cross-region.
Requirement 08: record p50, p95, p99, p99.9, error rate, saturation, and throttling.
Requirement 09: record warmup period, run duration, concurrency, request payload sizes, and key distribution.
Requirement 10: publish raw output under immutable evidence path and link from the benchmark doc.
