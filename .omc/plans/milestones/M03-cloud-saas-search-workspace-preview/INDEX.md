---
purpose: Stand up four axis previews in parallel on the M01 foundation + M02 Foundry substrate so vertical pilots can run.
---

---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M03
title: Cloud + SaaS + Search + Workspace Preview (4-way parallel)
wave: W-Cloud-Preview ∥ W-SaaS-Preview ∥ W-Search-Preview ∥ W-Workspace-Preview
status: partial-fanout-complete
owner: axis-cloud + axis-saas + axis-search + axis-workspace
purpose: Stand up four axis previews in parallel on the M01 foundation + M02 Foundry substrate so vertical pilots can run.
acceptance_authority: docs/ROADMAP.md §2.3, §2.4, §2.5, plus §2-Axis-2 (workspace) gate criteria
---

# M03 — Cloud + SaaS + Search + Workspace Preview

## Purpose
Run all four axis previews concurrently to compress build-out time. Each axis preview sits on the M01 foundation and consumes M02 Foundry capabilities for AI assists, gates, and evidence emission. Cloud is the substrate the other three run on; cross-axis cohesion is enforced per [`docs/DESIGN.md`](../../../docs/DESIGN.md) §10.

## Status
**gated on M02.** Once M02 acceptance gate passes, M03 unlocks all eight phases simultaneously.

## Scope
Eight phases:
- Cloud previews (4 phases): foundations (KMS/storage/network/IAM/region/cell), compute (VM/K8s/Functions), data + billing + observability + FinOps, marketplace + capacity + DC-ops.
- Cross-axis (4 phases): SaaS platform preview, Search preview (KR/JP/EN morphology + pgvector + RAG endpoint), Workspace 14-surface preview, regional-pack onboarding (≥ KR + 1 of {JP, US, EU}).

## Dependencies
- **Hard:** M01, M02 acceptance gates passed.
- **Soft:** M-CC-P05 provider-agnosticism active (Cloud kernels MUST use provider-neutral interfaces).
- **Soft:** M-CC-P06 distroless+LTS+image-discipline lanes active.

## Acceptance gate
- Cloud: per [`docs/ROADMAP.md`](../../../docs/ROADMAP.md) §2.3 — IAM (Cedar+SSO+STS), region/AZ/cell taxonomy, compute (k8s+functions), storage (object+block+KMS-shred), network (VPC+LB+DNS+interconnect), billing (per-resource metering + per-region tax-invoice), observability, Cloud control-plane API frozen v1, ≥ 2 regional packs onboarded.
- SaaS: per [`docs/ROADMAP.md`](../../../docs/ROADMAP.md) §2.4 — workflow engine, Object Graph property tiers, plugin substrate (signing + sandbox), public REST API stability tier, webhook signing, plugin marketplace catalog.
- Search: per [`docs/ROADMAP.md`](../../../docs/ROADMAP.md) §2.5 — pgroonga day-1, KR morphology (mecab-ko/khaiii), inverted index sharding, vector index (pgvector), tenant-private indexes, RAG endpoint to Foundry, per-class data boundary enforcement.
- Workspace: 14 [`docs/SPEC.md`](../../../docs/SPEC.md) §4 rows at `stable` tier (preview for translate).

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Cloud Foundations (KMS / Storage / Network / IAM / Region / Cell) | partial (in-flight per [`../../../.omx/notepad.md`](../../../.omx/notepad.md)) | [`phases/P01-cloud-foundations/INDEX.md`](phases/P01-cloud-foundations/INDEX.md) |
| P02 | Cloud Compute (VM / K8s / Functions / Capacity / DC-Ops) | partial (in-flight) | [`phases/P02-cloud-compute/INDEX.md`](phases/P02-cloud-compute/INDEX.md) |
| P03 | Cloud Data + Billing + Observability + FinOps + Marketplace | partial (in-flight) | [`phases/P03-cloud-data-billing-observability/INDEX.md`](phases/P03-cloud-data-billing-observability/INDEX.md) |
| P04 | SaaS Platform Preview (workflow engine, plugin substrate, marketplace) | stub | [`phases/P04-saas-platform-preview/INDEX.md`](phases/P04-saas-platform-preview/INDEX.md) |
| P05 | Search Preview (pgroonga + morphology + pgvector + RAG endpoint) | stub | [`phases/P05-search-preview/INDEX.md`](phases/P05-search-preview/INDEX.md) |
| P06 | Workspace Axis (Mail / Calendar / Drive / Meet / Chat / + 9 more) | stub | [`phases/P06-workspace-14-surfaces/INDEX.md`](phases/P06-workspace-14-surfaces/INDEX.md) |
| P07 | Regional Pack Onboarding (KR + one of JP/US/EU) | stub | [`phases/P07-regional-pack-onboarding/INDEX.md`](phases/P07-regional-pack-onboarding/INDEX.md) |
| P08 | Cross-Axis Contract Registry + Fitness Lanes | stub | [`phases/P08-cross-axis-contracts/INDEX.md`](phases/P08-cross-axis-contracts/INDEX.md) |

## Parallelism strategy
P01 + P02 + P03 (Cloud track) run as 3-way parallel inside the cloud team; they consume M01/M02 only. P04 + P05 + P06 (other-axis track) run as 3-way parallel; they consume Cloud as substrate but only need P01 ≥ 50% (cell + IAM + KMS + storage) to start. P07 (regional packs) runs once P01 IAM + region/AZ stable. P08 (cross-axis contracts) starts day-one and lands incrementally as cross-axis seams emerge. Target: 5-8 agents in parallel across the whole milestone.

## Hyperscaler practices adopted
- AWS Working-Backwards PRFAQ for each axis-preview launch.
- Google Design Doc per phase.
- Postmortem-blameless on any cross-axis contract violation.
- Microsoft 1ES CI templates per axis.
- Oracle Engineering-Excellence-Council merge gate for cross-axis contract changes (per [`/specs/cross-cutting/decision-rights.json`](../../../specs/cross-cutting/decision-rights.json) "Cross-axis contract: Council-Architecture").
- Rust toolchain gates inherited.

## Agent-navigability-pointer
Cloud track first-claim seed: continue the in-flight cloud kernel sequence per [`../../../.omx/notepad.md`](../../../.omx/notepad.md) latest checkpoint — next slice is cloud network LB API or VPC API. Workspace track first-claim seed: `crates/oya-workspace-mail-kernel/src/lib.rs::Mailbox` (after P06 IP-001 scaffold-claim).
