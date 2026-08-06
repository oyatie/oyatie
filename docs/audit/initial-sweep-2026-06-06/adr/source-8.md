# ADR Audit — SOURCE chunk 8

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 8
- **Slice range requested:** lines 50–56 of sorted `docs/decisions/ADR-*.md`
- **ADRs actually reviewed (7):** ADR-0052, ADR-0053, ADR-0054, ADR-0055, ADR-0056, ADR-0057, ADR-0058
- **Auditor posture:** READ-ONLY. Trust the *superseding* ADR over stale front-matter (keystone-map rule). This chunk is dominated by the **grit/icm cutover cluster (0052–0054)** — all retired by ADR-0116 / ADR-0118 — and the **v4 rename-plan / clean-arch cluster (0055–0058)**, whose *decisions* are largely TRUE and masterplan-relevant even though their *mechanism* references retired tooling.

---

### ADR-0052 — Canonical inventory ledger for the grit/icm cutover

- **decision_atom:** Before any artifact is archived or deleted in a tooling cutover, a single committed ADR-embedded ledger must classify every in-scope file/dir by a closed action set (KEEP/ANNOTATE/ARCHIVE/DELETE/REPLACE/FLAG); "inventory precedes deletion."
- **current_status:** front-matter `status: Superseded`, `superseded_by: [ADR-0118]`. (Body §Status still literally reads "Accepted" — internal contradiction; front-matter wins.)
- **disposition:** ARCHIVE.
- **governing:** ADR-0118 (retire archive-orphan fitness lane; `supersedes: [ADR-0052]`) — itself downstream of ADR-0116 (retire grit/icm/rtk/vox). The whole cutover this ledger served was undone.
- **truth_flag:** STALE. The *principle* ("inventory precedes deletion," archive-before-delete, rollback safety) is TRUE and durable; the *content* (223 rows of `bominal/`+`oyatie/` paths, grit/icm replacement classes, `oya-governance-inventory-tracker` lane) is dead — references the retired grit/icm cutover, retired `foundry` owner, and a `bominal/` repo layout that no longer governs.
- **in_masterplan:** NO. No `planning_impact`/`masterplan_ref` front-matter. The durable atom (archive-first deletion discipline) is masterplan-worthy but is not currently bound; the ADR's body is too tied to a dead cutover to backfill verbatim.
- **tensions:**
  - Body §Status says "Accepted" while front-matter says "Superseded" — direct self-contradiction.
  - Owner `foundry` is RETIRED vocabulary (ADR-0335; founder: cloud-intelligence/governance). Lane name already migrated to `oya-governance-inventory-tracker`, but owner field did not.
  - Cites ADR-0025 as "Foundry as engineering platform" — that framing is superseded by the foundry→intelligence/governance split (ADR-0335/0347).
  - Cites ADR-0015 flat-crates as live; ADR-0015 is itself PARTIAL-superseded by ADR-0131 (keystone §1.1) — the crate-layout half it leans on survives, so this cross-ref is still safe.
- **hyperscaler_challenge:** ALIGNED-on-principle / misaligned-on-form. Google/AWS/Azure absolutely gate destructive migrations behind an inventory+classification manifest (change-management/atomic-migration discipline). But none would encode a 223-row file manifest *inside a decision record*; they'd use a machine-readable artifact + tooling (exactly Alt-2, which this ADR rejected). Argues for ARCHIVE the instance, KEEP the principle elsewhere.
- **ai_slop:** Moderate. Fabricated-precision counts ("201 KEEP / 5 ANNOTATE / 15 ARCHIVE / 2 DELETE … TOTAL 223"), ceremony around a phantom path (`oyatie/.omx/ultragoal/`), and a self-aware "Critic iter-2 finding" framing. The ledger is a point-in-time scratch artifact dressed as a permanent decision.
- **refinement:** Archive to the frozen series. If the founder wants the durable rule, lift a one-line masterplan invariant ("destructive migrations require a committed pre-move classification manifest; archive-before-delete with rollback boundary") and drop the 223-row table.
- **consensus_needed:** no (cleanly superseded; archive).

---

### ADR-0053 — grit + icm + oya-tooling-agent-read as sole sanctioned agent primitives

- **decision_atom:** Agents coordinate and mutate state only through a fixed, audited primitive set (no raw `git`/`gh`); every coordination/state action emits to the audit chain, and extending the set requires an ADR.
- **current_status:** front-matter `status: Accepted`, `superseded_by: []` (STALE — never updated). De-facto retired: ADR-0116 names ADR-0053 a "historical inventory of the retired primitives"; its own landing-evidence block admits "superseded by ADR-0116 retirement so no further emissions expected."
- **disposition:** ARCHIVE (de-facto superseded; front-matter drift).
- **governing:** ADR-0116 (retire external agent-coordination tooling grit/rtk/icm/vox). The concrete primitives (`grit`, `icm`, `oya-tooling-agent-read`) are all retired.
- **truth_flag:** STALE. The *meta-principle* survives and is strong (agents act only through audited, provider-agnostic primitives; no un-audited `git`/`gh`; reshape data so special cases vanish — Linus discipline). The *named tooling* (grit/icm + the GitHub-CLI-wrapping helper) is dead.
- **in_masterplan:** PARTIAL. No `planning_impact` front-matter, but the principle (audited-primitive-only agent surface, provider-agnostic verb layer) directly underpins the canonical CI/governance posture (oya gate engine, ADR-0513/0514) and is arguably already live there under different names. The *binding* is absent.
- **tensions:**
  - `superseded_by: []` contradicts ADR-0116's explicit retirement and this ADR's own landing-evidence admission — a stale-front-matter drift exactly of the ADR-0136 class flagged in keystone §6/§1.3.
  - Owner `foundry` = retired brand.
  - "Provider-agnostic: helper wraps `gh` (GitHub CLI) today, swappable to GitLab/Gitea later" — collides with the FORGE fault-line (keystone §5): source canon is Forgejo-canonical→bespoke-VCS-destination (ADR-0363/0510), founder directive is GitHub. This ADR bakes GitHub-CLI as the day-0 substrate, which is consistent with the founder directive but inconsistent with source forge canon.
  - Depends on upstream `rtk-ai/grit` 0.3.0+ with a documented `grit session start` blocker — a hard external dependency the project later chose to eliminate (ADR-0116).
- **hyperscaler_challenge:** ALIGNED-on-principle / questionable-on-build-vs-buy. Google (Critique/Piper), AWS, Azure all enforce that automation acts through audited internal tooling rather than raw VCS CLI — strongly aligned. But a hyperscaler would NOT take a hard dependency on two third-party tools (`rtk-ai/grit`, `icm`) for its core agent-coordination substrate; it would own that surface (which is precisely the direction ADR-0116 then took). Argues for ARCHIVE + re-express the principle natively.
- **ai_slop:** Moderate. "Compounding principles incorporated by reference" (distroless image discipline, LTS-pinning, final-shape adoption) is padding only loosely connected to the actual decision; the ULID landing-evidence (`EVT-ADR-LAND-0053-01HXXMKPRGRITICM…`) is fabricated-precision ceremony.
- **refinement:** Set `superseded_by: [ADR-0116]` and `status: superseded` (drift fix — would be an AMEND if amendments were in scope; this pass is read-only). Preserve the meta-principle as a masterplan invariant: "agents mutate state only through audited, provider-agnostic primitives; no raw git/gh in agent paths."
- **consensus_needed:** no for the retirement; the surviving principle is non-contested.

---

### ADR-0054 — grit scaffold-claim pattern (icm-coordination-lock fallback)

- **decision_atom:** New-crate (and later rename) coordination among parallel agents uses an explicit open/close coordination-lock window before scaffolding, because the lock tool cannot lock symbols that don't exist yet.
- **current_status:** front-matter `status: deprecated`; body cleanly records "Superseded by ADR-0116 (2026-05-16)." This is the **best-formed retirement in the chunk** — front-matter, blockquote, and §Status all agree.
- **disposition:** ARCHIVE (cleanly deprecated/superseded).
- **governing:** ADR-0116 (`supersedes: ADR-0054`). New-crate scaffolds now use plain `git mv` in a per-agent worktree feeding the (then-canonical) Foundry pipeline.
- **truth_flag:** STALE. Correct and honest for its moment (the Lane-3 empirical finding that `Cargo.toml::workspace_members` returns zero matches in grit v0.3.0 is a real, well-grounded fact), but the entire mechanism is retired. The 2026-05-13 amendment (extend scaffold-claim to ~140-crate rename events) is what bridges this ADR to the 0056/0057 rename cluster.
- **in_masterplan:** NO. Pure mechanism ADR; no durable masterplan atom beyond "concurrent agents need an explicit coordination lock for create/rename ops," which is subsumed by whatever the current canonical pipeline provides.
- **tensions:**
  - Forward-references its own replacement chain: ADR-0116 retires it, but ADR-0116's replacement (Foundry pipeline ADR-0110/0112/0113) is itself later retired by ADR-0363 — so this ADR is two supersession-hops from current truth.
  - The amendment couples 0054 to ADR-0056/0057 (rename cutover) — those rename *decisions* survive even though 0054's lock *mechanism* did not, so the cluster must be split when archiving.
  - Owner `foundry` = retired brand.
- **hyperscaler_challenge:** QUESTIONABLE. The chicken-and-egg is real, but it is an artifact of choosing an external symbol-locking tool (grit) whose model can't represent not-yet-existing crates. A hyperscaler with an owned monorepo VCS (Piper/Mononoke-class) would not hit this; path-level/workspace-level locking is table stakes. The whole ADR is evidence the third-party-tool choice was wrong — argues for ARCHIVE (which already happened).
- **ai_slop:** Low–moderate. The worked 7-step example + per-crate ICM rows are operationally concrete (good), but the volume of "file upstream grit issue" follow-ups and the amendment's per-crate row enumeration is heavier than the decision warrants.
- **refinement:** None — leave archived. If the durable lesson is wanted: "owned VCS must support path/workspace-level locks for atomic multi-crate renames" → a constraint on the bespoke-VCS destination (ADR-0510).
- **consensus_needed:** no.

---

### ADR-0055 — Object Graph renamed to Ontology

- **decision_atom:** The typed-entity information layer is canonically named **Ontology** (Palantir-aligned: typed entities + links + actions + functions with audit-chain/RLS/jurisdiction overlays), retiring the "Object Graph"/"OG" vocabulary.
- **current_status:** front-matter `status: accepted` (minimal front-matter: only `id`, `status`, `doc_status`).
- **disposition:** KEEP (current, correct, non-conflicting). This is a live naming-canon decision that the keystone map itself relies on (RETIRED-VOCAB table: object-graph → ontology, governed by ADR-0055/0122/0130).
- **governing:** n/a (governing, not governed). Reinforced by ADR-0122 (ontology-crate-rename) and ADR-0130 (deprecate knowledge-graph-registry).
- **truth_flag:** TRUE. The rename is canon and corpus-wide enforced (`oya-check-glossary` hard-fails on "Object Graph" tokens).
- **in_masterplan:** PARTIAL. The *term* "ontology" is canonical and pervasive (masterplan domain table, intelligence/data layer). But this ADR carries NO planning front-matter (`planning_impact`, `supersedes`, `masterplan_ref` all absent) — so under the drift-prevention design it is part of the 91% unbound ADR mass. A naming-canon decision this load-bearing should be bound.
- **tensions:**
  - **NUMBER-COLLISION (NOT in keystone §6.1).** ADR-0057's front-matter declares `supersedes: docs/adr-archive/ADR-0055-object-graph-renamed-to-ontology.md` and its body "Supersedes ADR-0055 (v3-era rename plan ADR)." **No such file exists on disk** — the only on-disk ADR-0055 is THIS ontology-rename. So the number 0055 was reused: a prior `ADR-0055-rename-plan-v3-cutover` was (re)allocated/overwritten by the ontology rename, leaving ADR-0057 with a **phantom/stale supersession pointer**. This is a genuine collision/dangling-ref to surface alongside the keystone's known ADR-0377 duplicate.
  - References "Bominal ADR-0106/0107/0132" and `[[feedback-…]]` wiki-links — the `bominal/` inheritance layer; verify those still resolve post-consolidation.
- **hyperscaler_challenge:** ALIGNED. Adopting an established industry term (Palantir "Ontology") over a home-grown one ("Object Graph") is exactly the naming discipline a large eng org enforces via a glossary lint. No concern.
- **ai_slop:** None of substance. Tight, decision-shaped, with a real scope table and CI-enforcement hook.
- **refinement:** Add planning front-matter and bind to masterplan (it is a canonical term). Resolve/annotate the ADR-0057→0055 phantom supersession (either ADR-0057 should point at the actual retired v3 ADR id, or note the 0055 number was reused).
- **consensus_needed:** no on the rename itself; **yes** on the meta-question it exposes — see consensus question on number-reuse/collision policy.

---

### ADR-0056 — Rust Clean Architecture BNF v4.1 (flat microservice grammar + 12-layer enum)

- **decision_atom:** Every Rust crate is named `oya-<microservice>(-<bc-tokens>)?-<layer>` with a closed 12-value layer enum and a registry-validated microservice slot; ports live in `kernel`, and clean-architecture/dependency-direction rules are mechanically CI-enforced.
- **current_status:** front-matter `status: Accepted`, `authority_tier: 2`, `length_cap: 500`, `superseded_by: ~`. Richest front-matter in the chunk.
- **disposition:** KEEP (with one AMEND-flag for retired-vocab leakage in examples).
- **governing:** n/a (live standard). Companion to ADR-0015/0017 (flat crates, `oya-` prefix) and ADR-0058 (flat catalog).
- **truth_flag:** TRUE (PARTIAL on examples). The grammar + 12-layer enum + port-in-kernel + registry-validation are sound, durable, and the de-facto crate-naming canon. PARTIAL because several worked examples use **retired-brand `foundry`** (`oya-foundry-grit-cli`, `oya-foundry-icm-cli`, `foundry = { owner = "council-foundry" }` in the registry) — `foundry` brand is RETIRED (ADR-0335; → intelligence/governance) and `grit`/`icm` are retired tooling (ADR-0116). The grammar is right; the examples leak dead vocabulary.
- **in_masterplan:** PARTIAL. `authority_tier: 2` + `canonical_authority: docs/CONSTITUTION.md` is real authority wiring, but no `planning_impact`/`masterplan_ref`. The naming BNF is a cross-cutting invariant the masterplan should bind.
- **tensions:**
  - Retired-vocab leakage: `foundry`/`grit`/`icm` in examples and the microservice registry (keystone §2 lint signal: residual `oya-foundry-*` in *new* work = retired-vocab leakage).
  - Registry lists `foundry = { owner = "council-foundry" }` as a live microservice — contradicts ADR-0335 (foundry-µsvc retired, absorbed by intelligence).
  - `policy` registered as a microservice — fine for source (Cedar/Kubewarden), but note the LINUX owned-policy fault-line (keystone §5.2).
  - `length_cap: 500` front-matter vs an ADR that is already long — a self-imposed budget worth checking.
- **hyperscaler_challenge:** ALIGNED. A closed layer enum + mechanically-enforced dependency direction + registry-validated service names is exactly how Google/AWS-scale monorepos prevent architectural rot (Bazel visibility, layering lints). Strongly aligned; this is one of the better-engineered ADRs in the corpus.
- **ai_slop:** Low. Dense but substantive. Mild over-specification (14-lane enforcement matrix, `--report-only`→BLOCKER flip dates) but all decision-relevant.
- **refinement:** Scrub `foundry`/`grit`/`icm` from examples and the registry; replace with `intelligence`/`governance` and current tooling. Add masterplan binding. Confirm the slot2 cardinality note ("1..3 tokens") matches the BNF (`microservice ::= kebab-token ( "-" kebab-token )*` is unbounded in the grammar but the prose says 1..3 — minor grammar/prose mismatch).
- **consensus_needed:** no on the BNF; the foundry-in-registry leak is a mechanical cleanup, not a contested ruling.

---

### ADR-0057 — Cutover Mechanics: Rename Plan v4 (Hybrid C)

- **decision_atom:** The ~140-crate rename to BNF v4.1 executes as Shard 0 (pure-tooling precursor) + a single atomic Shard 1 squash-merge (one Cargo.lock event) gated on a 48 h freeze and 4-partition reviewer sign-off, dropping the v3 fitness/freeze/expedite machinery in favor of the existing claim-lock.
- **current_status:** front-matter `status: Accepted`, `authority_tier: 2`, `length_cap: 300`, `supersedes: docs/adr-archive/ADR-0055-object-graph-renamed-to-ontology.md`.
- **disposition:** AMEND (sound decision, but stale/dangling supersession pointer + retired-mechanism coupling) → trending ARCHIVE once the rename is confirmed executed (it is a one-shot cutover-mechanics ADR; its value is historical record after Shard 1 landed).
- **governing:** n/a as governed (it governs the rename), but it is **mechanism-coupled to retired tooling**: its freeze/lock substrate is grit `claim` (retired by ADR-0116), and its emergency lane uses `icm store` + `gh pr merge --admin` (retired primitives + the forge-CLI fault-line).
- **truth_flag:** PARTIAL. The *topology decision* (atomic single-lockfile rename, partitioned review, drop redundant freeze machinery) is TRUE and well-reasoned. But: (a) `supersedes:` points at a **non-existent file** `ADR-0055-rename-plan-v3-cutover.md` (the 0055 slot now holds the ontology rename) — a WRONG/dangling reference; (b) every coordination mechanism it specifies (grit symbol-lock, icm rationale rows) is retired.
- **in_masterplan:** NO. One-shot operational cutover; not masterplan material except as historical provenance for the current crate layout.
- **tensions:**
  - **Dangling supersession (collision twin of ADR-0055 finding):** claims to supersede `ADR-0055-rename-plan-v3-cutover.md` which is absent — either the v3 ADR was deleted/overwritten when 0055 was reused for the ontology rename, or it was never committed under that name. Auditors must not treat the current ADR-0055 (ontology) as superseded by 0057.
  - Mechanism coupling to retired grit/icm (ADR-0116) and to `gh pr merge --admin` (forge-CLI; keystone §5 forge fault-line).
  - Tightly bound to ADR-0054's amendment (rename-event scaffold-claim) and ADR-0056 (BNF) — same Shard 0 commit; the cluster archives together but the BNF *decision* (0056) survives independently.
- **hyperscaler_challenge:** ALIGNED-on-strategy / dated-on-tooling. Atomic large-scale renames with a single lockfile event, partitioned review, and a deterministic reverse-able rewrite tool is exactly hyperscaler monorepo migration practice (e.g., global atomic refactors). The grit/icm/gh-admin substrate is what a hyperscaler would replace with owned tooling — consistent with the project's own later direction (ADR-0510 bespoke VCS).
- **ai_slop:** Low–moderate. "Shard 1.5" naming-justification paragraph is defensive over-explanation; the rollback/expedite protocol has three nested emergency lanes that read as ceremony. Otherwise concrete.
- **refinement:** Fix or annotate the dangling `supersedes:` (point at the real retired v3 ADR id, or record the 0055 number-reuse). After confirming Shard 1 executed, archive as historical record. Re-express the surviving "atomic single-lockfile rename" pattern as guidance, decoupled from grit/icm.
- **consensus_needed:** no on the cutover; feeds the same number-reuse consensus question as ADR-0055.

---

### ADR-0058 — Flat microservice catalog (Product Groups retired)

- **decision_atom:** The architecture is a flat catalog of independently-deployable microservices (no arm/vertical/product-group/platform grouping in code/dirs/crate-names); "Healthcare/Enterprise/FinTech" are GTM segmentation only, and any tenant enables any à-la-carte subset.
- **current_status:** front-matter `status: accepted` (minimal front-matter).
- **disposition:** KEEP (current, load-bearing, governing). This is an upstream ancestor of the keystone's flat-only canon.
- **governing:** n/a as governed; it is reinforced/extended downstream by **ADR-0362** (full grouping retirement → flat-only; keystone §1.2) and ADR-0132. ADR-0058 is the early, ADR-0362 the definitive statement of the same principle.
- **truth_flag:** TRUE. Flat catalog is canon corpus-wide; consistent with BNF v4.1 (ADR-0056) and tenant/tenant-class doctrine.
- **in_masterplan:** PARTIAL. The flat-catalog principle is canonical posture, but this ADR has no planning front-matter and the definitive version is ADR-0362; masterplan should bind the *current* statement (0362) and treat 0058 as provenance.
- **tensions:**
  - **Catalog drift vs retired vocab:** the canonical catalog still lists `foundry` (now `foundry (internal-only)`) — retired brand (ADR-0335 → intelligence/governance). Also lists `connect` (Bominal "Workspace"→"Connect"), `cellar`, `dining`, `social`/`shorts` lineage — note ADR-0334 merged `shorts` into `social`; verify the catalog reflects that.
  - Redundancy with ADR-0362 (same decision, later and broader) — 0058 risks being read as the live statement when 0362 supersedes the framing.
  - "platform" retired as a substrate name here, but ADR-0056's registry/examples still use `oya-platform-*` in migration rows — cross-ADR vocab inconsistency (transitional, since those are the *old* names being renamed away).
- **hyperscaler_challenge:** QUESTIONABLE (scope, not structure). The flat-catalog *structure* is aligned (AWS is literally an à-la-carte flat service catalog; clean per-service deploy/scale boundaries are best practice). The *questionable* part is breadth: one org owning medical+pharmacy+hr+payroll+accounting+payments+banking+insurance+ads+analytics+manufacturing+logistics as first-class microservices is a scope no hyperscaler attempts in-house — they provide substrate and let ISVs build verticals. This is the "own everything" breadth tension (keystone §5.5) surfacing at the catalog level. Argues for KEEP-the-structure, flag-the-breadth.
- **ai_slop:** Low. Clear, decision-shaped, good examples. Mild redundancy with 0056/0362.
- **refinement:** Scrub `foundry` → `intelligence`/`governance` from the catalog; reconcile with ADR-0334 (shorts→social) and ADR-0362 (definitive flat-only). Bind the current flat-catalog statement to masterplan once 0058-vs-0362 precedence is settled.
- **consensus_needed:** no on flat-catalog; **yes** on the breadth question (is the full vertical catalog day-0 scope, or GTM aspiration?) — this is the recurring own-everything tension and is founder-level.

---

## Chunk notes for synthesis

**Two clean clusters, opposite fates.**

1. **Grit/icm cutover cluster (0052, 0053, 0054)** — ALL ARCHIVE/superseded, governed by **ADR-0116** (retire grit/rtk/icm/vox) + **ADR-0118** (retire 0052's archive-orphan lane). Important second-order finding: ADR-0116's *replacement* was the in-repo **Foundry pipeline (ADR-0110/0112/0113)**, which the keystone map shows was itself **retired by ADR-0363** (retire agentic-VCS Foundry). So 0052/0053/0054 are **two supersession-hops from current truth** — they were replaced by a thing that was then itself replaced. Surviving durable atoms worth lifting to masterplan: "destructive migrations require a committed pre-move classification manifest (archive-before-delete, rollback boundary)" (from 0052) and "agents mutate state only through audited, provider-agnostic primitives — no raw git/gh in agent paths" (from 0053). Both are principle-TRUE / instance-STALE.

2. **Rename / clean-arch cluster (0055, 0056, 0057, 0058)** — the *naming and structure decisions* are TRUE and canonical (Ontology rename, BNF v4.1 12-layer grammar, flat microservice catalog); the *cutover mechanics* (0057) are STALE because they ride the retired grit/icm lock. 0055/0056/0058 are KEEP; 0057 is AMEND→ARCHIVE-after-execution.

**NEW collision finding (not in keystone §6).** A second duplicate-number situation beyond the known ADR-0377: **ADR-0055 was number-reused.** ADR-0057 (`supersedes: docs/adr-archive/ADR-0055-object-graph-renamed-to-ontology.md`) and ADR-0056 (`related: ADR-0055`) point at a v3-rename-plan ADR-0055 that is **absent on disk**; the only on-disk 0055 is `object-graph-renamed-to-ontology`. Net: the 0055 slot was overwritten/reallocated, leaving ADR-0057 with a **dangling supersession pointer** and a risk that the live ontology-rename ADR is mis-read as superseded. This needs a founder/index ruling on number-reuse policy (it directly implicates the masterplan-generation design, which keys on ADR ids and `supersedes:` edges — a dangling/reused id poisons any generated graph).

**Pervasive retired-vocab leakage.** `foundry` (brand) and `grit`/`icm` (tooling) leak into otherwise-live ADRs: ADR-0056's microservice registry lists `foundry = { owner = "council-foundry" }` and uses `oya-foundry-grit-cli`/`oya-foundry-icm-cli` examples; ADR-0058's canonical catalog lists `foundry` as an internal microservice. Per keystone §2 this is retired-vocab leakage (brand RETIRED → intelligence/governance; founder: "cloud-intelligence is the valid name"). The *decisions* are sound; the *exemplar vocabulary* is dead.

**Masterplan-binding gap.** None of the 7 ADRs carry `masterplan_ref`/`planning_impact` front-matter (0056/0057 carry `authority_tier`/`canonical_authority` but not masterplan binding). This chunk is a microcosm of the keystone §4 "8.8% ADR binding" problem: at least three of these (Ontology rename 0055, BNF 0056, flat-catalog 0058) are canonical, cross-cutting invariants that the masterplan *must* capture under either authored-or-generated reading — yet none is bound. Under the **generated-from-ADRs** design, the dangling ADR-0055 supersession + retired-vocab leakage would corrupt the generated masterplan; under the **authored-as-SSOT** design, these three atoms should be hand-lifted into masterplan.json now.

**Cross-chunk tensions to escalate:**
- Forge/CLI fault-line (keystone §5): ADR-0053 bakes GitHub-CLI (`gh`) as the day-0 agent forge substrate and 0057's emergency lane uses `gh pr merge --admin` — consistent with the founder's GitHub directive, inconsistent with source Forgejo/bespoke-VCS canon (ADR-0363/0510). Surfaced, not resolved.
- Own-everything breadth (keystone §5.5): ADR-0058's full vertical catalog (medical→banking→ads as first-class µservices) is the breadth tension at catalog granularity — a hyperscaler provides substrate, not the verticals.
- foundry→intelligence/governance rename (ADR-0335/0347) has NOT propagated into 0056's registry or 0058's catalog — a corpus-wide cleanup the synthesis should track as a single mechanical sweep, not per-ADR amendments.
