# Remaining Work — Consensus Plan (rev 3)

**Status: rev 2 REJECTED by Critic. rev 3 shrinks rather than services.** Date: 2026-07-26.
Parent: `.omc/plans/reorg-pipeline-consensus-plan.md` rev 3 (reorg frozen at 45%).

---

## Why rev 3 is smaller, not bigger

The Critic returned REJECT with 5 CRITICAL and 8 MAJOR findings and a minimum-set of **8 changes** to earn APPROVE. Servicing all 8 grows the machinery program — which is the exact failure the parent plan was created to stop:

> *"When review findings compound into scope growth, the plan is the problem — shrink it, do not service it."*

So rev 3 does not answer the 8. It drops what the findings killed and keeps only what survived.

**The measured pattern in rev 2** (the durable lesson): *every claim the Architect verified reproduces; every claim the Architect did not touch is false.* A verified premise does not transfer its verification to what you build on top of it.

---

## Decisions

**D1 (RE-REVERSED) — item 4 is DROPPED, on evidence this time.**
Waiting beats rebuilding only when remaining run time `R < C` (rebuild cost). With dev-push ≈81 min and `C` ≈31 min, break-even is merge-base age > 50 min. `InFlight` means age < 81, so waiting wins on roughly the (50, 81) window — ~38% of its own trigger condition — and **loses by up to 50 minutes** below it. A bounded wait `B` that times out costs `B + 31`, strictly worse than 31: the fall-through is **additive, not alternative**. rev 2's P4 ("can only ever be faster") is arithmetically false.

Sizing was also a mechanism mismatch: 45% is an *affected-set escalation* rate, while the merge-base baseline is materialized per-job **by design on every PR** (`oya-ci-required.yml:141,322`), independent of tier. rev 2 multiplied two different mechanisms.

A correct design exists — *wait iff estimated remaining < rebuild cost* — but it needs a run-duration estimate nobody has measured, and `~31 min` itself has no artifact in the repo grounding it. **Measure before re-proposing.**

**D2 (DROPPED) — no `frozen_empty` freeze gate.** Verified directly: both firewall predicates (`baseline-ratchet/src/lib.rs:584`, `:525`) read `!signoff.is_signed_off(...)` with no `frozen_empty` special case, and the module doc states the design intent at `:56-60` — *"Same predicate, no special case."* The signoff door already carries two `authorized_by: "pending-founder-ratification"` entries that agents wrote and merged. A freeze code is a three-line-JSON speed bump, not a mechanism — and rev 2 used that same door one decision later to land its own ADR. Claiming it as enforcement was self-contradictory.

**D3 (KEPT) — the ADR amends ADR-0328, lands `Proposed`.** `ADR-0562:37` disclaims §10 as executable; ADR-0328 is the sequence authority. `Proposed` avoids REDing cross-artifact-agreement. It is a *record*, and rev 3 says so plainly instead of dressing it as enforcement.

**D4 (CORRECTED) — the ratchet baselines 35 same-capability facade *packages*, keyed by cargo package name.**
The unit was the defect: 35 is a package count; the same set keyed by buck2 *target label* is **89**, by dep-edge **141**. rev 2 declared a target-label key against a package-derived baseline, so its own census exit would have REDed at authoring time. Package-name keying also restores the parent plan's P1 mitigation (a buck2 label embeds the path and dies on any move) which rev 2 silently reversed.
Detection still reads the **buck2 graph** — `intelligence/facade/worker/BUCK:8,22` carries the edge with zero Cargo path-deps, invisible to a manifest scan.

---

## Scope — 1 PR, plus what is already in flight

### PR — facade→core ratchet + the ADR
- `facade_core_direct_dep`: frozen baseline at the **35** same-capability facade packages, keyed by cargo package name, `baseline-block-on-new`.
- `facade_core_no_ports_layer`: the subset in capabilities with zero `ports/` crates, which ADR-0562 §10.6 declares legal. **Open:** `marketplace` also has zero `ports/` and is excluded only by the same-capability scope choice — state the rule or include it.
- ADR amending ADR-0328, `Proposed`, recording the pause at 45% and what is given up.
- **Registration:** copy `ci/facade/caller-supplied-authorization/` — it is a complete exemplar (11 exist; rev 2's "no clean exemplar exists" was false). `ADR-0582:196-204` is the byte-exact path-justification block. Add T6 (`gate_registration.rs:848-914`, exact set convergence + panics on unknown `input_kind`), hermetic purity and policy-as-data (`gate-self-conformance/src/lib.rs:761-801`). Do **not** add a per-gate `OWNERS` — `**/OWNERS` is deliberately undeclared in the affected-set policy and would escalate the PR to FULL for nothing.
- **This PR adds a `BUCK`, so it is a guaranteed FULL-tier PR.** 28 of 30 measured FULL escalations are BUCK-edit-driven. Budget for it.

### In flight
- **#1423** — item 2 (`owning_service` capability-aware) + a self-conformance scanner fix. Pushed, CI running.
- **#1420, #1418, #1415, #1414** — open, mixed red. Triage each as *real defect* vs *CI friction* before touching.

### Explicitly out
Item 4 until measured. Any `frozen_empty` freeze gate. `feat/graph-equivalence-oracle`. Any crate movement. Cross-capability facade→core edges (40 packages / 214 edges) — recorded, not scoped.

---

## Corrections of record

| rev 2 claim | truth |
|---|---|
| `frozen_empty` is ungrandfatherable | Signoff-exempt at `lib.rs:584`, `:525`; door used twice by agents |
| baseline 35, keyed by target label | 35 packages **or** 89 target labels — rev 2 mixed the units |
| a target label survives relocation | A buck2 label embeds the path; it does not |
| `.github/**` is declared inert | 23-entry synthetic seed list; `[]` was the reverted false-green #1389/#1391 |
| `feat/trusted-baseline-await` has a drafted state machine | 0 ahead / 4 behind dev; the symbol does not exist. Item 4 is greenfield |
| No clean exemplar exists to copy | 11 complete exemplars exist |
| `ci/facade/` arrived in one bulk move (`33134e055`) | Shallow-clone artifact — that SHA is the parentless root |
| ~31 min recoverable | No artifact in the repo grounds this number |
