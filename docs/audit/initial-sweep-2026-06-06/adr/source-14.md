# ADR Audit — SOURCE chunk 14

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 14
- **Slice command:** `ls -1 .../decisions/ADR-*.md | sort | sed -n "92,98p"`
- **Range:** ADR-0114 … ADR-0120
- **ADRs actually reviewed (7):** 0114, 0115, 0116, 0117, 0118, 0119, 0120
- **Auditor posture:** READ-ONLY. Keystone map (`_map/canonical-posture-and-supersession-map.md`) treated as binding baseline. On-disk verification run for `specs/`, `registry/`, ADR-0363, ADR-0511, ADR-0083.

> **Cluster headline:** This is the *Foundry-pipeline / 2026-05-16 hygiene* batch. Five of seven ADRs (0114, 0116, 0117, 0118, 0119) explicitly stand on the agentic-VCS "Foundry pipeline (M01-P18 / ADR-0110-0113)" substrate that **ADR-0363 has since RETIRED** (Foundry brand dead → cloud-intelligence; forge → Forgejo/plain-git; canary → Argo-Rollouts). So the *mechanical outcomes* (flat `registry/`, flat `specs/`, retired grit/rtk/icm/vox, gitignored `.audit/`) are TRUE and survive, but the *Foundry-pipeline framing/branding/enforcement language* throughout is retired-vocab leakage that needs AMEND. Two ADRs are already self-superseded on disk (0120→0375). One ADR (0114) is a Proposed bespoke build that the hyperscaler-verdict and current CD canon both kill.

---

### ADR-0114 — Canary observability gate + rollback

- **decision_atom:** Promotion across dev→staging→production is gated by a per-product canary controller that emits PROMOTE/ROLLBACK/EXTEND_OBSERVATION/ESCALATE verdicts from latency/error-rate/SLO/KPI signals, graduates exposure in 5%→10%→25%→100% stages, and supports a signed+audited canonical-revert plus emergency-rewind rollback.
- **current_status:** Proposed (front-matter `status: Proposed`).
- **disposition:** ARCHIVE (as written) → the *principle* (progressive-delivery canary gate before prod promotion) survives, but the *implementation* (`oya-foundry-canary-controller-{kernel,app}`, cell-cohort registry, gh-api branch-protection bypass) is superseded by the current CD canon.
- **governing:** ADR-0363 (retires the Foundry agentic-VCS pipeline incl. ADR-0110 changeset-event-log on which the verdict-emission depends) + ADR-0511/§1.3 CI-CD chain (Argo Workflows + **Argo-Rollouts** is the canonical progressive-delivery/canary mechanism; ADR-0254-kubernetes-everywhere). Brand retired by ADR-0335/0347 (`oya-foundry-*`→`oya-governance-*`/intelligence).
- **truth_flag:** PARTIAL — the design intent is TRUE/sound; the artifacts and the `oya-foundry-canary-controller` brand are STALE/retired; the `gh api branches/<ref>/protection allow_force_pushes` mechanism is WRONG against the GitHub-vs-Forgejo forge canon (assumes GitHub branch-protection API on the very forge the canon is moving off).
- **in_masterplan:** NO — no `planning_impact`/`masterplan_ref` front-matter; never left Proposed; bespoke controller not reflected in canonical posture (CD = Buck2/Argo).
- **tensions:**
  - vs ADR-0363 — builds verdict emission on `changeset-event-log` (ADR-0110), a substrate ADR-0363 retired.
  - vs ADR-0511 + Argo-Rollouts — duplicates progressive-delivery that the k8s-native CD stack provides natively; bespoke ~600 LOC controller competes with adopted OSS.
  - vs forge canon (§5) — emergency-rewind uses GitHub-specific `gh api ... protection` and force-push, conflicting with Forgejo-canonical / bespoke-VCS direction.
  - retired-vocab — `oya-foundry-*` crate/lane names throughout (ADR-0347 → `oya-governance-*`); "cell as canary unit" leans on cell-architecture (ADR-0033) but ADR-0333 retired cell-as-microservice (cell survives only as a deployment *pattern* — partially compatible but must be re-grounded).
- **hyperscaler_challenge:** MISALIGNED. Google/AWS/Azure would NOT hand-roll a canary controller; they use Argo-Rollouts/Flagger/CodeDeploy/Spinnaker-class progressive delivery with the service mesh doing traffic-shaping. Hard-coded ratio thresholds (1.20×/1.50×) and a 30s-poll fail-closed gate is a reinvented wheel. Argues for ARCHIVE (keep the gate *requirement*, drop the bespoke build).
- **ai_slop:** Fabricated precision — exact thresholds (1.20/1.50/1.30 ratios, 900/1800/3600s windows, "~600 LOC + 4 lanes", "<15 min MTTD") are invented numbers presented as decided fact with no evidence basis. Open-questions-as-decisions pattern ("Decision: both/YES/...") inflates a Proposed doc into apparent settledness.
- **refinement:** Reframe as a one-paragraph decision: "canary-gated promotion is REQUIRED; mechanism = Argo-Rollouts analysis templates over the Argo Workflows CD pipeline; thresholds live in `registry/canary/thresholds.yaml` tuned per product." Drop the bespoke kernel/app, the force-push bypass, and all `oya-foundry-` naming.
- **consensus_needed:** yes — "Is progressive-delivery canary owned by adopted Argo-Rollouts (align with CD canon) or by a bespoke `oya` controller (own-the-substrate)? If we keep the gate as a *requirement*, ARCHIVE ADR-0114's implementation and re-issue against Argo-Rollouts."

---

### ADR-0115 — Registry consolidation: flat singular `registry/`

- **decision_atom:** `registry/` (flat, singular) is the single canonical machine-readable registry root where every direct child is a semantic class; the parallel `registries/cross-cutting/` (plural+nested) root is retired and its files `git mv`'d to `registry/`.
- **current_status:** Accepted.
- **disposition:** KEEP — verified on disk (`/registry/` exists, no `registries/` plural). Clean, well-formed topology decision; matches the BNF `kind` token in `oyatie-doctrine.json` P14.
- **governing:** n/a (governing, not governed). Sibling/precedent for ADR-0117 and ADR-0119 (which cite it as the flat-root pattern).
- **truth_flag:** TRUE — outcome confirmed on disk; reference-rewrite + grep-zero verification recorded.
- **in_masterplan:** PARTIAL — no `planning_impact`/`masterplan_ref` front-matter, but the outcome (`registry/` canonical root) is structurally load-bearing and referenced by `specs/root-hub-pointers.json`. Should carry a masterplan binding so the canonical-root invariant is enforced.
- **tensions:**
  - Embeds the retired sanctioned-primitives note (grit/rtk/icm/vox "deprecated") — forward-pointer to ADR-0116, fine as history.
  - "Bominal-inheritance ledger" / `registry/bominal-inheritance-overrides.json` (§Follow-up) is an obscure cross-repo concept not surfaced in the keystone map — possible orphan reference; low severity.
- **hyperscaler_challenge:** ALIGNED. A single flat, semantically-classed registry root with history-preserving moves and a grep-zero acceptance gate is exactly how a hyperscaler monorepo would consolidate config roots. No amend pressure.
- **ai_slop:** Mild — the 5-row "Rejected alternatives" table over-justifies a singular-vs-plural naming choice (English-plural-noise reasoning is filler). Not load-bearing.
- **refinement:** Add `planning_impact: true` + a `masterplan_ref` so the canonical-root invariant is gate-enforced (per planning-ssot-drift-prevention 8.8%-binding finding). Resolve/cite the `bominal-inheritance-overrides` artifact or drop the follow-up.
- **consensus_needed:** no.

---

### ADR-0116 — Retire external agent-coordination tooling (grit, rtk, icm, vox)

- **decision_atom:** The external out-of-repo coordination tools grit/rtk/icm/vox are retired from the prescribed agent surface; in-repo per-agent `git worktree` → PR-off-`dev` → pipeline is the sole canonical concurrent-work workflow (supersedes ADR-0054).
- **current_status:** accepted.
- **disposition:** AMEND — the *retirement* of grit/rtk/icm/vox is TRUE and canonical (keystone §2 confirms; supersedes ADR-0054). BUT the prescribed replacement is named "**Foundry pipeline (M01-P18)**", which ADR-0363 has since retired (agentic-VCS Foundry → plain git + Forgejo PRs + Prow-shaped cloud-ci; Foundry→Intelligence). Decision survives; framing/branding is stale.
- **governing:** Retirement is governing over ADR-0054 (keystone §1.2). The *replacement substrate naming* is governed-downstream by ADR-0363 (and ADR-0335/0347 brand retirement, ADR-0116→ "intelligence/oya-ci").
- **truth_flag:** PARTIAL — TRUE that the four tools are retired; STALE that "Foundry pipeline" is the live replacement name and that GitHub `gh pr create` + GitHub webhooks are the path (forge canon now Forgejo/bespoke).
- **in_masterplan:** PARTIAL — carries `planning_impact: true` (good; one of the few in this chunk). But the impact it encodes (Foundry-pipeline-as-canonical) is the retired framing, so the masterplan projection would inherit stale vocab unless reconciled.
- **tensions:**
  - vs ADR-0363 — names the now-retired Foundry agentic-VCS pipeline (incl. ADR-0110/0111/0112/0113) as canonical; the substrate it points at was dissolved.
  - vs forge canon (§5) — "PR off `dev`" + "GitHub webhook on the repo" assumes GitHub-native flow; founder directive is GitHub but source canon is Forgejo — surface, do not resolve.
  - retired-vocab — `oya-foundry-vcs-admission-gate` (enforced_by) → `oya-governance-*` per ADR-0347; "M01-P18", "wave-A/B" milestone vocab retired per GLOSSARY (M0-M3 → Wave names).
- **hyperscaler_challenge:** ALIGNED (on substance). Retiring bespoke out-of-repo lock tooling in favour of worktree-isolation + PR/merge-queue is exactly the hyperscaler monorepo model (one contribution path, no side-channel locks). The *brand* (Foundry) is the only misaligned part. Argues for AMEND, not ARCHIVE.
- **ai_slop:** Hedging/seam-narration — the long "until wave-B lands the seam is..." passage is operational hand-waving that dates the doc; "the `Bash(grit *)` permission ... left in place under deny-by-omission" is filler precision.
- **refinement:** Re-issue the decision_atom brand-neutral: "external coordination tools retired; canonical concurrent-work path = per-agent worktree → PR → merge-queue on the in-repo CI substrate (now intelligence/oya-ci per ADR-0363/0513)." Replace `oya-foundry-vcs-admission-gate` with `oya-governance-*`. Drop M01-P18/wave milestone vocab.
- **consensus_needed:** no on the retirement (settled); the forge-flow tension is already escalated under ADR-0116-via-0363/0510 (§5) — not re-opened here.

---

### ADR-0117 — Repo hygiene: gitignore `.audit/`, consolidate kyverno admission

- **decision_atom:** Gitignore + untrack the session-scoped `.audit/` artifact (keeping `.config/nextest.toml` tracked for CI `[profile.ci]`), and `git mv` the single-file `deploy/gitops/oya-vcs-admission/` into the established `infra/kyverno/` admission-policy root to remove a parallel admission root.
- **current_status:** Accepted.
- **disposition:** KEEP (terminal hygiene) — sound, low-risk, self-contained, with a documented reversal procedure. One stale governing-ref to fix (cosmetic AMEND candidate).
- **governing:** n/a. Note: its cross-check says "ADR-0052 now Superseded by ADR-0118" — consistent with this chunk. Watch the admission posture against ADR-0379 (Kubewarden = default admission, supersedes ADR-0183's Kyverno-split) — this ADR consolidates *into* `infra/kyverno/`, which is still the right physical root, but the engine-level admission canon is Kubewarden; not a contradiction (path vs engine), flag only.
- **truth_flag:** TRUE — clean hygiene decision; the `oya-vcs-admission` / `oya-foundry-vcs-provider-execution-gate-*` crate refs carry retired Foundry-VCS naming (cosmetic STALE).
- **in_masterplan:** NA — pure repo hygiene; no planning-impact expected. Correctly carries no masterplan binding.
- **tensions:**
  - Touches `oya-foundry-vcs-provider-execution-gate-{kernel,app}` crates — retired Foundry-VCS surface (ADR-0363); the *move* is still valid, the crate names are dead-walking.
  - admission-engine: `infra/kyverno/` root vs ADR-0379 Kubewarden-default — physical-root choice is fine; if Kyverno→Kubewarden migration happens the root name `kyverno/` becomes a misnomer. Low severity.
- **hyperscaler_challenge:** ALIGNED. Gitignoring transient session logs, keeping CI config tracked, and collapsing duplicate GitOps/admission roots is unremarkably correct monorepo hygiene. No amend pressure on substance.
- **ai_slop:** Minor over-engineering — a 3-step split-brain reversal + `data_loss_class: none` ceremony for gitignoring a 52KB file and moving one JSON is disproportionate, but harmless (good-practice excess, not fabrication).
- **refinement:** When the Foundry-VCS crates are renamed/retired per ADR-0363, update the inbound-ref list. If admission migrates to Kubewarden (ADR-0379), rename `infra/kyverno/` → engine-neutral `infra/admission/`.
- **consensus_needed:** no.

---

### ADR-0118 — Retire archive-orphan fitness lane

- **decision_atom:** Retire the one-time grit-era `archive-orphan` fitness lane (delete the `oya-governance-archive-orphan-{kernel,app}` crates, workspace members, catalog entries, and the pre-grit archive payload), keeping a historical lane record; supersedes ADR-0052.
- **current_status:** Accepted; `supersedes: [ADR-0052]`.
- **disposition:** KEEP — correct, self-consistent one-time-lane retirement with a documented reversal and an honest ADR-0108 lifecycle-waiver disclosure. Supersession edge to ADR-0052 matches keystone §1.1.
- **governing:** governing over ADR-0052 (keystone confirms ADR-0118 supersedes ADR-0052). Itself downstream of ADR-0116 (Foundry pipeline now canonical) — same stale-framing caveat as 0116.
- **truth_flag:** TRUE on the retirement; the "M01-P18 is the only forward VCS substrate" claim is STALE (ADR-0363 retired that substrate). The cross-check reference `registries/cross-cutting/fixuptasks.jsonl` (line 78) is **STALE/WRONG path** — ADR-0115 retired `registries/cross-cutting/` → `registry/fixuptasks.jsonl`; on-disk verification shows no `registries/` plural root. Internal cross-side inconsistency (0118 cites a path its sibling 0115 deleted).
- **in_masterplan:** NA — one-time cleanup lane; no planning binding warranted. Correctly carries none.
- **tensions:**
  - Self-inconsistent with ADR-0115 (cites retired `registries/cross-cutting/fixuptasks.jsonl`).
  - vs ADR-0363 — "M01-P18 remains the only forward VCS substrate" names the retired Foundry pipeline.
  - retired-vocab — `oya-governance-archive-orphan-*` already uses the *new* governance prefix (good — post-ADR-0347 naming), so this one is partially ahead of its siblings.
  - ADR-0108 waiver (`F-ADR0108-ONETIME-LANE-CARVEOUT`) is filed but unverified-as-landed; track that the `one_time_lane: true` carve-out actually exists.
- **hyperscaler_challenge:** ALIGNED. Deleting a one-time migration-verification gate after the migration completes (rather than leaving an always-green no-op runner = "false mechanical confidence") is exactly the right call; a hyperscaler would garbage-collect dead fitness lanes too. No amend pressure on substance.
- **ai_slop:** Low. The reversal/waiver detail is thorough rather than slop. The only issue is the stale `registries/cross-cutting/` path (factual drift, not slop).
- **refinement:** Fix the `registries/cross-cutting/fixuptasks.jsonl` ref → `registry/fixuptasks.jsonl`. Re-ground "M01-P18 only forward substrate" → intelligence/oya-ci (ADR-0363/0513). Confirm the ADR-0108 `one_time_lane` carve-out landed.
- **consensus_needed:** no.

---

### ADR-0119 — Specs flat-root topology

- **decision_atom:** `specs/` is the canonical flat root for machine-readable specifications; the former nested cross-cutting spec scope directory is retired and all children are hoisted to `specs/` via history-preserving `git mv`, with only the typed `specs/lifecycle-configs/` family retained as a grouping.
- **current_status:** Accepted.
- **disposition:** KEEP — verified on disk (`specs/masterplan.json`, `specs/root-hub-pointers.json` present at flat root). Faithfully applies the ADR-0115 flat-root pattern; well-formed.
- **governing:** n/a. Sibling-precedent of ADR-0115. Critically, this is the ADR that **physically establishes `specs/masterplan.json` and `specs/root-hub-pointers.json` at the path the keystone §4 masterplan-authority posture depends on** — load-bearing for the masterplan-SSOT question.
- **truth_flag:** TRUE — outcome confirmed on disk.
- **in_masterplan:** PARTIAL — no `planning_impact`/`masterplan_ref` front-matter, yet this ADR LOCATES the canonical-authority artifact (`specs/masterplan.json`). It is arguably the most masterplan-relevant ADR in the chunk and should carry a binding. The 37-row per-file naming-justification table is the *manifest of what lives at the canonical spec root* — useful backfill material.
- **tensions:**
  - Touches the masterplan-authored-vs-generated open question (§4): it cements `specs/masterplan.json` as a flat-root artifact, which the *masterplan-is-authority* reading wants; under the *generated-from-ADRs* reading, `masterplan.json` would be a build output and its path is an implementation detail. Flag under BOTH readings.
  - retired-vocab — Foundry-pipeline framing in Context/Operational sections (ADR-0363), same as siblings.
  - related-ref to ADR-0121 (onprem k8s) which is itself Superseded by ADR-0375 — stale forward link.
- **hyperscaler_challenge:** ALIGNED. A flat, semantically-named spec root consumed by tooling, with a typed-config sub-family retained, is standard monorepo `//specs` layout. No amend pressure on substance.
- **ai_slop:** Notable redundancy — the 37-row table repeats the identical justification string ("Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope") ~30 times. That is copy-paste filler; one rule + a file list would carry the same information. Flag as redundancy, not fabrication.
- **refinement:** Collapse the 37-row identical-justification table into one rule + a bulleted file list. Add `planning_impact: true` + `masterplan_ref` (this ADR locates the canonical authority artifact). Update the stale ADR-0121 related-link to ADR-0375.
- **consensus_needed:** yes (masterplan-adjacent) — "Is `specs/masterplan.json` an *authored* canonical artifact that lives permanently at this flat root (masterplan-as-authority), or a *generated* build-output whose path is an implementation detail (generated-from-ADRs)? ADR-0119 cements its path under the former reading."

---

### ADR-0120 — Rust-first on-prem tooling; every install paired with uninstall

- **decision_atom:** On-prem bring-up collapses to one Rust binary (`oya-onprem`) implementing a `Component` trait (install/uninstall/status) with a ≤3-file shell bootstrap layer, and every install/action MUST have an idempotent, reversible, auditable paired uninstall.
- **current_status:** Superseded (`superseded_by: [ADR-0375]`).
- **disposition:** ARCHIVE — correctly already marked Superseded by ADR-0375 (Talos+CAPI+ArgoCD fleet substrate). Keystone §1.1 confirms the edge.
- **governing:** ADR-0375 (Talos immutable node-OS + CAPI + ArgoCD; replaces the kubeadm/containerd/istio onprem stack that this ADR's shell scripts installed). Sibling ADR-0121 (onprem k8s stack) is also superseded by ADR-0375.
- **truth_flag:** PARTIAL/STALE — the *durable principle* (paired idempotent reversible uninstall; minimize shell, Rust elsewhere; topological reverse-dependency teardown; preserve-user-data flag) is TRUE and worth carrying forward EVEN under Talos. The *concrete substrate* (manually installing kubeadm/istio/containerd/podman via a Rust CLI on a Debian host) is dead — Talos is an immutable appliance OS with no per-component install/uninstall surface.
- **in_masterplan:** NA — superseded; should not be projected as live. The "paired-uninstall" doctrine could be salvaged into the Talos/CAPI lifecycle ADR if it isn't already there.
- **tensions:**
  - vs ADR-0375 — the entire premise (a host you install components onto) is replaced by immutable-node-OS (you don't install onto Talos; you declare desired state). Clean supersession, not a contradiction.
  - vs LINUX ADR-0025 (§5 fault-line) — LINUX wants a *Rust "Talos"* (own the node-OS); source ADR-0120's "Rust-first onprem tooling" was an earlier, weaker form of the same own-the-substrate instinct, now superseded by adopting *actual* Talos. Cross-side: the LINUX pilot is re-litigating the own-vs-adopt node-OS question that source closed in favour of adopt (Talos).
  - retired-vocab — `infra/onprem/foundry/install.sh`, `axis-foundry` owner — Foundry brand retired (ADR-0335).
  - Internal bug: "Adopting one of these would land in **ADR-0120**" (Rejected alternatives, Ansible row) self-references its own number — a copy-paste error; should name a future ADR.
- **hyperscaler_challenge:** MISALIGNED (as written) / the meta-principle is ALIGNED. No hyperscaler hand-installs kubeadm/istio via a bespoke CLI on mutable Debian hosts — they use immutable images + declarative reconciliation (exactly ADR-0375/Talos). BUT "every install has a reversible audited uninstall" is a sound infra-as-code invariant any of them would endorse. Argues the ARCHIVE is correct AND that the paired-uninstall doctrine should be re-homed, not lost.
- **ai_slop:** Internal contradiction/error — the self-referential "would land in ADR-0120" (citing itself as the future home) is a fabricated/broken cross-ref. Phase-A/B/C/D + milestone ids (`M03-P01-IP-001b`) use retired milestone vocab.
- **refinement:** Leave Superseded. Extract the paired-uninstall + reverse-dependency-teardown + preserve-user-data doctrine into (or confirm it exists in) the Talos/CAPI lifecycle ADR-0375 lineage so the genuinely-good invariant isn't lost with the dead substrate. Fix the self-referential ADR-0120 cite.
- **consensus_needed:** no (supersession is settled); optional founder note: "does the paired-reversible-uninstall doctrine survive into the Talos/declarative world, or is it moot under immutable nodes?"

---

## Chunk notes for synthesis

**1. This is the "2026-05-16 Foundry-pipeline + hygiene" batch — one date, one substrate, now retired.** All seven ADRs are dated 2026-05-16 and orbit the agentic-VCS "Foundry pipeline (M01-P18 / ADR-0110-0113)". That substrate was retired wholesale by **ADR-0363** (verified `Accepted`; explicitly names ADR-0110/0111/0112/0113/0116 as superseded). Net effect on this chunk: the *mechanical outcomes are TRUE and survive*, but the *framing, enforcement language, crate names, and milestone vocab are retired-vocab leakage*. The synthesis should treat 0114/0116/0118/0119 as "decision-sound, framing-stale" rather than wrong.

**2. Two clean classes of disposition:**
   - **KEEP (verified-on-disk topology):** ADR-0115 (`registry/` flat) + ADR-0119 (`specs/` flat) + ADR-0117 (hygiene) + ADR-0118 (one-time lane retirement). I confirmed `registry/` and `specs/` flat roots exist on disk with no `registries/` plural drift — these outcomes are real, not aspirational.
   - **ARCHIVE:** ADR-0114 (bespoke canary controller, killed by Argo-Rollouts/ADR-0511 + Foundry retirement) and ADR-0120 (already `Superseded by ADR-0375`).
   - **AMEND:** ADR-0116 (retirement true, "Foundry pipeline" replacement-name stale).

**3. Confirmed supersession edges (match keystone §1.1/§1.2):** ADR-0118 supersedes ADR-0052 ✓; ADR-0116 supersedes ADR-0054 ✓; ADR-0120 superseded_by ADR-0375 ✓. No drift in *these* edges — front-matter is honest here (unlike the ADR-0136 / ADR-0005 stale-front-matter cases the keystone flags elsewhere).

**4. One concrete internal-drift bug to surface:** ADR-0118 (line 78) cites `registries/cross-cutting/fixuptasks.jsonl` — a path its own sibling **ADR-0115 retired the same day** (→ `registry/fixuptasks.jsonl`). On-disk: no `registries/` plural root. This is a same-batch self-inconsistency; cheap fix, but it shows the 2026-05-16 batch wasn't internally cross-checked.

**5. One self-referential cite bug:** ADR-0120's Rejected-alternatives "would land in ADR-0120" cites its own number as a future home — broken cross-ref.

**6. AI-slop pattern across the chunk = fabricated precision + redundancy, not hallucinated architecture.** ADR-0114 invents exact thresholds/LOC/MTTD; ADR-0119 repeats one justification string ~30× in a 37-row table; ADR-0117 over-ceremonies a 52KB gitignore. None *fabricate a false architecture posture* — the slop is dressing, not substance. (Consistent with the keystone's "trust the superseding ADR" guidance; here the ADRs themselves are mostly honest about status.)

**7. Masterplan-binding gap (relevant to founder GOAL):** Only ADR-0116 carries `planning_impact: true`; the structurally load-bearing topology ADRs (0115 `registry/` root, 0119 `specs/` root — the latter *locates `specs/masterplan.json` itself*) carry NO masterplan binding. This is a live instance of planning-ssot-drift-prevention's "8.8% ADR binding" finding. **Backfill candidates for the masterplan:** the canonical-root invariants (`registry/` flat-singular; `specs/` flat root housing `masterplan.json`) are exactly the kind of true+relevant decisions the founder wants captured.

**8. Cross-chunk / cross-side tensions:**
   - **Forge (§5):** ADR-0114's emergency-rewind uses GitHub-specific `gh api .../protection allow_force_pushes` and ADR-0116 assumes GitHub webhooks/`gh pr create` — both lean GitHub-native (aligns with founder directive) but conflict with source's Forgejo-canonical posture. Surface; do not resolve.
   - **Own-vs-adopt node-OS (§5 fault-line):** ADR-0120's "Rust-first onprem tooling" is an early own-the-substrate instinct that source CLOSED by adopting actual Talos (ADR-0375). LINUX ADR-0025 re-opens it (wants a Rust "Talos"). The pilot is re-litigating a question source already decided toward *adopt*.
   - **Own-vs-adopt CD (ADR-0114):** bespoke `oya` canary controller vs adopted Argo-Rollouts — same own-vs-adopt axis as the §5 breadth tension; source's current CD canon (Buck2+Argo) says adopt.

**9. Masterplan open-question touchpoint:** ADR-0119 physically cements `specs/masterplan.json` at the flat root. Under *masterplan-as-authority* that path is canonical-permanent; under *generated-from-ADRs* it's a build-output detail. Flagged under both — this is the only consensus-needed item in the chunk beyond ADR-0114's own-vs-adopt.

**10. Hyperscaler verdict pattern:** the topology/hygiene/retirement decisions (0115/0116-substance/0117/0118) are all ALIGNED (standard monorepo practice). The two *bespoke-build* decisions (0114 canary controller, 0120 onprem CLI) are MISALIGNED on implementation — both reinvent wheels hyperscalers buy off-the-shelf (Argo-Rollouts; immutable-OS/declarative reconciliation). Clean signal: in this chunk, "own it" lost to "adopt it" every time the canon moved forward.
