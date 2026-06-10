# ADR Audit — source-44

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-44
- **range:** lines 302–308 of `ls docs/decisions/ADR-*.md | sort`
- **ADRs reviewed:** ADR-0365, ADR-0366, ADR-0367, ADR-0368, ADR-0369, ADR-0370, ADR-0371 (7)

> Cluster character: ADRs 0365–0369 are the **2026-05-26 agentic-platform governance apex** wave — the operational continuation of the ADR-0364 "masterplan is GENERATED from the ADR log" keystone (which the founder GOAL + the brief now treat as the resolved direction of the keystone §4 OPEN question). 0368 is the explicit north-star charter that 0364–0367+0369 all serve; it is slated to become ADR-0000 in the re-foundation. 0370–0371 are a tightly-coupled pair on the **local production-fidelity Kubernetes substrate** (multi-node Talos on Apple Silicon) and its **secure remote access** (Cloudflare Tunnel L4 + Access). All seven are `status: Accepted`, `supersedes:[]`, `superseded_by:[]`.

---

### ADR-0365 — Automated ADR lifecycle: research → consensus → ADR → auto-propagate

- **decision_atom:** Every planning_impact ADR is the output of a governed pipeline (best-practice-research → consensus-plan → ADR template) and, once accepted, auto-propagates by regenerating the masterplan plus every artifact in its `affected_surfaces`, enforced by provenance, propagation-drift, decision-door, and COE-to-gate gates.
- **domain:** governance-process (cross-cutting: docs-ssot-masterplan)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted, not Proposed)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — this is itself masterplan-generation machinery; it directly governs how the masterplan is produced (ADR-0364 §1 generation + this ADR's `oya gen propagate`).
- **tensions:** (1) §4 (self-hosting/dogfood) names "git + **Jenkins** + Forgejo (ADR-0363)" as the substrate — Jenkins is retired-as-destination (keystone §1.3 / §2; ADR-0511 Argo Workflows is the destination, Jenkins transitory). Naming drift, not a decision conflict. (2) Sits squarely on the keystone §4 masterplan authored-vs-generated OPEN question — this ADR is firmly in the *generated-from-ADRs* camp; consistent with the founder GOAL as stated in the brief, but note `planning-ssot-drift-prevention.md` argues the opposite (masterplan-is-authority-ADRs-bind-in).
- **hyperscaler_challenge:** ALIGNED. Google (design-doc→ADR funnel + generated docs), AWS (docs-as-build-artifact, one-way/two-way-door, COE-to-correction), and the Kubernetes KEP generative model all independently validate "one canonical source, generate the rest." Implication: keep; only amend the Jenkins reference.
- **ai_slop:** None. Crisp, evidence-cited, bounded. The "nothing generated is hand-maintained; nothing enforced is left to discipline" doctrine is a real principle, not filler.
- **refinement:** AMEND-on-touch only: replace "git + Jenkins + Forgejo" with the current CI truth (Buck2 + Argo Workflows + Forgejo, oya-ci target) per ADR-0511/0513. Immutability doctrine means this is a forward note, not an edit.
- **consensus_needed:** Does the founder ratify *generated-from-ADRs* (this ADR's premise) as THE masterplan model, formally closing the keystone §4 split against `planning-ssot-drift-prevention.md`?

### ADR-0366 — Agentic high-throughput, self-enforcing, self-repairing pipeline

- **decision_atom:** A high-throughput conflict-free agentic dev pipeline — single-threaded owner-agent-per-service on disjoint flat paths, speculative merge-queue, affected-targets + two-tier pre/post-submit, deterministic self-repair, anti-thin-scaffold (PR-FAQ + Definition-of-Done), and quality gates (canary/chaos/error-budget/DORA) — is the prerequisite substrate for all feature work.
- **domain:** ci-cd-build (cross-cutting: agentic-platform)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — declares itself the T3+ feature-work prerequisite; its D1–D6 gates are masterplan deliverables.
- **tensions:** (1) References "ADR-0349 (CI farm), ADR-0361 (Jenkins-native CI)" and "git+Jenkins+Forgejo" — same Jenkins-as-destination retired-vocab drift (keystone §1.3); the live destination is Argo Workflows (ADR-0511). (2) Explicitly states it does NOT revive the retired `oya vcs` claim-ratchet (ADR-0363) — this is correct alignment, not a tension, and shows awareness of the retired forge tooling. (3) `door: one-way` while its sibling mechanism ADR-0369 is `door: two-way` — internally consistent (0366 is the irreversible doctrine, 0369 the reversible mechanism choice).
- **hyperscaler_challenge:** ALIGNED. Directly imports AWS single-threaded-owner (STO), Google affected-targets/TAP + content-hash cache + presubmit/postsubmit + culprit-bisection, Kayenta canary, DORA four-keys, error-budget policy, PR-FAQ, DoD. This is canonical Google/AWS multi-team SRE/DevProd practice applied to agents. Implication: keep; amend Jenkins refs only.
- **ai_slop:** None. Dense but each gate maps to a named, real hyperscaler mechanism. Risk it flags on itself ("risk of over-engineering; mitigated by adopting not building") is appropriately self-aware.
- **refinement:** AMEND-on-touch: Jenkins→Argo Workflows refs. Watch scope: D1–D6 is a large build backlog; sequencing (which gate first) is a masterplan ordering concern, not an ADR defect.
- **consensus_needed:** None contested; the pipeline-before-features ordering is the founder's own stated argument (recorded in Rejected alternatives).

### ADR-0367 — Trustless pre-merge verification gateway (PR-ceremony-less)

- **decision_atom:** Replace human PR-review with trustless verification — the producing agent never certifies its own work; a trusted runner hermetically re-executes every gate from a clean checkout and SLSA/cosign-signs the evidence, and a separate-identity adversarial reviewer-agent (run as a signed CI stage on the Intelligence service) must independently APPROVE before auto-merge.
- **domain:** ci-cd-build (cross-cutting: security-supplychain)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — defines the merge gate (`untrusted-evidence`, `reviewer-independence`) that all agent merges pass through; first concrete dogfood job for the Intelligence service.
- **tensions:** (1) "the farm / Jenkins (ADR-0349/0361)" as the trusted runner — Jenkins retired-as-destination drift; the trusted runner is now the Argo Workflows / oya-ci farm (ADR-0511/0513). The *principle* (a trusted hermetic runner signs evidence) is substrate-agnostic and survives. (2) Correctly routes adversarial review through **Intelligence** (ADR-0363 post-Foundry name) — aligned with retired-vocab (foundry→intelligence). (3) Forgejo auto-merge dependency carries the forge fault-line (keystone §5.4: GitHub vs Forgejo vs bespoke-VCS) — the *mechanism* binds to Forgejo, which is itself transitory (ADR-0510) and conflicts with the founder's GitHub migration directive.
- **hyperscaler_challenge:** ALIGNED. Separation-of-duties, SLSA provenance + cosign signing, hermetic re-execution, doubt-driven/adversarial review are core Google BeyondCorp/Binary-Authorization + SLSA + Amazon-separation-of-duties practice. "Trust the substrate, not the agent" is exactly how hyperscalers treat untrusted build inputs. Implication: keep; the only questionable bit is whether a fully PR-ceremony-less flow under-weights human judgment on novel/irreversible changes — but ADR-0365/0368 `door:one-way` founder sign-off covers that.
- **ai_slop:** None. The threat model ("the evidence collection itself can be false") is precisely stated and the design is the structural answer. Strong ADR.
- **refinement:** AMEND-on-touch: trusted-runner = Argo/oya-ci, not Jenkins. Confirm Forgejo-vs-GitHub binding (see §5.4) — if GitHub becomes the host, the gateway re-targets GitHub Commit Status / required checks; the trust model is unchanged.
- **consensus_needed:** Forge fault-line — does the trustless gateway bind to Forgejo (canon), the bespoke VCS (ADR-0510 destination), or GitHub (founder directive)? Same question as 0369-D3.

### ADR-0368 — Self-governing agentic platform — north-star charter

- **decision_atom:** Oyatie is a self-improving, self-healing, self-governing agentic platform: a maximal safe fleet of agents executes a masterplan generated from ADRs, every failure becomes a new gate, deterministic violations self-repair, every step is gated, and the only human input is architectural decisions — which are themselves challenged against hyperscaler best-practice before acceptance.
- **domain:** governance-process (cross-cutting: agentic-platform)
- **current_status:** Accepted (2026-05-26) — apex; slated to become **ADR-0000** in the re-foundation (per its own body + ADR-0364 §6 / D7).
- **disposition:** KEEP — and this is the **masterplan-anchor candidate**: it is explicitly "the apex; every other ADR serves it."
- **proposed_resolution:** NA
- **governing:** — (it governs; nothing supersedes it)
- **truth_flag:** TRUE
- **in_masterplan:** YES — it IS the charter the masterplan instantiates. If any single ADR seeds the masterplan preamble, this is it.
- **tensions:** (1) Self-referential / aspirational by its own admission ("until then the charter is aspirational and the gates show as planned") — TRUE but its deliverables D1–D6 are largely meta-gates (`fleet-utilization`, `aspirational-enforcement`, `adr-challenge`) that are governance scaffolding, not yet built. Truth_flag stays TRUE because the ADR honestly labels itself aspirational. (2) The re-foundation-to-ADR-0000 claim depends on ADR-0364-D7 executing — collides with the linux pilot renumber plan (keystone §6.4) and the bespoke "re-found from ADR-0000" idea in `planning-ssot-consolidation.md`; if both fire, ADR-0000 is contested between this charter and the linux pilot's renumbered series. (3) D6 "even the founder's ADRs are challenged" is philosophically strong but operationally circular if the challenger (Intelligence-reviewer) is itself agent-produced — mitigated by the research-evidence + COE flywheel, but the "who watches the watcher" loop is inherent.
- **hyperscaler_challenge:** QUESTIONABLE (in degree, not direction). The *components* (COE→gate, error-budget, self-healing automation, design-review bar) are pure Google/Amazon SRE/operational-excellence. But "**maximum agents always; idle capacity is a defect/alert**" (D1) is NOT how hyperscalers operate — they optimize for *throughput at acceptable cost/quality*, not max concurrency for its own sake (utilization-maxxing causes thrash, contention, and review-queue saturation). Google/AWS would gate concurrency on the merge-train's healthy throughput, not pin it to "maximum." Implication: AMEND D1's framing from "max safe concurrency, idle = defect" to "concurrency scaled to sustained green merge-train throughput" — keep the charter, soften the utilization-maximalism.
- **ai_slop:** Low-but-present. The "self-improving, self-healing, self-governing" triad + "nothing escapes the bar" is charter rhetoric; it is grounded by concrete D1–D6 deliverables so it does not tip into pure slop, but D1's idle-is-a-defect is an unexamined maximalism worth challenging.
- **refinement:** AMEND D1 utilization framing (see hyperscaler_challenge). Resolve the ADR-0000 ownership question (charter vs re-foundation vs pilot renumber) before any re-foundation executes.
- **consensus_needed:** Two crisp founder questions — (a) "Is **maximum** agent concurrency the goal, or maximum *sustained-green-throughput* (which may be far below max concurrency)?" (b) "When the re-foundation runs, does THIS charter become ADR-0000, and how does that reconcile with renumbering the 26 linux pilot ADRs?"

### ADR-0369 — Gated stacked-trunk change-flow with a speculative merge-train

- **decision_atom:** The concurrent-agent change-flow mechanism is gated stacked-trunk on plain git + Forgejo PRs — ownership-sharded disjoint paths, ghstack-style small stacked diffs, a single binding required-status-check (the trusted-runner signature, NOT Forgejo's race-prone merge button), and a later speculative stack-aware merge-train — explicitly NOT Jujutsu/Gerrit-server/pre-receive-primary.
- **domain:** ci-cd-build (cross-cutting: forge-vcs)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** PARTIAL — D1–D3 are "now" deliverables; D4 (speculative merge-train) is explicitly deferred ("LATER", gated on ADR-0363 §3 concurrency trigger), so only part is currently in-scope for the masterplan.
- **tensions:** (1) Forge fault-line (keystone §5.4): the whole mechanism binds to **Forgejo** (no native merge-queue, forgejo#5102; merge-button races #11224/#8189), but Forgejo is itself transitory (ADR-0510 → bespoke VCS) and conflicts with the founder's GitHub directive. The ADR's design (binding gate = the required check, not the merge button) is admirably substrate-defensive and would largely port to GitHub, but the ground-truth Forgejo issue analysis is Forgejo-specific. (2) `door: two-way` (reversible) is correctly assigned — this is a swappable mechanism under the fixed 0367 trust model, unlike its one-way siblings. (3) Revives ADR-0111 (speculative merge-queue) — consistent, no conflict.
- **hyperscaler_challenge:** ALIGNED. Stacked-diffs (Meta/Phabricator, Google Critique), bors/TAP speculative merge-train (Rust project, Google), CODEOWNERS routing, "test the projected post-merge state" (Zuul/bors) are all proven hyperscaler/large-OSS practice. The jj rejection is well-reasoned (agentjj's jj→git reversion, no-staging-area squashing, single-writer working-copy). Implication: keep.
- **ai_slop:** None. This is the most empirically grounded ADR in the chunk — cites live Forgejo issue numbers and a real upstream project's reversion as evidence. Exemplary.
- **refinement:** AMEND-on-touch only IF the forge host changes (GitHub/bespoke-VCS) — then re-derive the "required-check-not-merge-button" binding against the new host's semantics; the mechanism choice (stacked-trunk + speculative train) is host-portable.
- **consensus_needed:** Same forge question as 0367 — "Which forge does the change-flow bind to: Forgejo (canon/transitory), bespoke monorepo-VCS (ADR-0510 destination), or GitHub (founder directive)?" This ADR cannot be fully masterplan-stable until §5.4 resolves.

### ADR-0370 — Local production-fidelity substrate: multi-node Talos on Apple Silicon

- **decision_atom:** The local dogfood substrate is multi-node Talos Linux on Parallels Desktop 26 (3 control-plane HA-etcd + 2 Kata-capable workers, Cilium CNI, everything-as-IaC via prlctl + ArgoCD app-of-apps), replacing single-node colima+k3s — with Kata cloud-hypervisor recorded as a KNOWN LOCAL GAP (Apple-Silicon nested virt is too shallow for CLH microVMs; Kata fidelity validated only in the real cloud).
- **domain:** node-os (cross-cutting: orchestration-scheduling)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** PARTIAL — TRUE as a decision, but D1 is a self-corrected partial: the original "nested virt works for Kata" make-or-break claim was **empirically falsified after testing** (shallow vEL2/vGIC, lima#4498), and the ADR honestly records the correction. So the substrate stands but its headline Kata-fidelity rationale is locally unmet (cloud-only). Flagged PARTIAL for that gap, not for any falsehood — the honesty is a strength.
- **in_masterplan:** YES — `M-PRODUCTION-FIDELITY-SUBSTRATE` milestone; IaC under `/infra/talos/`. This is dogfood infrastructure, masterplan-relevant as the substrate the agent fleet runs on.
- **tensions:** (1) Aligns with SOURCE canonical orchestration posture (keystone §3: Talos node-OS, ADR-0375) — consistent, not conflicting. (2) Crosses the keystone §5.3 isolation fault-line vs LINUX: LINUX ADR-0018/0025 want an owned Rust "Talos"/framekernel-as-host; SOURCE here doubles down on *actual* Talos. Surface only — this is the deliberate own-vs-adopt divergence. (3) Parallels is a paid dependency requiring colima shutdown to fit 128GB RAM — an operational constraint, not an architecture flaw; `door: two-way` correctly marks it reversible (UTM is the free fallback). (4) Kata-local-gap means ADR-0147's pinned `runtimeClassName: kata-cloud-hypervisor` cannot be honored locally — local workloads relax to default runtime; the pin holds only in cloud.
- **hyperscaler_challenge:** ALIGNED. Immutable API-managed node-OS (Talos), HA-etcd quorum, GitOps app-of-apps, chaos/anti-affinity fidelity, secrets-in-KMS-never-git are textbook production discipline; empirically proving the make-or-break assumption (then correcting it) is exactly the operational-excellence rigor a hyperscaler design review demands. Implication: keep.
- **ai_slop:** None. The post-test D1 correction is the opposite of slop — it's evidence overriding the original hopeful claim. Model ADR for honesty.
- **refinement:** Minor: the milestone `M-PRODUCTION-FIDELITY-SUBSTRATE` is fine, but note the `.omx/plans/` research paths (vs `.omc/`) — confirm path is real on disk (not load-bearing to the decision). No decision change needed.
- **consensus_needed:** "Given Kata cloud-hypervisor is unachievable locally on Apple Silicon, is multi-node Talos-on-Parallels still worth the 5-VM / colima-shutdown / paid-Parallels cost for the *remaining* fidelity (HA-etcd, chaos, anti-affinity), or is single-node + cloud-only-Kata-CI sufficient?"

### ADR-0371 — Secure control-plane access via Cloudflare Tunnel (L4) + Access

- **decision_atom:** The Talos control plane is served at k8s.oyatie.dev with no public IP and no inbound ports via a Cloudflare Tunnel as an L4 TCP route (preserving end-to-end apiserver mTLS by never terminating TLS at the edge) fronted by Cloudflare Access Zero-Trust Service-Auth, with the cloudflared connector running in-cluster as IaC and all credentials in OpenBao/Keychain.
- **domain:** networking-mesh (cross-cutting: identity-authn / security-supplychain)
- **current_status:** Accepted (2026-05-26)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE (PARTIAL implementation: D1/D5 done 2026-05-26; D2/D3/D4 pending the live apiserver VIP + deployed connector — the decision is sound, execution is in flight).
- **in_masterplan:** YES — same `M-PRODUCTION-FIDELITY-SUBSTRATE` milestone; the access layer above ADR-0370.
- **tensions:** (1) Introduces a **Cloudflare** runtime dependency on the control-plane access path — somewhat at odds with the corpus-wide "own/self-host the substrate" + OSI-strict posture (keystone §3 license posture); mitigated in-ADR (direct VIP works on-LAN; Cloudflare only for remote). This is a pragmatic managed-SaaS dependency for a *local dogfood* host, lower stakes than a product dependency. (2) `door: two-way` correct — the access mechanism is swappable (WireGuard VPN is the noted viable alternative). (3) No retired-vocab issues; cleanly scoped.
- **hyperscaler_challenge:** ALIGNED (with one note). Zero-Trust edge identity + outbound-only connector + no-inbound-ports + mTLS-preserved-via-L4-passthrough + least-privilege service tokens is exactly the BeyondCorp/Cloudflare-Access pattern hyperscalers use for control-plane access. The one questionable bit: a hyperscaler would typically own its identity-aware proxy (e.g., Google IAP) rather than depend on Cloudflare — but for a one-person dogfood host, adopting Cloudflare Access is the correct cost/value trade. Implication: keep; flag the managed-dependency for the eventual "own the IAP" ratchet (consistent with LINUX/SOURCE own-when-proven principle, keystone §5.5).
- **ai_slop:** None. The load-bearing constraint (L7 termination breaks client-cert mTLS → must be L4) is precisely correct and is the whole basis of the decision. Tight, well-reasoned.
- **refinement:** Pin the cloudflared image to a digest before production (the ADR already self-flags this under require-signed-images). No decision change.
- **consensus_needed:** None contested. Optional future: "When do we replace Cloudflare Access with an owned identity-aware proxy?" — a ratchet-threshold question, not a present blocker.

---

## Chunk notes

**Overall posture:** This is a high-quality, internally-coherent cluster — among the strongest-authored ADRs in the corpus. 0365–0369 form the **apex agentic-governance stack** (charter 0368 → lifecycle 0365 → pipeline 0366 → trust 0367 → mechanism 0369), all serving the ADR-0364 generated-masterplan keystone. 0370–0371 are a clean **local-substrate pair**. Every ADR is `Accepted`, none is `Proposed` (so **no unaccounted proposals** to RATIFY/DROP in this chunk), none supersedes or is superseded, all carry research citations and `verified_by` gates. No AI slop, no garbage, no WRONG decisions.

**Disposition tally:** KEEP ×7. AMEND-on-touch (forward-note only, immutability respected) for the Jenkins-as-CI-destination references in 0365/0366/0367, and for ADR-0368-D1's "maximum agents / idle-is-a-defect" framing.

**Cross-cutting tensions surfaced (do not resolve — flag for founder):**
1. **Jenkins retired-vocab drift** (0365 §4, 0366 refs, 0367 §2): three ADRs name "git + **Jenkins** + Forgejo" / "ADR-0361 Jenkins-native" as the trusted-runner/substrate. Per keystone §1.3/§2 the live CI destination is **Argo Workflows + Buck2 + oya-ci** (ADR-0511/0513); Jenkins is transitory bootstrap only. The *principles* (trusted hermetic runner, signed evidence) are substrate-agnostic and survive; only the named tool is stale. Trust the superseding chain (keystone §6).
2. **Forge fault-line (§5.4) is load-bearing for 0367 + 0369:** both bind the merge gateway/change-flow to **Forgejo** (Commit-Status-API, auto-merge, CODEOWNERS, ghstack-style PRs). Forgejo is itself transitory (ADR-0510 → bespoke hyperscaler monorepo-VCS) and conflicts with the founder's stated **GitHub** `jason931225/oyatie` migration directive. The designs are admirably substrate-defensive (the binding gate is the required *check*, not the merge button) and would largely port, but this is the single biggest open dependency for this chunk's masterplan-stability. **Surface to founder: which forge?**
3. **Masterplan authored-vs-generated (§4):** 0365 (and 0368) firmly assume *generated-from-ADRs*. The brief + founder GOAL treat this as the resolved direction (ADR-0364/0365 Accepted), but `planning-ssot-drift-prevention.md` still argues masterplan-is-authority. If the founder ratifies generated-from-ADRs, this whole chunk is the canonical machinery; one founder confirmation closes it.
4. **ADR-0000 / re-foundation contention:** ADR-0368 claims it "becomes ADR-0000 in the re-foundation" (ADR-0364-D7). This collides with (a) the `planning-ssot-consolidation.md` "re-found from ADR-0000 with `consolidates:` provenance" plan and (b) the guaranteed LINUX-pilot renumber-on-merge (keystone §6.4, all 26 linux ADRs collide with source 0001–0514). Sequencing the re-foundation must reconcile all three.
5. **ADR-0368-D1 utilization maximalism** is the one genuinely *questionable* hyperscaler-alignment item: "maximum agents always, idle capacity is a defect" is not how Google/AWS run — they gate concurrency on sustained-green throughput, not max-concurrency-for-its-own-sake. Recommend amending the framing (keep the charter, drop idle-is-a-defect).
6. **Kata-local-gap (0370-D1)** is honestly self-corrected: ADR-0147's pinned `kata-cloud-hypervisor` runtimeClass cannot be honored on the Apple-Silicon local substrate (shallow nested virt) — Kata fidelity is cloud-only; local workloads relax to the default runtime. Not a contradiction, a documented limit; but it means the *headline rationale* for going multi-node-Talos-local (Kata fidelity) is locally unmet — only HA-etcd/chaos/anti-affinity fidelity remains, which feeds the consensus question on 0370.

**LINUX-pilot relevance:** 0370 (multi-node Talos adoption) sharpens the keystone §5.3 isolation fault-line — SOURCE adopts *actual* Talos here while LINUX ADR-0018/0025 want an owned Rust framekernel/"Talos." Surface, do not resolve. 0367/0369's forge binding intersects §5.4. None of these seven ADRs reference or conflict with the linux pilot's owned-DB/owned-policy posture directly.

**No proposals to adjudicate; no archive/supersede candidates; the chunk is masterplan-ready pending the forge decision (§5.4) and two founder confirmations (generated-masterplan ratification; ADR-0000 re-foundation sequencing).**
