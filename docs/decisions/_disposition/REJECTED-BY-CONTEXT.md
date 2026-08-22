---
doc_status: archived
---

# Freshly Rejected ADRs — by context

**Total Rejected:** 144  
**Process note:** Most were **not** deep-read for accept/reject quality. They were closed under **end-state freeze** (no remaining Proposed). Treat this list as a **review queue** for re-open (new ADR or flip to Accepted) where the design is still wanted.

**Legend — how rejected:**  
- *freeze* = Proposed → Reject without full adversarial read  
- *missing→reject* = no status frontmatter → fail-closed Reject  
- *residual* = last 14 Proposed closed in residual pass  

## A. Trust, safety & vulnerable-user doctrine (11)

### ADR-300 — Whistleblower + Press-Freedom + Anonymity Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why anonymity is a substrate primitive, not a per-tenant afterthought Anonymity-preserving submission surfaces are a critical-path edge case (per documentation-rigor.md §3.2.5 rows 6, 7, 16, 21) because the stand

### ADR-301 — Survivor-Safety + Domestic-Abuse Mode
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why survivor-safety is a substrate primitive, not a per-µservice afterthought Survivor-safety is a critical-path edge case (per documentation- rigor.md §3.2.5 row 8) because the standard auth-defence pattern — SM

### ADR-302 — Deceased-User Inheritance Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why deceased-user inheritance is a substrate primitive, not a per-µservice afterthought Deceased-user inheritance is a critical-path edge case (per documentation-rigor.md §3.2.5 row 10) because the standard accou

### ADR-303 — Cognitive-Impairment and Decision-Resilience Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why decision-resilience is a substrate primitive, not a µservice afterthought Modern hyperscaler-class fintech, healthcare, and consumer platforms treat cognitive-state-aware decision resilience as a *first-class

### ADR-304 — Cross-Jurisdiction Conflict Resolution Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why cross-jurisdiction conflict resolution is a substrate primitive Modern hyperscaler-class multi-region platforms treat cross-jurisdiction conflict resolution as a *first-class substrate primitive* — wired into

### ADR-305 — Delegated-Agent Authority Chain Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why delegated-agent authority is a substrate primitive Modern hyperscaler platforms — Microsoft, Anthropic, OpenAI, Google, Salesforce, Atlassian, Slack, Notion, Zapier — all ship delegated- agent authority chain

### ADR-306 — Disaster-Mode + Cell-Resilience Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why disaster-mode is a substrate primitive Modern hyperscaler platforms treat disaster-mode + cell-resilience as a *first-class substrate primitive* — wired at the planetary edge, in every cell's control plane, a

### ADR-307 — <human-readable>
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why detection is a substrate primitive, not a per-µservice afterthought Mature hyperscaler platforms treat detection (fraud, abuse, policy-violation, AML, insider risk) as a *first-class substrate primitive* — wi

### ADR-308 — ML Model Lifecycle — EU AI Act + NIST AI RMF + ISO/IEC 42001 Compliance
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why ML model lifecycle is a substrate-level commitment, not a per-model afterthought Mature ML organizations treat model lifecycle as a *first-class substrate primitive* — not as a per-team or per-model afterthou

### ADR-309 — Detection Fairness + Civil-Rights Compliance Baseline
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why fairness is a substrate-level commitment, not a per-model afterthought Mature ML organizations treat fairness as a *first-class substrate primitive* — wired centrally so every model serving production traffic

### ADR-310 — Investigation Case-Management Substrate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why investigation case-management is a substrate primitive, not a per-µservice afterthought Mature trust-and-safety + fraud-investigation + regulator-response organizations treat case-management as a *first-class

## B. CI / build / admission design (not yet admitted) (42)

### ADR-14 — Build-vs-buy policy — per-microservice matrix (in-house obligatory / external acceptable / requires-
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** PRD §3.1 commitment 4 sets the build-vs-buy posture: "in-house build over external dep wherever the dep is not as mature as `axum` / `tokio` / `serde` / a Postgres driver / OS kernel-grade tools." TOOLCHAIN §6 codifies t

### ADR-16 — Wave and plane integration framework — descriptive wave names (W-Foundation through W-Region-Fan-Out
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The legacy `M0..M3 / minimum-shippable-tier` milestone vocabulary baked in a date-bound + commercial-launch-bound mental model that no longer matches the optimal-path framing under unconstrained time/resource (PRD §3.1).

### ADR-21 — Foundry capability registry and MCP gateway — `Capability` schema, MCP-compatible discovery, per-ten
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Capabilities are the unit of work in the Foundry runtime: each capability is a typed contract (input schema, output schema, autonomy requirement, data classes touched, audit-chain emission topic, regulatory packs consume

### ADR-25 — Foundry as the engineering platform — repoctl, catalog, gates, fitness functions, supply chain, cust
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The 2026-05-09 reframing folded the standalone "Foundry engineering platform" axis into Foundry. The thesis: every engineering surface that gates how we build (repoctl, catalog, claim-ceiling validator, foundation-bypass

### ADR-32 — DCIM software for Oyatie-owned DC operations — `crates/cloud-dcops-*` with anti-scope on custom 
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0028 commits the cloud microservice to a three-phase trajectory ending in greenfield Oyatie mega-DCs. From Phase 2 onward we operate physical infrastructure: rack-and-stack, power, cooling, network ops, sustainabilit

### ADR-35 — Workflow engine — hybrid state-machine + DAG (not pure BPMN), per-tenant versioning, jurisdiction ov
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Every axis needs workflow: SaaS tenant onboarding, Workspace meeting scheduling, Vertical-pack claim adjudication, Foundry agent task execution, Cloud DCIM workorder dispatch. The pack-of-19 foundation ADRs named workflo

### ADR-36 — Plugin substrate — Wasmtime + WASI Preview 2 with capability-gated context, Cosign signing, trust ti
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Plugins are how third parties extend the ecosystem without forking it: a tenant-supplied analyzer in Workspace Sheets, a vendor-supplied EMR adapter in Vertical-Healthcare, an ISV-supplied custom retrieval plugin in Sear

### ADR-40 — Progressive delivery — Argo Rollouts canary, blue-green for stateful surfaces, metric-gated rollback
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A release that ships to 100% of tenants on cut is the highest-blast-radius operation in the system. The pack-of-19 foundation ADRs decided that progressive delivery is mandatory but did not pin the mechanics: which contr

### ADR-44 — Service mesh — Istio Ambient mode for east-west, Envoy as edge gateway, mTLS everywhere, per-cell na
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Cross-axis traffic is the cohesion thesis at the network layer. If axes call each other over plain HTTP without identity, without encryption, without policy, the cohesion-invariant guarantees from ADR-0001 don't extend t

### ADR-45 — Database tier strategy — PostgreSQL + Citus for OLTP, ClickHouse-fork for OLAP, Iceberg + DataFusion
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Every axis stores state. The pack-of-19 foundation ADRs decided that database choice is a substrate concern but did not pin the per-tier strategy: which engine for OLTP, which for OLAP, which for lakehouse. Without a pin

### ADR-114 — Canary observability gate + rollback
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The FINAL-FINAL branch pipeline (per `feedback_branch_pipeline_final_final`, implemented 2026-05-16) auto-promotes `dev → staging → production`. The promotion workflows (`promote-dev-to-staging.yml`, `promote-staging-to-

### ADR-295 — Bootstrap CI SPIFFE Identity + T+8h Kill-Switch
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### What F5-247-02 actually says F5-Security's r1 verdict (CRITICAL) reads: > ADR-0247 §D-5 Stage 1 describes external CI (GitHub Actions / > CircleCI / temporary self-hosted runner) deploying cosign-verified > images. T

### ADR-297 — Abuse-Defence Baseline — Anti-Bot + Anti-Spoof + Anti-Scrape
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why abuse-defence is a substrate primitive, not a µservice afterthought Mature hyperscaler platforms treat abuse-defence as a *first-class substrate primitive* — wired at the planetary edge, in every internet-fac

### ADR-312 — Court-Warrant Scoped Piercing
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0311 establishes a hard default-deny boundary at the personal-tenant edge: no employer-tenant Cedar permit can read an employee's personal-tenant surfaces. The boundary is load-bearing — without it, the consumer-trus

### ADR-325 — Capability Tier Pricing Anchors Public
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Named pressure ADR-0316 declared the Bronze / Silver / Gold / Platinum tier shape but explicitly deferred the monetary anchor question to a successor ADR. In the intervening period (2026-05-19 to 2026-05-20) the post

### ADR-340 — Capacity model per microservice manifest (baseline_cpu_per_tenant + baseline_ram_per_tenant + storag
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: no canonical per-µservice capacity declaration today Oyatie has 77 active µservices (47 baseline + 9 ERP + 13 B2B-leader + the in-flight 8 healthcare/marketing splits captured by the realignment e

### ADR-341 — Cellular promotion gates — explicit per-Tier 0..4 machine-checkable criteria + auto-promotion via ce
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: ad-hoc cell promotion is incompatible with hyperscaler-grade rigor Hyperscalers operate cellular architectures (AWS, Stripe, Cloudflare, Salesforce, Microsoft, Apple per the ADR-0248 named-precede

### ADR-346 — oya verify --ci-required MUST locally mirror the full CI matrix (cargo fmt + cargo check + cargo cli
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: PR #177 surfaced 7 CI failures the local verifier missed On 2026-05-21, PR #177 was pushed to `dev` after the author ran `cargo check --workspace` locally and observed exit-0. CI subsequently surf

### ADR-347 — Foundry-fitness to governance bulk rename (doctrine-only; all governance-fitness-* CI lanes + cr
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: anachronistic ownership label after ADR-0335 foundry retirement ADR-0335 (foundry retired, absorbed by intelligence) retired the foundry microservice as a first-class deliverable in the Oyatie cor

### ADR-348 — Autosharding + auto-rebalance + dynamic sharding (cellular topology MUST support three control-plane
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: hyperscaler horizontal scalability requires within-cell + across-cell automation The hyperscaler-grade bar declared by `feedback_quality_performance_scalability_bar` requires horizontal scalabilit

### ADR-360 — CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cach
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Direct observation (2026-05-25): `oya verify --ci-required` runs `cargo {check,clippy,nextest} --workspace --all-targets` with **no affected-target selection**, so a change touching only docs/specs/evidence YAML still tr

### ADR-537 — Dogfood bootstrap order + Rust-owned stack doctrine — the circular-dependency-free ten-step bring-up
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The substrate has circular dependencies at first boot: data encrypts with KMS keys while a naive KMS would store its state in data; DNS names the control plane while the control plane would configure DNS; the CAS

### ADR-540 — Cargo workspace to Buck2 target parity gate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** FRIC-1781063357 and FRIC-008(b) describe a false-green class in which a Rust workspace member can carry test code that Cargo sees but Buck2 never compiles or runs. ADR-0538 made workspace member enumeration canonical thr

### ADR-544 — Friction-ledger closed-loop accounting meta-gate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The friction ledger (`.omc/ultragoal/friction-ledger.jsonl`) is the running record of every pipeline defect the agent fleet hits. The founder decision of 2026-06-10 makes it a first-class governed surface: *every frictio

### ADR-545 — Embedded-asset hermeticity gate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A Rust crate can embed a file at compile time with `include_str!`/`include_bytes!`. The macro resolves its string-literal argument **relative to the including source file**, and rustc reads that path from wherever the fi

### ADR-546 — Canonical-JSON determinism gate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Machine-readable JSON is a load-bearing governed surface across the repo: `specs/root-hub-pointers.json` is the authoritative agent entry surface, `specs/masterplan.json` and `specs/master-plan-sequencing.json` drive seq

### ADR-549 — buck-syntax-kernel: one sound BUCK/Starlark parsing oracle + fixer self-validation harness
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Three-plus gate/tool crates carried private, divergent text-heuristic BUCK parsers (`cloud-ci-embedded-asset-hermeticity-app`, `cloud-ci-kernel-purity-app`, `tools/buck-test-wiring-app`, `cloud-ci-account

### ADR-559 — Commission the cloud-iam Cedar PDP service (G004 slice 1): a runnable authorization-decision service
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Three live consumers already authorize through Cedar PEP **adapters** against a decision substrate that does not exist as a service: 1. **oya/identity** — `CedarWorkloadAuthorizer` behind the `WorkloadAuthorizer` port (A

### ADR-567 — Commission auth durable stores with Postgres + RLS (tenant-lifecycle-store-postgres and identity-sci
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0564 commissioned the tenant-lifecycle service with a deliberately transitional in-memory store. The auth/onboarding E2E audit records a persistent store as the next required slice (D5). This ADR delivers the durable

### ADR-570 — Clean-arch port-placement gate (ports defined in core, not adapters)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The owned-stack ports/adapters doctrine (CLAUDE.md `owned_stack_policy`; ADR-0510 transient-adapter framing; ADR-0547 kernel-purity) draws a clean-architecture seam: a **port** — the storage/repository interface a domain

### ADR-581 — Fail-closed verified-caller + PDP authorization for the workload-principal lifecycle control plane (
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `iam/facade/identity-workload-rest` (`iam-identity-workload-rest`) is the workload-identity REST PEP (ADR-0105 Layer 5). Among its routes it mounted two MUTATING control-plane custom methods: - `POST /principals/{id}:sus

### ADR-582 — DTO-authz-trust gate (caller-supplied authorization decision backstop)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A whole-repo security review found a #1 systemic antipattern at 30+ trust boundaries: **caller-supplied authorization trusted as the authz decision.** A request handler / use-case reads an *authorization decision* FROM t

### ADR-586 — Fail-closed verified-principal + server-side PDP authorization for tenant.create and the tenant-life
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A Wave-2 capability-tenancy security review surfaced two related self-attested-authorization defects in the tenancy delivery surface — the AUTH-005 class (PR #768 shipped an unauthenticated mutating control plane that pa

### ADR-587 — Fail-closed verified-principal + PDP authorization for the Cloud Network LB/VPC/DNS create control p
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The Cloud Network boundary crates own the tenant-facing create surfaces for three resource types: - `network/ports/lb` (`network-lb`) — `cloud.network.lb.create` (`create_cloud_network_load_balancer_from_api`) - `network

### ADR-588 — Fail-closed verified-principal + PDP authorization for the audit.event.emit boundary (C15 tamper-evi
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `audit/core/usecase` (`audit-usecase`) is the Platform Audit Chain app boundary: it owns CloudEvents envelope normalization, request-fingerprint idempotency, the immutable platform audit-chain append, and the eventing ou

### ADR-590 — Fail-closed verified-principal + server-side PDP authz for the Cloud Observability audit-read surfac
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `observability/core/api` owns the boundary [`read_cloud_observability_audit_from_api`] for the `cloud.observability.audit.read` surface. The surface serves immutable audit records: control-plane mutation history under th

### ADR-600 — Root-workspace-hygiene allowlist gate — make committed repo-root scratch structurally impossible
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The repository root accumulated committed process scratch: the Jun-10 G011 slice-6 burndown logs (`slice06-*.log`), retest/target scratch (`retest-targets.txt`, `backfill-targets.txt`, `branch-wired-members.txt`, `final-

### ADR-612 — buck2 Remote-Execution phase: deploy nativelink-scheduler + nativelink-worker, flip remote_enabled=t
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### 1. What is already built (consumed wholesale, not re-decided) - **ADR-0556** classified every build class as cold or warm as **policy-as-data** (`/specs/cache-warmth-policy.json`), fixed the one-way cold-required flo

### ADR-618 — Contract-slice conformance gate scope boundary: single-document internal-shape validation, cross-ref
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The `ci/facade/contract-slice-conformance` gate (ADR-0515 WS-D pure gate; the `source_migration_slice` Python→Rust retirement pattern) replaces the fleet of `scripts/tests/*_check.py` contract-slice validators with one o

### ADR-628 — Scan-root liveness: a declared coverage root that no longer resolves is a gate blind spot, not clean
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Gate policies declare the roots they scan. The cloud-ci fleet already enforces that a declared root cannot be **removed** without the removal being the subject of a reviewed change — the anti-narrowing ratchet, e.g. `rus

### ADR-629 — Crate-catalog coverage: every live crate carries a catalog row, closing the crate→row direction
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `registry/catalog/` holds one YAML file per crate, **keyed by FILENAME**. That single property makes a missing row invisible to the search anyone would actually run: the crate name lives in the PATH, not the contents, so

### ADR-631 — A capability that spans strata has a wrong boundary, not a tier problem: split iam into iam (S1 PDP)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** PR #1481 restored tier enforcement to capability roots. It requires every root in `capability_roots` to DECLARE `tier` + `substrate_dag_position.stratum` in `specs/capability-registry.json`, and makes an undeclared capab

## C. CAS / identity-for-build / RE path (8)

### ADR-38 — Trust framework — cross-microservice lineage, DSR cascade across all all microservices, Cosign-signe
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The cohesion thesis (ADR-0001) commits us to *one* audit chain, *one* identity surface, *one* consent store. The Data Use Boundary (ADR-0008) commits us to enforced cross-microservice flow gating. The per-vertical overri

### ADR-336 — Valkey is the canonical in-memory KV / cache / pubsub substrate (Redis retired for license drift)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: Redis Inc. relicensed in March 2024 from BSD to SSPL/RSAL Redis Inc. relicensed Redis on 2024-03-20 from the prior 3-clause BSD license to a dual SSPLv1 (Server Side Public License) / RSALv2 (Redi

### ADR-377 — GitHub board projection with git-ref CAS fallback
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0363 retired the bespoke agentic-VCS layer and made plain `git` + cloud-ci + GitHub (interim) the coordination substrate. ADR-0369 then selected gated stacked-trunk on plain git and GitHub PRs. ADR-0374 added the Git

### ADR-561 — Commission the workload-identity X.509-SVID issuance + PDP caller-tenant-binding substrate (G002 sli
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0559 commissioned the cloud-iam Cedar PDP as a runnable service. Its `AuthorizeRequest` carries `tenant_id` (proto field 2) and the gRPC handler binds it **verbatim** from the caller body (`grpc.rs`: `tenant_id: requ

### ADR-571 — Home the connect address-book domain into the comms capability and commission the contact-management
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The capability-first reorg (ADR-0562/0563) homes each capability's crates under `<capability>/{core,ports,adapters,facade}` via the deterministic reorg codemod. The `comms` communications plane was established by the twe

### ADR-589 — Fail-closed authz for the DSR erasure cascade (AUTH-005 / Wave-2b remediation)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `compliance/ports/dsr-usecase` is the application boundary for the GDPR **erasure cascade** (`dsr.cascade.execute`, `execute_dsr_cascade_from_api`). A single accepted request fans out irreversible erasure / correction di

### ADR-598 — Commission the comms meet capability-first core slice (comms-meet-api port + comms-meet-usecase)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The capability-first reorg (ADR-0562) homes each product capability under `<capability>/{core,ports,adapters,facade}/`. The `comms` capability tree was established by the twelfth strangler move (ADR-0562 §10.16, mail + m

### ADR-599 — Commission the comms calendar capability-first move + cloud-agnostic core slice (comms-calendar-doma
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The capability-first reorg (ADR-0562) homes each product capability under `<capability>/{core,ports,adapters,facade}/`. The `comms` capability tree was established by the twelfth strangler move (ADR-0562 §10.16, mail + m

## D. Security control-plane authz (AUTH-005 class) (11)

### ADR-7 — Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling (T1–T4) with per-capability runtim
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Cohesion (ADR-0001) requires a single authorization surface across all microservices. Without a unified policy DSL, every axis ships its own AuthZ logic — and the prior decade of multi-product engineering shows that drif

### ADR-22 — Autonomy ceiling — runtime enforcement via Cedar policy at every capability invocation
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A capability declares the autonomy tier it requires (T1 = recommend-only, T2 = supervised execution, T3 = scheduled autonomous, T4 = continuous autonomous). The actual autonomy granted to an invocation depends on the ten

### ADR-39 — Supply chain security — Trivy 4-layer scan, Cosign keyless signing, SBOM dual-format, signed commits
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The supply chain is the single most-exploited attack surface in enterprise software (SolarWinds, Codecov, log4shell, xz-utils backdoor). For Oyatie — which ships all microservices, dozens of vertical packs, and a third-p

### ADR-43 — Secrets management — OpenBao (MPL-2; supersedes Vault BUSL), per-tenant per-cell HSM partition (KCmi
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Secrets management touches every axis: per-tenant API keys for Foundry adapters, per-cell HSM partitions for KMS, per-capability rotating tokens for subscription-mode AI providers, signing keys for the Trust framework's 

### ADR-294 — Cedar Fragment Soak + Anomaly-Rollback
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### What F5-243-01 actually says F5-Security's r1 verdict (CRITICAL) reads: > ADR-0243 §D-6 + §D-10 specify <5s p99 hot-reload across cell > replicas via Kafka pub-sub. Adversary model: a compromised > intermediate signi

### ADR-543 — Commission the cloud-kms K8s operator (G002 slice 2)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** G002 (trust substrate) slice 1 landed the KMS enclave one-way-door, crypto-shred, typed root provenance, and zero-static-secrets leasing — but cloud-kms had NO operator: no CRDs, no reconciliation, no GitOps actuation. T

### ADR-573 — Fail-closed authz for the Cloud KMS crypto control plane (AUTH-005 / C5 remediation)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `secrets/ports/kms-api` is the API boundary for the **crown-jewel** Cloud KMS crypto surfaces `cloud.kms.encrypt` / `cloud.kms.decrypt` (`authorize_cloud_kms_encrypt_from_api` / `authorize_cloud_kms_decrypt_from_api` in 

### ADR-591 — Fail-closed authz for the Cloud FinOps report API (AUTH-005 capability-billing remediation)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `billing/ports/finops-api` is the boundary library for the Cloud FinOps report surface (`cloud.finops.report`): it normalizes a request and generates a multi-tenant **cloud-spend report** — `FINANCIAL_REGULATED_CREDIT`-c

### ADR-593 — Fail-closed authz for the Accounting + Payroll money-mutation control planes (AUTH-005 / Wave-2b mon
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Two HTTP runtime adapters bind the most money-sensitive surfaces in the ERP product verticals to the repo-native Hyper router/middleware foundation: - `billing/adapters/accounting-http` (`billing-accounting-http-adapter`

### ADR-603 — Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `oya/crm/crates/crm-revenue-app` exposes mutating multi-tenant CRM control planes (`adapter::http`, `adapter::grpc`, `adapter::asyncapi`) over the capabilities account-master, opportunity, quote, campaign, and servic

### ADR-607 — Fail-closed Cedar authz on the managed-K8s control-plane facades (cluster-lifecycle / control-plane-
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** GH #979 (parts 1+2): the managed-K8s control-plane facades trusted forgeable in-band authority — an AUTH-005 (ADR-0573) instance on the cluster-admission and quota control plane. - `cluster-lifecycle-app` compared an `x-

## E. Capability reorg / product surface commissioning (3)

### ADR-20 — Foundry multi-provider adapter model — `ProviderAdapter` trait, ProviderAuth, capability-level routi
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Foundry is the force-multiplier axis: every other axis (cloud, search, ads, saas, vertical, workspace) invokes Foundry capabilities, and each capability invocation must reach a model provider. The current draft surfaces 

### ADR-24 — Foundry eval harness and replay — per-capability golden sets, A/B routing, adversarial cohorts, regi
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A capability that lacks an eval set is a capability we cannot reason about. Without a golden set, regression detection is impossible; without adversarial cohorts, the autonomy ceiling and the data-class boundaries are th

### ADR-344 — Sustainability + finops dimensional model (per-call CO2-grams + watt-hours + USD-cost emitted alongs
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: regulatory wall arriving 2025-2026 across three major jurisdictions Three named regulatory pressures land within the same 18-month window and converge on the same demand: per-call, per-tenant, per

## F. Planning / governance machinery (7)

### ADR-236 — OP-11 Corpus Remediation Planning Contract
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The active masterplan records that the 2026-05-17 OP-11 audit found broad aspirational-enforcement drift: ADRs and standards cite lanes that are not implemented, some product and plan claims are not bound to validators, 

### ADR-293 — Foundry Meta-Trust-Root for Self-Modification Witness
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### What F5-247-01 actually says F5-Security's r1 verdict (CRITICAL) reads: > The Cedar fragment at ADR-0247 §D-8 permits self-modification actions > when `principal.is_human_approval_present(min_approvers: 2) || > princ

### ADR-542 — Cloud-Intelligence XPROXY External-Proxy Parity Lane: commissioning and governance path
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The cloud/cloud-intelligence service requires a parity lane that commissions and validates external-proxy (XPROXY) capability: the ability to proxy requests to external LLM providers (OpenAI-compatible, Gemini-native, An

### ADR-617 — The Living Monorepo Governance Graph — monorepo management + project lifecycle as one governed, fede
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0516 set the agentic-delivery-fabric apex; ADR-0517 ratified the owned content-addressed AST substrate *doctrine*, realized so far as the ADR-0580 `governance/corpus/` Phase -1 spike; ADR-0522 established "one graph,

### ADR-622 — Define a nonbinding FixupTask v2 successor foundation
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The append-only `registry/fixuptasks.jsonl` has historical rows but lacks a machine-checkable lifecycle for new or modified work. ADR-0619 requires retired predecessor context to remain in Git history rather than a reada

### ADR-623 — Keep the pre-roadmap Stage-1 evidence epoch mechanism-neutral
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The current `masterplan_v2.planning_entry_contract` correctly keeps binding planning and execution dispatch closed, but its open-state evaluator is not a complete exit contract. Its evidence pointer is a dated snapshot, 

### ADR-626 — Resolve fixup-ledger merges structurally instead of by hand
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `registry/fixuptasks.jsonl` is an append-mostly JSONL ledger. Line 1 is a schema header carrying no `id`; every other line is one task row keyed by `id`. Filing a finding appends a row at the end of the file. Two lanes t

## G. Kernel / bare-metal / Talos (5)

### ADR-2 — Establish the Tenant and Identity kernel as the single substrate every axis consumes
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The cohesion thesis (ADR-0001) names *single tenancy* and *single identity* as two of the six shared substrates. Without a dedicated kernel that owns the `Tenant` shape and the identity primitives, every axis trends towa

### ADR-23 — Foundry sandbox — Wasmtime + WASI Preview 2 for short-lived tools, Firecracker microVMs for full-ker
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Capabilities invoke tools — and a tool can be anything from a small deterministic transform (parse JSON, render a template, run a regex) to something that needs a full Linux kernel surface (run a compiler, exec a vendor 

### ADR-382 — Bare-metal Talos zero-day bring-up via Sidero Metal
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The substrate stack so far: - ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (production fleet pattern). - ADR-0378 — vfkit + Talos canonical **local** substrate (macOS dev box; single-VM bring-up via `talos-lo

### ADR-568 — born-accounting register_crate: the pure registrar kernel (RegisterCrateRequest → RegistrationPlan)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Every new-crate PR this session (#783 sqlx, #779/#780 gates) took roughly four CI round-trips because born-accounting is a derived join over about six hand-authored SSOTs with NO single entrypoint. Adding one crate means

### ADR-611 — Land the Asterinas real-boot foundation harness under kernel/ (kuberos Wave-1 shard-1)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0512 sanctions a single nested/excluded Cargo workspace at top-level `kernel/` (the rung-0 kuberos kernel; the `no_std`+custom-sysroot rung cannot share the std-targeted root lockfile), and ADR-0562 registers `kernel

## H. Intelligence / LLM substrate (4)

### ADR-10 — Regional pack architecture — canonical seams + per-locale plug-ins for regulatory, compliance, i18n,
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Korea-as-launch-locale was the prior framing; the 2026-05-09 reframing retired it in favor of **canonical-architecture + regional-pack plug-ins** so multiple markets onboard in parallel rather than retrofit Korea-specifi

### ADR-26 — In-house AI model substrate — long-horizon W-AI-Model-Substrate; consume providers until per-vertica
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** We do not aspire to be a frontier-LLM lab. The forces that pull us toward in-house models are different: per-vertical accuracy where regional and domain language matters more than generalist reasoning (Korean legal corpu

### ADR-27 — Robotics, vision, and speech sub-substrates — vision/speech model crates, robotics control plane, de
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The capability registry (ADR-0021) already accommodates capabilities whose model substrate is text — chat, summarization, code, retrieval. Vision (OCR, classification, detection, video analytics, scene understanding), sp

### ADR-254 — Deployment model spectrum
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Why a deployment spectrum exists at all The oyatie platform serves customers whose deployment requirements span a continuum of operational ownership and connectivity: - A B2C consumer (a personal user of Mail, Drive,

## I. Platform product/substrate foundations (early pack) (14)

### ADR-9 — Cell architecture — per-tenant per-region blast-radius cells with cell-routing primitives at edge / 
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The PRD declares horizontal-scale-end-to-end and cell-isolation-evidence as foundation invariants. Without explicit cell architecture, the flat-catalog cohesion claim degrades in two predictable ways: (a) a single tenant

### ADR-47 — Search backend strategy — pgroonga day-1 (LGPL legal isolation), Tantivy in-Rust at scale, OpenSearc
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Search backend is the indexing + retrieval engine that sits behind the search microservice (per ADR-0030). The pack-of-19 foundation ADRs decided that search is critical but did not pin the engine. The decision is constr

### ADR-49 — Cross-region replication + residency — per-pack default residency class, opt-in cross-region per con
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Data residency is the single most-litigated cloud-services compliance dimension. KR Personal Information Protection Commission (개인정보보호위원회) has issued multiple guidelines tightening cross-border transfer rules; EU regulat

### ADR-242 — `oyatie`-is-a-tenant doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Prior portfolio state (pre-keystone) The oyatie portfolio inherited from Bominal a doctrine that treated "internal-platform" use cases as architecturally distinct from "consumer-facing" use cases: - **ADR-0136 (Found

### ADR-244 — Tenant as Universal Scoping Primitive
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Prior portfolio state The pre-keystone portfolio scoped decisions along three orthogonal axes that frequently conflated: 1. **µservice audience** (per ADR-0220, ADR-0239, ADR-0221 §M-04): each µservice declared `audi

### ADR-272 — Cookie Consent + Per-Purpose Analytics Opt-In
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** oyatie is the open-source parallel to Bominal: a multi-product, multi-tenant SaaS substrate hosting Workflow Studio, Ontology, Cloud products, and a marketplace. Every product surface that touches an end-user web/mobile 

### ADR-273 — ADR-0273 — Per-tenant DKIM/SPF/DMARC email deliverability
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Why this is tier-1 lockdown Email is the single hardest substrate to ship correctly. A mistake on inbound auth (SPF/DKIM/DMARC) silently routes phishing into customer inboxes; a mistake on outbound auth silently rout

### ADR-311 — Dual-Tenant Identity — Personal-vs-Work Boundary
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Hyperscaler-grade consumer-and-enterprise platforms must serve a single human across two strictly-separated data domains: 1. **Work surfaces** — communications, files, calendars, workflow executions, payments, and other 

### ADR-313 — Conglomerate-Tenant Hierarchy — Sovereign-Child + Policy-Engine-Mediated Controlling-Entity Grant
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Holding companies, parent/subsidiary relationships, multi-brand groups, joint ventures, sovereign-wealth-fund portfolios, private-equity rolls, family offices, conglomerate-of-conglomerates (Berkshire-Hathaway, SoftBank 

### ADR-326 — Per-Tenant Data Residency Attestation
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Named pressure The ADR-0325 pricing anchor cross-references a `residency` dimension but ADR-0325 alone does not define the dimension's semantics; this ADR is its companion specification. Beyond pricing, residency pre

### ADR-338 — Pod runtime tier 0..3 (Kata + Cloud Hypervisor for tenant-untrusted + tenant-data substrate; runc fo
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: Kata-everywhere costs without security gain on trusted first-party code The ADR-0254 invariant ("K8s + Cloud Hypervisor + Kata pods") was authored before the cellular tier numbering convention fro

### ADR-343 — DR + RTO/RPO matrix per-µservice + per-compliance-pack (effective tenant RTO/RPO = max(µservice decl
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: ADR-0241 portfolio is one-dimensional; compliance reality is two-dimensional ADR-0241 established the canonical four-tier DR portfolio: T1 (< 5 min RTO, 0 RPO, active-active multi-AZ cross-region 

### ADR-564 — Commission the tenancy tenant-lifecycle registration service (G006 slice 1): a runnable tenant regis
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The auth/onboarding E2E ground-truth audit (`.omc/ultragoal/auth-onboarding-e2e-audit-findings.md`, 2026-06-19) found tenant registration **PARTIAL**: a real, conformance-tested lifecycle finite state machine exists, but

### ADR-592 — Tenant-scoped, body-fingerprinted accounting idempotency keys (cross-tenant collision fix)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The accounting journal capability records app-layer audit and Workflow-dispatch envelopes keyed by an idempotency key. The key is constructed in the app layer (`billing/core/accounting-app/src/lib.rs`) and consumed by th

## J. Eventing / audit stream (1)

### ADR-569 — Commission the data outbox CDC change-stream Postgres adapter (data-outbox-adapter-postgres)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Story G003 (the owned `data` persistence substrate; ADR-0536 D-10 change streams / D-13 messaging) has its SQL WRITE side commissioned: `libs/data-sql-adapter-sqlx` is the ADR-0510 transitional Postgres implement

## K. Other (38)

### ADR-4 — Plane separation across control / data / analytics with catalog-declared plane class
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Every Oyatie surface — across all all microservices — falls into one of three execution profiles: low-frequency / high-trust / audit-heavy operations that *configure* the system; high-frequency / latency-bounded operatio

### ADR-19 — PRIVACY-PROGRAM.md
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The consolidated docs tree (PRD, DESIGN, ROADMAP, PRIVACY-PROGRAM, COMPLIANCE-MATRIX, GLOSSARY, TOOLCHAIN, CONTRADICTION-LEDGER, ADR-INDEX, plus per-microservice + per-vertical + per-pack + per-runbook entries) is a livi

### ADR-48 — Korean morphology + multilingual tokenization — `Tokenizer` trait per language family, mecab-ko + kh
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Korean is morphologically rich: a single eojeol can carry stem + tense + politeness + connective in one orthographic word. Generic Unicode tokenization (whitespace + ICU) destroys retrievability for Korean queries. The K

### ADR-54 — Resolve new-crate chicken-and-egg via grit scaffold-claim pattern (icm-coordination-lock fallback)
- **How rejected:** deprecated without successor→rejected
- **About:** `grit claim` requires a real indexed code symbol in the form `<file>::<Identifier>`. A *new* crate (e.g., `tools/tooling-agent-read/`) has no source files yet, hence no indexed symbols, hence cannot be locked via `gr

### ADR-111 — Merge queue: projected-merge-state + fix-at-any-stage
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Under agentic load (N changesets in flight concurrently per ADR-0110), the merge queue is the only point of serialization. Three failure modes that the naive "FIFO + run-tests-then-merge" model can't handle: 1. **Diverge

### ADR-134 — Portfolio Hyperscaler Pattern Remediation Backlog
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The hyperscaler pattern audit and current PR review queue identified recurring portfolio gaps across Foundry, Workflow, Workflow Studio, Ontology, and Cloud: - LLM/tool invocation loops need bounded retry budgets and cir

### ADR-250 — Build-Ahead-of-Certification Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### The certification-versus-build race condition Every major regulated software market is gated by a certification artifact (PCI DSS Level 1, HIPAA + BAA, FedRAMP Moderate/High, IL5/ IL6, KR CSAP, EU PSD2, etc.) issued 

### ADR-257 — Ontology Object-Type Versioning + Deprecation Handshake
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Ontology is the canonical read substrate Per ADR-0145 (Inter-microservice Communication Reform) and the `feedback_workflow_objectgraph_adapter_layer` memory, the load-bearing architectural rule of the oyatie platform

### ADR-263 — ADR-0263 — Observability Emission Contract
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Three Pillars of Observability The canonical formulation of "observability" in production systems — as distinct from "monitoring" — was articulated by Charity Majors (CTO of Honeycomb) in 2017 ("Observability — A 3-Y

### ADR-276 — Backup + Portability Format (GDPR Article 20)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### What Article 20 actually requires Regulation (EU) 2016/679 Article 20 ("Right to data portability") is the canonical right: a data subject has the right to receive personal data concerning them, which they have provi

### ADR-284 — Platform-Owner-Name Indirection
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Why ADR-0242 left a hardcoded slug behind ADR-0242 was authored under the keystone-bundle constraint that 14 foundational ADRs land together as a mutually-reinforcing set. The authors' explicit framing was "establish

### ADR-292 — Minor User Doctrine — COPPA + KOSA + EU Age Verification
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### The Tier-1 lockdown framing The masterplan distinguishes between **Tier-1 lockdowns** (cannot ship without; statutory penalty exposure) and **Tier-2 hardenings** (should ship without; reputational exposure only). Min

### ADR-296 — Library-First Credential Sidecar
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### What F5-255-01 actually says F5-Security's r1 verdict (CRITICAL) reads: > ADR-0255-amendment §D-2 mandates that every caller's process > holds: (a) Anthropic/OpenAI/Google/Bedrock provider credentials > (resolved via

### ADR-298 — Emergency-Services Bypass — Life-Safety Hard Rule
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why emergency-services bypass is a substrate primitive, not a per-µservice afterthought Emergency-services traffic is the **single highest-priority class of request** any internet-facing platform can carry. A fai

### ADR-299 — Account-Recovery + Hijack-Recovery Resilience
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### §A. Why account-recovery is a substrate primitive, not a per-µservice afterthought Account-recovery is a critical-path edge case (per documentation- rigor.md §3.2.5 row 2) precisely because the standard auth-defence 

### ADR-314 — Marketplace as Universal Deal-Settlement Substrate
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** SAP S/4HANA parity cannot be achieved if marketplace only means retail catalog. FI, CO, MM, SD, SRM, CRM, GTS, TRM, EWM, and network products all need deal settlement as a common primitive. Purchase requisitions, sales o

### ADR-315 — ERP Coverage Doctrine - SAP-Parity Goal
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** SAP S/4HANA bundles FI, CO, MM, SD, PP, QM, PM, HCM, PS, PLM, EHS, SRM, CRM, SCM/APO, GTS, TM, EWM, TRM, RE-FX, IS-* packs, network products, platform extensibility, and data analytics. Competitors distribute similar cap

### ADR-317 — Role-Based Projection + Unified UX Shell Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** 1. Role switching is an explicit state transition, not a side effect of route changes. 2. The transition changes role_projection_id, permit_set_refs, Ontology projection refs, Workflow template library refs, UX shell ref

### ADR-318 — Adopt collar-color and workspace universality doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Oyatie is not a white-collar SaaS shell with optional field add-ons. It is a universal workspace substrate that must operate across technical and non-technical workers, office and non-office environments, every supported

### ADR-319 — Front Office / Middle Office / Back Office Information-Barrier Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Regulated finance does not treat all employees inside one legal customer as equivalent readers. A universal tenant boundary is necessary, but it is not sufficient for investment banking, brokerage, research, trading, ass

### ADR-320 — Apprentice, Intern, Resident, and Fellow Transient Identity Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** A.1 Oyatie models tenant as the universal scoping primitive. ADR-0244 already allows a tenant to represent an institution, employer, household, school, cohort, program, marketplace, or audience-defined operating cell. Ap

### ADR-321 — B2B SaaS Industry-Leader Coverage Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ## Section A - Context A.1. ADR-0315 already covers SAP S/4HANA parity and the first nine ERP microservice anchors. This decision deliberately does not reopen those nine directories. A.2. ADR-0314 makes marketplace the u

### ADR-324 — Anti-Script Anti-Template Doctrine
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### Named pressure The codex-erp-ip-w2 incident on 2026-05-18 ("lambda-wrap failure") is the proximate trigger. An agent on the codex worker fleet wrote a bash loop that fed a constant body template through jq with a sin

### ADR-337 — Apache Iceberg is the canonical OLAP table-format write path (Delta + Hudi demoted to migration adap
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: hyperscaler convergence on Iceberg as the interop format The four largest OLAP-table-format vendor camps converged on Apache Iceberg as the cross-vendor interop format between 2023-2025. **AWS** a

### ADR-339 — Shared IaC module library (`microservices/cloud-iac/modules/<context>/<primitive>/` is canonical; pe
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### A.1 Named pressure: 385-module-dir blast-radius today Oyatie has 77 active µservices (47 baseline + 9 ERP + 13 B2B-leader + the in-flight 8 healthcare/marketing splits captured by the realignment effort). Each µservi

### ADR-352 — Oyatie from-scratch architecture handoff
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Oyatie is an agentic-primary, machine-optimized, programmatically governed ecosystem-as-a-service platform. It is not a bundle of loosely related products. It is one coherent tenant-scoped operating substrate with flat m

### ADR-353 — Amendment — Library-First / Network-Opt-In Clarification
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### F-ANTI-2: the F4-architecture finding that triggered this amendment The 2026-05-20 multispectrum-review v2.4.0 F4-Architecture verdict (`evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`) issued fin

### ADR-354 — Amendment — HTTP/3 Fallback Chain, Strict TLS, ECH, PQC Hybrid
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ADR-0253 established HTTP/3 + QUIC as the default transport for all oyatie endpoints (KS#10 per `feedback_http3_quic_default_protocol`). However, the original ADR left the following operational parameters advisory or uns

### ADR-355 — Amendment — Library-First / Network-Opt-In Clarification
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### F-ANTI-1: the idea-refine finding that triggered this amendment The 2026-05-20 idea-refine pass over the 14-ADR foundational keystone bundle surfaced finding **F-ANTI-1**: > ADR-0145 (2026-05-18) retired the universa

### ADR-356 — Amendment — Library-First Ontology Read-Path Clarification
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** ### F-ANTI-3: the F4-architecture finding that triggered this amendment The 2026-05-20 multispectrum-review v2.4.0 F4-Architecture verdict (`evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`) issued fin

### ADR-541 — Corpus Liveness Graph: one content-addressed corpus graph with per-class decay invariants
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The founder directive of 2026-06-10 names a family of symptoms with no fundamental mechanism: missed directives from accepted ADRs, directive drift, lossy context across sessions, staleness, dead code, dead files, and un

### ADR-553 — Commission the identity runnable workload-identity service (G005 slice 1)
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** G005 promotes `oya/identity` from a set of library crates into a runnable service. The founder identity-layering directive (2026-06-10) fixes the architecture this slice must respect: **cloud-iam is the IdP substrate** (

### ADR-558 — Friction-ledger structural merge driver: id-aware union + second-author conversion
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `.omc/ultragoal/friction-ledger.jsonl` is an append-only, event-sourced JSONL surface: PRIMARY rows (`friction` + `status`) anchor a friction id, UPDATE rows (`status_update`) append disposition transitions, and the ADR-

### ADR-580 — corpus substrate Phase -1: the conservative-v1 syn-over-source AST extractor spike
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The live-AST governance substrate ("corpus") is the planned fail-closed answer to decay/drift/ staleness: a content-addressed AST graph where docs/directives are build artifacts and liveness is a fail-closed invariant ra

### ADR-610 — Policy-IR benchmark stage-0: pre-registered frozen rubric + fixture suite as governed data
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The owned Policy IR direction (see `docs/ideas/policy-pack-substrate.md`; Cedar is a benchmark dialect, not the north star) requires an engine-selection benchmark whose evidence is admissible: the grading rubric and the 

### ADR-620 — Pre-admission inventory provenance for history-only retirement observation surfaces
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** The total-accounting resolver creates a mechanical path-to-ADR inventory reference when an ADR contains an exact tracked repo-relative path token. It does not interpret ADR lifecycle status and must not be used as a plan

### ADR-621 — De-commit the active-artifact-contract graph projection
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** `registry/graph/active-artifact-contract-edges.json` is a deterministic projection of the ordered rows in `registry/artifact-capabilities-registry.json`: one `artifact_id -> artifact_profile` `declares` edge per row. The

### ADR-625 — Commit OpenTofu provider dependency locks for every deployable root
- **How rejected:** was Proposed → end-state freeze Reject (not context-reviewed)
- **About:** Before this decision, **zero** deployable OpenTofu roots carried a `.terraform.lock.hcl`. Every `tofu init` therefore resolved provider plugins to whatever the registry served at that moment. The gap is not theoretical, 

