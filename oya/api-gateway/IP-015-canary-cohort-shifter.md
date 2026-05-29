# IP-015: `oya-api-gateway-canary-cohort-shifter` crate

**Status:** design-ready
**Owner:** axis-network + ops-deployments
**Authority:** ADR-0114, ADR-0139.

## A — Scope

Worker that shifts canary traffic weight gated by SLO burn-rate evaluator.

## B — Acceptance criteria

- 5% / 25% / 50% / 100% gradual ramp.
- Auto-rollback on burn-rate breach.
- Soak ≥30s per ADR-0294.
- Audit per shift.

## Wave 15 A-G substance

### A - Problem
The gateway must shift tenant and cohort traffic gradually while proving that SLO burn, Cedar fragment soak, and rollback evidence are checked before each weight increase.

### B - Approach
Implement `oya-api-gateway-canary-cohort-shifter` from `catalog/oya-api-gateway-canary-cohort-shifter.yaml` as a worker that owns route weight transitions, cohort selection, SLO observation, soak enforcement, audit emission, and rollback handoff.

### C - Deliverables
- Cohort model for tenant, cell, region, route class, client type, and sticky assignment key.
- Ramp state machine for 0, 5, 25, 50, and 100 percent with ADR-0294 soak windows.
- SLO observer for edge availability, p95/p99 latency, TLS handshake success, and denial-rate anomaly.
- Rollback trigger for fast-burn, elevated 5xx, route-deny spike, bot-score anomaly, and audit-emission lag.
- Audit records for canary routed, weight shifted, rollback requested, rollback completed, and promotion completed.
- Dry-run mode for CI to verify proposed shifts without publishing xDS changes.

### D - Ordered implementation steps
1. Define cohort and ramp state models using route IDs and cell IDs.
2. Add SLO read port with fixture burn-rate series.
3. Add soak timer and Cedar fragment activation guard.
4. Implement weight publication through routing-worker/xDS adapter port.
5. Emit audit records for each shift and rollback decision.
6. Add tests for successful ramp, SLO breach rollback, stale metrics, and partial-cell depool.
7. Link runbook actions to `runbooks/blue-green-rollback.md`.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-canary-cohort-shifter --features fixtures` passes.
- Ramp cannot advance without a completed soak window and green SLO evidence.
- Fast-burn breach triggers rollback before the next shift.
- Dry-run output matches the intended xDS weight patch without mutating route state.
- Audit records include route, cohort, previous weight, next weight, SLO snapshot, and operator/CI principal.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-canary-cohort-shifter.yaml`, `capabilities/canary-route-shift.yaml`, `policy/ci-scope.cedar`, `runbooks/blue-green-rollback.md`, `slos/edge-availability.openslo.yaml`, `slos/edge-latency-p95.openslo.yaml`, `slos/edge-latency-p99.openslo.yaml`, and `ARCHITECTURE.md`.

### G - Counterpart comparison
GitLab API ingress is the concrete counterpart because large API surfaces use staged rollouts, feature cohorts, and automatic rollback when error or latency budgets burn. Oyatie applies that discipline at the gateway route-weight layer with Cedar soak and audit-chain evidence.

## Remediation notes

- Rewrote the canary stub into a worker plan with cohort modeling, SLO evidence, soak enforcement, and rollback gates.
- This IP should stay separate from IP-008: IP-015 decides when weights move, while the routing worker publishes validated route bundles.
- Future remediation should add explicit fixtures for edge-latency and denial-rate anomaly series once observability fixture paths are finalized.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Capability | `capabilities/canary-route-shift.yaml` | Route-shift action and risk class are declared. |
| CI scope | `policy/ci-scope.cedar` | Only approved CI/operator principals can shift traffic. |
| SLO availability | `slos/edge-availability.openslo.yaml` | Fast-burn breach blocks promotion. |
| SLO latency | `slos/edge-latency-p95.openslo.yaml` | Latency breach blocks promotion. |
| Rollback runbook | `runbooks/blue-green-rollback.md` | Rollback actions are operator-visible. |
| GitLab rollout | GitLab API ingress | Cohort rollout mirrors staged API deployment pressure. |
| xDS handoff | `IP-008-routing-worker-crate.md` | Weight publication occurs through validated route bundles. |
| Audit | `contracts/api-gateway.asyncapi.yaml` | Shift and rollback decisions include before/after weights. |
| Soak | `docs/decisions/ADR-0294.md` | Minimum soak completes before each ramp. |
| Cell behavior | `runbooks/cell-evac.md` | Depooled cells cannot receive new canary weight. |
| Dry run | `feature-parity-matrix-2026-05-20.md` | Management-surface gaps do not imply live mutation support. |
| Metrics | `dashboards/edge-overview.json` | Canary state is visible with route/cell labels. |

## Remediation follow-up checklist

- Add GitLab-style staged API rollout fixtures for 5, 25, 50, and 100 percent.
- Add fast-burn, stale metrics, and denial-rate spike rollback fixtures.
- Add dry-run output comparison against the intended route weight patch.
- Add depooled-cell fixture proving no canary weight lands in isolated cells.
- Add audit assertions for shift requested, shift applied, rollback requested, and rollback completed.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-015-canary-cohort-shifter.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-015-canary-cohort-shifter.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-015-canary-cohort-shifter.md`.
