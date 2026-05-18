# Multi-region — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0171, ADR-0180, ADR-0064.

## 1. Topology

The comms-email µservice deploys in every oyatie cluster
region. Each regional instance runs:

- A full set of adapter pools (SES regional + Postal regional
  + Mailgun + SMTP).
- A regional Postgres for suppression + idempotency.
- A regional audit-chain emission path (federated up to the
  global audit chain per ADR-0145).

## 2. Routing

Per IP-013:

- Tenant pack (ADR-0064) declares the bound region.
- comms-email lookup table maps `(tenant_id, pack)` → region.
- Sends are pinned to the bound region.

## 3. Failover

| Failure | First fallback | Second fallback | Sovereign |
| ------- | -------------- | --------------- | --------- |
| Bound region partial | retry within region | sibling region (if pack allows) | reject + page |
| Bound region full down | sibling region (if pack allows) | reject + page | reject + page |
| Provider outage in region | sibling provider in same region | sibling provider in sibling region | Postal-only — sovereign |

## 4. Active-active vs active-passive

- Cloud-hosted regions: active-active across regions; tenant
  pinning prevents cross-talk.
- Sovereign regions: active-active within the sovereign
  boundary; never cross out.

## 5. Data residency

- Per-tenant suppression list lives in the bound region's
  Postgres only.
- Cross-region replication is OFF for sovereign packs and ON
  (read-only sibling) for cloud-hosted packs that explicitly
  opt-in.

## 6. DR / BC

Per ADR-0180:

- RPO: 0 (audit chain seal is the source of truth; suppression
  list is reconstructable from the chain).
- RTO: 15 minutes for in-region recovery; 60 minutes for
  cross-region (when pack permits).

## 7. Region matrix (Phase 1)

| Region | SES | Postal | Mailgun | SMTP | Sovereign |
| ------ | --- | ------ | ------- | ---- | --------- |
| us-east-1 | ✓ | ✓ | ✓ | ✓ | — |
| us-west-2 | ✓ | ✓ | ✓ | ✓ | — |
| eu-central-1 | ✓ | ✓ | ✓ | ✓ | — |
| eu-west-1 | ✓ | ✓ | ✓ | ✓ | — |
| ap-northeast-2 (KR) | — | ✓ | — | ✓ | ✓ |
| me-south-1 (KSA/UAE) | — | ✓ | — | ✓ | ✓ |

## 8. Observability

- Per-region SLO dashboards (`p99 send latency` per region).
- Per-region audit-chain emission lag dashboard.
- Cross-region tenant pinning compliance dashboard.

## 9. Runbooks

- `ses-failover.md` — same-region SES → Postal.
- `postal-failover.md` — same-region Postal cold-side.
- `blacklist-recovery.md` — IP reputation recovery.

## 10. Open questions

- Multi-cloud (AWS + GCP + Azure) deployment topology — pending
  the cross-cloud root-of-trust ADR named in ADR-0202 open
  questions.
