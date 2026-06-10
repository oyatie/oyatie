# ADR Audit — SOURCE chunk 46

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`, `docs/decisions/`)
- **chunk:** 46 (auditor slice 316–322 of the `ls | sort` enumeration)
- **range:** ADR-0378 → ADR-0384 (7 ADRs, contiguous on disk)
- **ADRs reviewed:** ADR-0378, ADR-0379, ADR-0380, ADR-0381, ADR-0382, ADR-0383, ADR-0384
- **auditor lens:** masterplan = GENERATED from the ADR log; ADRs are the immutable SSOT; "not in a live ADR ⇒ not needed." Retired vocab applied: foundry→intelligence/governance, Redis→Valkey, Kafka→Pulsar, Jenkins-as-destination→Argo Workflows (per keystone §1.3 CI churn), tier-system→tenant-class.

---

### ADR-0378 — Canonical local substrate: vfkit + Talos Linux (retire colima)

- **decision_atom:** The single canonical LOCAL developer/CI substrate is vfkit + Talos Linux (immutable, API-driven, upstream Kubernetes + Cilium) with `admin@oya-local` as the default kube-context; colima (Lima/k3s/docker) and Sidero Omni are retired.
- **domain:** node-os (primary), ci-cd-build (secondary — it is the local CI host).
- **current_status:** Accepted (2026-05-27).
- **disposition:** KEEP — current and correct; directly extends the canonical orchestration posture (Talos, ADR-0375) down to the dev box. Self-consistent with keystone §3 orchestration row.
- **proposed_resolution:** NA (not Proposed).
- **governing:** n/a (it is the governing local-substrate ADR; supersedes no ADR, retires the colima *tooling*, not an ADR).
- **truth_flag:** TRUE.
- **in_masterplan:** YES — local-substrate canon is a dev-experience invariant the masterplan should record (dogfood-on-Talos). Couples to FD-001/promotion-flow facts.
- **tensions:** (1) The whole D4 leg names the **Forgejo→Jenkins** CI loop as the seam-retirer; per keystone §1.3 the *destination* CI is Argo Workflows (ADR-0511) and Jenkins is transitory bootstrap — so "Jenkins on Talos" is correct-for-now but already on a retirement glide-path. (2) References ADR-0130 as "observability substrate" but ADR-0130 in the keystone is the knowledge-graph-registry deprecation — likely a stale/mis-typed cross-ref (probably meant ADR-0186/0383). (3) Founder forge directive = GitHub, while this ADR hard-bets on self-hosted Forgejo-on-Talos (keystone fault-line #4).
- **hyperscaler_challenge:** Aligned. Google/AWS/Azure dogfood their own immutable node-OS (COS/Bottlerocket/Mariner) locally rather than a divergent k3s/docker convenience box; "production-fidelity local substrate" is exactly the hyperscaler instinct. No amend implication on the substrate choice itself.
- **ai_slop:** None — concrete, verifiable deliverables with operator-check `verified_by`.
- **refinement:** AMEND-on-merge only: fix the ADR-0130 cross-ref; reframe the D4 "Jenkins" leg as "CI engine (Jenkins transitory → Argo Workflows destination)" to match the live CI-churn truth.
- **consensus_needed:** None for the substrate; the embedded Forgejo/GitHub forge question is a *forge-domain* contest (escalated under ADR-0363/0510), not this ADR's to resolve.

---

### ADR-0379 — Kubewarden as the default Kubernetes admission/policy substrate (supersedes ADR-0183)

- **decision_atom:** Kubewarden (Rust→WASM policy modules) is the DEFAULT Kubernetes admission/policy substrate; Kyverno is retained as a first-class adapter; Cedar remains the universal application-layer authz engine — preserving ADR-0183's Cedar-vs-admission separation while swapping the admission engine.
- **domain:** authz-policy (primary), security-supplychain (secondary — image-signing admission per ADR-0181/0039).
- **current_status:** Accepted (2026-05-27).
- **disposition:** KEEP — this is the governing admission ADR and is the canonical posture per keystone §3 ("Kubewarden default, sup. ADR-0183"). Its supersession of ADR-0183 is correctly recorded in front-matter (`supersedes: [ADR-0183]`).
- **proposed_resolution:** NA.
- **governing:** This ADR governs; it supersedes **ADR-0183** (which should carry `superseded_by: [ADR-0379]` — keystone §1.1 lists ADR-0183 as the archive candidate, Cedar split principle surviving).
- **truth_flag:** TRUE.
- **in_masterplan:** YES — admission engine + the Cedar/admission separation are core security-substrate invariants.
- **tensions:** Mild — cites Istio Ambient `ext_authz` waypoint (ADR-0148) for Cedar wiring; the Cilium+Istio-Ambient mesh choice is itself live (keystone §3), so no conflict. Watch only the ADR-0183 stale-front-matter risk (verify the superseded side records the back-edge).
- **hyperscaler_challenge:** Aligned-to-questionable. The *separation* (app-authz PDP distinct from cluster admission) is textbook hyperscaler. The Rust→WASM-policy bet (Kubewarden) is a smaller-ecosystem choice than OPA/Gatekeeper, which a hyperscaler might default to for talent/tooling depth — but Kubewarden's WASM model is a defensible "we are a WASM-native shop" call. No archive implication; possible future AMEND if Kubewarden upstream momentum stalls vs OPA.
- **ai_slop:** None.
- **refinement:** None needed in-ADR. Cross-doc: ensure ADR-0183's front-matter `superseded_by` back-edge is written (drift-prevention).
- **consensus_needed:** None — clean supersession with retained principle.

---

### ADR-0380 — CI-loop closure on Talos: Jenkins farm re-establishment + Forgejo gating

- **decision_atom:** Re-establish the gated CI loop on the Talos substrate — install Jenkins gating plugins, redesign agent pods for Talos (drop colima-era SeaweedFS sccache + macOS hostPath, clone-on-demand instead), author the gated `oya gate run-all` job posting Forgejo commit-status via the ci-webhook-gateway (ADR-0374), and retire the temporary admin-merge seam.
- **domain:** ci-cd-build (primary), forge-vcs (secondary — Forgejo gating/webhook).
- **current_status:** Accepted (amendment) (2026-05-28).
- **disposition:** AMEND — the *decision atom* (gate every merge on real CI commit-status, retire admin-merge) is KEEP-worthy and TRUE, but the ADR is heavily Jenkins-mechanics-specific and Jenkins is **transitory bootstrap only** per the live CI-churn truth (keystone §1.3; ADR-0511 Argo Workflows = destination, ADR-0513 oya-ci Prow-shaped = target platform). The plugin/JCasC/agent-pod detail is implementation scaffolding around a transitory engine.
- **proposed_resolution:** NA (Accepted). (Note: it spawns ADR-0381 as Proposed — handled below.)
- **governing:** n/a (not superseded). Forward-looking governance over its CI-engine choice belongs to ADR-0511/0513/0514.
- **truth_flag:** PARTIAL — the *goal* (CI-gated merges, no admin override) is TRUE and current; the *Jenkins-farm mechanism* is STALE-by-trajectory (correct at authorship, on the retirement glide-path now). D6's "hyperscaler-grade throughput" is aspirational/unbuilt by its own admission.
- **in_masterplan:** PARTIAL — the masterplan should carry the invariant "merges gate on green CI commit-status (no admin-merge seam)" and the dogfood-Forgejo-webhook flow; it should NOT enshrine Jenkins-specific plugin lists (those are an implementation artifact of a transitory engine).
- **tensions:** (1) Jenkins-as-CI vs Argo-Workflows-as-destination (keystone §1.3) — the sharpest. (2) Depends on ADR-0349 SeaweedFS "restoration" while simultaneously deferring it — honest but means the stated end-state is partly unbuilt. (3) Forgejo-canonical vs founder-GitHub directive (fault-line #4) surfaces in D4/D5 (the amendment even shows the agent cloning from `github.com`, contradicting the Forgejo-dogfood goal — explicitly flagged as "stage-2 hardening: flip clone source to in-cluster Forgejo").
- **hyperscaler_challenge:** Questionable. A hyperscaler would not stand up Jenkins as its merge-gating CI farm in 2026; it would run a k8s-native orchestrator (the repo itself converges on Argo Workflows + bespoke oya-ci). The *amendment's* hyperscaler-lens (drop archived Kaniko, kill macOS hostPath, demand multi-node topology) is sharply hyperscaler-aligned. Archive implication: the Jenkins-mechanics layer should be re-expressed against the destination CI engine on consolidation.
- **ai_slop:** None — the amendment is self-aware and explicitly de-scopes "CI gates merges" from "CI gates at hyperscaler concurrency." Honest deferral language throughout.
- **refinement:** On masterplan generation, collapse ADR-0380 to its durable invariant (gated-merge-on-commit-status + ci-webhook-gateway front door + admin-merge-seam retired) and treat the Jenkins farm as a transitory implementation note superseded-in-trajectory by ADR-0511/0513.
- **consensus_needed:** "Is the local CI gate engine Jenkins (transitory) or do we leapfrog straight to Argo-Workflows/oya-ci on the local Talos box?" — a founder-level sequencing question given the engine is already declared transitory elsewhere.

---

### ADR-0381 — Kaniko → BuildKit migration + multi-node Talos cell topology

- **decision_atom:** Migrate the in-cluster image-build substrate from archived Kaniko to BuildKit (Apache-2, daemonless, SeaweedFS-backed `s3` cache) and replace the single-node Talos VM with a multi-pool cell topology (3 CP / 2 worker / CI-specialty / storage-specialty) enforcing cell boundaries via Cilium + ADR-0083 runtime-tier affinity.
- **domain:** ci-cd-build (BuildKit/image-build), node-os (multi-node Talos topology) — genuinely cross-cutting.
- **current_status:** Proposed (2026-05-28).
- **disposition:** AMEND-then-RATIFY — both decisions are substrate-correct and pass the explicit hyperscaler-lens; Proposed only because implementation IPs are gated on the ADR-0380 amendment landing. The Kaniko→BuildKit half is unambiguously correct (Kaniko is archived).
- **proposed_resolution:** **RATIFY** — Kaniko is genuinely archived/read-only upstream (license/maintenance lens (a) failure is real, not speculative), and the multi-node topology is the prerequisite that lifts ADR-0380 D6's parallelism ceiling. No unaccounted-for proposal; ratify with the BuildKit choice and the documented dial-down topology.
- **governing:** n/a; builds on ADR-0378 (substrate), ADR-0148 (Cilium), ADR-0083 (runtime-tier), ADR-0349 (SeaweedFS).
- **truth_flag:** TRUE (decision-space + recommended choice are sound and evidence-backed). PARTIAL only in the sense that it is not yet implemented.
- **in_masterplan:** YES (BuildKit-not-Kaniko is a durable substrate invariant; "no archived upstreams" is a standing rule). The specific local node-count/vCPU table is a dev-host operational detail, masterplan-PARTIAL.
- **tensions:** Low. Internally consistent with the ADR-0380 amendment that spawned it. Couples SeaweedFS cache (D4) to ADR-0349 restoration, which is itself deferred — so D4's exit-criteria depend on prior unbuilt work (sequencing risk, not contradiction).
- **hyperscaler_challenge:** Strongly aligned. BuildKit is exactly what GCP Cloud Build / GitHub Actions / Earthly use; multi-pool CP/Worker/Specialty IS the GKE/EKS/AKS node-pool model. "No managed-service dependency, no archived projects" is the hyperscaler-as-provider posture. No archive implication — this is the lens working as intended.
- **ai_slop:** None — the hyperscaler-lens (a)-(d) is applied per-choice with named rejected alternatives (Buildah, img, dind, k3s-multinode, bigger-single-node, AWS-managed).
- **refinement:** Promote to Accepted once ADR-0380 amendment lands; nothing to amend substantively.
- **consensus_needed:** None — well-reasoned; the only open knob is the dev-host resource baseline (22 vCPU / 46 GiB), already given a documented dial-down.

---

### ADR-0382 — Bare-metal Talos zero-day bring-up via Sidero Metal

- **decision_atom:** Use Sidero Metal (Talos-native, CAPI-integrated, Apache-2) as Oyatie's bare-metal Talos provisioning substrate — zero-touch PXE/iPXE enrollment → Server discovery → CAPI `Cluster`/`TalosControlPlane`/`MetalMachine` provisioning → one-command `up.sh` from cold hardware to a green control-plane.
- **domain:** node-os (primary; bare-metal provisioning), orchestration-scheduling (secondary — CAPI fleet pattern per ADR-0375).
- **current_status:** Proposed (2026-05-28).
- **disposition:** RATIFY (KEEP-as-Accepted on ratification) — it completes the substrate triangle (ADR-0375 fleet / ADR-0378 local-VM / ADR-0382 bare-metal) with the Talos maintainers' own first-party tool; passes the hyperscaler-lens cleanly.
- **proposed_resolution:** **RATIFY** + one-line why: Sidero is the canonical, first-party, actively-maintained bare-metal path for the already-chosen Talos+CAPI fleet (ADR-0375) — no credible alternative meets "Talos-native + CAPI + zero-day" (Tinkerbell/MAAS/Matchbox lose on Talos-integration; managed bare-metal fails self-host lens).
- **governing:** n/a; layers under ADR-0375 (CAPI fleet) and beside ADR-0378 (local VM sibling).
- **truth_flag:** TRUE (sound decision-space; PARTIAL only as unimplemented and gated on a physical lab machine for the live test).
- **in_masterplan:** YES — bare-metal zero-day provisioning is a real cloud-provider capability the masterplan should record as a substrate invariant (Oyatie provides bare-metal orchestration, does not consume it).
- **tensions:** Very low. Fully consistent with the Talos/CAPI/ArgoCD canon (keystone §3 orchestration row). Only soft risk: it presupposes Oyatie operates physical hardware, which is a strategic premise (own-the-metal) rather than a contradiction.
- **hyperscaler_challenge:** Aligned. Sidero Metal is the OSS analogue of Ironic/Tinkerbell/Nitro-provisioning; a hyperscaler absolutely owns its bare-metal provisioning plane rather than renting it. The "do not consume Equinix/Outposts" rejection is the correct provider posture. No amend implication.
- **ai_slop:** None — alternatives (Tinkerbell, MAAS, manual talosctl, Matchbox, managed bare-metal) each rejected with a specific reason; hyperscaler-lens applied explicitly.
- **refinement:** Promote to Accepted; consider folding ADR-0378/0381/0382 into a single "Talos substrate triad" reference at masterplan-generation time (three Proposed/Accepted node-os ADRs describe one coherent substrate story).
- **consensus_needed:** "Does Oyatie's near-term plan actually include owned bare-metal, or is this a long-horizon capability?" — affects whether ADR-0382 is RATIFY-now or RATIFY-as-deferred-capability.

---

### ADR-0383 — Observability stack reconciliation: keep Loki/Tempo/Mimir/Grafana under AGPL-3 (supersedes ADR-0042)

- **decision_atom:** Loki/Tempo/Mimir/Grafana (the LGTM stack) is the canonical observability storage+visualization layer, permitted under AGPL-3 via the fully-self-hosted-in-oya-cells exception (network clause satisfied, ops-platform-owned lifecycle), resolving the ADR-0042↔ADR-0186 contradiction and retiring ADR-0042's VictoriaMetrics/ClickHouse/Jaeger/in-house-Leptos choices.
- **domain:** observability (primary), compliance-residency (secondary — AGPL-3 license-posture reconciliation).
- **current_status:** Accepted (2026-05-28).
- **disposition:** KEEP — this is the canonical observability posture per keystone §3/§1.1 (ADR-0383 supersedes ADR-0042). The OTel emission contract from ADR-0042 is explicitly carried forward via ADR-0186.
- **proposed_resolution:** NA.
- **governing:** This ADR governs; supersedes **ADR-0042** (front-matter `supersedes: [ADR-0042]` is correctly set; ADR-0042 should carry the `superseded_by` back-edge — keystone §1.1 archive candidate). ADR-0186 is the canonical *architecture*; ADR-0383 is the canonical *license-reconciliation record*.
- **truth_flag:** TRUE.
- **in_masterplan:** YES — observability stack + the AGPL-3 self-hosted carve-out are first-order substrate + license invariants (keystone §3 license-posture row references the carve-out).
- **tensions:** Tension with the OSI-strict / "no AGPL/GPL in product code" posture (keystone §3 license row, ADR-0013/0211/0345) — resolved here *by design* via the self-hosted-substrate carve-out with evidence (ADR-0211 §35). This is the canonical place that tension is adjudicated, so it is a *resolved* tension, not an open one. Watch: the carve-out's durability depends on Grafana Labs not re-relicensing (the 30-day re-classification trigger is the mitigation).
- **hyperscaler_challenge:** Aligned-with-caveat. Every major cloud self-hosts AGPL/SSPL-adjacent OSS internally under the same network-clause analysis, so the *posture* is hyperscaler-standard. A hyperscaler might additionally hedge with an in-house metrics store (cf. ADR-0042's VictoriaMetrics instinct) to avoid LGTM lock-in — but reversing now is "churn without license-risk reduction" per the ADR. No archive implication; the in-house-portal long-horizon (W+24) keeps the hedge alive.
- **ai_slop:** None — three rejected alternatives, explicit gates, accurate AGPL §13 framing.
- **refinement:** None in-ADR. Ensure ADR-0042 records `superseded_by: [ADR-0383]` and `status: Superseded`.
- **consensus_needed:** None — this IS the consensus record that closed the ADR-0042/0186 contradiction.

---

### ADR-0384 — cloud-intelligence gateway Path B redesign: OAuth subscription-pool replacing static API-key pool

- **decision_atom:** Replace the cloud-intelligence gateway's static-API-key pool with an OAuth subscription-pool — a pure kernel `SubscriptionPool<OAuthSubscription>` state machine (round-robin/fill-first, cooldown, quota) plus a rest-layer `SubscriptionStore` doing OpenBao-bound, write-through refresh-token rotation, with per-provider adapters (v1: Anthropic + OpenAI-Codex) — matching the operator's actual subscription-OAuth entitlement model while retaining ADR-0373's API surface, audit chain, per-tenant Cedar isolation, and envelope encryption.
- **domain:** intelligence-ai (primary; cloud-intelligence gateway), identity-authn (secondary — OAuth/refresh-token/OpenBao credential lifecycle).
- **current_status:** Proposed (2026-05-28).
- **disposition:** AMEND-then-ratify-conditionally — the architecture is sound (the embedded adversarial critic verdict is "Non-fatal; design is sound"), but it carries **two BLOCKING open questions** (OQ-2 single-use-refresh-token write-through, OQ-5 unverified credential paths) and a near-fatal TLS-fingerprint risk (F2/OQ-7) whose resolution is a spike, not a decision. Correctly uses retired-vocab-clean naming (`cloud-intelligence`, NOT foundry/agent-gateway in the decision surface).
- **proposed_resolution:** **RATIFY (conditional)** — the *decision* (entitlement model must be OAuth-subscription, not static-key; kernel stays pure; CLIProxyAPI-as-sidecar is the explicit fallback) is correct and should be ratified; but ratification is conditional on the D3 TLS spike (OQ-7) confirming stock `reqwest` works OR the sidecar fallback being accepted. It is NOT a DROP — the static-key model is genuinely the wrong entitlement fit and this is the only ADR proposing the correction.
- **governing:** n/a as governed; it **supersedes the credential model of ADR-0373 only** (not its API surface/observability) — front-matter does not list `supersedes: [ADR-0373]`, which is a gap given the body explicitly says "supersedes its credential model." Flag as front-matter drift to fix.
- **truth_flag:** PARTIAL — the decision is TRUE/correct; several implementation facts are UNVERIFIED by the ADR's own admission (credential paths OQ-5, Gemini endpoint F5, TLS-fingerprint OQ-7), and the body still contains residual internal contradictions the critic caught (`console.anthropic.com` vs `claude.ai/oauth/authorize` — F1) that are "fixed in response text" but not all scrubbed from the D3 deliverable string.
- **in_masterplan:** PARTIAL — the masterplan should carry the invariant "cloud-intelligence gateway uses OAuth-subscription pooling with per-tenant Cedar isolation + OpenBao envelope-encrypted refresh tokens"; the provider-specific OAuth-endpoint/TLS-fingerprint mechanics are volatile implementation detail (provider-TOS-dependent) and should NOT be enshrined.
- **tensions:** (1) Front-matter omits `supersedes: [ADR-0373]` despite body claiming credential-model supersession — drift. (2) Cites ADR-0193 (ClickHouse OLAP) and corrects a prior mis-citation of it as the "Cedar policy basis" — the real Cedar ADR is ADR-0191 (and ADR-0183/0379 separation) — a self-corrected citation tangle that should be cleaned, not left "corrected silently." (3) D6 emits to Valkey Stream + ClickHouse (retired-vocab-clean: Valkey not Redis — good). (4) TOS-compliance is asserted-at-authorship with an explicit ongoing-monitoring caveat — a real strategic/compliance risk (multi-seat subscription OAuth re-use), not a technical bug.
- **hyperscaler_challenge:** Questionable-to-misaligned (strategically). A hyperscaler would not build its production AI gateway on **consumer-subscription OAuth refresh-tokens scraped from CLI credential stores** with a Cloudflare-bot-detection-evading TLS fingerprint (F2/F7) — that is a fragile, TOS-risk, single-operator-entitlement pattern, not a durable B2B substrate. The *gateway architecture* (pure kernel, pooled credentials, Cedar isolation, OpenBao) is hyperscaler-grade; the *credential-source* (personal Claude Pro/ChatGPT Plus/Gemini Advanced subscriptions) is a bootstrap/cost-arbitrage hack a hyperscaler with provider API contracts would not adopt. Amend implication: keep the gateway architecture; flag the subscription-OAuth credential model as an explicitly-bounded bootstrap posture with a planned migration to first-party provider API contracts.
- **ai_slop:** Low/None as slop, but HIGH ceremony — the ADR carries a full embedded critic-findings appendix (F1–F7) with "Response"/"ADR fix" pairs, some of which describe fixes as *to-be-applied* rather than *applied* (e.g., F1's endpoint correction is acknowledged but the D3 deliverable + config example still mix `console.anthropic.com`/`codex.openai.com` with the corrected `api.anthropic.com`/`auth.openai.com`). This is honest-but-unconsolidated: the decision is real, the document is mid-revision.
- **refinement:** Before RATIFY: (a) add `supersedes: [ADR-0373]` (credential-model scope note) to front-matter; (b) consolidate the F1/F5 endpoint corrections INTO the D3 deliverable + config examples rather than leaving them only in the critic-response prose; (c) resolve OQ-2 and OQ-5 (BLOCKING) and the OQ-7 TLS spike before D3; (d) bound the TOS/credential-source risk explicitly.
- **consensus_needed:** "Is the cloud-intelligence gateway's production credential model **consumer-subscription OAuth** (Claude Pro / ChatGPT Plus / Gemini Advanced refresh-tokens) as a lasting design, or a cost-bootstrap to be replaced by first-party provider API contracts at scale — and do we accept the standing TOS-monitoring + Cloudflare-TLS-fingerprint fragility that the consumer-subscription path requires?"

---

## Chunk notes

**Coherence.** This is a tight, internally-consistent **substrate cluster** (six of seven ADRs share `milestone` family M-LOCAL-CI-SUBSTRATE / the Talos substrate triangle; the seventh, ADR-0384, is the cloud-intelligence gateway and the only outlier). ADR-0378→0379→0380→0381→0382 form one narrative: pick the local substrate (0378), set its admission engine (0379), close the CI loop on it (0380), correct the two substrate gaps that surfaced mid-amendment (0381), and extend the same Talos/CAPI pattern to bare metal (0382). The chain is well-cross-referenced and the amendments are unusually self-aware (ADR-0380 explicitly de-scopes its own throughput claims and spawns ADR-0381 rather than bloating itself).

**Dispositions at a glance.** KEEP: 0378, 0379, 0383. RATIFY (Proposed→Accepted): 0381, 0382. AMEND: 0380 (decouple durable invariant from transitory Jenkins mechanics). AMEND-then-RATIFY(conditional): 0384.

**Proposed accounting (no unaccounted proposals).** Three Proposed ADRs in range — 0381 (RATIFY), 0382 (RATIFY), 0384 (RATIFY conditional on TLS-spike + BLOCKING-OQ resolution + supersedes-front-matter fix). None recommended DROP.

**Supersession hygiene.** Two clean supersessions present and front-matter-correct on the superseding side: ADR-0379 `supersedes: [ADR-0183]` and ADR-0383 `supersedes: [ADR-0042]` (both match keystone §1.1). Auditors of ADR-0183 and ADR-0042 must confirm the back-edges (`superseded_by`) are written — keystone §6 flags pervasive back-edge drift. ADR-0384 is the one *missing* supersession edge in this chunk: body supersedes ADR-0373's credential model but front-matter `supersedes: []` — fix on consolidation.

**Retired-vocab leakage check (clean overall).** This chunk is largely retired-vocab-clean: ADR-0384 correctly uses `cloud-intelligence` (not foundry) and Valkey (not Redis); no `tier-system`/`M0-M3`/Kafka/Backstage leakage. The one residual-vocab risk is **Jenkins-as-CI** pervading ADR-0380 (and the D4 leg of ADR-0378): per keystone §1.3 Jenkins is *transitory bootstrap* and Argo Workflows (ADR-0511) is the destination — so the heavy Jenkins-mechanics in 0380 reads as soon-to-be-retired implementation detail, not durable canon. ADR-0378's reference to "ADR-0130 (observability substrate)" is a stale/mis-typed cross-ref (ADR-0130 is the knowledge-graph-registry deprecation in the keystone) and should be corrected.

**Cross-side (LINUX pilot) note.** Domain overlap with the LINUX fault-lines: this chunk's heavy bet on **Talos + Sidero + CAPI** (0378/0382) is the exact SOURCE posture that LINUX ADR-0025 ("a Rust 'Talos', beat-or-parity") and ADR-0018 (framekernel-as-host) deliberately compete with (keystone fault-line #3). These SOURCE ADRs are TRUE-on-the-source-side; the tension is the LINUX own-the-node-OS ambition vs SOURCE's adopt-actual-Talos posture — surface, do not resolve.

**Masterplan-generation guidance.** Durable invariants this chunk contributes to a generated masterplan: (1) canonical local + bare-metal substrate = Talos (vfkit local / Sidero bare-metal), upstream-k8s parity, no colima/k3s divergence; (2) Kubewarden default admission + Cedar app-authz separation; (3) merges gate on green CI commit-status via the ci-webhook-gateway, no admin-merge seam (engine = transitory→Argo Workflows); (4) BuildKit not Kaniko, no archived upstreams; (5) LGTM observability under the self-hosted AGPL-3 carve-out; (6) cloud-intelligence gateway = OAuth-subscription pool with per-tenant Cedar isolation + OpenBao envelope encryption (credential-source posture flagged for founder consensus). Implementation-volatile detail (Jenkins plugin lists, node vCPU tables, provider OAuth endpoints/TLS fingerprints) should be referenced-not-enshrined.

**Founder-level open questions surfaced (consolidated):** (a) local CI engine — Jenkins-transitory vs leapfrog to Argo-Workflows/oya-ci; (b) is owned bare-metal near-term or long-horizon (gates ADR-0382 ratify-now vs deferred); (c) is the cloud-intelligence gateway's consumer-subscription-OAuth credential model a lasting design or a cost-bootstrap, given its standing TOS + Cloudflare-TLS-fingerprint fragility.
