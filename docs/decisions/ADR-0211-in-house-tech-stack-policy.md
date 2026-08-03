# ADR-0211 — In-House Tech Stack Policy

- **Status:** Accepted
- **Date:** 2026-05-18
- **Deciders:** council-architecture, axis-foundry, axis-platform, council-security
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Amends:** none
- **PR:** #143 close-out

## Context

Oyatie aims for hyperscaler-grade quality (Stripe / Linear / Palantir reference). Hyperscalers run on their own tech stack — AWS, Google Cloud, Microsoft Azure, and Oracle each own large portions of their dependency graph end-to-end. Their leverage comes from owning the substrate where it matters and adopting the open standard where adopting yields more leverage than building.

PR #143 introduced a large dependency surface (Cilium, Istio Ambient, Cedar, Wasmtime, OpenTofu, ClickHouse, Milvus, Postgres, Valkey, SeaweedFS, OpenBao, Backstage, Karpenter, Zitadel, Velero, OpenCost, Postal, Meilisearch, svelte-flow, plus many smaller deps). Without an explicit policy on which dependencies stay vs. which we replace in-house, three failure modes appear:

1. **Drift toward proprietary lock-in.** Picking a vendor by convenience today creates exit cost tomorrow. Without policy, ad-hoc adoption of a commercial product (e.g. Drata, Vanta, Snowflake, Datadog, MongoDB Atlas, Auth0) sneaks in via SDK + dashboard + workflow patterns and is then prohibitively expensive to remove.
2. **Drift toward in-house everything.** Building every layer ourselves means we ship slower, accumulate maintenance burden, and lose the network effects of CNCF-graduated standards. Hyperscalers do not rebuild every CNCF tool — they adopt them.
3. **No CI signal on classification.** Without policy, the trade-off is invisible at review time; a contributor cannot tell "is Datadog OK?" vs "is OpenTelemetry OK?" without searching ADRs and meeting notes.

This ADR codifies the classification rubric and the CI enforcement so the trade-off is explicit at scaffold time.

## Decision

**Adopt a three-class classification for every external dependency** declared in the workspace (Rust crates, K8s controllers/operators, SaaS products, runtimes, model providers, payment processors):

### Class A — Community-standard KEEP

The dependency is a CNCF-graduated or Linux-Foundation-hosted community standard with hyperscaler-reference adoption. KEEP indefinitely. Wrap behind a thin adapter trait so swap is possible but expected to remain. No Phase-2 replacement plan required.

**Test for Class A inclusion:**
- CNCF graduated OR Linux Foundation hosted (or equivalent stewardship: W3C, IETF, ISO, HL7, OASIS).
- Adopted by at least two of {AWS, Google, Microsoft, Oracle, Stripe, Cloudflare, Datadog, Adobe, Bell Canada, Capital One, Apple}.
- License is permissive (Apache-2.0 / MIT / BSD) or otherwise compatible with our commercial licensing path (e.g. AGPL3 only if self-hosted with network clause satisfied — see ADR-0186 Grafana cluster).

**Concrete Class A examples (PR-143 substrate):**
- Cilium (CNI / L3-L4) — CNCF graduated.
- Istio Ambient (L7 mesh) — CNCF graduated.
- Envoy Gateway — CNCF graduated.
- Kyverno (K8s admission) — CNCF graduated.
- Cedar (application authz) — AWS-open-sourced; W3C-class standard.
- Wasmtime — CNCF / Bytecode Alliance.
- OpenTofu — Linux Foundation (HashiCorp Terraform fork after BUSL pivot).
- ArgoCD — CNCF graduated.
- Cluster API — Kubernetes SIG.
- Karpenter — CNCF (donated by AWS).
- Prometheus / Mimir / Loki / Tempo / Grafana / OTel / AlertManager — all CNCF.
- OpenBao — Linux Foundation Vault fork after BUSL pivot.
- SPIFFE / SPIRE — CNCF.
- Cosign / Trivy — Sigstore / CNCF.
- PostgreSQL — long-running community standard; PostgreSQL license.
- OpenAPI 3.2.0 (Sept 2025) / AsyncAPI 3.1.0 (Jan 2026) / proto3 — open standards.
- ICU MessageFormat / Fluent (Mozilla) — open standards.
- WCAG 2.2 AA — W3C standard.
- OIDC / WebAuthn L3 / SCIM 2.0 — IETF / W3C / OASIS standards.
- Loro CRDT — open-source community.
- FHIR R4 — HL7 International standard.

These are the substrate, not the differentiation. We adopt and contribute back where useful (upstream PRs); we do not rebuild.

### Class B — Vendor-replaceable Phase-2 native

The dependency is a current-best vendor product that we use today behind a thin adapter trait, with an explicit plan to replace it with a native in-house implementation at a future Phase-2 milestone. The trigger for Phase-2 must be **value-anchored** (a measurable need or a code-level event), not **date-anchored** (a calendar deadline). Phase-2 may never trigger — that's acceptable; the discipline is to be ready.

**Test for Class B inclusion:**
- The dependency provides leverage today but limits our ceiling at hyperscaler scale.
- Phase-2 replacement is plausible (we have or can build the expertise).
- A specific quantitative trigger exists (tenant scale, region count, latency budget, license change, regulatory requirement).

**Concrete Class B examples (PR-143 substrate):**

| Vendor (today) | Phase-2 (native) | Trigger |
| --- | --- | --- |
| Keycloak (human OIDC bridge per ADR-0476/ADR-0482) | oya-identity | OIDC + OAuth 2.0 + WebAuthn + tenant IdP federation + MFA feature parity and `oya-identity` integration suite green; ADR-0187/Zitadel is superseded and is not the live default |
| Milvus (vector DB) | oya-vector-store-server | ≥1B vectors per cluster OR Foundry/intelligence RAG latency budget breached |
| ClickHouse (OLAP) | oya-olap-warehouse-server (DataFusion + Arrow + Parquet + custom merge-tree) | ≥100TB per tenant OR cross-tenant isolation breach |
| SeaweedFS (object store) | oya-object-store-server | ≥1PB cluster OR multi-region active-active write coordination need |
| Velero (K8s backup orchestrator) | oya-backup-orchestrator | AWS-Backup-parity needed at multi-tenant scale |
| OpenCost (FinOps) | oya-finops-portal | tenant-billing-portal needs OpenCost can't cover |
| Postal (sovereign email) | oya-comms-email-server | ≥1M sends/day/tenant × 7d sustained OR sovereign air-gap deploy |
| Backstage (developer portal) | oya-developer-portal | Backstage scope exceeded for service catalog + TechDocs |
| svelte-flow (canvas) | oya-canvas (Leptos-native, shares oya-client-shared-rust with Linux GTK) | ≥10K nodes per workflow OR svelte-flow perf ceiling per ADR-0185 |
| Meilisearch (search) | oya-search-server (Tantivy-based) | BUSL enterprise tier requirement OR Meilisearch licensing concern escalation |

Each Class B vendor MUST register in `registry/vendor-lockin-phaseout/index.json` with:
- `tier: II` (vendor-seamed)
- `seam_adapter_trait`: workspace path to the trait.
- `seam_adapter_impls`: ≥1 workspace path to a concrete adapter implementation.
- `replacement_path`: workspace path where the in-house Phase-2 will live.
- `replacement_readiness_gate`: the value-anchored trigger (above table column).

The `oya gate validate vendor-lockin-discipline` gate (BLOCKER per ADR-0173) enforces this registry shape.

**Identity disambiguation amendment (2026-07-02).** ADR-0394's IDP is the Internal Developer Platform portal/BFF, not the OIDC identity provider. Human identity is governed by ADR-0476/ADR-0482: Keycloak bridges Phase 1 and `oya-identity` is the founder-accepted bespoke Rust target after feature parity. The older ADR-0187/Zitadel row is historical and must not be used as ownership-ratchet live default or as a hidden ≥50K-tenant trigger in new prose.

### Class C — In-house mandatory differentiation

The capability IS our differentiation. It must be 100% in-house from day one. No vendor — even a temporary one — is appropriate, because using one would either leak our domain semantics to a competitor's surface or saddle us with an adapter that will leak abstractions into our value proposition.

**Test for Class C inclusion:**
- The capability is named in the product narrative as a differentiating moat.
- A vendor product would actively undermine the moat (data ownership, semantic ownership, integration depth).
- The cost of building in-house is justified by the moat value.

**Concrete Class C examples:**
- **Workflow Studio** (n8n-class hero product) — the user-facing automation engine + visual canvas that bridges every µservice via Workflow + Ontology adapter layer.
- **Ontology** (Palantir-class canonical data layer) — the typed entity graph that makes Workflow + intelligence cross-product coherent.
- **Foundry** (internal Hermes agentic substrate per ADR-0136 amendment) — the dev pipeline that builds every µservice + the eval substrate.
- **Intelligence** (consumer AI substrate per ADR-0220) — the user-facing AI brand surface.
- **Audit Chain** (Ed25519 + Merkle per Bominal ADR-0028) — the tamper-evident regulatory substrate.
- **Cell architecture** (per ADR-0009) — the multi-tenant blast-radius isolation.
- **Consent Graph** (per ADR-0214) — the cross-tenant visibility moat (LinkedIn-class graph).
- **Plugin App Store + Developer SDK** (per ADR-0213) — the third-party ecosystem moat.
- **Compliance evidence pipeline** (Drata / Vanta replacement; per ADR-0209) — in-house from day one because regulators trust must NOT route through a third party that could pivot pricing / policy / outage.

## CI enforcement

Three gates enforce this policy:

1. **`oya gate validate vendor-lockin-discipline`** (ADR-0173, BLOCKER) — verifies the `registry/vendor-lockin-phaseout/index.json` shape (tier classification, seam_adapter_trait + impl present for Tier II, refusal rationale + replacement path for Tier III).

2. **`oya gate validate vendor-classification-coverage`** (queued; PR-144) — verifies every workspace dependency declared in `Cargo.toml` AND every µservice manifest dependency declares its class.

3. **`oya-check-version-pin-source-cited`** (queued; PR-144 per ADR-0221 §M-01) — every version pin in ADRs/PRDs/specs must cite a WebSearch / Context7 / upstream source URL adjacent to the pin (date-anchored Phase-2 triggers are explicitly forbidden by the regex).

## In-house roadmap

This ADR is itself in-house policy — it codifies *when* a dependency moves from vendor to in-house. There is no Phase-2 replacement for this ADR; it is a doctrine, not a runtime capability.

Doctrine evolution is governed by the standard ADR amendment process (a new ADR-XXXX amending this one) — not by silent edit. The doctrine should be re-examined whenever:

- A new Phase-2 trigger fires (e.g. a Class B vendor crosses its trigger threshold and Phase-2 is initiated).
- A vendor changes license terms in a way that affects Class A eligibility (e.g. HashiCorp BUSL pivot triggered OpenTofu / OpenBao forks; ElasticSearch SSPL pivot triggered OpenSearch).
- A new CNCF graduation or W3C/IETF standard makes a previously Class B capability eligible for Class A.
- A new Class C capability is identified (a new product narrative moat).

## Alternatives considered

### Alternative 1 — "No policy; trust contributor judgment"

**Rejected because** in a multi-agent codebase with 70+ µservices, contributor judgment varies. Without policy, we end up with three vendors for the same capability (search, observability, secrets) within six months, and each one creates lock-in tax. Lack of policy *is* a policy — it's the "drift to lock-in" policy.

### Alternative 2 — "Build everything in-house from day one"

**Rejected because** hyperscalers themselves don't do this. AWS runs on a mix of Linux, PostgreSQL, Apache, OpenJDK, OpenTelemetry, and only builds the layers that are their differentiation. Building everything in-house dilutes engineering capacity away from Class C moat work and ships slower. The opportunity cost (delayed Workflow Studio / Ontology / Foundry / Intelligence depth) is unacceptable.

### Alternative 3 — "Adopt vendor for everything; never replace"

**Rejected because** at hyperscaler scale, every Class B vendor has a ceiling. Hitting the ceiling without a Phase-2 plan creates a forced migration under duress — exactly when we have the least engineering capacity. Building the adapter trait now and the Phase-2 implementation when the trigger fires is cheaper than emergency replacement.

### Alternative 4 — "Date-anchored Phase-2 triggers (e.g. 'replace Zitadel by Q3 2027')"

**Rejected because** date-anchored triggers create false urgency or false complacency. If the trigger date passes and the value is still there, we waste engineering capacity replacing a working dependency. If we hit the value-anchored trigger before the date, we delay replacement and incur risk. Value-anchored triggers (≥50K tenants, ≥1B vectors, ≥100TB) tie engineering work to measurable need. Per ADR-0221 §M-01, the CI gate `oya-check-version-pin-source-cited` explicitly forbids date-only Phase-2 triggers in any ADR.

### Alternative 5 — "Per-µservice policy (each µservice decides)"

**Rejected because** dependencies cut across µservices. If `microservices/identity/` adopts Auth0 and `microservices/foundry/` adopts Zitadel, we end up with two adapter surfaces, two evidence pipelines, two audit-chain integrations — duplicated work with no benefit. Policy must be workspace-wide.

## Consequences

### Positive

- **Hyperscaler-pattern alignment.** AWS, Google, Microsoft, Oracle do exactly this — adopt the open standard where it's the standard, build the differentiation where it's the moat. We get to leverage every CNCF graduation while owning the layers where we have to.
- **CI-enforceable.** `oya gate validate vendor-lockin-discipline` already enforces the registry shape (BLOCKER per ADR-0173). The classification is checkable at PR time, not at architecture-review time.
- **Phase-2 readiness.** Every Class B vendor has a documented seam, an in-house replacement path, and a value-anchored trigger. When the trigger fires, we already know what to build and where.
- **Cost discipline.** Phase-2 work is not started speculatively — only when the value trigger fires. Engineering capacity stays focused on Class C moat work.
- **Open-integration alignment with ADR-0216.** Trust via openness is the moat. Class A adoption signals that we use open standards; Class B documents the exit path; Class C is where we differentiate. Customers know they CAN leave if our Class C value declines.

### Negative

- **Classification overhead at scaffold time.** Adding a new dependency requires classifying it (A / B / C) and, for Class B, authoring the seam + Phase-2 path + trigger. This is real friction on minor utility additions.
  - **Mitigation:** ship a `cargo run -p oya-dev-cli -- vendor classify <crate>` helper (queued for PR-145) that proposes the classification from CNCF / Linux Foundation / SPDX metadata and prompts only on ambiguous cases.
- **Tension with rapid prototyping.** A speculative µservice may not be ready to commit to Class A / B / C. Marking as Class B (vendor-replaceable) is the default escape hatch but adds the trait + registry entry overhead.
  - **Mitigation:** prototype µservices may declare a transient `tier: II-pre` (pre-classified) entry in `registry/vendor-lockin-phaseout/index.json` that the validator accepts without seam_adapter_impls; the entry must classify into II or III before GA.
- **Class B Phase-2 work may compound.** If multiple Class B triggers fire simultaneously (e.g. Zitadel + Milvus + ClickHouse all cross scale thresholds in the same quarter), engineering capacity is overwhelmed.
  - **Mitigation:** quarterly vendor-trigger review by axis-platform; visibility into trigger proximity dashboards (queued for management-cockpit per ADR-0220).

### Operational

- **Vendor-lockin registry is the source of truth.** Any agent or human adding a dependency MUST update `registry/vendor-lockin-phaseout/index.json`. The CI gate fails the PR otherwise.
- **Phase-2 trigger monitoring.** Each Class B vendor's trigger is also a monitoring signal — tenant count, vector count, OLAP volume, sends/day, etc. These should be wired into observability per ADR-0210 trace-sampling-recipe so we see triggers approach in real-time.
- **License changes are first-class events.** When a Class A vendor changes license (HashiCorp BUSL, Elastic SSPL, MongoDB SSPL, Redis BSL), we re-classify within 30 days. Doctrine review checkpoint required.
- **Foundry vs Intelligence split (per ADR-0136 amendment + ADR-0220).** Foundry is INTERNAL; Intelligence is CONSUMER. Both are Class C (in-house mandatory). The shared substrate (Milvus, Wasmtime) is Class B but partitioned per-cell per-µservice — no shared collection.

## References

- ADR-0009 — Cell architecture (multi-tenant blast-radius isolation).
- ADR-0028 (Bominal inheritance) — Audit chain Ed25519 + Merkle.
- ADR-0061 — Per-product à-la-carte enablement.
- ADR-0083 — Tier 3 / Tier 1 source discipline (no `.unwrap()` in production code).
- ADR-0136 — Foundry as single µservice (consolidation 6→1); amendment-2026-05-18 clarifies INTERNAL scope.
- ADR-0145 — Cross-product invariants (audit + tracing + ontology-projection).
- ADR-0148 — Layered service mesh (Cilium L3/L4 + Istio Ambient L7).
- ADR-0173 — Vendor lockin discipline (Tier I / II / III classification + CI gate).
- ADR-0186 — Self-hosted Grafana AGPL3 (network clause satisfied).
- ADR-0192 — Milvus vector store.
- ADR-0193 — ClickHouse OLAP.
- ADR-0196 — SeaweedFS object store.
- ADR-0197 — Velero backup orchestration.
- ADR-0198 — Karpenter autoscaling.
- ADR-0199 — Tenant cost labels + FinOps via OpenCost.
- ADR-0200 — Wasmtime substrate.
- ADR-0201 — Postal sovereign email.
- ADR-0202 — OpenTofu IaC (Terraform deprecated path).
- ADR-0205 — svelte-flow canvas (Phase 1) → Leptos (Phase 2) per ADR-0185.
- ADR-0209 — In-house compliance evidence pipeline (Drata / Vanta replacement).
- ADR-0212 — Buildability doctrine.
- ADR-0213 — Plugin App Store + Developer SDK ecosystem moat.
- ADR-0214 — Consent graph cross-tenant visibility moat.
- ADR-0216 — Open integration & migration-out policy.
- ADR-0220 — Consumer intelligence substrate (`microservices/intelligence/`).
- ADR-0221 — Agentic development pipeline hardening (CI gates for version-pin source citation + date-anchored trigger detection).

## Named industry sources

- AWS — adopts Linux, PostgreSQL, Apache, OpenJDK, OpenTelemetry, Wasmtime, Cedar (open-sourced by AWS); builds Aurora, DynamoDB, S3, Nitro, Karpenter (donated to CNCF).
- Google — adopts CNCF stack pervasively (CNCF was Google-founded); builds Borg → Kubernetes (donated to CNCF), Spanner, BigQuery, Pub/Sub.
- Microsoft Azure — adopts Linux, PostgreSQL, OpenJDK, CNCF stack; builds Cosmos DB, AKS, Service Bus.
- Oracle — adopts Linux (Oracle Linux), MySQL (post-acquisition); builds Exadata, Autonomous Database.
- Stripe — adopts Ruby, PostgreSQL, MongoDB, Kafka; builds Stripe-internal ML stack, Stripe SDK.
- Cloudflare — adopts Rust, OpenSSL (BoringSSL fork), Linux; builds Workers, R2, Durable Objects.
- Datadog — adopts CNCF stack (OTel, Prometheus); builds Datadog APM, Synthetic Monitoring.
- Adobe — adopts CNCF stack pervasively; runs Cilium + Istio at production scale (CNCF case study).
- Bell Canada / Capital One — adopt CNCF stack (Cilium, Istio); reference cases for hyperscaler-pattern adoption.

These references substantiate the "Class A community-standard with hyperscaler-reference adoption" test — every Class A dependency in our registry must trace to ≥2 of the above adopters.
