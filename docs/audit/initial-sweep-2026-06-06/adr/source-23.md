# SOURCE ADR Audit — Chunk 23

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 23
- **Slice range (ls rows 155–161):** ADR-0180 → ADR-0186
- **ADRs actually reviewed (7):** ADR-0180, ADR-0181, ADR-0182, ADR-0183, ADR-0184, ADR-0185, ADR-0186
- **Auditor posture:** keystone map is binding baseline; trust the *superseding* ADR over stale front-matter; treat `foundry`/`Redis`/`Kafka`/`Jenkins-as-destination`/`tier-system` as retired vocabulary.

This is a tightly-coupled "hyperscaler layering" cluster — six of seven ADRs share the date 2026-05-18, the same `council-architecture` owner, the same `/specs/hyperscaler-architecture-invariants.json` spec, and a near-identical "zero overlap / each X owns exactly one concern + in-house roadmap table" template. ADR-0182/0183/0184/0185/0186 explicitly cross-reference each other as a contiguous architectural set.

---

### ADR-0180 — SLO composition + inheritance arithmetic

- **decision_atom:** Every parent product declares one `slos/composition.openslo.yaml` whose composition rule (serial / parallel / critical-path) is gate-validated so an impossible parent SLO (parent target exceeding its blocking children's composed budget) blocks promotion and feeds Flagger auto-rollback at the parent's promotion frontier.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP (with a minor AMEND nudge — see ai_slop/refinement).
- **governing:** n/a (not superseded).
- **truth_flag:** TRUE. Composition arithmetic is genuine SRE practice; the math (serial=product, parallel=1−∏(1−a), critical-path=min) is correct.
- **in_masterplan:** PARTIAL. Carries `related_specs` + status front-matter but no `masterplan_ref`/`planning_impact` binding (consistent with the corpus-wide 8.8% binding gap noted in the map §4). The decision atom is masterplan-ready as backfill.
- **tensions:**
  - References **ADR-0042** (observability OTel + in-house UI) which is **`superseded` by ADR-0383** (Loki/Tempo/Mimir/Grafana). The "+ in-house UI" framing of 0042 is dead; 0180's reference is to a retired observability posture. Reconcile the ref to ADR-0383/ADR-0186.
  - References **ADR-0121 portability invariant** (alt D rejection). ADR-0121 (onprem kubeadm/containerd/istio) is **`Superseded` by ADR-0375** (Talos+CAPI+ArgoCD). The *portability invariant* principle survives the supersession, but the citation is to a retired ADR.
  - Names **"Foundry"** as an example parent product (Decision §, "Workflow Studio, Foundry, Super-App") — **retired brand** per ADR-0335/0347 (foundry → cloud-intelligence + governance). Retired-vocab leakage.
  - `oya-check-slo-composition-feasibility` is an ADR-number-free semantic lane name — *good* (aligns with planning-ssot canonical-naming, not the forbidden `adr-0145-*` antipattern).
- **hyperscaler_challenge:** ALIGNED. Google SRE Workbook "Composing SLIs / Embedded SLO Hierarchy" is exactly this; AWS Service Quotas + Datadog SLO composition are real precedents. Google/AWS would absolutely make this decision. Argues for KEEP.
- **ai_slop:** Minor. The "Stripe internal SLO orchestrator" precedent is unverifiable fabricated-precision (Stripe's internal SLO tooling is not public); soften to "public eng posts describe per-API→product rollup." Otherwise clean.
- **refinement:** (1) Repoint ADR-0042 ref → ADR-0383/ADR-0186. (2) Drop "Foundry" example product → "cloud-intelligence." (3) Add `masterplan_ref` front-matter. (4) Note the `SLOComposition` kind is a local OpenSLO extension that may rename when upstream `Composite` lands — already honestly flagged in Negative §3, keep.
- **consensus_needed:** no.

---

### ADR-0181 — Container image promotion pipeline (dev→staging→prod, cosign-signed)

- **decision_atom:** Container images promote dev→staging→prod by byte-identical copy (no rebuild) with a distinct per-tier Sigstore/Cosign keyless (Fulcio OIDC) signature at each hop, and each cluster's admission policy refuses pods whose image lacks the cluster-tier signature, with promotion gated on soak (24h staging / 6h prod-canary) and emitting an audit-chain seal per hop.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND (sound core, stale CI/forge/brand references throughout).
- **governing:** n/a for the decision; but its enforcement plumbing references are stale (see tensions).
- **truth_flag:** PARTIAL. The promotion-ladder decision is TRUE and hyperscaler-correct; the implementation references are STALE (GitHub Actions / Foundry-pipeline / Kyverno-as-admission / ADR-0112 webhook).
- **in_masterplan:** PARTIAL. Front-matter has `related_specs` incl. `/specs/gitops-vcs-replacement.json` but no masterplan binding.
- **tensions:**
  - **Title says "tier promotion"** and body uses "three-tier" / "tier-bound" pervasively. This is *deployment-environment* tier (dev/staging/prod), NOT the retired tenant "tier-system" (ADR-0329 → tenant-class). No conflict in substance, but the word "tier" is a loaded retired term corpus-wide; a reviewer skim could misread. Worth a clarifying note.
  - **CI/forge churn:** "OIDC identity is the GitHub Actions or Foundry-pipeline tier-bound workflow" (§Decision 2) + "The Foundry pipeline's webhook-driven µservice promotion (per ADR-0112)" (Operational 6) + `.github/workflows/image-promote.yaml.template`. Per map §1.3/§2: **GitHub Actions retired as CI** (→ Argo Workflows, ADR-0511), **Foundry brand retired** (→ governance/intelligence, ADR-0335/0347), and **ADR-0112 is `Superseded` by ADR-0363** (retire agentic-VCS). Three retired references in the enforcement path.
  - **Admission engine drift:** §Decision 4 + Operational 3 wire enforcement to **Kyverno** "per ADR-0117 Kyverno consolidation." Peer ADR-0183 (this same cluster) makes Kyverno the admission engine, but ADR-0183 is now **`Superseded` by ADR-0379 (Kubewarden default admission)**. So 0181's Cosign-verifier-admission should now bind to Kubewarden, not Kyverno. Stale-by-association.
  - References **ADR-0145 Invariant 1 audit-chain seal** — live, good.
- **hyperscaler_challenge:** ALIGNED. AWS ECR cross-account byte-copy + Cosign, GCP Binary Authorization + Attestors, GKE refuse-unsigned are exactly this; keyless Fulcio over long-lived keys is current best practice. Google/AWS would make this. Argues KEEP-the-decision / AMEND-the-refs.
- **ai_slop:** Low. "Stripe internal canary-image promotion via signed-tag swap" is again unverifiable fabricated-precision. The 30h dev→prod minimum + expedited path is concrete and reasonable, not slop.
- **refinement:** (1) Repoint CI identity GitHub-Actions/Foundry → Argo Workflows / oya-ci (ADR-0511/0513). (2) Repoint admission Kyverno → Kubewarden (ADR-0379). (3) Drop ADR-0112 ref → ADR-0363 plain-git/Forgejo flow. (4) Optionally rename "tier" → "promotion-stage" to dodge retired-vocab collision. (5) Add masterplan binding.
- **consensus_needed:** no (refs are mechanical fixes; the decision stands).

---

### ADR-0182 — API Gateway (north-south) vs Service Mesh (east-west) separation; zero overlap

- **decision_atom:** North-south public ingress is owned solely by Envoy Gateway 1.8.0 (Gateway API v1.0; TLS/WAF-via-Coraza/OIDC/public-rate-limit) and east-west is owned solely by Cilium+Istio-Ambient (ADR-0148, mTLS/AuthorizationPolicy), with zero feature overlap and the gateway re-originating mTLS to the mesh.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND (decision is sound and current; contains a WRONG cross-reference + retired-vocab + a stale-by-association policy ref).
- **governing:** n/a (not superseded).
- **truth_flag:** PARTIAL — decision TRUE; one cross-ref is WRONG (see below).
- **in_masterplan:** PARTIAL. Good `related` graph; no masterplan binding.
- **tensions:**
  - **WRONG cross-ref (verified on disk):** §Decision/References cite **"ADR-0150 — policy engine separation (Cedar app authz vs Kyverno admission)."** On disk `ADR-0150` is **`ADR-0150-cursor-pagination-canonical.md`** — the policy-engine-separation ADR is actually **ADR-0183** (this very slice). This is a plain broken reference; 0182 means 0183. (The keystone map §1.3 policy chain also lists "ADR-0150 (Cedar engine)" — appears to inherit the same misnumber; flag for the synthesizer.)
  - **Stale-by-association:** §Decision 3/4 route east-west authz to Cedar `ext_authz` and admission to Kyverno (per ADR-0148/0183). ADR-0183 is now `Superseded` by **ADR-0379** (Kubewarden admission). Cedar-at-waypoint survives (ADR-0243/0246/0379 retain it); only the Kyverno-admission half is superseded.
  - **Retired vocab:** L49 "Redis-cluster-backed counters per ADR-0184 storage tier 3 — note: Valkey 8.1 since Redis 7.4+ relicensed." The ADR self-corrects inline (good), but the primary clause still says "Redis-cluster-backed." Per ADR-0336/0184, **Redis is retired → Valkey**; the residual "Redis" phrasing is leakage.
  - **`oyatie/tier` label / "sovereign-tier"** language inherited via ADR-0131 label set — see 0183 note; "tier" here is the deployment/runtime label axis, not the retired tenant tier-system. Not a conflict but a vocab-collision watch item.
  - In-house table references "cell-µservice's per-tenant rate-limit counters" — **cell as a microservice is retired** (ADR-0333 → cell is a deployment *pattern* only). Leakage.
- **hyperscaler_challenge:** ALIGNED. The crisp north-south/east-west split (edge gateway vs internal mesh) is exactly the Google/AWS/Azure posture (Cloud Load Balancing/ALB at edge, mesh internal); Envoy Gateway + Istio Ambient is vendor-neutral and is what App Mesh/Anthos run under the hood. Google/AWS would make this. The honest "two control planes" tradeoff is acknowledged. Argues KEEP-decision / AMEND-refs.
- **ai_slop:** Low. Thorough alternatives table (a–g) is justified for a load-bearing infra choice, not padding. The in-house "Why no in-house gateway: Envoy is what App Mesh/Anthos/Cloudflare use" is sound reasoning, not slop.
- **refinement:** (1) Fix ADR-0150 ref → ADR-0183. (2) Add "(now Kubewarden admission per ADR-0379)" beside the Kyverno mention. (3) Replace "Redis-cluster-backed" → "Valkey-cluster-backed." (4) Replace "cell-µservice" → "cell-pattern deployment / governance µservice." (5) Add masterplan binding.
- **consensus_needed:** no.

---

### ADR-0183 — Kubernetes policy engine separation: Cedar (app authz) vs Kyverno (admission)

- **decision_atom:** Cedar (app-layer principal×action×resource authz at the Istio waypoint ext_authz) and Kyverno (K8s admission: PSS-restricted, Cosign image verification, label discipline, mutating sidecar/SPIFFE injection) each own exactly one concern with zero overlap — **now superseded on the admission half by ADR-0379 (Kubewarden default admission)** while the Cedar app-authz separation principle is retained.
- **current_status:** Superseded (`superseded_by: [ADR-0379]`) — front-matter VERIFIED correct on disk; ADR-0379 carries `supersedes: [ADR-0183]`, status Accepted.
- **disposition:** ARCHIVE (superseded). Retain the Cedar/admission *separation-of-concerns principle* as the surviving doctrine; the Kyverno-specific admission engine choice is replaced.
- **governing:** **ADR-0379** (Kubewarden default admission substrate) governs the admission half. Cedar app-authz survives via ADR-0243/0246/0379. Map §1.1 explicitly: "archive (Cedar split principle survives)."
- **truth_flag:** STALE (the chosen admission engine Kyverno is no longer canonical; the separation thesis remains TRUE).
- **in_masterplan:** NA-ish / PARTIAL — as a Superseded ADR it should be archived/frozen rather than backfilled; only the surviving principle (Cedar app-authz vs admission-engine separation) is masterplan material, under whichever authority model wins (§4 open question).
- **tensions:**
  - This ADR is the *correct* target of ADR-0182's broken "ADR-0150" reference (see 0182). Downstream ADRs (0181 admission wiring, 0182 §3/4, 0186) that lean on "Kyverno admission per ADR-0183" inherit the supersession and should repoint to ADR-0379.
  - The 8 canonical Kyverno ClusterPolicies (PSS-restricted, image-signature, registry-allowlist, cedar-fragment-annotation, ambient-label-mutation, SPIFFE-binding, runtimeclass-tier-enforcement) are real, good content — but as Kyverno CRs they need a Kubewarden-policy translation under ADR-0379. The *policy intents* survive; the engine encoding changes.
  - `runtimeclass-tier-enforcement.yaml` "sovereign-tier namespaces must declare runtimeClassName: kata-clh-sev-snp per ADR-0147" — ADR-0147 runtime ladder is live canon (map §3). "sovereign-tier" = runtime/isolation tier, not retired tenant-tier. OK.
- **hyperscaler_challenge:** ALIGNED on the *principle* (separate app-authz PDP from cluster admission), QUESTIONABLE on the *engine*. Google/AWS/Azure do separate app authz (Cedar/AVP, IAM) from admission (Gatekeeper/Policy Controller, Azure Policy). The supersession to Kubewarden is itself defensible (Rust/WASM policy, founder Rust-primary bias). Argues ARCHIVE (already done) and confirms the supersession is hyperscaler-reasonable.
- **ai_slop:** Low. Strong, specific alternatives analysis (OPA Gatekeeper perf, Cedar analyzability set-difference proofs). "Cedar used by Amazon Verified Permissions, Confluent, Pinterest" — AVP is true; Confluent/Pinterest Cedar usage is plausible-but-unverified (mild fabricated-precision).
- **refinement:** (1) Add a one-line banner pointing readers to ADR-0379 (front-matter already does; a body Status-section note would help skimmers). (2) Ensure the surviving "separation principle" is captured in masterplan independent of the archived ADR. (3) Migrate the 8 ClusterPolicies' intents into ADR-0379's Kubewarden catalog.
- **consensus_needed:** no (supersession already ruled by ADR-0379). The only open meta-question is the §4 masterplan authored-vs-generated handling of *superseded* ADRs — covered globally, not per-ADR here.

---

### ADR-0184 — Storage tier layering: OLTP / read-replica / cache / search

- **decision_atom:** A four-tier storage layering where each tier owns one access pattern — Tier1 Postgres 18.4 primary (Citus 14 optional, Patroni HA, pgcat pooling) = source-of-truth; Tier2 Postgres streaming read replicas (CQRS); Tier3 Valkey 8.1 cluster (cache/sessions/rate-limit counters); Tier4 Meilisearch 1.9 (non-source-of-truth full-text/faceted search, rebuildable) — with strictly upward composition and no cross-boundary reach.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP (current, correct, retired-vocab-aware — actively does the Redis→Valkey retirement correctly).
- **governing:** n/a.
- **truth_flag:** TRUE. Notably this ADR is the *correct* implementation of map §2's Redis→Valkey retirement (ADR-0336) and uses permissive-license discipline aligned with ADR-0013/0211/0345.
- **in_masterplan:** PARTIAL. Good `related_specs`; no masterplan binding. Strong backfill candidate — it's the canonical data-tier composition decision.
- **tensions:**
  - **Cross-side fault-line (map §5.1):** LINUX **ADR-0001** wants a from-scratch Rust multi-model engine that *eliminates the PostgreSQL/sqlx dependency*; this ADR-0184 is the sharpest SOURCE counter-posture (Postgres 18.4 as canonical Tier-1 source-of-truth). Direct own-the-substrate vs assemble-proven-OSS tension. **Surface, do not resolve.**
  - **Adjacent to map §3 best-of-breed data posture** (Milvus/SeaweedFS/ClickHouse/TimescaleDB/Postgres+pgcat via ADR-0192/0196/0193/0194/0179). 0184 covers OLTP/cache/search but is silent on vector (Milvus, ADR-0192) and object (SeaweedFS/Ceph, ADR-0196) and OLAP (ClickHouse, ADR-0193) — it's a *partial* storage map presented with "four-tier" finality. Mild scope-overclaim; should cross-ref the vector/object/OLAP ADRs so the "tiers" aren't read as the *entire* storage strategy.
  - **Meilisearch BUSL-drift flag** (in-house table) is well-reasoned and consistent with the founder's OSI-strict + own-when-proven ratchet (ADR-0211/0345). Tantivy Phase-2 plan aligns with map §5.5 "own when proven." No conflict — exemplary.
  - References **ADR-0183** (Superseded) only for context ("policy engine separation"), not load-bearing — low impact, but repoint to ADR-0379/0243.
- **hyperscaler_challenge:** ALIGNED. RDS+read-replica+ElastiCache(Valkey)+OpenSearch / Cloud SQL+Memorystore(Valkey)+Vertex Search is exactly the named precedent. Valkey-over-Redis is precisely what AWS/GCP/Oracle did post-2024-relicense. Google/AWS would make this. Argues KEEP.
- **ai_slop:** Very low. Versions are specific and internally consistent; the "Stripe: Postgres+replicas+Redis+ES (pre-license-change shape)" caveat is honest. One nit: PostgreSQL release-notes URL in References (L190) is a copy-paste of an unrelated multi-version-release link (says "183/179/1613/1517/1422") that doesn't match "18.4" — minor citation hygiene.
- **refinement:** (1) Add explicit cross-refs to ADR-0192 (vector/Milvus), ADR-0196 (object/SeaweedFS+Ceph), ADR-0193 (OLAP/ClickHouse), ADR-0194 (TimescaleDB) so "four-tier" reads as the OLTP/cache/search slice, not the whole storage strategy. (2) Repoint ADR-0183 ref → ADR-0379/0243. (3) Fix the Postgres release-notes URL. (4) Add masterplan binding. (5) Surface the §5.1 LINUX-ADR-0001 tension explicitly for founder.
- **consensus_needed:** YES (load-bearing, cross-side). Question: **"Is Postgres 18.4 the canonical Tier-1 source-of-truth (SOURCE ADR-0184), or does the LINUX from-scratch Rust multi-model engine (ADR-0001) that eliminates Postgres become the substrate direction? These are mutually exclusive at Tier-1."**

---

### ADR-0185 — Workflow Studio client stack: per-surface native; OpenAPI contract is the unifier

- **decision_atom:** Each client surface uses its idiomatic native stack (Web: SvelteKit→Leptos sequential; Apple: Swift/SwiftUI only; Android: Kotlin/Compose; Windows: WinUI3/.NET10; Linux: GTK4/gtk-rs/libadwaita) with NO shared cross-ecosystem UI or business-logic layer — the sole cross-ecosystem unifier is OpenAPI 3.2.0 contract-first codegen, plus Style-Dictionary design tokens.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP (well-formed; minor retired-brand leakage + a version-feasibility watch).
- **governing:** n/a.
- **truth_flag:** TRUE (with PARTIAL on forward-dated tooling versions — see ai_slop).
- **in_masterplan:** PARTIAL. Has `related_specs` (manifest-schema only); no masterplan binding. Backfillable as the canonical client-stack decision.
- **tensions:**
  - **Retired-brand leakage:** body refers to "n8n-class first-hero product" and uses "Foundry" indirectly through the product ecosystem; more concretely it inherits the in-house-stack directive cluster but does not leak "foundry" by name — clean on that axis. The "Phase 2 / Phase 0" trigger language overlaps the retired **M0–M3 / Milestone / Wave** vocabulary concept (map §2) — but uses neutral "Phase" terms, which is fine.
  - **Rust-primary endgame** (Leptos Phase 2 + GTK sharing `oya-client-shared-rust`) is consistent with the founder's own-the-controlled-ecosystem posture and the map §5.5 ratchet. Aligned, not in tension.
  - **Internal consistency with ADR-0186 ref:** Operational §1 cites `oya-check-client-stack-discipline` "per ADR-0186 wiring" — 0186 is the observability backplane, an odd home for a *client-stack* fitness gate. Mild mis-wire (the gate is a governance/fitness lane, not observability). Low severity.
  - **ADR-0120 "backend is Rust per ADR-0120"** — ADR-0120 (rust-first onprem tooling) is **`Superseded` by ADR-0375** (Talos+CAPI+ArgoCD) per map §1.1. The *Rust-first* spirit survives but the citation is to a superseded ADR; the "backend is Rust" claim is better grounded in the broader Rust-primary doctrine than in 0120.
- **hyperscaler_challenge:** QUESTIONABLE. The per-surface-native + contract-unifier pattern is what Apple/Google/Microsoft do for *their own* platform SDKs — but **no single hyperscaler maintains five fully-native first-party UI stacks for one product**; they pick 1–2 platforms or accept a cross-platform layer (Google: Flutter exists *because* maintaining N native stacks is costly). Five native stacks for one Workflow-Studio is a startup-scale resourcing bet that AWS/Google would likely NOT make for a single product. Argues for a documented re-scoping checkpoint (which platforms are truly day-0), not archive.
- **ai_slop:** Moderate fabricated-precision risk. Pervasive forward-dated, hyper-specific versions (SvelteKit 2.55, Svelte 5.55, Vite 8.0/Rolldown, TypeScript 6.0/7.0-Go-beta, Leptos 1.0 ETA mid-2026, Swift 6.3, Kotlin 2.3, .NET 10 LTS, WinUI/Windows App SDK 1.8, gtk4-rs 0.11.3, GNOME 49/50) presented with release-month confidence as of 2026-05-18. Several are plausibly real for the date but the aggregate precision reads as over-specified; the LTS-rotation caveat (ADR-0098) partly mitigates. The exhaustive a–n alternatives list is somewhat padded (libcosmic/Slint/Iced rejections are reasonable but verbose).
- **refinement:** (1) Repoint "ADR-0120" → Rust-primary doctrine / current orchestration ADR-0375. (2) Move the `oya-check-client-stack-discipline` gate citation off ADR-0186 (observability) to the fitness/governance ADR. (3) Add an explicit "day-0 vs deferred surfaces" scoping table — which of the 5 stacks ship first vs are aspirational — to answer the resourcing challenge. (4) Trim alternatives to the load-bearing few. (5) Add masterplan binding.
- **consensus_needed:** YES. Question: **"Does oyatie commit to five fully-native first-party client stacks (Web/Apple/Android/Windows/Linux) day-0, or pick a 1–2 platform day-0 set with the rest deferred — given no hyperscaler maintains five native UI stacks per single product?"**

---

### ADR-0186 — Observability backplane layering: collection / storage / query / alert / SLO authoring; zero overlap

- **decision_atom:** A five-stage observability backplane — Stage1 OpenTelemetry Collector (agent DaemonSet + gateway, OTLP single ingest); Stage2 specialized stores (Prometheus hot + Mimir long + Loki logs + Tempo traces + Pyroscope profiles); Stage3 Grafana single pane; Stage4 AlertManager→PagerDuty+Opsgenie webhooks; Stage5 OpenSLO→sloth→Prometheus burn-rate rules — each stage owned by exactly one component, with second-tier federated self-monitoring.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP (current, aligns with the canonical observability posture; one WRONG cross-ref to fix).
- **governing:** n/a (this ADR *is* the live LGTM posture, consistent with ADR-0383 which superseded ADR-0042).
- **truth_flag:** TRUE (decision); one cross-ref is WRONG (see below).
- **in_masterplan:** PARTIAL. Good `related_specs`; no masterplan binding. This is the canonical observability backplane decision — strong backfill candidate; should be reconciled with ADR-0383 (which the map names as the governing observability ADR) so the two don't compete.
- **tensions:**
  - **WRONG cross-ref (verified on disk):** Stage 5 + References cite **"ADR-0130 — agentic SLO-gated promotion (OpenSLO mandatory authoring)"** and "per ADR-0130 + ADR-0180." On disk `ADR-0130` is **`ADR-0130-deprecate-knowledge-graph-registry-file-migrate-to-ontology.md`** — the *agentic SLO-gated promotion* ADR is **ADR-0139** (which 0186 *also* cites correctly elsewhere as "4-window burn-rate"). So 0186 conflates 0130↔0139. Plain broken reference.
  - **Relationship to ADR-0383 (map §1.1 governing observability ADR):** ADR-0383 (Loki/Tempo/Mimir/Grafana, AGPL-3 carve-out) is what *supersedes ADR-0042* and is named canonical in map §3. ADR-0186 describes the *same LGTM stack* in finer (5-stage) detail but does not reference ADR-0383 at all (it references the older ADR-0153 "high-level reference"). Potential duplication/competition between 0186 and 0383 — need to establish which is the authority and have the other defer.
  - **Retired vocab:** `oya-governance-slo-coverage` lane name (Operational §2) correctly uses the **post-rename `oya-governance-*`** prefix (not the retired `oya-foundry-fitness-*`) — exemplary, aligns with ADR-0347. Good signal.
  - **License posture:** the AGPL-3 carve-out reasoning for Mimir/Loki/Tempo/Grafana matches map §3 (ADR-0383 AGPL carve-out) and ADR-0013/0345 OSI-strict-with-server-substrate-carveout. Consistent.
  - **PagerDuty/Opsgenie** are the only vendor-coupled (non-OSS SaaS) elements; honestly flagged as webhook-abstracted, vendor-swappable in one config line. Acceptable under the doctrine.
- **hyperscaler_challenge:** ALIGNED. Specialized-store-per-signal + OTLP single ingest + single visualization pane is exactly AWS Managed Prometheus / GCP Managed Service for Prometheus / Azure Monitor posture; LGTM self-hosted is the standard sovereign alternative. Google/AWS would make this. Argues KEEP.
- **ai_slop:** Low. Specific version pins (Prometheus 3.12/LTS3.5, Mimir 3.0, Loki 3.4, Tempo 2.x, Grafana 12.x) carry the same forward-dating caveat as 0185 but are mitigated by the ADR-0098 LTS rotation note. The five-stage framing is genuinely useful, not padding.
- **refinement:** (1) Fix ADR-0130 ref → ADR-0139 (both occurrences). (2) Add ADR-0383 to `related` and explicitly state which of 0186/0383 is authoritative (recommend 0383 = posture decision, 0186 = layered implementation detail deferring to it). (3) Add masterplan binding. (4) Confirm OpenSLO `v1alpha` vs ADR-0180's `openslo/v1` — 0186 says v1alpha, 0180 uses `apiVersion: openslo/v1`; minor internal version inconsistency across the cluster to reconcile.
- **consensus_needed:** no (mechanical ref fixes + a 0186-vs-0383 authority note; not a contested decision).

---

## Chunk notes for synthesis

**1. This is a single authored "hyperscaler-layering" wave, not seven independent decisions.** ADR-0180–0186 (minus the older-numbered refs) all share date 2026-05-18, owner `council-architecture`, the `/specs/hyperscaler-architecture-invariants.json` spec, and a stamped template: *"each X owns exactly one concern / zero overlap"* + an *"In-house roadmap"* KEEP/Phase-2 table + identical "Rollback via git revert + Flux" boilerplate. 0182/0183/0184/0185/0186 cross-cite each other as a contiguous set. Treat them as one architectural sub-corpus when backfilling the masterplan — they define the gateway/policy/storage/client/observability planes respectively. The "in-house roadmap" tables are the clearest expression of the founder's "standard engines + in-house policy/product assets (AWS/Google/Microsoft/Oracle pattern)" doctrine and are high-value masterplan material.

**2. Two PLAIN-WRONG cross-references found (both verified on disk):**
   - **ADR-0182 → "ADR-0150" should be ADR-0183** (ADR-0150 is actually cursor-pagination; the policy-engine-separation ADR is 0183). NOTE: the **keystone map §1.3 itself lists "ADR-0150 (Cedar engine)"** — the map appears to inherit this same misnumber; the actual Cedar-engine/policy-separation lineage runs through ADR-0183/0243/0246/0379, not 0150. Synthesizer should correct the map.
   - **ADR-0186 → "ADR-0130" should be ADR-0139** (ADR-0130 is knowledge-graph-registry deprecation; agentic SLO-gated promotion is 0139, which 0186 also cites correctly elsewhere — internal self-contradiction).

**3. One genuine supersession in the slice:** ADR-0183 is `Superseded` by ADR-0379 (Kubewarden default admission) — front-matter VERIFIED correct both directions. This poisons the *admission-engine* references in ADR-0181 (Kyverno Cosign verifier) and ADR-0182 (§3/4 Kyverno admission), which still cite Kyverno-per-0183. **Cross-chunk action:** every ADR citing "Kyverno admission per ADR-0183/0117" must repoint to ADR-0379. The Cedar *app-authz* half survives untouched.

**4. Retired-vocabulary leakage cluster (map §2):** within this slice — "Foundry" as a product (0180), "GitHub Actions / Foundry-pipeline / ADR-0112 webhook" CI plumbing (0181), "Redis-cluster-backed" + "cell-µservice" (0182). Counter-signal (good): ADR-0184 correctly *executes* the Redis→Valkey retirement, and ADR-0186 correctly uses the post-rename `oya-governance-*` lane prefix. So the wave straddles the 2026-05-21 vocab-transition line: storage/observability ADRs are clean, gateway/promotion/SLO ADRs carry residue.

**5. Cross-side fault-line surfaced (map §5.1):** ADR-0184 (Postgres 18.4 = canonical Tier-1 source-of-truth) is the sharpest SOURCE counter to LINUX ADR-0001 (from-scratch Rust engine that *eliminates Postgres*). This is the single most load-bearing tension in the chunk and needs a founder ruling (consensus question recorded on 0184). ADR-0185's Rust-primary client endgame, by contrast, *agrees* with the LINUX own-the-Rust-ecosystem posture — not all cross-side signals are tensions.

**6. Hyperscaler-challenge outliers:** Five of seven are cleanly ALIGNED (0180, 0181, 0182, 0184, 0186). Two warrant a real challenge: ADR-0185 (five native client stacks per single product — no hyperscaler does this; resourcing/scope bet) and ADR-0183's *engine* choice (already resolved by the Kubewarden supersession). 0185's breadth is a startup-scope-vs-hyperscaler-discipline question worth a founder checkpoint.

**7. Masterplan-binding gap is uniform:** none of the seven carries `masterplan_ref`/`planning_impact` front-matter; all are PARTIAL on in_masterplan, consistent with the corpus-wide 8.8% binding figure (map §4). All seven decision_atoms above are written masterplan-ready. Under the §4 OPEN question (authored-as-SSOT vs generated-from-ADRs), the *Superseded* ADR-0183 is the one case where the two models diverge sharply: generated-from-ADRs would freeze it append-only; masterplan-as-authority would lift only the surviving separation principle. Flag both readings.

**8. Internal version-drift nit across the wave:** OpenSLO is `openslo/v1` in ADR-0180 but `v1alpha` in ADR-0186 — reconcile to one apiVersion when these are backfilled.
