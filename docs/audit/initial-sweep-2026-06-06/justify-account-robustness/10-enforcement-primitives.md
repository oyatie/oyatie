# 10 — KEYSTONE ENFORCEMENT PRIMITIVES (design, not build)

**Lane:** DESIGN the four self-maintaining gates that make drift + staleness *impossible* — the operational heart of **D-DOCTRINE** ("maintainable BY ENFORCEMENT; automate anything automatable; total accounting; robust-not-false"; `decision-record-oyatie-canon.md:177-184`).
**Mode:** READ-ONLY. No source/audit file edited; only this artifact written.
**Date:** 2026-06-06.
**Extends, does not redo:** `backlog-reconciliation/00-BACKLOG-RECONCILIATION.md` (§P/§G de-dup), `20-verify-foundry-hygiene.md`, `20-verify-register-coverage.md`, the source `phase0-automation-matrix.json` + `phase0-claim-evidence-map.json` seed contracts. This artifact applies the **charter lens** (hyperscaler + Linus-taste + arch-invariants + robust-not-false + total-accounting) to turn those prior planning rows into four crisp, buck2-native, generated-not-hand-maintained, RED/GREEN-proven gate designs.

---

## 0. WHY THESE FOUR, AND THE LIVE EVIDENCE THAT THE FEAR IS REAL

D-DOCTRINE's thesis (`:178`, verbatim): **"Drift + contradiction = faulty process + enforcement. Hand-reconciliation is not the fix; the enforcement that makes recurrence impossible IS the fix."** The whole consolidation exists because **two consensus bodies drifted** (`:166`) — and the reason they could drift is that **no gate required a decision to agree across its artifacts**. These four gates close that hole permanently. Each is grounded in a confirmed live false-enforcement exhibit (read from disk 2026-06-06):

| Charter exhibit | Verified on disk | Cite |
|---|---|---|
| `catalog axes_count:6` stale vs 7 | `docs/machine-readable/catalog.json:12` = `"axes_count": 6`; `docs/machine-readable/contracts.json:9` = `"axes_count": 7` | confirmed verbatim |
| 22 `oya-governance-*` crates NOT wired into CI | root `BUCK` = **0** `oya-governance` matches; only refs live in **retired** `infra/ci/jenkins/reported-status-contexts.json` + `oyaCiLane.groovy`. Live tree has **57** `oya-governance-*` dirs (`tools/`+`libs/`) — even more unwired than the charter's "22". | confirmed |
| `prd-axis-coverage` defined-not-active | appears only as an *event-name string* `"prd-axis-coverage"` at `catalog.json:207`; no enforcing gate crate, no BUCK target | confirmed |
| `diataxis-doc-class` planned-not-blocking | `docs/governance-lanes/diataxis-doc-class.md` = `doc_status: published`, `status: Accepted`, "enforces Directive 10" — but **no blocking gate is wired** (no BUCK target; ADR-0365 propagation is bound to `oya gen`/`oya gate` CLI authority, the producer register #20/#3 forbid) | confirmed |
| ADR-0363 "Foundry eradicated" false | 4,714 files / 36,210 occurrences / 780 `oya-foundry-*` live (`20-verify-foundry-hygiene.md:61-64`) | prior-verified |
| 0511→0513 supersession missing both ends | `ADR-0511 superseded_by:[]`, `ADR-0513` has no `supersedes` key (`00-BACKLOG-RECONCILIATION.md:166`) | prior-verified |
| ADR-INDEX status-counts drift | index claims 183/125/12; ground-truth scan = 172/131/16 (`20-verify-register-coverage.md:166-170`) | prior-verified |

**Common root cause (Linus-taste reading):** every one is *a claim with no fixture that BLOCKS when the claim is false* — an advisory shell pretending to be enforcement (the §K honest-claim + #21 claim-ceiling failure mode, `backlog:181`). The fix is not more docs; it is **one good data structure** (a generated accounting/agreement registry keyed by a stable id) **+ a gate that fails RED the instant the registry disagrees with reality.** No special cases.

### Design invariants shared by all four (the keystone contract)

1. **GENERATED, not hand-maintained.** Each gate consumes a registry that is *regenerated from source-of-truth* on every run; the gate's first assertion is `committed_registry == regenerated_registry` (no hand-edit drift — the ADR-0365 propagation-drift discipline, `ADR-0365:28-30`, but with the producer moved OFF the `oya` CLI per register #3/#20). Hand-editing a generated file is itself a RED.
2. **BUCK2-NATIVE producer, never the `oya` CLI.** Producer = a Rust gate crate emitted as a buck2 `rust_test`/`rust_binary` target (the `phase0-automation-matrix.json` `gate_contract.producer` = "cloud-ci/oya-ci Rust gate packet", and the AC-0.13 codegen that emits buck2 targets from the filesystem, `backlog:556`). **New `oya` CLI commands are forbidden** (register #20, `backlog:668`). This directly retires the present-day defect where ADR-0365's own gate is `verified_by: oya gen propagate --check` (`ADR-0365:26`).
3. **PROVEN by RED/GREEN fixtures it actually BLOCKS.** Reuses the on-disk fixture convention already established at `specs/fixtures/**`: a fixture JSON declares `expected_verdict: RED|GREEN` + `expected_violations:[…]`; the gate's own test asserts each RED fixture fails with exactly those violation codes and each GREEN passes (`tc-0.16-bad-operator-checklist-for-automatable-rule.json` is the live template). A gate with no RED fixture that demonstrably blocks is itself a claim-ceiling violation (#21) and must not be marked `green`.
4. **BLOCKING semantics are a required merge context, not advisory.** Per register #21 (`backlog:679`): "mechanically enforced means a required cloud-ci context plus RED/GREEN fixtures, not local `oya` output." Until the cloud-ci controller exists (P0.0), the matrix classifies the rule `automated_advisory_until_p0_0`; the *honest* status is recorded, never overstated (the matrix's own `status: seed-contract-not-green` discipline, `phase0-automation-matrix.json:_meta.status`).
5. **TOTAL ACCOUNTING is the join key.** All four share ONE generated registry id-space (`accounting-registry.generated.json`, proposed below). Gate-2 owns the registry; Gates 1/3/4 are *views/predicates* over it. This is the Linus "good data structure removes the special cases" move: don't build four bespoke scanners, build one accounting table and four predicates over it.

> **De-dup discipline (charter total-accounting + `00-BACKLOG-RECONCILIATION.md` ruling):** these four gates are the *enforcement substrate* the prior reconciliation already routed here — Gate-1 = register #11/pillar P (amend 0365, de-dup O2); Gate-2 = §G accounted-GC + total-accounting doctrine; Gate-3 = §G staleness/sinker + the linux Task-#14 ">48h stale-file" class; Gate-4 = register #20 automation-ratchet. They are authored as the **one conformance/enforcement lane** folded into D-CONFORM (`00-BACKLOG-RECONCILIATION.md:145`), NOT four parallel programs. Gate ids below are net-new tooling under amended ADR-0365 + new ADR-0520-class entries (the `>0514` free block, `20-verify-register-coverage.md:113`).

---

## GATE 1 — CROSS-ARTIFACT-AGREEMENT  (`cloud-ci-cross-artifact-agreement`)

**Charter role:** the gate whose *absence* let two consensus bodies drift (`decision-record:166`). A decision is real only if it maps to **{ADR, SSOT spec, masterplan entry, roadmap entry}** and all agree. (register #11 / pillar P, `backlog:521-525`; amends ADR-0365, de-dup O2.)

**Inputs (all generated-from-source, regenerated each run):**
- `docs/decisions/ADR-*.md` front-matter (the SSOT for decisions — ADRs are SSOT per founder memory; status 3-axis enum decision/maturity/constraint).
- `specs/*.json` machine-readable contracts (the spec face).
- `specs/masterplan.json` (the generated masterplan projection — ADR-0364 made it generated-from-ADRs).
- roadmap entries (`master-plan-sequencing.json`).
- A generated **decision-crosswalk** `decision-crosswalk.generated.json`: one row per decision id `{adr_id, spec_ids[], masterplan_node, roadmap_node, status_3axis, affected_surfaces[]}`, emitted by the Rust producer by parsing ADR front-matter `affected_surfaces` (the same field ADR-0365's propagation engine already names, `ADR-0365:24`).

**Blocking semantics (RED when):**
1. **orphan-decision** — an ADR is `Accepted` but lacks ≥1 of {spec, masterplan node, roadmap node}.
2. **unpropagated-decision** — a spec/masterplan/roadmap node references a decision id with no live ADR file (dangling edge).
3. **status-disagreement** — the 3-axis status differs across the four faces (e.g. ADR `Accepted` but masterplan node `proposed`).
4. **count/field drift** — a generated face ≠ regenerated-from-source (the `axes_count:6 vs 7` class; the ADR-INDEX 183/125/12 vs 172/131/16 class). Assert `committed == regenerated`.
5. **dual-decision collision** — two ADR files share one number, or two decisions claim the same masterplan node (the dup-0377 class; the O1/O2 additive-block collision class, `20-verify-register-coverage.md:148`).
6. **supersession-half-edge** — `superseded_by` set on one ADR without the reciprocal `supersedes` on the other (the 0511↔0513 missing-both-ends class).

**Generated-vs-hand:** the crosswalk + masterplan + ADR-INDEX are **100% generated**; humans edit ONLY ADR-front-matter source + spec source, then the producer regenerates. The gate's assertion #4 makes a hand-edit to any generated face a RED — drift is structurally impossible because the only way to change a generated face is to change source and rerun.

**RED/GREEN proof-spec** (`specs/fixtures/cross-artifact-agreement/`, on-disk fixture convention):
| Fixture | verdict | expected_violations | proves |
|---|---|---|---|
| `tc-XA-good-decision-all-four-agree.json` | GREEN | `[]` | a decision with {ADR Accepted, spec, masterplan node, roadmap node, reciprocal supersession} passes |
| `tc-XA-bad-adr-without-spec.json` | RED | `[orphan_decision]` | Accepted ADR, no spec → blocks (catches the net-new register #1/#2 "prose-only, no ADR" inverse) |
| `tc-XA-bad-masterplan-references-dead-adr.json` | RED | `[unpropagated_decision]` | masterplan node points at a renumbered/removed ADR → blocks (dangling-edge class) |
| `tc-XA-bad-status-mismatch.json` | RED | `[status_disagreement]` | ADR `Accepted` vs masterplan `proposed` → blocks |
| `tc-XA-bad-axes-count-drift.json` | RED | `[generated_face_drift]` | committed `axes_count:6` vs regenerated `7` → blocks (**the live catalog.json:12 vs contracts.json:9 exhibit, frozen as a fixture**) |
| `tc-XA-bad-dup-adr-number.json` | RED | `[dual_decision_collision]` | two ADR files, one number → blocks (**the live dup-0377 exhibit**) |
| `tc-XA-bad-half-supersession.json` | RED | `[supersession_half_edge]` | `0511.superseded_by:[0513]` but `0513.supersedes` absent → blocks (**the live 0511↔0513 exhibit**) |

The gate's Rust test loads each fixture, runs the predicate, and `assert_eq!(report.violations, fixture.expected_violations)`. **Self-test:** the gate must reproduce the seven live exhibits above as RED on the *current* corpus before it can be marked enforced — i.e. it is born already blocking real drift.

---

## GATE 2 — TOTAL-ACCOUNTING  (`cloud-ci-total-accounting`)

**Charter role:** D-DOCTRINE total-accounting (`:180`): "every file, doc, folder accounted-for AND justified: owner + justification(→decision/need) + reachability(→masterplan) + staleness policy(TTL). Unaccounted/unjustified ⇒ blocks or auto-archives. Generated-not-hand-maintained." This is the registry the other three gates join against.

**Inputs:**
- The repo file/folder tree (`git ls-files` — VCS-tracked truth; the `git ls-files` discipline from `backlog:377` that corrected the false "176G committed to VCS" claim — accounting operates on tracked files, not local junk).
- A generated **accounting registry** `accounting-registry.generated.json`: one row per tracked path `{path, resource_class, owner (OWNERS-derived), justification_ref (→ ADR id | spec id | need-ticket), reachability_ref (→ masterplan node), ttl_class, last_touch_commit}`. Owner is **OWNERS-file-derived** (hyperscaler doctrine), never hand-typed per-file.
- `OWNERS` files (hyperscaler visibility/ownership fences).
- The Gate-1 crosswalk (for the `reachability_ref` join — a path is reachable iff its justification ADR is reachable from masterplan).

**Blocking semantics (RED when):**
1. **unaccounted** — a tracked path has no registry row (new file landed without accounting).
2. **unowned** — a path's row has no OWNERS-resolvable owner.
3. **unjustified** — `justification_ref` is empty or points at a non-existent ADR/spec/need (orphan file — the `oya-governance-orphan-detection-kernel` already exists in `libs/`, wire it here).
4. **unreachable** — `reachability_ref` does not resolve to a live masterplan node (worth-documenting ⇒ worth-reading ⇒ must be reachable from masterplan, founder memory rule).
5. **no-ttl-class** — a path is not assigned a `ttl_class` (feeds Gate-3).
6. **registry-drift** — `committed accounting-registry ≠ regenerated` (hand-edit RED).

**Auto-archive (report-then-archive, never silent delete):** orphans (violation #3) and unreachables (#4) are *reported*; a path that stays orphaned past its `ttl_class` budget is **moved to an `_archive/` tree** (git-mv, reversible, never `rm`) by the controller — matching §G "report before delete" (`backlog:306`) and the founder rule "never delete/amend on an unverified verdict" (memory `verify-each-step`). Archive is a one-way-recoverable move, gated by a second verifier pass, never an in-gate deletion.

**Generated-vs-hand:** the registry is fully generated from `git ls-files` × OWNERS × ADR-front-matter. Humans never hand-maintain the accounting table; they add an OWNERS entry + an ADR/need justification, and the path becomes accounted on regeneration. **No special cases** — `_upstream/`, `vendor/`, `target/`, `buck-out/` are accounted by a single generated `resource_class: vendored|generated|ephemeral` rule with its own TTL, not by ad-hoc ignore-list sprawl.

**RED/GREEN proof-spec** (`specs/fixtures/total-accounting/`):
| Fixture | verdict | expected_violations | proves |
|---|---|---|---|
| `tc-TA-good-fully-accounted.json` | GREEN | `[]` | path with owner+justification+reachability+ttl passes |
| `tc-TA-bad-new-file-no-row.json` | RED | `[unaccounted]` | a tracked path absent from registry → blocks (no file escapes) |
| `tc-TA-bad-orphan-no-justification.json` | RED | `[unjustified]` | file whose justification ADR doesn't exist → blocks (**reproduces the foundry-residue class: 780 `oya-foundry-*` files justified by ADR-0363's false "eradicated" claim**) |
| `tc-TA-bad-unreachable-from-masterplan.json` | RED | `[unreachable]` | doc not reachable from any masterplan node → blocks |
| `tc-TA-bad-no-owner.json` | RED | `[unowned]` | path with no OWNERS match → blocks |
| `tc-TA-bad-hand-edited-registry.json` | RED | `[registry_drift]` | committed registry ≠ regenerated → blocks |
| `tc-TA-good-archive-candidate-reported.json` | GREEN-with-report | `[]` + `archive_candidates:[…]` | orphan past TTL is *reported* for archive, not deleted in-gate |

**Self-test:** run against the live tree, the gate must flag the 780 `oya-foundry-*` files as `unjustified` (their justification ADR-0363 claims they don't exist) and the 57 unwired `oya-governance-*` crates as `unreachable` (no BUCK target → not reachable from the build masterplan). It is born blocking the two largest live accounting holes.

---

## GATE 3 — STALENESS REAPER  (`cloud-ci-staleness-reaper`)

**Charter role:** §G accounted-GC / "garbage accumulation = a missing reaper" (`backlog:311`) + the linux Task-#14 ">48h stale-file (ai-slop pileup)" class. TTL per resource-class; **report-then-archive**. Maps to oya-ci `sinker` (Prow GC component, ADR-0513) + extensions.

**Inputs:**
- Gate-2's `accounting-registry.generated.json` (every row already carries `ttl_class` + `last_touch_commit`).
- A generated **TTL policy** `ttl-policy.generated.json`: `ttl_class → {budget, action: report|archive|delete, protected: bool}`, derived from §G resource classes (`backlog:304-310`):
  - `worktree` → owning-job + auto-remove-on-completion; orphan-reaped on schedule (the 176G class, but local-disk only per `backlog:377` — accounting is over `git ls-files`, worktrees are gitignored, so this class is *report on the runner*, not VCS-gated).
  - `branch` → TTL since last commit, merged/abandoned → prune (protected excluded), report-before-delete.
  - `build_artifact` (target/, buck-out/, CAS) → LRU size-cap eviction; never committed.
  - `container/image` → ttlSecondsAfterFinished + registry GC (keep-N-per-tag).
  - `process` → per-job process-group kill on completion/timeout.
  - **`stale_doc` (the >48h class)** → audit-artifact/doc untouched > budget AND unreachable (Gate-2 join) → report; persistently stale → archive. This is the ai-slop-pileup reaper (linux Task-#14).
- `git log` per-path `last_touch` (the staleness signal).

**Blocking semantics:** the reaper is primarily **report-then-act**, but it has a *blocking* face to satisfy "everything accounted" (`backlog:311` "a CI hygiene GATE that BLOCKS if accounting drifts"):
1. **stale-over-budget-unreachable** — a path past its TTL budget AND unreachable-from-masterplan (Gate-2 join) → **RED** (blocks merge that would *add* to the pile). Reachable-but-stale → report-only (a live ADR is allowed to be old).
2. **untyped-staleness** — a resource with no `ttl_class` → RED (defers to Gate-2 #5; the reaper refuses to run with un-TTL'd resources — no silent immortal files).
3. **reap-without-report** — an archive/delete action with no prior report record → RED (enforces report-then-archive ordering; never delete on an unverified verdict).

**Generated-vs-hand:** TTL policy is generated from the §G resource-class table (single source); per-path TTL assignment is generated by Gate-2. Humans set *class budgets* in one policy source, never per-file expiry. **Protected classes** (release tags, founder door:one-way records, ADR history) carry `protected:true` and are never reaped — the one explicit carve-out, declared in data, not code (Linus: no special cases in the scanner; the exception lives in the policy table).

**RED/GREEN proof-spec** (`specs/fixtures/staleness-reaper/`):
| Fixture | verdict | expected_violations | proves |
|---|---|---|---|
| `tc-SR-good-fresh-reachable.json` | GREEN | `[]` | recently-touched reachable doc passes |
| `tc-SR-good-old-but-reachable-adr.json` | GREEN | `[]` + `report:[]` | a 2-year-old *live* ADR is fine (age alone ≠ stale) |
| `tc-SR-bad-stale-unreachable-doc.json` | RED | `[stale_over_budget_unreachable]` | >48h untouched + unreachable audit-slop → blocks (**the Task-#14 ai-slop class, frozen as fixture**) |
| `tc-SR-bad-untyped-resource.json` | RED | `[untyped_staleness]` | resource with no ttl_class → blocks |
| `tc-SR-bad-reap-without-report.json` | RED | `[reap_without_report]` | archive action lacking a prior report record → blocks |
| `tc-SR-good-protected-not-reaped.json` | GREEN | `[]` | release tag past budget but `protected:true` is NOT reaped |
| `tc-SR-good-stale-reported-then-archived.json` | GREEN-with-report | `[]` + `archived:[…path→_archive/…]` | report-then-archive flow produces a reversible git-mv, no rm |

**Self-test:** against the live audit tree, must report the `synthesis/_partial-*.md` + `_verify-*.md` scratch artifacts (if untouched >budget and unreachable) as stale-doc archive candidates — proving the reaper would have caught the very ai-slop pileup the linux Task-#14 exists for.

---

## GATE 4 — AUTOMATION-RATCHET  (`cloud-ci-automation-ratchet`)

**Charter role:** register #20 (`backlog:661-668`): "anything enforceable or automatable MUST be enforced/automated; manual exceptions need owner + target-gate + RED/GREEN + retirement phase + evidence path; new `oya` CLI commands forbidden." **The seed contract already exists on disk** (`specs/phase0-automation-matrix.json`, `status: seed-contract-not-green`) with live fixtures — this design *extends and hardens* it, it does not invent it.

**Inputs:**
- `specs/phase0-automation-matrix.json` (the existing seed; classifications = `automated_blocking_now | automated_advisory_until_p0_0 | controller_owned_in_phase_1 | not_automatable_human_judgment`).
- A generated **enforcement-inventory** `enforcement-inventory.generated.json`: one row per *rule* discovered across {every ADR `enforcement_status` field, every spec gate_contract, every operating-contract requirement, every branch-protection constraint, every generated registry, every reviewer/multispectrum requirement} — the producer scans for anything *claiming* to be enforced and emits a row.
- The set of live buck2 gate targets (to verify each "automated" row actually has a wired producer).

**Blocking semantics (RED when):**
1. **enforceable-marked-human-judgment** — a row with `enforceable_or_automatable: true` classified `not_automatable_human_judgment` (**the live `tc-0.16-bad-operator-checklist-for-automatable-rule.json` violation** — a branch-protection invariant filed as an operator checklist). This is the *core ratchet*: you may not downgrade an automatable rule to manual.
2. **advisory-claiming-enforced** — a rule whose doc says "enforces"/"blocks" but has **no wired buck2 target** (the §K honest-claim + #21 claim-ceiling failure; the **live exhibit: 57 `oya-governance-*` crates with 0 root-BUCK references; `diataxis-doc-class.md` "enforces Directive 10" with no blocking gate; `prd-axis-coverage` event-name only**).
3. **oya-cli-authority** — a row whose `target_gate_or_controller` is an `oya` CLI command (**the live `tc-0.16-bad-oya-cli-authority.json` violation; and the live ADR-0365 `verified_by: oya gen propagate --check`/`oya gate validate` defect**). New `oya` CLI surface forbidden.
4. **incomplete-exception** — a `not_automatable_human_judgment` row missing any of {owner, target_gate, blocking_fixture, retirement_phase, evidence_path} (**the live `tc-0.16-bad-missing-field-unknown-classifier-duplicate.json` violation**).
5. **no-retirement** — a manual/advisory exception with `retirement_phase: none` for an item that is automatable (a bridge with no deletion criterion — the §Q port-to-Rust discipline, `backlog:551`).
6. **ratchet-regression** — an item previously classified `automated_blocking_now` downgraded in a later commit (the ratchet only tightens; monotonic — compare against the committed baseline).

**Generated-vs-hand:** the enforcement-inventory is generated by scanning all enforcement *claims* in the corpus; the matrix *classification* is founder-decided per row (genuine human judgment about whether something is automatable). The gate verifies the classification is *honest* (claims #1-#3) and *complete* (#4-#5) and *monotonic* (#6) — it does not decide automatability, it forbids dishonest/incomplete classification. This is the **claim-ceiling (#21) applied to the enforcement layer itself**: a gate that says "enforced" must prove it with a wired target + RED/GREEN, or be downgraded to the honest `advisory_until_p0_0` tier (the matrix's own `status: seed-contract-not-green` honesty).

**RED/GREEN proof-spec** — **the fixtures already exist on disk** (`specs/fixtures/phase0-automation-ratchet/`), reuse + extend:
| Fixture (live or net-new) | verdict | expected_violations | proves |
|---|---|---|---|
| `tc-0.16-good-human-judgment-with-retirement-path.json` *(live)* | GREEN | `[]` | a genuine human-judgment row WITH owner+target+fixture+retirement+evidence passes |
| `tc-0.16-bad-operator-checklist-for-automatable-rule.json` *(live)* | RED | `[enforceable_or_automatable_marked_human_judgment]` | automatable branch-protection filed as manual checklist → blocks |
| `tc-0.16-bad-oya-cli-authority.json` *(live)* | RED | `[oya_cli_authority]` | row whose producer is an `oya` CLI command → blocks |
| `tc-0.16-bad-missing-field-unknown-classifier-duplicate.json` *(live)* | RED | `[incomplete_exception]` | exception missing required fields → blocks |
| `tc-AR-bad-advisory-claiming-enforced.json` *(net-new)* | RED | `[advisory_claiming_enforced]` | a "enforces"-claiming rule with no wired buck2 target → blocks (**reproduces the 57-unwired-crates + diataxis-not-blocking exhibits**) |
| `tc-AR-bad-ratchet-regression.json` *(net-new)* | RED | `[ratchet_regression]` | `automated_blocking_now` downgraded vs baseline → blocks |

**Self-test:** run the producer over the live corpus; the inventory must emit `advisory_claiming_enforced` for the 57 `oya-governance-*` crates (0 BUCK targets), for `diataxis-doc-class` ("enforces Directive 10", no gate), and for `prd-axis-coverage` (event-name only). It is born flagging every live advisory-shell-claiming-enforced.

---

## 1. HOW THE FOUR COMPOSE (one substrate, not four programs)

```
                 git ls-files × OWNERS × ADR-front-matter × specs × masterplan (SOURCE)
                                          │  (one Rust producer packet, buck2 rust_binary)
                                          ▼
                        accounting-registry.generated.json   ◄── the ONE good data structure
                          (path → owner+justification+reachability+ttl_class)
            ┌─────────────────┬──────────────────┬─────────────────────┐
            ▼                 ▼                  ▼                     ▼
   GATE 2 total-accounting  GATE 1 cross-artifact  GATE 3 staleness    GATE 4 automation-ratchet
   (rows complete?)         (decision faces agree?) (TTL over budget?)  (claims honest+monotonic?)
            └─────────────────┴──────────────────┴─────────────────────┘
                       all emit RED/GREEN vs specs/fixtures/**; all are
                       REQUIRED cloud-ci contexts (not advisory) post-P0.0
```

- **Gate-2 is the producer of record**; Gates 1/3/4 are predicates over its registry (+ their own face-specific source). No four parallel scanners → Linus "good data structure kills the special cases."
- **Sequencing (charter parallelizable-lanes + `00-BACKLOG-RECONCILIATION:145`):** all four ship in the **G-INTEGRITY track** (specs+filesystem, NO buck2-build-graph dependency, so they ship Phase-0 before the build migration — "the false-green firewall must not wait", Architect verdict `:341`). They become *required* cloud-ci contexts at P0.0; until then their honest classification is `automated_advisory_until_p0_0`, recorded in the matrix, never overstated.
- **Bootstrapping honesty:** each gate's status in `phase0-automation-matrix.json` starts at the matrix's `seed-contract-not-green` and may only flip to `automated_blocking_now` once its own RED/GREEN self-test reproduces the live exhibits above — i.e. a gate cannot claim "enforced" until it has demonstrably blocked the real drift it targets. Gate-4 enforces this on Gates 1-3 (the ratchet polices itself).

---

## 2. RETURN DIGEST

**Four keystone gates (all buck2-native Rust producers, generated-not-hand, RED/GREEN-proven, required-context-not-advisory; one shared `accounting-registry.generated.json`):**

1. **CROSS-ARTIFACT-AGREEMENT** (`cloud-ci-cross-artifact-agreement`; amends ADR-0365, de-dup O2). Inputs: ADR front-matter + specs + masterplan + roadmap + generated `decision-crosswalk`. BLOCKS: orphan-decision · unpropagated-decision · status-disagreement · generated-face-drift · dual-decision-collision · supersession-half-edge. *This is the gate whose absence let the two consensus bodies drift.* PROOF: 7 fixtures freezing the LIVE exhibits — `axes_count:6 vs 7` (catalog.json:12 vs contracts.json:9), dup-0377, 0511↔0513 half-edge — each must reproduce RED on the current corpus.

2. **TOTAL-ACCOUNTING** (`cloud-ci-total-accounting`; owns the registry). Inputs: `git ls-files` × OWNERS × ADR justification × masterplan reachability. BLOCKS: unaccounted · unowned · unjustified(orphan) · unreachable · no-ttl-class · registry-drift(hand-edit). Auto-archive = report-then-git-mv-to-`_archive/`, never rm, gated by a second verifier (never delete on unverified verdict). PROOF: 7 fixtures; self-test must flag the live 780 `oya-foundry-*` (unjustified vs ADR-0363's false "eradicated") + 57 unwired `oya-governance-*` (unreachable).

3. **STALENESS REAPER** (`cloud-ci-staleness-reaper`; §G sinker++, Task-#14 >48h class). Inputs: Gate-2 registry + generated `ttl-policy` per resource-class (worktree/branch/artifact/image/process/stale_doc) + `git log` last-touch. Report-then-archive; BLOCKS: stale-over-budget-AND-unreachable · untyped-staleness · reap-without-report. Age alone ≠ stale (a live ADR may be old); protected classes (release tags, door:one-way records) never reaped (declared in policy data, not code). PROOF: 7 fixtures incl. the ai-slop `_partial`/`_verify` scratch-doc class as archive candidate.

4. **AUTOMATION-RATCHET** (`cloud-ci-automation-ratchet`; register #20; **seed `phase0-automation-matrix.json` + 4 live fixtures already on disk** — hardened here). Inputs: the matrix + generated `enforcement-inventory` (every enforcement *claim* in the corpus) + live buck2 target set. BLOCKS: enforceable-marked-human-judgment · advisory-claiming-enforced(no wired target) · oya-cli-authority · incomplete-exception · no-retirement · ratchet-regression(monotonic). PROOF: reuse 4 live RED/GREEN fixtures + 2 net-new; self-test must flag the 57 `oya-governance-*` crates, `diataxis-doc-class`, `prd-axis-coverage`, and ADR-0365's own `oya gen`-bound `verified_by` as advisory-claiming-enforced / oya-cli-authority. Gate-4 polices Gates 1-3 (a gate may not claim "enforced" until its self-test reproduces its live exhibits as RED).

**Cross-cutting invariants:** (a) every generated face asserts `committed == regenerated` → hand-edit = RED → drift structurally impossible; (b) producer is always a buck2 Rust gate crate, **never a new `oya` CLI command** (#20); (c) no gate flips to `automated_blocking_now` until its RED/GREEN self-test demonstrably blocks the real live drift it targets (#21 claim-ceiling applied to the gates themselves); (d) all four = G-INTEGRITY track, Phase-0, no buck2-build-graph dependency, so the false-green firewall ships before the migration; (e) ONE accounting registry, four predicates — no special cases.

**Coverage I did NOT do (no silent caps):** I did not author the Rust producer crates, the JSON registry schemas, or the fixture files themselves (this is DESIGN, not build, per lane). I read the four false-enforcement exhibits to ground-truth (catalog/contracts axes_count, governance-crate BUCK refs, diataxis/prd-axis lane status, ADR-0365 verified_by) but did NOT re-scan all 346 ADRs or all ~90 specs individually — I reused the prior-verified counts in `20-verify-register-coverage.md` (172/131/16) and `20-verify-foundry-hygiene.md` (4,714/780) rather than re-deriving them. I did not verify the `merge-queue-parked-pr.json` (the renamed structural-lock) internals or the full `phase0-claim-evidence-map.json` body — only confirmed they exist as the registries Gate-1/4 consume. The "22 governance crates" charter figure is a *lower bound*: live tree has 57, all equally unwired from root BUCK.
