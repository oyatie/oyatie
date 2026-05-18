---
doc_class: CompetitorParityMatrix
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + ops-product
related_adrs: [ADR-0133]
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/compliance.md
  - microservices/foundry-evidence/threat-model.md
doc_status: published
---

# foundry-evidence — competitor parity matrix

Per ADR-0133, this matrix is **CI-asserted**: every "asserted" row has a CI lane that verifies the claim; "declared-gap" rows are honest acknowledgements of areas not yet at parity. Aspirational claims are forbidden at exit-gate.

## Targets

| Target | What it is | Why we benchmark |
|---|---|---|
| AWS CloudTrail with Audit Lake | AWS's audit-log substrate (CloudTrail) + Audit Manager + Audit Lake (S3 + Lake Formation analytics) | Industry default for AWS-resident workloads; rich regulator-export tooling |
| Google Cloud Audit Logs Premium | GCP's audit-log substrate (Cloud Audit Logs) + Chronicle / Security Operations | Industry default for GCP-resident workloads; correlation depth |
| Azure Sentinel | Azure SIEM with audit-log ingestion + KQL + threat detection | Industry default for Azure-resident; rich threat-analytics |
| Splunk (Enterprise Security) | Per-event search + SIEM | Cross-cloud audit aggregation incumbent |
| LogicMonitor | Hybrid observability + audit | Enterprise multi-cloud incumbent |

## Dimensions

| Dimension | foundry-evidence | AWS CloudTrail+Audit Lake | GCP Audit Logs Premium | Azure Sentinel | Splunk | LogicMonitor | Status |
|---|---|---|---|---|---|---|---|
| Per-invocation evidence pack schema (model + prompt + output hash + autonomy-tier + guardrail + eval) | YES (canonical schema; ADR-0024) | NO (generic event log) | NO (generic event log) | NO (generic SIEM) | NO (generic) | NO | **ahead** |
| Cryptographic per-event signature (Ed25519) | YES (via audit-chain substrate; Bominal ADR-0028) | partial (CloudTrail log file validation, SHA + RSA) | partial (log integrity validation) | NO (Sentinel-side hash, but not per-event PKI) | NO | NO | **asserted** |
| Merkle-tree inclusion proof | YES (audit-chain substrate) | NO (file-level hash chain only) | NO | NO | NO | NO | **asserted** |
| HSM-backed signing keys | YES (audit-chain substrate; OCI Cloud-HSM) | partial (AWS KMS; CloudTrail integration not HSM-default) | partial (Cloud KMS; HSM-tier optional) | partial (Azure Key Vault HSM optional) | N/A | N/A | **asserted** |
| WORM storage (Object Lock Compliance) | YES (via audit-chain substrate) | YES (S3 Object Lock Compliance) | YES (Cloud Storage retention policies; GCS Locked Retention) | YES (Immutable Blob Storage) | partial (depends on store) | NO | **parity** |
| Regulator-export framework profiles (EU AI Act / HIPAA / GDPR / KR PIPA / SOC 2 / ISO 27001) | YES (six profiles; citation-anchored; CI-asserted) | partial (Audit Manager has SOC 2 + HIPAA frameworks; no EU AI Act) | partial (Compliance reports; no EU AI Act dedicated) | partial (Sentinel content for SOC 2 + HIPAA + GDPR; no EU AI Act) | partial (Compliance suite; varies) | NO | **ahead on EU AI Act** |
| Two-person rule on export issuance | YES (Cedar `regulator-export-scope.cedar`) | NO (IAM-policy can simulate; not enforced by service) | NO (IAM-policy; not service-enforced) | NO | NO | NO | **ahead** |
| Per-tenant cryptographic isolation (per-pack chain locality) | YES (Bominal ADR-0028 §"Chain locality") | partial (per-account; not per-tenant within account) | partial | partial | NO | NO | **ahead** |
| ULID + idempotency-key dedup on emit | YES | YES (event_id) | YES (insertId) | YES | YES | YES | **parity** |
| Sub-100ms per-event query (single-tenant) | YES (p99 ≤ 100 ms; CI-asserted) | partial (Lake queries are seconds; CloudTrail Lake fast scan ~minutes) | partial (BigQuery export; seconds) | partial (KQL; seconds) | YES (search; sub-second on hot tier) | partial | **parity (with caveats)** |
| Cross-cloud unified evidence | partial (oyatie-only; not a multi-cloud aggregator) | NO (AWS-only) | NO (GCP-only) | NO (Azure-only) | YES (multi-cloud is the use case) | YES | **declared-gap vs Splunk + LogicMonitor for cross-cloud aggregation** |
| ML-based anomaly detection over evidence stream | NO at M01 (substrate emits to observability for tenant-side downstream use) | YES (Audit Lake + Detective) | YES (Chronicle) | YES (Sentinel UEBA) | YES (Splunk ITSI / UBA) | partial | **declared-gap** |
| Per-event PII redaction + tokenisation | partial (`payload_data_class` gate; tenant supplies hashing; no built-in DLP at M01) | YES (Macie integration) | YES (DLP API integration) | YES (Purview integration) | YES (DLP add-ons) | NO | **declared-gap** |
| Real-time streaming export to tenant SIEM | partial (Workflow events; no direct firehose at M01) | YES (CloudTrail to Kinesis / OpenSearch) | YES (Pub/Sub + Dataflow) | YES (Sentinel integration with external SIEM) | YES (Splunk Connect) | partial | **declared-gap; M02 scheduled** |
| AI-Act Art. 12/18/26 framework profile out-of-box | YES | NO | NO | NO | NO | NO | **ahead** |
| HIPAA §164.312(b) audit-control profile out-of-box | YES | partial (Audit Manager) | partial | partial (Sentinel content) | partial | NO | **parity** |
| Eval-evidence integration (per-invocation eval-verdict frozen) | YES (ADR-0024) | NO | NO | NO | NO | NO | **ahead** |
| Autonomy-tier (T0..T3) decision capture | YES | NO | NO | NO | NO | NO | **ahead** |
| Guardrail-decision per-invocation capture | YES | NO | NO | NO | NO | NO | **ahead** |
| Per-pack regional data residency (11 packs) | YES | partial (per-region; tenant configures) | partial | partial | partial | partial | **ahead** |
| Cross-tenant query refusal (Cedar default-deny) | YES | partial (IAM-based; default-deny by-policy) | partial | partial | depends on customer setup | NO | **parity** |
| Two-channel root publication (S3 + Mimir + GitHub-pinned) | YES (via audit-chain substrate) | NO (CloudTrail log file validation is single-channel) | NO | NO | NO | NO | **ahead** |
| Pre-signed URL TTL ≤ 5 min on plaintext fetch | YES | partial (S3 pre-signed URLs configurable; default 15 min) | partial | partial | varies | varies | **asserted** |
| Self-observability (audit-chain emits SLI to observability µservice; gates own promotion) | YES (ADR-0130 SLO-gated promotion) | partial (CloudWatch metrics + Operational Excellence pillar) | partial | partial | partial | partial | **ahead on tight loop** |

## Declared gaps (honest)

Per ADR-0133:

| Gap | Today (M01) | Plan |
|---|---|---|
| Cross-cloud unified evidence | oyatie-resident only; multi-cloud aggregation not in scope | Tenant-controlled export to tenant SIEM is the supported path; multi-cloud unified evidence aggregator is out-of-scope for foundry-evidence (this is a Splunk/Datadog/LogicMonitor competition area; oyatie's competitive differentiation is on per-agent-invocation depth, not multi-cloud breadth) |
| ML anomaly detection | Not in foundry-evidence M01 | Downstream observability + governance µservices handle this; foundry-evidence emits the stream |
| Built-in DLP / tokenisation | `payload_data_class` gating only | M02 scheduled — pluggable DLP filter at recorder boundary |
| Real-time SIEM firehose | Workflow events bus only | M02 scheduled — Kafka-compatible firehose adapter |

## Asserted dimensions (CI lanes)

| Dimension | CI lane |
|---|---|
| Cryptographic per-event signature | substrate `merkle-verify-drill` (inherited from audit-chain) |
| Merkle-tree inclusion proof | `verification-stack-drill` (inherited) |
| HSM-backed signing keys | `hsm-availability-drill` (inherited) |
| Regulator-export framework profiles | `regulator-profile-drill` |
| Two-person rule | `regulator-export-2pr-drill` |
| Per-pack cryptographic isolation | `cross-pack-replication-forbidden` |
| Per-event query p99 | `load-drill` |
| Pre-signed URL TTL | `cedar-policy-fingerprint-match` |
| Self-observability | `agentic-slo-gated-promotion` per ADR-0130 |

## Differentiation summary

Where we are **ahead**: per-invocation pack schema; AI-Act native; eval-evidence integration; autonomy-tier capture; guardrail-decision capture; two-person rule on export; per-pack chain locality; two-channel root publication; honest claim posture (ADR-0133).

Where we are **at parity**: WORM storage; HSM signing (substrate); idempotency dedup; HIPAA + GDPR profiles; default-deny.

Where we have **declared gaps**: cross-cloud aggregation; ML anomaly; built-in DLP; real-time SIEM firehose.

## Review cadence

- Quarterly review of competitor offerings + dimension list.
- Honest gap updates land via PR + CI assertion that any "asserted" claim is still verified.

## References

- ADR-0133 (industry-best-practice conformance program).
- `microservices/foundry-evidence/compliance.md`.
- Published documentation: AWS CloudTrail + Audit Manager + Audit Lake; Google Cloud Audit Logs + Chronicle; Microsoft Sentinel; Splunk Enterprise Security; LogicMonitor (referenced at the date of this document).
