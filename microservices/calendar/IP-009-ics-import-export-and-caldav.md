---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-009-ics-import-export-and-caldav
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-rfc-4791-conformance, oya-governance-rfc-5545-conformance, oya-governance-caldav-backend-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: ics-import-export — kernel + domain + usecase + adapter-icalendar + adapter-caldav-radicale + adapter-caldav-sabredav

## Intent

Implement the ics-import-export BC per PRD §"Bounded Contexts" row 6.
.ics import/export per RFC 5545. CalDAV adapter per RFC 4791 + RFC
6638 + RFC 7953 — Radicale primary; SabreDAV for pack-us-healthcare
per ADR-CAL-0001.

## ChangeSet boundary

7 crates: `-kernel`, `-domain`, `-usecase`, `-adapter` (protocol-
neutral), `-adapter-icalendar` (.ics parser/emitter), `-adapter-
caldav-radicale`, `-adapter-caldav-sabredav`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-kernel/` | create | IcsParser + IcsEmitter + CalDavBackend port traits |
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-domain/` | create | parse-time invariants (bounded; XXE-refused; size-capped) |
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-usecase/` | create | import-ics + export-ics orchestrators |
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-adapter-icalendar/` | create | libical-derived RFC 5545 parser/emitter |
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-adapter-caldav-radicale/` | create | Radicale 3.2.3 CalDAV backend adapter |
| `microservices/calendar/src/crates/oya-calendar-ics-import-export-adapter-caldav-sabredav/` | create | SabreDAV 4.6 CalDAV backend adapter (us-healthcare only) |
| `microservices/calendar/tests/e2e/caldav-clients.rs` | create | E2E against Apple Calendar + Thunderbird + Evolution + DAVx5 |
| `microservices/calendar/tests/e2e/ics-roundtrip-x-extensions.rs` | create | X-extension preservation round-trip (Hyrum #3) |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-ics-import-export-domain -- bounded_input
cargo nextest run -p oya-calendar-ics-import-export-adapter-icalendar -- import_10k
cargo nextest run -p tests --test e2e_caldav_clients
cargo run -p oya-dev-cli -- gate validate rfc-4791-conformance --microservice calendar
cargo run -p oya-dev-cli -- gate validate rfc-5545-conformance --microservice calendar
cargo run -p oya-dev-cli -- gate validate caldav-backend-conformance --microservice calendar
```

## Test Plan

- PRD AC-04 — RFC 4791 CalDAV PROPFIND/REPORT/PUT/DELETE end-to-end
  against Apple Calendar + Thunderbird + Evolution + DAVx5.
- PRD AC-05 — .ics import 10k events p95 ≤ 30s (per problem statement).
- Strong-ETag = SHA-256 of canonicalised iCalendar (Hyrum #7).
- X-extension round-trip preservation (Hyrum #3).
- Both backends pass libical RFC 4791 conformance corpus.

## Halt Conditions

- PRD AC-04 fails against any of the 4 CalDAV clients — block.
- libical CalDAV conformance corpus regresses — block.

## Next IP

[`IP-010-tzdb-refresh-worker.md`](IP-010-tzdb-refresh-worker.md)

## References

- RFC 4791; RFC 5545; RFC 6638; RFC 7953.
- ADR-CAL-0001 (CalDAV backend); ADR-CAL-0003 (CalDAV-first frontend).
- Radicale 3.2.x — `radicale.org`.
- SabreDAV 4.6 — `sabre.io/dav/`.
- libical CalDAV corpus — `github.com/libical/libical`.
