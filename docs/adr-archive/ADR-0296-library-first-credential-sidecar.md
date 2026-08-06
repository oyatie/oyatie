---
id: ADR-0296
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-intelligence
  - axis-policy-engine
  - axis-audit-chain
  - axis-identity
  - ops-sre-reliability
  - ops-compliance
supersedes: []
amends: []
requires_amendment_to:
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md (§D-2 library-first dispatch surface gains the sidecar key-holder primitive OR the ≤60s OpenBao token TTL constraint; in-process provider-credential caching prohibited beyond a single in-flight call; audit-signing key never resident in caller process memory)
  - ADR-0355-amendment-library-first-network-opt-in-clarification.md (§D-2 caller-process scope reduced — audit-signing key + provider credentials move out of caller process into the sidecar; ADR-0294 soak-window applicable to sidecar permit fragment publications)
  - ADR-0246-policy-engine-substrate-promotion.md (Cedar fragments touching tool-call permits must include a `sidecar_credential_handle_lifetime_ms` context attribute; `oya-check-library-first-credential-sidecar` lane added to coverage report)
superseded_by: [ADR-0709]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0050-event-bus-kafka.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0192-milvus-vector-database.md
  - ADR-0200-wasmtime-substrate.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0355-amendment-library-first-network-opt-in-clarification.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0293-governance-meta-trust-root.md
  - ADR-0294-cedar-fragment-soak-anomaly-rollback.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/intelligence.json
  - /specs/microservices/cloud-secrets.json
  - /specs/byok-credential-model.json
  - /specs/library-first-sidecar-uds-protocol.json
  - /specs/credential-handle-lifecycle.json
related_memory:
  - feedback_byok_everywhere_credentials
  - feedback_cedar_as_universal_gate
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_no_silent_regression
  - feedback_clean_architecture_requirements
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_intelligence_two_layer_substrate
  - feedback_substrate_vs_product_layering
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-4-of-4
authority_for_existence: docs/architecture/keystone-bundle-2026-05-20-synthesis.md §5.4
closes_findings:
  - F5-255-01 (Library-first dispatch concentrates credentials, CRITICAL)
  - ASC-01 (Library-first design = wide attack surface for in-process credentials)
  - CW-07 (Library-first credentials caching pattern unspecified for TTL/lifecycle)
naming_justifications:
  - name: oyatie.intelligence.credential-sidecar
    bnf_v4_1: tenant=`oyatie` (reserved-namespace) · sub_scope=`intelligence.credential-sidecar` (kebab-case, hyphenated, no underscores) · arity=3
    layer_enum_adr_0105: `substrate`
    rationale: Per-cell sidecar process holding credential + audit-signing key material; the sidecar is a substrate-tier service per ADR-0245 substrate-vs-product layering, not a product
  - name: oyatie.intelligence.credential-sidecar-attestor
    bnf_v4_1: tenant=`oyatie` · sub_scope=`intelligence.credential-sidecar-attestor` · arity=3
    layer_enum_adr_0105: `observability`
    rationale: Per-sidecar attestor that emits credential-handle lifetime telemetry to the observability substrate; not in the data path; observability-layer fit
  - name: oya-shared-credential-sidecar-uds
    bnf_v4_1: shared-domain crate per `feedback_glossary_shared_not_platform`; kebab-case
    layer_enum_adr_0105: `shared`
    rationale: Shared crate exposing the UDS protocol that callers use to invoke sidecar operations; consumed by every library-first caller
  - name: oya-check-library-first-credential-sidecar
    bnf_v4_1: gate-name convention `oya-check-<predicate>` per ADR-0212
    layer_enum_adr_0105: `gate`
    rationale: CI lane verifying every library-first caller links the sidecar UDS client (not direct OpenBao access) AND every credential handle has bounded lifetime
enforcement_status: advisory-until-credential-sidecar-deployed
enforced_by:
  - oya gate validate library-first-credential-sidecar
  - oya gate validate credential-handle-lifetime-bound
  - oya gate validate audit-signing-key-not-in-caller-process
  - oya gate validate openbao-token-ttl-bound
  - oya gate validate sidecar-uds-protocol-compliance
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0296: Library-First Credential Sidecar

## Status

Proposed — 2026-05-20.

Promotion-gate fix **4 of 4** for the keystone bundle 2026-05-20
(`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.4).
This ADR closes F5-Security finding **F5-255-01** (CRITICAL,
library-first dispatch concentrates LLM credentials + Cedar
evaluation + audit signing in every caller process) and ASC-01 +
CW-07. ADR-0255 (and its amendment) cannot promote from `Proposed`
to `Accepted` until this ADR's sidecar mechanics are implemented.

Enforcement is `advisory-until-credential-sidecar-deployed`. The CI
lanes that enforce this ADR become BLOCKER once:

1. The credential sidecar pod is deployed in every Tier-2 +
   Tier-3 cell and reports green heartbeats for ≥ 7 consecutive
   days.
2. Every library-first caller in the platform's ≈ 30+ Rust
   µservices links the `oya-shared-credential-sidecar-uds` crate
   instead of accessing OpenBao directly.
3. An end-to-end RCE-blast-radius rehearsal in
   `dev-tools-cell-staging` has verified that a simulated RCE in
   a low-trust µservice yields zero usable provider credentials
   and zero usable audit-signing keys (matches the F5-255-01
   exploit scenario).
4. `oya-check-library-first-credential-sidecar` lane scans every
   library-first caller and reports zero direct-credential-access
   findings.

## Date

2026-05-20.

## Context

### What F5-255-01 actually says

F5-Security's r1 verdict (CRITICAL) reads:

> ADR-0255-amendment §D-2 mandates that every caller's process
> holds: (a) Anthropic/OpenAI/Google/Bedrock provider credentials
> (resolved via OpenBao but cached in-process), (b) Cedar
> evaluator state for local evaluation, (c) audit-chain signing
> key (for in-process seal emission), (d) OTel propagation tokens,
> (e) tool-registry snapshot. The library is linked into every
> µservice in every cell. Attack surface: any vulnerability in
> any µservice that yields RCE grants the attacker the union of
> all linked secrets — every LLM provider API key, every audit-
> signing key, every tenant compliance-pack-overlay knowledge.

The exploit path:

1. Adversary identifies a CVE in `microservices/marketplace/` (the
   lowest-trust µservice in the platform, since it runs tenant-
   submitted plugin code adjacent).
2. RCE in the marketplace process yields read access to the
   process's memory.
3. Library-first design means provider credentials for Anthropic,
   OpenAI, Bedrock are all cached in the marketplace process
   (because marketplace calls the LLM substrate for plugin
   description summarisation, similarity ranking, content
   moderation, etc.).
4. The audit-signing key is also cached in the marketplace process
   (because marketplace emits its own audit rows per ADR-0145
   invariant 1).
5. Attacker exfiltrates the provider credentials. Now attacker can
   make LLM calls billed to oyatie's accounts (financial loss +
   abuse) AND can use the audit-signing key to forge audit rows
   (reputational + compliance loss).

F5 ranks this CRITICAL because:

1. **Blast radius is platform-wide.** A single CVE in any of the
   ≈ 30+ Rust µservices that link the library exposes platform-
   wide provider credentials. Per ADR-0145 invariant 1, every
   µservice that emits audit rows also holds the audit-signing
   key.
2. **The provider credentials are high-value.** They are billed-
   per-token to oyatie's accounts with major LLM vendors. An
   adversary can exfiltrate the credentials and use them to bill
   millions of dollars in LLM usage before detection.
3. **The audit-signing key is the keystone of the platform's
   integrity claims.** A forged audit row signed by the platform's
   audit-signing key passes every downstream verification (until
   the next Merkle seal, at which point the forgery may or may
   not be detected depending on the seal cadence per ADR-0251
   §D-1).

### What ASC-01 + CW-07 add

ASC-01 (attack-surface concern) and CW-07 (cryptographic weakness)
reinforce F5-255-01:

> The library-first decision (well-justified for availability)
> inverts the secret-handling pattern; every linker is a potential
> leakage point.

> Amendment §D-2 says credentials are resolved via shared-secret-
> reference but does not bound in-process TTL. Add ≤60s TTL +
> response-wrapping.

Both findings agree on the resolution shape: bound the in-process
lifetime of credential material to ≤60s OR remove it from the
caller process entirely via a sidecar.

### Why sidecar-OR-≤60s-TTL specifically

Five alternatives were considered:

| Alternative | Why rejected |
|---|---|
| **A. Status quo — library-first with in-process credential caching.** | Rejected by F5 verdict; CVSS 9.8 equivalent finding; CRITICAL. |
| **B. Pure mediator — every caller routes credential operations through a central Intelligence µservice over the network.** | Violates ADR-0255 amendment's library-first decision (which closed the universal-mediator anti-pattern). Reintroduces the network hop the library-first explicitly removed. |
| **C. ≤60s OpenBao token TTL (no sidecar).** | Bounds in-process exposure to ≤60s but does NOT solve the audit-signing-key concentration — the audit-signing key is platform-wide, not tenant-scoped, and cannot be issued as a 60s token. |
| **D. Sidecar key-holder pattern — co-located process that holds credentials + audit-signing key; callers invoke via UDS for sign + dispatch operations only.** | Selected for the audit-signing key + tenant-scoped credentials. Composes with ≤60s OpenBao TTL for provider credentials (which are issued just-in-time per call). |
| **E. WASM sandboxing for low-trust callers (in-cell intelligence-coordinator over network).** | Considered as additional hardening for the lowest-trust callers (marketplace, plugin runtime); folded into §D-7 RECOMMENDED hardening but not mandated for all callers because WASM sandboxing has performance + complexity costs not yet justified for the higher-trust callers. |

The selected resolution combines two patterns. Both are required:

1. **Audit-signing key sidecar.** A per-cell sidecar process holds
   the audit-signing key. Callers invoke `Sign(payload)` via a
   Unix Domain Socket (UDS); the sidecar returns the signature.
   The key never leaves the sidecar memory; callers never see it.
   RCE in any caller yields no access to the key.
2. **≤60s OpenBao token TTL for provider credentials.** Provider
   credentials (LLM API keys) are fetched per-call via OpenBao
   with a ≤60s token TTL and `response-wrapping` (the credential
   is single-use). RCE in any caller during an in-flight call
   yields at most the current call's credential, valid for ≤60s
   beyond exfiltration.

### Why both — not either-or

A naive reading of the synthesis doc could interpret §5.4 as
"sidecar OR ≤60s OpenBao TTL." The verdict is more nuanced:

- The audit-signing key is a single platform-wide key that must be
  available to thousands of callers per second. Issuing a fresh
  audit-signing key per call from OpenBao is operationally
  infeasible (Merkle-chain continuity would require complex
  state-sharing across rotated keys). The sidecar pattern is the
  only viable mitigation for the audit-signing key.
- Provider credentials are per-(provider, tenant) and rotate on
  vendor cadence (days to months). They CAN be issued as ≤60s
  OpenBao tokens via response-wrapping. The sidecar pattern is
  ALSO viable for provider credentials but adds latency; the
  ≤60s OpenBao TTL is the lighter-weight option.

The required combination:
- Audit-signing key → sidecar (always).
- Provider credentials → either sidecar OR ≤60s OpenBao token TTL
  + response-wrapping (caller chooses per workload).

Three named hyperscaler patterns inform the resolution:

- **AWS Nitro Enclaves (2020+).** Hardware-isolated execution
  environment for credential handling. AWS recommends Nitro
  Enclaves for any workload holding sensitive credentials; the
  enclave's memory is unreadable from the host. The sidecar
  pattern in this ADR mirrors Nitro Enclaves at the process-
  isolation level (without hardware enclave dependency).
- **HashiCorp Vault Agent + Sidecar Injector (2018+).** The Vault
  Agent runs alongside application processes and manages
  authentication + token renewal + secret retrieval. Applications
  invoke the Agent via local listener; secrets never enter the
  application process beyond per-call lifetime. OpenBao (the open-
  source fork of Vault) supports the same pattern.
- **GCP Workload Identity Federation + Secret Manager.** Per-
  workload identity binding; secrets are fetched at injection time
  with bounded TTL.
- **Cloudflare Distributed Keyless SSL (2014+) + Geo Key Manager
  (2017+).** Cryptographic operations are performed by a key
  holder process; the application never sees the private key.
  The exact pattern this ADR adopts for audit-signing.

### Why now (2026-05-20)

Three forcing functions:

1. **F5-255-01 is one of the keystone bundle's four CRITICAL
   findings.** ADR-0255 cannot promote to `Accepted` until it is
   closed.
2. **The library-first amendment is already published.**
   ADR-0255-amendment landed 2026-05-20 with library-first
   dispatch as the canonical pattern. The amendment cannot stand
   without this ADR's credential isolation.
3. **The autonomous-masterplan workflow makes the sidecar
   especially urgent.** `oyatie.foundry.*` workflows call
   Intelligence for every LLM operation (ADR drafting, code
   review, eval); if a foundry workflow process is compromised,
   the platform-wide credential exposure extends to self-
   modification authority. The sidecar's hard isolation closes
   this exposure path.

## Decision

The keystone establishes eight decision sub-sections, D-1 through
D-8.

### D-1. Per-cell credential sidecar — definition

The `oyatie.intelligence.credential-sidecar` is a per-cell pod
deployed alongside every workload that performs LLM dispatch +
audit emission. The sidecar holds:

| Key class | Scope | Sidecar role |
|---|---|---|
| **Audit-signing key** (Ed25519) | Per-cell, per-tenant | The sidecar holds the per-cell-per-tenant audit-signing key in memory; callers invoke `Sign(payload)` via UDS; the key is never exposed to caller processes |
| **Provider credentials (default-pool)** | Per-cell, per-provider | When a caller invokes `Dispatch(call_spec)` via UDS, the sidecar fetches the provider credential from OpenBao with response-wrapping, executes the LLM call directly, and returns the LLM response — the caller never sees the credential |
| **Provider credentials (provider-BYOK)** | Per-tenant | Same flow, but the credential is resolved via the tenant's SecretReference per ADR-0255 §D-4; the caller still never sees it |
| **Cedar evaluator hot-cache (PII attributes)** | Per-cell | Per F5-292-01 (MINOR_PII concern), Cedar context attributes carrying PII are dereferenced inside the sidecar; the caller passes only opaque IDs |
| **Tool-registry snapshot** | Per-cell | Held in sidecar memory; callers query via UDS for tool-call validation |

The sidecar deployment topology:

```
┌──────────────────────────────────────────────────────────────┐
│                    Tier-3 Data Plane Cell                     │
│                                                                │
│   Pod 1                Pod 2               Pod 3              │
│   ┌─────────────┐     ┌─────────────┐    ┌─────────────┐     │
│   │ marketplace │     │ workflow    │    │ messenger   │     │
│   │ µservice    │     │ -engine     │    │ µservice    │     │
│   │ container   │     │ µservice    │    │ container   │     │
│   │             │     │ container   │    │             │     │
│   │ ┌─────────┐ │     │ ┌─────────┐ │    │ ┌─────────┐ │     │
│   │ │UDS      │ │     │ │UDS      │ │    │ │UDS      │ │     │
│   │ │client   │ │     │ │client   │ │    │ │client   │ │     │
│   │ └────┬────┘ │     │ └────┬────┘ │    │ └────┬────┘ │     │
│   └──────┼──────┘     └──────┼──────┘    └──────┼──────┘     │
│          │                    │                   │           │
│          │ /run/oya-sidecar.sock                  │           │
│          ▼                    ▼                   ▼           │
│   ┌────────────────────────────────────────────────────────┐ │
│   │     oyatie.intelligence.credential-sidecar             │ │
│   │     (DaemonSet; one per node)                           │ │
│   │                                                          │ │
│   │     - Holds audit-signing key (per cell, per tenant)    │ │
│   │     - Mediates LLM dispatch + provider creds            │ │
│   │     - Dereferences PII Cedar context                    │ │
│   │     - Tool-registry snapshot held in-process            │ │
│   │     - Memory-isolated from caller pods (separate cgroup,│ │
│   │       separate user ns, separate seccomp profile,       │ │
│   │       separate AppArmor/SELinux profile)                │ │
│   └────────────────────────────────────────────────────────┘ │
│                                                                │
└──────────────────────────────────────────────────────────────┘
```

### D-2. UDS protocol surface — exhaustive enumeration

The sidecar exposes a Unix Domain Socket at `/run/oya-sidecar.sock`
inside each pod (mounted via a CSI volume from the DaemonSet
sidecar's pod onto every workload pod). The UDS protocol is
defined as a typed Rust trait:

```rust
// crates/oya-shared-credential-sidecar-uds/src/protocol.rs

/// Surface exposed by the credential sidecar. Callers MUST use
/// this trait through the UDS client; direct OpenBao access is
/// prohibited by `oya-check-library-first-credential-sidecar`.
#[async_trait]
pub trait CredentialSidecar {
    /// Sign a payload using the per-cell-per-tenant audit-signing
    /// key. Returns the Ed25519 signature. The key never leaves
    /// the sidecar process.
    async fn sign_audit_row(
        &self,
        tenant_id: &TenantId,
        payload: &[u8],
    ) -> Result<AuditSignature, SidecarError>;

    /// Dispatch an LLM call. The sidecar resolves the provider
    /// credential (default-pool or provider-BYOK SecretReference),
    /// executes the LLM call, and returns the response. The
    /// caller never sees the credential.
    async fn dispatch_llm(
        &self,
        call_spec: LlmCallSpec,
    ) -> Result<LlmCallResponse, SidecarError>;

    /// Resolve a PII opaque ID to its plaintext value (for Cedar
    /// context evaluation). The sidecar performs the
    /// dereferencing; the caller never sees the plaintext.
    /// Returns only the attributes the caller is authorised to
    /// see per per-call Cedar context.
    async fn resolve_pii_attribute(
        &self,
        tenant_id: &TenantId,
        opaque_id: &PiiOpaqueId,
        requested_attributes: &[PiiAttributeKey],
        caller_principal: &Principal,
    ) -> Result<PiiAttributeBundle, SidecarError>;

    /// Validate a tool-call against the sidecar's tool-registry
    /// snapshot + CRL. Returns Permit/Forbid/Stale.
    async fn validate_tool_call(
        &self,
        tenant_id: &TenantId,
        tool_id: &ToolId,
        call_context: &CallContext,
    ) -> Result<ToolCallValidation, SidecarError>;

    /// Query a credential handle's remaining lifetime (for
    /// telemetry). Does NOT return the credential itself.
    async fn credential_handle_lifetime_ms(
        &self,
        handle: &CredentialHandle,
    ) -> Result<u64, SidecarError>;

    /// Health probe. Returns immediately. Used by k8s readiness.
    async fn healthz(&self) -> Result<HealthStatus, SidecarError>;
}

#[derive(Debug, Clone)]
pub struct LlmCallSpec {
    pub tenant_id: TenantId,
    pub principal: Principal,
    pub provider: ProviderClass,         // Anthropic, OpenAI, etc.
    pub model_id: ModelId,
    pub audience: AudienceTag,            // per ADR-0255 §D-15
    pub messages: Vec<LlmMessage>,
    pub tool_specs: Vec<ToolSpec>,
    pub guardrails_profile: GuardrailsProfile,
    pub data_class: DataClass,
    pub max_tokens: u32,
    pub temperature: f32,
    pub idempotency_key: Uuid,
    pub cedar_decision_witness: CedarDecisionWitness,  // proves Cedar evaluated this dispatch
}

#[derive(Debug)]
pub struct LlmCallResponse {
    pub content: LlmContent,
    pub tokens_billed: TokenCount,
    pub provider_request_id: String,
    pub audit_row_id: Uuid,  // sidecar emits its own audit row
}

#[derive(Debug, thiserror::Error)]
pub enum SidecarError {
    #[error("Cedar evaluation denied this operation: {0}")]
    CedarDenied(String),

    #[error("provider credential resolution failed: {0}")]
    CredentialResolutionFailed(String),

    #[error("OpenBao token expired before LLM call completed (TTL too short)")]
    OpenBaoTokenExpiredMidCall,

    #[error("provider API rate-limited or unavailable")]
    ProviderUnavailable,

    #[error("tool-registry snapshot stale beyond ${0}s; refresh required")]
    ToolRegistryStale(u64),

    #[error("sidecar handle limit exceeded; backpressure")]
    SidecarOverload,

    #[error("UDS connection broken")]
    UdsBroken,
}
```

The protocol is intentionally **narrow**. Five operations:
`sign_audit_row`, `dispatch_llm`, `resolve_pii_attribute`,
`validate_tool_call`, `credential_handle_lifetime_ms`. Anything
else — fetching credentials directly, modifying the audit-signing
key, listing OpenBao secrets — is **not exposed**. The narrow
surface is the load-bearing security property: an attacker with
UDS access can only invoke these five operations, not pivot to
arbitrary credential access.

### D-3. TTL enforcement — credential handle lifecycle

Provider credentials are fetched per-call from OpenBao with
short-lived tokens. The TTL is enforced at three layers:

#### D-3.1. OpenBao response-wrapping

The sidecar requests credentials via OpenBao's response-wrapping
primitive (Vault's `wrapping_token` feature, supported by OpenBao
fork). The flow:

1. Sidecar issues a request to OpenBao for the provider credential.
2. OpenBao returns a **wrapping token** instead of the credential
   itself. The wrapping token has a 60-second TTL.
3. The sidecar IMMEDIATELY unwraps the token to obtain the
   credential.
4. The credential is held in sidecar memory ONLY for the duration
   of the LLM call (typically 1-30 seconds).
5. Upon LLM call completion, the credential is zeroized in
   sidecar memory.

If the LLM call takes longer than 60s + (call duration), the
sidecar refreshes the credential mid-call via a fresh wrapping
token. Each refresh is logged.

#### D-3.2. Memory zeroization

The sidecar links the `zeroize` crate (zeroize v1.x maintained by
Rust Crypto) and wraps every credential in `Zeroizing<Vec<u8>>`.
On drop, the memory is overwritten with zeroes before
deallocation. The sidecar additionally:

1. Locks credential memory pages via `mlock()` to prevent swap-
   out.
2. Uses `madvise(MADV_DONTDUMP)` to prevent inclusion in core
   dumps.
3. Disables coredump generation entirely for the sidecar process
   via `prctl(PR_SET_DUMPABLE, 0)`.

These three Linux primitives are the OS-level analog to AWS
Nitro Enclaves' hardware-enforced memory isolation.

#### D-3.3. Per-call telemetry

Every credential resolution emits a CredentialHandleEvent to the
sidecar-local OTel collector:

```rust
pub struct CredentialHandleEvent {
    pub event_type: CredentialHandleEventType,
    pub handle_id: Uuid,
    pub tenant_id: TenantId,
    pub provider: ProviderClass,
    pub resolved_at: SystemTime,
    pub resolved_via: ResolutionSource,  // OpenBao | TenantProviderCredentialMode
    pub ttl_seconds: u64,
    pub call_completed_at: Option<SystemTime>,
    pub zeroized_at: Option<SystemTime>,
    pub call_duration_ms: Option<u32>,
}

pub enum CredentialHandleEventType {
    Resolved,
    Refreshed,
    CallCompleted,
    Zeroized,
    LeakSuspected,  // e.g., handle outlived its OpenBao TTL
}
```

The `LeakSuspected` event triggers a SEV-2 alert. A leak suspicion
is raised when:
- A credential handle's TTL elapses without a corresponding
  `Zeroized` event.
- A credential handle exceeds 5 minutes total lifetime (well past
  the 60s OpenBao TTL).
- The sidecar process crashes without zeroizing handles.

### D-4. Audit-signing key — sidecar-only residency

The audit-signing key is the highest-stakes key in this ADR's
scope. Its handling differs from provider credentials:

#### D-4.1. Lifetime

The audit-signing key is per-cell-per-tenant. It is loaded into
the sidecar memory at sidecar startup from OpenBao (with response-
wrapping for the initial load) and remains in sidecar memory for
the sidecar's lifetime (typically days to weeks per pod).

The key rotates per the tenant's compliance pack rotation cadence
(per ADR-0251). On rotation:
1. The new key is fetched from OpenBao.
2. The new key is held alongside the old key for a 24h overlap
   period.
3. After overlap, the old key is zeroized.

This rotation cadence does NOT match the ≤60s OpenBao TTL pattern
of provider credentials because Merkle-chain continuity across
audit-signing key rotations requires a key-handoff protocol that
is incompatible with sub-minute rotations.

#### D-4.2. Sidecar memory isolation

The sidecar process is **strongly isolated** from caller pods:

| Mechanism | Configuration |
|---|---|
| **Kubernetes Pod isolation** | Sidecar is a separate Pod (DaemonSet pattern), not a sidecar container in the caller's pod. Inter-pod IPC via UDS mounted from a shared CSI volume. |
| **User namespace** | Sidecar runs as a dedicated UID/GID outside the caller's UID/GID range |
| **Seccomp profile** | Restricts syscalls to: `read`, `write`, `recvmsg`, `sendmsg`, `socket`, `bind`, `accept`, `connector`, `close`, `epoll_*`, `clock_gettime`, `mlock`, `munlock`, `prctl`, `getrandom`, `exit_group`, `mmap`, `munmap`, `mprotect`, `futex`. NO `ptrace`, NO `process_vm_readv`, NO `process_vm_writev`. |
| **AppArmor / SELinux profile** | Refuses any file access outside `/run/oya-sidecar.sock` (the UDS path), `/etc/oya/sidecar/`, `/proc/self/`, `/dev/urandom`, and the audit-emit Kafka endpoint |
| **Memory limits** | Cgroup memory limit set explicitly to prevent OOM-killer escalation to root |
| **No host PID namespace** | Sidecar's PID namespace is isolated from caller pods |
| **gVisor or Kata containers (RECOMMENDED)** | For Tier-2 control-plane cells, sidecar runs in gVisor or Kata; for Tier-3 data-plane cells, runtime is per-cell choice |

The result: even if a caller pod is compromised AND escapes its
container, accessing the sidecar pod's memory requires escaping
the kernel-level isolation (kernel CVE territory), not just
container escape.

#### D-4.3. Audit-row signing protocol

```rust
// crates/oya-shared-credential-sidecar-uds/src/audit.rs

impl CredentialSidecar for CredentialSidecarImpl {
    async fn sign_audit_row(
        &self,
        tenant_id: &TenantId,
        payload: &[u8],
    ) -> Result<AuditSignature, SidecarError> {
        // Step 1: Verify the caller principal via UDS peer cred
        let peer_principal = self.uds_peer_cred()?;

        // Step 2: Evaluate Cedar permit for "may this principal
        // sign audit rows for this tenant?"
        let cedar_decision = self.cedar_evaluator.evaluate(EvaluationRequest {
            principal: peer_principal.clone(),
            action: Action::from("AuditChain::Action::SignRow"),
            resource: Resource::Tenant(tenant_id.clone()),
            context: btreemap! {
                "payload_hash".to_string() => sha3_256(payload).into(),
            },
            tenant_id: tenant_id.clone(),
            evaluation_id: Uuid::new_v4(),
        }).await?;

        if cedar_decision.decision != Decision::Permit {
            return Err(SidecarError::CedarDenied(
                cedar_decision.reason()
            ));
        }

        // Step 3: Acquire the audit-signing key (sidecar-resident)
        let signing_key = self.tenant_audit_signing_keys
            .get(tenant_id)
            .ok_or(SidecarError::CredentialResolutionFailed(
                format!("no audit signing key for tenant {}", tenant_id)
            ))?;

        // Step 4: Sign the payload (Ed25519)
        let signature = signing_key.sign(payload);

        // Step 5: Emit a meta-audit-row recording the sign event
        self.emit_meta_audit_row(MetaAuditRow {
            event: "audit_row_signed",
            tenant_id: tenant_id.clone(),
            caller_principal: peer_principal,
            payload_hash: sha3_256(payload),
            signed_at: SystemTime::now(),
        }).await?;

        Ok(AuditSignature {
            signature,
            signing_key_fingerprint: signing_key.fingerprint(),
            signed_at: SystemTime::now(),
        })
    }
}
```

Note that the audit-signing key is referenced via
`self.tenant_audit_signing_keys.get(tenant_id)` — a process-local
lookup. The caller never sees this map; the caller invokes via UDS
and receives only the signature output.

### D-5. Observability for credential-handle lifetime

Per F5-255-01 recommendation (b): "Audit-signing keys must be
fetched fresh per-emission and never cached; alternatively, use a
sidecar pattern where the audit-signing key never leaves a sidecar
process with separate memory space."

The "alternative" is selected: sidecar pattern. To monitor that
the sidecar's isolation is intact, four metric panels are
required:

#### D-5.1. Metric panels

| Panel | Signal | SLO | Anomaly trigger |
|---|---|---|---|
| **Credential handle lifetime distribution** | Per-handle `(resolved_at, zeroized_at, ttl_seconds)` triples; p50/p99 lifetime | p99 ≤ 90s; p50 ≤ 5s | Any handle exceeding 300s lifetime |
| **Handle-leak suspicion rate** | `LeakSuspected` events per minute | 0/min | Any non-zero rate; SEV-2 alert |
| **Sidecar process memory pressure** | RSS, mlock-locked-pages, swap-out attempts | Stable RSS within cgroup limit; no swap | Any swap-out attempt (memory should be mlocked) |
| **UDS connection count + throughput** | Active connections; ops/sec | Steady-state matching caller pod count; no unbounded growth | Connection count > 10× caller-pod count suggests connection leak |

#### D-5.2. Dashboards

- **Per-sidecar dashboard.** Shows the four panels above per-
  sidecar-pod.
- **Cross-cell credential-leak heatmap.** Aggregates
  `LeakSuspected` events by cell + provider + tenant.
- **Audit-signing key rotation timeline.** Shows when each
  tenant's key was last rotated; flags any key older than
  rotation cadence + 14d grace.

### D-5.3. Failure semantics

If the sidecar process dies (panic, SIGKILL, OOM kill):

1. **Caller-side behavior.** Caller's UDS reads return
   `SidecarError::UdsBroken`. Caller MUST NOT attempt to fall
   back to direct OpenBao access (which is what the library-first
   anti-pattern was). Instead, caller emits a
   `CallerSidecarUnavailable` log + metric and either:
   - For LLM dispatch: queues the call into a per-tenant
     retry queue with exponential backoff up to 60s; if not
     resolved, returns `ServiceUnavailable` to the upstream
     caller.
   - For audit signing: writes the unsigned payload to a per-
     pod write-ahead log; the sidecar (when restored) processes
     the WAL and signs deferred audit rows.
   - For PII dereferencing: returns `ServiceUnavailable`.
   - For tool-call validation: returns `Forbid` (fail-closed).
2. **Kubelet restart.** The DaemonSet's restart policy ensures the
   sidecar pod is restarted within ≤ 5s. Tools should not see
   prolonged unavailability.
3. **Credential handle restoration.** The sidecar's startup
   sequence:
   - Loads the tenant audit-signing keys via OpenBao response-
     wrapping (one fetch per tenant the sidecar serves).
   - Processes any deferred-audit-row WALs from caller pods.
   - Marks itself ready in the K8s readiness probe.
4. **No credential persistence across restarts.** The sidecar does
   NOT persist credentials to disk. All in-memory state is lost on
   restart; the sidecar reconstructs state at startup. This is the
   anti-corruption barrier against persistent compromise.

### D-6. Per-tenant audit-signing key segregation

The sidecar holds **one audit-signing key per tenant per cell**.
The keys are NOT shared across tenants. The advantage of per-tenant
keys:

1. **Per-tenant blast radius bound.** Compromise of one tenant's
   audit-signing key (which would require sidecar compromise +
   memory disclosure) does not compromise other tenants' audit
   rows.
2. **Per-tenant rotation cadence.** Tenants on stricter
   compliance packs (HIPAA, DoD) can rotate their audit-signing
   key more aggressively than tenants on relaxed packs.
3. **Per-tenant provider-BYOK pathway.** Tenants with encryption-BYOK per
   ADR-0251 §D-10 may supply their own audit-signing key via
   the SecretReference primitive; the sidecar loads it instead
   of the platform-default.

The per-tenant key segregation imposes a cost: the sidecar must
hold N keys for a cell serving N tenants. For high-N cells (e.g.,
100+ tenants), this means 100+ Ed25519 keys (32 bytes each) plus
metadata in sidecar memory — well within practical limits (a few
KB per tenant).

### D-7. Untrusted-caller carve-out — WASM sandboxing

Per F5-255-01 recommendation (c): "Run untrusted µservices
(marketplace, plugin runtime) in WASM sandboxes (per ADR-0200)
that CANNOT link the full intelligence-client library — they must
call an in-cell intelligence-coordinator (which IS a network hop,
but only for low-trust callers)."

This ADR ratifies the carve-out as RECOMMENDED hardening for the
lowest-trust callers:

| Caller class | Pattern |
|---|---|
| **High-trust callers** (substrate µservices: workflow-engine, audit-chain, identity, policy-engine, intelligence, tenancy, cell, billing) | Library-first dispatch via the credential-sidecar UDS per §D-1 through §D-6 |
| **Medium-trust callers** (product µservices: mail, drive, calendar, messenger, workflow-studio) | Library-first dispatch via the credential-sidecar UDS per §D-1 through §D-6 |
| **Low-trust callers** (marketplace plugin runtime, externally-developed plugin BCs, tenant-submitted code via WASM) | Wasmtime sandbox per ADR-0200; the WASM module cannot link the credential-sidecar UDS client directly; instead, it issues a network-RPC call to a per-cell `oyatie.intelligence.coordinator` µservice that mediates between WASM modules and the sidecar |

The Wasmtime sandbox provides:
- Memory isolation (WASM linear memory cannot read host memory).
- Syscall denial (WASM has no syscalls; host imports are
  explicit).
- No UDS access (the host environment does not export the UDS
  client to the WASM module).

The intelligence-coordinator µservice's role:
- Authenticates the WASM module's caller via per-tenant identity.
- Translates the WASM module's RPC into a sidecar UDS call.
- Returns the result through the network.

The network-hop is acceptable for low-trust callers because (a)
their throughput is lower than substrate callers, (b) their
correctness budget is laxer, and (c) the trust isolation gain is
worth the latency.

### D-8. CI lane scope — `oya-check-library-first-credential-sidecar`

The CI lane scans every library-first caller in the platform's
Rust workspace and verifies four properties:

1. **No direct OpenBao access.** Static analysis searches for
   `openbao_client::*` imports outside of the
   `oyatie.intelligence.credential-sidecar` crate; any other crate
   importing direct OpenBao client triggers a finding.
2. **No audit-signing key handle.** Static analysis searches for
   `Ed25519SigningKey::*` instantiation outside of the sidecar
   crate. Callers that need to sign should call
   `sidecar.sign_audit_row()` via UDS.
3. **UDS client linked.** Every µservice that emits audit rows or
   makes LLM calls must link the
   `oya-shared-credential-sidecar-uds` crate.
4. **No in-process credential caching.** Static analysis searches
   for any `Mutex<Credential>` or `RwLock<Credential>` outside
   the sidecar crate; any other caching of credentials triggers
   a finding.

The CI lane runs as part of every PR's build phase and emits
findings into the PR review.

## Consequences

### Positive

1. **The F5-255-01 exploit window is closed.** An RCE in any
   library-first caller (including the lowest-trust marketplace
   µservice) yields ZERO usable credentials: the audit-signing
   key is in a separate process (sidecar), provider credentials
   are in OpenBao with ≤60s TTL response-wrapped tokens, and
   neither is accessible to the caller's memory.
2. **The library-first decision is preserved.** Callers continue
   to invoke Intelligence dispatch via in-process library code;
   the sidecar is co-located on the same node (DaemonSet); the
   UDS hop is sub-millisecond. The "no universal mediator" property
   of the library-first amendment is maintained.
3. **The audit-signing key has bounded blast radius.** Per-tenant
   keys mean cross-tenant audit-forging is impossible without
   compromising each tenant's sidecar key separately.
4. **Observability for credential lifetime is automatic.** Every
   credential resolution emits telemetry; leak suspicion triggers
   SEV-2 alerts. This is more visibility than the prior library-
   first amendment offered.
5. **The mechanism composes cleanly with ADR-0293 / 0294 / 0295.**
   The sidecar's Cedar evaluation against per-call permits routes
   through the same fragment lifecycle (soak per ADR-0294); the
   sidecar's identity is registered per ADR-0242 reserved-
   namespace; the sidecar's audit-signing key rotation goes
   through the meta-trust-root-witnessed substrate-deploy path
   per ADR-0293 + ADR-0295.

### Negative

1. **Operational overhead increases.** Every cell now runs a
   credential-sidecar pod. For a fleet of ~100 cells, that's 100
   additional pods plus their resource budget (~500m CPU / 1Gi
   memory baseline per sidecar pod).
2. **Per-call latency increases by UDS round-trip.** The sub-
   millisecond UDS hop adds ~0.5ms p99 to every LLM dispatch +
   audit-emit operation. For high-throughput workloads (>1000
   ops/sec per pod), this is a ~5-10% overhead.
3. **Sidecar is a new failure domain.** If the sidecar dies, the
   caller cannot serve. Mitigations:
   - K8s readiness probe + DaemonSet restart (≤5s outage).
   - Per-pod write-ahead log for deferred audit signing.
   - Fail-closed for tool-call validation (deny by default).
4. **Per-tenant key segregation adds sidecar memory cost.** ~100
   bytes of metadata per tenant; negligible for normal-cell
   sizes; potentially significant for ultra-multi-tenant cells
   (>10K tenants in a single cell). The mitigation: tenant
   density per cell is bounded by ADR-0248 cellular architecture
   to <500 tenants typically.
5. **Sidecar code is itself a high-value attack target.** The
   sidecar holds all the keys; sidecar compromise is total
   credential exposure for the cell. Mitigations:
   - Strong process isolation per §D-4.2.
   - Sidecar code is in-scope for every multispectrum-review
     v2.4.0 cycle.
   - Sidecar's audit-signing key rotation is gated by the meta-
     trust-root witness per ADR-0293.
   - gVisor or Kata containers RECOMMENDED for control-plane
     cells.
6. **Wasmtime-sandboxed callers incur an additional network
   hop.** Low-trust callers (marketplace, plugin runtime) call
   the intelligence-coordinator µservice over the network rather
   than the in-pod sidecar. This is a deliberate trust trade-off
   but does cost ~2-5ms p99 per call for low-trust callers.

### Neutral

1. **The mechanism does NOT eliminate OpenBao as a substrate.**
   OpenBao remains the source of truth for provider credentials.
   The sidecar is the access mediator, not the credential store.
2. **The mechanism does NOT change the provider-BYOK posture per ADR-0255
   §D-4.** provider-BYOK provider credentials flow through the
   sidecar identically to default-pool credentials; the only
   difference is the resolution path (tenant SecretReference vs
   OpenBao default-pool).
3. **The mechanism is invisible to customer tenants.** Customer-
   tenant principals see the same LLM call API surface; the
   sidecar is below the API boundary.

## Detailed Mechanics

### D-1 expanded — DaemonSet vs sidecar-container topology

Two deployment topologies were considered:

| Topology | Strengths | Weaknesses |
|---|---|---|
| **DaemonSet (selected).** One sidecar pod per node; multiple workload pods on the node share its UDS via a hostPath/CSI volume. | Lower pod count (1 per node vs 1 per workload); resource sharing; isolation across workloads at the kernel level. | Shared UDS means a compromised workload pod can read other workload pods' UDS messages if seccomp/AppArmor are not properly configured. Mitigation: per-tenant UDS path (`/run/oya-sidecar-{tenant_id}.sock`); UDS peer-credential verification rejects mismatched callers. |
| **Sidecar container (rejected).** One sidecar container per workload pod, sharing the pod's network + storage namespaces. | Stronger per-workload isolation; UDS is pod-local. | Higher pod count; sidecar resource budget multiplied by workload count; harder to upgrade (every workload pod must restart for sidecar upgrade). |

The DaemonSet topology is selected because (a) the per-tenant UDS
path + UDS peer-credential verification provides equivalent
workload isolation, (b) the operational simplicity of DaemonSet
upgrade is high-value, and (c) resource consolidation reduces
fleet-wide overhead.

### D-2 expanded — UDS peer credential verification

Every UDS connection's peer is verified via the `SO_PEERCRED`
socket option (Linux-specific; equivalent on macOS via
`LOCAL_PEERCRED`). The sidecar reads the calling process's UID,
GID, and PID, then looks up the corresponding K8s pod via:

```rust
async fn uds_peer_cred(&self) -> Result<Principal, SidecarError> {
    let peer = self.uds.peer_cred()?;  // (uid, gid, pid)

    // Resolve PID to K8s Pod via /proc/<pid>/cgroup
    let pod_metadata = self.kubelet_client
        .lookup_pod_by_pid(peer.pid)
        .await?;

    // Verify the Pod's service account matches one of the
    // sidecar's allowed callers per its Cedar fragment.
    let principal = pod_metadata.service_account_to_principal()?;

    Ok(principal)
}
```

The peer-credential check is the binding between UDS-level access
(a syscall) and Cedar-level access (a principal). Without this
binding, any process on the node with file-system access to the
UDS could impersonate any caller.

### D-3 expanded — OpenBao token response-wrapping flow

```rust
async fn dispatch_llm(
    &self,
    call_spec: LlmCallSpec,
) -> Result<LlmCallResponse, SidecarError> {
    // 1. Verify caller principal via UDS peer cred
    let peer = self.uds_peer_cred()?;

    // 2. Cedar evaluate "may this principal dispatch LLM calls
    //    on behalf of this tenant for this audience for this
    //    data class with this provider?"
    let cedar = self.cedar_evaluator.evaluate(EvaluationRequest {
        principal: peer.clone(),
        action: Action::from("Intelligence::Action::DispatchLlm"),
        resource: Resource::Llm {
            provider: call_spec.provider.clone(),
            model: call_spec.model_id.clone(),
        },
        context: btreemap! {
            "audience".to_string()        => call_spec.audience.into(),
            "data_class".to_string()      => call_spec.data_class.into(),
            "tenant_id".to_string()       => call_spec.tenant_id.clone().into(),
        },
        tenant_id: call_spec.tenant_id.clone(),
        evaluation_id: Uuid::new_v4(),
    }).await?;

    if cedar.decision != Decision::Permit {
        return Err(SidecarError::CedarDenied(cedar.reason()));
    }

    // 3. Resolve provider credential via OpenBao response-wrapping
    //    The wrapping token has 60s TTL; we immediately unwrap.
    let cred_handle = Uuid::new_v4();
    let provider_cred: Zeroizing<Vec<u8>> = self.openbao_client
        .request_wrapped_credential(
            call_spec.tenant_id.clone(),
            call_spec.provider.clone(),
            /* ttl */ Duration::from_secs(60),
        )
        .await?
        .unwrap_immediately()
        .await?;

    // Emit telemetry
    self.emit_credential_event(CredentialHandleEvent {
        event_type: CredentialHandleEventType::Resolved,
        handle_id: cred_handle,
        tenant_id: call_spec.tenant_id.clone(),
        provider: call_spec.provider.clone(),
        resolved_at: SystemTime::now(),
        resolved_via: ResolutionSource::OpenBao,
        ttl_seconds: 60,
        call_completed_at: None,
        zeroized_at: None,
        call_duration_ms: None,
    });

    // 4. Execute LLM call (the provider_cred is consumed here;
    //    HTTP request constructed with it; cred is in flight for
    //    the duration of the HTTP RTT)
    let call_start = SystemTime::now();
    let llm_response = self.llm_dispatcher
        .dispatch(call_spec.provider.clone(),
                  call_spec.model_id.clone(),
                  &provider_cred,
                  &call_spec.messages,
                  &call_spec.tool_specs,
                  call_spec.max_tokens,
                  call_spec.temperature)
        .await?;
    let call_end = SystemTime::now();

    // 5. Zeroize the credential (Zeroizing<Vec<u8>>'s Drop will
    //    handle this; we make it explicit for telemetry)
    drop(provider_cred);

    self.emit_credential_event(CredentialHandleEvent {
        event_type: CredentialHandleEventType::Zeroized,
        handle_id: cred_handle,
        tenant_id: call_spec.tenant_id.clone(),
        provider: call_spec.provider.clone(),
        resolved_at: call_start,
        resolved_via: ResolutionSource::OpenBao,
        ttl_seconds: 60,
        call_completed_at: Some(call_end),
        zeroized_at: Some(SystemTime::now()),
        call_duration_ms: Some(call_end.duration_since(call_start)?.as_millis() as u32),
    });

    // 6. Sign + emit the audit row for this dispatch
    let audit_payload = serialize_audit_row(&peer, &call_spec, &llm_response);
    let audit_signature = self.sign_audit_row(
        &call_spec.tenant_id,
        &audit_payload,
    ).await?;
    let audit_row_id = self.audit_emitter
        .emit(audit_payload, audit_signature)
        .await?;

    Ok(LlmCallResponse {
        content: llm_response.content,
        tokens_billed: llm_response.tokens_billed,
        provider_request_id: llm_response.provider_request_id,
        audit_row_id,
    })
}
```

### D-4 expanded — Postgres schema for sidecar state + telemetry

```sql
-- microservices/cloud-secrets/migrations/0091_credential_sidecar.sql

CREATE TABLE credential_sidecar_pods (
    sidecar_pod_id          UUID PRIMARY KEY,
    cell_id                 TEXT NOT NULL,
    node_id                 TEXT NOT NULL,
    pod_name                TEXT NOT NULL UNIQUE,
    started_at              TIMESTAMPTZ NOT NULL,
    last_heartbeat_at       TIMESTAMPTZ NOT NULL,
    sidecar_version         TEXT NOT NULL,
    seccomp_profile_id      TEXT NOT NULL,
    apparmor_profile_id     TEXT,
    selinux_profile_id      TEXT,
    serves_tenant_ids       TEXT[] NOT NULL,
    uds_path                TEXT NOT NULL,
    isolation_runtime       TEXT NOT NULL CHECK (isolation_runtime IN (
        'runc', 'gvisor', 'kata-containers', 'firecracker'
    )),
    UNIQUE (cell_id, node_id, pod_name)
);

CREATE TABLE credential_handle_events (
    event_id                BIGSERIAL PRIMARY KEY,
    event_type              TEXT NOT NULL CHECK (event_type IN (
        'Resolved', 'Refreshed', 'CallCompleted', 'Zeroized', 'LeakSuspected'
    )),
    handle_id               UUID NOT NULL,
    sidecar_pod_id          UUID NOT NULL REFERENCES credential_sidecar_pods(sidecar_pod_id),
    tenant_id               TEXT NOT NULL,
    provider                TEXT NOT NULL,
    resolved_at             TIMESTAMPTZ NOT NULL,
    resolved_via            TEXT NOT NULL CHECK (resolved_via IN (
        'OpenBao', 'TenantProviderCredentialMode', 'Sidecar-Held'
    )),
    ttl_seconds             INTEGER,
    call_completed_at       TIMESTAMPTZ,
    zeroized_at             TIMESTAMPTZ,
    call_duration_ms        INTEGER,
    leak_suspicion_reason   TEXT
);

-- Time-series-style index for telemetry queries
CREATE INDEX credential_handle_events_by_time
    ON credential_handle_events (resolved_at DESC);
CREATE INDEX credential_handle_events_by_handle
    ON credential_handle_events (handle_id, event_type);
CREATE INDEX credential_handle_events_leak_suspicion
    ON credential_handle_events (event_type, resolved_at DESC)
    WHERE event_type = 'LeakSuspected';

CREATE TABLE tenant_audit_signing_keys (
    tenant_id               TEXT NOT NULL,
    cell_id                 TEXT NOT NULL,
    key_version             INTEGER NOT NULL,
    key_fingerprint         BYTEA NOT NULL UNIQUE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    rotated_at              TIMESTAMPTZ,
    rotation_overlap_until  TIMESTAMPTZ,
    zeroized_at             TIMESTAMPTZ,
    rotation_witnessed_by   BYTEA REFERENCES meta_trust_root_ceremonies(attestation_hash),
    PRIMARY KEY (tenant_id, cell_id, key_version)
);

CREATE TABLE sidecar_caller_principals (
    sidecar_pod_id          UUID NOT NULL REFERENCES credential_sidecar_pods(sidecar_pod_id),
    caller_principal        TEXT NOT NULL,
    caller_pod_name         TEXT NOT NULL,
    first_seen_at           TIMESTAMPTZ NOT NULL,
    last_seen_at            TIMESTAMPTZ NOT NULL,
    request_count           BIGINT NOT NULL DEFAULT 0,
    cedar_denial_count      BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (sidecar_pod_id, caller_principal, caller_pod_name)
);

CREATE TABLE deferred_audit_row_wal (
    wal_entry_id            BIGSERIAL PRIMARY KEY,
    caller_pod_id           UUID NOT NULL,
    tenant_id               TEXT NOT NULL,
    payload                 BYTEA NOT NULL,
    queued_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at            TIMESTAMPTZ,
    audit_row_id            UUID,
    failure_reason          TEXT
);
```

### D-7 expanded — intelligence-coordinator surface

```rust
// microservices/intelligence/src/coordinator.rs

/// Network-RPC surface for low-trust (Wasmtime-sandboxed) callers.
/// The coordinator forwards calls to the appropriate per-cell
/// credential-sidecar via UDS. It is a thin proxy.
#[tonic::async_trait]
pub trait IntelligenceCoordinator {
    async fn dispatch_llm_for_wasm(
        &self,
        request: Request<DispatchLlmForWasmRequest>,
    ) -> Result<Response<LlmCallResponse>, Status>;

    async fn validate_tool_call_for_wasm(
        &self,
        request: Request<ValidateToolCallForWasmRequest>,
    ) -> Result<Response<ToolCallValidation>, Status>;

    // Note: low-trust callers do NOT have access to
    // sign_audit_row or resolve_pii_attribute — those are
    // reserved for trusted callers.
}
```

## Implementation Footprint

### Microservice scope

| Microservice | Change | Effort |
|---|---|---|
| `microservices/cloud-secrets/` | Add credential-sidecar Postgres tables; add response-wrapping primitive surface | ≈ 2 weeks |
| `microservices/intelligence/` | Add intelligence-coordinator BC for WASM callers; remove direct credential-caching code | ≈ 3 weeks |
| `microservices/observability/` | Add credential-handle-lifetime panels + alerting | ≈ 2 weeks |
| `microservices/policy-engine/` | Cedar fragments authorising sidecar callers per per-tenant audit-signing | ≈ 1 week |
| `crates/oya-shared-credential-sidecar-uds/` (new) | Shared crate exposing UDS protocol; consumed by every library-first caller | ≈ 4 weeks |
| `crates/oyatie-intelligence-credential-sidecar/` (new) | The sidecar binary; DaemonSet pod | ≈ 6 weeks |
| `crates/oya-shared-zeroize-helper/` (new) | Reusable zeroize wrapper conforming to FIPS 140-3 §4.7.5 Purge | ≈ 1 week |
| ~30 library-first caller crates | Refactor to use the sidecar UDS client instead of direct OpenBao access; remove in-process credential caches; remove direct audit-signing key handling | ≈ 1-2 weeks each, parallelized |

Total: ≈ 50-60 weeks of engineering effort across the ~30+ caller
crates plus the new substrate. Parallelizable; calendar time
≈ 8-10 weeks.

### CI lane scope

| CI lane | Behavior |
|---|---|
| `oya-check-library-first-credential-sidecar` | Static analysis: no direct OpenBao access outside sidecar; no direct audit-signing-key handles outside sidecar; UDS client linked |
| `oya-check-credential-handle-lifetime-bound` | Telemetry analysis: no `LeakSuspected` events in the last 7 days; p99 handle lifetime ≤ 90s |
| `oya-check-audit-signing-key-not-in-caller-process` | Memory introspection on running pods (via eBPF probe) verifies that no caller pod has Ed25519 signing-key memory regions |
| `oya-check-openbao-token-ttl-bound` | OpenBao audit-log analysis: every token issuance has TTL ≤ 60s for credential paths |
| `oya-check-sidecar-uds-protocol-compliance` | Static analysis of UDS client usage: only the 5 documented operations invoked; no protocol drift |

### Observability scope

The sidecar emits to the per-cell observability collector per
ADR-0263. Required dashboards:

1. **Sidecar fleet health.** Heartbeats, version distribution,
   isolation-runtime distribution.
2. **Credential handle lifetime histogram.** Per-provider, per-
   tenant percentiles.
3. **Leak suspicion timeline.** Bucketed counts of
   `LeakSuspected` events.
4. **Audit-signing key rotation timeline.** Per-tenant, per-cell
   rotation cadence.

## Migration

### Stage 0 — Shared crate + sidecar binary (T+0 to T+6w)

| Step | Action |
|---|---|
| 0.1 | `oya-shared-credential-sidecar-uds` crate scaffolded with protocol traits + types |
| 0.2 | `oyatie-intelligence-credential-sidecar` binary scaffolded with seccomp, AppArmor, SELinux profiles, mlock, zeroize |
| 0.3 | DaemonSet Helm chart authored under `microservices/intelligence/iac/helm/credential-sidecar/` |
| 0.4 | Integration tests covering UDS protocol + Cedar evaluation + OpenBao response-wrapping |

### Stage 1 — Sidecar deployment to staging (T+6w to T+8w)

| Step | Action |
|---|---|
| 1.1 | Sidecar DaemonSet deployed to `dev-tools-cell-staging`; runs alongside existing callers without enforcement |
| 1.2 | Telemetry collection begins; baseline metrics established |
| 1.3 | First library-first caller migrated to UDS client (the simplest one: `microservices/workflow-engine`) |
| 1.4 | E2E rehearsal: a controlled RCE in a staging caller pod; verify no credentials are accessible |

### Stage 2 — Caller migration (T+8w to T+16w)

| Step | Action |
|---|---|
| 2.1 | High-trust callers (substrate µservices) migrated to UDS client; ~ 8 weeks parallelized across crews |
| 2.2 | Medium-trust callers (product µservices) migrated; ~ 4 weeks |
| 2.3 | Low-trust callers (marketplace, plugin runtime) routed via intelligence-coordinator over network; ~ 2 weeks |
| 2.4 | All direct OpenBao access in non-sidecar crates removed |
| 2.5 | All in-process audit-signing-key handling in non-sidecar crates removed |

### Stage 3 — Production deployment + advisory → BLOCKER (T+16w to T+18w)

| Step | Action |
|---|---|
| 3.1 | Sidecar DaemonSet deployed to Tier-2 + Tier-3 cells |
| 3.2 | Telemetry steady-state verified; no leak suspicions |
| 3.3 | Five CI lanes flip from advisory to BLOCKER |
| 3.4 | ADR-0255 amendment + ADR-0246 are correspondingly amended per the `requires_amendment_to` list above |
| 3.5 | The bundle's promotion gate for ADR-0255 closes |

## References

### Primary

- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.4
  — authority for this ADR's existence.
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json`
  — F5-255-01 (CRITICAL).

### Related ADRs

- ADR-0255 (Intelligence as Two-Layer AI Substrate) + ADR-0255
  amendment — library-first amendment that necessitates this
  ADR's credential isolation.
- ADR-0246 (Policy Engine Substrate Promotion) — Cedar fragment
  validation for sidecar operations.
- ADR-0145 (Inter-Microservice Communication Reform) — invariant
  1 (each caller emits its own seal) drives the audit-signing-key
  distribution that this ADR mediates via sidecar.
- ADR-0200 (Wasmtime Substrate) — WASM sandbox for low-trust
  callers per §D-7.
- ADR-0263 (Observability Emission Contract) — credential handle
  events emit per this contract.
- ADR-0251 (Compliance Pack + Cell Certification Levels) —
  per-tenant audit-signing key rotation cadence per pack.
- ADR-0293 (Foundry Meta-Trust-Root) — audit-signing key rotation
  ceremony is witnessed by the meta-trust-root.
- ADR-0294 (Cedar Fragment Soak + Anomaly-Rollback) — sidecar's
  Cedar fragments soak through this lifecycle.
- ADR-0295 (Bootstrap CI SPIFFE + Kill-Switch) — sidecar binary
  is produced under the bootstrap CA's attestation chain during
  Stage-1.

### Industry references

- **AWS Nitro Enclaves (re:Invent 2019, GA 2020).** Hardware-
  isolated credential-handling enclave; primary precedent for the
  process-isolation pattern this ADR adopts.
- **HashiCorp Vault Agent + OpenBao (2018+).** Sidecar pattern
  for credential resolution; the wrapping-token + auto-renewal
  primitives this ADR uses.
- **GCP Workload Identity Federation + Secret Manager (2020+).**
  Per-workload identity binding + short-lived TTL.
- **Cloudflare Distributed Keyless SSL (2014+) + Geo Key Manager
  (2017+).** Cryptographic operations performed by a separate
  key-holder process; the audit-signing flow in this ADR is
  directly modeled on this pattern.
- **gVisor and Kata Containers (Google, Intel, 2018+).** Strong
  workload isolation runtimes; RECOMMENDED for Tier-2 sidecars.
- **AWS Lambda + EFS short-lived credential model.** Per-invocation
  credentials with bounded TTL.
- **Stripe API key rotation + Vault integration patterns
  (Brandur Leach blog 2018-2022).** Per-call credential resolution
  + bounded in-process lifetime.

### Cryptographic + standards references

- **FIPS 140-3 §4.7.5 Purge procedure.** Used by the zeroize
  helper crate.
- **NIST SP 800-57 Rev. 5 "Recommendation for Key Management."**
  Per-tenant key segregation precedent.
- **NIST SP 800-209 "Security Guidelines for Storage Infrastructure"
  §4.2.4.** Key isolation in shared-tenancy storage.
- **NIST SP 800-204D "DevSecOps for CI/CD."** Secret-handling
  guidance.
- **Mozilla's Security Engineering blog (2020-2023) "How we keep
  our root keys safe."** Key-isolation precedent.
- **Linux man pages: `mlock(2)`, `madvise(2)`, `prctl(2)`,
  `seccomp(2)`, `getpeercred(7)`.** OS primitives used by the
  sidecar.
- **rustsec/zeroize crate (v1.x, MIT/Apache 2.0).** The
  `Zeroizing<T>` wrapper used.

### Slice cross-references

- **Slice 1 (runbooks):**
  `docs/runbooks/credential-sidecar-startup-recovery.md`,
  `docs/runbooks/credential-handle-leak-investigation.md`,
  `docs/runbooks/audit-signing-key-rotation.md`,
  `docs/runbooks/credential-sidecar-upgrade-procedure.md`,
  `docs/runbooks/sidecar-rce-blast-radius-rehearsal.md` are
  required by this ADR's CI lanes; their authoring is in Slice 1
  scope.
- **Slice 3 (ADR-0246 amendment):** The
  `oya-check-library-first-credential-sidecar` CI lane is added
  to ADR-0246's fragment-validation lane catalogue; the actual
  amendment to ADR-0246 is in Slice 3 scope.
- **Slice 4 (naming justifications):** The four new names
  (`oyatie.intelligence.credential-sidecar`,
  `oyatie.intelligence.credential-sidecar-attestor`,
  `oya-shared-credential-sidecar-uds`,
  `oya-check-library-first-credential-sidecar`) are justified in
  this ADR's front matter `naming_justifications:` block per
  `feedback_naming_justification`.

### Specifications

- `/specs/library-first-sidecar-uds-protocol.json` (new) —
  canonical machine-readable record of the UDS protocol surface,
  the five exposed operations, and per-operation Cedar permits.
- `/specs/credential-handle-lifecycle.json` (new) — canonical
  record of credential handle lifetime invariants + telemetry
  contract.
- `/specs/byok-credential-model.json` — extended to add the
  sidecar mediation step in the resolution flow.

### Memory references

- `feedback_byok_everywhere_credentials` — substrate owns zero
  credentials; the sidecar is the substrate's surface for
  credential resolution, not a credential store.
- `feedback_intelligence_two_layer_substrate` — AI substrate
  layer A; the sidecar is a substrate-tier component per
  ADR-0245 substrate-vs-product layering.
- `feedback_substrate_vs_product_layering` — the sidecar is
  substrate, not product; it serves every tenant uniformly.
- `feedback_cedar_as_universal_gate` — every sidecar operation
  is Cedar-gated; the protocol's narrow surface translates
  directly into a small set of Cedar permits.
- `feedback_clean_architecture_requirements` — the UDS-only
  surface + the seccomp/AppArmor isolation + the per-tenant
  segregation match the clean-architecture invariants.
- `feedback_no_silent_regression` — the ADR-0255 amendment is
  itself amended through this ADR (documented + CI-enforced),
  not silently changed.
- `feedback_autonomous_implementation_artifacts` — the
  autonomous-masterplan workflows operate through the sidecar
  identically to customer-tenant workflows; no separate path.
- `feedback_naming_justification` — the four new names carry
  inline justification.

---

**End of ADR-0296.**
