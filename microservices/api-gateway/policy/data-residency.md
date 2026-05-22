# API Gateway Data Residency

**Authority:** ADR-0244 (tenant scoping) + ADR-0248 (cellular) + ADR-0251 (compliance packs).
**Last reviewed:** 2026-05-20.

## A — Principle

The gateway rejects cross-cell routing at the edge. Tenant residency is checked BEFORE workload dispatch using `tenant_id`, `cell_id`, and the regional pack attached to the hostname.

A cross-jurisdiction request is denied by `policy/sov-cloud-overlay.cedar` before reaching upstream — no data ever crosses a jurisdiction without explicit Cedar permit.

## B — Pack → cell-jurisdiction mapping

| Pack | Permitted cell jurisdictions | DR cell |
|---|---|---|
| pack-us (default) | us-east, us-west | cross-region |
| pack-eu | eu-frankfurt, eu-ireland | within EU |
| pack-kr | ap-seoul ONLY | ap-seoul-2 |
| pack-cn-pipl-2021 | cn-shanghai ONLY | cn-shanghai-2 |
| pack-us-healthcare | us-east, us-west (HIPAA BAA cells) | cross-region |
| pack-fedramp-high | sov-cell-fedramp-high | per-contract |
| pack-il5 | sov-cell-il5 | per-contract |
| pack-il6 | sov-cell-il6 | per-contract |
| pack-ksa-pdpl | me-riyadh ONLY | me-riyadh-2 (when provisioned) |
| pack-ae-pdpl | me-dubai ONLY | me-dubai-2 (when provisioned) |
| pack-jp | ap-tokyo | cross-region within JP cells |
| pack-sg | ap-singapore | cross-region within SEA |
| pack-au | ap-sydney (when provisioned) | per-contract |
| pack-in | ap-mumbai (when provisioned) | per-contract |
| pack-br | sa-saopaulo | per-contract |

## C — Enforcement layers

1. **DNS layer.** Per-tenant Anycast hint records steer to in-jurisdiction cells.
2. **TLS layer.** Server certs are per-region; cross-region cert presents triggers TLS mismatch.
3. **Gateway Cedar layer.** `policy/sov-cloud-overlay.cedar` forbids cross-jurisdiction routing.
4. **Audit layer.** Every routing decision logs `cell_jurisdiction` + `tenant_id` + `compliance_packs[]`.

This is a design surface; runtime residency proof remains owned by deployment + audit evidence gates per ADR-0250 build-ahead-of-certification.

## D — Cross-border data flow exceptions

Where regulation permits cross-border transfer:

- **pack-eu** ↔ adequacy-decision destinations (UK, US under DPF, etc.): permitted per `policy/sov-cloud-overlay.cedar` `permit-eu-adequacy-export`.
- **pack-kr** ↔ PIPA Art. 28 consent-based: per-data-subject consent required + recorded in identity µservice.
- **pack-cn-pipl-2021** ↔ CAC Art. 38 assessment-registered destinations only.

## E — References

- ADR-0244, ADR-0248, ADR-0251
- `compliance.md §C pack-overlay-roster`
- `policy/sov-cloud-overlay.cedar`
