# 00 — Supersession-Completeness (flat-crates decision + A-INTEGRITY edge sweep)

> **Status:** synthesis of ADVERSARIALLY-VERIFIED findings. Every claim below traces to a
> path:line that was opened and confirmed in `20-verified.md` (which re-opened each cited line in
> the three `10-*.md` lanes). **Substantive refutations during verification: 0.**
> **Read-only synthesis** — this document proposes the fixes; it does not apply them. Each
> mutating fix is gated on founder ratification (the founder rule: verify at every step, never
> mutate on an unverified verdict).
>
> Source trees: ADRs + standards + specs + registry under `/Users/jasonlee/Developer/source/`.
> Founder rulings under `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (cited as `canon:NN`).
> `docs/machine-readable/decisions.json` is DRIFTED and was NOT trusted. `.claude/worktrees/**` excluded as stale clones.

---

## 1. THE FLAT-CRATES DECISION (source-backed, for founder ratification)

### 1.1 Verdict (plain statement)

**The flat top-level `crates/` LOCATION is SUPERSEDED and FORBIDDEN. The flat crate NAMING
`oya-<ctx>-<role>[-<capability>]` SURVIVES.** These are two separate axes and must be ruled on
separately — conflating them is the root of the ~50 stale references below.

- **(a) LOCATION = SUPERSEDED / FORBIDDEN.** The on-disk address `crates/oya-<ctx>-<role>/Cargo.toml`
  (manifest-path depth 3) is killed. The canonical layout is now
  `{oya,cloud}/<service>/crates/<crate>/Cargo.toml` (depth 5) for service code, and `libs/<lib>/`
  for shared code.
  - `ADR-0512-canonical-monorepo-pattern.md:55-57` — "Service code lives at
    `{oya,cloud}/<service>/crates/<crate>/` … **A flat top-level `crates/` directory is forbidden.**
    `microservices/` is legacy/removal-candidate."
  - `ADR-0512:62` — "The architecture-boundaries gate requires service code under
    `{oya,cloud}/<service>/crates/` or shared code under `libs/<lib>/` (**flat `crates/` rejected** …).
    Workspace-topology validation fails on: a flat `crates/` directory."
- **(b) NAMING = SURVIVES.** The crate *name* (the `[package].name` and crate-dir basename) is unchanged.
  - `ADR-0512:59` — "EVERY service crate … lives at `{oya,cloud}/<service>/crates/oya-<...>/`" (keeps the
    `oya-` prefix and the role-structured name); "crate directory basename MUST equal `[package].name`."
  - `ADR-0512:66` — "package names … are unchanged."
  - `ADR-0131-per-microservice-flat-layout.md:97-98` — still prescribes `oya-<ms>-<bc>-<layer>` crate
    directories *inside* the per-service `crates/`.
  - **Therefore any reference to the *string* `oya-<ctx>-<role>` is NOT a contradiction.** A reference is
    stale ONLY when it asserts/enforces the crate **lives at top-level `crates/…`**, enforces the depth-3
    manifest-path rule, or names `microservices/<ms>/crates` (ADR-0357's dead path).

### 1.2 The supersession chain (verified frontmatter)

```
ADR-0015 (status: accepted; superseded_by:[ADR-0131], PARTIAL — only docs-vs-crates top-level split)
   └─► ADR-0131 (Accepted) supersedes ADR-0015 (partial) + ADR-0119 (partial)
           amended 2026-06-02: service root microservices/<ms>/ → {oya,cloud}/<service>/
   └─► ADR-0512 (Accepted, founder-locked 2026-05-29) — CANONICAL
           supersedes [ADR-0357 (Proposed, never accepted), ADR-0509 (Accepted)]
           amends ADR-0131
           "A flat top-level crates/ directory is forbidden."
```

- `ADR-0015-architectural-flattening-target.md:3` `status: accepted`; `:5` `superseded_by: [ADR-0131]`;
  `:6` `supersession_note` ("Partial — … status stays accepted").
- `ADR-0131-per-microservice-flat-layout.md:3` `status: Accepted`; `:8-10` `supersedes:` ADR-0015 (partial)
  + ADR-0119 (partial); `:11` `superseded_by: []`; `:24-28` amendment to `{oya,cloud}/<service>/`.
- `ADR-0512-canonical-monorepo-pattern.md:3` `status: Accepted`; `:9-11` `supersedes: [ADR-0357, ADR-0509]`;
  `:12-13` `amends: [ADR-0131]`; `:22` founder-locked 2026-05-29.
- `ADR-0357-vertical-slice-monorepo-nesting.md:3` `status: Proposed` (NEVER accepted); `:8` `supersedes: []`;
  `:9` `amends: []`; **no `superseded_by` field at all**.
- `ADR-0509-hyperscaler-service-decomposition-pattern.md:4` `status: Accepted`; `:10` `superseded_by: []`
  (no back-pointer despite being superseded by 0512).

### 1.3 EXACT fixes (each cited)

**FIX-FC-1 — Flip ADR-0015 status.** Change `ADR-0015-...:3` `status: accepted` → `status: Superseded`
(its top-level-`crates/` LOCATION clauses are dead; the BC/layer rules it also carries are what the
`supersession_note` was protecting, but the literal status-vs-edge rule requires the flip — see §3).
The `superseded_by: [ADR-0131]` edge (`:5`) already exists, so this is a status-only correction. Also
update `docs/ADR-INDEX.md:37` (`ADR-0015 | accepted | …flat-crates…`).

**FIX-FC-2 — Retarget or retire the `oya-governance-flat-crates` GATE (the live contradiction, HIGHEST SEVERITY).**
The gate STILL enforces the SUPERSEDED depth-3 topology and therefore actively REJECTS ADR-0512's canonical
depth-5 path:
  - `docs/governance-lanes/flat-crates.md:7` `status: Accepted`; `:20` `severity: BLOCKER`;
    `:9`/`:15` manifest modeled as `crates/<name>/Cargo.toml`; `:42-44`,`:48`
    `let depth = …count(); if depth != 3 { return Err(… NestedCrate …) }`.
  - `{oya,cloud}/<service>/crates/<crate>/Cargo.toml` has depth 5 → `depth != 3` ⇒ `NestedCrate` BLOCKER-fail.
  - **Fix:** retire this lane and fold its intent into the already-wired `lean-a1-architecture` lane
    (`registry/quality/lanes.yaml:485-493`, `source: ADR-0056`, runs
    `cargo run -p oya-dev-cli -- gate validate architecture-boundaries`), updating that gate to require
    `{oya,cloud}/<service>/crates/` or `libs/<lib>/` per `ADR-0512:62`. Then strike the stale BLOCKER claims:
    `docs/governance-lanes/INDEX.md:28`, `docs/AGENTS.md:231` (D7), `templates/checklists/done-definition-checklist.md:33` (D7).
  - Note the registration drift: the lane is doc-declared a live BLOCKER but is **ABSENT from the executable
    registry** (`registry/quality/lanes.yaml`, grep flat = 0). It enforces the wrong topology AND is not machine-wired.

**FIX-FC-3 — Eradicate the live top-level `crates/` dir (it IS the forbidden topology).**
On disk: `crates/` EXISTS, is **UNTRACKED** (`git ls-files crates/` = 0), and is code-empty — exactly
3 `.DS_Store` files + 2 empty dirs (`oya-application-app/`, `oya-audit-chain-emission-api/`), no `Cargo.toml`,
no `src/`. Per `ADR-0512:57` (must be empty after migration) / `:62` (validation fails on a flat `crates/`),
**delete the directory.**

**FIX-FC-4 — Sweep the stale flat-crates LOCATION/gate bindings (~50 refs; each cited).**
Seed corners (the "Flat-crates binding" GATE/topology wording):
  - `ADR-0001-...:106` (`every crate under crates/oya-*`), `ADR-0008-data-use-boundary.md:126`
    (`oya-governance-flat-crates` gate), `ADR-0013-...:109` (`every crate under crates/oya-*`),
    `ADR-0020-...:175` (`Flat-crates binding … crates/oya-foundry-adapter-kernel`),
    `ADR-0022-...:175` (`Flat-crates binding … crates/oya-foundry-policy-kernel`).
ADR-0015 self-assertions: `ADR-0015-...:34,39,160,201`; index row `docs/ADR-INDEX.md:37`.
ADR-0357 dead `microservices/<ms>/crates` path: `ADR-0357-...:25,27,31,35`.
specs/: `per-microservice-flat-layout.json:252,345,365`; `masterplan.json:6046-6047` (cites SUPERSEDED
  ADR-0357, not 0512); `cloud-strangler-migration-target.json:26` (cites superseded ADR-0357).
standards/design/roadmap/checklists/products: `ROADMAP.md:52,63,172`; `DESIGN.md:23,64,186,337,446,497,505`;
  `SPEC.md:26,142`; `TOOLCHAIN.md:202`; `standards/code-style.md:57`; `standards/commit-message.md:103-104`;
  `standards/code-review.md:35`; `standards/clean-architecture.md:49,396`; `standards/code-style-rust.md:142,268`;
  `standards/crate-naming-convention.md:42,423`; `standards/ci-lanes.md:155`; `standards/testing.md:245`;
  `products/_TEMPLATE.md:56,315`; `products/foundry/PRD.md:731,1035`; `products/cloud/PRD.md:807`;
  `../../../../templates/checklists/pre-push.md:29`; `../../../../templates/checklists/pre-merge.md:21,26`; `../../../../templates/checklists/vertical-onboarding.md:19`.
registry/ + misc: `bounded-contexts.json:6`; `artifact-capabilities-registry.json:577`;
  `stub-audit/2026-05-17/adrs.jsonl:15,28,29,48`; `stub-audit/2026-05-17/missing-fitness-crates.json:106,899`;
  `milestone-audit/index.json:804-805,2545`; `stub-audit/2026-05-17/ips.jsonl:33`; `MISTAKES-LEDGER.md:56`;
  `CHANGELOG.md:438,441,453,454`; `README.md:1558`; `PRIVACY-PROGRAM.md:178`; `ADR-CONSOLIDATION-PLAN.md:61`;
  `ADR-LEGACY-REGRESSION-MAPPING.md:108`; `quality/ai-slop-defense/impossible-to-fail-environment-spec.md:69`,
  `…/ai-slop-failure-mode-catalogue.md:63`, `…/gap-analysis-ai-vs-production.md:40`.
  - **DO NOT rewrite the ~650 type-(b) NAMING comment-headers** (`// crates/oya-*-kernel`,
    `source_of_truth: crates/oya-*`) — they assert the crate NAME, which survives. Only rewrite the path
    when such a header becomes a real on-disk path (then to `{oya,cloud}/<svc>/crates/<crate>/`).

**FIX-FC-5 — Record the chain in the index/back-pointers (integrity).**
`docs/ADR-INDEX.md` has **0 rows** for ADR-0509 and ADR-0512 — add them (0512 is the canonical, founder-locked
governing ADR and must be reachable). Flip `docs/ADR-INDEX.md:317` (0357 still `Proposed`) and `:37`
(0015 still `accepted`). Add `superseded_by: [ADR-0512]` + flip status to Superseded on
`ADR-0509-...:4,10`. Add a `superseded_by: [ADR-0512]` field + flip status on `ADR-0357-...:3` (currently
no field, status Proposed) — note the founder may prefer to mark 0357 "Rejected/Withdrawn (never ratified)"
since it was Proposed, not Accepted; that is a founder call.

---

## 2. SUPERSESSION-COMPLETENESS REGISTER (A-INTEGRITY status-enum + edge sweep)

> This is exactly what a cross-artifact-agreement / supersession-completeness gate must enforce.
> Two sub-registers: **(2A)** status-vs-edge drifts (id → status fix) and **(2B)** directive-without-edge
> pairs (stale ADR → superseding ADR/ruling → edge to write).

### 2A. CONFIRMED status-vs-edge drifts (id → status fix)

Scope: 347 `ADR-*.md` scanned; **Case B (status Superseded + empty edge) = 0**. 7 drifts confirmed.

| # | ADR | drift (verified path:line) | status fix |
|---|-----|----------------------------|------------|
| 1 | **ADR-0015** | `status: accepted` (`:3`) + `superseded_by:[ADR-0131]` (`:5`, body `:13`); self-declared via `supersession_note` (`:6`) | flip `accepted`→`Superseded` (see FIX-FC-1) |
| 2 | **ADR-0316** | `status: Proposed` (`:3`) + `superseded_by:[ADR-0329]` (`:28`); ADR-0329 is Accepted | flip `Proposed`→`Superseded` (or resolve the Proposed: ratify/drop, then supersede) |
| 3 | **ADR-0358** | `status: Proposed` (`:3`) + YAML **block-list** `superseded_by: [ADR-0392, ADR-0408]` (`:9-11`) — missed by inline-only scans | flip `Proposed`→`Superseded` (§2 reversed by 0392/0408 per `amendment_note`) |
| 4 | **ADR-0482** | `status: Accepted` (`:3`) + `amended_by:[kubers-anchor-2026-05-28]` (`:13`) — **non-ADR token** (dangling) | fix the dangling amender to a real ADR id, or convert to a tracked amendment record; keep status or set `Amended` per policy |
| 5 | **ADR-0052** | **body-vs-frontmatter:** frontmatter `status: Superseded` (`:4`) + `superseded_by:[ADR-0118]` (`:11`) but BODY `> **Status:** Accepted` (`:29`) + `> **Superseded-by:** —` (`:32`) | fix the BODY to match frontmatter (Superseded → ADR-0118) |
| 6 | **ADR-0363** | `status: Accepted` (`:3`) + `amended_by:[ADR-0510, ADR-0513]` (`:10`) — `superseded_by` empty (`:9`) | judgment call: "Accepted + amended_by" may be acceptable, but the amenders are themselves stale (0513 Superseded); reconcile to a live amender or set `Amended` |
| 7 | **ADR-0054** | `status: deprecated` (`:3`) + body `Superseded by ADR-0116` (`:9`) + `> **Superseded-by:** ADR-0116` (`:13`) | flip `deprecated`→`Superseded`; add frontmatter `superseded_by:[ADR-0116]` (edge currently only in body) |

Correctly EXCLUDED (verified consistent, not drift): ADR-0120/0121 (`Superseded` + edge + body all agree),
ADR-0147 (`Amended`, in-body self-amendment, no outbound edge — internally consistent).

### 2B. CONFIRMED directive-without-edge pairs (stale ADR → superseding ruling/ADR → EDGE TO WRITE)

Class: an **Accepted** ADR rendered stale by a LATER Accepted ADR or a founder consolidation-ruling, carrying
**NO** `superseded_by`/`amended_by`/`supersedes`/`amends` edge — so the ADR graph lands a fresh reader on the
stale directive as live canon. **Flat-crates (§1) is one instance of this class.** 5 confirmed (P1–P5).

| # | Stale ADR (status, edge state) | Superseding ruling + Accepted ADR | EDGE TO WRITE |
|---|--------------------------------|-----------------------------------|---------------|
| **P2** | **ADR-0187** Zitadel-primary — Accepted (`:3`), `superseded_by:[]` (`:8`), title/decision "canonical/primary IdP" (`:17,37`). **HIGHEST SEVERITY** (D5 "hard contradiction C-4") | D5 `canon:31` (Zitadel demoted to vendored bridge; superseded-as-endpoint by 0476) + **ADR-0476** Accepted (`:4`) | **DOUBLE write:** (i) `ADR-0187.superseded_by` ← `[ADR-0476]` + status → Superseded-as-endpoint; (ii) **fix the 0421 mis-number:** `ADR-0476-...:9` currently `supersedes:[ADR-0421]` must become `supersedes:[ADR-0187]` (the promised edge never landed) |
| **P1** | **ADR-0160** Flagger — Accepted (`:3`), `superseded_by:[]` (`:8`), "Flagger 1.x canonical … Why Flagger over Argo Rollouts" (`:42,62`) | D10 `canon:62` ("Supersede Flagger (0160)", door: two-way) + **ADR-0515** Argo Rollouts canonical (`:80,83`; Accepted `:4`) | write `ADR-0160.superseded_by` ← `[ADR-0515]` (or amend) + add `ADR-0160` to `ADR-0515.supersedes` (`:9` currently omits 0160) + flip 0160 status |
| **P3** | **ADR-0374** Forgejo→Jenkins webhook gateway — Accepted (`:3`), `superseded_by:[]` (`:9`), "git+Jenkins+Forgejo substrate" (`:55-56`), "Jenkins-as-orchestrator" (`:188`) | D2 `canon:153` (supersede Forgejo ADRs incl. 0374) + D-FORGE-CLARIFY `canon:207` (Forgejo eradicated) + D-CICD/**ADR-0515** (oya-ci replaces Jenkins) | add `ADR-0374` to `ADR-0515.supersedes` (only in `related` of the now-Superseded ADR-0513 `:14,25`) + write `ADR-0374.superseded_by` ← `[ADR-0515]` + flip status |
| **P4** | **ADR-0380** Jenkins-farm-on-Talos + Forgejo gating — `Accepted (amendment)` (`:3`), `superseded_by:[]` (`:9`), title `:55` | same D2/D-FORGE-CLARIFY/D-CICD + **ADR-0515**. Source admits the gap: `ADR-0513-...:22-23` "formal supersession of ADR-0380 lands at the Phase-1 cutover" — never landed (0513 itself Superseded; 0380 absent from 0515.supersedes) | land the promised edge: add `ADR-0380` to `ADR-0515.supersedes` + write `ADR-0380.superseded_by` ← `[ADR-0515]` + flip status |
| **P5** | **ADR-0335** foundry→intelligence — Accepted (`:3`), **no** `superseded_by`/`amended_by`, "intelligence is the canonical AI substrate" (`:158,514`) | D-INTEL FINAL `canon:90` (RE-HOME the 96k-LOC engine DOWN from oya/intelligence into cloud/cloud-intelligence) | write a **"superseded-on-cutover (pending build)" marker** per the D-META ratchet rule (`canon:26`) — NOT immediate archival (D-INTEL is ratchet-sequenced, build-first, `canon:98-99`); ADR-0335 currently carries no marker at all |

Correctly EXCLUDED (verified): ADR-0010 (directive MIS-CITATION — `:7` is "Regional pack architecture", not
Argo; the real pair is P1); ADR-0195 vs ADR-0377-kafka (NOT a contradiction — 0195 already sources from Pulsar
`:71-72`, 0377-kafka supersedes ADR-0005 the real Kafka ADR and cites 0195 as the one that introduced KoP
`ADR-0377-kafka...:22,102`); ADR-0363 (edged via `amended_by:[ADR-0510,ADR-0513]` `:10` — stale-via-stale-chain,
not no-edge; already in §2A).

Secondary (same shape, but **Proposed** not Accepted → outside the strict lane; flagged for the sweep):
ADR-0387 (`:3` Proposed, Forgejo gateway), ADR-0377-forgejo (Proposed conditional, Forgejo + DUP-id), ADR-0347
(`:3` Proposed, foundry-fitness rename), ADR-0040 (proposed, D10 reconcile target), ADR-0510 (`:4` Proposed,
title "Forgejo transitory", already carries `amends:[ADR-0363]`).

---

## 3. PRINCIPLE (the procedure that, when skipped, IS the contradiction)

> **A later directive that moves away from an earlier ADR MUST, in the same act, (1) write the
> supersession/amendment edge (`superseded_by`/`amended_by` on the stale ADR AND `supersedes`/`amends`
> on the new one) AND (2) flip the stale ADR's `status` off `accepted`/`Accepted`/`Proposed`/`deprecated`.
> The ABSENCE of either is the contradiction — it is a procedure failure, not a content disagreement.**

Corollaries enforced by this register:
- **Both directions of the edge must exist.** P2 shows a half-edge is still a failure: even where a ruling
  *named* the successor (D5 → 0476), 0476's `supersedes` points at the wrong id (0421) — the graph still
  strands the reader on "Zitadel primary."
- **Status and edge are coupled invariants.** Case B (Superseded with empty edge) = 0 today; the live failures
  are all the forward direction (edge or ruling exists, status not flipped) — §2A items 1-7.
- **Ratchet/build-first moves still owe a marker.** Where the move is sequenced for a later cutover (P5,
  flat-crates-on-cutover), the owed artifact is a "superseded-on-cutover (pending build)" marker per
  `canon:26`, not immediate archival — but a marker MUST be present; "nothing at all" is the failure.
- **A NAME surviving is not a LOCATION surviving.** The flat-crates case shows the inverse hazard: ~650 refs
  were NOT contradictions (they assert the surviving NAME); only the ~50 LOCATION/gate refs are. The gate that
  enforces a superseded topology is the load-bearing failure, not the naming string.

### Confirmed-contradiction count

- **Flat-crates (the named exemplar):** 1 supersession chain fully mapped (5 ADRs) + **1 live BLOCKER gate
  enforcing the superseded topology** (highest severity) + 1 live untracked `crates/` dir embodying it +
  ~50 stale LOCATION/gate refs + 3 index/back-pointer integrity gaps (ADR-INDEX missing 0509/0512;
  0357 unmarked; 0509 no back-pointer).
- **Status-vs-edge drifts (§2A): 7** confirmed (6 forward Case-A + 1 body-vs-frontmatter; Case B = 0).
- **Directive-without-edge pairs (§2B): 5** confirmed Accepted-stale-without-edge (P1–P5), + 5 Proposed-status
  secondaries flagged.
- **TOTAL confirmed supersession-completeness contradictions: 12** (7 status-vs-edge + 5 directive-without-edge),
  of which flat-crates is the 1 instance whose downstream blast radius (gate + dir + ~50 refs + 3 integrity gaps)
  is separately enumerated in §1.
- **Adversarial verification substantive refutations: 0.** 3 minor citation-precision deltas only (none change
  a verdict): ADR-0374 status line is `:3` not `:6`; ADR-0131 naming string is literally `oya-<ms>-<bc>-<layer>`;
  ADR-INDEX resolves at `docs/ADR-INDEX.md`.
