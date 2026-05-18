---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-010-tzdb-refresh-worker
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + ops-sre-reliability
acceptance_lanes: [cargo-nextest, oya-governance-tzdb-staleness-bound, kubectl-apply-dry-run]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: tzdb-refresh-worker — CronJob per ADR-CAL-0004

## Intent

Implement the IANA tzdb refresh worker per ADR-CAL-0004. Polls
`data.iana.org/time-zones/releases/` every 24h. Opens release-pin-
bump ChangeSet when new release detected. Runs RFC 5545 corpus +
DST edge-case matrix before promotion.

## ChangeSet boundary

1 crate (`oya-calendar-tzdb-refresh-worker`) + CronJob manifest in
IaC + per-tenant pin config schema.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-tzdb-refresh-worker/Cargo.toml` | create | crate manifest |
| `microservices/calendar/src/crates/oya-calendar-tzdb-refresh-worker/src/main.rs` | create | poll-loop binary |
| `microservices/calendar/src/crates/oya-calendar-tzdb-refresh-worker/src/poller.rs` | create | IANA release-stream poller |
| `microservices/calendar/src/crates/oya-calendar-tzdb-refresh-worker/src/bumper.rs` | create | opens release-pin-bump ChangeSet |
| `microservices/calendar/iac/helm/templates/cronjob.yaml` | already created in IP-001 | binds the worker as a CronJob |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-tzdb-refresh-worker
cargo run -p oya-dev-cli -- gate validate tzdb-staleness-bound --microservice calendar
```

## Test Plan

- Poller smoke test against a mocked IANA release stream.
- Bump-ChangeSet opens with the correct semver bump for `chrono-tz`.
- Per-tenant pin override config schema parses correctly.
- 30d staleness SLO emits the expected metric.

## Halt Conditions

- Poller crashes on malformed IANA release index — block.

## Next IP

[`IP-011-contracts-openapi-asyncapi-proto.md`](IP-011-contracts-openapi-asyncapi-proto.md)

## References

- ADR-CAL-0004 (tzdb refresh + pinning).
- IANA tz mailing list — `mm.icann.org/pipermail/tz/`.
- `chrono-tz` — `crates.io/crates/chrono-tz`.
- `microservices/calendar/runbooks/timezone-db-refresh.md` (refresh path).
- `microservices/calendar/runbooks/tzdb-rollback.md` (rollback path).
- `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml`.
