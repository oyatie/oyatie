---
title: "VERIFY — CI-shape tension: linux D3 (Prow+Tekton+Argo) vs source-confirmed bespoke-Prow-only"
lane: backlog-reconciliation / 20-verify-ci
date: 2026-06-06
mode: READ-ONLY verification (no files edited)
stakes: HIGHEST (CI/CD canonical shape; founder door:one-way)
verdict: CONTRADICTION CONFIRMED — linux D3 "Prow+Tekton+Argo" overstates the shape vs source-confirmed bespoke-Prow-only
---

# 20-verify-ci — CI-shape tension verification

All claims below are read directly from the on-disk ADRs in
`/Users/jasonlee/Developer/source/docs/decisions/` and the source backlog
`/Users/jasonlee/Developer/source/.omx/backlog/platform-readiness-backlog.md`
+ `/Users/jasonlee/Developer/source/.omx/HANDOFF-platform-readiness-2026-06-01.md`,
and the linux decision-record
`/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`.

## 1. TRUE current state of each ADR (verbatim frontmatter)

| ADR | file | status (on disk) | supersedes | superseded_by | Tekton? | Argo? |
|---|---|---|---|---|---|---|
| **0513** | ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md | **Accepted** | *(no key)* | *(no key)* | NO | NO |
| **0514** | ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md | **Proposed** | `[]` | `[]` | NO (refs Prow plank) | NO |
| **0511** | ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md | **Proposed** | `[ADR-0359]` | **`[]` (EMPTY — NOT superseded)** | rejects Tekton | endorses Argo Workflows |
| **0349** | ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md | **Proposed** | `[]` | `[]` | names+rejects Tekton (§F) | ArgoCD (CD) |
| **0359** | ADR-0359-jenkins-completely-replaces-github-actions.md | **Superseded** | `[]` | **`[ADR-0511]`** | NO | NO |
| **0361** | ADR-0361-jenkins-native-cicd-revamp-execution.md | **Proposed** | `[]` | *(no key)* | NO | NO |
| **0408** | ADR-0408-buck2-driven-ci-cd.md | **Proposed** | `[ADR-0358]` | `[]` | NO | NO |

### Key findings on status

1. **ADR-0511 is NOT marked superseded.** Its frontmatter is literally `status: Proposed`
   with `superseded_by: []`. The lane brief's premise "ADR-0511 superseded_by ADR-0513"
   is **NOT yet true on disk** — it is a PENDING fix, not an applied state.
   - `ADR-0511 …:11  supersedes: [ADR-0359]`
   - `ADR-0511 …:12  superseded_by: []`
2. **ADR-0513 has NO `supersedes`/`superseded_by`/`amends` keys at all** (grep confirmed:
   ">> 0513 has NO supersedes/superseded_by/amends keys at all <<"). So the intended
   0511→0513 supersession link is **missing on BOTH ends** — there is no clean
   supersession in metadata. This is exactly the drift the backlog flags (see §3).
3. **Only ADR-0513 is Accepted.** 0349/0361/0408/0514 are all `Proposed`; 0359 is `Superseded`.
   The single Accepted CI/CD ADR is the bespoke-Prow one.

## 2. Is Argo Workflows rejected? Does any accepted ADR endorse Tekton?

**Argo Workflows:**
- The only ADR *endorsing* Argo Workflows as the CI orchestrator is **ADR-0511, which is `Proposed` (never Accepted)**. Its own basis line says the destination stack = "Buck2 … + Argo Workflows (k8s-native CI orchestration, REPLACES transitory Jenkins) + ArgoCD/Argo Rollouts (CD)" (ADR-0511 line 19).
- The Accepted ADR-0513 makes **no mention of Argo Workflows** (grep: "NO 0511 / no Argo-Workflows reference in 0513"). It names a bespoke-Rust Prow shape instead.
- Source backlog explicitly overrides 0511: *"NOT Argo Workflows (ADR-0511 overtaken)."* (backlog line 202) and *"Argo (Workflows) is NOT the CI orchestrator; oya-ci bespoke-Rust Prow … is."* (backlog line 237).
- **ArgoCD / Argo-Rollouts (CD only) may remain** — backlog line 237: *"(ArgoCD/Argo-Rollouts for CD/GitOps may remain — distinct from CI orchestration; confirm in reconcile.)"* This matches the brief. CD-Argo is allowed; CI-Argo-Workflows is rejected.

**Tekton — NO accepted ADR endorses it. It is mentioned in exactly 3 ADRs, all `Proposed`, and in every case it is rejected or third-party context:**
- **ADR-0511 (Proposed)** line 86 — explicitly REJECTED: *"**Tekton instead of Argo Workflows** — rejected for now: … a second pipeline ecosystem adds surface for no benefit."*
- **ADR-0349 (Proposed)** §F line 165 — named in the forbidden-alternative competitor list ("Jenkins X, Tekton, Flux CD, Spinnaker …"), i.e. rejected; line 219 mentions Tekton only as RedHat OpenShift hyperscaler-precedent prose.
- **ADR-0345 (Proposed)** line 170 — Tekton named only as a Google upstream-OSS-contribution example (stewardship policy), not a Oyatie adoption.
- **ADR-0513 / 0514 (the oya-ci ADRs): zero Tekton mentions.** The bespoke shape is pure Prow-component-decomposition (hook/plank/crier/tide/deck/sinker/plugins) reimplemented in Rust.

**Conclusion:** Argo Workflows = rejected (only endorser is a Proposed, overtaken ADR). Tekton = never endorsed by any ADR, rejected where named. The source-confirmed destination is **bespoke-Rust Prow component-shape ONLY** (ADR-0513, the sole Accepted CI ADR).

## 3. Does the linux D3 "Prow+Tekton+Argo" CONTRADICT the backlog?

**YES — contradiction confirmed.** The linux decision-record D3 (decision-record-oyatie-canon.md):

- **Line 157**: *"**Ruling: RATIFY the unified `Run`+graph reshape of ADR-0513.** oya-ci = ONE Rust-native CI/CD system = two nouns (`Run` + Buck2 build-graph over a CAS core) · **four faces (Prow trustless gate+merge-queue · Tekton typed-task/provenance spine · Argo DAG engine · provenance)** · the build-graph 'brain' as the differentiator. Supersedes the 'clone Prow's 8 components' framing …"*
- **Line 158**: *"resolves 0511 (keep Argo's DAG/event IDEAS, drop its etcd-CRD substrate; reject both 'adopt-Argo-wholesale' and 'clone-all-Prow')"*

The D3 "four faces" framing names **Tekton** and **Argo** as design pillars/faces of oya-ci. This is the source of the brief's "Prow+Tekton+Argo" phrasing (also encoded in linux task #17 title: "Design unified oya-ci (Prow+Tekton+Argo)").

This contradicts the source-confirmed shape:
- Source register #16 (backlog line 585): *"Consolidate into ONE canonical CI ADR (**oya-ci Prow destination = ADR-0513**; mark the rest superseded/amended-into-it)."* — **Prow destination, singular.** No Tekton; no Argo Workflows.
- Source §G shape constraint (backlog line 202): *"new CI work fits the oya-ci Prow shape (kube-rs controller, trunk-sourced buck2 affected-gate, Forgejo-native hook/plank/crier/tide/deck/plugins) … NOT Argo Workflows (ADR-0511 overtaken)."*
- The Accepted ADR-0513 body builds **Prow's decomposition in Rust** with NO Tekton/Argo-Workflows layer.

### Nuance — is it a hard contradiction or an over-labeled framing?

The contradiction is real but is best characterized as **the linux D3 over-labels the design with adopted-product NAMES that the source-confirmed canon rejects as products:**
- "Argo DAG engine" as a *face* of oya-ci ⇒ source says reject Argo-as-product, **keep only the DAG/event IDEAS** (D3's own line 158 says "keep Argo's DAG/event IDEAS, drop its etcd-CRD substrate"). So even linux D3 internally agrees Argo-the-product is dropped — but the headline "four faces (… Argo DAG engine …)" still reads as endorsing Argo as a component, which the backlog forbids ("NOT Argo Workflows").
- "Tekton typed-task/provenance spine" as a *face* ⇒ source has **no ADR endorsing Tekton at all**; Tekton is rejected wherever named. There is no source basis for a Tekton face. This is the **sharpest contradiction**: D3 introduces a Tekton pillar with zero ADR support, against an Accepted bespoke-Prow ADR and a register-#16 consolidation that names only Prow.

Net: the *substance* both sides want is a bespoke-Rust system that internally provides Prow-style gating/merge-queue + DAG orchestration + typed-task/provenance — but **the labeling differs and matters**, because the source canon is explicit that these are *bespoke capabilities inspired by ideas*, NOT adoptions of Tekton or Argo Workflows as products. D3's "Prow+Tekton+Argo four faces" reads as a multi-product adoption; the backlog mandates bespoke-Prow-shape-only with Argo/Tekton as rejected products (CD-Argo excepted).

## 4. Corroborating source evidence (confirms backlog premise)

- **ADR-0359 → 0511 link IS present** on disk: 0359 `status: Superseded`, `superseded_by: [ADR-0511]`; 0511 `supersedes: [ADR-0359]`. So the 0359/0511 pair is internally consistent — but 0511 itself was then overtaken by 0513 without a metadata link.
- **Backlog line 201** (consistency conflict, verbatim): *"ADR-0511 (Proposed 2026-05-29) names ARGO WORKFLOWS as destination orchestrator … ADR-0513 (Accepted+founder-locked 2026-05-30) names bespoke-Rust oya-ci Prow controller. CONFLICT on the orchestrator, both live, no clean supersession in metadata. … RESOLVE: mark ADR-0511 superseded_by ADR-0513 …"*
- **Backlog line 585 (register #16, verbatim):** *"CI-ADR-SPRAWL CONSOLIDATION … ADR-0349/0359/0361/0408/0511/0513/0514 are 7+ overlapping CI/CD ADRs = single-canonical-source violation … Consolidate into ONE canonical CI ADR (oya-ci Prow destination = ADR-0513; mark the rest superseded/amended-into-it)."* — confirms the exact 7-ADR consolidation set in the brief.
- **HANDOFF line 101-102 (verbatim):** *"ADR-hygiene (renumber dup ADR-0377; mark **ADR-0511 superseded_by ADR-0513**)"* — the supersession is an OUTSTANDING hygiene fix, not applied.
- **HANDOFF line 21-22:** SCM/CI/CD = bespoke Rust services *"with Forgejo/Jenkins/**Argo bridge adapters only**"* — Argo is bridge-adapter, not destination orchestrator.
- **Backlog line 595:** documents the operating-contract drift — `docs/AGENTS.md` still says (ADR-0349/0359/0361 Jenkins-sole) and *"register #16 CI-ADR consolidation MUST update the operating contract."*

## 5. VERDICT + exact reconciliation needed

**Verdict:**
1. ADR-0511 is **NOT** marked superseded on disk (`status: Proposed`, `superseded_by: []`); the brief's stated "superseded_by ADR-0513" is a pending, unapplied fix.
2. **No Accepted ADR endorses Tekton or Argo Workflows as a CI product.** Tekton is rejected wherever named (all in Proposed ADRs); Argo Workflows is endorsed only by the Proposed-and-overtaken ADR-0511. ArgoCD/Argo-Rollouts for CD may remain.
3. The single Accepted CI ADR (0513) is **bespoke-Rust Prow component-shape ONLY**, with no Tekton and no Argo Workflows.
4. **The linux D3 "Prow+Tekton+Argo (four faces)" framing DOES contradict the source-confirmed bespoke-Prow-only canon** — most sharply on Tekton (no ADR support at all), and on labeling Argo as a "face/engine" when the backlog says "NOT Argo Workflows". D3's own body partly self-corrects ("keep Argo's DAG/event IDEAS, drop its substrate"), so the conflict is one of headline framing + Tekton-introduction rather than deep architectural intent — but it must be reconciled because the door is one-way and the headline shape propagates into ADRs/specs.

**Exact reconciliation needed:**
- **R1 — Re-label D3 to bespoke-Prow-only.** Rewrite the D3 headline from "four faces (Prow · Tekton · Argo DAG engine · provenance)" to: oya-ci = bespoke-Rust, Forgejo-native, K8s-native (kube-rs) reimplementation of **Prow's full component shape** (hook/plank/crier/tide/deck/sinker/plugins) over a Buck2 build-graph + CAS core, where DAG-orchestration and typed-task/provenance are **bespoke capabilities inspired by Argo/Tekton ideas — NOT adoptions of those products.** Drop "Tekton spine" and "Argo DAG engine" as named faces; if the typed-task/provenance and DAG capabilities are wanted, state them as oya-ci-native features.
- **R2 — Drop Tekton entirely** from the canonical CI shape (no ADR supports it; rejected where named). Retain only "provenance/typed-task as a bespoke oya-ci concern" if desired, without the Tekton label.
- **R3 — Confine Argo to CD.** Argo Workflows = rejected for CI. ArgoCD/Argo-Rollouts may remain for CD/GitOps (confirm in the CD ruling, distinct from CI orchestration). Reflect HANDOFF's "Argo bridge adapters only" for the transitory period.
- **R4 — Apply register #16 consolidation:** seed ONE canonical CI ADR = oya-ci Prow destination (ratify/reshape ADR-0513); mark the rest of {0349, 0359 (already), 0361, 0408, 0511, 0514} superseded/amended-into-0513. Note 0408/0514 are AMEND-in-place adopted-substrate ADRs per decision-record line 78, not archived.
- **R5 — Fix the missing supersession metadata (both ends):** set ADR-0511 `status: Superseded`, `superseded_by: [ADR-0513]`; add the reciprocal `supersedes:`/relate entry to ADR-0513 (which currently has NO supersession keys). This is the HANDOFF line-101 + backlog line-201 + register-#16 outstanding fix.
- **R6 — Update operating contract:** `docs/AGENTS.md` still reflects Jenkins-sole (ADR-0349/0359/0361); per backlog line 595 the register-#16 consolidation must update it to oya-ci Prow destination. Jenkins/GH-Actions scaffold stays an explicitly-UNRATIFIED de-facto bridge under build-first-cutover-later (decision-record line 78), retired when oya-ci is built+proven.

**Cross-check note:** decision-record line 78 (founder, 2026-06-06) ALREADY rules *"0511 Argo (conflicts with the D3 oya-ci ruling; its ideas already absorbed); only oya-ci 0513 is ratified canon"* and line 158 says reject adopt-Argo-wholesale. So the linux side's *founder rulings* already converge on bespoke-Prow-only — the contradiction is between those rulings and the **D3 headline labeling (line 157) + task-#17 title**, which still carry "Prow+Tekton+Argo." Reconciliation R1-R2 is therefore an internal-consistency fix on the linux D3 wording to match its own founder ruling AND the source canon; R3-R6 are the cross-repo applications.
