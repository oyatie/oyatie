# Reorg + Pipeline — Consensus Plan (rev 3, FINAL)

**Status: APPROVED SCOPE, pending execution approval.**
Date: 2026-07-26 · Scope decision: **freeze the reorg at 45%, ship 4 small fixes, return to product.**

---

## How this plan got here

Three adversarial passes ran: an independent code review of the graph-equivalence oracle, an Architect pass, and a Critic pass. Each found real defects. **Each time, the response was more machinery** — Phase 0 grew from "land two PRs" to four oracle fixes, a 508-package census, a contested 72-crate architectural ruling, and registry repair.

The Critic named it:

> *"On the axis the founder cares about, rev 2 is worse than rev 1."*

That is the finding. The reviews were technically correct on every point, and following all of them produced a larger machinery program. **The plan was the problem, not the reviews.**

rev 3 is therefore not rev 2 with the ten Critic fixes applied. It is a smaller plan that keeps only what is separable, small, and high-value — and stops.

---

## Decision

**Freeze the reorg at 45% migrated. Ship 4 fixes. Return to product.**

The 55% that has not moved stays until a product reason touches it. No census, no oracle wiring, no further batches, no `base/` creation, no `oya-check-*` ruling.

### Why freezing beats finishing

- **Finishing changes no architectural fact.** 34/112 facades violate the ADR's central rule; 887/887 `rust_library` targets are PUBLIC. A fully-migrated tree is exactly as unenforced as today's.
- **Each batch actively de-enforces.** `owning_service()` returns `None` outside `cloud`/`oya`, so every migrated crate leaves tier enforcement. 416 of 927 crates are already invisible to it.
- **The remaining cost is not relocation.** It is catalog rows, scan-root updates, baseline re-keying, face rulings, and registry repair — 69 of 73 `absorbs_current_dirs` prefixes name directories that no longer exist.
- **Dual state is already entrenched** and has not been the binding constraint on product work. 0 of 45 masterplan items are verified-done for reasons unrelated to directory layout.

### What is explicitly given up

Legacy `cloud/`, `oya/`, `libs/`, `tools/` keep crates indefinitely. `base/` is not created; `libs/oya-shared-*` (51) and `libs/oya-http-*` (8) stay put. The capability registry stays partly stale. **These are accepted, not deferred** — no phase later reclaims them.

---

## The 4 items

Each is separable, small, needs no placement work, and has a mechanically checkable exit.

### 1. facade→core ratchet — stop the erosion
The ADR's central rule (`facade` reaches `core` only through `ports`) has no gate and 34 violators.
- Frozen baseline at the current violators; `baseline-block-on-new`.
- **Key by cargo package name, not path** — a path-keyed baseline is invalidated by any future move (the precedent `tier-dependency-acyclicity-baseline.json` keys by path and has exactly this fragility).
- **Exit:** RED fixture proves a NEW violation is born-blocking; a dynamic census over the live tree reproduces exactly the baselined violators and asserts its scan set is non-empty.

### 2. `owning_service()` — close a live blind spot
Hardcodes `parts[0] == "cloud" || parts[0] == "oya"`, accepts `_service_roots` and ignores it, so the policy file lies. **416 of 927 crates are already unclassified.** This is present-tense breakage, not a migration concern.
- Make it capability-aware; consume the configured roots.
- **Exit:** the count of `unclassified_roots` crates drops to the genuinely-meta set; a test pins that a capability-rooted crate classifies non-`None`.

### 3. Scan-root coverage — a security gap, not a hygiene one
Three policies' `scan_roots` omit capability roots. **Two are authz gates**, in a repo whose dominant review finding is forgeable caller-supplied authz.
- `caller-supplied-authorization/dto-authz-trust-policy.json` and `endpoint-authorization-coverage/authz-coverage-policy.json` omit `ci`, `build`, `governance`, `kernel`, `os`, `app`, `base`, `tools`.
- `automation-language-policy/rust-first-automation-policy.json` covers only `scripts, tools, bin, infra, .codex, .github/workflows, cloud`.
- Extend #1404's registry-derived coverage to all three. Verify each addition introduces zero findings **before** adding it.
- **Exit:** every crate-bearing top-level directory appears in every policy's roots, or carries a declared reasoned omission.

### 4. Wait-for-in-flight baseline — the latency fix that is sound
The baseline for dev commit X publishes only when X's own dev-push run completes (~81 min) while `dev` moves every ~25 min, so the cache loses a race it cannot win. #1410 started 20 min before its own merge-base baseline existed.
- `trusted_dev_push_run_state` (drafted): Complete / InFlight / Absent. On InFlight, wait — same artifact, same exact-SHA validation, no soundness change. Bounded wait; fall through to cold rebuild on timeout.
- **Do NOT relax the exact-merge-base requirement.** `regressions = head − baseline` requires `baseline_stale ⊆ baseline_true`.
- **Do NOT ship the "publish earlier" idea** — the artifact is not consumable until the run concludes `success`, which is an anti-laundering predicate. It buys zero minutes.
- **Exit:** a structured per-run baseline-decision artifact is uploaded (it is currently written only on the hit path and never uploaded), and it shows hits on PRs whose merge-base had an in-flight run.

**No latency target is set.** Measured median is ~75 min against a single ~31 min lever; any threshold under ~45 min would be unreachable, which is the trap rev 2 fell into twice.

---

## In flight — finish, do not abandon

- **#1416** (os, 41 crates): complete, reviewed, rebased, canonical-json fixed. **Land it.** The freeze applies to new batches; abandoning finished work costs more than landing it and would leave `cloud-os` half-referenced.
- **#1399**: green, threads cleared. Land it.
- **`feat/graph-equivalence-oracle`**: has 4 HIGH findings and is wired to nothing. **Do not fix, do not land.** Its only consumer was the batch program that is now frozen. Close the branch or leave it unmerged with the findings recorded on #1419.
- **`feat/trusted-baseline-await`**: becomes item 4.

---

## Anti-inversion — the part that binds

1. **These 4 items are the entire machinery scope.** New findings become filed issues, never scope. Currently filed and staying filed: #1411, #1412, #1417, #1419.
2. **After item 4 lands, machinery work stops by default** and requires explicit justification to resume.
3. **Report the merge ratio each session.** This session: 6 PRs merged, 5 issues filed — near 1:1, which is the inversion in miniature.
4. **#1411 has one promotion condition:** flipping `warm_reads_licensed` to `true` is blocked on it. Otherwise it stays filed.

---

## Pre-mortem (regenerated — rev 2's imported its scenarios from review)

**P1 — The facade→core baseline is keyed by path and a later move invalidates it.** Even frozen, a single future relocation turns 34 baselined entries into 34 phantom-new violations, and the fix looks like laundering. *Mitigation:* key by cargo package name at authoring time (item 1).

**P2 — Extending authz scan roots surfaces real findings and blocks the PR.** Two authz gates gaining 8 roots may light up on code nobody has reviewed for authz. *Mitigation:* verify zero findings per root *before* adding it; add roots individually, not as a set; a root that produces findings gets its own remediation issue rather than blocking item 3.

**P3 — Freezing at 45% is later reversed, and the half-state hardens.** A future session re-reads ADR-0562, sees an unfinished mandate, and restarts batches. *Mitigation:* this plan is the record; ADR-0562 needs an amendment stating the program is intentionally paused at 45% with the four enforcement items as the substitute. **Without that amendment the freeze is a session-local decision that the next agent will overturn.**

---

## Test plan

| Layer | What |
|---|---|
| Unit | `trusted_dev_push_run_state`: Complete beats InFlight regardless of order; non-success never waits; unknown status not waitable; provenance mismatch rejected |
| Unit | `owning_service()`: capability-rooted crate classifies non-`None`; meta roots still `None`; configured roots actually consumed |
| Contract | facade→core: RED fixture, new violation born-blocking; frozen baseline cannot grow; baseline key survives a synthetic relocation |
| Live-corpus | Dynamic census reproduces exactly the baselined violators; scan set asserted non-empty (a fixture can pass while the real scan enumerates zero files) |
| Per-root | Each scan-root addition verified zero-finding before landing |
| Failure injection | Kill the dev-push run mid-wait ⇒ falls through to cold rebuild, does not hang |
| Observability | Structured baseline-decision artifact uploaded per run |

---

## ADR

**Decision.** Freeze the capability-first reorg at 45% migrated. Ship four separable enforcement/latency fixes. Return to product. Land in-flight work; do not start new batches.

**Drivers.** Finishing the reorg changes no architectural fact while each batch actively removes crates from tier enforcement. The remaining cost is obligations and registry repair, not relocation. Three adversarial passes showed the program grows under review rather than converging.

**Alternatives considered.** *Finish the reorg with the 10 Critic fixes* — rejected: larger Phase 0, uncapped, later return to product, and the Critic's own stakeholder read was that it is worse on the axis that matters. *Finish as pure relocation, enforcement deferred* — rejected: honest and cheap per batch, but leaves the shape unenforced for the entire program and still costs ~11 batches of obligations. *Continue ad-hoc* — rejected: that is what produced this session's 1:1 merge-to-issue ratio.

**Why chosen.** It is the only option whose exit is reachable, whose scope cannot grow, and which returns to product in four PRs rather than eleven-plus batches.

**Consequences.** The tree stays dual-state indefinitely. `base/` is never created. The capability registry stays partly stale. The 34 facade violators persist but cannot grow. ADR-0562 needs an amendment or the next agent restarts the batches (P3).

**Follow-ups.** #1411 (promotion condition: `warm_reads_licensed` flip), #1412, #1417, #1419 (also carries a wrong visibility figure — 887/887 `rust_library`, not "936 targets, 0 non-PUBLIC").
