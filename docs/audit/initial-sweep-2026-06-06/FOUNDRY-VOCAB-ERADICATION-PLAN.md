# FOUNDRY-VOCAB-ERADICATION-PLAN — the held big-2 item (i)

**Status:** PLAN-READY — awaiting founder **approach-approval** (door:one-way). NO source mutation has occurred.
**Scope target:** `/Users/jasonlee/Developer/source` (read-only during planning).
**Author posture:** evidence-driven, real counts + file paths. This is the wave-3 vocab-eradication batch that closes the deferral recorded in **F-0024** and **ADR-0335 D-37 / D-43** (the "future rename in a separate cleanup wave").
**Plan doc:** `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/FOUNDRY-VOCAB-ERADICATION-PLAN.md`

---

## 0. Why this exists (governing doctrine, source-cited)

- **ADR-0335** (`docs/adr-archive/ADR-0335-intelligence-microservice-consolidation.md`): foundry µservice RETIRED, absorbed by intelligence. **D-37**: existing `oya-foundry-*` crates retained as transition debt (precedent ADR-0333 D-59) to avoid a 122-crate cascade across 43 dependents. **D-39**: new code MUST NOT generate `oya-foundry-*`. **D-43**: *"future renaming of `oya-foundry-*` crates to `oya-intelligence-*`"* deferred to a separate cleanup wave. **This plan IS that wave.**
- **ADR-0347** (`docs/adr-archive/ADR-0347-governance-fitness-bulk-rename.md`): doctrine-only declaration that foundry-*fitness* lanes → `oya-governance-*`; the actual file renames were deferred to "Wave 15-ZB". **This plan absorbs Wave 15-ZB.**
- **ADR-0363** (`docs/adr-archive/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md`) + **ADR-0116**: foundry-as-VCS-substrate retired → **vcs sense routes to RETIRED.**
- **ADR-0515 / ADR-0513**: one canonical bespoke-Rust CI/CD posture supersedes the **jenkins/forgejo** self-hostable substrate cluster (ADR-0349/0374/0387) — F-0008.
- **D-FORGE / D-FOUNDRY-CLARIFY** (`registry/catalog/oya-check-brand-residue.yaml` + `libs/oya-check-brand-residue/src/lib.rs`): `forgejo` already executable in the deny-list; `foundry` is tracked but **`executable: false`** pending a cleared corpus. **The enforcement promotion at the end of this plan is what makes the eradication permanent.**

---

## 1. INVENTORY — per-vocab residue totals (tracked files, evidence-counted 2026-06-07)

Counted with `git grep -I` on the `cleanup/whole-tree-2026-06-07` branch of `/Users/jasonlee/Developer/source`.
**Build artifacts excluded** (`buck-out/`, `target/` — both git-untracked, 0 tracked files). `.omc/` (OMC session state, 726 tracked files) is reported as a **separate bucket** — it is regenerated/ephemeral state, not product source, and is scrubbed last.

| vocab | occurrence-lines (tracked, non-`.omc`) | files (tracked, non-`.omc`) | `.omc` bucket (lines / files) | carve-outs |
|---|---|---|---|---|
| **foundry** | **21,663** | **3,214** | 1,923 / 244 | Palantir Foundry allowed (307 lines co-occur with "palantir"; ~596 "palantir" total) |
| **jenkins** | **6,386** | **1,225** | (subset of `.omc`) | — |
| **oya-vcs** | **1,105** | **101** | (subset of `.omc`) | — |
| **forgejo** | **54** | **13** | (subset of `.omc`) | — |

> Note on count drift vs F-0024 (which recorded foundry 4567 / jenkins 1626 / oya-vcs 253 / forgejo 18): F-0024 counted file-hit `-c` and a pre-wave-2 snapshot; the wave-2 crate deletions (`c43d38b20`) already removed the 4 `oya-vcs-*` crates + `oya-ci-webhook-gateway-jenkins-adapter`. The line-level numbers above are the **current** residue. Either way the **direction and ranking are identical**: foundry ≫ jenkins ≫ oya-vcs ≫ forgejo.

### 1a. `foundry` carve-out rule (MUST honor)
- **External "Palantir Foundry" is ALLOWED** — do NOT rename. Detection: any line matching `/palantir/i` on the same line, OR prose comparing/contrasting the external product. ~307 lines. These get an **allowlist entry** in the brand-residue lane config (file-path + line-anchor allowlist), not a rename.
- **`talos` is NOT forbidden** — explicitly out of scope (it is the boot/init substrate, kept). Do not count, do not touch.

### 1b. Highest-density `foundry` directories (rename surface, 2-level)
```
442 oya/intelligence      272 registry/catalog     220 docs/decisions
195 docs/user-journeys    130 docs/personas         68 oya/governance
 51 docs/standards         46 oya/workflow-studio    40 oya/developer-sdk
 (plus ~30 cloud/* microservice dirs at ~20 files each)
```

### 1c. Structural identifiers (the load-bearing renames — NOT prose)
- **655 distinct `oya-foundry-*` identifiers** in the tree (crate names, catalog `context:`, owners, lane IDs).
- **`Cargo.toml` workspace-metadata block** `[workspace.metadata.oya.microservices.foundry]` (line 772) + `owner = "council-foundry"` (lines 773, 1061) + `owner = "axis-foundry"` (line 969).
- **1,129** `axis-foundry` / `council-foundry` owner references across the corpus.
- **1 BUCK target name**: `oya/intelligence/crates/oya-intelligence-supervisor-app/BUCK:27 → name = "oya-foundry-supervisor"` (path already moved to intelligence; the BUCK *target name* still carries forbidden vocab — exemplary residue).
- `Cargo.lock`: 0 foundry hits (no published-crate name dependence — good, lock regenerates).

---

## 2. SENSE ROUTING — the 4-way classification of every `foundry`

Required routing: **platform→oya-intelligence · fitness→oya-governance · vcs→retired · re-home-deferred**.
Signal-based line classification (lines may match >1 signal; resolution order is **vcs → fitness → platform → re-home-deferred**, i.e. most-specific first):

| sense | route | dominant signal | line-level estimate | distinct `oya-foundry-*` idents |
|---|---|---|---|---|
| **VCS** | **RETIRED** (delete / supersede-note; no replacement term) | `vcs`, `scm`, `repoctl`, `git-substrate`, `sapling`, `M01-P*`, `write-gate`, co-`forgejo` | ~415 | webhook-receiver-kernel + vcs-substrate refs (small set; per ADR-0363/0116) |
| **FITNESS** | **oya-governance-\*** | `fitness`, `governance`, `axis-foundry`, `council-foundry`, `check-*`, `lane`, `cohesion`, `brand-residue`, `*-ceiling`, `*-cite`, `data-class` | ~4,073 | **82** (e.g. `oya-foundry-cohesion-fitness-kernel`, `oya-foundry-brand-residue-kernel`, `oya-foundry-claim-ceiling-kernel`, `oya-foundry-data-class-fitness-kernel`, `oya-foundry-constitution-cite-kernel`) |
| **PLATFORM** | **oya-intelligence-\*** | `intelligence`, `supervisor`, `provider(s)`, `eval`, `mcp`, `capability registry`, `runtime`, `account`, `run/step pipeline`, `engineering platform` (ADR-0025) | ~9,636 | **250** (e.g. `oya-foundry-supervisor`, `oya-foundry-eval-app`, `oya-foundry-providers-router-adapter`, `oya-foundry-account-*`, `oya-foundry-capability-kernel`) |
| **RE-HOME-DEFERRED** | **HOLD** (founder decides target; do not guess) | remainder: bare prose, persona/user-journey narrative, ambiguous catalog `context: foundry` with no platform/fitness/vcs signal | remainder (~7,000 prose lines + the rest of the 655−332≈323 idents not cleanly bucketed) | ~323 idents needing per-ident adjudication |

**Hard rule (verify-each-step):** the line-estimates above are signal-grep approximations and OVERLAP. The executor MUST produce a **per-ident adjudication table** (655 rows) and a **per-file disposition** before mutating — no batch mutates a `foundry` token whose sense is `re-home-deferred` until that ident has a founder-confirmed target. **Re-home-deferred is a real bucket, not a dumping ground**: it is the explicit "ask the founder" pile.

---

## 3. F-0004 — fix the TEMPLATE SOURCE, not the 1,200 outputs (jenkins boilerplate)

**Finding F-0004** (`FINDINGS-LEDGER.md:18`): ~945 oya + ~260 cloud `ARCH/README/PRD` (and runbooks) carry **identical jenkins boilerplate generated from a canonical template** — recommendation is *"fix the template source + regenerate; never hand-scrub 1200 files."*

**Root cause located (evidence):**
- The boilerplate originates from **ADR-0349** (`docs/decisions/ADR-0709-general-live-apex.md`): *"Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates."* Its `enforced_by` lane list — **`oya-governance-jenkins-github-actions-parity`** (841 occurrences) and **`oya-governance-jenkins-jcasc-only`** (738 occurrences) — is the dominant jenkins residue, stamped into per-service docs.
- The stamping vector: per-microservice doc-sets where the **same files repeat across ~80 services** — `IP-WAVE-15-ZD-sharding-automation.md` (80 copies), `hot-split.md`/`cold-merge.md`/`auto-rebalance.md` (82 each), `dpia.md` (87). md5 confirms each copy has per-service substitution but shares the jenkins lane/prose block (= templated injection, not hand-authored).
- Distribution: jenkins is **318 lines in ADR source** (the SSOT), **225 lines in `*.generated.*`**, and **~4,000 lines in stamped per-service output**.

**Therefore F-0004 fix order (template-first):**
1. **Supersede ADR-0349** wording via the ADR-0515 cluster re-author (F-0008) so the SSOT no longer names Jenkins/ArgoCD as canonical substrate → replaced by the one bespoke-Rust CI posture. This kills the 318 source lines + the lane-name SSOT.
2. **Retire the two jenkins-named lanes** (`oya-governance-jenkins-github-actions-parity`, `oya-governance-jenkins-jcasc-only`, `oya-governance-jenkins-canonical-no-gha-residue`) at their catalog/lanes.yaml definition.
3. **Fix the doc-set scaffold/generator** that injects the lane block into per-service docs (candidates: `oya/*/crates/*-doc-set-scaffold`, the masterplan/board-sync generators under `oya/developer-sdk/.../commands/generate/`, and the IP-WAVE-15-ZD stamping path). Then **regenerate** all 80–87× copies from the corrected template — **never hand-edit the ~1,200 outputs.**
4. Producer-regen produces the new generated outputs; a **diff-oracle** confirms the regen removed all jenkins lines.

---

## 4. STAGED · BATCHED · DRY-RUN EXECUTION PLAN

**Cross-cutting rules for every batch:**
- **Dry-run first**: each batch is authored as a scripted, idempotent rename set; run it against a throwaway worktree, capture the diff, count residual hits, BEFORE touching the real tree.
- **Each rename touches 5 ref-classes** — the executor MUST update all 5 atomically per ident: **(a) BUCK** target names + deps; **(b) Cargo** `[workspace] members` + `name =` + path deps + `Cargo.lock` regen; **(c) code** `use`/imports/string-literals; **(d) docs** ADR cross-cites + per-service docs; **(e) registry** catalog `context:`/lane IDs + `registry/quality/lanes.yaml` + manifests.
- **Producer-regen per batch**: re-run the accounting-registry-producer + masterplan/board-sync/architecture-graph generators so generated faces match the renamed source.
- **Gate-smoke per batch**: `cargo check --workspace --keep-going`, `cargo nextest run` on touched crates, `oya gate run-all` (or the cloud-ci-firewall runner), and the brand-residue lane in **report-mode** (count remaining hits) — must be GREEN (or hits monotonically decreasing) before the next batch.
- **door:one-way sign-off** is taken at the points marked **🚪** below — irreversible, founder-paired.

---

### BATCH 0 — Adjudication + dry-run harness (NO mutation)  🚪 **door #1: approach-approval (THIS doc)**
- Produce the **655-row per-ident adjudication table** (ident → sense → target-name → ref-classes-touched → batch#). Resolve every `re-home-deferred` ident to a founder-confirmed target OR explicitly park it.
- Produce the **carve-out allowlist** (the ~307 Palantir-Foundry lines, file-path + line-anchor) for the brand-residue lane config.
- Build the dry-run rename script + diff-oracle.
- **Exit:** founder approves the routing + target-name convention. **Nothing renamed yet.**

### BATCH 1 — `forgejo` eradication (smallest, fully-decided)
- **Why first:** 54 lines / 13 files; sense is unambiguous (RETIRED per ADR-0363; already `executable: true` would fail if it weren't historical-allowlisted). Lowest blast radius → proves the harness.
- **Refs:** ADR-0513 + ADR-0515 source lines (supersede-note, keep as historical-context only); `infra/external-secrets/externalsecret-forgejo-ci-token.yaml` (delete — dead secret); `infra/gitops/jenkins-vcs-token.secret.template.yaml` (delete with Batch 2); 24 generated-file lines (regenerate); `libs/oya-check-brand-residue` already lists it.
- **Producer-regen:** accounting-registry + enforcement-inventory + architecture-graph.
- **Gate-smoke:** brand-residue lane stays GREEN (forgejo already executable, only historical refs remain).

### BATCH 2 — `jenkins` template-source fix + retire (F-0004, depends on ADR-0515 re-author)
- **Why second:** large (6,386 lines) but **template-driven** — fix once, regenerate. Sequenced after the ADR-0515 cluster re-author (F-0008) supersedes ADR-0349.
- **Step 2a (SOURCE):** supersede ADR-0349 wording; retire the 3 jenkins-named lanes from `registry/quality/lanes.yaml` + `registry/catalog/`.
- **Step 2b (TEMPLATE):** fix the doc-set scaffold/generator injecting the lane block; delete `infra/gitops/jenkins-vcs-token.secret.template.yaml`.
- **Step 2c (REGEN):** regenerate all 80–87× stamped per-service docs (`IP-WAVE-15-ZD-sharding-automation.md`, `hot-split/cold-merge/auto-rebalance/dpia.md`, ARCH/README/PRD). **Never hand-scrub.**
- **Gate-smoke:** diff-oracle confirms 0 jenkins lines in regenerated output; `cargo check` green; brand-residue lane reports jenkins count → 0.

### BATCH 3 — `oya-vcs` eradication (RETIRED sense, crates already gone)
- **Why third:** 1,105 lines / 101 files; the 4 `oya-vcs-*` crates were already deleted in wave-2 (`c43d38b20`), so this is **residual references** (catalog records, evidence/*, ADR cross-cites, gitops admission policy).
- **Refs:** `registry/catalog/oya-vcs-*-gate-*.yaml` (delete), `registry/vcs/`, `evidence/gitops-vcs/`, `deploy/gitops/oya-vcs-admission` policy (ADR-0117), ADR cross-cites become historical-context.
- **Producer-regen + gate-smoke:** accounting-registry must show the deleted catalog records gone; registry-drift lane green.

### BATCH 4 — `foundry` FITNESS sense → `oya-governance-*` (the 82 fitness idents + ADR-0347 Wave 15-ZB)
- **Why fourth:** medium-sized, fully-decided target (ADR-0347), self-contained in the governance/check-family.
- **Refs per ident (×82):** crate dir `git mv` + `name =` + Cargo members + BUCK + `use` paths + catalog `context:` + lane IDs in `registry/quality/lanes.yaml` + ADR cross-cites + per-service `governance_lanes` manifest arrays + `.github/branch-protection.yaml` required-checks + `tools/hooks/_canonical-primitives.md`.
- **Special case:** the residue-grep lane `oya-governance-no-foundry-fitness-residue` *intentionally contains the word* — keep its **historical-context allowlist** so it does not self-trip.
- **Owners:** `axis-foundry`/`council-foundry` (fitness-owned subset) → `axis-governance`/`council-architecture` per ADR-0132.
- **Producer-regen + gate-smoke** after the bulk-rename PR (single Wave 15-ZB-style fan-out, NOT 34 per-lane PRs per ADR-0347).

### BATCH 5 — `foundry` PLATFORM sense → `oya-intelligence-*` (the 250 platform idents, ADR-0335 D-43)
- **Why fifth / largest:** the 122→250-ident cascade ADR-0335 deferred. Highest blast radius (43+ dependent crates). Sub-batch by dependency layer: **kernel → adapter → app → application**, leaf-first, so each sub-batch keeps `cargo check` green.
- **Refs per ident (×250):** same 5 ref-classes; plus `Cargo.toml` `[workspace.metadata.oya.microservices.foundry]` block (line 772) → rename to `intelligence` (or delete if intelligence block already exists) + `owner = "council-foundry"` (773, 1061) → `council-intelligence`; the BUCK target `oya-foundry-supervisor` (supervisor-app/BUCK:27) → `oya-intelligence-supervisor`.
- **Producer-regen:** full regen (accounting-registry, masterplan, board-sync, architecture-graph); `Cargo.lock` regen.
- **Gate-smoke:** per sub-batch `cargo check --workspace` + nextest on touched crates; full `oya gate run-all` at batch end.

### BATCH 6 — `foundry` RE-HOME-DEFERRED + remaining prose (founder-adjudicated only)
- **Refs:** the ~323 unbucketed idents + ~7,000 prose lines (personas, user-journeys, narrative docs) that don't carry a platform/fitness/vcs signal.
- **Process:** each disposed per the Batch-0 adjudication table; bare prose `Foundry`→sense-correct replacement; the **carve-out Palantir lines are SKIPPED** (allowlist).
- **Gate-smoke:** brand-residue lane reports foundry count approaching 0 (excluding allowlisted Palantir + historical-context anchors).

### BATCH 7 — `.omc/` state bucket + ENFORCEMENT PROMOTION  🚪 **door #2: make-it-permanent**
- Scrub the `.omc/` bucket (244 files / 1,923 foundry lines) — these are regenerable session state; regenerate rather than hand-edit where possible.
- **Promote `foundry` to `executable: true`** in `registry/catalog/oya-check-brand-residue.yaml` + add to `FORBIDDEN_BRAND_TOKENS` in `libs/oya-check-brand-residue/src/lib.rs` (currently `&["forgejo"]` → `&["forgejo", "foundry", "jenkins", "oya-vcs"]`), with the Palantir + historical-context allowlist wired in.
- **RED/GREEN proof:** add a fixture that a re-introduced `foundry`/`jenkins`/`oya-vcs`/`forgejo` token now FAILS the lane (RED), and the cleared corpus PASSES (GREEN).
- **Exit (door:one-way):** founder signs off that the deny-list is live and merge-blocking → a dropped brand can never come back up.

---

## 5. Ordering rationale (one line)
forgejo (proves harness) → jenkins (template-first, depends on ADR-0515) → oya-vcs (residual) → foundry-fitness (decided target) → foundry-platform (largest cascade, leaf-first) → foundry-re-home (founder-adjudicated) → enforcement-promotion (permanent). Each batch is dry-run → 5-ref-class atomic rename → producer-regen → gate-smoke → next.

## 6. Verification ledger this plan answers
- **F-0004** (jenkins template) — §3, Batch 2 (template-source fix, never hand-scrub).
- **F-0024** (forbidden-vocab crate members + foundry workspace-metadata) — §1c, Batches 4–5, 7.
- **F-0008** (CI/CD ADR cluster) — Batch 2 prerequisite (ADR-0515 supersede).

**Counts are from the live `cleanup/whole-tree-2026-06-07` tree, 2026-06-07. No source was mutated to produce this plan.**
