# ADR Audit Artifact — source-24

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-24 (slice 162–168 of `ls ADR-*.md | sort`)
- **range / ADRs reviewed:** ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191, ADR-0192, ADR-0193 (7 ADRs)
- **cluster:** identity/authn substrate (0187–0191) + data-engine canonical (0192 vector, 0193 OLAP)
- **auditor:** ADR AUDITOR (coverage backfill)
- **date:** 2026-06-06
- **method:** keystone map read first; each file read fully; cross-checked against retired-vocab table + supersession graph + canonical-posture table.

---

### ADR-0187 — Canonical OIDC IdP: Zitadel primary; OIDC/SAML/SCIM/Passkeys/WebAuthn first-class

- **decision_atom:** Zitadel (Apache-2.0, Go single-binary, multi-tenant Instances/Organizations) is the canonical fleet-wide OIDC/SAML/SCIM 2.0/WebAuthn IdP, deployed one-Instance-per-sovereign-pack behind the `oya-shared-oidc-client-kernel` adapter so it is vendor-replaceable by an in-house `oya-identity-server` under concrete value-anchored triggers.
- **domain:** identity-authn
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP (minor AMEND-on-touch only — see truth_flag). Core decision is current, correct, non-conflicting and is the named governing ADR for the canonical-posture identity row.
- **proposed_resolution:** NA (Accepted, not Proposed)
- **governing:** n/a
- **truth_flag:** TRUE (one stale ref: `related:` still cites ADR-0183 as the live Cedar app-authz/Kyverno-admission ADR; per keystone §1.1 ADR-0183 is **Superseded by ADR-0379** (Kubewarden default admission), though Cedar app-authz split survives. The Zitadel decision itself is unaffected — this is a cross-ref hygiene nit, not a decision error.)
- **in_masterplan:** YES — names the canonical identity substrate the masterplan binds (FD-001 Tenant RBAC depends on this issuer); carries explicit phase/trigger planning_impact.
- **tensions:** None hard. Soft: the "in-house roadmap" Phase-2 `oya-identity-server` ambition is the same own-when-proven ratchet flagged in keystone §5.5 — consistent with SOURCE's staged-ownership posture (the trigger threshold, not the principle, is the only open variable). No LINUX collision on this axis (LINUX has no identity ADR).
- **hyperscaler_challenge:** ALIGNED. Google (Identity Platform), AWS (Cognito→in-house), Microsoft (Entra), Oracle (OCI IDCS) each run their own multi-tenant IdP behind stable OIDC/SAML/SCIM wire protocols — exactly this ADR's adapter-isolation + eventual-in-house trajectory. Does NOT argue for amend/archive.
- **ai_slop:** Low. Dense but substantive; the vendor comparison table is real and decision-bearing. Footnoted version pins (Zitadel chart v9.34.1 / v2.55) are the only recency-rot risk — gate `oya-check-vendor-recency` already owns that.
- **refinement:** On next touch, fix the ADR-0183→ADR-0379 cross-ref (and ADR-0182 Envoy/Istio refs are fine).
- **consensus_needed:** None contested. (If founder later mandates day-0 in-house identity, the Phase-0/2 split would need revisiting — but nothing today forces that.)

---

### ADR-0188 — Passkey / WebAuthn Level-3 substrate; TOTP fallback; SMS rejected

- **decision_atom:** WebAuthn Level 3 (Passkeys primary, hardware-key fallback, TOTP recovery-only) is the canonical phishing-resistant strong-auth substrate via `webauthn-rs`, with SMS/email-sole-factor/security-questions/non-number-matched-push forbidden per NIST SP 800-63B.
- **domain:** identity-authn
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP. Standards-conformant, no vendor substrate ("Phase 0 = Phase 2"), no conflict.
- **proposed_resolution:** NA
- **governing:** n/a
- **truth_flag:** TRUE. Self-aware that it introduces no vendor-replaceable substrate (W3C/FIDO standard + OSS commodity). Credential storage is plain Postgres (clean). No retired-vocab leakage.
- **in_masterplan:** PARTIAL — binds the auth-factor floor that ACR (ADR-0189) and edge/origin authz (ADR-0191) consume; not itself a top-line masterplan substrate row but a load-bearing identity-cluster invariant.
- **tensions:** None.
- **hyperscaler_challenge:** ALIGNED. Apple/Google/Microsoft/Cloudflare/GitHub/Stripe all ship Passkey-first + reject SMS — this ADR mirrors the current hyperscaler floor exactly. Does NOT argue for amend/archive.
- **ai_slop:** Low. The credential ladder, browser matrix, and attestation/AAGUID policy are concrete and correct. Slight over-elaboration (caBLE/conditional-UI pseudo-flows) but informative, not slop.
- **refinement:** None required. (Browser min-versions are a recency-rot item but low-stakes.)
- **consensus_needed:** None.

---

### ADR-0189 — Step-up authentication ACR classes (`routine`/`elevated`/`sensitive`/`critical`); ACR-bound Cedar gates

- **decision_atom:** Authentication strength is a controlled four-level ACR enum (routine<elevated<sensitive<critical) with explicit factor/session-age requirements per level, enforced as a Cedar `principal.acr_level >= AcrLevel::"x"` obligation at the waypoint that returns `step_up_required` below the per-action floor.
- **domain:** identity-authn (cross-cutting into authz-policy via the Cedar obligation)
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP, with AMEND-on-touch flag: the Cedar dependency rides on ADR-0183, which is now superseded by ADR-0379 — but the ACR floor lives in Cedar *app-authz* (the principle ADR-0379 explicitly retains), so the decision survives intact; only the citation is stale.
- **proposed_resolution:** NA
- **governing:** n/a (Cedar PDP governance now reads through ADR-0243/0246/0379, not ADR-0183 — for cross-ref correctness only)
- **truth_flag:** PARTIAL — TRUE decision with one STALE cross-ref (ADR-0183) and one retired-vocab leak: Alternatives §"One-shot re-authentication" cites *"the agentic-Foundry where the policy itself is the long-lived consent envelope."* `Foundry` is RETIRED → intelligence (ADR-0335/0347). Cosmetic prose-residue, not a decision defect (MFL-0002-class brand leakage).
- **in_masterplan:** PARTIAL — defines a fleet-wide policy invariant (ACR enum) the masterplan-bound Cedar gates depend on; carries advisory-lane planning_impact (`lean-a15`/`lean-a16`).
- **tensions:** Naming watch (NOT a conflict): the ACR `elevated/sensitive/critical` ladder is a DIFFERENT axis from LINUX ADR-0021's autonomy-tier T1–T4 and from the retired tenant `tier-system` (ADR-0329). Keystone §2/§5.2 already flag that these three "tier-like" axes must not be conflated — this ADR's ACR enum is clean and distinct.
- **hyperscaler_challenge:** ALIGNED. Stripe verified-mode, Google Workspace "recent sensitive action," AWS console MFA step-up are the cited references and the design is a faithful generalization. Does NOT argue for amend/archive.
- **ai_slop:** Low. Cedar snippets and step-up flow are concrete. The four-class enum is intentionally coarse (self-documented).
- **refinement:** On touch — (1) repoint ADR-0183→ADR-0379/0243; (2) scrub "agentic-Foundry" → "agentic-intelligence".
- **consensus_needed:** None contested.

---

### ADR-0190 — SCIM 2.0 inbound provisioning for enterprise tenants; pluggable adapter for non-SCIM HRIS

- **decision_atom:** The identity µservice serves its OWN per-tenant SCIM 2.0 (RFC 7643/7644) endpoint (`/scim/v2/{tenant}`) for inbound Users/Groups lifecycle from enterprise IdPs, write-through to Zitadel in Phase 0, with a pluggable `HrisAdapter` trait translating non-SCIM HRIS (Workday/BambooHR/Rippling) into internal SCIM ops.
- **domain:** identity-authn
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP. In-house-from-inception server, standards-bound, no conflict.
- **proposed_resolution:** NA
- **governing:** n/a
- **truth_flag:** TRUE. Clean. Phase-2 swap removes the Zitadel write-through hop; consumer SCIM clients see no change. No retired vocab.
- **in_masterplan:** PARTIAL — enterprise-tenant provisioning is a tenancy/identity invariant the masterplan binds (per-seat billing side-effects, tenant lifecycle via ADR-0175); not a top-line substrate row.
- **tensions:** None. (Coordinates with tenancy ADR-0175/billing per-seat metric — coherent, not conflicting.)
- **hyperscaler_challenge:** ALIGNED. "Serve our own SCIM, propagate to upstream IdP store" is explicitly the AWS Cognito / Google Workspace SCIM posture, and SCIM-as-table-stakes for enterprise B2B is correct. Does NOT argue for amend/archive.
- **ai_slop:** Low. Endpoint table, schema URNs, lifecycle-propagation table, and `HrisAdapter` trait are all concrete and correct.
- **refinement:** None required.
- **consensus_needed:** None.

---

### ADR-0191 — Edge authz tier (Envoy: IP/rate/WAF/bot/DDoS) vs origin authz tier (Istio waypoint Cedar PDP: identity/context/step-up/data-class); never overlap

- **decision_atom:** Authorization is split into two strictly-disjoint tiers — the edge (Envoy Gateway: IP/ASN/geo/rate/WAF/bot/DDoS, "knows packets") and the origin (Istio waypoint Cedar PDP: principal/action/resource/tenant/residency/ACR/data-class, "knows people") — with a lint gate forbidding either tier from reimplementing the other's concern.
- **domain:** authz-policy (cross-cutting into networking-mesh)
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP, with AMEND-on-touch: two retired-substrate leaks (Redis→Valkey) and the ADR-0183 cross-ref. Boundary-discipline decision itself is sound and current.
- **proposed_resolution:** NA
- **governing:** n/a for the decision; Redis→Valkey governed by ADR-0336; Cedar admission now ADR-0379 (sup. 0183).
- **truth_flag:** PARTIAL — TRUE boundary doctrine, but contains STALE retired-vocab: edge tier table says *"Redis-backed per-IP counters"* and origin tier table says *"per-tenant Redis cache"*. **Redis is RETIRED → Valkey (ADR-0336).** Also `related:`/cross-refs cite ADR-0183 (superseded by ADR-0379). Note: ADR-0193 (same chunk) correctly already says "Tier 3 Valkey" — so this is isolated drift in 0191, worth an AMEND.
- **in_masterplan:** PARTIAL — codifies the fleet-wide authz-tier invariant (and the `oya-check-authz-tier-discipline` advisory→blocker gate) the masterplan/security posture binds; not a standalone substrate row.
- **tensions:** None hard. Mild internal staleness only (Redis residue). Cleanly defers WAF/edge to ADR-0182/0157 and origin to Cedar — consistent with canonical posture §3 policy row.
- **hyperscaler_challenge:** ALIGNED. Explicitly mirrors Cloudflare (WAF/bot/DDoS edge + Access origin), AWS (CloudFront/WAF/Shield edge + IAM/Verified-Permissions/Cedar origin), Google (Cloud Armor edge + IAM Conditions origin). "Edge knows packets; origin knows people" is the correct hyperscaler separation. Does NOT argue for archive; the Redis residue argues for AMEND.
- **ai_slop:** Low. The boundary table ("if you find yourself wanting to…→enforce at…") is genuinely useful decision content, not filler.
- **refinement:** On touch — (1) Redis→Valkey in both rate-limit/cache rows (ADR-0336); (2) ADR-0183→ADR-0379/0243 cross-ref.
- **consensus_needed:** None.

---

### ADR-0192 — Vector database canonical: Milvus disaggregated cluster; pgvector adapter only for ≤10M-vector tenants

- **decision_atom:** Milvus 2.6.x (Apache-2.0, CNCF-graduated, four-plane disaggregated) is the canonical fleet-wide vector database behind the `oya-shared-vector-store-kernel` port, with pgvector permitted only as the embedded tier for ≤10M-vector tenants and an in-house `oya-vector-store-server` deferred behind value-anchored triggers.
- **domain:** data-engine-db (cross-cutting into intelligence-ai — embeddings/agent-memory)
- **current_status:** Accepted (2026-05-18); `supersedes: [ADR-0046]`
- **disposition:** KEEP (governing/canonical), with AMEND-on-touch for retired-vocab. This is the cited governing ADR for the data/storage vector row in keystone §3 and the supersession edge in §1.1 (ADR-0046 → ADR-0192). Decision is current and correct.
- **proposed_resolution:** NA
- **governing:** This ADR is itself governing (supersedes ADR-0046, which is the ARCHIVE target — already correctly flagged superseded on disk).
- **truth_flag:** PARTIAL — TRUE decision, but heavy RETIRED-vocab/path leakage: (1) `deciders: axis-foundry`, owner narrative, and ALL Milvus Helm/Kustomize/runbook/SLO paths are under **`microservices/foundry/...`** — `foundry` is RETIRED → **intelligence** (ADR-0335/0347); embedding-retrieval is an intelligence AI-workload, not "Foundry." (2) "Foundry-emitted vectors," "Foundry's embedding-model adapter," "Foundry agent retrieval" throughout. (3) Log-broker correctly canon = Pulsar 4.2 (good, post-ADR-0377), but still says "Kafka is supported but not canonical" — acceptable as compat note. (4) `oya-check-vector-store-discipline ... ADR-0135 aspirational` ref. None of this changes the TRUE decision (Milvus canonical) — it is brand/path residue requiring a rename pass.
- **in_masterplan:** YES — canonical data-substrate row; carries phase/trigger planning_impact, SLOs, capacity model, gate (`oya-check-vector-store-discipline` advisory→blocker). This is masterplan-binding.
- **tensions:** **Cross-side fault-line (keystone §5.1).** LINUX **ADR-0001** wants a from-scratch Rust multi-model engine *eliminating PostgreSQL/sqlx*, and LINUX **ADR-0020** flags Milvus as an UNSAFE deferral with a hard vector-count gate. This ADR picks best-of-breed Milvus NOW with in-house deferred — direct "own-the-substrate vs assemble-proven-OSS" tension with the pilot. Surface, do not resolve. (Both share the "own when proven" ratchet; the disagreement is the trigger threshold — LINUX wants day-0 ownership, SOURCE wants value-anchored Phase-2.)
- **hyperscaler_challenge:** ALIGNED (with nuance). NVIDIA AI Enterprise names Milvus; Cloudflare Vectorize/Uber Michelangelo mirror the disaggregated shape; AWS/Google/MS/Oracle each run their own vector substrate (the Phase-2 in-house parallel is honest). A hyperscaler WOULD adopt a mature OSS vector DB now and own later — exactly this ADR. The questionable-but-minor bit: four-plane Milvus ops surface (etcd+Pulsar+SeaweedFS) is heavy; Qdrant-single-binary is the Rust-fleet-aligned alternative the ADR explicitly keeps as a ≤10M adapter. Net: ALIGNED; does NOT argue for archive (argues only for the foundry→intelligence rename AMEND).
- **ai_slop:** Low-moderate. Very long but decision-dense (index-type pins, isolation primitives, sizing model, trigger conditions are all real). The "production-validation evidence" customer list is mild name-dropping but supports the trigger rationale. Not slop.
- **refinement:** On touch — mechanical `foundry`→`intelligence` rename across deciders/owner/paths/prose (ADR-0335/0347); keep Milvus decision verbatim.
- **consensus_needed:** **Founder question (contested cross-side):** "Does the pilot's day-0 own-the-DB-engine ambition (LINUX ADR-0001/0020) override SOURCE's Milvus-now / own-when-proven posture for vector retrieval — or do both coexist with Milvus as the staging substrate until the pilot engine proves out?" This is the sharpest data-tier tension in the corpus and needs an explicit ruling.

---

### ADR-0193 — OLAP analytics warehouse canonical: ClickHouse 26.3 LTS

- **decision_atom:** ClickHouse 26.3 LTS (Apache-2.0, coordinator-free via ClickHouse Keeper) is the canonical fleet-wide OLAP warehouse for tenant-facing dashboards, telemetry/billing rollups, and audit-log query — with Materialized Views as the default stream-processing tier (Flink escalation per ADR-0195) and an in-house `oya-olap-warehouse-server` (DataFusion/Arrow/Parquet) deferred behind value-anchored triggers.
- **domain:** data-engine-db (cross-cutting into observability — telemetry rollups)
- **current_status:** Accepted (2026-05-18)
- **disposition:** KEEP (canonical). Current, correct, the cited data/storage OLAP row. Minor AMEND-on-touch for a superseded cross-ref.
- **proposed_resolution:** NA
- **governing:** n/a
- **truth_flag:** PARTIAL — TRUE decision, with two staleness nits: (1) references **ADR-0042** as the live observability stack in Context/References — ADR-0042 is **Superseded by ADR-0383** (Loki/Tempo/Mimir/Grafana) per keystone §1.1; the "Prometheus remote-write … cold-tier alternative to Mimir" line is consistent with 0383 but the 0042 citation is stale. (2) "Kafka Engine integration … Pulsar exposes a Kafka protocol; ClickHouse Kafka engine connects via the Kafka wire protocol" — factually the KoP wire-compat path (consistent with ADR-0377-kafka-to-pulsar) so this is OK, not a Kafka-as-canonical regression. Correctly uses "Tier 3 Valkey" (good — no Redis residue here). Decision unaffected.
- **in_masterplan:** YES — canonical data-substrate row; carries phase/trigger planning_impact, SLOs, capacity model, gate (`oya-check-olap-tier-discipline` advisory→blocker), 7-year compliance-retention bindings. Masterplan-binding.
- **tensions:** Same family as ADR-0192's §5.1 fault-line: LINUX ADR-0001's "eliminate PostgreSQL / own the multi-model engine" posture pulls against SOURCE assembling best-of-breed (ClickHouse here). Softer than the vector case (no LINUX OLAP-specific ADR), but the own-vs-assemble breadth tension (keystone §5.5) applies. Surface, do not resolve.
- **hyperscaler_challenge:** ALIGNED. Cloudflare/Uber/Shopify/eBay/Plausible run ClickHouse at scale; AWS Redshift / Google BigQuery / MS Synapse / Oracle ADW / Snowflake each own their OLAP engine — the Phase-2 DataFusion-based in-house parallel is honest and correct. A hyperscaler WOULD pick ClickHouse-now / own-later. Does NOT argue for archive.
- **ai_slop:** Low-moderate. Long but decision-dense (cluster shape, use-case table, MV semantics, TTL/cold-tier, multi-tenancy primitives, triggers all concrete). Customer-reference lists are mild but support trigger rationale.
- **refinement:** On touch — repoint ADR-0042→ADR-0383 (observability stack) in Context/References; optionally annotate the "Kafka engine" line as KoP-wire-compat-over-Pulsar to pre-empt a false Kafka-regression flag.
- **consensus_needed:** None contested independently (rides the same data-tier own-vs-assemble founder question as ADR-0192; no separate decision needed).

---

## Chunk notes

**Shape of this chunk.** Two tight, internally-coherent clusters authored same-day (2026-05-18) by the same axes: the **identity/authn substrate** (0187 Zitadel IdP → 0188 WebAuthn → 0189 ACR step-up → 0190 SCIM → 0191 edge/origin authz split) and the **data-engine canonical pair** (0192 Milvus vector, 0193 ClickHouse OLAP). All seven are `Accepted`, all carry empty supersede edges except 0192 which correctly `supersedes: [ADR-0046]`. **No Proposed ADRs in this slice** — so no RATIFY/DROP dispositions are owed. **Disposition tally: 7 KEEP, 0 ARCHIVE/SUPERSEDE/MERGE/UNCLEAR.** Net quality is high; these are genuine, hyperscaler-aligned, well-reasoned decisions, not slop or garbage.

**Systemic staleness threads (AMEND-on-touch, not ARCHIVE):**
1. **ADR-0183 ghost-reference.** Four of the five identity ADRs (0187/0189/0191, plus 0187/0191 cross-refs) cite ADR-0183 as the live Cedar app-authz/Kyverno-admission ADR. Per keystone §1.1, **ADR-0183 is Superseded by ADR-0379** (Kubewarden default admission; Cedar app-authz split principle retained). The *decisions* are unaffected (they depend on the surviving Cedar-app-authz principle), but the citations are stale. A single mechanical cross-ref repoint (0183→0379/0243) fixes the cluster.
2. **Redis→Valkey residue (ADR-0336).** ADR-0191 still says "Redis-backed per-IP counters" / "per-tenant Redis cache." Redis is RETIRED → Valkey. Notably ADR-0193 (same chunk, same day) already says "Tier 3 Valkey" — so this is isolated drift in 0191 and an easy AMEND.
3. **`foundry` brand/path residue (ADR-0335/0347).** ADR-0192 is the worst offender: `axis-foundry` decider, "Foundry-owned" Milvus cluster, and every Helm/Kustomize/runbook/SLO path under `microservices/foundry/...`. `foundry` is RETIRED → **intelligence**. ADR-0189 has a one-line "agentic-Foundry" prose leak. None change the TRUE decision; all are MFL-0002-class brand-residue requiring a rename pass on touch.
4. **ADR-0042 observability cross-ref (ADR-0383).** ADR-0193 cites ADR-0042 (superseded by ADR-0383) as the live observability stack; the actual content (Mimir/Prometheus-remote-write) is already consistent with 0383, so it's a citation-only fix.

**Masterplan binding.** 0187/0192/0193 are top-line canonical-substrate rows the masterplan binds (identity issuer, vector DB, OLAP DB) and each carries explicit phase/trigger/SLO/gate planning_impact — they are exactly the kind of LIVE Accepted ADR the generated masterplan should ingest. 0188/0189/0190/0191 are load-bearing identity/authz invariants (auth-factor floor, ACR enum, SCIM contract, authz-tier discipline) with advisory-lane planning_impact — PARTIAL binding, keep.

**Cross-side fault-line surfaced (do not resolve here).** ADR-0192 (and softly 0193) sits directly on keystone §5.1: LINUX **ADR-0001** ("eliminate PostgreSQL," own a from-scratch multi-model engine) + LINUX **ADR-0020** (Milvus = UNSAFE deferral, hard vector-count gate) vs SOURCE's best-of-breed-now (Milvus/ClickHouse) + own-when-proven Phase-2. Both sides share the "own when proven" ratchet; the live disagreement is the **trigger threshold** (LINUX day-0 ownership vs SOURCE value-anchored Phase-2). The one founder question worth escalating from this chunk: *does the pilot's day-0 own-the-data-engine ambition override SOURCE's Milvus/ClickHouse-now staging posture, or do they coexist with the OSS substrate as staging until the pilot engine proves out?* This is the single highest-value contested decision touching this slice.

**Hyperscaler verdict (all 7):** aligned. Every decision matches what Google/AWS/Azure/Oracle actually do (own multi-tenant IdP behind stable wire protocols; Passkey-first + no-SMS; risk-tiered step-up; serve-own-SCIM; edge-packets/origin-people authz split; adopt mature OSS data engines now and own later). No decision in this chunk is misaligned; none argues for archive on hyperscaler grounds. The only hyperscaler-flavored questionable note is Milvus's four-plane ops weight vs a Rust-native single-binary (Qdrant) — already handled by the ADR keeping Qdrant as a ≤10M adapter.
