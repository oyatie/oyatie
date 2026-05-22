# IP-002 — Tracking Board Projection + SSE/WebSocket Fanout

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0131
Sequence: 2 / 10
Depends-on: IP-001

---

## Scope

Stand up the real-time tracking board with sub-500ms p99 fanout. Wire the Valkey-backed snapshot with PostgreSQL as the durable backing store. Implement SSE and WebSocket fanout for ED dashboards.

## Deliverables

- `src/crates/emergency-trackingboard/` — board projection + fanout.
- `src/crates/emergency-bedcontrol/` — bed grid aggregate + state machine.
- Valkey snapshot manager.
- SSE endpoint at `GET /ed/board` (with SSE content-type).
- WebSocket endpoint at `wss://.../ed/board`.
- gRPC `BoardSubscribe` stream RPC.
- Charge-nurse `BedReassign` RPC.
- OpenSLO `tracking-board-staleness.openslo.yaml` wired.
- Cedar `charge-nurse-can-reassign-bed.cedar` enforced.

## Acceptance

- p99 mutation-to-dashboard ≤ 500 ms in load test with 50 concurrent dashboards.
- 50-bed ED cold load ≤ 1.5 s.
- Bed reassign atomicity verified.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-002-tracking-board.md:13` - Stand up the real-time tracking board with sub-500ms p99 fanout. Wire the Valkey-backed snapshot with PostgreSQL as the durable backing store. Implement SSE and WebSoc...; `microservices/emergency/implementation-plans/IP-002-tracking-board.md:24` - - OpenSLO `tracking-board-staleness.openslo.yaml` wired..
