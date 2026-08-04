# Implementation Plan — Finalize Lens A "Instrument" as Production Design Authority (iteration 6 — close-out)

## RALPLAN-DR Summary

### Principles
1. **Single source, polarity-correct.** The lock is the *superset*; every consumer (the HTML now, the Leptos token crate later) is a *subset* whose values must equal the lock's. Drift is a parse-detectable defect, not a style nit.
2. **Token-purity is bidirectional.** The source must be *complete* (renders every grammar rule it mandates) **and** *minimal* (zero orphan tokens). In a maximal-restraint Lens A language the source can rot upward, so both bounds are tested. An *orphan* is a token referenced by **neither** a grammar rule, a consumer, **nor** a documented superset family (complete status ramps; dark-generation role inputs). Superset-family members are legitimate by declaration — this is what lets the lock legitimately hold tokens the current HTML does not yet use **without** contradicting minimality (resolving the AC13↔D1 tension).
3. **Lock the data contract; defer only placement.** The canonical token block becomes machine-parseable *now* (stable delimiters) so the deferred buck2 `#[test]` is a ~20-line extraction-and-assert, not a re-keying. A named gate without a parseable contract is "later means never."
4. **Honest authority status.** Design-converged ≠ enforced. The artifact is LOCKED-as-design but ENFORCEMENT-PENDING until the token data lives in a tracked, buck2-wired home; `.omc/` is a *named transitional quarantine*, not a permanent authority home.
5. **Architecture-ready ≠ scaffolded.** Dark-theme readiness = theme-neutral role naming + a reserved selector + one generation rule (one sentence) — never 30 hand-typed `/* TODO */` keys that manufacture the very drift this iteration removes.

### Decision Drivers (top 3)
1. **Rust-purity ratchet + token-crate destination** — the canonical token data must be one machine-extractable source feeding both claude.ai/design and the Leptos token crate with zero hand-sync.
2. **Authority-bar honesty** (founder enforcement-layering: *flag-only = incomplete*) — you cannot certify "production authority" for a gitignored, gate-unreadable file; you can state enforcement-pending and make the future gate trivially cheap.
3. **Lens A restraint + AA floor** — every promoted token must be grammar-justified (minimality) and pass AA (the `--text-faint` g-9 fix); the state gallery exists to prove the floor on every state at *compact* density.

### Viable Options
- **Option A — Edit-in-place + lock the data contract (CHOSEN).** Promote the lock `:root` to the superset, fence it with `@tokens` delimiters, replace the dark stub with one rule, upgrade the HTML in place (six fixes + compact 7-state gallery), keep grammar inside the lock. Two files. ADR states enforcement-pending; deferred test is mechanical extraction.
  - *Pros:* shortest diff; single-source preserved; portable data contract captured now at zero infra cost; honest authority status; no new drift surface.
  - *Cons:* enforcer not running this round (mitigated by parseable contract + named follow-up); AA/aria verified manually this round.
- **Option B — Build the buck2-wired token crate + `#[test]` under `libs/` this iteration (REJECTED).**
  - *Pros:* meets the full Principle-1 authority bar immediately.
  - *Cons:* a cross-graph build-infra workstream (libs/ placement, BUCK+reindeer, generated faces, freshness/affected-set gates) blocking a *design* finalization. **Invalidation:** out of scope for a design lock; the hard part for portability (the parseable data contract) is captured by Option A at zero infra cost, leaving only mechanical test placement — exactly what may be deferred.
- **Option C — Standalone `tokens.css` extracted from the lock, HTML `@import`s it (REJECTED).**
  - *Pros:* one literal "token file."
  - *Cons:* a browser-self-contained prototype cannot `@import` a markdown fence, and a third copy (lock + css + HTML) multiplies drift — the precise antipattern. **Invalidation:** single-source; the fenced block already *is* the token file.

---

## Deliverables (exact file targets)

**File 1 — `/Users/jasonlee/Developer/oyatie/.omc/design/DESIGN-LANGUAGE.md`** (the lock)
- **D1 — Superset promotion.** Replace the `:root` block (lines 21–44) so the lock is the superset: keep all current tokens, promote from the HTML the status ramps (`--ok-7/-ink/-line`, `--warn-7/-ink/-line`, `--err-7/-ink/-line`, `--info-7/-ink/-line`, `--pend-7/-ink`), `--on-color`, `--surface-sub`. Keep `--text-faint: var(--g-9)` (lock is already correct; the HTML moves to it, not vice-versa). **Derive the promotion set mechanically** (extract HTML `:root` → diff against lock → promote the difference), not by transcribing this list. **Dedup:** `--accent-on` and `--on-color` are the same role (white ink on a saturated/accent fill), both `#fff`; collapse to the broader-used `--on-color`, delete `--accent-on`, and repoint its single usage (HTML ~line 420). One name per role.
- **D1b — Data contract delimiters.** Wrap the token block (inside the ```css fence) in stable sentinels: `/* @tokens:begin v0.1 */ … /* @tokens:end */`. This is the parseable contract the deferred `#[test]` consumes.
- **D2-kill — Dark stub dies.** Replace line 46 with the single rule: *"Role keys are theme-neutral; `:root[data-theme="dark"]` is reserved; dark VALUES are generated from the light role-set by the token crate, never hand-authored."* No enumerated dark keys.
- **D-invariant — Shared conventions.** In the Grammar section, add the consumer-⊆-lock invariant + value-equality + the **minimality ceiling** (every lock token referenced by ≥1 grammar rule, ≥1 consumer, **OR** declared in a documented superset family — complete status ramps, dark-generation role inputs; zero *true* orphans), as conventions both claude.ai/design and the token crate follow.
- **D-authority — Honesty.** Change line 5 to `Bar = production design authority (design-converged; ENFORCEMENT-PENDING)` and qualify line 1 `LOCKED v0.1` → `LOCKED v0.1 (design); enforcement-pending`. Add a short **Authority status** note: converged + design-complete, not yet gate-enforced; `.omc/` is a named transitional quarantine; the Principle-1 bar is met only when the `@tokens` block lives in a tracked buck2 home (`libs/`) with the extraction `#[test]`.

**File 2 — `/Users/jasonlee/Developer/oyatie/.omc/design/prototypes/desktop-overview-instrument.html`** (reference render)
- Fix line 58 `--text-faint: var(--g-8)` → `var(--g-9)` (value-equality with lock).
- Demonstrate all **six critic fixes** as rendered behavior (verify-then-complete each): (1) `aria-sort` on every sortable `<th>` in a real `<button>`, per-table single active sort, glyph agrees; (2) `--text-faint` = g-9; (3) Cmd-K real combobox (`aria-expanded` on trigger, `aria-activedescendant` tracking); (4) topology outer-shape==health, healthy = single neutral fill (remove redundant inner `--ok-6` dot), legend mirrors 1:1; (5) every status chip carries an icon (no color/dot-alone); (6) prod/global scope = caution treatment in tenant chip + breadcrumb leaf + destructive palette actions.
- Add the full **7-state gallery at COMPACT density**: default · hover/focus · loading-skeleton · empty-teaching · error(cause+remediation) · 403(+why) · streaming/partial. Real content, no lorem; same `--s-*` row metrics as the main instrument (not an expanded showcase).

**Deliverable 3 — Grammar shared conventions:** formalized *inside* File 1's Grammar section (no new file). ponytail: a third doc would be a fourth drift surface; the lock already owns the grammar.

**Rust-purity:** all artifacts already sit under the gitignored `.omc/design`; the work adds only `.md`/`.html` feeders. No new file outside `.omc/`.

---

## Ordered Steps
1. **Pre-compute AA.** From the lock hexes, compute WCAG contrast for every text-token × surface pair; confirm g-9 (#69727f)/#fff ≥ 4.5:1 and that no <14px text token resolves to g-8. (Proves the promoted set is AA-clean before editing.)
2. **Edit File 1:** superset block + `@tokens` delimiters (D1/D1b); kill dark stub → one rule (D2-kill); add invariant + minimality ceiling (D-invariant); fix authority framing (D-authority).
3. **Edit File 2:** `--text-faint`→g-9; verify/complete the six fixes; add the compact 7-state gallery.
4. **Verify** against AC1–AC14 (below).
5. **Confirm quarantine:** no new `.sh`/`.py`/executable; `git check-ignore .omc/design`.

---

## Acceptance Criteria (testable)
- **AC1 — Superset.** Every `--token` in the HTML `:root` exists in the lock `@tokens` block. Set-difference (HTML \ lock) = ∅.
- **AC2 — Value-equality.** For every shared token, HTML value == lock value (catches the g-8/g-9 drift). Diff = ∅.
- **AC3 — Delimiter contract.** Lock token block bounded by `/* @tokens:begin … */`/`/* @tokens:end */`; a ~20-line parse of that fence yields a name→value map with 0 parse errors and count == lock token count.
- **AC4 — aria-sort (per-table).** Each sortable `<table>` has **at most one** `<th aria-sort>` ≠ "none"; every sortable th's control is a real `<button>`; glyph matches the `aria-sort` direction. (Scoped per-table — no false-fail with multiple tables.)
- **AC5 — Cmd-K combobox.** Trigger/input has `role=combobox` + `aria-expanded`; listbox options carry ids; `aria-activedescendant` references a present option id.
- **AC6 — Topology.** Healthy nodes use a single neutral fill (no inner `--ok-6` dot); outer shape encodes health; legend entries match map shapes 1:1.
- **AC7 — Status chips.** Count(status chips) == count(chips containing an icon element + text). No chip differentiated by color/dot alone.
- **AC8 — Caution scope.** prod/global shows caution treatment in tenant chip, breadcrumb leaf, and destructive palette actions.
- **AC9 — State matrix at compact density.** All 7 states rendered with real content (no "lorem"/placeholder), using the compact `--s-*` row metrics of the main instrument — not an expanded showcase.
- **AC10 — Dark architecture (NOT enumeration).** Lock contains the theme-neutral-roles rule + reserved `:root[data-theme="dark"]` mention + "generated, never hand-authored" clause, and **zero** hand-typed dark token values.
- **AC11 — AA contrast.** Every text token, over the surface it's used on, ≥4.5:1 (<14px) or ≥3:1 (≥14px/non-text); specifically `--text-faint` (g-9)/#fff ≥4.5; no <14px text token resolves to g-8.
- **AC12 — Rust-purity quarantine.** No `.sh`/`.py`/executable added under `.omc/`; `git check-ignore .omc/design` returns the path.
- **AC13 — Minimality ceiling (superset-aware).** Every `--token` in the lock `:root` is referenced by ≥1 grammar rule (prose), ≥1 consumer (the HTML), **OR** is a declared member of a documented superset family (complete status ramps = the **closed step set** `-1,-6,-7,-ink,-line` for `--{ok,warn,err,info,pend}`, where `pend` omits `-line`; plus dark-generation role inputs). The step set is closed — a future crate may not self-declare arbitrary `-N` steps in-family. {lock tokens} \ ({referenced} ∪ {declared superset family}) = ∅. Resolves the AC13↔D1 tension: superset members feeding the future token crate are legitimate, not orphans. (Today lock == HTML, so the residual is ∅ under either reading — no behavior change now.)
- **AC14 — Authority honesty.** Lock no longer asserts unconditional "production design authority / LOCKED"; it states design-converged + enforcement-pending + `.omc/` transitional, with the `libs/` + `#[test]` condition for meeting the bar.

---

## Verification Steps (incl. AA / contrast / aria)
- **Subset + value-equality (AC1/AC2/AC3):** extract both `:root` blocks, build name→value maps, assert HTML⊆lock and equal values. *Transitionally* a grep+diff; *canonically* the deferred pure-Rust `#[test]` parsing the `@tokens` fence + HTML `:root`. Because the delimiter contract is locked now, that test is extraction, not re-keying.
- **Contrast (AC11):** for each text-token hex, compute WCAG relative luminance and ratio `(L1+.05)/(L2+.05)` against its background hex; assert thresholds; record the computed ratios in the verification log. Same calc becomes the buck2 a11y test body.
- **aria (AC4/AC5/AC7):** grep the HTML for `aria-sort`, `role=combobox`, `aria-expanded`, `aria-activedescendant`, `role=status|alert|log`; count against required elements; manually confirm glyph⇔`aria-sort` agreement and per-table single active sort.
- **Topology (AC6):** inspect topology cells (HTML ~line 1041+) — confirm no inner `--ok-6` circle on healthy cells, shape-encoded health, legend 1:1.
- **Minimality (AC13):** for each lock token name, grep usage across HTML + lock prose; any name not found must appear in the declared superset-family list (status ramps / dark-generation inputs); the residual (neither referenced nor declared) must be ∅ (else the token is used, declared, or deleted).
- **Quarantine (AC12):** `git status --porcelain .omc/design` shows only ignored/untracked feeders; `find .omc/design -name '*.sh' -o -name '*.py'` = ∅; `git check-ignore .omc/design`.

---

## ADR — Lens A "Instrument" Design Authority: data-contract-now, enforcement-pending

**Status:** Accepted (design-converged; enforcement-pending).

**Decision.** Finalize the Lens A language by making the lock the single *superset* token source, fencing its token block with a machine-readable `@tokens` delimiter (the portable data contract), replacing the dark-theme stub with one theme-neutral generation rule, and upgrading the reference render with the six critic fixes plus a compact 7-state gallery. The artifact is labeled **LOCKED v0.1 (design); enforcement-pending** — not unconditional "production authority" — and the `.omc/` location is named a transitional quarantine. The buck2-wired `#[test]` that turns the contract into an enforced gate is a named follow-up, made trivial (mechanical extraction) by locking the data contract now.

**Drivers.** Rust-purity ratchet + token-crate destination (one extractable source, zero hand-sync); authority-bar honesty (flag-only = incomplete — cannot certify a gate-unreadable file as enforced); Lens A restraint + AA floor (every token grammar-justified and AA-clean).

**Alternatives.** (B) Build the token crate + `#[test]` under `libs/` now — rejected: blocks a design lock on cross-graph build infra; the portability-critical contract is captured here at zero infra cost. (C) Standalone `tokens.css` — rejected: a self-contained prototype can't import a fence, and a third copy multiplies drift.

**Why chosen.** Shortest diff that preserves single-source, captures the portable contract immediately, removes (not adds) a drift surface by killing the dark stub, and tells the truth about enforcement status — the previous round's authority overclaim, floor-only invariant, and self-manufactured dark drift are all resolved; and the AC13↔D1 minimality-vs-superset tension is resolved by scoping minimality to exclude declared superset families.

**Consequences.** Positive: one parseable token source feeding both consumers; bidirectional token-purity (complete + minimal); a deferred gate that is a ~20-line extraction; honest authority framing. Negative/accepted: the gate does not run this iteration (AA/aria verified manually); the HTML `:root` remains a second copy (necessary for a browser-self-contained prototype) until the crate generates it.

**Follow-ups.** (1) Land the `@tokens` block + HTML `:root` parser as a pure-Rust `#[test]` in a tracked `libs/` crate (asserts AC1/AC2/AC11/AC13) — this is the act that promotes the artifact past enforcement-pending. (2) Generate `:root[data-theme="dark"]` values from the light role-set via the token crate (no hand-authoring). (3) Resolve the open accent-hue and comfortable-density questions on real screens.

— Files to edit: `/Users/jasonlee/Developer/oyatie/.omc/design/DESIGN-LANGUAGE.md`, `/Users/jasonlee/Developer/oyatie/.omc/design/prototypes/desktop-overview-instrument.html`. No new files; no artifact outside the gitignored `.omc/design`.