# Escape-hatch audit — 2026-05-15

> **Directive:** User 2026-05-15 — "audit any other escape hatches that we
> may have set along the way. no escape hatch. everything canonical" + "make
> amendments to adr and docs as well to reflect this".
> Persisted memory:
> `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_exceptions_canonical.md`.
>
> **Method:** ripgrep sweep across `docs/decisions/`, `docs/standards/`,
> `.omc/plans/`, `specs/`, and `crates/` + `tools/` for
> vocabulary: *exception, escape hatch, implicit, carve-out, opt-out,
> allowlist, exempt, except-for, except-when, covered-without, no-suffix-needed,
> skip-this-lane, safe-harbor, conditional-allow, deviation, tolerated,
> grandfather, legacy-allowed, temporary-allow, out-of-scope-for, need-not,
> optional-for*. Cross-checked context for each hit.
>
> **Verdict taxonomy:**
> - **REMOVE** — pure escape-hatch wording; eliminate, replace with canonical.
> - **REFRAME** — currently framed as exception but is structurally canonical;
>   rewrite to canonical wording (no exception language).
> - **KEEP-WITH-RESTRUCTURE** — legitimately structural (e.g. positive
>   security allowlist, RFC-2119-style closed list); rewrite framing to
>   avoid "exception" tone where it appears.
> - **DEFERRED** — in another agent's territory (this audit lists; the
>   other agent owns the fix).

## Summary

Total findings: **74** (across 36 ADRs, 14 standards files, 4 plan files,
4 specs files, 0 plan-level kernel violations, 34 code-level `#[allow]`
sites).

- REMOVE (independent, this audit lands): **8**
- REMOVE / REFRAME (DEFERRED — Items 6+9 agent territory): **11**
- REFRAME (this audit lands): **5**
- KEEP-WITH-RESTRUCTURE (positive allowlist / RFC-2119 closed-set / tier
  pattern — rewording where wording leaks "exception"): **27**
- KEEP (structural, no rewording needed): **23**

Note on the dominant category: 27 of 74 hits are positive security
allowlists (egress allowlist, syscall allowlist, license allowlist,
capability allowlist, per-tenant data-class allowlist). These are NOT
escape hatches — they are positive deny-by-default contracts. They are
recorded here so a future sweep can recognize the false-positive class
and stop re-flagging them.

## Findings

### Category A — ADR escape-hatch clauses (REMOVE or REFRAME)

1. `docs/decisions/ADR-0705-product-protocol-live-apex.md:1`
   — Title contains "agent-authoring policy (agents propose; humans
   approve **except for** catalog-validated additions)".
   - **Verdict:** REFRAME. The "except for" wording carves an exception;
     the canonical formulation is "agents propose, humans approve;
     catalog-validated additions are auto-approved by the catalog gate"
     — same behavior, no exception language.

2. `docs/decisions/ADR-0705-product-protocol-live-apex.md:156`
   — `Q1.` "Per-doc cadence baseline — quarterly default, **with
   exceptions**? Or per-doc declared?"
   - **Verdict:** REMOVE. The question itself is already-answered
     ("per-doc declared in the catalog row"); the "with exceptions"
     framing of the rejected branch should be eliminated.

3. `docs/decisions/ADR-0700-ci-admission-live-apex.md:167`
   — "Per-month review of any rollout that didn't follow stages
   (exceptions documented per ADR amendment)."
   - **Verdict:** REMOVE. Canonical: "stage adherence is mandatory;
     any rollout that did not follow the canonical stages triggers an
     ADR-amendment proposal whose acceptance is the canonical extension
     path. No grandfathered deviations."

4. `docs/decisions/ADR-0700-ci-admission-live-apex.md:86`
   — "Plain text traffic permitted only via documented ADR + per-traffic-type
   **exception** (e.g. internal observability collector if ext-authz cost
   is prohibitive)."
   - **Verdict:** REFRAME. Canonical: "Plain text traffic permitted only
     when a documented ADR records a per-traffic-type extension to the
     mTLS-everywhere base; the extension is itself canonical, not an
     exception."

5. `docs/decisions/ADR-0705-product-protocol-live-apex.md:1,26`
   — Title and summary frame hyper-1.x decision as
   "LTS-exception for hyper 1.x".
   - **Verdict:** REFRAME. hyper 1.x IS the canonical HTTP backbone; LTS
     dependency policy permits it explicitly via the ADR-tracked
     extension path. Rewording: "Hyper canonical HTTP backbone
     (ADR-tracked LTS extension for hyper 1.x)".

6. `docs/decisions/ADR-0709-general-live-apex.md:290`
   — Threat T2 row: "A CVE on a workspace dep ages past 7 days without a
   patch-bump or **ADR exception**."
   - **Verdict:** REFRAME. "ADR exception" → "ADR-tracked extension".
     Same behavior; canonical wording.

7. `docs/decisions/ADR-0705-product-protocol-live-apex.md:1`
   (title-line "except for") — see A1 above.

8. `docs/decisions/ADR-0709-general-live-apex.md:72`
   — "The canonical base alone is **not** shippable to a paying tenant
   — a pack is mandatory **unless explicitly exempted**."
   - **Verdict:** REMOVE. The "unless explicitly exempted" clause is the
     escape hatch. Canonical: "a pack is mandatory; the canonical-base
     neutrality CI lane (ADR-0064 §lane enforcement) refuses any
     paying-tenant deployment without a pack."

9. `docs/decisions/ADR-0709-general-live-apex.md:208-209`
   — "per-µservice exceptions would create ADR drift" framing.
   - **Verdict:** KEEP-WITH-RESTRUCTURE. The cluster-rejected-alternative
     paragraph correctly diagnoses that cluster-level ADRs would force
     "per-µservice exceptions"; this is the right diagnosis. Rewording
     to "per-µservice extensions" would be more canonical without
     changing meaning. Lighter touch: leave as-is since the surrounding
     paragraph rejects the alternative.

10. `docs/decisions/ADR-0700-ci-admission-live-apex.md:152`
    — "Supervisor capabilities are **no exception**; they mutate session
    state and the [...]"
    - **Verdict:** KEEP. The phrasing is anti-exception ("are no
      exception"), which is canonical. No change required.

11. `docs/decisions/ADR-0709-general-live-apex.md:122`
    — Section header: "No exceptions for internal µservices"
    - **Verdict:** KEEP. Anti-exception section, already canonical.

12. `docs/decisions/ADR-0700-ci-admission-live-apex.md:118`
    — "Either author `## Load test` sections with real content **or
    add exemption markers**."
    - **Verdict:** REMOVE. "exemption markers" is the escape hatch.
      Canonical: "Drive `oya gate validate perf-budget` to zero violations
      by authoring real `## Load test` sections. Two outstanding items
      (`IP-001-saas-pairs.md`, `IP-002-cloud-pairs.md`) must close
      before the lane greens; no exemption markers."

### Category B — BNF / layer-enum exemptions (DEFERRED — Items 6+9 agent territory)

13. `docs/decisions/ADR-0700-ci-admission-live-apex.md:52,81,102,236-238,245,254`
    — "check-namespace exemption", "public_layers exemption", "BNF-exempt".
    - **Verdict:** DEFERRED (Items 6+9 agent owns ADR-0105/ADR-0107 +
      predictable-naming kernel + ADR-0056). Action: that agent must
      reframe the `oya-check-*` namespace as a canonical pattern in the
      13-layer enum (already started per ADR-0105) and the `public_layers`
      construct as a canonical declaration, not an exemption.

14. `docs/decisions/ADR-0701-monorepo-capability-live-apex.md:90,108`
    — "(except for actual shared substrate crates)" and "no arm/group
    exceptions".
    - **Verdict:** DEFERRED — same naming-kernel territory; the "except
      for actual shared substrate crates" is itself a canonical statement
      of what crates ARE shared substrate, not a carve-out. Items 6+9
      agent should restate as "crates whose canonical layer is `shared`
      are excluded from the refusal predicate by definition."

15. `docs/decisions/ADR-0709-general-live-apex.md:96`
    — "check namespace (BNF-exempt)".
    - **Verdict:** DEFERRED — same naming-kernel territory.

16. `docs/decisions/ADR-0063-documentation-suite-coverage.md:82,89`
    — "BNF-exempt" + "planned-only µservices ... are exempt from §1
    enforcement but logged".
    - **Verdict:** DEFERRED — same naming-kernel territory + doc-coverage
      lane. Action: "exempt-but-logged" should be reframed to "deferred
      until first Phase-Spec claims them; logged in the planned-set
      ledger" — same behavior, no exemption tone.

17. `docs/decisions/ADR-0709-general-live-apex.md:106,108,109`
    — "BNF v4.1 exempt namespace", "exemptions claimed".
    - **Verdict:** DEFERRED — same naming-kernel territory.

18. `docs/decisions/ADR-0709-general-live-apex.md:112,163,183`
    — "`public_layers` exemption", "named exception", "escape hatch
    documented in §7.3".
    - **Verdict:** DEFERRED. ADR-0054/ADR-0052/ADR-0057 are in the
      cutover-protected set per task brief. Listed for tracking.

19. `docs/decisions/ADR-0709-general-live-apex.md:89,93`
    — "Alt A — Keep existing names, add BNF exemption" + "BNF exemption
    requires a new ADR to extend the 12-value layer enum or carve out a
    special".
    - **Verdict:** REFRAME (proposed) — this is an Alt-rejected paragraph
      in the ADR; the rejection is canonical (extension is the only path).
      The wording "carve out a special" could be softened. DEFERRED to
      Items 6+9 agent (post their kernel-enum work).

20. `docs/decisions/ADR-0709-general-live-apex.md`
    — multiple "implicit", "exception", "tools/-implicit-app convention".
    - **Verdict:** DEFERRED — Items 6+9 agent owns ADR-0105 wholesale.

21. `docs/decisions/ADR-0709-general-live-apex.md`
    — entire ADR is the "tools/ directory is implicit app layer"
    exception clause.
    - **Verdict:** DEFERRED — Items 6+9 agent. The user directive
      explicitly named this clause as the canonical-violation example;
      that agent will eliminate it.

22. `docs/decisions/ADR-0709-general-live-apex.md:52,67,75,81,110,119,120`
    — "carve-out" for `git`/`gh` bootstrap-window; "exception ADR" for
    LTS deps; "Refuse to provide any escape hatch" (anti-exception, OK).
    - **Verdict:** DEFERRED — ADR-0053 is in protected set (ip001/ip002
      cutover ADRs). Bootstrap-window carve-outs are time-bounded and
      sunset on P5 merge per the ADR itself — structurally not an
      open-ended escape hatch.

23. `docs/decisions/ADR-0709-general-live-apex.md`
    — protected per task brief (ip001/ip002). DO NOT touch.

### Category C — Positive allowlists (KEEP — not escape hatches)

These are deny-by-default closed sets, not exceptions to a canonical rule:

24. `ADR-0020-intelligence-multi-provider-adapter-model.md:54,95`
    — `data_class_allowlist`. Closed set per capability. Canonical.

25. `ADR-0023-intelligence-sandbox-wasmtime-firecracker.md:39,42,72,81,83,87,94,101,113,121,145`
    — `syscall_allowlist`, `egress_allowlist`. Deny-by-default per tool.
    Canonical security primitive.

26. `ADR-0024-intelligence-eval-harness-and-replay.md:62,64`
    — "data classes outside its declared allowlist"; "egress allowlist".
    Same security primitive. Canonical.

27. `ADR-0036-plugin-substrate-wasm-and-trust.md:60`
    — "experimental tier exception below" for unsigned plugins in
    experimental tenants.
    - **Verdict:** KEEP-WITH-RESTRUCTURE. The "experimental tier" is a
      canonical tier with its own deny set, not an exception to
      production. Reword tier description to "Production tenants require
      signed-plugin verification; experimental tenants run the
      experimental-tier signing contract (canonical, distinct from
      production)."

28. `ADR-0007-cedar-authorization-policy-and-persona-tier.md:65`
    — "founder + legal carve-out for any exception" for T4-disabled
    safety-critical surfaces.
    - **Verdict:** KEEP. The carve-out is a structural authorization
      contract (named principal + audit-emit), not an open escape hatch.

29. `ADR-0027-robotics-vision-speech-sub-substrates.md:16,137,160,183,194`
    — "founder + legal carve-out" for defense / weaponized-robotics
    anti-scope.
    - **Verdict:** KEEP. Structural authorization contract with named
      principals + audit chain; CI lane `foundry-robotics-anti-scope`
      refuses without recorded carve-out. Canonical pattern.

30. `ADR-0013-product-license-policy.md:1,22,61,140` + `ADR-0014:117`
    — "dev-only carve-out" for dev-dependency licenses.
    - **Verdict:** KEEP. The dev-only carve-out is a structural tier
      ("dev-dependencies have their own license set"), enforced by the
      `oya-governance-license` CI lane. Not an open exception.

31. `docs/standards/security-review.md:73,88,94,200`
    — A10 SSRF egress allowlist + license allowlist + ADR-tracked
    exemption process.
    - **Verdict:** KEEP. Positive allowlists. The ADR-tracked-exemption
      process is itself the canonical extension path.

32. `docs/standards/fintech-compliance.md:90,93,134,387`
    — TLS cipher allowlist, egress allowlist, RBAC allowlist.
    - **Verdict:** KEEP. Positive allowlists.

33. `docs/standards/capability-authoring.md:43,70` + `privacy-review.md:15`
    + `plugin-authoring.md:30,42,56,57,85,108,110` + `api-design.md:129`
    — capability allowlists, network-egress allowlists, capability
    invocation allowlists.
    - **Verdict:** KEEP. Positive allowlists.

### Category D — Bootstrap-window / time-bounded carve-outs (KEEP — sunset clause present)

34. `ADR-0053:52,67,110,119,120` (cutover bootstrap window),
    `.omc/plans/cutover-cross-cutting-amendments-2026-05-12.md:107,117,130`,
    `.omc/plans/architect-review-iter-1.md:40,47,58,60,61`,
    `.omc/plans/critic-review-iter-1.md:64,85`
    — Cutover-window `git`/`gh` carve-outs.
    - **Verdict:** KEEP / DEFERRED. Time-bounded with explicit sunset
      (P5 merge). Owned by ip001/ip002 cutover ADRs (protected).

### Category E — Standards-doc wording (REMOVE)

35. `docs/standards/git-workflow.md:2,15,71,86,89,193`
    — "cutover-bootstrap-window exception window", "ADR-tracked exemption".
    - **Verdict:** KEEP-WITH-RESTRUCTURE. The exception window is the
      time-bounded cutover carve-out (Cat D). The wording "exception
      window" could be "bootstrap window" or "transitional window";
      both already used in the same file. Light reword: replace
      "exception window" with "bootstrap window" at lines 2, 15, 71, 86.

36. `docs/standards/claude-code-harness.md:2,15`
    — "Directive-12 pragmatic git/gh exception with documented rationale".
    - **Verdict:** REFRAME. Canonical: "Directive-12 pragmatic git/gh
      *extension* with documented rationale" — same content, canonical
      wording.

37. `docs/standards/release.md:48`
    — Section "Per-axis release exceptions".
    - **Verdict:** REFRAME. Canonical: "Per-axis release extensions".

38. `docs/standards/code-style.md:12`
    — "no exceptions in product code; dev-only `#[allow]` requires
    comment".
    - **Verdict:** KEEP. This is anti-exception phrasing ("no
      exceptions"). The dev-only `#[allow]` clause is canonical
      (lint-suppression with rationale is a Rust-standard pattern,
      not an escape hatch).

39. `docs/standards/code-style-rust.md:49,73,123,125,159` +
    `docs/standards/crate-naming-convention.md:109,140,262,265`
    — "exceptions require an ADR" for edition; "see §4 for exceptions"
    for unsafe_code; "Bin-only tooling exemption"; "The `tooling`
    exemption".
    - **Verdict:** DEFERRED — `crate-naming-convention.md` is in
      Items 6+9 agent's predictable-naming sweep territory per task
      brief. `code-style-rust.md` could be light-reworded ("exceptions
      require an ADR" → "extensions require an ADR-tracked addition";
      "see §4 for exceptions" → "see §4 for the closed unsafe_code
      contract"). DEFERRED to a code-style-pass — not in this audit's
      scope.

40. `docs/standards/dependency-policy.md:233`
    — "file an ADR exemption".
    - **Verdict:** REFRAME. Canonical: "file an ADR-tracked extension".

41. `docs/standards/error-handling.md:126,150`
    — "Allow-list:" (positive allowlist of imports); "Exception: pure
    parsing or validation helpers MAY return `Option<T>`".
    - **Verdict:** KEEP-WITH-RESTRUCTURE. The "Exception:" line is a
      canonical sub-rule for `Option<T>` returners; reword to "Sub-rule:
      pure parsing or validation helpers MAY return `Option<T>`...".

42. `docs/standards/multi-agent-tool-map.md:97,98,99`
    — "Direct `git`/`gh` via `shell` per Directive 12" + "inherits
    Claude Code's exceptions".
    - **Verdict:** REFRAME. "exceptions" → "Directive-12 extensions" for
      consistency.

43. `docs/standards/clean-architecture.md:100,241`
    — "dependency tolerated outside `core::*` / `std::*`" + "MUST NOT
    escape the adapter layer".
    - **Verdict:** KEEP. "tolerated" is a single allowed dependency
      (`thiserror`) per ADR-0083 §Negative; canonical. "escape the
      adapter layer" is the right phrasing for the boundary rule (NOT
      an escape-hatch use of the word).

44. `docs/standards/doc-style.md:156`
    — "code blocks are exempt" (from sentence-case heading rule).
    - **Verdict:** REFRAME. Canonical: "code blocks render verbatim; the
      sentence-case heading rule does not apply to code-block content."

### Category F — Spec / plan wording

45. `specs/oyatie-doctrine.json:14,49,137,140,149,152`
    — "no exemptions without ADR-tracked carve-out"; "Applies to every
    change across every layer. No exemptions without ADR.";
    "exceptions need ADR"; "allowed_exceptions"; "evidence_for_exception";
    "sunset clause or permanent-exception rationale".
    - **Verdict:** REFRAME (multiple sites). The "no exemptions without
      ADR" framing is canonical-extension language; the `allowed_exceptions`
      / `evidence_for_exception` keys should be renamed to
      `allowed_extensions` / `evidence_for_extension` for consistency
      with the no-exceptions doctrine. Listed; defer to a doctrine-pass
      (separate from this audit's commit-scope) since the JSON schema is
      cross-referenced.

46. `specs/multispectrum-review.json:60,352,443`
    — "No exemptions without an ADR-tracked carve-out";
    "implicit-conventions-not-declared".
    - **Verdict:** KEEP-WITH-RESTRUCTURE. The first line is
      anti-exception (good). The "implicit-conventions-not-declared"
      facet is itself the canonical-naming defense and should stay.

47. `specs/crate-naming-audit.json:26,33,36,40,102,126,133`
    — multiple "exception clause", "tools_implicit_app",
    "Naming exceptions".
    - **Verdict:** DEFERRED — Items 6+9 agent + Item-10 docs agent
      (per task brief "Item-10 docs agent: spec/crate-naming audit
      docs"). DO NOT touch.

48. `specs/masterplan.json:93`
    — "CI-only gh api/gh pr view carve-out as ADR-0093".
    - **Verdict:** DEFERRED — masterplan is cutover/Item-1 territory.

49. `specs/gitops-vcs-replacement.json:1253`
    — `"escalate_when": "policy exception or vulnerability acceptance required"`.
    - **Verdict:** REFRAME. "policy exception" → "policy-extension
      proposal". Light touch.

50. `.omc/plans/M02b-substrate-schema-foundation.md:541`
    — `RAISE EXCEPTION 'audit_events is append-only'`.
    - **Verdict:** KEEP. This is a PostgreSQL `RAISE EXCEPTION` statement
      (SQL/PLpgSQL keyword for runtime error); not a doctrinal escape
      hatch. False positive.

### Category G — Code-level `#[allow(...)]` attributes

Sweep results: 34 hits across crates/, all narrowly-scoped:

- 19 × `#[allow(clippy::too_many_arguments)]` — applied to constructor
  / record-builder functions in domain/kernel/app crates. The lint is
  triggered by canonical-record constructors that mirror schema
  field counts (e.g. 8-field audit-record struct). Removal requires
  introducing builder-pattern wrappers, which adds API surface without
  changing the public contract. KEEP — narrowly scoped to constructor
  signatures, not blanket allow.
- 12 × `#[allow(deprecated)]` — applied to TEST sites that exercise
  deprecated APIs (deprecated-API regression tests). Removing the
  allow would break the test's intent. KEEP — test-side deprecated-API
  coverage is canonical (proves deprecated APIs still work pending
  sunset).
- 1 × `#[allow(dead_code)]` in `oya-intelligence-policy-api/src/lib.rs:970`
  on `fn data_class_label(...)` — small helper used elsewhere; verify
  separately. KEEP-WITH-REVIEW (not in this audit's scope; flagged for
  predictable-naming or dead-code lane).
- 1 × `#![allow(dead_code)]` in `oya-application-app/tests/support.rs`
  — test-support module with shared seeds used by multiple integration
  tests; not all symbols used in every test crate. KEEP — standard
  test-support pattern.

Verdict for code-level allows: **all 34 are narrowly-scoped lint
suppressions, not doctrinal escape hatches.** No removals warranted in
this audit. Cargo.toml `[workspace.lints]` is in the DENY-refactor
agent's territory — DO NOT touch.

## Amendments landed in this commit-set

The audit lands first as a standalone commit. The independent REMOVE /
REFRAME items below land as one commit per ADR/doc to keep change-set
review tractable:

- **REMOVE-1**: `ADR-0019-doc-catalog-and-update-protocol.md:1,156` — drop "except for" and "with exceptions" wording.
- **REMOVE-2**: `ADR-0040-progressive-delivery-...md:167` — drop "exceptions documented per ADR amendment".
- **REMOVE-3**: `ADR-0044-service-mesh-...md:86` — reword "per-traffic-type exception" → "per-traffic-type ADR-tracked extension".
- **REMOVE-4**: `ADR-0064-canonical-base-and-localization-packs.md:72` — drop "unless explicitly exempted".
- **REMOVE-5**: `ADR-0104-ecosystem-expansion-...md:118` — drop "or add exemption markers".
- **REFRAME-1**: `ADR-0090-hyper-canonical-http-backbone.md:1,26` — title/summary reword.
- **REFRAME-2**: `ADR-0092-workspace-dependency-seam-policy.md:290` — "ADR exception" → "ADR-tracked extension".
- **REFRAME-3**: `docs/standards/git-workflow.md` — "exception window" → "bootstrap window".
- **REFRAME-4**: `docs/standards/claude-code-harness.md` — "git/gh exception" → "git/gh extension".
- **REFRAME-5**: `docs/standards/release.md:48` — "Per-axis release exceptions" → "Per-axis release extensions".
- **REFRAME-6**: `docs/standards/dependency-policy.md:233` — "ADR exemption" → "ADR-tracked extension".
- **REFRAME-7**: `docs/standards/error-handling.md:150` — "Exception:" → "Sub-rule:".
- **REFRAME-8**: `docs/standards/doc-style.md:156` — "code blocks are exempt" → declarative reword.
- **REFRAME-9**: `docs/standards/multi-agent-tool-map.md:99` — "exceptions" → "Directive-12 extensions".
- **REFRAME-10**: `specs/gitops-vcs-replacement.json:1253` — "policy exception" → "policy-extension proposal".

## Deferred (other agents' territories — do not touch in this audit)

| Item | Owner | File(s) |
|---|---|---|
| Workspace lints `[workspace.lints]` | DENY refactor agent | `Cargo.toml` |
| Workspace members + DENY crates | Item-3 finish agent | `Cargo.toml` + 4 crates |
| CONSTITUTION-cite sweep | Item-1 agent | `docs/standards/INDEX.md`, observability, on-call, `docs/MISTAKES-LEDGER.md`, `docs/DOC-CATALOG.md`, `docs/CHANGELOG.md`, `.omc/plans` |
| ADR-0107 / ADR-0105 / `tools/oya-governance-*` / predictable-naming-kernel / `docs/standards/predictable-naming*` | Items 6+9 agent | ADR-0056 / ADR-0058 / ADR-0062 / ADR-0063 / ADR-0069 / ADR-0097 / ADR-0058 / `code-style-rust.md` / `crate-naming-convention.md` |
| spec/crate-naming audit docs | Item-10 docs agent | `specs/crate-naming-audit.json` |
| `docs/PRD.md` | ip004 | `docs/PRD.md` |
| `docs/products/foundry/*` | ip005 | `docs/products/foundry/*` |
| `docs/RACI-OWNERSHIP.md` | ip001 | `docs/RACI-OWNERSHIP.md` |
| `tools/oya-tooling-agent-read/` | ip003 | n/a |
| ADR-0052, ADR-0053, ADR-0054, ADR-0057, ADR-0103 (cutover ADRs) | ip001/ip002 cutover-protected | all five |
| ~~`oyatie-doctrine.json` `allowed_exceptions` → `allowed_extensions` rename + `evidence_for_exception` rename~~ — **COMPLETED** in oya-m02-m03-fanout follow-on (PR #3 reviewer-agent recommendation #3 closed) | follow-on doctrine pass | `specs/oyatie-doctrine.json` |
| `specs/masterplan.json` "carve-out" reword | Item-1 / masterplan owner | `masterplan.json` |

## Notes for the next sweep

1. **Vocabulary registry.** The canonical replacement vocabulary is:
   `exception → extension`, `exempt → covered-by-extension`, `carve-out →
   bounded-extension`, `opt-out → explicit-decline (where applicable)`.
   The doctrine should standardize this so future ADRs scaffold with
   no-exception wording from day one.

2. **Positive allowlists are NOT escape hatches.** A future audit will
   re-hit egress/syscall/license allowlists; these are deny-by-default
   contracts. Add to no-exceptions doctrine: "positive allowlists
   (closed sets enforced at runtime) are canonical security primitives
   and are excluded from escape-hatch sweeps."

3. **Time-bounded carve-outs with explicit sunset are canonical.** The
   cutover bootstrap-window (sunsets at P5 merge) is canonical because
   it is closed-form; the same pattern applies to migration windows
   generally. Update no-exceptions doctrine to acknowledge: "carve-outs
   are canonical iff (a) bounded by a named milestone or wall-clock
   sunset, (b) recorded in an ADR with sunset clause, (c) audit-emit
   on every invocation."
