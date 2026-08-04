# Ultragoal: FD-001 Enterprise SaaS — vertical core + unified shell + full cloud substrate, dogfooded

**Authored:** 2026-06-09 · **Base:** `dev` @ `8acec8920` · **Mode:** aggregate /goal, parallel lanes after contract lock

## Mission (founder, 2026-06-09)
Ship production-ready, industry-leading, trailblazing Enterprise SaaS: the FIRST VERTICAL MODULE (FD-001 tenancy+RBAC core — the masterplan/ADR-0217 mandated first deliverable) + the UNIFIED FRONTEND SHELL hosting modules + EVERY required cloud substrate at FULL extent (IdP, Cedar policy engine, KMS, persistence, observability, audit, metering/billing, messaging, network/DNS) to support AND dogfood the product. Never MVP/demo/good-enough/defer. Every delivery friction is a pipeline-product failure → becomes an enforced gate/automation.

## Founder directives (binding, recorded in session memory)
1. Root `goal.json` is stale — authority order: HANDOFF.md → /specs/masterplan.json (FD-001) + master-plan-sequencing → ADR-0516..0535 fabric canon.
2. Authorization = **RBAC + ABAC + PBAC** full spectrum (Cedar natively models all three). Read every "Tenant RBAC" shorthand as full-spectrum.
3. **Proven patterns, Rust reimplementation** — every decision cites its hyperscaler precedent; no invented architecture where proven practice exists.
4. **No mockups/prototypes** — production-grade running systems only; rename/productionize the Leptos "prototype" crate.
5. **Parallelize** — API-first contract lock, then independent worktree lanes (masterplan `parallel_lanes_after_contract_lock` doctrine).
6. **ALL CLI retired** — authority = cloud-ci gates/required contexts; operations = console + API; no new CLI surfaces ever.
7. **Cloud-native, Kubernetes-native operation — written in Rust, whole stack owned**: CRDs + operators/reconcilers + GitOps for everything, zero imperative ops; AND the stack itself is owned Rust end-to-end — kuberos-kernel (`cloud-kernel`) → Talos-like OS (`cloud-os`) → bespoke Rust Kubernetes substrate (`cloud-k8s`) → Rust cloud services → Rust oyatie products. Upstream k8s/containerd/Talos = transitional impls behind stable interfaces, cutover-gated (ADR-0510), never terminal.
8. Consolidate the 6 `consolidate/*` snapshot branches per HANDOFF §4 founder map (office-pilot conformance pattern).

## Decision basis (16-domain × 5-company research, 2 workflows, source-grounded)
- **IdP:** single-homed write CP + cell-replicated authn DP; offline credential verification (cell-local JWKS, never sync introspection); Oracle-style identity domains w/ primordial operator domain + sealed offline FIDO2 break-glass; passkeys v1; CAEP-style event revocation + Cedar issue-time cutoff. (AWS IAM/Entra/OCI 5/5 convergence.)
- **Authz:** Cedar **embedded in-process PDP** (4–11µs p99) in every service; central policy-store CP compiles/signs/pushes **content-addressed policy bundles** via the delivery fabric; tenant = Cedar namespace; `forbid` reserved for the structural tenant-isolation invariant; retire the hand-rolled `oya-policy-cedar-*` evaluator (ADR-0243 violation — two decision algorithms must never coexist).
- **Cells:** cluster-per-cell (Talos k8s), zero shared state, caps published in TPS/tenants/GB, ≤70% tested max; thinnest Rust router serving last-known-good content-addressed route tables.
- **Shell:** ONE platform-owned production Leptos shell (ADR-0393) owning all chrome + sole token brokerage; **build-time composition from buck2 monorepo** (no iframes — Google retired them; no module federation); design system as merge gate; the console is the **replacement operator surface for all retired CLIs**.
- **Control plane:** uniform resource-provider contract (shared Rust contract-test crate gating CI before service #2 diverges — ARM/AIP/CloudControl lesson); AIP-151 operation ledger; client-UUID idempotency; K8s-native actuation via reconcilers.
- **Observability:** OpenSLO files = single codegen source → multiwindow multi-burn-rate alerts → **automatic rollback triggers**; one wide-event per unit of work; static-threshold paging CI-rejected.
- **Delivery fabric:** presubmit latency SLO (~10–15min, ≥95% predictive) + exhaustive postsubmit w/ auto-bisect/auto-revert; Tide pessimistic merge queue first; code review = last human gate; shadow→warn→enforce for every new gate.
- **KMS:** AWS domain model (per-cell sealing roots → versioned per-tenant KEKs as wrapped tokens → per-object DEKs); one-way door in the type system (KEK plaintext only in mlock'd zeroize enclave process); rotation = key versions, never re-encryption; static stability (bounded-TTL DEK cache + bucket keys: reads never need live KMS); per-tenant KEK → quorum crypto-shred offboarding. Transitional custody = OpenBao behind owned interface (ADR-0510).
- **Persistence:** owned `oya-data` Rust SQL interface; PROVEN CRDB/TiKV-class transitional impl (ADR-0510 cutover-gated to W5 bespoke multi-Raft leader-per-range + HLC ClockSource trait + Pebble-class LSM); RLS tenant isolation; transactional outbox; CDC.
- **Storage (CAS):** four planes; metadata in oya-data (Tectonic keyspaces); strong read-after-write from first commit; BLAKE3 content addressing (dedup within tenant-KEK boundary); Object-Lock compliance-mode semantics at launch (audit/WORM sink).
- **Compute:** one shared fleet (Borg/Twine; per-team clusters rejected w/ 20–30% tax evidence); Cedar-enforced isolation ladder (first-party = hardened runc; tenant-influenced = Firecracker microVM); Talos zero-SSH validated.
- **Messaging:** Pulsar VALIDATED launch-primary behind thin owned Rust client; queue/stream/bus as 3 single-concern surfaces over ONE substrate; at-least-once + outbox = effectively-once; per-key ordering only.
- **Metering/Billing:** pipeline not query (at-least-once → idempotent dedup `(tenant,resource,dimension,usage_hour)` → hourly rating → monthly invoice); FOCUS 1.2 internal schema day one (+tenant_id/cell_id); versioned immutable price book; append-only identified line items; restatement-then-freeze close; double-entry subledger (debits=credits transactional invariant); 6h lateness window w/ explicit rejection; KR-VAT native.
- **Audit:** CloudEvents envelope + GCP-AuditLog-shaped payload as one libs/ crate emitted from tower middleware; admin stream always-on, **no kill switch (CI lint)**; audit-chain → CloudTrail-grade signed digest chain anchored in CAS WORM; verification = gate app/console surface (NOT a CLI per directive 6).
- **Network/DNS:** W0-critical DNS to Route-53 doctrine (shuffle-shard-of-4, serves from signed snapshots, runs with control plane dead, minimum-answer floor invariant); Katran-class Rust L4 (aya eBPF/XDP) + GFE-class Rust L7; config-compiler/dataplane split per network service.
- **Gateway/SSOT:** Smithy ARCHITECTURE in Rust (typed model + traits + emitters → OpenAPI/proto3/GraphQL/axum/tonic/clients as content-addressed outputs; OpenAPI emitted, never authored); gateway as Cedar PEP; two-stage rate limiting; one Check/Report substrate for quotas AND metering.

## Dogfood bootstrap order (circular-dependency-free, 10 steps)
0. Root-of-trust ceremony (Shamir M-of-N offline, dual-control safes) → 1. KMS unseal (OpenBao+PKCS#11; KMS storage = own local Raft group, NOT oya-data — breaks KMS↔DB cycle) → 2. Secrets + workload identity (CA mints SPIFFE mTLS certs at pod admission; fetch-fail = deploy-fail; zero static secrets) → 3. IdP (human/agent principals; primordial domain) → 4. Cedar PDP (embedded; all authz from here) → 5. Network/DNS (bootstrap from hand-signed seed snapshot; CAS later) → 6. Persistence (envelope-encrypted; separate single-Raft bootstrap metastore) → 7. CAS (metadata seeded from static-config instance; recursion-break ADR; DNS switches to CAS snapshots) → 8. Messaging (outbox relay; two loss classes) → 9. Audit/compliance (digest chain anchored in CAS WORM; logging-mode-first) → 10. Commercial/edge (metering/billing/gateway; internal chargeback for every service from this step). Hard rule as buck2 dep lint: no Tier-N service links a live client of Tier>N.

## Governance (every story)
Isolated worktree branch off `dev` → PR → single required context `oya-ci-required` green → review threads resolved → squash-merge. SSH-signed commits. NO `*.generated.json` add/modify in any diff (materializer script + diff-policy gate). No new CLI surfaces. Every service: slos/*.openslo.yaml before promotion (ADR-0130, live slo-coverage gate); K8s-native operational shape (CRD/operator/GitOps); clean architecture + API-first contracts before handlers; multispectrum evidence. Shadow→warn→enforce for new gates. Each story cites its hyperscaler precedent in its ADR/PR.

## Flagged founder decision points (carried, non-blocking)
- Cell boundary GTM posture (cell-within-region vs region-is-cell vs OCI realm) — research says GTM choice, not engineering.
- Audit retention posture (fixed ≥400d vs customer-configurable to 10y).
- Pricing/metering dimensions + price-sheet structure for the first module (needed before invoice rating goes live).
- Numeric W5 cutover trigger table per substrate (ADR-0510 format) — oya-data/CAS/SCM.
- HSM procurement timing (software-FIPS + OpenBao seal-wrap until hardware custody?).
- Firecracker adopt-vs-reimplement confirmation (research: adopt the proven Rust artifact behind a bespoke runtime shim).
- Policy-Zones logging-mode at FD-001 launch vs enforce for messenger/mail personal/professional split under KR/EU posture.
- oyago/oyapy transpiler destination path (HANDOFF §7.1).

## Story lane map
G01 serialized contract-lock → G02..G09 + G12 parallel worktree lanes → G10 integration fan-in → G11 continuous ratchet → G13 final gate.
