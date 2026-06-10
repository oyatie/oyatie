# UNIFIED EXECUTION PLAN — Oyatie consolidation (amendment + migration)

**STATUS: pending — gated on the WIP G0–G4 user gates + the live authority-flip.** Nothing executes against `jason931225/oyatie` until those clear (per D-MERGE). This MERGES two plans into one execution authority:

- **EXECUTION AUTHORITY (the HOW):** `source/.omc/plans/monorepo-consolidation-migration.md` (live-verified, ralplan-deliberate). Its framework governs ALL execution. **Read it as the operative procedure** — this doc does not duplicate it.
- **DECISION RULINGS (the WHAT):** `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (the ~30 founder D-decisions). The SSOT of what to change.
- This UNIFIED plan **supersedes `AMENDMENT-PLAN.md`** (whose live-repo assumptions — linear/one-doc-PR/signed-required — were corrected by the WIP's verified reality: **squash-only · `github-mirror` remote · signatures-not-live-yet-but-flipping · Buck2-whole-graph is the real gate**). Founder folds this into the `.omc` plan on commit; the uncommitted WIP file is left untouched.
- **Date:** 2026-06-06.

---

## 0. Two scopes, one queue

| Scope | What | Source |
|---|---|---|
| **AMENDMENT (A-lanes)** | fix source's EXISTING canon — foundry rename, integrity sweep, vocab, Proposed-ledger, CC-1..13, NEW ADRs — **amend-in-place + additive, NO renumber** (D13-amended) | AMENDMENT-PLAN rulings + decision record |
| **MIGRATION (M-lanes)** | bring the linux pilot + 5 siblings INTO source (L1 office … L11 framekernel) | WIP plan §6/§7 (verbatim authority) |

Both run **as lanes in the WIP's single serial ralph loop**, under its identical constraints (§1 below). Amendment lanes clean the canon FIRST; migration lanes land external code onto the cleaned canon.

---

## 1. Execution framework — ADOPTED WHOLESALE from the WIP plan (the spine)

Per **D-MERGE**, every lane (A and M) obeys the WIP plan's framework — do NOT re-derive it, follow `monorepo-consolidation-migration.md`:
- **Principles §1** — BUILD-TO-BOTH-GATES · authority-drift-first-class · migrate-clean-first/no_std-last · one-lane-one-squash-PR-whole-graph-gated · merge-not-duplicate · only-our-code-moves · canonical-homes-with-standing-tools/-exception · live-computed-identifiers · **FORGE-EXPLICIT (`github-mirror`, never `origin`/Forgejo)** · gate-before-start.
- **Gating pre-lanes §7** — **0.4** authority-snapshot + signing pre-provision (signing now DONE — SSH-signed verified) · **0.5** truthing + source/merge-surface manifests · **0.6** no_std inertness spike · **0.7** governance-file bootstrap. **All four MUST complete before any A- or M-lane.**
- **Ralph loop §8** — strict serial, one driver, one graph mutation; loop step 0 re-diffs live protection (G0 HALT on flip); rebase-on-dev → build-to-both-gates → squash-merge → rebase + re-run-authority-gate.
- **Live ground-truth §0** — live gate `github-lane-unlocker-required` (Buck2-only, signatures off); target `oya-ci-required`+signing (ADR-0513, in flight); squash-only; 723-member root workspace; `origin`=Forgejo → push `github-mirror`.
- **Verify-at-each-step (founder rule)** layered on: separate verifier lane vs primary sources; no phantom findings; no self-approval.

---

## 2. Unified lane queue (insert A-lanes before M-lanes in the serial loop)

```
PRE-LANES (gating, once):  0.4 authority+signing → 0.5 manifests → 0.6 no_std-inertness → 0.7 governance-bootstrap
                           [GATE-BEFORE-START + USER G0..G4]

AMENDMENT LANES (source-internal, clean existing canon — amend-in-place + additive, NO renumber):
  A1  foundry per-file rename (CORRECTED 4-WAY, current reality):
        platform → oya-intelligence-* (current home; cloud-intelligence RE-HOME is a DEFERRED campaign, §4)
        fitness  → oya-governance-*
        agentic-VCS → RETIRED (git+Forgejo+oya-ci)
        HARD carve-outs: Palantir-Foundry (43), Marlboro-Forge, the foundry→governance retirement record
        (coordinates with each M-lane's codename-rename pass; template-first)
  A2  integrity sweep — KCMVP/KISA restore · tautology fixes (0006 Ontology→Ontology) · ALL dangling supersedes/amends
        (against the STABLE existing id-space — no renumber, so refs are fixed once)
  A3  vocab namespacing — tier → autonomy_tier / eu_ai_act_risk_tier / dr_tier / storage_tier / tenant_class; 0163 tiers→stages
  A4  Proposed-ledger resolution — 132 Proposed → RATIFY (~122) / DROP (0325,0316,0349-canonical-claim) / KEEP / AMEND-0352
        + the CI-cluster drop (0349/0361/0359/0511 ADR-debt; Jenkins scaffold stays unratified bridge; resolve byp_adr_0349)
  A5  NEW / reshaped ADRs (ADDITIVE — into the live free block, NOT a renumber):
        oya-ci reshape (0513; supersede/relate 0511/0124, phase 0369/0367/0366) · unified safety-gate (D-SAFETY) ·
        KR EmploymentClassification enum as KR localization-pack model (D-KR) · infra-sovereignty ordered+M0 schedule (D-SEQ) ·
        domain-cohesion meta-ADR (D15) · masterplan-generated-wiring meta-ADR (D1) · data-engine-endpoint ADR (D-INTEL) ·
        amend cloud-intelligence docs to cite ADR-0389/0390
  A6  CC-1..CC-13 doc fixes (WF2 register) — masterplan-authority-inversion, Cedar→contract+PARC, Jenkins/ArgoCD→oya-ci,
        Forgejo→GitHub-now, data-bridges+Kafka→Pulsar(D-EVENT), isolation framekernel-host+assume-breach,
        identity oya-identity+Zitadel-bridge, CC-11 tenant-class, CC-12 gate-waves, CC-13 linux North-Star (CC-10 framing-only)

MIGRATION LANES (WIP §6, onto the cleaned + stable-id canon):
  L1 office · L2 oyago · L3 oyapy · L4 claude-SDK · L5 codex-SDK(MERGE) · L6 k8s(MERGE) · L7 containerd ·
  L8 cloud-data/db-engine (CONDITIONAL — dropped if source absent, WIP G4) · L9 node-os · L10 docs(+13 pilot ADRs into live free block) · L11 framekernel(no_std,LAST)
```

**Sequencing rationale:** A-lanes first so the migration lands onto a foundry-clean, integrity-clean, namespaced canon; A5/A6 NEW+CC ADRs are additive into the live free block (same mechanism as L10's pilot-ADR additions — no collision since no renumber). Each lane is one squash-PR; the serial loop sequences the global Buck2 graph.

---

## 3. The WHAT per A-lane → decision record

Each A-lane applies specific D-decisions (SSOT = decision record). Key: D11 (integrity sweep + foundry-rename batch, RULED before backfill) · D12 (vocab) · D14 (Proposed) · D-EVENT (Pulsar) · D-SAFETY / D-KR / D-SEQ / D15 / D1 / D-INTEL (new ADRs) · D-RECOVER (.Trash + bominal restorations: KR payroll packs, First-Proof-Slice+M3, released-view, drop Train). The corrected **foundry routing (D-INTEL)**: rename to CURRENT homes (oya-intelligence / oya-governance / retired), NOT the future cloud-intelligence re-home.

---

## 4. DEFERRED ratchet campaigns (NOT in this queue — later, gated)

Per D13-amend + D-INTEL + build-first-cutover-later, these are big multi-quarter campaigns AFTER migration settles:
1. **Full ADR-0000+ re-foundation** (renumber/consolidate the whole corpus) — deferred (D13-amend).
2. **AI-engine RE-HOME** `oya/intelligence` (96k LOC) → `cloud/cloud-intelligence` (D-INTEL; the Bedrock-engine relocation).
3. **Governance build-out** spec-shell → live Rust crates (D-INTEL).
4. **AI-substrate maturity program** — 23-dim GA-parity + 4 beat-bars, ADR-0123-gated (D-INTEL).
5. **Owed-depth authoring** per vertical at each vertical's M0 gate (D-DEPTH); vertical-coverage map (#18).
6. **Stale-file >48h audit** (campaign task #14 — after amendments land).

---

## 5. Verification + user gates (merge of both)

- **WIP gates G0–G4** (§11): G0 authority-flip HALT · G1 github-mirror push creds · G2 tools/ standing-exception ratify · G3 signing (DONE) · G4 db-engine/cloud-k8s/codename/no_std-inertness confirmations.
- **Per-lane (A + M):** the WIP §9 verification plan (Cargo+Buck2 dual build, whole-graph buck2 -check matrix, brand-residue scan, multispectrum evidence, 5-H2 PR body, squash-merge) PLUS the founder verify-at-each-step (separate verifier lane vs primary sources; A-lane-specific: foundry 0-residual-outside-carve-outs, KCMVP restored, 0 dangling edges on the stable id-space, Proposed-ledger zero-unaccounted).
- **No A- or M-lane opens until pre-lanes 0.4–0.7 are green + USER sign-off.**

---

## 6. CONFORMANCE GATES — ralplan revision (per D-CONFORM, additive; WIP loop unchanged)

The conformance audit found the WIP per-lane acceptance is build+brand-centric. **Add these ~12 architecture/governance gates to the WIP loop's STEP 7 + the §5 per-lane verification** (every M-lane AND A-lane must pass them; they do NOT change the serial loop / pre-lanes / sequence / authority machinery):

1. **BNF layer-suffix ENUM** — every crate ends in a closed-enum layer suffix (`-kernel/-domain/-usecase/-adapter-<tech>/-app/-check-<discipline>`); reject `-core/-runtime/-port/-api-contracts/-gateway/-web` + snake_case `ctrd_*`/`meta_v1` (ADR-0056/0105).
2. **Hexagonal layer-import-matrix** — kernels import only kernels/ports; adapters→their kernel + one tech; api→kernel-only; app→no-app; ports-in-kernel; LEAN-A2 (ADR-0056 §import-matrix; the biggest reshape — oyago/oyapy/claude/codex monoliths SPLIT).
3. **Microservice slot2 registration** — each migrated service registered in the flat catalog/registry (ADR-0131/0115).
4. **Manifest hygiene** — `resolver="2"`, `version.workspace=true`, `publish=false`, `license="Apache-2.0"`, `[lints] workspace=true`, `[lib] doctest=false`, rust-version pinned to the workspace toolchain (ADR-0092; engineering-conventions §1).
5. **Dependency-rationale no-orphan** — every `[workspace.dependencies]` entry has a justification row; new external dep needs deny.toml clearance + own-vs-reuse rationale (ADR-0003/0092).
6. **Vendor A/B/C registry** — vendored deps classified + registered (not blind-inherited); fix misplaced `deny.toml` (office) — content-correctness, not just presence (ADR-0013 §36-38).
7. **Per-service colocation + buildability-bar ADR-shape** — each service carries PRD/contracts/decisions/catalog/slos/threat-model + builds standalone (ADR-0212/0034).
8. **rebrand-arrow / retired-terms scan** — beyond brand-residue: catch retired vocab (M0-M3, tier-system, "Foundry" live, rebrand arrows) (D11/D12).
9. **`data_class`** on every new kernel-struct field (already in WIP STEP 7 — keep).

**Migration-fit decisions wired in (D-CONFORM):** transpiler-go/transpiler-py names · fresh-attributed-import-all + claude MIT→Apache · non-Rust = fixtures-as-test-data / port-real-tools · nested-workspaces = collapse-2-STD / **exclude-the-12-kernel-subtree** / exclude-vendored (pre-lane 0.6 must prove the full 12-entry kernel-exclude inert).

**Unrouted founder decisions now ROUTED here** (were open in the audit): import-history (D-CONFORM #2), token-budget (#1), relicense (#2), non-Rust homing (#3), nested-workspaces (#4). All five closed.

---

*End UNIFIED EXECUTION PLAN. Authority: WIP `monorepo-consolidation-migration.md` (HOW) + decision record (WHAT) + §6 conformance gates (D-CONFORM). STATUS: pending — gated on G0–G4 + authority-flip. AUDIT ✓ · RALPLAN ✓ (this revision) · CONSOLIDATION = next.*
