---
id: ADR-0555
title: "Unaccounted artifacts are unmergeable: advisory→blocking accounting conversion + the structural accounting model"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-12
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amended_by: []
depends_on: [ADR-0515, ADR-0551]
amends: []
related: [ADR-0111, ADR-0363, ADR-0539, ADR-0541, ADR-0544, ADR-0546, ADR-0548, ADR-0550]
related_specs:
  - /specs/root-hub-pointers.json
  - /specs/reachability-registry.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0555: Unaccounted artifacts are unmergeable — advisory→blocking accounting conversion + the structural accounting model

## Status

**Proposed - 2026-06-12 (authored for founder sign-off; door: one-way).**

## Context

Two founder directives (2026-06-12, binding) drive this decision:

1. *"Drift, unaccounted code, file, docs should be structurally impossible by design."*
2. *"No CLI. Talos-style cloud native."*

The firewall's accounting codes `unowned` and `unreachable` were `advisory-until-infra`:
they EMITTED counts (the burn-down dashboard) but never flipped the verdict. PR #698's
independent review watched **+7 unowned / +6 unreachable keys ride through a merge
flagged-but-tolerated** — new unaccounted artifacts were dashboard rows, not merge
blockers. Per directive (1), an unaccounted artifact must be UNMERGEABLE: structural
impossibility at the admission boundary, not a dashboard. This is FRIC-1781330000.

The mechanics to convert safely already exist: ADR-0551's merge-base frozen baseline
(frozen-mode-wins; a same-PR flip cannot disarm; the sign-off door spans both predicates
for one regen), and PR #698's stated paved road for new blocking debt classes:
*"advisory-first + reviewed disposition flip remains the paved road"* — with the sign-off
door reserved for a NEW blocking class arriving with a non-empty initial baseline. This
conversion is the disposition flip of EXISTING advisory codes whose keys are already
captured in the baseline face, so the pre-existing debt is **grandfathered mechanically**
by the frozen reference — no sign-off entry is needed or used; the door stays reserved for
genuine growth exemptions.

Precedent (proven patterns, Rust reimplementation per standing doctrine): Google/Chromium
**OWNERS files** make ownership a structural property of the tree (nearest-ancestor
resolution), enforced at review/admission, not advisory; GitHub **CODEOWNERS** is the same
shape. **Betterer / eslint-ratchet** freeze existing debt against the merge-base and block
only growth. **Bazel** anchors target determination on the merge-base. **Kubernetes
Prow/Tide** holds merge authority in gate services behind required contexts (ADR-0515).
**GitOps reconciliation** (Argo/Flux; Talos Linux's API-driven, no-shell management model)
makes drift convergence a control loop over declared state — the destination surface model
of D4.

## Decision

### D1 — Convert the exists-but-unaccounted codes to blocking, grandfathered at the merge-base

The disposition table (`libs/oya-ci-config/src/bundled/gate-disposition.json` — DATA, not
code) flips:

| gate | code | was | now | live keys grandfathered |
|---|---|---|---|---|
| cloud-ci-total-accounting | `unowned` | advisory-until-infra | **baseline-block-on-new** | 16,924 (pre-seed) |
| cloud-ci-total-accounting | `unreachable` | advisory-until-infra | **baseline-block-on-new** | 14,119 (pre-seed) |
| cloud-ci-total-accounting | `no_ttl_class` | advisory-until-infra | **baseline-block-on-new** | 0 |
| cloud-ci-staleness-reaper | `untyped_staleness` | advisory-until-infra | **baseline-block-on-new** | 0 |
| cloud-ci-staleness-reaper | `stale_over_budget_unreachable` | advisory-until-infra | advisory-until-infra (**stays**) | 54 |

`stale_over_budget_unreachable` stays advisory **by design, not by debt**: its keys enter
by TIME passing (a file ages past its TTL budget on everyone's clock), not by PR action.
Blocking-on-new for a time-driven set blames PRs for age accrued elsewhere — the
archetypal unfair brick. Its convergence surface is the staleness-reaper reconciler
(report → `git mv` → `_archive/`, second-verifier-gated), and — after this conversion —
no NEW artifact can even enter its population unregistered, because `unreachable` blocks
at creation. Its `infra_prereq` is renamed to `staleness-reaper-archival-reconciler` to
state the true prerequisite.

**Grandfather mechanics (no new machinery — ADR-0551 verbatim):** the per-code mode is
read from the FROZEN merge-base baseline. In the PR that carries the flip, the frozen
modes are still advisory, so the flip is inert for its own PR (it cannot brick itself,
and a hostile same-PR flip cannot disarm anything — frozen-mode-wins). The conversion
**arms at the first merge-base advance after the flip merges**: from then on the frozen
reference carries `mode: baseline-block-on-new` plus the full grandfathered key set.
Pre-existing debt is tolerated (`current ∩ frozen`); a NEW key is a compare-mode
regression AND un-signed-off ratchet growth (the settle regen carries it into the
proposed face) — RED on both predicates. Burn-down is shrink-only: registering an
artifact removes its key at the next regen, and the frozen reference shrinks at the next
merge-base advance. Zero growth without the founder door.

### D2 — Registration is automated where derivable; a FAIL is never a bare flag

Founder: flagging/red-gating isn't enough. The accounting derivation is already
mechanical where it can be:

- **ownership** derives from nearest-ancestor `OWNERS` files (Google/Chromium pattern);
- **reachability** derives from workspace-member containment (`cargo-members`), exact
  mentions in `specs/masterplan.json` / `specs/root-hub-pointers.json` /
  `docs/DOC-CATALOG.md`, and — new in this decision — the reviewed
  **`specs/reachability-registry.json`**: explicit `{prefix, anchor}` registrations of
  whole trees (dir prefixes) or exact paths, each naming WHY the tree is reached.
  Registration is a review-visible design act (the ADR-0551 trust class, same as
  `ratchet-policy.json`), never a silent exemption; the producer fails LOUD on a
  malformed entry.

What remains at FAIL time is precisely the **non-derivable residue** — *who owns this?*
/ *what should point at this?* are design acts no tool may invent (ADR-0548 D7 soundness;
the BUCK-fixer refusal precedent). Therefore:

1. **The FAIL output prints the exact registration edit** (or the precise decision
   needed). Remediation text is DATA in the disposition table, stamped per-code into
   `gate-baseline.generated.json` by the producer, carried into the firewall report, and
   printed by the merge-authority test next to the offending keys — never a bare flag.
2. **`--fix` applies the decided edit and self-validates.** The producer binary gains two
   TRANSITIONAL bridges: `--fix-owners <dir>=<owner>` (writes `<dir>/OWNERS`, then
   re-runs the ownership derivation over tracked ∪ the new file and reports coverage)
   and `--fix-reachability <prefix>=<anchor>` (appends the sorted registry entry, then
   round-trips the fail-loud loader and reports coverage). Both take the human decision
   as INPUT; both refuse bare/duplicate registrations. Automation applies edits — it
   never invents decisions.
3. **Seed registrations land with this decision** so the conversion arms with the common
   PR shapes already tractable (and as live burn-down proof): the ownership markers
   `docs/decisions/OWNERS` + `docs/standards/OWNERS` + `specs/OWNERS` (council-architecture —
   the dominant decisions/standards owner in ADR front-matter, incl. this ADR, and a `specs` `owner_team` value) and
   `ci/OWNERS` +
   `third-party/OWNERS` + `evidence/OWNERS` (cloud-ci-platform — the owner the existing
   `.omc/ultragoal/OWNERS` precedent names); registry prefixes for `docs/decisions/`
   (crosswalk-accounted), `specs/fixtures/` (dir-loaded data-under-test), `third-party/`
   (lockfile-derived), `evidence/` (TTL-accounted). The enforcement-liveness gate's
   hermetic clean-checkout producer fixture
   `ci/facade/hook-wiring/tests/scm-facts.fixture.json`
   is a born-accounted data-under-test artifact under the cloud-ci gate tree, consumed by
   the enforcement-liveness Buck gate to avoid relying on an absent generated SCM face.
   New artifacts OUTSIDE these trees must register in their own PR — the paved road is
   one OWNERS file + (where applicable) one registry line, both printed verbatim by the
   failing gate.

Hardening (2026-06-12, FRIC-1781400000 — closes the two acknowledged weaknesses from
the PR #704 independent review, MED-2; amended in place per the ADR-0551 precedent for
hardening a same-cycle decision):

- **OWNERS content schema (ownership was EXISTENCE-ONLY).** Ownership now requires
  existence AND valid content. The minimal schema codifies what the live corpus already
  does (all 15 OWNERS files at codification): each line, after trimming, is empty
  (ignored), a `#` comment (ignored), or an **owner principal** — a lowercase
  DNS-1123-label-shaped team identifier (`[a-z0-9]` plus interior `-`, 1..=63 chars; the
  K8s name shape, e.g. `cloud-ci-platform`). A valid file carries **at least one
  principal** and zero unparseable lines. Fail-closed: an empty, comment-only, garbage,
  non-UTF-8, or unreadable OWNERS file is NOT ownership — and it still **poisons
  resolution at its directory** (no fall-through to a broader valid ancestor), so invalid
  content can never yield owned rows and corrupting an existing OWNERS surfaces as NEW
  unowned keys (firewall RED) instead of being silently absorbed by a parent
  registration. RED/GREEN data-under-test corpus (dir-loaded by the producer's
  `owners_schema_fixtures_execute_red_green_cases`):
  `specs/fixtures/owners-schema/tc-OWN-bad-empty-file.json`,
  `specs/fixtures/owners-schema/tc-OWN-bad-comment-only.json`,
  `specs/fixtures/owners-schema/tc-OWN-bad-garbage-content.json`,
  `specs/fixtures/owners-schema/tc-OWN-bad-invalid-poisons-no-fall-through.json`,
  `specs/fixtures/owners-schema/tc-OWN-bad-over-broad-excess-unowned.json` (the
  breadth-bound RED exhibit),
  `specs/fixtures/owners-schema/tc-OWN-good-single-principal.json`, and
  `specs/fixtures/owners-schema/tc-OWN-good-comments-and-multiple-principals.json`.
- **Breadth bound (registrations were BREADTH-UNLIMITED).** A single OWNERS file's
  nearest-ancestor coverage is capped by the policy-as-data bound
  `[owners] max_paths_per_owners_file` (`oya-ci.toml`, reviewed TOML — the ratchet-policy
  trust class; a zero bound is structurally rejected by the closed-schema loader).
  Default **2000**, sized from the measured live distribution (2026-06-12: max legitimate
  coverage 886 paths = `registry/catalog/OWNERS`, next 387 = `docs/decisions/OWNERS`, of
  18,400 tracked) — >2x headroom for legitimate growth while a root/`cloud/`-level bulk
  OWNERS claiming thousands is caught. Exceeding the bound leaves the **excess paths
  unowned** (the first `<bound>` covered paths, path-sorted for deterministic
  regeneration, keep ownership) with the remediation *split the registration into
  narrower subtree OWNERS files* — a subtree OWNERS + ADR mention + registry line can no
  longer bulk-neuter a tree's unowned leg.
- **Automation (flag-isn't-enough).** `--fix-owners` validates the principal against the
  schema before writing (it can only EMIT valid OWNERS content), self-validates through
  the now content-aware derivation, and refuses an over-broad registration with no
  residue. The producer prints an `owners integrity:` line naming the exact fix for every
  invalid or over-broad OWNERS file, and the `unowned` remediation DATA
  (gate-disposition table → stamped into `gate-baseline.generated.json` → printed by the
  firewall) carries the schema and the split fix verbatim.
- **Zero live-corpus regression.** All live OWNERS files parse valid and sit under the
  bound (pinned by `live_owners_corpus_is_schema_valid_and_under_breadth_bound`), so the
  conversion's grandfathered baseline does not grow from this hardening.

### D3 — The structural model: every artifact class is SOURCE-OF-TRUTH or DERIVED

Every repo artifact class is exactly one of:

- **(a) SOURCE-OF-TRUTH** — registered/accounted at creation; admission is structurally
  impossible otherwise. This conversion is the ratchet step for (a).
- **(b) DERIVED** — generated from sources; drift impossible by regeneration
  (committed==regenerated byte parity; the faces pattern, docs-as-build-artifacts).

Honest map of the CURRENT classes:

| artifact class | model | accounted by (today) | gap / migration ratchet |
|---|---|---|---|
| code (`*.rs` under members) | (a) | cargo-members reachability (structural) + OWNERS + ADR justification (`unjustified` blocking) | OWNERS coverage is partial: 5 seed trees; remainder grandfathered, burns down per-PR via D2 |
| BUCK / Cargo manifests | (a) | target-parity (ADR-0540), workspace-glob-coverage (ADR-0538), manifest-hygiene, freshness (ADR-0539) | BUCK *content* validation is lexical; ADR-0549 buck-syntax kernel is the successor |
| `docs/decisions/` (ADRs) | (a) | self-justifying + crosswalk-accounted (orphan/unpropagated/status are BLOCKING cross-artifact codes); reachability via registry prefix; OWNERS seed | `unpropagated_decision` carries 151 grandfathered keys — its own burn-down |
| `docs/*` prose | (a) | DOC-CATALOG protocol + `unreachable` now blocking for NEW files | existing prose grandfathered (~5.6k unreachable keys); per-file registration on touch |
| `specs/*.json` | (a) | root-hub/masterplan mentions + registry; OWNERS seed (`specs/`) | fixtures registered as a tree (consuming tests are the pointer); per-fixture pointers = ADR-0541 graph edges |
| configs (`oya-ci.toml`, policies) | (a) | closed-schema loaders (fail-loud) + canonical-json (ADR-0546) | — |
| workflows (`.github/`) | (a) | gate-registration meta-test (completeness invariant) | OWNERS not seeded; new workflow files print the D2 decision |
| `evidence/` | (a) | OWNERS seed + registry prefix + TTL accounting (staleness budgets) | archival reconciler (D4) is the decay sink |
| `*.generated.json` faces | (b) | registry-drift + freshness byte-parity (committed==regenerated); under member dirs ⇒ structurally reachable | a *foreign* hand-written `.generated.json` is structurally UNTRACKABLE (`**/*.generated.json` is gitignored by default; verified adversarially); a force-add is governed by `registry/generated-artifact-control-plane.json` (owner/generator/materialization declared, Rust gate authoritative) |
| `ephemeral` class (`.omc/state/`, run scratch) | excluded BY REVIEWED DATA | carved out by class in `unit-class-policy.json` — not part of the durable corpus, so it carries no registry row | the carve-out table itself is reviewed, drift-checked DATA; widening it is a visible policy edit |
| `third-party/` vendor | (b) | derived from `Cargo.lock` via reindeer; registry prefix + OWNERS seed encode that derivation | — |
| `husk` (unclassified catch-all) | neither — the residue | grandfathered (~6.9k keys); NEW husk files now unmergeable unregistered | every husk key burned down is a classification decision; the class itself must trend to zero |

**Destination substrate:** ADR-0541's Corpus Liveness Graph — the whole corpus as one
content-addressed AST graph where every decay symptom (unowned, unreachable, stale,
drifted, unpropagated) is a graph pathology detected by fail-closed invariants, and
docs/directives are build artifacts. This decision's registries (OWNERS files, the
reachability registry, the disposition table) are the explicit edge declarations that
graph ingests; the conversion makes their absence unmergeable so the graph never inherits
an unaccounted node.

### D4 — Surface model (the Talos directive): gates + reconcilers are canonical; CLI bridges are transitional

Normative, so no future gate ships CLI-first:

1. The canonical enforcement surface is the **gate services behind the single required
   context `oya-ci-required`** (ADR-0515), and the canonical automation surface is the
   **K8s operator/CRD direction of ADR-0548 D3** — `GatePolicy` / `Baseline` /
   `Exception` / `GateRun` — with **reconciliation loops as the drift-convergence
   mechanism** (the registration registries and baselines of this decision are exactly
   the declared state those reconcilers converge on; the staleness-reaper archival loop
   is the first nominated reconciler).
2. Every `--fix` / `--verify` binary — including this decision's `--fix-owners` /
   `--fix-reachability`, the face-settle `--verify`, the embedded-asset fixer, and the
   canonical-json `--fix` — is a **TRANSITIONAL local bridge** under `cli_surface_policy`
   (retirement-marked; local feedback only; NEVER merge authority). Its named successor
   is a reconciler over the same DATA. A new gate MUST land as
   gate-test-plus-policy-DATA first; a bridge binary is an optional convenience added
   after, never the enforcement surface.

## Verification (RED/GREEN, data-under-test)

- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-new-unowned-file-blocked.json` — a NEW
  unowned file is RED on BOTH predicates in the armed configuration (remediation carried).
- `specs/fixtures/cloud-ci-firewall/tc-FW-bad-new-unreachable-file-blocked.json` — a NEW
  unreachable file is RED on BOTH predicates.
- `specs/fixtures/cloud-ci-firewall/tc-FW-good-converted-debt-tolerated-and-burns-down.json`
  — grandfathered keys tolerated; burn-down shrinks; GREEN.
- `specs/fixtures/cloud-ci-firewall/tc-FW-good-advisory-code-reports-not-fails.json` +
  `tc-FW-good-advisory-growth-allowed.json` — repointed to the surviving advisory
  exemplar (`stale_over_budget_unreachable`), matching the live disposition truth.
- Live pin `converted_accounting_codes_block_new_keys_when_armed`
  (`ci/facade/baseline-ratchet/tests/firewall.rs`): the LIVE producer
  stamps the flipped modes + remediation; armed frozen reference tolerates today's corpus,
  goes RED on a synthetic NEW unowned file, a NEW unreachable file, and the same-PR regen
  laundering shape.
- Producer self-tests: registration-registry fail-loud parsing + exact prefix matching;
  `--fix-owners` / `--fix-reachability` apply-and-self-validate (and refuse bare
  decisions); disposition stamping asserts blocking modes + remediation presence.
- `firewall_is_green_on_the_live_corpus_with_the_baseline` stays GREEN throughout (the
  flip cannot brick the PR that carries it, nor clean dev).
- Adversarial harness (run pre-merge, evidence in the converting PR): four live attack
  variants against the ARMED reference — plain unaccounted file, registry-prefix
  laundering, OWNERS laundering, `.generated.json` class laundering — each unmergeable
  (RED on ≥1 blocking code, or structurally untrackable), with byte-identical revert.
- Ledger row FRIC-1781330000 (friction-accounting gate enforces its closure evidence).

## Consequences

- **Day-2 behavior change:** one merge after this lands, a PR adding ANY file that is not
  ownership- and reachability-registered is unmergeable, with the exact registration edit
  printed by the failing required check. The paved road for the common shapes (new ADR,
  new fixture, new gate crate, vendored dep, evidence) is pre-seeded; a genuinely new
  tree costs one OWNERS file + one registry line in the same PR — the founder-intended
  registration-at-creation act.
- In-flight sibling PRs are NOT bricked retroactively: their merge-bases predate the flip
  (advisory frozen modes) until they rebase; after rebasing they register their new files
  via the printed edits.
- The registration registry and OWNERS files are PR-editable, review-visible policy
  surfaces (the ADR-0551 trust class). This decision closes the structurally INVISIBLE
  channel (unaccounted-by-default); a hostile-but-visible registration is a review
  problem, and frozen-policy-wins remains the named follow-up hardening (ADR-0551).
- The `husk` and prose grandfathered sets are large (≈6.9k + ≈5.6k keys); they shrink
  only by deliberate classification/registration. That is the intended ratchet — visible,
  shrink-only, never growable.
- `unaccounted` (the no-registry-row code) remains structurally empty by producer
  construction (every tracked path gets a row); the real exists-but-unaccounted classes
  are exactly the ones this decision converts.

## Alternatives considered

1. **Sign-off door for the initial freeze (treat the flip as a new blocking class).**
   Rejected: the codes already exist in the merge-base baseline with their keys; the F4
   paved road (PR #698) is the reviewed disposition flip with mechanical grandfathering —
   the door stays reserved for genuine growth exemptions, keeping it small and auditable.
2. **Blanket class exemptions instead of registration (e.g. exempt `generated`/`vendor`
   from unowned).** Rejected: name-a-file-`.generated.json`-and-skip-accounting is a
   laundering vector; the reviewed registry names WHY a tree is accounted and is
   itself drift-checked DATA.
3. **Per-class disposition granularity (mode per (gate, code, unit_class)).** Rejected
   for now: redesigns the baseline schema for a need the prefix registry covers with
   review-visible DATA; revisit inside the ADR-0541 graph where class is a node property.
4. **Also convert `stale_over_budget_unreachable`.** Rejected: time-driven sets blame the
   wrong PR (D1); its sink is the archival reconciler, and `unreachable`-at-creation now
   starves its growth.
