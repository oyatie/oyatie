# IP-017: Sovereign-cell routing adapter

**Status:** design-ready
**Owner:** axis-network + ops-platform
**Authority:** ADR-0244, ADR-0248, ADR-0251.

## A — Scope

Adapter that consults tenant compliance-pack roster + cell jurisdiction map and returns a routing decision. Enforced at the gateway BEFORE upstream call.

## B — Acceptance criteria

- Sub-100µs decision.
- Per-pack permit/forbid via `policy/sov-cloud-overlay.cedar`.
- Audit on every cross-jurisdiction admit attempt.
- 100% coverage on the per-pack matrix.

## C — References

- ADR-0244, ADR-0248, ADR-0251
- `policy/data-residency.md`
- `policy/sov-cloud-overlay.cedar`

## Wave 15 A-G substance

### A - Problem
North-south ingress must reject or reroute requests that would cross a tenant's compliance pack, cell jurisdiction, or certification boundary before any upstream service receives traffic.

### B - Approach
Implement the sovereign-cell routing adapter as the gateway-local enforcement layer for compliance-pack roster, cell jurisdiction map, pack version, tenant residency, and cross-jurisdiction audit evidence. It consumes signed principal/cell context and Cedar policy outcomes; it does not call a remote policy service on the hot path.

### C - Deliverables
- Residency input model containing tenant, source cell, requested route, route data class, pack roster, cell jurisdiction, pack version, and certification status.
- Decision model for allow local, allow alternate cell, deny residency, deny missing certification, deny stale pack, and audit-only cross-jurisdiction attempt.
- Cedar context builder for `policy/sov-cloud-overlay.cedar`.
- Per-pack matrix fixtures for us, eu, kr, cn-pipl-2021, us-healthcare, fedramp-high, il5, il6, ksa-pdpl, ae-pdpl, jp, sg, au, in, and br.
- Audit payloads for cross-jurisdiction admit attempt, residency deny, pack stale, and cell depool/repool.
- Sub-100 microsecond in-memory decision path using preloaded pack/cell maps.

### D - Ordered implementation steps
1. Load pack roster and cell jurisdiction fixtures from policy/data residency artifacts.
2. Define adapter input/output DTOs shared with routing-usecase.
3. Build Cedar evaluation context and fail-closed default decisions.
4. Add per-pack matrix tests and stale-pack tests.
5. Add depool/repool behavior for isolated or uncertified cells.
6. Add audit-chain event builders.
7. Prove no remote network calls occur on the decision path.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-sov-cell-routing --features fixtures` passes once the crate is introduced.
- Per-pack matrix coverage is 100 percent for all packs listed in `manifest.json`.
- Cross-jurisdiction attempts emit audit evidence even when denied.
- Stale pack version, missing cell certification, and isolated cell fail closed.
- Decision fixtures stay under the sub-100 microsecond budget.

### F - Evidence
Grounding files: `policy/data-residency.md`, `policy/sov-cloud-overlay.cedar`, `manifest.json`, `ARCHITECTURE.md`, `runbooks/cell-evac.md`, `multi-region.md`, `contracts/api-gateway.openapi.yaml`, and `contracts/api_gateway.proto`.

### G - Counterpart comparison
Salesforce API ingress is the concrete counterpart because enterprise API calls often require org/region routing, compliance packs, and residency restrictions before object access. Oyatie applies that discipline at cell-routing time with Cedar-backed residency decisions and audit-chain evidence.

## Remediation notes

- Expanded IP-017 into a service-specific sovereign routing plan, including decision states, pack matrix coverage, and audit requirements.
- The file is not currently listed in `manifest.json`; follow-up should add IP-017 to the machine-readable IP inventory if this plan is promoted with IP-001..016.
- Keep remote policy-engine calls out of the hot path; this adapter must use caller-side Cedar/library evaluation and preloaded cell maps.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Pack roster | `manifest.json` | Every declared pack has a matrix fixture. |
| Residency policy | `policy/sov-cloud-overlay.cedar` | Permit/forbid logic is explicit and fail-closed. |
| Data residency | `policy/data-residency.md` | Jurisdiction and data-class rules map to decisions. |
| Cell routing | `ARCHITECTURE.md` | Cell-aware routing occurs before upstream calls. |
| Salesforce pressure | Salesforce API ingress | Org/region API calls respect residency boundaries. |
| Cross-jurisdiction audit | `contracts/api-gateway.asyncapi.yaml` | Attempts are recorded even when denied. |
| Cell evacuation | `runbooks/cell-evac.md` | Depooled/isolated cells cannot be selected. |
| Multi-region | `multi-region.md` | Alternate-cell decisions preserve pack constraints. |
| REST fields | `contracts/api-gateway.openapi.yaml` | Tenant, cell, route, and pack fields can be represented. |
| gRPC fields | `contracts/api_gateway.proto` | Internal callers carry equivalent residency context. |
| Hot path | `ARCHITECTURE.md` | Decision path uses preloaded data and caller-side policy. |
| Manifest debt | `manifest.json` | IP-017 absence is recorded as follow-up, not hidden. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-017-sov-cell-routing.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` with drill cadence `quarterly`.
- RTO/RPO target: RTO p99 <= `3600` seconds; RPO p99 <= `300` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `true`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-017-sov-cell-routing.md`.
