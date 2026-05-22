# IP-003 — Protocol Activation + Bundle Timer

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-0332 (in flight) | ADR-0251
Sequence: 3 / 10
Depends-on: IP-001, IP-002

---

## Scope

Trauma / Stroke / STEMI / Sepsis protocol activation with regulator-window bundle timer. Sepsis 3-hour bundle compliance SLO wired.

## Deliverables

- `src/crates/emergency-protocol/` — protocol activation aggregate.
- Bundle timer (per-protocol target windows).
- Pager / messenger integration on activation.
- Auto-suggest Sepsis on SIRS/qSOFA detection (alert, not auto-activate).
- Cedar `trauma-alert-bypass-rules.cedar` enforced.
- OpenSLO `protocol-bundle-compliance.openslo.yaml` + `door-to-needle.openslo.yaml` + `door-to-balloon.openslo.yaml` wired.
- `ed.protocol.activated`, `ed.protocol.deactivated`, `ed.protocol.window.breached` events.

## Acceptance

- Trauma alert end-to-end activation < 2 s.
- STEMI door-to-balloon SLO measurable.
- Sepsis bundle compliance ≥ 90% achievable in simulation.
- Window breach event fires deterministically.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/emergency/implementation-plans/IP-003-protocol-activation.md:23` - - `ed.protocol.activated`, `ed.protocol.deactivated`, `ed.protocol.window.breached` events..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emergency/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emergency/implementation-plans/IP-003-protocol-activation.md:13` - Trauma / Stroke / STEMI / Sepsis protocol activation with regulator-window bundle timer. Sepsis 3-hour bundle compliance SLO wired.; `microservices/emergency/implementation-plans/IP-003-protocol-activation.md:22` - - OpenSLO `protocol-bundle-compliance.openslo.yaml` + `door-to-needle.openslo.yaml` + `door-to-balloon.openslo.yaml` wired..
