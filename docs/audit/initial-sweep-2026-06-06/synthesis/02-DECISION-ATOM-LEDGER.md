# 02 — DECISION-ATOM LEDGER (masterplan backfill spec)

> Every TRUE decision atom surviving the audit, DEDUPED across both sides. This IS the masterplan backfill spec: under the ratified generated-from-ADRs model (ADR-0364/0365), each atom becomes a `planning_impact:true` ADR (or amended front-matter) keyed to a `domain` enum; the generator emits the masterplan from these.
> Columns: **atom | source ADR id(s) | domain | in_masterplan (yes/no/partial) | proposed masterplan section**.
> Cross-side dedupe rule: where LINUX and SOURCE decide the same thing, ONE atom carries both ids with the own-vs-assemble phasing noted. Retired-vocab-only AMENDs do NOT create new atoms — they refine an existing one.
> **Proposed `domain` enum (closed, ~16) used below:** `governance-ssot`, `policy-authz`, `identity-crypto`, `data-storage`, `eventing`, `observability`, `isolation-runtime`, `orchestration-fleet`, `ci-cd`, `forge-scm`, `build-toolchain`, `intelligence-ai`, `tenancy`, `api-surface`, `compliance-residency`, `ownership-doctrine`.

---

## A. governance-ssot (the masterplan machinery itself)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Masterplan is a GENERATED projection of the accepted `planning_impact:true` ADR log; status derived from gates; hand-edits fail a drift gate | SRC-0364, SRC-0365 | governance-ssot | yes | §0 SSOT-model (THE FORK — ratify) |
| Generative ADR template: rich structured front-matter, lean prose; `domain` enum field; ADR-0365 lifecycle + domain-cohesion gate | SRC-0364, SRC-0365 | governance-ssot | yes | §0 ADR-contract |
| ADRs are immutable authored SSOT (append-only; supersede, never edit); re-found log from ADR-0000 with `consolidates:` provenance | SRC-0364, ideas/planning-ssot-consolidation | governance-ssot | partial | §0 ADR-immutability |
| Doc-as-SSOT machinery: per-doc owner/trigger/cadence/deps + machine-readable mirror + validation lanes | SRC-0019 | governance-ssot | no | §0 doc-catalog |
| Machine-readable artifact contract (9-capability declaration; registry control plane) as SSOT-binding mechanism | SRC-0069 | governance-ssot | no | §0 artifact-contract |
| Worth-documenting ⇒ reachable-from-masterplan; emit a must-read session-context bundle (authority-chain + operating contract + AGENTS.md + standards) | README binding principle | governance-ssot | no | §0 session-context-bundle |
| Domain-cohesion: closed `domain` enum + read-set resolution (enum-keyed first, vector recall later) + contradiction gate at decision time | README binding principle | governance-ssot | no | §0 cohesion-gate |
| No-dangling / no-reuse ADR-id invariant (supersedes/amends edges must resolve); `decisions.json next_adr` re-derived from disk | keystone §6, SRC-0364 | governance-ssot | no | §0 id-discipline |
| Decision-debt elimination: every Proposed ADR → ratify/drop with door-class (one-way ⇒ sign-off; two-way ⇒ auto-green) | SRC-0364, README mandate | governance-ssot | no | §0 proposed-ledger |
| Function-named semantic ids; ADR-number-keyed / M0x-keyed gate names FORBIDDEN | keystone §2, consolidation | governance-ssot | no | §0 naming-canon |
| Sunset→deprecation→removal lifecycle automation schema + generic lifecycle framework | SRC-0108, SRC-0109 | governance-ssot | no | §0 lifecycle |
| Aspirational-enforcement / honest-claims gates; portfolio remediation backlog refuses self-certification | SRC-0123, SRC-0129, SRC-0134, SRC-0135 | governance-ssot | no | §0 honesty-gates |
| Hyperscaler architecture invariants spec + portfolio binding | SRC-0128 | governance-ssot | no | §0 invariants |

## B. ownership-doctrine (the shared ratchet — dedupe target)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| **Own-when-proven ratchet (ONE reconciled rubric)** — vendored/assemble now; own only when it beats the vendored oracle on a four-axis no-cherry-pick scorecard over a sustained production span | SRC-0173, SRC-0211 + L-0019, L-0020, L-0022 | ownership-doctrine | partial | §1 ratchet (RECONCILE both vocabularies) |
| Portfolio-level capacity gate: pick ONE crown-jewel from-scratch substrate day-0; the rest DEFER_VENDORED (the missing gate) | NEW (synthesis) ← §4 hyperscaler | ownership-doctrine | no | §1 capacity-gate |
| Numeric-trigger "decided-but-deferred" ownership pattern (cutover fires on a measured threshold) | SRC-0510 | ownership-doctrine | partial | §1 deferred-trigger |
| Default-deny vendor adoption + seam-and-multi-impl + phase-out registry | SRC-0173 | ownership-doctrine | no | §1 vendor-policy |
| OSS stewardship Class-C + CVE-response SLA | SRC-0345 | ownership-doctrine | no | §1 oss-stewardship |
| Soften the "no custom silicon" anti-scope from "never" to "not in the day-0 horizon" (TPU/Graviton/Cobalt precedent) | SRC-0032 (amend) | ownership-doctrine | no | §1 silicon-scope |

## C. data-storage (keystone fault #1 — own-vs-assemble, phased)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| OLTP = Postgres + Citus, retained as reused substrate of record; Postgres 18.4 Tier-1 source-of-truth | SRC-0045, SRC-0179, SRC-0184 + L-0001 (concedes) | data-storage | partial | §2 oltp (FOUNDER: own-end-state?) |
| Owned Rust multi-model engine (cloud-data) = the Spanner-class DIFFERENTIATOR; replaces etcd as orchestration datastore; pg-wire optional | L-0001 | data-storage | no | §2 differentiator-engine |
| Postgres connection pooling canonical: pgcat | SRC-0179 | data-storage | partial | §2 pooling |
| Vector DB: Milvus >10M; pgvector ≤10M | SRC-0192 (sup. 0046) | data-storage | partial | §2 vector |
| OLAP warehouse: ClickHouse 26.3 LTS, layered on Iceberg | SRC-0193, SRC-0337 | data-storage | no | §2 olap |
| Tenant timeseries: TimescaleDB | SRC-0194 | data-storage | no | §2 timeseries |
| Object storage: SeaweedFS primary + Ceph RGW scale-up | SRC-0196 | data-storage | no | §2 object |
| Cache/KV: Valkey (not Redis) — relicense response | SRC-0336 | data-storage | no | §2 cache |
| Storage tier layering OLTP/read-replica/cache/search; CSI StorageClass abstraction (hot/warm/cold) | SRC-0184, SRC-0161 | data-storage | no | §2 storage-tiers |
| Read replicas + CQRS where appropriate (per-µsvc opt-in) | SRC-0172 | data-storage | no | §2 cqrs |
| Storage local-now, disaggregation-ready | L-0005 | data-storage | no | §2 storage-evolution |
| Time coordination: HLC default + swappable Clock port for TrueTime at Tier-4 | SRC-0252 + L-0006 | data-storage | no | §2 time-coordination |
| Search engine: Meilisearch 1.9 day-1 → Tantivy Phase-2 (pgroonga dropped) | SRC-0184 (sup. 0047) | data-storage | no | §2 search |
| KR morphology + multilingual tokenization (Tokenizer trait; mecab-ko/khaiii) re-homed onto search path | SRC-0048 | data-storage | no | §2 tokenization |

## D. eventing

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Eventing backbone: Pulsar 4.x + Oxia (KoP wire-compat); Kafka retired | SRC-0377-kafka (sup. 0005), SRC-0195 | eventing | no | §3 broker |
| Transactional-outbox pattern + CloudEvents envelope + per-tenant/per-cell partitioning (survives broker swap) | SRC-0005 (atoms), SRC-0153 | eventing | no | §3 outbox |
| Event schema versioning; ULID as canonical event_id (needs a real home) | SRC-0154 | eventing | no | §3 schema-versioning |
| Schema registry: Apicurio (Confluent-compat; AsyncAPI 3.x + proto3 + OpenAPI 3.1) | SRC-0166 | eventing | no | §3 schema-registry |
| Webhook DLQ + exponential-backoff retry kernel | SRC-0169 | eventing | no | §3 webhook-delivery |

## E. observability

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Observability stack: Loki/Tempo/Mimir/Grafana (LGTM, AGPL-3 carve-out) — authoritative over the 5-stage layering | SRC-0383 (sup. 0042), SRC-0186 (defers) | observability | no | §4 stack |
| OTel emission contract (gen_ai semconv) | SRC-0263 | observability | no | §4 emission |
| Observability backplane layering: collection/storage/query/alert/SLO authoring, zero overlap | SRC-0186 | observability | no | §4 layering |
| SLO composition + inheritance arithmetic; five-tier RPO/RTO recovery model | SRC-0180, SRC-0152 | observability | no | §4 slo |
| Public status page derived from SLO state | SRC-0168 | observability | no | §4 status-page |

## F. policy-authz (own-vs-reuse Cedar — dedupe target)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| **Cedar = external-standard authorization CONTRACT; owned compile-to-Rust PARC engine behind it** (vendor cedar-policy as day-0 adapter + differential oracle; own the compiler when it beats the oracle) | SRC-0243, SRC-0246, SRC-0183(principle) + L-0021 | policy-authz | no | §5 authz-engine (FOUNDER: own vs reuse) |
| Re-author the missing canonical Cedar-engine pick (phantom ADR-0150-cedar does not exist) or fold into 0243 | SRC-0243 (NEW) | policy-authz | no | §5 authz-engine-anchor |
| Cedar fragment soak / anomaly / rollback governance | SRC-0294 | policy-authz | no | §5 fragment-governance |
| Autonomy ceiling: persona-tier T1–T4, runtime-enforced via Cedar at every capability invocation; effective-ceiling = min; no-false-allow | SRC-0022 (authority), SRC-0007, SRC-0099 | policy-authz | no | §5 autonomy-ceiling |
| Edge-authz tier vs origin Cedar PDP boundary | SRC-0191 | policy-authz | no | §5 edge-origin |
| Default k8s admission: Kubewarden (Cedar app-authz vs admission separation retained; Kyverno demoted to adapter) | SRC-0379 (sup. 0183) | policy-authz | no | §5 admission |
| EU AI Act graduated risk-tier model (namespaced distinct from autonomy/tenant/storage tiers) | SRC-0144 | policy-authz | no | §5 eu-ai-act |
| Per-µsvc data-class hard-deny overrides (tenant-admin cannot raise) | SRC-0034 | policy-authz | no | §5 data-class-floor |
| Four-eyes default-deny write-gate | SRC-0091 | policy-authz | no | §5 write-gate |

## G. identity-crypto (one hard contradiction — resolve to 0476)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| **Owned bespoke Rust identity (oya-identity) is canonical destination; Zitadel/Keycloak demoted to Phase-1 bridges** (founder-locked) | SRC-0476 (sup. 0187) | identity-crypto | no | §6 idp (FOUNDER: confirm) |
| Tenant+identity kernel = single substrate every axis consumes | SRC-0002 | identity-crypto | no | §6 identity-kernel |
| Passkey/WebAuthn substrate; bespoke webauthn-rs relying party | SRC-0188, SRC-0507 | identity-crypto | no | §6 webauthn |
| Step-up authentication ACR classes | SRC-0189 | identity-crypto | no | §6 step-up |
| SCIM 2.0 provisioning for enterprise tenants | SRC-0190 | identity-crypto | no | §6 provisioning |
| Crypto provider: aws-lc-rs (→ owned oya-crypto); authenticator reference OpenSK | SRC-0506, SRC-0508 | identity-crypto | no | §6 crypto |
| Secrets: OpenBao (over BUSL-Vault) + per-tenant/per-cell HSM (KCMVP for KR) + per-capability SecretProvider | SRC-0043 | identity-crypto | no | §6 secrets |

## H. isolation-runtime (keystone fault #3 — own-host vs assemble, phased)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Runtime ladder: native → sandbox → microVM → confidential (Cloud-Hypervisor primary; Kata/firecracker); pod runtime tiers 0–3 | SRC-0147, SRC-0338 | isolation-runtime | no | §7 runtime-ladder |
| **Isolation default for first-party: runc-for-density (0338) vs assume-breach-microVM-for-all (L-0023)** — opposite defaults | SRC-0338 vs L-0023 | isolation-runtime | no | §7 default (FOUNDER) |
| WASM runtime canonical: wasmtime + WASI Preview 2 (capability-gated, Cosign-signed, trust tiers) | SRC-0200, SRC-0036 | isolation-runtime | no | §7 wasm |
| Container runtime: one OCI/CRI frontend + pluggable IsolationBackend port (native/sandbox/microvm/confidential) | L-0014 | isolation-runtime | no | §7 runtime-frontend |
| **Owned host: framekernel + Capsule + owned VMM** — gated successor to Talos+Kata (H2, uncommitted; consensus=FALSE) | L-0018, L-0014, L-0017 | isolation-runtime | no | §7 owned-host (FOUNDER: target vs research) |
| Container base image: distroless static-debian12:nonroot | SRC-0146 | isolation-runtime | no | §7 base-image |
| Assume-breach / strength-by-blast-radius posture (authorship NOT a trust axis); BeyondProd/Nitro/NIST-800-207 | L-0023 | isolation-runtime | no | §7 security-posture |
| Own L0–L8 container platform (build/registry/conmon) — DEFER L7 build-engine per hyperscaler lens | L-0017 | isolation-runtime | no | §7 container-platform |
| Kernel-level capabilities as ports (Linux extension now, frame later); latent owned eBPF dataplane | L-0026, L-0024 | isolation-runtime | no | §7 kernel-capabilities |

## I. orchestration-fleet

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Fleet substrate: Talos immutable node-OS + CAPI + ArgoCD (Sidero bare-metal) | SRC-0375 (sup. 0121/0120), SRC-0370, SRC-0378, SRC-0382 | orchestration-fleet | no | §8 fleet |
| Node-OS: adopt Talos day-0; owned Rust "Talos" gated successor (Rust-vs-Go security edge structural, unproven) | L-0025 + SRC-0375 | orchestration-fleet | no | §8 node-os (FOUNDER: funded destination?) |
| Cellular scale-out, bounded blast radius (~5k-node k8s limit) | L-0012 | orchestration-fleet | no | §8 cellular |
| Orchestration control-plane: own the cellular + owned-datastore differentiator under an etcd-v3 adapter (NOT a full apiserver/scheduler rewrite) | L-0015 (amend) | orchestration-fleet | no | §8 control-plane (FOUNDER scope) |
| Multi-cluster federation: ArgoCD ApplicationSets + Cluster API | SRC-0171 | orchestration-fleet | no | §8 federation |
| Kubernetes-everywhere; deployment-model spectrum | SRC-0254 | orchestration-fleet | no | §8 k8s-everywhere |

## J. ci-cd + build-toolchain

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Build/RBE: Buck2 + Reindeer + NativeLink-RBE (Bazel/rules_rust reversed) | SRC-0392, SRC-0408 (rev. 0358) | build-toolchain | no | §9 build |
| **CI destination: bespoke-Rust oya-ci Prow (0513, founder-locked) vs Argo Workflows (0511)** — reconcile | SRC-0513 vs SRC-0511 | ci-cd | no | §9 ci-destination (FOUNDER) |
| CD: ArgoCD + Argo-Rollouts; progressive delivery canary/blue-green, burn-rate-gated rollback (reconcile Flagger-0160 → Argo-Rollouts) | SRC-0040, SRC-0160(amend), SRC-0511 | ci-cd | no | §9 cd |
| `oya` gate engine = governance overlay; gate sink = forge commit-status; Jenkins transitory bootstrap only | SRC-0513, SRC-0380, SRC-0511 | ci-cd | no | §9 gate-engine |
| Agentic SLO-gated promotion (resolve ledger contradiction: Mimir-as-ledger vs git-JSONL) | SRC-0139 | ci-cd | no | §9 promotion-gate |
| Supply-chain: Trivy 4-layer + Cosign keyless + SBOM dual-format + signed commits/tags (admission = Kubewarden) | SRC-0039 | ci-cd | no | §9 supply-chain |
| Container image promotion pipeline dev→staging→prod, cosign-signed | SRC-0181 | ci-cd | no | §9 image-promotion |
| Automation-first pipeline: sccache + remote-exec + affected-graph testing | SRC-0050 | ci-cd | no | §9 automation |
| Chaos engineering substrate (Chaos Mesh vs Litmus — reconcile to Argo ecosystem) | SRC-0165 | ci-cd | no | §9 chaos |
| GitOps: trunk-based dev, release branch at tag, root-Cargo merge serialization (forge-neutral) | SRC-0041 | ci-cd | no | §9 gitops |
| **[PROMOTE] Affected-gated migration engine ("Sweep"): risk-classed mass-transform + auto-quarantine + auto-merge-on-green** (Tide client) | ideas/affected-gated-migration-engine | ci-cd | no | §9 sweep (FOUNDER promote-narrow) |

## K. forge-scm (the three-way fault-line — FOUNDER)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| **Canonical forge HOST: GitHub (founder directive) vs Forgejo-transitory (0363) vs bespoke-VCS-destination (0510)** | SRC-0017, SRC-0363, SRC-0374, SRC-0387, SRC-0510 | forge-scm | partial | §10 forge-host (FOUNDER) |
| CI automation SUBSTRATE is forge-neutral (gates ≠ `gh api`/Actions/branch-protection REST) | SRC-0039/0041/0124/0139/0170/0171 (amend) | forge-scm | no | §10 forge-substrate |
| Merge automation = projected-merge-state + fix-at-any-stage, folded into oya-ci Tide (salvaged from retired 0111/0124) | SRC-0111, SRC-0124 (salvage) | forge-scm | no | §10 merge-queue |
| Brand + repo layout: Oyatie/oYa/oyatie.com; `oya-` Cargo prefix; canonical monorepo `{oya,cloud}/<service>/` | SRC-0017, SRC-0512 | forge-scm | no | §10 brand-layout |

## L. intelligence-ai

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Intelligence = two-layer AI substrate (consumer AI + internal self-modification); absorbs retired Foundry; Governance separate | SRC-0255, SRC-0335, SRC-0220 | intelligence-ai | no | §11 substrate |
| Multi-provider adapter model (ProviderAdapter trait + runtime router + failover + cost-ceiling pre-flight) — re-home to intelligence; scope-down programmatic consumer-subscription auth | SRC-0020, SRC-0384 | intelligence-ai | no | §11 provider-routing |
| Capability registry-as-SSOT + MCP discovery + per-tenant endpoint isolation | SRC-0021 | intelligence-ai | no | §11 capability-registry |
| LLM/inference gateway (OAuth subscription-pool redesign) | SRC-0384 | intelligence-ai | no | §11 inference-gateway |
| Plugin substrate marketplace economics (wasmtime + Cosign + trust tiers) | SRC-0036 | intelligence-ai | no | §11 plugin-marketplace |
| Supervisor: Rust (not Node); best-effort→full durability for audit-adjacent rows | SRC-0096, SRC-0098 | intelligence-ai | no | §11 supervisor |
| Meta-trust-root for self-modification | SRC-0293 | intelligence-ai | no | §11 trust-root |
| **[PROMOTE] Agent-execution-controller (PR#605): run a CLI agent as a K8s Job → sealed evidence-bundle.v1; per-changeset cost budgets + override-frequency alarms (salvaged from 0113)** | ideas/agent-execution-controller, SRC-0113(salvage) | intelligence-ai | no | §11 agent-execution (FOUNDER promote-narrow vs decline) |
| Self-hosting / self-modification doctrine; oyatie-is-a-tenant | SRC-0247, SRC-0242 | intelligence-ai | no | §11 dogfood |

## M. tenancy

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Tenant = universal scoping primitive; tenant-class (demo_trial/paid) + composable billing_components (NOT tiers) | SRC-0244, SRC-0329 (sup. 0316) | tenancy | no | §12 tenant-model |
| Per-tenant audit-chain slicing (partition by tenant_id; sovereign dedicated shard; retrieval API) | SRC-0162 | tenancy | no | §12 audit-slicing |
| Per-tenant resource quotas; layered throttling (IP/key/user/tenant) re-keyed to tenant_class | SRC-0155, SRC-0178 | tenancy | no | §12 quotas |
| Tenant lifecycle workflow | SRC-0175 | tenancy | no | §12 lifecycle |
| Environment stages (test/staging/prod) — renamed from "environment tiers" | SRC-0163 | tenancy | no | §12 env-stages |
| Feature-flag µsvc for runtime gradual rollout (separate from ChangeSet acceptance) | SRC-0159 | tenancy | no | §12 feature-flags |
| FinOps cost-attribution + chargeback policy | SRC-0174 | tenancy | no | §12 finops |

## N. api-surface

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Public API stability tiers preview/stable/GA; semver-diff PR gate; contract-first SDK generation | SRC-0037 | api-surface | no | §13 stability |
| API gateway (north-south) vs service mesh (east-west) separation, zero overlap; dedicated gateway tier | SRC-0182, SRC-0157 | api-surface | no | §13 gateway-mesh |
| Service mesh: Cilium L3/L4 + Istio Ambient L7 (layered, zero overlap) | SRC-0148 (sup. 0044 framing) | api-surface | no | §13 mesh |
| HTTP backbone: hyper canonical + strategic hyper/axum split | SRC-0090 | api-surface | no | §13 http-backbone |
| API hygiene canon: idempotency keys, cursor pagination, X-Request-Id propagation, latency-budget reporting, typed Handler trait, TenantSlug home | SRC-0149, SRC-0150, SRC-0151, SRC-0093, SRC-0094, SRC-0095 | api-surface | no | §13 api-hygiene |
| Internal vs external API surface separation | SRC-0177 | api-surface | no | §13 surface-separation |
| Brown-out + graceful-degradation signal API | SRC-0176 | api-surface | no | §13 degradation |
| Tenant-facing CLI `oya` (separate from internal oya-dev-cli) | SRC-0167 | api-surface | no | §13 cli |
| gRPC transport provisional then owned framing | L-0007 | api-surface | no | §13 grpc |
| Rust error-handling tiers (thiserror/anyhow/no-panics); workspace dependency-seam policy; idempotency middleware | SRC-0083, SRC-0092 | api-surface | no | §13 rust-conventions |
| CRDT portability trait + alternate-adapter compile gate | SRC-0142 | api-surface | no | §13 crdt-portability |

## O. compliance-residency

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Cross-region replication + residency: per-pack default class, opt-in cross-region per consent, immutable post-create (recreate-not-mutate) | SRC-0049, SRC-0158 | compliance-residency | no | §14 residency |
| Sovereign cloud / air-gapped deployment (per-pack variant; on-prem registry/Bao/audit-shard/no egress) | SRC-0164 | compliance-residency | no | §14 sovereign (FOUNDER: committed FD?) |
| PII registry canonical (cross-cutting data classification) | SRC-0156 | compliance-residency | no | §14 pii |
| Trust framework: cross-µsvc lineage, DSR cascade, Cosign proof-of-erasure, tenant trust portal | SRC-0038 | compliance-residency | no | §14 trust |
| Data-use-boundary segregation (DUBO) | SRC-0008 | compliance-residency | no | §14 dubo |
| OSI-strict license posture; no AGPL/GPL in product code (server-side carve-outs w/ evidence) | SRC-0013, SRC-0211 | compliance-residency | no | §14 license |

## P. structure / repo-topology (cross-cutting invariants)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Flat microservice catalog; no-grouping forward-policy (grouping = presentation tag only) | SRC-0058, SRC-0132, SRC-0362 | governance-ssot | no | §15 catalog |
| Per-µsvc flat colocation layout; canonical monorepo `{oya,cloud}/<service>/` | SRC-0131, SRC-0512 | governance-ssot | no | §15 layout |
| Rust Clean Architecture BNF v4.1 + 13-value canonical layer enum (application→usecase) | SRC-0056, SRC-0105, SRC-0106 | governance-ssot | no | §15 grammar |
| Flat-singular `registry/` + flat `specs/` root (locates masterplan.json) | SRC-0115, SRC-0119 | governance-ssot | no | §15 roots |
| Ontology naming canon (object-graph/knowledge-graph-registry → ontology) | SRC-0055, SRC-0122, SRC-0130 | governance-ssot | no | §15 ontology |
| Wave/plane integration framework (preview/stable/GA; descriptive wave names; no M0–M3) | SRC-0016 | governance-ssot | no | §15 waves |
| Glossary/terminology canon + `oya-check-glossary` CI lane (industry-aligned; KR-EN parity) | SRC-0018 | governance-ssot | no | §15 glossary |
| Plane separation control/data/analytics (catalog-declared plane class) | SRC-0004 | governance-ssot | no | §15 planes |
| Cohesion thesis: one product across flat catalog joined at shared substrates | SRC-0001 | governance-ssot | no | §15 cohesion |
| Hexagonal, no-std, source-compatible structure; typed config + module packaging; modern-only hardware; dependency posture | L-0002, L-0016, L-0013, L-0003 | governance-ssot | no | §15 pilot-structure |
| Audit chain + evidence emission = single tamper-evident record-keeping substrate | SRC-0003 | governance-ssot | no | §15 audit-substrate |
| Ontology typed-entity layer (engine-enforced; per-property classification) | SRC-0006 | data-storage | no | §2 ontology-types |

## Q. product-scope (FOUNDER breadth rulings — atoms exist but scope is contested)

| atom | ADR id(s) | domain | in_mp | masterplan section |
|---|---|---|---|---|
| Vertical catalog (medical/pharmacy/banking/insurance/ads/manufacturing/logistics) as owned first-class microservices vs substrate+ISV | SRC-0058, SRC-0010, SRC-0030, SRC-0031 | ownership-doctrine | no | §16 verticals (FOUNDER scope) |
| Client strategy: five native stacks vs stage-to-1–2 day-0 | SRC-0185, SRC-0051 | api-surface | no | §16 clients (FOUNDER scope) |
| DCIM build-in-house (Phase-2+) + no-custom-silicon (soften) | SRC-0032 | ownership-doctrine | no | §16 dc-ops (FOUNDER scope) |
| Workflow engine: bespoke FSM+DAG day-0 vs adopt-Temporal-and-own-the-overlay | SRC-0035 | intelligence-ai | no | §16 workflow-engine (FOUNDER scope) |
| Industry-best-practice conformance program: one mega-lane vs N per-axis scorecards | SRC-0133 | governance-ssot | no | §16 conformance (FOUNDER) |

---

## SALVAGE ATOMS (must not be lost when their host ADR is ARCHIVED)
- **Banned-direct-git / agents-act-only-through-audited-provider-agnostic-primitives** (from SRC-0053/0103) → fold into forge-scm/governance.
- **Destructive migrations require a committed pre-move classification manifest** (from SRC-0052) → governance-ssot lifecycle.
- **Per-changeset cost budgets (USD/tokens/invocations) + monthly-team-budget + override-frequency alarming** (from SRC-0113) → intelligence-ai agent-execution (PR#605).
- **Projected-merge-state + file-overlap merge algorithm** (from SRC-0111/0124) → forge-scm Tide.
- **Generic deprecation-lane pattern** (from SRC-0138/0143) → governance-ssot lifecycle.
- **Paired-reversible-uninstall doctrine** (from SRC-0120) → re-home under Talos/declarative-immutable-node, or mark moot if immutable nodes obviate it (FOUNDER).
- **Canary-gate-before-prod principle** (from SRC-0114) → ci-cd cd (re-issue against Argo-Rollouts).

## DEDUPE LOG (cross-side atoms collapsed to one)
- ownership ratchet: SRC-0173/0211 ≡ L-0019/0020/0022 → ONE `ownership-doctrine` atom (reconcile vocab).
- policy engine: SRC-0243/0246/0183 ≡ L-0021 → ONE `policy-authz` atom (own-vs-reuse phasing).
- data tier: SRC-0045/0179/0184 ⟂ L-0001 → ONE phased `data-storage` atom (OLTP reused; differentiator owned).
- time/clock: SRC-0252 ≡ L-0006 → ONE atom (swappable Clock port).
- isolation: SRC-0147/0338/0375 ⟂ L-0014/0017/0018/0023 → phased `isolation-runtime` + `orchestration-fleet` atoms (assembled now, owned-gated successor).
- node-OS: SRC-0375 ⟂ L-0025 → ONE atom (adopt Talos; owned Rust successor gated).

## COVERAGE / HONESTY
- Atoms are drawn from per-ADR audits (SRC 0001–0186, all LINUX) + posture/digest verdicts (SRC 0187–0514). `[GAP]` source ADRs (chunks 2/4/9: ~0008–0014/0023–0029/0059–0064) contribute foundation atoms inferred from posture — the verifier lane should confirm none carry an un-captured TRUE atom.
- **in_masterplan is `no` for nearly every atom** (the ~8.8%-binding finding). The only `yes` rows are the masterplan-generation machinery itself (0364/0365); `partial` = referenced in MASTERPLAN.md/specs but not bound. This ledger IS the backfill worklist.
- Under the ratified generated-from-ADRs model, "backfill" = author/clean each atom's owning ADR front-matter (`planning_impact:true`, `domain`, resolved edges) so `oya gen masterplan` emits these sections.
