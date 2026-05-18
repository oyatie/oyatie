---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + council-design-system + ops-security
deciders: council-architecture, ops-security, axis-workflow, council-design-system, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS L2 + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0035, ADR-0037, ADR-0056, ADR-0065, ADR-0103, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-0164]
related_specs: [/specs/microservices/workflow-studio.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Studio Layer-A substrate change OR new node-library activation OR LLM-assist provider change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.5.34, A.8.2, A.8.3, A.8.4, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "OWASP ASVS L2 V1-V14"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 / §164.310 / §164.312 / §164.314 / §164.316"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234 §29-44"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: workflow-studio µservice

## Purpose

Identify, classify, and mitigate threats to the workflow-studio µservice's confidentiality, integrity, availability, and privacy posture. Studio is the visual authoring product — the largest Leptos webapp surface in oyatie + a load-bearing per-seat-billed product. A compromise here cascades to tenant business-logic confidentiality, per-seat billing integrity, and the canonical workflow_spec.v1.json supply-chain feeding the engine. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, and GDPR DPAs at first-tenant onboarding.

## Scope

### In-scope

All components introduced by the workflow-studio PRD + PHASE-01:

| Layer-A (adopted OSS / hyperscaler service) | Layer-B (oyatie-owned) |
|---|---|
| CDN (OCI CDN; static asset distribution; per-pack edge) | `oya-workflow-studio-visual-canvas-*` (9 crates) |
| WAF (OCI WAF; ingress in front of CDN + editor REST) | `oya-workflow-studio-dsl-emitter-*` (6 crates) |
| Postgres + Citus (editor session state, per-seat license attribution, draft persistence) | `oya-workflow-studio-dsl-loader-*` (6 crates) |
| Valkey (ephemeral CRDT collab state + WebSocket lease coordination) | `oya-workflow-studio-collab-crdt-*` (8 crates) |
| WebSocket gateway (axum-WS-based; CRDT op fan-out + debugger streaming) | `oya-workflow-studio-node-library-registry-*` (9 crates) |
| Object storage (signed per-pack node library binaries) | `oya-workflow-studio-jurisdiction-overlay-renderer-*` (5 crates) |
| LLM-assist bridge to foundry-providers (out-of-process; SDK boundary) | `oya-workflow-studio-replay-debugger-frontend-*` (6 crates) |
|  | `oya-workflow-studio-license-gate-cedar-*` (7 crates) |
|  | WASM bundle distributed via CDN |
|  | Per-tenant editor sessions |
|  | Per-pack node library descriptors + signatures |
|  | Per-tenant collab CRDT state |
|  | Audit-chain seals over saves + license-gate decisions |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS — owned by `cloud-k8s` threat model.
- Threats to CDN / WAF infrastructure layer — owned by `cloud-iac` µservice threat model; this document inherits.
- Threats to the Studio's downstream consumers (workflow-engine, ontology, foundry-providers, tenancy) — each owns its own threat model.
- Threats to the workflow specs themselves AT EXECUTION (engine concern; engine threat model covers).
- Threats to OpenBao secret manager — owned by `cloud-secrets`.
- Threats to LLM-assist provider's own model (prompt injection AT THE LLM, hallucination quality) — partially owned by foundry-providers' threat model; Studio's prompt-injection-into-spec-pipeline is in-scope here.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────────┐
│   Browser (Leptos WASM) ↔ user (tenant operator + business user + developer)   │
│         │                                                                      │
│         │ (HTTPS, OIDC, mTLS within cluster)                                   │
│         ▼                                                                      │
│  ┌─ CDN (OCI; per-pack edge) ─────────────────────────────────────────────┐    │
│  │  - Static assets (WASM bundles, node library descriptors, spec schema) │    │
│  │  - Per-tenant cache key                                                │    │
│  │  - SRI hashes for WASM chunks                                          │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│  ┌─ WAF (OCI) + Public ingress (Envoy/Istio) ────────────────────────────┐    │
│  │  - TLS + rate limit + DDoS + CSP enforcement                          │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────┼──────────────────────────────────────────────┘
                                  ▼
┌─ workflow-studio cluster (per-cell, per-pack) ─────────────────────────────────┐
│                                                                                │
│  Trust boundary 1: External → Studio ingress (REST + WebSocket)                │
│                                                                                │
│  ┌─ visual-canvas-rest ────┐  ┌─ collab-crdt-worker (WebSocket gateway) ───┐   │
│  │ OIDC tenant-scoped      │  │ - OIDC validated at WS upgrade             │   │
│  │ + Cedar license-gate    │  │ - tenant-binding rebound at each WS message│   │
│  └─────────────────────────┘  └────────────────────────────────────────────┘   │
│  ┌─ node-library-registry-rest ─┐  ┌─ license-gate-cedar (in-process) ─────┐  │
│  │ Signed library distribution │  │ Per-seat Cedar policy evaluation       │  │
│  └──────────────────────────────┘  └────────────────────────────────────────┘  │
│                                                                                │
│  Trust boundary 2: Per-tenant Citus partition + RLS                            │
│                                                                                │
│  ┌─ Postgres + Citus (editor sessions + per-seat license attribution) ──┐     │
│  │  - tenant_id partition + RLS                                          │     │
│  │  - per-tenant connection pool                                         │     │
│  └────────────────────────────────────────────────────────────────────────┘    │
│  ┌─ Valkey (ephemeral CRDT) ─┐ ┌─ Object storage (node libraries) ───────┐     │
│  │ tenant-prefixed key       │ │ pack-scoped bucket                       │     │
│  └───────────────────────────┘ └──────────────────────────────────────────┘    │
│                                                                                │
│  Trust boundary 3: WS gateway → CRDT op fan-out (per-definition lease)         │
│                                                                                │
│  ┌─ WebSocket gateway lease coordinator ────────────────────────────────┐     │
│  │  - one WS pod owns one (tenant, definition_id) lease via Valkey       │     │
│  │  - cross-tenant collab forbidden by tenant-binding on connect        │     │
│  │  - per-definition message routing                                    │     │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                                                                │
│  Trust boundary 4: Studio → engine SDK (cross-µservice)                        │
│                                                                                │
│  ┌─ workflow-engine SDK calls ──────────────────────────────────────────┐     │
│  │  - mTLS + SPIFFE identity (workflow-studio-rest)                     │     │
│  │  - tenant context passed via header; engine server-stamps on receive │     │
│  │  - spec submission carries Ed25519 signature                          │     │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                                                                │
│  Trust boundary 5: Studio → LLM-assist (foundry-providers SDK)                 │
│                                                                                │
│  ┌─ foundry-providers SDK (LLM bridge) ────────────────────────────────┐      │
│  │  - mTLS + SPIFFE identity                                            │      │
│  │  - tenant-prompt audit-emitted (90d retention)                       │      │
│  │  - LLM-completion validated against spec schema before user-surfaced │      │
│  │  - Prompt-injection signature scrubbed before LLM submission         │      │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  Trust boundary 6: Audit chain emission                                        │
│                                                                                │
│  ┌─ audit-chain-emitter (in-process; signs save + license events) ─────┐     │
│  │  - Ed25519 signing key from OpenBao (rotated 90d)                    │     │
│  │  - Merkle-chain over per-tenant per-definition save sequence         │     │
│  └────────────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **External → Studio ingress** (CDN+WAF + TLS + OIDC + CSP).
2. **Per-tenant Citus partition + RLS** (the load-bearing isolation boundary for sessions/licenses).
3. **WS gateway → per-definition lease** (cross-tenant collab forbidden).
4. **Studio → engine SDK** (mTLS + SPIFFE).
5. **Studio → LLM-assist SDK** (prompt-injection scrub + LLM output validation).
6. **Audit-chain emission** (Ed25519 signing; non-repudiation).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Editor session state (active drafts, cursor, viewport) | `BEHAVIORAL_TENANT_PRODUCT` | High | 30d hot (Postgres) + Valkey ephemeral while active | Postgres + Valkey |
| Spec drafts (unsigned, pre-submit) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_QUASI_IDENTIFIER` | Medium-High | 30d hot until tenant promotes or discards | Postgres |
| Collab CRDT op stream | `BEHAVIORAL_TENANT_PRODUCT` | High | Valkey ephemeral while session active; sealed deltas to Postgres on save | Valkey + Postgres |
| Per-seat license attribution rows | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | 24mo cold (Postgres) + audit-chain seal | Postgres + audit-chain µservice |
| Node library descriptors + signatures | `INTERNAL_ONLY` + `AUDIT` (signature) | Low-Medium | append-only git history + per-pack signed distribution | Object storage + git |
| WASM bundle chunks + SRI hashes | `INTERNAL_ONLY` | Low | per-release; previous versions retained 90d | CDN + repo |
| LLM-assist prompts (tenant-issued) | `BEHAVIORAL_TENANT_PRODUCT` + occasionally `PII_IDENTIFYING` (when tenant prose mentions user data) | High | 90d hot for audit; aggressive purge after | Postgres (audit) |
| LLM-assist completions (returned drafts) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 90d hot for audit | Postgres |
| WebSocket session secrets (per-connection token) | `SECRET` | Critical | ephemeral; TTL ≤ 1h | OpenBao + in-memory |
| Cedar policy fragments (per-tenant license claims) | `INTERNAL_ONLY` + occasionally `SECRET` (entitlement signature) | High | git-versioned; per-tenant entitlement in OpenBao | git + OpenBao |
| Audit-chain Ed25519 signing keys | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Editor REST SDK API keys | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Hashed tenant ID (used in CDN cache key + topic namespace) | `SENSITIVE_PIPA_ART23` | High | salted; rotation 12mo | OpenBao tenant-resolver |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant business user (human, in browser) | Untrusted external | OIDC + MFA | Open editor; drag-drop authoring; save own tenant's specs; collab |
| External tenant developer (human, in browser) | Untrusted external | OIDC + MFA | DSL view; diff/PR UI; git-backed authoring |
| External tenant vertical specialist (human, in browser) | Untrusted external | OIDC + MFA | Switch jurisdiction overlay; export evidence |
| External tenant agentic developer role (programmatic) | Untrusted external | Per-tenant SDK API key | Submit spec via API; LLM-assist invocation |
| Studio REST handler (in-process) | Trusted internal | OpenBao-issued ServiceAccount + SPIFFE | Read/write editor session state; submit specs to engine |
| WebSocket gateway worker (in-process) | Trusted internal | SPIFFE | Fan-out CRDT ops within per-tenant per-definition lease |
| Node-library distribution (in-cluster process) | Trusted internal | SPIFFE | Sign + publish per-pack node libraries to CDN/object-storage |
| LLM-assist bridge (cross-µservice via SDK) | Semi-trusted internal | mTLS + SPIFFE | Submit tenant prompts to foundry-providers; receive completions |
| Workflow-engine spec-store (cross-µservice via SDK) | Semi-trusted internal | mTLS + SPIFFE | Receive Studio's spec submissions; verify signature |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Read Studio code at PR-review time; refuse changes violating gates |
| Council operators (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin operations on Studio config; emergency override (2-person rule + audit) |
| External auditor | Read-only external, time-boxed | OIDC + MFA + JIT short-lived token | Read editor session audit trail; cannot pivot to tenant draft contents |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; XSS attempts on editor surface |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness (e.g., node library compromise) |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure Cedar policy or LLM-assist prompt template (mitigated by PR review + LEAN gates) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor; mitigated by least-privilege + audit-chain + 2-person rule |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Tenant-A user opens an editor session and impersonates tenant-B via cookie/token manipulation**
- Asset: editor session boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - OIDC token bound to authenticated tenant; engine validates `tenant_id` claim on every REST call.
  - Server-side stamping: editor REST overrides any client-supplied tenant_id with the OIDC claim.
  - WebSocket re-validates tenant_id on every message dispatch (no trust of long-held connection).
  - Mismatch attempts return 401 + audit-emit `studio_tenant_spoofing_attempt`.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Forged spec signature: malicious actor submits spec to engine claiming Studio authorship via leaked SPIFFE**
- Asset: Studio SPIFFE identity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SPIFFE identity bound to pod; cannot be used outside cluster.
  - Token rotation 24h.
  - Engine verifies submission carries valid Ed25519 signature from Studio-issued key, not just SPIFFE identity (defense-in-depth).
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.17, A.8.7

**T-S-03 — WebSocket session token replay: attacker captures WS upgrade token and replays**
- Asset: WebSocket session
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - WS session token bound to (tenant_id, user_id, definition_id) tuple at upgrade.
  - TTL ≤ 1h; rebinding required after.
  - Idle disconnect after 5min of no client message.
  - Replay-attack window protected by per-message nonce + monotonic sequence counter.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15

**T-S-04 — Node library publisher impersonation (attacker pushes a malicious node library claiming pack-eu authority)**
- Asset: node library publishing identity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Per-pack node library signing key in OpenBao; rotation 90d.
  - Publishing requires 2-person rule + signed commit.
  - Studio refuses libraries whose signing key is revoked OR not in allowed-publisher set.
  - CDN edge SRI hashes prevent tampering in transit.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.32, A.8.32, A.8.33

**T-S-05 — LLM-assist completion forged (attacker injects completion via prompt-injection in tenant prose)**
- Asset: LLM-assist output
- Likelihood: H (prompt injection is common) / Impact: M / Risk: **M-H**
- Mitigations:
  - Tenant prose scrubbed for prompt-injection markers before LLM submission (basic regex + content-policy classifier).
  - LLM completion ALWAYS validated against canonical spec schema before user-surfaced.
  - User explicitly accepts LLM-drafted spec before save; no auto-submission.
  - Anti-pattern: bypassing schema validation pipeline is forbidden; LEAN check `oya-governance-llm-assist-validation-required`.
- Owner: axis-workflow + ops-security
- Residual: M (LLM-assist is inherently advisory; tenant approval is the load-bearing control)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.28; OWASP Top 10 LLM A01 prompt injection

### Tampering (T)

**T-T-01 — Spec draft tampering during edit (CRDT op forgery)**
- Asset: collab CRDT op stream
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - CRDT ops carry sender authenticated identity; server-side stamp on receive.
  - WS message integrity via per-message HMAC over (session_token, sequence_num, payload).
  - Tampered ops reject + audit-emit `studio_crdt_op_tampering_attempt`.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.8.32

**T-T-02 — Editor session state corruption via concurrent write race**
- Asset: Postgres EditorSession row
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Optimistic concurrency check (`version` column) on every session update.
  - Single-writer invariant: one WS gateway pod owns active editor session via Valkey lease.
  - Lease TTL ≤ 5min.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC8.1; ISO 27001 A.8.32

**T-T-03 — Node library binary tampering at CDN edge**
- Asset: node library distribution at CDN
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Per-pack Ed25519 signature on every library descriptor; Studio verifies on load.
  - SRI hashes for WASM chunks delivered via CDN.
  - LEAN check `oya-governance-node-library-signature-verification` validates on every PR build.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1, CC8.1; ISO 27001 A.5.28, A.8.32

**T-T-04 — Jurisdiction overlay tampering (forge `kr@preview` overlay to render forbidden config)**
- Asset: jurisdiction overlay descriptors
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Overlays are git-versioned; signed-commit policy enforced.
  - Overlay resolver verifies overlay descriptor signature before render.
  - State-machine refuses overlays whose version-SHA doesn't match the spec's pinned overlay version.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC8.1; ISO 27001 A.8.32

**T-T-05 — Per-seat license attribution tampering (forge low-seat-count to evade billing)**
- Asset: SeatLicense row in Postgres
- Likelihood: L (requires Postgres-level write) / Impact: H / Risk: **M**
- Mitigations:
  - Postgres row append-only (audit table); UPDATE/DELETE refused by trigger.
  - Each row Ed25519-signed at insert (insert-time signature).
  - Tampering detected on next aggregation; audit-chain seal gap fires Sev-1 alert.
- Owner: ops-security + axis-workflow + tenancy
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1, CC8.1; ISO 27001 A.5.28, A.8.32

**T-T-06 — WASM bundle tampering: attacker injects malicious code into a WASM chunk before CDN ingress**
- Asset: WASM bundle integrity
- Likelihood: L / Impact: H (RCE in tenant browser) / Risk: **M**
- Mitigations:
  - SRI hash per chunk in HTML; browser refuses mismatched chunk.
  - Per-release SBOM published; `cargo deny` + Trivy scan at build time.
  - CDN-side immutability lock on uploaded bundle.
  - LEAN check `oya-governance-wasm-bundle-sri` validates every chunk has SRI.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.28, A.8.7, A.8.32

### Repudiation (R)

**T-R-01 — Tenant operator denies authorship of a workflow definition save**
- Asset: save event
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every save requires OIDC-bound identity + recorded in audit-chain with actor.
  - Save event signed (Ed25519) over (tenant_id, spec_id, version_sha, author_oidc_sub, timestamp).
  - 2-person rule for production-tier saves (configurable).
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15

**T-R-02 — Tenant denies LLM-assist consent (claims they didn't authorize prompt to leave tenant boundary)**
- Asset: LLM-assist prompt
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - LLM-assist invocation requires explicit per-session opt-in click; consent event audit-emitted.
  - Foundry-providers tenant DPA carries LLM-routing disclosure.
- Owner: council-privacy + axis-workflow
- Residual: L
- Frameworks: GDPR Art. 6(1)(a) consent; SOC 2 CC4.1

**T-R-03 — Collab conflict resolution denied (one user claims they didn't author the surviving op)**
- Asset: CRDT op stream history
- Likelihood: L / Impact: L-M / Risk: **L**
- Mitigations:
  - Every CRDT op carries author identity; conflict resolution UI surfaces the author of each branch.
  - Audit-chain seal includes conflict-resolution decision (user selected branch A or B).
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.28

### Information Disclosure (I)

**T-I-01 — Cross-tenant editor session leak via Citus partition bypass**
- Asset: EditorSession Postgres rows
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Citus partition + Row-Level Security (RLS) BOTH enforce tenant isolation; defense-in-depth.
  - Per-tenant Postgres connection pool — connection's session variable carries tenant_id; RLS predicate reads it.
  - LEAN check `oya-governance-citus-rls-enforced` validates schema + policies on every PR.
  - Per-tenant query audit via Postgres extension `pgaudit`.
  - Penetration test against tenant boundary annually.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32

**T-I-02 — XSS in node config panel injecting attacker JS via tenant-rendered field values**
- Asset: Studio editor visual surface
- Likelihood: H (XSS is the #1 webapp threat) / Impact: H / Risk: **H**
- Mitigations:
  - All rendered text goes through Leptos virtual-DOM (text nodes only; no `innerHTML`).
  - Strict CSP: `script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`; no `unsafe-inline`; no `unsafe-eval`.
  - `Trusted Types` enforced (`require-trusted-types-for 'script'` directive).
  - DOMPurify-equivalent sanitization for any HTML-permitted node config (e.g., embedded markdown previews).
  - LEAN check `oya-governance-xss-vector-scan` greps for `innerHTML` / `outerHTML` / `dangerouslySetInnerHTML` patterns.
  - Annual XSS pen-test against Studio.
- Owner: axis-workflow + ops-security + council-design-system
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.28; OWASP Top 10 A03 injection

**T-I-03 — DSL injection: tenant pastes crafted JSON into developer DSL view; injected fields bypass schema**
- Asset: workflow_spec.v1.json content
- Likelihood: M / Impact: M / Risk: **M-H**
- Mitigations:
  - Server-side spec schema validation on every save (canonical JSON-Schema 2020-12).
  - Client-side validation is advisory only; never load-bearing.
  - Unknown / unexpected fields rejected (additionalProperties: false on every nested object).
  - LEAN check `oya-governance-workflow-spec-schema-strict` validates spec parser at PR-time.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-I-04 — Collab op leak: subscriber on definition A receives ops from definition B (lease misrouting)**
- Asset: WebSocket CRDT op delivery
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - WS gateway lease is keyed on (tenant_id, definition_id) tuple; routing by consistent-hash.
  - Server-side filter on every outbound message: (subscriber.tenant_id, subscriber.definition_id) == (op.tenant_id, op.definition_id).
  - Cross-definition / cross-tenant delivery attempt audit-emits Sev-1 alert.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3; GDPR Art. 25, Art. 32

**T-I-05 — LLM-assist prompt leakage: tenant prose contains PII; prompt sent to foundry-providers; LLM provider logs**
- Asset: LLM-assist prompt content
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations:
  - SDK PII redactor strips obvious PII (emails / phone numbers / SSNs / IDs) from prose before foundry-providers submission.
  - Tenant onboarding discloses LLM-routing; per-tenant LLM provider choice (BYO-LLM available); zero-retention LLM models preferred.
  - Foundry-providers tenant DPA includes upstream-LLM disclosure clause.
  - Audit-chain seal on every LLM-assist invocation; tenant can DSR-revoke later.
- Owner: council-privacy + axis-workflow + foundry-providers
- Residual: M (PII redactor is heuristic; some PII may still leak; tenant disclosure is the proportionate control)
- Frameworks: GDPR Art. 6, 25, 32; KR PIPA Art. 29; HIPAA §164.502(b) (minimum-necessary)

**T-I-06 — Replay-debugger leaks step payload across tenant boundaries**
- Asset: replay-debugger-frontend stream
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Engine replay-debugger-backend already enforces tenant scope via Cedar (engine's threat model T-I-04).
  - Studio replay-debugger-frontend additionally enforces tenant binding on incoming WS stream.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3; GDPR Art. 32

**T-I-07 — Per-tenant branding mid-render injection (CSS or script via tenant-uploaded branding asset)**
- Asset: Studio render surface
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - **Per-tenant branding mid-render is FORBIDDEN by anti-pattern policy** (per `/specs/microservices/workflow-studio.json` §anti_patterns).
  - LEAN check `oya-governance-no-tenant-branding-mid-render` forbids any tenant-uploaded asset rendered in same DOM tree as canvas.
  - Post-GA marketplace branding (if introduced) restricted to iframed sandboxes with separate CSP.
- Owner: ops-security + council-design-system
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.7

**T-I-08 — CDN cache pollution: tenant-A's editor state cached under tenant-B's key**
- Asset: CDN edge cache
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Per-tenant CDN cache key: `(tenant_hash, pack, version)` always included in cache-key; verified on every CDN-Studio response.
  - Static assets (WASM, schema, node library) are tenant-agnostic; tenant-specific content NEVER cached at CDN edge.
  - LEAN check `oya-governance-cdn-cache-key-tenant-isolated` validates cache configuration.
- Owner: cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3

### Denial of Service (D)

**T-D-01 — Per-tenant editor session flood overwhelms WS gateway**
- Asset: WS gateway capacity
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-tenant active-session cap (default 50); refuse `429` above cap.
  - Fair-share scheduling: one tenant cannot starve another via spam editor opens.
  - HPA on WS gateway pods; min 3 replicas, max 100.
  - Pre-warmed pool of 5 standby pods; cold-start ≤ 1s.
- Owner: ops-sre-reliability + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Collab desync flood: malicious user spams CRDT ops, exhausting Valkey**
- Asset: Valkey ephemeral state
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-(tenant, user) WS message rate limit (default 100 ops/sec); excess refused + connection throttled.
  - Per-tenant Valkey memory cap.
  - Slow-client quarantine: client > 10× rate-limit threshold → disconnect after 60s + tenant notified.
- Owner: axis-workflow + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6, A.8.14

**T-D-03 — CDN purge gap: stale WASM bundle served after security-patch release**
- Asset: CDN edge cache freshness
- Likelihood: M / Impact: M (vulnerable code serving) / Risk: **M**
- Mitigations:
  - CDN purge after every release; purge propagation SLI ≤ 60s p99.
  - Versioned WASM bundle path (`/v1.2.3/canvas.wasm`); old path returns 410 Gone after deprecation window.
  - Browser-side version pin: HTML carries `<meta data-studio-version="...">`; mismatch triggers reload.
- Owner: cloud-iac + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6

**T-D-04 — LLM-assist timeout cascade: foundry-providers slow → Studio editor unresponsive**
- Asset: LLM-assist bridge
- Likelihood: M / Impact: M (LLM-assist is non-critical) / Risk: **M**
- Mitigations:
  - LLM-assist requests run in background; editor UX unaffected by LLM latency.
  - Timeout 10s server-side; circuit breaker after 3 consecutive timeouts → LLM-assist temporarily disabled per-tenant.
  - User sees "LLM-assist degraded; please retry or proceed manually" banner.
- Owner: axis-workflow + foundry-providers
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6

**T-D-05 — Postgres lock contention on hot definition (10+ users editing same doc, all saving simultaneously)**
- Asset: Postgres definition row
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Save path goes through engine spec-store; Studio doesn't lock Postgres row directly.
  - Local edit buffer + CRDT op stream absorbs concurrent edits; engine save is single transaction.
  - Optimistic concurrency on EditorSession row; retry with backoff on conflict.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

**T-D-06 — Jurisdiction-overlay drift: stale overlay descriptor served, mismatching engine's view**
- Asset: jurisdiction overlay descriptor consistency
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Overlay descriptor versioned + signed; Studio verifies on every load.
  - Engine + Studio share `overlay_version_sha`; mismatch surfaces "overlay version mismatch; refresh editor" UI.
  - Quarterly drift audit: synthetic test loads each (pack, overlay-version) and verifies round-trip.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

**T-D-07 — License-gate failure-open: Cedar evaluator crashes → editor opens without license check**
- Asset: license-gate-cedar evaluator
- Likelihood: L / Impact: H (billing-bypass) / Risk: **M**
- Mitigations:
  - **Default-deny fail-closed**: Cedar evaluator failure refuses editor open with 503 Service Unavailable + audit-emit.
  - Cedar evaluation cached per-(tenant, principal) for 30s; cache failure → re-evaluate, not fail-open.
  - Health probe on Cedar evaluator; HPA + circuit-breaker.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.15, A.8.4

**T-D-08 — WebSocket gateway restart drops active editor sessions**
- Asset: active editor sessions during deploy
- Likelihood: M / Impact: M (UX-only; no data loss because CRDT state persisted) / Risk: **M**
- Mitigations:
  - Rolling deploy of WS gateway; lease handoff during rolling restart.
  - Browser client auto-reconnects on WS close (exponential backoff).
  - Local edit buffer preserves unsent changes during disconnect.
- Owner: ops-sre-reliability + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

### Elevation of Privilege (E)

**T-E-01 — XSS escalates to session token theft + cross-tenant editing**
- Asset: editor session token
- Likelihood: M (any XSS → session theft) / Impact: H / Risk: **H**
- Mitigations:
  - All XSS mitigations from T-I-02 apply.
  - Session token is `HttpOnly` cookie; JS cannot read.
  - Sensitive WS operations re-validate OIDC bearer token from in-memory state (not cookie); XSS can't extract.
- Owner: ops-security
- Residual: L (depends on XSS prevention floor)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.7, A.8.28

**T-E-02 — Operator JIT elevation abused to read tenant draft contents**
- Asset: operator-override path
- Likelihood: L (insider-malicious) / Impact: H / Risk: **M**
- Mitigations:
  - 2-person rule required for draft read on behalf of tenant; audit-chain emission.
  - Read pattern detection: > 5 cross-tenant draft reads/min triggers anomaly alert.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4

**T-E-03 — Cedar policy bypass via crafted entitlement claim**
- Asset: license-gate-cedar evaluator
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4 used; field length bounded; fuzzing at CI.
  - Entitlement claims signed by tenancy µservice; Studio verifies signature before Cedar evaluation.
  - Server-side stamping of (tenant_id, principal_id); client cannot supply.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.28

**T-E-04 — Node library plugin executes arbitrary WASM with elevated capabilities**
- Asset: WASM plugin sandbox boundary
- Likelihood: L (Wasmtime mature) / Impact: H / Risk: **M**
- Mitigations:
  - Node libraries are descriptors only (declarative configuration), NOT executable WASM at Studio level.
  - Engine ADR-0037 plugin substrate is engine concern; Studio doesn't execute plugin code.
  - LEAN check `oya-governance-editor-execution-forbidden` validates Studio crates contain no Wasmtime/exec primitives.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.4, A.8.28

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Editor session events + LLM-assist prompts | Multiple authoring sessions can link to single end-user via prompt patterns. | Per-tenant session scoping; LLM prompt redaction; correlation IDs tenant-scoped. | M |
| T-L-02 | Identifiability | Hashed tenant ID in CDN cache key | sha256(tenant_id)[..16] may be re-identifiable via auxiliary data. | Salted hash; salt rotated 12mo. | L |
| T-L-03 | Non-repudiation | Spec authorship | Tenant may deny authorship. | Signed commits; per-save audit-chain seal. | L |
| T-L-04 | Detectability | Editor session timing patterns | Tenant authoring cadence correlates with business events. | Expected; behavioral; consent at onboarding. | M |
| T-L-05 | Disclosure | LLM-assist prompt routing to third-party LLM provider | Tenant prose may reveal end-user data to LLM provider. | PII redactor + tenant disclosure + BYO-LLM option + zero-retention models. | M |
| T-L-06 | Unawareness | End-user unaware authoring affects them | Tenant's end-user may not know their data shaped a workflow definition. | Tenant DPA upstream-disclosure clause. | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure cascade across drafts | End-user erasure across drafts / LLM prompts / editor sessions. | DSR cascade per `oya-dsr-cascade-runner`; 30d SLA. | M (best-effort) |
| T-L-08 | Non-compliance | Per-seat license attribution retention beyond consent | License-row retention 24mo may exceed end-user consent. | Retention bounded; audit-chain provides forensic vs operational distinction. | L |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Per-tenant Citus partition + RLS | Preventive | axis-workflow | `oya-governance-citus-rls-enforced` lane |
| OIDC tenant-scope binding | Preventive | ops-security | OIDC audit log |
| Server-side tenant_id stamping (WS + REST) | Preventive | axis-workflow | LEAN check on rest crate |
| Ed25519 spec signature at save | Preventive | axis-workflow | `oya-governance-workflow-spec-signature-verification` lane (engine half) |
| Ed25519 audit-chain seals | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Strict CSP + Trusted Types | Preventive (XSS) | council-design-system + ops-security | LEAN check + headers regression test |
| SRI hashes on WASM chunks | Preventive | axis-workflow | `oya-governance-wasm-bundle-sri` lane |
| Per-pack node library Ed25519 signing | Preventive | ops-security | `oya-governance-node-library-signature-verification` lane |
| CRDT op HMAC + sequence counter | Preventive (replay) | axis-workflow | CRDT regression tests |
| Single-writer Valkey lease per editor session | Preventive | axis-workflow | concurrent-writer integration test |
| Per-tenant Studio session rate limit | Preventive (DoS) | axis-workflow | Studio REST metrics |
| LLM-assist PII redactor | Preventive | axis-workflow + council-privacy | quarterly synthetic-PII drill |
| LLM-assist completion schema validation | Preventive | axis-workflow | `oya-governance-llm-assist-validation-required` lane |
| Cedar per-seat license-gate (default-deny) | Preventive (billing) | ops-security + tenancy | per-IP integration test |
| 2-person rule for operator overrides | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| Cross-tenant collab forbidden | Preventive | axis-workflow | LEAN check on collab-crdt-worker |
| Per-tenant CDN cache key | Preventive | cloud-iac | `oya-governance-cdn-cache-key-tenant-isolated` lane |
| No tenant-branding-mid-render | Preventive | council-design-system | `oya-governance-no-tenant-branding-mid-render` lane |
| Editor-execution-forbidden | Preventive | axis-workflow | `oya-governance-editor-execution-forbidden` lane |
| Network policy: Studio → engine / ontology / foundry-providers / tenancy SDKs only | Preventive | ops-sre-reliability | Kubernetes NetworkPolicy review |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-05 (LLM-assist prompt PII leakage) | M | PII redactor is heuristic; tenant disclosure is the proportionate control. | Quarterly |
| T-S-05 (prompt injection bypass) | M | LLM-assist is advisory; tenant explicit-accept is load-bearing. | Quarterly |
| T-L-01 (linkability across sessions) | M | Inherent to editor authoring tracing. | Annually |
| T-L-04 (detectability via timing) | M | Tenant business reality. | Annually |
| T-L-05 (LLM provider routing) | M | Tenant disclosure + BYO-LLM option. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant joint-controllership. | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Bounded by retention windows. | Annually |

Sign-off:

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`
- council-design-system: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

- KR PIPA Art. 23 (sensitive PII): hashed tenant IDs sensitive when paired with auxiliary; salt-rotation in T-L-02 satisfies Art. 23.
- KR PIPA Art. 29 (technical safeguards): every T-*-NN mitigation maps to one of 12 prescribed safeguards.
- KR PIPA Art. 23-2 (cross-border): KR tenant data stays in pack-kr cluster (LLM-assist routes to KR-resident LLM provider only).
- KR-ISMS-P §2.7 (접근통제) + §2.5 (인적보안): per-seat Cedar + 2-person rule map directly.

### pack-us-healthcare (HIPAA)

- HIPAA §164.312(a)(1) (access control): per-tenant Citus + RLS + Ed25519 audit-chain.
- HIPAA §164.312(b) (audit controls): editor save audit-chain emission; retention ≥ 6y for pack-us-healthcare definitions.
- HIPAA §164.502 (minimum-necessary): SDK redactor for LLM-assist; data_class markers on visual canvas warn before save.
- HIPAA §164.504(e) (Business Associate Agreement): oyatie operates as BA for HIPAA-scope tenants; BAA at `microservices/workflow-studio/legal/baa-template.md`.

### pack-eu (GDPR + EDPB + NIS2)

- GDPR Art. 25: every mitigation mapped to Schrems-II-compatible TOM.
- GDPR Art. 35 DPIA: this threat model + the DPIA at `dpia.md` satisfy DPIA for high-risk processing.
- GDPR Art. 32: every T-*-NN mitigation contributes to Art. 32 security posture.
- GDPR Arts. 44-50: pack-eu cluster EU-resident; LLM-assist routes EU-resident LLM provider when in pack-eu.
- NIS2 2022/2555: when oyatie crosses thresholds, 24h/72h/1mo timelines apply; `incident-response.md` reflects.
- AI Act 2024 (high-risk LLM systems): LLM-assist invocation logged + auditable.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections at `regional-packs/<pack>/workflow-studio-overlay.md`; each follows same structure with local PII law's articles + local cybersecurity-framework controls; maps to this document's threat IDs via cross-mapping in `compliance.md`.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x through CC9.x covered as cited inline | `microservices/workflow-studio/compliance.md` |
| ISO 27001:2022 | Annex A.5-A.8 controls cited inline | `microservices/workflow-studio/compliance.md` |
| GDPR | Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/workflow-studio/dpia.md` + `compliance.md` |
| OWASP ASVS L2 | V1 V2 V3 V4 V5 V7 V8 V9 V11 V12 V13 V14 covered | `microservices/workflow-studio/compliance.md` |

## Re-review Triggers

- Any change to the trust boundary diagram (new boundary, removed boundary, modified actor).
- Any Layer-A version upgrade (CDN / WAF / Postgres / Valkey / WebSocket gateway library) where upstream release notes mention security fixes.
- New node library activation (each new library is a new code-distribution surface).
- LLM-assist provider change.
- New pack activation.
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 or Sev-2 incident in workflow-studio).
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0035 (Bominal): Workflow engine (engine context for Studio cross-µservice).
- ADR-0037 (Bominal): Plugin substrate; engine concern.
- ADR-0056: BNF v4.1.
- ADR-0065: Leptos for browser UI; defines the WASM target.
- ADR-0103 (Bominal): Workflow hexagonal migration; inherited.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- ADR-0140: Cedar policy enforcement.
- ADR-0164 (Bominal): Workflow canonical spec format.
- `microservices/workflow-studio/PRD.md`.
- `microservices/workflow-studio/dpia.md`.
- `microservices/workflow-studio/compliance.md`.
- `/specs/microservices/workflow-studio.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS v4.0.
- OWASP Top 10 LLM Applications (2023).
- NIST SP 800-154.
