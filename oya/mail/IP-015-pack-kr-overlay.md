---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-015-pack-kr-overlay
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + council-privacy + pack-kr-council
acceptance_lanes: [oya-governance-pack-kr-overlay, oya-governance-retention-floor-conformance, oya-governance-residency-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: pack-kr overlay activation

## Intent

Activate the pack-kr overlay for mail: KR-FSS 5y retention floor; KR PIPA Art. 23/28/29 conformance; 전자문서법 (Framework Act on Electronic Documents and Transactions) Art. 5 electronic-document integrity via Ed25519 audit-chain seals; KR-resident KMS mandatory. ISMS-P controls map.

## ChangeSet boundary

Per-pack configuration + per-pack overlays under `microservices/mail/iac/kustomize/overlays/pack-kr/`. Additive edits to existing `microservices/mail/policy/data-residency.md` + `microservices/mail/threat-model.md` + `microservices/mail/dpia.md` + `microservices/mail/compliance.md` + `microservices/mail/multi-region.md`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | base overlay |
| `microservices/mail/iac/kustomize/overlays/pack-kr/retention-floor-5y.yaml` | create | retention-policy ConfigMap override |
| `microservices/mail/iac/kustomize/overlays/pack-kr/kms-region-kr.yaml` | create | KMS endpoint = OCI KR (ap-seoul-1) |
| `microservices/mail/iac/kustomize/overlays/pack-kr/isms-p-control-map.yaml` | create | ISMS-P control IDs ↔ mail BC mapping |
| `microservices/mail/threat-model.md` | additive | append "Pack-kr overlay" section |
| `microservices/mail/dpia.md` | additive | append KR PIPA Art. 23/28/29 overlay |
| `microservices/mail/compliance.md` | additive | append ISMS-P + 전자문서법 mapping |
| `microservices/mail/multi-region.md` | additive | append pack-kr region + DR pair |

## Pack-kr overlay scope

| Control | Source | Implementation |
|---|---|---|
| KR-FSS 5y retention floor | KR Commercial Code Art. 33 | `retention-policy-domain::statutory_floor(PackId::KrFss, _) = 5y` |
| KR PIPA Art. 23 sensitive PII | PIPA Art. 23 | Cedar policy forbid: `forbid (...) when {resource has sensitive_pipa_art23 && !principal.has_pipa_art23_clearance}` |
| KR PIPA Art. 28 secure storage | PIPA Art. 28 | Tenant DEK in KR-resident KMS; cross-region replication forbidden |
| KR PIPA Art. 29 access logs | PIPA Art. 29 | Audit-chain seal every read of PIPA-Art23 data; 1y minimum retention |
| KR PIPA Art. 22-2 special protections | PIPA Art. 22-2 | 2-person rule for sensitive PII access escalation |
| 전자문서법 Art. 5 integrity | Framework Act | Ed25519 audit-chain seal on every Send + Receive |
| KR-PIPA notification 72h | PIPA Art. 34 | breach-notification template in `incident-response.md` |
| ISMS-P 11.3.5 mail security | ISMS-P | DKIM mandatory; SPF mandatory; MTA-STS published per tenant |
| KR-FSS supervisory access (regulated tenants only) | FSS guideline | dedicated audit endpoint with FSS-recognised cert |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate pack-kr-overlay --microservice mail
cargo run -p oya-dev-cli -- gate validate retention-floor-conformance --microservice mail
cargo run -p oya-dev-cli -- gate validate residency-conformance --microservice mail
```

## Test Plan

- KR-FSS-flagged tenant: 5y retention floor applies even if tenant configures 1y.
- KMS endpoint resolution: KR-pack tenant's KMS calls resolve to KR-resident endpoint only.
- Cross-region replication: forbidden; CI lane + runtime check.
- Audit retention: 1y minimum; 5y for KR-FSS tenant.
- Sensitive PII access: Cedar deny without art23_clearance.
- KR-PIPA Art. 34 notification template renders correctly.
- ISMS-P control map: every control maps to a verifiable mail BC implementation.

## Halt Conditions

- KR-FSS tenant retention < 5y → fail (regulatory critical).
- KMS endpoint resolves outside KR for pack-kr tenant → fail.

## Per-pack overlay rollout (subsequent packs)

This IP delivers pack-kr. Subsequent IPs (M03-onward1 onward) deliver:
- pack-eu (GDPR + ePrivacy + DORA)
- pack-us (CCPA + CAN-SPAM + SOC 2)
- pack-us-healthcare (HIPAA + BAA + 6y retention)
- pack-jp (APPI + 個人情報保護法)
- pack-sg (PDPA SG)
- pack-au (Privacy Act 1988 + APP)
- pack-in (DPDPA 2023)
- pack-br (LGPD)
- pack-ae (UAE PDPL)
- pack-ksa (KSA PDPL)

Each follows the same shape: kustomize overlay + additive edits to threat-model/dpia/compliance/multi-region.

## References

- KR Personal Information Protection Act (PIPA) Arts. 15, 22-2, 23, 28, 29, 34
- KR PIPA Enforcement Decree Art. 30 (retention defaults)
- KR Commercial Code Art. 33 (5y retention for regulated comms)
- 전자문서법 (Framework Act on Electronic Documents and Transactions) Art. 5
- ISMS-P (Korea Information Security Management System – Personal Information; 정보보호 및 개인정보보호 관리체계 인증)
- KR Financial Services Commission (FSS) supervisory guidelines
- ADR-0117 (residency)
- Naver Works Mail KR-FSS compliance reference

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-015-pack-kr-overlay.md` matched `multi-region`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
