# ADR Audit — SOURCE, Chunk 6

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 6
- **Slice command range:** `ls … | sed -n "36,42p"`
- **ADR range:** ADR-0038 … ADR-0044 (the "platform-discipline / ops-substrate" pack of 2026-05-09)
- **ADRs actually reviewed (7):** 0038, 0039, 0040, 0041, 0042, 0043, 0044
- **Cross-checks run:** ADR-0379 (Kubewarden supersedes ADR-0183 Kyverno-default; Kyverno→adapter), ADR-0383 (supersedes ADR-0042, confirmed on disk), ADR-0511 (Argo Workflows CI destination), GLOSSARY L88/L191/L396/L698 (KCMVP canonical term), keystone supersession + retired-vocab map.

> **Pack-level note:** all seven are dated 2026-05-09, all still `status: proposed` except 0042 (`superseded`). They are the same authoring batch as chunk 5's API-stability/cohesion ADRs and share three systemic defects: (1) retired-brand leakage (`foundry` owner + `oya-foundry-*` strings — RETIRED per ADR-0335/0347), (2) GitHub-as-forge assumptions baked into CI/branch-protection that collide with the Forgejo/Argo-Workflows canon, (3) `oya-governance-*` lane names already used (good — that part is current), but pinned to `.github/workflows/*.yml` runners (stale). None of these are wrong *decisions*; they are stale *bindings*. The decision atoms are mostly TRUE and masterplan-ready.

---

### ADR-0038 — Trust framework (cross-microservice lineage, DSR cascade, Cosign proof-of-erasure, tenant trust portal)

- **decision_atom:** A single cross-microservice data-lineage graph drives a DSR cascade that erases/corrects/exports a data subject across every store and emits a Cosign-signed, audit-chained proof-of-erasure *per affected store*, surfaced to tenants through one trust portal.
- **current_status:** Proposed (`status: proposed`, `superseded_by: -`).
- **disposition:** KEEP (with AMEND for vocab/typo). The decision is sound, regulator-grounded (PIPA/GDPR/CCPA), and non-conflicting. Amend only: the doubled-word "across all all microservices" (title + Decision §) and the retired `Foundry` axis label.
- **governing:** n/a (not superseded). Depends on ADR-0003 (audit chain), ADR-0008 (DUBO), ADR-0039 (signing chain).
- **truth_flag:** TRUE (decision); PARTIAL on surface wording (the "all all" doubling and `Foundry` axis are stale text, not wrong logic).
- **in_masterplan:** PARTIAL/NO — no `planning_impact`/`masterplan_ref` front-matter (only `id/status/doc_status`); this is exactly the 8.8%-binding gap the drift-prevention doc flags. The DSR-cascade + proof-of-erasure decision is strong backfill material but is not yet bound.
- **tensions:**
  - `Foundry` listed as a first-class DSR axis (SaaS/Workspace/Vertical/**Foundry**/Cloud/Search/Ads/Analytics) and `FoundryAgentMemory` StoreRef — collides with ADR-0335 (foundry retired → absorbed by intelligence). Should read **Intelligence**.
  - References ADR-0042 (observability) for per-axis SLA mirror — ADR-0042 is now superseded by ADR-0383; the cross-ref is to a dead ADR.
  - Trust-portal "API stability mirror (per ADR-0037)" + "plugin trust tier matrix (per ADR-0036)" are intra-pack and fine.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all ship cross-service DSR/erasure tooling (AWS Macie + "Right to be forgotten" runbooks; Google Cloud DLP; Azure Purview/PrivacyManager). A *signed per-store proof-of-erasure* with a transparency-log index is more rigorous than any hyperscaler default — a genuine differentiator, not over-engineering. Argues for KEEP.
- **ai_slop:** "across all all microservices" (doubled word, appears twice — fabricated-emphasis slop); mild redundancy in the "trust moat / proof is the differentiator / cohesion-thesis customer-facing artifact" alternatives prose. Otherwise concrete.
- **refinement:** Fix "all all"→"all"; rename `Foundry`→`Intelligence` axis + `FoundryAgentMemory`→`IntelligenceAgentMemory`; repoint ADR-0042 refs to ADR-0383; add `planning_impact: true` + `masterplan_ref` front-matter; collapse the SLA-tier table (Preview 30d/Stable 14d/GA 7d) into the canonical tenant-class vocabulary check (these are *maturity* tiers, not the retired tenant tier-system — acceptable, but label to avoid confusion with ADR-0329).
- **consensus_needed:** no — decision is uncontested; only mechanical reconciliation needed.

---

### ADR-0039 — Supply-chain security (Trivy 4-layer, Cosign keyless, SBOM dual-format, signed commits/tags, Kyverno admission)

- **decision_atom:** Every production artifact carries verifiable provenance — Trivy 4-layer scan + Cosign keyless (Fulcio/Rekor) signing + dual-format SBOM (SPDX 2.3 + CycloneDX 1.5) + signed commits/tags + cluster admission that refuses unsigned images, gated by the `oya-governance-supply-chain` CI lane, targeting SLSA Build L3.
- **current_status:** Proposed.
- **disposition:** AMEND. Core supply-chain decision is current and excellent; two bindings drifted: (1) **Kyverno is no longer the *default* admission engine** — ADR-0379 (Accepted, supersedes ADR-0183) makes **Kubewarden** default with Kyverno retained as a first-class adapter; (2) the canonical CI lane is pinned to `.github/workflows/*.yml` and "GitHub branch-protection ruleset," which collides with the Forgejo/Argo-Workflows canon.
- **governing:** Admission posture now governed by **ADR-0379** (Kubewarden default; Kyverno = adapter). CI substrate governed by **ADR-0511** (Argo Workflows) + **ADR-0513/0514** (oya-ci). Forge governed by ADR-0363/0510 (Forgejo).
- **truth_flag:** PARTIAL — the discipline is TRUE; the Kyverno-as-default and GitHub-Actions-as-host claims are STALE.
- **in_masterplan:** NO front-matter binding. Owner is `foundry` (retired → should be `governance`/cloud-intelligence per ADR-0347). High-value backfill once rebound.
- **tensions:**
  - Kyverno-default vs **ADR-0379 Kubewarden-default** (direct, resolvable — demote Kyverno to "or Kubewarden" / make Kubewarden primary).
  - `.github/workflows/oya-governance-supply-chain.yml` + "GitHub branch-protection ruleset" vs Forgejo-canonical forge (ADR-0363/0510) and Argo-Workflows CI (ADR-0511). Also collides with the founder's GitHub directive — note the three-way forge fault-line (map §5): the ADR's GitHub assumption *accidentally* aligns with the founder but *conflicts* with source canon.
  - Owner `foundry` (RETIRED brand, ADR-0335/0347).
  - "License-policy violations = fail; reject SSPL/AGPL outside legal isolation" is consistent with ADR-0013/0211/0345 (good).
- **hyperscaler_challenge:** ALIGNED, leaning best-in-class. AWS (Signer + ECR scanning + SLSA via CodeBuild), Google (Binary Authorization + SLSA-on-GKE, Sigstore origin), Azure (Defender + Notary/Ratify) all do exactly this; keyless Sigstore + Rekor is the modern posture Google itself champions. The decision is *more* aligned with hyperscaler practice than most of the corpus. Argues for AMEND (rebind tooling), not archive.
- **ai_slop:** Low. The SolarWinds/Codecov/log4shell/xz preamble is justified context, not filler. Some boilerplate in alternatives.
- **refinement:** Swap admission section to "Kubewarden default (ADR-0379), Kyverno adapter"; abstract the CI lane away from `.github/` to the governance-gate engine + Argo Workflows runner; rename owner `foundry`→`governance`; add planning front-matter; keep Cosign-keyless/SBOM-dual/SLSA-L3 verbatim (all current).
- **consensus_needed:** no on the discipline; **yes** only insofar as it touches the forge question — but that consensus belongs to the forge cluster, not this ADR.

---

### ADR-0040 — Progressive delivery (Argo Rollouts canary, blue-green for stateful, metric-gated rollback ≥14.4× burn-rate)

- **decision_atom:** Argo Rollouts is the canonical progressive-delivery controller — canary 5/25/50/100% with Google-SRE burn-rate analysis gates (auto-rollback at 1h burn-rate ≥14.4×), blue-green for stateful surfaces, per-region phased rollout, per-cell as the rollback unit.
- **current_status:** Proposed.
- **disposition:** KEEP. This is the cleanest ADR in the chunk. Argo Rollouts/ArgoCD survives as canon (map §3 CI/CD: "ArgoCD/Argo-Rollouts (CD)"); burn-rate math is battle-tested and copied straight from the SRE Workbook. Minor AMEND only for `foundry` owner/axis-label and the dead ADR-0042 ref.
- **governing:** n/a — not superseded; consistent with ADR-0511/0514 CD posture.
- **truth_flag:** TRUE.
- **in_masterplan:** NO front-matter binding; owner `foundry` (retired). Strong backfill candidate — the burn-rate thresholds are reusable masterplan invariants.
- **tensions:**
  - References ADR-0042 (observability metric store) for the analysis-template query source — ADR-0042 is superseded by ADR-0383; repoint to ADR-0383/ADR-0186 (the LGTM/Mimir metric backplane).
  - `Foundry` row in the per-axis cadence table + owner `foundry` (retired → Intelligence/governance).
  - Mentions ADR-0045 (data-plane primaries) for blue-green stateful surfaces — outside my slice; cross-ref appears valid.
- **hyperscaler_challenge:** ALIGNED. Google (Cloud Deploy + canary), AWS (CodeDeploy canary/blue-green, App Mesh), Azure (Deployment Slots, Flagger) all do staged-canary + metric-gated rollback. The 14.4× multi-window burn-rate is literally Google's own SRE doctrine. Fully defensible. Argues KEEP.
- **ai_slop:** Low. "a rollout is just a controlled experiment in degrading reliability" is a touch rhetorical but accurate. Alternatives are crisp.
- **refinement:** Repoint ADR-0042→ADR-0383/0186; rename owner+axis from `foundry`; add planning front-matter; optionally lift the burn-rate table into a `/specs/*.json` invariant since it is reused by ADR-0042/0383 SLO catalogs.
- **consensus_needed:** no.

---

### ADR-0041 — GitOps (trunk-based dev, release branch cut at tag, merge queue with one-PR-at-a-time root-Cargo-touch)

- **decision_atom:** Trunk-based development on `main` with short-lived feature branches, squash/rebase-only linear history, release branches cut at tag, branch-protection-as-code, and a merge queue that serializes any PR touching workspace-root manifests to make the historical root-`Cargo.toml` race impossible by construction.
- **current_status:** Proposed.
- **disposition:** AMEND. The *process* decisions (trunk-based, linear history, release-at-tag, root-manifest serialization) are TRUE and durable. But the ADR is hard-wired to **GitHub** (`.github/branch-protection.yaml`, GitHub merge-queue schema, `.github/CODEOWNERS`) — this collides head-on with the Forgejo-canonical forge (ADR-0363/0510) and is the sharpest forge-binding ADR in my chunk. Also carries retired vocab.
- **governing:** Forge substrate governed by **ADR-0363/0510** (Forgejo canonical/transitory; bespoke-VCS destination). The merge-queue/branch-protection *semantics* survive; the *GitHub-specific encoding* does not.
- **truth_flag:** PARTIAL — process TRUE; GitHub-platform binding STALE; example `worker-3/CUG-42-trust-portal-skeleton` uses RETIRED `CUG` vocab (ADR/GLOSSARY: CUG→Team, retired 2026-05-09); CODEOWNERS path `crates/oya-foundry-* @foundry` uses RETIRED prefix (→`oya-governance-*` per ADR-0347).
- **in_masterplan:** NO binding; owner `foundry` (retired). The self-reference "adopts the gitops posture from the legacy `ADR-0041-gitops-devops-best-practices.md`" implies a *prior* ADR-0041 — a same-number lineage worth flagging against the map's collision concerns (this is an intra-decisions historical-lineage note, not a cross-dir collision, but it muddies the ADR-0041 identity).
- **tensions:**
  - **GitHub branch-protection/merge-queue/CODEOWNERS vs Forgejo canon (ADR-0363/0510).** Forgejo's merge-queue + branch-protection model differs; the YAML here is non-portable. THREE-way forge fault-line (map §5).
  - Required status checks list (`oya-governance-cohesion/supply-chain/api-semver/ads-gate-singleton/vertical-override-pack/dcim-substrate/workflow-cohesion/cloud-surface/license-policy`) is a good current `oya-governance-*` set, but pinned to GitHub `required_status_checks.contexts` — needs Forgejo Commit-Status / Argo-Workflows mapping (map §3 "Forgejo Commit Status as the gate sink").
  - `CUG-42` retired-vocab; `oya-foundry-*` retired-prefix; owner `foundry`.
- **hyperscaler_challenge:** QUESTIONABLE (as written). The *practice* (trunk-based, linear history, merge queue) is exactly what Google (monorepo + Critique + TAP), Meta, and GitHub itself do — fully aligned. BUT Google/AWS/Azure would **not** bind their branch model to a single SaaS forge's YAML schema; they abstract over the SCM (Google's Piper/CitC, the map's named bespoke-VCS destination). The GitHub-coupling is the un-hyperscaler-like part. Argues AMEND toward a forge-neutral encoding.
- **ai_slop:** Low-moderate. "ADR-0041-equivalent posture" section is slightly self-referential/hedgy. Otherwise concrete and well-reasoned (the root-Cargo race is a real, cited failure mode).
- **refinement:** Abstract branch-protection/merge-queue/CODEOWNERS to a forge-neutral spec with Forgejo + GitHub adapters; replace `CUG-42` example with `Team-42`; fix `oya-foundry-*`→`oya-governance-*` in CODEOWNERS; rename owner; resolve the "legacy ADR-0041" lineage (cite explicitly or drop); add planning front-matter.
- **consensus_needed:** YES — this ADR is load-bearing on the contested forge question. Crisp question: *"Is the canonical branch/merge-queue/branch-protection model authored forge-neutrally (Forgejo + GitHub adapters) or pinned to the founder's GitHub `jason931225/oyatie`, given source canon says Forgejo-then-bespoke-VCS?"*

---

### ADR-0042 — Observability stack (OTel SDK + VictoriaMetrics, in-house Leptos portal, gen_ai semconv)

- **decision_atom:** OpenTelemetry is the canonical instrumentation surface with license-clean storage (VictoriaMetrics/ClickHouse/Jaeger), per-cell namespacing, per-tenant cost-attribution, and gen_ai semantic conventions per capability — **but the AGPL-3 avoidance / VictoriaMetrics-canonical storage decision is RETIRED.**
- **current_status:** **Superseded** (`status: superseded`, `superseded_by: [ADR-0383]`) — front-matter is clean and self-aware; the ADR even carries a full "Superseded by" block. Matches the keystone map exactly.
- **disposition:** ARCHIVE. Superseded on disk; the storage-tier decision is reversed.
- **governing:** **ADR-0383** (Loki/Tempo/Mimir/Grafana retained under AGPL-3 with three carve-out gates; supersedes ADR-0042) + **ADR-0186** (LGTM backplane = canonical architecture). The OTel instrumentation surface, per-cell namespace, per-tenant cost dashboards, and gen_ai semconv are explicitly *carried forward* by ADR-0186 — those sub-decisions survive the archive.
- **truth_flag:** STALE (as a whole) — but PARTIAL-TRUE: the instrumentation/semconv half is still valid and re-homed in ADR-0186. The reversed half is the AGPL-3 prohibition + VictoriaMetrics-canonical claim.
- **in_masterplan:** N/A as authority (superseded). The *surviving* sub-decisions (OTel contract, gen_ai semconv) should be backfilled from ADR-0383/0186, not from here.
- **tensions:**
  - Sharp internal-corpus reversal: ADR-0042 says "Loki/Tempo/Mimir/Grafana AGPL-3 forbidden in product surface" — ADR-0383 *reverses* this with a self-hosted-network-clause carve-out. This is the canonical example (map §1.1) of "trust the superseding ADR."
  - `gen_ai.system = "oya-foundry"` + "Every Foundry capability invocation" — retired `foundry` brand again.
  - Many downstream ADRs in my own chunk (0038/0040/0043) still cite ADR-0042 as live — they need repointing to 0383/0186 (drift propagation).
- **hyperscaler_challenge:** QUESTIONABLE in original form, resolved by the supersession. The original "build an in-house Leptos observability portal to avoid Grafana" is exactly what no hyperscaler would do — Google/AWS/Azure ship managed Grafana (AMG) or their own consoles and would never hand-roll a Leptos UI to dodge AGPL. ADR-0383's reversal (keep Grafana LGTM, self-host) is the more hyperscaler-aligned answer. Confirms ARCHIVE.
- **ai_slop:** Moderate (in original body): "in-house Leptos portal long-horizon" is the kind of fabricated-ambition (W+18/W+24 stretch) that the founder's "own everything" tendency produces; the supersession effectively flagged it as slop. The `gen_ai.system` model list ("claude-opus-4-7-1m | gpt-5 | gemini-3-pro") is fabricated-precision on model names.
- **refinement:** No edit (read-only/archive). On the eventual consolidation pass: harvest the OTel-contract + gen_ai-semconv + per-tenant-cost sub-decisions into the masterplan via ADR-0383/0186; do NOT carry the VictoriaMetrics-canonical or AGPL-prohibition claims.
- **consensus_needed:** no — already resolved by ADR-0383.

---

### ADR-0043 — Secrets management (OpenBao supersedes Vault BUSL, per-tenant per-cell HSM, per-capability SecretProvider)

- **decision_atom:** OpenBao (MPL-2, Vault-API-compatible) is the canonical secrets store, with per-tenant per-cell HSM partitions (KCMVP-validated for KR + FIPS 140-3 Level 3 globally), a rotating ≤15-min session-token vault for external-AI adapters, and a per-capability `SecretProvider` trait so axes never touch raw secrets, all Cedar-gated and audit-chained.
- **current_status:** Proposed.
- **disposition:** AMEND (urgent text fix) → then KEEP. The decision is sound and current (OpenBao-over-BUSL-Vault matches the map's license-strict posture; Valkey/OpenBao/OSI-strict are all consistent). BUT the document is **textually corrupted**: the KR cryptographic-module standard "**KCMVP**" has been find-replaced into the nonsense token "**KCminimum-shippable-tier**" in ~12 places (Decision, HSM table, Consequences, Operational, References). GLOSSARY L88/L191/L396/L698 confirm KCMVP is the canonical term (한국암호모듈검증).
- **governing:** n/a (not superseded). License posture consistent with ADR-0013/0211/0345/0336 (the OSI-strict / BUSL-and-AGPL-out family).
- **truth_flag:** PARTIAL — decision TRUE; the document contains **GARBAGE tokens** ("KCminimum-shippable-tier" for KCMVP) that make the regulatory claim literally unreadable/wrong as written. This is the founder's "plain garbage/stale/wrong markdown" warning materializing.
- **in_masterplan:** NO binding; owner `foundry` (retired). Strong backfill once the KCMVP token is restored.
- **tensions:**
  - `KCminimum-shippable-tier` token corruption vs GLOSSARY-canonical **KCMVP** (internal contradiction with the project's own glossary). Almost certainly the residue of a global find-replace (a "tier" rename — possibly collateral damage from the ADR-0329 tier-system retirement sweep — that clobbered "KCMVP" via a "tier"→"shippable-tier" substitution). This is the single sharpest data-integrity defect in my chunk.
  - Owner `foundry`; "Foundry adapters / Foundry subscription-mode adapters" (retired → Intelligence per ADR-0335).
  - References ADR-0042 (observability intrusion alert, tracing filter) — dead ADR, repoint to 0383.
  - `K8 KEK rotation` in HSM table reads like another truncated token ("K8s"? or a key-class label) — minor, flag for review.
  - Consumes ADR-0044 service-mesh CA (correct dependency direction).
- **hyperscaler_challenge:** ALIGNED. AWS (KMS + CloudHSM + Secrets Manager), Google (Cloud KMS + Cloud HSM + Secret Manager), Azure (Key Vault Managed HSM) all do per-tenant KEK/DEK envelope encryption + FIPS 140-3 L3 HSM partitions + short-lived tokens. OpenBao-over-Vault is the correct license-driven call post-BUSL (HashiCorp's own relicense forced this industry-wide). The per-capability `SecretProvider` trait is good interface-enforced hygiene. Argues KEEP (after text fix).
- **ai_slop:** The corrupted `KCminimum-shippable-tier` token is the worst slop signal in the chunk — fabricated/garbled precision masquerading as a real standard name. Otherwise the ADR is concrete and well-structured.
- **refinement:** Global restore `KCminimum-shippable-tier`→`KCMVP` (verify against GLOSSARY L191); rename owner+`Foundry`→Intelligence/governance; repoint ADR-0042→0383; clarify `K8 KEK`; add planning front-matter; the OpenBao/HSM/SecretProvider core needs no design change.
- **consensus_needed:** no on the decision; **yes** as a data-integrity escalation — flag that a find-replace sweep corrupted a regulatory term, and the same sweep may have damaged sibling ADRs (audit the whole corpus for `*-shippable-tier` residue).

---

### ADR-0044 — Service mesh (Istio Ambient east-west, Envoy edge gateway, mTLS everywhere, audited cross-cell)

- **decision_atom:** Istio Ambient mode (ztunnel L4 + waypoint L7) is the canonical east-west mesh and Envoy Gateway the canonical north-south edge, with STRICT mTLS everywhere (SPIFFE SVID identity, per-cell HSM-issued CA), per-cell namespace isolation, and every cross-cell call explicitly Cedar-policied + audit-chained.
- **current_status:** Proposed.
- **disposition:** KEEP (minor AMEND for retired prefix). No superseding ADR found on disk; consistent with the K8s-everywhere + Cedar-gate + audit-chain canon (ADR-0254/0243/0003). Istio Ambient + Envoy ext-authz→audit-emitter is a coherent, current design.
- **governing:** n/a — not superseded. Ext-authz audit binding consistent with ADR-0003; Cedar gating consistent with ADR-0243/0246. mTLS CA consumes ADR-0043 (correct direction).
- **truth_flag:** TRUE — the only stale element is the `oya-foundry-<cell-id>` namespace in the per-cell namespace tree (retired prefix). Owner `cloud` is correct (not retired).
- **in_masterplan:** NO front-matter binding; otherwise clean. Good backfill — the per-traffic-type mTLS/Cedar/audit table is a reusable masterplan invariant.
- **tensions:**
  - `oya-foundry-<cell-id>` namespace + `oya-cloud-dcops-<cell-id>` — `oya-foundry-*` is retired (→ `oya-intelligence-*` per ADR-0335). The namespace tree should rename.
  - References ADR-0042 (observability collector for north-south OTel) — dead ADR, repoint to 0383/0186.
  - Istio Ambient is a *substrate* choice; watch against the orchestration cluster (ADR-0375 Talos + CAPI + ArgoCD). Talos+Istio-Ambient coexist fine (Talos is node-OS, Istio is mesh) — no conflict, but the pairing should be confirmed in the orchestration ADRs. The map lists no canonical mesh ADR in §3, so ADR-0044 is effectively the de-facto mesh authority and SHOULD be promoted/bound.
  - LINUX-side tension (cross-side): LINUX ADR-0018/0014 (framekernel "we are the host," own isolation, no separate containerd) competes with this Istio/Envoy/containerd-substrate posture (map §5 fault-line 3) — own-the-host vs assemble-the-substrate. Surface only.
- **hyperscaler_challenge:** ALIGNED. AWS (App Mesh/VPC Lattice), Google (Anthos Service Mesh = managed Istio; GKE Dataplane V2/Cilium), Azure (Istio-based add-on) all converge on Istio-or-Cilium mTLS meshes with SPIFFE identity. Istio *Ambient* specifically is the resource-efficient direction Google/Solo.io are pushing. One caveat: hyperscalers increasingly prefer **Cilium/eBPF** over Istio sidecars — Ambient narrows that gap, so the choice is defensible but worth a Cilium-vs-Istio-Ambient note. Argues KEEP.
- **ai_slop:** Low. "~40-60% resource reduction vs sidecar" is a plausible-but-unsourced precision figure (mild fabricated-precision — should cite). Otherwise concrete and well-bounded with a clean anti-scope.
- **refinement:** Rename `oya-foundry-<cell-id>`→`oya-intelligence-<cell-id>`; repoint ADR-0042→0383/0186; cite the 40-60% figure or soften it; add planning front-matter; **promote ADR-0044 to canonical-mesh authority** in the map/masterplan (it currently has no §3 row — a gap, since mesh is load-bearing). Add an explicit Cilium-vs-Istio-Ambient one-liner given hyperscaler drift.
- **consensus_needed:** no on the decision; a *light* founder confirmation that Istio-Ambient (not Cilium/eBPF) is the canonical mesh would close the §3 gap. Phrase: *"Is Istio Ambient the canonical service mesh, or should the mesh be Cilium/eBPF given hyperscaler convergence and the Talos substrate?"*

---

## Chunk notes for synthesis

**1. This is one authoring batch (2026-05-09) with uniform, mechanical defects — not design errors.** All seven ADRs share: (a) `owner: foundry` (six of seven; ADR-0038=council-architecture, ADR-0044=cloud) — RETIRED brand per ADR-0335/0347, should be `governance`/`cloud-intelligence`; (b) live cross-refs to **ADR-0042**, which is itself superseded by ADR-0383 — so 0038/0040/0043/0044 all point at a dead ADR (drift propagation cluster); (c) no `planning_impact`/`masterplan_ref` front-matter — they sit in the 91.2% unbound set the drift-prevention doc flags. **Fixing the batch is four global renames + four repoints, not seven redesigns.** The decision atoms are TRUE and masterplan-ready.

**2. ADR-0042 is the keystone supersession in this chunk and the drift epicenter.** It is correctly `superseded` on disk (ADR-0383, confirmed) but is still cited as live by half the chunk. Any masterplan backfill must harvest observability sub-decisions from ADR-0383/0186, not 0042, and a corpus-wide "repoint dead ADR-0042 refs" pass is warranted (this chunk alone has 4 stale citations).

**3. ADR-0043 is a DATA-INTEGRITY ALARM, not just a stale ADR.** The KCMVP→"KCminimum-shippable-tier" token corruption (~12 occurrences of a non-existent standard name) is almost certainly collateral from a global "tier" find-replace — plausibly the same ADR-0329 tier-system-retirement sweep that the founder warned produces "plain garbage" markdown. **Recommend a corpus-wide grep for `*-shippable-tier` / clobbered-`tier` residue** beyond my slice; sibling ADRs touched by the same sweep may carry identical corruption. This is the single highest-priority finding in chunk 6.

**4. Forge fault-line concentrates in ADR-0041 (and grazes ADR-0039).** ADR-0041 hard-binds the branch/merge-queue/CODEOWNERS model to GitHub YAML; ADR-0039 binds the supply-chain lane to `.github/workflows` + GitHub branch-protection. Both collide with source canon (Forgejo-then-bespoke-VCS, ADR-0363/0510) AND with the Argo-Workflows CI destination (ADR-0511). Note the irony: the GitHub-coupling *accidentally* matches the founder's GitHub migration directive while *contradicting* the very source canon the audit must respect — exactly the three-way tension in map §5. **The process decisions survive; only the forge encoding is contested.** ADR-0041 is the one consensus-gated ADR in my chunk.

**5. Admission-engine drift: ADR-0039's Kyverno is demoted.** ADR-0379 (Accepted, supersedes ADR-0183) made **Kubewarden** the default admission substrate with Kyverno as a first-class adapter. ADR-0039 still presents Kyverno as canonical. Resolvable by one section edit; flagged so the masterplan records Kubewarden-default, Kyverno-adapter.

**6. Hyperscaler verdict for the chunk: strongly ALIGNED.** Supply-chain (0039), progressive delivery (0040), secrets/HSM (0043), and service mesh (0044) are precisely what Google/AWS/Azure ship — in several cases (Cosign keyless, SLSA L3, 14.4× multi-window burn-rate, OpenBao-post-BUSL, Istio Ambient) the source ADRs are *more* modern/rigorous than hyperscaler defaults. The two places hyperscalers would diverge: (a) the original ADR-0042 "build an in-house Leptos portal to dodge AGPL Grafana" — already corrected by ADR-0383's reversal to managed-Grafana-style LGTM; (b) ADR-0041's single-forge YAML coupling — hyperscalers abstract over the SCM. Both divergences are toward *less* ownership/NIH, which is the recurring SOURCE-vs-LINUX "own vs assemble" axis (map §5 fault-lines 5).

**7. Mesh authority gap.** The keystone map §3 has no canonical "Service mesh" row, yet ADR-0044 is a clean, accepted-in-spirit mesh decision with no superseding ADR. Recommend promoting ADR-0044 to the canonical-posture table (mesh = Istio Ambient + Envoy + SPIFFE mTLS) and resolving the Cilium/eBPF-vs-Istio-Ambient question for the record.

**8. Retired-vocab leakage inventory (this chunk):** `foundry` owner ×6 + `oya-foundry-*` strings (0038 axis/StoreRef, 0039 crate path, 0040 cadence row, 0041 CODEOWNERS, 0042 gen_ai.system, 0043 adapter prose, 0044 namespace) → all should map to **Intelligence/cloud-intelligence/governance**; `CUG-42` example in ADR-0041 → **Team-42** (CUG retired 2026-05-09); `oya-foundry-fitness-*`-era lane names are already correctly `oya-governance-*` (the one place the batch is *ahead* of its own retirements).
