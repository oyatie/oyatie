# 10 — Flat-crates contradiction map

Scope: fully map the supersession of the flat **top-level `crates/`** topology, find EVERY remaining
stale assertion of it across `docs/decisions/`, `docs/standards/`, `specs/`, `registry/`, distinguish
**location** (superseded) from **naming** (survives), and record the live-tree + gate-wiring state.

All paths are under `/Users/jasonlee/Developer/source/` unless noted. Every claim is cited `path:line`
with a verbatim snippet. `.claude/worktrees/**` copies are excluded from all counts (stale clones).

---

## 1. Supersession chain (verified)

```
ADR-0015 (accepted, partially superseded)
   └─ superseded_by:[ADR-0131]  (only the docs-vs-crates top-level split)
ADR-0131 (Accepted) supersedes ADR-0015 (partial) + ADR-0119 (partial)
   └─ amended 2026-06-02: top-level service root microservices/<ms>/ → {oya,cloud}/<service>/
ADR-0357 (Proposed, NEVER accepted) — vertical-slice nesting microservices/<ms>/crates/
ADR-0509 (Accepted) — single-crate-per-service
ADR-0512 (Accepted, founder-locked, 2026-05-29) — CANONICAL
   ├─ supersedes ADR-0357, ADR-0509
   ├─ amends ADR-0131
   └─ amended 2026-06-02: service root = {oya,cloud}/<service>/crates/<crate>/; libs/<lib>/
       "A flat top-level `crates/` directory is **forbidden**."
```

Source frontmatter:

- ADR-0015: `docs/decisions/ADR-0709-general-live-apex.md:3` — `status: accepted`;
  `:5` — `superseded_by: [ADR-0131]`;
  `:6` — `supersession_note: "Partial — ADR-0131 supersedes only the docs-vs-crates top-level split..."`
- ADR-0131: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md:8-10` — `supersedes:` ADR-0015 (partial) + ADR-0119 (partial);
  `:24-28` — **"Amended — 2026-06-02 (pure split):** ADR-0512/platform-readiness updates the top-level service root from `microservices/<ms>/` to `{oya,cloud}/<service>/`."`
- ADR-0512: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md:9-13` — `supersedes: [ADR-0357, ADR-0509]`, `amends: [ADR-0131]`;
  `:53-57` — `"Service code lives at {oya,cloud}/<service>/crates/<crate>/ ... A flat top-level crates/ directory is **forbidden**. microservices/ is legacy/removal-candidate"`;
  `:62` — `"The architecture-boundaries gate requires service code under {oya,cloud}/<service>/crates/ or shared code under libs/<lib>/ (flat crates/ rejected ...). Workspace-topology validation fails on: a flat crates/ directory; ..."`

ADR-0357 is **Proposed**, not Accepted (`docs/decisions/ADR-0709-general-live-apex.md:3` `status: Proposed`) —
ADR-0512 superseded a never-ratified proposal. ADR-0509 is `status: Accepted` (`ADR-0509-...:4`).

---

## 2. LOCATION vs NAMING verdict (the core distinction)

- **(a) LOCATION — flat top-level `crates/` topology = SUPERSEDED / FORBIDDEN.** The on-disk address
  `crates/oya-<ctx>-<role>/Cargo.toml` (depth-3) is killed by ADR-0512:53,57,62. Canonical is now
  `{oya,cloud}/<service>/crates/<crate>/Cargo.toml` (depth-5), shared `libs/<lib>/`.
- **(b) NAMING — the crate name `oya-<context>-<role>[-<capability>]` = SURVIVES.** ADR-0131:97-98 still
  prescribes `oya-<ms>-<bc>-<layer>` crate dirs *inside* `src/crates/`; ADR-0512:59 keeps "crate directory
  basename MUST equal `[package].name`"; ADR-0056 (BNF v4.1) remains the naming authority (ADR-0131:383).
  ADR-0512:66 confirms "package names ... are unchanged."

**Therefore:** any reference to the *string* `oya-<ctx>-<role>` is NOT a contradiction. A reference is
stale ONLY when it asserts/enforces the crate **lives at top-level `crates/...`** (the LOCATION), or
enforces the depth-3 manifest-path rule, or names `microservices/<ms>/crates` (ADR-0357's dead path).

Most of the 656 `crates/oya-` hits in `docs/decisions/` are NAMING/illustrative `// crates/oya-x-kernel`
code-comment headers or `source_of_truth: crates/oya-...` registry rows — type (b), survive-with-path-rewrite,
NOT topology contradictions. The high-signal LOCATION/topology contradictions are enumerated in §3.

---

## 3. LIVE TREE STATE (verified)

- **NEW canonical roots PRESENT + git-tracked:** `oya/`, `cloud/`, `libs/` all exist
  (`git ls-tree --name-only HEAD` lists `cloud`, `libs`, `oya`). Legacy `microservices/` is **absent**
  (`ls microservices` → "No such file or directory").
- **STALE top-level `crates/` dir EXISTS on disk** but is the forbidden topology:
  - `ls -la crates` → `crates/.DS_Store`, `crates/oya-application-app/`, `crates/oya-audit-chain-emission-api/`.
  - **NOT git-tracked:** `git ls-files crates/` → 0 lines; `git status --short crates/` → empty; `git check-ignore crates` → rc=1 (not even gitignored — just untracked cruft).
  - Contents are **only** `.DS_Store` files (`find crates -type f` → 3 `.DS_Store` files, no `Cargo.toml`,
    no `src/`). The two subdirs use the `oya-<ctx>-<role>` naming but hold no code.
  - **Verdict:** a residual empty `crates/` shell that ADR-0512:57 ("must be empty after verified
    migration") / `:62` ("Workspace-topology validation fails on: a flat `crates/` directory") says must be
    DELETED. It is the live embodiment of the forbidden topology — sweep target (delete dir).

---

## 4. GATE WIRING STATE (verified) — the live contradiction

The `oya-governance-flat-crates` gate **still actively enforces the SUPERSEDED depth-3 topology**:

- `docs/governance-lanes/flat-crates.md:9` — `purpose: Verify workspace stays flat (no nested crates/foo/sub/) ...`
- `:7` — `status: Accepted` (dated `:8` `2026-05-12`)
- `:15` — failure mode `crate manifest path has depth > crates/<name>/Cargo.toml`
- `:42-48` — kernel sketch: `let depth = c.manifest_path.split('/').count(); if depth != 3 { return Err(...NestedCrate...) }`
- `:20` — `severity: BLOCKER`
- `:10` — `enforces: STANDARD/flat-workspace`

**This gate REJECTS the ADR-0512 canonical layout.** `{oya,cloud}/<service>/crates/<crate>/Cargo.toml`
has manifest-path depth 5, which `depth != 3 ⇒ NestedCrate` BLOCKER-fails. The flat-crates lane enforces
exactly the topology ADR-0512:53,57,62 forbids. **DIRECT live contradiction.**

Gate registration drift (also a contradiction):

- The lane IS listed as a live `existing`/`BLOCKER` gate in:
  - `docs/governance-lanes/INDEX.md:28` — `| flat-crates | existing | STANDARD/flat-workspace | oya-governance-flat-crates-kernel | tools/oya-governance-flat-crates | cargo run -p oya-governance-flat-crates | 100 | BLOCKER |`
  - `docs/AGENTS.md:231` — D7 lane list includes `... bypass, flat-crates, runbook-index-resolves, doc-catalog`
  - `templates/checklists/done-definition-checklist.md:33` — same D7 lane list
- But it is **ABSENT from the executable lane registry** `registry/quality/lanes.yaml`
  (`grep -in 'flat|nestedcrate|crates_checked|flat-workspace'` → rc=1, no match). The lane actually wired
  in lanes.yaml is `lean-a1-architecture` → `cargo run -p oya-dev-cli -- gate validate architecture-boundaries`
  (`registry/quality/lanes.yaml:485-493`, `source: ADR-0056`), which is the lane ADR-0512:62 says should be
  flipped to enforce `{oya,cloud}/<service>/crates/`. So: doc-layer claims an active BLOCKER flat-crates gate
  that (a) enforces the wrong topology and (b) is not in the machine lane registry.

Other live gate-doc references to the flat-crates lane (each repeats the superseded model implicitly):

- `docs/MISTAKES-LEDGER.md:56` — MFL-0012, `oya-governance-flat-crates` "active gate" guarding against
  reintroducing `modules/services/platform` — framed around top-level `crates/` as the canonical destination.
- `docs/PRIVACY-PROGRAM.md:178` and `docs/decisions/ADR-0709-general-live-apex.md:126` — the
  `oya-governance-flat-crates` **GATE** "rejects any new flat crate whose dep graph imports an ads/analytics
  adapter" (naming-survives wording, but the gate name binds the superseded lane).
- `docs/quality/ai-slop-defense/impossible-to-fail-environment-spec.md:69` — `MFL-0012 ... oya-governance-flat-crates | shipped`.
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md:63` (AIS-040) +
  `docs/quality/ai-slop-defense/gap-analysis-ai-vs-production.md:40` — flat-crates lane cited as enforcement.
- `docs/products/foundry/PRD.md:731` — `oya-governance-flat-crates validates every kernel-shape change`.
- `registry/glossary-vocabulary/warning-sources.tsv:9001` — STANDARD source `docs/governance-lanes/flat-crates.md`.

---

## 5. STALE flat top-level `crates/` LOCATION assertions to sweep (the requested map)

Seed refs (confirmed) — the four-corners "Flat-crates binding" GATE/topology wording:

| path:line | verbatim snippet | type |
|---|---|---|
| `docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md | `Applies to: every crate under \`crates/oya-*\`, every catalog record...` | (a) LOCATION |
| `docs/adr-archive/ADR-0008-data-use-boundary.md | `Architecture fitness gate — \`oya-governance-flat-crates\` rejects any new flat crate...` | GATE name binds superseded lane |
| `docs/adr-archive/ADR-0013-product-license-policy.md | `Applies to: every crate under \`crates/oya-*\`, every npm package...` | (a) LOCATION |
| `docs/adr-archive/ADR-0020-intelligence-multi-provider-adapter-model.md | `Flat-crates binding: the sealed provider-adapter contract lives in \`crates/oya-foundry-adapter-kernel\`; ... land under \`crates/oya-foundry-*\`` | (a) LOCATION |
| `docs/adr-archive/ADR-0022-autonomy-ceiling-runtime-enforcement.md | `Flat-crates binding: autonomy-ceiling enforcement lives in \`crates/oya-foundry-policy-kernel\` and ... through flat \`oya-foundry-*\` crates` | (a) LOCATION |

ADR-0015 self-assertions (the authoritative source of the superseded topology — survives as accepted for
BC/layer rules but its top-level-`crates/` location clauses are superseded by ADR-0131/0512):

| path:line | snippet |
|---|---|
| `ADR-0015-...:34` | `Live baseline ... every workspace member lives under \`crates/oya-*\` ... top-level modules/, services/, platform/, and tools/ are absent` |
| `ADR-0015-...:39` | (code block) `crates/oya-<context>-<role>[-<capability>]/` |
| `ADR-0015-...:160` | Operational CI: `oya-governance-flat-crates (path + legacy-root + role-boundary validator)` |
| `ADR-0015-...:201` | `ADR-0001 (cohesion — substrate kernels are flat crates), ... (contract registry cites flat crates)` |
| `docs/ADR-INDEX.md:37` | `ADR-0015 | accepted | Architectural flattening target — flat-crates \`crates/oya-<context>-<role>[-<capability>]/\`` |

ADR-0357 dead path `microservices/<ms>/crates/` (Proposed, superseded by ADR-0512) — still asserted as if live:

| path:line | snippet |
|---|---|
| `ADR-0357-...:25` | `Today all 546 code crates live in a flat \`crates/oya-*\` directory ... metadata ... lives separately under \`microservices/<ms>/\`` |
| `ADR-0357-...:27` | `the \`architecture-boundaries\` gate currently enforces flat \`crates/\`` |
| `ADR-0357-...:31` | `a service's crates move to \`microservices/<ms>/crates/oya-<service>-<layer>\` ... gate flips to enforce code under \`microservices/<ms>/crates/\`` |
| `ADR-0357-...:35` | `Until then, flat \`crates/\` remains canonical and gate-enforced.` |

specs/ (machine-readable) — assert the superseded topology as the migration TARGET:

| path:line | snippet |
|---|---|
| `specs/per-microservice-flat-layout.json:252` | `"to": "microservices/<ms>/crates/oya-<ms>-<bc>-<layer>/"` |
| `specs/per-microservice-flat-layout.json:345` | `"description": "no crate creation outside microservices/<ms>/crates/"` |
| `specs/per-microservice-flat-layout.json:365` | `"... [workspace.members] paths reference microservices/<ms>/crates/<crate>"` |
| `specs/masterplan.json:6046-6047` | `"choice": "flat crates/ layout", "verdict": "CHANGE -> vertical-slice nesting microservices/<ms>/crates + libs/ (ADR-0357)"` (cites the SUPERSEDED ADR-0357, not 0512) |
| `specs/cloud-strangler-migration-target.json:26` | `... vertical-slice nesting microservices/<ms>/crates, ADR-0357 ...` (cites superseded ADR-0357) |

docs/ standards + design + roadmap — "flat-crates target" / top-level `crates/oya-*` topology language:

| path:line | snippet |
|---|---|
| `docs/ROADMAP.md:63` | `flat-crates guard passes: every workspace crate is under \`crates/oya-*\` ...` |
| `docs/ROADMAP.md:52` | `... RBAC/ABAC (Cedar) at flat-crates target` |
| `docs/ROADMAP.md:172` | `flattening-additive-splits (... forward-only inside \`crates/oya-*\` per ADR-0015)` |
| `docs/DESIGN.md:446` | `the live workspace contains 64 \`crates/oya-*\` members ... top-level modules/, services/, platform/, and tools/ are retired` |
| `docs/DESIGN.md:23,64,337` | `Owning bounded context (flat-crates target)` / `maps to ... flat-crates targets` / `The flat-crates target encodes the layers as crate-level roles per ADR-0015` |
| `docs/DESIGN.md:497` | `Every leaf in the v2 backlog cites a flat-crates target` |
| `docs/DESIGN.md:186` | `\| flat-crates-move \| Mutates root \`Cargo.toml [workspace.members]\` \| per ADR-0015 phase PR ...` |
| `docs/SPEC.md:26,142` | `owning crate — flat-crates target` / `flat-crates target catalog per ADR-0015/0222` |
| `docs/TOOLCHAIN.md:202` | `"description": "Required when scope=crate; flat-crates target name"` |
| `docs/standards/code-style.md:57` | `One Rust crate per flat-crates target per ADR-0015` |
| `docs/standards/commit-message.md:103-104` | `Moves runtimes from services/* to crates/oya-*-runtime per ADR-0015 / flat-crates target.` |
| `docs/standards/code-review.md:35` | `Traceability — flat-crates targets touched ...` |
| `docs/standards/clean-architecture.md:49,396` | `ADR-0015 flat crates` |
| `docs/standards/code-style-rust.md:142,268` | `ADR-0015 (flat crates)` |
| `docs/standards/crate-naming-convention.md:42,423` | `ADR-0015 flat crates` |
| `docs/standards/ci-lanes.md:155` | `... ADR-0015 (flat crates) ...` |
| `docs/standards/testing.md:245` | `ADR-0003 (audit chain), ADR-0015 (flat crates).` |
| `docs/products/_TEMPLATE.md:56,315` | `Cite the flat-crates target prefix (e.g. \`crates/oya-foundry-*\`)` / `Every flat-crates target referenced exists in \`Cargo.toml\`` |
| `docs/products/foundry/PRD.md:1035`, `docs/products/cloud/PRD.md:807` | `Every flat-crates target referenced exists in \`Cargo.toml\` or planned roadmap` |
| `templates/checklists/pre-push.md:29` | `... if this is a flat-crates move PR (per ADR-0015), \`registry/migrations/2026-flat-crate-migration/\` entry added` |
| `templates/checklists/pre-merge.md:21,26` | `lists flat-crates targets touched` / `flat-crates-move → merge-queue serialization on root Cargo.toml` |
| `templates/checklists/vertical-onboarding.md:19` | `Vertical kernel flat-crates target reserved at \`crates/oya-vertical-<name>-kernel-*\`` |
| `docs/DESIGN.md:505`, `docs/specs/deep-dive-oyatie-sst-consolidation.md:17,52`, `docs/specs/deep-dive-trace-...:39` | `flat-crates ADR-0015 target` / `Cargo.toml ... flat-crates workspace, 140+ crates` |

registry/ — references the superseded model by name:

| path:line | snippet |
|---|---|
| `registry/bounded-contexts.json:6` | `... per BC per ADR-0015 flat-crates + ADR-0056 BNF v4.1` |
| `registry/artifact-capabilities-registry.json:577` | `Machine-readable BC registry per ADR-0015 flat-crates.` |
| `registry/stub-audit/2026-05-17/adrs.jsonl:15,28,29,48` | `crate:oya-governance-flat-crates` missing-crate refs (ADR-0008 L126, ADR-0015 L82/L158, ADR-0025 L91) |
| `registry/stub-audit/2026-05-17/missing-fitness-crates.json:106,899` | `"crate": "oya-governance-flat-crates"` |
| `registry/milestone-audit/index.json:804-805,2545` + `registry/stub-audit/2026-05-17/ips.jsonl:33` | `IP-002-flat-crates-guard` (P06 regional-pack-flattening IP) |
| `docs/MISTAKES-LEDGER.md:56` | MFL-0012 flat-crates active gate row (see §4) |
| `docs/CHANGELOG.md:438,441,453,454`, `docs/README.md:1558` | `flat-crates guard` / `flat-crates governance model` / `flat-crates-move-pr.md` runbook |
| `docs/ADR-CONSOLIDATION-PLAN.md:61`, `docs/ADR-LEGACY-REGRESSION-MAPPING.md:108` | legacy consolidation rows naming ADR-0015 flat-crates |

Naming-survives (type b) examples — NOT contradictions, listed for completeness (do NOT rewrite the name,
only the path when these become real on-disk paths): the ~650 `// crates/oya-<x>-kernel` code-comment
headers and `source_of_truth: crates/oya-...` rows across `docs/decisions/` (e.g. `ADR-0011:51,63,75,87`,
`ADR-0005:41,81`, `ADR-0020:33,93`, `ADR-0023:33`, `ADR-0024:33`, `ADR-0043:33,71,88`). These assert the
crate NAME, not that top-level `crates/` is the canonical root; under ADR-0512 the same crate sits at
`{oya,cloud}/<svc>/crates/<crate>/` keeping the identical `[package].name`.

---

## 6. Back-pointer / index integrity gaps (supersession not fully recorded)

- `docs/adr-archive/ADR-0509-hyperscaler-service-decomposition-pattern.md — `superseded_by: []` (EMPTY) and `:4` `status: Accepted`, despite
  ADR-0512:9-11,22 declaring it superseded. No back-pointer; status not flipped to Superseded.
- `docs/adr-archive/ADR-0357-vertical-slice-monorepo-nesting.md — has **no** `superseded_by` field at all (`grep -c superseded_by` → 0);
  still `status: Proposed`. ADR-0512:22 supersedes it but the file is unmarked.
- `docs/ADR-INDEX.md` — has **no row** for ADR-0509 or ADR-0512 (`grep -c 'ADR-0509|ADR-0512'` → 0).
  The canonical, founder-locked governing ADR (0512) is absent from the index; 0357 is still listed
  `Proposed` (`:317`) and 0015 still `accepted` with the full flat-crates title (`:37`).

---

## 7. NOT COVERED / caveats (no silent caps)

- The broad `microservices/` legacy-path footprint is large: **374 files** under
  `docs/decisions/ + docs/standards/ + specs/ + registry/` contain the string `microservices/`
  (`grep -rln 'microservices/'`, worktrees excluded). Most are legacy IaC/path EXAMPLES (e.g.
  `microservices/observability/iac/helm/...` at `ADR-0193:218`, `ADR-0192:231`) — superseded by
  ADR-0131/0512's `{oya,cloud}/<service>/iac/`, but they are iac-path drift, a SUPERSET of the strict
  flat-`crates/`-topology lane. I enumerated the high-signal `microservices/<ms>/crates` topology hits
  in §5; I did NOT line-enumerate all 374 `microservices/` files (out of lane scope; flagged for the
  microservices-root contradiction map).
- The ~650 type-(b) NAMING refs in `docs/decisions/` were sampled, not exhaustively line-listed — by the
  §2 verdict they are not contradictions. If a future pass treats path-rewrite of crate-comment headers
  as in-scope, that set must be re-enumerated.
- I scanned the canonical trees only; `.claude/worktrees/**` (185+ stale `ci-lanes.md` clones etc.) were
  deliberately excluded — they are throwaway agent worktrees, not SSOT.
- `docs/machine-readable/decisions.json` was NOT trusted/used per instruction (known-drifted).

---

## 8. Counts

- Supersession chain ADRs verified: 5 (0015, 0131, 0357, 0509, 0512).
- Live tree: NEW roots present = 3 (`oya/`, `cloud/`, `libs/`); legacy `microservices/` = absent;
  stale top-level `crates/` dir = PRESENT, untracked, code-empty (3 `.DS_Store`, 2 empty `oya-*` dirs) → DELETE.
- Live flat-crates GATE enforcing superseded depth-3 topology: 1 lane spec
  (`docs/governance-lanes/flat-crates.md`, BLOCKER) + claimed in 3 doc lists
  (governance-lanes/INDEX:28, AGENTS:231, done-definition-checklist:33); ABSENT from machine registry
  `registry/quality/lanes.yaml` (0 matches).
- Stale flat top-level `crates/` LOCATION / `microservices/<ms>/crates` / `flat-crates target` assertions
  enumerated in §5: ~50 distinct `path:line` references across decisions (incl. 5 seed corners + ADR-0015
  self + ADR-0357 dead-path×4), 5 specs/, ~25 standards/design/roadmap/checklists/products, ~14 registry/.
- Integrity gaps: ADR-0509 `superseded_by:[]` + status Accepted; ADR-0357 no `superseded_by` + status
  Proposed; ADR-INDEX missing 0509 + 0512 rows.
- Verdict: **LOCATION superseded/forbidden (ADR-0512); NAMING `oya-<ctx>-<role>` survives (ADR-0056/0131/0512).**
