---
doc_class: ImplementationPlan
impl_plan_id: IP-004-emission-domain
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-audit-chain-emission-domain

## Intent

Pure event-classification + envelope construction + period bucketing math. Zero I/O. Imports only `-kernel`.

## Crate Naming

`oya-audit-chain-emission-domain` per BNF v4.1.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../src/crates/oya-audit-chain-emission-domain/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/envelope.rs` | create — `build_envelope(event) -> EventEnvelope` (deterministic; canonical-serialise; sha256 payload) |
| `.../src/period.rs` | create — `period_for(timestamp, pack) -> PeriodId` bucketing math |
| `.../src/classification.rs` | create — event-class validation (dotted-namespace shape) |

## Acceptance Gates

```bash
cargo check / build / clippy / nextest / coverage ≥ 95% line / 90% branch
property-test: envelope build is deterministic over 10k random AuditEvents
```

## References

- Bominal ADR-0003 §"Canonical serialisation".
- `microservices/audit-chain/PRD.md`.
