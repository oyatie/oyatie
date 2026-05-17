---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M05
title: Cloud-Stable + Search-Stable
wave: W-Cloud-Stable + W-Search-Stable
status: in-progress
owner: axis-cloud + axis-search + ops-compliance + regional-packs
purpose: Promote Cloud and Search from preview to public GA with marketplace, ISV onboarding, multi-AZ failover, FinOps surfaces, and regulator attestation.
acceptance_authority: docs/ROADMAP.md §2.8
---

# M05 — Cloud-Stable + Search-Stable

## Purpose
Take Cloud and Search to public GA. Add the surfaces that distinguish a public cloud from an internal substrate: marketplace, ISV onboarding, multi-AZ failover automation, FinOps surfaces, public SLA commitment (99.99% Cloud, KR ranking quality bar for Search). Bring regulator-equivalent (CSAP/ISMAP/FedRAMP/GAIA-X/MeitY/LGPD/NDMO/TDRA/IRAP) attestation for the regions in scope.

## Status
**gated on M04.** Public GA cannot ship until first commercial wave (M04) proves the stack.

## Scope
Four phases: Cloud-Stable GA (marketplace + ISV + multi-AZ + FinOps), Search-Stable GA (public web search + crawler + freshness + KG + SERP with sponsored-slot infrastructure ready), regulator attestation per region in scope (KR CSAP + K-ISMS-P + KCMVP HSM production-grade; JP/US/EU per regional pack), Cloud SLA + Search SLO commitment.

## Dependencies
- **Hard:** M04 pilot retention ≥ 80% over 8 weeks (proves the stack).
- **Hard:** M01-P13 distroless + image-discipline lanes green on all production binaries.
- **Hard:** M01-P15 supply-chain (Cosign+Rekor+SBOM+SLSA) lanes green.

## Acceptance gate
- Cloud SLA committed at 99.99% public-GA target.
- Public Search SLO + KR ranking quality bar met.
- KR CSAP + K-ISMS-P + KCMVP HSM in production attested.
- ≥ 5 regional packs onboarded (KR + JP + US + EU + one of IN/BR/KSA/UAE/ANZ/SG).
- Marketplace catalog public; ≥ 10 ISV listings.
- Multi-AZ failover automation drilled per [`docs/INCIDENT-MANAGEMENT.md`](../../../docs/INCIDENT-MANAGEMENT.md) quarterly.
- FinOps surface exposes per-tenant per-axis cost allocation (per [`../../../.omx/notepad.md`](../../../.omx/notepad.md) 2026-05-11 FinOps checkpoint).

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Cloud-Stable GA (Marketplace + ISV + Multi-AZ + FinOps) | stub | [`phases/P01-cloud-stable-ga/INDEX.md`](phases/P01-cloud-stable-ga/INDEX.md) |
| P02 | Search-Stable GA (Crawler + Freshness + KG + SERP + Sponsored-Slot Infra) | stub | [`phases/P02-search-stable-ga/INDEX.md`](phases/P02-search-stable-ga/INDEX.md) |
| P03 | Regulator Attestation Per Region in Scope | stub | [`phases/P03-regulator-attestation/INDEX.md`](phases/P03-regulator-attestation/INDEX.md) |
| P04 | Cloud SLA + Search SLO Public Commitment | stub | [`phases/P04-sla-slo-commitment/INDEX.md`](phases/P04-sla-slo-commitment/INDEX.md) |

## Parallelism strategy
P01, P02, P03 run in parallel (Cloud team, Search team, compliance team). P04 starts at P01 + P02 ≥ 80% (SLA/SLO require stable surface measurements). Target: 3 agents per phase.

## Hyperscaler practices adopted
- All practices inherited from M01-M04.
- Plus: public-SLA commitment process (AWS-style); regulator-attestation evidence-pack auto-generation (Oracle-style auditor portal).

## Agent-navigability-pointer
First-claim seed for Cloud-Stable GA: `crates/oya-cloud-marketplace-kernel/src/lib.rs::SellerApplication` (existing per [`../../../.omx/notepad.md`](../../../.omx/notepad.md) 2026-05-11 cloud marketplace checkpoint).
