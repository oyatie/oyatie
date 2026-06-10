# 10 — TOTAL ACCOUNTING LEDGER (justify-account-robustness lane)

**Charter lens:** D-DOCTRINE §"Total accounting" (`decision-record-oyatie-canon.md:180`, verbatim): *"every file, doc, folder accounted-for AND justified: owner + justification (→ decision/need) + reachability (→ masterplan) + staleness policy (TTL). Unaccounted/unjustified ⇒ blocks or auto-archives. Generated-not-hand-maintained."* Plus the masterplan-SSOT-reachability principle: **worth-documenting ⇒ worth-reading ⇒ must be reachable from the masterplan/workflow; else archive.**

**Scope of this lane:** every TOP-LEVEL tree of `/Users/jasonlee/Developer/source` + the `docs/` and `specs/` subdir cluster map. READ-ONLY. All counts read live from disk 2026-06-06. EXTENDS `backlog-reconciliation/00-BACKLOG-RECONCILIATION.md` §T-STRUCT / §1B-C (sprawl) and the `01-ADR-DISPOSITION-TABLE.md`; does not re-derive verified tree counts but re-checks them with the accounting (owner/justify/reach/TTL) lens.

**Two trees, do not conflate:** `/Users/jasonlee/Developer/source` = the monorepo being audited (subject). `/Users/jasonlee/Developer/linux/docs/audit/...` = where this audit's artifacts live (output). The task body names both `source` (subject of the ledger) and `linux/docs/...` (artifact home); they are distinct.

---

## 0. METHODOLOGY + COVERAGE (no silent caps)

**Fully walked (100%):** all 27 top-level entries of `source/` enumerated by `ls`; git-tracked-file counts (`git ls-files`, excluding `/target/`) computed for every one; reachability tested for every top-level dir-name against the four registries (`specs/masterplan.json`, `specs/root-hub-pointers.json`, `docs/DOC-CATALOG.md`, `docs/MASTERPLAN.md`) via grep hit-counts; OWNERS/CODEOWNERS swept tree-wide.

**Fully walked:** `docs/` (87 entries `ls`-enumerated; cluster homes cross-checked for duplication) and `specs/` (106 entries `ls`-enumerated; `.json` vs `.md` split confirmed).

**Sampled / bounded (stated explicitly):**
- Owner-frontmatter coverage measured by grep over `docs/**/*.md` (`^owners:` = 96 of 2358 files); I did NOT open all 2358 to confirm each is a real frontmatter block vs body text — I verified the pattern is a genuine YAML frontmatter key on 3 sampled files. The 96 figure is a lower-bound floor on owner-tagged docs.
- The `ttl|staleness|expires` grep returned 869 hits; this is substring noise (matches body prose), NOT 869 enforced TTLs — I did NOT treat it as real TTL coverage. The only verified TTL primitive is `_sunset`/`sunset_at` in machine-readable JSON specs (confirmed in `markdown-retirement-policy.json`).
- `services/` husk: walked git-tracked files (0 excl `target/`) + on-disk `find`; did NOT enumerate every file under the untracked `services/ci-webhook-gateway/target/` build cache (it is build output, not source).
- I did NOT recurse into every crate under `oya/` (14,649 tracked files) / `cloud/` (1,771) / `libs/` (586) — per-crate accounting is the job of `registry/catalog/*.yaml` (the existing generated code-record). I accounted for these trees at the TREE level (owner = workspace/axis; reachability = `Cargo.toml` members) which is the correct granularity for a top-level accounting ledger.

---

## 1. ACCOUNTING LEDGER — every top-level tree of `source/`

Columns: **tracked** = git-tracked files excl `/target/`. **Owner** = who owns it (no `OWNERS` files exist anywhere — see §3 finding O-1 — so "owner" = the de-facto team/registry/axis that governs it). **Justification** = the decision/need it traces to. **Reachable?** = appears in a canonical registry (M=masterplan.json, RH=root-hub-pointers.json, DC=DOC-CATALOG, MP=MASTERPLAN.md, CT=Cargo.toml workspace members). **Verdict** = KEEP / ARCHIVE / MERGE / NEEDS-OWNER.

| # | path | tracked | de-facto owner | justification → decision/need | reachable? | verdict |
|---|---|---|---|---|---|---|
| 1 | `oya/` | 14649 | council-architecture / per-axis | D-PURESPLIT (`:172`) canonical PRODUCTS tree; ADR-0131/0512 flat-layout | **CT only** (433 members); M/RH/DC/MP = 0 | **KEEP** (but reachability gap R-1) |
| 2 | `cloud/` | 1771 | council-architecture / platform | D-PURESPLIT canonical PLATFORM tree (dogfood substrate, D-LAYER) | **CT only** (100 members); MP=1 mention | **KEEP** (R-1) |
| 3 | `libs/` | 586 | per-lib axis | shared `libs/<lib>/` is the ONLY allowed shared home (`00-BACKLOG-RECON:110`) | **CT only** (168 members); M/RH/DC/MP=0 | **KEEP** (R-1) |
| 4 | `crates/` | **0** | — | top-level flat `crates/` is ADR-0512-FORBIDDEN; both entries (`oya-application-app`, `oya-audit-chain-emission-api`) contain ONLY `.DS_Store`; the REAL crates live at `oya/.../crates/...` (Cargo.toml line 181) | NOT a member; not tracked | **ARCHIVE/DELETE** (orphan husk; T-STRUCT action a) |
| 5 | `services/` | **0** | — | D-PURESPLIT ERADICATE husk; 0 git-tracked files; subdirs hold only `.DS_Store` + one untracked `ci-webhook-gateway/target/` build cache | none | **ARCHIVE/DELETE** (sprawl husk; T-STRUCT a) |
| 6 | `platforms/` | **1** | — | single untracked-shape `BUCK` (Buck2 host execution-platform def, `host_configuration`); D-PURESPLIT ERADICATE; the 1 file is a build-config, not a service | none | **MERGE** the BUCK into root/buck config, then DELETE dir (sprawl husk; T-STRUCT a) |
| 7 | `test-results/` | **0** | — | local test-run output (`.last-run.json` + `cloud-iac-module-archives/`); 0 git-tracked; build/run artifact | none | **ARCHIVE/DELETE** (gitignore; not source) |
| 8 | `docs/` | 2880 | council-architecture | doc corpus; markdown-retirement target | **RH=19, DC=14, M=8, MP=3** | **KEEP** (well-reached; but internal dup — §2) |
| 9 | `specs/` | 202 | council-architecture | machine-readable SSOT (95 `.json`); masterplan + contracts home | **RH=149, DC=30, M=27, MP=10** | **KEEP** (most-reached tree; canonical) |
| 10 | `registry/` | 1057 | council-architecture | generated registries (catalog/, adr/, data-class/, bounded-contexts) | **RH=25, DC=10, M=2** | **KEEP** (the generated-registry home — candidate for accounting-model, §5) |
| 11 | `evidence/` | 1532 | per-lane | evidence bundles for gate/claim verification (robust-not-false) | **MP=14, M=3, RH=3** | **KEEP** but **NEEDS-OWNER+TTL** (1532 files, many `aborted-persona-*`/dated run snapshots = staleness-prone) |
| 12 | `tasks/` | 120 | per-lane | `*-plan.md` lane plans (119 md + 1 json) | M=2 only | **MERGE** with `docs/specs/` — see DUP-1 (110 exact-slug overlap) |
| 13 | `contracts/` | 87 | api-platform | API SSOT (`.proto`/openapi/asyncapi); api-contract-ssot-canonical | **DC=5, MP=4** | **KEEP** |
| 14 | `infra/` | 88 | platform/SRE | IaC + GitOps (capi, cilium, kms, gitops, forge) | MP=2 only | **KEEP** but **NEEDS-OWNER**; reachability weak (R-2) |
| 15 | `tools/` | 258 | dev-tooling | dev CLIs + vendored `agent-skills` subtree | MP=2 only | **KEEP** but **NEEDS-OWNER** (R-2) |
| 16 | `scripts/` | 37 | dev-tooling | helper scripts; tension with Q "pure-Rust, no new .sh/.py" gate | **0 anywhere** | **NEEDS-OWNER** + justify vs pillar-Q (UNACCOUNTED) |
| 17 | `benchmarks/` | 8 | per-service perf | perf budgets (`docs/performance-budgets/` twin) | **0 anywhere** | **NEEDS-OWNER** (UNACCOUNTED; tie to perf-budget SLO) |
| 18 | `templates/` | 30 | council-architecture | doc/code templates | RH=2, DC=1 | **MERGE** with `docs/templates/` — DUP-2 (13 identical basenames) |
| 19 | `packs/` | 60 | localization axis | compliance/localization packs (au/br/cn/eu) | **0 anywhere** | **MERGE** — pack sprawl DUP-3 (4 homes) |
| 20 | `regional-packs/` | 6 | localization axis | regional overlays (eu/jp/kr/ksa) | RH=3 | **MERGE** — pack sprawl DUP-3 |
| 21 | `plan/` | 4 | council-architecture | 4 hyperscaler-greenfield planning `.md` | **M=4, MP=5** | **KEEP** but staleness-prone (large hand-md plans; reconcile vs masterplan) |
| 22 | `tests/` | 8 | qa | `cross-microservice/` scenario `.md` (NOT Rust tests) | MP=1 | **KEEP** but **NEEDS-OWNER**; name misleads (no code) |
| 23 | `third-party/` | 68 | dependency-governance | vendored deps + `fixups/` (reindeer) | **0 anywhere** | **KEEP** (vendor; justify via deny.toml/reindeer.toml) but UNACCOUNTED in registries |
| 24 | `toolchains/` | 1 | dev-tooling | Buck2 toolchain `BUCK` def | **0 anywhere** | **KEEP** (build-config; tiny) |
| 25 | `bin/` | 1 | dev-tooling | single `oya` launcher script | **RH=1** | **KEEP** (but tension w/ #20 automation-ratchet "forbid new `oya` CLI") |
| 26 | `memory/` | 2 | — | 2 dated `feedback_oya_*` md notes (May 2026) | **0 anywhere** | **ARCHIVE** (stale agent-feedback scratch; ORPHAN) |
| 27 | `benchmarks`…covered. **Root loose files** (`goal.json`, `ADR-INVENTORY.tsv`, `Jenkinsfile`, `Dockerfile.distroless`) | n/a | mixed | `Jenkinsfile` = UNRATIFIED bridge (T-CI R6); `ADR-INVENTORY.tsv` 265KB hand-file vs generated `registry/adr/` | mixed | **NEEDS-OWNER** (`Jenkinsfile` flag; `ADR-INVENTORY.tsv` likely DUP of generated index) |

---

## 2. DOCS/ + SPECS/ CLUSTER MAP (sub-tree accounting)

`docs/` (87 entries) is the largest doc surface. Reachability into it is strong at the top (RH=19) but the *internal* clusters duplicate the top-level trees and each other:

| docs/ cluster | concern | duplicate-of / collision | verdict |
|---|---|---|---|
| `docs/specs/` (116 `.md`, 110 `task-*`) | lane task-specs | **DUP-1: 110 exact-slug overlap with top-level `tasks/`** (`task-X.md` ↔ `X-plan.md`) AND name-collides `/specs` (which is `.json` machine-specs) | **MERGE into `tasks/`; rename to avoid `/specs` collision** |
| `docs/templates/` (34) | templates | **DUP-2: 13 identical basenames with top-level `templates/`** (`adr-template.md`, `design-doc-template.md`, `capability-record-template.yaml`…), plus `-v2` split-brain variants | **MERGE → one templates home** |
| `docs/products/` | product docs | overlaps `specs/products/` (RETIREMENT.md) + product-axis catalog | **MERGE/clarify** |
| `docs/regional-packs/` + `docs/localization-packs/` | localization docs | **DUP-3: pack sprawl** — 4 homes total (`packs/`, `regional-packs/`, `docs/regional-packs/`, `docs/localization-packs/`) | **MERGE → one pack taxonomy (D-KR trichotomy)** |
| `docs/machine-readable/` (`catalog.json`) | generated doc-catalog mirror | the JSON SSOT for `DOC-CATALOG.md` | **KEEP** (this is the right generated pattern) |
| `docs/decisions/` (354) | ADRs (SSOT) | accounted by `01-ADR-DISPOSITION-TABLE.md`; dup-0377 known | **KEEP** (see ADR lane) |
| `docs/governance-lanes/` (67), `docs/standards/` (105), `docs/runbooks/` (172), `docs/personas/` (133), `docs/user-journeys/` (190) | governance/standards/ops corpora | high-volume, low owner-frontmatter coverage | **KEEP but NEEDS-OWNER+TTL** (staleness-prone bulk) |

`specs/` (106 entries, 95 `.json` + subdirs `capabilities/ catalog/ design-system/ fixtures/ microservices/ openslo/ policy/ products/ proto/`): this is the **most-reachable, most-canonical tree** (RH=149). `specs/microservices/` (31) is a name-residue of the pre-pure-split era — flag for rename vs D-PURESPLIT, though it is spec-data not a service tree. `specs/proto/` vs top-level `contracts/*.proto` = potential proto split (two proto homes) — flag NEEDS-RECONCILE.

---

## 3. HEADLINE FINDINGS (orphans / unaccounted / unjustified / duplicates / sprawl)

**O-1 — ZERO OWNERS files tree-wide (systemic accounting failure).** `find -iname OWNERS` = **0** anywhere (excl `.git`/`target`/`third-party`). The only ownership signals are `.github/CODEOWNERS` (1) + two advisory "codeowners-mirror.md" docs (`docs/governance-lanes/`, `.omc/fitness-lanes/`) + sparse `owners:`/`owner_team:` frontmatter (96/2358 docs = **~4% of docs**, near-0% of code). Hyperscaler doctrine (Google/Meta) mandates `OWNERS` at every package boundary. **The "owner" half of total-accounting is essentially un-enforced.** This is the single largest D-DOCTRINE gap.

**O-2 — Canonical CODE trees are NOT reachable from the masterplan registries.** `oya/`/`cloud/`/`libs/` have **0** hits in `masterplan.json` and `root-hub-pointers.json`; they are reachable ONLY via `Cargo.toml` workspace members (433/100/168). The masterplan SSOT enumerates DOC/SPEC trees but not the code it governs. Per the reachability principle this is a **reachability GAP (R-1)**, not an archive call (the code is obviously load-bearing) — the fix is to make `Cargo.toml` (or a generated code-tree manifest) a *declared companion registry* of the masterplan so code becomes reachable-by-construction.

**ORPHAN husks (0 git-tracked, no decision, not reachable) → ARCHIVE/DELETE:** `crates/` (only `.DS_Store`), `services/` (0 tracked; only `.DS_Store` + an untracked build-cache), `test-results/` (run output), `memory/` (2 stale dated feedback notes). `platforms/` (1 untracked BUCK) → MERGE-then-delete. These are exactly the D-PURESPLIT sprawl husks (`services/`, `platforms/`, flat `crates/`) the founder ruled ERADICATE; this lane independently re-confirms them with git-tracking evidence.

**UNACCOUNTED (exists, real content, in NO registry):** `scripts/` (37; also collides with pillar-Q "no new .sh/.py"), `benchmarks/` (8), `third-party/` (68), `packs/` (60), `toolchains/` (1), `memory/` (2). None appear in masterplan.json/root-hub/DOC-CATALOG/MASTERPLAN.md. `third-party/` is justifiable (vendor, governed by `deny.toml`/`reindeer.toml`) but should be *declared* as vendored-excluded, not silently absent.

**UNJUSTIFIED / tension:** `bin/oya` + `scripts/` sit against register #20 automation-ratchet (forbid new `oya` CLI / no new .sh) and pillar-Q. `Jenkinsfile` at root = unratified CI bridge (already flagged T-CI R6). `ADR-INVENTORY.tsv` (265KB hand-maintained) likely duplicates the generated `registry/adr/` index → violates generated-not-hand-maintained.

**DUPLICATES (same concern, two+ homes):**
- **DUP-1:** `docs/specs/` (110 `task-*.md`) ⟷ `tasks/` (110 `*-plan.md`) — **exact 110-slug overlap** → MERGE to one lane-task home (and rename to stop colliding with `/specs`).
- **DUP-2:** `templates/` ⟷ `docs/templates/` — **13 identical basenames** + `-v2` split-brain → MERGE.
- **DUP-3 (pack sprawl):** `packs/` + `regional-packs/` + `docs/regional-packs/` + `docs/localization-packs/` — **4 homes** for localization/regional packs → MERGE to one D-KR pack trichotomy.
- Secondary: `docs/products/` ⟷ `specs/products/`; `contracts/*.proto` ⟷ `specs/proto/`.

**SPRAWL (husks, per task definition):** `services/` (5 husk subdirs, 0 tracked), `platforms/` (husk), flat `crates/` (husk) — all three named husks present and empty-of-source, exactly as T-STRUCT predicted.

**STALENESS-PRONE (no TTL, dated/bulk, no owner):** `evidence/` (1532; `aborted-persona-*` + dated run snapshots), `plan/` (4 large hand-md), `memory/` (stale), the bulk doc corpora (`runbooks/` 172, `personas/` 133, `user-journeys/` 190 with ~4% owner coverage). Only TTL primitive in the repo is `_sunset.sunset_at` in machine JSON specs — there is **no TTL on any markdown corpus or on the husk dirs**, so nothing auto-archives them; they accumulate (the ai-slop pileup that task #14 anticipates).

---

## 4. WHAT THE PRIOR AUDIT ALREADY HAD vs WHAT THIS LANE ADDS

- `00-BACKLOG-RECONCILIATION.md` §T-STRUCT already ruled `services/`+`platforms/`+flat-`crates/` = sprawl to eradicate (verified tree counts oya=87/cloud=25/services=5-husks). **This lane EXTENDS** by: (a) proving with `git ls-files` that the husks have **0 tracked source** (stronger than "no real crates"); (b) the **O-1 zero-OWNERS systemic finding** (not in prior work); (c) the **O-2 code-trees-unreachable-from-masterplan** reachability gap; (d) **DUP-1/2/3** doc-cluster duplications (new); (e) the explicit **accounting-model schema** (§5).

---

## 5. PROPOSED ACCOUNTING-MODEL SCHEMA (generated, not hand-maintained)

The repo already has the *fragments*: `registry/catalog/*.yaml` (per-crate records), `docs/machine-readable/catalog.json` (per-doc fields: owner_team/update_trigger/cadence/dependent_docs/validation_check), `markdown-retirement-policy.json._sunset` (TTL primitive). **No single registry unifies them across ALL top-level paths** — that is the D-DOCTRINE gap. Proposed: ONE generated `registry/accounting-ledger.json` (regenerated in CI from the trees themselves, validated by a blocking gate with RED/GREEN fixtures), every accounted path carrying:

| field | meaning | source-of-truth (generated from) |
|---|---|---|
| `path` | repo-relative path of the accounted unit (tree / dir / doc-cluster / crate) | filesystem walk |
| `unit_class` | enum: `code-tree` \| `doc-cluster` \| `spec` \| `registry` \| `evidence` \| `vendor` \| `build-config` \| `husk` | classifier |
| `owner` | OWNERS-file team id (REQUIRED; gate fails if absent) | nearest `OWNERS` (must be created — fixes O-1) |
| `justification_ref` | decision/need it traces to (ADR id / D-ruling / spec id) | ADR frontmatter / decision-record |
| `reachable_from` | list of registries that point to it (must be non-empty) | masterplan.json / root-hub / Cargo.toml / DOC-CATALOG |
| `reachability_ok` | bool: `reachable_from` non-empty (gate blocks if false) | derived |
| `ttl_policy` | `sunset_at` / `review_by` / `permanent` / `none` | `_sunset` or per-class default |
| `staleness_status` | derived: `fresh` \| `due` \| `stale` (vs ttl + git mtime) | git log + ttl |
| `tracked` | git-tracked source-file count (excl target) | `git ls-files` |
| `verdict` | `KEEP` \| `ARCHIVE` \| `MERGE` \| `NEEDS-OWNER` (auto-derived; husk+0-tracked ⇒ ARCHIVE) | rules |
| `dup_of` | if duplicate, the canonical home it must merge into | dedup index |

**Enforcement (robust-not-false):** the gate must BLOCK merge when any path has empty `reachable_from`, missing `owner`, or `verdict=ARCHIVE`-but-still-present — proven by RED fixtures (a husk dir / an owner-less crate / an unreachable tree must each turn the gate RED) and GREEN fixtures (fully-accounted path passes). This is the generated-not-hand-maintained, actually-blocking model D-DOCTRINE demands; it directly closes O-1 (owner), O-2 (reachability), and the DUP/SPRAWL/STALENESS findings.

---

## 6. EVIDENCE INDEX (path : finding)
- `decision-record-oyatie-canon.md:180` — D-DOCTRINE total-accounting clause (charter).
- `find -iname OWNERS` = 0 tree-wide; only `.github/CODEOWNERS` + 2 mirror docs (O-1).
- `Cargo.toml:181` `"oya/application/crates/oya-application-app"` = real home; top-level `crates/oya-application-app/` holds only `.DS_Store` (orphan, finding #4).
- `git ls-files services/ | grep -vc /target/` = **0**; `crates/`=0; `test-results/`=0 (orphan husks).
- `platforms/BUCK` = Buck2 `execution_platform` host-config (1 untracked file).
- masterplan/root-hub reachability scan: `oya/cloud/libs` = 0 in masterplan.json+root-hub (O-2/R-1); `specs/`=RH149/M27 (canonical); `scripts/benchmarks/packs/third-party/toolchains/memory` = 0 everywhere (UNACCOUNTED).
- `comm -12` docs/specs ↔ tasks = **110 exact slugs** (DUP-1); `templates ∩ docs/templates` = **13 basenames** (DUP-2); 4 pack homes (DUP-3).
- `markdown-retirement-policy.json._sunset.sunset_at:2026-08-31` = only real TTL primitive; `docs/**/*.md` `^owners:` = 96/2358 (~4%).
- `registry/catalog/*.yaml` + `docs/machine-readable/catalog.json` = existing generated-record fragments to unify (§5).
