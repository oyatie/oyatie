---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-sheets + council-design-system + ops-security
deciders: council-architecture, ops-security, axis-sheets, council-design-system, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS L2 + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0035, ADR-0056, ADR-0065, ADR-0103, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-SHEETS-0001, ADR-SHEETS-0002, ADR-SHEETS-0003, ADR-SHEETS-0004, ADR-SHEETS-0005, ADR-SHEETS-0006, ADR-SHEETS-0007]
related_specs: [/specs/microservices/sheets.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Sheets Layer-A substrate change OR new function-library release OR AI-formula provider change OR XLSX importer version bump
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.5.34, A.8.2, A.8.3, A.8.4, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "OWASP ASVS L2 V1-V14"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 / §164.310 / §164.312 / §164.314 / §164.316"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555", "EU AI Act 2024 (when AI-formula in regulated workflow)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234 §29-44"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: sheets µservice

## Purpose

Identify, classify, and mitigate threats to the sheets µservice's confidentiality, integrity, availability, and privacy posture. Sheets is the spreadsheet + structured-data hero product. A compromise here cascades to tenant financial-modelling confidentiality, per-seat billing integrity, the XLSX import supply-chain feeding tenant browsers, and the canonical cell graph feeding the cell µservice + downstream embed surfaces (docs + slides). This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, and GDPR DPAs at first-tenant onboarding.

## Scope

### In-scope

All components introduced by the sheets PRD + PHASE-01:

| Layer-A (adopted OSS / hyperscaler service) | Layer-B (oyatie-owned) |
|---|---|
| CDN (OCI CDN; static asset distribution; per-pack edge) | `oya-sheets-cell-grid-*` |
| WAF (OCI WAF; ingress in front of CDN + editor REST) | `oya-sheets-formula-engine-*` |
| Postgres + Citus (workbook metadata + cell rows + per-seat license + sharing/ACL + comments + version-history pointers) | `oya-sheets-recalc-engine-*` |
| Valkey (ephemeral CRDT collab state + WebSocket lease coordination + recalc-progress streaming) | `oya-sheets-formatting-*` |
| WebSocket gateway (axum-WS-based; CRDT op + cursor + recalc-progress fan-out) | `oya-sheets-pivot-tables-*` |
| S3 (workbook snapshots + version-history binaries; per-pack bucket) | `oya-sheets-charts-*` |
| OCI Object Storage (Arrow/Parquet large-sheet blocks; per-(tenant, workbook, sheet) key) | `oya-sheets-data-validation-*` |
| Apache Arrow 18.x (columnar in-memory format for large-sheet analytical recalc) | `oya-sheets-collab-crdt-*` (Loro 1.x adapter) |
| Parquet 18.x (snapshot serialisation for cold large-sheet blocks) | `oya-sheets-import-export-*` (gVisor sandboxed XLSX pipeline) |
| Loro 1.x CRDT (collab merge engine; aligned with workflow-studio ADR-WS-0001) | `oya-sheets-large-sheet-storage-*` |
| calamine 0.26 (XLSX read; sandboxed) | `oya-sheets-sharing-acl-*` |
| rust_xlsxwriter 0.79 (XLSX write; sandboxed) | `oya-sheets-comments-*` |
| ClamAV + OPSWAT MetaDefender (XLSX upload AV scan; defense-in-depth) | `oya-sheets-version-history-*` |
| gVisor user-mode sandbox (XLSX import/export pipeline isolation) | `oya-sheets-named-ranges-*` |
| foundry-runtime ML inference bridge (AI-formula + smart-fill + anomaly detection) | `oya-sheets-ai-formula-*` |
| (custom Leptos canvas chart renderer; in-process) | `oya-sheets-connected-sheets-*` |
|  | `oya-sheets-trigger-bridge-*` |
|  | `oya-sheets-embed-bridge-*` |
|  | `oya-sheets-license-gate-cedar-*` |
|  | WASM bundle distributed via CDN |
|  | Per-tenant editor sessions |
|  | Per-tenant workbook + cell + comment + version state |
|  | Per-range ACL Cedar policy fragments |
|  | Audit-chain seals over cell-edits + sharing changes + formula-engine-upgrades + license-gate decisions |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS — owned by `cloud-k8s` threat model.
- Threats to CDN / WAF infrastructure layer — owned by `cloud-iac` µservice threat model; this document inherits.
- Threats to the Sheets's downstream consumers (cell, ontology, foundry-runtime, tenancy, workflow-engine, docs, slides) — each owns its own threat model.
- Threats to the cell µservice's per-workbook cell substrate at storage — cell threat model covers.
- Threats to OpenBao secret manager — owned by `cloud-secrets`.
- Threats to AI-formula provider's own model (prompt injection AT THE LLM, hallucination quality) — partially owned by foundry-runtime's threat model; Sheets's prompt-injection-into-formula-pipeline is in-scope here.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────────┐
│   Browser (Leptos WASM) ↔ user (tenant operator + analyst + financial modeller) │
│         │                                                                      │
│         │ (HTTPS, OIDC, mTLS within cluster)                                   │
│         ▼                                                                      │
│  ┌─ CDN (OCI; per-pack edge) ─────────────────────────────────────────────┐    │
│  │  - Static assets (WASM bundles, design-system primitives, spec schema) │    │
│  │  - Per-tenant cache key                                                │    │
│  │  - SRI hashes for WASM chunks                                          │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│  ┌─ WAF (OCI) + Public ingress (Envoy/Istio) ────────────────────────────┐    │
│  │  - TLS + rate limit + DDoS + CSP enforcement                          │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────┼──────────────────────────────────────────────┘
                                  ▼
┌─ sheets cluster (per-cell, per-pack) ─────────────────────────────────────────┐
│                                                                                │
│  Trust boundary 1: External → Sheets ingress (REST + WebSocket + upload)       │
│                                                                                │
│  ┌─ cell-grid-rest ───────┐  ┌─ collab-crdt-worker (WebSocket gateway) ────┐   │
│  │ OIDC tenant-scoped     │  │ - OIDC validated at WS upgrade              │   │
│  │ + Cedar license-gate   │  │ - tenant-binding rebound at each WS message │   │
│  └────────────────────────┘  └─────────────────────────────────────────────┘   │
│  ┌─ import-export-worker (gVisor sandboxed) ───────────────────────────────┐  │
│  │  - XLSX import via calamine 0.26 in gVisor user-mode sandbox            │  │
│  │  - ClamAV + OPSWAT AV scan sidecars BEFORE entering the sandbox         │  │
│  │  - XLSX export via rust_xlsxwriter 0.79 in gVisor                        │  │
│  └──────────────────────────────────────────────────────────────────────────┘  │
│                                                                                │
│  Trust boundary 2: Per-tenant Citus partition + RLS                            │
│                                                                                │
│  ┌─ Postgres + Citus (workbook + cell + license + acl + comments + version) ─┐ │
│  │  - tenant_id partition + RLS                                              │ │
│  │  - per-tenant connection pool                                             │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│  ┌─ Valkey (ephemeral CRDT + recalc-progress) ─┐ ┌─ S3 (snapshots + history) ─┐│
│  │ tenant-prefixed key                         │ │ pack-scoped bucket          ││
│  └──────────────────────────────────────────────┘ └─────────────────────────────┘│
│  ┌─ OCI Object Storage (Arrow/Parquet large-sheet blocks) ─────────────────┐  │
│  │ per-(tenant, workbook, sheet) key                                       │  │
│  └──────────────────────────────────────────────────────────────────────────┘  │
│                                                                                │
│  Trust boundary 3: WS gateway → CRDT op fan-out (per-workbook lease)           │
│                                                                                │
│  ┌─ WebSocket gateway lease coordinator ────────────────────────────────┐     │
│  │  - one WS pod owns one (tenant, workbook_id) lease via Valkey         │     │
│  │  - cross-tenant collab forbidden by tenant-binding on connect        │     │
│  │  - per-workbook message routing                                      │     │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                                                                │
│  Trust boundary 4: Sheets → cell SDK + foundry-runtime SDK + ontology SDK      │
│                                                                                │
│  ┌─ Cross-µservice SDK calls (mTLS + SPIFFE) ───────────────────────────┐    │
│  │  - cell µservice (per-workbook cell storage substrate)                │    │
│  │  - foundry-runtime µservice (AI-formula + smart-fill bridge)          │    │
│  │  - ontology µservice (typed-column descriptor reads)                  │    │
│  │  - tenancy µservice (per-seat licensing + tenant resolution)          │    │
│  │  - workflow-engine µservice (trigger-bridge dispatch)                 │    │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                                                                                │
│  Trust boundary 5: Sheets → AI-formula (foundry-runtime SDK)                   │
│                                                                                │
│  ┌─ foundry-runtime SDK (AI-formula + smart-fill) ──────────────────────┐    │
│  │  - mTLS + SPIFFE identity                                            │    │
│  │  - tenant-prompt audit-emitted (90d retention)                       │    │
│  │  - AI-formula completion validated against formula-engine schema     │    │
│  │  - Prompt-injection signature scrubbed before LLM submission         │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  Trust boundary 6: Audit chain emission                                        │
│                                                                                │
│  ┌─ audit-chain-emitter (in-process; signs cell-edit + sharing + license) ┐    │
│  │  - Ed25519 signing key from OpenBao (rotated 90d)                      │    │
│  │  - Merkle-chain over per-tenant per-workbook event sequence            │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  Trust boundary 7: XLSX import → gVisor sandbox boundary                       │
│                                                                                │
│  ┌─ gVisor user-mode sandbox (XLSX import/export isolation) ──────────────┐    │
│  │  - calamine 0.26 + rust_xlsxwriter 0.79 run inside sandbox             │    │
│  │  - syscall surface restricted                                          │    │
│  │  - egress denied (no network from inside sandbox)                      │    │
│  │  - filesystem: read-only tempdir per job                               │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────────────────────────────┘
```

Seven trust boundaries:
1. **External → Sheets ingress** (CDN+WAF + TLS + OIDC + CSP).
2. **Per-tenant Citus partition + RLS** (the load-bearing isolation boundary).
3. **WS gateway → per-workbook lease** (cross-tenant collab forbidden).
4. **Sheets → cross-µservice SDKs** (mTLS + SPIFFE).
5. **Sheets → AI-formula SDK** (prompt-injection scrub + LLM output validation).
6. **Audit-chain emission** (Ed25519 signing; non-repudiation).
7. **XLSX import → gVisor sandbox** (defense-in-depth against malicious workbook supply-chain).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Workbook metadata + cell rows | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_QUASI_IDENTIFIER` / `PHI` | High | 30d hot (Postgres) + version-history S3 cold | Postgres + S3 |
| Editor session state (active drafts, cursor, viewport) | `BEHAVIORAL_TENANT_PRODUCT` | High | 30d hot (Postgres) + Valkey ephemeral while active | Postgres + Valkey |
| Collab CRDT op stream | `BEHAVIORAL_TENANT_PRODUCT` | High | Valkey ephemeral while session active; sealed deltas to Postgres on save | Valkey + Postgres |
| Recalc-progress streaming events | `INTERNAL_ONLY` | Low | ephemeral; Valkey | Valkey |
| Per-seat license attribution rows | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | 24mo cold (Postgres) + audit-chain seal | Postgres + audit-chain |
| Per-range ACL rows | `AUDIT` | High | append-only; retention per pack | Postgres + audit-chain |
| Comments + threaded notes | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` | High | 30d hot; retention per pack | Postgres |
| Version-history snapshots | `BEHAVIORAL_TENANT_PRODUCT` | High | 90d hot; 7y cold (per ADR-0028 retention) | S3 |
| Large-sheet Arrow/Parquet blocks | `BEHAVIORAL_TENANT_PRODUCT` | High | 30d hot; cold-tier per pack | OCI Object Storage |
| WASM bundle chunks + SRI hashes | `INTERNAL_ONLY` | Low | per-release; previous versions retained 90d | CDN + repo |
| XLSX uploads (raw) | `BEHAVIORAL_TENANT_PRODUCT` | High | quarantine 7d post-import; then deleted | S3 (quarantine bucket) |
| XLSX export jobs (output) | `BEHAVIORAL_TENANT_PRODUCT` | High | 24h hot for download; then deleted | S3 |
| AI-formula prompts (tenant-issued) | `BEHAVIORAL_TENANT_PRODUCT` + occasionally `PII_IDENTIFYING` | High | 90d hot for audit | Postgres (audit) |
| AI-formula completions (returned drafts) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 90d hot for audit | Postgres |
| Smart-fill inference outputs | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 90d hot for audit | Postgres |
| Connected-sheets external-query results | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` | High | per refresh; not retained beyond materialized range | Postgres |
| WebSocket session secrets (per-connection token) | `SECRET` | Critical | ephemeral; TTL ≤ 1h | OpenBao + in-memory |
| Cedar policy fragments (per-tenant license + range-ACL) | `INTERNAL_ONLY` + occasionally `SECRET` (entitlement signature) | High | git-versioned; per-tenant entitlement in OpenBao | git + OpenBao |
| Audit-chain Ed25519 signing keys | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Editor REST SDK API keys | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Hashed tenant ID (used in CDN cache key + topic namespace) | `SENSITIVE_PIPA_ART23` | High | salted; rotation 12mo | OpenBao tenant-resolver |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant business power user | Untrusted external | OIDC + MFA | Open workbook; cell editing; save; share; collab |
| External tenant analyst | Untrusted external | OIDC + MFA | Authoring; pivot tables; charts; conditional formatting |
| External tenant financial modeller | Untrusted external | OIDC + MFA | Formula authoring; named ranges; connected sheets |
| External tenant agentic developer role (programmatic) | Untrusted external | Per-tenant SDK API key | Submit cell ops via SDK; AI-formula invocation |
| Sheets REST handler (in-process) | Trusted internal | OpenBao-issued ServiceAccount + SPIFFE | Read/write workbook state; submit to cell µservice |
| WebSocket gateway worker (in-process) | Trusted internal | SPIFFE | Fan-out CRDT ops within per-tenant per-workbook lease |
| Recalc worker (in-process) | Trusted internal | SPIFFE | Compute recalc plan; emit progress events |
| XLSX import/export worker (gVisor sandboxed) | Semi-trusted internal | SPIFFE | Convert XLSX↔canonical-sheet inside sandbox |
| AV-scan sidecars (ClamAV + OPSWAT) | Trusted internal | SPIFFE | Scan uploaded XLSX for malware before import |
| AI-formula bridge (cross-µservice via SDK) | Semi-trusted internal | mTLS + SPIFFE | Submit tenant prompts to foundry-runtime; receive completions |
| Cell µservice (cross-µservice via SDK) | Semi-trusted internal | mTLS + SPIFFE | Receive Sheets cell submissions; durably persist cell rows |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Read Sheets code at PR-review time; refuse changes violating gates |
| Council operators (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin operations on Sheets config; emergency override (2-person rule + audit) |
| External auditor | Read-only external, time-boxed | OIDC + MFA + JIT short-lived token | Read workbook audit trail; cannot pivot to tenant draft contents |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; XLSX malware upload attempts |
| Attacker — targeted | Untrusted | none | Sophisticated; XLSX supply-chain awareness; formula-engine fuzzing |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure Cedar range-ACL or AI-formula prompt template (mitigated by PR review + LEAN gates) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor; mitigated by least-privilege + audit-chain + 2-person rule |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Tenant-A user opens a workbook session and impersonates tenant-B via cookie/token manipulation**
- Asset: workbook session boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - OIDC token bound to authenticated tenant; engine validates `tenant_id` claim on every REST call.
  - Server-side stamping: editor REST overrides any client-supplied tenant_id with the OIDC claim.
  - WebSocket re-validates tenant_id on every message dispatch (no trust of long-held connection).
  - Mismatch attempts return 401 + audit-emit `sheets_tenant_spoofing_attempt`.
- Owner: axis-sheets + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Forged cell-edit signature: malicious actor submits cell-edit to cell µservice claiming Sheets authorship via leaked SPIFFE**
- Asset: Sheets SPIFFE identity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SPIFFE identity bound to pod; cannot be used outside cluster.
  - Token rotation 24h.
  - Cell µservice verifies submission carries valid Ed25519 signature from Sheets-issued key.
- Owner: ops-security + axis-sheets
- Residual: L

**T-S-03 — WebSocket session token replay**
- Asset: WebSocket session
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - WS session token bound to (tenant_id, user_id, workbook_id) tuple at upgrade.
  - TTL ≤ 1h; rebinding required after.
  - Idle disconnect after 5min of no client message.
  - Per-message nonce + monotonic sequence counter.
- Owner: ops-security
- Residual: L

**T-S-04 — Malicious XLSX file impersonating a benign workbook (supply-chain attack via tenant upload)**
- Asset: XLSX upload pipeline
- Likelihood: H (XLSX-based malware is documented; e.g., Emotet historically rode XLSX macros) / Impact: H / Risk: **H**
- Mitigations:
  - **gVisor user-mode sandbox** around the entire calamine 0.26 import process; no syscall surface to host.
  - **ClamAV + OPSWAT MetaDefender** scan EVERY uploaded XLSX BEFORE entering the sandbox; both must pass.
  - VBA macros / Apps-Script-equivalent code blocks structurally stripped on import (per ADR-SHEETS-0007 named-limit list).
  - Embedded XLSX images downgraded to canonical format (PNG); proprietary formats stripped.
  - Embedded XLSX external links removed; replaced with audit row.
  - File size cap (configurable; default 200 MB per upload); larger uploads refused.
  - LEAN check `oya-governance-sheets-import-sandboxed-and-avscan-required` validates import pipeline at PR-time.
- Owner: ops-security + axis-sheets
- Residual: L (defense-in-depth multiple layers; supply-chain risk inherent to file-import features)
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.28, A.5.32, A.8.7, A.8.32

**T-S-05 — AI-formula completion forged (attacker injects completion via prompt-injection in tenant prose)**
- Asset: AI-formula output
- Likelihood: H (prompt injection is common) / Impact: M / Risk: **M-H**
- Mitigations:
  - Tenant prose scrubbed for prompt-injection markers before LLM submission (basic regex + content-policy classifier).
  - LLM completion ALWAYS validated against formula-engine grammar before user-surfaced.
  - User explicitly accepts AI-formula draft before save; no auto-submission at T1.
  - T2 cross-µservice auto-apply gated by Cedar + ChangeSet review per ADR-SHEETS-0005.
  - LEAN check `oya-governance-ai-formula-validation-required`.
- Owner: axis-sheets + ops-security
- Residual: M
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.28; OWASP Top 10 LLM A01 prompt injection

### Tampering (T)

**T-T-01 — Cell-edit tampering during CRDT op stream (forgery)**
- Asset: collab CRDT op stream
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - CRDT ops carry sender authenticated identity; server-side stamp on receive.
  - WS message integrity via per-message HMAC over (session_token, sequence_num, payload).
  - Tampered ops reject + audit-emit `sheets_crdt_op_tampering_attempt`.
- Owner: axis-sheets
- Residual: L

**T-T-02 — Workbook row corruption via concurrent write race**
- Asset: Postgres Workbook row
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Optimistic concurrency check (`version` column) on every workbook-meta update.
  - Single-writer invariant: one WS gateway pod owns active workbook lease via Valkey.
  - Lease TTL ≤ 5min.
- Owner: axis-sheets
- Residual: L

**T-T-03 — Formula-engine function tampering (malicious function-library version pushed)**
- Asset: function-library distribution
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Function library compiled INTO the Sheets binary (no runtime function loading); function-library upgrade requires Sheets binary release + signed-commit policy.
  - LEAN check `oya-governance-sheets-formula-engine-correctness` runs the Excel-reference corpus on every PR build per ADR-SHEETS-0002.
  - Major function-library upgrade triggers formula-engine-rollback runbook readiness drill.
- Owner: ops-security + axis-sheets
- Residual: L

**T-T-04 — XLSX export tampering (attacker manipulates exported XLSX in transit)**
- Asset: XLSX export download
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - TLS 1.3 in transit; pre-signed S3 download URL (TTL ≤ 1h).
  - Optional: tenant-side signature verification UI shows the workbook version_sha at export time.
- Owner: ops-security + axis-sheets
- Residual: L

**T-T-05 — Per-seat license attribution tampering (forge low-seat-count to evade billing)**
- Asset: SeatLicense row in Postgres
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Postgres row append-only (audit table); UPDATE/DELETE refused by trigger.
  - Each row Ed25519-signed at insert.
  - Tampering detected on next aggregation; audit-chain seal gap fires Sev-1 alert.
- Owner: ops-security + axis-sheets + tenancy
- Residual: L

**T-T-06 — WASM bundle tampering: attacker injects malicious code into a WASM chunk before CDN ingress**
- Asset: WASM bundle integrity
- Likelihood: L / Impact: H (RCE in tenant browser) / Risk: **M**
- Mitigations:
  - SRI hash per chunk in HTML; browser refuses mismatched chunk.
  - Per-release SBOM published; `cargo deny` + Trivy scan at build time.
  - CDN-side immutability lock on uploaded bundle.
  - LEAN check `oya-governance-wasm-bundle-sri`.
- Owner: axis-sheets + ops-security
- Residual: L

**T-T-07 — Per-range ACL tampering (attacker bypasses range-ACL to read masked column)**
- Asset: per-range ACL Cedar policy fragment
- Likelihood: L / Impact: H (data exfiltration) / Risk: **M**
- Mitigations:
  - Per-range ACL stored as Cedar policy fragments per ADR-SHEETS-0006; evaluator default-deny.
  - LEAN check `oya-governance-sheets-range-acl-cedar-required` validates ACL evaluation on every render-path PR.
  - Server-side ACL filter applied BEFORE returning cell payload to client; client-side filtering never load-bearing.
  - Range-ACL changes audit-chain sealed.
- Owner: ops-security + axis-sheets
- Residual: L

### Repudiation (R)

**T-R-01 — Tenant operator denies authorship of a cell edit**
- Asset: cell-edit event
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every cell-edit requires OIDC-bound identity + recorded in audit-chain with actor.
  - Cell-edit event signed (Ed25519) over (tenant_id, workbook_id, sheet_id, cell_ref, old_value_hash, new_value_hash, author_oidc_sub, timestamp).
  - Cell-edit sampling rate: 1.0 for high-sensitivity tenants (pack-us-healthcare); configurable per pack for lower-sensitivity workloads.
- Owner: ops-security + axis-sheets
- Residual: L

**T-R-02 — Tenant denies AI-formula consent**
- Asset: AI-formula prompt
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - AI-formula invocation requires explicit per-session opt-in click; consent event audit-emitted.
  - Foundry-runtime tenant DPA carries LLM-routing disclosure.
- Owner: council-privacy + axis-sheets
- Residual: L

**T-R-03 — Sharing-permission change denied**
- Asset: share/ACL change event
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Every sharing change requires OIDC-bound identity + audit-chain seal.
  - Per-range ACL changes additionally emit `share_acl_changed` event with old/new ACL hashes.
- Owner: ops-security
- Residual: L

### Information Disclosure (I)

**T-I-01 — Cross-tenant workbook leak via Citus partition bypass**
- Asset: Workbook + Cell Postgres rows
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Citus partition + Row-Level Security (RLS) BOTH enforce tenant isolation.
  - Per-tenant Postgres connection pool — connection's session variable carries tenant_id; RLS predicate reads it.
  - LEAN check `oya-governance-citus-rls-enforced`.
  - Per-tenant query audit via Postgres extension `pgaudit`.
  - Penetration test against tenant boundary annually.
- Owner: ops-security + axis-sheets
- Residual: L

**T-I-02 — XSS in cell config panel injecting attacker JS via tenant-rendered cell values**
- Asset: Sheets editor visual surface
- Likelihood: H (XSS is the #1 webapp threat) / Impact: H / Risk: **H**
- Mitigations:
  - All rendered cell values go through Leptos virtual-DOM (text nodes only; no `innerHTML`).
  - Strict CSP: `script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`; no `unsafe-inline`; no `unsafe-eval`.
  - `Trusted Types` enforced.
  - LEAN check `oya-governance-xss-vector-scan`.
  - Annual XSS pen-test against Sheets.
- Owner: axis-sheets + ops-security + council-design-system
- Residual: L

**T-I-03 — Formula-injection: tenant pastes crafted formula; spoofed function evaluates with attacker-supplied args**
- Asset: formula-engine evaluation surface
- Likelihood: M / Impact: M / Risk: **M-H**
- Mitigations:
  - Formula-engine grammar restricts function names to the registered ≥400-function set per ADR-SHEETS-0002; unknown function names rejected.
  - Cell value sanitisation: leading `=` is interpreted as formula only after OIDC-authenticated cell-edit context.
  - LEAN check `oya-governance-sheets-formula-engine-grammar-strict`.
- Owner: axis-sheets
- Residual: L

**T-I-04 — Collab op leak: subscriber on workbook A receives ops from workbook B (lease misrouting)**
- Asset: WebSocket CRDT op delivery
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - WS gateway lease is keyed on (tenant_id, workbook_id) tuple; routing by consistent-hash.
  - Server-side filter on every outbound message: (subscriber.tenant_id, subscriber.workbook_id) == (op.tenant_id, op.workbook_id).
  - Cross-workbook / cross-tenant delivery attempt audit-emits Sev-1 alert.
- Owner: axis-sheets
- Residual: L

**T-I-05 — AI-formula prompt leakage: tenant prose contains PII; prompt sent to foundry-runtime; LLM provider logs**
- Asset: AI-formula prompt content
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations:
  - SDK PII redactor strips obvious PII (emails / phone numbers / SSNs / IDs) from prose before foundry-runtime submission.
  - Tenant onboarding discloses LLM-routing; per-tenant LLM provider choice (BYO-LLM available); zero-retention LLM models preferred.
  - Foundry-runtime tenant DPA includes upstream-LLM disclosure clause.
  - Audit-chain seal on every AI-formula invocation; tenant can DSR-revoke later.
- Owner: council-privacy + axis-sheets + foundry-runtime
- Residual: M (PII redactor is heuristic)
- Frameworks: GDPR Art. 6, 25, 32; KR PIPA Art. 29; HIPAA §164.502(b) (minimum-necessary)

**T-I-06 — Per-tenant branding mid-render injection (CSS or script via tenant-uploaded branding asset)**
- Asset: Sheets render surface
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - **Per-tenant branding mid-render is FORBIDDEN by anti-pattern policy**.
  - LEAN check `oya-governance-no-tenant-branding-mid-render`.
  - Post-GA marketplace branding (if introduced) restricted to iframed sandboxes with separate CSP.
- Owner: ops-security + council-design-system
- Residual: L

**T-I-07 — CDN cache pollution: tenant-A's workbook state cached under tenant-B's key**
- Asset: CDN edge cache
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Per-tenant CDN cache key.
  - Static assets are tenant-agnostic; tenant-specific content NEVER cached at CDN edge.
  - LEAN check `oya-governance-cdn-cache-key-tenant-isolated`.
- Owner: cloud-iac + ops-security
- Residual: L

**T-I-08 — Connected-sheets credential leakage (external-source credentials stored in cell context)**
- Asset: external-source DB credentials for connected-sheets
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - External-source credentials NEVER stored in Postgres workbook rows; always referenced from OpenBao via `${openbao:...}` syntax.
  - Connected-query results materialize as cell values only (no credentials in cell payload).
  - Tenant operator must have explicit `sheets.connected_sheets.<source>.read` Cedar entitlement.
  - LEAN check `oya-governance-no-secrets-in-cell-payload`.
- Owner: ops-security + axis-sheets
- Residual: L

**T-I-09 — XLSX export carries sensitive PII / PHI without redaction**
- Asset: exported XLSX file
- Likelihood: M / Impact: H / Risk: **M-H**
- Mitigations:
  - Export pipeline reads per-range ACL Cedar policy; cells outside requestor's ACL are masked at export.
  - Data-class markers carried into XLSX export metadata; tenant-side viewer surfaces them.
  - Export audit-chain sealed.
- Owner: axis-sheets + council-privacy
- Residual: M (operator can intentionally export PII within their authorised ACL; this is by design)

### Denial of Service (D)

**T-D-01 — Per-tenant workbook session flood overwhelms WS gateway**
- Asset: WS gateway capacity
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-tenant active-session cap (default 50); refuse `429` above cap.
  - Fair-share scheduling.
  - HPA on WS gateway pods; min 3 replicas, max 100.
- Owner: ops-sre-reliability + axis-sheets
- Residual: L

**T-D-02 — Recalc storm: 1k+ users editing same formula chain causes recalc-engine queue saturation**
- Asset: recalc-engine capacity
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-tenant recalc-queue depth cap; excess defers with backoff.
  - Recalc-engine HPA on queue depth.
  - Dep-graph cycle detection (per ADR-SHEETS-0004); cycles trigger error not infinite-loop.
  - Slow-formula budget: any single recalc plan > 30s is killed + tenant notified.
- Owner: axis-sheets + ops-sre-reliability
- Residual: L

**T-D-03 — XLSX bomb (zip-bomb / formula-bomb) crashes XLSX import**
- Asset: XLSX import pipeline
- Likelihood: M / Impact: H (worker pool exhaustion) / Risk: **M-H**
- Mitigations:
  - gVisor sandbox enforces RAM + CPU + wall-clock budget per import job.
  - Decompression bomb detection: archive expansion ratio > 100× refused.
  - Formula-bomb detection: ≥ 10M cell formulas in a single workbook refused; tenant prompted to split.
- Owner: axis-sheets + ops-security
- Residual: L

**T-D-04 — Collab desync flood: malicious user spams CRDT ops, exhausting Valkey**
- Asset: Valkey ephemeral state
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-(tenant, user) WS message rate limit (default 100 ops/sec).
  - Per-tenant Valkey memory cap.
  - Slow-client quarantine.
- Owner: axis-sheets + ops-sre-reliability
- Residual: L

**T-D-05 — AI-formula timeout cascade**
- Asset: AI-formula bridge
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - AI-formula requests run in background.
  - Timeout 10s server-side; circuit breaker after 3 consecutive timeouts.
  - User sees "AI-formula degraded; please retry or proceed manually" banner.
- Owner: axis-sheets + foundry-runtime
- Residual: L

**T-D-06 — Postgres lock contention on hot workbook**
- Asset: Postgres workbook row
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Save path goes through cell µservice SDK; Sheets doesn't lock Postgres row directly.
  - Local edit buffer + CRDT op stream absorbs concurrent edits.
- Owner: axis-sheets
- Residual: L

**T-D-07 — License-gate failure-open (Cedar evaluator crash)**
- Asset: license-gate-cedar evaluator
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - **Default-deny fail-closed**: Cedar evaluator failure refuses workbook open with 503.
  - Cedar evaluation cached per-(tenant, principal) for 30s; cache failure → re-evaluate.
  - Health probe on Cedar evaluator.
- Owner: ops-security + axis-sheets
- Residual: L

**T-D-08 — Connected-sheets external-source slow query exhausts worker pool**
- Asset: connected-sheets worker
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - External-source query timeout 30s server-side.
  - Per-(tenant, source) concurrent-query cap.
  - Circuit breaker on consecutive timeouts.
- Owner: axis-sheets + ops-sre-reliability
- Residual: L

**T-D-09 — Chart render storm: 1k+ charts on a single dashboard sheet**
- Asset: charts BC + browser rendering capacity
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Per-sheet chart count cap (default 100); excess prompted to split.
  - Lazy-render charts not in viewport.
  - Chart-render budget breach surfaces "degraded; reduce chart count" banner.
- Owner: axis-sheets + council-design-system
- Residual: L

### Elevation of Privilege (E)

**T-E-01 — XSS escalates to session token theft + cross-tenant editing**
- Asset: editor session token
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - All XSS mitigations from T-I-02.
  - Session token `HttpOnly` cookie; JS cannot read.
  - Sensitive WS operations re-validate OIDC bearer token from in-memory state.
- Owner: ops-security
- Residual: L

**T-E-02 — Operator JIT elevation abused to read tenant workbook contents**
- Asset: operator-override path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - 2-person rule required for draft read on behalf of tenant.
  - Read pattern detection.
- Owner: ops-security
- Residual: L

**T-E-03 — Cedar policy bypass via crafted entitlement claim**
- Asset: license-gate-cedar + range-ACL evaluators
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4 used; field length bounded; fuzzing at CI.
  - Entitlement claims signed by tenancy µservice.
  - Server-side stamping of (tenant_id, principal_id).
- Owner: ops-security
- Residual: L

**T-E-04 — Sheets formula-engine executing arbitrary host code**
- Asset: formula-engine evaluation surface
- Likelihood: L (formula engine is pure Rust, no eval-like primitives) / Impact: H / Risk: **M**
- Mitigations:
  - Formula engine is a closed set of ≥400 functions per ADR-SHEETS-0002; NO `EXEC`, `SHELL`, `EVAL`-class functions.
  - LEAN check `oya-governance-editor-execution-forbidden` validates Sheets crates contain no `process::Command` / exec primitives outside the import-export sandboxed worker.
  - VBA / Apps-Script-equivalent explicitly excluded per ADR-SHEETS-0007 + ADR-SHEETS-0005 T2 review.
- Owner: axis-sheets + ops-security
- Residual: L

**T-E-05 — XLSX import code path escapes gVisor sandbox**
- Asset: gVisor user-mode sandbox boundary
- Likelihood: L (gVisor mature; Google production hardened) / Impact: H / Risk: **M**
- Mitigations:
  - gVisor user-mode sandbox; restricted syscall surface.
  - calamine 0.26 (Rust) is memory-safe by language.
  - Defense-in-depth: ClamAV + OPSWAT scan before sandbox entry.
  - Resource budget enforced per job.
- Owner: ops-security + axis-sheets
- Residual: L

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Cell-edit events + AI-formula prompts | Multiple authoring sessions can link to single end-user via prompt patterns. | Per-tenant session scoping; AI-formula prompt redaction. | M |
| T-L-02 | Identifiability | Hashed tenant ID in CDN cache key | sha256(tenant_id)[..16] may be re-identifiable. | Salted hash; salt rotated 12mo. | L |
| T-L-03 | Non-repudiation | Cell-edit authorship | Tenant may deny authorship. | Signed commits; per-edit audit-chain seal. | L |
| T-L-04 | Detectability | Cell-edit timing patterns | Tenant authoring cadence correlates with business events. | Expected; behavioral; consent at onboarding. | M |
| T-L-05 | Disclosure | AI-formula prompt routing to third-party LLM provider | Tenant prose may reveal end-user data to LLM provider. | PII redactor + tenant disclosure + BYO-LLM option + zero-retention models. | M |
| T-L-06 | Unawareness | End-user unaware their data is in a tenant workbook | Tenant's end-user may not know data is in cell payload. | Tenant DPA upstream-disclosure clause. | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure cascade across workbooks | End-user erasure across workbook cells / AI prompts / connected-sheets results. | DSR cascade per `oya-dsr-cascade-runner`; 30d SLA. | M (best-effort) |
| T-L-08 | Non-compliance | Per-seat license attribution retention beyond consent | License-row retention 24mo may exceed end-user consent. | Retention bounded; audit-chain forensic vs operational distinction. | L |
| T-L-09 | Disclosure | XLSX export carries PII outside the platform | Tenant exports workbook; PII leaves the platform on download. | ACL-aware export masking; export audit; tenant DPA on export discipline. | M |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Per-tenant Citus partition + RLS | Preventive | axis-sheets | `oya-governance-citus-rls-enforced` lane |
| OIDC tenant-scope binding | Preventive | ops-security | OIDC audit log |
| Server-side tenant_id stamping | Preventive | axis-sheets | LEAN check on rest crate |
| Ed25519 cell-edit signature at save | Preventive | axis-sheets | per-IP integration test |
| Ed25519 audit-chain seals | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Strict CSP + Trusted Types | Preventive (XSS) | council-design-system + ops-security | LEAN check + headers regression test |
| SRI hashes on WASM chunks | Preventive | axis-sheets | `oya-governance-wasm-bundle-sri` lane |
| gVisor sandbox for XLSX import/export | Preventive | ops-security + axis-sheets | `oya-governance-sheets-import-sandboxed-and-avscan-required` lane |
| ClamAV + OPSWAT AV scan (XLSX) | Preventive | ops-security | AV-scan integration test |
| CRDT op HMAC + sequence counter | Preventive (replay) | axis-sheets | CRDT regression tests |
| Single-writer Valkey lease per workbook | Preventive | axis-sheets | concurrent-writer integration test |
| Per-tenant rate limit | Preventive (DoS) | axis-sheets | Sheets REST metrics |
| AI-formula PII redactor | Preventive | axis-sheets + council-privacy | quarterly synthetic-PII drill |
| AI-formula completion schema validation | Preventive | axis-sheets | `oya-governance-ai-formula-validation-required` lane |
| Cedar per-seat license-gate (default-deny) | Preventive (billing) | ops-security + tenancy | per-IP integration test |
| Cedar per-range ACL (default-deny) | Preventive | ops-security + axis-sheets | `oya-governance-sheets-range-acl-cedar-required` lane |
| 2-person rule for operator overrides | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| Cross-tenant collab forbidden | Preventive | axis-sheets | LEAN check on collab-crdt-worker |
| Per-tenant CDN cache key | Preventive | cloud-iac | `oya-governance-cdn-cache-key-tenant-isolated` lane |
| No tenant-branding-mid-render | Preventive | council-design-system | `oya-governance-no-tenant-branding-mid-render` lane |
| Editor-execution-forbidden | Preventive | axis-sheets | `oya-governance-editor-execution-forbidden` lane |
| Network policy: Sheets → cell/ontology/foundry-runtime/tenancy/audit-chain/workflow-engine/docs/slides/drive/forms/mail/community SDKs only | Preventive | ops-sre-reliability | Kubernetes NetworkPolicy review |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-05 (AI-formula prompt PII leakage) | M | PII redactor is heuristic; tenant disclosure is proportionate. | Quarterly |
| T-S-05 (prompt injection bypass) | M | AI-formula is advisory; tenant explicit-accept is load-bearing. | Quarterly |
| T-I-09 (XLSX export PII) | M | Operator-authorised export within ACL is by-design; tenant DPA covers. | Annually |
| T-L-01 (linkability across sessions) | M | Inherent to spreadsheet authoring tracing. | Annually |
| T-L-04 (detectability via timing) | M | Tenant business reality. | Annually |
| T-L-05 (LLM provider routing) | M | Tenant disclosure + BYO-LLM option. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant joint-controllership. | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Bounded by retention windows. | Annually |
| T-L-09 (XLSX export off-platform leakage) | M | Tenant DPA on export discipline. | Annually |

Sign-off:

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`
- council-design-system: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

- KR PIPA Art. 23 (sensitive PII): hashed tenant IDs sensitive when paired with auxiliary; salt-rotation in T-L-02 satisfies Art. 23.
- KR PIPA Art. 29 (technical safeguards): every T-*-NN mitigation maps to one of 12 prescribed safeguards.
- KR PIPA Art. 23-2 (cross-border): KR tenant data stays in pack-kr cluster (AI-formula routes to KR-resident LLM provider only).
- KR-ISMS-P §2.7 (접근통제) + §2.5 (인적보안): per-seat Cedar + 2-person rule + range-ACL map directly.

### pack-us-healthcare (HIPAA)

- HIPAA §164.312(a)(1) (access control): per-tenant Citus + RLS + per-range Cedar ACL + Ed25519 audit-chain.
- HIPAA §164.312(b) (audit controls): cell-edit audit-chain emission; retention ≥ 6y for pack-us-healthcare workbooks.
- HIPAA §164.502 (minimum-necessary): per-range ACL + SDK redactor for AI-formula + data_class markers warn before share.
- HIPAA §164.504(e) (BAA): oyatie operates as BA; BAA at `microservices/sheets/legal/baa-template.md`.

### pack-eu (GDPR + EDPB + NIS2)

- GDPR Art. 25: every mitigation mapped to Schrems-II-compatible TOM.
- GDPR Art. 35 DPIA: this threat model + `dpia.md` satisfy DPIA for high-risk processing.
- GDPR Art. 32: every T-*-NN mitigation contributes.
- GDPR Arts. 44-50: pack-eu cluster EU-resident; AI-formula routes EU-resident provider.
- NIS2 2022/2555: when oyatie crosses thresholds, 24h/72h/1mo timelines apply.
- EU AI Act 2024: AI-formula invocation logged + auditable; T2 cross-µservice scope per ADR-SHEETS-0005.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/sheets-overlay.md`.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x through CC9.x covered inline | `microservices/sheets/compliance.md` |
| ISO 27001:2022 | Annex A.5-A.8 controls cited inline | `microservices/sheets/compliance.md` |
| GDPR | Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/sheets/dpia.md` + `compliance.md` |
| OWASP ASVS L2 | V1 V2 V3 V4 V5 V7 V8 V9 V11 V12 V13 V14 covered | `microservices/sheets/compliance.md` |
| WCAG 2.2 AA | Accessibility for screen-reader friendly cell-grid | `microservices/sheets/compliance.md` |

## Re-review Triggers

- Any change to the trust boundary diagram.
- Any Layer-A version upgrade (CDN / WAF / Postgres / Valkey / WebSocket gateway / Loro / Arrow / Parquet / calamine / rust_xlsxwriter / ClamAV / OPSWAT).
- New function-library release (formula-engine version bump per ADR-SHEETS-0002).
- AI-formula provider change.
- New pack activation.
- Annual scheduled review (Q2 each year).
- Post-incident review.
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0056: BNF v4.1.
- ADR-0065: Leptos for browser UI.
- ADR-0103 (Bominal): hexagonal migration; inherited.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0135: Sheets net-new µservice.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- ADR-SHEETS-0001..0007 (local).
- `microservices/sheets/PRD.md`.
- `microservices/sheets/dpia.md`.
- `microservices/sheets/compliance.md`.
- `/specs/microservices/sheets.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS v4.0.
- OWASP Top 10 LLM Applications (2023).
- NIST SP 800-154.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
