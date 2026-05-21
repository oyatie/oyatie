---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P01
title: Cloud Foundations (KMS / Storage / Network / IAM / Region / Cell)
status: in-progress (IP-001 adapter-port green; IP-002 OCI object+block plus S3 object request-contract green 2026-05-20; IP-003 OCI network plus self-hosted VPC+DNS request-contract green 2026-05-21; IP-004 Cedar bind plus federated OIDC STS API runtime plus provider-managed IdP CRUD API plus OCI/self-hosted provider-runtime request-contract green 2026-05-21; live-smoke follow-up pending)
purpose: Lift the in-flight cloud kernels to API+app+adapter+runtime completeness; provider-agnostic by default.
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: >
  Merge-variant 2: net-new types are added as modules inside existing live
  oya-cloud-* crates (no new crate scaffolds, no new workspace deps).
  Pattern mirrors F-M02B-PLAN-LIVE-CRATE-RECONCILIATION; M03+ tracking task
  is F-M03-PLAN-LIVE-CRATE-RECONCILIATION (filed 2026-05-17,
  session_id=claude-durable-goal-2026-05-17-m03-p01-agent).
---

# M03-P01 — Cloud Foundations

## Purpose
Continue the in-flight cloud bring-up per [`../../../../../.omx/notepad.md`](../../../../../.omx/notepad.md) 2026-05-11 checkpoints (KMS, storage, network, IAM, region, cell, surface). Bring each up to API+app+adapter+runtime completeness with provider-agnostic interfaces (Directive 4).

## Acceptance
- `cloud.kms.{encrypt,decrypt}`, `cloud.storage.{object,block}.*`, `cloud.network.{vpc,lb,dns,cdn,interconnect,ddos,mesh}.*`, `cloud.iam.{role,sts}.*`, `cloud.region.list`, `cloud.az.list` SPEC §7 rows all green at `stable` tier.
- Cell-routing primitive (`oya-platform-cell-kernel` from M01-P05) integrated.
- Provider adapters: at least 2 provider implementations per kernel, with self-hosted/on-prem/colo counted as a first-class target alongside {AWS, OCI, GCP, Azure, NaverCloud, NHN, KT, KakaoCloud}.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cloud KMS provider-agnostic API + adapter set | adapter-port-request-contract-green; live-smoke pending | [`IP-001-kms-api-adapters.md`](IP-001-kms-api-adapters.md) |
| IP-002 | Cloud Storage object + block API + adapter set | object-and-block-oci-plus-s3-object-request-contract-green; live-smoke pending | [`IP-002-storage-api-adapters.md`](IP-002-storage-api-adapters.md) |
| IP-003 | Cloud Network VPC + LB + DNS + CDN + interconnect API | vpc-dns-second-provider-selfhosted-lb-interconnect-oci-request-contract-green; cdn-remaining-second-provider-live-smoke pending | [`IP-003-network-api-adapters.md`](IP-003-network-api-adapters.md) |
| IP-004 | Cloud IAM Cedar + SSO + STS API | cedar-bind-app-composition-green; federated-oidc-sts-api-runtime-green; provider-managed-idp-crud-api-green; oci-and-selfhosted-provider-runtime-request-contract-green; live-smoke pending | [`IP-004-iam-cedar-sso-sts.md`](IP-004-iam-cedar-sso-sts.md) |
| IP-005 | Cloud region + AZ + cell taxonomy | partial | [`IP-005-region-az-cell-taxonomy.md`](IP-005-region-az-cell-taxonomy.md) |

## Estimated parallelism
5 agents in parallel (kernel slices already shipped per notepad; each IP is API/app/adapter completion).

## Symbols-touched
`crates/oya-cloud-{kms,storage,network,iam,region,surface,cell}-{api,app,adapter-aws,adapter-oci,adapter-gcp,adapter-azure}-*`.

## Adapter ground-truth (2026-05-16)

| Kernel | Status | Live backend(s) |
|---|---|---|
| KMS | `oya-cloud-kms-api` + `oya-cloud-kms-domain` complete; OpenBao + OCI adapter request contracts green; live-provider smoke pending | OpenBao on-prem at `https://kms.oyatie.com` (Shamir 5/3, file storage on ZFS, audit log on `/srv/oyatie/audit-chain/openbao-audit.jsonl`); OCI KMS vault `bitween-default-vault` + AES-256 master key in `cloud` compartment |
| Storage | `oya-cloud-storage-domain` object + block provider ports, `oya-cloud-storage-adapter-oci` object/block request contracts, and `oya-cloud-storage-adapter-s3` object request contract green; live-smoke pending | OCI Object Storage namespace `axdotp9iv3ua` + bucket `oyatie-audit-cold-backup` (Archive tier); OCI Block Volume request-contract scoped to cloud compartment; S3 request-contract scoped to `ap-northeast-2` bucket `oyatie-s3-cold-backup` |
| Network | `oya-cloud-network-{dns,lb,vpc}-api` partial plus `oya-cloud-network-adapter-oci` VPC/VCN, Load Balancer, DNS, and FastConnect request contracts green and `oya-cloud-network-adapter-selfhosted` VPC + DNS request contracts green; CDN, remaining second-provider adapters, and live smoke pending — IP-003 | OCI nonprod VCN (10.0.0.0/16), IGW + NAT GW + Service GW + 3 NSGs + public/private subnets; self-hosted/colo VPC and DNS targets via site/cell/fabric control-plane request contracts |
| IAM | `oya-cloud-iam-{api,app,domain}` partial plus OCI and self-hosted/colo provider-runtime request-contract adapters green; Cedar bind, federated OIDC STS API runtime, and provider-managed IdP create/list/update/delete API are green; live smoke pending — IP-004 | OCI tenancy `bitween` (ap-chuncheon-1) + 4 sub-compartments (foundry / cloud / prod / nonprod) |
| Region/Cell | `oya-cloud-region-api` partial — IP-005 | KR-Chuncheon AD-1 + on-prem KR-Seoul (per ADR-0043) — 2 cells live |

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P01 complete: cloud foundations API+app+adapter completion; provider-agnostic across ≥2 providers per kernel" -i critical -k "M03,P01,cloud-foundations,complete"
```
