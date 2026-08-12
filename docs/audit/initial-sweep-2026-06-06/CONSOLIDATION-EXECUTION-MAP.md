# CONSOLIDATION + CLEANUP EXECUTION MAP

**STATUS: `pending-approval` · door:one-way founder sign-off**

> **Role of this file:** the single actionable execution driver that integrates the three sweep lanes
> (LANE 1 doc/json SSOT-mapping · LANE 2 dead-code/dead-file/stale-ref · LANE 3 maintainability/DX/hyperscaler)
> into one sequenced run. It front-loads the convergence so it executes fast the moment the source tree is
> released by the buck2-fix executor. **It is a working artifact — archive it after the pass completes.**
> **It authors no content and mutates nothing.** It is the conductor, not the band.
>
> **Authority chain:** `BIG-HYGIENE-PASS-PLAN.md` (ralplan-approved governing plan, B1→B2→A→C) ·
> `STORE-SCHEMA.md` (FROZEN contract, B2 schema-freeze granted 2026-06-07) ·
> `decision-record-oyatie-canon.md` (D-SSOT-CURRENT-TRUTH, D-DOCORG, D-EXCELLENCE-MANDATE, D-CICD-AUTHORITY) ·
> the masterplan-SSOT reachability principle (ADRs = SSOT; masterplan/index are GENERATED VIEWS;
> worth-documenting ⇒ worth-reading ⇒ reachable-else-archive).
>
> **Verified ground truth (read-only, 2026-06-08):**
> - The 3+1 SSOT stores EXIST + populated at `/Users/jasonlee/Developer/source/registry/stores/`:
>   `design-store.json` (201,974 B) · `registry-store.json` (358,108 B) · `instructions-store.json` (62,839 B) · `canon-id-crosswalk.json` (2,791 B). **B2 truth-capture has begun; this is not greenfield.**
> - Recovery anchor `e38624dc4` exists ("full source tree pre-aggressive-cleanup … over-delete recoverable from this commit"), pushed to `github-mirror`.
> - Active branch `cleanup/whole-tree-2026-06-07`; a background executor is committing concurrently (observed HEAD `38bc5797b`, drifted from LANE 2's snapshot `3319a2bab`). **All counts here are snapshots and will drift.**
> - The ONE canonical required CI is `.github/workflows/oya-ci-required.yml`; it fans in producer-regen (`oya-cloud-ci-accounting-registry-app`) + 7 floor gates (cross-artifact-agreement, total-accounting, staleness-reaper, automation-ratchet, bnf-layer-suffix, manifest-hygiene, cargo-prefix) + registry-drift + firewall-app. This is the keep-list spine.

---

## §0 — SOURCE-FORCED PRE-FLIGHT (run before ANY read/grep/delete, every batch)

The #1 catastrophic risk is the CWD-contamination trap: the session CWD `/Users/jasonlee/Developer/linux` is the **linux kernel port** (a DIFFERENT repo). The canonical tree is `/Users/jasonlee/Developer/source` (the oyatie monorepo). Every step uses absolute `/Users/jasonlee/Developer/source/...` paths.

**Gate G0 (self-check, blocking, every batch):**
1. `git -C /Users/jasonlee/Developer/source rev-parse --abbrev-ref HEAD` == `cleanup/whole-tree-2026-06-07` — else ABORT.
2. The executor has RELEASED the tree (no concurrent commits in flight; `git -C …/source status --porcelain` reflects only your staged batch). **Do not mutate while the buck2-fix executor is committing.**
3. The recovery anchor `e38624dc4` is reachable (`git -C …/source merge-base --is-ancestor e38624dc4 HEAD`).
4. You are NOT in the port: a tree showing `stack/ kernel/legacy-kernel/ legacy-port/` is the WRONG repo → ABORT.

**Verify-each-step gate (founder rule, every wave):** every disposition is logged with cited evidence (file:line / count / repro). No phantom findings. A finding moves a file out of KEEP only on a verified verdict. Trust nothing as-is.

---

## §1 — THE FEW SSOT FILES (the target set everything consolidates INTO)

Everything folds into this short, keyed, gate-honest set. After the pass, the 6,273 md / 1,938 json sprawl
(LANE 1; LANE 2 tracked-file recount 17,786 incl. 5,711 md + 1,592 json) collapses to:

| # | SSOT file (absolute) | Absorbs | Contract |
|---|---|---|---|
| 1 | `/Users/jasonlee/Developer/source/registry/stores/design-store.json` | the 348 `docs/decisions/` ADRs (metadata + edges) · PRDs · specs · runbooks (axis) · the oya/** IP/PRD/spec metadata · `proposed-forward-plan` entities (OPEN-2) | STORE-SCHEMA §1; closed enums; no tombstones (§0.1.2); edges→current only |
| 2 | `/Users/jasonlee/Developer/source/registry/stores/registry-store.json` | the 896 `registry/catalog/*.yaml` (~1:1) + loose `registry/*.json` operational ledgers that are catalog-shaped | STORE-SCHEMA §3; closed `role`/`plane`/`data_class` enums (OPEN-3 normalized) |
| 3 | `/Users/jasonlee/Developer/source/registry/stores/instructions-store.json` | `AGENTS.md` · `CLAUDE.md` · `.claude/{commands,skills,settings.json}` narration · the forbidden-vocab RULE (§2.3) | STORE-SCHEMA §2 |
| 4 | `/Users/jasonlee/Developer/source/registry/stores/canon-id-crosswalk.json` | the id↔id remap (old slug → canonical id) that lets the excise/scrub stay referentially honest | STORE-SCHEMA crosswalk |
| 5 | `/Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md (clean ADR series) | the SSOT decisions of record. Prose body stays as Obsidian-wikilink `body_ref` (§7); metadata+edges mirror into design-store | ADRs = SSOT; masterplan GENERATED from these |
| 6 | `/Users/jasonlee/Developer/source/docs/machine-readable/masterplan.generated.json` (the generated view) + `specs/masterplan.json` retires-to-generated | the GENERATED masterplan/index VIEW (§1.6, §8) — never an entity, never hand-maintained | regenerated from ADRs/stores; drift = gate RED |
| 7 | a NEW keyed **findings/backlog json store** (sibling under `registry/stores/` or a named `registry/*.jsonl`) | `FINDINGS-LEDGER.md` rows (F-NNNN tabular) + the existing `registry/mistakes-ledger.json` / `fixuptasks.jsonl` standing backlog | tabular data → keyed store, not hand-md |
| 8 | select-essential survivors (§7 carve-out): `README.md` · `CLAUDE.md` · `AGENTS.md` · `LICENSE` · `SKILL.md` · live `docs/runbooks/*` · `oya-ci.toml` · the gate config/baseline/disposition JSON | orientation + live runbooks + the gate substrate | KEEP — not folded, not deleted |

Everything not in this set is FOLD (into 1–4/7), RATIFY (into 5, then archive the prose), or DELETE (git history is the archive — STORE-SCHEMA §0.1.2 no-tombstone, no `_archive` dir).

---

## §2 — CONSOLIDATION WAVES (sequenced onto B2 → A → C, door:one-way batch boundaries)

Legend: **KEEP** (untouched, in keep-list) · **FOLD** (metadata→store, prose→body_ref or excise) · **RATIFY** (decision→clean ADR, then archive carrier) · **ARCHIVE** (git-history-only; delete from tree) · **DELETE** (stale/transient; git-history-only).
**DESTRUCTIVE steps are marked ⚠️** — they require: recovery anchor `e38624dc4` confirmed + one signed commit per batch + founder-awareness at the door.

### WAVE 0 — PRE-FLIGHT + KEEP-LIST FREEZE (non-destructive; door:one-way to start B2)
- Run Gate G0 (§0). Confirm executor has released the tree.
- **Freeze the KEEP-LIST first (KEEP-POSTURE):** the build-graph + CI spine is sacrosanct.
  - KEEP: `oya-ci-required.yml` + producer-regen (`oya-cloud-ci-accounting-registry-app`) + the 7 floor gates + `registry-drift` + `firewall-app`; `oya-ci.toml`; `oya-ci-config` bundled JSON (`gate-disposition.json`, `gate-baseline*.json`, `gate-baseline.signoff.json`).
  - KEEP: `registry/stores/*` (the 4 SSOT files) + `registry/catalog/*.yaml` (folds INTO registry-store, not deleted until fold verified).
  - KEEP: README/CLAUDE/AGENTS/LICENSE/SKILL.md (§7 carve-out); live `docs/runbooks/*`; gate fixtures + live `evidence/ci` baselines.
  - KEEP (do NOT flag dead): the 246-no-reverse-dep lib set is a SUPERSET — protect all `*-app`/`*-api`/`*-domain` deploy leaves (reachable via buck/k8s, not cargo deps).
- Verify: keep-list is a concrete file list checked against `cargo metadata` + the CI YAML + the gate config. **No deletion happens in Wave 0.**

### WAVE 1 — AUDIT-DOC CONSOLIDATION (freshest sprawl; the `initial-sweep-2026-06-06` corpus)
**Why first:** it is the freshest pileup (45 top-level md + 214 recursive + 20 subdirs + 13 json) and its decisions gate the rest. Disposition is per the LANE 1 classification (verified each from its STATUS line).

- **RATIFY → clean ADR (then ⚠️ archive the carrier):** the 6 door:one-way design docs + the approved plans —
  `OYA-CI-HERMETIC-EXECUTION-DESIGN.md` (ADR-0515 family, git-facts boundary Option C) ·
  `OYA-CI-CONFORMANCE-FLOOR-PLAN.md` (engine-vs-policy seam) ·
  `OYA-CI-VCS-AGNOSTIC-SEAM-REFINEMENT-PLAN.md` (git→scm-facts rename, ScmFactsSource trait) ·
  `LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md` (founder apex hermeticity) ·
  `AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE.md` (auto-remediation engine generalization — also the §3 gate-seed north-star) ·
  `PLATFORM-PRODUCTIZATION-ARCHITECTURE.md` (capstone → small ADR cluster A/B/C/D) ·
  `CICD-DESIGN-PLAN.md` (APPROVED → ADR-0515/D-CICD-AUTHORITY) ·
  `ARCH-DESIGN-DOC-PLAN.md` (APPROVED Phase-2 reference) ·
  `AMENDMENT-PLAN.md` (the 111KB carrier OF the ratified dispositions — its decisions ARE the ADR amendments) ·
  `F0029-enum-reconciliation.md` (residue → ADR-0106; worksheet archives) ·
  `CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN.md` (retire CLI-governance → firewall pipeline; STEP1-TRIAGE/CICD-RESEARCH-INPUTS archive after).
- **SEED `proposed-forward-plan` (OPEN-2), then ⚠️ archive prose:** `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md` (self-declared NOT door:one-way) · `MIGRATION-L2-L11-CONFORMANCE-READINESS.md` (audit/readiness, not a decision).
- **FOLD → findings/backlog store (#7):** `FINDINGS-LEDGER.md` (F-NNNN tabular rows — the clearest fold case).
- **KEEP (active governing artifacts until the pass ends):** `STORE-SCHEMA.md` (FROZEN contract) · `BIG-HYGIENE-PASS-PLAN.md` (governing plan) · `README.md` (audit-dir index) · THIS map.
- **⚠️ ARCHIVE-SUPERSEDED (git-history-only) once their decisions ratify:** `00-MASTER-CONTRADICTION-REGISTER.md` · `00-PERSONAL-ADR-VERIFICATION.md` · `adr/` (54 WF1 per-agent chunk reports) · `PHASE-0-FIREWALL-PLAN.md` (already landed as ADR-0515) · the go-live runbooks (`PHASE-0-GO-LIVE-RUNBOOK`, `FIREWALL-GO-LIVE-RUNBOOK`, `DEV-CUTOVER-PLAN`, `PAIRED-HANDOFF`) · the foundry/rename scrub maps (`FOUNDRY-ADJUDICATION-TABLE`, `FOUNDRY-PROSE-SCRUB-MAP`, `FOUNDRY-VOCAB-ERADICATION-PLAN`, `ADR-SLUG-RENAME-MAP`, `GATE-PREFIX-RENAME-PLAN`) · the hand-plans (`MIGRATION-PLAN-RESYNC`, `UNIFIED-EXECUTION-PLAN`, `PHASE-1-AMENDMENT-LANES`, `FULL-SCOPE-OF-WORK`) · the `_arch/_critic` consensus rounds + the 20 working subdirs · `B2-SCHEMA-FREEZE-READINESS.md`.
- **⚠️ DELETE-STALE (git-history-only):** `CONTINUATION-PROMPT.md` · `HANDOFF.md` (transient session bridges).
- **Door + verify:** one signed commit "wave-1: audit-doc ratify+fold+archive". After commit, the only audit-dir survivors are the KEEP set above. Re-run the keep-list assertion.

### WAVE 2 — B2 TRUTH-CAPTURE: FINISH THE FOLD INTO THE STORES (non-destructive — write the stores; do NOT yet delete sources)
The stores exist but are partial. Finish capturing the broad source INTO them BEFORE any A-delete (truth must be in the store before the carrier is removed).
- **FOLD → design-store:** `docs/decisions/` 348 ADR metadata+edges (excise the 22 `superseded_by`, no tombstones, OPEN-1) · `docs/specs` (116) · `docs/runbooks` (203, RUNBOOKS axis) · `docs/standards` (101) · `docs/personas` (131) · `docs/user-journeys` (913 — the largest non-ADR md pocket; metadata→store, prose→limited Obsidian body_ref; **founder fold-vs-archive call flagged**) · `docs/governance-lanes` (65).
- **FOLD → design-store (oya/**, the biggest target after docs/):** the 1,623 oya md + 632 oya json — IP/PRD/spec metadata → design-store entities (IPS/PRDS/SPECS-MS axes); prose → Obsidian body_ref; per-service fixtures/specs → design-store or registry per kind.
- **FOLD → registry-store:** the 896 `registry/catalog/*.yaml` (~1:1, already clean) + the loose `registry/*.json` operational ledgers (microservices, bounded-contexts, knowledge-graph-*, merge-queue-*) that are catalog-shaped.
- **FOLD → instructions-store:** AGENTS/CLAUDE/.claude narration + the forbidden-vocab RULE.
- **FOLD → findings/backlog store (#7):** `registry/mistakes-ledger.json` + `fixuptasks.jsonl` join the FINDINGS-LEDGER rows.
- **GENERATE the views:** regenerate `docs/machine-readable/masterplan.generated.json` from the stores/ADRs; `specs/masterplan.json` retires to the generated path.
- **Verify-each-step:** the 4 STORE-SCHEMA §4 guards run GREEN on every store write — accessor (keyed read), formatter (canonical bytes), merge-driver (no dup keys), entity-incremental-gate (off-enum / dangling-edge = RED). Crosswalk covers every excised-slug inbound ref. **No source file is deleted in Wave 2.**

### WAVE 3 — ⚠️ A-DELETE: CONFIDENTLY-DEAD CODE + STALE TREES (DESTRUCTIVE; keep-list-first; over-delete-OK-recover-from-`e38624dc4`)
Only the LANE 2 confidently-dead/superseded set — gated by the predicate `dead = (no reverse-dep) AND (not invoked by oya-ci-required.yml) AND (not a *-app/*-api/*-domain deploy-leaf in BUCK)`.
- **⚠️ DELETE (legacy CLI-governance cluster, Task #26 in_progress):** `oya-check-*` (71 of 72, minus `oya-check-brand-residue` which the active accounting-registry producer path-depends on) · `oya-governance-*` (56) · `oya-dev-cli` — **BLOCKED until `Makefile:43` (`verify-deploy-contract` → `cargo run -p oya-dev-cli`) is rewired/dropped first.** This is the one hard ordering constraint.
- **⚠️ DELETE (orphan leaf libs):** `oya-json-kernel`, `oya-shuffle-sharding` (0 reverse-deps, no source import, only catalog/spec bookkeeping refs).
- **⚠️ DELETE (dead files/trees):** `oya/intelligence/_legacy-foundry/` (3 files; cites 8 PHANTOM governance crates + superseded ADR-0346/0347 Proposed / ADR-0349 Superseded) · `registry/vcs/` (5 files; oya-vcs retired by ADR-0363) · the foundry doc/contract/template trees (~56 files under `docs/foundry`, `docs/products/foundry`, `docs/runbooks/foundry`, `docs/teams/axis-foundry`, `contracts/openapi/foundry`, `templates/foundry-supervisor` — **templates tree deleted** on integ/docs; remaining doc/contract trees still open).
- **⚠️ DELETE (stale-plan pockets):** `tasks/` (119 of 120 are completed/abandoned `*-plan.md`/`*-todo.md` referencing superseded idents) · `.omc/plans/` M02/foundry/jenkins milestone pocket (110 of 362).
- **⚠️ PRUNE (agent runtime scratch, not canon):** `.omc` (595 md / 142 json) + `.omx` (129 md / 328 json) — gitignored-class tooling state; prune aggressively (live `.omc` state + git history suffice).
- **EVIDENCE split:** KEEP live gate-baselines + current `evidence/ci`; ⚠️ ARCHIVE historical `evidence/per-change`, `evidence/audits` snapshots (git-history-only).
- **Door + verify (per batch):** rewire Makefile FIRST (separate signed commit). Then one signed commit per delete batch ("wave-3-N: A-delete <cluster>"). After EACH batch: `cargo metadata` resolves + `oya-ci-required.yml` green locally (or its gates compile) + no dangling crosswalk ref. If any gate goes RED on a delete → `git revert`/restore from `e38624dc4`; over-delete is recoverable, a broken gate is not shippable.

### WAVE 4 — ⚠️ A-DELETE: FOLDED MD/JSON SOURCES (DESTRUCTIVE; only AFTER Wave 2 fold verified)
Now that truth is in the stores (Wave 2) and verified, delete the now-redundant carriers.
- **⚠️ DELETE the folded prose/metadata md** whose entities are confirmed present in the stores (design/registry/instructions), prose preserved as Obsidian body_ref where §7 keeps it: the `docs/` fold residue (user-journeys/personas/standards/specs metadata-folded), the oya/** IP/PRD residue, the registry/catalog yaml (after registry-store byte-confirms the 1:1).
- **Guard:** delete a carrier ONLY if (entity present in store) AND (body_ref resolves OR §7 says excise-prose). The entity-incremental-gate is the poka-yoke — it will not let a store accept an off-enum/dangling fold, so a green guard is the delete precondition.
- **Door + verify:** signed commit per coherent sub-tree. After each: stores still GREEN on all 4 guards; generated masterplan regenerates clean; `registry-drift` byte-equal.

### WAVE 5 — C-ENFORCE: WIRE THE GATES SO RECURRENCE IS IMPOSSIBLE (the poka-yoke close-out)
Turn this one-time sweep into standing automation (see §3). Wire the dead-code/stale-ref/doc-SSOT/anti-pattern gates into `oya-ci-required.yml` (or the backbone matrix where repo-wide). **Door:** one signed commit; the new gates must be born-blocking or advisory-until-infra per their disposition in `gate-disposition.json`. After this wave, **this map is archived** (git-history-only).

---

## §3 — GATE SEED (poka-yoke: failure = process-failure, so the process can't fail twice)

Each sweep finding seeds a standing gate so the sprawl/drift/dead-code cannot recur. All tie to the
`AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE` engine: **gate = pure fn over a producer-built
face** (`evaluate_keyed(&Value) -> BTreeSet<Finding>`), disposition is DATA in `gate-disposition.json`, the firewall
blocks `current \ baseline` regressions, the ratchet forbids new baseline keys without `gate-baseline.signoff.json`.

| Seed gate | Disposition | Fed by lane | Mechanism (face → predicate) |
|---|---|---|---|
| **dead-code gate** | advisory-until-infra → blocking | LANE 2 | producer emits a crate-reachability face; FAIL a crate iff `(no in-workspace reverse-dep) AND (not invoked by oya-ci-required.yml) AND (not a *-app/*-api/*-domain BUCK deploy-leaf)`. Prevents another oya-check-*/oya-dev-cli legacy pileup. |
| **dead-file gate (staleness-reaper)** | born-blocking (exists) | LANE 2 | extend the live `oya-cloud-ci-staleness-reaper-app` to flag completed-plan/todo pockets (`tasks/`, `.omc/plans/` milestone residue) and `_legacy-*` dirs. |
| **stale-ref / no-dangling-ref gate** | blocking | LANE 2 | face = the canon-id-crosswalk + ADR slug set; FAIL on any reference to a non-existent slug (`ADR-0091-multispectrum` phantom), a PHANTOM crate (the 8 in `_legacy-foundry`), a retired ident (foundry 2520 / jenkins 806 / multispectrum 183 / bominal 144 / oya-vcs 67 files) outside the allowed `evidence/` + retired-vocab gate. Extends the existing forbidden-vocab gate (§2.3 instructions-store). |
| **unaccounted-Proposed gate** | blocking | LANE 2 | FAIL on any ADR `status=Proposed` cited as live authority (ADR-0346/0347). Enforces masterplan-SSOT "resolve every Proposed". |
| **doc-SSOT / anti-drift gate (registry-drift + cross-artifact-agreement)** | born-blocking (exists) | LANE 1 | the 4 STORE-SCHEMA §4 guards + `registry-drift` (committed==regenerated byte-equal) + `cross-artifact-agreement`: an entity may live in exactly ONE store; masterplan is GENERATED (hand-edit = RED); off-enum/dangling-edge = RED; no second source of truth may reappear. This is THE gate that prevents the md/json sprawl from regrowing. |
| **no-tombstone gate** | blocking | LANE 1 | FAIL on any `superseded_by`/`_archive`/history-in-store (STORE-SCHEMA §0.1.2). Git is the only archive. |
| **rustfmt gate (`unformatted`, §H)** | AutoFix (highest-confidence) | LANE 3 | `rustfmt --edition 2024 --check` (12% sampled drift). Wire into `oya-ci-required.yml` (currently only on the backbone matrix). |
| **clippy gate (`non_idiomatic`, §H)** | AutoFix — BLOCKED until lints re-enabled | LANE 3 | the §H gate is toothless while `Cargo.toml:767-769` + `:1195-1207` set `dead_code/unused_imports/unwrap_used/expect_used/panic` to `allow` ("TEMPORARY Wave 15"). RE-ENABLE deny (per-crate `cfg(test)` allow) — the comment itself names this path. Then clippy --fix gates the 4648 unwrap / 5485 expect / 561 panic sites. |
| **hyperscaler-patterns gate (§A.4)** | AutoGenerate templated block | LANE 3 | face over `infra/**/*.k8s.yaml`; FLAG `missing_readiness_probe`/`missing_liveness_probe` (registry/openbao/observability) + emit templated probe blocks. |
| **scalability gate (§A.3)** | gated-core + fenced advisory | LANE 3 | `single_leader_unsharded` (6 single-replica stateful workloads) — **cross-check against declared single-node-local intent (ADR-0378) before blocking**; `synchronous_unbatched_fanout` (sqlx outbox await-in-for) = advisory, not AutoFix (may be intentionally serial). |
| **twelve-factor gate (§A.5)** | config_in_code=advisory · graceful-shutdown=advisory · logs=AutoFix | LANE 3 | FLAG non-overridable `pub const ..._DEFAULT_BIND_ADDR` (the real config_in_code, not the env-overridden defaults); FLAG `*-app` mains with no SIGTERM handler. (logs-to-stdout already compliant — 0 file-logger findings.) |
| **DX / dev_env seam gate (§A.2 hermeticity)** | advisory | LANE 3 | flag the manual `direnv allow` setup step + the missing `oya init`/scaffold subcommand (clone-then-it-just-works gap). Ties to the hermetic-just-works doctrine. |

---

## §4 — RISKS (load-bearing files that LOOK stale but are LIVE; the over-delete posture)

1. **`oya-check-brand-residue` is NOT dead** — it is the one oya-check-* the active `oya-cloud-ci-accounting-registry-app` producer path-depends on. Deleting the family blindly breaks the producer → the whole required CI. The keep-list pins it; the dead-code predicate excludes it.
2. **The 246-no-reverse-dep lib set is a SUPERSET, not a dead-list.** It includes legitimate `*-app`/`*-api`/`*-domain` deploy leaves reachable via buck/k8s, not cargo deps. **The per-crate BUCK binary-target enumeration (763 BUCK files) is the verification step that MUST precede any A-delete on the broad set** — it was NOT done in the sweep (LANE 2 explicit caveat). The predicate's `(not a deploy-leaf in BUCK)` clause is the guard.
3. **`oya-dev-cli` is wired into `Makefile:43`** (`verify-deploy-contract`). It is NOT hard-removable until that target is rewired/dropped first (separate commit, ordered before its delete). Also `bin/oya` is LIVE in `backbone-microservices-ci.yml:313` (`./bin/oya gate validate cargo-prefix`) — do NOT delete `bin/oya` while that CI leg exists.
4. **Gate baselines + fixtures look like stale JSON but are gate-load-bearing.** `gate-baseline*.json`, `gate-baseline.signoff.json`, `gate-disposition.json`, the gate fixtures, and live `evidence/ci` baselines are the firewall's frozen-key memory. Deleting them silently lets regressions through. KEEP-list pins them; they are NEVER folded into the SSOT stores (they are gate-output/config data, a separate axis from the 4 stores).
5. **`registry/catalog/*.yaml` must survive UNTIL registry-store byte-confirms the 1:1 fold.** Delete in Wave 4 only after the fold is verified — never speculatively in Wave 3.
6. **The 22 `superseded_by` ADRs:** "archive-superseded" means git-history-only EXCISION, NOT a kept `_archive/` dir (STORE-SCHEMA §0.1.2, OPEN-1). Building an `_archive` tree would itself be a no-tombstone gate violation.
7. **`user-journeys` (913 md) fold-vs-archive is a FOUNDER CALL** — the largest non-ADR md pocket; flagged for explicit decision before Wave 2 fold/Wave 4 delete. Do not auto-archive durable journeys.
8. **Concurrent-executor drift:** counts are snapshots (HEAD moved `3319a2bab`→`38bc5797b` during the sweep). Re-verify counts at execution; do not act on the sweep's frozen numbers without a fresh `find`/`cargo metadata`/`git ls-files`.
9. **Over-delete posture (the safety net):** every destructive batch is one signed commit on top of recovery anchor `e38624dc4` (pushed to `github-mirror`). KEEP-POSTURE means "when unsure, KEEP and let a gate adjudicate later" — but over-delete is explicitly recoverable from `e38624dc4`, so a *false delete* is a `git restore`, whereas a *broken gate* is unshippable. The asymmetry favors: protect the build-graph/CI spine absolutely, be aggressive on doc/plan/runtime-scratch sprawl.
