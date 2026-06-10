# Source ADR Audit — Chunk 22

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 22
- **Range (slice 148–154):** ADR-0173 … ADR-0179
- **ADRs actually reviewed:** ADR-0173, ADR-0174, ADR-0175, ADR-0176, ADR-0177, ADR-0178, ADR-0179 (7 of 7)
- **Auditor posture:** READ-ONLY. Trust the superseding ADR over stale front-matter (keystone map §6). Treat `foundry`/`tier`/`Redis`/`Jenkins-as-destination`/`Istio`/`Backstage` as retired vocabulary.

---

### ADR-0173 — Vendor lock-in avoidance and stack ownership

- **decision_atom:** Default posture is OWN-the-stack via permissively-licensed OSS; every external vendor is classified into a three-tier taxonomy (I OWNED / II VENDOR-SEAMED-with-mandatory-phase-out-plan / III FORBIDDEN), and every Tier-II adoption MUST carry a port-in-kernel adapter trait + a second impl + a registry entry, mechanically enforced by a CI gate.
- **current_status:** Accepted (2026-05-18). Legacy markdown-table header — NOT YAML front-matter (no `id:`/`status:`/`supersedes:` keys; uses `| Status | Accepted |` table).
- **disposition:** AMEND. The doctrine (default-deny vendor adoption + seam-and-multi-impl + phase-out registry) is sound and load-bearing, but the ADR is shot through with retired vocabulary and references to superseded ADRs; it needs a reconciliation pass.
- **governing:** N/A for the core doctrine (it is the umbrella). Stale references governed by: ADR-0335 (foundry retired→intelligence), ADR-0363 (retire agentic-VCS; supersedes the ADR-0113 "Foundry VCS substrate" cited as a readiness gate), ADR-0383 (observability supersedes the cited ADR-0042 in-house-UI), ADR-0375 (supersedes ADR-0121 onprem k8s), ADR-0336 (Redis→Valkey — partially acknowledged here as "Tier I with asterisk").
- **truth_flag:** PARTIAL. Core doctrine TRUE; the worked examples and vendor inventory are STALE (foundry brand, Forgejo-as-target, ADR-0113 VCS substrate, ADR-0042 in-house-UI, "ADR-0007 cedar"/"ADR-0045 db-tier"/"ADR-0028 cloud-µsvc"/"ADR-0050 automation-first" reference numbers do not match the canonical ADRs used elsewhere in chunk — possible early-corpus numbering).
- **in_masterplan:** PARTIAL/UNCLEAR. Carries NO YAML front-matter at all, so it cannot bind into masterplan.json via the `masterplan_ref` mechanism (planning-ssot-drift-prevention.md found only 8.8% binding — this is one of the unbound). Under the consolidation design (ADRs-generate-masterplan) it would also fail because `planning_impact`/`supersedes`/`status` keys are absent.
- **tensions:**
  - **Forge keystone (§5.4):** Names GitHub as Tier-II with replacement target "self-hosted Forgejo" gated on "Foundry VCS substrate (ADR-0113) reaches release-pointer parity." Conflicts THREE ways — founder's migration directive is GitHub-as-canonical; source canon (ADR-0363/0510) makes Forgejo *transitory* and bespoke-VCS the destination; and ADR-0113 itself is SUPERSEDED by ADR-0363 (agentic-VCS retired). The cited readiness gate points at a dead ADR.
  - **Retired-brand leakage (§2):** "foundry providers," "foundry-runtime model substrate," `oya-foundry-*` crate paths throughout — all retired by ADR-0335/0347. Founder: "cloud-intelligence is the valid name."
  - **Observability (§1.1):** cites "ADR-0042 observability OTel + in-house UI" as a Tier-I steward reference; ADR-0042 is `superseded` by ADR-0383 (the in-house-UI was dropped for Grafana stack).
  - **Self-consistency vs ADR-0179:** This ADR lists "Redis or KeyDB or Valkey" and "MinIO/Garage/SeaweedFS" as Tier-I; the canonical posture (§3) has narrowed to Valkey and SeaweedFS-primary/Ceph-scale — the menu here is pre-narrowing.
- **hyperscaler_challenge:** ALIGNED (verdict). Google/AWS/Azure all run formal vendor-portability / exit-cost programs (AWS Well-Architected OPS-4, which this ADR cites; Google's "no single-vendor lock-in" CNCF criterion). A default-deny-with-seam posture is exactly what a hyperscaler-grade platform does. The *over-ambition* (own the LLM substrate, own the VCS) is where a real hyperscaler would diverge — they buy frontier models and use managed git — which argues for AMEND (soften "own everything eventually" to "seam everything, own when proven," matching the §5.5 shared ratchet), not archive.
- **ai_slop:** Minor. Compliance section name-drops four standards (AWS WA, CNCF, NIST SP 800-53 SA-12, ISO 27001 A.15) as bullet-point garnish — borderline fabricated-precision but each maps to a real control. The "Stripe pulling off Mongo, Linear standing up its own scheduler" case studies are unsourced anecdote. The six-adapter "worked example today" claims crates exist that the retired-foundry reorg has since renamed.
- **refinement:** (1) Convert to YAML front-matter so it can bind to masterplan. (2) Rewrite all `foundry`→`intelligence` per ADR-0335. (3) Replace the GitHub→Forgejo readiness gate (dead ADR-0113 ref) with the current forge posture or mark it CONTESTED pending founder ruling. (4) Drop ADR-0042 in-house-UI reference; point at ADR-0383. (5) Reconcile the Tier-I OSS menu with the narrowed canonical posture (Valkey, SeaweedFS/Ceph).
- **consensus_needed:** YES. *"Is the vendor-lock-in doctrine's stated end-state 'own the entire stack (LLM substrate, VCS, DB engine)' still the directive, or is the operative rule the weaker 'seam everything now, own only when an internal substrate proves parity'? And does the GitHub-vs-Forgejo-vs-bespoke forge question resolve to GitHub-canonical (founder) for this registry?"*

---

### ADR-0174 — FinOps cost-attribution + chargeback policy

- **decision_atom:** Per-tenant cost attribution is first-class: every cloud resource carries a canonical seven-tag block (`tenant_id`/`cell_id`/`microservice`/`plane`/`environment`/`cost_center`/`sustainability_class`), a pinned chargeback formula converts labelled spend into per-tenant bills, a streaming-MAD anomaly detector pages on-call, and a signed quarterly per-tenant report feeds sovereign regulators.
- **current_status:** Accepted (2026-05-18); enforcement `advisory-until-per-microservice-cost-center-declared`. Clean YAML front-matter.
- **disposition:** AMEND (lightly). Core decision is sound, well-formed, has proper front-matter and enforced_by gate. Needs minor retired-vocab and cross-ref fixes.
- **governing:** No supersession. Stale references: ADR-0020 "foundry multi-provider adapter" / "foundry capability registry cost field" governed by ADR-0335 (foundry→intelligence rename).
- **truth_flag:** TRUE (decision); PARTIAL on vocabulary (foundry capability-cost references are stale brand).
- **in_masterplan:** PARTIAL. Has YAML front-matter with `enforced_by` but NO `masterplan_ref` / `planning_impact` keys — unbound under both masterplan readings. Declares its own canonical spec (`specs/finops-cost-attribution.json`).
- **tensions:**
  - References ADR-0179 by name for "multi-cloud vendor-independence" — correct and live (0179 is canonical per §3). Good cross-ref.
  - "foundry capability registry cost field (ADR-0020)" and "per-invocation_cost(capability) is the foundry capability registry cost field" — retired brand (§2). Should read intelligence.
  - `tenant_allocation_ratio` distinguishes `Dedicated` vs `Shared-*` cells — consistent with ADR-0009 cell architecture; no conflict.
  - D-5 public surface exposes charges via `internal-api.oyatie.com` per ADR-0177 — internally consistent with chunk-mate ADR-0177.
- **hyperscaler_challenge:** ALIGNED. AWS Cost Anomaly Detection, GCP FinOps, Azure Cost Management all do exactly tag-at-provision + streaming anomaly + chargeback. The deliberate *rejection* of provider-native cost APIs (Alt-2) in favor of a vendor-neutral tag schema is the correct multi-cloud call a hyperscaler-grade-but-portable platform makes. No archive pressure.
- **ai_slop:** Low. The reference list cites real FinOps Foundation / Stripe / hyperscaler material. `sustainability_class` / carbon-attribution (C-3) is plausibly scope-creep garnish (KR Carbon Neutrality Act 2050 / EU CSRD) but it is a legitimately tagged dimension, not fabricated.
- **refinement:** (1) `foundry`→`intelligence` in the capability-cost references. (2) Add `masterplan_ref` + `planning_impact` front-matter to bind into planning SSOT. (3) Confirm `cost_center` closed-enum lives in registry as claimed (verify-on-merge).
- **consensus_needed:** NO. Non-contested utility decision; only needs the mechanical brand/front-matter touch-up.

---

### ADR-0175 — Tenant lifecycle workflow

- **decision_atom:** A canonical six-state tenant lifecycle (Pending→Active→Suspended→Migrating→Offboarded→DeletionConfirmed, plus terminal Cancelled) where every transition is a saga (per ADR-0222) and `delete_saga` cannot complete until every data-class-bearing µservice emits its proof-of-erasure receipt (per ADR-0038).
- **current_status:** Accepted (2026-05-18); enforcement `advisory-until-tenancy-microservice-implements`. Clean YAML front-matter.
- **disposition:** KEEP. Current, correct, well-formed, non-conflicting. The cleanest ADR in the chunk.
- **governing:** None — no supersession, no stale refs of consequence.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL. YAML + `enforced_by` present; no `masterplan_ref`/`planning_impact` binding keys. Declares `specs/tenant-lifecycle.json` as canonical machine artifact.
- **tensions:**
  - D-6 ties billing-scope exclusion to ADR-0174 chargeback (Pending/Cancelled/Offboarded/DeletionConfirmed not billable) — internally consistent with chunk-mate ADR-0174. Good.
  - References ADR-0002 (tenant+identity kernel), ADR-0038 (DSR/erasure), ADR-0222 (saga policy), ADR-0009 (cells) — all live, no retired-vocab. Notably does NOT use the retired tenant "tier-system" vocabulary (§2) — clean.
  - Watch (cross-side, not a conflict): LINUX pilot has no tenant-lifecycle equivalent; this is a source-only concern.
- **hyperscaler_challenge:** ALIGNED. Explicit cites AWS Organizations account-lifecycle, Google Workspace domain-lifecycle, Stripe account state-machine. A central, auditable, saga-driven tenant state machine with provable erasure (GDPR Art.17 / KR PIPA / CCPA / LGPD) is precisely the hyperscaler control-plane shape. No archive/amend pressure.
- **ai_slop:** None material. Citations are real; the GDPR/PIPA/CCPA/LGPD enumeration is load-bearing (the retention-window Offboarded-vs-DeletionConfirmed split is justified by GDPR Art.17 §3 exceptions).
- **refinement:** Add `masterplan_ref`/`planning_impact` front-matter for SSOT binding. Otherwise masterplan-ready as-is — this is good backfill material.
- **consensus_needed:** NO.

---

### ADR-0176 — Brown-out + graceful-degradation signal API

- **decision_atom:** Every public µservice RPC emits a normative `oya-degradation-class: nominal|degraded|brownout|outage` response header (plus a per-µservice Prometheus gauge), computed as the max of SLO-burn / resource-pressure / dependency classes, so upstream callers and the mesh make explicit static-stability fallback and retry-budget decisions instead of inferring degradation from latency tails.
- **current_status:** Accepted (2026-05-18); enforcement `advisory-until-public-rpc-coverage-complete`. Clean YAML front-matter.
- **disposition:** AMEND (lightly). Sound decision; one stale mesh reference needs reconciliation.
- **governing:** Mesh churn — front-matter `related` lists BOTH ADR-0044 (Istio ambient + Envoy gateway) AND ADR-0148 (Cilium service mesh), and the body says "The mesh layer (Istio per ADR-0148)" — internally contradictory (attributes Istio to the Cilium ADR). Canonical posture (§3 isolation/mesh) is Cilium per ADR-0148; Istio is the retired/superseded mesh. D-4 correctly says "Cilium L7."
- **truth_flag:** TRUE (decision); PARTIAL on the Istio-vs-Cilium mesh attribution bug.
- **in_masterplan:** PARTIAL. YAML + `enforced_by`; no `masterplan_ref`. Declares `specs/brownout-degradation-signal.json`.
- **tensions:**
  - **Mesh contradiction (internal):** body line "mesh layer (Istio per ADR-0148)" mislabels — ADR-0148 is Cilium. Either Istio is retired (use Cilium) or the ref is wrong. Flag for naming fix.
  - **Redis leakage:** Alt-2 names "Redis pub/sub" as a rejected side-channel — Redis is retired→Valkey (§2). Cosmetic (it is being *rejected*), but residual retired-vocab.
  - Cross-ref to chunk-mate ADR-0178: D-3 of 0178 feeds headroom into THIS ADR's brownout classifier ("per-tenant headroom < 0.05 → degraded"). Consistent bidirectional coupling.
  - References ADR-0128 hyperscaler-invariants (INV-STATIC-STABILITY / INV-CIRCUIT-BREAKER) — live.
- **hyperscaler_challenge:** ALIGNED. Explicit, header-based degradation signaling is the documented Google-SRE / Netflix-Hystrix / Cloudflare-Workers pattern (all cited). A hyperscaler would absolutely prefer an explicit in-band signal over latency-tail inference. The bespoke `oya-degradation-class` header is a reasonable house standard. No archive pressure; only the mesh-name AMEND.
- **ai_slop:** Low. The burn-rate thresholds (≤1.0/≤2.0/≤14.0) echo Google SRE multiwindow burn-rate alerting — legitimate, not fabricated. The Rust caller snippet (D-5) is illustrative, fine.
- **refinement:** (1) Fix "Istio per ADR-0148" → "Cilium per ADR-0148"; drop ADR-0044 Istio from `related` or mark it superseded. (2) Replace "Redis pub/sub" example with Valkey or a neutral phrasing. (3) Add `masterplan_ref`.
- **consensus_needed:** NO (mechanical mesh-name fix; not a contested decision).

---

### ADR-0177 — Internal vs external API surface separation

- **decision_atom:** Split the API surface into two gateway tiers — public (`api.oyatie.com`, semver-stable per ADR-0037, OAuth+per-key, public rate limits) and internal (`internal-api.oyatie.com`, mesh-mTLS/SPIFFE only, semver-waived, 10× budget, invisible outside the mesh) — with every OpenAPI route declaring `api_surface: public|internal`.
- **current_status:** Accepted (2026-05-18); enforcement `advisory-until-gateway-tier-deployed`. Clean YAML front-matter.
- **disposition:** AMEND (lightly). Strong, hyperscaler-standard decision; carries one superseded cross-ref and one retired-grouping echo.
- **governing:** D-3 routes internal changelog "into the relevant ChangeSet (ADR-0110)" — ADR-0110 (changeset state machine) is `Superseded` by ADR-0363 (retire agentic-VCS; §1.1). Stale ref. Also front-matter `related` lists ADR-0044 (Istio) while body uses Cilium (ADR-0148) — same mesh churn as 0176.
- **truth_flag:** TRUE (decision); PARTIAL on the ADR-0110 ChangeSet ref (points at a superseded VCS machine).
- **in_masterplan:** PARTIAL. YAML + `enforced_by`; no `masterplan_ref`. Declares `specs/api-surface-separation.json`.
- **tensions:**
  - **Superseded ref:** "ChangeSet (ADR-0110)" — 0110 retired by 0363. Internal-changelog mechanism should point at the current plain-git/Forgejo-PR flow, not the dead changeset state machine.
  - **Retired grouping echo:** Alt-3 cites "the 'no bundle' decision (ADR-0132)" and "flat-catalog thesis (ADR-0001, ADR-0058)" — these are consistent with ADR-0362 (full grouping retirement, flat-only), so the *direction* is right, but the cited ADR-0001/0058 cohesion-thesis numbering is the early-corpus numbering, not the canonical flat-only governor (ADR-0362). Worth a ref refresh.
  - Cross-ref to chunk-mate ADR-0178: D-1 internal tier rate-limit "per ADR-0178" — consistent; 0178 D-5 reciprocally grants internal 10× per-IP budget. Good bidirectional coupling.
- **hyperscaler_challenge:** ALIGNED. The public-vs-internal (control-plane vs data-plane) gateway split is textbook — explicitly cites Stripe (api vs internal-api), AWS (api.aws vs control-plane), Google Cloud (cloudresourcemanager vs cellservices). 404-not-403 for internal routes (invisible, not merely forbidden) is the correct security posture. No archive pressure.
- **ai_slop:** Low. The Stripe "internal handbook excerpt published as a public blog" citation is slightly hand-wavy (fabricated-precision risk), but the pattern itself is real and well-known.
- **refinement:** (1) Replace ADR-0110 ChangeSet ref with the current git/PR changelog flow. (2) Refresh the flat-catalog citation to ADR-0362 (the canonical grouping-retirement governor). (3) Fix Istio→Cilium mesh ref. (4) Add `masterplan_ref`.
- **consensus_needed:** NO.

---

### ADR-0178 — Layered throttling (per-IP / per-API-key / per-user / per-tenant)

- **decision_atom:** Four throttle layers evaluated outermost-first (per-IP anti-abuse → per-API-key developer budget → per-user within-tenant → per-tenant cell-level), each with its own token-bucket counter store, 429+`oya-throttle-class` denial, and a `0..1` headroom header that feeds the ADR-0176 brown-out classifier.
- **current_status:** Accepted (2026-05-18); enforcement `advisory-until-public-rpc-coverage-complete`. Clean YAML front-matter.
- **disposition:** AMEND. Architecturally sound and hyperscaler-standard, but carries TWO live retired-vocabulary hits (Redis counter store, and tenant `tier` Free/Pro/Enterprise budgets) plus a retired-brand ref — needs reconciliation, not just cosmetics.
- **governing:** Retired-vocab governed by ADR-0336 (Redis→Valkey) and ADR-0329 (tier-system retired→tenant-class). The "foundry MCP gateway (ADR-0021)" reference governed by ADR-0335 (foundry→intelligence). NOTE: source ADR-0021 is "foundry-capability-registry-and-mcp-gateway"; LINUX ADR-0021 is owned-policy — number collision on merge (§6.4), but this ref points at the SOURCE 0021 correctly.
- **truth_flag:** PARTIAL. Decision TRUE; the per-tenant `tier` axis (Free=1k/Pro=10k/Enterprise=negotiated) is STALE vocabulary — ADR-0329 retired the tenant "tier-system" in favor of `tenant_class` (`demo_trial`|`paid`) + composable `billing_components`. The throttle budgets should key off the canonical tenant_class/billing_components, not Free/Pro/Enterprise tiers. Redis as counter store is STALE→Valkey.
- **in_masterplan:** PARTIAL. YAML + `enforced_by`; no `masterplan_ref`. Declares `specs/throttling-tiers.json` (note: the spec name "throttling-TIERS" itself echoes retired tier vocabulary, though here "tiers" = throttle *layers*, an unfortunate collision with the retired tenant-tier term).
- **tensions:**
  - **Retired tenant-tier (§2, ADR-0329):** D-2 / D-6 per-tenant budgets keyed `Free`/`Pro`/`Enterprise`. This is the retired capability-tier / tier-system axis. Must reconcile to `tenant_class` + `billing_components`. Distinct from the (live) autonomy-tier T1–T4 axis — but these throttle tiers are the *retired billing* axis, not autonomy. Real conflict.
  - **Redis (§2, ADR-0336):** per-IP/per-user/per-tenant counter stores specified as "Redis" — retired→Valkey.
  - **foundry brand (§2):** "foundry MCP gateway per-tenant rate limit (ADR-0021)" — retired→intelligence.
  - **Istio leakage:** related/body reference ADR-0044 Istio ambient alongside ADR-0148 Cilium — same mesh churn.
  - Cross-ref to chunk-mate ADR-0176 (headroom→brownout) and ADR-0177 (internal 10× budget) — both consistent and bidirectional. Good.
- **hyperscaler_challenge:** ALIGNED on shape (cites Cloudflare/AWS-API-Gateway/Stripe/Twilio/Shopify layered throttling — all real). MISALIGNED on the tenant-tier key: a hyperscaler keys quota tiers off the billing/account-class primitive, which here has been canonically renamed to tenant_class — so the Free/Pro/Enterprise hardcode argues for AMEND. Alt-4 correctly rejects ML-adaptive limiting as canonical-default (right call; explicit budgets are observable).
- **ai_slop:** Low. Citations real; the ≤2ms-p99 added-latency claim (C-2) is unsourced fabricated-precision ("measured in the existing observability µservice latency budget" — no evidence link).
- **refinement:** (1) Re-key per-tenant budgets from Free/Pro/Enterprise to `tenant_class`(`demo_trial`|`paid`)+`billing_components` per ADR-0329. (2) Redis→Valkey for all counter stores. (3) `foundry`→`intelligence` for the ADR-0021 ref. (4) Rename the spec/lane away from "tiers" to "layers" to avoid colliding with retired tier vocabulary. (5) Fix Istio→Cilium. (6) Substantiate or drop the ≤2ms claim.
- **consensus_needed:** YES (scoped). *"Per-tenant throttle budgets are currently keyed off retired Free/Pro/Enterprise tiers — should they re-key onto the canonical `tenant_class`(demo_trial/paid) + `billing_components`, and is throttle budget a property of billing_components or a separate quota dimension?"*

---

### ADR-0179 — Postgres connection pooling canonical: pgcat

- **decision_atom:** pgcat (Rust, pgbouncer-compatible, multi-tenant/shard-aware) is the canonical Postgres connection pooler for every Postgres-dependent µservice — per-cell DaemonSet by default (per-pod sidecar only under a declared tenant-isolation constraint), transaction-mode default, with a required `postgres` block in each µservice manifest enforced by a schema gate.
- **current_status:** Accepted (2026-05-18). Clean YAML front-matter + an unusually honest `renumber_note` (originally ADR-0173 in PR#143 Fix-L, rebumped to 0179 after concurrent Fix-J/Fix-K allocation).
- **disposition:** KEEP. This IS the canonical data-tier posture per keystone map §3 ("Postgres + pgcat relational pooling," ADR-0179). Current and correct. Only nit: two cited justifier ADRs are themselves superseded (see below) but the pgcat decision stands.
- **governing:** None for the decision. Cited justifiers ADR-0120 (rust-first onprem tooling) and ADR-0121 (onprem k8s) are BOTH `Superseded` by ADR-0375 (Talos+CAPI+ArgoCD; §1.1). The "Rust-first per ADR-0120" and "hyperscaler-portable per ADR-0121" rationales survive in spirit (Talos is also Rust-adjacent/portable) but the ref numbers are stale.
- **truth_flag:** TRUE. The pgcat decision is canonical and matches §3. PARTIAL only on the two stale superseded-ADR justifier refs (0120/0121).
- **in_masterplan:** PARTIAL-to-YES. Strongest masterplan posture in the chunk: declares `related_specs` pointing at `/specs/hyperscaler-architecture-invariants.json` and `/specs/microservices/manifest-schema.json`, and the decision is reflected in the canonical-posture map (§3). Still lacks an explicit `masterplan_ref`/`planning_impact` key, so not formally bound under planning-ssot-drift-prevention's gate.
- **tensions:**
  - **Superseded justifiers:** "Rust-first per ADR-0120" / "AWS-specific violates ADR-0121 portable invariant" — 0120/0121 superseded by ADR-0375. Refresh to ADR-0375.
  - **Cross-side data-tier fault-line (§5.1):** SOURCE here commits to Postgres+pgcat (assemble proven OSS); LINUX ADR-0001 wants a from-scratch Rust multi-model engine that *eliminates the PostgreSQL/sqlx dependency*. pgcat is meaningless if Postgres is eliminated — this is the sharpest unflagged source↔linux conflict per §5 verdict. SURFACE: pgcat-canonical (source) directly assumes the Postgres SOURCE keeps; LINUX ADR-0001 would obsolete it.
  - Internal-consistency: rejects AWS RDS Proxy (Alt-D) and Supavisor/Elixir (Alt-C) on Rust-first + portability grounds — consistent with §3 OSI-strict/own-the-substrate posture.
- **hyperscaler_challenge:** ALIGNED. Every named precedent is real (Stripe pgbouncer-fork, Notion pgcat, Linear pgbouncer-txn-mode, Supabase Supavisor, AWS RDS Proxy). A connection pooler in front of Postgres at fleet scale is non-negotiable hyperscaler practice. The choice of pgcat (Rust) over RDS-Proxy is the *portable* choice a multi-cloud platform makes — AWS would use RDS Proxy, but a deliberately cloud-portable platform would not, and this ADR justifies that correctly. No archive pressure.
- **ai_slop:** None material. The renumber_note is refreshingly honest provenance, not slop. Precedents are concrete and verifiable.
- **refinement:** (1) Refresh ADR-0120/0121 justifier refs to ADR-0375 (Talos stack). (2) Add explicit `masterplan_ref`/`planning_impact` so this canonical decision binds formally. (3) Flag the §5.1 Postgres-elimination tension with LINUX ADR-0001 in any merge plan (pgcat presupposes Postgres survives).
- **consensus_needed:** YES (cross-side, load-bearing). *"Does the platform keep Postgres as a substrate (making pgcat canonical, ADR-0179), or does the LINUX-pilot owned-multi-model-engine direction (ADR-0001, 'eliminate PostgreSQL') govern long-term — and if both, where is the boundary?"*

---

## Chunk notes for synthesis

**This chunk is the "hyperscaler control-plane discipline" cluster (PR#143 Fix-J/K/L round).** All seven ADRs are same-day (2026-05-18), authored in one anti-hyperscaler-pattern audit pass, and they form a tightly cross-referenced lattice: 0174 chargeback ↔ 0175 lifecycle (billing scope) ↔ 0176 brownout ↔ 0177 surface-split ↔ 0178 throttling (headroom→brownout, internal-10×-budget) ↔ 0179 pgcat (pool budget = capacity-model input). The internal coupling is genuinely consistent — no chunk-internal contradictions in the *decisions*; the defects are all stale *references* and retired *vocabulary*, not logic errors.

**Dominant pattern — well-formed decisions, stale skins.** Six of seven carry clean YAML front-matter + an `enforced_by` advisory gate + a declared `/specs/*.json` canonical artifact. The decisions are largely TRUE and masterplan-ready as backfill. The systematic defects are:
1. **No `masterplan_ref`/`planning_impact` binding on any of the seven** — these are part of the 91.2% unbound ADR mass (planning-ssot-drift-prevention found only 8.8% binding). Whichever masterplan reading wins (authored-authority vs generated-from-ADRs), all seven need binding metadata.
2. **Retired-vocabulary leakage**, concentrated in 0173 and 0178: `foundry` brand (0173, 0174, 0178 → should be intelligence per ADR-0335); tenant `tier`/Free-Pro-Enterprise (0178 → tenant_class per ADR-0329); `Redis` (0176 alt, 0178 → Valkey per ADR-0336).
3. **Superseded cross-refs:** ADR-0042 in-house-UI (0173 → ADR-0383); ADR-0113 Foundry-VCS-substrate (0173 → retired by ADR-0363); ADR-0110 ChangeSet (0177 → retired by ADR-0363); ADR-0120/0121 (0179 → ADR-0375). Auditors must trust the superseding ADR.
4. **Mesh churn (Istio↔Cilium):** 0176/0177/0178 all list ADR-0044 (Istio ambient) in `related` while their bodies use Cilium (ADR-0148). 0176 even mis-attributes "Istio per ADR-0148." Canonical is Cilium. Mechanical naming reconciliation across the cluster.

**Format outlier:** ADR-0173 alone uses the legacy markdown-table header (no YAML), making it structurally unbindable to masterplan and the worst-drifted of the chunk despite carrying the most strategically important doctrine (vendor lock-in). It is the AMEND-with-most-work item.

**Cross-chunk / cross-side tensions to escalate:**
- **Forge keystone (§5.4)** surfaces in 0173: GitHub→Forgejo readiness gate keyed on a *superseded* ADR-0113. Three-way conflict (founder-GitHub vs Forgejo-transitory vs bespoke-VCS-destination). Top tension.
- **Data-tier fault-line (§5.1)** surfaces in 0179: pgcat-canonical presupposes Postgres survives, directly opposing LINUX ADR-0001's "eliminate PostgreSQL." Sharpest unflagged source↔linux conflict per the keystone verdict — pgcat is the concrete artifact that breaks if ADR-0001 wins. Top tension.
- **Number collision (§6.4):** ADR-0179 cites "ADR-0021 foundry MCP gateway" (source 0021); LINUX ADR-0021 is owned-policy — guaranteed collision on merge. The renumber_note on 0179 itself documents the corpus's allocation chaos (concurrent agent rebumps), reinforcing §6.3's "never trust decisions.json next_adr at face value."

**Disposition summary:** KEEP×2 (0175, 0179), AMEND×5 (0173 heavy; 0174/0176/0177 light; 0178 medium). No ARCHIVE/SUPERSEDE in this chunk — none of these are superseded; they are the current control-plane doctrine that merely needs vocabulary/ref reconciliation and masterplan binding. None are GARBAGE; the worst truth_flag is PARTIAL.
