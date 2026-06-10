# ADR Audit — SOURCE, Chunk 1

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 1 of N
- **Range:** ADR-0001 … ADR-0007
- **ADRs actually reviewed:** ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007 (7 files, all read in full)
- **Auditor posture:** READ-ONLY. Keystone map consulted first. No audited doc modified. Cross-refs spot-verified on disk (ADR-0011 retains `oya-foundry-capability-kernel`; ADR-0055/0058 exist; ADR-0006 carries a self-contradicting rename line).

This is the **Foundation cluster** — the six-substrate cohesion core. These seven ADRs are the most load-bearing in the entire corpus: nearly every later ADR (and the masterplan itself) is downstream of them. Most are TRUE in spirit but carry stale naming (`foundry`), inconsistent front-matter status (Proposed vs Accepted for what is clearly load-bearing-and-built), and one (ADR-0005) is a confirmed retired-in-fact decision per the keystone map.

---

### ADR-0001 — Adopt the cohesion thesis: one product across a flat microservice catalog joined at six shared substrates

- **decision_atom:** Oyatie is one cohesive product over a flat (ungrouped) catalog of microservices, joined at exactly six shared, single-owner substrates — tenancy, identity, audit-chain, capability-registry, agent-runtime, autonomy-ceiling — and no microservice may re-implement any of the six (CI-enforced by a cohesion fitness lane).
- **current_status:** `accepted` (front-matter) / "Accepted" (body). Consistent.
- **disposition:** **AMEND** (the decision is canonical and TRUE; the prose carries retired vocabulary that must be reconciled).
- **governing:** Not superseded. It is itself a governing keystone. Naming reconciliation governed by ADR-0335 (foundry retired), ADR-0347 (foundry→governance lane rename), ADR-0362 (grouping retirement, consistent with this ADR).
- **truth_flag:** **PARTIAL** — the cohesion thesis is TRUE and foundational; but the catalog block hard-codes a retired substrate brand (`Foundry (internal-only): grit, icm, ...` and `oya-foundry-capability-kernel` / `oya-foundry-runtime-*` / `oya-foundry-policy-kernel` as the substrate owners). Per keystone §2, `foundry`, `grit`, `icm` are all RETIRED; `Wave` framework and "Foundry" brand are dead. The six-substrate *invariant* is true; the *names* are stale.
- **in_masterplan:** **PARTIAL** — the cohesion thesis is the spine of MASTERPLAN.md's "one tenancy / one audit / one identity / one ceiling" framing, so the decision is reflected. But the ADR carries **no planning front-matter** (`planning_impact`, `supersedes`, `masterplan_ref`, `deliverables`) — only `id/status/doc_status`. Under either masterplan reading (authored vs generated) this ADR is under-instrumented; per drift-prevention's 8.8%-binding finding, this keystone is almost certainly in the unbound 91%.
- **tensions:**
  - vs **ADR-0058** (flat catalog / Product-Groups-retired) and **ADR-0362** (full grouping retirement): consistent in direction, but ADR-0001's `Related:` list points at 0058/0059 while the live grouping-retirement governor is 0362 — cross-ref is stale/incomplete.
  - vs **ADR-0335** (foundry retired → intelligence) and **ADR-0347** (lane rename): the substrate-owner crate names in the Decision section (`oya-foundry-*-kernel`) are retired-vocab leakage; the keystone-map "valid name" is cloud-intelligence + governance.
  - vs **LINUX fault-line #5** ("own everything" breadth): the cohesion thesis is the source-side justification for owning substrates; aligns with linux's OWN_DAY0 instinct but source frames it as cohesion-of-assembled-OSS, not build-from-scratch.
- **hyperscaler_challenge:** **aligned.** Google (one Spanner/one IAM/one Borg), AWS (one IAM/one CloudTrail), Azure (one AAD/one ARM) all run exactly this "single substrate, many products on top" model. A hyperscaler would absolutely make this call. The breadth (six substrates day-0 for a startup) is more aggressive than any of them shipped at seed stage, but the *architecture* is hyperscaler-canonical. Argues for **amend** (naming), not archive.
- **ai_slop:** Mild. "collapses that drift to zero" / "mechanically true" are confident-precision flourishes. The phrase "every feature and product ... independent, modular, and integrate-able like microservices in clean architecture" is filler-ish. The Rust `ForbiddenPattern` enum is illustrative-not-binding (fine). "Oyatie is the same product as Bominal, built in parallel" is an unexplained load-bearing claim dropped without a governing ADR — borderline fabricated-context.
- **refinement:** (1) Replace `oya-foundry-*` substrate-owner names with `oya-intelligence-*` / `oya-governance-*` per 0335/0347. (2) Add planning front-matter binding to masterplan. (3) Fix `Related:` to cite 0335/0347/0362. (4) Drop or ground the "same product as Bominal" claim. (5) Reconcile the `Wave integration framework` reference (Wave names retired per GLOSSARY).
- **consensus_needed:** **yes** — "Is the **six-substrate** count canonical and frozen, or has it drifted (e.g. is 'single agent runtime' still a substrate now that Foundry is absorbed into intelligence, and does 'single capability registry' survive ADR-0011 as-is)? Founder ruling needed because this count is the masterplan's organizing invariant."

---

### ADR-0002 — Establish the Tenant and Identity kernel as the single substrate every axis consumes

- **decision_atom:** A single co-located kernel pair (`oya-tenancy-kernel` owning the `Tenant` shape; `oya-identity-kernel` owning principals/sessions/credentials with Cedar-backed RBAC/ABAC and STS-only short-lived credentials) is the sole authority for tenancy and identity; region-binding is immutable, regulatory-packs are set-valued and inherited by every axis, and all federated IdPs ship as regional-pack seams.
- **current_status:** `proposed` (front-matter) / "Proposed" (body).
- **disposition:** **AMEND** (sound and load-bearing, but status is stale-as-Proposed for a substrate the masterplan declares production-depth, and it embeds the retired `tier` vocabulary).
- **governing:** Not superseded. Canonical posture intact (keystone §3 Identity row points at Zitadel/OIDC ADR-0187–0190 as the *implementation* substrate; this ADR is the *kernel-shape* decision and remains compatible). Naming `autonomy_tier` is the LIVE T1–T4 axis (NOT the retired tenant-tier of ADR-0329) — do not archive on that basis.
- **truth_flag:** **TRUE** (decision is correct and current). One **STALE** sub-element: the `Tenant.autonomy_tier: AutonomyTier // T1..T4 ceiling for foundry agents` comment uses retired `foundry` brand. The model is right; the comment is stale.
- **in_masterplan:** **PARTIAL** — MASTERPLAN.md FD-001 = "Tenant RBAC at full production depth (NOT a preview)", which is *this* decision at production depth — yet the ADR still reads `status: proposed`. That is a direct masterplan↔ADR status contradiction. No planning front-matter present.
- **tensions:**
  - vs **MASTERPLAN.md FD-001:** masterplan says production-depth; ADR says Proposed. Status drift — masterplan should win, ADR should be promoted to Accepted.
  - vs **SOURCE ADR-0187 (Zitadel primary OIDC IdP):** this ADR says "Cedar-backed RBAC/ABAC + STS" and "federated IdPs ship as regional-pack seams" — needs an explicit cross-ref that Zitadel (0187) is the concrete IdP realizing the `oya-identity-kernel` seam, else a reader can't tell if identity is owned-kernel or Zitadel-fronted. Reconcilable, but unbound.
  - vs **LINUX ADR-0021 (owned, Cedar-compatible policy):** linux positions an owned successor to the Cedar model this ADR depends on. Own-vs-reuse tension (keystone fault-line #2), not a contradiction.
  - **Alternative-B prose garble:** "KCminimum-shippable-tier/CSAP constraints" reads like a failed find-replace of a retired term (likely "KC-tier"/"K-ISMS") — a genuine WRONG string fragment.
- **hyperscaler_challenge:** **aligned.** Immutable region-binding + residency-class as identity + per-region IdP packs is exactly the GCP/AWS sovereign-region and Azure-sovereign-cloud model. STS-only short-lived creds = AWS STS / GCP Workload Identity / Azure Managed Identity doctrine. A hyperscaler makes this call. Argues for **amend** (promote status, fix stale strings), not archive.
- **ai_slop:** Low-moderate. The corrupted "KCminimum-shippable-tier" token is an internal-contradiction/garble signal. "the single most-reviewed kernel in the repository, by design" is rhetorical flourish. Open-questions Q1–Q4 are genuinely useful (not slop).
- **refinement:** (1) Promote `status: proposed` → `accepted` to match FD-001. (2) Fix the "KCminimum-shippable-tier" garble. (3) Replace "foundry agents" comment with "intelligence agents". (4) Add explicit cross-ref to ADR-0187 (Zitadel) clarifying owned-kernel-fronts-Zitadel vs owns-IdP. (5) Add planning front-matter / `masterplan_ref: FD-001`.
- **consensus_needed:** **yes** — "Does `oya-identity-kernel` **own** the IdP, or **front** Zitadel (ADR-0187)? FD-001 production-depth needs this resolved: build-vs-buy on the single most-regulated substrate."

---

### ADR-0003 — Audit chain and evidence emission as the single tamper-evident record-keeping substrate

- **decision_atom:** Every regulated event in every axis lands in a single append-only, BLAKE3 hash-chained, per-tenant-sharded audit log with periodic cross-tenant Merkle-root anchoring (Rekor-anchored, published to a trust portal), enforcing the PRD hard-zero "no egress without consent receipt" and ≤4-hour regulator evidence regeneration; DSR erasure is satisfied by KMS key-shred + invalidation pointer without breaking chain append-only-ness.
- **current_status:** `proposed` (front-matter) / "Proposed" (body).
- **disposition:** **AMEND** (decision is excellent and TRUE; promote status; reconcile minor naming and the persistence-substrate reference).
- **governing:** Not superseded. Compatible with canonical posture. One downstream reconciliation: postgres adapter is named `oya-audit-chain-adapter-postgres-*` — consistent with source's Postgres+pgcat relational posture (ADR-0179), and NOT in conflict with linux ADR-0001's "eliminate PostgreSQL" (that's a linux-side divergence, surfaced under fault-line #1, not a source defect).
- **truth_flag:** **TRUE.** No retired vocabulary in the core decision. Only stale item: Open-Question Q4 owner is `foundry` ("trust portal") — retired owner brand; trust-portal now lives under intelligence/governance.
- **in_masterplan:** **PARTIAL** — the audit-chain substrate is part of the cohesion claim the masterplan asserts, but the ADR has no planning front-matter and the PRD-metric linkage ("zero egress", "≤4h regen") is not bound back into masterplan.json as a tracked deliverable.
- **tensions:**
  - vs **ADR-0005 (Kafka eventing):** ADR-0003's audit emitter "consumes these directly" from the eventing backbone — ADR-0005 is retired-in-fact (Kafka→Pulsar, ADR-0377). The audit chain's transport dependency therefore inherits 0005's staleness: the audit-emission path should now reference Pulsar+Oxia, not Kafka. **Cross-chunk tension** (governing ADR-0377-kafka-to-pulsar).
  - vs **ADR-0383 (Loki/Tempo/Mimir observability):** none — audit-chain (tamper-evident regulatory record) is correctly distinct from observability telemetry. Good separation.
  - Q4 owner `foundry` → retired.
- **hyperscaler_challenge:** **aligned, with a caution.** AWS CloudTrail + QLDB (immutable ledger), GCP Cloud Audit Logs, Azure Monitor all do tamper-evident per-tenant audit. The hash-chain + Merkle-root + Rekor anchoring is closer to what AWS QLDB / Azure Confidential Ledger ship — so a hyperscaler *does* make this call. Caution: hyperscalers generally **buy/operate** the ledger substrate rather than hand-roll BLAKE3 chaining in product code; the "in-house kernel" framing is the one place a hyperscaler might push back (operability cost). Argues for **amend** (consider whether the ledger is owned or substrate-backed), not archive.
- **ai_slop:** Low. Strong, specific ADR. "~1–2 ms hot-path" is plausible precision, not fabricated. The required-emitters table still lists "Foundry — runtime / Foundry — engineering platform" axis labels (retired brand) — minor.
- **refinement:** (1) Promote status to Accepted. (2) Re-point the event-transport dependency from ADR-0005/Kafka to ADR-0377/Pulsar. (3) Rename "Foundry —" axis labels to intelligence/governance. (4) Q4 owner foundry→intelligence. (5) Add planning front-matter + bind the two PRD hard metrics as masterplan deliverables.
- **consensus_needed:** **no** — decision is sound; reconciliation is mechanical. (Soft: own-vs-buy the ledger is worth a founder note but is not blocking.)

---

### ADR-0004 — Plane separation across control / data / analytics with catalog-declared plane class

- **decision_atom:** Every surface declares exactly one of three planes (Control / Data / Analytics) in its catalog record; cross-plane calls must be explicitly declared (ProjectionRead | EventReplay | ReadOnlyApi) with recorded justification, and a CI lane hard-fails undeclared cross-plane edges — control never reads the data-plane store directly, data never writes the control store directly, analytics never writes operational stores.
- **current_status:** `proposed` (front-matter) / "Proposed" (body).
- **disposition:** **KEEP** (current, correct, non-conflicting, well-formed) — with a recommended status promotion (so technically a light AMEND, but the decision needs no reconciliation).
- **governing:** Not superseded. No governing ADR overrides it. Cleanly compatible with ADR-0001/0005/0011/0015.
- **truth_flag:** **TRUE.** No retired vocabulary in the decision. The plane×axis matrix uses "Foundry" as an axis label (row 4) — minor stale-brand leakage, not a truth defect.
- **in_masterplan:** **PARTIAL** — plane class is asserted as a cross-microservice contract field (DESIGN §10) the masterplan relies on, but no planning front-matter; binding status unknown/likely unbound.
- **tensions:**
  - vs **ADR-0005:** declares events as the cross-plane mechanism, inheriting the Kafka→Pulsar staleness (transport only; the plane *contract* is unaffected). Low-severity cross-chunk tension.
  - vs **ADR-0015 (architectural-flattening, PARTIAL-superseded by ADR-0131):** ADR-0004 cites ADR-0015's kernel/domain/app/api/worker/adapter role mapping; since 0015 is partially superseded, the cross-ref should also acknowledge ADR-0131 (per-µsvc flat layout). Minor stale cross-ref.
  - "Foundry" axis label in the matrix — retired brand.
- **hyperscaler_challenge:** **aligned — strongly.** The ADR itself cites this: "AWS, Google, Azure, and every mature cloud provider has the three-plane model." Correct — control-plane/data-plane separation is industry-canonical (and analytics-plane = the standard OLTP/OLAP split). A hyperscaler unambiguously makes this call. Argues to **keep**.
- **ai_slop:** Very low. Tight, well-reasoned, evidence-anchored. Minor: "all all microservices" typo (Context para 1) and "every operational incident in the legacy corpus has a plane-mix root cause" is an unfalsifiable sweeping claim (mild hedging-inverse / overclaim).
- **refinement:** (1) Promote status. (2) Fix "all all" typo. (3) Add ADR-0131 to the ADR-0015 cross-ref. (4) Rename "Foundry" matrix axis. (5) Soften the "every incident" claim to cite specific LEDG entries.
- **consensus_needed:** **no.**

---

### ADR-0005 — Eventing backbone on Apache Kafka with outbox pattern, CloudEvents envelope, per-tenant/per-cell partitioning

- **decision_atom (as-written):** Apache Kafka (KRaft) is the single eventing backbone with transactional-outbox emission, CloudEvents 1.0 envelope, Protobuf payloads, in-house schema registry, and `(tenant_shard, cell_id)` partitioning. **(as-corrected by canon):** the *broker choice* is retired — Pulsar 4.x + Oxia (KoP wire-compat) is canonical per ADR-0377; the **outbox + CloudEvents + Protobuf + in-house-registry + per-tenant/per-cell partitioning** decisions survive.
- **current_status:** `proposed` (front-matter / body) — but **retired-in-fact** per keystone map.
- **disposition:** **ARCHIVE** (broker decision superseded) / partial **SUPERSEDE**. Archive the Kafka-broker selection; the outbox-pattern + envelope + partitioning sub-decisions should be re-homed (folded forward) into the superseding eventing ADR rather than lost.
- **governing:** **ADR-0377-kafka-to-pulsar-via-kop** (`Accepted`, supersedes ADR-0005) → with ADR-0195 / ADR-0397 in the eventing-canon cluster (keystone §1.1 and §1.2).
- **truth_flag:** **PARTIAL** — the *broker* (Kafka) is **STALE/WRONG** as current truth (Pulsar is canonical). The *patterns* (outbox, CloudEvents, Protobuf, in-house registry, per-tenant/per-cell key) remain **TRUE**. Note the irony: ADR-0005's own Alternative-D **rejects Apache Pulsar** ("smaller community ... Kafka's tooling more mature") — and canon later **chose Pulsar anyway**. The rejection reasoning is now falsified by the corpus.
- **in_masterplan:** **NO** (as a current decision) — masterplan should reflect Pulsar via ADR-0377, not Kafka via ADR-0005. The ADR's stale `status: proposed` front-matter (vs canon's "superseded") is exactly keystone §6 fault-line #6 (supersession drift): trust the superseding ADR.
- **tensions:**
  - vs **ADR-0377-kafka-to-pulsar (Accepted):** direct supersession — ADR-0005 still reads `proposed`, never updated to `superseded_by: ADR-0377`. Stale front-matter; must be reconciled on any merge.
  - vs **ADR-0003 / ADR-0004:** both depend on "the eventing backbone" for audit-emission / cross-plane mechanism — both transitively reference a retired broker. The dependents need re-pointing.
  - **Owner `foundry`** (eventing kernel) — retired brand.
  - **Self-falsifying Alternative D:** rejects the option canon ultimately adopted.
- **hyperscaler_challenge:** **questionable (for the as-written Kafka choice).** Hyperscalers do NOT self-host Kafka as a startup substrate — they use managed (AWS MSK / Kinesis, GCP Pub/Sub, Azure Event Hubs). The ADR's "managed Kafka via cloud microservice" mitigation half-admits this. The license-driven *reasoning* (Apache-2-only, no Confluent/Redpanda BSL) is sound and hyperscaler-compatible, but the specific Kafka-self-host conclusion is the kind a hyperscaler would not make; the corpus's own pivot to Pulsar validates the challenge. Argues for **archive** (already done by 0377).
- **ai_slop:** Low. Well-structured. The slop here is *structural*, not textual: a fully-detailed, code-bearing `Proposed` ADR that the corpus has already superseded but never marked — the most dangerous kind of stale doc because it reads authoritative.
- **refinement:** (1) Set front-matter `status: superseded`, `superseded_by: [ADR-0377-kafka-to-pulsar-via-kop]`. (2) Extract the still-true patterns (outbox/CloudEvents/Protobuf/partitioning) and confirm they're carried in ADR-0377; if not, fold them. (3) Re-point ADR-0003/0004 transport references. (4) Owner foundry→intelligence.
- **consensus_needed:** **no** (supersession is already decided by canon) — but flag for the synthesis that this is a textbook stale-front-matter case the masterplan-backfill must NOT capture at face value.

---

### ADR-0006 — Ontology as the engine-enforced typed-entity layer with per-property tier classification

- **decision_atom:** A single typed-entity layer ("Ontology", Palantir-shaped) is the sole information layer: every entity carries TenantId + ObjectId + per-property `PropertyTier` (scalar/vector/timeseries/geo/ciphertext/struct) + per-property `data_class`, with engine-enforced per-tenant Postgres RLS isolation and an audit-chain emission hook on every mutation; property-tier loosening requires explicit human approval.
- **current_status:** `accepted` (front-matter) / "Accepted" (body).
- **disposition:** **KEEP** (canonical and TRUE) — with a mandatory **AMEND** to fix a literal self-contradiction in the prose.
- **governing:** Not superseded. Naming governed by ADR-0055 (object-graph→ontology rename, `Accepted`) and ADR-0122 (ontology-crate-rename per keystone §1.1). This ADR is the canonical Ontology decision.
- **truth_flag:** **PARTIAL** — decision TRUE, but the document contains a **WRONG/GARBAGE sentence**: line 11 `(rewritten 2026-05-13 — "Ontology" renamed to "Ontology")` and line 22 `"Ontology" was the prior name for this layer. Per session decision 2026-05-13, it is renamed to **Ontology**` — both are **A renamed to A**, an internal contradiction caused by a find-replace that overwrote the old term ("Object Graph") on both sides. Per ADR-0055 the true statement is "**Object Graph** renamed to Ontology." This is a concrete garbage artifact the founder warned about.
- **in_masterplan:** **PARTIAL** — Ontology is a named layer in the cohesion catalog (ADR-0001) and is part of the masterplan's information-layer story, but no planning front-matter on this ADR; binding likely absent.
- **tensions:**
  - vs **ADR-0055 (Object Graph → Ontology):** ADR-0006's broken "Ontology renamed to Ontology" prose contradicts the source-of-truth rename ADR, which correctly says Object-Graph→Ontology. Direct internal contradiction.
  - vs **ADR-0046/0192 (vector store: pgvector ≤10M → Milvus >10M):** ADR-0006's `vector` tier says "pgvector → in-house HNSW/IVF as scale demands" — but canon (ADR-0192, supersedes ADR-0046) is **Milvus** as the canonical vector store >10M, not "in-house HNSW/IVF." The Ontology vector-adapter roadmap is stale vs the canonical vector decision. **Cross-chunk tension** (governing ADR-0192).
  - vs **LINUX ADR-0001 ("eliminate PostgreSQL") / ADR-0020 (Milvus unsafe-deferral):** ADR-0006 leans hard on Postgres (Citus, TimescaleExtension, PostGIS, RLS) — the sharpest collision point with linux's own-DB posture (keystone fault-line #1). Surface only.
  - "Bominal ADR-0106/0107/0132 inherited" cross-refs — inherited-doc provenance, fine but unverifiable here.
- **hyperscaler_challenge:** **questionable.** Palantir Foundry's Ontology is the explicit model and it is a real, sophisticated pattern — but Palantir is the only hyperscaler-class vendor that ships it; AWS/GCP/Azure do NOT offer a unified typed-entity "ontology" layer (they offer building blocks: Lake Formation, BigQuery, Purview). So "would a hyperscaler make this call?" is genuinely mixed: it's a Palantir bet, not an AWS/GCP/Azure bet. The per-property-tier + RLS + data-class enforcement is sound; the all-in-one-Ontology framing is the ambitious part. Argues for **keep but scrutinize scope** (not archive) — and note the vector-tier roadmap needs amend to Milvus.
- **ai_slop:** **Moderate-high on one axis:** the "Ontology renamed to Ontology" double is a textbook find-replace artifact (fabricated/garbled precision). Otherwise the ADR is substantive. The `Naming justification (BNF v4.1)` block is real, not slop.
- **refinement:** (1) **Fix the rename sentences** to "Object Graph renamed to Ontology" (lines 11 and 22) per ADR-0055. (2) Update `vector` tier adapter from "in-house HNSW/IVF" to **Milvus** (ADR-0192, >10M) / pgvector (≤10M). (3) Add planning front-matter. (4) Verify the Bominal inherited-ADR refs resolve.
- **consensus_needed:** **no** (the rename is settled by ADR-0055; just a doc fix) — though the Postgres-heavy Ontology vs linux own-DB is a founder-level tension to carry into synthesis (not this ADR's question to answer).

---

### ADR-0007 — Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling (T1–T4) with per-capability runtime enforcement

- **decision_atom:** Cedar (Apache-2, formally-verified evaluator) is the sole RBAC/ABAC engine across all axes; persona tiers T1–T4 are the autonomy-ceiling scale; every capability declares `autonomy_tier_required` and the runtime gate hard-fails any invocation exceeding the tenant's accepted ceiling, composing Cedar allow + ceiling check + per-class data-use check + T3 step-approval + audit emission in one evaluation path.
- **current_status:** `proposed` (front-matter) / "Proposed" (body).
- **disposition:** **AMEND** (canonical and TRUE; promote status, fix retired naming, add the now-canonical admission-layer cross-ref).
- **governing:** Not superseded. Cedar-as-universal-gate is reaffirmed by canon (ADR-0243 Cedar universal gate, ADR-0246 policy-engine substrate promotion). Admission-layer companion is now **ADR-0379 (Kubewarden default admission, supersedes ADR-0183's Kyverno)** — ADR-0007 (app-authz PDP) and ADR-0379 (k8s admission) are the two complementary halves of the policy posture (keystone §3 Policy row). The T1–T4 autonomy scale is LIVE and distinct from the retired tenant-tier (ADR-0329).
- **truth_flag:** **TRUE.** Core decision correct and current. Stale items: owner field includes `foundry`; runtime gate crate is `oya-foundry-runtime-policy-*` (retired brand); Q3/Q4 owners are `foundry`. Names stale, decision sound.
- **in_masterplan:** **PARTIAL** — Cedar + autonomy ceiling is part of FD-001's RBAC-at-production-depth and the masterplan's "single autonomy ceiling" substrate, so reflected in spirit, but `status: proposed` contradicts production-depth, and no planning front-matter binds it.
- **tensions:**
  - vs **ADR-0379 (Kubewarden admission) / ADR-0183 (superseded Cedar-vs-Kyverno split):** ADR-0007 predates the admission-layer decision; it should cross-ref ADR-0379 to make clear Cedar = app-authz PDP while Kubewarden = k8s admission. Missing cross-ref, not a conflict.
  - vs **LINUX ADR-0021 (owned, Cedar-compatible, tier-aware policy language):** linux builds an owned successor that extends Cedar's PARC model + Lean soundness and adds autonomy-tier T1–T4 as first-class — i.e. linux is the owned port of *this exact* ADR's model. Own-vs-reuse tension (keystone fault-line #2); compatible direction, surface it.
  - vs **ADR-0002:** ADR-0002 also asserts "Cedar is the sole authoritative AuthZ engine" — consistent, but the two ADRs co-own the Cedar decision; ownership/site of record should be deduped (0007 should be the canonical Cedar ADR; 0002 should defer).
  - `foundry` owner/crate names — retired.
- **hyperscaler_challenge:** **aligned.** AWS literally authored Cedar (and AVP — Amazon Verified Permissions runs it); a hyperscaler not only makes this call, one of them *built the engine*. ABAC + formally-verified evaluator + declarative policy is GCP IAM Conditions / Azure ABAC territory. The graded T1–T4 autonomy ceiling for *agents* is ahead of where hyperscalers have shipped, but directionally where agent-platforms (incl. AWS Bedrock Agents guardrails) are heading. Strongly argues to **keep/amend**, never archive.
- **ai_slop:** Low. Concrete Cedar + Rust examples, real tier semantics, useful open questions. The "prior decade of multi-product engineering shows..." is an unfalsifiable appeal but minor. No fabrication.
- **refinement:** (1) Promote status to Accepted. (2) Replace `oya-foundry-runtime-policy-*` and `foundry` owners with intelligence/governance names. (3) Add ADR-0379 cross-ref (admission layer) + ADR-0243/0246 (Cedar-universal-gate reaffirmation). (4) Dedupe Cedar ownership vs ADR-0002 (make 0007 canonical). (5) Add planning front-matter / `masterplan_ref`.
- **consensus_needed:** **yes** — "Cedar as the **adopted** engine (ADR-0007/0243) vs the LINUX **owned, Cedar-compatible** policy language (linux ADR-0021): does the masterplan adopt Cedar long-term, or is Cedar the vendored adapter pending an owned port? This is the policy-substrate own-vs-reuse ruling and it is load-bearing for both repos."

---

## Chunk notes for synthesis

**This is the keystone cluster.** ADR-0001…0007 are the six-substrate cohesion core that the entire 346-ADR corpus and the masterplan are built on. They are, in substance, the most TRUE and most masterplan-ready decisions in the corpus — which is exactly why their defects matter most.

**Pattern 1 — Status drift (the dominant defect).** 5 of 7 read `status: proposed` (0002, 0003, 0004, 0005, 0007) while MASTERPLAN.md FD-001 declares this substrate at "full production depth (NOT a preview)" and the corpus treats them as built/governing. Only ADR-0001 and ADR-0006 are `accepted`. This is a systemic front-matter-vs-reality gap: the foundational substrates are marked Proposed years into the build. **For the masterplan-backfill, treat these as effectively-Accepted-and-load-bearing, but flag every one for status promotion.** This directly feeds the founder's open question — if masterplan is *generated from ADR front-matter* (planning-ssot-consolidation design), these stale `proposed` statuses would generate a WRONG masterplan; if masterplan is *authored authority* (drift-prevention design), the ADRs need to be bound back and their status reconciled. Either reading, status drift is the #1 cleanup.

**Pattern 2 — Retired-vocabulary leakage (`foundry`) is corpus-wide and starts here.** Six of seven ADRs leak the retired `foundry` brand in crate names (`oya-foundry-capability-kernel`, `oya-foundry-runtime-*`, `oya-foundry-policy-kernel`, `oya-foundry-runtime-policy-*`), owner fields (`foundry`), axis labels ("Foundry — runtime"), and code comments ("foundry agents"). Per keystone §2 + ADR-0335/0347, the live names are **cloud-intelligence** (consumer AI) + **governance** (CI/gates). The foundation cluster predates the retirement and was never reconciled. Confirmed on disk: ADR-0011 still names `oya-foundry-capability-kernel` too — so the substrate-owner naming is consistently stale across the foundation set, not a one-off.

**Pattern 3 — No planning front-matter anywhere in the cluster.** All seven carry only `id/status/doc_status`. None carries `planning_impact`, `supersedes`, `superseded_by` (in front-matter; some use body blockquotes), `deliverables`, or `masterplan_ref`. This is the concrete face of drift-prevention's "8.8% ADR binding" finding — the *most* load-bearing ADRs are in the unbound 91%. Whatever the founder decides on authored-vs-generated, these seven must be instrumented first.

**Pattern 4 — Two concrete garbage/contradiction artifacts (founder's "plain wrong" warning, confirmed).**
- **ADR-0006:** "Ontology renamed to Ontology" (×2) — a find-replace that destroyed both sides of an Object-Graph→Ontology rename. Literally self-contradicting; the truth is in ADR-0055.
- **ADR-0002:** "KCminimum-shippable-tier/CSAP constraints" — a corrupted token (failed find-replace of a retired tier/KC term). 
These are the only outright-garbage strings in the cluster, but they're real and quotable.

**Pattern 5 — ADR-0005 is the one genuine ARCHIVE.** It is a fully-detailed, code-bearing, authoritative-reading `Proposed` ADR that the corpus already superseded (Kafka→Pulsar, ADR-0377) but never marked `superseded`. It is the cluster's clearest stale-front-matter trap and it poisons ADR-0003/0004, which depend on "the eventing backbone." Its own Alternative-D *rejected Pulsar*, which canon then adopted — the rejection reasoning is falsified. The masterplan-backfill must capture **Pulsar via ADR-0377**, never Kafka via ADR-0005. The survivable sub-decisions (outbox / CloudEvents / Protobuf / in-house registry / per-tenant-per-cell partitioning) should be confirmed-carried in the superseding ADR.

**Pattern 6 — Stale cross-refs to superseded peers.** ADR-0004 cites ADR-0015 (PARTIAL-superseded by ADR-0131) without acknowledging 0131. ADR-0006's vector tier cites "in-house HNSW/IVF" instead of the canonical Milvus (ADR-0192, supersedes ADR-0046). ADR-0001's Related list points at 0058/0059 but the live grouping-retirement governor is 0362. The foundation cluster's cross-ref graph was frozen at ~ADR-0059 and never re-pointed to the 0300s/0400s governors.

**Cross-chunk tensions to carry forward:**
- **Eventing:** ADR-0005 (Kafka, here) → ADR-0377-kafka-to-pulsar (Pulsar+Oxia). Re-point ADR-0003/0004 transport deps.
- **Vector store:** ADR-0006 (pgvector→in-house, here) → ADR-0046→ADR-0192 (Milvus). 
- **Policy:** ADR-0007/0002 (Cedar app-authz, here) → ADR-0243/0246 (universal gate) + ADR-0379 (Kubewarden admission, sup. 0183). Two complementary halves; cross-refs missing.
- **Identity:** ADR-0002 kernel-shape (here) → ADR-0187 (Zitadel) — own-vs-front IdP unresolved.
- **Naming:** ADR-0006 (here) → ADR-0055/0122 (Object-Graph→Ontology rename); the rename ADR is the truth, ADR-0006's prose is broken.

**Cross-side (LINUX) tensions seeded by this cluster (surface, do not resolve):**
- ADR-0006's Postgres-heavy Ontology (Citus/Timescale/PostGIS/RLS) is the **sharpest collision** with LINUX ADR-0001 "eliminate PostgreSQL" + ADR-0020 Milvus-unsafe-deferral (keystone fault-line #1). 
- ADR-0007/0002 Cedar-adopted vs LINUX ADR-0021 owned-Cedar-compatible policy language (fault-line #2 — own-vs-reuse, same model).
- The whole cohesion-thesis "own the six substrates" posture (ADR-0001) is the source-side analogue of LINUX's OWN_DAY0 instinct, but framed as cohesion-of-assembled-OSS rather than build-from-scratch (fault-line #5 — trigger-threshold disagreement, not principle).

**Net dispositions for this chunk:** KEEP 2 (0004, 0006 — both with mandatory minor amends), AMEND 4 (0001, 0002, 0003, 0007), ARCHIVE 1 (0005, governed by ADR-0377). No MERGE. Two consensus-needed founder rulings of broad consequence: the **six-substrate count/identity-own-vs-front** (0001/0002) and **Cedar-adopt-vs-own** (0007 vs linux 0021).
