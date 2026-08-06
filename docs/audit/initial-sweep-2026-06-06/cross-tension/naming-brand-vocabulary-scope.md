# Cross-Tension Register — Theme: naming / brand / vocabulary / scope

> CONTRADICTION HUNTER pass, theme `naming-brand-vocabulary-scope`, initial sweep 2026-06-06.
> READ-ONLY audit. No audited doc was edited; this is the only artifact written.
> Ground truth: SOURCE = `~/Developer/source` (346 ADRs), LINUX = `~/Developer/linux` (26 pilot ADRs).
> Baseline: `_map/canonical-posture-and-supersession-map.md` + the wm4gkcey5 linux register.
> Founder goal: masterplan = single source of truth; backfill it with true+relevant decisions.
> Every masterplan-related call is flagged under BOTH open readings (authored-as-SSOT vs generated-from-ADRs).

## How to read this register
Each tension carries: the conflicting positions (ADR ids + file refs); **TRUE CONTRADICTION** vs **RECONCILABLE OVERLAP** vs **DATA-INTEGRITY DEFECT**; which governs (latest/locked ADR); a **surgical resolution** (cross-ref / supersede / clarify / data-fix — never new policy); a **DECISION-NEEDED-FROM-FOUNDER** flag with a crisp question where it is genuinely the founder's call; and the resulting **disposition changes**.

Severity legend: **CRITICAL** (corrupts the corpus or blocks merge) · **MAJOR** (mis-leads readers / poisons the generated masterplan) · **MINOR** (hygiene).

---

## T-1 — `foundry` brand is RETIRED yet leaks corpus-wide; two Accepted ADRs disagree on whether the rename even happened
**Severity: CRITICAL. Class: TRUE CONTRADICTION (between two Accepted ADRs) + measured residue.**

**Positions:**
- **Brand retired.** `ADR-0335` (Accepted 2026-05-21) retires the `foundry` µservice, absorbs it into `intelligence`, drops "retired external agent harness"; founder-confirmed "cloud-intelligence is the valid name" (map §2, GLOSSARY L1032). `ADR-0347` (Proposed) renames every `oya-foundry-fitness-*` CI lane to `oya-governance-*`.
- **Rename DEFERRED.** `ADR-0335` D-37..D-48 + R-13/R-14 explicitly **do NOT rename** the crates: *"existing crates are not renamed in this ADR to avoid a 122-crate rename cascade across 43 dependent crates… future rename is a separate cleanup wave"* (`source/docs/adr-archive/ADR-0335-intelligence-microservice-consolidation.md 647-652`). `oya-foundry-*` survives as "transition debt."
- **Rename DECLARED DONE.** `ADR-0363` (Accepted 2026-05-26, **5 days later**) §Context-1: *"**The Foundry name was eradicated** (ADR-0362 + the #181–#184 cutover): the former `oya-foundry-*` crates were renamed across three namespaces — `oya-intelligence-*` (116), `oya-governance-*` (39+2), `oya-vcs-*` (20). `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell"* (`ADR-0363-…md:35`).
- **Ground truth (grep, 2026-06-06):** `oya-foundry-*` still appears across **59 ADR files** in `docs/decisions/` (excl. the retirement ADRs), incl. `oya-foundry-fitness-*` ×71, `oya-foundry-capability-kernel` ×6, `oya-foundry-vcs-merge-queue-fix-loop-app` ×5, `oya-foundry-supervisor-kernel`, `oya-foundry-eval-*`, `oya-foundry-account-adapter-*`. **222 ADR files** mention "foundry" at all.

**Verdict:** `ADR-0335` (defer) and `ADR-0363` (eradicated) are a **true contradiction** on the rename's completion state, and the **grep proves `ADR-0363`'s "eradicated" claim is factually false** — the rename did not fully land. The two ADRs are internally inconsistent about the SAME mechanical fact.

**Which governs:** `ADR-0335`'s *deferral* is the truthful description of disk state (rename pending); `ADR-0363`'s "eradicated/renamed-already" prose is stale-optimistic and must not be trusted. The brand-is-dead *intent* (0335/0347/founder) governs; the *completion claim* (0363) does not.

**Surgical resolution:**
1. Treat the `foundry`-brand residue as a single corpus-wide AMEND-vocabulary class (NOT archive — the decisions are sound, the names are dead). The mechanism already exists: fold it into the `ADR-0347` bulk rename (`s/^oya-foundry-fitness-/oya-governance-/`) **plus** an `oya-foundry-*` (non-fitness) → `oya-intelligence-*` sweep per `ADR-0335` D-40/D-43.
2. Correct `ADR-0363:35`: replace "The Foundry name was eradicated… were renamed" with "the Foundry-rename was SEQUENCED (per ADR-0335 D-43 / ADR-0347); residue persists pending the bulk-rename wave" — a one-clause factual correction, not new policy.
3. Promote `ADR-0347` from `Proposed` → `Accepted` (it is the binding mechanism the other two assume already ran).

**DECISION-NEEDED-FROM-FOUNDER:**
> The `foundry`→`intelligence`/`governance` rename is declared done by ADR-0363 but ~59 ADRs + the crate tree still carry `oya-foundry-*`. Do you authorize ONE bulk-rename wave (ADR-0347 generalized: `oya-foundry-fitness-*`→`oya-governance-*`, other `oya-foundry-*`→`oya-intelligence-*`) as a precondition to masterplan backfill — and is `cloud-intelligence` the crate prefix (`oya-cloud-intelligence-*`) or `oya-intelligence-*` (ADR-0335 D-40 says the latter)?

**Disposition changes:** `ADR-0363` moves **keep → AMEND** (factual correction of the "eradicated" claim). `ADR-0347` **Proposed → recommend ACCEPT**. The ~59 foundry-residue ADRs already carry AMEND-for-naming in their per-chunk dispositions; this confirms the batch and its single mechanism.

---

## T-2 — DATA-INTEGRITY ALARM: a global "tier"/"MVP" find-replace corrupted the KR regulatory term **KCMVP** into the non-existent token `KCminimum-shippable-tier`
**Severity: CRITICAL. Class: DATA-INTEGRITY DEFECT (corpus corruption).**

**Evidence (grep, 2026-06-06):** the string `shippable-tier` appears in **8 ADR files** with **31 occurrences**:
- `KCminimum-shippable-tier` ×16 (+ `-validated` ×2, `/CSAP` ×1) — this is **`KCMVP`** (Korea Cryptographic Module Validation Program) with `MVP`→`minimum-shippable-tier` blindly substituted inside the acronym.
- `minimum-shippable-tier` ×11 standalone — this is **`MVP`** (the retired M0–M3/MVP milestone vocabulary per GLOSSARY L250/L504) substituted, but the substitution ran over `KCMVP` and `KISA-…MVP` too.
- Affected files: `ADR-0043-secrets-management-openbao-and-hsm-per-cell.md` (the regulatory/HSM ADR — worst hit: "KCminimum-shippable-tier HSM", "KCminimum-shippable-tier + FIPS 140", "KR Crypto…"), `ADR-0018-glossary-and-terminology-canon.md` (the GLOSSARY itself, which DEFINES the canonical term), `ADR-0002-tenant-and-identity-kernel.md`, `ADR-0009`, `ADR-0016`, `ADR-0029`, `ADR-0052`, `ADR-0121`. Also `PAYROLL_minimum-shippable-tier_PROMPT.md` (a corrupted filename token).

**Verdict:** Not a disagreement — **mechanical corpus corruption** from a global `s/MVP/minimum-shippable-tier/` sweep (likely the M0–M3/MVP retirement) that had no word-boundary guard and ate the regulated term `KCMVP`. The founder's "verify recent edits are not plain wrong" warning is **confirmed positive** here (though this corruption is source-side and pre-dates the linux auto-reconcile).

**Which governs:** the GLOSSARY (`ADR-0018`) is the canonical authority for `KCMVP`; the corrupted token is authoritative nowhere.

**Surgical resolution (data-fix, not policy):**
1. Corpus-wide grep `s/KCminimum-shippable-tier/KCMVP/` and `s/KISA-minimum-shippable-tier/KISA-MVP-equivalent/` (restore the regulatory acronyms), `s/CSAP \/ KCMVP/` etc. preserved.
2. For the genuine retired-MVP cases (`M0..M3 / minimum-shippable-tier` in the glossary's retired-vocab list), replace with the canonical Wave-name replacement ("descriptive Wave names" per map §2), NOT the corrupt token.
3. Run a one-time corpus lint: any `*-shippable-tier` residue is a data-integrity FAIL.

**DECISION-NEEDED-FROM-FOUNDER:**
> A find-replace sweep corrupted the Korean regulatory term **KCMVP** into `KCminimum-shippable-tier` in 8 ADRs (incl. the GLOSSARY and the HSM/secrets ADR-0043). Confirm a corpus-wide data-integrity pass to (a) restore `KCMVP`/`KISA` regulatory tokens and (b) replace genuine retired-`MVP`/`M0–M3` milestone references with canonical Wave names — BEFORE any masterplan backfill, since the masterplan must not inherit corrupted regulatory vocabulary.

**Disposition changes:** `ADR-0043` and `ADR-0018` (GLOSSARY) gain a **data-fix** flag on top of their existing dispositions (the per-chunk picture already AMENDs/ARCHIVEs several of these; this adds the corruption-repair as a hard precondition). The defect is corpus-wide, so it is a sweep, not a per-ADR rewrite.

---

## T-3 — `tier` is overloaded across **5+ live axes** after the tenant tier-system retirement; collision risk for the masterplan vocabulary
**Severity: MAJOR. Class: RECONCILABLE OVERLAP (needs namespacing, not supersession).**

**Positions:** `ADR-0329` (Accepted) retires the **customer-facing capability-tier ladder** (Bronze/Silver/Gold/Platinum) → `tenant_class` (`demo_trial`|`paid`) + composable `billing_components`. But `ADR-0329` itself (B2.036–B2.039, A.3) **explicitly PRESERVES** four other "tier" vocabularies as unrelated axes:
- **ADR-0248 cellular criticality** Tier 0–Tier 4 / `cell-tier-*` / `dr_tier` (infra availability).
- **ADR-0037 public-API stability** tiers (preview/stable/GA).
- **ADR-0083 Rust error-handling** Tier 1/Tier 2 library classification.
- **ADR-0252** HLC-vs-TrueTime "tier" (clock discipline, B2.099).

Plus, per the per-chunk findings, downstream ADRs add **even more** "tier" axes: `ADR-0163` *Environment Tiers* (env stages), `ADR-0159` persona-tier, `ADR-0161` storage-tier, `ADR-0144` EU-AI-Act risk-tier, and the **LINUX autonomy-tier T1–T4** (policy autonomy ceiling, `ADR-0021`). Residual Bronze/Silver/Gold/Platinum survives in **9 ADRs** outside the retirement set (grep).

**Verdict:** **Reconcilable** — these are genuinely different concerns sharing one English word. `ADR-0329` already does the right thing (allow-list, not exclude-list). The risk is purely **masterplan naming collision**: a generated or authored masterplan with a flat "tier" namespace would conflate them.

**Which governs:** `ADR-0329` is the canonical tier-retirement authority and already enumerates the survivors; it governs the namespacing rule.

**Surgical resolution:**
1. No supersession. Adopt `ADR-0329`'s allow-list as the masterplan's canonical tier-namespace map: `capability-tier` = RETIRED; `tenant_class` (tenancy), `cell-tier`/`dr_tier` (availability), `api-stability` (preview/stable/GA), `rust-error-tier` (library), `autonomy-tier T1–T4` (policy), `env-stage` (ADR-0163 — recommend RENAME from "environment tier" to "environment stage" per the per-chunk q), `risk-tier` (ADR-0144 EU-AI-Act) are DISTINCT.
2. Clean up the 9 residual Bronze/Silver/Gold/Platinum occurrences via `ADR-0329`'s zero-residue lane (already authorized, B2.064/B2.067).

**DECISION-NEEDED-FROM-FOUNDER:**
> After retiring the capability-tier ladder, "tier" still names ≥5 live axes (tenant_class, cell/dr criticality, api-stability, rust-error, autonomy T1–T4, env). Should the masterplan enforce a namespaced vocabulary (each axis gets a distinct word, e.g. rename ADR-0163 "environment tiers" → "environment stages") so no two axes collide on the bare word "tier"?

**Disposition changes:** No new archives. `ADR-0163` gains an AMEND-for-naming (env-stage rename) recommendation; the 9 residual-Bronze ADRs are caught by the existing zero-residue lane.

---

## T-4 — The OPEN masterplan fork: authored-as-SSOT vs generated-from-ADRs — two design docs point in OPPOSITE directions, and it gates ALL backfill
**Severity: CRITICAL (it gates the founder's stated goal). Class: TRUE CONTRADICTION between two design docs; founder's call.**

**Positions (both read in full):**
- **Generated-from-ADRs.** `source/docs/ideas/planning-ssot-consolidation.md`: *"ADRs (`docs/decisions/`) = the authored, immutable decision log — the SSOT… `masterplan` = GENERATED from ADR front-matter… never hand-maintained… Build `oya gen masterplan` + a drift gate."* Status is **derived from gate output, NOT stored in the ADR**. Proposes **re-founding the ADR log from ADR-0000** with `consolidates:` provenance and archiving the old series frozen.
- **Masterplan-is-authority.** `source/docs/ideas/planning-ssot-drift-prevention.md`: *"masterplan.json **is the one planning authority**; ADRs + canonical specs **bind into it**… via a strict `planning-ssot-coverage` gate (frontmatter `masterplan_ref`, bidirectional, supersession-aware)."* Found only **8.8% ADR binding** today.
- **MASTERPLAN.md** front-matter (verified): `shape: compatibility_projection`, `authority_tier: 0`, `canonical_authority: /specs/masterplan.json` — i.e. the human doc is explicitly NOT the authority; `/specs/masterplan.json` is. Neither design doc has won.

**Verdict:** **True contradiction in direction.** Consolidation has ADRs generate the masterplan (status is computed; ADRs immutable). Drift-prevention has the masterplan as authority that ADRs bind into (status authored in ADR front-matter via `masterplan_ref`). These are mutually exclusive and the founder's goal ("masterplan = SSOT, backfill it") is ambiguous between them: "backfill it" reads as authored-authority, but "ADRs are immutable SSOT" reads as generated.

**Which governs:** **UNRESOLVED — founder's call.** Both designs are internally coherent; the corpus has not picked one. This is the single highest-leverage naming/scope decision because it determines whether backfill = "write decisions INTO masterplan.json" or "add `planning_impact`/`deliverables` front-matter to ADRs and regenerate."

**Surgical resolution:** None possible without the ruling — do NOT pick a side (per ground-truth instruction). Surface only: every "should this be bound into the masterplan?" question in this register (T-3, T-5, T-7, and the per-chunk 91.2%-unbound mass) is **blocked** on this fork.

**DECISION-NEEDED-FROM-FOUNDER (the keystone question):**
> Is the masterplan **AUTHORED** (you write true decisions directly into `/specs/masterplan.json`; ADRs bind in via `masterplan_ref`; drift-prevention.md design) or **GENERATED** from immutable ADR front-matter (`planning_impact`+`deliverables`; status computed from gates; consolidation.md design, incl. the re-found-from-ADR-0000 step)? This single ruling decides HOW every true+relevant decision in this audit gets backfilled, and whether the ADR log is re-founded from ADR-0000.

**Disposition changes:** None directly, but this gates the backfill disposition of EVERY keep/amend ADR in the corpus. If "generated" wins: a `planning_impact` + `deliverables` tagging pass becomes a precondition (only 8.8% bound today). If "authored" wins: decisions are written into masterplan.json and `masterplan_ref` back-edges added.

---

## T-5 — `ADR-number-keyed` names are FORBIDDEN going forward, yet the corpus is saturated with them (and the masterplan would inherit them)
**Severity: MAJOR. Class: RECONCILABLE OVERLAP (forward-policy vs un-migrated legacy).**

**Positions:** `planning-ssot-consolidation.md` §"Canonical naming" declares ADR-number-keyed / wave-milestone-keyed identifiers a **FORBIDDEN antipattern**: *"a name that encodes provenance instead of function — ADR-number-keyed (`adr-0145-*`, lane purposes that read 'per ADR-0110'), wave/milestone codes (`M01-P18`)… Decisions move/supersede; a name pinned to a number rots."* Enforced by a planned `canonical-naming` lint. But: **24 ADR files** still carry `M0[0-3]-P[0-9]` milestone-keyed triggers (grep); per-chunk findings flag `adr-0145-*` gate names, `M01-P18`, retired wave names (`W-Foundry-Preview`), and CUG→Team residue throughout.

**Verdict:** **Reconcilable** — a forward-policy that has not been back-applied. Not a contradiction; a migration debt with a designed (but unbuilt) enforcement lane.

**Which governs:** the consolidation §canonical-naming policy governs forward; the legacy names are pre-policy debt.

**Surgical resolution:**
1. The milestone-keyed triggers (`M01-Pxx`) are independently retired by GLOSSARY M0–M3 retirement → descriptive Wave names (map §2); rewrite each milestone-keyed trigger to a Wave-name + numeric form (mechanical, per the per-chunk findings).
2. Defer `adr-NNNN-keyed` gate/lane renames to the `canonical-naming` lint build (consolidation D7) — but note ADR *files* stay numbered (`ADR-0000+` is the log index, explicitly exempt).

**DECISION-NEEDED-FROM-FOUNDER:** (sub-question of T-4)
> The canonical-naming policy forbids ADR-number-keyed and M0x-milestone-keyed identifiers, but 24+ ADRs still use them. Is the `canonical-naming` lint + the M0x→Wave-name rewrite a precondition for backfill (mandatory if the masterplan is GENERATED, since rotted names would propagate), or a fast-follow cleanup?

**Disposition changes:** The 24 M0x-bearing ADRs gain an AMEND-for-naming (milestone→Wave) flag, consistent with their existing per-chunk dispositions.

---

## T-6 — Number collisions: duplicate ADR-0377; cross-dir ADR-0055/0145; the guaranteed LINUX↔SOURCE 0001–0026 collision on merge
**Severity: MAJOR (CRITICAL on merge). Class: TRUE COLLISION (genuine id reuse).**

**Positions / evidence (verified on disk):**
- **Duplicate ADR-0377 (same dir):** `ADR-0377-forgejo-board-git-ref-cas-fallback.md` (status `Proposed (conditional)`) AND `ADR-0377-kafka-to-pulsar-via-kop.md` (status `Accepted`, supersedes ADR-0005). Two authoritative ADRs, one number — a real collision (map §6.1). `ADR-0510`'s own `numbering_note` confirms `decisions.json next_adr` is **stale** and "must be re-derived from the on-disk corpus, not trusted at face value."
- **Dangling supersession (per-chunk chunk-8):** `ADR-0057` declares `supersedes: ADR-0055-rename-plan-v3-cutover.md` which does NOT exist on disk (the only on-disk 0055 is object-graph→ontology). A dangling pointer that poisons any generated-from-ADRs graph.
- **Cross-dir namespace overlaps:** `decisions/ADR-0055-…` vs `advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md`; `decisions/ADR-0145-…` vs `operators/ADR-0145-runtime-impact-changelog.md` (map §6.2).
- **LINUX↔SOURCE merge collision (guaranteed):** all 26 linux pilot ADRs (0001–0026) carry `renumber_note` and collide with existing source 0001–0026 (e.g. linux ADR-0001 distributed-DB vs source ADR-0001 foundation; linux ADR-0021 owned-policy vs source ADR-0021 foundry-capability-registry; linux ADR-0017 container-platform vs source ADR-0017). Map §6.4.

**Verdict:** **True collisions.** Severity is **MAJOR now, CRITICAL the moment the masterplan is generated from ADR supersede-edges** (a duplicate or dangling id breaks the graph) **or the moment linux merges** (all 26 collide).

**Which governs:** the `Accepted` member wins each duplicate (ADR-0377-kafka over the conditional-Proposed board ADR); the on-disk corpus governs `next_adr`, never `decisions.json`.

**Surgical resolution:**
1. Renumber the conditional-Proposed `ADR-0377-forgejo-board` to the first free number above dev's highest (per `ADR-0510`'s own precedent: it took 0510 to dodge a collision).
2. Fix `ADR-0057`'s dangling `supersedes:` pointer (point at the real predecessor or delete the edge).
3. Renumber all 26 linux pilot ADRs to `ADR-0515+` on merge (never at face value) — the `renumber_note` already mandates this.
4. Re-derive `decisions.json next_adr` from `ls docs/decisions/`.

**DECISION-NEEDED-FROM-FOUNDER:**
> Do you want a hard "no-id-reuse + no-dangling-supersedes-ref" invariant on ADR ids (mandatory if the masterplan is GENERATED from supersede-edges) — and on merge, do the 26 linux pilot ADRs renumber to ADR-0515+ (next free block), or are they consolidated INTO existing source ADRs via the `consolidates:` re-foundation step from consolidation.md?

**Disposition changes:** `ADR-0377-forgejo-board` gains a **renumber** flag; `ADR-0057` gains an AMEND (fix dangling ref). All 26 linux ADRs carry an implicit renumber-on-merge (already in their front-matter).

---

## T-7 — "Own everything" breadth/scope/sequencing: founder ratchet language is SHARED, but the trigger threshold and day-0 breadth diverge sharply
**Severity: MAJOR. Class: RECONCILABLE OVERLAP on principle; genuine scope tension on threshold + catalog breadth.**

**Positions:**
- **Shared principle.** LINUX `ADR-0019` (universal port ratchet: "vendored adapter now, owned adapter when *ready* AND *proven*") + `ADR-0022` ("adopt the hyperscaler method, own the Rust implementation") + `ADR-0020` (staged-ownership) explicitly match SOURCE's own-when-proven ratchet (`ADR-0211`/`ADR-0173`). Both sides agree: own when proven, not speculatively. Map §5 confirms "the disagreement is the *trigger threshold*, not the principle."
- **Day-0 breadth divergence.** LINUX repeatedly chooses OWN_DAY0 (DB engine ADR-0001, policy language ADR-0021, kernel/framekernel, node-OS ADR-0025 "Rust Talos", container runtime ADR-0014). SOURCE stages: best-of-breed OSS now (Postgres+Citus, Talos, Cedar, Forgejo), own only when a numeric trigger fires (e.g. `ADR-0510`'s explicit clone-time/`.git`-size/fan-out thresholds for the bespoke VCS).
- **Catalog breadth (source-internal scope).** `ADR-0058` flat catalog lists medical→pharmacy→hr→payroll→banking→insurance→ads→analytics→manufacturing→logistics as **first-class owned microservices** — a vertical breadth no hyperscaler builds in-house (they ship substrate; ISVs build verticals). `ADR-0185` commits to **five native first-party client stacks** per product. Per-chunk verdict: structure aligned, scope questionable.

**Verdict:** **Reconcilable on the ratchet principle** (both repos share it, verbatim). **Genuine open scope question** on (a) the trigger threshold and (b) whether the owned/first-class breadth (full vertical catalog, 5 native clients, day-0 owned DB+policy+kernel+OS) is day-0 architecture or staged/aspirational. `ADR-0510` is the model the rest should follow: it records the bespoke-VCS destination as DECIDED but gates the cutover on explicit numeric triggers — "recorded-but-deferred," not silently absent.

**Which governs:** the shared own-when-proven ratchet governs the *principle*; the *threshold* and *breadth* are unbound founder scope calls. `ADR-0510`'s numeric-trigger pattern is the locked precedent for how to record a decided-but-deferred ownership target.

**Surgical resolution:**
1. No supersession. Apply the `ADR-0510` numeric-trigger pattern uniformly: record each OWN_DAY0 ambition (DB engine, policy language, node-OS, kernel) as a *decided destination behind a measured trigger*, not a day-0 build — which is exactly what LINUX `ADR-0019`/`ADR-0020` already say ("proven over a production burn-in span, never a one-shot benchmark"). Cross-ref LINUX ADR-0019/0020 ↔ SOURCE ADR-0211/0510 as the shared ratchet.
2. The vertical-catalog (ADR-0058) and 5-native-client (ADR-0185) breadth are separate scope rulings (substrate+ISV vs own-the-vertical; 5 vs 1–2 day-0 clients) — surface, do not resolve.

**DECISION-NEEDED-FROM-FOUNDER:**
> The own-when-proven ratchet is shared across both repos; the open call is the *threshold* and *breadth*. (a) Should every OWN_DAY0 target (owned DB engine, owned policy language, Rust node-OS, framekernel) be recorded as a DECIDED-but-DEFERRED destination behind explicit numeric triggers (the ADR-0510 pattern), rather than day-0 builds? (b) Is the full first-class vertical catalog (ADR-0058: medical/banking/insurance/ads/manufacturing/logistics as owned microservices) and the 5-native-client commitment (ADR-0185) genuine day-0 scope, or substrate + ISV-built verticals + a 1–2 client day-0 set?

**Disposition changes:** None forced. LINUX ADR-0001 keeps its post-reconcile clarification (it already walked back "eliminate Postgres" to "owns the differentiator layer; Postgres+Citus retained" — verified on disk, this is the sharpest cross-side item now *defused*). ADR-0058/0185 carry their existing keep/amend with a scope-ruling flag.

---

## T-8 — Forge brand: GitHub (founder) vs Forgejo (transitory canon) vs bespoke-VCS (destination) — a THREE-way naming/destination tension
**Severity: MAJOR. Class: TRUE TENSION (three locked positions); surface-only per ground truth.**

**Positions:**
- **Founder directive:** migration to **GitHub** `jason931225/oyatie`.
- **Source canon (transitory):** `ADR-0363` adopts self-hosted **Forgejo** + plain git; *rejects GitHub as substrate* ("GitHub is bootstrap-only"; "we use selfhosted forgejo").
- **Source canon (destination):** `ADR-0510` (Proposed, founder-authored) makes Forgejo explicitly **transitory** and names a **bespoke hyperscaler monorepo-VCS** (Piper/Sapling/Mononoke-class, Rust) as the DECIDED destination, cutover gated on numeric triggers.

**Verdict:** **True three-way tension.** The founder's GitHub directive conflicts with even the *transitory* Forgejo canon (0363 rejects GitHub-as-substrate), and the long-horizon canon (0510) is "own the VCS entirely." Per ground-truth instruction: **surface, do not resolve.**

**Which governs:** unresolved across the three; `ADR-0510` is the latest (and founder-authored) and frames the layering (GitHub bootstrap → Forgejo transitory → bespoke destination), but its `status: Proposed — do NOT auto-merge` means it is not locked.

**Surgical resolution:** None this pass. Note only that `ADR-0510` already provides the reconciling frame IF the founder accepts it (GitHub = bootstrap host layer; Forgejo = transitory; bespoke = destination) — these are different layers, not necessarily a flat contradiction. The naming residue (`oya-foundry-vcs-*` crates, `github.com/jason931225/oyatie` URLs hard-baked into ADR-0041/0124/0171 IaC) is the leakage to clean once the forge is locked.

**DECISION-NEEDED-FROM-FOUNDER (the forge ruling):**
> Is the canonical forge GitHub (your migration directive), Forgejo-transitory (ADR-0363), or bespoke-monorepo-VCS-destination (ADR-0510)? ADR-0510's frame is "GitHub=bootstrap, Forgejo=transitory host, bespoke=destination" — do you accept that layering (which makes the three positions sequential not contradictory), or is GitHub the permanent host (retiring the Forgejo/bespoke canon)? This unblocks ADR-0510 (currently Proposed) and the GitHub-hardcoded IaC in ADR-0041/0124/0171.

**Disposition changes:** `ADR-0510` (Proposed) awaits the ruling; the GitHub-hardcoded forge ADRs (0041/0124/0139/0171/0173) carry AMEND-or-archive pending the forge lock (already in their per-chunk dispositions). Surface-only — no resolution this pass.

---

## T-9 — `cell` / `shorts` / `retired external agent harness` / `Furnace` / `CUG` retired-vocabulary leakage (low-severity confirmation)
**Severity: MINOR. Class: RECONCILABLE OVERLAP (retired-vocab residue).**

**Positions / evidence:** `ADR-0333` retires `cell`-as-microservice → cell-as-pattern; `ADR-0334` merges `shorts`→`social`; `ADR-0335` drops `retired external agent harness`; `Furnace`/`Foundry Furnace` retired (LEDG-013); `CUG`→`Team` (GLOSSARY L252). Grep confirms low residue: `microservices/cell/` in 1 ADR file; the bulk of the leakage is the `foundry` brand (T-1) and `tier` (T-3), already covered. LINUX side is clean: **0** Bronze/tier, **0** tenant-class, only **1** "foundry" mention (`ADR-0020:89` — an OTel example "all Foundry invocations", which is correct-as-history but should read "intelligence").

**Verdict:** **Reconcilable** — minor retired-vocab residue, fully governed by existing retirement ADRs. No contradiction.

**Which governs:** the respective retirement ADRs (0333/0334/0335) + GLOSSARY.

**Surgical resolution:** Fold into the same corpus-wide vocab-lint as T-1/T-3 (residue greps already authorized by ADR-0347/0329). Fix LINUX `ADR-0020:89` "all Foundry invocations" → "all intelligence invocations" (one token; the only foundry leak on the pilot side, confirming the wm4gkcey5 reconcile is otherwise clean on this theme).

**DECISION-NEEDED-FROM-FOUNDER:** None — mechanical, governed by existing ADRs.

**Disposition changes:** LINUX `ADR-0020` gains a one-token AMEND-for-naming (foundry→intelligence in the OTel example). No source-side changes beyond the existing vocab-lint.

---

## Summary — disposition deltas caused by these tensions

| ADR | Side | Prior disposition | Delta from this theme | Driver |
|---|---|---|---|---|
| ADR-0363 | source | keep | → **AMEND** (correct the false "Foundry name was eradicated/renamed" claim) | T-1 |
| ADR-0347 | source | (Proposed) | → recommend **ACCEPT** (it is the binding rename mechanism others assume ran) | T-1 |
| ADR-0043 | source | amend | + **DATA-FIX** (restore KCMVP from corruption) — hard precondition | T-2 |
| ADR-0018 (GLOSSARY) | source | — | + **DATA-FIX** (restore KCMVP/MVP tokens in the canon doc) | T-2 |
| ADR-0163 | source | amend | + AMEND-for-naming ("environment tiers"→"stages") | T-3 |
| ADR-0377-forgejo-board | source | — | → **RENUMBER** (resolve the duplicate-0377 collision) | T-6 |
| ADR-0057 | source | amend | + AMEND (fix dangling `supersedes: ADR-0055-rename-plan-v3`) | T-6 |
| ~24 M0x-keyed ADRs | source | (various) | + AMEND-for-naming (M0x→Wave-name) | T-5 |
| ~59 foundry-residue ADRs | source | (AMEND batch) | confirmed single AMEND-vocab batch + mechanism (ADR-0347 generalized) | T-1 |
| ADR-0510 | source | — | remains **Proposed**; awaits forge ruling (surface-only) | T-8 |
| ADR-0020 | linux | keep/amend | + one-token AMEND (foundry→intelligence in OTel example) | T-9 |
| 26 linux pilot ADRs | linux | — | renumber-on-merge confirmed (to ADR-0515+ or `consolidates:`) | T-6 |

## Founder decisions needed (consolidated, by leverage)
1. **[KEYSTONE, T-4] Masterplan AUTHORED vs GENERATED** — decides HOW every true decision is backfilled; gates everything else. Includes: re-found ADR log from ADR-0000 (yes/no)?
2. **[T-8] Forge ruling** — GitHub vs Forgejo-transitory vs bespoke-destination; accept ADR-0510's layered frame or make GitHub permanent? Unblocks ADR-0510 + GitHub-hardcoded IaC.
3. **[T-1] Foundry rename wave** — authorize the bulk `oya-foundry-*`→`oya-intelligence-*`/`oya-governance-*` rename (ADR-0347 generalized) as a backfill precondition; confirm `oya-intelligence-*` (not `oya-cloud-intelligence-*`) prefix.
4. **[T-2] KCMVP data-integrity sweep** — authorize corpus-wide restoration of regulatory tokens corrupted by the MVP/tier find-replace, before backfill.
5. **[T-7] Own-everything breadth/threshold** — apply the ADR-0510 numeric-trigger "decided-but-deferred" pattern to all OWN_DAY0 targets? Is the full vertical catalog (ADR-0058) + 5-native-clients (ADR-0185) day-0 scope or substrate+ISV?
6. **[T-6] ADR-id invariant** — enforce no-id-reuse + no-dangling-supersedes (mandatory if masterplan is generated); linux 0001–0026 renumber to 0515+ vs `consolidates:`.
7. **[T-3/T-5] Vocabulary namespacing** — enforce a namespaced "tier" map + the canonical-naming lint (forbid ADR-number/M0x-keyed names) as backfill preconditions?

---
*End of register. READ-ONLY pass; the only write is this file. Trust the superseding ADR over stale front-matter; treat `foundry`/`tier`/`retired external agent harness`/`M0–M3`/`KCMVP-corruption` per the resolutions above; the masterplan authored-vs-generated fork (T-4) is the keystone founder call that gates all backfill.*
