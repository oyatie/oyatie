# 00 — Governance-File Bootstrap Bundle (WIP pre-lane 0.7)

**Lane:** pre-lane 0.7 (governance-file bootstrap) · **Mode:** READ-ONLY on `source`; WRITE only template artifacts under `prelane-0.7/` · **Date:** 2026-06-06
**Goal:** Produce reproducible per-PR governance scaffolding that `source`'s live merge-gate reads, so each consolidation-lane PR into `jason931225/oyatie` (the migrated `linux/stack` monorepo) is reproducible and gate-passable.

> **Trust boundary:** every quoted path below was read read-only from `/Users/jasonlee/Developer/source`. Nothing in `source` was modified. Each scaffolding block is grounded in the real source path it came from; honest gaps are marked **GAP**.

---

## 0 — Enforcement reality (ENFORCED vs ASPIRATIONAL) — read this first

The single most important finding: the governance prose names enforcers that **do not exist on disk**. What actually runs differs from what the docs describe. Per-lane PRs must satisfy the **real** gate, and should *also* produce the documented artifacts (they are cheap and the named enforcers may be armed later).

| Requirement | Documented enforcer (named in prose) | On-disk reality | Status |
|---|---|---|---|
| 5-H2 PR body + `## Code Review` | `traceability-validator` + `oya-pr-review` + `scripts/hooks/guard-pr-merge-review.mjs` | No `.mjs` named `guard-pr-merge-review`; no crate/lane named `traceability-validator`. `oya-pr-review` exists only as a **required-check context-name string** (branch-protection/commit-status), governed by `aspirational_enforcement_gate.rs` + posted by `ci-webhook-gateway` — not a 5-H2 parser. See gap #2. | **ASPIRATIONAL** (referenced; no validator parses the H2s) |
| PR traceability (the *real* lane) | `oya-governance-pr-traceability` → `cargo run -p oya-dev-cli -- gate validate pr-traceability` | **EXISTS**: `libs/oya-governance-pr-traceability-kernel/src/lib.rs` + `libs/oya-check-pr-traceability/`. Checks **3 booleans only**: `cites_phase_id`, `cites_plan_or_adr`, `has_decision_log_row`. Has a `Warn`/`Block` ratchet. **Does NOT parse the 5 H2 headings or `## Code Review` text.** | **ENFORCED** (but weaker than prose) |
| Multispectrum evidence | `oya-check-dependency-seam` sub-check `multispectrum-evidence-attached` | **EXISTS**: `libs/oya-check-dependency-seam/`; `gate validate dependency-seam` is a real lane. Seam sub-checks are `severity_day_0: report-only`, `severity_day_8: error` (one-week soak per ADR-0092). | **ENFORCED** (report-only → error soak) |
| ADR shape | `oya-governance-adr-shape` + `scripts/validate-adr-shape.mjs` | **EXISTS**: `scripts/validate-adr-shape.mjs` delegates to `cargo run -p oya-dev-cli -- lint adr-shape`. | **ENFORCED** |
| DOC-CATALOG row coverage | `oya-governance-doc-catalog` → `gate validate doc-catalog` | **EXISTS** as a lane. **Path-drift flag:** the doc-catalog lane spec reads `docs/CATALOG.md`, but the real file is `docs/DOC-CATALOG.md` (`docs/CATALOG.md` does not exist) — see pre-lane 0.5 §6(b). | **ENFORCED** (with known path bug) |
| CHANGELOG row | `oya-governance-changelog-row` (D18) | Named in `done-definition-checklist.md` / AGENTS.md D18; **not** in the 96-lane `lanes.yaml` `gate validate` roster. | **ASPIRATIONAL** (checklist-only) |
| Local pre-push gate | `scripts/hooks/pre-push.sh` | **EXISTS** and is the one real local hook: `exec oya verify --ci-required --include-deferred` (falls back to `cargo run -q -p oya-dev-cli -- verify …`). | **ENFORCED** (the actual aggregate gate) |

**Net:** the real machine floor a consolidation PR must clear is **`oya verify --ci-required --include-deferred`** (the pre-push shim → `oya-dev-cli verify` → `gate run-all`), plus the per-PR lanes in `lanes.yaml` (96 lanes; `gate validate <name>`). The 5-H2 / `## Code Review` / D18-changelog shape is **documented doctrine** that the *named* validators don't yet enforce — produce them anyway (the seam lane reads the multispectrum evidence, and the soak window arms the rest).

**Source basis:** `docs/AGENTS.md:68-75,211-244`; `templates/checklists/done-definition-checklist.md`; `docs/DOC-UPDATE-PROTOCOL.md`; `docs/templates/pull-request-template-v2.md`; `registry/quality/lanes.yaml` (96 lanes); `scripts/hooks/pre-push.sh`; `scripts/validate-adr-shape.mjs`; `libs/oya-governance-pr-traceability-kernel/src/lib.rs`; `libs/oya-check-dependency-seam/`. Repo-wide `find`/`grep` for `guard-pr-merge-review`, `traceability-validator`, `oya-pr-review` returned **no implementation**.

---

## 1 — The 5-H2 PR-body template (fill-in-the-blanks)

**Source:** `docs/templates/pull-request-template-v2.md` (TPL-PR, Accepted 2026-05-12) — verbatim section set; also stated in `docs/AGENTS.md:211-219` and `docs/DOC-UPDATE-PROTOCOL.md` step 18.
**Shape:** 5 author-owned H2 sections + 1 lead-only `## Code Review` H2 added at merge. RFC-2119 active.

```markdown
## Issue

Closes #<n>   <!-- or `Refs #<n>` if not closing. -->
Change class: <feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update>
<!-- consolidation-lane default: `migration` (crate moves) or `docs` (ADR/catalog amendments). Change class MUST be on this line. -->

## Summary

- <1-3 bullets: WHAT + WHY. The diff shows the what; this adds the why.>
- Canonical authority read first (AGENTS.md §Pre-flight item 2): <docs/… path(s)>

## Verification

<!-- Each line MUST carry a PASS|FAIL token + actual command-output excerpt. -->
- `cargo nextest run --workspace --all-features --no-fail-fast` — <PASS|FAIL> — <excerpt>
- `cargo clippy --workspace --all-features --all-targets -- -D warnings` — <PASS|FAIL> — <excerpt>
- `cargo deny check` — <PASS|FAIL> — <excerpt>
- `oya verify --ci-required --include-deferred` — <PASS|FAIL> — <excerpt>   <!-- the real pre-push gate -->
- `oya gate validate <lane>` (per-change-class fitness lanes) — <list lane + PASS|FAIL each>
- Per-change-class reviewer agent: <agent-name> — verdict <APPROVE | REQUEST CHANGES>

## Traceability

- Catalog records touched: <list under registry/catalog/ or docs/DOC-CATALOG.md row id(s)>
- Cross-axis contracts touched: <list under contracts/> (per docs/DESIGN.md §10)
- ADRs cited: <ADR-#### list>   <!-- legacy ADR-#### forbidden in active text per docs/ADR-CONSOLIDATION-PLAN.md -->
- MISTAKES-LEDGER row (if regression-class): MFL-NNNN
- Implementation Plan ID (if executing an IP): IP-NNN-<slug>
- Inventory ledger row (if migration-class): INV-NNNN (per ADR-0052)
<!-- pr-traceability lane (REAL) checks: phase-id cited? plan-or-ADR cited? decision-log row present? Make all three explicit here. -->

## Evidence

- Audit-chain emission ID: EVT-<topic>-<ulid> (per ADR-0003)
- Multispectrum evidence file: /evidence/multispectrum/<change_id>-<unix_ts>.json   <!-- see §3 -->
- Foundation-bypass (if any): <bypass-id> + renewal date
- Per-pack regulator-watch impact (if any): <oya-pack-XX.regulator list>
- SBOM / SLSA / Cosign (if shipping a binary): <path|ref> / <L1|L2|L3> / <digest>

<!-- merge-gate (ASPIRATIONAL): lead reviewer adds `## Code Review` below at merge. -->
## Code Review  _(lead-only — never added by the worker agent)_

- Reviewer agent: <rust-reviewer | doc-style-reviewer | security-reviewer | … per AGENTS.md §Per-change-class reviewer agents>
- Verdict: <APPROVE | REQUEST CHANGES>
- Resolved items: <list>
- Deferred items: <list with owners + follow-up issue refs>
- Linus good-taste audit row: <special cases eliminated | "none — no candidates">
```

**Note (worker vs lead):** the worker agent **MUST NOT** add `## Code Review`; only the lead reviewer agent signs it at merge. Adding it as a worker is flagged by the *documented* `guard-pr-merge-review.mjs` rule (`pull-request-template-v2.md` agent-instructions fence + `AGENTS.md:168`) — **but that hook is ASPIRATIONAL** (does not exist on disk), so on dev today this is a discipline convention, not a hard block.

---

## 2 — Done-Definition D1..D18 checklist (per-lane)

**Source:** `docs/AGENTS.md:221-244` (core D1-D18) + `templates/checklists/done-definition-checklist.md` (CHK-DONE, Accepted 2026-05-12; per-change-class additions). The two are intentional mirrors — AGENTS.md is the §source; the checklist extends with per-class rows. (Minor drift: AGENTS.md D7 is folded into the checklist's enumerated `oya-governance-{…}` lane set; checklist D12 = `oya verify --ci-required`, AGENTS.md D12 = "required cloud-ci/oya-ci context.")

Walk **all** D1-D18, then the **per-change-class** rows that apply. Each row carries a typed verification path (lane name / command / `(advisory)`).

### Core (every change class)

- [ ] **D1** All Pre-flight-checklist items checked. *Verify:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated same PR per `docs/DOC-CATALOG.md`. *Lane:* `oya-governance-doc-catalog`.
- [ ] **D3** New ADRs authored from `docs/templates/adr-template-v2.md`. *Lane:* `oya-governance-adr-shape` (real: `validate-adr-shape.mjs` → `oya-dev-cli lint adr-shape`).
- [ ] **D4** New runbooks authored from `runbook-template-v2.md`; discoverable in `docs/RUNBOOKS-INDEX.md`. *Lane:* `oya-governance-runbook-index-resolves`.
- [ ] **D5** New capabilities ship record + eval set + autonomy tier + audit topic + Cosign signing. *Lane:* `oya-governance-capability-publish`.
- [ ] **D6** New schemas carry `data_class` per field. *Lane:* `oya-governance-data-class` (real: `gate validate data-class`).
- [ ] **D7** Per-PR fitness lanes pass: `oya-governance-{license, data-class, cohesion, glossary, adr-citation, brand-residue, bypass, flat-crates, runbook-index-resolves, doc-catalog}`. *Verify:* CI status.
- [ ] **D8** Reviewer agent ran; verdict in `## Code Review`. *Lane (ASPIRATIONAL):* `guard-pr-merge-review.mjs` (does not exist on disk).
- [ ] **D9** `cargo nextest run --workspace --all-features --no-fail-fast` passes. *Verify:* output in `## Verification`.
- [ ] **D10** `cargo clippy --workspace --all-features --all-targets -- -D warnings` passes. *Verify:* output.
- [ ] **D11** `cargo deny check` passes. *Verify:* output.
- [ ] **D12** `oya verify --ci-required` passes (checklist) / required cloud-ci/oya-ci context (AGENTS.md). *Verify:* output. *(Real:* `pre-push.sh` → `oya verify --ci-required --include-deferred`.)
- [ ] **D13** Performance changes carry benchmark + ≥2 stress scenarios. *Lane:* `oya-governance-perf-evidence`.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Lane:* `oya-governance-schema-migration`.
- [ ] **D15** PR has 5 canonical H2s; `## Code Review` at merge. *Lane (ASPIRATIONAL):* `traceability-validator` (does not exist). *Real proxy:* `oya-governance-pr-traceability` (3-boolean check).
- [ ] **D16** Audit-chain emission `EVT-*` ID in `## Evidence`. *Lane:* `oya-governance-audit-emission`.
- [ ] **D17** `docs/MISTAKES-LEDGER.md` row added if mechanical prevention shipped. *Lane:* `oya-governance-mistakes-ledger-cite`.
- [ ] **D18** `docs/CHANGELOG.md` row added if canonical doc touched. *Lane (ASPIRATIONAL):* `oya-governance-changelog-row` (not in `lanes.yaml`); checklist-enforced only.

### Per-change-class additions (consolidation-lane-relevant subset)

**migration** (crate moves — the dominant consolidation class):
- [ ] Schema up + down + dry-run + per-tenant + per-cell rollback shipped. *Lane:* `oya-governance-schema-migration`. *(N/A for pure crate moves; mark not-applicable.)*
- [ ] Inventory row added per `templates/checklists/inventory-update-checklist.md` (ADR-0052). *Lane:* `oya-governance-inventory-tracker`.

**refactor** (pure crate relocate / rename, no semantic change):
- [ ] Public API surface unchanged. *Command:* `cargo public-api --diff`.
- [ ] `cargo-semver-checks` clean. *Command:* `cargo semver-checks check-release`.
- [ ] Linus good-taste audit row in `## Code Review`. *(advisory)*

**docs** (ADR / catalog / standard amendments):
- [ ] `docs/DOC-CATALOG.md` trigger event named in PR `## Issue`. *Lane:* `oya-governance-doc-catalog`.
- [ ] `doc-style-reviewer` agent verdict captured. *Lane (ASPIRATIONAL):* `guard-pr-merge-review.mjs`.

**ADR** (new/amended ADR):
- [ ] `adr-template-v2.md` shape complete (Context/Decision/Drivers/Alternatives/Why-chosen/Consequences/Follow-ups). *Lane:* `oya-governance-adr-shape`.
- [ ] `docs/ADR-INDEX.md` updated. *Lane:* `oya-governance-adr-citation`.

*(feature / bugfix / chore / capability / plugin / runbook / pack-update rows also exist in `done-definition-checklist.md` — read there if a lane is one of those classes.)*

### Loop-cancellation re-walk
Per `docs/AGENTS.md:270-272 §Long-running loop rule`: in any Ralph/autopilot/ultrawork/team loop, **MUST** re-walk every applicable row above against latest state before exiting. Loops **MUST NOT** exit silently. Cancel via `/oh-my-claudecode:cancel` only when (a) change complete + verified, OR (b) loop structurally blocked. If any row is unchecked, the change is **not "done."**

---

## 3 — Multispectrum evidence JSON template (the real schema)

**Canonical schema:** `/specs/multispectrum-review.json` — `EXE-MULTISPECTRUM-REVIEW` **v2.4.0** (`_meta.version`), owner `council-architecture`, `status: Accepted`. Human gateway: `docs/standards/multispectrum-review.md` (thin pointer). Pre-PR fill template: `templates/checklists/pre-pr-multispectrum.json` (TPL-PRE-PR-MULTISPECTRUM v1.0.0).
**Emit path:** `/evidence/multispectrum/<change_id>-<unix_ts>.json` (per `AGENTS.md:68`).
**Read by:** `oya-check-dependency-seam` sub-check `multispectrum-evidence-attached`.

### Required top-level fields (schema `evidence_schema.required`)
`change_class_id`, `git_sha`, `freshness_unix`, `facets`. (Real examples also carry `schema_version`, `change_id`, `timestamp_iso`, `agent_id`, `scope`, `summary`, `nonclaims`, `verification`, `known_gaps`, `verdict` — recommended, not schema-required.)

### Change-class enum (`change_classes`, closed CC-1..CC-7)
| id | meaning | mandatory_artifacts |
|---|---|---|
| CC-1 | Kernel-layer public API (pure types / port trait decls) | ADR cite + failing-fixture in adversarial test dir + consensus-debate synthesis at `/evidence/debate/<change_id>-synthesis.json` |
| CC-2 | Adapter / infrastructure crate (provider binding, framework glue) | request-body limit decl (if HTTP) + timeout policy decl (if I/O) |
| CC-3 | Application use-case orchestrator / domain business logic | — |
| CC-4 | Pure refactor, no semantic change (rename/move/reorg) | before/after symbol-set parity check (`cargo metadata` or grep) |
| CC-5 | Documentation / ADR / standard / non-code artifact | — |
| CC-6 | Generated code or vendored upstream | regeneration source pinned (git_sha + tool ver) + supply-chain scan result |
| CC-7 | Test / benchmark / fixture-only change | fixture pair (passing + failing) for new lane sub-checks |

> **Consolidation-lane mapping:** crate-move PRs → **CC-4** (pure refactor; supply the symbol-set parity artifact) when no public-API change, else **CC-1/CC-2** by layer. ADR/catalog amendment PRs → **CC-5**.

### Facet roster (21 total: 12 F-family allocated [10 active + F12 reserved], 2 M-family, 7+ A-family open)
F-family (`required_when`):
- `F1_linus` (always), `F2_hyperscaler` (always), `F3_adversarial` (always), `F4_ergonomic` (always), `F5_quality` (always), `F6_alternatives` (always), `F7_security` (always), `F9_compliance` (always)
- `F8_performance` (CC-1 OR CC-2 OR any hot-path), `F10_reversibility` (CC-1 OR CC-2 OR cross-µsvc boundary OR data-migration), `F11_observability` (CC-1 OR CC-2 adding I/O OR new HTTP route/state-machine/bg-worker), `F13_migration` (breaking public surface OR schema/migration file added OR dep major bump)
- `F12` intentionally reserved/omitted (closed cap = 10 active + 2 reserved = 12).

M-family (fire on `meta_review_triggered`: CC-1 OR new ADR/standard/spec OR breaking-API OR new µsvc/lane):
- `M1_challenge_assumption`, `M2_zoomed_out_fit`.

A-family (policy-adherence; OPEN cap, each new one = 1 ADR; fire on file-set predicates):
- `A1_naming` (new file/rename in /specs,/registry,/evidence,/templates,docs/decisions,crates,tools), `A2_documentation` (any docs/*.md or standard/ADR/PRD edit), `A3_structure` (new file/dir, or move/rename of durable homes), `A4_architecture` (new crate, or cross-layer Cargo.toml dep edit), `A5_dependency` (workspace.dependencies / new dep / version bump), `A6_schema` (new JSON spec/registry, `$defs`/`required[]` change, schema version bump), `A7_algorithm` (new algorithm/heuristic, complexity-class change).
> **Consolidation note:** crate-move PRs trip **A1+A3+A4+A5** (renames, new dirs, new crates, Cargo.toml dep edits) mechanically — those subagent reviews are required when the file-set predicate matches.

### Rigor (`depth_levels`)
- `deep` — must produce ≥1 finding OR explicit `null_finding_reason`; trigger questions answered.
- `scan` — must record `considered=true`; findings may be empty.
- `skip` — explicit not-applicable; requires `considered=false` AND `not_applicable_reason`.

### Per-class rigor matrix (the two consolidation-relevant classes)
**CC-5 (doc_only):** `F1=deep, F2=scan, F3=scan, F4=deep, F5=scan, F6=deep, F7=scan` (F1..F7 only).
**CC-1 (kernel public API):** `F1..F9 + M1 + M2 = deep` and `consensus_debate_required: true` (all 21 facets, full debate). CC-2/CC-3/CC-4/CC-6/CC-7 declare the F1..F7 baseline subset (read their `rigor_per_facet` block before emitting).

### facet_evidence object shape (`evidence_schema.definitions.facet_evidence`, `required: ["considered"]`)
`considered` (bool, required) · `not_applicable_reason` (str, required if `considered=false`) · `findings` (array) · `fixuptasks` (array of `F-<SHORT>-<n>` ids) · `null_finding_reason` (str, required if rigor=deep AND findings=[]).

### Verdict file (separate, per `verdict_schema`)
**Emit path:** `/evidence/multispectrum/<change_id>-verdict.json`. **Required:** `change_id`, `spec_version` (MUST be ≥`2.4.0`), `evaluated_at_unix`, `wave_tag`, `facet_verdicts[]`, `aggregate_verdict`, `promotion_gate_status`. Aggregate rules: **GREEN** = all required facets pass + no consensus/wave pending; **YELLOW** = any facet needs-work or synthesis pending, none failing; **RED** = any facet auto_checks=fail OR consensus escalated OR mandatory artifact missing.

### Fill-in template (CC-4 example — adapt `change_class_id` + facet set per class)

```json
{
  "schema_version": "multispectrum-review/v2.4.0",
  "change_id": "<e.g. lane-L3-move-talos-runtime-crate>",
  "change_class_id": "CC-4",
  "freshness_unix": 0,
  "timestamp_iso": "<ISO-8601 Z>",
  "git_sha": "<40-char HEAD SHA at emit time>",
  "agent_id": "<executor/session id>",
  "scope": ["<paths touched>"],
  "summary": "<one paragraph: what moved/changed and why>",
  "nonclaims": ["<explicit out-of-scope statements>"],
  "facets": {
    "F1_linus":       { "considered": true, "rigor": "deep", "verdict": "PASS", "findings": ["…"], "null_finding_reason": "" },
    "F2_hyperscaler": { "considered": true, "rigor": "scan", "verdict": "PASS", "findings": [] },
    "F3_adversarial": { "considered": true, "rigor": "scan", "verdict": "PASS", "findings": [] },
    "F4_ergonomic":   { "considered": true, "rigor": "deep", "verdict": "PASS", "findings": ["…"], "null_finding_reason": "" },
    "F5_quality":     { "considered": true, "rigor": "scan", "verdict": "PASS", "findings": [] },
    "F6_alternatives":{ "considered": true, "rigor": "deep", "verdict": "PASS", "alternatives": ["…"], "selected": "…", "rejected": ["…"] },
    "F7_security":    { "considered": true, "rigor": "scan", "verdict": "PASS", "findings": [] }
  },
  "verification": [
    { "command": "<cmd>", "status": "passed", "evidence": "<stdout excerpt>" }
  ],
  "known_gaps": ["<honest gaps>"],
  "verdict": "GREEN"
}
```
*(CC-4 mandatory artifact: also attach the before/after symbol-set parity check. CC-1 adds F8/F9/M1/M2 deep + the consensus-debate synthesis at `/evidence/debate/<change_id>-synthesis.json`. A-family facets [A1/A3/A4/A5] append as their file-set predicates trip — add them as additional keys under `facets`.)*

**Real reference example (read for format):** `/Users/jasonlee/Developer/source/evidence/multispectrum/hook-context-removal-1780274908.json` (a complete CC-2 instance with F1..F13 + verification + known_gaps + verdict).

---

## 4 — The `## Code Review` merge-gate expectation

**Documented contract** (`docs/AGENTS.md:151-168,219`; `pull-request-template-v2.md`):
- Each change class has a designated **reviewer agent** that runs on the PR and signs `## Code Review` at merge.
- The `## Code Review` section **MUST** contain: **(a) reviewer-agent name, (b) verdict (`APPROVE` | `REQUEST CHANGES`), (c) resolved items, (d) deferred items.** Without this section the documented merge gate refuses.
- Named enforcer: `scripts/hooks/guard-pr-merge-review.mjs`, claimed as `PreToolUse` on `Bash`.
- Worker agents **MUST NOT** add `## Code Review`; only the lead reviewer agent at merge.

**Reality (GAP):** `scripts/hooks/guard-pr-merge-review.mjs` **does not exist** in `source` (only `scripts/hooks/pre-push.sh` is present). No `PreToolUse` Bash hook enforcing `## Code Review` was found. **So `## Code Review` is ASPIRATIONAL on dev today.** Produce the section by convention; do not expect a hard mechanical block from the named hook. The closest *real* enforcer is `oya-governance-pr-traceability` (3-boolean phase/plan-ADR/decision-log check), which does **not** inspect `## Code Review`.

---

## 5 — DOC-CATALOG.md + CHANGELOG.md row formats

> **Location (per pre-lane 0.5 §6b): `docs/`, NOT root.** Canon files are `/Users/jasonlee/Developer/source/docs/DOC-CATALOG.md` and `/Users/jasonlee/Developer/source/docs/CHANGELOG.md`. **No root-level copies exist.** Amendment lanes add rows INTO these `docs/`-rooted files. In the migrated `linux/stack` monorepo, the equivalent canon homes are `docs/DOC-CATALOG.md` + `docs/CHANGELOG.md` (mirror the source convention; do **not** create root copies).
> **Known path-drift (0.5 §6b, carried not fixed):** the `doc-catalog` fitness-lane spec reads `docs/CATALOG.md`, but the real file is `docs/DOC-CATALOG.md`. Reconcile in the conformance/amendment lane.

### DOC-CATALOG row (8 columns)
**Source:** `docs/DOC-CATALOG.md §0` (reading guide) + §2 catalog tables. Machine mirror: `docs/machine-readable/catalog.json`.

```
| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
```
- `id` — stable doc identifier (e.g. `doc.masterplan`, `doc.adr_0513`).
- `path` — file path (rooted at `docs/`; specs use `/specs/…`).
- `owner_team` — team-charter ID matching `docs/teams/<team-id>/CHARTER.md`.
- `update_trigger` — obligating event (an `EVT-*` from §1, e.g. `EVT-ADR-AUTHORED`, `EVT-ADR-PROMOTED`, `EVT-FLAT-CRATE-MOVED`).
- `update_cadence` — latest refresh schedule absent triggers (e.g. `per event`, `quarterly`).
- `dependent_docs` — docs that MUST be re-read/re-authored when this changes (`-` if none).
- `validation_check` — the CI/agent check that must pass (e.g. `doc-catalog-self-coverage`, `authority-cohesion`, `spec-contract-mirror`).
- `agent_authoring_allowed` — `YES` | `NO` (Tier-1 docs are `NO`: agents may DRAFT, named owner approves).

**Real example row (Tier 1):**
```
| `doc.masterplan` | `MASTERPLAN.md` | `council-architecture` | master-plan authority or sequencing change | per change + quarterly | PRD.md, DESIGN.md, ROADMAP.md, RACI-OWNERSHIP.md, RISK-REGISTER.md | `master-plan-completion`, `doc-catalog-self-coverage` | NO |
```

### CHANGELOG entry (date-grouped H2 + bullets, NOT a pipe-table)
**Source:** `docs/CHANGELOG.md` (observed format) + `docs/DOC-UPDATE-PROTOCOL.md` step 8.
- **DOC-UPDATE-PROTOCOL canonical inline form (step 8):** `<doc.id> <iso-date> <author> <one-line summary>`.
- **Observed live format in `docs/CHANGELOG.md`:** an `## <YYYY-MM-DD> — <title>` H2 header followed by `-` bullets describing the change and citing the evidence path. Example:

```markdown
## 2026-05-20 — ADR-0320 transient program identity doctrine authored

- Added ADR-0320 for <subject>; <key decision summary>.
- Captured multispectrum evidence at `evidence/multispectrum/adr-0320-transient-identity-1779293714.json`.
```
> Reconcile both forms: the prose step-8 one-liner is the *protocol* statement; the live file uses dated H2 + bullets + evidence-path citation. For a consolidation PR, append a dated H2 block whose bullets name the moved crates / amended ADR and cite the multispectrum evidence file.

**D18 condition:** add a CHANGELOG row **only if the change touches a canonical doc** (AGENTS.md D18). Pure crate moves that touch no `docs/` canon do not require a CHANGELOG row; ADR/catalog amendments do.

---

## 6 — ADR-shape front-matter template

**Source:** `docs/templates/adr-template-v2.md` (TPL-ADR, Accepted 2026-05-12; supersedes `adr-template.md`). Enforcing lane: `oya-governance-adr-shape` → real impl `scripts/validate-adr-shape.mjs` → `cargo run -p oya-dev-cli -- lint adr-shape`. File path convention: `docs/decisions/ADR-####-<slug>.md`. H1 form: `# ADR-####: <Title>` (colon form; the `# ADR-#### — title` dash form was normalized away per CHANGELOG 2026-05-16).

### Required front-matter (YAML, on every ADR — for ADR-INDEX autogen + supersession graph)
```yaml
---
id: ADR-####
title: "<Decision title in imperative form>"
status: Proposed | Accepted | Deprecated | Superseded
date: YYYY-MM-DD
owner_team: <team-id from docs/teams/>
co_owners: [<team-id>, <team-id>]
supersedes: [ADR-####, ...]
superseded_by: [ADR-####, ...]
related: [ADR-####, ...]
tags: [architecture, security, privacy, capability, tooling, ...]
purpose: |
  One paragraph: what this ADR decides + why future engineers read it. Used by ADR-INDEX renderer.
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

### Required body sections (shape checked by `oya-governance-adr-shape`)
`## Context` → `## Decision` (active voice, present tense, RFC-2119 MAY) → `## Decision drivers` (top 3, one line each) → `## Alternatives considered` (**MUST** list ≥2 viable incl. status-quo; Name/Pros/Cons/Reason-rejected) → `## Why chosen` → `## Consequences` (### Positive / ### Negative [honest] / ### Operational) → `## Compounding principles incorporated by reference` (RECOMMENDED) → `## Follow-ups` (numbered, each with target ADR id + owner + tracking ref) → `## References`.

> Per D3 + the ADR per-class checklist: new ADR ⇒ author from this template, then update `docs/ADR-INDEX.md` (`oya-governance-adr-citation`) and add the DOC-CATALOG row (`doc.adr_####`) + CHANGELOG dated block. Supersession uses `docs/templates/adr-supersession-template.md`.

---

## 7 — Honest gaps (requirements with NO on-disk source basis as named)

1. **`guard-pr-merge-review.mjs`** — referenced in `AGENTS.md:168`, `done-definition-checklist.md` (D8), `pull-request-template-v2.md`. **Not found** anywhere in `source` (repo-wide `find`/`grep`, excl. `.claude/worktrees/`). The `## Code Review` merge-gate is therefore **documented doctrine, not enforced** on dev.
2. **`traceability-validator`** + **`oya-pr-review`** — named as the 5-H2 enforcers (`AGENTS.md:211`, `pull-request-template-v2.md` `enforcing_fitness_lane`). **`traceability-validator` has no crate/lane/file** (confirmed absent). **`oya-pr-review` is NOT a crate/lane either** — it is a hosted **required-check *context name*** (a branch-protection / commit-status string). It surfaces in real CI plumbing only as that token: `oya/developer-sdk/.../aspirational_enforcement_gate.rs:105` (`if branch_required_contexts.contains("oya-pr-review")`), `oya/ci-webhook-gateway/src/receiver.rs:302` (`assert!(text.contains("oya-pr-review")) // the honest boundary`), and `libs/oya-governance-gate-catalog-domain/src/lib.rs:200` (catalog comment). So `oya-pr-review` is an *aspirational-enforcement* required-context that the webhook gateway posts/expects — **not** a validator that parses the 5 H2 headings. The real lane that runs locally is `oya-governance-pr-traceability` (3-boolean check: phase-id / plan-or-ADR / decision-log; `Warn`|`Block` ratchet) — it does **not** parse the 5 H2 headings.
3. **`oya-governance-changelog-row` (D18)** — named in AGENTS.md/checklist but **absent from the 96-lane `registry/quality/lanes.yaml` `gate validate` roster.** CHANGELOG row is **checklist-enforced only**, not mechanically gated.
4. **CHANGELOG row format dual-spec** — `DOC-UPDATE-PROTOCOL.md` step 8 says `<doc.id> <iso-date> <author> <one-line>`, but the live `docs/CHANGELOG.md` uses dated `## H2 + bullets + evidence-path`. Both documented; the live file is the de-facto format. (Reconciliation, not a missing basis.)
5. **`doc-catalog` lane path-drift** — lane spec reads `docs/CATALOG.md`; real file is `docs/DOC-CATALOG.md` (no `docs/CATALOG.md` exists). Flagged in pre-lane 0.5 §6b; reconcile in conformance lane. (Bug, not a fabrication.)
6. **Facet descriptions** — `/specs/multispectrum-review.json` `facets.*` carry **names only** (e.g. "Linus critique"), no inline `description` text in the facet objects; the trigger-question prose lives elsewhere in the spec (`alias_map` + per-facet trigger registries). Facet *semantics* are therefore name-driven; the real example evidence file is the best concrete reference for expected `findings` content.
7. **`oya gate`/Jenkins = legacy bridge** — `AGENTS.md:68` + ADR-0513/ADR-0363 mark the live merge destination as Prow-shaped **cloud-ci/oya-ci** (`oya/ci-controller`); `oya gate`/Jenkins are transitional evidence until the P0.0 cutover. So "the gate" today = `oya verify --ci-required` + `lanes.yaml` lanes, not a hosted required-check that is fully wired.

---

## 8 — Sources (all read read-only from `/Users/jasonlee/Developer/source`)

- `docs/AGENTS.md` (operating contract — §multispectrum :68-75; §PR shape :211-219; §Done-Definition :221-244; §reviewer agents :151-168; §Long-running loop :270-272)
- `templates/checklists/done-definition-checklist.md` (CHK-DONE; per-change-class additions)
- `docs/DOC-UPDATE-PROTOCOL.md` (5-stage; CHANGELOG step 8; 5-H2 step 18)
- `docs/templates/pull-request-template-v2.md` (TPL-PR; 5 H2 + Code Review)
- `docs/templates/adr-template-v2.md` (TPL-ADR; front-matter + body shape)
- `/specs/multispectrum-review.json` (v2.4.0; change_classes, facets, evidence_schema, verdict_schema, depth_levels, mandatory_subchecks_in_seam_lane)
- `templates/checklists/pre-pr-multispectrum.json` (TPL-PRE-PR-MULTISPECTRUM; fill-template + agent steps)
- `docs/standards/multispectrum-review.md` (thin human gateway; canonical-homes table)
- `docs/DOC-CATALOG.md` (§0 reading guide, §1 EVT-* events, §2 catalog rows) + `docs/machine-readable/catalog.json`
- `docs/CHANGELOG.md` (live dated-H2 + bullets format)
- `registry/quality/lanes.yaml` (96 lanes; real `gate validate <name>` roster)
- `scripts/hooks/pre-push.sh` (the one real local gate: `oya verify --ci-required --include-deferred`)
- `scripts/validate-adr-shape.mjs` (real ADR-shape enforcer → `oya-dev-cli lint adr-shape`)
- `libs/oya-governance-pr-traceability-kernel/src/lib.rs` + `libs/oya-check-pr-traceability/` (real PR-traceability check)
- `libs/oya-check-dependency-seam/` (seam lane reading multispectrum evidence)
- `evidence/multispectrum/hook-context-removal-1780274908.json` (real complete evidence example)
- `docs/audit/initial-sweep-2026-06-06/_execution/prelane-0.5/00-PRELANE-0.5-MANIFESTS.md` §6b (docs/-not-root convention; doc-catalog path-drift)
- Negative searches (no on-disk match): `guard-pr-merge-review*`, `traceability-validator`, `oya-pr-review` (repo-wide, excl. `.claude/worktrees/`).
