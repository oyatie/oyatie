---
doc_status: published
id: ADR-0701
title: "Live monorepo capability layout, faces, and reorg doctrine"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0011, ADR-0017, ADR-0026, ADR-0034, ADR-0036, ADR-0058, ADR-0131, ADR-0145, ADR-0159, ADR-0165, ADR-0177, ADR-0188, ADR-0197, ADR-0199, ADR-0201, ADR-0206, ADR-0218, ADR-0220, ADR-0245, ADR-0246, ADR-0255, ADR-0280, ADR-0307, ADR-0331, ADR-0332, ADR-0333, ADR-0335, ADR-0336, ADR-0338, ADR-0343, ADR-0344, ADR-0351, ADR-0363, ADR-0370, ADR-0375, ADR-0376, ADR-0378, ADR-0379, ADR-0476, ADR-0479, ADR-0480, ADR-0482, ADR-0510, ADR-0512, ADR-0520, ADR-0532, ADR-0552, ADR-0561, ADR-0562, ADR-0564, ADR-0571, ADR-0580, ADR-0591, ADR-0598, ADR-0599, ADR-0604, ADR-0614, ADR-0615, ADR-0617, ADR-0620, ADR-0621, ADR-0635]
superseded_by: []
amends: []
amended_by: [ADR-0710, ADR-0719]
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0701-D1
    description: "Live apex source-of-truth for topic monorepo_capability: Live monorepo capability layout, faces, and reorg doctrine."
    exit_criteria: "docs/decisions/ADR-0701-monorepo-capability-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0701: Live monorepo capability layout, faces, and reorg doctrine

## Status

**Accepted** — live consolidated source-of-truth entry for topic `monorepo_capability` (E5 2026-08-06).

**Live amendment (ADR-0719, founder 2026-08-21).** Repo-root and capability/`app/<product>/` **directory shape** is the closed set in [ADR-0719 D-8](ADR-0719-eac-serving-control-north-star.md): cap-root **`cedar/`**; no cap-root `policy/` or `contracts/`; SLO source is IR; **no `specs/` catch-all**; **no `HANDOFF.md`**; public proto/H3; leftover REST/JSON deleted. Pipeline names: [ADR-0719 D-10](ADR-0719-eac-serving-control-north-star.md) **presubmit / postsubmit / nightly / weekly / promotion / release**. Cloud-provider placement: [ADR-0719 D-11](ADR-0719-eac-serving-control-north-star.md). Node OS: [ADR-0719 D-13](ADR-0719-eac-serving-control-north-star.md) — no in-tree `kernel/`/`os/`; upstream Talos; port-engine regenerates when owned. That set **scoped-supersedes** member gists below that require census files (`manifest.json`, `catalog.yaml`, `scorecards/`, `IPs/`, `AUDIT-FINDINGS`, dashboard JSON, `dpia.md` essays), `microservices/` paths, Helm/Tofu/Kyverno as **sources**, both `cedar/` and `policy/` as cap-root children, or root `contracts/microservice-contracts.yaml` as IDL SSOT (ADR-0011 gist). Faces (`core`/`ports`/`adapters`/`facade`), ADR-0562/0615 placement, and shrink-only `reorg_now` of `oya/`/`libs/`/`cloud/`/`infra/`/`tools/`/`toolchains/` still stand. On conflict, ADR-0719 D-8/D-10 win. Hygiene/reorg may go red on live gates that still require the superseded files.

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **62** Accepted ADRs in the `monorepo_capability` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `monorepo_capability` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
   **Layout closed-set and “should not exist” classes:** ADR-0719 D-8.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-11** (ADR-0011-cross-microservice-contract-registry): We adopt **`contracts/microservice-contracts.yaml`** as the source-of-truth registry of cross-microservice contracts, with sub-directories for protocol-specific specs, a gating CI lane, an explicit cross-microservice contract change protocol, and auto-generated multi-language SDKs. ### Registry layout ``` contracts/ microservice-contracts.yaml # so
- **ADR-17** (ADR-0017-brand-naming-and-repo-layout): We adopt **Oyatie** as the product brand, **`oYa`** as the logo abbreviation, **`oyatie.com`** as the domain, **`oya-<microservice>-(<bc>-)?<layer>`** as the Cargo prefix per BNF v4.1 (ADR-0056), and explicitly retain the repo path / GitHub slug **`jason931225/oyatie`**. ### Brand rules | Element | Value | Notes | |---|---|---| | Product name | **O
- **ADR-26** (ADR-0026-in-house-ai-model-substrate-roadmap): We commit to a long-horizon **W-AI-Model-Substrate** wave that produces in-house production model training and inference for Oyatie-specific tasks. We are not a frontier-LLM lab; we consume Anthropic / OpenAI / Gemini until the in-house variant beats the per-vertical eval set per ADR-0024. The `ProviderAdapter` trait extends to `oya-internal-<model
- **ADR-34** (ADR-0034-per-microservice-data-class-overrides): Every microservice crate in the flat catalog that handles regulated data ships a **microservice override pack** in its kernel crate under `oya-<microservice>-override-pack-kernel`. The pack is a structured map from data class to hard-deny policy. It binds at runtime; tenant admin Cedar policies cannot raise the ceiling. **Naming justification (BNF 
- **ADR-36** (ADR-0036-plugin-substrate-wasm-and-trust): We adopt **Wasmtime + WASI Preview 2** as the canonical plugin runtime; **capability-gated `PluginContext`** as the only API surface plugins see; **Cosign keyless signing + Rekor transparency log** as the signing chain; **three trust tiers** (verified-isv / community / experimental); a **per-plugin per-tenant resource cap** model; and a **marketpla
- **ADR-58** (ADR-0058-flat-microservice-catalog): We adopt the **flat microservice catalog** as the canonical architecture. No vertical, arm, product group, or platform grouping exists in code, directories, or architecture. Every feature and product is an independent microservice registered in `[workspace.metadata.oya.microservices]` (per ADR-0056 BNF v4.1). ### Canonical flat catalog (complete as
- **ADR-131** (ADR-0131-per-microservice-flat-layout): oyatie adopts one universal artifact layout: **flat colocated service folders under `{oya,cloud}/<service>/`**, mandatory for every service/product in the repo. `oya/` holds product/domain services; `cloud/` holds platform/tenant-substrate services; shared cross-cutting code remains under `libs/`. Sales segmentation remains a PRD-frontmatter field,
- **ADR-145** (ADR-0145-inter-microservice-communication-reform): **Three weaker invariants replace the universal-mediator rule.** ### Invariant 1 — Audit invariant (decentralized) Every state-changing inter-µservice call MUST emit an audit-chain seal at the **calling** service (NOT at a mediator). Each µservice owns its own audit emission. The audit-chain µservice provides canonical seal storage but does NOT med
- **ADR-159** (ADR-0159-feature-flag-substrate): Oyatie adopts a dedicated `feature-flags` µservice as the canonical runtime feature-flag substrate. Properties: ### Three orthogonal gating tiers 1. **Code-deploy gate** = ChangeSet `acceptance_status` (ADR-0110). Code lives in `dev` / `staging` / `production`. 2. **Traffic-shape gate** = Progressive delivery via Flagger (ADR-0160) with SLO-gated p
- **ADR-165** (ADR-0165-chaos-engineering-substrate): Oyatie adopts **Chaos Mesh 2.x** (CNCF incubating) as the canonical chaos engineering substrate. Every µservice that declares production SLOs ships a per-µservice chaos catalog at `chaos/scenarios/*.yaml`; a nightly job runs each scenario against the µservice's staging environment; SLO breach during a scenario is a release blocker. ### Operational 
- **ADR-177** (ADR-0177-internal-external-api-surface-separation): ### D-1. Two gateway tiers | Tier | Hostname | Audience | Stability tier | Auth | Rate limit | | --- | --- | --- | --- | --- | --- | | **Public** | `api.oyatie.com` | External customers, public-SDK consumers, partners | `Public-Stable` or `Public-Preview` per ADR-0037 | OAuth 2.0 + per-key signature | Per-public-key, per-IP (ADR-0178) | | **Interna
- **ADR-188** (ADR-0188-passkey-webauthn-substrate): **WebAuthn Level 3 is the canonical strong-authentication substrate. Passkeys are the primary credential. The Rust implementation is `webauthn-rs` v0.5+ (kanidm/webauthn-rs).**[^2] TOTP (RFC 6238) is the only sanctioned fallback when Passkey is unavailable. SMS OTP is forbidden. ### Credential ladder | Tier | Credential | Acceptance | Notes | |---|
- **ADR-197** (ADR-0197-backup-substrate-velero-pgbackrest-restic): ### D-1. Three prongs, one per concern - **Velero 1.18.0** — Kubernetes state (manifests, CRDs, ConfigMaps, Secrets, PVCs) + persistent volume content via the integrated filesystem-backup uploader (kopia, the modern replacement for restic in Velero). Velero is the only prong that understands Kubernetes objects. - **pgBackRest 2.58.0** — Postgres po
- **ADR-199** (ADR-0199-per-tenant-cost-attribution-finops-substrate): ### D-1. Canonical tenant label block (CI-enforced) Every Kubernetes workload + cloud resource MUST carry: | Label | Cardinality | Required | Source | |-----------------------------|-------------|----------|---------------------------------| | `oya.io/tenant-id` | per pod / resource | yes | tenancy µservice (ULID) | | `oya.io/cost-center` | per µse
- **ADR-201** (ADR-0201-email-transactional-comms-adapter-substrate): Introduce a substrate-level email-comms adapter pattern owned by the new `microservices/comms-email/` µservice and exposed via the `crates/oya-shared-email-comms-kernel` trait + real adapter set. ### Adapter set (no Noop fallback) - **`SesEmailComms`** — AWS SES (default for cloud-hosted clusters). - **`PostalEmailComms`** — Postal self-hosted (AGP
- **ADR-206** (ADR-0206-i18n-substrate-fluent-icu): ### Authoring source-of-truth: Fluent (Mozilla) Translatable strings author at `clients/i18n/source.ftl` (Fluent grammar). Why Fluent over PO/MO: - **Rust-native** — `fluent-rs` is the canonical Rust impl (maintained by Mozilla + community). - **Expressive** — variants (gender, plural, select) + nested message references + terms (reusable noun phra
- **ADR-218** (ADR-0218-tenant-granular-control-surface): Ship a Tenant Admin Console inside the Application B2B shell. The console is the canonical tenant-facing control plane for: - employees and roles, including SCIM-provisioned users and tenant-extension roles; - products enabled a-la-carte per tenant; - access policies through visual Cedar-fragment builders; - tenant-scoped data classifications and l
- **ADR-220** (ADR-0220-consumer-intelligence-substrate): Create `microservices/intelligence/` as the consumer-facing AI substrate for B2B tenants and B2C personal users. The user-visible brand label is **oyatie intelligence**. Intelligence remains internal only: - retired external agent harness agentic development toolchain; - CI/CD orchestration; - internal eval substrate; - internal evidence collection. Int
- **ADR-245** (ADR-0245-substrate-vs-product-layering): ### D-1. Two-rule doctrine The doctrine is two rules that compose: **Rule 1 — Substrates are audience-neutral and capability-focused.** A substrate µservice provides a capability (storage, policy evaluation, identity issuance, cell management, observability rollup, compute scheduling, network routing, secrets management, audit emission, ontology pr
- **ADR-246** (ADR-0246-policy-engine-substrate-promotion): ### D-1. Promote `cedar-fragment-coverage` BC to peer µservice `microservices/policy-engine/` The `cedar-fragment-coverage` bounded context resident in `microservices/ontology/` is promoted to its own peer µservice `microservices/policy-engine/`. The new µservice is a *substrate* (per ADR-0245 substrate-vs-product layering); it is consumed by every
- **ADR-255** (ADR-0255-intelligence-as-two-layer-ai-substrate): ### D-1. Two-layer model — AI Substrate + Consumer Brand Surface `microservices/intelligence/` is restructured as a single µservice containing two clearly-separated layers expressed as bounded contexts: **Layer A — AI Substrate.** Audience-neutral. Serves every tenant. Eight BCs (per D-2). Deployed per Tier 3 data-plane cell. Provides the universal
- **ADR-280** (ADR-0280-substrate-of-substrate-dependency-doctrine): ### D-1. Canonical substrate dependency DAG declared in `specs/substrate-dependency-dag.json` The substrate-of-substrate dependency Directed Acyclic Graph is declared as a single canonical machine-readable artifact at `/specs/substrate-dependency-dag.json`. The artifact is the **single source of truth** for substrate dependency direction, bootstrap
- **ADR-307** (ADR-0307-detection-substrate-streaming-batch): ### §B. Detection substrate as a single-concern flat µservice Establish `microservices/detection/` as a substrate-tier flat µservice (per ADR-0131 per-microservice flat layout + ADR-0132 no-grouping rule) exposing eight substrate primitives: 1. **Streaming pipeline (Apache Flink-class).** Consumes audit events from Kafka per ADR-0263; per-family ru
- **ADR-331** (Cross-µservice tenant_class Adoption Template): ### B.1 Decision statement Every active µservice in the Oyatie corpus (77 µservices at this ADR's authoring; future µservices on creation) MUST implement the twelve adoption surfaces specified in §D below. Each µservice MUST file a per-µservice IP at `microservices/<name>/IPs/IP-tenant-class-adoption.md` following the skeleton in §D-13. The `ci-ten
- **ADR-332** (Healthcare Domain Decomposition — Eight New Domain Microservices + Integration-S): Enforcement status is `advisory-until-eight-microservices-scaffold-lands`. The doctrine is authoritative for future authoring waves the moment this ADR lands; the BLOCKER promotion happens once the eight new microservice folders exist under `microservices/` with the minimum-viable anchor set (PRD, ARCHITECTURE, manifest, compliance, contracts skele
- **ADR-333** (ADR-0333-cell-microservice-retired-pattern-not-service): D-1. `microservices/cell/` is retired as a standalone µservice. D-2. `microservices/cell/` keeps only a `RETIRED.md` redirect marker. D-3. Historical cell service content is not the live authority after this ADR. D-4. ADR-0248 remains the canonical cellular architecture doctrine. D-5. ADR-0248 is amended only where it names a central cell µservice 
- **ADR-335** (ADR-0335-intelligence-microservice-consolidation): ### D-1..D-12. Service boundary D-1. `microservices/intelligence/` is retired as a standalone µservice. D-2. `microservices/intelligence/` keeps only a `RETIRED.md` redirect marker plus historical-evidence subdirectories explicitly preserved. D-3. Historical intelligence service content is not the live authority after this ADR. D-4. `microservices/intelligence/` 
- **ADR-336** (Valkey is the canonical in-memory KV / cache / pubsub substrate (Redis retired f): ### B.1 Decision statement Valkey (Linux Foundation BSD-3-Clause fork of Redis 7.2.4, current mainline 8.x) is the canonical Oyatie in-memory key-value, cache, pubsub, and streams substrate. Redis 7.4+ (Redis Inc. SSPLv1 / RSALv2 dual-license) is retired from the Oyatie substrate allow-list. Pre-7.4 Redis (BSD-3-Clause) remains license-clean but is
- **ADR-338** (Pod runtime tier 0..3 (Kata + Cloud Hypervisor for tenant-untrusted + tenant-dat): ### B.1 Decision statement Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `pod_runtime_tier` field whose value is an integer in `{0, 1, 2, 3}`. The integer maps to a RuntimeClass (D-4), a nodepool placement contract (D-3), and a Kyverno admission policy (D-5). The CI lane `oya-check-pod-runtime-tier` (D-6) validates **[AMENDED 2026-08-19 — PATH SUPERSEDED, ENFORCEMENT SUPERSEDED.]** Two clauses of this gist are dead as written. (1) The declaration site `microservices/<name>/manifest.json` does not exist: `microservices/` holds zero tracked files, so any check keyed to that path matches nothing and passes vacuously. The field itself is alive and widespread — 78 of 107 tracked `manifest.json` files declare `pod_runtime_tier`, across 21 capability roots — at the ADR-0562/ADR-0615 capability-rooted shape `<capability>/manifest.json`, `<capability>/<service>/manifest.json`, or `app/<product>/manifest.json`. Read the path expression as capability-rooted, not `microservices/`-rooted. (2) The named enforcement vehicle, a Kyverno admission policy, is superseded by ADR-0710: admission is ValidatingAdmissionPolicy + CEL over a projected param resource, and the policy-engine category is ruled out as the default. The tier→RuntimeClass and tier→nodepool mappings themselves are untouched by this amendment. ADR-0714 (Proposed) would additionally retire the 0..3 integer in favour of orthogonal isolation-property / placement / trust axes; until it Accepts, the integer stands.
- **ADR-343** (DR + RTO/RPO matrix per-µservice + per-compliance-pack (effective tenant RTO/RPO): ### B.1 Decision statement Every Oyatie µservice that produces a workload declares a top-level `dr` block in its `microservices/<name>/manifest.json` carrying five required fields: `rto_p99_seconds` (integer ≥ 0), `rpo_p99_seconds` (integer ≥ 0), `multi_region_active_active` (boolean), `backup_substrate` (array of allowlisted substrate identifiers 
- **ADR-344** (Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD): ### B.1 Decision statement Every audit-chain row emitted under ADR-0263 MUST carry five additional envelope fields — `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region` — computed at emission time on the same HLC tick as the audit row itself per ADR-0252. The values are derived by the emitting µservice from its declared `sustain
- **ADR-351** (ADR-0351-cell-rebalancer-and-cell-lifecycle-microservices): ### D-1 — Two new µservices created (amends ADR-0333) Two new µservices are added to the canonical 77 → 78 → 79 µservice count: ``` microservices/cell-rebalancer/ # NEW — D-2 microservices/cell-lifecycle/ # NEW — D-3 ``` Both follow ADR-0131 per-microservice flat layout. Neither is a suite (per ADR-0132). Both are substrate µservices per ADR-0245 (
- **ADR-363** (ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate): ### 1. Change-coordination substrate = plain git + GitHub (interim) PRs + Prow/cloud-ci required contexts Adopt the standard self-hosted substrate for coordination: plain `git`, GitHub PRs, branch protection, required status checks, webhooks, and auto-merge. Current merge/exit authority is Prow-shaped cloud-ci/oya-ci required context plus reviewer 
- **ADR-370** (ADR-0370-local-production-fidelity-substrate-talos-apple-silicon): The local production-fidelity substrate is **multi-node Talos Linux on Parallels Desktop 26**: 1. **Topology:** 3 control-plane (embedded-etcd HA, floating VIP) + 2 workers (Kata-capable), on the Parallels Shared net (10.211.55.0/24). Talos is immutable + API-managed (no SSH) — genuine prod fidelity, not a dev shim. 2. **Hypervisor = Parallels 26**
- **ADR-375** (ADR-0375-talos-capi-argocd-fleet-substrate): - **Node OS:** Talos (immutable, API-managed). Bare-metal nodes auto-install zero-touch from a **USB** image (config baked for the control plane; fetched for spoke nodes). Cloud nodes use CAPI cloud images. - **Cluster lifecycle:** **Cluster API** — declarative `Cluster`/`MachineDeployment` CRs in git, reconciled by controllers. The management/cont
- **ADR-376** (ADR-0376-managed-kubernetes-product-surface): Oyatie's managed-Kubernetes offering is a **TWO-TIER product** on top of the ADR-0375 substrate. The tenant picks the tier; **the default is hosted**. - **Hosted control plane = DEFAULT tier.** Tenant Kubernetes control planes run as pods inside Oyatie's management (control-plane) cluster via **Kamaji** (`github.com/clastix/kamaji`), a hosted-contr
- **ADR-378** (ADR-0378-canonical-local-substrate-vfkit-talos): **vfkit + Talos is the single canonical local substrate; colima is retired.** The distinction is the guest, not the hypervisor (both use Apple VZ): Talos is the production Kubernetes OS; colima is a laptop k3s/docker box. 1. Talos node (vfkit VM, 192.168.64.3) managed directly via `talosctl`; canonical config home `~/.oya/talos-local/`. Kube contex
- **ADR-379** (ADR-0379-kubewarden-default-admission-substrate): 1. **Kubewarden is the default Kubernetes admission/policy substrate.** Admission policies (image signature verification, PSS restricted-by-default, label/annotation discipline, runtime-tier enforcement) are authored as **WASM policy modules** — written in Rust and compiled to WASM where practical — and enforced by the Kubewarden policy server. Thi **[AMENDED 2026-08-18 by ADR-0710 — SUPERSEDED IN PART.]** The Kubewarden-as-default clause is overturned: the admission substrate is the API server itself (ValidatingAdmissionPolicy + CEL, with Pod Security Admission for the pod-security baseline), and the base platform overlay ships no policy webhook. Kyverno, Kubewarden and Gatekeeper/OPA remain available as adapters but none is the default. The Kubewarden Applications and the enforcing Kyverno ClusterPolicy remain deployed until ADR-0710 D-1's PSA presence-half census closes; see ADR-0710 for the removal precondition.
- **ADR-476** (oya-identity: bespoke Rust human identity substrate): Build **oya-identity** — a bespoke, Rust-native human identity substrate — under `microservices/oya-identity/`. Keycloak (ADR-0421) is the Phase-1 bridge; oya-identity is the canonical long-term target. The planning and tracking bridge for the ADR-0476 identity surface with the ADR-0506, ADR-0507, and ADR-0508 provider bridges is `oya/identity/IP-0
- **ADR-479** (oya-meter — bespoke Rust usage metering substrate): Ship `microservices/oya-meter/` as a single-concern Rust µservice (ADR-0131 flat layout, ADR-0132 no-suite). OpenMeter is retired as an active dependency (ADR-0429 → Superseded). ### D1 — µservice scaffold `microservices/oya-meter/` — Rust workspace, Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2. **ClickHouse** (ADR-0193) is the
- **ADR-480** (oya-cost: bespoke Rust K8s cost allocation substrate): Ship `microservices/oya-cost/` as a bespoke Rust µservice (Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2). ClickHouse for time-series cost data; PostgreSQL for catalog. Cedar (ADR-0083) gates per-tenant cost-API access. OpenCost is retired.
- **ADR-482** (ADR-0482-bespoke-substrate-roadmap): Adopt a phased bespoke roadmap with explicit timeline tiers and bridge mappings. Unless a later Accepted ADR explicitly narrows a component's bridge posture, each bespoke component ships with a parallel OSS bridge, quality-gated cutover criteria, and tenant opt-in granularity. No hard-deadline cutover — quality gates only. ADR-0394 is the explicit 
- **ADR-510** (SCM destination = bespoke hyperscaler monorepo-VCS; GitHub transitory; cutover n): ### 1. The SCM destination is the bespoke hyperscaler monorepo-VCS — DECIDED The long-horizon SCM destination is a bespoke, self-hostable, Rust, hyperscaler-grade monorepo-VCS server in the Piper/Sapling/Mononoke class. This is a **decided destination, not an open "whether."** What remains open is only the **timing** of the cutover (§3). ### 2. Git
- **ADR-512** (ADR-0512-canonical-monorepo-pattern): Adopt the canonical pattern **"vertical-slice monorepo · one workspace · bounded-context crates · dependency-rule modules · Buck2 graph"**: 1. **Layout.** Service code lives at `{oya,cloud}/<service>/crates/<crate>/`; product-facing/domain services live under `oya/`, platform/tenant substrate services under `cloud/`, and shared cross-cutting librar
- **ADR-520** (Owned substrate stack (fabric->DB->object-store->k8s->OS->kernel): transitional-): Ratify the kernel-to-fabric owned-substrate stack as the fabric's foundation: Agentic Delivery Fabric → bespoke distributed-SQL DB (metadata) + bespoke infinite-scale object-store (content) → k8s + containerd → Talos-style OS → kuberos-kernel. EVERY layer is: **owned · cloud-native · infinite-scale · productized · transitional-impl-behind-a-stable- **[AMENDED 2026-08-19 — LADDER FLOOR RETIRED.]** The rung-0 floor named in this gist, the kuberos framekernel, no longer exists and is not returning. Founder decision of 2026-08-02 was to keep Asterinas and delete kuberos/cloud-kernel; commit `c2ee2631a` (2026-08-14) executed it, removing 229 files of which 170 were under `cloud/cloud-kernel`. `cloud/` now holds zero tracked files. Read the ladder without a kuberos rung: the node substrate is Linux via upstream Talos today, `kernel/` holds the Asterinas evaluation as a black-box upstream pin rather than an owned kernel, and the Asterinas-versus-Linux selection is deferred behind the `os/ports/kernel-abi` seam. `kernel/` and `os/` remain registered meta directories at rungs 0 and 1; it is their hand-written CONTENTS that are superseded, by port-engine regeneration rather than by deletion. ADR-0712 and ADR-0713 (both Proposed) carry the forward shape and are not law.
- **ADR-532** (Platform product-line taxonomy + canonical product names (the seven lifecycle-to): Adopt the seven canonical lifecycle-tooling PRODUCTS as the product line, each a GENERIC ENGINE whose behavior is pure DATA in a repo-root config, with three independently-versioned composables (ENGINE binary/crate + POLICY config + RUNNER wrapper): - **P1 oya-build** — hermetic buck2 graph + reindeer vendoring + toolchains. - **P2 oya-ci-floor** —
- **ADR-552** (Stable/volatile SCM-facts split: history-derived facts leave the merged surface): 1. **Stable/volatile split as specified in option 3.** Schema bump `oya-ci/scm-facts/v1` → `v2` (this ADR amends the ADR-0526 face shape; the `ScmFactsSource` trait — the VCS-agnostic seam — is UNCHANGED: same three primitives, so a future bespoke SCM source is unaffected). 2. **Single-owner, canonical (#696 precedent).** No second comparison or se
- **ADR-561** (Commission the workload-identity X.509-SVID issuance + PDP caller-tenant-binding): Commission three additive units (clean-arch kernel/adapter/app split, mirroring the ADR-0559 PDP split), plus the X.509 work in the trustd domain to carry a SPIFFE identity: 1. **`oya-identity-workload-svid-kernel`** (PURE, no-IO): the cell-rooted `SpiffeId` + `WorkloadPath` value types; the `WorkloadIdentityIssuer` / `SvidVerifier` / `TrustBundleS
- **ADR-562** (Capability-first repo organization + the closed capability registry — the ratifi): ### §0 Path-anchor reading rule (how to read the backticked paths in this ADR) This ADR is simultaneously a **destination spec** and an **executed-move log**, so a majority of its backticked paths do not resolve against the live tree **by construction**. A dead anchor here is therefore not evidence of staleness. Every backticked path in this docume
- **ADR-564** (Commission the tenancy tenant-lifecycle registration service (G006 slice 1): a r): Commission the tenancy capability's tenant-lifecycle service as G006 slice 1: a **runnable tenant registration / lifecycle delivery surface** in the ADR-0550 adapter/app shape, reusing the locked lifecycle usecase + contract FSM wholesale (zero forked transition logic). ### D1 — Service shape (ADR-0131 / ADR-0550 seams) - `tenancy/adapters/tenant-l
- **ADR-571** (Home the connect address-book domain into the comms capability and commission th): ### D1 — MOVE: home the address-book domain into `comms/core` Relocate `oya/connect/crates/oya-address-book-domain` → `comms/core/connect-address-book-domain` via the reorg codemod (history-preserving `git mv` + package/lib-name de-brand per ADR-0532/0533: cargo `connect-address-book-domain` == path-tail, the `connect-` discriminator retained to na
- **ADR-580** (corpus substrate Phase -1: the conservative-v1 syn-over-source AST extractor spi): ### D1 — Conservative-v1 posture: `syn`-over-source behind a stable `AstSource` seam The v1 extractor parses committed Rust source with `syn` 2.x. The fact producer is hidden behind a stable `AstSource` trait so a W-tier successor (a bespoke rowan-style CST / a semantic ra adapter) can replace the producer LATER without disturbing the fact model or
- **ADR-591** (Fail-closed authz for the Cloud FinOps report API (AUTH-005 capability-billing r): Make the Cloud FinOps report surface fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new sour
- **ADR-598** (Commission the comms meet capability-first core slice (comms-meet-api port + com): ### D1 — BUILD the cloud-agnostic core slice (clean-arch, owned-stack shape) Add the port `comms/ports/meet-api` (`comms-meet-api`) and the usecase `comms/core/meet-usecase` (`comms-meet-usecase`): - The port defines the seam concrete adapters implement LATER: `MeetSessionStore` (a tenant-scoped session-persistence repository trait) plus the typed 
- **ADR-599** (Commission the comms calendar capability-first move + cloud-agnostic core slice ): ### D1 — MOVE the domain into its capability home (de-branded, glob-covered) Relocate `oya/calendar/crates/oya-calendar-domain` to `comms/core/calendar-domain` via the deterministic reorg codemod (ADR-0563), de-branding the cargo name `oya-calendar-domain` → `comms-calendar-domain` (drop the vendor prefix; path-tail == cargo name; face dir not in t
- **ADR-604** (De-commit the scm-facts boundary snapshot — the last committed pure-derivation f): STOP committing `scm-facts.generated.json`. It is declared `materialization_mode: not-tracked-in-git` (was `main-branch-materialized`) with `merge_policy: never-manual-merge-regenerate-from-source-tree` in `registry/generated-artifact-control-plane.json`, removed from git (`git rm --cached`), and covered by the existing `**/*.generated.json` ignore
- **ADR-614** (De-commit the reorg move-manifest bijection (finish the pure-derivation strangle): STOP committing `specs/reorg/move-manifest.generated.json`. Declare it `materialization_mode: not-tracked-in-git` in `registry/generated-artifact-control-plane.json`, remove it from git (`git rm --cached`), and cover it by `.gitignore` (the existing broad `**/*.generated.json` rule; the ADR-0563 negation `!specs/reorg/move-manifest.generated.json` 
- **ADR-615** (Capability boundary rulings — resolving ADR-0562's flagged_boundaries (the subst): ### §1 The governing rule — the substrate/product split The disposition of every flagged surface is decided by **ADR-0562 §3 rule #5 and §6**, applied verbatim: > A deployable surface composing **2+ capabilities** for a tenant → `app/<product>/`. > A **single-capability** sold surface **is a `facade/`** of that capability (§6: "a single-capability 
- **ADR-617** (The Living Monorepo Governance Graph — monorepo management + project lifecycle a): ### §1 The thesis — the monorepo IS one live governed graph The monorepo and its lifecycle are **one live, federated (per-cell), content-addressed graph** — `governance/corpus/` extended into the single management + lifecycle substrate for the whole repo. This is **not a new system**: it is the productized development pipeline (the fabric) with the
- **ADR-620** (Pre-admission inventory provenance for history-only retirement observation surfa): Record non-authoritative path-to-ADR inventory references for exactly these candidate surfaces: - `ci/facade/scm-facts-snapshot/src/lib.rs` - `ci/facade/scm-facts-snapshot/src/retirement.rs` - `ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs` - `registry/history-only-retirement/OWNERS` - `registry/history-only-retirement/control-plane.js
- **ADR-621** (De-commit the active-artifact-contract graph projection): **Proposed — 2026-07-24.** Council ratification and acceptance are **not established**: this two-way candidate awaits qualified-authority review. Protected admission of the implementation proves only the admitted mechanics and gate behavior; it neither changes this record's lifecycle nor supplies the missing authority. It is not a legal, operationa
- **ADR-635** (Face-aware substrate dependency graph v2: five typed graphs and derived failure ): ### D1 — `dependency_units` are `runtime_face`-qualified `specs/substrate-dependency-dag.json` v2 declares exactly 19 unique, closed `dependency_units` in the bounded W0-C topology slice. A unit id combines a canonical capability and its founder-locked `runtime_face`, for example `cell.envelope`, `cell.lifecycle.cp`, `iam.local-verifier`, or `polic

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-197 residual

**ADR-0197-backup-substrate-velero-pgbackrest-restic** — ### D-1. Three prongs, one per concern - **Velero 1.18.0** — Kubernetes state (manifests, CRDs, ConfigMaps, Secrets, PVCs) + persistent volume content via the integrated filesystem-backup uploader (kopia, the modern replacement for restic in Velero). Velero is the only prong that understands Kubernetes objects. - **pgBackRest 2.58.0** — Postgres point-in-time recovery (PITR) via WAL archive + full

### ADR-220 residual

**ADR-0220-consumer-intelligence-substrate** — Create `microservices/intelligence/` as the consumer-facing AI substrate for B2B tenants and B2C personal users. The user-visible brand label is **oyatie intelligence**. Intelligence remains internal only: - retired external agent harness agentic development toolchain; - CI/CD orchestration; - internal eval substrate; - internal evidence collection. Intelligence owns consumer AI: - per-tenant AI contex

### ADR-363 residual

**ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate** — ### 1. Change-coordination substrate = plain git + GitHub (interim) PRs + Prow/cloud-ci required contexts Adopt the standard self-hosted substrate for coordination: plain `git`, GitHub PRs, branch protection, required status checks, webhooks, and auto-merge. Current merge/exit authority is Prow-shaped cloud-ci/oya-ci required context plus reviewer approval; the legacy CI bridge may post bridge sta

### ADR-378 residual

**ADR-0378-canonical-local-substrate-vfkit-talos** — **vfkit + Talos is the single canonical local substrate; colima is retired.** The distinction is the guest, not the hypervisor (both use Apple VZ): Talos is the production Kubernetes OS; colima is a laptop k3s/docker box. 1. Talos node (vfkit VM, 192.168.64.3) managed directly via `talosctl`; canonical config home `~/.oya/talos-local/`. Kube context `admin@oya-local` is the default. 2. Bring-up is

### ADR-552 residual

**Stable/volatile SCM-facts split: history-derived facts leave the merged surface** — 1. **Stable/volatile split as specified in option 3.** Schema bump `oya-ci/scm-facts/v1` → `v2` (this ADR amends the ADR-0526 face shape; the `ScmFactsSource` trait — the VCS-agnostic seam — is UNCHANGED: same three primitives, so a future bespoke SCM source is unaffected). 2. **Single-owner, canonical (#696 precedent).** No second comparison or settle implementation: the freshness gate's own chec

### ADR-480 residual

**oya-cost: bespoke Rust K8s cost allocation substrate** — Ship `microservices/oya-cost/` as a bespoke Rust µservice (Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2). ClickHouse for time-series cost data; PostgreSQL for catalog. Cedar (ADR-0083) gates per-tenant cost-API access. OpenCost is retired.

### ADR-11 residual

**ADR-0011-cross-microservice-contract-registry** — We adopt **`contracts/microservice-contracts.yaml`** as the source-of-truth registry of cross-microservice contracts, with sub-directories for protocol-specific specs, a gating CI lane, an explicit cross-microservice contract change protocol, and auto-generated multi-language SDKs. ### Registry layout ``` contracts/ microservice-contracts.yaml # source of truth: every cross-microservice contract r

### ADR-562 residual

**Capability-first repo organization + the closed capability registry — the ratified hyperscaler source-tree shape every r** — ### §0 Path-anchor reading rule (how to read the backticked paths in this ADR) This ADR is simultaneously a **destination spec** and an **executed-move log**, so a majority of its backticked paths do not resolve against the live tree **by construction**. A dead anchor here is therefore not evidence of staleness. Every backticked path in this document is exactly one of: | Class | Meaning | Resolves

### ADR-255 residual

**ADR-0255-intelligence-as-two-layer-ai-substrate** — ### D-1. Two-layer model — AI Substrate + Consumer Brand Surface `microservices/intelligence/` is restructured as a single µservice containing two clearly-separated layers expressed as bounded contexts: **Layer A — AI Substrate.** Audience-neutral. Serves every tenant. Eight BCs (per D-2). Deployed per Tier 3 data-plane cell. Provides the universal dispatch + credential-resolution + guardrails + a

### ADR-520 residual

**Owned substrate stack (fabric->DB->object-store->k8s->OS->kernel): transitional-impl-behind-a-stable-interface, none blo** — Ratify the kernel-to-fabric owned-substrate stack as the fabric's foundation: Agentic Delivery Fabric → bespoke distributed-SQL DB (metadata) + bespoke infinite-scale object-store (content) → k8s + containerd → Talos-style OS → kuberos-kernel. EVERY layer is: **owned · cloud-native · infinite-scale · productized · transitional-impl-behind-a-stable-interface** → owned-bespoke on its own timeline, w **[AMENDED 2026-08-19 — LADDER FLOOR RETIRED.]** The rung-0 floor named in this gist, the kuberos framekernel, no longer exists and is not returning. Founder decision of 2026-08-02 was to keep Asterinas and delete kuberos/cloud-kernel; commit `c2ee2631a` (2026-08-14) executed it, removing 229 files of which 170 were under `cloud/cloud-kernel`. `cloud/` now holds zero tracked files. Read the ladder without a kuberos rung: the node substrate is Linux via upstream Talos today, `kernel/` holds the Asterinas evaluation as a black-box upstream pin rather than an owned kernel, and the Asterinas-versus-Linux selection is deferred behind the `os/ports/kernel-abi` seam. `kernel/` and `os/` remain registered meta directories at rungs 0 and 1; it is their hand-written CONTENTS that are superseded, by port-engine regeneration rather than by deletion. ADR-0712 and ADR-0713 (both Proposed) carry the forward shape and are not law.

### ADR-307 residual

**ADR-0307-detection-substrate-streaming-batch** — ### §B. Detection substrate as a single-concern flat µservice Establish `microservices/detection/` as a substrate-tier flat µservice (per ADR-0131 per-microservice flat layout + ADR-0132 no-grouping rule) exposing eight substrate primitives: 1. **Streaming pipeline (Apache Flink-class).** Consumes audit events from Kafka per ADR-0263; per-family rules + ML models score in-flight; signals emitted t

### ADR-580 residual

**corpus substrate Phase -1: the conservative-v1 syn-over-source AST extractor spike** — ### D1 — Conservative-v1 posture: `syn`-over-source behind a stable `AstSource` seam The v1 extractor parses committed Rust source with `syn` 2.x. The fact producer is hidden behind a stable `AstSource` trait so a W-tier successor (a bespoke rowan-style CST / a semantic ra adapter) can replace the producer LATER without disturbing the fact model or any consumer. `salsa`/`rowan` stay ABSENT for v1

### ADR-218 residual

**ADR-0218-tenant-granular-control-surface** — Ship a Tenant Admin Console inside the Application B2B shell. The console is the canonical tenant-facing control plane for: - employees and roles, including SCIM-provisioned users and tenant-extension roles; - products enabled a-la-carte per tenant; - access policies through visual Cedar-fragment builders; - tenant-scoped data classifications and labels; - approval workflows through Workflow Studi

### ADR-482 residual

**ADR-0482-bespoke-substrate-roadmap** — Adopt a phased bespoke roadmap with explicit timeline tiers and bridge mappings. Unless a later Accepted ADR explicitly narrows a component's bridge posture, each bespoke component ships with a parallel OSS bridge, quality-gated cutover criteria, and tenant opt-in granularity. No hard-deadline cutover — quality gates only. ADR-0394 is the explicit portal amendment: Backstage is a bounded one-way i

### ADR-188 residual

**ADR-0188-passkey-webauthn-substrate** — **WebAuthn Level 3 is the canonical strong-authentication substrate. Passkeys are the primary credential. The Rust implementation is `webauthn-rs` v0.5+ (kanidm/webauthn-rs).**[^2] TOTP (RFC 6238) is the only sanctioned fallback when Passkey is unavailable. SMS OTP is forbidden. ### Credential ladder | Tier | Credential | Acceptance | Notes | |---|---|---|---| | 1 (preferred) | Passkey (synced via

### ADR-615 residual

**Capability boundary rulings — resolving ADR-0562's flagged_boundaries (the substrate/product split + the 14 app-vs-capab** — ### §1 The governing rule — the substrate/product split The disposition of every flagged surface is decided by **ADR-0562 §3 rule #5 and §6**, applied verbatim: > A deployable surface composing **2+ capabilities** for a tenant → `app/<product>/`. > A **single-capability** sold surface **is a `facade/`** of that capability (§6: "a single-capability > app is a *mis-placed facade*"; a capability faca

### ADR-561 residual

**Commission the workload-identity X.509-SVID issuance + PDP caller-tenant-binding substrate (G002 slice 1; live mTLS = sl** — Commission three additive units (clean-arch kernel/adapter/app split, mirroring the ADR-0559 PDP split), plus the X.509 work in the trustd domain to carry a SPIFFE identity: 1. **`oya-identity-workload-svid-kernel`** (PURE, no-IO): the cell-rooted `SpiffeId` + `WorkloadPath` value types; the `WorkloadIdentityIssuer` / `SvidVerifier` / `TrustBundleSource` ports; and the fail-closed `bind_caller_ten

### ADR-145 residual

**ADR-0145-inter-microservice-communication-reform** — **Three weaker invariants replace the universal-mediator rule.** ### Invariant 1 — Audit invariant (decentralized) Every state-changing inter-µservice call MUST emit an audit-chain seal at the **calling** service (NOT at a mediator). Each µservice owns its own audit emission. The audit-chain µservice provides canonical seal storage but does NOT mediate the call itself. Enforcement: per-µservice `o

### ADR-476 residual

**oya-identity: bespoke Rust human identity substrate** — Build **oya-identity** — a bespoke, Rust-native human identity substrate — under `microservices/oya-identity/`. Keycloak (ADR-0421) is the Phase-1 bridge; oya-identity is the canonical long-term target. The planning and tracking bridge for the ADR-0476 identity surface with the ADR-0506, ADR-0507, and ADR-0508 provider bridges is `oya/identity/IP-017-bespoke-identity-authn-crypto-bridge.md`; its o

### ADR-564 residual

**Commission the tenancy tenant-lifecycle registration service (G006 slice 1): a runnable tenant register/provision/read d** — Commission the tenancy capability's tenant-lifecycle service as G006 slice 1: a **runnable tenant registration / lifecycle delivery surface** in the ADR-0550 adapter/app shape, reusing the locked lifecycle usecase + contract FSM wholesale (zero forked transition logic). ### D1 — Service shape (ADR-0131 / ADR-0550 seams) - `tenancy/adapters/tenant-lifecycle-store-inmemory` — a faithful in-memory `T

### ADR-532 residual

**Platform product-line taxonomy + canonical product names (the seven lifecycle-tooling products + the de-oyatie rename se** — Adopt the seven canonical lifecycle-tooling PRODUCTS as the product line, each a GENERIC ENGINE whose behavior is pure DATA in a repo-root config, with three independently-versioned composables (ENGINE binary/crate + POLICY config + RUNNER wrapper): - **P1 oya-build** — hermetic buck2 graph + reindeer vendoring + toolchains. - **P2 oya-ci-floor** — producer → faces → shrink-only ratchet → registry

### ADR-206 residual

**ADR-0206-i18n-substrate-fluent-icu** — ### Authoring source-of-truth: Fluent (Mozilla) Translatable strings author at `clients/i18n/source.ftl` (Fluent grammar). Why Fluent over PO/MO: - **Rust-native** — `fluent-rs` is the canonical Rust impl (maintained by Mozilla + community). - **Expressive** — variants (gender, plural, select) + nested message references + terms (reusable noun phrases). - **One source, many targets** — adapters co

### ADR-26 residual

**ADR-0026-in-house-ai-model-substrate-roadmap** — We commit to a long-horizon **W-AI-Model-Substrate** wave that produces in-house production model training and inference for Oyatie-specific tasks. We are not a frontier-LLM lab; we consume Anthropic / OpenAI / Gemini until the in-house variant beats the per-vertical eval set per ADR-0024. The `ProviderAdapter` trait extends to `oya-internal-<model-id>` so the cutover is one router preference chan

### ADR-201 residual

**ADR-0201-email-transactional-comms-adapter-substrate** — Introduce a substrate-level email-comms adapter pattern owned by the new `microservices/comms-email/` µservice and exposed via the `crates/oya-shared-email-comms-kernel` trait + real adapter set. ### Adapter set (no Noop fallback) - **`SesEmailComms`** — AWS SES (default for cloud-hosted clusters). - **`PostalEmailComms`** — Postal self-hosted (AGPL), Ruby on Rails + RabbitMQ + MariaDB. Default fo

### ADR-614 residual

**De-commit the reorg move-manifest bijection (finish the pure-derivation strangler for the last committed reorg face)** — STOP committing `specs/reorg/move-manifest.generated.json`. Declare it `materialization_mode: not-tracked-in-git` in `registry/generated-artifact-control-plane.json`, remove it from git (`git rm --cached`), and cover it by `.gitignore` (the existing broad `**/*.generated.json` rule; the ADR-0563 negation `!specs/reorg/move-manifest.generated.json` and its "must be TRACKED" comment are removed). It

### ADR-335 residual

**ADR-0335-intelligence-microservice-consolidation** — ### D-1..D-12. Service boundary D-1. `microservices/intelligence/` is retired as a standalone µservice. D-2. `microservices/intelligence/` keeps only a `RETIRED.md` redirect marker plus historical-evidence subdirectories explicitly preserved. D-3. Historical intelligence service content is not the live authority after this ADR. D-4. `microservices/intelligence/` is the canonical AI substrate µservice. D-5. `micr

### ADR-620 residual

**Pre-admission inventory provenance for history-only retirement observation surfaces** — Record non-authoritative path-to-ADR inventory references for exactly these candidate surfaces: - `ci/facade/scm-facts-snapshot/src/lib.rs` - `ci/facade/scm-facts-snapshot/src/retirement.rs` - `ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs` - `registry/history-only-retirement/OWNERS` - `registry/history-only-retirement/control-plane.json` - `specs/history-only-retirement-control-plane

### ADR-331 residual

**Cross-µservice tenant_class Adoption Template** — ### B.1 Decision statement Every active µservice in the Oyatie corpus (77 µservices at this ADR's authoring; future µservices on creation) MUST implement the twelve adoption surfaces specified in §D below. Each µservice MUST file a per-µservice IP at `microservices/<name>/IPs/IP-tenant-class-adoption.md` following the skeleton in §D-13. The `ci-tenant-class-adoption-check` CI lane (specified in §E

### ADR-36 residual

**ADR-0036-plugin-substrate-wasm-and-trust** — We adopt **Wasmtime + WASI Preview 2** as the canonical plugin runtime; **capability-gated `PluginContext`** as the only API surface plugins see; **Cosign keyless signing + Rekor transparency log** as the signing chain; **three trust tiers** (verified-isv / community / experimental); a **per-plugin per-tenant resource cap** model; and a **marketplace economics** spec (revenue share + payout cadenc

### ADR-332 residual

**Healthcare Domain Decomposition — Eight New Domain Microservices + Integration-Substrate Narrowing** — Enforcement status is `advisory-until-eight-microservices-scaffold-lands`. The doctrine is authoritative for future authoring waves the moment this ADR lands; the BLOCKER promotion happens once the eight new microservice folders exist under `microservices/` with the minimum-viable anchor set (PRD, ARCHITECTURE, manifest, compliance, contracts skeleton, SLO skeleton, Cedar skeleton).

### ADR-17 residual

**ADR-0017-brand-naming-and-repo-layout** — We adopt **Oyatie** as the product brand, **`oYa`** as the logo abbreviation, **`oyatie.com`** as the domain, **`oya-<microservice>-(<bc>-)?<layer>`** as the Cargo prefix per BNF v4.1 (ADR-0056), and explicitly retain the repo path / GitHub slug **`jason931225/oyatie`**. ### Brand rules | Element | Value | Notes | |---|---|---| | Product name | **Oyatie** | Title case; never `oyatie` in prose | |

### ADR-343 residual

**DR + RTO/RPO matrix per-µservice + per-compliance-pack (effective tenant RTO/RPO = max(µservice declared, all-applicable** — ### B.1 Decision statement Every Oyatie µservice that produces a workload declares a top-level `dr` block in its `microservices/<name>/manifest.json` carrying five required fields: `rto_p99_seconds` (integer ≥ 0), `rpo_p99_seconds` (integer ≥ 0), `multi_region_active_active` (boolean), `backup_substrate` (array of allowlisted substrate identifiers per §D-5), and `failover_runbook` (string path to

### ADR-604 residual

**De-commit the scm-facts boundary snapshot — the last committed pure-derivation face (completes ADR-0595)** — STOP committing `scm-facts.generated.json`. It is declared `materialization_mode: not-tracked-in-git` (was `main-branch-materialized`) with `merge_policy: never-manual-merge-regenerate-from-source-tree` in `registry/generated-artifact-control-plane.json`, removed from git (`git rm --cached`), and covered by the existing `**/*.generated.json` ignore. It is derived on demand via `buck2 run //ci/faca

### ADR-199 residual

**ADR-0199-per-tenant-cost-attribution-finops-substrate** — ### D-1. Canonical tenant label block (CI-enforced) Every Kubernetes workload + cloud resource MUST carry: | Label | Cardinality | Required | Source | |-----------------------------|-------------|----------|---------------------------------| | `oya.io/tenant-id` | per pod / resource | yes | tenancy µservice (ULID) | | `oya.io/cost-center` | per µservice | yes | µservice manifest | | `oya.io/worklo

### ADR-280 residual

**ADR-0280-substrate-of-substrate-dependency-doctrine** — ### D-1. Canonical substrate dependency DAG declared in `specs/substrate-dependency-dag.json` The substrate-of-substrate dependency Directed Acyclic Graph is declared as a single canonical machine-readable artifact at `/specs/substrate-dependency-dag.json`. The artifact is the **single source of truth** for substrate dependency direction, bootstrap ordering, failure-cascade rules, SLO composition,

### ADR-333 residual

**ADR-0333-cell-microservice-retired-pattern-not-service** — D-1. `microservices/cell/` is retired as a standalone µservice. D-2. `microservices/cell/` keeps only a `RETIRED.md` redirect marker. D-3. Historical cell service content is not the live authority after this ADR. D-4. ADR-0248 remains the canonical cellular architecture doctrine. D-5. ADR-0248 is amended only where it names a central cell µservice as the enforcement substrate. D-6. Cellular topolo

### ADR-617 residual

**The Living Monorepo Governance Graph — monorepo management + project lifecycle as one governed, federated, content-addre** — ### §1 The thesis — the monorepo IS one live governed graph The monorepo and its lifecycle are **one live, federated (per-cell), content-addressed graph** — `governance/corpus/` extended into the single management + lifecycle substrate for the whole repo. This is **not a new system**: it is the productized development pipeline (the fabric) with the corpus as its substrate; docs-as-code, git-hygien

### ADR-591 residual

**Fail-closed authz for the Cloud FinOps report API (AUTH-005 capability-billing remediation)** — Make the Cloud FinOps report surface fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new source file `billing/ports/finops-api/src/authz.rs` de

### ADR-344 residual

**Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD-cost emitted alongside every audit row;** — ### B.1 Decision statement Every audit-chain row emitted under ADR-0263 MUST carry five additional envelope fields — `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region` — computed at emission time on the same HLC tick as the audit row itself per ADR-0252. The values are derived by the emitting µservice from its declared `sustainability_emission_model` manifest block, which maps

### ADR-351 residual

**ADR-0351-cell-rebalancer-and-cell-lifecycle-microservices** — ### D-1 — Two new µservices created (amends ADR-0333) Two new µservices are added to the canonical 77 → 78 → 79 µservice count: ``` microservices/cell-rebalancer/ # NEW — D-2 microservices/cell-lifecycle/ # NEW — D-3 ``` Both follow ADR-0131 per-microservice flat layout. Neither is a suite (per ADR-0132). Both are substrate µservices per ADR-0245 (serve every product surface; not product-specific)

### ADR-479 residual

**oya-meter — bespoke Rust usage metering substrate** — Ship `microservices/oya-meter/` as a single-concern Rust µservice (ADR-0131 flat layout, ADR-0132 no-suite). OpenMeter is retired as an active dependency (ADR-0429 → Superseded). ### D1 — µservice scaffold `microservices/oya-meter/` — Rust workspace, Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2. **ClickHouse** (ADR-0193) is the time-series usage backend; **PostgreSQL** is the

### ADR-165 residual

**ADR-0165-chaos-engineering-substrate** — Oyatie adopts **Chaos Mesh 2.x** (CNCF incubating) as the canonical chaos engineering substrate. Every µservice that declares production SLOs ships a per-µservice chaos catalog at `chaos/scenarios/*.yaml`; a nightly job runs each scenario against the µservice's staging environment; SLO breach during a scenario is a release blocker. ### Operational shape 1. **Chaos Mesh control plane per cell.** Cl

### ADR-598 residual

**Commission the comms meet capability-first core slice (comms-meet-api port + comms-meet-usecase)** — ### D1 — BUILD the cloud-agnostic core slice (clean-arch, owned-stack shape) Add the port `comms/ports/meet-api` (`comms-meet-api`) and the usecase `comms/core/meet-usecase` (`comms-meet-usecase`): - The port defines the seam concrete adapters implement LATER: `MeetSessionStore` (a tenant-scoped session-persistence repository trait) plus the typed lifecycle commands (`OpenRoomRequest`, `JoinSessio

### ADR-379 residual

**ADR-0379-kubewarden-default-admission-substrate** — 1. **Kubewarden is the default Kubernetes admission/policy substrate.** Admission policies (image signature verification, PSS restricted-by-default, label/annotation discipline, runtime-tier enforcement) are authored as **WASM policy modules** — written in Rust and compiled to WASM where practical — and enforced by the Kubewarden policy server. This aligns admission policy with the WASM-native ser **[AMENDED 2026-08-18 by ADR-0710 — SUPERSEDED IN PART.]** The Kubewarden-as-default clause is overturned: the admission substrate is the API server itself (ValidatingAdmissionPolicy + CEL, with Pod Security Admission for the pod-security baseline), and the base platform overlay ships no policy webhook. Kyverno, Kubewarden and Gatekeeper/OPA remain available as adapters but none is the default. The Kubewarden Applications and the enforcing Kyverno ClusterPolicy remain deployed until ADR-0710 D-1's PSA presence-half census closes; see ADR-0710 for the removal precondition.

### ADR-245 residual

**ADR-0245-substrate-vs-product-layering** — ### D-1. Two-rule doctrine The doctrine is two rules that compose: **Rule 1 — Substrates are audience-neutral and capability-focused.** A substrate µservice provides a capability (storage, policy evaluation, identity issuance, cell management, observability rollup, compute scheduling, network routing, secrets management, audit emission, ontology projection, intelligence inference, workflow orchest

### ADR-131 residual

**ADR-0131-per-microservice-flat-layout** — oyatie adopts one universal artifact layout: **flat colocated service folders under `{oya,cloud}/<service>/`**, mandatory for every service/product in the repo. `oya/` holds product/domain services; `cloud/` holds platform/tenant-substrate services; shared cross-cutting code remains under `libs/`. Sales segmentation remains a PRD-frontmatter field, not a directory split. Historical references to `

### ADR-336 residual

**Valkey is the canonical in-memory KV / cache / pubsub substrate (Redis retired for license drift)** — ### B.1 Decision statement Valkey (Linux Foundation BSD-3-Clause fork of Redis 7.2.4, current mainline 8.x) is the canonical Oyatie in-memory key-value, cache, pubsub, and streams substrate. Redis 7.4+ (Redis Inc. SSPLv1 / RSALv2 dual-license) is retired from the Oyatie substrate allow-list. Pre-7.4 Redis (BSD-3-Clause) remains license-clean but is non-canonical due to absent upstream maintenance

### ADR-510 residual

**SCM destination = bespoke hyperscaler monorepo-VCS; GitHub transitory; cutover numerically triggered** — ### 1. The SCM destination is the bespoke hyperscaler monorepo-VCS — DECIDED The long-horizon SCM destination is a bespoke, self-hostable, Rust, hyperscaler-grade monorepo-VCS server in the Piper/Sapling/Mononoke class. This is a **decided destination, not an open "whether."** What remains open is only the **timing** of the cutover (§3). ### 2. GitHub + plain git is the explicit TRANSITORY canonic

### ADR-159 residual

**ADR-0159-feature-flag-substrate** — Oyatie adopts a dedicated `feature-flags` µservice as the canonical runtime feature-flag substrate. Properties: ### Three orthogonal gating tiers 1. **Code-deploy gate** = ChangeSet `acceptance_status` (ADR-0110). Code lives in `dev` / `staging` / `production`. 2. **Traffic-shape gate** = Progressive delivery via Flagger (ADR-0160) with SLO-gated promotion (ADR-0139). 1% → 5% → 25% → 100% of traff

### ADR-370 residual

**ADR-0370-local-production-fidelity-substrate-talos-apple-silicon** — The local production-fidelity substrate is **multi-node Talos Linux on Parallels Desktop 26**: 1. **Topology:** 3 control-plane (embedded-etcd HA, floating VIP) + 2 workers (Kata-capable), on the Parallels Shared net (10.211.55.0/24). Talos is immutable + API-managed (no SSH) — genuine prod fidelity, not a dev shim. 2. **Hypervisor = Parallels 26** (`prlctl` creates/configures/boots VMs **headless

### ADR-376 residual

**ADR-0376-managed-kubernetes-product-surface** — Oyatie's managed-Kubernetes offering is a **TWO-TIER product** on top of the ADR-0375 substrate. The tenant picks the tier; **the default is hosted**. - **Hosted control plane = DEFAULT tier.** Tenant Kubernetes control planes run as pods inside Oyatie's management (control-plane) cluster via **Kamaji** (`github.com/clastix/kamaji`), a hosted-control-plane manager listed in the CNCF landscape. Kam

### ADR-177 residual

**ADR-0177-internal-external-api-surface-separation** — ### D-1. Two gateway tiers | Tier | Hostname | Audience | Stability tier | Auth | Rate limit | | --- | --- | --- | --- | --- | --- | | **Public** | `api.oyatie.com` | External customers, public-SDK consumers, partners | `Public-Stable` or `Public-Preview` per ADR-0037 | OAuth 2.0 + per-key signature | Per-public-key, per-IP (ADR-0178) | | **Internal** | `internal-api.oyatie.com` | Other Oyatie µse

### ADR-246 residual

**ADR-0246-policy-engine-substrate-promotion** — ### D-1. Promote `cedar-fragment-coverage` BC to peer µservice `microservices/policy-engine/` The `cedar-fragment-coverage` bounded context resident in `microservices/ontology/` is promoted to its own peer µservice `microservices/policy-engine/`. The new µservice is a *substrate* (per ADR-0245 substrate-vs-product layering); it is consumed by every other µservice — substrate or product — via a thi

### ADR-599 residual

**Commission the comms calendar capability-first move + cloud-agnostic core slice (comms-calendar-domain/api/usecase)** — ### D1 — MOVE the domain into its capability home (de-branded, glob-covered) Relocate `oya/calendar/crates/oya-calendar-domain` to `comms/core/calendar-domain` via the deterministic reorg codemod (ADR-0563), de-branding the cargo name `oya-calendar-domain` → `comms-calendar-domain` (drop the vendor prefix; path-tail == cargo name; face dir not in the name). The crate lands under the existing `comm

### ADR-635 residual

**Face-aware substrate dependency graph v2: five typed graphs and derived failure closure** — ### D1 — `dependency_units` are `runtime_face`-qualified `specs/substrate-dependency-dag.json` v2 declares exactly 19 unique, closed `dependency_units` in the bounded W0-C topology slice. A unit id combines a canonical capability and its founder-locked `runtime_face`, for example `cell.envelope`, `cell.lifecycle.cp`, `iam.local-verifier`, or `policy.authoring.cp`. Every unit declares a capability

### ADR-338 residual

**Pod runtime tier 0..3 (Kata + Cloud Hypervisor for tenant-untrusted + tenant-data substrate; runc for first-party + edge** — ### B.1 Decision statement Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `pod_runtime_tier` field whose value is an integer in `{0, 1, 2, 3}`. The integer maps to a RuntimeClass (D-4), a nodepool placement contract (D-3), and a Kyverno admission policy (D-5). The CI lane `oya-check-pod-runtime-tier` (D-6) validates declaration presence + valid integer + nodepool-pl **[AMENDED 2026-08-19 — PATH SUPERSEDED, ENFORCEMENT SUPERSEDED.]** Two clauses of this gist are dead as written. (1) The declaration site `microservices/<name>/manifest.json` does not exist: `microservices/` holds zero tracked files, so any check keyed to that path matches nothing and passes vacuously. The field itself is alive and widespread — 78 of 107 tracked `manifest.json` files declare `pod_runtime_tier`, across 21 capability roots — at the ADR-0562/ADR-0615 capability-rooted shape `<capability>/manifest.json`, `<capability>/<service>/manifest.json`, or `app/<product>/manifest.json`. Read the path expression as capability-rooted, not `microservices/`-rooted. (2) The named enforcement vehicle, a Kyverno admission policy, is superseded by ADR-0710: admission is ValidatingAdmissionPolicy + CEL over a projected param resource, and the policy-engine category is ruled out as the default. The tier→RuntimeClass and tier→nodepool mappings themselves are untouched by this amendment. ADR-0714 (Proposed) would additionally retire the 0..3 integer in favour of orthogonal isolation-property / placement / trust axes; until it Accepts, the integer stands.

### ADR-34 residual

**ADR-0034-per-microservice-data-class-overrides** — Every microservice crate in the flat catalog that handles regulated data ships a **microservice override pack** in its kernel crate under `oya-<microservice>-override-pack-kernel`. The pack is a structured map from data class to hard-deny policy. It binds at runtime; tenant admin Cedar policies cannot raise the ceiling. **Naming justification (BNF v4.1, ADR-0056):** - `oya-medical-override-pack-ke

### ADR-571 residual

**Home the connect address-book domain into the comms capability and commission the contact-management port + usecase (wav** — ### D1 — MOVE: home the address-book domain into `comms/core` Relocate `oya/connect/crates/oya-address-book-domain` → `comms/core/connect-address-book-domain` via the reorg codemod (history-preserving `git mv` + package/lib-name de-brand per ADR-0532/0533: cargo `connect-address-book-domain` == path-tail, the `connect-` discriminator retained to name the address-book's product origin and keep the

### ADR-512 residual

**ADR-0512-canonical-monorepo-pattern** — Adopt the canonical pattern **"vertical-slice monorepo · one workspace · bounded-context crates · dependency-rule modules · Buck2 graph"**: 1. **Layout.** Service code lives at `{oya,cloud}/<service>/crates/<crate>/`; product-facing/domain services live under `oya/`, platform/tenant substrate services under `cloud/`, and shared cross-cutting libraries under `libs/<lib>/`. Each service co-locates c

### ADR-58 residual

**ADR-0058-flat-microservice-catalog** — We adopt the **flat microservice catalog** as the canonical architecture. No vertical, arm, product group, or platform grouping exists in code, directories, or architecture. Every feature and product is an independent microservice registered in `[workspace.metadata.oya.microservices]` (per ADR-0056 BNF v4.1). ### Canonical flat catalog (complete as of 2026-05-13) ``` Intelligence (internal-only, not te

### ADR-621 residual

**De-commit the active-artifact-contract graph projection** — **Proposed — 2026-07-24.** Council ratification and acceptance are **not established**: this two-way candidate awaits qualified-authority review. Protected admission of the implementation proves only the admitted mechanics and gate behavior; it neither changes this record's lifecycle nor supplies the missing authority. It is not a legal, operational, custody, affected-party, or production-readines

### ADR-375 residual

**ADR-0375-talos-capi-argocd-fleet-substrate** — - **Node OS:** Talos (immutable, API-managed). Bare-metal nodes auto-install zero-touch from a **USB** image (config baked for the control plane; fetched for spoke nodes). Cloud nodes use CAPI cloud images. - **Cluster lifecycle:** **Cluster API** — declarative `Cluster`/`MachineDeployment` CRs in git, reconciled by controllers. The management/control-plane cluster runs CAPI core + Talos providers
