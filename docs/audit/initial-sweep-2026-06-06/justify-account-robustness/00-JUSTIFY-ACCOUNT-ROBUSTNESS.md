# 00 — JUSTIFY / ACCOUNT / ROBUSTNESS — MASTER SYNTHESIS

> **Lane:** justify-account-robustness (synthesizer). **Date:** 2026-06-06. **Mode:** READ-ONLY synthesis of the six lane artifacts (`10-*.md` in this folder); every cited line was verified live by the lane authors against `/Users/jasonlee/Developer/source` on 2026-06-06. No source file mutated.
> **Charter:** FOUNDER D-DOCTRINE — (a) hyperscaler monorepo doctrine, (b) Linus taste, (c) arch invariants, (d) ROBUST-NOT-FALSE, (e) TOTAL ACCOUNTING. Subject tree = `source/`; artifact home = `linux/docs/audit/...` (distinct trees, handled correctly throughout).
> **Extends, does not redo:** `synthesis/01-ADR-DISPOSITION-TABLE.md`, `synthesis/decision-record-oyatie-canon.md`, `backlog-reconciliation/00-BACKLOG-RECONCILIATION.md`, `docs-sweep/00-REST-OF-DOCS-REGISTER.md`, `monorepo-conformance/00-CONFORMANCE-REGISTER.md`.
> **Source artifacts synthesized:** `10-justify-decisions.md`, `10-arch-invariants.md`, `10-total-accounting.md`, `10-robustness-enforcement.md`, `10-doc-reorg-plan.md`, `10-enforcement-primitives.md`.

**ONE-LINE VERDICT:** The source monorepo's enforcement *logic* is largely real and fixture-backed, but its enforcement *reality* is advisory — the single required CI context (`oya-ci-required`) has **no proven live producer**, so **0 lanes actually block a merge today**; meanwhile multiple Accepted ADRs assert completion that the live tree disproves (Foundry "eradicated", pure-split "complete"), the canonical structural invariants (layer enum, bounded-context coupling, visibility fences) are defined-or-false-enforced, and the OWNER + reachability halves of total-accounting are essentially un-enforced tree-wide. The fix is **one generated accounting registry + four keystone gates that are born blocking the live exhibits**, feeding **one unified amendment campaign** under founder sign-off.

---

## (1) JUSTIFICATION — decisions to AMEND / ARCHIVE under the charter lens (highest-severity first)

These EXTEND the verified disposition table; each is a charter violation the prior audit missed or under-weighted, confirmed file+line live 2026-06-06.

| # | Decision | Verdict | Charter clause | Evidence (path:line) | Action |
|---|---|---|---|---|---|
| **S1** | ADR-0363 false "eradicated" claim | **AMEND (mandatory)** | (d) robust-not-false + (e) accounting | `ADR-0363:35` verbatim: *"The Foundry name was eradicated … `microservices/foundry/` (597 files) was kept"* — self-contradicting in ONE line; status `Accepted` | Drop "eradicated"; enumerate residue (11 canonical dirs incl. **live** `docs/teams/axis-foundry`; honest = 11 dirs + 4110 file-level mentions, NOT the worktree-inflated 2180/3771); wire a proven `no-foundry-token` RED/GREEN gate |
| **S2** | 0511↔0513 dual CI-destination, supersession missing BOTH ends | **AMEND (mandatory, both files)** | (b) stable contracts + (e) accounting | `ADR-0511:3,19` (Proposed) names **Argo Workflows** "the destination CI orchestrator" (founder-basis 2026-05-29); `ADR-0513` (Accepted 2026-05-30) makes **bespoke-Rust-Prow** the destination, `relates:[0380,0111,0116,0374,0363,0392]` OMITS 0511; 0513 has no `supersedes` keys; broken chain `0359→0511→(gap)→0513` | Set `0511 Superseded` / `superseded_by:[0513]`; add reciprocal `supersedes:[0511]`; consolidate 0349/0359/0361/0408/0511/0513/0514 → **ONE** ADR (0513); strike Tekton from "four faces" (zero Accepted support) |
| **S3** | dup-0377 + non-enum free-text status | **AMEND (renumber one)** | (e) accounting + (b) good-data-structures | Two files share `id: ADR-0377` — `ADR-0377-kafka-to-pulsar-via-kop.md` (Accepted) AND `ADR-0377-forgejo-board-git-ref-cas-fallback.md` (status free-text `Proposed (conditional: …)`, not an enum member) | Renumber the forge variant into the free block `>0514`; normalize status to enum `Proposed`, move condition to body |
| **S4** | phantom `ADR-0150-cedar-policy-engine.md` cited BY FILENAME | **AMEND (assign real id)** | (e) accounting + (b) stable contracts | `ADR-0150` on disk is **cursor-pagination**; `ADR-0243:17,28,101` cites `ADR-0150-cedar-policy-engine.md` (no such file); ≥10 ADRs reference 0150; the Cedar engine keystone (D6) has NO real id | Assign Cedar-engine a real id in the free block; repoint all `-cedar-policy-engine.md` citations; leave 0150-cursor-pagination untouched |
| **S5** | ADR-0335 TWO status lines | **AMEND** | (d) robust-not-false + (e) accounting | `ADR-0335:3 status: Accepted` AND `:771 status: completed-locally` (latter not an enum member) → masterplan generator silently picks one (latent false-green) | Remove the `:771` stray; if a real sub-state, move to a body field |
| **S6** | `axes_count` 6 ≠ 7 live | **AMEND + GATE** | (a) generated-not-hand-maintained + (d) robust | `catalog.json:12 "axes_count": 6` vs `contracts.json:9 "axes_count": 7` (DESIGN.md:17,23-31 enumerates 7 incl. NEW Workspace axis) | GENERATE `axes_count` from the single axis enum; add cross-artifact-agreement gate; DO NOT hand-fix one number (recreates drift) |
| **S7** | Structure ADRs encode the retired 3rd tree | **AMEND (both)** | (c) min-blast-radius / D-PURESPLIT two-tree | `ADR-0131` references `microservices/` **×15**; `ADR-0512` **×6** — both Accepted — contradicting the founder pure-split (`oya/`+`cloud/` only) | Rewrite all `microservices/` → `{oya,cloud}/<service>/`; state two-tree-only rule. Prior table softened this to "drop examples" — this is an arch-invariant contradiction, not cosmetic |
| **S8** | Linus-taste failures | **AMEND** | (b) no-special-cases / distrust-over-abstraction | `ADR-0368:20` *"the fleet is kept at maximum safe concurrency at all times"* (idle=defect over-abstraction); `ADR-0109:114` *"Both patterns are canonical"* + self-flagged Pattern-B carve-out vs `:38` no-exceptions-canonical | 0368 → reframe to "capacity-bounded, M0-gated parallelism" (D8); 0109 → collapse to ONE parameterized lifecycle-kernel shape |

**CI-cluster net (§1 of `10-justify-decisions.md`):** DROP 0349/0361 (never-ratified Jenkins debt) + 0359-tombstone; AMEND-metadata 0511(Superseded)/0513(canon); AMEND-in-place 0408/0514 (adopted Buck2 substrate). Matches founder D-CICD + backlog T-CI; the charter ADDS the machine-proof requirement (no "consolidated" claim until the supersession graph is acyclic + complete, gate-verified).

**Identity/policy/data/event spot-rejudgements:** 0476/0187 (Zitadel→oya-identity demotion, missing `superseded_by` edge + Cedar mis-cited as 0083); 0243/0246 + 0457/0429/0443/0428-refs (all anchor Cedar to phantom 0150 or mis-cite 0083 → repoint to the real Cedar id, S4); 0005 ARCHIVE (Kafka broker retired-in-fact, 0377-kafka supersedes); 0006 fix self-referential "Ontology→Ontology" tautology; 0045 fix "Citus=AGPL" factual error.

---

## (2) ARCH-INVARIANT VIOLATION REGISTER — enforced vs aspirational

Distinction the lane forces: a rule can be **DEFINED + have a real fixtured checker + still be UNENFORCED** because the checker is (a) wired to zero lanes, (b) wired to the WRONG check, (c) a pure SCAFFOLD returning Ok, or (d) report-only. Three of those four false-enforcement shapes are live in source.

**GENUINELY ENFORCED (real fixtured gate, live lane, fail-closed) — the robust exemplars:**
- **I5 data_class on kernel fields** — `data_class_gates.rs` walks every `*-kernel` src, fail-closed, foundation lane active. Caveat: 289-row `legacy-unannotated-fields.tsv` allowance with no visible TTL.
- **I6 package.name==basename + oya-* prefix** — 0/723 basename mismatches; oya-* live-enforced + fixtured (`architecture_boundaries.rs:461`, fixtured `:1169`) on a real BLOCKER lane.
- **I2a dependency-direction matrix** — REAL + fixtured (`architecture_boundaries.rs`, runs `cargo metadata`, rejects forbidden edges, RED/GREEN self-tests) — **but drifted-permissive** (see below).

**DEFINED-but-NOT-ENFORCED / FALSE-ENFORCED (the charter's target set):**

| # | Invariant | State | Evidence | Fix |
|---|---|---|---|---|
| I1a | BNF 13-layer enum machine-readable | **3 disagreeing copies** | ADR-0056 decision (`:69`, has `application`) vs `layer-enum-adr-0105.md:64-88` (has `check`, NO `api`) vs `predictable-naming-kernel/src/lib.rs:32` ALLOWED_ROLES (has `api`, no `check`). Hand-maintained ×3, drifted | GENERATE enum from one SSOT |
| I1b/c | terminal-token∈enum + catalog role∈enum | **NO (checker orphaned)** | The one real+fixtured checker (`predictable-naming-kernel`) is wired to **ZERO** of the 97 lanes, absent from `run_all.rs`. Dark-checker would catch **74 non-enum crate dirs** (oya-identity, *-runtime, *-service×6, *-application, non-sanctioned -adapter-stripe/-cedar/…) + **137 catalog roles outside enum** (api×45, runtime×55, application×23 [retired ADR-0106], test×11, bindings×3) | Wire into the required roster + RED/GREEN proof |
| I2b | ports-in-kernel / impls-in-adapter | **NO** | No code-shape checker; the ADR-0105-cited `crates/oya-dev-cli/src/layered_architecture_gates.rs` **DOES NOT EXIST** | Author the code-shape gate |
| I2c | no_std-kernel | **NO** | 0/136 `*-kernel` declare `#![no_std]`; 2 kernels (`oya-cloud-intelligence-kernel`, `oya-ci-controller-kernel`) pull tokio/sqlx/reqwest | Author no_std gate (caveat: judged by manifest grep, not a no_std build) |
| I2a-drift | role-table permissiveness | **PARTIAL/false** | `allowed_dependency_roles()` re-admits `application`/`runtime`/`test`/`bindings`/`api` — the EXACT roles the naming-kernel REJECTS (the two gates contradict each other); reads role from catalog not crate-name (R-002 unenforced); gives `api` an edge to `app` | Reconcile to the strict closed enum; cross-check suffix==role |
| I4a | visibility fences (min-blast-radius) | **NO (HIGH)** | 781/832 BUCK targets `visibility=["PUBLIC"]`, 51 scoped, **0 PACKAGE files** → Buck2 coupling unbounded | Add PACKAGE defaults + scoped visibility + gate |
| I4b | one-version | **PARTIAL** | 636 inherit `version.workspace`; **66 hardcode** a version; no gate | Add `version.workspace` gate to 100% |
| I4c | cross-microservice refusal (LEAN-A2) | **NO (false-enforcement, HIGH)** | Lane `lean-a2-bounded-contexts` (BLOCKER, source ADR-0056, purpose "no cross-µservice deps") dispatches `gate validate cedar-fragment-coverage` — a **verbatim copy-paste WRONG CHECK**; isolation unenforced while claiming BLOCKER | Wire the real bounded-contexts checker |
| I4d | tenant-boundary oya→cloud | **REPORT-ONLY** | `architecture_boundaries.rs:558-599` computes oya→cloud edges, PRINTS, never fails (`:591 REPORT-ONLY`, fixtured `:1393`) | Promote to fail-closed |
| I3a | affected-targets (ADR-0360) | **ASPIRATIONAL** (honest) | `ADR-0360:1 Proposed`, `:24` evidence-blocked | Build, then ratify |
| I3b | one-lane-one-path (ADR-0366) | **ASPIRATIONAL** (Accepted, gates absent) | Accepted door:one-way cites 6 `verified_by` gates (concurrent-safe-paths, merge-queue-health, self-repair-coverage, definition-of-done, error-budget-policy, dora-metrics) — **NONE** exist in 97 lanes or run_all.rs | Build the 6 gates or downgrade the contract |

**EXTENDED FALSE-ENFORCEMENT EXHIBITS (beyond the founder's named set):**
1. `oya-shared-architecture-check-cli` — pure SCAFFOLD; all 7 subcommands (incl. DependencyDirection/LayerCorrectness/Report) print `"SCAFFOLD"` and `return Ok(())` (`main.rs:42-74`). The ADR-0056-named LEAN-A1 orchestrator enforces NOTHING.
2. `oya-check-layered-architecture-discipline` (BLOCKER lane) actually checks ADR-0148/0182/0183/0184 mesh-config (`lib.rs:1`), NOT the code layer enum — a **naming false-affordance** (real+fixtured for its real domain, but the name implies the hexagonal enum is gated; it is not).
3. LEAN-A2 wrong-check (I4c). 4. ADR-0366 phantom gates (I3b). 5. orphaned predictable-naming-kernel (I1b).

**NET:** strong on cheap mechanical invariants, weak-to-false on the expensive structural ones that bound blast radius. Two BLOCKER-labeled arch lanes + one mis-named BLOCKER produce a **FALSE GREEN on exactly the hexagonal/bounded-context invariants** — so "merge green" does not mean "layer-conformant" even for the existing 723 source crates, the same gaps the conformance register flagged for the migrants.

---

## (3) TOTAL-ACCOUNTING LEDGER — orphans / unaccounted / sprawl + the generated schema

**COVERAGE:** all 27 `source/` top-level entries fully walked (`ls` + `git ls-files` excl `/target/` + 4-registry reachability grep each); `docs/` (87) + `specs/` (106) cluster homes fully enumerated + dedup-checked.

**HEADLINE FINDINGS:**
- **O-1 SYSTEMIC — ZERO `OWNERS` files tree-wide.** `find -iname OWNERS` = 0 (only `.github/CODEOWNERS` + 2 advisory mirror docs); `^owners:` frontmatter = 96/2358 docs (~4%), near-0% of code. The OWNER half of total-accounting is essentially un-enforced — the **largest D-DOCTRINE gap, not in the prior audit.**
- **O-2 REACHABILITY GAP — canonical CODE trees unreachable from masterplan.** `oya/`·`cloud/`·`libs/` = **0 hits** in `masterplan.json` AND `root-hub-pointers.json`; reachable ONLY via `Cargo.toml` members (433/100/168). Masterplan SSOT enumerates docs/specs but not the code it governs. Fix = make Cargo.toml (or a generated code-tree manifest) a declared companion registry (reachable-by-construction).
- **ORPHAN HUSKS (0 git-tracked → ARCHIVE/DELETE):** `crates/` (only `.DS_Store`; real crates at `oya/.../crates/` per `Cargo.toml:181`), `services/` (0 tracked), `test-results/`, `memory/` (2 stale notes); `platforms/` (1 untracked BUCK) → MERGE-then-delete. Re-confirms T-STRUCT sprawl with git-tracking proof.
- **UNACCOUNTED (real content, in NO registry):** `scripts/` (also collides pillar-Q no-new-.sh), `benchmarks/`, `third-party/`, `packs/`, `toolchains/`, `memory/`.
- **DUPLICATES:** DUP-1 `docs/specs/` (110 `task-*.md`) ⟷ `tasks/` (110 `*-plan.md`) = exact 110-slug overlap + collides `/specs`; DUP-2 `templates/` ⟷ `docs/templates/` = 13 identical basenames + `-v2` split-brain; DUP-3 pack sprawl = 4 homes; secondary `docs/products`⟷`specs/products`, `contracts/*.proto`⟷`specs/proto`.
- **STALENESS:** only TTL primitive = `_sunset.sunset_at` in machine JSON; NO TTL on any markdown corpus or husk dir → nothing auto-archives (the ai-slop pileup). `evidence/` (1532, dated/aborted snapshots), bulk corpora `runbooks/172·personas/133·user-journeys/190` at ~4% owner coverage.

**PROPOSED GENERATED `registry/accounting-ledger.json` schema (every accounted path carries):** `path · unit_class · owner (OWNERS-derived, gate fails if absent) · justification_ref (→ADR/D-ruling/spec) · reachable_from (must be non-empty) · reachability_ok (bool) · ttl_policy · staleness_status · tracked (git ls-files count) · verdict (KEEP/ARCHIVE/MERGE/NEEDS-OWNER, auto-derived) · dup_of`. Enforced by a blocking gate with RED fixtures (a husk dir / owner-less crate / unreachable tree must each turn RED). Unifies existing fragments: `registry/catalog/*.yaml` (per-crate), `docs/machine-readable/catalog.json` (per-doc), `markdown-retirement-policy.json._sunset` (TTL).

---

## (4) FALSE-ENFORCEMENT REGISTER — every claim-to-enforce that does not block, + the fix

**HEADLINE RATIO:** ~96 lanes claimed-as-enforcement (`ci-lanes.md`) / 91 registered-active + 5 planned (`lanes.yaml`) / 109 aggregated (`gate run-all`) / **0 proven-blocking in an executing CI today**. ≥17 lanes self-declare "advisory/deferred/until…lands" yet sit in the enforcement catalog.

| FE | Sev | Finding | Evidence | Fix-to-make-it-real |
|---|---|---|---|---|
| **FE-1** | P0/P1 (APEX) | The sole required branch-protection context `oya-ci-required` has **NO proven live producer** | `infra/branch-protection/dev.json:2` + `.github/branch-protection.yaml:2-5` both disclaim the config is applied ("not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live"); producer is `oya-ci-controller-kernel:471`, wired only into Helm/ArgoCD scaffolding | Stand up + PROVE the producer posts `oya-ci-required` on a real SHA; apply ruleset; snapshot live required-checks. **Nothing else is real until this.** |
| **FE-2** | P0 | `protection-context-match` (the gate built to KILL silent-bypass) would itself FAIL against live config | Required = `[oya-ci-required]`, posted by NEITHER `reported-status-contexts.json`'s 17 names NOR the one workflow's job names; kernel does exact string-match no carve-out (`lib.rs:146-152`) → fixture-backed but unfired = the exact silent-bypass it forbids | Run it in an executing presubmit (goes RED today, forcing FE-1); add a wired RED proof |
| **FE-3** | P1 | THREE disagreeing "required context" lists; producer/required intersection EMPTY | branch-protection `[oya-ci-required]` vs `oyaCiLane.groovy` 16 vs `reported-status-contexts.json` 17 (+ a 4th in `Jenkinsfile:3`) | Single SoT (the registry), GENERATE all four config artifacts from it, assert equality |
| **FE-4** | P2 (extends) | ADR-0363 "Foundry eradicated" still false; dead brand wired into the live catalog | 4,110–4,714 residue files; `ci-lanes.md` still wires `foundry-eval-nightly` (118), `axis-foundry` owner (8,63,95), `foundry-tool` wasm class (90) | Execute sense-routed rename; fix the claim; make `brand-residue` BLOCKING (RED fixture) |
| **FE-5** | P2 (extends) | `prd-axis-coverage` + `diataxis-doc-class` are NOT registered lanes at all | Absent from `lanes.yaml`; live only in docs/specs/evidence — worse than "defined-not-active" | Register with check_command + RED/GREEN, OR demote every doc claim that implies they enforce |
| **FE-6** | P2 (revises) | "22 governance crates unwired" is STALE → **59** (libs×39, tools×17 + kernels); all 91 active lanes have a check_command; substance collapses into FE-1 | per-crate census | Correct the count to 59/91/109; resolve FE-1; then per-crate RED/GREEN fixture audit |
| **FE-7** | P1 | `axes_count:6` stale vs 7 (hand-set, should be generated) | `catalog.json:12` | Generate from axis enum; add `axes_count != len(axes)` check |
| **FE-8** | P1 | Foundation-bypass `byp_adr_0346_oya_verify_ci_mirror` EXPIRED by 2 days | created epoch-day 20594 + 14 = 20608; today 20610; it bypasses the `oya verify` ↔ `gate run-all` mirror requirement. Bypass-kernel logic robust + fixtured (`blocks_on_expired_entries_in_block_phase`) but lane runs only on the not-live farm | Run `foundation-bypass` in presubmit (RED today); renew-with-justification or close by making `oya verify --ci-required` block on `gate run-all` |
| **FE-9** | P0 | D-PURESPLIT "ERADICATE everything else" (door:one-way) violated | `services/` (6 dirs) + `platforms/` (1) live; `crates/` vs `oya/.../crates/` split-brain; no BLOCKING lane enforces service-tree purity | Add a BLOCKING `service-tree-purity` lane (RED fixture); migrate/archive the 6+1; do not claim "complete" until green |
| **FE-10** | P2 | Naming collision — TWO unrelated "claim-ceiling" gates | `oya-check-claim-ceiling` (ADR-0037 maturity, fixtured/real) vs `oya-governance-claim-ceiling-kernel` (ADR-0054 agent claim-DEPTH ratchet) | Rename the depth ratchet to `agent-claim-depth-ceiling`; reserve "claim-ceiling" for #21 |

**ROBUST (credit, so the fix-list is honest):** the ADR-0221 ×4 bash gates are genuine RED/GREEN fixtures (`adr-0221-governance-gates.sh` builds bad+clean fixtures per gate); bypass-kernel, ADR-0037 claim-ceiling, and protection-context-match KERNELS have real blocking logic + unit fixtures. **Uniform failure mode = FE-1: no live producer runs them.** ROOT CAUSE: gate LOGIC is largely real; gate ENFORCEMENT is advisory because the producer is not proven live, branch-protection is a self-disclaimed target, and the human-readable mirror presents advisory/planned lanes as blocking.

---

## (5) DOC-REORG PLAN — Diátaxis topology + unified per-doc record + the enforcing gate

**CORPUS CORRECTION (load-bearing):** the reorg corpus is `/Users/jasonlee/Developer/source/docs/` (44 subdirs / 2888 files), NOT `linux/docs/`. All scheme files (`doc-style.md`, `DOCUMENTATION.md`, `DESIGN.md`, `catalog.json`, `contracts.json`, `planning-ssot-consolidation.md`) exist ONLY under `source/docs/`. Plan written so the same topology+record+gate apply to `source/docs/` now and the migrated `linux/stack` docs after consolidation (one-version).

**(a) TOPOLOGY — 44 subdirs collapse to 6 top-level homes** (Diátaxis 4 + Project + immutable decisions; evidence `doc-style.md:41-52`, `DOCUMENTATION.md:25`, `planning-ssot-consolidation.md:107-113,134`):
- `tutorials/` (≤500) ← tutorials/ + onboarding/
- `how-to/` (≤300) ← runbooks/ checklists/ release/ advanced-cicd/ operators/ customer-success/
- `reference/` (≤600, GENERATED build-output) ← specs/ api/ machine-readable/ standards/ localization-packs/ regional-packs/ performance-budgets/ policies/ automation/ + `reference/journeys/` (user-journeys 1413 + personas 131 = 53% of corpus, MUST be generated from templates)
- `explanation/` (≤400) ← architecture/ ideas/ research/ teams/
- `_project/` (generated-reference) ← products/ prds/ plans/ gtm/ investor/ governance*/ quality/ audits/ + top-level MASTERPLAN/PRD/ROADMAP/RISK/DESIGN/SPEC/GLOSSARY/DOC-CATALOG (MASTERPLAN→GENERATED-REFERENCE closes CC-3)
- `decisions/` (355) UNCHANGED, immutable — the ONLY Diátaxis-exempt namespace (ADRs=SSOT), the single principled special case
- `foundry/` sense-route (CC-1) before filing; raw/ wiki/ site/ harness/ ci/ agents/ → triage explanation/ or ARCHIVE.
Invariants: exactly-one-home (`doc-style.md:180`), reference/=build-output-never-hand-edited, top-level = quadrant roots only.

**(b) DRIFT RECONCILED:** (1) `catalog.json:12 axes_count:6` stale vs `products.json axes:7` + `DESIGN.md` 7 → emit from the DESIGN §1 axis enum, never hand-set. (2) competing 5th-quadrant names Project vs decision → split `_project/` (generated) + `decisions/` (immutable). (3) doc-catalog lane reads non-existent `docs/CATALOG.md` (real = `docs/DOC-CATALOG.md`; `governance-lanes/doc-catalog.md:8,13`) → live false-enforcement, repoint input. (4) CC-3 authority inversion folded into the reachability field.

**(c) UNIFIED PER-DOC RECORD (`doc-records.generated.json`):** `{id, path, doc_class[Tutorial|HowTo|Reference|Explanation|Project|Decision], axis[7-enum|cross-cutting], tier[0|1|2|3|cross-cutting], owner_team, reachability{class[DECISION|INSTRUCTION|GENERATED-REFERENCE|ORPHAN], source_ref, in_masterplan}, generated, agent_authoring_allowed, ttl{update_cadence, last_reviewed, stale_after_days}, validation_check[], dependent_docs[]}`. Every field traced to an EXISTING attribute (provenance table in `10-doc-reorg-plan.md` §c). Supersedes the split `CatalogRow`; one shape for all docs incl. ADRs (Linus no-special-cases); `.generated.json` convention already used by `masterplan.generated.json`.

**(d) GATES (advisory→BLOCKING, each proven by a committed RED fixture AND present in `lanes.yaml` status:active):**
- `diataxis-doc-class`: spec Accepted (`governance-lanes/diataxis-doc-class.md:5`) but NOT in lanes.yaml = PLANNED-NOT-BLOCKING → promote to active+BLOCKER (report-only→error@day-8 soak); RED = its own `failure_modes`.
- `prd-axis-coverage`: NO lane spec, only a validation_check STRING (`DOC-CATALOG.md:306`) = DEFINED-NOT-ACTIVE → author lane+kernel+entry; RED = PRD missing the 7th (workspace) axis fails; sources the live enum so stale axes_count can't pass.
- `doc-catalog`: IN `lanes.yaml:44` BLOCKER but reads non-existent `docs/CATALOG.md` + only checks row-coverage = ENFORCED-BUT-BROKEN → replace with `oya-governance-doc-record` lane (input `docs/DOC-CATALOG.md` + `doc-records.generated.json`); RED = doc missing axis must fail.
- `axes_count` generation guard folds into the doc-record lane.
Robust bar: no gate counts as "wired" until its committed RED fixture blocks in CI; also wire the unwired `oya-governance-*` crates the lanes depend on.

---

## (6) ENFORCEMENT-PRIMITIVES — the 4 keystone gates (RED/GREEN-proven)

All four = buck2-native Rust producers, generated-not-hand, required-context-not-advisory, sharing ONE `accounting-registry.generated.json` (Linus "good data structure kills the special cases"); all ship in the **G-INTEGRITY track** (specs+filesystem, no buck2-build-graph dep) so the false-green firewall lands Phase-0 before the migration. Shared contract: every generated face asserts `committed == regenerated` (hand-edit = RED → drift structurally impossible); producer is always a buck2 Rust crate, **never a new `oya` CLI command** (register #20, directly retiring ADR-0365's `verified_by: oya gen propagate --check` defect); report-then-archive never delete-on-unverified-verdict.

1. **CROSS-ARTIFACT-AGREEMENT** (`cloud-ci-cross-artifact-agreement`; amends ADR-0365, de-dup O2). Maps a decision → {ADR, spec, masterplan, roadmap} via generated `decision-crosswalk`. BLOCKS: orphan-decision · unpropagated-decision · status-disagreement · generated-face-drift · dual-decision-collision · supersession-half-edge. **The gate whose absence let the two consensus bodies drift.** PROOF: 7 fixtures freezing LIVE exhibits — `axes_count:6 vs 7`, dup-0377, 0511↔0513 half-edge — must reproduce RED on the current corpus.
2. **TOTAL-ACCOUNTING** (`cloud-ci-total-accounting`; owns the registry). `git ls-files × OWNERS × ADR-justification × masterplan-reachability`. BLOCKS: unaccounted · unowned · unjustified · unreachable · no-ttl-class · registry-drift. Auto-archive = report-then-git-mv-to-`_archive/`, never rm, second-verifier-gated. PROOF: 7 fixtures; self-test must flag the live 780 `oya-foundry-*` (unjustified vs 0363's false "eradicated") + 57 unwired `oya-governance-*` (unreachable).
3. **STALENESS REAPER** (`cloud-ci-staleness-reaper`; §G sinker++, linux Task-#14 >48h class). Gate-2 registry + generated `ttl-policy` per resource-class (worktree/branch/artifact/image/process/stale_doc) + git-log last-touch. Report-then-archive. BLOCKS: stale-over-budget-AND-unreachable · untyped-staleness · reap-without-report. Age alone ≠ stale; protected classes never reaped (declared in policy data). PROOF: 7 fixtures incl. the `_partial`/`_verify` ai-slop scratch-doc archive class.
4. **AUTOMATION-RATCHET** (`cloud-ci-automation-ratchet`; register #20). **Seed `phase0-automation-matrix.json` + 4 RED/GREEN fixtures ALREADY ON DISK** — hardened, not invented. Matrix + generated `enforcement-inventory` + live buck2 target set. BLOCKS: enforceable-marked-human-judgment · advisory-claiming-enforced(no wired target) · oya-cli-authority · incomplete-exception · no-retirement · ratchet-regression(monotonic). PROOF: reuse 4 live fixtures + 2 net-new; self-test flags 57 `oya-governance-*` crates, `diataxis-doc-class`, `prd-axis-coverage`, ADR-0365's `oya gen`-bound `verified_by` as advisory-claiming-enforced/oya-cli-authority. **Gate-4 polices Gates 1-3** — none flips to `automated_blocking_now` until its self-test reproduces its live exhibits as RED (#21 applied to the gates themselves).

Composition: Gate-2 is the producer of record; Gates 1/3/4 are predicates over its registry. ONE registry, four predicates, no four parallel scanners.

---

## (7) THE SEQUENCED ACTION LIST — the UNIFIED amendment campaign (dependency order)

Mapping convention: **A1-A6** = the single-owner amendment lanes already in the consolidation plan; **net-new ADR** = a `>0514` free-block entry; **P0-build** = a Phase-0 G-INTEGRITY enforcement build. 🚪 = door:one-way / founder-sign-off point.

**PHASE 0 — FALSE-GREEN FIREWALL (build the enforcement substrate FIRST; everything downstream is unprovable without it).** Independent of the build-graph migration; ships before consolidation mutation.

1. **🚪 FE-1 APEX — stand up + PROVE the `oya-ci-required` producer**, apply the ruleset, snapshot live required-checks. *Nothing below is "enforced" until this is real.* (robustness §6 step 1; founder go required — this is the authority point for the whole CI claim.)
2. **Build the 4 keystone gates as P0-builds** (§6), each born blocking its live exhibits, in the G-INTEGRITY track:
   - Gate-2 total-accounting first (owns `accounting-registry.generated.json`) → Gates 1/3/4 as predicates over it. (closes O-1 owner, O-2 reachability, DUP/SPRAWL/STALENESS; feeds the §3 ledger schema.)
   - Gate-1 cross-artifact-agreement — the machine-proof that makes S1-S7 amendments mechanically true (its RED fixtures = axes_count 6≠7, dup-0377, 0511↔0513 half-edge).
   - Gate-4 automation-ratchet — reuses the on-disk seed matrix + 4 fixtures; **polices Gates 1-3** (no gate may self-certify "enforced").
3. **FE-2/FE-3 — single SoT for required contexts**, generate all four config artifacts from it, assert equality via `protection-context-match --live-required-contexts`; add the wired RED proof.
4. **Wire the orphaned/dark real checkers into the required roster:** `predictable-naming-kernel` (I1b), real bounded-contexts checker replacing the LEAN-A2 cedar copy-paste (I4c), foundation-bypass lane (FE-8), `brand-residue` (FE-4). Promote tenant-boundary oya→cloud from report-only to fail-closed (I4d).

**PHASE 1 — UNIFIED ADR/DOC AMENDMENT CAMPAIGN (gated on the Phase-0 gates existing so each amendment is verified, not prose).** These fold into A1-A6 single-owner lanes; **mutate source only after founder-go + WIP-commit-first** (per consolidation execution state).

5. **A-CI consolidation (merges S2 + CI-cluster net):** 0349/0361 DROP, 0359 tombstone, 0511→Superseded-by-0513, 0408/0514 AMEND-in-place, fold 0111/0116; consolidate to ONE CI ADR (0513). 🚪 founder D-CICD already ruled bespoke-Prow; the amendment makes the supersession graph acyclic+complete, Gate-1-verified. Strike Tekton from "four faces."
6. **A-STRUCT (merges S7 + FE-9):** rewrite `microservices/` in ADR-0131 (×15) + ADR-0512 (×6) → `{oya,cloud}/`; add the BLOCKING `service-tree-purity` lane; migrate/archive `services/`(6)+`platforms/`(1); collapse the `crates/` split-brain. 🚪 D-PURESPLIT is door:one-way — do not claim "complete" until the lane is green.
7. **A-FOUNDRY (merges S1 + FE-4):** execute the sense-routed foundry rename (platform→oya-intelligence / fitness→oya-governance / vcs→retired); rewrite ADR-0363:35 to drop "eradicated" + enumerate residue; `brand-residue` proven blocking.
8. **A-INTEGRITY (merges S3+S4+S5+S6 + FE-7):** renumber dup-0377 forge variant >0514; assign Cedar-engine a real id + repoint all `-cedar-policy-engine.md` and `0083` mis-cites (0243/0246/0476/0457/0429/0443/0428); remove ADR-0335 `:771` stray; GENERATE `axes_count`. All become RED fixtures inside Gate-1 (already frozen there).
9. **A-TASTE (S8):** AMEND 0368 (capacity-bounded, M0-gated) + 0109 (one parameterized lifecycle-kernel). 🚪 founder taste call on framing.
10. **A-IDENTITY/DATA:** 0476/0187 supersession edges; 0005 ARCHIVE (Kafka broker); 0006 tautology fix; 0045 Citus-license fix. (branch-locality: 0421/0457/0429/0443/0428/0488 re-resolve at merge from origin/dev.)

**PHASE 2 — DOC-REORG + ACCOUNTING CLOSURE (after the gates + amendments land).**
11. **Doc-reorg (§5):** 44→6 Diátaxis homes on `source/docs/`; unified `doc-records.generated.json`; promote `diataxis-doc-class` + author `prd-axis-coverage` + replace broken `doc-catalog` with `oya-governance-doc-record` (FE-5); fix the `docs/CATALOG.md` path bug. DUP-1/2/3 merges.
12. **Total-accounting closure:** create OWNERS files tree-wide (closes O-1); declare Cargo.toml/code-tree manifest as a masterplan companion registry (closes O-2); archive the orphan husks (`crates/`, `services/`, `test-results/`, `memory/`); add TTL to bulk markdown corpora; burn down the 289-row legacy-data_class allowance with a TTL.
13. **Net-new ADRs (>0514 free block):** the 4 keystone gates (ADR-0520-class under amended ADR-0365); the accounting-ledger schema; the service-tree-purity invariant; the generated layer-enum SSOT (I1a).

**DEPENDENCY SPINE:** FE-1 (live producer) → Gate-2 (registry) → Gates 1/3/4 → amendment campaign (each amendment verified by a gate) → doc-reorg + accounting closure. **Every "complete/eradicated/enforced" claim stays claim-ceiling-#21-blocked until its gate is green** — robust-not-false enforced on the campaign itself.

**FOUNDER SIGN-OFF / door:one-way points:** (i) FE-1 producer = the authority for all CI-enforcement claims; (ii) A-CI 0513 canon; (iii) A-STRUCT D-PURESPLIT completeness; (iv) A-TASTE framing; (v) source mutation = founder-go + WIP-commit-first.

---

## COVERAGE — fully audited vs sampled / deferred (no silent caps)

**FULLY AUDITED:**
- **Decisions:** 51 ADRs re-judged under the charter lens (CI 12 / structure 5 / enforcement 8 / identity-policy-data-event 9 / cross-ref-in-§0 17), each file+line-cited live.
- **Arch invariants:** all of ADR-0056/0105/0360/0366 + the live enforcers (`architecture_boundaries.rs`, `data_class_gates.rs`, `predictable-naming-kernel`, `shared-architecture-check-cli`, `layered-architecture-discipline`); 723 workspace crates, 903 catalog records, 97 lanes (by source+fixture+wiring inspection).
- **Total accounting:** all 27 `source/` top-level entries walked (`ls` + `git ls-files` + 4-registry reachability each); `docs/` (87) + `specs/` (106) cluster homes enumerated + dedup-checked.
- **Robustness:** `ci-lanes.md`, `lanes.yaml` (813 lines), both branch-protection files, the one GHA workflow (618 lines), Jenkinsfile + groovy + reported-status-contexts.json, the ADR-0221 ×4 bash gates, the protection-context-match/bypass/claim-ceiling kernels, the bypass record (expiry arithmetic).
- **Doc-reorg:** the scheme-defining files (`doc-style.md`, `DOCUMENTATION.md`, `DESIGN.md`, `planning-ssot-consolidation.md`, `catalog.json`, `contracts.json`, the 3 governance-lane specs).
- **Enforcement primitives:** the 4 live false-enforcement exhibits ground-truthed (catalog/contracts axes_count, governance-crate BUCK refs, diataxis/prd-axis lane status, ADR-0365 verified_by); the on-disk seed matrix + 4 fixtures.

**SAMPLED / DEFERRED (explicit):**
- ~294 of 345 source ids: verdict UNCHANGED, stand on the verified disposition table (body-level charter pass NOT done on them).
- Gate-roster MEMBERSHIP of the 59 oya-governance-* crates (confirmed they appear in BUCK files + all 91 active lanes have a check_command; defined-vs-rostered-in-the-required-set unverified).
- Branch-local supersede-edges for 0421/0457/0429/0443/0428/0488 (live on origin/dev, re-resolve at merge).
- LINUX L-0001..0026 deferred to the table (charter-consistent; residual = L-0001:36 half-applied Postgres scrub).
- NOT EXECUTED: any gate, `cargo metadata`, `buck2 cquery`, the live GitHub branch-protection API (enforcement reality inferred from source+fixtures+lane wiring + repo self-disclaimers, NOT a farm run); no_std judged by `#![no_std]` presence + manifest grep, not a no_std build; Buck2 visibility counted by grep over BUCK literals.
- Per-crate accounting NOT recursed (oya/14649 · cloud/1771 · libs/586 — accounted at tree level, the correct granularity); the 1413 user-journeys / 131 personas classified GENERATED by sampled estimate, not line-by-line.
- Foundry residue: honest canonical figure = 11 dirs + ~4110 file-level mentions (the 2180/3771/4714 raw counts are worktree-inflated or use looser exclusions; sibling census-of-record `20-verify-foundry-hygiene.md` = 4714/780 is deferred-to, not re-derived).
- Charter "22 governance crates" = a lower bound; live = 57–59, all unwired from root BUCK.

**Corpus status (verified):** 135 Accepted / 99 Proposed / 14 Superseded (of 348 .md / 345 distinct ids).

**Artifact:** `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md`
