---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P01
title: Cloud Foundations (KMS / Storage / Network / IAM / Region / Cell)
status: partial (in-flight)
purpose: Lift the in-flight cloud kernels to API+app+adapter+runtime completeness; provider-agnostic by default.
---

# M03-P01 — Cloud Foundations

## Purpose
Continue the in-flight cloud bring-up per [`../../../../../.omx/notepad.md`](../../../../../.omx/notepad.md) 2026-05-11 checkpoints (KMS, storage, network, IAM, region, cell, surface). Bring each up to API+app+adapter+runtime completeness with provider-agnostic interfaces (Directive 4).

## Acceptance
- `cloud.kms.{encrypt,decrypt}`, `cloud.storage.{object,block}.*`, `cloud.network.{vpc,lb,dns,cdn,interconnect,ddos,mesh}.*`, `cloud.iam.{role,sts}.*`, `cloud.region.list`, `cloud.az.list` SPEC §7 rows all green at `stable` tier.
- Cell-routing primitive (`oya-platform-cell-kernel` from M01-P05) integrated.
- Provider adapters: at least 2 of {AWS, OCI, GCP, Azure, NaverCloud, NHN, KT, KakaoCloud} per kernel.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cloud KMS provider-agnostic API + adapter set | partial | [`IP-001-kms-api-adapters.md`](IP-001-kms-api-adapters.md) |
| IP-002 | Cloud Storage object + block API + adapter set | partial | [`IP-002-storage-api-adapters.md`](IP-002-storage-api-adapters.md) |
| IP-003 | Cloud Network VPC + LB + DNS + CDN + interconnect API | stub | [`IP-003-network-api-adapters.md`](IP-003-network-api-adapters.md) |
| IP-004 | Cloud IAM Cedar + SSO + STS API | partial | [`IP-004-iam-cedar-sso-sts.md`](IP-004-iam-cedar-sso-sts.md) |
| IP-005 | Cloud region + AZ + cell taxonomy | partial | [`IP-005-region-az-cell-taxonomy.md`](IP-005-region-az-cell-taxonomy.md) |

## Estimated parallelism
5 agents in parallel (kernel slices already shipped per notepad; each IP is API/app/adapter completion).

## Symbols-touched
`crates/oya-cloud-{kms,storage,network,iam,region,surface,cell}-{api,app,adapter-aws,adapter-oci,adapter-gcp,adapter-azure}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P01 complete: cloud foundations API+app+adapter completion; provider-agnostic across ≥2 providers per kernel" -i critical -k "M03,P01,cloud-foundations,complete"
```
