---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry
deciders: council-architecture, ops-security, axis-foundry, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP LLM Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0025, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every adapter / vendor change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.8.2, A.8.3, A.8.5, A.8.11, A.8.12, A.8.16, A.8.20, A.8.21, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27"
  - "GDPR Arts. 5, 6, 25, 28, 32, 44-50"
  - "EU AI Act Reg. (EU) 2024/1689 Arts. 13, 14, 50"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 17/18/23/28/29", "KR ISMS-P §2.6 (외부 시스템 보안)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 (BAA + risk analysis)", "§164.312(e) (transmission security)"]
  pack-eu: ["GDPR Arts. 28 + 32 + 44-50 (SCC + transfers)", "EU AI Act Arts. 50 (transparency)"]
  pack-jp: ["APPI Arts. 23 + 24 (cross-border transfer)"]
  pack-sg: ["PDPA §26 (transfer limitation) + MAS-TRM v2021"]
  pack-au: ["Privacy Act 1988 APP 8 (cross-border)"]
  pack-in: ["DPDPA 2023 §16 (cross-border)"]
  pack-br: ["LGPD Art. 33 (international transfer)"]
  pack-ae: ["UAE PDPL Art. 22 (cross-border)"]
  pack-ksa: ["PDPL Art. 29 (cross-border) + SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: foundry-providers µservice

## Purpose

Identify, classify, and mitigate threats to the foundry-providers µservice's confidentiality, integrity, availability, privacy, and AI-supply-chain posture. This µservice is the **sole egress path** to third-party foundation-model vendors for every oyatie product; a compromise here exfiltrates tenant data to the vendor or to an adversary who has impersonated the vendor. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, and (for EU tenants) EU AI Act conformity assessors at first-tenant onboarding.

## Scope

### In-scope

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres (provider-config persistence) | `oya-foundry-providers-router-*` (9 crates) |
| Valkey (rate-limit / token-bucket state) | `oya-foundry-providers-adapter-*` (8 crates per-vendor + transport) |
| OpenBao agent socket (credential resolution) | provider-router decision algebra |
| Upstream proxy fleet (mTLS egress) | per-vendor BLAKE3+Ed25519 envelope |
| Istio service mesh (intra-cluster mTLS) | adapter-substitution defence (signed crate digests) |
| OTel collector (telemetry to observability) | per-tenant Cedar policy gates |

### Out-of-scope

- Threats to OpenBao itself — owned by the `cloud-secrets` µservice's threat model. This document inherits OpenBao threats as upstream.
- Threats to the underlying Kubernetes cluster — owned by `cloud-k8s` threat model.
- Threats to the workload µservices that call foundry-providers — each owns its own threat model.
- Threats to upstream vendor infrastructure (api.anthropic.com, api.openai.com, generativelanguage.googleapis.com) — out of oyatie control; mitigated by behavior-monitoring (provider-health-monitor) and adapter-version-pin runbook.

## Trust Boundaries

```text
┌─ Workload cluster (cell µservice's network) ──────────────────────────────┐
│                                                                            │
│  foundry-runtime ─── mTLS ──▶ provider-router-rest ─┐                      │
│                                                     │                      │
│                                Trust boundary 1     │                      │
│                                (mesh ingress)       │                      │
│                                                     ▼                      │
│  ┌─ provider-router (in-process) ───────────────────────────────────────┐  │
│  │                                                                     │  │
│  │  Cedar policy gate ──▶ residency check ──▶ capability fit ──▶ pick  │  │
│  │                                                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                  │                                         │
│           Trust boundary 2       │                                         │
│           (router → adapter)     ▼                                         │
│  ┌─ adapter-<vendor>-<transport> (in-process; per-vendor crate) ───────┐   │
│  │                                                                    │   │
│  │   ──▶ openbao-bridge.resolve(SecretReference)  (in-memory)         │   │
│  │   ──▶ build upstream HTTP request (credential in Authorization)    │   │
│  │   ──▶ BLAKE3 hash + Ed25519 envelope                               │   │
│  │                                                                    │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                  │                                         │
│           Trust boundary 3       │  mTLS to vendor edge                    │
│           (egress to vendor)     ▼                                         │
└──────────────────────────────────│─────────────────────────────────────────┘
                                   ▼
┌─ Internet ────────────────────────────────────────────────────────────────┐
│   api.anthropic.com  /  api.openai.com  /  generativelanguage.googleapis  │
│       claude.ai (subscription)  /  chatgpt.com  /  gemini.google.com      │
└────────────────────────────────────────────────────────────────────────────┘

┌─ Dedicated cloud-secrets cluster ─────────────────────────────────────────┐
│                                                                            │
│   OpenBao primary + replicas (per pack)                                   │
│       (resolved exclusively via agent socket inside the adapter pod)      │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## Actors

| Actor | Role | Posture |
|---|---|---|
| Workload µservice principal | calls `provider-router` | OIDC + SPIFFE; tenant-id-scoped |
| Tenant operator | manages per-tenant provider config + credentials | OIDC + Cedar policy |
| ops-security on-call | rotates credentials; investigates breach | OpenBao admin (2-person rule) |
| axis-foundry on-call | adapter incident response | mesh-internal admin |
| Vendor edge (Anthropic / OpenAI / Google) | upstream API | mTLS; pinned cert |
| Adversary (external) | attempts credential exfil / response tampering | none — modelled below |
| Adversary (internal supply chain) | attempts adapter-substitution attack | none — modelled below |

## Threats (STRIDE + OWASP LLM + LINDDUN)

### T-01: Credential theft (Spoofing + Information Disclosure; OWASP LLM06)

**Vector.** An adversary obtains a tenant's vendor API key or subscription cookie via:
- Adapter code logging credential bytes (developer error).
- Credential leak to structured logs / OTel trace span attributes / Grafana panel / agent-chat-window / git-commit / build-log / error message.
- Stolen OpenBao token from a compromised pod.
- Side-channel via the proxy fleet (TLS interception by adversary CA).

**Mitigations.**
1. **Code-path isolation.** `ResolvedCredential` is an in-process-only opaque type with `Debug`+`Display` impls that emit `***REDACTED***`; `Drop` zeroises memory. Conformance verified by `oya-foundry-providers-credential-isolation` LEAN lane (Slice D + crate `oya-check-credential-isolation`): grep `cargo expand` output for credential-emitting code paths and structural patterns; zero-occurrence required.
2. **No-serde rule.** `ResolvedCredential` deliberately does NOT impl `Serialize`/`serde::Serialize`/`Display`; any attempt to include it in JSON/logs/spans is a compile-time error.
3. **Per-call credential lifetime.** Credentials are resolved just-in-time per upstream HTTP call and dropped immediately; no caching beyond the OpenBao agent lease.
4. **mTLS egress with pinned CA.** Adversary cannot MITM upstream vendor edges; pinned vendor cert per adapter crate.
5. **OpenBao token-tightening.** The pod-bound OpenBao token has only `read` scope on `openbao://<pack>/<tenant>/providers/*`; no `list`/`write`.
6. **Audit-chain emission.** Every credential resolution emits `CredentialResolved(tenant, vendor, lease_id, hash_of_caller_ctx)` event so unusual resolution patterns surface in `observability`.
7. **Repo-wide pre-merge sweep.** `oya-check-no-raw-credentials` LEAN lane greps the entire merged diff for credential-shaped strings (Anthropic / OpenAI / Google key signatures); BLOCKER lane.

**Residual risk.** Vendor-side token-exposure (e.g., vendor breach) is upstream; mitigated by rotation (runbook `credential-rotation`).

### T-02: Rate-limit abuse / cost-overrun (Denial of Service + Repudiation)

**Vector.** A tenant (intentionally or via a compromised workload µservice) drives provider calls past the vendor's published rate limit, incurring upstream throttles or vendor cost overruns.

**Mitigations.**
1. **In-process token bucket** per (tenant, vendor) keyed in Valkey (per-pack sentinel HA); rejected calls return 429 before any upstream HTTP attempt.
2. **Per-tenant per-vendor cost ceiling** (configured by tenant operator; default capped at 100× median); ceiling breach emits `CostCeilingBreached` event + opens `#inc-` channel.
3. **Vendor-published rate-limit floor** enforced at adapter level even when tenant config requests higher.
4. **Backpressure to provider-router** when sustained throttling detected; router demotes the vendor for the affected tenant for a configurable cool-down (default 5 min) and routes to next-best.

**Residual risk.** Vendor-side noisy-neighbour effects across tenant boundaries upstream; mitigated by per-pack provider segregation where vendors support it.

### T-03: Provider response tampering (Tampering)

**Vector.** An adversary positioned between the adapter and the vendor edge tampers with the response body (e.g., injecting a tool-use instruction that exfiltrates data).

**Mitigations.**
1. **TLS with pinned vendor CA** — single-cert pin; any cert rotation triggers explicit adapter version bump.
2. **BLAKE3 content hash** computed at the adapter on the raw response bytes immediately after read; the hash is sealed in the `ProviderInvoked` event alongside the request hash; downstream consumers verify the seal before honouring the response.
3. **Ed25519 envelope.** The adapter's per-pod Ed25519 signing key signs `(request_hash || response_hash || metadata)`; only Ed25519-valid events are accepted by `foundry-evidence`.
4. **Response-shape conformance check.** Each adapter has a per-vendor `ResponseValidator` that asserts the response matches the documented schema; deviation triggers `ResponseShapeAnomaly` event + adapter-quarantine.
5. **Tool-use sandboxing.** Tool calls returned by the vendor are NOT executed by the adapter; they are emitted as proposals and the workload µservice's own tool-use authority (per `cell`'s Cedar policy) decides.

**Residual risk.** Compromise of the vendor's own infrastructure is upstream; mitigated by behavior-monitoring (response-shape anomaly + cost anomaly) + adapter-version-pin runbook for fast rollback.

### T-04: In-house-model rollout regression (Tampering + Repudiation)

**Vector.** A blue/green rollout of an ADR-0026 in-house model degrades response quality silently; the router continues sending tenant traffic to it because the SLI is not yet wired or the threshold is mis-tuned.

**Mitigations.**
1. **Burn-rate-driven router demote.** Provider-health-monitor publishes `oya_foundry_providers_provider_health{vendor="in-house"}` rolling SLIs; router auto-demotes in-house when burn-rate exceeds threshold (default: 14.4× over 1h).
2. **Canary cohort weighting.** In-house adapter is initially routed only for `(tenant.canary_cohort=true) AND (capability_fit_score ≥ floor)`; ramp 1 % → 10 % → 50 % → 100 % per `iac/helm/provider-router/values.yaml`.
3. **Mandatory parity tests** (golden set) per ADR-0026 §"rollout gates"; in-house adapter cannot be activated for a tenant unless its golden-set verdict-correctness ≥ 0.95 of incumbent.
4. **Tenant opt-out flag** per-tenant Cedar fragment; tenant operator can pin "no in-house" at any time.
5. **Runbook `in-house-model-rollback`.** One-command revert to last-green provider for the tenant.

**Residual risk.** Subtle quality regression below SLI sensitivity; mitigated by eval-set per capability + per-quarter golden-set refresh.

### T-05: Adapter-substitution attack (Elevation of Privilege + Tampering; OWASP LLM05 supply-chain)

**Vector.** An adversary replaces an adapter crate in the dependency graph (e.g., via a malicious dependency, a typo-squat, or a compromised maintainer) with one that emits credentials or tampered responses.

**Mitigations.**
1. **Cargo workspace lockfile is BLOCKER-gated.** `cargo deny check` lane refuses any unknown registry; only crates.io + internal mirror permitted.
2. **Per-crate digest pinning.** `Cargo.lock` is signed (Ed25519) and lockfile hash recorded in audit-chain on every promotion.
3. **Adapter crate provenance.** Each adapter crate publishes its Sigstore attestation (build-from-source + SBOM); `oya-foundry-providers-adapter-digest-verified` lane checks adapter crate digests against the audit-chain-pinned list at deploy time.
4. **Mesh-level adapter identity.** The adapter pod's SPIFFE identity is checked by `foundry-evidence`; only known adapter identities can sign events.
5. **Quarterly threat-modelling review** of the adapter dependency graph; any new transitive dependency triggers a manual review.

**Residual risk.** Sufficiently determined supply-chain attack with maintainer compromise; mitigated by 2-person rule for any adapter-crate publish.

### T-06: Subscription-channel substitution (Elevation of Privilege)

**Vector.** Subscription transports (Claude Pro/Max, ChatGPT Plus, Gemini Advanced) drive a session-cookie-based channel that the vendor can change unilaterally. An adversary observing the channel change might MITM the new flow.

**Mitigations.**
1. **Adapter-version-pin runbook.** Tenants can pin to a specific adapter version; vendor channel changes do not break pinned tenants until they explicitly upgrade.
2. **Subscription-cookie scope minimisation.** Cookies stored in OpenBao with `read` scope on a per-pod-per-call lease; never persisted in the adapter.
3. **Channel-shape conformance check.** The subscription adapter validates response shape on every call; deviation triggers adapter-quarantine + on-call page.
4. **Tenant transparency.** Tenants opting for subscription transport are explicitly informed of the FRAGILE-vs-API trade-off in their onboarding docs.

**Residual risk.** Subscription channel may go down for hours without notice; tenants are encouraged to use API transports where business allows.

### T-07: Prompt-injection / data-exfil via tool calls (OWASP LLM01 + LLM02 + LLM06)

**Vector.** A model response includes a tool-use instruction that, if executed, would exfiltrate tenant data outside the residency pack.

**Mitigations.**
1. **Adapter never executes tool calls.** Tool calls are emitted as proposals; `cell`'s policy plane decides.
2. **Per-tool Cedar policy.** Tool-execution decisions are gated by Cedar fragments; cross-pack data flow is default-deny.
3. **Audit-chain emission** of every tool-call proposal so suspicious patterns are detectable.
4. **Tenant operator visibility.** Per-tenant dashboard surfaces tool-call counts + denied tool-call attempts (panel in `dashboards/provider-call-rate.json`).

### T-08: Cross-pack data leakage (Information Disclosure; LINDDUN Linkability/Identifiability)

**Vector.** A pack-kr tenant's request is routed through a vendor edge in a non-permitted geography, exporting personal data outside KR jurisdiction without SCC.

**Mitigations.**
1. **Residency-aware router (FR-14).** Provider-router refuses any (pack × vendor × region) tuple not in `policy/data-residency.md`.
2. **Per-pack permitted-vendor matrix** in `policy/data-residency.md`; default-deny.
3. **EU AI Act Art. 50 disclosure** record emitted per call when jurisdiction is EU; tenant operator + DPO can audit.
4. **Per-call OTel `jurisdiction_code` tag.**
5. **`oya-foundry-providers-residency-conformance` LEAN lane.**

**Residual risk.** Vendor edge routing internally to another region after request leaves oyatie; mitigated by vendor-region-pinning + DPA terms.

### T-09: Self-DoS via mis-tuned router (Availability)

**Vector.** A tenant's Cedar policy or capability profile is mis-configured such that no provider matches; every request returns `NoProvider` and the tenant's workload halts.

**Mitigations.**
1. **Policy-validation at admission.** Tenant operator policy changes are dry-run against a canary capability set before they take effect.
2. **Default-allow fallback** is **explicitly disabled**; mis-configuration produces a deterministic deny rather than a silent route to an unintended vendor.
3. **Helpful error.** `NoProvider` error includes a structured diagnostic explaining which constraints failed (visibility for tenant operators).
4. **Per-pack reference baseline.** Tenant onboarding provides a baseline capability-profile template that always has at least one matching provider in the pack.

### T-10: Provider-router-rest cross-tenant authz bypass (Elevation of Privilege)

**Vector.** A workload µservice request injects a different tenant's `tenant_id` to invoke a foreign provider session.

**Mitigations.**
1. **SPIFFE-rooted tenant binding.** The workload µservice's SPIFFE identity is validated; the tenant claim must match the spiffe ID's tenant attribute.
2. **Cedar `tenant-scope.cedar` policy** with default-deny; explicit `forbid` rule for cross-tenant action.
3. **Audit-chain emission** of every router call with `principal.tenant_id` + `resource.tenant_id`; deny events surface in `observability`.

## Risk Acceptance Register

| Risk ID | Risk | Treatment | Owner | Review |
|---|---|---|---|---|
| RAR-FP-01 | Subscription-channel breakage on vendor UI change | accept + monitor + adapter-version-pin | axis-foundry | quarterly |
| RAR-FP-02 | Vendor-side breach exposes credentials | accept + rapid rotation runbook | ops-security | continuous |
| RAR-FP-03 | In-house model has subtle quality regression below SLI | accept + golden-set + tenant opt-out | axis-foundry | per-release |
| RAR-FP-04 | Adapter dep-graph supply-chain compromise | accept + sigstore + 2-person rule | ops-security | quarterly |

## Verification

- `cargo run -p oya-dev-cli -- gate validate credential-isolation --microservice foundry-providers` exits 0.
- `cargo run -p oya-dev-cli -- gate validate residency-conformance --microservice foundry-providers` exits 0.
- Per-quarter red-team exercise on T-01 + T-03 + T-05; postmortem in `evidence/red-team/`.
- Per-release adapter parity tests including golden-set verdict-correctness against incumbent providers.

## References

- ADR-0025, ADR-0026, ADR-0028, ADR-0117, ADR-0139, ADR-0131.
- OWASP LLM Top 10 (2023) — `owasp.org/www-project-top-10-for-large-language-model-applications`.
- NIST SP 800-154 — Guide to Data-Centric System Threat Modeling.
- EU AI Act Reg. (EU) 2024/1689.
- Bominal ADR-0028 (audit-chain).
