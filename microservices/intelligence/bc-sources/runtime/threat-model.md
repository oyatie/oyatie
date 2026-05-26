---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-runtime + ops-security
deciders: council-architecture, ops-security, axis-foundry-runtime, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP Top 10 for LLM Applications (2025) + NIST SP 800-154 + MITRE ATLAS
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Foundry-substrate or runtime-pool architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU AI Act Arts. 9-15 (risk management + data + transparency + human oversight + accuracy + cybersecurity)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR FSC AI Guideline 2024"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316", "FDA SaMD pre-market (when capability touches clinical decision support)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35", "EU AI Act Arts. 9-15 (high-risk systems)", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2", "METI AI Governance Guidelines 2024"]
  pack-sg: ["PDPA 2012 §11-26", "MAS FEAT Principles + Veritas Toolkit"]
  pack-au: ["Privacy Act 1988 APP 1-13", "AHRC Human Rights and Technology AI guidance"]
  pack-in: ["DPDPA 2023 §6-10", "MeitY AI Advisory 2024"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "ANPD AI guidance"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021", "UAE Charter for Responsible AI"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SDAIA Generative AI guidelines"]
doc_status: published
---

# Threat Model: foundry-runtime µservice

## Purpose

Identify, classify, and mitigate threats to the foundry-runtime µservice's confidentiality, integrity, availability, and privacy posture. The foundry-runtime is the execution plane for every hosted-agent invocation in oyatie; a compromise here cascades to every Foundry-class product (Workflow Studio first; subsequent hero products). This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, and (where engaged) EU AI Act notified bodies under Arts. 9 + 15.

## Scope

### In-scope

All components introduced by ADR-0025 + ADR-0131 (per-microservice flat layout) for the foundry-runtime µservice, deployed in a **dedicated runtime Kubernetes cluster** (decision confirmed 2026-05-17 per PRD OQ#1; matches AWS Bedrock + GCP Vertex isolation posture):

| Layer-A substrate (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Kubernetes runtime pods | `oya-foundry-runtime-capability-executor-*` (8 crates) |
| Valkey 8.1 (Redis wire-compat) OSS LTS (session-state hot tier) | `oya-foundry-runtime-session-state-*` (9 crates) |
| Postgres 16 LTS (session-state cold + registry mirror) | `oya-foundry-runtime-invocation-orchestrator-*` (7 crates) |
| Istio mesh (mTLS + traffic split) | `oya-foundry-runtime-runtime-pool-*` (6 crates) |
| SPIRE (SPIFFE identity for runtime pods) | `oya-foundry-runtime-capability-registry-cache-*` (7 crates) |
| OpenBao (secret-manager; SecretReference materialisation) | Capability descriptors mirrored from foundry-supervisor |

### Out-of-scope

- Threats to the underlying Kubernetes cluster + container runtime — owned by the `cloud-k8s` µservice's threat model.
- Threats to LLM provider backends themselves (OpenAI / Anthropic / Bedrock / Vertex / Azure OpenAI / OSS local providers) — covered by the `foundry-providers` µservice's threat model. This document inherits provider compromise as upstream.
- Threats to guardrail rule execution logic — owned by `foundry-guardrails`.
- Threats to the OpenBao secret-manager itself — owned by `cloud-secrets`.
- Threats to the workload µservices that consume foundry-runtime (workflow-engine + product surfaces) — each owns its own threat model.
- Threats to capability authoring UX (lives in Workflow Studio surface).

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators (Workflow Studio)    Customer applications              │
│         │                                  │                               │
│         │ (HTTPS + OIDC + mTLS)            │ (per-tenant API key)          │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy/Istio gateway) ──────────────────────────────┐   │
│  │  - TLS termination + WAF + DDoS protection                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated foundry-runtime cluster ────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ capability-executor-rest ─────────┐                                    │
│  │  OIDC tenant-scoped reads/writes   │                                    │
│  │  Cedar policy at boundary          │                                    │
│  └────────────────────────────────────┘                                    │
│             │                                                              │
│  Trust boundary 2: Per-tenant Valkey multi-tenancy (KEYDB prefix isolation) │
│             │                                                              │
│  ┌─ Valkey cluster (HA; per-pack) ────────────────────────────────────┐     │
│  │  Per-tenant key prefix `<tenant_hash>:` enforced by SessionStore  │     │
│  │  TLS + AUTH; KMS-bound on-disk encryption                         │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│  ┌─ Postgres (mirror + cold session restore + mutation log) ────────┐      │
│  │  TDE; per-pack KMS keyring; row-level security per tenant         │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 3: Runtime pod → sibling µservice (Foundry plane)          │
│             │                                                              │
│  ┌─ mTLS + SPIFFE to foundry-providers / foundry-guardrails /        ┐     │
│  │  foundry-evidence / foundry-supervisor                            │     │
│  │  Sibling refuses non-runtime SPIFFE identity                      │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                                                                            │
│  Trust boundary 4: Runtime pod → LLM provider (via foundry-providers)      │
│             │                                                              │
│  ┌─ Provider credentials NEVER resident in runtime pod ────────────┐       │
│  │  foundry-providers holds credentials; runtime asks it to invoke │       │
│  │  with opaque session token (no raw provider keys downstream)    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│                                                                            │
│  Trust boundary 5: Runtime pod → tenant data (capability descriptor)       │
│             │                                                              │
│  ┌─ Capability descriptor pulled from registry mirror (Postgres) ──┐       │
│  │  Cedar enforces tenant-scope on descriptor read                 │       │
│  │  Cross-tenant descriptor read default-deny                      │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster ingress** (TLS + WAF + OIDC).
2. **Per-tenant Valkey + Postgres multi-tenancy** (KEYDB prefix + RLS; the load-bearing isolation boundary for session-state).
3. **Runtime pod → sibling Foundry µservice** (mTLS + SPIFFE).
4. **Runtime pod → LLM provider** (mediated via `foundry-providers`; credentials never resident in runtime).
5. **Runtime pod → capability descriptor** (Cedar tenant-scope on registry-cache reads).

## Assets & Data Classification

Per Bominal ADR-0028 + the `oya-check-data-class` lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Capability descriptor (tenant + system) | `INTERNAL_ONLY` (text); `BEHAVIORAL_TENANT_PRODUCT` when tenant-tailored | Medium | mirrored from foundry-supervisor | Postgres mirror (read-only mirror) |
| Session conversation history | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` when user identifiers in payload + occasionally `PHI` (pack-us-healthcare) | High | 14d Valkey hot + 90d Postgres cold + 6y for HIPAA scope | Valkey + Postgres |
| Session scratchpad (tool-call working state) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | same as conversation history | Valkey + Postgres |
| Invocation lifecycle record | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | 90d hot + 2y cold + 6y HIPAA | Postgres + audit-chain seal |
| Invocation step trace (per provider/guardrail call) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 7d hot + 6mo cold | foundry-evidence (downstream) |
| Autonomy-tier ceiling per tenant | `INTERNAL_ONLY` | High | replicated from tenancy | local registry cache |
| Cedar policy fragments | `INTERNAL_ONLY` (policy text) | Medium | git history | `microservices/intelligence-runtime/policy/*.cedar` |
| SPIFFE identity material (runtime pods) | `SECRET` | Critical | SPIRE TTL 24h | SPIRE |
| Valkey AUTH password | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| Postgres connection credential | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| KMS keyring (per-pack; SSE for Valkey + Postgres + S3) | `SECRET` | Critical | OpenBao with 90d rotation + HSM-backed where available | OpenBao + KMS |
| Tenant identifier (hashed) | `SENSITIVE_PIPA_ART23` | High | salted-hash; raw mapping in OpenBao | OpenBao tenant-resolver |
| Audit-chain seal records (per invocation event) | `AUDIT` | High | append-only; immutable | audit-chain µservice |
| Provider credential | `SECRET` | Critical | **NOT** stored in runtime | `foundry-providers` µservice only |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human; Workflow Studio user) | Untrusted external | OIDC + MFA | Author capability descriptor; invoke own tenant's capabilities; read own sessions |
| Customer application (machine) | Untrusted external | Per-tenant API key (rotated 30d) | Invoke own tenant's capabilities |
| Workload µservice (workflow-engine; in same trust domain) | Semi-trusted internal | mTLS + SPIFFE | Dispatch capability invocations on behalf of tenant principal |
| Runtime pod (`oya-foundry-runtime-capability-executor-app`) | Trusted internal | SPIFFE `spiffe://oyatie/foundry-runtime/<pod>` | Execute capability invocations; read registry cache; read/write session-state; emit events |
| foundry-supervisor (sibling) | Trusted internal | SPIFFE | Push CapabilityRegistryUpdated events; refuse non-supervisor traffic |
| foundry-providers (sibling) | Trusted internal | SPIFFE | Accept ProviderInvoker calls; hold LLM credentials |
| foundry-guardrails (sibling) | Trusted internal | SPIFFE | Accept GuardrailChecker calls |
| foundry-evidence (sibling) | Trusted internal | SPIFFE | Accept EvidenceEmitter calls; seal events |
| Reviewer agent | Trusted internal | OIDC-bound CI identity | Refuse merges that violate gate |
| Council-architecture / ops-security operators | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin-level access; JIT elevation |
| External auditor (SOC 2 / ISO / EU AI Act notified body) | Read-only external on bounded window | OIDC + MFA + JIT short-lived token | Read scoped subsets |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness |
| Attacker — prompt-injection (in-band) | Adversarial input | indistinguishable from legitimate input | Embeds adversarial content in payload to break capability semantics or exfiltrate sibling/session data |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure capability descriptors, autonomy ceilings, runbook steps |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat for confidentiality |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood; impact; risk score; mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Tenant-A submits invocation request impersonating Tenant-B**
- Asset: capability-executor REST endpoint
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: OIDC bearer carries bound tenant_id; X-Scope-OrgID header must match; mismatch returns 401 + `oya_tenant_spoofing_attempt_total` emission; Cedar policy at boundary refuses cross-tenant resource access.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); pack-kr KR PIPA Art. 29

**T-S-02 — Adversary stands up a workload claiming runtime SPIFFE identity**
- Asset: runtime SPIFFE identity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: SPIRE issues SVIDs bound to pod identity (UID + namespace + service-account); cluster admission controllers block unauthorised pod deployments; sibling µservices validate the issuing SPIRE server identity.
- Owner: ops-security + axis-foundry-runtime
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.15, A.8.3, A.8.7

**T-S-03 — Attacker forges CapabilityRegistryUpdated event to inject malicious capability**
- Asset: capability-registry-cache (would mirror a forged descriptor and start dispatching against it)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: events from foundry-supervisor signed with Ed25519; signature verified at runtime-side consumer; unsigned/invalid signature events dropped + audit-emitted; supervisor's signing key 90d rotation.
- Owner: axis-foundry-runtime + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.2; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)(c)

**T-S-04 — Attacker impersonates sibling µservice (e.g., foundry-providers) to inject false provider responses**
- Asset: ProviderInvoker port
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: mTLS + SPIFFE identity verification on both sides; runtime refuses connections from non-`foundry-providers` SPIFFE identities; certificate pinning via SPIRE-issued root.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.7

**T-S-05 — Attacker impersonates an external auditor's JIT token**
- Asset: Auditor read scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: per `policy/auditor-scope.cedar` — engagement-window enforcement; tenant-scope subset; TTL ≤4h; non-renewable without ops-security re-issue.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.5.17, A.5.18, A.8.2, A.8.3

### Tampering (T)

**T-T-01 — Capability descriptor tampering at the registry mirror (Postgres row mutation)**
- Asset: `capability_mirror` table
- Likelihood: L / Impact: H (malicious descriptor → arbitrary provider calls) / Risk: **M**
- Mitigations: mirror is replicated from foundry-supervisor as the source-of-truth; runtime cannot write to mirror table outside replication path; replication integrity verified by row-level signature check at read time; tampering detection emits `oya_capability_mirror_signature_invalid_total`; failed signature → descriptor blacklisted + dispatch refused.
- Owner: axis-foundry-runtime + cloud-secrets (Postgres infrastructure)
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.32, A.8.33; GDPR Art. 32(1)(b)

**T-T-02 — Session-state tampering via Valkey admin path**
- Asset: Valkey cluster
- Likelihood: L / Impact: H (cross-session leakage, false conversation memory injection) / Risk: **M**
- Mitigations: Valkey ACL — `default` user disabled; per-app role with prefix-scoped commands only; admin commands (FLUSHDB / CONFIG SET) require JIT OpenBao elevation + 2-person rule; session writes carry per-turn HMAC validated by session-state usecase at read-time; HMAC mismatch quarantines the session.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.17, A.8.7, A.8.12

**T-T-03 — Invocation lifecycle record tampering (insider mutates audit row)**
- Asset: `invocation_lifecycle` Postgres table
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: lifecycle records sealed by audit-chain Ed25519 + Merkle inclusion; Postgres row carries seal hash; tamper detected at read time + audit-chain reconstruction; admin writes require JIT + 2-person rule.
- Owner: axis-foundry-runtime + audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)(c)

**T-T-04 — Capability descriptor in-flight tampering between supervisor and mirror**
- Asset: replication path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: replication over mTLS; descriptors signed by supervisor's Ed25519; signature stored alongside row; runtime verifies signature on every cache load.
- Owner: axis-foundry-runtime + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.17, A.8.7

**T-T-05 — Autonomy-tier ceiling mutation at tenancy → cache replication**
- Asset: tenant autonomy ceiling cache
- Likelihood: L / Impact: H (silent privilege escalation) / Risk: **M**
- Mitigations: ceiling carries Ed25519 from tenancy µservice; runtime refuses ceiling row without valid signature; ceiling refresh emits `TenantTierCeilingChanged` audit-chain record.
- Owner: axis-foundry-runtime + tenancy
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.17, A.8.7; EU AI Act Art. 14 (human oversight) — tier ceiling is the human-oversight control

### Repudiation (R)

**T-R-01 — Tenant denies authorship of a capability invocation**
- Asset: invocation lifecycle record
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: every invocation carries OIDC bearer + actor binding + audit-chain Ed25519 seal; per-changeset evidence at `microservices/intelligence-runtime/evidence/multispectrum/*.json` git-committed; commit signed.
- Owner: axis-foundry-runtime + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Arts. 5(2), 30

**T-R-02 — Autonomy violation refused but actor denies attempt**
- Asset: `AutonomyViolationDetected` event
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations: violation event carries actor SPIFFE + tenant + capability_id + requested_autonomy_level + ceiling_at_check_time; sealed by audit-chain; immutable.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.8.15

**T-R-03 — Rollback of capability version executed without traceable trigger**
- Asset: capability mirror rollback path
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: capability version rollback is mediated by foundry-supervisor (not runtime); runtime consumes the rollback event; the rollback event is sealed at supervisor; runtime emits `CapabilityRollbackObserved` upon mirror update.
- Owner: axis-foundry + axis-foundry-runtime
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.8.15

### Information Disclosure (I)

**T-I-01 — Cross-tenant session leak via prefix-scope misconfiguration**
- Asset: Valkey session-state cluster
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: SessionStore implementation enforces tenant-prefix on every Valkey op; integration test asserts cross-tenant reads return empty; LEAN check `oya-check-session-prefix-isolation` greps for any unprefixed Valkey call; per-tenant Valkey ACL prevents prefix bypass.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3, A.8.12; GDPR Arts. 5(1)(f), 25, 32

**T-I-02 — Session conversation history leakage via prompt-injection in input payload**
- Asset: per-session conversation history
- Likelihood: H (LLM apps regularly susceptible per OWASP LLM01) / Impact: H / Risk: **H**
- Mitigations: foundry-guardrails called BEFORE provider dispatch; guardrails strip injected instructions + flag adversarial-pattern matches; session-state never echoed back into a different tenant's invocation (cross-session contamination check); per-turn output passes guardrails again before persistence; OWASP LLM01 + LLM02 + LLM06 mitigations applied at dispatch time.
- Owner: axis-foundry-runtime + foundry-guardrails
- Residual: M (adversarial-input baseline never fully eliminated)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12, A.8.32; GDPR Arts. 5(1)(c), 25, 32; EU AI Act Arts. 15 (accuracy + robustness + cybersecurity); OWASP LLM Top 10 LLM01/LLM02/LLM06

**T-I-03 — Runtime pod exposes provider credentials via memory dump or env leak**
- Asset: Provider credentials
- Likelihood: H (engineers regularly leak via logs / coredumps) / Impact: H / Risk: **H**
- Mitigations: **Provider credentials NEVER resident in runtime pod** (architectural separation; foundry-providers µservice holds credentials and exposes invocation surface to runtime over mTLS); runtime asks foundry-providers to call provider with opaque session token; coredumps disabled in production runtime pods; log redactor; coredump scan in CI; secret-scanner sweeps every commit + log emission. Provider-credential isolation e2e test verifies no provider secret materialises in runtime memory.
- Owner: ops-security + foundry-providers + axis-foundry-runtime
- Residual: L (architectural mitigation makes this defence-in-depth strong)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32(1)(a)(b)(c)(d); EU AI Act Art. 15 (cybersecurity)

**T-I-04 — Capability descriptor with PII in default-prompt fields exposes data on hot-reload**
- Asset: Capability descriptor text
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: descriptor authoring at foundry-supervisor enforces `data_class` annotation per field; PII-class fields excluded from runtime mirror; runtime never logs descriptor content; descriptor diff in audit-chain redacts annotated fields.
- Owner: axis-foundry-runtime + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.11, A.8.12

**T-I-05 — Invocation step trace exposes tenant data downstream**
- Asset: Step trace emitted to foundry-evidence
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: trace payload carries `data_class` per field; foundry-evidence applies redaction at ingest; downstream observability dashboards mask `data_class=PII` at view time even if persisted; per-tenant scoping enforced at evidence read path (Cedar).
- Owner: axis-foundry-runtime + foundry-evidence
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15; GDPR Arts. 5(1)(c), 25, 32

**T-I-06 — Autonomy ceiling exposed cross-tenant via metric labels**
- Asset: `oya_foundry_runtime_autonomy_ceiling{tenant=...}` metric
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: ceiling metric tagged with hashed tenant id only; never raw; cross-tenant dashboards aggregate without per-tenant label; Cedar refusal on direct query.
- Owner: axis-foundry-runtime
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12

### Denial of Service (D)

**T-D-01 — Capability dispatch burst flood (single tenant or platform-wide)**
- Asset: Runtime pool capacity
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations: per-tenant dispatch rate limits (per autonomy tier); per-capability concurrency cap; HPA on runtime-pool with hard ceiling; circuit-breakers fail-fast against slow providers; reject-on-saturation returns 429 not silent backlog; capacity-model.md sizing.
- Owner: axis-foundry-runtime + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Valkey cluster overload via session-state burst writes**
- Asset: Valkey session-state cluster
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: per-tenant rate limits on session-state ops (max ops/sec); Valkey cluster horizontally sharded; replication factor 1 primary + 1 replica per shard; loss of one replica does not break ingest; backpressure → executor returns "session-state-busy" rather than crashing.
- Owner: axis-foundry-runtime + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14

**T-D-03 — Long-running invocation starves pool (single invocation blocks pod)**
- Asset: Runtime pod
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: per-invocation timeout per capability descriptor (default 60s; max 300s); TimeoutClock port enforces; expired invocations emit `InvocationFailed{reason=timeout}` + free pod; pod-level concurrent invocation cap.
- Owner: axis-foundry-runtime
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-04 — Capability registry mirror replication storm overloads runtime cache**
- Asset: capability-registry-cache
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: cache update is incremental (per-row); supervisor emits diff events not full snapshots; runtime applies updates in a bounded batch; tail-end events ignored (debounce).
- Owner: axis-foundry-runtime + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30

**T-D-05 — Pool drain abuse (drain triggered too frequently)**
- Asset: runtime-pool drain primitive
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: drain requires SPIFFE-bound supervisor identity OR JIT operator approval; drain rate-limited (max 1 drain per pod per 5 min); cluster-wide drain requires 2-person rule.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.30, A.8.4

### Elevation of Privilege (E)

**T-E-01 — Capability requested above tenant autonomy ceiling silently executed (autonomy bypass)**
- Asset: AutonomyGate port
- Likelihood: M / Impact: H (silent tier escalation) / Risk: **H**
- Mitigations: AutonomyGate is the FIRST step in usecase dispatch; refusal emits `AutonomyViolationDetected` + 403 to caller; ceiling cache read carries signature validation per T-T-05; LEAN check `oya-check-autonomy-gate-presence` asserts gate invoked before provider call.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.3, A.8.4; EU AI Act Art. 14 (human oversight); ADR-0022 enforced

**T-E-02 — Runtime escape: capability code executes outside the pod boundary**
- Asset: Pod isolation
- Likelihood: L (capability code is descriptor-only in M01 per PRD OQ#5) / Impact: H / Risk: **M**
- Mitigations: M01 disallows tenant-supplied code execution (PRD OQ#5 resolved disallow); when sandbox decided subsequent-to-M01-completion, gVisor / Firecracker / WASM isolate per per-tenant autonomy ceiling; today's pod has seccomp + AppArmor + non-root + read-only FS; egress network policy default-deny except sibling SPIFFE-validated endpoints.
- Owner: ops-security + axis-foundry-runtime
- Residual: L (descriptor-only invariant is the load-bearing control in M01)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.4, A.8.7; EU AI Act Art. 15 (cybersecurity)

**T-E-03 — Pod-level privilege escalation via container escape (CVE)**
- Asset: container runtime
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: pod runs as non-root + read-only FS + seccomp default + AppArmor `runtime/default`; container image base re-built weekly with current OS patches; CIS Kubernetes Benchmark applied; Trivy + Grype container scan; signed images via Cosign.
- Owner: ops-security + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.7

**T-E-04 — Cedar policy escape via crafted descriptor field**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: Cedar v4 (no known escape vectors); fragments fuzzed at CI time; field input lengths bounded at REST boundary; oversized inputs rejected pre-Cedar.
- Owner: axis-foundry-runtime + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — Operator-level Valkey or Postgres admin command used to delete tenant sessions**
- Asset: Valkey FLUSHDB / Postgres DROP
- Likelihood: L (insider) / Impact: H / Risk: **M**
- Mitigations: admin commands JIT-only via OpenBao; 2-person rule; every admin call audit-emitted; mass-deletion anomaly metric; soft-delete window 30d.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4, A.8.16; GDPR Art. 17

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Session-state across invocations | Linkable per session_id even with hashed tenant; multi-session linkability via shared identifier patterns | session_id rotation per session; cross-session correlation requires `data_class=PII_QUASI` annotation | M |
| T-L-02 | Identifiability | Hashed tenant_id | sha256(tenant_id ++ salt)[..16] re-identifiable via auxiliary data | salt rotation 12mo; small-tenant cardinality protection | L |
| T-L-03 | Non-repudiation | Tenant invocation authorship | Tenant may deny invocation lineage | Signed commits + OIDC + audit-chain | L |
| T-L-04 | Detectability | Invocation timing | Burst patterns correlate with tenant business events | Tenant consent at onboarding (telemetry necessary for SLA) | M |
| T-L-05 | Disclosure | Step trace fan-out to foundry-evidence | Trace exposes per-call working data | data_class redactor + Cedar tenant-scope on evidence reads | L |
| T-L-06 | Unawareness | Tenant's end-user (consumer of tenant's product) | Unaware their interaction triggers runtime invocation containing their identifiers | Joint-controllership in tenant DPA cascades disclosure | M |
| T-L-07 | Non-compliance | Right-to-erasure on end-user across multiple sessions / pack regions | DSR cascade per `data-residency.md`; soft-delete within 30d | M (subject to retention windows) |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| AutonomyGate as first step in usecase dispatch | Preventive | axis-foundry-runtime | `oya-check-autonomy-gate-presence` lane |
| Provider credentials never resident in runtime pod | Architectural | foundry-providers + axis-foundry-runtime | e2e test `provider-credential-isolation` |
| Valkey tenant-prefix scoping enforced by SessionStore | Preventive | axis-foundry-runtime | `oya-check-session-prefix-isolation` |
| mTLS + SPIFFE on every sibling call | Preventive | ops-security | Istio mesh telemetry |
| Cedar tenant-scope on REST endpoints | Preventive | axis-foundry-runtime + ops-security | Cedar fuzz lane |
| Capability descriptor signature validation on cache load | Preventive | axis-foundry-runtime | `oya_capability_mirror_signature_invalid_total > 0` alert |
| Ed25519 audit-chain seal on every invocation event | Detective + Non-repudiation | audit-chain | audit-chain regression tests |
| Per-tenant + per-capability rate limits | Preventive (DoS) | axis-foundry-runtime | rate-limit metrics |
| Pod seccomp + AppArmor + non-root + RO FS | Preventive | ops-security + cloud-k8s | container security audit |
| Per-invocation timeout enforcement (TimeoutClock) | Preventive | axis-foundry-runtime | timeout metric |
| Foundry-guardrails called before provider dispatch | Preventive (LLM injection) | foundry-guardrails | dispatch flow assertion |
| Drain primitive emits InvocationCancelled per parked invocation | Detective + Recovery | axis-foundry-runtime | drain runbook |
| 2-person rule for admin ops + JIT elevation | Preventive (insider) | ops-security | OpenBao JIT logs |
| DSR cascade in session-state worker | Preventive (compliance) | council-privacy | DSR queue dashboard |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-02 (prompt-injection) | M | Adversarial-input baseline cannot be fully eliminated; guardrails are load-bearing | Quarterly |
| T-L-01 (linkability across invocations) | M | Inherent to session semantics; mitigated to acceptable by data_class + sampling | Annually |
| T-L-04 (detectability via timing) | M | Tenant business reality; consent at onboarding covers | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant responsibility; joint-controllership | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Subject to retention windows; DSR cascade is best-effort | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks engaged: KR-ISMS-P + KR PIPA + KR 전자문서법 + KR FSC AI Guideline 2024.

- **KR PIPA Art. 23**: hashed tenant id sensitive; salt rotation per T-L-02.
- **KR PIPA Art. 29**: every T-*-NN mitigation maps to one of the 12 prescribed safeguards.
- **KR FSC AI Guideline 2024 §3 (human-in-loop)**: AutonomyGate satisfies; T1+ tiers require explicit per-invocation acknowledgement; FSC notification on autonomy violation > 0 per tenant per week.

### pack-us-healthcare (HIPAA-scoped)

- **HIPAA §164.312(a)(1)**: Cedar + Valkey tenant-prefix + Postgres RLS satisfies; runtime as Business Associate.
- **HIPAA §164.312(b)**: audit-chain on every invocation; retention ≥ 6y for PHI-touching sessions.
- **HIPAA §164.502(b)**: data_class minimum-necessary redaction at step trace emission.
- **FDA SaMD pre-market**: if a tenant capability is clinical decision support, capability descriptor must carry FDA classification tag; runtime refuses unclassified clinical capabilities in pack-us-healthcare.

### pack-eu (GDPR + EU AI Act + NIS2)

- **GDPR Arts. 25 + 32**: every mitigation contributes to risk-appropriate posture.
- **EU AI Act Art. 9 (risk management)**: this threat model + DPIA are the foundational artifacts.
- **EU AI Act Art. 10 (data and data governance)**: data_class taxonomy + retention + tenant-scope enforcement.
- **EU AI Act Art. 13 (transparency)**: capability descriptor includes purpose-of-use + autonomy tier; tenant DPA cascades to data subjects.
- **EU AI Act Art. 14 (human oversight)**: AutonomyGate is the human-oversight technical control.
- **EU AI Act Art. 15 (accuracy + robustness + cybersecurity)**: foundry-guardrails + circuit-breakers + per-invocation timeout + provider-credential isolation.
- **NIS2**: incident reporting timelines (24h + 72h + 1mo) in `incident-response.md`.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-runtime-overlay.md`; each follows this structure with local citations.

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered inline per row above | `microservices/intelligence-runtime/compliance.md` |
| ISO 27001:2022 | Annex A.5–A.8 controls covered inline | `compliance.md` |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `dpia.md` + `compliance.md` |
| EU AI Act | Arts. 9, 10, 13, 14, 15 cited inline (high-risk system posture) | `dpia.md` + `compliance.md` |

## Re-review Triggers

- Any change to trust boundary diagram (new boundary, removed boundary).
- Any LTS upgrade (Valkey / Postgres / Istio / SPIRE) with security-relevant release notes.
- Any new pack activation.
- Annual scheduled review (Q2).
- Post-incident review (Sev-1 or Sev-2 in foundry-runtime or any siblings).
- Pen-test or audit finding.
- Each capability autonomy tier elevation (per-tenant ceiling raise).

## References

- ADR-0022 (autonomy tiers); ADR-0024 (eval harness); ADR-0025 (runtime consolidation); ADR-0028 (audit chain — Bominal inherited); ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum); ADR-0117 (residency); ADR-0139 (SLO gate); ADR-0131 (flat layout); ADR-0140 (Cedar policy).
- `microservices/intelligence-runtime/PRD.md`.
- `microservices/intelligence-runtime/dpia.md`.
- `microservices/intelligence-runtime/compliance.md`.
- `microservices/intelligence-runtime/policy/{runtime-isolation, data-residency}.md`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021); OWASP API Top 10 (2023); OWASP Top 10 for LLM Applications (2025).
- MITRE ATLAS — `atlas.mitre.org`.
- NIST SP 800-154 (data-centric threat modeling).
- EU AI Act (Regulation 2024/1689).
