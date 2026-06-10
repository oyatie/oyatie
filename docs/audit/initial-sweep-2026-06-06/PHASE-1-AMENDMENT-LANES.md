---
title: "PHASE-1 — AMENDMENT-LANES PLAN (the gate-verified A-lane sequencing of the ratified dispositions)"
status: PLAN (READ-ONLY). Organizes the founder-RATIFIED disposition set into gate-verified A-lanes. No re-audit, no new findings — every ADR/fix is cited to a 00-MASTER-CONTRADICTION-REGISTER row.
charter: D-DISPOSITIONS-RATIFIED (founder 2026-06-06) · D-SEQUENCE (firewall-first; amendments are GATE-VERIFIED) · D-FOUNDRY-CLARIFY (foundry = FORBIDDEN VOCAB) · D-FORGE-CLARIFY (Forgejo DROPPED → GitHub-interim → Sapling-bespoke)
authority:
  - SSOT (the WHAT): docs/audit/initial-sweep-2026-06-06/00-MASTER-CONTRADICTION-REGISTER.md  (§1 contradictions+lane · §3 keystone · §4 169-ADR disposition table)
  - lane structure (the HOW-shape): docs/audit/initial-sweep-2026-06-06/UNIFIED-EXECUTION-PLAN.md  (A-lanes + §6 conformance gates)
  - prerequisite (firewall-first): docs/audit/initial-sweep-2026-06-06/PHASE-0-FIREWALL-PLAN.md  (the 4 keystone gates + producer MUST be RED/GREEN-live FIRST)
  - rulings: docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md  (D-SEQUENCE L195-201 · D-FOUNDRY-CLARIFY L203-204 · D-FORGE-CLARIFY L206-207 · D-DISPOSITIONS-RATIFIED L209-210)
date: 2026-06-06
mode: PLAN. Assembled READ-ONLY. Organizes the ratified set; mutates nothing. Every mutating amendment is gated on its Phase-0 verifying gate being live + RED/GREEN-proven + founder door:one-way sign-off.
---

# PHASE-1 — AMENDMENT-LANES PLAN

> **What this is.** The founder has RATIFIED the Accepted-ADR audit dispositions (D-DISPOSITIONS-RATIFIED,
> decision-record L209-210). This plan SEQUENCES every ratified disposition + contradiction into the
> gate-verified Phase-1 A-lanes. It does **not** re-audit and adds **no** new findings — each row cites its
> 00-MASTER-CONTRADICTION-REGISTER home (`reg §4:<id>` = the disposition table row; `reg §1:<C-/H->` = the
> contradiction row; `reg §3:KEYSTONE` / `reg §2A` for the keystone + status-vs-edge drifts).
>
> **Firewall-first (non-negotiable).** Phase-1 amendments are GATE-VERIFIED: **no amendment merges until its
> verifying Phase-0 gate is live + RED/GREEN-proven** (D-SEQUENCE L196-201; PHASE-0-FIREWALL-PLAN §7 EXIT).
> The dependency spine is `producer → accounting-registry → gates → amendments → reorg`. This doc owns the
> **amendments** segment only; Phase-0 must be DONE (FIREWALL REAL, §7) before lane A-anything opens.
>
> **No-renumber discipline (D13-AMENDED).** Amend-in-place + additive into the live free block. The full
> ADR-0000+ re-foundation is DEFERRED (decision-record L42-44). The ONE exception is the dup-`0377` id
> collision (A-INTEGRITY) — a mid-space renumber that is an integrity fix, not a re-foundation.

---

## §0 — THE GATE ALPHABET (the 5 Phase-0 gates that verify each amendment)

Every amendment is verified by **at least one** of the Phase-0 keystone gates (PHASE-0-FIREWALL-PLAN §5).
A lane may not flip an amendment to merged until the named gate is `automated_blocking_now` AND its self-test
reproduces its live exhibit as RED (FIREWALL-PLAN §6.2 born-already-blocking).

| Gate code (this doc) | Phase-0 gate | What it proves for an amendment | FIREWALL-PLAN ref |
|---|---|---|---|
| **G-XA** | `cloud-ci-cross-artifact-agreement` (GATE-1) | ADR↔spec↔masterplan↔roadmap agree; supersession edges reciprocal; **no half-edge**; **dup-ADR-number** caught; **status-enum** value legal (KEYSTONE-extended) | §5.2 GATE-1 |
| **G-TA** | `cloud-ci-total-accounting` (GATE-2) | every path owned + justified + reachable; **no foundry residue** orphan; **brand-residue/forbidden-vocab** (foundry, Forgejo) is a row-level RED | §5.2 GATE-2 |
| **G-SR** | `cloud-ci-staleness-reaper` (GATE-3) | archived ADRs reported→`git mv`, never `rm`; over-budget unreachable husks flagged | §5.2 GATE-3 |
| **G-AR** | `cloud-ci-automation-ratchet` (GATE-4) | no "enforces" claim without a wired buck2 target; advisory-claiming-enforced blocked | §5.2 GATE-4 |

**KEYSTONE extension to G-XA (reg §3 + decision-record L210).** Phase-0 must add to the cross-artifact gate:
1. the `superseded-on-cutover` / `bridge-unratified` **status-enum value** (so 0374/0380/0239/Flagger/Zitadel
   READ as bridges, not ratified canon); 2. the **no-dangling-ref / supersession-completeness invariant**
   (both edge directions written + the stale ADR's status flipped, in the same act; half-edges still RED).
This extension is itself a Phase-0 deliverable (FIREWALL-PLAN GATE-1 self-test: axes-6-vs-7 + dup-0377 +
0511 half-edge); it must be RED/GREEN-proven before A-INTEGRITY / A-CI / A-IDENTITY supersession writes merge.

> **Coupled-invariant rule (reg §3, verbatim obligation).** A directive that moves away from an earlier ADR
> MUST, in one act: (1) write BOTH directions of the supersession/amendment edge AND (2) flip the stale ADR's
> status off `accepted`/`Accepted`/`Proposed`/`deprecated`. Skipping either IS the contradiction (C-13/P2 was
> a half-edge; P5/0335 was "nothing at all"). G-XA enforces this; every SUPERSEDE/ARCHIVE row below obeys it.

---

## §1 — CROSS-LANE EXECUTION ORDER (the one screen)

Read top-to-bottom. Each lane is one squash-PR in the WIP serial ralph loop (UNIFIED-EXECUTION-PLAN §1);
the gate in the right column must be GREEN-on-the-candidate-SHA before the lane's amendments may merge.

```
PHASE-0 MUST BE DONE (FIREWALL REAL):  ADR-0515 Accepted · oya-ci-required producer BLOCKS · 4 gates automated_blocking_now
  │  door:one-way + founder sign-off (Phase-0 EXIT gate)  — KEYSTONE status-enum + no-dangling-ref invariant LIVE in G-XA
  ▼
A-INTEGRITY  ── runs FIRST.  status-enum + no-dangling-ref are the substrate every later edge-write needs.
  │  (status-enum value · dup-0377 renumber · phantom-edge repoints · status-vs-edge §2A drifts · 0057 ARCHIVE edge-first)
  │  verified by: G-XA (edges/status) + G-SR (0057 git-mv)
  ▼
A-STRUCT  ── retarget the flat-crates BLOCKER gate (FC-2) so it stops enforcing the dead topology; pure-split.
  │  (0015 SUPERSEDE · 0509 SUPERSEDE · flat-crates FC-1..5 · {oya,cloud} two-tree · 0067 reshape)
  │  verified by: G-XA (0015/0509 edges) + G-TA (untracked crates/ orphan; path reachability)  · PRE: retarget lean-a1 gate
  ▼
A-FOUNDRY  ── ONLY AFTER the foundry-vocab enum is live in G-TA.  Eradicate the word; the big rename batch.
  │  (foundry → intelligence / governance-own-service / vcs-retired · 0136/0137/0143 SUPERSEDE · 0097/0101/0102/0138 ARCHIVE · 0018 glossary row DELETE · Cedar oyatie.foundry.* )
  │  verified by: G-TA (foundry forbidden-vocab = row RED; 0-residue outside Palantir-Foundry carve-out) + G-XA (supersede edges) + G-SR (4 ARCHIVE git-mv)
  ▼
A-CI  ── the 6→1 cluster + Forgejo eradication.  Needs A-INTEGRITY's edge substrate + the foundry-vocab gate.
  │  (0160/0374/0380 supersede-edge writes P1/P3/P4 · 0053/0103 SUPERSEDE · 0515 Forgejo eradication · 0116/0133/0173/0363/0181/0202 + 036x change-flow + GH-Actions→oya-ci + ArgoCD-as-bridge)
  │  verified by: G-XA (P1/P3/P4 reciprocal + superseded-on-cutover status) + G-TA (Forgejo forbidden-vocab) + G-AR (no fake "enforced")
  ▼
A-IDENTITY  ── the dual-bridge fix.  Needs A-INTEGRITY (the 0476 phantom-0421 repoint) done first.
  │  (0187 SUPERSEDE-as-endpoint P2 · 0476 supersedes:[0421]→[0187] · Zitadel rejected→adopted Phase-1 bridge)
  │  verified by: G-XA (P2 double-write + half-edge invariant)
  ▼
A-DATA  ── propagate the vendor rulings (Kafka→Pulsar, Redis→Valkey) + the D-D1 temporal keystone.
  │  (0059/0091/0153/0154/0169/0192/0193 Kafka→Pulsar · 0191/0208 Redis→Valkey · 0006 effective-dating/consistency-token · 0184 owned-endpoint · 0377-kafka dup-id is A-INTEGRITY)
  │  verified by: G-XA (vendor-ruling propagation = unpropagated_decision) + G-TA (path)
  ▼
A-HYPERSCALER  ── FIX-DON'T-DISCARD SPOF/scale set (the rows that don't fit the mechanical lanes).
  │  (0028/0031 SPOFs · 0202 ArgoCD-as-engine→adopt-pattern · 0482 bespoke-roadmap→D8 gate · 0029/0030/0062/0067/0098/0508 M0/D8-gating)
  │  verified by: G-AR (no fake-green scale claim) + G-XA (D8/D-SEQ edges)  · NOTE: 0202 supersede-framing coordinates with A-CI; 0067 reshape coordinates with A-STRUCT
  ▼
A-TASTE  ── LAST.  over-engineering / sequencing / reconcile nits that depend on nothing downstream.
  │  (0001 0008/0034 KR-minor-age · 0017 GitHub-ratchet · 0109 0159 0203 0234 0237 0238 + the M0-gate framings not owned by A-HYPERSCALER)
  │  verified by: G-XA (reconcile = status_disagreement) + G-AR (no premature "from day one")
  ▼
KEEP-minor batch (no-op touch-ups) ── ride each lane's mechanical sweep; never a standalone lane.  verified by: G-TA + G-XA
  ▼
UNLOCKS PHASE-2 (doc-reorg 44→6 Diátaxis · OWNERS/reachability closure · net-new ADRs)  — decision-record L200
```

**Why this order (cross-lane dependency logic):**
- **A-INTEGRITY precedes everything that writes an edge.** The KEYSTONE status-enum value + the no-dangling-ref
  invariant are the *substrate* the SUPERSEDE edge-writes (A-CI P1/P3/P4, A-IDENTITY P2, A-FOUNDRY 0136/0137/0143,
  A-STRUCT 0015/0509) all depend on — you cannot legally write `superseded-on-cutover` until the enum value
  exists, and the phantom-edge repoints (C-20: 0421/0409/0397/0088/0012) must land before the edges that
  reference the renumbered/repointed ids. dup-0377 renumber must also precede A-DATA's 0377-kafka content
  amend (you amend the surviving id, not the collision). (reg §1 C-1/C-4/C-16/C-17/C-20 + §2A drifts.)
- **A-STRUCT retargets the live BLOCKER gate FIRST (FC-2, HIGHEST SEVERITY).** `governance-lanes/flat-crates.md`
  is an Accepted BLOCKER that enforces the *dead* depth-3 topology (rejects 0512's depth-5). It must be retired
  / folded into the wired `lean-a1-architecture` lane **before** the pure-split path renames land, or the gate
  rejects the new structure. (reg §1 C-3 / FC-2.)
- **A-FOUNDRY opens ONLY after the foundry forbidden-vocab enum is live in G-TA.** Per D-FOUNDRY-CLARIFY the
  word is FORBIDDEN VOCAB; the gate must be able to fail any reintroduction *before* the sweep, else the sweep
  has no firewall and re-drifts. (decision-record L204.)
- **A-CI needs both A-INTEGRITY (edge substrate) and the foundry-vocab gate** (0181/0202/0363 carry foundry +
  Forgejo residue). Forgejo eradication is its own systematic sweep, gated like foundry. (D-FORGE-CLARIFY L207.)
- **A-IDENTITY's 0476 fix is half-A-INTEGRITY** (the phantom `0421` repoint is a §1 C-20 phantom-edge) and
  half-A-IDENTITY (the Zitadel rejected→adopted flip). The integrity half lands in A-INTEGRITY; the doctrine
  half here. (reg §1 C-13/C-21.)
- **A-HYPERSCALER is carved out** because the SPOF/scale FIX-DON'T-DISCARD set (decision-record L210) spans
  A-TASTE + A-CI + A-STRUCT and needs the single D8/D-SEQ capacity-gate framing applied coherently; splitting
  it across lanes would fragment the one architectural call. It runs after the mechanical lanes so its
  reshapes (0067, 0202) layer onto an already-clean structure/CI canon.

---

## §2 — THE LANES (each: owned ADRs/fixes · intra-lane order · verifying gate · door:one-way points)

> **Door discipline (every lane).** Every source mutation is door:one-way + per-PR founder sign-off
> (decision-record L201; FIREWALL-PLAN §6.3 #4). The **SUPERSEDE/ARCHIVE overrules** and the **ADR-0515
> self-fix** (adding 0160/0374/0380 to its `supersedes`) are named door:one-way points called out inline.
> Verification is a **separate verifier lane vs the real source** — no self-approval, no phantom findings.

---

### A-INTEGRITY — status-enum · supersession-edges · phantom-refs · dup-id (RUNS FIRST)
**Owns (reg §1 + §4 + §2A):** the graph substrate. CRIT C-1, C-4, C-10, C-16, C-17, C-20; HIGH H-2, H-3,
H-10, H-14, H-21; the §2A status-vs-edge drifts H-SE-0316 / -0358 / -0482 / -0052 / -0054 (H-SE-0363 is
content-CI → A-CI). ARCHIVE **0057** (`reg §4:0057`). AMEND **0119** (`reg §4:0119` back-edge),
**0144** (wrong 0140-Cedar edge), **0148/0182** (phantom-0150→0183, `reg §4:0148,0182` HIGH), **0150**
(fix every doc citing it as Cedar), **0069** (phantom-0088 + filenames), **0370** (supersession/reconcile
edges to 0378), **0331/0330/0332/0333/0350** (dangling related-filenames / stale 0316 / 0263 edges),
**0117** (Kyverno→0379 + vcs refs share-cut with A-CI), the **phantom-edge AMEND set 0478/0479/0480/0481**
(`reg §4` — `supersedes:[0457/0429/0443]` + `amends:[0428]`+0409/0434; their content tails ClickHouse→A-DATA
[0479], D8→A-HYPERSCALER [0480], Forgejo→A-CI [0481]), and the **stale-edge / vocab mechanical sweep
0152/0157/0176/0177/0178** (`reg §4` — drop the stale 0044 mesh edges; `dr_tier`/`tenant_class` namespacing
per D12; PARC split; the service-ADR path touch-ups). The C-1 Proposed-foundation ratify/demote
(0002/0007/0022/0111/0392/0408 → D14 Proposed-ledger) lands here as the dependency-integrity precondition.

**Intra-lane order:**
1. **Land the KEYSTONE first** (it is a Phase-0 deliverable consumed here): confirm G-XA carries the
   `superseded-on-cutover`/`bridge-unratified` status value + the no-dangling-ref/supersession-completeness
   invariant, RED/GREEN-proven (reg §3 KEYSTONE). *door:one-way — overrules the status enum shape.*
2. **dup-`0377` renumber** (the one allowed mid-space renumber; integrity fix, not re-foundation) — must
   precede A-DATA's 0377-kafka content amend. (reg §4:0377-kafka "renumber the duplicate-0377 id collision".)
3. **Phantom-edge repoints (C-20):** 0421→(0187 for the IdP case, handed to A-IDENTITY) · 0409/0397/0088/0012
   removed/repointed · 0150-as-policy → 0183 (C-16) across 0239/0148/0182 + 0069 filenames + 0331/0330 names.
4. **§2A status-vs-edge drifts:** flip 0316/0358 (Proposed→Superseded), 0482 non-ADR `amended_by`→real id,
   0052 body↔frontmatter, 0054 deprecated→Superseded+frontmatter edge.
5. **Back-edges + dangling:** 0119 (0131 partial-supersedes), H-2 0012, H-14, H-21 0183→0379.
6. **ARCHIVE 0057 — fix the dangling+colliding supersedes edge FIRST, then `git mv` to `_archive/`** (never
   `rm`; G-SR). *door:one-way — the ARCHIVE overrules.* (reg §1 C-4; §4:0057.)
7. **C-1 Proposed-foundation:** ratify-or-demote 0002/0007/0022/0111/0392/0408 per D14; record in the
   Proposed-ledger so dependents (0001/0028/0034/0099/0116/0515) are not Accepted-on-Proposed.

**Verifying gates:** G-XA (every edge reciprocal; dup-0377; status legal; half-edge RED) + G-SR (0057 git-mv) + G-TA (no new dangling row). **PRE:** G-XA KEYSTONE extension must be `automated_blocking_now`.

---

### A-STRUCT — flat-crates eradication · pure-split (RUNS AFTER A-INTEGRITY)
**Owns (reg §1 + §4):** CRIT C-3 (folds FC-1..5), C-5; HIGH H-11, H-20. SUPERSEDE **0015**
(`reg §4:0015`, FC-1 — edge already exists, flip `accepted`→Superseded under 0512/0131) + **0509**
(`reg §4:0509`, status drift — 0512 supersedes but still Accepted; flip + back-pointer + repoint 5 citers, FC-5).
AMEND **0056/0058** ({oya,cloud} two-tree topology), **0001/0011** (flat-catalog paths), **0067** (ops
mega-service reshape — coordinates with A-HYPERSCALER), **0132/0478** (`microservices/`→{oya,cloud}),
H-20 bulk path rename fleet-wide.

**Intra-lane order:**
1. **FC-2 (HIGHEST SEVERITY) — retarget the live BLOCKER gate FIRST.** Retire `oya-governance-flat-crates`
   (`governance-lanes/flat-crates.md` BLOCKER, depth-3); fold intent into the wired `lean-a1-architecture`
   lane (`lanes.yaml:485-493`, `validate architecture-boundaries`) per `ADR-0512:62`. Until this lands, the
   gate rejects the pure-split renames. (reg §1 C-3/FC-2.) *door:one-way — overrules the active gate.*
2. **FC-1 — flip 0015 → Superseded** (edge exists). **0509 → Superseded** + back-pointer 0509→0512 + repoint
   5 citers. *door:one-way — the SUPERSEDE overrules.* (`reg §4:0015,0509`.)
3. **FC-3 — delete the untracked code-empty `crates/` dir** (`git ls-files crates/`=0; G-TA orphan).
4. **FC-4 — sweep ~50 stale LOCATION/gate refs** (keep ~650 surviving-NAME headers).
5. **FC-5 — ADR-INDEX rows for 0509/0512; flip ADR-INDEX 0015/0357 rows.**
6. **Pure-split topology:** 0056/0058/0001/0011 + H-20 bulk additive path rename to `{oya,cloud}/<service>/`
   (mechanical, D13-AMENDED); 0067 ops-reshape (hand-off shape to A-HYPERSCALER).

**Verifying gates:** G-XA (0015/0509 edges + ADR-INDEX agreement) + G-TA (untracked `crates/` orphan; every
renamed path reachable). **PRE:** FC-2 gate-retarget complete; A-INTEGRITY edge substrate live.

---

### A-FOUNDRY — eradicate the FORBIDDEN VOCAB (RUNS AFTER the foundry-vocab gate is live)
**Owns (reg §1 + §4 + D-FOUNDRY-CLARIFY):** CRIT C-2, C-8, C-11, C-14; HIGH H-8. SUPERSEDE **0136/0137/0143**
(`reg §4` — foundry-µservice/BCs/release → cloud-intelligence framework; salvage 6-BC reasoning). ARCHIVE
**0097/0101/0102/0138** (`reg §4` — cosmetic rename / temporary-bypass-promoted / settings-render / Strangler-
to-dead-address). AMEND the de-foundry batch: **0001 0011 0018(glossary-row DELETE) 0062 0063 0065 0066 0067
0069 0091 0096 0098 0099 0100 0139 0146 0158 0161 0162 0163 0164 0167 0168 0174 0189 0192 0200 0220 0222 0258
0335 0351 0389 0390** + Cedar `oyatie.foundry.*` principals (`reg §4` each; H-8 list). **0335** carries the
**P5 superseded-on-cutover marker** + the 3-way carve-out + supersession banners on 0220/0239 (`reg §4:0335`).

**Routing (D-FOUNDRY-CLARIFY L203-204, SOURCE-BACKED 3-way):** foundry-platform/name → **intelligence**;
**governance stays its OWN service**; agentic-VCS → **retired**. `foundry` is FORBIDDEN VOCAB (delete the
0018 glossary row; the word is gate-blocked). HARD carve-out: external **"Palantir Foundry"** (kept).
**OPEN FLAG (do NOT silently fold):** founder's verbal "all → intelligence" is NOT yet source-backed and
contradicts `ADR-0363:79` (governance own-service); **until ruled, A-FOUNDRY follows the SOURCE 3-way** —
do not dissolve governance or un-retire vcs (decision-record L204). This is a per-lane founder sign-off item.

**Intra-lane order:**
1. **PRE-CHECK: the foundry forbidden-vocab enum must be LIVE + RED/GREEN-proven in G-TA** before any sweep
   (so reintroduction is un-mergeable). (decision-record L204.) *door:one-way — overrules the vocab enum.*
2. **0018 glossary `Foundry` row DELETE** (C-2 the terminology-canon ADR) — template-first for the batch.
3. **SUPERSEDE 0136/0137/0143** into the cloud-intelligence framework successor (write reciprocal edges +
   status; G-XA). *door:one-way — the SUPERSEDE overrules.*
4. **ARCHIVE 0097/0101/0102/0138** (`git mv` → `_archive/`; G-SR; salvage atomic-render+sref / template).
   *door:one-way — the ARCHIVE overrules.*
5. **The rename batch** (intelligence / governance / retired) across the H-8 list + Cedar principals;
   coordinate the 0200 sandbox-class enum + 0258 `oya.foundry.v2.*` mesh package (touch code — sequence).
6. **0335 P5 marker + 3-way carve-out + 0220/0239 banners** (the consolidating ADR; its missing artifact is
   the superseded-on-cutover marker — reg §1 note on P5 + §4:0335).

**Verifying gates:** G-TA (foundry = forbidden-vocab row RED; 0-residue outside Palantir carve-out) + G-XA
(0136/0137/0143 reciprocal edges + 0335 marker legal) + G-SR (4 ARCHIVE git-mv). **PRE:** foundry-vocab enum
live in G-TA; A-INTEGRITY edge substrate.

---

### A-CI — the 6→1 CI/CD cluster · Forgejo eradication (RUNS AFTER A-INTEGRITY + foundry-vocab gate)
**Owns (reg §1 + §4):** CRIT C-6, C-9, C-12(=P1), C-15, C-18(=P3), C-19(=P4); HIGH H-4, H-9, H-15, H-16,
H-SE-0363. SUPERSEDE **0053/0103** (`reg §4` — dead grit/icm; by 0116/0363/0515; flip status). **Bridge /
superseded-on-cutover** edge-writes: **0374 (P3)** + **0380 (P4)** by 0515; **0239** mark-historical by 0335
(banner). **Edge-write 0160 (P1)** (clean SUPERSEDE owned graph-wise here even though `reg §4:0160` lists it
in the clean-9: write `0160.superseded_by←[0515]` + add to `0515.supersedes`). AMEND the CI/Forgejo batch:
**0017(GitHub-ratchet) 0063 0066 0067 0116 0123 0133 0171 0173 0181 0202 0234 0363 0365 0366 0367 0369 0370
0375 0378 0383 0391 0393 0515** (`reg §4` each — GH-Actions→oya-ci · Cedar→PARC · Jenkins/ArgoCD→oya-ci ·
ArgoCD-as-bridge · Forgejo→GitHub-interim).

**THE ADR-0515 SELF-FIX (named door:one-way).** `reg §1 C-12/C-18/C-19` + `reg §4:0515`: **add 0160, 0374,
0380 to `0515.supersedes`** (P1/P3/P4 omissions) AND eradicate Forgejo from `0515:76,96` AND ratify/gate the
Proposed-0408/0392 deps. This is the keystone self-correction of the Phase-0 ratifying ADR; founder sign-off
required. (D-FORGE-CLARIFY L207 names 0515's "Forgejo-native"/`ForgeAdapter` prose as the fix target.)

**Intra-lane order:**
1. **PRE: A-INTEGRITY's status-enum + no-dangling-ref live; foundry-vocab gate live** (0181/0202/0363 carry
   foundry + Forgejo residue).
2. **The 0515 self-fix** (add 0160/0374/0380 to supersedes; Forgejo eradication; 0408/0392 ratify-gate).
   *door:one-way + founder sign-off — the ADR-0515 self-fix.*
3. **Write the bridge edges with the keystone status value:** 0374(P3)/0380(P4) → `superseded-on-cutover` by
   0515 (keep the physical Jenkins/Forgejo scaffold as the unratified bridge); 0160(P1) clean-supersede by
   0515; 0239 mark-historical by 0335 (banner). *door:one-way — the SUPERSEDE/mark-historical overrules.*
4. **SUPERSEDE 0053/0103** (grit/icm; flip status). *door:one-way.*
5. **Forgejo systematic eradication sweep** (D-FORGE-CLARIFY): 0515 + 0363 + 0510(cutover) + the
   `oya-ci-controller-forgejo-adapter` framing + 0375/0383/0391/0369/0378/0366/0365 substrate refs →
   GitHub-interim → Sapling-bespoke; the producer is already correct (GitHub poster).
6. **GH-Actions→oya-ci + masterplan-authority-inversion** (H-9: 0063/0066/0067 read ADR front-matter, not
   masterplan §2.1) + **ArgoCD-as-bridge** (C-15/0202: reframe Tier-A → owned oya-cd, keep Tier-B/C) +
   **0173 stale-pick sweep** (HIGH leverage) + 0133 Argo/Flagger reconcile + 0181 Flagger→Argo + 0116
   Foundry-pipeline-mapping→oya-ci + 0123/0117 dead-vcs refs.

**Verifying gates:** G-XA (P1/P3/P4 reciprocal + `superseded-on-cutover` status legal; 0515 supersedes
complete) + G-TA (Forgejo = forbidden-vocab row RED) + G-AR (no advisory-claiming-enforced in the CI lanes).
**PRE:** A-INTEGRITY substrate + foundry-vocab gate.

---

### A-IDENTITY — the dual-bridge fix (RUNS AFTER A-INTEGRITY's phantom-0421 repoint)
**Owns (reg §1 + §4):** CRIT C-13(=P2), C-21. SUPERSEDE-as-endpoint **0187** (`reg §4:0187` — Zitadel-canonical
vs D5; `0187.superseded_by←[0476]`, status→superseded-as-endpoint; keep operative as the Phase-1 bridge).
AMEND **0476** (`reg §4:0476` — `supersedes:[0421]→[0187]` DOUBLE write; Zitadel rejected→adopted Phase-1
bridge; long-term endpoint = bespoke oya-identity; Forgejo; phantom-0409). AMEND **0188** (RP-home reword once
0187 demotes) + **0189** (PARC ref) ride here.

**Intra-lane order:**
1. **PRE: A-INTEGRITY has repointed the phantom `0421`** (C-20) so the 0476 `supersedes` write targets a real id.
2. **P2 DOUBLE write:** `0476.supersedes:[0421]→[0187]` AND `0187.superseded_by←[0476]` (both directions,
   one act — the half-edge invariant; G-XA). *door:one-way — the SUPERSEDE-as-endpoint overrules.*
3. **Flip 0476 Zitadel rejected→adopted Phase-1 bridge** (inverting the D5-contradiction); status
   `superseded-on-cutover`-class for the long-term bespoke-oya-identity endpoint.
4. **0188 RP-home reword + 0189 PARC ref.**

**Verifying gates:** G-XA (P2 double-write reciprocal; half-edge RED if only one direction). **PRE:**
A-INTEGRITY phantom-0421 repoint + status-enum live.

---

### A-DATA — propagate the vendor rulings + the D-D1 temporal keystone
**Owns (reg §1 + §4):** CRIT C-7; HIGH H-5, H-18, H-19. AMEND **0059/0091/0153/0154/0169/0192/0193** (Kafka→
Pulsar; eventual fan-out only; D-EVENT) + **0191/0208** (Redis→Valkey; H-18) + **0184** (owned-endpoint vs
"None planned" Postgres) + **0166** (Apicurio port/ratchet) + **0006** (the D-D1 keystone — effective-dating
kernel temporal type + consistency token + repair the self-rename tautology; `reg §4:0006`, H-5/U-2).
0060 inherited-Kafka row + 0195 Pulsar-naming on Kafka-Engine default + **0175** (`reg §4` tenant-lifecycle
saga; confirm 0222 ratification; reconcile D-D1) + the **0479** ClickHouse-as-endpoint-vs-D4 content tail
(its phantom-edge act is owned by A-INTEGRITY). *(0377-kafka content is amended here but its dup-id renumber
is owned by A-INTEGRITY.)*

**Intra-lane order:**
1. **PRE: A-INTEGRITY dup-0377 renumber done** (amend the surviving id).
2. **Vendor-ruling propagation:** Kafka→Pulsar across the 0059/0091/0153/0154/0169/0192/0193/0060/0195 set
   (adopt the D-D1 consistency-token model on the critical path; Kafka = wire-compat only). H-19 phantom-0397
   fix rides A-INTEGRITY.
3. **Redis→Valkey** (0191/0208; D12).
4. **0184 owned-endpoint** tag-transitional; 0166 port+ratchet.
5. **0006 effective-dating + consistency token** (net-new kernel temporal type — the D-D1 keystone property,
   payroll-close read-your-writes) + the Ontology→Ontology self-rename repair (D11(b)).

**Verifying gates:** G-XA (Kafka→Pulsar / Redis→Valkey = `unpropagated_decision` until written; 0006 spec
agreement) + G-TA (path). **PRE:** A-INTEGRITY dup-0377.

---

### A-HYPERSCALER — FIX-DON'T-DISCARD SPOF / scale (NEW lane; the SPOF/scale fixes that don't fit elsewhere)
**Owns (reg §1 §2 + decision-record L210 FIX-DON'T-DISCARD set):** HIGH H-12, H-13/U-1, H-22, H-23; §2 SPOFs
+ over-eng + the CRITICAL own-vs-adopt. AMEND **0028** (same-provider primary+secondary SPOF → multi-provider
control-plane isolation; H-23) · **0031** (literal-singleton ads-gate → logically-single/physically-replicated;
H-22) · **0202** (ArgoCD-as-engine → adopt-pattern/owned oya-cd; C-15 — *supersede-framing coordinates with
A-CI*) · **0482** (unbounded-ambition → insert D8 capacity-budget gate; non-ADR `amended_by` rides
A-INTEGRITY) · **0029** (Workspace/M365 parity → M0 sequencing gate) · **0030** (from-scratch search →
own-vertical M0 gate; keep KR morphology moat) · **0062** (day-1-100M → "M0-gated, scale-on-proven-demand") ·
**0067** (ops mega-service decompose — *reshape coordinates with A-STRUCT*) · **0098** (power-loss → flip
durability default to `fsync(parent_dir)`; H-13/U-1) · **0508** (owned-silicon → bind to D8/D-SEQ M0; KEEP-
minor but hyperscaler-tagged, `reg §4:0508`).

**Why a separate lane.** Per D-DISPOSITIONS-RATIFIED these are ratified **FIX-DON'T-DISCARD** (decision-record
L210) and apply ONE coherent D8/D-SEQ capacity-gate + multi-provider-isolation framing that would otherwise
fragment across A-TASTE/A-CI/A-STRUCT. The owned rows whose *substance* lives in another lane (0202 supersede
edge → A-CI; 0067 reshape → A-STRUCT; 0482 dangling-amender → A-INTEGRITY) are cross-referenced, not
duplicated — this lane owns the **scale/SPOF remediation content**, the other lane owns the edge/structure act.

**Intra-lane order:** 1. SPOF fixes (0028 multi-provider, 0031 replicated serving path) — table-stakes
availability. 2. The own-vs-adopt content for 0202 (ArgoCD pattern, not engine) — hand the edge to A-CI.
3. The M0/D8 capacity-gate framings (0029/0030/0062/0067/0482/0508) — one consistent "M0-gated, not day-1"
rewrite. 4. 0098 durability flip.

**Verifying gates:** G-AR (no fake-green "from day one"/scale claim without a wired gate) + G-XA (D8/D-SEQ
edges; 0202 own-vs-adopt agreement). **PRE:** A-STRUCT (0067 structure) + A-CI (0202 edge) landed.

---

### A-TASTE — over-engineering / sequencing / reconcile nits (RUNS LAST)
**Owns (reg §1 + §4):** HIGH H-1, H-6, H-7, H-11(taste half), H-17. AMEND **0001(residual taste) 0029/0030/
0062/0067** (the framings NOT owned by A-HYPERSCALER — i.e. any taste residue once the scale-gate is applied),
**0008/0034** (KR minor-age reconcile, H-1), **0017** (GitHub-ratchet taste half), **0109** ("automation cost
≈0" robust-not-false), **0159** (feature-flag owned-vs-Flipt reconcile, H-17), **0203** (doc-engine vs
D-DOCORG), **0234/0238** (Connect/super-app self-inconsistency), **0237** (Strangler trigger re-derive),
**0388** (Diátaxis topology). KEEP+AMEND **0237/0351/0365/0377-kafka/0389/0391** content-only tails ride
their primary lanes; the pure-taste residue lands here.

**Intra-lane order:** 1. Reconciles with no downstream dep (0008/0034 KR-minor-age; 0159 flag; 0017 ratchet).
2. Self-inconsistency fixes (0234/0238/0237). 3. Doc-org (0203/0388 → single D-DOCORG Diátaxis). 4. Residual
M0-framing taste once A-HYPERSCALER's scale-gate is in (so no double-edit).

**Verifying gates:** G-XA (reconcile = `status_disagreement` resolved) + G-AR (no premature "from day one").
**PRE:** A-HYPERSCALER (so taste residue layers on the gated framing); nothing downstream depends on A-TASTE.

---

### KEEP-minor batch (no standalone lane)
The ~108 KEEP dispositions (incl. KEEP+AMEND content tails) are mostly **no-op**; the minor path/vocab/
citation touch-ups ride the mechanical sweep of whichever lane owns the file. **Batched touch-ups by lane:**
- **path/`microservices/`→{oya,cloud}:** 0185 0186 0196 0210 0240 + every path-bearing KEEP — ride **A-STRUCT** H-20 sweep.
- **foundry-word KEEP touch-ups:** 0051 0060 0090 0104 0105 0106 0108 0115 0122 0130 0151 0167 0168 0189 0200 0205 0222 0240 0241 0258 0329 0351 0362 — ride **A-FOUNDRY** vocab sweep.
- **Flagger→Argo / vcs-ref KEEP touch-ups:** 0165 0172 0180 0375 0383 — ride **A-CI**.
- **Pulsar/Valkey KEEP touch-ups:** 0061 0194 0195 0196 0197 — ride **A-DATA**.
- **integrity-ref KEEP touch-ups:** 0055 0117 0128 0142 0179 0331 0330 0332 0333 0350 — ride **A-INTEGRITY**.
- **identity KEEP touch-ups:** 0188 0189 0190 0506 0507 — ride **A-IDENTITY**.
- **pure-KEEP exemplars (no touch):** 0008 0034 0083 0092 0093 0094 0095 0118 0123 0129 0131 0132 0135 0145 0149 0155 0156 0198 0199 0201 0204 0206 0207 0209 0223 0235 0364 0368 0371 0373 0376 0379 0390 0508 0512 — verified-clean, no edit (G-TA confirms reachable/owned only).

**Verifying gate:** G-TA + G-XA (a KEEP touch-up must not introduce a dangling ref or vocab residue).

---

## §3 — COVERAGE LEDGER (every ratified mutating row is placed)

> Confirms: every ratified **ARCHIVE / SUPERSEDE / AMEND** row from `reg §4` is placed in exactly one
> **primary** lane (content owner). Rows whose graph-act lives elsewhere are cross-referenced (XREF), not
> dropped or double-counted.

| Disp | count | Placement |
|---|---|---|
| **ARCHIVE (5)** | 5 | **0057**→A-INTEGRITY · **0097 0101 0102 0138**→A-FOUNDRY |
| **SUPERSEDE — 9 clean** | 9 | **0015 0509**→A-STRUCT · **0053 0103 0160**→A-CI · **0136 0137 0143**→A-FOUNDRY · **0187**→A-IDENTITY |
| **SUPERSEDE — 3 bridge/on-cutover** | 3 | **0374 0380**→A-CI (P3/P4) · **0239**→A-CI mark-historical edge / banner obligation shared with A-INTEGRITY (C-17 phantom-0150 anchor) |
| **AMEND (76) + KEEP+AMEND (6)** | 82 | distributed below |

**AMEND/KEEP+AMEND primary-lane distribution (82 rows):**
- **A-INTEGRITY:** 0069 0117 0119 0144 0148 0150 0182 0330 0331 0332 0333 0350 0370 (+ §2A drifts 0316 0358 0482 0052 0054 as edge/status fixes; 0482 content→A-HYPERSCALER) — **13** content + drifts.
- **A-STRUCT:** 0001 0011 0056 0058 0067 0132 0478 — **7** (0001/0011/0067 shared content; primary structural act here).
- **A-FOUNDRY:** 0018 0062 0063 0065 0066 0069 0091 0096 0098 0099 0100 0139 0146 0158 0161 0162 0163 0164 0167 0168 0174 0189 0192 0200 0220 0222 0258 0335 0351 0389 0390 — **31** (de-foundry batch; 0062/0098 scale-content→A-HYPERSCALER).
- **A-CI:** 0017 0063 0066 0067 0116 0123 0133 0171 0173 0181 0202 0234 0363 0365 0366 0367 0369 0370 0375 0378 0383 0391 0393 0515 — **24** (CI/Forgejo; 0202 scale→A-HYPERSCALER).
- **A-IDENTITY:** 0188 0189 0476 — **3** (0187 SUPERSEDE counted above).
- **A-DATA:** 0006 0059 0060 0091 0153 0154 0166 0169 0184 0191 0192 0193 0195 0208 0377-kafka — **15** (content; 0377 dup-id→A-INTEGRITY; 0091/0192 de-foundry→A-FOUNDRY).
- **A-HYPERSCALER:** 0028 0029 0030 0031 0062 0067 0098 0202 0482 0508 — **10** (scale/SPOF content; structural/edge acts XREF to A-STRUCT/A-CI/A-INTEGRITY).
- **A-TASTE:** 0008 0034 0109 0159 0203 0234 0237 0238 0388 + residual-taste of 0001/0017/0029/0030/0062/0067 — **~9** primary + taste-residue.

> **Shared rows (intentional, not double-disposition).** Several ADRs carry fixes for >1 lane (e.g. **0067**
> = A-STRUCT reshape + A-HYPERSCALER decompose + A-CI GH-Actions/grit + A-FOUNDRY foundry; **0202** =
> A-CI edge + A-HYPERSCALER own-vs-adopt; **0335** = A-FOUNDRY content + A-CI 0239 banner). Each lane owns a
> DISTINCT act on the file; the file is touched once per lane's sweep in serial order, never re-disposed. This
> is the reg's own model (reg §4 one-line-why frequently spans lanes; reg §1 assigns the *contradiction* owner).

**No disposition dropped.** All 5 ARCHIVE + 12 SUPERSEDE + 82 AMEND/KEEP+AMEND rows are placed. The 6 §2A
status-vs-edge drifts are folded into A-INTEGRITY (0316/0358/0482/0052/0054) + A-CI (0363) as edge/status
fixes, exactly as the reg models them (reg §4 NOTE on accounting). The 3 REFUTED items (R-1/R-2/R-3) are
carried, not actioned (reg §1 REFUTED).

---

## §4 — DOOR:ONE-WAY + FOUNDER-SIGN-OFF REGISTER (every source mutation)

Per decision-record L201 + FIREWALL-PLAN §6.3: **every source mutation is door:one-way + per-PR founder
sign-off.** The named irreversible points:

| # | Door point | Lane | Why irreversible |
|---|---|---|---|
| 1 | **Phase-0 EXIT (FIREWALL REAL)** — prerequisite | — | unlocks all of Phase-1; KEYSTONE status-enum + no-dangling-ref invariant now live in G-XA |
| 2 | **Every SUPERSEDE edge-write** | A-STRUCT(0015/0509) · A-FOUNDRY(0136/0137/0143) · A-CI(0053/0103/0160/0374/0380/0239) · A-IDENTITY(0187) | supersession is build-first-cutover-later; the SUPERSEDE OVERRULES — once the status flips + edge lands it is not auto-reversible |
| 3 | **Every ARCHIVE** | A-INTEGRITY(0057) · A-FOUNDRY(0097/0101/0102/0138) | `git mv`→`_archive/` (never `rm`), second-verifier-gated; the ARCHIVE OVERRULES |
| 4 | **The ADR-0515 self-fix** | A-CI | adds 0160/0374/0380 to `0515.supersedes` (P1/P3/P4) + Forgejo eradication + 0408/0392 ratify-gate; corrects the Phase-0 ratifying ADR — founder sign-off |
| 5 | **0335 P5 superseded-on-cutover marker** | A-FOUNDRY | the consolidating ADR gains the missing bridge marker + 0220/0239 banners |
| 6 | **A-FOUNDRY routing flag** | A-FOUNDRY | "all → intelligence" is NOT source-backed (contradicts ADR-0363:79); founder must confirm 3-way vs fold before the sweep dissolves governance/un-retires vcs |
| 7 | **C-1 Proposed-foundation ratify-or-demote** | A-INTEGRITY | ratifying 0002/0007/0022/0111/0392/0408 (D14 Proposed-ledger) is a foundation commitment |
| 8 | **FC-2 flat-crates gate retarget** | A-STRUCT | retiring a live Accepted BLOCKER gate changes what the structure firewall enforces |
| 9 | **Each per-lane squash-PR** | all | "every source mutation" (canon L201); door:one-way ADRs cannot auto-merge (ADR-0365 decision-door) |

**Verification discipline (founder rule, every door):** a **separate verifier lane vs the real source** —
no self-approval in the authoring context; no phantom findings; never amend/delete on an unverified verdict.

---

## §5 — RETURN SUMMARY

### Lane headcounts (primary content owner; SUPERSEDE/ARCHIVE + AMEND/KEEP+AMEND)
| Lane | SUPERSEDE | ARCHIVE | AMEND/K+A (primary) | Lane total (primary rows) |
|---|---|---|---|---|
| **A-INTEGRITY** | — | 1 (0057) | 13 + 5 §2A drifts | ~19 |
| **A-STRUCT** | 2 (0015,0509) | — | 7 | 9 |
| **A-FOUNDRY** | 3 (0136,0137,0143) | 4 (0097,0101,0102,0138) | 31 | 38 |
| **A-CI** | 4 (0053,0103,0160,0374) + 0380 + 0239 = 6 | — | 24 | 30 |
| **A-IDENTITY** | 1 (0187) | — | 3 | 4 |
| **A-DATA** | — | — | 15 | 15 |
| **A-HYPERSCALER** (new) | — | — | 10 | 10 |
| **A-TASTE** | — | — | ~9 + taste-residue | ~9 |
| **KEEP-minor batch** | — | — | rides each lane | (~108 KEEP no-op, batched) |

*(Totals reconcile to the ratified set: 12 SUPERSEDE [9 clean + 3 bridge] + 5 ARCHIVE + 82 AMEND/K+A. Cross-lane
shared rows — 0067/0202/0335/0091/0192/0482/0001/0017/0062/0098 — are counted once at their primary content
owner and XREF'd at the secondary act; this is why per-lane primary totals sum above the unique-id count.)*

### Execution-order summary (the unlock chain)
```
[Phase-0 FIREWALL REAL]  → unlocks Phase-1  (G-XA KEYSTONE status-enum + no-dangling-ref LIVE)
  → A-INTEGRITY   (status-enum/no-dangling substrate; dup-0377; phantom repoints; 0057 ARCHIVE)   [G-XA,G-SR]
  → A-STRUCT      (FC-2 gate retarget FIRST; 0015/0509 SUPERSEDE; pure-split)                      [G-XA,G-TA]
  → A-FOUNDRY     (foundry-vocab gate live; 0018 row delete; SUPERSEDE 0136/7/3; ARCHIVE x4; sweep)[G-TA,G-XA,G-SR]
  → A-CI          (0515 self-fix; P1/P3/P4 edges; 0053/0103 SUPERSEDE; Forgejo eradication; GHA→oya-ci)[G-XA,G-TA,G-AR]
  → A-IDENTITY    (P2 0187/0476 double-write; Zitadel rejected→adopted bridge)                      [G-XA]
  → A-DATA        (Kafka→Pulsar; Redis→Valkey; 0006 effective-dating keystone)                      [G-XA,G-TA]
  → A-HYPERSCALER (0028/0031 SPOF; 0202 own-vs-adopt; D8/M0 gates; 0098 durability)                 [G-AR,G-XA]
  → A-TASTE       (KR-minor-age; reconciles; doc-org; residual M0-framing)                          [G-XA,G-AR]
  → KEEP-minor    (no-op touch-ups ride each lane's sweep)                                          [G-TA,G-XA]
  → unlocks PHASE-2 (doc-reorg 44→6 Diátaxis · OWNERS/reachability · net-new ADRs)
```
**Gate-unlock rule:** each lane's amendments may merge only when its right-column gate(s) are
`automated_blocking_now` + self-test-RED-proven on the candidate SHA (firewall-first; D-SEQUENCE L196-201).

### Dispositions that could not be placed
**None.** Every ratified ARCHIVE (5), SUPERSEDE (9 clean + 3 bridge), and AMEND/KEEP+AMEND (82) row from
`00-MASTER-CONTRADICTION-REGISTER §4` is placed in a primary lane with its verifying gate. The only items not
actioned are the **3 REFUTED** (R-1/R-2/R-3), which are carried-not-dropped by design (reg §1 REFUTED). The
single **open founder flag** (A-FOUNDRY "all → intelligence" vs source-backed 3-way, decision-record L204) is
a sign-off item, not an unplaced disposition.

---

*End PHASE-1 AMENDMENT-LANES PLAN. Authority: 00-MASTER-CONTRADICTION-REGISTER (SSOT/WHAT) + UNIFIED-EXECUTION-
PLAN (lane shape) + PHASE-0-FIREWALL-PLAN (firewall-first prerequisite) + decision-record D-SEQUENCE/D-FOUNDRY-
CLARIFY/D-FORGE-CLARIFY/D-DISPOSITIONS-RATIFIED. READ-ONLY organization of the ratified set; no re-audit, no new
findings; nothing mutated. STATUS: pending — gated on Phase-0 FIREWALL REAL + per-lane gate-green + door:one-way.*
