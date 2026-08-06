---
id: ADR-0237
status: Superseded
deciders: council-architecture, council-product, council-privacy, axis-mail, axis-messenger, axis-calendar, ops-sre-reliability, ops-release-management
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0056, ADR-0105, ADR-0110, ADR-0114, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0238]
related_memory: [feedback_no_silent_regression, feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145), feedback_bominal_inheritance_precedence]
related_specs:
  - /specs/per-microservice-flat-layout.json
  - /specs/microservices/mail.json
  - /specs/microservices/messenger.json
  - /specs/microservices/calendar.json
session_context:
  authored: 2026-05-17
  parallel_session_caveat: |
    Authored in oyatie 2026-05-17 as the operational companion to ADR-0238
    (super-app expansion; originally drafted as ADR-0126 in the
    oyatie 2026-05-17 session, renumbered 2026-05-18 to avoid collision with
    dev's ADR-0126 Employment classification). ADR-0238 establishes the
    target topology; this ADR establishes how legacy `oya-connect-*`
    consumers migrate to the new flat µservices without breaking
    Hyrum's-Law-bound external behaviour.
purpose: |
  Operationalise the → 8-flat-µservice dissolution via Strangler Pattern
  (per agent-skills deprecation-and-migration SKILL.md §"Migration Patterns").
  Govern adapter-layer translation, feature-flagged traffic-shifting, zero-
  active-usage verification, code-removal sweep, and umbrella-folder retirement.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0237: dissolution — Strangler-pattern migration

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

ADR-0238 dissolves into 8 first-class flat µservices (mail, messenger,
calendar, community, social, shorts, network, anonymous). The new µservices
**already ship in parallel** under `microservices/<ms>/` per ADR-0131; the
extant legacy crates `oya-connect-{mail,messenger,calendar}-domain` (and their
planned layer fan-outs) remain in the workspace. This is the textbook
*"replacement exists; old system still exists; consumers exist on both"*
state that the agent-skills deprecation-and-migration skill (SKILL.md
§"Migration Patterns" + §"Verification") was written for.

Five facts force the Strangler approach over a Big-Bang cutover:

1. **Code is a liability.** Per the skill's first principle, every line of
   `oya-connect-*` code we keep alive costs ongoing maintenance — security
   patches, dependency updates, mental overhead. We must commit to removal,
   not indefinite parallel maintenance.
2. **Hyrum's Law.** Every observable behaviour of the legacy `oya-connect-*`
   surface — error variant ordering, timing characteristics, log line formats,
   metric label cardinality, even idempotency-key collision behaviour — is
   depended on somewhere. We cannot "just swap symbols"; we must migrate.
3. **Per `feedback_no_silent_regression`** (the workspace-wide
   no-silent-regression principle that ADRs 0067 / 0083 / 0091 / 0108 / 0114
   / 0130 / 0133 all cite), public-contract changes require deprecation
   notices + version bumps + sunset schedules. The legacy `oya-connect-*`
   crate surface IS a public contract because it appears in
   `/specs/microservices/{mail,messenger,calendar}.json` and is consumable
   by downstream sdks.
4. **The Churn Rule (skill SKILL.md §"Step 3").** If we own the
   infrastructure being deprecated, we are responsible for migrating our own
   consumers — we cannot announce deprecation and leave them stranded.
5. **Big-Bang is incompatible with ADR-0139's agentic SLO-gated promotion.**
   ADR-0139 requires sustained SLO conformance before any µservice promotes
   past dev. A single-cutover migration would have no SLO history on the new
   surface at the moment of cutover; the gate would refuse.

## Decision

The → 8-flat-µservice dissolution is migrated via the **Strangler
Pattern** as defined in the agent-skills deprecation-and-migration skill
(SKILL.md §"Strangler Pattern"). The migration proceeds through **6
sequential phases**, each gated by a concrete verification command.

### Phase 1 — New µservices ship in parallel  *(current state, 2026-05-17)*

`microservices/{mail,messenger,calendar}/` are stood up with full pack-fill
(mail 84 files, messenger 96 files, calendar in-progress) per ADR-0238. Legacy
`oya-connect-*` crates continue to serve 100% of traffic. New `oya-<ms>-*`
crates serve 0% of production traffic; they are exercised only by their own
test sets + dev-cluster canary.

**Entry gate:** ADR-0238 accepted; new µservice PRDs published.
**Exit gate:** All three of HG-MAIL, HG-MESSENGER, HG-CALENDAR pass at p99
SLOs in dev cluster sustained 7d.

### Phase 2 — Adapter layer  *(3-month soak)*

An adapter crate per legacy → new µservice is introduced:

- `oya-mail-migration-adapter` — re-exports `oya-mail-*` public types
  + ports through the legacy `oya-mail-*` symbol paths.
- `oya-messenger-migration-adapter` — same shape for messenger.
- `oya-calendar-migration-adapter` — same shape for calendar.

Each adapter follows the skill's Adapter Pattern (SKILL.md §"Adapter Pattern"):
old interface, new implementation. The legacy `oya-mail-domain` crate
becomes a `#[deprecated]` re-export shim that delegates to the adapter.

**Entry gate:** Phase 1 exit gate green.
**Exit gate:** All consumers of `oya-connect-*` compile against the
adapter-shimmed surface; `cargo nextest run --workspace` exits 0 with all
legacy + new tests passing. 3-month soak counted from this exit.

### Phase 3 — Feature-flagged traffic-shift  *(6-week canary)*

Each adapter consults a feature flag at entry-point to decide whether to
delegate to the new µservice or to the legacy in-process path:

```rust
// In oya-mail-migration-adapter
pub fn deliver_mail(req: MailDeliveryRequest) -> Result<...> {
    if feature_flags::is_enabled("oya-mail-strangler", &req.tenant_id) {
        oya_mail_outbound_smtp_usecase::deliver(req.into())
    } else {
        legacy_in_process::deliver(req)  // SUPERSEDED 2026-XX-YY
    }
}
```

Traffic shift cadence (per skill SKILL.md §"Strangler Pattern"):

| Week | New µservice share | Legacy share | Gate |
|---|---|---|---|
| 0 | 0% | 100% | Phase 2 exit gate green |
| 1 | 10% | 90% | Canary: error-rate Δ ≤ 0.1%; p99 latency Δ ≤ 5% |
| 2 | 10% | 90% | (soak) |
| 3 | 50% | 50% | Same gates as week 1 |
| 4 | 50% | 50% | (soak) |
| 5 | 100% | 0% (legacy idle) | Same gates as week 1 |
| 6 | 100% | 0% (legacy idle) | (soak); legacy receives 0 RPS sustained 7d |

Canary cadence matches ADR-0114 (canary-observability-rollback); rollback
preserves the previous traffic share atomically.

**Entry gate:** Phase 2 exit gate green; 3-month adapter soak elapsed.
**Exit gate:** New µservice carries 100% of traffic for 7 consecutive days;
legacy in-process path receives 0 RPS sustained 7d.

### Phase 4 — Zero-active-usage verification

Before any code-removal, **prove** that no active consumer depends on the
legacy `oya-connect-*` symbols (per skill SKILL.md §"Step 4: Remove the Old
System" item 1: *"Verify zero active usage (metrics, logs, dependency
analysis)"*).

Concrete verification commands (each must exit 0 / produce expected output):

```bash
# 1. Dependency-graph CI lane: zero remaining `use oya_connect_<bc>_*` outside the adapter crates
cargo run -p oya-dev-cli -- gate validate connect-legacy-symbol-zero-usage

# 2. Workspace cargo-tree: only adapter crates depend on the legacy *-domain crates
cargo tree -e normal -p oya-mail-domain    --invert | grep -v 'oya-mail-migration-adapter' | wc -l    # expect 0
cargo tree -e normal -p oya-messenger-domain --invert | grep -v 'oya-messenger-migration-adapter' | wc -l # expect 0
cargo tree -e normal -p oya-calendar-domain  --invert | grep -v 'oya-calendar-migration-adapter'  | wc -l # expect 0

# 3. Production telemetry: zero legacy code-path traversals over the prior 14 days
cargo run -p oya-dev-cli -- vcs query --metric connect_legacy_codepath_traversals_14d    # expect 0

# 4. Grep the entire codebase for stray imports outside adapter crates
rg "use oya_connect_(mail|messenger|calendar)" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
```

**Entry gate:** Phase 3 exit gate green.
**Exit gate:** All 4 verification commands above produce expected zero/empty
outputs; CI lane `connect-legacy-symbol-zero-usage` is BLOCKER green.

### Phase 5 — Code removal sweep

Per the skill (SKILL.md §"Step 4" items 2–4: *"Remove the code; Remove
associated tests, documentation, and configuration; Remove the deprecation
notices"*):

Per legacy crate family:

1. Remove `crates/oya-connect-<bc>-{domain,kernel,usecase,api,adapter*,rest,worker,sdk,app}/`.
2. Remove `crates/oya-connect-<bc>-migration-adapter/` (the adapter has served
   its purpose).
3. Remove the corresponding `[workspace] members = [...]` entries from the
   root `Cargo.toml`.
4. Remove `microservices/{mail,messenger,calendar}/deprecation-notice.md` and
   `migration-from-connect.md` (they served their purpose).
5. Remove `/specs/microservices/{mail,messenger,calendar}.json` legacy
   pointer (it now redirects to `/specs/microservices/{mail,messenger,calendar}.json`
   — promote those files in the same ChangeSet).
6. Remove deprecation notices from CLI help text (`oya vcs --help` no longer
   prints "(connect-* family deprecated)" hints).
7. Emit an ADR-0114-shaped post-removal observability check: alert if any
   prod metric named `oya_connect_*` is still emitted 7 days post-removal.

Each removal ships as one ChangeSet per legacy crate family — 3 ChangeSets
total (mail, messenger, calendar). Each is reviewed via the multispectrum
review per docs/AGENTS.md.

**Entry gate:** Phase 4 exit gate green.
**Exit gate:** All 3 removal ChangeSets merged; `cargo build --workspace`
exits 0; no `oya_connect_*` symbol resolves anywhere in the workspace.

### Phase 6 — umbrella µservice retirement

When the LAST of the 8 sub-µservices (per ADR-0238's retirement trigger:
HG-MAIL, HG-MESSENGER, HG-CALENDAR, HG-COMMUNITY, HG-SOCIAL, HG-SHORTS,
HG-NETWORK, HG-ANONYMOUS — all green at p99 SLO sustained 30d) crosses its
own Phase 5 exit, the umbrella µservice retires:

1. Delete `microservices/connector/` folder (this RETIREMENT-PLAN.md and any
   sibling artifacts in it).
2. Remove the `connect-umbrella-retirement-readiness` CI lane from
   `.github/branch-protection.yaml` (per ADR-0238 §Operational).
3. Update `docs/architecture/product-graph.md` and `product-graph.html` —
   strip the "Connect" umbrella node, keep the 8 children as first-class
   nodes.
4. Emit a final registry/placeholder-debt/adr-follow-ups.yaml#connect-umbrella-retirement-marker marker ADR closing the
   cycle (separate ADR; this ADR-0237 does not pre-author it).

**Entry gate:** All 8 HG-<MS> gates green at p99 SLO sustained 30d AND all
extant Phase 5 exits green (mail / messenger / calendar code removed).
**Exit gate:** No `microservices/connector/` folder, no `oya-connect-*` symbol,
no `/specs/microservices/*.json` file.

## Alternatives Considered

### (a) Big-Bang cutover

- **Pros**:
  - Fastest calendar time — one ChangeSet, one cutover date.
  - No adapter crate to author + maintain for 3 months.
  - No feature-flag plumbing.
- **Cons**:
  - Zero SLO history on new surface at moment of cutover → ADR-0139 gate
    refuses promotion.
  - Hyrum's-Law-bound external consumers break invisibly — every undocumented
    behaviour we miss is a P1 page.
  - Rollback is all-or-nothing; partial rollback impossible.
  - Violates skill SKILL.md §"Step 3: Migrate Incrementally" outright.
- **Rejected** because (i) it is incompatible with ADR-0139 and (ii) it
  violates the no-silent-regression principle on at minimum the timing /
  error-variant axes — we cannot prove behavioural parity without a soak.

### (b) Indefinite parallel maintenance

- **Pros**:
  - Zero migration risk — both surfaces remain forever.
  - External consumers never break.
- **Cons**:
  - Code is a liability (skill SKILL.md §"Core Principles" #1). Every line
    of `oya-connect-*` we keep alive costs ongoing maintenance — security
    patches, dependency updates, onboarding overhead, doubled test surface.
  - Zombie code (skill SKILL.md §"Zombie Code") — within 6 months no team
    actively maintains the legacy path; vulnerabilities accumulate silently.
  - "Two systems doing the same thing is double the maintenance, testing,
    documentation, and onboarding cost" (skill SKILL.md §"Common
    Rationalizations").
- **Rejected** outright. The skill is explicit that this rationalization
  ("We can maintain both systems indefinitely") is a failure mode, not a
  strategy.

### (c) Adapter-only without removal

- **Pros**:
  - Migration surface is hidden behind the adapter forever; external
    consumers never need to change imports.
  - Zero risk of "we missed a consumer" because the legacy symbol paths
    still resolve.
- **Cons**:
  - The legacy crate names persist forever in the workspace member list, in
    `cargo tree`, in onboarding docs, in `/specs/microservices/*.json`.
  - Zombie-code accumulation: nobody owns the adapter once the Strangler
    canary is at 100%, but it cannot be removed.
  - Violates skill SKILL.md §"Step 4" (removal is a required step, not
    optional).
  - The `microservices/connector/` umbrella folder never retires →
    ADR-0238 §"umbrella retires when..." trigger is unreachable.
- **Rejected** because keeping the adapter forever is the textbook
  Zombie-Code anti-pattern (skill SKILL.md §"Zombie Code").

### (d) Strangler with adapter + removal  ← **CHOSEN**

- **Pros**:
  - Follows the skill's prescribed pattern verbatim.
  - Per-canary-step SLO + error-rate gates catch Hyrum's-Law regressions
    before they reach 100% traffic.
  - Removal is a required terminal step → no zombie code.
  - Compatible with ADR-0139's agentic SLO-gated promotion (canary
    accumulates SLO history before cutover).
  - Compatible with ADR-0114's canary-observability-rollback (the same
    canary mechanism gates both this migration and ongoing releases).
- **Cons**:
  - 6–12 month total migration window.
  - Adapter crates must be authored + maintained for the duration.
  - Feature-flag plumbing required.
- **Accepted** because the cons are bounded calendar time, while the
  alternatives' cons are unbounded liability.

## Consequences

### Positive

- **No silent regression.** Each canary step gates on error-rate and p99
  latency deltas; any Hyrum's-Law-bound behavioural drift trips the gate
  before it reaches 100% traffic.
- **Terminal-state cleanliness.** Phase 5 + Phase 6 guarantee no zombie
  code, no zombie µservice folder, no zombie spec pointers.
- **Per-consumer migration ownership (Churn Rule).** Each migration-path
  consumer's owning axis is responsible for migrating their own imports;
  the legacy-owning axes (axis-mail / axis-messenger / axis-calendar) own
  the adapter authoring + canary gating; nobody migrates from the legacy
  surface without being explicitly migrated by a known owner.
- **Verifiable progress.** Every phase has a concrete exit gate verified
  by command, not by assertion.

### Negative

- **6–12 month total migration window.** Each legacy crate family's
  Strangler completes in ~5–7 months; the 8-µservice umbrella retirement
  (ADR-0238's trigger) requires the 5 net-new µservices to *also* reach
  HG-<MS> green at p99 SLO sustained 30d, which may stretch to 12 months
  from this ADR.
- **Adapter authoring + maintenance cost.** 3 adapter crates +
  feature-flag plumbing + canary observability hooks per crate. ~2
  engineer-weeks per crate up-front + ~0.5 engineer-week/month maintenance
  for the 6-month soak per crate.
- **Hyrum's-Law-bound consumers must migrate.** Per
  `feedback_no_silent_regression`, every removed `oya-connect-*` symbol
  carries a deprecation notice + sunset schedule. External consumers
  reading `/specs/microservices/*.json` get a 6-month sunset window.

### Migration cost quantification

| Cost class | Quantity | Mean per-unit cost | Total |
|---|---|---|---|
| Adapter crate authoring | 3 | ~2 engineer-weeks | ~6 engineer-weeks |
| Feature-flag plumbing | 3 | ~0.5 engineer-week | ~1.5 engineer-weeks |
| Canary observability hooks | 3 | ~0.5 engineer-week | ~1.5 engineer-weeks |
| Adapter maintenance (6-mo soak each) | 3 × 6 mo | ~0.5 engineer-week/mo | ~9 engineer-weeks |
| Removal ChangeSets | 3 | ~1 engineer-week | ~3 engineer-weeks |
| Umbrella retirement ChangeSet | 1 | ~1 engineer-week | ~1 engineer-week |
| Verification CI lanes | 2 (zero-usage + retirement-readiness) | ~1 engineer-week | ~2 engineer-weeks |
| **Total** | | | **~24 engineer-weeks** spread across 6–12 calendar months |

(Per skill SKILL.md §"Common Rationalizations": *"Compare migration cost to
ongoing maintenance cost over 2–3 years."* Ongoing maintenance of three
parallel legacy crate families over 2 years ≫ 24 engineer-weeks.)

### Operational

- **New CI lanes** (both registered in `.github/branch-protection.yaml`):
  - `oya-governance-connect-legacy-symbol-zero-usage` (REPORT-ONLY until
    Phase 3 exit; BLOCKER from Phase 4 onward).
  - `oya-governance-connect-umbrella-retirement-readiness` (REPORT-ONLY
    until all 8 HG-<MS> gates green; BLOCKER for the umbrella-retirement
    ChangeSet).
- **Deprecation notices** rendered in:
  - `microservices/{mail,messenger,calendar}/deprecation-notice.md` (formal
    skill-template notice).
  - CLI help text — `oya vcs --help` prints a `(connect-* family
    deprecated; see microservices/<ms>/migration-from-connect.md)` hint
    until Phase 5 removal.
  - `/specs/microservices/{mail,messenger,calendar}.json` `deprecated`
    field set to `true`; `replacement_path` field points to
    `/specs/microservices/<ms>/<ms>.json` (to be promoted in Phase 5).
- **Per-microservice migration owners** (Churn Rule):
  - mail Strangler → axis-mail.
  - messenger Strangler → axis-messenger.
  - calendar Strangler → axis-calendar.

## Clean Architecture Impact

| Lane | Impact | Action |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Preserved | adapter crates live at the `adapter` layer; legacy `-domain` re-exports do not violate layer rules |
| `per-microservice-layout` (ADR-0131) | Not affected (adapters live under `crates/`, not under `microservices/<ms>/`) | none |
| `connect-legacy-symbol-zero-usage` (NEW) | New REPORT-ONLY→BLOCKER | refuses Phase 5 removal until usage is zero |
| `connect-umbrella-retirement-readiness` (NEW) | New REPORT-ONLY→BLOCKER | refuses Phase 6 removal until all 8 HG-<MS> green |

## Verification

Per the skill SKILL.md §"Verification" checklist:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  Each HG-<MS> gate accepting at p99 SLO sustained 30d (Phase 1 + ADR-0238).
- [ ] **Migration guide exists with concrete steps and examples.**
  `microservices/{mail,messenger,calendar}/migration-from-connect.md` per
  this ADR cycle.
- [ ] **All active consumers have been migrated** (verified by metrics /
  logs / dependency analysis). Phase 4 verification commands all exit 0.
- [ ] **Old code, tests, documentation, and configuration are fully removed.**
  Phase 5 removal sweep completed; `cargo build --workspace` exits 0; no
  `oya_connect_*` symbol resolves.
- [ ] **No references to the deprecated system remain in the codebase.**
  `rg "oya_connect_" --type rust` produces zero hits outside historical
  ADR / RETIRED.md / git-log surfaces.
- [ ] **Deprecation notices are removed** (they served their purpose).
  Phase 5 sweeps `microservices/{mail,messenger,calendar}/deprecation-notice.md`
  and `migration-from-connect.md`.

## References

- ADR-0056: BNF v4.1 naming.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0114: Canary observability + rollback.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0238: super-app expansion into 8 flat µservices (target topology).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward-policy.
- ADR-0133: Industry best-practice conformance program.
- `feedback_no_silent_regression.md` — workspace-wide no-silent-regression principle (Linus-style; cited by ADRs 0067 / 0083 / 0091 / 0108 / 0114 / 0130 / 0133).
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern, Adapter Pattern, Churn Rule, Verification checklist.
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- `/specs/microservices/{mail,messenger,calendar}.json` — legacy reference pointers (sunset at Phase 5).
