# ADR Audit Artifact — source-30

- **Side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **Chunk:** source-30 (auditor slice lines 204–210 of `ls docs/decisions/ADR-*.md`)
- **Range:** ADR-0239 → ADR-0245
- **ADRs reviewed:** 7 (0239, 0240, 0241, 0242, 0243, 0244, 0245)
- **Auditor role:** ADR Auditor (coverage backfill), READ-ONLY except this artifact
- **Keystone map consulted:** `_map/canonical-posture-and-supersession-map.md`
- **Cross-cutting note:** ADRs 0242–0245 are keystone positions **1-of-14, 2-of-14, 3-of-14, 4-of-14** of the `2026-05-20-foundational-doctrine` bundle (ADR-0242…ADR-0255). All four are `status: Proposed` + `planning_impact: true` and declare partial acceptance is rejected (mutually-reinforcing). They are the SOURCE-side counterweight to the LINUX pilot's own-DB / own-policy / framekernel divergence and are highly masterplan-binding.

---

### ADR-0239 — Foundry Scope Clarification (Internal-Only Amendment)

- **decision_atom:** `microservices/foundry/` is the INTERNAL-only Hermes/CI/eval/evidence substrate; consumer-facing AI is the separate `microservices/intelligence/` µservice ("oyatie intelligence" brand), and every µservice manifest declares an `audience` field.
- **domain:** intelligence-ai, agentic-platform
- **current_status:** Accepted (amendment to ADR-0136)
- **disposition:** ARCHIVE
- **governing:** ADR-0242 + ADR-0244 (which explicitly `amends:` 0239 and supersede the audience-as-µservice-scope framing); brand layer further governed by retirement keystones ADR-0335 (foundry→intelligence) / ADR-0347.
- **proposed_resolution:** n/a (status is Accepted, not Proposed).
- **truth_flag:** STALE — the "Foundry is an internal-only µservice with an `audience` field" model was explicitly killed within ~weeks by ADR-0242/0244 ("audience is a property of the caller, not the callee"; `audience` field REMOVED) and the "Foundry" brand itself is RETIRED per ADR-0335/0347. The doc still reads `Accepted` with no `superseded_by:` — front-matter drift (same class as the keystone map's ADR-0136 drift note).
- **in_masterplan:** PARTIAL — no `planning_impact:` flag; its operational residue (foundry-as-internal-substrate role) survives but its decision content is absorbed by 0242/0244. Historical record at best.
- **tensions:** Directly contradicted by ADR-0242 §D-3 ("No internal-only µservices") and ADR-0244 §D-11 (audience moves to tenant). Also collides with retired-vocabulary: "Foundry" brand dead (ADR-0335), "oyatie intelligence" naming survives only as `microservices/intelligence/`.
- **hyperscaler_challenge:** Misaligned. Google/AWS/Azure do NOT model "internal vs consumer" as a *service* property — they operate internal teams as tenants of their own platform (the exact argument ADR-0242 makes against this ADR). Argues for ARCHIVE.
- **ai_slop / refinement / consensus_needed:** Not slop — it was a real, well-reasoned decision that the portfolio then outgrew. Refinement: mark `superseded_by: [ADR-0242, ADR-0244]` and `status: Superseded`. No founder question.

---

### ADR-0240 — Sovereign cloud per regional pack

- **decision_atom:** Each regional pack declares a `sovereign_cloud_overlay.yaml` enumerating mandatory certified substrate providers (primary+secondary) and the data classes that must remain on them, with Cedar-time denial of cross-provider egress for sovereign-tagged data.
- **domain:** compliance-residency, isolation-runtime (cloud-substrate)
- **current_status:** Accepted (enforcement advisory-until-per-pack-overlay-finalized)
- **disposition:** KEEP (with AMEND for ref hygiene)
- **governing:** n/a (not superseded).
- **proposed_resolution:** n/a (Accepted).
- **truth_flag:** TRUE — sovereign-cloud residency is a real, durable, regulator-driven decision (CSAP/NDMO/GAIA-X/METI/FedRAMP). Minor STALE references: cites ADR-0121 (on-prem k8s, **superseded by ADR-0375 Talos+CAPI** per keystone map) for the multi-tool IaC base, and lists `Redis`/`ElastiCache`/`Memorystore` in the provider module matrix (Redis is RETIRED → Valkey per ADR-0336 — but here it is naming the *cloud providers' managed Redis-compatible services*, a softer leak).
- **in_masterplan:** PARTIAL — no explicit `planning_impact:` flag, but carries `enforced_by: oya gate validate sovereign-cloud-overlay` and a full implementation surface; it is a binding compliance invariant. Treat as masterplan-relevant.
- **tensions:** Provider matrix references the ADR-0121 onprem stack now superseded by ADR-0375 (Talos). No conflict in substance — the per-pack-provider concept is orthogonal to the kubeadm→Talos change — but the `cloud-iac` (OpenTofu+ArgoCD/Flux) wording predates Talos canon.
- **hyperscaler_challenge:** Aligned. AWS GovCloud, Azure Government, Google sovereign-cloud / "Sovereign Controls", and the EU "sovereign cloud" partner programs (T-Systems/OVH/Bleu) are exactly this pattern. A hyperscaler WOULD make this decision; arguably oyatie goes further (multi-provider per pack) which is more vendor-independent than any single hyperscaler. Does not argue for archive.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: update IaC-base citation from ADR-0121→ADR-0375; restate Redis-compatible managed services as "Valkey-compatible / provider-managed cache." No founder question.

---

### ADR-0241 — DR + business-continuity portfolio policy

- **decision_atom:** Every µservice declares one of four DR tiers (T1 <5min RTO/0 RPO … T4 <24h RTO/<1h RPO) in its manifest, with tier-bound replication shapes, drill cadences (quarterly T1/T2, semi-annual T3/T4), brown-out degraded-dependency coordination, and quarterly regulator evidence packets.
- **domain:** dr-resilience, observability
- **current_status:** Accepted (enforcement advisory-until-per-microservice-tier-declared)
- **disposition:** KEEP (with light AMEND)
- **governing:** n/a.
- **proposed_resolution:** n/a (Accepted).
- **truth_flag:** TRUE — standard hyperscaler 4-tier DR (AWS Well-Architected, Azure Site Recovery, Google SRE). One STALE example: the T1 list names "Foundry runtime" and "foundry" (retired brand per ADR-0335; should read intelligence/governance substrate). DR-tier "T1–T4" is a DIFFERENT axis from retired tenant "tier-system" (ADR-0329) and from autonomy-tiers — no conflation, but worth flagging since "tier" is overloaded across this chunk.
- **in_masterplan:** PARTIAL — no `planning_impact:` flag; carries `enforced_by` gate + implementation surface; a binding resilience invariant. Masterplan-relevant.
- **tensions:** Naming residue ("Foundry runtime" in T1 table). Depends on ADR-0240 (sovereign cloud) for cross-provider failover substrate — coherent. No hard conflict.
- **hyperscaler_challenge:** Aligned. The 4-tier RTO/RPO model is literally AWS/Azure/Google doctrine. Yes they would make this decision. No archive argument.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: replace "Foundry runtime" with the post-0335 intelligence/governance naming. No founder question.

---

### ADR-0242 — `oyatie`-is-a-tenant doctrine  (keystone 1-of-14)

- **decision_atom:** `oyatie` is registered as a first-class reserved-namespace org-tenant of its own multi-tenant platform with dotted hierarchical sub-scopes (`oyatie.foundry.ci-agent`, etc.); there are NO internal-only µservices or bypass paths — same Cedar gates, DSAR, retention, audit, and FinOps apply to `oyatie` as to any customer tenant.
- **domain:** tenancy, governance-process
- **current_status:** Proposed (`planning_impact: true`; advisory-until-tenant-bootstrap-lands)
- **disposition:** KEEP (RATIFY)
- **governing:** n/a (it is itself governing; supersedes the 0136-amend/0220/0239 audience framing via its `amends:` list).
- **proposed_resolution:** **RATIFY (accept).** Why: it is the load-bearing tenancy keystone the whole 0242–0255 bundle and the masterplan tenancy posture (keystone map §3 "tenant = universal scoping primitive; oyatie-is-a-tenant") depend on; leaving it Proposed leaves the masterplan tenancy spine unaccounted.
- **truth_flag:** TRUE — strongest argued ADR in the chunk; "mature platforms operate as tenants of their own platform" (Amazon-on-AWS, Stripe-on-Stripe, Microsoft IT-as-Azure-tenant) is correct and well-cited.
- **in_masterplan:** YES — `planning_impact: true`, keystone 1-of-14, defines bootstrap sequence + reserved-namespace + tenant-model spec; directly binds `/specs/tenant-model.json` and `/specs/platform-architecture.json`.
- **tensions:** (1) Retires the ADR-0239/0220/0136-amendment audience model — clean (it is the superseder). (2) The `oya`/`oyat`/`oyati` reserved-root family and dotted `oyatie.foundry.*` sub-scopes re-encode the RETIRED "foundry" brand at the *namespace* level — when the brand is dead per ADR-0335/0347, persisting `oyatie.foundry.*` principal scopes is retired-vocab leakage that this keystone bakes into bootstrap. Flag for rename to `oyatie.intelligence.*` / `oyatie.governance.*`. (3) `audience: INTERNAL` removal on ~46 manifests is asserted but is a forward sweep, not done.
- **hyperscaler_challenge:** Aligned (emphatically). This IS the hyperscaler shape; the ADR's own evidence is the argument. No archive.
- **ai_slop / refinement / consensus_needed:** Not slop — dense but substantive. Consensus_needed (founder): "Given ADR-0335 retires the Foundry brand, should the reserved sub-scope family be `oyatie.foundry.*` (as written) or renamed to `oyatie.intelligence.*` + `oyatie.governance.*` before bootstrap hard-codes it?"

---

### ADR-0243 — Cedar as Universal Gate  (keystone 2-of-14)

- **decision_atom:** Every policy-class decision (routing, authz, activation, attribution, retention, eligibility, feature flags — ~23 enumerated sites) is evaluated by the Cedar policy engine via versioned/signed/hot-reloadable fragments with CI-enforced permit + default-deny coverage; code never decides policy, it asks Cedar.
- **domain:** authz-policy, security-supplychain
- **current_status:** Proposed (`planning_impact: true`; advisory-until-policy-engine-substrate-lands)
- **disposition:** KEEP (RATIFY)
- **governing:** n/a (extends ADR-0150; clarifies ADR-0183).
- **proposed_resolution:** **RATIFY (accept).** Why: it is the canonical authz posture in the keystone map §3 (Cedar = universal authorization gate, ADR-0243/0246/0379) and every other keystone (0242 reserved-namespace, 0244 scoping, 0251 packs) routes through it; cannot remain Proposed without un-anchoring the policy spine.
- **truth_flag:** TRUE — aligns with keystone-map canonical posture. Note: §D-2/§D-5/§D-10/§D-11 + change-log were amended in-place (2026-05-20 Wave-3-A) by ADR-0293/0294/0295 (soak window, meta-trust-root, bootstrap-CA kill-switch) — i.e. an immutable-SSOT ADR was edited rather than appended; a process flag under the ADRs-are-append-only doctrine (keystone map §4). Performance budgets (<1ms p99) are modeled, not measured (honestly labeled).
- **in_masterplan:** YES — `planning_impact: true`, keystone 2-of-14; binds `/specs/cedar-fragment-schema.json`, `/specs/policy-gate-coverage.json`, `/specs/microservices/policy-engine.json`.
- **tensions:** (1) References `Kafka` for the fragment-reload pub-sub + audit enqueue (ADR-0050 inheritance) — Kafka is RETIRED → Pulsar 4.x+Oxia per ADR-0377; stale-substrate leak. (2) Uses `Valkey` hot-cache (correct, post-0336). (3) In-place edits vs append-only ADR doctrine (above). (4) Cedar-universal vs LINUX ADR-0021 owned compile-to-Rust policy language — keystone map fault-line #2 ("own vs reuse Cedar"); LINUX positions as owned successor, not a flat contradiction.
- **hyperscaler_challenge:** Aligned. AWS Verified Permissions (Cedar), GCP Org Policy consolidation, OPA-at-Netflix/Pinterest are the cited precedent; single-policy-engine consolidation is the recognized hyperscaler pattern. Yes they would. No archive.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: swap Kafka→Pulsar/Oxia in §D-6/§D-10; resolve the append-only-vs-in-place-amend process question. Consensus_needed (founder, shared with whole bundle): under the ADRs-generate-masterplan doctrine, are these Wave-3-A surgical edits legitimate, or must 0293/0294/0295 be standalone superseding ADRs?

---

### ADR-0244 — Tenant as Universal Scoping Primitive  (keystone 3-of-14)

- **decision_atom:** Tenant ID + dotted hierarchical sub-scope is the single universal scoping primitive for every routing/authz/attribution/retention/residency/audit/cost/encryption/compliance decision; the µservice `audience` field is removed and replaced by `tenant.audience_type`, with a canonical `tenants` table schema and Cedar entity-types.
- **domain:** tenancy, api-contracts (canonical schema/Cedar entity-types)
- **current_status:** Proposed (`planning_impact: true`; advisory-until-tenant-substrate-lands)
- **disposition:** KEEP (RATIFY) — with AMEND on the `tier` column
- **governing:** n/a (governing; `amends:` 0220/0239/0221 §M-04).
- **proposed_resolution:** **RATIFY (accept).** Why: it is the schema/primitive backbone the keystone-map tenancy posture rests on (ADR-0244 cited in §3) and the concrete `tenants` DDL is what `/specs/tenant-model.json` is generated from; must be Accepted to be masterplan-binding.
- **truth_flag:** PARTIAL — the doctrine (audience→tenant; uniform primitive) is TRUE and hyperscaler-correct, but the §D-3 DDL still ships a free-text `tier` column with `CHECK (tier IN ('free','standard','pro','enterprise',...))`. Per keystone map §2, the "tier / tier-system / capability-tier" vocabulary is RETIRED by **ADR-0329 → tenant-class (`demo_trial`|`paid`) + composable `billing_components`**. This ADR (2026-05-20) predates/contradicts the 0329 retirement and persists a retired axis in the canonical tenant schema — a STALE/WRONG sub-decision inside an otherwise TRUE ADR.
- **in_masterplan:** YES — `planning_impact: true`, keystone 3-of-14; the canonical tenant schema → `/specs/tenant-model.json`, `/specs/microservice-manifest-schema.json`.
- **tensions:** (1) `tier` column vs ADR-0329 tenant-class retirement (above) — the sharpest in-chunk drift. (2) Carries `policy_evaluation_mode` / `ontology_read_mode` "library-first" enums citing ADR-0246-amendment + ADR-0257-amendment — more in-place "amendment" coupling (same append-only process flag as 0243). (3) `audience_type` enum is large and Wave-3-G-extended (B2C_FAMILY_PARENT, EMERGENCY_SERVICES, etc.) — broad but internally consistent. (4) Schema is explicitly Postgres+Citus — collides with LINUX ADR-0001 "eliminate PostgreSQL / own multi-model engine" (keystone-map fault-line #1).
- **hyperscaler_challenge:** Aligned on the primitive (AWS IAM principal-path, GCP resource hierarchy, Azure AAD tenant, Stripe connected-accounts all cited correctly). Questionable on the retained `tier` column — a hyperscaler post-2329-equivalent would express plan/quota as composable billing components, not a frozen `free/standard/pro/enterprise` enum. Argues for AMEND (not archive): align the `tier` column to ADR-0329 `tenant_class` + `billing_components`.
- **ai_slop / refinement / consensus_needed:** Not slop (the bulk is rigorous schema work). Refinement: replace `tier` column with `tenant_class` + `billing_components` per ADR-0329; resolve the `*-amendment` in-place-edit pattern. Consensus_needed (founder): "Should ADR-0244's canonical `tenants` schema be amended to ADR-0329 `tenant_class` semantics before it is ratified, so the generated `/specs/tenant-model.json` does not bake in retired `tier` vocabulary?"

---

### ADR-0245 — Substrate vs Product Layering  (keystone 4-of-14)

- **decision_atom:** Every µservice declares a manifest `tier` of substrate | product | service-cell | reserved (audience-neutral capability vs tenant-scoped surface vs peer-cell vs certification-gated placeholder), CI-enforced for SLO bar, versioning/sunset, dependency direction (substrate must-not-depend-on product), deployment cadence, and observability defaults.
- **domain:** governance-process, ci-cd-build (manifest/tier enforcement)
- **current_status:** Proposed (`planning_impact: true`; advisory-until-tier-field-lands-and-classified)
- **disposition:** KEEP (RATIFY)
- **governing:** n/a (governing; `amends:` 0131/0132/0145).
- **proposed_resolution:** **RATIFY (accept).** Why: it is the structural replacement for the retired `audience` field (depends-on ADR-0242) and supplies the substrate/product layering the masterplan uses to assign SLO/versioning/DAG rules; the full 62-µservice classification table is the kind of authored truth the masterplan is generated from.
- **truth_flag:** PARTIAL — doctrine TRUE and hyperscaler-grounded (AWS foundational-vs-application, Apple framework-vs-app, GCP/Salesforce/Azure tiers). STALE residue: §D-3 still classifies `microservices/foundry/` as `substrate-meta` (RETIRED brand per ADR-0335; the table itself admits "Foundry's identity is being decomposed per ADR-0247"), and the count tables include `shorts` and `anonymous` as standalone products though ADR-0334 folds shorts→social and the doc elsewhere notes anonymous folded into community — internal count drift the ADR self-flags ("counts will harden during the classification sweep").
- **in_masterplan:** YES — `planning_impact: true`, keystone 4-of-14; binds `/specs/microservice-tier-classification.json`, `/specs/microservice-dependency-dag.json`, `/specs/substrate-slo-bar.json`.
- **tensions:** (1) `foundry` as `substrate-meta` vs ADR-0335 retirement + ADR-0247 dissolution — the ADR knows this and time-boxes it ("Final dissolution path is owned by ADR-0247"). (2) `shorts` product row vs ADR-0334 (shorts→social) — stale catalog entry. (3) The 62-µservice count is self-described as not-yet-reconciled. (4) substrate "own-the-capability" tiering broadly agrees with LINUX OWN_DAY0 ambition but at the assemble-OSS altitude (keystone-map fault-line #5, trigger-threshold not principle).
- **hyperscaler_challenge:** Aligned. The substrate/product/peer-cell/reserved split with per-tier SLO + deprecation policy is exactly AWS/GCP/Azure/Apple/Salesforce practice (all five cited). Yes they would. No archive.
- **ai_slop / refinement / consensus_needed:** Not slop — it is the most operationally concrete of the four keystones. Refinement: reclassify `foundry` row per ADR-0335/0247 (intelligence/governance, not `substrate-meta`); drop/fold `shorts` and `anonymous` rows per ADR-0334/community-fold; reconcile the 62-count. No new founder question beyond the bundle-wide brand-rename one raised under 0242.

---

## Chunk notes

**Shape of the chunk.** This slice is a clean two-band split:
- **0239–0241** are older `Accepted` portfolio ADRs (2026-05-18). 0240 (sovereign cloud) and 0241 (DR tiers) are durable, hyperscaler-aligned, KEEP-with-minor-ref-hygiene. 0239 (foundry internal-only) is the odd one out — explicitly retired-in-fact by the keystone bundle but still wearing `Accepted` front-matter (ARCHIVE / mark superseded).
- **0242–0245** are keystones 1–4 of the 14-ADR `2026-05-20-foundational-doctrine` bundle, all `Proposed` + `planning_impact: true`. All four are RATIFY candidates and are highly masterplan-binding; they are the SOURCE-side spine for tenancy, policy, scoping, and layering.

**No-unaccounted-proposals discipline.** Four `Proposed` ADRs in range (0242, 0243, 0244, 0245) — all resolved **RATIFY**. None should DROP: each carries `planning_impact: true`, sits in the canonical-posture table of the keystone map, and generates a named `/specs/*.json`. The blocker to ratifying them cleanly is not soundness — it is three drift items below.

**Three drift items the bundle should fix before/at ratification (all AMEND, not ARCHIVE):**
1. **Retired-vocab "tier" inside ADR-0244's canonical tenant schema.** The `tenants.tier` CHECK-enum (`free/standard/pro/enterprise`) contradicts ADR-0329's tenant-class retirement. Because the masterplan is GENERATED from these ADRs, this would inject dead vocabulary into `/specs/tenant-model.json`. Highest-priority fix in the chunk.
2. **Retired "foundry" brand re-encoded structurally.** ADR-0242 hard-codes `oyatie.foundry.*` reserved sub-scopes into the bootstrap sequence; ADR-0245 classifies `microservices/foundry/` as `substrate-meta`; ADR-0241/0239 name "Foundry runtime." Brand is dead per ADR-0335/0347 → should be `intelligence` / `governance`. This is namespace-level, not just prose-level, leakage.
3. **Retired substrates (Kafka) and append-only-ADR process.** ADR-0243 still cites Kafka (retired → Pulsar/Oxia, ADR-0377) for pub-sub/audit enqueue; and ADR-0243/0244 were edited *in place* by Wave-3-A "amendments" (0293/0294/0295/0246-amendment/0257-amendment), which sits uneasily with the keystone-map §4 ADRs-are-append-only-immutable-SSOT doctrine.

**Founder consensus questions surfaced (crisp):**
- (Bundle-wide) Should the retired-"foundry"-brand reserved namespace (`oyatie.foundry.*`) and `substrate-meta`/T1 "Foundry" classifications be renamed to `oyatie.intelligence.*` + `oyatie.governance.*` BEFORE ADR-0242 bootstrap hard-codes them?
- (0244) Should the canonical `tenants` schema be re-cut to ADR-0329 `tenant_class` + `billing_components` before ratification, so generated specs don't carry retired `tier` vocabulary?
- (0243/0244, process) Under the ADRs-generate-masterplan / append-only doctrine, are the Wave-3-A in-place "surgical amendments" legitimate, or must 0293/0294/0295/0246-amendment/0257-amendment be standalone superseding ADRs?

**Cross-side (LINUX) tensions touched by this chunk (surface, do not resolve):** ADR-0244's Postgres+Citus canonical tenant schema vs LINUX ADR-0001 "eliminate PostgreSQL" (fault-line #1); ADR-0243 Cedar-universal vs LINUX ADR-0021 owned compile-to-Rust policy (fault-line #2, own-vs-reuse, not flat conflict); ADR-0245 assemble-OSS substrate tiering vs LINUX OWN_DAY0 breadth (fault-line #5, trigger-threshold).

**Number-collision note:** all seven SOURCE ADRs here are 0239–0245 and will collide on merge with the LINUX pilot's 0001–0026 only insofar as the pilot renumbers into the 0515+ band — no in-range collision within SOURCE itself. No duplicate-ID issue in this slice (unlike the ADR-0377 duplicate flagged in the keystone map).
