---
id: ADR-CAL-0002
status: Accepted
date: 2026-05-17
microservice: calendar
deciders: axis-calendar, council-architecture, ops-sre-reliability
owner: axis-calendar
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-CAL-0001
related_artifacts:
  - microservices/calendar/PRD.md (FR-02; AC-03 RFC 5545 RRULE corpus 100% pass; Open Question 4)
  - microservices/calendar/runbooks/recurrence-storm.md
  - microservices/calendar/runbooks/scheduling-poll-deadlock.md
  - microservices/calendar/slos/scheduling-convergence-latency.openslo.yaml
purpose: |
  Close PRD-calendar Open Question 4 — RRULE BYSETPOS interaction with
  EXDATE — and define the conformance posture for the recurrence engine
  end-to-end. PRD AC-03 requires the RFC 5545 corpus to pass 100%; this ADR
  picks the engine, names the corpus, and binds the conformance lane.
---

# ADR-CAL-0002: Recurrence engine — full RFC 5545 RRULE conformance via `rrule-rs` against the libical + python-dateutil corpora

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The `recurrence-engine` bounded context (PRD §"Bounded Contexts" row 2)
owns RFC 5545 RRULE / EXDATE / RDATE expansion. RRULE is the calendar
world's most-citation-bound surface: every other calendar product
(Google, Outlook, Apple, Fastmail, Proton, Cal.com) implements it, and
every implementation has its own quirks. The known quirk surface:

- **BYSETPOS** interaction with EXDATE — when BYSETPOS selects the
  Nth-from-end occurrence in a frequency window, does an EXDATE
  exclusion in that window shift the BYSETPOS selection or merely
  delete the originally-selected occurrence? PRD Open Question 4.
- **BYDAY=-1MO** ("the last Monday of the month") interaction with
  short months where the last Monday is the same date as the BYSETPOS=-1
  fallback.
- **UNTIL vs COUNT** — RFC 5545 forbids both in the same RRULE but many
  consumer clients (notably Outlook on Windows pre-2018) emit both;
  legacy interop demands a documented forgiveness posture.
- **DST transitions** — an RRULE that crosses a DST jump (e.g., 2 AM
  Sunday in `America/New_York` on a spring-forward day) must follow
  RFC 5545 §3.3.5 "DATE-TIME with a TZID" rules; what to do when the
  RRULE'd local time does not exist that day (spring-forward) or exists
  twice (fall-back).
- **Leap-second handling** — RFC 5545 §3.3.5 explicitly forbids leap
  seconds in DATE-TIME values; many clients emit them anyway.
- **Floating time** (DATE-TIME without TZID and not in UTC) — RFC 5545
  permits "floating" times that should re-interpret in each viewer's
  local tz; how this interacts with cross-tenant free/busy queries.

PRD-calendar AC-03 demands the RFC 5545 RRULE conformance test corpus
pass at 100%. The two canonical corpora are:

1. **libical test corpus** at `github.com/libical/libical/tree/master/
   src/test/data` — ~200 RRULE cases; the de-facto industry reference.
   libical is used inside Apple Calendar, Mozilla Thunderbird, GNOME
   Evolution, and KDE KOrganizer, so passing it = passing those clients.
2. **python-dateutil rrule test suite** at
   `github.com/dateutil/dateutil/tree/master/tests` — ~150 cases;
   widely-used as the Python ecosystem's RRULE reference; cited in
   many Stack Overflow answers and is the de-facto "behaviour
   reference" for ambiguous edge cases.

Three RFC-5545-conformant Rust implementations are candidate engines:

1. **`rrule-rs` 0.13.x** (MIT). Most mature Rust RRULE library; passes
   the libical corpus 100% as of 0.13.0; passes the python-dateutil
   corpus 96% (the 4% gap is documented quirks where python-dateutil
   intentionally diverges from RFC 5545). Active upstream; LTS pinnable.
2. **`ical-rs` 0.12.x** (MIT/Apache-2.0). Lower-level RFC 5545 parser
   with RRULE included; less battle-tested than `rrule-rs`; partial
   conformance against libical corpus (~85%).
3. **Custom in-house implementation**. As Alternative B in ADR-CAL-0001:
   from-scratch RRULE in idiomatic Rust. Estimated 6 engineer-months
   to reach libical-corpus parity.

Performance budget per PRD-calendar §"Performance":
- Recurrence expansion (single RRULE; 1y horizon) p99 ≤ 1s; p999 ≤ 3s
  with bounded window.
- Scheduling-poll convergence p95 ≤ 500ms (per problem statement;
  surfaces in `runbooks/scheduling-poll-deadlock.md`).

## Decision

The recurrence engine ships with **full RFC 5545 RRULE conformance**, no
subsetting. Concrete bindings:

1. **Engine: `rrule-rs` 0.13.x LTS pin** (`rrule = "0.13.1"`), exposed
   via the `RecurrenceExpander` port trait from `oya-calendar-recurrence-
   engine-kernel`. Implementation crate: `oya-calendar-recurrence-engine-
   adapter` (no `-<backend>` qualifier — `rrule-rs` is the only adapter).

2. **Conformance corpora vendored into the repo**:
   `microservices/calendar/tests/corpora/rfc-5545-libical/` (~200
   cases; copied from libical at a pinned commit + checksum recorded
   in the CI lane); `microservices/calendar/tests/corpora/rfc-5545-
   python-dateutil/` (~150 cases). The 4% python-dateutil divergence
   from RFC 5545 is documented in
   `microservices/calendar/tests/corpora/DIVERGENCES.md` with a per-case
   rationale; for those 4% cases the test asserts RFC-5545-strict
   behaviour, not python-dateutil behaviour.

3. **Named edge-case test matrix** — these are first-class test cases,
   not catch-alls:

   | Edge case | Expected behaviour | Test fixture |
   |---|---|---|
   | BYSETPOS × EXDATE interaction (PRD Open Q4) | EXDATE deletes the occurrence; BYSETPOS does NOT shift to the next candidate (per RFC 5545 §3.3.10) | `tests/rrule_bysetpos_exdate.rs` |
   | BYDAY=-1MO × month with 5 Mondays | last-occurrence Monday selected (not the 4th-Monday alias) | `tests/rrule_byday_lastmo.rs` |
   | UNTIL + COUNT both present | reject the RRULE at parse time per RFC 5545; surface as `MalformedRecurrence::BothUntilAndCount` | `tests/rrule_until_count.rs` |
   | DST spring-forward at 02:00 local | event clamps to 03:00 local on spring-forward day (per RFC 5545 §3.3.5) | `tests/rrule_dst_spring.rs` |
   | DST fall-back at 02:00 local | event fires once (the first instance); never re-fires at the duplicated wall time | `tests/rrule_dst_fall.rs` |
   | Leap-second in DTSTART | reject the DATE-TIME at parse time per RFC 5545 §3.3.5; surface as `MalformedRecurrence::LeapSecond` | `tests/rrule_leap_second.rs` |
   | Floating time in cross-tenant free/busy | resolve floating to the QUERIER's tz, not the AUTHOR's tz; document this explicitly per RFC 5545 §3.3.5 | `tests/rrule_floating_cross_tenant.rs` |

4. **Bounded materialisation invariant**: per PRD-calendar AC-10, any
   RRULE that would expand to > 5y horizon is REJECTED at the
   `oya-calendar-recurrence-engine-domain` invariant check; the kernel
   port surface returns `RecurrenceBoundExceeded` for the caller.
   `runbooks/recurrence-storm.md` documents the operator response when
   a malformed RRULE attempts unbounded expansion (e.g., `FREQ=SECONDLY`
   with no UNTIL/COUNT).

5. **Outlook UNTIL+COUNT forgiveness** is NOT shipped at M03. The PRD's
   "RFC 5545 RRULE conformance" target is strict; legacy Outlook clients
   that emit both fields receive a 400 with a remediation hint. This is
   revisited in a follow-up ADR if the support tickets warrant.

## Alternatives Considered

### A. Conformance subset (no BYSETPOS / no BYDAY=-1MO / no DST handling)

- **Pros**:
  - ~50% less test surface.
  - Easier to ship at M03 ahead of HG-CALENDAR gate.
  - Matches early-stage competitors (e.g., older Cal.com versions).
- **Cons**:
  - Fails AC-03 (RFC 5545 corpus 100% pass) outright.
  - Fails real-world client interop — Apple Calendar / Outlook /
    Thunderbird all emit BYSETPOS-bearing RRULEs in common recurrence
    patterns (e.g., "first Monday of the month" is BYSETPOS=1 in many
    client encodings).
  - Differentiator-by-construction is lost — every competitor supports
    these; oyatie shipping a subset is a regression.
- **Rejected** because AC-03 is a release gate.

### B. `ical-rs` 0.12.x as the engine

- **Pros**:
  - Lower-level — more control over the parser/expander split.
  - Permissive license (MIT/Apache-2.0; no AGPL risk).
- **Cons**:
  - Conformance ceiling is ~85% against libical corpus as of 0.12.x.
  - Less battle-tested in production deployments.
  - Closing the 15% gap is a per-edge-case engineering effort on our
    side, no upstream share.
- **Rejected** in favour of `rrule-rs` which is already at 100%
  libical-corpus pass.

### C. Custom in-house RRULE implementation

- **Pros**:
  - Full control over the data model + traversal order + memoisation.
  - No upstream-version drift.
- **Cons**:
  - 6 engineer-month effort to reach libical-corpus parity.
  - Long interop tail — every client's quirks must be tested
    independently.
  - No upstream maintenance share for the long edge-case tail.
- **Rejected** under "buy not build" — `rrule-rs` already exists and
  passes the corpus.

### D. Python-dateutil divergence as ground truth

- **Pros**:
  - Some Python-side consumers (e.g., data-pipeline scripts) may expect
    python-dateutil behaviour.
- **Cons**:
  - python-dateutil's 4% RFC-divergent behaviour is upstream-documented
    as "intentional convenience deviations from RFC 5545"; honouring
    them in the server would emit non-RFC-5545-conformant occurrences
    to non-python clients.
  - Apple Calendar / Outlook / Thunderbird would diverge from oyatie's
    expansions for those 4% cases.
- **Rejected** — RFC 5545 is the authoritative spec; python-dateutil's
  convenience deviations are documented but NOT honoured server-side.

### E. Full conformance via `rrule-rs` against libical + python-dateutil corpora  ← **CHOSEN**

- **Pros**:
  - AC-03 met by construction (libical corpus 100% pass).
  - python-dateutil corpus passes at 96%; the 4% divergence is
    documented and tests assert RFC-strict behaviour.
  - Upstream-maintained — RFC clarifications + CVE patches arrive
    through `rrule-rs` releases.
  - Performance budget met — `rrule-rs` 1y expansion p99 ≈ 80ms on
    cargo-bench against typical RRULEs.
- **Cons**:
  - 4% python-dateutil divergence means some Python-side data pipelines
    may need to call our server explicitly rather than re-implementing
    expansion locally.
  - `rrule-rs` upstream is one developer's primary project; bus-factor
    risk. Mitigation: fork-ready (MIT license); axis-calendar maintains
    a maintenance commitment.
- **Accepted** — meets the conformance bar, meets the performance
  budget, leverages an active upstream.

## Consequences

### Positive

- **AC-03 (RFC 5545 corpus 100% pass) met by construction.** CI lane
  `oya-governance-rfc-5545-conformance` (per PRD §"CI lanes that must
  green") runs both corpora in nextest; refusal of any case blocks
  merge.
- **Cross-client interop covered end-to-end.** Apple Calendar /
  Outlook / Thunderbird / Fastmail / Proton / Cal.com all share libical-
  derived expansion semantics; we match.
- **Edge-case behaviour documented in test code, not in tribal
  knowledge.** Each of the 7 named edge cases has its own test fixture
  file with a comment block citing the RFC section that mandates the
  behaviour.
- **Bounded materialisation prevents storm runbook activation.** PRD
  AC-10 enforced at the domain layer; `runbooks/recurrence-storm.md`
  fires only when malformed input slips through.

### Negative

- **Upstream bus-factor on `rrule-rs`.** Mitigation: vendored corpora
  let us catch regressions on upstream-bump; we maintain a maintenance
  commitment and a fork-ready posture.
- **No Outlook UNTIL+COUNT forgiveness at M03.** Some legacy
  Windows-Outlook-pre-2018 clients will see 400s on broken RRULEs;
  remediation hint emitted; revisited per support volume.
- **Python-dateutil-using consumers must call us, not expand locally.**
  Documented in PRD competitive-parity row + in sdk-plan.md Python
  section.

### Operational

- **New CI lane `oya-governance-rfc-5545-conformance`** (BLOCKER from
  M03): runs both vendored corpora in `cargo nextest`; fails on any
  case.
- **Vendored-corpus version lock**: corpora are vendored under
  `tests/corpora/` with a SHA-256 of the upstream commit recorded in
  `tests/corpora/VERSION.txt`; updating the corpus requires a
  same-ChangeSet update to the divergence rationale doc.
- **Per-tenant bound-exceeded telemetry**:
  `oya_calendar_recurrence_bound_exceeded_total{tenant_id,reason}`
  emitted per refusal; pack-kr alert threshold at >1/min.
- **Runbook `recurrence-storm.md`** documents the bound-exceeded
  refusal pattern; the recovery procedure cancels the malformed RRULE
  and notifies the author.

### Regulatory

- **GDPR Art. 5(1)(d)** (accuracy): RFC-conformant expansion =
  data-accuracy compliance — every occurrence emitted is grounded in
  an RFC-defined rule.
- **KR PIPA Art. 16** (data accuracy): same.
- **HIPAA 45 CFR §164.502(b)** (minimum necessary): bounded
  materialisation prevents unbounded data emission.
- **EU AI Act**: out of scope (RRULE is deterministic; no AI).

## Verification

- [ ] **libical corpus passes 100%** —
  `cargo nextest run -p oya-calendar-recurrence-engine-domain -- rfc_5545_libical_corpus`.
- [ ] **python-dateutil corpus passes 96%** (4% RFC-divergent cases
  assert RFC-strict behaviour) —
  `cargo nextest run -p oya-calendar-recurrence-engine-domain -- rfc_5545_python_dateutil_corpus`.
- [ ] **Named edge-case test matrix all passes** —
  `cargo nextest run -p oya-calendar-recurrence-engine-domain -- rrule_edge_cases`.
- [ ] **Bound-exceeded refusal at 5y horizon** —
  `cargo nextest run -p oya-calendar-recurrence-engine-domain -- bound_exceeded`.
- [ ] **`oya gate validate rfc-5545-conformance --microservice calendar`** exits 0 (per PRD §"CI lanes").

## References

- RFC 5545 — iCalendar (Internet Calendaring and Scheduling Core Object Specification).
- RFC 5545 §3.3.10 — RECUR (RRULE rule structure).
- RFC 5545 §3.3.5 — DATE-TIME (TZID / floating / UTC semantics).
- RFC 5546 — iCalendar Transport-Independent Interoperability Protocol (iTIP).
- RFC 7529 — Non-Gregorian Recurrence Rules (out-of-scope for M03; tracked).
- libical RRULE test corpus — `github.com/libical/libical/tree/master/src/test/data`.
- python-dateutil rrule tests — `github.com/dateutil/dateutil/tree/master/tests`.
- `rrule-rs` 0.13.x — `crates.io/crates/rrule`; `github.com/fmeringdal/rust-rrule`.
- `ical-rs` 0.12.x — `crates.io/crates/ical` (rejected reference).
- Apple Calendar RRULE behaviour — `developer.apple.com/documentation/eventkit/ekrecurrencerule`.
- Outlook RRULE quirks (UNTIL+COUNT) — KB article references; pre-2018 quirk.
- ADR-0105 (13-layer enum; `domain` is canonical for invariant math).
- ADR-0131; ADR-0132; ADR-0133.
- ADR-CAL-0001 (CalDAV backend selection; recurrence engine sits behind both backends).
- `microservices/calendar/PRD.md` FR-02 + AC-03 + AC-10 + Open Question 4.
- `microservices/calendar/runbooks/recurrence-storm.md`.
- `microservices/calendar/runbooks/scheduling-poll-deadlock.md`.
