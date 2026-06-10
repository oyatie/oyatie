# ADR Audit — SOURCE chunk 21

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 21
- **Slice range (map index 141–147):** ADR-0166 … ADR-0172
- **ADRs actually reviewed (7):** ADR-0166 schema-registry, ADR-0167 tenant-cli, ADR-0168 public-status-page, ADR-0169 webhook-dlq-retry, ADR-0170 developer-portal, ADR-0171 multi-cluster-federation, ADR-0172 cqrs-read-replicas
- **Cross-chunk fact (all 7):** All carry `supersedes: []`. None carry `masterplan_ref`, `planning_impact`, `deliverables`, or `milestone` planning front-matter (verified by grep), and none of these ADR numbers appear in `/specs/masterplan.json` (verified) — so `in_masterplan = NO` for the entire slice. This is the keystone map's 8.8%-ADR-binding gap made concrete: this is a coherent "hyperscaler audit-row C1–C6" cluster (Tier-C nice-to-haves) that is entirely unbound from the masterplan.

---

### ADR-0166 — Schema Registry (Apicurio Registry; Confluent-compat; AsyncAPI 3.x + proto3 + OpenAPI 3.1; backward-compat CI lane)

- **decision_atom:** Adopt Apicurio Registry 3.x as the single canonical schema registry for all schema kinds (AsyncAPI 3.x events, proto3 gRPC, OpenAPI 3.1 REST, Avro legacy, JSON Schema config), with per-subject compatibility levels and a release-blocking backward-compat CI lane.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** AMEND — sound and current substrate choice, but carries stale retired-vocabulary references that need reconciliation.
- **governing:** none supersedes it; but its `related`/body cites ADR-0005 (Kafka eventing — retired→Pulsar per ADR-0377-kafka-to-pulsar) and ADR-0110 (changeset state machine — Superseded by ADR-0363) and ADR-0009 "cell architecture" (cell-as-microservice retired by ADR-0333; survives only as deployment pattern). Per-cell Apicurio deployment is fine since "cell" survives as a pattern, but the ADR-0110 ChangeSet-promotion-pipeline reference (line 103) is to a retired mechanism.
- **truth_flag:** PARTIAL — the core decision (Apicurio canonical registry) is TRUE; references to ADR-0005 Kafka envelope and ADR-0110 promotion pipeline are STALE.
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Has a companion `/specs/schema-registry-canonical.json`, so it is spec-bound but not masterplan-bound.
- **tensions:**
  - ADR-0005 (Kafka): ADR-0166 frames AsyncAPI/Avro around the Kafka eventing backbone and touts "Confluent-compat API so future Kafka consumers integrate trivially" — but Kafka is RETIRED→Pulsar+Oxia (KoP wire-compat). The Confluent-compat selling point survives only via KoP; the "future Kafka consumers" framing is anachronistic.
  - ADR-0110 (line 103, ChangeSet promotion pipeline): retired by ADR-0363 (agentic-VCS retirement). Publication-as-ChangeSet-step needs retargeting onto plain-git + Forgejo/Prow-shaped CI.
  - "per-cell Apicurio + PostgreSQL backing": consistent with ADR-0333 (cell = deployment pattern) and with source's Postgres-canonical posture (ADR-0045/0179).
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all run schema registries (Confluent SR, AWS Glue SR, GCP Pub/Sub schema registry); choosing the OSS Apache-2.0 Apicurio over commercial Confluent for a self-host/sovereign posture is exactly what a hyperscaler building its own substrate would do. Argues for amend (fix retired refs), not archive.
- **ai_slop:** Low. Mild fabricated precision in the subject-naming examples (plausible, not load-bearing). Alternatives A–E are genuine and well-differentiated — not filler.
- **refinement:** Replace ADR-0005/ADR-0110 references with the current eventing (ADR-0377-kafka-to-pulsar / ADR-0195 / ADR-0397) and forge/CI (ADR-0363/0513) chains; reframe "Confluent-compat for Kafka consumers" as "KoP wire-compat." Add `masterplan_ref` + `planning_impact` front-matter to bind it.
- **consensus_needed:** no.

---

### ADR-0167 — Tenant-facing CLI binary `oya` (separate from internal `oya-dev-cli`)

- **decision_atom:** Ship a separate, narrowly-scoped tenant-facing Rust CLI (`oya-tenant-cli`, distributed as `oya`) built only against the public SDK, on Tier-A semver with an 18-month sunset window, distinct from the internal `oya-dev-cli`.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** AMEND — the decision is sound and current; the command surface leaks RETIRED "foundry" brand vocabulary.
- **governing:** none supersedes the CLI decision. But the `oya foundry capability invoke/list` command group and the prose "orchestrates the Foundry pipeline" / "agentic tenants per ADR-0021" reference the RETIRED "foundry" brand (ADR-0335 retired foundry→absorbed by intelligence; founder: "cloud-intelligence is the valid name").
- **truth_flag:** PARTIAL — the architectural decision (two distinct CLIs, Tier-A isolation, OAuth-2.1 device-code, Rust static binary) is TRUE and excellent; the `oya foundry capability …` command names are STALE retired-vocab leakage (should be `oya intelligence capability …`).
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Closes hyperscaler audit Row C1 (spec-bound to `/specs/hyperscaler-architecture-invariants.json`).
- **tensions:**
  - Retired-vocab: `oya foundry capability invoke` (lines 65–66, 176) and "Foundry pipeline" (line 30) vs ADR-0335/0347 foundry-retirement. The command surface is a public, tenant-facing API on Tier-A semver — shipping a retired brand in a Tier-A surface is the costliest place to leak it.
  - Binary-name collision: two binaries both surfaced as `oya` (internal workspace target `oya`, tenant target `oya-tenant` aliased to `oya` in tenant channels). Self-aware and mitigated, but a genuine ergonomic footgun worth a lint.
  - ADR-0021 reference: source ADR-0021 = foundry-capability-registry (agentic tenant path). NOTE: source ADR-0021 ≠ LINUX ADR-0021 (owned-policy) — number collision on merge per map §6.4; not a same-repo tension.
  - ADR-0037 (Tier-A 18-month sunset, N-2 compat window): consistent and load-bearing.
- **hyperscaler_challenge:** ALIGNED. AWS, GCP, Azure, Stripe, GitHub all ship a tenant CLP distinct from internal tooling; Rust static binary + OAuth-2.1 device-code + `--output json` agentic contract is exactly the hyperscaler shape. The decision to NOT extend the internal CLI (confidentiality + dependency-closure) is precisely the reasoning a hyperscaler would use. Argues for amend (rename foundry→intelligence command group), not archive.
- **ai_slop:** Low–moderate. Perf/size budgets (≤25MB, ≤80ms p99, ≤50MB RSS, ≤3 backend calls) are fabricated-precision targets with no cited basis — plausible but unverifiable; acceptable as budgets but flag as authored-precision. Otherwise the alternatives and references are strong.
- **refinement:** Rename `oya foundry capability …` → `oya intelligence capability …`; replace "Foundry pipeline" prose. Confirm the `oya`/`oya-tenant` binary-name strategy is enforced by a lint. Bind to masterplan.
- **consensus_needed:** no (the foundry→intelligence rename is already governed by ADR-0335; this is mechanical cleanup, not a contested ruling).

---

### ADR-0168 — Public status page derived from SLO state

- **decision_atom:** Deploy a public, auto-derived status page (`status.oya.dev` + per-pack subdomains) projected from per-µservice SLO state (ADR-0139) plus pushed incident/maintenance narrative, on a dedicated outage-independent cell, with Statuspage.io-compatible JSON/RSS/webhook/email subscribe surfaces.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** KEEP — current, correct, well-formed; one stale observability reference to note (not blocking).
- **governing:** none supersedes it.
- **truth_flag:** TRUE — the decision stands. One STALE citation: `related: [ADR-0042]` and body "ADR-0042 Grafana stack / in-house UI" — ADR-0042 is Superseded by ADR-0383 (Loki/Tempo/Mimir/Grafana). The page still pulls from Mimir remote-read (line 183/202), which is correct under ADR-0383, so the substrate is right; only the citation is stale.
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Closes hyperscaler audit Row C2.
- **tensions:**
  - ADR-0042 (observability) cited but Superseded by ADR-0383. Mechanical re-cite; the actual telemetry path (Mimir/Grafana) is already the ADR-0383 stack.
  - "per-product (per ADR-0001 flat catalog) … foundry, etc." (line 88): lists `foundry` as a public-facing product component — retired brand (should be intelligence). Minor retired-vocab leakage in an example list.
  - Depends on ADR-0167 (`oya status`), ADR-0169 (webhook-delivery for subscriber notifications), ADR-0171 (federation-tier cluster for outage independence) — all in this same chunk; internally coherent cluster.
- **hyperscaler_challenge:** ALIGNED. status.stripe.com / status.aws.amazon.com / status.cloudflare.com / githubstatus / status.anthropic.com are all exactly this. Auto-deriving from SLO state (vs manual SRE updates) and running the page on a separately-isolated cell so it survives auth outages is the correct, hyperscaler-grade decision. No amend pressure on the substance.
- **ai_slop:** Low. The ASCII architecture diagram and the perf budgets are authored precision but reasonable. The "~$1.5k/mo Statuspage.io / ~$200/mo per region cell" cost figures are fabricated-precision but used only to justify build-vs-buy, which is legitimate. Alternatives A–D genuine.
- **refinement:** Re-cite ADR-0042→ADR-0383; drop `foundry` from the product-component example; bind to masterplan. Otherwise ship as-is.
- **consensus_needed:** no.

---

### ADR-0169 — Webhook DLQ + exponential-backoff retry (shared delivery kernel)

- **decision_atom:** Introduce one shared `oya-shared-webhook-delivery-kernel` that every outbound-webhook µservice integrates, owning HMAC-SHA256 signing (Stripe-Signature parity), 13-retry exponential backoff (~75min), per-tenant circuit breaker isolation, Postgres DLQ, secret-rotation dual-sign window, and a Tier-A tenant `/v1/webhook_endpoints` API.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** KEEP — current, correct, well-formed; one retired-vocab example leak.
- **governing:** none supersedes it.
- **truth_flag:** TRUE. The "one shared kernel not N reimplementations" decision and all the behavioral contracts (signing, replay window, circuit breaker, failure-mode catalog) are sound and current.
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Closes hyperscaler audit Row C3.
- **tensions:**
  - Retired-vocab: "Foundry capability completions" listed as a webhook event source (line 23) — should be intelligence (ADR-0335). Example-list leakage only.
  - ADR-0005 outbox (line 110, "kernel sits on top of the outbox pattern"): the outbox PATTERN survives Kafka→Pulsar migration (map §1.1 — "outbox pattern survives, Kafka retired"), so this reference is TRUE in substance even though ADR-0005's Kafka backbone is retired. No change needed beyond awareness.
  - Shared-crate-bottleneck risk (Negative #1) is honestly self-flagged and tied to the canonical shared-crate update protocol — good hygiene, not a contradiction.
  - Consistent with ADR-0145 "every µservice integrates the canonical trait" precedent (live).
- **hyperscaler_challenge:** ALIGNED. Stripe Webhooks / GitHub Webhooks / Slack Events / AWS EventBridge / PagerDuty are the cited references and the design parities them faithfully (HMAC-SHA256, idempotency-key, per-tenant circuit breaker, DLQ-replay). The "build a shared substrate, reject AWS EventBridge for residency/lock-in" reasoning is exactly hyperscaler-internal logic. No amend pressure on substance.
- **ai_slop:** Low. The DLQ row-count estimate (30k/day × 13 × 30 = ~12M rows) and perf budgets are authored precision but used to justify Postgres partitioning — legitimate. The 13-retry/~75min schedule deviates deliberately from Stripe's 3-day window with stated rationale — good, not slop. Alternatives A–E genuine (E is a "partial accept," which is honest).
- **refinement:** Replace "Foundry capability completions" with intelligence; optionally note ADR-0005's outbox-survives-Kafka-retirement status inline; bind to masterplan.
- **consensus_needed:** no.

---

### ADR-0170 — Backstage-style internal developer portal

- **decision_atom:** (Historical/superseded) The internal developer portal problem — fleet-wide discoverability over ~60 µservices + ~200+ crates with catalog/TechDocs/SLO/runbook/ADR/scorecard aggregation — is real; the Backstage SUBSTRATE answer is reversed in favor of a bespoke-Rust IDP (Leptos portal + ops-BFF) per ADR-0394.
- **current_status:** Superseded (`superseded_by: [ADR-0394]`), self-declared in body header (2026-05-29).
- **disposition:** ARCHIVE — superseded; retain as historical record. The discoverability problem and the surface set survive onto ADR-0394; only the Backstage substrate is dead.
- **governing:** **ADR-0394** (bespoke-Rust IDP central hub; Leptos + ops-BFF). Confirmed on disk: ADR-0394 file = `ADR-0394-bespoke-rust-idp-central-hub.md`. Matches keystone map §1.1 (ADR-0170→ADR-0394; Backstage quarantined).
- **truth_flag:** PARTIAL — the front-matter is honestly STALE-on-purpose: ADR-0170 reads `status: Superseded, superseded_by: [ADR-0394]`, **but ADR-0394 itself is `status: Proposed`** ("DRAFT for founder review … must NOT auto-merge"). So a load-bearing Accepted decision was marked Superseded by an ADR that has not been accepted. The *problem statement* is TRUE; the supersession is **prematurely recorded** (decision-pending), exactly the PR#605-style "decision-pending, not slop" pattern.
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json.
- **tensions:**
  - **Supersession-by-a-Proposed-ADR drift (load-bearing):** ADR-0170 `Superseded` ← ADR-0394 `Proposed`. Three downstream Accepted ADRs (ADR-0203 docs-three-tier, ADR-0209 compliance-evidence, ADR-0213 EaaS developer-sdk + ADR-SDK-0007) still reference "ADR-0170 Backstage" as a live primitive. If ADR-0394 is declined, ADR-0170 must revert to Accepted; if promoted, those three need retargeting. This is a genuine open ruling.
  - ADR-0017 reference (line 120, "Oyatie uses GitHub per ADR-0017 brand-naming-and-repo-layout"): touches the FORGE fault-line (map §5 #4) — GitHub-vs-Forgejo-vs-bespoke-VCS. Surfaced only; not resolved here.
  - ADR-0120 Rust-first "documented exception" for Node.js Backstage: ADR-0394 explicitly says this carve-out is "no longer acceptable" under the hardened bespoke-over-OSS + Leptos + container doctrine — confirming the substrate reversal is doctrine-driven, not erroneous.
  - ADR-0025 reference ("Foundry as engineering platform"): retired brand.
- **hyperscaler_challenge:** MISALIGNED with the original Backstage pick, ALIGNED with the ADR-0394 reversal. Google (no Backstage — internal bespoke), AWS, Azure all run bespoke internal portals; Spotify/Expedia/Netflix adopters cited by ADR-0170 are mid-tier, not hyperscalers. A true hyperscaler building its own substrate would build bespoke (as ADR-0394 now does). The original ADR-0170 decision was hyperscaler-questionable; the supersession corrects it. Argues for archive (already done).
- **ai_slop:** Low in the body (it is a genuine, detailed Backstage design). The slop risk is now organizational, not textual: keeping a fully-fleshed superseded design + three dependents pointing at it is a maintenance hazard until ADR-0394 is ruled on.
- **refinement:** Hold ADR-0170 frozen-as-historical; gate the `superseded_by` on ADR-0394's promotion. Add a `decision_pending: ADR-0394-proposed` note so downstream auditors don't treat the supersession as final. Retarget ADR-0203/0209/0213 only when ADR-0394 is accepted.
- **consensus_needed:** **YES** — "Is ADR-0394 (bespoke-Rust Leptos IDP) promoted, making ADR-0170 Backstage formally dead and triggering ADR-0203/0209/0213 retargeting — or declined, reverting ADR-0170 to Accepted? An Accepted ADR is currently marked Superseded by a Proposed one."

---

### ADR-0171 — Multi-cluster federation via ArgoCD ApplicationSets + Cluster API

- **decision_atom:** Adopt a three-component federation substrate — ArgoCD ApplicationSets (app deploy across N clusters), Cluster API/CAPI (declarative cluster lifecycle), and a dedicated meta-pack "federation" control plane (GeoDNS + multi-cluster routing) — to manage 12+ per-pack regional clusters without per-cluster drift or manual toil.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** AMEND — substrate is current and aligned with source's Talos+CAPI+ArgoCD canon, but the body still cites the RETIRED Istio mesh and Foundry brand and a partially-superseded infra ADR.
- **governing:** none supersedes ADR-0171 itself. It is corroborated by the canonical orchestration posture (ADR-0375 Talos+CAPI+ArgoCD; ADR-0370/0378/0382) and CD posture (ADR-0408/0511 Argo). But:
- **truth_flag:** PARTIAL — ArgoCD ApplicationSets + CAPI + federation-control-plane decision is TRUE and matches source's current k8s canon (Talos+CAPI+ArgoCD per ADR-0375). STALE: the heavy reliance on Istio multi-primary (ADR-0148) — Istio appears in the keystone map as part of the retired kubeadm/containerd/istio onprem stack superseded by ADR-0375 (Talos). The Istio mesh dependency (lines 45, 199, 206) needs reconciliation against the post-ADR-0375 mesh posture.
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Closes hyperscaler audit Row C5.
- **tensions:**
  - **ADR-0148 Istio:** map §1.1 lists "ADR-0121 onprem k8s (kubeadm/containerd/istio) Superseded by ADR-0375." ADR-0171 leans on Istio multi-primary as the in-mesh cross-cluster discovery layer — needs reconciliation with whatever mesh ADR-0375/0370 settle on. This is the sharpest stale dependency in the slice.
  - **ADR-0121:** cited repeatedly as "sovereign on-prem authority" and "canonical hyperscaler-bar standards index" (lines 102, 155, 203–204) — but ADR-0121 is Superseded by ADR-0375 (Talos+CAPI+ArgoCD) per map §1.1. The federation design happens to align with ADR-0375 (CAPI+ArgoCD), so the substance is fine; the citations are stale.
  - **Foundry brand:** "Foundry's per-tenant GPU pool" / "Foundry GPU pools" (lines 27, 100) — retired→intelligence.
  - **Forge reference:** the ApplicationSet `repoURL: https://github.com/oyatie/oya.git` (line 78) bakes the GitHub URL into canonical IaC — touches the forge fault-line (founder=GitHub vs source-canon=Forgejo/bespoke-VCS, map §5 #4). Surfaced only.
  - LINUX cross-side tension (map §5 #3): LINUX ADR-0025 wants a Rust "Talos"; ADR-0171 adopts actual CAPI/ArgoCD/Talos-class substrate — own-vs-assemble tension at merge.
- **hyperscaler_challenge:** ALIGNED. GKE Multi-Cluster Ingress/Anthos, AWS EKS+ArgoCD, Azure Arc are the exact references; CAPI + ArgoCD ApplicationSets is the CNCF-graduated mainstream for fleet management. A hyperscaler manages clusters declaratively via a control plane exactly like this. The Istio dependency is the only piece a post-Talos hyperscaler-lens might revisit. Argues for amend (re-cite ADR-0121→0375; reconcile Istio).
- **ai_slop:** Low. The ApplicationSet YAML is concrete and correct-shaped. Cluster-count projections (≥3 packs, ≥12 clusters by M02) are authored estimates but used legitimately. Alternatives A–F genuine (F is an honest "partial accept" splitting per-cell vs per-pack ArgoCD). Note: "M01/M02/M03" milestone vocabulary is RETIRED→Wave names (map §2, GLOSSARY L250/L504) — minor leakage across the whole slice.
- **refinement:** Re-cite ADR-0121→ADR-0375; reconcile the Istio multi-primary dependency against the current mesh posture under ADR-0375/0370; rename Foundry GPU pools→intelligence; note the GitHub repoURL under the forge fault-line; replace M0x milestones with Wave names; bind to masterplan.
- **consensus_needed:** no (mechanical re-citation; the federation decision itself is uncontested and aligned with ADR-0375).

---

### ADR-0172 — Read replicas + CQRS where appropriate (high-read BCs only, per-µservice opt-in)

- **decision_atom:** Adopt a NARROW (read-replica, NOT event-sourced) CQRS split for exactly three high-read bounded contexts — `social.feed`, `messenger.search`, `ontology.entity-query` — with per-tenant Read-Your-Writes LSN-pinning, explicit staleness budgets, and a per-µservice opt-in protocol; all other µservices stay single-primary Postgres.
- **current_status:** Accepted (2026-05-18), `superseded_by: []`.
- **disposition:** AMEND — the decision is sound and current; carries a RETIRED-vocab Redis reference, a stale-eventing reference, and depends on a Superseded read-path ADR.
- **governing:** none supersedes ADR-0172. It explicitly extends ADR-0045 (database-tier Postgres-canonical), which is the live posture.
- **truth_flag:** PARTIAL — the narrow-CQRS-for-three-BCs decision and the per-BC opt-in protocol (preventing "CQRS everywhere" drift) are TRUE and well-reasoned. STALE bits: (a) Redis named as the optional cache layer (Alt F, "per-BC Redis cache is OPTIONAL") — Redis is RETIRED→Valkey (ADR-0336, license-driven); (b) `related: [ADR-0141]` "workflow→ontology read path direct" — ADR-0141 is Superseded by ADR-0145 (map §1.1); (c) ADR-0005 Kafka eventing reference (outbox pattern survives, Kafka retired).
- **in_masterplan:** NO — no planning front-matter; not in masterplan.json. Closes hyperscaler audit Row C6.
- **tensions:**
  - **Redis→Valkey:** lines 129–132 (Alt F) and the "Redis caches" resource example posture name Redis as the cache substrate. Per ADR-0336 + GLOSSARY L1122, Redis (canonical substrate) is RETIRED→Valkey (BSD-3, OSI-strict). Retired-vocab leak in a load-bearing alternative.
  - **ADR-0141 (Superseded by ADR-0145):** ADR-0172's entire `ontology.entity-query` motivation cites ADR-0141 (workflow→ontology direct read path) as live (lines 27, 213) — but ADR-0141 is Superseded. The high-read ontology path is still real; the citation must retarget to ADR-0145 (inter-µsvc comms reform), which ADR-0172 already also cites for staleness budgets, so the fix is clean.
  - **LINUX cross-side (map §5 #1):** the sharpest cross-repo tension in this chunk — ADR-0172 doubles down on Postgres (Postgres 17 LTS primary + replicas + pgpool-II) as the canonical store, while LINUX ADR-0001 wants a from-scratch Rust multi-model engine that *eliminates the PostgreSQL/sqlx dependency*. ADR-0172 is the concrete embodiment of source's "assemble proven Postgres" posture that LINUX ADR-0001 most directly contradicts.
  - ADR-0005 outbox: survives Kafka retirement (substance TRUE, Kafka-backbone citation stale).
  - ADR-0171 dependency (per-cell pgpool-II HA via ApplicationSets) — same-chunk, coherent.
- **hyperscaler_challenge:** ALIGNED (with one nuance). AWS RDS Aurora (≤15 read replicas), RDS Proxy, Citus, GCP read replicas, Twitter fan-out, LinkedIn Espresso are the cited precedents and the *narrow* read-replica CQRS (explicitly rejecting event-sourced-CQRS-everywhere) is exactly the disciplined choice a hyperscaler makes. Nuance: a true hyperscaler at >50k QPS social-feed scale would likely move `social.feed` to fan-out-on-write or a purpose-built read store (the ADR cites Twitter fan-out but chooses replica-read) — defensible for current scale, flagged for M04 revisit (which the ADR itself does via the sharding "Deferred" alt). Argues for amend (retired refs), not archive.
- **ai_slop:** Low. The read:write ratios (~100×–1000×, ~50×–200×, ~500×) and 10k/50k QPS ceilings are fabricated-precision estimates with no cited telemetry — plausible but unverifiable authored numbers driving the whole decision; flag as the load-bearing-but-unsourced precision. The 6-phase migration plan and failure-mode catalog are genuinely useful, not filler. Alternatives A–F are strong and differentiated.
- **refinement:** Replace Redis→Valkey in Alt F and resource examples; retarget ADR-0141→ADR-0145; note ADR-0005 outbox-survives-Kafka inline; cite a telemetry source (or label as projection) for the QPS/ratio numbers; replace M0x milestones with Wave names; bind to masterplan. Flag the LINUX ADR-0001 "eliminate Postgres" conflict explicitly for the merge.
- **consensus_needed:** no for the source-internal decision; **the cross-side Postgres-own-vs-assemble question is a founder ruling but belongs to the LINUX ADR-0001 audit, not this ADR.**

---

## Chunk notes for synthesis

**1. This is one coherent cluster: "hyperscaler audit rows C1–C6."** ADR-0167 (C1 CLI), ADR-0168 (C2 status page), ADR-0169 (C3 webhooks), ADR-0170 (C4 dev portal), ADR-0171 (C5 federation), ADR-0172 (C6 CQRS) are a single batch authored the same day (2026-05-18) by overlapping councils, each closing one row of `/specs/hyperscaler-architecture-invariants.json`, each tagged "Tier C nice-to-have." ADR-0166 (same date) is the adjacent schema-registry. They should be synthesized as a *group* ("the 2026-05-18 hyperscaler-parity batch"), and they bind to a spec (`hyperscaler-architecture-invariants.json`) but NOT to the masterplan — a clean example of the keystone map's spec-bound-but-masterplan-unbound gap.

**2. Zero masterplan binding across all 7.** None carry `masterplan_ref`/`planning_impact`/`deliverables`; none appear in `masterplan.json` (both verified). Under the founder's "masterplan = SSOT, backfill it with true decisions" goal, the seven decision_atoms above are ready-to-backfill material. Under the *generated-from-ADRs* reading (planning-ssot-consolidation.md), these ADRs would FIRST need `planning_impact` front-matter added before the masterplan could be regenerated from them — so the open authored-vs-generated question directly blocks how this batch gets into the masterplan. Flag both readings.

**3. Dominant pattern = STALE CROSS-REFERENCES, not wrong decisions.** Six of seven are substantively sound but cite retired/superseded peers: Kafka/ADR-0005 (0166, 0169, 0172), ChangeSet/ADR-0110 (0166), Foundry brand (0167, 0168, 0169, 0170, 0171), Istio+ADR-0121 (0171), Redis (0172), ADR-0141 superseded (0172), ADR-0042 superseded (0168), M0x milestones (0167, 0168, 0169, 0170, 0171, 0172). The disposition skews AMEND (4: 0166/0167/0171/0172) / KEEP (2: 0168/0169) / ARCHIVE (1: 0170). The corpus' own superseding ADRs already govern every one of these fixes — this is reconciliation debt, not architectural disagreement.

**4. The one genuine open ruling: ADR-0170 ← ADR-0394.** An Accepted, load-bearing ADR (0170) is marked `Superseded` by a `Proposed` ADR (0394, "must NOT auto-merge"), while three Accepted dependents (ADR-0203/0209/0213 + ADR-SDK-0007) still point at the dead substrate. This mirrors the PR#605 decision-pending pattern (not slop, awaiting founder). It is the only consensus_needed=YES in the slice.

**5. Cross-chunk / cross-side tensions to escalate:**
   - **Forge fault-line touches this chunk twice:** ADR-0170 (line 120 "Oyatie uses GitHub per ADR-0017") and ADR-0171 (ApplicationSet `repoURL: github.com/oyatie/oya.git` baked into canonical IaC). Source canon = Forgejo/bespoke-VCS; founder directive = GitHub. The federation IaC hard-coding GitHub is a concrete artifact of the unresolved fault-line. Surface, do not resolve.
   - **LINUX ADR-0001 vs source Postgres posture:** ADR-0172 is the *most direct* source-side embodiment of "assemble proven Postgres" that LINUX ADR-0001 ("eliminate PostgreSQL/sqlx") contradicts (map §5 #1). The two should be cross-referenced at synthesis as the sharpest data-tier own-vs-assemble conflict.
   - **LINUX ADR-0025 (Rust "Talos") vs source ADR-0171/0375 (actual CAPI/ArgoCD/Talos):** own-the-substrate vs assemble (map §5 #3).

**6. AI-slop level: LOW across the slice, with one consistent tell.** Every ADR leans on **fabricated-precision performance/cost budgets** (CLI ≤25MB/≤80ms; webhook 12M-row estimate; CQRS 10k/50k QPS + read:write ratios; status-page $1.5k/$200 costs) presented without cited telemetry. None are internally contradictory and all are used to justify real decisions, so this is "authored precision" rather than hallucinated filler — but the numbers should be labeled as projections or sourced. The alternatives-considered sections are uniformly strong (genuine A–F enumerations with honest "partial accept"/"deferred" verdicts), which is the opposite of slop. No fabricated source ADRs detected.
