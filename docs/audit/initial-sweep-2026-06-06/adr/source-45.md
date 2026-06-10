# ADR Audit — SOURCE chunk 45

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** 45 (slice `sed -n "309,315p"` of sorted `decisions/ADR-*.md`)
- **range:** ADR-0372 … ADR-0377 (two distinct ADR-0377 files — genuine number collision)
- **ADRs-reviewed:** 7 — ADR-0372, ADR-0373, ADR-0374, ADR-0375, ADR-0376, ADR-0377-forgejo-board, ADR-0377-kafka-to-pulsar
- **auditor stance:** READ-ONLY; trust the *superseding* ADR over stale front-matter; retired vocab per keystone map (foundry→intelligence/governance, tier→tenant-class, Kafka→Pulsar, Jenkins-as-destination retired→Argo Workflows per ADR-0511).

---

### ADR-0372 — Frontend stack: SolidJS/TS app-shell + Rust→WASM compute modules

- **decision_atom:** The app-shell frontend is **Leptos (Rust→WASM, SSR+hydration)** as canonical; the earlier SolidJS/TS choice was reversed and this ADR is now superseded.
- **domain:** product-ux (frontend/app-shell).
- **current_status:** `Superseded` (front-matter `superseded_by: [ADR-0393]`; body "SUPERSEDED by ADR-0393 (2026-05-29)"). ADR-0393 confirmed present on disk.
- **disposition:** ARCHIVE — cleanly superseded; retain as historical record of the SolidJS decision + its 2026-05-27 Leptos-retention amendment.
- **proposed_resolution:** NA (status is Superseded, not Proposed).
- **governing:** ADR-0393 (`ADR-0393-leptos-canonical-app-shell-frontend.md`) — Leptos canonical; SolidJS not a canonical target.
- **truth_flag:** STALE — internally self-consistent but no longer the live decision; its own Decision section ("TypeScript + SolidJS") is the *opposite* of current canon. The supersession chain (Accept SolidJS → amend to Leptos-retained → supersede to Leptos-canonical) is unusually churny in <4 days; the body still reads as if SolidJS were the decision, which is index-poisoning for any reader who skips the Status block.
- **in_masterplan:** PARTIAL — the *surviving* truth (Leptos canonical + backend-stays-Rust + WASM-for-compute-only) belongs in the masterplan; the SolidJS decision itself does NOT.
- **tensions:** Whiplash with ADR-0393 (same domain, reversed within 3 days). Cross-side: LINUX has no competing frontend ADR, so no two-repo tension here. Retired-vocab note: D2/D3 still describe SolidJS as the surviving build track in some deliverable text — superseded.
- **hyperscaler_challenge:** ALIGNED (the *reasoning*), QUESTIONABLE (the *outcome*). Google/AWS/Azure overwhelmingly ship **TS/JS shells** and reserve WASM for compute — exactly ADR-0372's original logic. ADR-0393's reversal back to full-shell Rust/WASM (Leptos) is the *anti-hyperscaler-norm* choice; a hyperscaler would not put its operator console on a single-maintainer-heavy Rust-frontend stack. Implication: flag ADR-0393 (the live decision) as the hyperscaler-questionable one; 0372 archives regardless.
- **ai_slop:** No. Honest, well-cited (krausest, Leptos discussion #2627). The amendment + supersede are deliberate founder decisions, not slop.
- **refinement:** On merge, do not carry the body's "Decision" into any generated masterplan — only the Status/supersession edge.
- **consensus_needed:** Founder question (belongs to ADR-0393, surfaced here): *Is a full Rust/WASM (Leptos) app-shell defensible against the hyperscaler norm of TS shells + WASM-compute-only, given the talent-pool and cold-start-TTI penalties this very ADR documents?*

### ADR-0373 — cloud-intelligence gateway production design (provider-abstraction, key-pool resilience, audit)

- **decision_atom:** The cloud-intelligence LLM gateway adopts a provider-abstraction adapter behind a canonical OpenAI-compatible REST surface (OpenAPI 3.2.0, SSE passthrough), a key-pool failure→blacklist→cooldown→restore resilience state machine with per-provider circuit breakers, Bedrock-shaped immutable per-invocation audit with default-off body logging, and a provisional OpenSLO SLI set.
- **domain:** intelligence-ai (cross-cut: api-contracts for the OpenAI-compatible surface).
- **current_status:** `Accepted` (2026-05-26; `superseded_by: []`).
- **disposition:** KEEP — current, correct, on-vocabulary (uses **cloud-intelligence**, the post-0335 name; "foundry" absent).
- **proposed_resolution:** NA (Accepted).
- **governing:** —
- **truth_flag:** TRUE. Design-stage record honestly scoped via manifest `explicit_non_claims` (no live deploy, no measured SLO, no runtime audit persistence). The Rejected-alternatives note even explains WHY it took number 0373 instead of reusing 0370/0371/0372 — exemplary honest-claims hygiene.
- **in_masterplan:** YES — canonical design of the cloud-intelligence consumer-AI gateway; aligns with keystone Intelligence/AI posture (two-layer intelligence substrate, ADR-0255/0335).
- **tensions:** None material. Minor: cites ADR-0145 (inter-µsvc comms) which the keystone map flags as a reform anchor (0140/0141 superseded *into* 0145) — citation is to the live 0145, so fine. Naming aligns with the keystone "cloud-intelligence is the valid name" ruling.
- **hyperscaler_challenge:** ALIGNED. This IS the hyperscaler pattern — explicitly grounded in Azure APIM AI gateway, AWS Bedrock model-invocation-logging, Cloudflare AI Gateway, Kong AI Proxy, LiteLLM, OWASP LLM Top-10. Fail-closed on key exhaustion (denial-of-wallet defense) is exactly what AWS/Azure do. No amend/archive implication.
- **ai_slop:** No. Dense but every claim is sourced to a named brief; "provisional SLO, no official vendor SLO" is an honest hedge, not slop.
- **refinement:** None required; ensure the provisional-SLO caveat survives into any masterplan projection so no SLA claim is implied.
- **consensus_needed:** None.

### ADR-0374 — CI webhook gateway (Forgejo → Jenkins gated pipeline trigger)

- **decision_atom:** A flat single-concern Rust microservice (`ci-webhook-gateway`) converts a Forgejo `pull_request` event into a gated Jenkins `oyaCiLane` run — fail-closed constant-time HMAC verification, closed-router PR-event parsing, kick-only dispatch (never self-certifies) — retiring the manual `enforce_admins` admin-relax-merge seam.
- **domain:** ci-cd-build (cross-cut: forge-vcs — it is the Forgejo↔CI trigger seam).
- **current_status:** `Accepted` (2026-05-26; `superseded_by: []`).
- **disposition:** AMEND — the *trigger mechanism* (HMAC webhook receiver, kick-only, honest-Unimplemented boundaries) is sound and KEEP-worthy, but the **Jenkins-as-orchestrator** decision (the "RESOLVED 2026-05-26" section) is overtaken by the CI churn chain: ADR-0511 makes **Argo Workflows** the destination orchestrator and **Jenkins transitory bootstrap**, and ADR-0513 introduces bespoke-Rust oya-ci. The gateway's `PipelineDispatcher` trait was deliberately built agnostic, so the dispatch *target* must be re-pointed Jenkins→Argo-Workflows/oya-ci.
- **proposed_resolution:** NA (Accepted).
- **governing:** ADR-0511 (Argo Workflows supersede Jenkins-as-destination), ADR-0513 (oya-ci) — for the orchestrator clause only; the receiver/HMAC/dispatch-port decision survives.
- **truth_flag:** PARTIAL — the receiver design is TRUE; the "Jenkins sequences admission→gates→reviewer→merge" orchestrator clause and every `oyaCiLane`/`infra/ci/jenkins/*` reference is STALE post-0511/0513. The trait-agnostic design means the fix is a re-point, not a rebuild — the ADR even pre-authorized this as "a cheap, reversible follow-up."
- **in_masterplan:** PARTIAL — keep the webhook-trigger + fail-closed-HMAC + kick-only-trusted-runner-separation principles; drop/replace the Jenkins-as-orchestrator commitment with the Argo-Workflows/oya-ci destination.
- **tensions:** (1) Jenkins-as-orchestrator vs ADR-0511 Argo-Workflows-as-destination (CI churn chain, keystone §1.3). (2) Forge substrate = Forgejo here, but keystone §5 fault-line: founder migration directive = **GitHub** `jason931225/oyatie`, and ADR-0510 reframes Forgejo as *transitory* with bespoke-VCS as destination — so this gateway's Forgejo-specific webhook coupling sits on a contested substrate. (3) Correctly uses post-foundry naming ("foundry name eradicated repo-wide per ADR-0362") — good vocab hygiene, explicitly supersedes the ADR-0112 foundry receiver.
- **hyperscaler_challenge:** ALIGNED (pattern), QUESTIONABLE (engine). Fail-closed HMAC-on-raw-body-before-parse, idempotent at-least-once, producer-must-not-certify-its-own-work (trusted-runner separation) are textbook hyperscaler CI security. But Jenkins-as-orchestrator is NOT what Google/AWS/Azure run (they use Prow/CodeBuild/Pipelines, not Groovy DAGs) — and the repo itself already corrected this via ADR-0511 (Argo) + ADR-0513 (Prow-shaped oya-ci). Implication: amend the orchestrator target to match the already-decided Argo/oya-ci destination.
- **ai_slop:** No. Strong honest-boundaries discipline (typed `Unimplemented`/501 + placeholder-debt tokens instead of lying stubs). The "fix the gateway, do NOT revert to admin-merge" runbook rule is real operational thinking.
- **refinement:** Re-point `PipelineDispatcher` from the Jenkins `oyaCiLane` runner to the Argo-Workflows/oya-ci destination; update the 15-status-context / `infra/ci/jenkins/*` references; reconcile the Forgejo coupling against the GitHub-migration + bespoke-VCS-destination fault-line.
- **consensus_needed:** *Given the founder's GitHub migration directive and ADR-0510's bespoke-VCS destination, should the CI-trigger seam be rebuilt against GitHub/bespoke-VCS webhooks rather than hardwired to Forgejo — and does the dispatch target move to Argo Workflows / oya-ci now (vs after a Jenkins bootstrap)?*

### ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (retire Omni / OCI-TF / on-prem)

- **decision_atom:** The cluster fleet adopts **Talos (immutable node-OS) + Cluster API (declarative lifecycle) + per-cell Argo CD (pull GitOps)** with Cilium L3/L4 + Istio Ambient L7 and Kata/Cloud-Hypervisor untrusted worker pools, retiring Sidero Omni and the kubeadm/containerd/istio-envoy on-prem stack (supersedes ADR-0120/0121).
- **domain:** node-os (cross-cut: orchestration-scheduling).
- **current_status:** `Accepted` (2026-05-27; `supersedes: [ADR-0120, ADR-0121]`, `superseded_by: []`).
- **disposition:** KEEP — this IS the current canonical orchestration/node-OS posture per keystone §3 (Talos+CAPI+ArgoCD). Supersession of 0120/0121 is confirmed on the keystone supersession graph.
- **proposed_resolution:** NA (Accepted).
- **governing:** — (it is itself the governing ADR for the retired 0120/0121).
- **truth_flag:** TRUE. On-vocabulary, cleanly supersedes, declares its known SPOF gap honestly (single-site CAPI management control plane until HA hardened).
- **in_masterplan:** YES — keystone canonical Orchestration/k8s posture.
- **tensions:** **Major cross-side fault-line (keystone §5.3):** LINUX ADR-0025 wants a *Rust "Talos"* (beat-or-parity) and LINUX ADR-0018 a framekernel "we are the host, no separate containerd" model — directly competing with SOURCE's adoption of *actual* Talos + containerd. This is the own-the-node-OS vs assemble-the-substrate tension. Also note ADR-0375 D4 says GitOps source "= GitHub at bootstrap, flips to Forgejo post-cutover (ADR-0247)" — collides with the keystone §5 GitHub-vs-Forgejo-vs-bespoke-VCS three-way (founder directive = GitHub canonical, not a bootstrap-only mirror).
- **hyperscaler_challenge:** ALIGNED. Talos+CAPI+ArgoCD is precisely the CNCF-standard declarative cluster-lifecycle pattern; the ADR explicitly rejects proprietary Sidero Omni *because* "it is not how hyperscalers provision." Kata/Cloud-Hypervisor for untrusted tenants mirrors GKE Sandbox/Fargate-microVM. No amend/archive implication; this is a model hyperscaler-substrate choice.
- **ai_slop:** No. Concrete `infra/` paths, real `verified_by` shell commands, sourced provider pins.
- **refinement:** Reconcile D4's "flip to Forgejo post-cutover" against the live GitHub-canonical founder directive (keystone §5). No other change.
- **consensus_needed:** *Is SOURCE's adoption of actual Talos the canonical node-OS, or does LINUX ADR-0025's "Rust Talos" / ADR-0018 framekernel-as-host supersede it on merge? (own-vs-adopt the node-OS — the sharpest substrate-ownership decision in the two-repo set.)*

### ADR-0376 — Oyatie managed-Kubernetes product surface (two-tier: hosted-default + dedicated-premium)

- **decision_atom:** Oyatie's managed-Kubernetes product is a **two-tier offering on the ADR-0375 substrate** — Kamaji hosted control-planes-as-pods as the DEFAULT (dense, GKE/EKS/OKE economics) and a dedicated full Talos spoke per tenant as the PREMIUM (sovereign/air-gapped) — adopting Kamaji as a second additive clusterctl-compliant CAPI control-plane provider, dogfood-first, with billing/SLA/DPIA/external-GA deferred.
- **domain:** orchestration-scheduling (cross-cut: product-ux / marketplace-commerce — it is a sellable SKU surface).
- **current_status:** `Accepted` (2026-05-27; `superseded_by: []`).
- **disposition:** KEEP — current, builds cleanly on 0375, supersedes nothing.
- **proposed_resolution:** NA (Accepted).
- **governing:** —
- **truth_flag:** TRUE. Decision-stage (D1–D4 explicitly "BUILT in later lanes, NOT now"); names the four future microservices and resolves the placeholder-debt token. Uses **tenant** scoping correctly; the word "tier" here = **product tier (hosted/premium)**, which is NOT the retired tenant "tier-system" of ADR-0329 — *watch-item but legitimate* (different axis, same caution the keystone flags for autonomy-tiers).
- **in_masterplan:** YES (as a decision record / forward product doctrine; no runtime claim yet).
- **tensions:** (1) Naming overload — "two-tier" / "premium tier" sits adjacent to the retired tier-system vocab; recommend masterplan projection say "hosted vs dedicated **SKU**" to avoid `tier`-leakage triggering the ADR-0329 lint. (2) Kamaji = CNCF *Sandbox* single-backer (Clastix) — tracked risk, k0smotron named as drop-in fallback (good two-way-door hygiene). (3) Strategically extends ADR-0375; if LINUX's own-node-OS posture (ADR-0025) wins on merge, this entire product surface re-bases.
- **hyperscaler_challenge:** ALIGNED. The hosted-control-plane-as-pods (Kamaji) default = literally the GKE/EKS/OKE economic model; the control-plane-economics framing (~$73/tenant/month dedicated tax) and the Gardener-vs-Kamaji sourced bake-off ("why not Gardener?") are exactly the trade study a hyperscaler PM would run. Rejecting Gardener on substrate-fit (not CAPI-native → substrate U-turn) is defensible. No amend/archive implication.
- **ai_slop:** No — this is one of the higher-quality ADRs in the chunk: real cost figures, named production adopters (NVIDIA DOCA/DPF, Rackspace, OVHcloud, IONOS), an explicit founder-challenge alternative with sourced comparison, and a clean two-way-door fallback.
- **refinement:** In any masterplan projection, rename "tier" → "SKU/plan" to dodge retired-tier-vocab lint; keep the deferred GA legs (billing/SLA/DPIA) flagged as not-yet-decided.
- **consensus_needed:** None on the decision itself. (Latent: if/when external-GA ADR lands, revisit Gardener per the ADR's own "re-evaluate if time-to-market for full external product dominates" trigger.)

### ADR-0377 (forgejo-board) — Forgejo board projection with git-ref CAS fallback

- **decision_atom:** Autonomous masterplan-deliverable tracking uses **Forgejo Issues + exclusive scoped labels** as the human/audit board projection with **plain git-ref compare-and-swap** (`refs/heads/claims/<deliverable-id>`) as the concurrency lock — no GitHub Projects, no revived `oya git`/`oya vcs`, no bespoke board daemon.
- **domain:** agentic-platform (cross-cut: forge-vcs).
- **current_status:** `Proposed (conditional: Accepted only after ADR-0377-D2 and ADR-0377-D3 code/tests pass)`.
- **disposition:** AMEND — the decision is sound but carries two defects: (a) it is a **proposal that must be RATIFIED or DROPPED**, and (b) it **collides on the ADR-0377 number** with the Accepted Kafka→Pulsar ADR (keystone §6.1) — one MUST renumber. Recommend this conditional one renumbers (the Kafka one is `Accepted` and supersedes ADR-0005, so it holds the number; this board ADR is the junior `Proposed` claimant).
- **proposed_resolution:** **RATIFY** (conditionally, on its own terms) — the mechanism is correct and ADR-0363-compliant (git-ref CAS is the right concurrency primitive; labels-as-projection-not-lock is the right call). But ratification is **gated on D2/D3 tests landing**, exactly as the ADR self-specifies; until then it stays Proposed. *Why:* no unaccounted proposal — this one is intentionally implementation-gated and the gate is legitimate, so it ratifies-on-evidence rather than dropping. **Pre-condition: renumber off 0377.**
- **governing:** — (not superseded; the issue is the number-collision, not supersession).
- **truth_flag:** PARTIAL — TRUE as a design, but UNPROVEN by its own admission (D2/D3 unimplemented) and WRONG-numbered (duplicate 0377). The git-CAS-on-refs claim is technically credible.
- **in_masterplan:** PARTIAL — the *mechanism* (forge board = projection, git refs = SSOT lock) belongs once ratified; today it is a not-yet-accepted proposal so it cannot bind as canonical.
- **tensions:** (1) **Hard ID collision** with ADR-0377-kafka-to-pulsar (keystone §6.1). (2) Substrate coupling to **Forgejo** sits on the keystone §5 forge fault-line (founder = GitHub; ADR-0510 = bespoke-VCS destination) — and the ADR *explicitly rejects GitHub Projects*, which directly contradicts the founder's GitHub migration directive. (3) Depends on the OPEN masterplan authored-vs-generated question (keystone §4): it treats `/specs/masterplan.json` as the deliverable SSOT that the board projects from — consistent with masterplan-as-authority, but the consolidation design wants ADRs-generate-masterplan; flag under both readings.
- **hyperscaler_challenge:** QUESTIONABLE. Google/AWS/Azure do NOT build task-boards on git-ref CAS + forge labels — they use real work-queue/issue systems (internal Bug/Buganizer-class) with proper schedulers; git-refs-as-claim-locks is a clever bootstrap but not a pattern a hyperscaler would standardize on at scale (no lease/GC story beyond the noted stale-claim risk). Implication: acceptable as a low-dependency dogfood bootstrap, but flag as a not-hyperscaler-grade pattern to revisit; do not enshrine as long-term canon.
- **ai_slop:** No. Honest, conditional, names its own unproven legs and the stale-claim/lease gap. The "labels lag, git ref is authority" reasoning is genuine distributed-systems thinking.
- **refinement:** (1) **Renumber** off 0377 (e.g. to the next free id) before anything else. (2) Reconcile the Forgejo + reject-GitHub-Projects stance against the GitHub-migration directive. (3) Land D2/D3 tests to lift the conditional.
- **consensus_needed:** *Which ADR keeps the number 0377 — and does a git-ref-CAS + Forgejo-label board survive the founder's GitHub directive, or should the board project onto GitHub Issues/Projects instead (reversing this ADR's central rejection)?*

### ADR-0377 (kafka-to-pulsar) — Migrate Kafka to Pulsar via KoP wire-compat

- **decision_atom:** Standalone **Kafka is retired**; the cluster runs **Pulsar 4.x + Oxia** as the sole canonical event-bus, with a **KoP (Kafka-on-Pulsar) wire-compat proxy** fronting existing Kafka clients (zero Phase-1 code change) over a 3-phase migration; the ADR-0005 streaming-semantics (transactional outbox, at-least-once, consumer-group fanout) carry forward, and ADR-0005's Kafka-substrate clause is superseded-in-part.
- **domain:** data-engine-db (eventing/streaming).
- **current_status:** `Accepted` (2026-05-28; `supersedes: [ADR-0005]`, `superseded_by: []`).
- **disposition:** KEEP — this IS the canonical eventing posture per keystone §3/§2 (Kafka→Pulsar+Oxia, KoP wire-compat; supersedes ADR-0005). Confirmed governing ADR on the keystone supersession graph.
- **proposed_resolution:** NA (Accepted).
- **governing:** — (it is the governing/superseding ADR over ADR-0005).
- **truth_flag:** PARTIAL → mostly TRUE with a **dangling-reference defect**: it leans on **ADR-0397** ("Pulsar 4.x + Oxia canonical event-bus, this session") and **ADR-0436** (RisingWave consumer) as related/confirming authorities, but **neither ADR-0397 nor ADR-0436 exists on disk in `decisions/`** (verified). The keystone map also cites ADR-0397 as governing eventing — so this is a corpus-wide dangling citation, not just local. ADR-0195 *does* exist (confirmed) but on disk is titled "stream-processing-tier," whereas this ADR describes 0195 as "introduced Pulsar/KoP log-broker substrate" — a possible title/scope drift to verify. Net: the *decision* is TRUE and on-vocabulary; the *citation graph* is STALE/partly fabricated-forward.
- **in_masterplan:** YES — canonical Eventing posture (Pulsar+Oxia, outbox-pattern retained, Kafka retired).
- **tensions:** (1) **ID collision** with ADR-0377-forgejo-board (keystone §6.1) — same number, different domain; one renumbers. This Accepted+supersedes-0005 ADR has the stronger claim to keep 0377. (2) Dangling deps on missing ADR-0397/0436 (the canonical-confirmation it rests on is unverifiable on disk). (3) `decisions.json next_adr` staleness (keystone §6.3) likely contributed to the duplicate 0377 allocation.
- **hyperscaler_challenge:** ALIGNED. The Hyperscaler-Lens pre-check table is built into the ADR (active upstream, Apache-2.0, self-hostable, Yahoo-origin/FAANG-adjacent internal use) — consolidating two streaming substrates into one (Pulsar) with a wire-compat bridge is exactly the migration discipline AWS/Google run (MSK→in-house, Kafka→Pub/Sub-class). KoP-as-bridge-not-destination + phased cutover with offset-parity validation is textbook. No amend/archive implication on the decision; only fix the citations.
- **ai_slop:** No. Concrete 3-phase plan, named tools (MirrorMaker 2 / Pulsar migration utility), quantified ~5% KoP overhead + mitigation, real research-basis path. The only slop-risk is citing not-yet-existent ADR-0397/0436 as if authoritative — a forward-citation hygiene miss, not fabricated reasoning.
- **refinement:** (1) Verify/author the missing ADR-0397 (Pulsar+Oxia canonical) and ADR-0436 (RisingWave) or repoint the citations — currently the keystone's canonical eventing authority (ADR-0397) is a dangling reference corpus-wide. (2) Confirm ADR-0195's actual title/scope vs the "introduced KoP" description. (3) Resolve the 0377 number collision in this ADR's favor.
- **consensus_needed:** *Does the canonical Pulsar+Oxia authority (ADR-0397) actually exist, or must it be authored before the Kafka-retirement masterplan claim can bind? (a dangling-SSOT-reference question, not a design dispute).*

---

## Chunk notes

**Overall posture:** 4 KEEP-grade Accepted ADRs (0373 cloud-intelligence gateway, 0375 Talos/CAPI/ArgoCD substrate, 0376 managed-k8s product, 0377-kafka-to-pulsar), 1 clean ARCHIVE (0372 SolidJS, superseded by 0393), and 2 AMEND-grade items (0374 Jenkins-orchestrator-stale, 0377-forgejo-board conditional+number-collision). No GARBAGE; no ai-slop in the chunk — these are dense but honestly-cited founder/council decisions with strong honest-claims discipline (typed `Unimplemented` boundaries, manifest non-claims, sourced best-practice briefs).

**Three structural defects to escalate:**
1. **ADR-0377 is a genuine duplicate number** (keystone §6.1): one `Accepted` (kafka-to-pulsar, supersedes 0005) and one `Proposed (conditional)` (forgejo-board). The Accepted/supersedes-0005 one should keep 0377; the conditional board ADR should renumber. Do NOT index by flat `ADR-0377` until resolved.
2. **Dangling forward-citations:** the Kafka ADR-0377 rests on **ADR-0397 (Pulsar+Oxia canonical)** and **ADR-0436 (RisingWave)** as confirming authorities — *neither exists on disk in `decisions/`* (verified). The keystone map itself cites ADR-0397 as the governing eventing ADR, so this is a corpus-wide dangling-SSOT reference, not a local typo. ADR-0372→ADR-0393 supersession, by contrast, IS verified on disk.
3. **CI-orchestrator drift:** ADR-0374's "RESOLVED: Jenkins-as-orchestrator" is overtaken by the keystone CI churn chain (ADR-0511 Argo Workflows = destination, Jenkins transitory; ADR-0513 oya-ci). The receiver/HMAC/dispatch-port survives; the orchestrator target must be re-pointed Jenkins→Argo/oya-ci. The ADR's trait-agnostic design pre-authorized this exact re-point.

**Cross-side (LINUX↔SOURCE) fault-lines touched by this chunk:**
- **Node-OS ownership (sharpest):** ADR-0375 adopts *actual Talos*; LINUX ADR-0025 wants a *Rust "Talos"* and ADR-0018 a framekernel-as-host — own-vs-adopt the node-OS. Surface, do not resolve.
- **Forge substrate (three-way):** ADR-0374, ADR-0375-D4, and ADR-0377-forgejo-board all hardwire **Forgejo** (and 0377-board explicitly *rejects GitHub Projects*), directly contra the founder's **GitHub** `jason931225/oyatie` migration directive and ADR-0510's bespoke-VCS destination. Every Forgejo-coupled decision in this chunk inherits this contested substrate.

**Masterplan authored-vs-generated (keystone §4) — both readings flagged:** ADR-0377-forgejo-board treats `/specs/masterplan.json` deliverables as the SSOT the board projects from (masterplan-as-authority reading). Under the opposite consolidation reading (ADRs-generate-masterplan), the board would project from generated-deliverables derived from ADR front-matter. The decision is consistent with the founder's stated GOAL (masterplan-as-SSOT) but contradicts the ADR-immutability/generated-from-ADRs design — DO NOT assume; flag.

**Vocab hygiene in-chunk:** Clean. ADR-0373 uses **cloud-intelligence** (not foundry); ADR-0374 explicitly notes "foundry name eradicated repo-wide (ADR-0362)"; ADR-0377-kafka uses **Pulsar** (retiring Kafka per retired-vocab). One watch-item: ADR-0376's "two-tier / premium tier" product language sits adjacent to the retired **tier-system** vocab (ADR-0329) — it is a legitimately different axis (product SKU, not tenant-class), but recommend masterplan projection say "SKU/plan" to avoid retired-`tier` lint leakage.
