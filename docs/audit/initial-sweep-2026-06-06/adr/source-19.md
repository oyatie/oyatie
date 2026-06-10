# ADR Audit — SOURCE, Chunk 19

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 19
- **Slice requested:** `ls | sort | sed -n "127,133p"` → ADR-0152 … ADR-0158
- **ADRs actually reviewed (7):** ADR-0152, ADR-0153, ADR-0154, ADR-0155, ADR-0156, ADR-0157, ADR-0158
- **Auditor posture:** READ-ONLY. No audited doc edited. Dispositions are recommendations for the synthesis pass.
- **Cluster identity:** This is the **cross-cutting hyperscaler-pattern band** — reliability (RPO/RTO), eventing correctness (outbox, schema-versioning), tenant isolation (quotas), privacy (PII registry), edge (api-gateway), and geo (multi-region disposition). All seven are `Accepted`, all dated 2026-05-18, all cite Tier-A hyperscaler precedent. None carry planning front-matter; none are referenced in `MASTERPLAN.md`.

---

### ADR-0152 — RPO/RTO Canonical (Five-Tier Recovery Model)

- **decision_atom:** Every µservice declares one of five recovery tiers (R0 realtime → R4 best-effort, each with explicit RTO+RPO numbers) in its `backfill-replay.md` front-matter, aggregated in `specs/microservices/rpo-rto-targets.json`, enforced by the `oya-check-rpo-rto-coverage` gate.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP. Sole canonical RPO/RTO model in the corpus (no duplicate found); well-formed; non-conflicting.
- **governing:** n/a (governs itself; feeds ADR-0158's per-µservice `multi-region.md` RPO/RTO fields and ADR-0139 SLO-gated promotion).
- **truth_flag:** TRUE.
- **in_masterplan:** NO. Legacy bullet-list format (no YAML front-matter, so no `masterplan_ref`/`planning_impact`); not referenced in MASTERPLAN.md. Decision is masterplan-ready but unbound.
- **tensions:**
  - ADR-0158 consumes R-tier numbers in each µservice's `multi-region.md` for active_passive failover — coherent dependency, not a conflict.
  - ADR-0139 (agentic-SLO-gated-promotion) is a separate SLO/observability substrate; mild naming-overlap risk ("reliability bar" appears in both) but different artifacts. Worth a one-line cross-ref so R-tiers and SLO objects are not conflated.
  - LINUX-side: no direct conflict (pilot has no recovery-tier ADR).
- **hyperscaler_challenge:** ALIGNED. AWS Well-Architected Reliability Pillar and Google SRE both mandate declared RPO/RTO per workload; a 5-tier ladder is exactly the AWS/Azure tiered-DR shape. Argues KEEP.
- **ai_slop:** none material. Minor: "oyatie's 33 µservices" hardcodes a count that ADR-0157 bumps to 34 and ADR-0158 enumerates ~30 — count-drift, cosmetic.
- **refinement:** Add YAML front-matter (`id`, `status`, `supersedes/superseded_by`, `masterplan_ref`) to match the 0157/0158 format; replace the hardcoded "33" with "every µservice"; explicit cross-ref to ADR-0158 + ADR-0139.
- **consensus_needed:** no.

---

### ADR-0153 — Outbox Pattern

- **decision_atom:** The transactional outbox (append an outbox row in the same DB transaction as the aggregate mutation; a publisher worker drains it FIFO) is the ONLY canonical way a µservice emits an event accompanying a state change — direct `event_bus.publish()` is forbidden.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP (with AMEND-grade vocab nit). The pattern itself is explicitly retained by the keystone map ("transactional-outbox pattern retained; Kafka retired") even though the eventing substrate moved Kafka→Pulsar (ADR-0377). The decision survives the substrate change.
- **governing:** n/a for the pattern. Substrate context now governed by ADR-0377-kafka-to-pulsar (Pulsar 4.x + Oxia) + ADR-0195/0397 — but those do NOT supersede the outbox contract.
- **truth_flag:** TRUE (the pattern). PARTIAL on framing: the implicit "DB + bus" substrate assumption predates the Pulsar cutover.
- **in_masterplan:** NO. No front-matter; not in MASTERPLAN.md.
- **tensions:**
  - **Retired-vocab leakage:** decider list includes `axis-foundry` — "foundry" brand RETIRED (ADR-0335 → intelligence; ADR-0347 → governance). Stale authorship attribution.
  - Eventing substrate: body says "event bus" generically and lists CDC/Debezium as a later layer — compatible with ADR-0377 Pulsar; but a reader could assume Kafka. Worth a one-line "substrate per ADR-0377" note.
  - 2PC rejection cites Pat Helland — sound, no tension.
- **hyperscaler_challenge:** ALIGNED. Stripe/Uber/Confluent all run transactional outbox; AWS and Azure docs prescribe it for the dual-write hazard. The "in-process outbox default, Debezium-CDC optional later" sequencing is exactly the mainstream call. Argues KEEP.
- **ai_slop:** mild. "Stripe-grade event correctness" / "every serious distributed-systems shop" is rhetorical filler. The 5-carrier-then-28 rollout count is fabricated-precision-ish but harmless.
- **refinement:** Drop `axis-foundry` (→ `axis-intelligence`/`axis-governance` per current org vocab); add "substrate per ADR-0377 (Pulsar)" note; add front-matter; trim rhetoric.
- **consensus_needed:** no.

---

### ADR-0154 — Event Schema Versioning

- **decision_atom:** Every emitted event MUST carry an explicit per-event `version` header (+ `event_id` ULID) in its AsyncAPI 3.1.0 envelope, evolving under SemVer rules (MINOR additive, MAJOR breaking with ≥30-day overlap), enforced by `oya-check-event-schema-versioning`; a standalone schema-registry µservice is deferred (on-disk AsyncAPI docs are SSOT meanwhile).
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND. Sound core decision, but two broken cross-references and one retired-substrate mention need reconciliation before it is masterplan-ready.
- **governing:** n/a (no superseding ADR). Substrate channel-list governed by ADR-0377 (Pulsar replaces Kafka).
- **truth_flag:** PARTIAL — the versioning decision is TRUE; the citations are WRONG.
  - **WRONG cross-ref #1:** body cites "ADR-0061 (no silent regression)" — on disk `ADR-0061` is `application-b2b-unified-shell`, NOT a no-silent-regression ADR. Fabricated/mis-numbered citation.
  - **WRONG cross-ref #2:** body cites "`event_id` (ULID per ADR-0156)" — ADR-0156 is the **PII registry** and contains zero ULID content. The ULID identifier decision lives elsewhere (or is uncaptured); this is a mis-numbered citation.
- **in_masterplan:** NO. No front-matter; not in MASTERPLAN.md.
- **tensions:**
  - **Retired substrate:** decision text "every channel (WebSocket, AMQP, NATS, **Kafka**)" — Kafka standalone is RETIRED → Pulsar+Oxia (ADR-0377-kafka-to-pulsar supersedes ADR-0005). Stale channel enumeration.
  - **Retired-vocab leakage:** decider `axis-foundry` (RETIRED brand, ADR-0335/0347).
  - Cross-ref to ADR-0145 (AsyncAPI 3.1.0 adoption) — valid, coherent.
- **hyperscaler_challenge:** ALIGNED on principle. AWS EventBridge Schema Registry, Confluent, Stripe `api_version` all enforce explicit per-event versioning with back-compat windows. BUT a hyperscaler would NOT defer the registry indefinitely with "on-disk docs are SSOT" — AWS/Confluent make the *registry the enforcing runtime*, not a doc. Argues AMEND (tighten the deferral to a dated trigger), not archive.
- **ai_slop:** internal-contradiction-grade — the two wrong ADR citations are fabricated precision (citing specific ADR numbers that do not support the claim). This is the slop signature the founder warned about.
- **refinement:** Fix both citations (ULID source ADR; the real "no silent regression" governing decision, if any); replace "Kafka" with "Pulsar/KoP" or generic "event bus per ADR-0377"; drop `axis-foundry`; give the deferred registry a numeric/dated trigger; add front-matter.
- **consensus_needed:** **yes** — the ULID-identifier decision appears to be cited but not actually captured in a real ADR (0156 is wrong). If ULID is canonical, it needs its own atom in the masterplan.

---

### ADR-0155 — Per-Tenant Resource Quotas

- **decision_atom:** Every µservice enforces per-tenant quotas on five canonical axes (rate, concurrent-in-flight, memory, storage, connections); the tenancy µservice owns the canonical definitions, runtime µservices query it, and an exceeded quota returns `429` + `Retry-After` + `X-Tenant-Quota-{Axis,Limit,Used}` headers, with cell-level isolation guaranteed.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP. Well-formed, current, non-conflicting; the natural companion to ADR-0128 INV-SHUFFLE-SHARDING and ADR-0157 gateway rate-limit.
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** NO. No front-matter; not in MASTERPLAN.md.
- **tensions:**
  - "tenancy µservice OWNS quota defs + runtime queries it" makes tenancy a hot critical-path dependency — the ADR self-flags this ("needs caching"). Mild tension with any "no synchronous cross-µservice critical-path call" invariant; reconcile with ADR-0155's own caching note + ADR-0157 edge rate-limit (some of this is enforceable at the gateway, reducing per-request tenancy calls).
  - "Cell-level isolation: per-tenant quota cannot bleed cells" uses `cell` correctly as the deployment-pattern/boundary sense (post-ADR-0333) — NOT the retired cell-µservice. Acceptable.
  - tenant-class (demo_trial/paid, ADR-0329) is the live quota-scaling axis — ADR-0155 does not mention how quotas vary by tenant_class; refinement opportunity, not a conflict.
- **hyperscaler_challenge:** ALIGNED. AWS SaaS Lens prescribes exactly these per-tenant isolation axes; AWS Service Quotas + GCP quotas + Azure throttling all do the five-axis + 429/Retry-After shape. Argues KEEP.
- **ai_slop:** none material. "fails at scale" is light hedging but defensible.
- **refinement:** Add front-matter; cross-ref ADR-0157 (which quotas enforce at edge vs workload) and ADR-0329 tenant_class (per-class quota defaults); state the caching/replication strategy for the tenancy hot path explicitly.
- **consensus_needed:** no.

---

### ADR-0156 — PII Registry Canonical (Cross-Cutting Data Classification)

- **decision_atom:** Each µservice manifest declares a top-level `data_classes_processed` array (union of per-bounded-context `data_classes_owned`), aggregated into `specs/microservices/pii-registry.json` so the DSR (Data Subject Request) cascade has a machine-readable fan-out plan; coherence is gate-validated via extended `oya-check-data-class` rules.
- **current_status:** Accepted (2026-05-18).
- **disposition:** KEEP. Clean, compliance-grounded, non-conflicting; correct dependency on ADR-0008 data-use-boundary (verified on disk).
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** NO. No front-matter; not in MASTERPLAN.md. (Strong masterplan candidate — it is a compliance-evidence artifact.)
- **tensions:**
  - **Inbound mis-citation (not this ADR's fault):** ADR-0154 wrongly cites ADR-0156 as the "ULID" source. 0156 has no ULID content. Flag belongs to 0154, but auditors comparing the pair should note the dangling reference resolves to nothing here.
  - References ADR-0008-data-use-boundary — valid, on disk.
  - GDPR Art. 30 / CPRA / Korea PIPA grounding is accurate; consistent with the KR-sovereignty posture (KCMVP/ADR-0190 elsewhere).
- **hyperscaler_challenge:** ALIGNED. AWS Macie + GCP DLP + Azure Purview all maintain a per-account data-class inventory; GDPR Art. 30 legally requires the register. Shipping a deterministic registry first and deferring ML-tag discovery (Macie-style) is the correct sequencing. Argues KEEP.
- **ai_slop:** none.
- **refinement:** Add front-matter; promote to a masterplan compliance atom; specify the registry's derived-artifact regeneration contract (which gate run rebuilds it) to satisfy the drift-prevention design.
- **consensus_needed:** no.

---

### ADR-0157 — Dedicated API Gateway Tier (separate from per-µservice rate-limit)

- **decision_atom:** Adopt a dedicated `api-gateway` µservice (Envoy 1.30 data plane + Envoy Gateway 1.1 / K8s Gateway API control plane + Coraza WAF + Envoy global-ratelimit) as the canonical, per-cell north-south edge — terminating TLS, verifying JWT/mTLS/OAuth2.1-PAR, applying coarse Cedar tenant-scoping, WAF, DDoS, first-tier global rate-limit, OpenAPI-3.1 schema-fail-fast, and trace-context injection — while domain authz, per-resource Cedar, and INV-SHUFFLE-SHARDING stay at the workload tier.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND. Architecturally excellent and current (full YAML front-matter, `architectural_authority: ADR-0182`); the amend is purely retired-substrate hygiene (Redis→Valkey; cell-µservice phrasing).
- **governing:** n/a (no superseder). Architectural authority = ADR-0182 (gateway-vs-mesh separation, verified on disk); picks the implementation under it.
- **truth_flag:** TRUE (decision). PARTIAL on substrate naming (Redis).
- **in_masterplan:** PARTIAL. Has proper ADR front-matter (`id/status/supersedes/superseded_by/related/related_specs`) and binds to `/specs/hyperscaler-architecture-invariants.json` + `/specs/per-microservice-flat-layout.json` — but NO `masterplan_ref`, and not referenced in MASTERPLAN.md. Best-structured ADR in the chunk.
- **tensions:**
  - **Retired-vocab (Redis):** rate-limit backend cited as "Redis" 4× (`ratelimit-redis sidecar`, "regional Redis cluster"). Redis-as-substrate is RETIRED → **Valkey** (ADR-0336, license-driven OSI-strict). Direct retired-vocab violation; should read Valkey.
  - **cell-µservice phrasing:** "the cell-µservice provisions the Redis cluster" / "cell-µservice tenant-routing layer" — `cell` as a microservice is RETIRED → deployment-pattern only (ADR-0333). Needs rephrase to "cell control-plane / cell pack."
  - Coherent with ADR-0148 (Cilium primary mesh, Istio Ambient opt-in) — note ADR-0157 body says "Istio (already adopted in ADR-0148)" while ADR-0148 actually makes Cilium primary + Istio Ambient Tier-2; mild drift in how the mesh decision is characterized.
  - Cites ADR-0121 (onprem k8s) as the portability anchor — ADR-0121 is SUPERSEDED by ADR-0375 (Talos+CAPI+ArgoCD). The portability *invariant* survives, but the citation is stale; should point at ADR-0375.
  - Bumps µservice count to 34 — conflicts with ADR-0152's "33" and ADR-0158's enumeration. Count is a moving target; stop hardcoding.
- **hyperscaler_challenge:** ALIGNED. Stripe (Cloudflare-fronted edge), AWS API Gateway/CloudFront, Google Cloud Armor+CLB, Azure Front Door+WAF all ship a dedicated defense-in-depth edge tier distinct from the service mesh. Rejecting cloud-managed-gateway-only for portability and per-µservice-gateway for audit-uniformity are both the calls a hyperscaler makes. Argues KEEP-with-amend (substrate naming only).
- **ai_slop:** none material — this is high-quality, evidence-dense. Minor over-enumeration of OWASP items, but accurate.
- **refinement:** s/Redis/Valkey/ (ADR-0336); s/cell-µservice/cell control-plane (ADR-0333); re-anchor ADR-0121 → ADR-0375; reconcile the ADR-0148 mesh characterization (Cilium-primary, not "Istio adopted"); add `masterplan_ref`; stop hardcoding µservice counts.
- **consensus_needed:** no (implementation-level; well-bounded).

---

### ADR-0158 — Per-µservice Multi-Region Disposition + Sovereign Tenant Region-Pin + Global Control-Plane

- **decision_atom:** Every µservice declares one of three multi-region dispositions (`active_active` | `active_passive` | `single_region`) in `manifest.json`; per-pack sovereign-tenant residency overlays it (a `pack-ksa` tenant is pinned to KSA cells, even rendering single-region-elsewhere µservices unavailable to it); a global control plane (tenant-registry replicated cross-region) routes tenant→cell while the data plane stays strictly regional (no cross-region DB transactions), gate-enforced by `oya gate validate multi-region-disposition` + `sovereign-tenant-pin`.
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND. Strong, current decision (full front-matter, evolves ADR-0049); amend for retired-µservice-name leakage in the enumeration tables + one substrate-naming watch (Postgres/Patroni).
- **governing:** n/a (no superseder). Evolves ADR-0049 (cross-region residency, on disk); consumes ADR-0142 (CRDT), ADR-0157 (edge routing), ADR-0152 (RPO/RTO), ADR-0139 (SLO promotion).
- **truth_flag:** TRUE (decision). PARTIAL on the per-µservice table (lists retired-name µservices).
- **in_masterplan:** PARTIAL. Has ADR front-matter + binds `/specs/multi-region-disposition-canonical.json` + `/specs/hyperscaler-architecture-invariants.json`; no `masterplan_ref`; not in MASTERPLAN.md.
- **tensions:**
  - **Retired-vocab (foundry):** disposition table lists `foundry` `single_region` ("GPU pool pinned to region") and the context/alternatives reuse `foundry`. Brand RETIRED → **intelligence** (consumer AI) / **governance** (CI) per ADR-0335/0347. The GPU-pool µservice should be named `intelligence` (or the relevant successor), not `foundry`.
  - **Retired-vocab (shorts):** table lists `shorts` as `active_active`, and the sovereign-pin example uses "shorts µservice is single-region US." `shorts` µservice is RETIRED → merged into **social** (ADR-0334). Stale enumeration; example should use a live single-region µservice.
  - **Substrate watch (Postgres):** tenant-registry "replicated via Patroni cross-region async replication." Patroni = Postgres HA — consistent with SOURCE canonical (Postgres+pgcat, ADR-0179), so TRUE for source. **Cross-side tension:** this is the sharpest contact point with LINUX **ADR-0001** (eliminate the PostgreSQL/sqlx dependency, own-the-multi-model-engine). Surface for the merge: source's global control plane is Postgres-native; linux wants Postgres gone.
  - **`cell` usage:** "cell control-plane per region" / "per-cell" — used in the post-0333 pattern sense (acceptable), but `cell` also appears in the disposition table as a µservice row (`cell | active_passive`). Per ADR-0333 `cell` is retired *as a microservice* → deployment pattern; the table row reifies it as a service. Flag.
  - `421 Misdirected Request` redirect semantics + anycast DNS + "no cross-region DB transactions" are internally consistent and align with ADR-0049/0157.
- **hyperscaler_challenge:** ALIGNED. Google Spanner Universe topology, Stripe tenant-tag routing, Cloudflare anycast+steering, AWS multi-region (services declare own disposition), Azure Cosmos multi-region-writes all match: global control plane + regional data plane + per-service disposition. Rejecting "active-active everywhere" and "global Spanner-everywhere" on cost/portability grounds is exactly the hyperscaler call. Argues KEEP-with-amend (naming only).
- **ai_slop:** none material — dense and accurate. The only defect is stale µservice names in the tables (retired-vocab leakage), not fabrication.
- **refinement:** Replace `foundry`→`intelligence`, `shorts`→`social` (or drop), reconcile the `cell` table-row vs pattern; explicitly tag the Postgres/Patroni dependency as a known LINUX-merge fault-line; add `masterplan_ref`; cross-ref ADR-0152 R-tiers for the active_passive RPO/RTO fields.
- **consensus_needed:** **yes** — the Postgres-native global control plane (ADR-0158/0179) vs LINUX ADR-0001 "eliminate PostgreSQL" is a load-bearing, cross-side architecture decision the founder must rule on before merge.

---

## Chunk notes for synthesis

**Overall verdict:** 7/7 `Accepted` and substantively TRUE. No archives, no supersessions. Net: **3 KEEP (0152, 0155, 0156), 4 AMEND (0153, 0154, 0157, 0158)**. The amends are dominated by two repeating defects, not by wrong decisions.

**Pattern 1 — Retired-vocabulary leakage is the dominant defect in this band.** Every amend traces to retired vocab persisting in `Accepted` ADRs:
- `axis-foundry` decider (0153, 0154), `foundry` µservice (0158) — RETIRED brand (ADR-0335/0347 → intelligence/governance).
- `shorts` µservice (0158) — RETIRED → social (ADR-0334).
- `Redis` substrate (0157 ×4) — RETIRED → Valkey (ADR-0336).
- `Kafka` channel (0154) — RETIRED standalone → Pulsar/Oxia (ADR-0377).
- `cell`-as-µservice phrasing (0157, 0158 table row) — RETIRED as service → deployment pattern (ADR-0333).
These are 2026-05-18 ADRs predating the 0329–0377 retirement wave; the leakage is chronological, not semantic error. They are exactly the "residual `oya-foundry-*` / Redis / Kafka strings persist corpus-wide" signal the keystone map (§2 lint signal) predicts. **Recommend a single corpus-wide retired-vocab lint pass rather than per-ADR rewrites.**

**Pattern 2 — Citation integrity failure (ADR-0154).** Two cited ADR numbers do not support the claims: ADR-0061 ("no silent regression" → actually b2b-unified-shell) and ADR-0156 ("ULID source" → actually PII registry). This is the founder's "fabricated precision" slop signature. It also reveals a **possibly-uncaptured canonical decision: the ULID event-id choice** has no real home ADR — a masterplan-backfill gap, flagged for consensus.

**Pattern 3 — Masterplan binding is ~0% in this band.** 0/7 carry `masterplan_ref`; 0/7 are referenced in MASTERPLAN.md. The legacy three (0152/0153/0154/0155/0156 bullet-list style) lack YAML front-matter entirely; the modern two (0157/0158) have rich front-matter + `related_specs` but still no `masterplan_ref`. This band is a microcosm of the map's "8.8% ADR binding" finding. **Every decision here is genuine masterplan backfill material** — they are precisely the cross-cutting invariants a single-source-of-truth masterplan should encode.

**Pattern 4 — Stale upstream citations.** 0157 anchors portability on ADR-0121 (SUPERSEDED by ADR-0375 Talos) and characterizes the mesh as "Istio adopted" when ADR-0148 makes Cilium primary. The *invariants* survive; the *citations* drifted. Trust the superseding ADR (map §6 discipline).

**Cross-chunk / cross-side tensions to escalate:**
1. **Postgres fault-line (ADR-0158 ↔ LINUX ADR-0001).** Source's global control plane is Postgres/Patroni-native (consistent with ADR-0179). LINUX ADR-0001 wants PostgreSQL eliminated. This is the single sharpest cross-side conflict touching my chunk — load-bearing, needs a founder ruling.
2. **ULID decision homelessness (ADR-0154).** If ULID is canonical for `event_id`, it needs a real ADR atom in the masterplan; currently cited to the wrong ADR.
3. **µservice-count drift (33 vs 34 vs enumerated ~30).** 0152 says 33, 0157 says 34, 0158 enumerates a different set including retired names. Symptom of an unmaintained inventory; the count should be derived, never hardcoded.
4. **Outbox/eventing layering (ADR-0153) vs canonical eventing (ADR-0377/0195/0397).** The outbox *pattern* is retained per the map, but the ADR's implicit substrate framing predates Pulsar; a one-line "substrate per ADR-0377" reconciliation closes it.

**Clusters:** (a) **Eventing-correctness pair** 0153+0154 (outbox + schema-versioning) — both substrate-stale, both `axis-foundry`-tainted, should be reconciled together against ADR-0377. (b) **Edge+geo pair** 0157+0158 — the two modern, well-structured ADRs; tightly coupled (0157 enforces 0158's sovereign-pin at edge); amend together for Redis/foundry/shorts naming. (c) **Standalone compliance/reliability trio** 0152+0155+0156 — clean KEEPs, prime masterplan-backfill atoms, just need front-matter.
