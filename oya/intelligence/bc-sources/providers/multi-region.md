---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-foundry, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/policy/data-residency.md
  - microservices/intelligence-providers/capacity-model.md
  - microservices/intelligence-providers/cost-budget.md
  - microservices/intelligence-providers/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (foundry-providers µservice)

## Purpose

Define the multi-region topology across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets, failover procedures.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA) | OCI us-phoenix-1 (HIPAA) | DR pair; isolated | Conditional |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture

For packs with a DR pair:

```text
┌─ Pack <X> ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Primary region                          DR-pair region                    │
│  ┌─────────────────────────────┐         ┌─────────────────────────────┐   │
│  │ provider-router-rest (HA)   │         │ provider-router-rest (HA)   │   │
│  │ adapter pods (per vendor)   │         │ adapter pods (per vendor)   │   │
│  │ Postgres primary + replica  │ ←async→ │ Postgres replica            │   │
│  │ Valkey Sentinel HA           │ ←async→ │ Valkey replica               │   │
│  │ OpenBao agent (per pod)     │         │ OpenBao agent (per pod)     │   │
│  │ Egress proxy → vendor edges │         │ Egress proxy → vendor edges │   │
│  └─────────────────────────────┘         └─────────────────────────────┘   │
│            │                                       │                       │
│            └─────────── shared DNS / SPIFFE trust ──┘                      │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

DR-pair semantics:
- **Async streaming replication** for Postgres provider-config (RPO ≤ 60 s).
- **Async replication** for Valkey state (RPO ≤ 60 s; bucket recovery is operationally OK with brief over-throttle window).
- **Independent OpenBao deployment per region** (no cross-region credential replication; OpenBao agent in the DR region reads from the DR OpenBao primary). Cross-region OpenBao is owned by `cloud-secrets` µservice.
- **DNS-based failover** + SPIFFE identity remains valid (multi-region trust bundle).

## Cross-Pack Replication

**Forbidden.** Per `policy/data-residency.md`: cross-pack data replication is default-deny. The router decision is per-pack; a pack-kr tenant's invocation never leaves pack-kr's substrate even during DR.

Exception: per-pack SCC entitlement in tenant config (rare; recorded at onboarding via DPO).

## BCDR Posture

| Pack type | RPO target | RTO target | Notes |
|---|---|---|---|
| DR pair (eu / us / us-healthcare / au / in / br / ae / ksa) | ≤ 60 s | ≤ 5 min | Async replication; warm-standby |
| Single-region (kr / jp / sg) | ≤ 60 s in-region (no cross-region DR) | ≤ 15 min in-region | Tolerated for single-region packs; geographic constraint |

### RPO calculation

- Provider-router is **stateless** at runtime (decisions are per-call); on restart, no provider-router-owned state is lost.
- Postgres provider-config: async replication ≤ 60 s lag.
- Valkey token-bucket state: async replication ≤ 60 s lag; brief over-throttle window after failover is acceptable.
- OpenBao credentials: owned by `cloud-secrets`; inherits its RPO.

### RTO drills

- Quarterly DR drill: failover one pack's primary to DR pair; verify tenant invocations resume ≤ 5 min.
- Evidence: `evidence/runbook-drills/dr-failover/<pack>-<unix_ts>.json`.

## Failover Procedure

For DR-pair packs:

1. Primary-region health degraded (multiple AZs affected); declare Sev-1.
2. CommsLead notifies tenants of DR cutover in ≤ 30 min.
3. OpsLead executes: `cargo run -p oya-dev-cli -- vcs region-failover --pack <p> --to-region <dr-region> --reason "<id>"`.
4. The CLI: (a) updates DNS to point to DR region; (b) promotes DR Postgres replica to primary; (c) DR Valkey takes over; (d) DR OpenBao instance is queried by DR adapter pods; (e) emit `RegionFailover` audit-chain event.
5. Verify tenant invocations resume; observability shows `provider-router` qps recovering.
6. Postmortem; re-establish replication from new primary back to recovered region; eventually fail back if desired.

## Federated Multi-Pack (intentionally NOT supported)

Cross-pack failover is intentionally not supported. A pack-kr tenant whose entire KR substrate is unavailable does NOT fail over to pack-jp. This preserves the residency invariant. Tenants in single-region packs are notified at onboarding of this trade-off.

## Vendor Edge Multi-Region

Anthropic / OpenAI / Google operate their own multi-region vendor edges. The router selects the **vendor edge closest to the pack** (e.g., pack-kr selects Anthropic KR-SCC edge). When a vendor edge in a pack becomes degraded:
- Router auto-fails-over to next-best **same-pack** vendor edge (e.g., pack-kr Anthropic → pack-kr Gemini → pack-kr in-house).
- Cross-pack vendor-edge selection is forbidden per `policy/data-residency.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region --microservice foundry-providers` exits 0.
- Quarterly DR drill: evidence recorded.
- Per-pack DR-pair latency: replication lag dashboard panel.

## References

- ADR-0117 — pack residency model.
- `microservices/intelligence-providers/policy/data-residency.md`.
- `microservices/intelligence-providers/failure-modes.md`.
- OCI region table — `oracle.com/cloud/data-regions`.
