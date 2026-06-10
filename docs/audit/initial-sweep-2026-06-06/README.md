# Initial documentation sweep — 2026-06-06

> **Provenance notice (2026-06-09):** Historical working plans preserved from the transient
> kernel-snapshot branch (2026-06-08), provenance-only, NOT live authority. Live authority is
> HANDOFF.md + /specs/masterplan.json + ADR-0516..0535. Any forbidden-vocabulary terms inside
> (foundry, jenkins, forgejo, oya-vcs) are historical quotations from planning work, not active
> usage.

**Purpose.** Thorough, consensus-first audit of ALL documentation on both sides before any pilot→`jason931225/oyatie` migration. Find contradictions, inconsistencies, refinement opportunities, AI-slop, and decisions a hyperscaler would not make. For **every ADR** decide **keep / amend / archive** — **both sides**. Promote-or-remove ideas/plans not yet ADRs.

**Sides.**
- `source` = `~/Developer/source` (GitHub `jason931225/oyatie`) — the company monorepo. **346 ADRs**, ~2,357 docs.
- `linux` = `~/Developer/linux` — the substrate PILOT (staging). **26 ADRs** (0001–0026) + context/research/migration.

**Ground rules (binding).**
1. **READ-ONLY.** Audit agents read audited docs and write ONLY their own artifact under this directory. **No amendments to any ADR or doc this pass.**
2. **Consensus-first.** Check → surface tensions → reach agreement with the founder → *then* amend (separate gated pass). No copy-paste of pilot docs into source.
3. **Artifact per agent.** Huge task list → every agent emits a durable findings file here so work is never lost.
4. **Hyperscaler challenge.** Every decision is challenged with *"would Google / AWS / Azure actually do this?"* — if not, flag why and whether it argues for amend/archive.
5. **PR #605 treated as MERGED (simulated).** `docs/ideas/agent-execution-controller.md` is canonical, decision-pending (promote-as-narrower vs decline) — not slop.

**Workspace layout.**
- `_map/` — keystone canonical-posture + supersession + retired-vocabulary map (built first; every audit agent reads it).
- `adr/` — per-chunk ADR disposition artifacts (`source-N.md`, `linux-N.md`).
- `cross-tension/` — cross-side + intra-side contradiction-hunt artifacts (themed).
- `hyperscaler/` — "would G/A/A do this?" challenge artifacts.
- `ideas/` — ideas/plans promote-or-remove register.
- `docs-sweep/` — WF2 rest-of-docs review artifacts.
- `synthesis/` — `00-MASTER-REGISTER.md` (the decision-ready doc) + `01-ADR-DISPOSITION-TABLE.md`.

**Disposition rubric.**
- **KEEP** — current, correct, non-conflicting, well-formed.
- **AMEND** — sound decision but stale/drifted/needs reconciliation (naming, superseded refs, missing reconciliation with a later ADR, hyperscaler-lens fix).
- **ARCHIVE** — fully superseded (cite governing ADR/decision), obsolete, or redundant/merged.

**Mandate — no unaccounted proposals (founder, 2026-06-06).** Every `Proposed` ADR (99 on the source side) MUST be resolved to **RATIFY (accept)** or **DROP (archive)** with rationale + door-class — never left in limbo. Synthesis emits a **Proposed-resolution ledger** (all `Proposed` → ratify/drop) for founder batch sign-off (`door: one-way` ⇒ founder sign-off; `door: two-way` ⇒ auto on green). This is the decision-debt elimination ADR-0364 already mandates.

**Binding principle — worth-documenting ⇒ worth-reading ⇒ reachable (founder, 2026-06-06).**
The masterplan is GENERATED (ADRs = SSOT), but its generation logic must capture **all necessary machine-readable context — including operational *instructions* every fresh session must read, not just decisions.** Test for every artifact: *if it is worth documenting (md/json), it is worth reading* → it must **belong to some part of the workflow AND be reachable from the masterplan**. Masterplan generation must therefore emit/point-to a **must-read session-context bundle** (extending the `root-hub-pointers.json → masterplan → companion_docs` chain) so a fresh agent resolving the SSOT inherits every worth-reading instruction.

**Per-doc reachability classification (added to every disposition):**
- **DECISION** → an ADR → the generated masterplan.
- **INSTRUCTION / SESSION-CONTEXT** → the must-read session-context bundle (authority chain, operating contract, AGENTS.md, skills/commands, load-bearing standards).
- **GENERATED-REFERENCE** → built from specs, never hand-edited.
- **ORPHAN** (none of the above) → **not needed → archive/delete.**

**Binding principle — domain-cohesion / no-contradiction by construction (founder, 2026-06-06).**
Hand-maintained `related:` is too weak. **Shared foundation:** synthesis produces a **closed `domain` ENUM** (controlled vocabulary, ~12–20 domains); every ADR is keyed to one+ domain. On author/amend the mandatory read-set is resolved by **either** mechanism:
- **(A) domain-keyed enum (deterministic):** read-set = all ADRs sharing the `domain` enum key — exact-match, zero-infra, trivially machine-resolvable, gate-friendly.
- **(B) vector recall (semantic):** embed ADRs, retrieve top-k nearest (filtered by domain) — catches subtle/cross-domain overlaps an enum bucket misses; rides on source's vector store (`ADR-0046`/`ADR-0192` Milvus; dogfood per `ADR-0247`), no new dependency.

Either way a **`domain-cohesion` gate** runs an explicit contradiction check over the read-set and **fails at decision time** (key-match / cosine-similarity ≠ logical contradiction). **Recommended ratchet:** ship **(A) enum-keyed first**, layer **(B) vector** only when bucket-coarseness bites (vendored-simple-now / own-when-proven). Deliverables: (a) the enum domain taxonomy + per-ADR `domain` (synthesis); (b) the `domain → ADRs` index (+ optional embedding index); (c) a meta-ADR adding the `domain` enum field to the ADR-0364 template + the `domain-cohesion` gate to the ADR-0365 lifecycle. The audit's disposition table gains a **`domain` column**.

**Verification gate — verify at EACH step; trust nothing as-is (founder, 2026-06-06).**
Every phase output is checked by a **SEPARATE verifier lane against evidence + the real files** (never against its own claims, never self-approval) before the next phase consumes it: the **map** (supersession graph / retired-vocab spot-checked vs real ADR front-matter), the **audit** artifacts (auditors actually read their chunk, stayed read-only, dispositions grounded, every ADR covered), the **cross-tension/hyperscaler** claims (each asserted contradiction confirmed at the cited file:line — no phantom findings), the **ideas** classifications, and the **synthesis**. The running `wsqyend9q` ran its internal phases without inter-step verification, so its post-hoc verifier pass **re-checks every phase against primary sources.** Future workflows (WF2, amend, backfill, stale-file) each carry a built-in verify stage; **no deletion or amendment is made on an unverified verdict.**

For the synthesis specifically: the per-agent artifacts under this directory are the **durable record**; the synthesis is an **index/rollup, never a replacement** — nothing worth preserving may live *only* in the synthesis. When synthesis returns, a **SEPARATE verifier pass** (not the synthesis author — no self-approval) checks it against **(a)** every audit artifact (`adr/*`, `cross-tension/*`, `hyperscaler/*`, `ideas/*`) and **(b)** the relevant **real** ADR/source files:
- **ID-set coverage** — all 372 ADRs present in `01-DISPOSITION-TABLE.md` (mechanical id-set diff vs `adr/*.md`).
- **Decision-atom coverage** — every atom in artifacts is in `02-DECISION-ATOM-LEDGER.md` or consciously deduped (not dropped).
- **Tension/finding coverage** — every cross-tension, hyperscaler-misaligned, `truth_flag≠true`, and `consensus_needed` flag is represented in `00-MASTER-REGISTER.md`.
- **Accuracy spot-checks** — re-read a sample of real ADR files; confirm disposition/atom/status/`domain` match (catches "plain wrong" rollups).
- **Loss report** — anything dropped/distorted is folded back **before** the consensus gate.

**Deferred phases (sequenced):** WF1 ADR audit (running) → synthesis → **losslessness verification** (separate lane) → WF2 rest-of-docs review → **`/deep-interview`** seeded with the verified synthesis = the consensus vehicle (founder rulings on contradictions, decisions, Proposed ratify/drop, ideas, design forks, changes; the `door:one-way` sign-off) → gated amendments (supersede/re-author into clean `ADR-0000+` + ADR-0365 lifecycle) → masterplan backfill (generate) → stale-file (>48h untouched) ai-slop pass (after amendments, since validity can change). Founder priority: ADRs first — the interview may take the ADR synthesis first and rest-of-docs as a later segment.
