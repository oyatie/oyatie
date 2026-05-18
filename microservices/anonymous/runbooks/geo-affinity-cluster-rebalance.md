---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Geo-Affinity Cluster Rebalance
microservice: anonymous
severity: "Sev-3 (planned) / Sev-2 (k-anonymity floor breach)"
status: Accepted
owner_team: axis-anonymous + ops-data
date: 2026-05-17
related_adrs: [ADR-ANON-0007]
related_artifacts:
  - microservices/anonymous/PRD.md §"Bounded Contexts" / community-definition
  - microservices/anonymous/failure-modes.md FM-13
doc_status: published
---

# Runbook: Geo-Affinity Cluster Rebalance

## Trigger

| Signal | Severity |
|---|---|
| Geo-cluster cardinality drops below k=50 (planned rebalance) | Sev-3 |
| Geo-cluster cardinality drops below k=50 (unplanned; new sparse region onboarded) | Sev-2 |
| Geo-cluster cardinality drops below k=10 (emergency; anonymisation-fallback required) | Sev-2 |
| User reports unable to participate in geo-cluster ("waiting for k-anonymity floor") | per-report |

## Pre-checks

1. Query the geo-cluster cardinality: `cargo run -p oya-dev-cli -- anonymous community-definition cluster-cardinality --kind geo --pack <pack>`
2. List affected geo-clusters: those with cardinality < threshold
3. Determine parent-cluster: the geo-cluster's parent geo-region per ADR-ANON-0007 hierarchical model
4. Confirm the parent-cluster's cardinality meets k=50

## Steps — Planned rebalance

| Step | Action | Time budget |
|---|---|---|
| 1 | List sparse geo-clusters with `cardinality < 50` | ≤ 5 min |
| 2 | For each sparse cluster, identify its parent-region (one level up in geo-hierarchy: e.g., "Boise, ID" → "Idaho") | ≤ 10 min |
| 3 | Notify members of sparse cluster: they will be merged into parent-region cluster on <date> | 14-day notice |
| 4 | Execute merge: `cargo run -p oya-dev-cli -- anonymous community-definition merge --child <sparse-id> --parent <parent-id> --effective <date>` | ≤ 5 min |
| 5 | Bindings under the child cluster are re-issued under the parent cluster (BBS+ re-credential issuance) | ≤ 1h per 1000 bindings |
| 6 | Audit-chain seals `AffinityClusterMerged` event | – |
| 7 | Verify post-merge cardinality on parent ≥ 50 | ≤ 5 min |

## Steps — Emergency (k=10 anonymisation-fallback)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-2 incident | ≤ 5 min |
| 2 | Immediately pause new bindings into the sparse cluster: `cargo run -p oya-dev-cli -- anonymous community-definition pause-bindings --cluster <sparse-id>` | ≤ 1 min |
| 3 | Pause new posts to the sparse cluster (Cedar policy adds short-term `paused` flag) | ≤ 1 min |
| 4 | Notify members; rebalance into parent OR offer geo-anonymisation-fallback (member joins next-level-up region cluster) | ≤ 24h |
| 5 | Execute merge as planned-rebalance Step 4-7 | per planned timing |

## Anonymisation-fallback decision tree

```
Geo-cluster cardinality < k=50?
├─ YES, but ≥ k=10 → planned rebalance (14-day notice); merge into parent-region cluster
├─ < k=10 but ≥ k=5 → emergency rebalance (24h notice); merge into parent-region cluster
└─ < k=5 → IMMEDIATE pause + emergency rebalance; merge into grandparent-region cluster
```

## Hierarchical region model (per pack)

| Pack | Region hierarchy |
|---|---|
| pack-us | ZIP code → metro area → state → "USA" (last resort) |
| pack-eu | locality → NUTS-3 region → NUTS-2 region → country → "EU" (last resort) |
| pack-uk | postal town → ceremonial county → region → "UK" |
| pack-kr | 동/면 → 시/군 → 광역시/도 → "Korea" |
| pack-jp | 市区町村 → 都道府県 → "Japan" |
| pack-sg | postal sector → "Singapore" (one-level — Singapore is small enough that k=50 typically holds at city-level) |

## Failure modes

| Failure | Mitigation | Severity escalation |
|---|---|---|
| Member refuses merge | offer geo-anonymisation-fallback (deeper-hierarchy region) or unbind affinity | per-tenant policy |
| Parent cluster also below k=50 | merge into grandparent; if grandparent also sparse, escalate to country-level fallback | Sev-2 → Sev-1 if blocking |
| Re-credential issuance fails for some members | retry; if persistent, member's binding expires until next attestation cycle | Sev-3 |

## Cross-µservice coordination

- `tenancy`: tenant operators notified
- `audit-chain`: cluster merges sealed
- `ontology`: `Affinity` entity updated to reflect new cardinality and hierarchy

## References

- ADR-ANON-0007 — affinity-cluster k-anonymity floor + anonymisation-fallback
- Sweeney L. (2002), "k-anonymity: A model for protecting privacy"
- NUTS classification (EU regional statistical units)
