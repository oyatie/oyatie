# ADR Audit — source-48

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-48
- **range:** `ls -1 docs/decisions/ADR-*.md | sort | sed -n "330,336p"`
- **ADRs reviewed (7):** ADR-0394, ADR-0408, ADR-0476, ADR-0478, ADR-0479, ADR-0480, ADR-0481

> Audited READ-ONLY against the keystone map (`_map/canonical-posture-and-supersession-map.md`). Statuses re-verified on disk 2026-06-06. Cross-references resolved against `decisions/` corpus.
>
> **Chunk-wide structural finding (applies to all 7):** every one of these ADRs supersedes/amends a Phase-1 predecessor ADR (0170, 0358, 0421, 0457, 0429, 0443, 0428) and cites a dense web of dependency ADRs (0397 Pulsar, 0406 Postgres, 0411 Crossplane, 0416 Connect-RPC, 0420 Polars, 0407 OTel, 0409 oya-vcs, 0434 portal, 0418 multi-region, 0423 Karpenter, 0403 CloudEvents, 0449 Kiota, 0451 oya-notify). **Of those, only ADR-0170, ADR-0358, ADR-0193, ADR-0509 exist as files in `decisions/`.** The five Phase-1 predecessors (0421/0457/0429/0443/0428) and the entire 0397–0451 dependency band are ABSENT from this branch (they live on origin/dev per keystone §6.3 — gaps 0393–0407 documented-open, corpus runs to ADR-0509). These ADRs therefore supersede phantoms-on-this-branch and depend on phantoms-on-this-branch. Not fatal (the superseding-ADR-is-truth rule applies), but it means the masterplan cannot be generated from this branch alone — the dependency graph is dangling.

---

### ADR-0394 — Bespoke-Rust IDP central hub (Leptos portal + ops-BFF; supersedes ADR-0170 Backstage)

- **decision_atom:** Oyatie's internal developer platform is a bespoke-Rust central operator hub — a Leptos (Rust/WASM SSR) portal-shell over a credential-holding ops-BFF, exposing 18 Cedar-gated surfaces (catalog, scaffolder, scm, cicd, observability, agent-fleet, finops, flags, secrets, incidents, scorecards, status, audit/rbac, provisioning, docs) with AI agents as first-class API consumers — superseding Backstage (ADR-0170).
- **domain:** product-ux (IDP/operator console), cross-cutting with observability (catalog/SLO/CI aggregation surface).
- **current_status:** Proposed (2026-05-29; explicitly "DRAFT for founder review; must NOT auto-merge" — overturns Accepted load-bearing ADR-0170).
- **disposition:** AMEND (then RATIFY). Decision direction is sound and the supersession is correctly wired (ADR-0170 confirmed `status: Superseded, superseded_by: [ADR-0394]` on disk). Needs amend for: (a) Jenkins/Forgejo references should name the current canonical CI/SCM posture (Argo Workflows per ADR-0511, not Jenkins; the BFF-decouples-from-cutover argument is correct but the named substrates are stale); (b) the "foundry→non-foundry rename before agent-fleet console binds" pre-req is the retired-vocab leak the keystone flags (foundry→intelligence/governance per 0335/0347).
- **proposed_resolution:** RATIFY — this is a real, load-bearing doctrine reversal (Backstage→bespoke Leptos IDP) with explicit dependent-retargeting (0203/0209/0213) and a confirmed bidirectional supersession marker; the only blocker is founder sign-off, not soundness. Do not DROP.
- **governing:** supersedes ADR-0170 (developer portal / Backstage) — confirmed Superseded on disk.
- **truth_flag:** PARTIAL — core decision TRUE; named CI/SCM substrates (Jenkins/Forgejo direct) and "foundry" identifier are STALE.
- **in_masterplan:** PARTIAL — IDP/operator-console surface is masterplan-worthy and milestone-tagged (M-IDP-CENTRAL-HUB), but as Proposed it is a candidate, not yet a live masterplan atom.
- **tensions:** (1) OIDC issuer for IDP login is left undecided IN THE ADR — "Zitadel (ADR-0187) vs bespoke oya-identity-oidc-issuer-kernel" — which is the exact Zitadel-vs-oya-identity fault-line that ADR-0476 (in this same chunk) resolves in favor of bespoke; 0394 should now bind to oya-identity (ADR-0476), not present it as open. (2) FinOps surface explicitly waits on "oya-cost/meter/billing trio when ADR-0478/0479/0480 land" — those are Accepted in this same chunk, so the wait is resolved. (3) Numbering: self-documents the 0392/0408 out-of-band Buck2 allocation collision.
- **hyperscaler_challenge:** ALIGNED — Google (Cloud Console/internal IDP), AWS (internal consoles), and Meta all run bespoke first-party operator consoles, not Backstage in their core control plane; "agents as first-class API consumers" is a genuinely forward bet. Implication: keep (RATIFY), no archive.
- **ai_slop:** None — substantive, self-aware (flags its own non-auto-merge, quarantines charts, retargets dependents). The 18-module enumeration is concrete, not padding.
- **refinement:** Replace Jenkins/Forgejo-direct mentions with Argo-Workflows/bespoke-SCM-behind-BFF framing; bind OIDC to ADR-0476; drop "foundry" identifier per 0335/0347.
- **consensus_needed:** "ADR-0394 leaves the IDP-login OIDC issuer open between Zitadel (0187) and bespoke oya-identity; ADR-0476 (same wave) makes oya-identity canonical and retires the Zitadel-class choice. Confirm 0394 binds to oya-identity so the IDP does not re-open a settled identity decision?"

---

### ADR-0408 — Buck2-driven CI/CD (RBE + affected-targets + image builds; reverses ADR-0358 §2 Bazel CI engine)

- **decision_atom:** The CI build engine is Buck2 (not Bazel): the CI orchestrator drives `buck2 build`/`buck2 test` against a self-hosted NativeLink RBE, selects graph-exact affected targets via `buck2 cquery rdeps(//..., <changed>)`, and builds container images through Buck2's content-addressed cache, with the `oya` governance overlay layered unchanged on top.
- **domain:** ci-cd-build.
- **current_status:** Proposed (2026-05-29; "DRAFT for founder review; must NOT auto-merge").
- **disposition:** AMEND (then RATIFY). The Buck2-engine decision itself is TRUE and survives in the keystone's net-current CI truth (Buck2 build/RBE + Argo Workflows orchestration). BUT the ADR's load-bearing framing — "Jenkins invokes buck2 … Jenkins remains the orchestrator (ADR-0359) … ADR-0359 is NOT superseded" — is now FALSE: ADR-0359 is `Superseded by ADR-0511` on disk (Jenkins is transitory bootstrap; Argo Workflows is the destination orchestrator). The engine reversal stands; the orchestrator binding is stale.
- **proposed_resolution:** RATIFY the Buck2-engine decision, but the orchestrator clause must be amended from "Jenkins (sole CI per ADR-0359)" to "the destination CI orchestrator (Argo Workflows per ADR-0511; Jenkins transitory bootstrap)." Without that amend the ADR ratifies a retired orchestrator posture.
- **governing:** supersedes ADR-0358 (§2 Bazel CI engine only — confirmed ADR-0358 still `Proposed` on disk, rest-of-roadmap intact). Stale dependency: ADR-0359 (now Superseded by 0511); see ADR-0511/0513/0514 for the live orchestration chain.
- **truth_flag:** PARTIAL — Buck2-as-engine TRUE; "Jenkins is the orchestrator / 0359 not superseded" WRONG (0359 is superseded by 0511).
- **in_masterplan:** PARTIAL — Buck2 build/RBE engine is part of the canonical CI/CD posture (keystone §3 CI/CD row) and belongs in the masterplan; the Jenkins-orchestrator binding does not.
- **tensions:** Direct tension with ADR-0511 (Argo Workflows supersedes Jenkins): 0408 asserts ADR-0359 is "complementary, NOT superseded" and "Jenkins remains the orchestrator," which 0511 explicitly overturned. The Buck2 layer is non-conflicting; only the orchestrator sentence collides. Also a `decisions.json next_adr` / out-of-band-allocation collision (self-documented, shared with ADR-0392/0394).
- **hyperscaler_challenge:** ALIGNED on engine — Google's internal CI is Bazel/Blaze + remote execution; Buck2 (Meta-origin) + self-hosted RBE + graph-exact affected-target presubmit is exactly the Google-TAP / Meta pattern the ADR cites. QUESTIONABLE on orchestrator — no hyperscaler runs Jenkins as destination CI; the keystone already retired Jenkins-as-destination. Implication: amend the orchestrator clause to Argo Workflows; keep the Buck2 engine decision.
- **ai_slop:** None — strong honesty section ("Buck2-driven CI is 0% adopted … NO numeric figure is asserted"). The drift is a real cross-ADR staleness, not fabrication.
- **refinement:** Re-target every "Jenkins" mention to "the destination orchestrator (Argo Workflows, ADR-0511; Jenkins transitory)"; note 0359→0511 supersession explicitly.
- **consensus_needed:** "ADR-0408 (Buck2 CI engine) binds the engine to Jenkins-as-sole-orchestrator (ADR-0359), but ADR-0511 retired Jenkins-as-destination in favor of Argo Workflows. Ratify the Buck2 engine decision while re-pointing its orchestrator to Argo Workflows?"

---

### ADR-0476 — oya-identity: bespoke Rust human identity substrate (supersedes ADR-0421 Keycloak)

- **decision_atom:** Human identity is a bespoke Rust-native substrate `microservices/oya-identity` — an OIDC provider + OAuth 2.0 authorization server (openidconnect-rs/oxide-auth/webauthn-rs, Postgres-backed, per-tenant Crossplane realms, Cedar over unified human+SPIFFE principals) — replacing Keycloak (ADR-0421, retained only as a Phase-1 traffic-shadowed bridge until parity).
- **domain:** identity-authn, cross-cutting with crypto-keymgmt (passkeys/WebAuthn/MFA).
- **current_status:** Accepted (2026-05-28, founder-locked).
- **disposition:** AMEND. Decision is sound and Accepted/founder-locked, but it carries a **direct, unreconciled tension with the keystone canonical posture** (ADR-0187 = Zitadel primary OIDC IdP, confirmed `Accepted` on disk) and a **WRONG Cedar citation** (cites "Cedar (ADR-0083)" — ADR-0083 is `rust-error-handling-tier-decision`, NOT Cedar; the real Cedar gate ADRs are 0099/0243/0246/0191). Both must be amended before this is masterplan-clean.
- **proposed_resolution:** N/A (status is Accepted, not Proposed). If treated as the canonical identity decision, it must explicitly supersede/retarget ADR-0187 (Zitadel) and 0188/0189/0190 (the Zitadel-anchored passkey/step-up/SCIM stack), which it does NOT currently do.
- **governing:** supersedes ADR-0421 (Keycloak) — but ADR-0421 is ABSENT from `decisions/` on this branch (referenced in 2 files only). The keystone-canonical Zitadel ADR-0187 is the one that actually needs superseding and is not named.
- **truth_flag:** PARTIAL — bespoke-identity direction is TRUE and founder-locked; the "Cedar (ADR-0083)" citation is WRONG; the front-matter omits the Zitadel-0187 supersession it logically implies (STALE relative to canonical posture).
- **in_masterplan:** PARTIAL — bespoke oya-identity is masterplan-grade (M-IDENTITY-V2) and likely THE forward identity atom, but the masterplan currently encodes Zitadel (0187) as canonical; the two cannot both be the SSOT.
- **tensions:** (1) **Hard conflict with ADR-0187 (Zitadel primary OIDC IdP, Accepted)** and the keystone "Identity/crypto" canonical row — this is the sharpest tension in the chunk; oya-identity does not cite or supersede 0187 at all. (2) Mis-cites Cedar as ADR-0083. (3) D5 cites "SPIFFE federation (ADR-0394)" but ADR-0394 is the IDP-hub ADR, not a SPIFFE ADR — another WRONG citation. (4) Rejects Zitadel in "alternatives" on a "Go-based" objection while the canonical posture already chose Zitadel — the ADR re-litigates a settled (in the other direction) decision.
- **hyperscaler_challenge:** ALIGNED in principle — Google Identity Platform, AWS IAM Identity Center, Meta human-auth are indeed bespoke, not operated OSS. QUESTIONABLE on timing/cost — a bespoke OIDC+OAuth+WebAuthn server is a 6–12mo security-critical build; hyperscalers reached bespoke identity at massive scale, not as a pre-product-market-fit pilot. Implication: keep the direction (it is the genuine hyperscaler end-state) but amend to (a) name Zitadel/0187 as the superseded Phase-1 anchor, (b) fix Cedar citation, (c) keep Zitadel/Keycloak as the bridge until parity (ADR already says this for Keycloak).
- **ai_slop:** Low — the Hyperscaler-lens table and alternatives are substantive; but the repeated wrong ADR numbers (0083 Cedar, 0394 SPIFFE) are citation-slop that would poison a generated masterplan.
- **refinement:** Fix Cedar→0099/0243; fix SPIFFE citation; ADD explicit supersession of ADR-0187 (Zitadel) and retarget 0188/0189/0190; reconcile with ADR-0394's open OIDC-issuer question.
- **consensus_needed:** "The canonical posture names Zitadel (ADR-0187, Accepted) as primary OIDC IdP, but ADR-0476 (Accepted, founder-locked) makes bespoke oya-identity canonical without superseding 0187. Which is the masterplan SSOT for human identity — and does oya-identity formally supersede the entire Zitadel stack (0187/0188/0189/0190)?"

---

### ADR-0478 — oya-billing: bespoke Rust billing engine (supersedes ADR-0457 Lago)

- **decision_atom:** The canonical billing plane is a bespoke Rust engine `microservices/oya-billing` (Axum + Connect-RPC, Postgres ledger, Pulsar event fan-out, Stripe-default with a PaymentAdapter trait, tenant-isolated billing-as-a-product), replacing Lago (ADR-0457) which is retired for AGPL-3.0 redistribution risk in the billing-as-a-product topology plus Ruby/Rails doctrine conflict.
- **domain:** marketplace-commerce (billing/invoicing), cross-cutting with finops-cost.
- **current_status:** Accepted (2026-05-28, founder-locked).
- **disposition:** AMEND. Decision is sound (AGPL avoidance is consistent with the keystone OSI-strict / no-AGPL-in-product posture, ADR-0013/0211/0345; Pulsar/Postgres alignment is correct). Same **WRONG Cedar citation (ADR-0083)** as the rest of the cluster. Supersedes ADR-0457 (Lago) which is ABSENT from this branch.
- **proposed_resolution:** N/A (Accepted).
- **governing:** supersedes ADR-0457 (Lago) — absent on branch (referenced in 2 files).
- **truth_flag:** PARTIAL — billing-engine direction + AGPL rationale TRUE; "Cedar (ADR-0083)" WRONG.
- **in_masterplan:** PARTIAL — masterplan-grade (M-BILLING-ENGINE-V2); part of the oya-cost/meter/billing FinOps trio that ADR-0394 explicitly waits on.
- **tensions:** (1) Cedar mis-citation (0083). (2) Consistent with ADR-0479/0480 integration chain (meter→billing, cost→billing) — internally coherent within the trio. (3) AGPL rationale aligns cleanly with keystone license posture (no tension there — this is a correct application of the OSI-strict doctrine, same family as Redis→Valkey/0336 and observability→AGPL-carve-out/0383).
- **hyperscaler_challenge:** ALIGNED — Stripe Billing, Shopify Billing, AWS Billing are bespoke internal planes; billing-as-a-product (D5) is exactly the Stripe model. The AGPL-in-redistributed-product concern is a real, hyperscaler-grade legal driver. Implication: keep; amend citation only.
- **ai_slop:** Low — concrete primitives (subscription states, PaymentAdapter trait, integration ASCII diagram). Boilerplate "Implementation pattern (ADR-0509 alignment)" footer is templated across all five but is accurate, not slop.
- **refinement:** Fix Cedar→0099/0243; confirm ADR-0457 lands on the merged corpus or mark the supersedee as a known-absent predecessor.
- **consensus_needed:** None contested — the only open item is the corpus-wide Cedar-citation fix.

---

### ADR-0479 — oya-meter: bespoke Rust usage metering substrate (supersedes ADR-0429 OpenMeter)

- **decision_atom:** Usage metering is a bespoke Rust substrate `microservices/oya-meter` (Axum + Connect-RPC, ClickHouse time-series + Postgres meter-catalog, Pulsar CloudEvents ingest, Polars window-aggregates, Cedar-gated per-tenant aggregate API, Kiota-generated tenant SDK), feeding oya-billing — replacing OpenMeter (ADR-0429, retired Go/managed-service seam).
- **domain:** finops-cost (metering), cross-cutting with data-engine-db (ClickHouse/Polars aggregation path).
- **current_status:** Accepted (2026-05-28, founder-locked).
- **disposition:** AMEND. Sound and well-scoped (ClickHouse ADR-0193 confirmed Accepted on disk; bounded-complexity argument is honest at ~4–6mo). Same **WRONG Cedar citation (ADR-0083)**. Supersedes ADR-0429 (OpenMeter), absent on branch.
- **proposed_resolution:** N/A (Accepted).
- **governing:** supersedes ADR-0429 (OpenMeter) — absent on branch (referenced in 3 files).
- **truth_flag:** PARTIAL — metering direction TRUE; Cedar (0083) WRONG. Note the meter table cites "cloud-intelligence" (correct retired-vocab replacement for foundry) — vocabulary-clean.
- **in_masterplan:** PARTIAL — masterplan-grade (M-METERING-V2); the FinOps-trio middle tier ADR-0394 waits on.
- **tensions:** (1) Cedar mis-citation. (2) Integration chain with 0478/0480 is internally consistent (oya-cost→oya-meter→oya-billing). (3) Uses "cloud-intelligence/token" meter — correctly post-foundry vocabulary, no leak. (4) Depends on Polars(0420)/Kiota(0449)/CloudEvents(0403)/Pulsar(0397) — all absent from this branch (origin/dev band).
- **hyperscaler_challenge:** ALIGNED — AWS Metering Service / GCP Usage Tracking are bespoke internal primitives; high-volume ingest separated from low-volume billing settlement (the 0132 single-concern split) is the correct hyperscaler decomposition. Implication: keep; amend citation only.
- **ai_slop:** Low — the resource/action/unit meter table and topic-schema (`usage.{tenant}.{resource}.{action}.v1`) are concrete design, not filler.
- **refinement:** Fix Cedar→0099/0243; flag the absent dependency band (0420/0449/0403/0397) for merge-time resolution.
- **consensus_needed:** None contested beyond the chunk-wide Cedar-citation fix.

---

### ADR-0480 — oya-cost: bespoke Rust K8s cost allocation substrate (supersedes ADR-0443 OpenCost)

- **decision_atom:** K8s cost allocation is a bespoke Rust substrate `microservices/oya-cost` reducing to three primitives (Karpenter node-price + Mimir pod-resource-fraction + tenant/region/runtime-tier attribution), storing in ClickHouse with Cedar-gated per-tenant cost APIs and cost-as-SLI, feeding oya-meter→oya-billing — replacing OpenCost (ADR-0443, retired Go/Prometheus-scrape-only operator).
- **domain:** finops-cost, cross-cutting with observability (Mimir metrics ingest, cost-as-SLI/OpenSLO).
- **current_status:** Accepted (2026-05-28, founder-locked).
- **disposition:** AMEND. Sound and the most concretely scoped of the trio (explicit D1–D5 deliverables with exit criteria; Mimir ADR-0383 is the keystone-canonical observability stack — correct). Same **WRONG Cedar citation (ADR-0083)**. Supersedes ADR-0443 (OpenCost), absent on branch.
- **proposed_resolution:** N/A (Accepted).
- **governing:** supersedes ADR-0443 (OpenCost) — absent on branch (referenced in 3 files).
- **truth_flag:** PARTIAL — cost-allocation direction TRUE; Cedar (0083) WRONG.
- **in_masterplan:** PARTIAL — masterplan-grade (M-COST-ALLOCATION-V2); the FinOps-trio upstream cost source ADR-0394 waits on.
- **tensions:** (1) Cedar mis-citation. (2) Integration block has a directional ambiguity vs ADR-0479: 0480 says "oya-cost publishes per-request cost signals consumed by oya-meter" AND "oya-cost is the upstream cost source," while D4 says "Feed oya-meter → oya-billing" — the meter↔cost direction reads cleanly (cost→meter→billing) but the prose is mildly circular; worth one editorial pass. (3) Depends on Karpenter(0423)/OpenSLO(0441)/Polars(0420)/multi-region(0418) — absent on branch.
- **hyperscaler_challenge:** ALIGNED — AWS Cost Allocation Tags + Cost Explorer and GCP Cost Recommendations + Billing BigQuery export are bespoke; no hyperscaler ships OpenCost internally. The "three primitives" reduction is a credible bounded-scope claim. Implication: keep; amend citation only.
- **ai_slop:** Low — deliverable table with exit criteria is the most implementation-ready in the chunk.
- **refinement:** Fix Cedar→0099/0243; tighten the cost↔meter directional prose; flag absent deps.
- **consensus_needed:** None contested beyond chunk-wide Cedar-citation fix.

---

### ADR-0481 — oya-flags: bespoke Rust feature flag server (supersedes flagd; amends ADR-0428 OpenFeature SDK)

- **decision_atom:** The feature-flag server is a bespoke Rust service `microservices/oya-flags` (Axum + Connect-RPC speaking the OpenFeature evaluation protocol, Postgres + in-memory hot cache, Cedar-expression targeting, flag-as-code via oya-vcs GitOps validated by Buck2 CI, OTel-instrumented, sub-ms eval) — replacing the flagd DaemonSet while preserving the OpenFeature SDK (ADR-0428, amended server-only).
- **domain:** ci-cd-build (flag-as-code/release control), cross-cutting with intelligence-ai (cloud-intelligence Cluster I/II/III routing reads flags).
- **current_status:** Accepted (2026-05-28, founder-locked).
- **disposition:** AMEND. Sound; cleanly preserves the OpenFeature SDK as the open compat protocol while owning the server (the keystone-consistent "open SDK, bespoke server" pattern). Same **WRONG Cedar citation (ADR-0083)**. The amends-not-supersedes framing (ADR-0428 SDK retained, server-only swap) is correct and `supersedes: []` / `amends: [ADR-0428]` is the right front-matter shape.
- **proposed_resolution:** N/A (Accepted).
- **governing:** amends ADR-0428 (OpenFeature/flagd) — absent on branch (referenced in 2 files); no ADR superseded.
- **truth_flag:** PARTIAL — flag-server direction TRUE; Cedar (0083) WRONG. Cites Buck2 CI (ADR-0408) as the schema-validation gate — but 0408 is only `Proposed` and binds to retired Jenkins-orchestrator framing, so D3's "Buck2 gate is BLOCKER" depends on an unratified, partially-stale ADR.
- **in_masterplan:** PARTIAL — masterplan-grade (M-FEATURE-FLAGS-V2); has the richest front-matter deliverables block (D1–D5 with verified_by gates) of any ADR in the chunk.
- **tensions:** (1) Cedar mis-citation. (2) Hard-depends on ADR-0408 (Buck2 CI, Proposed + stale-orchestrator) for its D3 blocker gate, and on ADR-0476 (oya-identity, the contested-vs-Zitadel one) for auth, and on ADR-0384 (cloud-intelligence routing, `Proposed`) for its headline consumer — three of its load-bearing dependencies are Proposed/contested while it is itself Accepted. (3) Correctly post-foundry vocabulary ("cloud-intelligence").
- **hyperscaler_challenge:** ALIGNED — Google Gatekeeper-flags, Meta GateKeeper/ExperimentFramework, Stripe internal flags are all bespoke servers behind an open SDK; the OpenFeature-SDK-as-compat-shim is precisely that pattern. Implication: keep; amend citation only.
- **ai_slop:** Low — the deliverables block with explicit `exit_criteria`/`verified_by` per deliverable is the strongest evidence-discipline in the chunk; not slop.
- **refinement:** Fix Cedar→0099/0243; gate D3's "Buck2 BLOCKER" on ADR-0408 ratification (or re-point to the destination orchestrator); note its Accepted-depends-on-Proposed inversion.
- **consensus_needed:** "ADR-0481 is Accepted but its D3 release-gate hard-depends on ADR-0408 (Buck2 CI, still Proposed and bound to retired Jenkins framing) and its headline consumer ADR-0384 is Proposed. Should an Accepted ADR's exit criteria depend on unratified ADRs, or should D3 be re-pointed to the current canonical CI gate?"

---

## Chunk notes

**1. Two distinct waves in this slice.** (a) ADR-0394 + ADR-0408 are the **2026-05-29 founder-reversal pair** — both `Proposed`, both self-labelled "must NOT auto-merge," both reversing an Accepted/reasoned predecessor (Backstage / Bazel-CI), both sharing the out-of-band 0392/0394/0408 numbering collision the keystone §6.3 documents. (b) ADR-0476/0478/0479/0480/0481 are the **2026-05-28 "bespoke-Rust-replaces-Phase-1-OSS" quintet** — all `Accepted`, all founder-locked, all following an identical template (Context → Decision D1–D5 → Hyperscaler-lens table → Alternatives → Consequences → ADR-0509 footer). The quintet is internally coherent and forms a self-consistent FinOps/identity/flags substrate (identity↔billing↔meter↔cost↔flags all cross-reference correctly).

**2. CORPUS-WIDE Cedar mis-citation (HIGH PRIORITY for a generated masterplan).** All five Accepted ADRs (0476/0478/0479/0480/0481) cite **"Cedar (ADR-0083)"**. ADR-0083 on disk is `rust-error-handling-tier-decision`, NOT Cedar. The real Cedar decisions are ADR-0099/0243/0246/0191 (keystone §3 policy row). This is a single propagated typo across a whole wave; if the masterplan is GENERATED from ADR front-matter/citations, this would mis-wire the entire authz dependency edge for the FinOps+identity+flags cluster. Recommend a one-line bulk-amend (Cedar = ADR-0099/0243) — does not change any decision_atom.

**3. The Zitadel↔oya-identity fault-line is now LIVE in the corpus, not just the keystone.** ADR-0187 (Zitadel primary OIDC IdP) is `Accepted` on disk and is the keystone-canonical identity decision. ADR-0476 (Accepted, founder-locked) makes bespoke oya-identity canonical, **rejects Zitadel in its alternatives table, but never supersedes ADR-0187**. ADR-0394 (same chunk) leaves the IDP-login issuer explicitly OPEN between the two. So three ADRs in or adjacent to this chunk disagree about the OIDC IdP, with no supersession edge connecting them. This is the single most important founder decision surfaced by this chunk: **is oya-identity the SSOT (and must it formally supersede the 0187/0188/0189/0190 Zitadel stack), or is Zitadel still canonical?** Cannot be resolved by an auditor — flagged for founder.

**4. CI/CD staleness in ADR-0408.** ADR-0408's Buck2-build-engine decision is TRUE and lives on in the canonical CI posture, but its orchestrator binding ("Jenkins is sole CI, ADR-0359 NOT superseded") is FALSE: ADR-0359 is `Superseded by ADR-0511` (Jenkins transitory; Argo Workflows destination). The Buck2 engine and Argo orchestrator compose fine — 0511 itself says so ("composes with the Buck2 reversal PR") — but 0408's text must be amended off the Jenkins framing before it can be ratified into the masterplan. ADR-0481's D3 inherits this staleness (it gates flag releases on the Proposed, Jenkins-bound 0408).

**5. Dangling dependency graph on this branch.** Of the ~25 distinct ADRs cited as supersedees/dependencies across this chunk, only 0170, 0358, 0193, 0509, 0187, 0099 exist as files in `decisions/`. The five Phase-1 predecessors (0421/0457/0429/0443/0428) and the entire 0397–0451 infrastructure band (Pulsar/Postgres/Crossplane/Connect-RPC/Polars/OTel/oya-vcs/portal/notify/Karpenter/CloudEvents/Kiota) are absent — they sit on origin/dev (keystone §6.3: corpus runs to 0509, gaps 0393–0407 documented-open). The superseding-ADR-is-truth rule keeps these decisions valid, but **a masterplan generated from THIS branch alone would have broken dependency edges**; merge-time re-resolution against the full origin/dev corpus is required.

**6. Disposition summary.** 0 KEEP-clean / 7 AMEND / 0 ARCHIVE / 0 MERGE / 0 UNCLEAR. Two Proposed (0394, 0408) → both RATIFY-after-amend (real load-bearing reversals, not DROP candidates). Five Accepted (0476/0478/0479/0480/0481) → AMEND for the shared Cedar mis-citation; 0476 additionally needs the Zitadel-supersession reconciliation; 0408/0481 additionally need the CI-orchestrator de-staling. No ai-slop, no garbage, no obsolete ADRs in this chunk — the divergence here is genuine forward-architecture (bespoke-replaces-OSS at every FinOps/identity seam, the hyperscaler end-state), with the defects being citation hygiene and cross-ADR staleness rather than wrong decisions.
