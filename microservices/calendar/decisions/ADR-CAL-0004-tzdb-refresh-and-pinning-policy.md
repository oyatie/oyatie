---
id: ADR-CAL-0004
status: Accepted
date: 2026-05-17
microservice: calendar
deciders: axis-calendar, council-architecture, ops-sre-reliability, council-privacy
owner: axis-calendar + ops-sre-reliability
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-CAL-0002
related_artifacts:
  - microservices/calendar/PRD.md (FR-11; Open Question 2; §Performance — DST + jurisdiction rules)
  - microservices/calendar/runbooks/timezone-db-refresh.md
  - microservices/calendar/runbooks/tzdb-rollback.md
  - microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml
purpose: |
  Close PRD-calendar Open Question 2 — time-zone source-of-truth choice
  (chrono-tz vs ICU4X) — and define the IANA tzdb refresh cadence,
  per-tenant pinning policy, and rollback procedure when a release
  introduces a regression for a tenant's pack.
---

# ADR-CAL-0004: IANA tzdb refresh + per-tenant pinning policy — chrono-tz LTS pin; cluster-default refreshed within 30d of upstream; per-tenant override allowed for regulated sectors

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The `calendar` µservice's correctness depends on accurate time-zone
data: DST rules, jurisdiction-rule changes, historical offsets, and
new tz introductions all flow through the IANA Time Zone Database
("tzdb" / "tzdata"). Per PRD-calendar FR-11, every event carries an
IANA tz string (e.g., `Asia/Seoul`); per AC-08, encryption-at-rest
applies to event content but NOT to the tz string itself (the tz
string is metadata, not content). Per the recurrence engine
(ADR-CAL-0002), DST spring-forward / fall-back rules are RFC-5545-
strict, which means they are tzdb-strict.

IANA releases tzdb on an "as needed" cadence — typically 4–8 releases
per year — driven by national-government rule changes (e.g., Korea's
abolition of summer time in 1988; Russia's removal of DST in 2011;
Lebanon's last-minute DST change in 2023 which broke many calendar
systems). Each release is named `YYYYx` (e.g., `2024a`, `2024b`).
Concrete release stream: `data.iana.org/time-zones/releases/`.

Two Rust-side options for consuming the tzdb:

1. **`chrono-tz`** — vendored tzdb-data; static compile-time inclusion;
   `chrono` integration; widely-used. Releases of `chrono-tz` track
   IANA releases with ~1–4 week latency. ~2 MiB binary-size impact.
2. **ICU4X `icu_calendar` + `icu_timezone`** — modern ICU-based tz
   handling; supports a richer locale + calendar surface (Hijri,
   Buddhist, Japanese imperial, etc.); data layer is more flexible
   (vendored or runtime-loaded). Releases of ICU4X track IANA with
   variable latency depending on the data-layer choice.

Performance budget: tz lookup is on the agenda-render hot path; PRD
performance target is agenda render p95 ≤ 250ms. tz lookup must be
sub-millisecond at p99 to leave headroom for the rest of the render.

Regulatory considerations:
- **KR pack**: Korea has been DST-free since 1988; tzdb correctness
  matters for cross-tenant queries spanning Korea + DST-using regions.
- **EU pack**: EU is contemplating DST abolition; rule may change
  mid-2026 or later; tzdb updates will land mid-quarter.
- **US healthcare pack**: HIPAA appointment scheduling requires
  patient-facing times to be unambiguous; DST transitions at 02:00
  local are a known patient-confusion point.
- **JP pack**: Japan has no DST since 1948; tzdb is stable.
- **AE / KSA packs**: ISO calendars + Hijri overlay; ICU4X's richer
  calendar support would help here, but only at the calendar-system
  layer, not at the tz layer.

Failure modes (catalogued in `runbooks/timezone-db-refresh.md` and the
new `runbooks/tzdb-rollback.md`):
- **Latest tzdb release introduces a bug** (rare but happened with
  `2023a` Lebanon entry). Rollback needed.
- **Tenant pinned to an older tzdb release** to preserve audit
  reproducibility for past appointments. Must support per-tenant
  override.
- **Cross-tenant queries spanning tenants pinned to different tzdb
  releases**. Must reconcile; usually use the QUERIER's pin.

## Decision

The calendar µservice ships with **`chrono-tz` 0.10.x as the primary
tz engine, with the cluster-default tzdb release refreshed within 30d
of every upstream IANA release** (the "refresh window"). Per-tenant pin
override is supported for regulated sectors (us-healthcare,
ksa, ae) where audit reproducibility requires a stable tz baseline.
Cluster-default and per-tenant pinned baselines coexist via a config
flag at the kernel layer.

Concrete bindings:

1. **Engine: `chrono-tz` 0.10.x LTS pin.** Crate dependency
   `chrono-tz = "0.10.0"` in `oya-calendar-event-store-adapter`
   (per the `TimeZoneResolver` port trait declared in
   `oya-calendar-event-store-kernel` per PRD §"Port traits"). ICU4X
   `icu_calendar` is consulted only for Hijri / Japanese imperial
   calendar overlays in pack-ae / pack-ksa / pack-jp — not for tz
   computation.

2. **Refresh cadence**: an automated poller (the
   `oya-calendar-tzdb-refresh-worker` worker, registered as a
   per-cell CronJob in `iac/helm/templates/cronjob.yaml`) polls
   `data.iana.org/time-zones/releases/` every 24h. On detecting a new
   release, it:
   - Opens a release-pin-bump ChangeSet against `dev` (per ADR-0134
     ChangeSet flow);
   - Bumps `chrono-tz` to the smallest semver that includes the new
     IANA release;
   - Runs the RFC 5545 RRULE corpus + the named DST edge-case test
     matrix (per ADR-CAL-0002 named tests) against the new tzdb;
   - On all-green, opens the ChangeSet for review;
   - On any-red, files a fixup-task and pages axis-calendar +
     ops-sre-reliability.

   **30d refresh window**: the cluster-default tzdb release MUST be no
   more than 30d behind the latest IANA release. The SLO
   `slos/tzdb-staleness-bound.openslo.yaml` enforces this as a
   correctness SLO (target = 0 days exceeding 30d; any exceedance is
   Sev-2).

3. **Per-tenant pin override**: per-tenant config key `tzdb_pin`
   (default `cluster-default`; can be set to a specific IANA release
   like `2024a`). Pinned tenants render appointments using the pinned
   release; this is audit-grade for healthcare appointment history.
   Per-tenant pins are bounded — must be no more than 12 months
   stale; tenants attempting to pin beyond 12 months receive a
   refusal with a remediation hint.

4. **Cross-tenant query resolution**: when a free/busy query crosses
   tenants pinned to different tzdb releases, the QUERIER's pin is
   authoritative (per RFC 5545 §3.3.5 floating-time semantics
   extended). The reconciler emits a
   `CrossTenantTzdbDivergenceObserved` workflow event for audit.

5. **Rollback procedure** (per `runbooks/tzdb-rollback.md`): when a
   tzdb release introduces a regression visible in the corpus or in
   tenant traffic, ops-sre-reliability opens a same-day rollback
   ChangeSet that pins `chrono-tz` to the prior LTS. Rollback is
   atomic across the cluster.

## Alternatives Considered

### A. ICU4X `icu_timezone` as the primary tz engine

- **Pros**:
  - Richer locale + calendar surface (Hijri, Buddhist, Japanese
    imperial); ideal for ae/ksa/jp/in packs.
  - Modern Unicode consortium-backed implementation.
  - Flexible data layer — vendored or runtime-loaded.
- **Cons**:
  - tzdb refresh cadence is less predictable than `chrono-tz`.
  - Heavier binary size impact (~6 MiB for the full data set vs
    ~2 MiB for `chrono-tz`).
  - Less battle-tested in production deployments than `chrono-tz`.
  - Per-tenant pin override is more complex with ICU4X's runtime-
    loaded data layer.
- **Rejected** for primary tz computation; **accepted** for
  calendar-system overlays (Hijri / imperial) in regional packs. ICU4X
  is consulted at a different layer.

### B. Bundled tzdb under our own control (vendor + freeze)

- **Pros**:
  - Maximum stability — we control every byte of the tzdb.
  - No upstream-bump risk.
- **Cons**:
  - We become responsible for every rule-change tracking — Korea, EU,
    Lebanon, Russia, every national tz change. ~5–8 changes per year.
  - No upstream maintenance share.
  - Bus-factor on whoever-tracks-tz-changes at axis-calendar.
- **Rejected** under "buy not build" — IANA + `chrono-tz` already do
  this, and well.

### C. Per-tenant pin only (no cluster-default)

- **Pros**:
  - Maximum tenant control — every tenant pins explicitly.
- **Cons**:
  - Onboarding friction — every tenant must pick a tzdb release,
    which is an expert task.
  - Cluster-default-less posture means stale tenants accumulate over
    time with no forcing function.
- **Rejected** — cluster-default + per-tenant override is the right
  default.

### D. `chrono-tz` 0.10.x + 30d cluster-default refresh + per-tenant pin override  ← **CHOSEN**

- **Pros**:
  - Battle-tested engine; sub-millisecond p99 lookups (PRD perf
    budget met by ~250×).
  - Automated refresh + corpus-validation gate catches regressions
    before promotion.
  - Per-tenant pin satisfies regulated-sector audit reproducibility.
  - 30d refresh window is responsive enough that no tenant is more
    than ~6 weeks behind IANA on the default path.
- **Cons**:
  - `chrono-tz` ~1–4 week latency from IANA — within the 30d window,
    but tight on bad-luck cadences.
  - Per-tenant pin staleness is an operator-visible concern; mitigate
    with a 12-month staleness cap.
- **Accepted** — meets the correctness bar, the performance budget,
  and the regulated-sector audit requirement.

## Consequences

### Positive

- **Correctness within 30d of every IANA release.** SLO-enforced.
- **Regulated-sector audit reproducibility.** healthcare / KSA / AE
  packs can pin tenant tzdb for ≤12mo with documented refusal beyond.
- **Performance budget met by ~250×.** `chrono-tz` p99 lookup ~4µs;
  PRD budget is ≤1ms for the tz portion of agenda render.
- **Cross-tenant divergence observable.** `CrossTenantTzdbDivergence
  Observed` event emits to audit; provides forensic trail for any
  appointment reconciliation issue.

### Negative

- **`chrono-tz` upstream cadence is a dependency.** ~1–4 week lag
  from IANA. Mitigation: 30d refresh window admits the lag; the
  refresh worker pages early if the lag approaches 21d.
- **Per-tenant pin complexity at cross-tenant query time.** The
  QUERIER's-pin-wins rule resolves the ambiguity; documented in
  `policy/event-isolation.md` and in `runbooks/timezone-db-refresh.md`.
- **Pinned tenants drift from the cluster default over time.** The
  12-month staleness cap forces a forcing function; tenants
  approaching 12mo receive an automated reminder 30d ahead.

### Operational

- **New CI lane `oya-governance-tzdb-staleness-bound`** (BLOCKER from
  M03): refuses promotion if the cluster-default tzdb is >30d stale.
- **New CronJob `oya-calendar-tzdb-refresh-worker`**: deployed via
  `iac/helm/templates/cronjob.yaml`; runs every 24h; opens a release-
  pin-bump ChangeSet on new IANA release.
- **Runbook `tzdb-rollback.md`** documents the same-day rollback flow
  for regressions.
- **Telemetry**: `oya_calendar_tzdb_release_in_use{tenant_id,pin}`,
  `oya_calendar_tzdb_cluster_default_staleness_days`,
  `oya_calendar_tzdb_cross_tenant_divergence_total`.

### Regulatory

- **GDPR Art. 5(1)(d)** (accuracy): 30d refresh window =
  data-accuracy compliance.
- **KR PIPA Art. 16** (data accuracy): same.
- **HIPAA 45 CFR §164.502(b)** (minimum necessary): tz pin per
  tenant satisfies the appointment-history reproducibility.
- **EU regulation**: if/when the EU abolishes DST, the refresh
  worker picks up the IANA release within 30d; tenant comms are
  emitted via the standard ADR-0140 Cedar-policy overlay.
- **KSA PDPL + UAE PDPL**: Hijri overlay consulted via ICU4X at the
  calendar-system layer; tz remains chrono-tz.

## Verification

- [ ] **`chrono-tz` LTS pin in `Cargo.toml`** —
  `cargo tree -p oya-calendar-event-store-adapter | grep '^chrono-tz' | head -1` shows the pinned version.
- [ ] **Refresh worker deployed** —
  `kubectl -n calendar get cronjob oya-calendar-tzdb-refresh-worker` exists.
- [ ] **Staleness SLO authored** —
  `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml` exists.
- [ ] **Rollback runbook authored** —
  `microservices/calendar/runbooks/tzdb-rollback.md` exists.
- [ ] **`oya gate validate tzdb-staleness-bound --microservice
  calendar`** exits 0.

## References

- IANA Time Zone Database — `data.iana.org/time-zones/releases/`.
- tz mailing list — `mm.icann.org/pipermail/tz/`.
- `chrono-tz` 0.10.x — `crates.io/crates/chrono-tz`.
- ICU4X `icu_timezone` — `github.com/unicode-org/icu4x`.
- RFC 5545 §3.3.5 — DATE-TIME (TZID / floating / UTC semantics).
- RFC 6557 — Procedures for Maintaining the IANA Time Zone Database.
- Korean Standard Time (KST) history — KR tz abolition of DST in 1988.
- EU Council DST proposal — `eur-lex.europa.eu` legislative tracking.
- Lebanon 2023 DST change — public news + tzdb 2023a/2023b release notes.
- ADR-0105 (13-layer enum; `adapter` is canonical for tz lookup).
- ADR-0131; ADR-0132; ADR-0133.
- ADR-CAL-0002 (RRULE conformance; DST edge-case test matrix).
- `microservices/calendar/PRD.md` FR-11 + Open Question 2.
- `microservices/calendar/runbooks/timezone-db-refresh.md`.
- `microservices/calendar/runbooks/tzdb-rollback.md`.
- `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml`.
