# Planning SSOT Consolidation (Task #50 — the last cutover step)

> Plan for collapsing the sprawling planning/docs surface into a single docs-centered SSOT
> where **ADRs are the authored decision log and masterplan is GENERATED from them**.
> Grounded in two research passes (hyperscaler best-practice + repo inventory), 2026-05-26.

## Problem
The planning surface is fragmented and partly duplicative. The repo has *already* centralized on
`specs/` as machine-readable authority (`masterplan.json` canonical; `docs/MASTERPLAN.md` is a
"compatibility projection"), but: (1) `tasks/*.md` re-states masterplan/sequencing scope;
(2) `docs/plans/M01-*` is stale (2026-05-12/13, superseded 2026-05-19); (3) the **"pack" schemas
overlap** (`compliance-pack-schema` vs `sovereign-cloud-overlays` vs `compliance-pack-floors`);
(4) masterplan is *hand-authored*, risking drift from the ADR decisions it should reflect.

## Target topology (docs-centered, ADR-derived)
1. **ADRs (`docs/decisions/`) = the authored, immutable decision log** — the SSOT. Append-only;
   supersede, never edit. (Google canonical-doc rule; ADR immutability — adr.github.io.)
2. **`masterplan` = GENERATED from ADR front-matter** (`planning_impact`, `status`, `supersedes`,
   `superseded_by`, `related`) + the planning specs — never hand-maintained. Build a generator
   (`oya gen masterplan`) + a **drift gate** that fails if the committed masterplan ≠ regenerated.
   (AWS "docs as a build artifact"; OpenAPI generate-don't-dual-author.)
3. **`docs/` organized by Diátaxis**: `tutorials/`, `how-to/` (runbooks), `reference/` (generated
   from machine specs + crate/microservice catalog), `explanation/` (architecture, ideas). Generated
   reference is build output, never hand-edited. (diataxis.fr.)
4. **Three planning layers, one artifact each, cross-linked by ID** (never restated):
   roadmap = generated masterplan; decisions = ADRs; tactical tasks = a single ID-referencing list.
   (Atlassian roadmap-vs-backlog; GitLab "create once, reference.")
5. **Docs-as-code governance**: CODEOWNERS routing + per-doc `last_reviewed`/freshness field enforced
   as a gate. (Google freshness dates; Spotify co-location.)

## Keep / Merge / Retire (from repo inventory)
- **KEEP (canonical):** `specs/masterplan.json` (becomes generated), `master-plan-sequencing.json`,
  `planning-closure-contract.json`, `root-hub-pointers.json`, `docs/decisions/*` (300 ADRs),
  `evidence/goals/*`, `docs/localization-packs/kr/` (FD-001 Korea pack evidence).
- **RETIRE:** `tasks/plan.md`, `tasks/todo.md` (duplicate masterplan/sequencing — move active CS-*
  rows to `evidence/goals/` or a CI registry), `tasks/enterprise-microservices-*.md` (empty).
- **ARCHIVE:** `docs/plans/M01-foundation-cc-01-cutover/` (stale; to `.omc/archive/…`).
- **Deprecate (until PHASE-5):** `docs/MASTERPLAN.md` — mark "generated; do not edit."
- **KEEP as working docs:** `.omx/plans|context|specs` (tactical, not strategic; not duplication).

## Pack deconfliction (the duplication you flagged)
`compliance-pack-schema` (ADR-0251), `sovereign-cloud-overlays` (ADR-0179), `compliance-pack-floors`
(ADR-0343), `pack-overlay-schema` all assert pack-level jurisdiction/provider/stringency rules.
**Action:** one deconfliction ADR — decide the split by *lifecycle stage* (authoring vs activation
vs request-time evaluation) OR merge into one pack-metadata schema. Genuinely distinct and kept:
`regional/localization-pack` (locale+ops, orthogonal), `capability-pack` (product tier).
Most "*-pack" mentions (`dr-pack`, `evidence-pack`, `api-pack`, …) are prose subdomains, not schemas.

## The ADR template is the prerequisite (a generated masterplan is only as actionable as ADR fields)
A generated roadmap needs ADRs to carry **structured, machine-extractable fields** — prose alone
can't be projected. Define an ADR template + front-matter schema (itself an ADR — the meta-decision):

**Precedent: Kubernetes KEP** (the one validated real-world instance of generating a roadmap from
decision-record metadata). KEP splits an immutable narrative (`README.md`) from machine-readable
`kep.yaml` (stage/milestone), and graduates stages via Production-Readiness-Review/CI signals — not
a hand-set flag. Our ADR ≙ KEP narrative; our front-matter ≙ `kep.yaml`; `verified_by` ≙ PRR criteria.

```yaml
id, title, status(Proposed|Accepted|Superseded|Rejected|Deprecated), date, owner
planning_impact: true            # filter — only these enter the roadmap
supersedes / superseded_by       # invalidation edges (the only mutable links)
depends_on: [ADR-...]            # sequencing edges (beyond supersede)
milestone: M0x                   # roadmap grouping
affected_surfaces: {crates, microservices, specs}   # scope + ownership
deliverables:                    # immutable DEFINITION only — NO status field
  - {id, description, exit_criteria, verified_by(gate/evidence)}
```
**Status is NOT stored in the ADR** — it's *derived* from `verified_by` at generation time
(gate green ⇒ done; in progress ⇒ in_progress; else planned). This keeps the record immutable
(only `status`/`supersedes` ever change — MADR/Log4brains/Azure-WAF rule), makes roadmap drift
structurally impossible (status is computed, never authored — Fern "generated can't rot"), and
prevents spec-saturation (a deliverable can't be `done` without a passing gate — Amazon mechanisms).
Generator: filter `planning_impact && status==Accepted` → topo-sort by `depends_on`/`supersedes`
→ group by `milestone` → emit each `deliverable` with CI-derived status + exit_criteria + verified_by.
**Balance:** rich *structure*, lean *prose* (MADR-minimal narrative). Cap status to KEP's ~2 axes —
no `blocked`/`at-risk`/`deferred-q3` sprawl. No assignees/due-dates/sprint fields (≠ Jira). Generate
the **roadmap, not OKRs** (KEP keeps org-objectives separate). Transitional fallback if full
CI-derivation isn't ready: a `deliverable-status.json` ledger keyed by id, itself gate-checked
against `verified_by` evidence — never status fields inside the ADR.

## The decision-record system generates TWO things (and ratifies shared contracts)
The ADR system is the agreed source for both:
- **(A) Roadmap** — generated from `planning_impact` ADRs' `deliverables` (above).
- **(B) Shared contracts** — schemas, data models, API/event/proto interfaces. **The registry
  artifact is the normative source; the ADR records the decision *about* it by reference (cite-as),
  never a copy of the schema** (RFC 6596 canonical-link / Google AIP records rules-not-bytes /
  spec-first SSOT — embedding schema in ADR prose = dual-source drift, a hard gate failure).
  Canonical artifacts stay where they live (`schema-registry-canonical.json` is the *index*; bytes
  live in OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 / JSON-schema / Cedar files). ADR front-matter:
  ```yaml
  contracts:
    - {id/path, pinned_version(semver|commit), change_type: additive|breaking,
       compatibility_mode: BACKWARD|FORWARD|FULL(+_TRANSITIVE),
       consumers: [...]  (Pact-derived where possible), migration: <window/deprecation ref>}
  ```
  **Risk-tiered ratification** (Google's anti-bottleneck lesson — pure human review didn't scale):
  *additive* = auto-ratify via CI diff+lint (no board); *breaking/novel* = AIP-style quorum
  (domain TL + ≥1 reviewer, supersede-don't-edit) + mandatory migration + consumer accounting.
  **Contract-traceability + compatibility gate** enforces: (1) every registry entry → a ratifying
  ADR (orphans fail); (2) **computed diff == declared `change_type`** (`buf breaking` for proto,
  OpenAPI/AsyncAPI diff, JSON-schema/Avro compat, Cedar diff — declaring `additive` on a breaking
  diff fails); (3) hard wire-invariants (proto field-number/wire-type reuse → unconditional fail,
  no ADR can approve it); (4) breaking ⇒ new version id + deprecation + `can-i-deploy`-style consumer
  disposition (migrated/acknowledged/opted-out); (5) TRANSITIVE history check; (6) SSOT guard (fail
  if a schema body is embedded in an ADR). *Agreement = the diff passing, not a human checkbox.*

## Diátaxis reorganization of `docs/` (confirmed)
Four quadrants, each doc has exactly one home:
- **tutorials/** — learning-oriented (onboarding, *-first-week guides).
- **how-to/** — task-oriented (runbooks, playbooks, migration-playbooks).
- **reference/** — information-oriented, **generated** (masterplan, crate/µservice catalog, contract
  registry views, gate catalog) — never hand-edited.
- **explanation/** — understanding-oriented (architecture, ideas, regional-pack rationale).
- **decisions/** — ADRs stay as their own immutable log (the authored source the above generate from).

## Full reset: re-found the ADR log from ADR-0000 (re-foundation, NOT in-place renumber)
Treat this as a clean re-founding of the decision log. **Do not rewrite IDs in the existing 300
files** — that destroys the immutable audit trail (the trustworthiness-via-stable-IDs rule the
research flagged). Instead:
- **Distill**: the audit (step 0) classifies LIVE / superseded / obsolete / duplicate; only LIVE
  decisions carry forward.
- **Re-author survivors** into a fresh canonical **ADR-0000+** sequence in the new template, each
  with `consolidates: [old-ADR-XXXX, …]` provenance (one new ADR may absorb several old ones).
- **Archive the old series frozen** (`docs/decisions-archive/` + git history) — history preserved,
  not deleted; immutability doctrine then holds *going forward* on the clean series.
- **Rewrite every ADR reference repo-wide** old→new from the mapping table (specs, gates, CLAUDE.md,
  root-hub-pointers, registry, code). High blast radius — this is why #50 is last.
- Numbering reset is the act of re-foundation; the masterplan + contract registry then generate from
  the clean ADR-0000+ series.

## Build order (this is task #50, AFTER T3)
0. **Audit existing docs + all ~300 ADRs** (prerequisite — drives every step below). Produce a
   classification per artifact: is it `planning_impact`? does it conform to the new ADR template?
   which Diátaxis quadrant (tutorial/how-to/reference/explanation/decision)? stale / duplicate /
   retire? pack-overlap? The 2026-05-26 planning-surface inventory (Explore agent) is the first
   pass (cluster-level keep/merge/retire); extend it to a **per-ADR template-conformance + Diátaxis
   bucket** report. This audit is itself a candidate for parallel sub-agents over ADR ranges.
1. **ADR template + front-matter schema ADR** — the generative contract (roadmap `deliverables` +
   `contracts`), with the immutability split (status NOT in the ADR; see above).
2. **ADR completeness gate** — fail any `planning_impact: true` ADR missing `deliverables`/
   `exit_criteria`/`milestone`. Forces the discipline without manual policing.
3. **Backfill** the planning-impact ADRs to the template (only `planning_impact`, not all ~300).
4. **`oya gen masterplan`** generator (ADRs + status-ledger/CI → roadmap projection).
5. **Masterplan drift gate** (committed == regenerated), wired into `presubmit`.
6. **Contract-traceability + compatibility gate** (registry entry → ratifying ADR; breaking →
   migration deliverable).
7. Retire tasks/, archive docs/plans/, deprecate docs/MASTERPLAN.md, add CODEOWNERS + freshness gate.
8. Pack deconfliction ADR + schema merge/split.
9. **Reorganize `docs/` into Diátaxis quadrants** (confirmed) — generated `reference/` is build output.

## Antipatterns this fixes
Parallel overlapping planning artifacts (Google GooWiki); dual-source drift (masterplan hand-authored
vs ADRs); "spec-saturated, code-starved" governance; pack over-fragmentation. (Sources banked in the
2026-05-26 best-practice research.)

## Canonical naming (self-explanatory; no provenance-in-name)
ONE canonical convention for every governance identifier — gates, lanes, specs, plans, docs, registries
(crate names already governed by the ADR-0105 BNF). Names describe **function**, best-practice / Google
AIP descriptive-identifier style — `kebab-case`, verb-noun, self-explanatory at a glance.
- **Good**: `presubmit` (retired CLI `gate validate contract-traceability`), `masterplan-drift`, `docs-folder-discipline`,
  `adr-planning-completeness` (here `adr` = the artifact type, not a number).
- **FORBIDDEN** (the antipattern): a name that encodes *provenance* instead of *function* —
  ADR-number-keyed (`adr-0244-*`, `adr0145-gates`, lane purposes that read "per ADR-0110"),
  wave/milestone codes (`M01-P18`, `lean-a1`, wave-numbers), or opaque ids. Decisions move/supersede;
  a name pinned to a number rots. (ADR *files* stay numbered `ADR-0000+` — that's the decision-LOG
  index, NOT a gate/lane/spec name.)
- **Enforcement**: a `canonical-naming` lint/gate — gate+lane+spec+plan ids must match a semantic
  `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` shape and must NOT match `adr-?\d{3,}` / `\bm\d{2}-p\d{2}\b` /
  bare wave-number patterns. Ships as build work (serializes after D2/D3/D4), alongside
  `docs-folder-discipline`.
- **Re-foundation cleanup**: rename existing offenders to semantic names — `adr_0145_gates` →
  function-named modules; lane `purpose` strings keyed to "ADR-NNNN" → describe the check + reference
  the ADR by a `source:`/`adr:` *field*, not in the id. (Part of D7.)
