# 20 — Adversarial verification of the three contradiction lanes

READ-ONLY adversarial re-verification of `10-flat-crates-map.md`, `10-status-edge-drift.md`,
`10-directive-without-edge.md`. For EACH finding I opened the cited file at the cited line and
confirmed (or refuted) the claim verbatim. Default verdict = REFUTED if the citation did not check out.

All cited paths are under `/Users/jasonlee/Developer/source/` unless prefixed `linux:`.
`.claude/worktrees/**` excluded as stale. The decisions dir is `docs/decisions/`; the ADR index is
`docs/ADR-INDEX.md`; the canon ruling file is
`linux:docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (NOT under source/).

VERDICT HEADLINE: **all material findings CONFIRMED.** 0 substantive refutations. 3 minor
citation-precision corrections (line-label off by a few lines / path-form), none of which change a
verdict. My own independent re-scan FOUND THE ARTIFACTS' PARSERS MORE PRECISE THAN A NAIVE REGEX
(they correctly excluded YAML-null `~` edges and a documentation template block that a naive scan
false-positives on) — this strengthens, not weakens, their counts.

---

## A. flat-crates-map — CONFIRMED

### A1. Supersession chain (frontmatter) — CONFIRMED
- **ADR-0015** `status: accepted` (`ADR-0015-architectural-flattening-target.md:3`), `superseded_by: [ADR-0131]` (`:5`), `supersession_note` "status stays accepted" (`:6`). Body flat refs `:34`,`:39`. CONFIRMED.
- **ADR-0131** `status: Accepted` (`ADR-0131-per-microservice-flat-layout.md:3`); `supersedes: ADR-0015 (partial), ADR-0119 (partial)` (`:8-10`); `superseded_by: []` (`:11`); amendment 2026-06-02 `microservices/<ms>/` → `{oya,cloud}/<service>/` (`:24-28`). CONFIRMED.
- **ADR-0512** `status: Accepted` (`ADR-0512-canonical-monorepo-pattern.md:3`); `supersedes: [ADR-0357, ADR-0509]` (`:9-11`); `amends: [ADR-0131]` (`:12-13`); "founder-locked" 2026-05-29 (`:22`); **"A flat top-level `crates/` directory is forbidden"** (`:55-57`); canonical `{oya,cloud}/<service>/crates/<crate>/` + `libs/<lib>/` (`:53-57`); gate enforcement (`:62`). CONFIRMED.
- **ADR-0357** `status: Proposed` (`ADR-0357-vertical-slice-monorepo-nesting.md:3`); `supersedes: []` (`:8`); `amends: []` (`:9`); has **no `superseded_by` field at all** (grep exit 1). CONFIRMED.
- **ADR-0509** `status: Accepted` (`ADR-0509-hyperscaler-service-decomposition-pattern.md:4`); `superseded_by: []` (`:10`). CONFIRMED.

### A2. LOCATION-vs-NAMING verdict — CONFIRMED
- (a) LOCATION flat top-level `crates/` = SUPERSEDED/FORBIDDEN — `ADR-0512:55-57,62` verbatim "flat top-level `crates/` directory is forbidden" / "flat `crates/` rejected". CONFIRMED.
- (b) NAMING `oya-<context>-<role>` SURVIVES — `ADR-0512:59` ("EVERY service crate … lives at `{oya,cloud}/<service>/crates/oya-<...>/`"), `ADR-0131:97-98` (`crates/` + `oya-<ms>-<bc>-<layer>/`). CONFIRMED. *Minor:* the map glossed the surviving naming as `oya-<context>-<role>[-<capability>]`; `ADR-0131:98` literally uses `oya-<ms>-<bc>-<layer>`. Same convention family (oya- prefix, context/role-structured); substance holds.

### A3. Live tree — CONFIRMED (git-verified)
- `oya/` tracked (14649 files), `cloud/` tracked (1771), `libs/` tracked (586). `microservices/` absent (dir gone, `git ls-files` = 0). CONFIRMED.
- Stale top-level `crates/` dir EXISTS on disk, **UNTRACKED** (`git ls-files crates/` = 0), code-empty: exactly 3 `.DS_Store` + 2 empty dirs `oya-application-app/`, `oya-audit-chain-emission-api/`. CONFIRMED verbatim — it is the live embodiment of the forbidden topology.

### A4. Gate wiring (the live contradiction) — CONFIRMED — HIGHEST SEVERITY
- `governance-lanes/flat-crates.md`: `status: Accepted` (`:7`), severity `BLOCKER` (`:20`), enforces `oya-governance-flat-crates` (`:10`); kernel hard-fails `if depth != 3 { … NestedCrate … }` (`:42-44`,`:48`), with manifest depth modeled as `crates/<name>/Cargo.toml` (`:9`,`:15`). This depth-3 rule REJECTS the canonical depth-5 `{oya,cloud}/<service>/crates/<crate>/Cargo.toml` of ADR-0512. **Direct contradiction. CONFIRMED.**
- Claimed live BLOCKER in 3 doc lists: `governance-lanes/INDEX.md:28` (flat-crates row, BLOCKER), `docs/AGENTS.md:231` (D7), `../../../../templates/checklists/done-definition-checklist.md:33` (D7). All three CONFIRMED. *Path note:* AGENTS.md D7 resolves at `docs/AGENTS.md:231` (a second `AGENTS.md` exists at repo root; the cited content is the `docs/` copy).
- ABSENT from executable registry: `registry/quality/lanes.yaml` — independent case-insensitive grep for flat-crates/flat_crates = **0 matches**. The wired architecture lane is `lean-a1-architecture` → `cargo run -p oya-dev-cli -- gate validate architecture-boundaries`, `source: ADR-0056` (`lanes.yaml:485-493`). CONFIRMED — the BLOCKER is doc-declared but not machine-wired.

### A5. Stale LOCATION/gate refs (sampled) — CONFIRMED
- `ADR-0001-...:106` (`crates/oya-*`), `ADR-0008-data-use-boundary.md:126` (`oya-governance-flat-crates` gate), `ADR-0015:34,39,160,201`, `ADR-0357:25,27,31,35` (dead `microservices/<ms>/crates` path), `specs/masterplan.json:6046-6047` (cites superseded ADR-0357 verdict), `standards/crate-naming-convention.md:42` (ADR-0015 flat crates), `registry/bounded-contexts.json:6` (ADR-0015 flat-crates). All CONFIRMED. (~50-entry table sampled, not line-exhausted — consistent with map's own no-silent-cap note.)

### A6. Integrity gaps — CONFIRMED
- `docs/ADR-INDEX.md`: **0 rows for ADR-0509, 0 for ADR-0512** (grep counts = 0). CONFIRMED.
- `docs/ADR-INDEX.md:317` still lists ADR-0357 `Proposed`. CONFIRMED.
- `docs/ADR-INDEX.md:37` still lists ADR-0015 `accepted` with the full flat-crates title. CONFIRMED.
- ADR-0509 `status Accepted` + `superseded_by: []` (no back-pointer despite being superseded by 0512). CONFIRMED. ADR-0357 has no `superseded_by` field + `Proposed`. CONFIRMED.

---

## B. status-edge-drift — CONFIRMED (+ independent re-scan agrees)

Scope: independent `ls ADR-*.md` = **347 files** (matches claim). My independent enumeration of
non-`superseded`-status files carrying a genuine non-empty `superseded_by` (inline `[ADR…]` OR
YAML block-list) returned exactly: **ADR-0015, ADR-0316, ADR-0358** — matching the artifact's
superseded_by-based Case-A set. Case-B (status `Superseded` + empty `superseded_by`) = **0** (independently reproduced).

- **ADR-0015** (named exemplar) — `status: accepted` (`:3`) + `superseded_by: [ADR-0131]` (`:5`), self-declared via `supersession_note` (`:6`). CONFIRMED.
- **ADR-0316** — `status: Proposed` (`:3`) + `superseded_by: [ADR-0329]` (`:28`). CONFIRMED.
- **ADR-0358** — `status: Proposed` (`:3`) + YAML **block-list** `superseded_by:` `- ADR-0392` / `- ADR-0408` (`:9-11`). CONFIRMED — would be missed by an inline-`[…]`-only scan; only caught by block-aware parsing.
- **ADR-0482** — `status: Accepted` (`:3`) + `amended_by: [kubers-anchor-2026-05-28]` (`:13`), a **non-ADR token** (dangling). CONFIRMED.
- **ADR-0052** — frontmatter `status: Superseded` (`:4`) + `superseded_by: [ADR-0118]` (`:11`) but BODY `> **Status:** Accepted` (`:29`) and `> **Superseded-by:** —` (`:32`). CONFIRMED body-vs-frontmatter contradiction.
- **ADR-0363** — `status: Accepted` (`:3`) + `amended_by: [ADR-0510, ADR-0513]` (`:10`). CONFIRMED.
- **ADR-0054** — `status: deprecated` (`:3`) + body `Superseded by ADR-0116` (`:9`) / `> **Superseded-by:** ADR-0116` (`:13`). CONFIRMED.

Borderline-excluded correctly verified: ADR-0120 `status: Superseded` (`:3`), ADR-0121 `status: Superseded` (`:3`) — correctly NOT Case-A. ADR-0147 `status: Amended` (`:3`) + `superseded_by: []` (`:9`) — internally consistent, correctly excluded.

**Adversarial note (artifact vindicated):** my naive regex initially over-counted (28 "non-empty" superseded_by vs the artifact's 24) because it treated `superseded_by: ~` (YAML null — ADR-0056:21, ADR-0057:17, ADR-0083:18) and a documentation **template block** (`ADR-0065:66-70` "`superseded_by: ADR-#### | null`") as real edges. On inspection those are null/template, NOT drifts. The artifact's "24 non-empty / 21 correctly-superseded / 3 Case-A-from-superseded_by" is the ACCURATE figure; my over-count was the false positive. No refutation.

---

## C. directive-without-edge — CONFIRMED (P1–P5)

Canon ruling file confirmed at `linux:docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`.

- **P1 — ADR-0160 Flagger** — `ADR-0160-progressive-delivery-flagger.md:3` `status: Accepted`; `:8` `superseded_by: []`; `:42` "adopts **Flagger 1.x** as the canonical progressive-delivery controller"; `:62` "Why Flagger over Argo Rollouts". vs **D10** `canon:62` "Supersede Flagger (0160)" (door two-way) + **ADR-0515** Argo Rollouts canonical (`ADR-0515-...:80,83`; `status: Accepted` `:4`). NO EDGE: 0160 empty AND ADR-0515 `supersedes: [ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511, ADR-0513, ADR-0514]` (`:9`) — 0160 absent. CONFIRMED.
- **P2 — ADR-0187 Zitadel-primary** — `ADR-0187-...:3` Accepted; `:8` `superseded_by: []`; `:17`/`:37` canonical IdP. vs **D5** `canon:31` "0187 demoted … superseded-as-endpoint by 0476" + "0476 `supersedes:[0187]` (fix the 0421 mis-number)" + "hard contradiction (C-4)". **DOUBLY edgeless:** 0187 empty AND `ADR-0476-...:9` `supersedes: [ADR-0421]` — NOT [0187] (mis-number un-fixed); `ADR-0476:10` `superseded_by: []`; ADR-0476 `status: Accepted` (`:4`). CONFIRMED. Highest severity.
- **P3 — ADR-0374 Forgejo→Jenkins webhook gateway** — `ADR-0374-ci-webhook-gateway-forgejo-jenkins.md:3` `status: Accepted`; `:9` `superseded_by: []`; `:55-56` git+Jenkins+Forgejo substrate; `:188` "Jenkins-as-orchestrator". vs **D2** `canon:153` + **D-FORGE-CLARIFY** `canon:207` (Forgejo eradicated; names 0374-class). NO EDGE: ADR-0374 appears only in `related` of the now-`Superseded` ADR-0513 (`ADR-0513-...:14` related, `:25` "retains ADR-0374's webhook"; `ADR-0513:3` `status: Superseded`); ADR-0515 omits 0374 from `supersedes`. CONFIRMED. *Minor:* map labeled the status citation `:6` Accepted — the status line is `:3`; `:6` is the `date`. Substance unaffected.
- **P4 — ADR-0380 Jenkins-farm-on-Talos + Forgejo gating** — `ADR-0380-...:3` `status: Accepted (amendment)`; `:9` `superseded_by: []`; `:55` title. vs same D2/D-FORGE-CLARIFY/D-CICD. Source admits the gap: `ADR-0513-...:22-23` "formal supersession of ADR-0380 lands at the Phase-1 cutover" — never landed (0513 itself Superseded; 0380 absent from 0515's supersedes). CONFIRMED.
- **P5 — ADR-0335 foundry→intelligence** — `ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md:3` `status: Accepted`; no `superseded_by`/`amended_by` (grep empty); `:158` "`microservices/intelligence/` is the canonical AI substrate"; `:514` "only approved AI substrate kernel surface". vs **D-INTEL** `canon:90` "RE-HOME the 96k-LOC engine DOWN from oya/intelligence into cloud/cloud-intelligence". NO EDGE. Caveat preserved: ratchet/superseded-on-cutover (`canon:98-99`,`:26`), currently absent entirely. CONFIRMED.

### Directive corrections / exclusions — CONFIRMED as correct
- **"ADR-0010 = Argo-Rollouts" is a MIS-CITATION in the directive input** — `ADR-0010-regional-pack-architecture.md:7` is "Regional pack architecture", unrelated to Argo. The real Flagger-vs-Argo pair is P1 (0160). CONFIRMED — the artifact was right to flag the input's error.
- **ADR-0195 vs 0377 NOT a contradiction** — `ADR-0195-stream-processing-tier.md:71-72` already sources from Apache Pulsar (Kafka-wire-protocol/KoP); `ADR-0377-kafka-to-pulsar-via-kop.md:22` (0005=the real Kafka ADR) and `:102` ("ADR-0195 … introduced KoP"). Consistent with D-EVENT `canon:147` (Pulsar canonical). CONFIRMED exclusion.
- **ADR-0363 excluded from strict no-edge lane** — `ADR-0363-...:10` `amended_by: [ADR-0510, ADR-0513]` (not edgeless; stale-via-stale-chain). CONFIRMED.

---

## REFUTED / false-positive list

**0 substantive refutations.** No finding's core claim failed verification. The following are minor
citation-precision deltas (verdicts unchanged):

1. **ADR-0374 status line label** — directive-without-edge cited `:6 Accepted`; the `status: Accepted` line is `:3` (`:6` is `date:`). Substance (Accepted + edgeless) intact.
2. **ADR-0131 naming string** — flat-crates-map glossed the surviving naming as `oya-<context>-<role>[-<capability>]`; `ADR-0131:98` literally reads `oya-<ms>-<bc>-<layer>`. Same convention family; the (b) NAMING-survives verdict stands (independently supported by `ADR-0512:59`).
3. **ADR-INDEX path-form** — flat-crates-map wrote `ADR-INDEX.md:37/:317`; the file resolves at `docs/ADR-INDEX.md` (correct relative to docs/). Line numbers exact.

Additionally, **my own naive re-scan produced 2 classes of false positive that the artifacts
correctly avoided** (logged here so the over-count is not mistaken for artifact error):
- `superseded_by: ~` (YAML null) at ADR-0056:21 / ADR-0057:17 / ADR-0083:18 is NOT an edge.
- The template block `ADR-0065:66-70` ("`superseded_by: ADR-#### | null`") is documentation, not a live edge.
These confirm the status-edge artifact's 24/21/3 counts as the accurate figures.

---

## COUNTS
- Findings re-verified: flat-crates-map = 6 sections (chain·verdict·tree·gate·stale-refs·integrity), all CONFIRMED; status-edge = 7 drifts + 347-file scope + Case-B=0, all CONFIRMED (independently reproduced); directive-without-edge = P1–P5 + 3 exclusions + 1 input mis-citation, all CONFIRMED.
- **Substantive refutations: 0.**
- Minor citation-precision corrections: 3 (none change a verdict).
- Naive-rescan false positives I caught and discarded (artifacts were right): 2 classes.

## NOT COVERED (no silent caps)
- The ~50-entry stale-LOCATION/gate ref table (flat-crates-map §5) and the ~650 type-(b) NAMING comment-headers were SAMPLED (7 representative hits opened), not line-exhausted — consistent with the map's own no-silent-cap note; the sampled hits were 7/7 accurate.
- The 374-file `microservices/` superset (a different, broader topology than strict flat-`crates/`) was NOT re-verified here — it is out of scope of these three lanes per the map's own note.
- `machine-readable/decisions.json` untrusted per instruction; not consulted.
- `.claude/worktrees/**` excluded as stale clones.
