# BIG HYGIENE PASS — whole-source-tree cleanup + fresh-authored canonical SSOT + tighten-the-knobs

**STATUS: `pending approval (door:one-way founder sign-off)`**

> **Mode:** `/ralplan` consensus, **DELIBERATE** (high-risk: a whole-monorepo aggressive deletion + a fresh-authored SSOT, on top of live CI/governance enforcement that is still a façade).
> **Role of this file:** Planner output for Architect + Critic review. **This is a PLAN.** It does NOT execute, mutate source, commit, or push. Every source mutation it describes is a `door:one-way` step parked for explicit founder sign-off. The recovery checkpoint already exists (below) — this plan does NOT re-create it.
> **Date:** 2026-06-07.
> **Target repo (SOURCE-FORCED):** `/Users/jasonlee/Developer/source` — the oyatie monorepo (`jason931225/oyatie`). **The session CWD is `/Users/jasonlee/Developer/linux`, the linux *port* (a DIFFERENT repo). This plan and every agent it spawns must `cd /Users/jasonlee/Developer/source` and self-check `pwd` BEFORE any read/grep/delete — see §0.3 SOURCE-FORCED protocol. The CWD-contamination bug (agents auditing the wrong repo) is the #1 catastrophic risk and is fenced below.**

---

## ⚠️ GROUND TRUTH (verified 2026-06-07 — load-bearing, read before the plan body)

| Fact | Verified value | How |
|---|---|---|
| **Target repo** | `/Users/jasonlee/Developer/source` (top-level: `oya/ cloud/ docs/ registry/ libs/ tools/ infra/ bin/ benchmarks/ contracts/ evidence/ memory/ templates/ tasks/ specs/ scripts/ packs/ regional-packs/ plan/ platforms/ tests/ third-party/ toolchains/`) | `ls -d */` |
| **Session CWD (the WRONG repo)** | `/Users/jasonlee/Developer/linux` — the linux **port** (markers: `stack/ kernel/legacy-kernel/ legacy-port/ toolchains/`). An agent that sees `stack/`+`legacy-kernel/` is in the PORT and **must ABORT.** | `pwd` |
| **Checkpoint commit (recovery anchor)** | `e38624dc4` — *"checkpoint: full source tree pre-aggressive-cleanup — recovery anchor for the D-SSOT-CURRENT-TRUTH whole-tree cleanup"* | `git log -1 e38624dc4` |
| **Branch** | `cleanup/whole-tree-2026-06-07` (checked out, **clean tree — 0 dirty**, the recoverability precondition is SATISFIED) | `git status --porcelain` = ∅ |
| **Checkpoint pushed** | `remotes/github-mirror/cleanup/whole-tree-2026-06-07` exists | `git branch -a` |
| **Remotes** | `github-mirror = github.com/jason931225/oyatie` (push HERE) · `origin = forgejo.local` (**NEVER push** — forgejo is DROPPED, D-FORGE-CLARIFY) | `git remote -v` |
| **Total tracked files** | **24,332** | `git ls-files \| wc -l` |
| **Top-level tracked counts** | `oya 14650` · `cloud 1808` · `docs 2886` · `evidence 1533` · `registry 1057` · `libs 586` · `specs 234` · `tools 269` · `tasks 120` · `infra 92` · `contracts 87` · `third-party 68` · `packs 60` · `scripts 44` · `templates 30` · `benchmarks 8` · `tests 8` · `regional-packs 6` · `plan 4` · `memory 2` · `bin 1` · `platforms 1` | `git ls-files <dir> \| wc -l` |
| **`bin/oya` (retired CLI) is LIVE in CI** | `.github/workflows/backbone-microservices-ci.yml:313` → `run: ./bin/oya gate validate cargo-prefix --workspace Cargo.toml --prefix oya-` | `grep -n` |

**Implication of the counts:** the *deletable* doc/registry/evidence/spec mass (`docs 2886 + registry 1057 + evidence 1533 + specs 234 + tasks 120 + templates 30 + memory 2 ≈ 5,862 files`, majority stale per the doctrine) is **distinct from** the live code mass (`oya 14650 + cloud 1808 + libs 586` = product/platform Rust, mostly KEEP). Workstream A's blast radius is concentrated in the doc/registry/evidence/spec layer + dead trees (`services/ crates/` are empty=0; `platforms/ plan/ tasks/` mostly stale; `bin/oya` retired). The keep-list is determined against the build-graph + CI + gate, NOT against the doc tree.

---

## ⚠️ CORRECTED GROUND TRUTH — `oya/*.md` doc-mass + enforcement state (orchestrator re-counted directly, ITERATION-2; supersedes BOTH reviewers' numbers)

> **Why this section exists:** the Architect's "**6,550 runbooks**" framing and the Critic's "**~5,377 PROPOSED**" extrapolation were **BOTH WRONG** (the Architect mislabeled the whole `oya/*.md` mass as runbooks; the Critic extrapolated PROPOSED from a sample instead of grepping frontmatter). The orchestrator re-counted directly. **These verified numbers are load-bearing and replace the wrong ones everywhere in this plan.** Both over-claims are explicitly retracted (see §A.0 / §A.2 / Pre-mortem S2).

| Verified fact (`/Users/jasonlee/Developer/source`, re-counted ITERATION-2) | Value | What it means for the plan |
|---|---|---|
| **`oya/*.md` total** | **6,550** | the doc-mass under adjudication. NOT all runbooks. NOT all PROPOSED. |
| **— RUNBOOKS (path `oya/**/runbooks/`)** | **974** | **KEEP-in-place** (operational truth; §A.0 row). The "6,550 runbooks" framing was a **~6.7× over-claim** — retracted. |
| **— IPs (path `oya/**/ip/` or `implementation-plans/`)** | **IP/spec family ≈ 2,422 by `doc_class` frontmatter (189 by dir-path; 2,950 by `IP-` filename-prefix)** | adjudicated by frontmatter SUBTYPE, NOT bulk-deleted (see below). |
| **— impl-plans (path `implementation-plans/`)** | **128** | subset of the IP family; same frontmatter adjudication. |
| **— files w/ EXPLICIT PROPOSED-scaffold frontmatter** | **≈408 (union of the 4 explicit frontmatter signals; 692 any-token)** | `rust_code_status: not-authored-in-this-wave` ∪ `documentation-and-contracts-only` ∪ `status: PROPOSED` ∪ `lifecycle_rule: PROPOSED`. **This is the explicit-DESTROY floor — NOT the Critic's extrapolated 5,377** (retracted as an over-claim of the *explicit* signal). |
| **Forbidden-vocab contamination — retired-CLI** | **1,266** `oya/*.md` cite `bin/oya` / `oya verify` / `oya gate` | every such assertion is RETIRED-not-truth → §B2 HARD pre-filter (N1) drops/re-authors it; never copied into the fresh SSOT. |
| **Forbidden-vocab contamination — dropped-infra** | **1,820** `oya/*.md` cite `jenkins` / `forgejo` / `foundry` | same N1 pre-filter (Foundry-the-external-product carve-out still applies; `foundry`-the-internal-tool is forbidden). |
| **`libs/oya-check-doc-axis` scan roots** | `docs/decisions`, `docs/ideas`, `docs/`, `microservices/` — **NEVER `oya/`** | the gate that should police the 6,550-file mass **does not scan it at all** → §C1(a) must extend the scan root. |
| **`oya-check-doc-axis` CI wiring** | wired into **0** `.github/workflows/` | it is an **UNWIRED façade** today → §C1(d) must WIRE it into the `oya-ci-required` fan-in. |
| **`oya-check-doc-axis` strictness** | warning-not-error / non-strict; allow-lists `LEGACY_DOCS_ROOT_FILES`(40) + `LEGACY_DOCS_SUBDIRS`(34) are **appendable arrays** | → §C1(b) flip `strict=true`/blocking; §C1(c) replace appendable arrays with the CLOSED D-SSOT allow-list enum. |
| **`.claude/worktrees/`** | **2,080,603 UNTRACKED `.md`** (only 9 `.claude` files tracked) | a tree-wide `grep` would **drown in 2M phantom hits** → §A.2 ref-scrub MUST exclude it (N2); AND it is local-disk sprawl → distinct `rm -rf` local-hygiene step (NOT git). |
| **Firewall NOT live** (per `PHASE-0-FIREWALL-PLAN.md:44-48`) | live-`dev` required = `["github-lane-unlocker-required"]`; `oya-ci-required` **required-by-nobody**; the 4 keystone gate crates are **born-blocking-SHADOW** (run locally, NOT CI-blocking) | every "GATE smoke / firewall green" in this plan means **locally-run shadow gate**, NOT a live blocking context → §B2 test-plan relabel (MUST-FIX 4). |

> **COUNTS CAVEAT (load-bearing):** These counts are INDICATIVE and vary by grep-definition (dir-path vs filename-prefix vs doc_class vs frontmatter-signal-union); the per-file frontmatter read AT EXECUTION is authoritative — the METHOD, not the number, is load-bearing.

**Net effect on the plan:** the adjudication is **per-file frontmatter SUBTYPE**, not directory and not extrapolation (§A.0/§A.2 rewritten per MUST-FIX 2). The enforcement workstream (C) is an **UNWIRED, non-blocking, wrong-scoped façade** that needs four concrete fixes (§C rewritten per MUST-FIX 3). The contamination (1,266 + 1,820) makes the §B2 truth-extraction pass a **brand-residue hazard** requiring a HARD pre-filter (§B2 + Pre-mortem S4 per MUST-FIX 1).

---

## RALPLAN-DR SUMMARY

### Principles (the design invariants — every stage is checked against these)

1. **KEEP-POSTURE, default-DESTROY (founder, D-SSOT-CURRENT-TRUTH CLEANUP EXECUTION POSTURE).** The correct posture is *enumerate what to KEEP* (clear, stated reason) and **DESTROY everything else**; ambiguous → DESTROY. The asymmetry the founder ruled is load-bearing: a wrong DELETE is recoverable from `e38624dc4`; *leaving sprawl is not*. There is **no "review" pile.** This is a ONE-TIME sweep posture, explicitly NOT a standing rule.
2. **SSOT holds ONLY current truth (D-SSOT-CURRENT-TRUTH).** No superseded entities, no history, no tombstones, no live `_archive/`. Git history is the *sole* archive (the "why" is recovered by git-archaeology). No-dangling is satisfied by **FULL EXCISION + ref-scrub**, never by tombstone-entities. The lineage philosophy is **git-as-history, not ADRs-as-history.**
3. **Draft-fresh from the LIVE state; do NOT migrate the mess (founder, Workstream-B charter).** Current truth comes from the **live code / build-graph / gate-config / running contracts** — NOT from the sprawled docs (which were stale narrations). The fresh JSON SSOT is *authored from the real current state*, then a one-pass truth-extraction salvages anything that exists ONLY in a doc; the old docs are then DESTROYED.
4. **SOURCE-FORCED + verifier-separated (the procedural fix for the CWD bug + verify-each-step).** Every agent `cd`s to `/Users/jasonlee/Developer/source` and `pwd`-self-checks (ABORT if it sees the port). Every delete batch + every authored store is verified in a **separate verifier lane** (no self-approval, no phantom findings) against real files/evidence. Nothing is claimed done on an unverified verdict.
5. **Maintainable BY ENFORCEMENT; recoverable; door:one-way-gated (D-DOCTRINE).** Sprawl is a *process failure*; the fix is the enforcement (Workstream C) that makes recurrence structurally impossible, not hand-discipline. Every mutation: `git rm` only (NEVER `filter-branch`/BFG — breaks signed-commit integrity + is unrecoverable), SIGNED commits, **github-mirror only**, NO blind `git add -A` (the checkpoint that needed it is already landed and exempt), per-batch kill-list shown for door:one-way founder review before commit.

### Decision Drivers (top 3 — what actually forces the design)

1. **Sequencing A-vs-B is the dominant correctness risk: deleting a doc that holds current-truth NOT derivable from code is unrecoverable as *canon* (only as a git blob).** If A (delete) runs before B (draft-fresh) captures that truth, the fresh JSON ships *incomplete* and the truth is demoted to git-archaeology — exactly the silent-loss the founder's "verify nothing dropped is worth preserving" rule (D-AUTHORITY-CONVERSATION) forbids. This forces **B-captures-truth-FIRST** (Option-1 below). The checkpoint makes the *bytes* recoverable, but not the *canon-membership* — so we do not rely on it as the primary defense for truth-loss.
2. **The CWD-contamination bug already caused agents to audit the WRONG repo.** The session runs in the linux port; the target is `source`. A delete agent that fails to `cd`+`pwd`-check could `git rm` the *port*. This forces a HARD **SOURCE-FORCED protocol with an abort-on-port-marker self-check** as a precondition gate on every A-lane and B-lane task — a structural fence, not a reminder.
3. **The enforcement that would PREVENT regression is itself still a façade (D-SEQUENCE / CICD-DESIGN-PLAN ground truth).** The cross-artifact / total-accounting / staleness / no-dangling gates exist as crates but are wired into ZERO blocking pipelines (the live required context is a structural false-green). So the cleanup's no-dangling safety net must be a **manual `grep`-verify per batch** (substituting for the not-yet-live cross-artifact gate), and Workstream C lands the firewall to *enforce* stays-current-truth-only AFTER the cleanup establishes it (founder: cleanup-now establishes; firewall-later prevents regression).

### Viable Options for the A/B/C sequencing (≥2, bounded pros/cons)

**Option 1 (RECOMMENDED) — B-captures-truth-FIRST, then A-deletes, then C-enforces (truth-extraction barrier between B and A).**
Draft the fresh JSON SSOT from the live state, AND run a one-pass **truth-extraction** over the to-be-deleted doc set (a read-only diff: "what current-truth does this doc assert that is NOT already in the fresh store or derivable from code?") → fold survivors into the fresh store → **gate A on a green truth-extraction sign-off** → then A deletes per-domain → then C lands the firewall.
- *Pros:* directly mitigates Driver-1 (the unrecoverable-canon risk) — nothing is deleted until its current-truth is provably captured. Matches the founder's "draft a new json … is better than moving the mess" + "verify nothing dropped is worth preserving." The truth-extraction pass is the explicit, auditable barrier the pre-mortem S2 needs.
- *Cons:* truth-extraction over the doc-mass is the long-pole effort — the **6,550 `oya/*.md`** (974 RUNBOOKS-KEEP / ≥699 PROPOSED-DESTROY / the IP-spec family split per-file by frontmatter) **plus** the doc/registry/evidence/spec layer (`docs 2886 + registry 1057 + evidence 1533 + …`, majority-stale). Must be SOURCE-FORCED + frontmatter-subtype-driven (exhaustive on high-signal types: RUNBOOKS/decision-rationale/sequencing; sampled-then-default-DESTROY on bulk-stale evidence/indexes). Risks *over-capturing* stale narration as if current (mitigated: extraction asserts current-truth ONLY, validated against live code; a doc claim that contradicts live code is stale, not truth) **and brand-laundering** (mitigated: the N1 forbidden-vocab pre-filter — Pre-mortem S4). B and A cannot fully parallelize per-domain until the store schema is frozen.

**Option 2 — A-deletes-first, rely on `e38624dc4` to recover any truth found missing during B.**
Delete aggressively now (default-DESTROY), draft the fresh JSON after, and if B discovers a needed truth, recover the specific blob from the checkpoint.
- *Pros:* fastest to the clean current-only end-state; maximal alignment with the raw "bias HARD to DELETE" posture; smallest coordination surface (A is a self-contained sweep). The checkpoint genuinely makes bytes recoverable.
- *Cons:* **fails Driver-1** — recovery is *reactive* and depends on B *noticing* the gap; an un-noticed current-truth silently becomes git-archaeology-only and is dropped from canon (violates "verify nothing dropped is worth preserving"). Re-introducing a recovered blob after a signed-delete batch muddies the clean-current-only invariant. **Not recommended** as the primary path; retained as the *fallback recovery mechanism* inside Option 1 (if truth-extraction misses something, `e38624dc4` is the net).

**Option 3 — Interleave per-domain (for each domain: draft-fresh that domain's store slice → extract-truth → delete that domain → next domain).**
- *Pros:* bounds blast radius to one domain at a time; each domain is a complete door:one-way unit (cleanest founder-review granularity); a mistake in one domain doesn't block others.
- *Cons:* the store SCHEMA + the 4 guards (Workstream B) must be frozen FIRST regardless (cross-domain keys/merge-driver can't be designed per-domain) — so it's really "Option 1 with A interleaved per-domain after the schema freeze," not a distinct sequencing. **Adopted as the *intra-A execution shape* under Option 1** (see §A.0): schema-freeze once (B1), then per-domain extract→delete waves.

**Chosen: Option 1, with Option 3 as the intra-execution shape and Option 2's checkpoint-recovery as the fallback net.** Rationale: Driver-1 (unrecoverable-canon) is the dominant risk and only Option 1 structurally prevents it; the per-domain interleave (Option 3) gives clean door:one-way granularity once the schema is frozen; the checkpoint (Option 2) is the safety net, not the plan.

---

## DELIBERATE-MODE ADDITIONS

### Pre-mortem (≥3 failure scenarios + mitigation — now 4: S4 added ITERATION-2 for brand-residue contamination, N1)

**Scenario 1 — "We deleted a live dependency → broke the build/gate."** An A-lane `git rm`s a file that is referenced by the buck2 build-graph, a CI workflow, the 96-lane governance gate, a producer's required-set, or a running service — and the build/gate goes RED (or worse, a *gate-required baseline* like the 8 benchmarks vanishes and GATE-2 total-accounting fails), or a `contracts/` consumer loses its codegen input.
- *Mitigation:* (a) The KEEP-LIST is determined **FROM the build-graph + CI + gate + accounting-registry required-set** + the per-file frontmatter subtype for the `oya/*.md` mass (§A.0.1), not by eyeballing the doc tree (§A.1 keep-list derivation is producer-driven + frontmatter-driven, SOURCE-FORCED). (b) Every delete batch runs **SCOPED ref-scrub** (`grep -rn … --exclude-dir={.claude/worktrees,target,.git,buck-out}` — verify ZERO references to any deleted path/id remain; the manual no-dangling net; N2 excludes mandatory else 2M phantom hits) + **producer-regen** (the accounting-registry / masterplan / index producers re-emit so the required-set stays consistent) + **build / LOCAL shadow-gate smoke** (`buck2 build` affected closure + the **locally-run born-blocking-shadow gate crate** — NOT the live `oya-ci-required` context, which is not live) in the verifier lane BEFORE the batch commits. (c) The CONSOLIDATE-with-dependency items are explicit + FRONT-LOADED: `benchmarks 8→1` updates the accounting-registry required-set (regen + baseline update or the local GATE-2 accounting net fails); `contracts 87→1` re-points codegen/consumers FIRST, then deletes the scattered files. (d) `bin/oya` delete is sequenced LAST, AFTER re-homing the cargo-prefix gate, then paired with scrubbing its CI caller (`backbone-microservices-ci.yml:313`, the only live caller) in the SAME batch — no dangling caller, no broken-CI window.

**Scenario 2 — "The fresh JSON lost current-truth that existed ONLY in a deleted doc."** B authored the fresh store from live code, A deleted the docs, and three weeks later it's discovered that (e.g.) a non-code-derivable operational fact — a runbook step, a deliberate guard rationale, a sequencing decision — lived only in a doc that's now a git blob, never folded into the fresh store. It is silently dropped from canon.
- *Mitigation:* (a) The **truth-extraction barrier** (Option 1) is mandatory and GATES A: no domain's docs are deleted until a verifier confirms the truth-extraction pass over that domain produced either "already in the fresh store" or "folded into the fresh store" for every current-truth assertion — a doc with un-extracted current-truth BLOCKS its own deletion. (b) The extraction asserts **current-truth ONLY**, validated against live code (a doc claim contradicting live code is *stale*, correctly dropped). (c) Fallback net: `e38624dc4` recovers the specific blob if a miss is found post-delete (Option 2 as net). (d) D-AUTHORITY-CONVERSATION's "verify nothing dropped is worth preserving" is the explicit verifier acceptance criterion for the barrier.

**Scenario 3 — "CWD-contamination recurs / wrong-repo deletion."** A spawned delete agent, running in the session whose CWD is the linux *port*, fails to `cd`/`pwd`-check (or a relative path resolves against the port) and `git rm`s files in `/Users/jasonlee/Developer/linux` — destroying the kernel port instead of the source sprawl.
- *Mitigation:* (a) **SOURCE-FORCED protocol (§0.3) as a PRECONDITION GATE on every A/B task:** the task's first action is `cd /Users/jasonlee/Developer/source && pwd` and an **abort-on-port-marker self-check** — if `pwd` is not exactly `/Users/jasonlee/Developer/source` OR the tree contains the port markers (`stack/`, `legacy-kernel/`, `legacy-port/`), the agent ABORTS and reports, mutating nothing. (b) **Absolute paths only** in every `git rm` / `grep` / producer invocation — no relative paths that could resolve against the port. (c) The verifier lane independently re-confirms the operating repo is `source` (checks `git remote get-url github-mirror` == `…/oyatie` AND the checkpoint `e38624dc4` is reachable) before accepting any batch. (d) Per-batch kill-list (read-only manifest of exact absolute paths) is shown for door:one-way founder review — a path under `/Users/jasonlee/Developer/linux` in a kill-list is an instant ABORT signal a human catches.

**Scenario 4 — "Truth-extraction launders RETIRED brand-residue into the fresh SSOT → the brand-residue gate goes RED on the freshly-authored stores."** `1,266` `oya/*.md` cite `bin/oya`/`oya verify`/`oya gate` and `1,820` cite `jenkins`/`forgejo`/`foundry` (CORRECTED GROUND TRUTH). If the §B2 truth-extraction copies an assertion like "run `oya gate validate …` before merge" or "the forgejo mirror holds X" verbatim into the fresh store *as if it were current-truth*, the fresh SSOT ships with **forbidden vocabulary baked in** — and the moment the brand-residue / forbidden-vocab gate is wired (Workstream C), it goes **RED on the freshly-authored canon**, i.e. the cleanup re-creates the exact contamination it was meant to eradicate (D-CLOUD-NATIVE violation, self-inflicted).
- *Mitigation:* (a) **HARD forbidden-vocab/retired-CLI PRE-FILTER on the §B2 extraction pass (MUST-FIX N1):** before any assertion is folded into the fresh store, it is scanned for `bin/oya`/`oya verify`/`oya gate`/`jenkins`/`forgejo`/`foundry` (Foundry-the-external-product carve-out excepted). Any hit means the assertion is **RETIRED-not-truth** → it is either (i) **re-authored against the live pipeline** (e.g. "`oya gate validate cargo-prefix`" → the equivalent GitHub-Actions/cloud-native gate lane, validated against §0.4 sources 1–5) or (ii) **DROPPED** — it is **NEVER copied verbatim** into the fresh SSOT. (b) The pre-filter is a precondition of the B2 barrier-green (a domain whose extraction still contains forbidden vocab is BLOCKED). (c) The verifier lane independently greps the freshly-authored stores for the forbidden set → ANY hit FAILS the B2 verifier acceptance (expanded test plan, B row). (d) Because the *source* docs are heavily contaminated (1,266 + 1,820), the extraction is treated as **brand-residue-hostile by default**: the burden is on the extractor to prove an assertion is live-pipeline-current, not on the filter to prove it is retired.

### Expanded test plan (how each workstream is RED/GREEN / build-smoke-verified — no false-green, no phantom findings)

| Workstream | What is proven | RED proof | GREEN proof | Verifier-lane check (separate, no self-approval) |
|---|---|---|---|---|
| **A — Cleanup** | Every delete batch leaves the build-graph + CI + gate consistent; ZERO dangling refs; KEEP-list never violated. | After a batch, **scoped** `grep -rn <deleted-id/path> /Users/jasonlee/Developer/source` (with `--exclude-dir={.claude/worktrees,target,.git,buck-out}` — N2) returns a hit → batch FAILS (dangling), revert. A `buck2 build <affected-closure>` or the **locally-run shadow gate crate** goes RED → batch FAILS. | Post-batch: ZERO scoped-grep hits to any deleted path/id; producer-regen idempotent; affected `buck2 build` + the **locally-run shadow gate crate (born-blocking-shadow — NOT the live `oya-ci-required` blocking context, which is not yet live)** GREEN; the 8→1 benchmark / 87→1 contracts consolidations leave the **GATE-2 accounting baseline (locally-run; the only accounting net since GATE-2 is not CI-live)** GREEN. | Independent agent re-runs the **scoped** grep ref-scrub (same excludes) + the local build/shadow-gate smoke on the REAL tree (not the executor's report); diffs the kill-list against `git ls-files` to confirm only-intended-paths-gone + no KEEP-list path deleted. |
| **B — Fresh SSOT** | The fresh JSON store(s) parse, are keyed/enum-accessed, round-trip through the canonical formatter, CONTAIN every current-truth from the truth-extraction, and are **FREE of forbidden vocab** (N1). The 4 guards BLOCK (not just parse). | Hand a malformed/duplicate-key store to the **keyed accessor** → it errors (not silently returns last). Hand an un-formatted store to the **canonical-formatter gate** → RED. A truth-extraction item with no home in the store → the barrier gate RED (blocks A). An extracted assertion citing `bin/oya`/`oya verify`/`oya gate`/`jenkins`/`forgejo`/`foundry` that reaches the store → the **forbidden-vocab pre-filter RED (blocks B2-green)**. The **entity-incremental gate** on a PR that adds an off-schema entity → RED. | A round-tripped store is byte-identical through the formatter (idempotent); the keyed accessor resolves every enum key; the truth-extraction barrier is GREEN for a domain (every current-truth has a home); **`grep` for the forbidden set over the freshly-authored stores returns ZERO hits**; the **key-aware merge driver** resolves a synthetic concurrent-key-add without clobber. | Independent agent re-derives a sample of "current truth" from live code/gate-config and confirms it is present in the store (catches B authoring from stale docs instead of live state); confirms no extracted item is stale-misclassified-as-truth (contradicts live code); **independently greps the freshly-authored stores for `bin/oya`/`oya verify`/`oya gate`/`jenkins`/`forgejo`/`foundry` (ex-Foundry-product) → ANY hit FAILS B (N1 brand-residue net).** |
| **C — Tighten knobs** | `oya-check-doc-axis`, after the 4 concrete fixes, STRUCTURALLY blocks ad-hoc doc `.md`/`.json` outside the stores/allow-list **and actually scans `oya/`** (it does not today) **and is actually wired into CI** (0 workflows today); the allow-list is the closed enum (not the appendable `LEGACY_DOCS_*` arrays); accounting-registry total-accounting blocks unaccounted files. | **Scan-root fix:** an off-allow-list `.md` placed *inside `oya/`* on a test PR → the gate goes RED (proves the scan root now covers the 6,550-mass; pre-fix it would be IGNORED). **Strict fix:** the same fixture produces an `error` exit, not a `warning` (pre-fix: non-strict warn). **Wiring fix:** the gate appears as a required check on the test PR's `oya-ci-required` fan-in (pre-fix: absent from all workflows → never runs). **Enum fix:** appending a row to `LEGACY_DOCS_ROOT_FILES`/`LEGACY_DOCS_SUBDIRS` no longer silently widens the allow-list (the closed enum rejects it). Drop an allow-listed store row's producer → total-accounting RED. | A PR that only touches allow-listed stores/generated-views is GREEN; the gate's allow-list is the closed enum from D-SSOT-CURRENT-TRUTH (NOT the appendable arrays); the gate is a constituent required lane of `oya-ci-required` and the fan-in is green IFF every constituent gate is green (maps onto CICD-DESIGN-PLAN Stage-1 surface-all). | Independent agent runs the RED fixtures itself on the real gate binary (no self-approval); confirms the scan root includes `oya/` (and the whole tree, ex-`.claude/worktrees`/`target`/`buck-out`); confirms `strict=true`/blocking (no advisory-shell-claiming-enforced); confirms the gate is present in `.github/workflows/` as a required `oya-ci-required` constituent (not 0); diffs the live closed enum vs the D-SSOT-CURRENT-TRUTH allow-list and confirms the appendable arrays are gone. |

---

## §0 — DISCIPLINE & PROTOCOL (the procedural fix — baked into every stage)

The founder root-caused the sprawl as a **FAILURE IN PROCEDURE** (reactive ad-hoc per-dir workflow-spawning + the CWD-contamination bug). These protocols are the methodical replacement and apply to **every task in every workstream.**

### §0.1 — Discipline invariants (non-negotiable, on every mutation)
- **Separate verifier lane.** Authoring/deleting and verification are different agents in different passes. No self-approval. No phantom findings — every verifier claim cites a real file/line/command-output. A failed verdict means iterate, never claim-done.
- **Recoverable.** All recovery is from the checkpoint `e38624dc4` (committed, pushed github-mirror). `git rm` ONLY. **NEVER** `filter-branch`/BFG/history-purge (breaks signed-commit integrity + is unrecoverable).
- **Door:one-way founder sign-off** at every marked gate (`🚪`) and before every source-mutation batch commit. A read-only **kill-list manifest** (KEEP vs DESTROY + the reference-graph) precedes any deletion and is shown for review.
- **SIGNED commits, github-mirror ONLY.** `origin` = forgejo.local — **NEVER push there** (forgejo DROPPED, D-FORGE-CLARIFY). Commit signing: SSH `id_ed25519` (founder-provisioned, registered as a signing key). **BLOCKING PRE-FLIGHT (must pass before the first deletion batch starts):** verify signing is live by making a throwaway commit with `git commit -S --allow-empty -m "signing-preflight-check"` on a throwaway branch, then confirming `git cat-file -p HEAD` contains a `gpgsig` field — if the field is absent or `git commit -S` errors, the cleanup MUST NOT start any mutation batch until signing is confirmed live.
- **NO blind `git add -A`.** Stage explicit absolute paths per batch. The one `git add -A`-class operation that was needed — the checkpoint of the whole dirty tree — is **already landed (`e38624dc4`) and is the sole exemption.**
- **Forbidden vocab.** `forgejo`, `foundry`, `jenkins`, `oya-vcs`/agentic-VCS are FORBIDDEN (D-CLOUD-NATIVE) — eradicate where touched; never re-introduce. **Jenkins = the dropped CI bridge** (GitHub Actions is the live authority); the 89 Jenkinsfiles stay an explicitly-UNRATIFIED bridge until oya-ci is proven — quarantine-not-delete in *this* pass (CICD-DESIGN-PLAN Stage 1E). Carve-out: external **"Palantir Foundry"** (kept). **oya-CLI stays RETIRED** — do NOT record any revival anywhere.

### §0.2 — Item tagging (every actionable item is tagged)
- `[automatable-now]` — an agent can do it under the SOURCE-FORCED + verifier protocol today (most A/B authoring + grep/build smoke).
- `[with-infra]` — needs an infra piece first (e.g. the firewall gate binary, the producer-regen, the keyed accessor) before it can be done/enforced.
- `[inherently-manual]` — needs a human: founder door:one-way sign-off, the truth-extraction "is this worth preserving?" judgment calls flagged for founder, commit-signing-key confirmation.

### §0.3 — SOURCE-FORCED protocol (the CWD-bug structural fence — PRECONDITION on every A/B task) `[automatable-now]`
Every spawned agent's **first action**, before any read/grep/delete/author:
```
cd /Users/jasonlee/Developer/source
test "$(pwd)" = "/Users/jasonlee/Developer/source" || { echo "ABORT: not in source"; exit 1; }
# abort if the tree shows linux-PORT markers (we'd be in the wrong repo):
for marker in stack legacy-kernel legacy-port; do
  test -e "$marker" && { echo "ABORT: linux-port marker '$marker' present — wrong repo"; exit 1; }
done
# confirm the source identity + recovery anchor reachable:
git remote get-url github-mirror | grep -q 'oyatie' || { echo "ABORT: not the oyatie mirror"; exit 1; }
git cat-file -t e38624dc4 >/dev/null 2>&1 || { echo "ABORT: checkpoint e38624dc4 unreachable — recovery net absent"; exit 1; }
```
- **Absolute paths only** in every subsequent `git rm`/`grep`/producer call. No relative path may resolve against the port.
- The verifier lane independently re-runs this self-check before accepting any batch (Pre-mortem S3c).
- A kill-list entry under `/Users/jasonlee/Developer/linux` is an instant ABORT (Pre-mortem S3d).

### §0.4 — Authoritative truth-sources for Workstream B (where "current truth" comes from — Driver-2 / Principle-3)
Current truth is **NOT the docs** (stale narrations). The authoritative sources, in priority order:
1. **The live build-graph** — `Cargo.toml` workspace members, `BUCK`/`BUILD` targets, the dependency closure (what actually compiles/links = the real component set).
2. **The CI config** — `.github/workflows/*` (what actually runs/gates), the required-context wiring.
3. **The 96-lane governance gate + accounting-registry** — `registry/catalog/*`, the gate roster, the required-set + baselines (what is actually enforced).
4. **Running contracts** — `contracts/` (live API/proto contracts consumed by codegen) — the real interface surface.
5. **Running services** — the live `oya/`+`cloud/` service trees + their manifests (what actually exists).
6. **Truth-extraction residue** — current-truth assertions found ONLY in a to-be-deleted doc that are NOT derivable from 1–5 (the §B truth-extraction pass; folded into the fresh store before the doc is deleted).
Docs are consulted ONLY as source #6 (extraction), never as the primary authoring source. A doc claim that contradicts sources 1–5 is **stale**, not truth.

---

## §A — WORKSTREAM A: AGGRESSIVE CLEANUP, KEEP-POSTURE `[gated on B1 schema-freeze + per-domain truth-extraction]`

**Goal:** the source tree contains ONLY current truth — the fixed carve-out markdowns (README/CLAUDE/AGENTS/SKILL/LICENSE/`.github` specials) + the **974 KEEP-in-place RUNBOOKS** + ACCEPTED/implemented IPs/specs + the domain stores + generated views + live code/configs/build-graph + gate-required baselines. **ZERO** PROPOSED-scaffolds-for-unwritten-code + stale narration. Default for **PROPOSED-vapor and stale narration** = DESTROY; **RUNBOOKS and ACCEPTED/implemented IPs/specs default KEEP** (DESTROY only on a *positive* staleness signal — names a service/contract/gate that no longer exists); recoverable from `e38624dc4`.

> **Scale note (CORRECTED GROUND TRUTH):** the doc-mass under adjudication is **6,550 `oya/*.md`** (NOT "a SELECT few" — the old "SELECT few carve-out" framing referred only to the *fixed carve-out enum* and is corrected here). Of these, **974 are RUNBOOKS (KEEP-in-place)**, **≥408 carry EXPLICIT PROPOSED-scaffold frontmatter (DESTROY floor — union of 4 explicit frontmatter signals; 692 any-token)**, and the IP/spec family (**IP/spec family ≈ 2,422 by `doc_class` frontmatter (189 by dir-path; 2,950 by `IP-` filename-prefix) + 128 impl-plans**) is adjudicated **per-file by frontmatter SUBTYPE**, NOT bulk-deleted and NOT extrapolated. The whole repo's real `.md` count is **10,672** — Workstream A operates on that real denominator, not a guess.

**Dependency:** `→ B1` (store schema frozen) `→` per-domain `B-extract` (truth captured) before that domain's delete. **Blocks:** C (the firewall enforces the clean state A establishes). **Sequencing:** Option-1 (B-first) with Option-3 per-domain interleave.

### §A.0 — The KEEP-LIST (the enumerated "clear-reason-to-keep" set — everything else is DESTROY)
A file/dir is KEPT **only** if it matches one of these (the closed allow-list from D-SSOT-CURRENT-TRUTH, source-forced-derived):

| KEEP reason | What it covers | How it's determined (producer-driven, not eyeballed) |
|---|---|---|
| **Live code referenced by the build-graph** | `oya/` + `cloud/` Rust crates/services in the `Cargo.toml`/`BUCK` closure; `libs/` referenced by a member; `toolchains/`, `third-party/` (vendored, behind ports) | the workspace member set + `buck2 targets`/`cargo metadata` reachability closure |
| **Live CI / gate referents** | `.github/workflows/*` (live authority — D-CICD-AUTHORITY); files a workflow reads; the 96-lane gate roster + its inputs | grep the workflows + the gate roster for referenced paths |
| **Gate-required baselines in the accounting-registry** | `registry/catalog/*` rows in the required-set; the gate baselines; **`benchmarks` → CONSOLIDATE 8→1** (gate-baseline, KEEP-consolidated) | the accounting-registry-producer required-set output |
| **Running contracts** | **`contracts/` → CONSOLIDATE 87→1** keyed JSON (machine-consumed, KEEP-consolidated; re-point consumers FIRST) | the codegen/consumer reachability |
| **Carve-out markdown + tool-mandated** | `README.md` / `CLAUDE.md` / `AGENTS.md` + GitHub special files (`.github/*` specials) + `SKILL.md` + `LICENSE` | the fixed carve-out enum (D-SSOT-CURRENT-TRUTH) |
| **`oya/**/runbooks/*.md` — RUNBOOKS (974)** | operational truth (deploy/rollback/incident steps) not derivable from code | **KEEP-in-place** by `doc_class: RUNBOOK` (or path `oya/**/runbooks/`). DESTROY only on a *positive* staleness signal (names a service/contract/gate that no longer exists in the live build-graph). **NOTE: this retracts the Architect's "6,550 runbooks" over-claim — the runbook count is 974.** |
| **`oya/**/{ip,implementation-plans}/*.md` — ACCEPTED/implemented IPs & specs** | implementation plans / specs whose code IS authored & in the build-graph | **default KEEP** by frontmatter `status: ACCEPTED`/`implemented` AND `rust_code_status != not-authored-in-this-wave`. DESTROY only on positive staleness (the named crate/service is gone). IP/spec family ≈ 2,422 by `doc_class` frontmatter (189 by dir-path; 2,950 by `IP-` filename-prefix) + 128 impl-plans are split here per-file. |
| **The domain stores + generated views** | the fresh JSON SSOT store(s) authored in B + their generated human-readable views | Workstream B output (the replacement for the deleted sprawl) |
| **The active migration plan** | the in-flight monorepo-consolidation migration plan (live, execution authority — D-MERGE) | named explicitly; not auto-derived |

**The complementary DESTROY rows (positive-signal, frontmatter-driven — NOT directory, NOT extrapolation):**

| DESTROY reason | What it covers | How it's determined (per-file frontmatter SUBTYPE) |
|---|---|---|
| **PROPOSED-scaffold-for-unwritten-code** | docs that describe code that was never authored | `rust_code_status: not-authored-in-this-wave` ∪ `documentation-and-contracts-only` ∪ `status: PROPOSED` ∪ `lifecycle_rule: PROPOSED` — the **≥408 explicit floor (union of 4 explicit frontmatter signals; 692 any-token)** + any other PROPOSED-vapor surfaced by the frontmatter read. **🚪 founder must RATIFY "PROPOSED-IP-for-unwritten-code = DESTROY"** — this is the largest single judgment call in the pass (see §A.0.1). |
| **Stale narration** | prose that contradicts §0.4 sources 1–5, or narrates a superseded state | positive staleness signal vs the live build-graph/CI/gate/contracts — NOT "ambiguous". |
| **Forbidden-vocab-only docs** | docs whose entire content is retired-CLI / dropped-infra narration (`bin/oya`/`oya verify`/`oya gate`/`jenkins`/`forgejo`/`foundry`) with no salvageable current-truth | the §B2 N1 pre-filter flags these; if extraction yields nothing live-current, DESTROY. |

> **RETRACTION (ITERATION-2):** the Critic's "**~5,377 PROPOSED**" was an extrapolation from a sample, NOT a frontmatter count. The **verified explicit-PROPOSED floor is ≈408 (union of the 4 explicit frontmatter signals; 692 any-token)**. The adjudication is per-file frontmatter, so the true DESTROY count for the IP/spec family is *discovered by reading frontmatter*, bounded below by ≈408 — it is **not** asserted to be 5,377. Equally, the "6,550 runbooks" framing is retracted (runbooks = 974). The IP count of "2,950 (path-counted)" is relabeled: 2,950 is the `IP-` filename-prefix count; the `doc_class` frontmatter count is ≈ 2,422 and the `/ip/`+`/implementation-plans/` dir-path count is 189.

**Everything NOT matching a KEEP row and NOT matching a DESTROY row is CONSOLIDATE-into-a-store (if current-scattered) or DESTROY (default for genuinely-ambiguous non-code-derivable prose; ambiguous→DESTROY per the founder posture). RUNBOOKS and ACCEPTED IPs/specs are exempt from ambiguous→DESTROY — they need a *positive* staleness signal.**

### §A.0.1 — The frontmatter-subtype adjudication method (the keep/destroy determination, mechanized) `[automatable-now]` `→ 🚪`
The keep-list determination **reads each `oya/*.md`'s YAML frontmatter** (`doc_class` + `status` + `rust_code_status` + `lifecycle_rule`) — NOT the directory, NOT a sample extrapolation:
1. `doc_class: RUNBOOK` (or path `oya/**/runbooks/`) → **KEEP-in-place** unless positive-stale.
2. `status: ACCEPTED`/`implemented` AND `rust_code_status != not-authored-in-this-wave` → **KEEP** unless positive-stale.
3. `rust_code_status: not-authored-in-this-wave` ∪ `documentation-and-contracts-only` ∪ `status: PROPOSED` ∪ `lifecycle_rule: PROPOSED` → **DESTROY** (the PROPOSED-vapor class; **founder ratifies the class-rule at §A.0.1's 🚪**, then the per-file application is `[automatable-now]`).
4. No frontmatter / un-typed prose → **sampling rule:** draw a random sample of min(N, dir-file-count) files per directory where N = max(10, ceil(dir-file-count × 0.10)) (i.e. 10% of the dir, floored at 10 files). Run §B2 truth-extraction on the sample with the N1 forbidden-vocab pre-filter. If ZERO files in the sample contain live-current truth (all are stale narration or empty-after-filter), **default-DESTROY the entire directory** without per-file extraction — the sample constitutes ≥90% confidence that the dir yields no current truth. If ANY sampled file contains live-current truth, fall back to **per-file extraction for that directory**. This rule is concrete and automatable; the sample manifest and hit-count must be recorded in the per-domain barrier sign-off for founder review.
**`🚪` door:one-way:** founder ratifies the **class-rule "PROPOSED-IP-for-unwritten-code = DESTROY"** (the single largest judgment call) BEFORE the per-file pass runs. The output feeds the §A.1 kill-list manifest.

### §A.1 — Derive the KEEP-LIST + the kill-list manifest (read-only) `[automatable-now]` `→ 🚪`
SOURCE-FORCED. Run the producers (build-graph reachability, CI/gate referent grep, accounting-registry required-set) to compute the KEEP set; the complement (over `git ls-files`) is the **kill-list candidate**. Emit a read-only **kill-list manifest** = `{KEEP (reason) | DESTROY | CONSOLIDATE-into-<store> + the reference-graph}` per path. **`🚪` door:one-way:** founder reviews the kill-list manifest before ANY deletion. `[inherently-manual]` for the founder review + any ambiguous-but-flagged judgment call (default still DESTROY).

### §A.2 — Per-domain DESTROY waves (Option-3 interleave; each wave a door:one-way unit) `[automatable-now per batch]` `→ 🚪 per batch`
**Wave ordering (front-load the lowest-risk + the only-accounting-net consolidations FIRST):** the FIRST `[with-infra]` waves are **`benchmarks 8→1`** and **`contracts 87→1`** (GATE-2 accounting baseline is the only accounting net AND it is not CI-live — do these while the tree is still pristine so the baseline-delta is clean). Then `[automatable-now]` low-risk: `memory(2) → templates(30) → tasks(120) → specs(234) → plan/platforms → evidence(1533, mostly historical) → registry(1057, majority-stale) → docs(2886) → the oya/*.md frontmatter-subtype waves (974 RUNBOOKS KEEP / ≥408 PROPOSED-DESTROY (union of 4 explicit frontmatter signals; 692 any-token) / IP-spec per-file ≈ 2,422 by doc_class) → tools(269) → libs(586 verify-vs-graph; do-not-delete the gate crates + `oya-check-doc-axis`, see §A.3) → bin/oya LAST (after its cargo-prefix-gate re-home)`. **Dedup note: the kill-list manifest must deduplicate IP-vs-spec frontmatter membership so a file is not double-counted across the IP and spec waves.**
1. **Precondition:** that domain's `B-extract` truth-extraction is GREEN (every current-truth folded into the fresh store, **and the N1 forbidden-vocab pre-filter is clean for that domain**) — else the domain's delete is BLOCKED (Pre-mortem S2a + S4).
2. **Delete batch:** `git rm` the DESTROY set (absolute paths, explicit — NO `git add -A`).
3. **Ref-scrub (SCOPED — N2):** `grep -rn <deleted-id/path> /Users/jasonlee/Developer/source --exclude-dir={.claude/worktrees,target,.git,buck-out}` → ZERO hits (the manual no-dangling net) or revert. **The excludes are MANDATORY: `.claude/worktrees/` alone holds 2,080,603 untracked `.md` that would false-FAIL every batch (CORRECTED GROUND TRUTH).**
4. **Producer-regen:** re-emit accounting-registry / masterplan / index so the required-set stays consistent.
5. **Build / shadow-gate smoke:** affected `buck2 build <affected-closure>` + **the locally-run gate crate (born-blocking-shadow — NOT the live `oya-ci-required` blocking context, which is not yet live per the firewall ground truth)** GREEN.
6. **Verifier lane** (separate agent) independently re-runs 3+5 (same scoped excludes) on the real tree + diffs the kill-list vs `git ls-files`.
7. **`🚪` founder sign-off → SIGNED commit → push github-mirror.**

### §A.2.1 — Local-disk-sprawl hygiene (distinct from the git-tracked sweep) `[automatable-now]`
`.claude/worktrees/` holds **2,080,603 UNTRACKED `.md`** (only 9 `.claude` files are git-tracked) — regenerable agent scratch, NOT canon. Remove it as **local-disk hygiene** via `rm -rf /Users/jasonlee/Developer/source/.claude/worktrees/` (a filesystem op, **NOT a `git rm`** — it is untracked, so git never sees it). This is a *distinct one-line step* from the tracked-file DESTROY waves: it both reclaims disk and removes the phantom-hit source the §A.2 ref-scrub excludes. SOURCE-FORCED self-check applies (confirm `pwd` = source before the `rm -rf`).

### §A.3 — The named special cases (explicit, do-not-miss) `[automatable-now / with-infra]`
- **`bin/oya` (retired CLI, the ONLY live CI caller):** `backbone-microservices-ci.yml:313` (`./bin/oya gate validate cargo-prefix …`) is the **only live CI reference** to it. **ORDER: re-home the cargo-prefix gate onto the cloud-native pipeline FIRST** (NICE-TO-HAVE 9 — do not break the only live CI before the re-home target exists), THEN delete `bin/oya` **AND** scrub the `:313` caller **in the SAME batch** — no dangling CI caller, no window where the live CI is broken. Sequenced **LAST** in §A.2. `[with-infra]` (needs the gate re-home target). oya-CLI **stays retired** — no revival recorded.
- **`benchmarks 8→1` (gate-baselines, KEEP-consolidated):** consolidate to ONE md-or-json; update the accounting-registry required-set (producer-regen + baseline update) **or the GATE-2 accounting baseline (locally-run; the only accounting net, not CI-live) fails.** **FRONT-LOADED as one of the FIRST `[with-infra]` waves** (MUST-FIX 4) — gated on CICD-DESIGN-PLAN Stage-1. `[with-infra]`.
- **`contracts 87→1` (live, KEEP-consolidated):** re-point codegen/consumers to the ONE keyed JSON **FIRST**, then delete the scattered 87. **FRONT-LOADED as one of the FIRST `[with-infra]` waves** (MUST-FIX 4). `[with-infra]`.
- **`libs/oya-check-doc-axis` + the 4 keystone gate crates — DO NOT DELETE (NICE-TO-HAVE 8):** in the `libs/(586)` wave, these are **KEEP (live enforcement substrate)** — `oya-check-doc-axis` is the gate Workstream C extends/wires; the 4 keystone gate crates are born-blocking-shadow and are the local accounting/staleness/cross-artifact nets this plan relies on. A kill-list entry naming any of them is a mistake → ABORT that batch. (They are build-graph-referenced, so the producer-driven keep-list keeps them — this note is a belt-and-suspenders guard.)
- **`services/` + `crates/` = 0 tracked files:** empty trees → DESTROY the dirs. `[automatable-now]`.
- **`third-party/` (68):** vendored-behind-ports → KEEP (build-graph referenced). `[automatable-now]` (verify referenced).
- **`evidence/` (1533, mostly historical):** historical evidence is git-history material → DESTROY the stale majority; KEEP only gate-required current evidence (per the accounting-registry required-set). `[with-infra]` (needs the required-set to discriminate).
- **`multispectrum-review` doctrine (D-MULTISPECTRUM-RETIRED; founder 2026-06-07 — critique DROPPED, not re-homed):** the 21-facet consensus-debate review is retired/superseded (accounting half → firewall; critique half DROPPED, sacrificing ADR-0322 substance-bar + ADR-0247 SOC2 CC8.1). **`[DONE 2026-06-07]` destroy-now slice committed+pushed (`7a3455e5`, github-mirror):** `docs/standards/multispectrum-review*.md` + `templates/checklists/pre-pr-multispectrum.*` + `evidence/debate/**` (324) + 5 dangling-free residue, paired with atomic producer face-regen (registry 24161→23842; gate-baseline shrink-only); registry-drift + cloud-ci-firewall + total-accounting GREEN; 42 crate tests for the .md-referencing crates GREEN. **SEQUENCED with Task #25/#26 (load-bearing today, gated):** the lane `libs/oya-check-dependency-seam` `multispectrum-evidence-attached` subcheck + fixtures (atomic edit, keep the 3 mechanical seam subchecks), `specs/multispectrum-review.json` (runtime-read by forbidden-vocab `oya-vcs-admission-gate-*` → destroy in the SAME sweep), the CI-consumed `evidence/multispectrum/backbone-*` + `backbone-microservices-ci.yml` filters/parse-args, the `oya-dev-cli`/`oya gate`/jenkins bridge, and the **ADR-0092 amend** (strip the 3 multispectrum subchecks first). **DE-REQUIRE (not re-back) the GATE-4 `multispectrum/reviewer requirement` row + AC-0.12 input + `bespoke-cloud-toolchain-services.json:174`** in lockstep, or GATE-4 reds. Drop Proposed ADRs 0327/0323/0322/0247; regen `specs/masterplan.json` dependency-seam milestone; rewrite the two `tenant-rbac` `evidence_refs` strings (KEEP crates); scrub the 3 cosmetic dangling refs. `[with-infra]`.

---

## §B — WORKSTREAM B: DRAFT-FRESH THE CANONICAL SSOT (do NOT migrate the mess) `[automatable-now / with-infra]`

**Goal:** author FRESH from the TRUE CURRENT STATE (§0.4 sources 1–5) the canonical JSON SSOT store(s) + LIMITED Obsidian-format markdowns + define the json→web generation path (later). The fresh JSON **REPLACES** the deleted sprawl. Do NOT carry forward sprawled docs.

**Dependency:** none upstream for schema (B1 can start now). **Blocks:** A (per-domain delete gated on **B1 schema + B2 extraction ONLY** — see the demotion note) + C (the doc-axis allow-list keys on the store schema; C1 consumes the B4 entity-incremental guard).

> **CRITICAL-PATH DEMOTION (MUST-FIX 8):** **A.2 deletion gates on B1 (schema-freeze) + B2 (truth-extraction) ONLY.** The 4 guards (B4: accessor/formatter/merge-driver/entity-incremental gate) and the json→web generation path are **NOT on the A-critical-path and NOT on the B-spine** — they are **co-sequenced with Workstream C** (C1 consumes the entity-incremental guard; C3 consumes the accounting-registry which already exists as a built crate). Front-loading B4 would needlessly block A behind `[with-infra]` substrate that A does not need. B3 (author-fresh) is the spine deliverable; B4 lands alongside C.

### §B1 — Freeze the store SCHEMA + the disciplined authoring method (the keystone — A depends on this) `[automatable-now]` `→ 🚪`
Define the canonical store model = **one-canonical-store-per-domain** (D-SSOT-CURRENT-TRUTH doc-as-data model): the domains are **instructions · design/masterplan · registry-catalog** (keyed/enum-accessed JSON), per the allow-list. Define the keyed schema (enum keys, no free-form), the authoring method (author from §0.4 sources, never from docs), and the doc-axis enum binding (D-DOCORG's ADR-0388 7-axis: `DECISIONS/PLANS/INDEX/SPECS-MS/SPECS-CRATE/RUNBOOKS/IPS` + transient `IDEAS`). **`🚪` founder sign-off freezes the schema** (A's per-domain interleave depends on a frozen schema — Option-3 note). `[inherently-manual]` for the schema-freeze door.

### §B2 — Truth-extraction pass (the barrier between B and A — Driver-1 / Pre-mortem S2) `[automatable-now]` `→ 🚪 per domain`
SOURCE-FORCED, read-only over the to-be-deleted doc set, PER DOMAIN. For each doc: "what current-truth does it assert that is NOT already in the fresh store or derivable from §0.4 sources 1–5?" → (a) already-captured: mark, (b) current-truth-not-yet-in-store: **fold into the fresh store**, (c) contradicts live code: **stale → drop** (correctly). The pass output is the **per-domain barrier gate**: a domain's docs cannot be deleted (§A.2 step-1) until every current-truth assertion is captured. `[inherently-manual]` for the founder "is this genuinely worth preserving?" judgment on flagged ambiguous items (D-AUTHORITY-CONVERSATION verify-nothing-lost). **`🚪` per-domain barrier sign-off.**

### §B3 — Author the fresh stores from the live state `[automatable-now]`
Populate the frozen-schema stores from §0.4 sources 1–5 + the B2 extraction residue. Keyed/enum-accessed. Author the LIMITED Obsidian-format markdowns (bounded prose: DECISIONS/RUNBOOKS/IPS per D-DOCORG). Define (do NOT build yet) the **json→web human-readable generation path** (later; pipeline-generated, NOT a CLI — D-CLOUD-NATIVE / D-ARCH-DASHBOARD).

### §B4 — Build the 4 guards (the structural defenses) `[with-infra]` — **DEMOTED OFF the A-critical-path; co-sequenced with Workstream C**
> **Not a precondition of A.2 deletion** (MUST-FIX 8). A.2 gates on B1 (schema) + B2 (extraction) only. These guards are the substrate C composes with — build them **alongside C**, not before A.
1. **Keyed accessor** — resolves enum keys; ERRORS on malformed/duplicate-key (never silently returns last).
2. **Canonical formatter** — idempotent byte-canonical round-trip; un-formatted store = RED.
3. **Key-aware merge driver** — resolves concurrent key-adds without clobber (git merge driver for the store).
4. **Entity-incremental gate** — a PR adding an off-schema entity = RED; blocks shadow/off-axis entities. **C1 consumes this guard.**
Each guard ships with RED/GREEN fixtures proving it BLOCKS (expanded test plan, B row). `[with-infra]` (co-sequenced with C; C3 consumes the accounting-registry, which already exists as a built crate — see Tasks #33–45). The **json→web human-readable generation path is likewise DEFINED-not-built and co-sequenced with C** (NOT on the spine).

---

## §C — WORKSTREAM C: TIGHTEN THE KNOBS (the procedural fix — sprawl structurally impossible) `[with-infra]`

**Goal:** enforcement that makes sprawl structurally impossible to recur. **Maps onto the existing firewall** (`CICD-DESIGN-PLAN.md` — the ONE canonical CI / `oya-ci-required` blocking context produced by GitHub Actions) — the doc-axis gate is a constituent lane of that firewall fan-in.

> **⚠️ STARTING STATE (CORRECTED GROUND TRUTH — C is an UNWIRED FAÇADE today, not "an existing gate to flip"):** `libs/oya-check-doc-axis` (a) scans ONLY `docs/decisions`,`docs/ideas`,`docs/`,`microservices/` — it **NEVER scans `oya/`** (the 6,550-file mass this whole pass is about); (b) is **warning-not-error / non-strict**; (c) allow-lists via **appendable arrays** `LEGACY_DOCS_ROOT_FILES`(40) + `LEGACY_DOCS_SUBDIRS`(34) (anyone can widen the allow-list by appending); (d) is wired into **0 `.github/workflows/`** (it never runs in CI). So Workstream C is NOT "flip a switch" — it is **four concrete structural fixes** (C1.a–d). The firewall itself is also not live: `oya-ci-required` is required-by-nobody; the 4 keystone gate crates are born-blocking-SHADOW (CORRECTED GROUND TRUTH).

**Dependency:** `→ A` (the firewall enforces the clean state A establishes) `→ B1` (the doc-axis allow-list keys on the store schema) `→ B4` (C1 consumes the entity-incremental guard — co-sequenced, not blocking-before-A). `→ CICD-DESIGN-PLAN Stage-1` (the firewall fan-in surface). **Founder sequencing:** cleanup-now establishes; firewall-later prevents regression (D-SSOT-CURRENT-TRUTH: "do not wait for the firewall" to clean, but the firewall lands after to ENFORCE stays-current-truth-only).

### §C1 — Make `oya-check-doc-axis` a REAL blocking gate (four concrete sub-fixes — it is an unwired façade today) `[with-infra]` `→ 🚪`
The **structural fix**. Each sub-fix is independently RED/GREEN-provable (expanded test plan, C row):
- **(a) Extend the scan root to cover `oya/` (and the whole tree).** Today it scans only `docs/*` + `microservices/`; it must scan `oya/` (the 6,550-mass) and the rest of the tree, **excluding `.claude/worktrees/`, `target/`, `.git/`, `buck-out/`** (the 2M-phantom dirs — N2). RED proof: an off-allow-list `.md` inside `oya/` goes RED (today: silently ignored).
- **(b) Flip `strict=true` / blocking.** The gate must `error`-exit (fail the check), not emit a `warning`. RED proof: the §(a) fixture produces a non-zero exit.
- **(c) Replace the appendable `LEGACY_DOCS_*` arrays with the CLOSED D-SSOT allow-list enum.** No more silent widening by appending a row; the allow-list is the closed set from D-SSOT-CURRENT-TRUTH (carve-out markdown + the 974 RUNBOOKS + the domain stores + generated views + live configs + build-graph + the consolidated benchmarks/contracts). RED proof: appending to the (now-removed) array does nothing; an entry not in the enum is rejected.
- **(d) WIRE the gate into CI as a constituent lane of the `oya-ci-required` fan-in.** It is in **0 workflows** today — it must become a required check. RED proof: the gate appears as a required status on a test PR (today: absent → never runs). `[with-infra]`, **gated on CICD-DESIGN-PLAN Stage-1** (the fan-in surface must exist before a lane can join it).

**C1 is the PR-server-side structural fix** (server-enforced, cannot be bypassed by a local config). `[with-infra]`.

### §C2 — Write-hooks (block ad-hoc doc outputs at author-time) `[with-infra]` — **ADVISORY-ONLY (defense-in-depth, NOT the structural fix)**
Pre-write/pre-commit hooks that block doc `.md`/`.json` outside the stores at author-time. **These are advisory-only** — a local hook can be skipped (`--no-verify`), so they are defense-in-depth layered on top of C1, **not the enforcement.** The structural guarantee is C1's server-side PR gate (D-DOCORG two-layer drift defense: the local hook catches drift early, but the PR gate is the one that cannot be bypassed).

### §C3 — The allow-list + accounting-registry as the closed set `[with-infra]`
Wire the C1(c) closed enum as the doc-axis gate's allow-list; the accounting-registry total-accounting gate (GATE-2, **already a built crate** — Tasks #33–34) blocks unaccounted files (every file owned + justified + reachable + TTL). This is the standing enforcement that replaces the one-time cleanup posture. C3 **consumes the existing accounting-registry** — no new producer needed. `[with-infra]`. **`🚪` founder sign-off** before any of C's gates flip blocking (matches the CICD-DESIGN-PLAN HALT-before-blocking discipline; a flip before its infra exists wedges every PR — the firewall pre-mortem S2).

---

## STAGED LANES — dependency spine + gates (at-a-glance)

```
  ┌─ B1 schema-freeze 🚪 ──┬─► B3 author-fresh ─────────────┐
  │                        │   (with-infra)                 │
  │                        ▼                                ▼
  └─► B2 truth-extraction (N1 forbidden-vocab pre-filter)  [stores authored]
      🚪(per-domain barrier)                                │
      + §A.0.1 frontmatter-subtype 🚪(PROPOSED=DESTROY)     │
                                                            ▼
                          ┌──── A.1 keep-list/kill-list 🚪 ◄┘
                          │     (per-file frontmatter subtype; NOT extrapolation)
   THE A-SPINE GATES ONLY │
   ON B1 + B2 ───────────►▼
                          A.2 per-domain DESTROY waves  (+ A.2.1 rm -rf .claude/worktrees)
                          (each: extract+N1-green → rm → SCOPED ref-scrub
                           [excl .claude/worktrees,target,.git,buck-out] →
                           producer-regen → buck2 build + LOCAL shadow-gate smoke →
                           verifier-lane → 🚪 → SIGNED commit → mirror)
                          FIRST waves: benchmarks 8→1, contracts 87→1; bin/oya LAST
                                                  │
   ── CO-SEQUENCED, OFF THE A-SPINE ─────────────▼──────────────────────────
   B4 build-4-guards (with-infra)  ───►  C1 oya-check-doc-axis 4 fixes 🚪
   (accessor/formatter/merge/             (a)scan-root+oya/ (b)strict (c)closed-enum
    entity-incremental → feeds C1)        (d)WIRE-into-CI  [gated on CICD Stage-1]
   json→web DEFINED-not-built                       ├─ C2 write-hooks (ADVISORY-only)
                                                     └─ C3 closed enum + accounting-registry
                                                        (GATE-2 exists) 🚪 flip-blocking
                                                        (firewall ENFORCES the clean state)
```
**Gates (`🚪` door:one-way founder sign-off):** B1 schema-freeze · §A.0.1 PROPOSED-IP=DESTROY class-rule · B2 per-domain truth-extraction barrier (incl. N1 forbidden-vocab pre-filter) · A.1 kill-list manifest · **every A.2 batch commit** · C3 firewall-flip-blocking. **B4 guards are co-sequenced with C (no longer a pre-A gate — MUST-FIX 8). Verifier lane is separate at every A.2 batch and every B/C guard.**

---

## ADR — Big Hygiene Pass sequencing + procedural fix

- **Decision:** Execute the whole-source-tree hygiene pass as **three workstreams under Option-1 sequencing (B-captures-truth-FIRST → A-deletes-per-domain → C-enforces)**, with the Option-3 per-domain interleave as the intra-A shape (after the B1 schema-freeze) and the checkpoint `e38624dc4` as the recovery net. KEEP-posture (default-DESTROY, ambiguous→DESTROY). SOURCE-FORCED + separate-verifier-lane on every task. The fresh JSON SSOT is authored from the LIVE state (build-graph/CI/gate/contracts/services), NOT migrated from the sprawled docs.
- **Drivers:** (1) deleting a doc holding current-truth-not-in-code is unrecoverable as *canon* → B-first. (2) the CWD-contamination bug already deleted/audited the wrong repo → SOURCE-FORCED abort-on-port-marker fence. (3) the enforcement that prevents regression is **an UNWIRED, non-blocking, wrong-scoped façade** (`oya-check-doc-axis` doesn't scan `oya/`, isn't strict, has appendable allow-lists, is in 0 workflows) → manual SCOPED grep ref-scrub net now + the four concrete C1 fixes land after to enforce. (4) the source docs are **brand-contaminated** (1,266 cite retired-CLI, 1,820 cite dropped-infra) → a HARD N1 forbidden-vocab pre-filter on B2 so the fresh SSOT doesn't ship the contamination.
- **Counts corrected (ITERATION-2):** the adjudication is **per-file frontmatter SUBTYPE**, not directory and not extrapolation. Verified: `oya/*.md`=6,550; RUNBOOKS=974 (KEEP); explicit-PROPOSED-floor=≈408 (union of 4 explicit frontmatter signals; 692 any-token) (DESTROY); IP/spec family ≈ 2,422 by `doc_class` frontmatter (189 by dir-path; 2,950 by `IP-` filename-prefix) + 128 impl-plans (per-file split). The Architect's "6,550 runbooks" and the Critic's "~5,377 PROPOSED" are **both explicitly RETRACTED** as over-claims. These counts are INDICATIVE — the per-file frontmatter read AT EXECUTION is authoritative.
- **Alternatives considered:** Option-2 (A-first, rely on checkpoint) — rejected as primary (reactive recovery fails the verify-nothing-lost rule), retained as the fallback net. Option-3 (pure per-domain interleave) — folded into Option-1 as the intra-A shape (the cross-domain schema must freeze first regardless).
- **Why chosen:** Option-1 is the only sequencing that *structurally* prevents the dominant risk (unrecoverable-canon truth-loss) via the mandatory truth-extraction barrier that GATES deletion; it matches the founder's "draft fresh, don't migrate the mess" + "verify nothing dropped is worth preserving"; the per-domain interleave gives clean door:one-way granularity; the checkpoint is the net, not the plan.
- **Consequences:** the truth-extraction pass is the long-pole effort (read-only, SOURCE-FORCED, frontmatter-subtype-driven over the 6,550 `oya/*.md` + the doc/registry/evidence layer). **A.2 deletion gates on B1 (schema) + B2 (extraction+N1) ONLY; the B4 guards + json→web are DEMOTED off the A-spine and co-sequenced with C.** Git becomes the sole history (archaeology recovers the "why"). The clean state is established BY the cleanup and KEPT by Workstream-C — but C is **four concrete fixes to an unwired façade**, not a flip: (a) scan `oya/`, (b) strict-blocking, (c) closed-enum, (d) wire-into-CI. `bin/oya` delete is sequenced LAST and coupled to its CI-caller scrub **after** the cargo-prefix-gate re-home; benchmarks 8→1 and contracts 87→1 are FRONT-LOADED CONSOLIDATE-with-dependency waves. `.claude/worktrees/` (2M untracked) is a distinct `rm -rf` local-hygiene step + a mandatory ref-scrub exclude.
- **Follow-ups (for the Architect/Critic + later campaigns):** (a) the json→web human-readable generation path is DEFINED not built (later, pipeline-generated; co-sequenced with C, off the spine). (b) Workstream-C gates flip blocking only after their infra exists (HALT-before-blocking, per CICD-DESIGN-PLAN) — C1(d) wiring is **gated on CICD-DESIGN-PLAN Stage-1**. (c) the truth-extraction's "worth preserving?" judgment calls + the §A.0.1 "PROPOSED-IP=DESTROY" class-rule are flagged for founder ratification (inherently-manual). (d) the C2 write-hooks are advisory-only; C1 is the server-side structural fix. (e) **OPEN TENSION for the Architect** (see below).

---

## OPEN TENSION I could not fully resolve (for the Architect)

**The B-before-A truth-extraction barrier vs the founder's "bias HARD to DELETE / ambiguous→DESTROY" posture are in genuine partial conflict at the margin.** The barrier (Option-1, Driver-1) says *do not delete a doc until its current-truth is provably captured* — which introduces a "capture-first" step that looks like the "review pile" the founder explicitly forbade ("no default review pile"). I resolved this by scoping the barrier to **current-truth assertions ONLY** (validated against live code; a doc that asserts nothing-not-in-code is deleted immediately with no review, and ambiguous-staleness→DESTROY) — so the barrier is *not* a review pile, it is a narrow truth-capture gate on the small subset of docs that assert non-code-derivable current-truth. **But the boundary "asserts non-code-derivable current-truth" vs "stale narration" is a judgment the truth-extraction agent must make, and getting it wrong in the DESTROY direction silently drops canon (S2) while getting it wrong in the KEEP direction re-creates the review pile.** The Architect should rule: (a) is the truth-extraction barrier's current-truth-only scoping tight enough to honor "no review pile," or (b) should the barrier be even thinner (e.g. extraction limited to a fixed high-signal doc set — RUNBOOKS/decision-rationale/sequencing — with everything else default-DESTROY-no-extraction relying purely on the `e38624dc4` net), accepting a higher S2 truth-loss risk in exchange for stricter posture-fidelity? My recommendation is (a) with the high-signal dirs (RUNBOOKS, decision/sequencing rationale) exhaustively extracted and the bulk-stale dirs (evidence, generated indexes) sampled-then-default-DESTROY — but this is the one place the founder's two rulings (capture-nothing-lost vs bias-to-delete) pull against each other and warrants the consensus ruling.
