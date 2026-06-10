# ADR Audit — source-32

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** source-32
- **Range (ls slice 218–224):** ADR-0253, ADR-0254, ADR-0255, ADR-0257, ADR-0258, ADR-0263, ADR-0272
- **ADRs reviewed:** 7
- **Auditor mode:** coverage backfill (READ-ONLY; only this artifact written)
- **Baseline:** `_map/canonical-posture-and-supersession-map.md`

Cluster character: this slice is dominated by the **2026-05-20 keystone bundle**
(ADR-0242→0255, here 0253/0254/0255) plus the **Tier-1 lockdown bundle**
(0257/0258/0263) and one **privacy keystone** (0272). Six of seven are `Proposed`;
only ADR-0258 is `Accepted`. These are all genuine, load-bearing platform-substrate
decisions — none is garbage — but they carry pervasive **retired-vocabulary residue**
(Kafka, Foundry/foundry-builder, ADR-0042 dead stack) and **two systemic WRONG
cross-references** (ADR-0249 mis-cited as "Foundry dissolution"; ADR-0246 mis-cited
as "audit-chain substrate") that AMEND must fix before any masterplan generation.

---

### ADR-0253 — Network topology: Anycast apex + edge POPs + Cilium-ambient service mesh inside cells

- **decision_atom:** The canonical network topology is Anycast+GeoDNS apex DNS → planetary edge POPs (Cloudflare Workers now, self-hosted Pingora by Year 3+) → per-cell TLS-1.3-only ingress → Cilium-L3/L4 + Istio-Ambient-L7 in-cell mesh with SPIFFE/SPIRE workload identity (hourly SVID rotation), ML-KEM-768+X25519 post-quantum hybrid KEX (Year 2), HTTP/3 client-side, SPIRE-federated inter-cell and WireGuard cross-provider tunnels, all gated by Cedar consuming SPIFFE-ID as principal.
- **domain:** networking-mesh (cross-cuts crypto-keymgmt for PQ/TLS posture)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the topology is sound, hyperscaler-aligned, and the in-cell mesh half is already locked by ADR-0148; ratify, but AMEND the Kafka reference (see truth_flag).
- **governing:** n/a (not archived)
- **truth_flag:** PARTIAL — overall TRUE, but §D-14 names "Kafka for ordered durable streams" and §D-15 NATS+Kafka, which is STALE: ADR-0377-kafka-to-pulsar (Accepted, supersedes ADR-0005) retired standalone Kafka → Pulsar 4.x + Oxia. Must re-point to Pulsar.
- **in_masterplan:** YES (`planning_impact: true`; enforced_by `oya gate validate network-topology-coherence` etc.)
- **tensions:** (a) LINUX fault-line #3 — SOURCE assembles Talos+containerd+Cilium+Pingora vs LINUX framekernel "we are the host". (b) Internal: "edge is NOT in K8s" boundary is clean but presupposes a `microservices/edge-gateway/` that does not yet exist (advisory-until-substrate-lands). (c) Cedar-everywhere east-west adds a PDP hop on every L7 call — latency budget pressure vs the ≤100ms in-cell SLO it itself sets.
- **hyperscaler_challenge:** Aligned. Google/AWS/Cloudflare all run exactly this shape (Anycast + GFE/CloudFront + per-POP TLS1.3 + SPIFFE/ALTS-equiv + PQ-hybrid 2024-26). The ONE questionable bet is the Year-3 self-host-Pingora-POP ambition (own 300 POPs by Year 7) — hyperscalers took a decade and billions; argues for AMEND to time-box/soften the self-host trigger, not archive.
- **ai_slop:** Low. Dense but every claim is load-bearing and cited. Minor over-spec (BGP ASN acquisition at Year 4) reads aspirational.
- **refinement:** Replace Kafka with Pulsar; reconcile the edge-POP self-host roadmap against finops reality.
- **consensus_needed:** "Do we commit Day-0 doctrine to OWNING the planetary edge (Pingora POPs, own ASN, own BGP) — or is the edge a permanent best-of-breed Cloudflare lease? This is the same own-vs-lease axis as the DB/policy/kernel fault-lines."

---

### ADR-0254 — Deployment-model spectrum (five models)

- **decision_atom:** oyatie supports exactly five discrete deployment models (shared-cloud, dedicated-cloud, hybrid/BYO-cloud, on-prem-connected, on-prem-air-gapped) that all ship the SAME Helm charts / Cedar bundles / container images / workflows (one build, Palantir-Apollo lesson) — differing only in cell substrate and control-plane connectivity — orchestrated by a NEW `microservices/deployment-control-plane/` µservice distributing signed `.oab` artifact bundles (cosign + SLSA-L3, offline-verifiable).
- **domain:** orchestration-scheduling (cross-cuts compliance-residency for IL5/6 + air-gap)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the five-model spectrum + single-build invariant is correct and competitively necessary; ratify, but AMEND the broken keystone cross-references and Foundry residue first.
- **governing:** n/a
- **truth_flag:** PARTIAL — decision TRUE, but the "Connection to other keystones" section is internally inconsistent: it cites **ADR-0249 as both "per-tenant data residency spectrum" AND "multi-category-marketplace"**, and elsewhere the bundle implies "ADR-0249 Foundry dissolution" (mirroring ADR-0255). ADR-0249 on disk is `multi-category-marketplace-doctrine` (Proposed). These are WRONG cross-refs. Also `.oab` manifest hardcodes `builder_id: "oyatie-foundry-builder-prod"` + `build_workflow: "foundry-release-build"` — retired Foundry brand residue (→ intelligence/governance per ADR-0335/0347).
- **in_masterplan:** YES (`planning_impact: true`; gates `deployment-model-coherence`, `artifact-bundle-signature`, `air-gap-bundle-manifest`).
- **tensions:** (a) Depends on `deployment-control-plane`, `cloud-secrets`, per-cloud OpenTofu modules that don't exist yet (advisory-until-lands). (b) The "same bits across all five models" invariant is strong but air-gapped LLM serving (no cloud egress) per ADR-0255 quietly breaks "same images" for the intelligence substrate — needs reconciliation. (c) Tier vocabulary: uses "Tier 3 cell" (cell-tier taxonomy per ADR-0248) — distinct from retired tenant "tier-system" (ADR-0329); not a conflict but a lint trap.
- **hyperscaler_challenge:** Aligned-to-leading. Palantir Apollo / Snowflake-Confluent BYOC / GHES air-gapped are the exact references; the single-build-across-models doctrine is the empirically-correct hyperscaler lesson. AWS/Azure would make this same call. No archive pressure.
- **ai_slop:** Low-moderate. The per-crate BC×layer table (~23 crates for one µservice) is plausible but very speculative for a Proposed µservice that doesn't exist — reads as generated scaffolding.
- **refinement:** Fix ADR-0249/keystone cross-refs; strip foundry-builder naming; reconcile air-gapped intelligence vs single-image invariant.
- **consensus_needed:** "Is Day-1 doctrine genuinely all five deployment models (incl. IL5/6 air-gapped with CDS diode), or do we ratify shared/dedicated/hybrid now and defer on-prem/air-gapped to a triggered ADR? The masterplan binds to whichever set we ratify."

---

### ADR-0255 — Intelligence as two-layer AI substrate

- **decision_atom:** Rewrite Intelligence from a consumer-only µservice (ADR-0220) into one µservice with two BC layers — an audience-neutral AI Substrate (8 BCs: transport/credential-resolver/policy-engine-client/guardrails/audit-emit/tool-registry/audience-policy-router/cost-attribution) serving every tenant including `oyatie` itself, and a consumer-scoped Brand Surface (6 BCs) — with opt-in provider-BYOK as the canonical LLM-credential model (zero provider creds in B2B-regulated paths; SecretReference primitive), day-one multi-modal transport, stateless dispatch composed with Workflow durability, caller-side RAG, and absorption of Foundry's providers/guardrails/eval BCs.
- **domain:** intelligence-ai (cross-cuts agentic-platform for the self-modification audience tag)
- **current_status:** Proposed (amends ADR-0220)
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — this is the canonical Intelligence posture per the keystone map (§3 Intelligence row cites ADR-0255 + ADR-0335); ratify, but AMEND the ADR-0249 mis-citation and audience-tag enum that still hardcodes `oyatie-self-modification`/`oyatie.foundry.*` brand strings.
- **governing:** n/a (it is the governing ADR for the Intelligence domain alongside ADR-0335)
- **truth_flag:** PARTIAL — decision TRUE and current, BUT repeatedly cites **"ADR-0249 (Foundry dissolution, keystone #13)"** as the authority for Foundry's BC redistribution. ADR-0249 is `multi-category-marketplace-doctrine`. The REAL foundry retirement is **ADR-0335** (Accepted). This is a WRONG governing-citation; the absorption logic is correct but bound to the wrong ADR id. Also retains `oyatie.foundry.*` principal naming ("naming preserved operationally") — defensible as principal-scope, but `foundry` brand is RETIRED (ADR-0335/0347), so this needs an explicit note that these are sub-scope principals, not the dead brand.
- **in_masterplan:** YES (`planning_impact: true`; gates `intelligence-two-layer-coherence`, `byok-everywhere`, `no-credentials-in-substrate`, `foundry-bc-absorption-complete`).
- **tensions:** (a) Direct overlap with the LINUX/SOURCE intelligence posture and with ADR-0335 (which is the Accepted authority for foundry→intelligence). Two ADRs (0255 Proposed, 0335 Accepted) both claim the foundry-absorption decision — MERGE-adjacent; 0255 should cite 0335 as governing. (b) Provider-BYOK vs encryption-BYOK disentanglement is clean and correct (cross-refs ADR-0251 §D-10). (c) `transport` BC lists vLLM/SGLang/TensorRT self-hosted adapters → ties to ADR-0211 own-when-proven.
- **hyperscaler_challenge:** Aligned. AWS Bedrock / Azure AI Foundry / Apple Intelligence are the exact "one substrate, layered brand surfaces, audience-aware policy at the call boundary" pattern. Azure literally calls theirs "AI Foundry" — ironic given oyatie RETIRED its own "Foundry" brand. No archive pressure; the model is industry-correct.
- **ai_slop:** Low. The 8+6 BC decomposition and the `secret_references` DDL are concrete and coherent, not slop.
- **refinement:** Re-cite Foundry dissolution to ADR-0335 (not 0249); annotate `foundry` principal strings as retired-brand-but-live-subscope; confirm MERGE relationship with ADR-0335.
- **consensus_needed:** "ADR-0335 (Accepted) and ADR-0255 (Proposed) both decide foundry→intelligence absorption. Which is the governing SSOT for the masterplan, and does 0255 get re-authored as an amendment-of-0335 rather than a peer keystone?"

---

### ADR-0257 — Ontology Object-Type versioning + deprecation handshake

- **decision_atom:** Every Ontology Object Type carries a `schema_revision` (strict SemVer 2.0.0); evolution is additive-by-default with mechanical buf-style breaking-change enforcement; breaking changes require a three-state ACTIVE→DEPRECATED→TOMBSTONED handshake with ≥12-month grace, mandatory per-consumer manifest pinning (`requires_schema_revision`), explicit consumer acknowledgement, Cedar-gated writes, dual-write window, and HLC-ordered cross-cell propagation (5s p99) — preventing Palantir-Foundry-class silent schema regressions.
- **domain:** workflow-ontology (cross-cuts api-contracts; explicitly aligns with ADR-0258)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — this is the canonical, correct, hyperscaler-grade fix for the "F-MISSED-2 Palantir timebomb"; ratify, but AMEND the Kafka transport references.
- **governing:** n/a (it extends/amends ADR-0106 ontology architecture + clarifies ADR-0145)
- **truth_flag:** PARTIAL — decision TRUE, but the event transport repeatedly names **"ADR-0050 Outbox-to-Kafka"** and "broadcast over Kafka" (§D-1 storage, §D-5, §D-6 budget). Kafka is STALE → Pulsar 4.x + Oxia per ADR-0377. The outbox PATTERN survives (per keystone map §1.1: "outbox pattern survives, Kafka retired"); only the transport name must change. Valkey hot-cache reference (§D-1) is correct (Redis→Valkey per ADR-0336).
- **in_masterplan:** YES (`tier_1_lockdown_bundle: true`; `planning_impact` implied; gates `schema-revision-present/semver/consumer-pin/...`).
- **tensions:** (a) Twin of ADR-0263 §D-15 and ADR-0258 §D-8 (three separate "additive-only + deprecation handshake" engines for ontology-schema, log-schema, and API-versioning respectively) — same doctrine, three implementations; possible consolidation candidate but each governs a distinct contract surface, so KEEP-distinct is defensible. (b) Presupposes HLC primitive from ADR-0252 (correctly cited). (c) Mentions "Foundry pipeline cascade re-rebase" (§Context item 6) — retired-vocab residue, illustrative only.
- **hyperscaler_challenge:** Aligned-to-leading. Stripe API versioning + Palantir Foundry SchemaRevision + protobuf/Avro wire rules are the exact references; Google/AWS would make this same call (it IS their call). No archive pressure.
- **ai_slop:** Low. The DDL + Rust lifecycle state machine + BreakingChangeKind enum are concrete and internally consistent.
- **refinement:** Kafka→Pulsar in the transport references; cross-link the three deprecation-handshake engines (0257/0258/0263) so the masterplan records them as one doctrine, three surfaces.
- **consensus_needed:** "Three independent additive-only/deprecation-handshake mechanisms (ontology schema, log schema, API generations) — ratify as three peers, or factor a shared `oya-shared-deprecation-handshake` kernel they all bind to?"

---

### ADR-0258 — API versioning model (dual-mode: Stripe header-pinning public, URL versioning internal mesh)

- **decision_atom:** Public APIs use Stripe-style request-time date-stamped pinning via `X-Oyatie-API-Version` header (per-tenant Cedar-gated default, ≥12-month sunset, RFC-8594 Deprecation/Sunset headers, auto-generated per-language SDKs from OpenAPI 3.2.0/proto/AsyncAPI 3.1.0); internal mesh APIs use integer-major URL versioning (`/v1/`,`/v2/`) for deterministic L7 routing; each µservice versions independently; webhooks carry dual header+body version.
- **domain:** api-contracts
- **current_status:** Accepted (the only Accepted ADR in this slice; `doc_status: published`, tier-1-lockdown)
- **disposition:** AMEND (keep the decision; fix brand residue only)
- **proposed_resolution:** n/a — already Accepted; no RATIFY/DROP needed.
- **governing:** n/a
- **truth_flag:** PARTIAL — decision TRUE and Accepted, but examples hardcode the retired **Foundry** brand on the public surface: gRPC package `oya.foundry.v1.CapabilityService` (§D-2, Appendix A.2) and "Foundry" listed as a public API surface alongside Workspace/Cloud/Verticals (§Context, §D-1). Per ADR-0335 the consumer-facing brand is `cloud-intelligence`; "Foundry" as a public product surface is dead. Lane name `oya-governance-api-semver` (§D-8) correctly uses the NEW `oya-governance-*` prefix (good — ADR-0347 compliant). Also references ADR-0042 observability (§References) which is superseded by ADR-0383 — a stale ref, not load-bearing.
- **in_masterplan:** YES — BINDING; the strongest masterplan binding in this slice (Accepted, rollout-phased W+0→W+180, BLOCKER lanes).
- **tensions:** (a) "Foundry" as a public surface contradicts ADR-0335 (foundry brand retired). (b) References "tier" vocabulary inherited from ADR-0037 (API stability tiers preview/stable/GA — DIFFERENT axis from retired tenant tier-system; not a conflict). (c) Twin deprecation-handshake doctrine with ADR-0257/0263 (see those).
- **hyperscaler_challenge:** Aligned-to-leading. Stripe/Square/Plaid header-pinning + AWS-Twirp/Google-Stubby URL-mesh-versioning are the exact references; the dual-mode split is the empirically-correct hyperscaler synthesis. AWS/Stripe would make this identical call.
- **ai_slop:** Very low — this is the most rigorous, alternatives-analyzed, rollback-planned ADR in the slice; clearly the reason it reached Accepted.
- **refinement:** Replace `oya.foundry.*` gRPC package + "Foundry" public-surface naming with `cloud-intelligence` per ADR-0335; drop the dead ADR-0042 ref → ADR-0383. (Append-only amendment, since it's Accepted/immutable per the generated-from-ADRs doctrine.)
- **consensus_needed:** none on the decision itself; only the mechanical brand-rename (founder already ruled "cloud-intelligence is the valid name").

---

### ADR-0263 — Observability emission contract

- **decision_atom:** Every oyatie workload emits all Three Pillars (structured JSON logs `oyatie/log/v1`, OTel traces with W3C Trace Context propagation, Prometheus/OpenMetrics metrics) with a MANDATORY `tenant_id` label + dotted sub-scope on every emission, exemplars linking metrics→traces, `audit_id` on every state-changing emission, PII scrubbed at the emission boundary (never storage), additive-only versioned schema with a deprecation handshake — stored on the Mimir+Loki+Tempo+ClickHouse substrate with a cross-cell aggregator.
- **domain:** observability
- **current_status:** Proposed (amends ADR-0042, ADR-0153, ADR-0186)
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — the emission contract is correct, necessary (closes the PR-#143 14-instance drift), and its chosen stack (Loki/Tempo/Mimir/Grafana) MATCHES the canonical ADR-0383 stack; ratify, but AMEND the dead `amends:` target.
- **governing:** n/a (it is itself the governing emission-contract ADR)
- **truth_flag:** PARTIAL/STALE-frontmatter — decision TRUE, but `amends: ADR-0042` (observability-stack-OTel+in-house-UI) — **ADR-0042 is `superseded` by ADR-0383** (confirmed on disk: `status: superseded, superseded_by: [ADR-0383]`). You cannot amend a superseded ADR; the `amends:` edge must re-point to **ADR-0383** (Loki/Tempo/Mimir/Grafana). The stack NAMES in the body are already correct (they match 0383) — only the front-matter citation is stale. Also names Kafka headers for traceparent propagation (§D-3) and ADR-0005 outbox (Kafka residue → Pulsar), and "Foundry-pipeline lane" / `oya:foundry:ci-agent:` metric examples (§D-7/D-11) — retired Foundry brand residue.
- **in_masterplan:** YES (gates `observability-emission-contract`, `tenant-label-presence`, `trace-context-propagation`, `audit-id-on-state-change`).
- **tensions:** (a) `amends` a superseded ADR (0042) — the sharpest front-matter drift in this slice; must re-point to 0383. (b) Twin deprecation-handshake doctrine with 0257/0258. (c) Sub-scope cardinality (≤10k tenants/µservice/cell) vs `oyatie.dev.<engineer>`/`oyatie.preview.<pr>` ephemeral tenants — mitigated by rollup-to-parent, but a real cardinality-budget risk.
- **hyperscaler_challenge:** Aligned. Charity-Majors Three-Pillars + W3C Trace Context + per-tenant Mimir/Loki/Tempo isolation + exemplars is exactly Google/Honeycomb/Grafana-Labs canon. The mandatory-`tenant_id`-on-every-emission rule is stronger than most hyperscalers but justified by the oyatie-is-a-tenant doctrine. No archive pressure.
- **ai_slop:** Low-moderate. The reverse-cross-referenced audit-event-class registry (§D-13, listing 46 classes from ADR-0297/0313/0319) is elaborate but plausibly real registry-contract work.
- **refinement:** Re-point `amends: ADR-0042` → `ADR-0383`; Kafka→Pulsar; strip `oya:foundry:*` / "Foundry-pipeline" brand residue (→ governance/intelligence per ADR-0347).
- **consensus_needed:** none on the decision; mechanical AMEND of the superseded-ADR amendment edge + brand/transport residue.

---

### ADR-0272 — Cookie consent + per-purpose analytics opt-in

- **decision_atom:** Every oyatie web/mobile property ships a Tier-1, substrate-owned (no third-party CMP) Consent Management Platform exposing exactly five canonical purposes (necessary/preference/statistics/marketing/personalization), cookie-less first-party analytics by default, no pre-checked opt-ins / no "Accept all" default, Global Privacy Control honored, WCAG 2.2 AA + dark-pattern-linted UI, per-jurisdiction "strictest-wins" overlay (GDPR/ePrivacy/PIPA/PDPL/CCPA-CPRA/LGPD/UK/CH), per-tenant policy override, and immutable signed consent records in the per-tenant audit chain.
- **domain:** compliance-residency (cross-cuts product-ux for the CMP surface; authz-policy for Cedar packs)
- **current_status:** Proposed
- **disposition:** AMEND
- **proposed_resolution:** RATIFY — privacy-by-default, pre-traffic install is exactly the right (cheapest) moment and the design is regulator-defensible; ratify, but AMEND the wrong ADR-0246 audit-chain citation.
- **governing:** n/a
- **truth_flag:** PARTIAL — decision TRUE, but §D-4 and §"Positive" cite **"the tenant's audit chain (ADR-0246 audit-chain substrate)"** — **ADR-0246 is `policy-engine-substrate-promotion`, NOT audit-chain** (audit-chain is ADR-0003 / ADR-0028, which this very ADR cites correctly elsewhere). WRONG cross-reference repeated 2–3×. Otherwise internally consistent. `enforcement_status: blocker-on-keystone-merge` is the only non-advisory enforcement in this slice (stronger posture than its peers).
- **in_masterplan:** YES (12 BLOCKER `oya gate validate` lanes; amends ADR-0099 data-class registry + ADR-0251 compliance packs).
- **tensions:** (a) Wrong ADR-0246 citation (mechanical). (b) D-6 "zero third-party tags" forecloses common SaaS UX (Intercom/Hotjar/Google-Maps/YouTube) — a real, deliberate cost owned under ADR-0211 in-house; not a conflict but a heavy product constraint to surface. (c) Pulls in ~9 new µservices (consent-ledger, cmp-runtime, etc.) — large scope for a Proposed ADR.
- **hyperscaler_challenge:** Questionable-to-aligned. The privacy SUBSTANCE is leading (strictest-wins, GPC, cookie-less default). BUT no hyperscaler builds its OWN CMP — Google/AWS/Microsoft and most regulated SaaS license OneTrust/Cookiebot/Usercentrics (which this ADR explicitly REJECTS in Alt-2 on in-house/ADR-0211 grounds). So a hyperscaler would likely BUY this, not build it. This argues the in-house-CMP decision is defensible-but-contrarian, not misaligned — keep, but flag the build-vs-buy as the live question.
- **ai_slop:** Low. Regulatory citations are specific and accurate; alternatives analysis is thorough (10 alts).
- **refinement:** Fix the ADR-0246→ADR-0003/0028 audit-chain citation; validate the 9-µservice scope against finops/staging reality.
- **consensus_needed:** "Build a bespoke in-house CMP (consent-ledger + cmp-runtime + overlay-resolver + GPC handler + dark-pattern-lint + i18n + vendor-registry + DSAR-bridge = ~9 µservices) under ADR-0211, or license a commercial CMP and own only the consent-ledger-of-record? Every named hyperscaler buys; this ADR builds."

---

## Chunk notes

**Systemic findings across source-32 (carry up to the masterplan-generation step):**

1. **Two repeated WRONG cross-references that must be AMENDED before generation:**
   - **ADR-0249 mis-cited as "Foundry dissolution, keystone #13"** in BOTH ADR-0254 and ADR-0255. On disk ADR-0249 = `multi-category-marketplace-doctrine`. The REAL foundry retirement is **ADR-0335 (Accepted)**. The absorption LOGIC is correct; only the governing-id is wrong. ADR-0255 should likely be re-bound as an amendment of ADR-0335 (MERGE-adjacent — both decide foundry→intelligence).
   - **ADR-0246 mis-cited as "audit-chain substrate"** in ADR-0272. On disk ADR-0246 = `policy-engine-substrate-promotion`; audit-chain is ADR-0003/0028. Mechanical fix.

2. **Stale `amends:` edge in ADR-0263** — it amends **ADR-0042, which is `superseded` by ADR-0383** (verified on disk). The body's stack names already match ADR-0383 (Loki/Tempo/Mimir/Grafana); only the front-matter edge is dead. Re-point `amends: → ADR-0383`. (Confirms keystone-map fault-line #6: trust the superseding ADR.)

3. **Retired-vocabulary residue is pervasive but cosmetic** (decisions remain TRUE):
   - **Kafka** named as transport in ADR-0253 (§D-14/15), ADR-0257 (§D-1/5/6), ADR-0263 (§D-3) — all STALE → **Pulsar 4.x + Oxia** (ADR-0377-kafka-to-pulsar, supersedes ADR-0005). The outbox PATTERN survives; only the broker name changes.
   - **Foundry** brand residue: ADR-0254 (`foundry-builder-prod`/`foundry-release-build` in `.oab` manifest), ADR-0255 (`oyatie.foundry.*` principals + "Foundry dissolution"), ADR-0258 (`oya.foundry.v1.CapabilityService` gRPC package + "Foundry" public surface), ADR-0263 (`oya:foundry:ci-agent:` metrics + "Foundry-pipeline lane"). All RETIRED → `cloud-intelligence` (consumer) / `governance` (CI) per ADR-0335/0347. Note ADR-0258 already uses the correct `oya-governance-api-semver` lane prefix — so the corpus is mid-transition.
   - **Correct vocabulary observed:** Valkey (ADR-0257 hot-cache, ✓ ADR-0336), `oya-governance-*` lane prefix (ADR-0258, ✓ ADR-0347).

4. **Three parallel "additive-only + deprecation-handshake" engines** in this slice — ADR-0257 (ontology schema), ADR-0258 (API generations), ADR-0263 §D-15 (log schema). Same doctrine, three distinct contract surfaces. Defensible as three peers, but the masterplan should record them as ONE doctrine with three bindings (and a founder question on whether to factor a shared kernel).

5. **Proposal accounting (no unaccounted Proposals):** of 7 ADRs, 6 are Proposed → all RATIFY (none DROP — every one is a genuine, current, non-conflicting substrate decision). 1 (ADR-0258) is Accepted/BINDING. Zero ARCHIVE, zero GARBAGE in this slice.

6. **Net disposition:** 7× AMEND (all because of mechanical cross-ref / vocabulary fixes), 0× KEEP-clean, 0× ARCHIVE, 0× SUPERSEDE, 0× UNCLEAR. The decisions are sound; the metadata/vocabulary is mid-migration. None of these blocks masterplan generation once the two wrong cross-refs (0249, 0246) and the dead `amends` (0042→0383) are corrected.

7. **LINUX-pilot interaction:** only ADR-0253 directly touches a LINUX fault-line (#3, isolation/runtime own-host vs assemble-substrate). The other six are SOURCE-only substrate doctrine with no LINUX collision beyond the guaranteed number-renumber on merge.
