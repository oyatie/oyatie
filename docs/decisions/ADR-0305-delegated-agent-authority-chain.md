---
id: ADR-0305
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-ai-safety
  - ops-sre-reliability
  - ops-security
  - ops-trust-and-safety
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-consent
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0255-intelligence-two-layer-substrate.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0272-cookie-consent-per-purpose.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0295-bootstrap-ci-spiffe-kill-switch.md
  - ADR-0296-library-first-credential-sidecar.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-doctrine.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0301-survivor-safety-domestic-abuse-mode.md
  - ADR-0302-deceased-user-inheritance-doctrine.md
  - ADR-0303-cognitive-impairment-decision-resilience.md
  - ADR-0304-cross-jurisdiction-conflict-resolution.md
  - ADR-0306-disaster-mode-cell-resilience.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/identity.json
  - /specs/microservices/api-gateway.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/tenancy.json
  - /specs/delegated-agent-token-schema.json
  - /specs/agent-attestation-chain.json
  - /specs/compliance-pack-schema.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_amazon_shape_cellular_architecture
  - feedback_compliance_pack_primitive
  - feedback_naming_justification
  - feedback_intelligence_two_layer_substrate
  - feedback_self_modification_doctrine
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: critical-path-cluster-delegated-agent-authority-chain
purpose: >
  Establish the Delegated-Agent Authority Chain doctrine — a
  substrate-level primitive that introduces a per-tenant
  `delegated_agent_token` model, a tenant-attested delegation chain,
  bot-management attestation-aware allow paths, scope inheritance
  from the authorizing tenant, cross-tenant delegation blocking, and
  cryptographic audit linkage from the delegated principal back to
  the authorizing human. The bar is: an LLM agent (Anthropic Claude
  or OpenAI assistant or similar), an IFTTT webhook, an n8n workflow
  step, a Zapier task — all legitimate automations acting on behalf
  of a human user — operate without false-positive blocking by
  bot-defence (per ADR-0297), inherit only the tenant scope the
  authorizing human possesses, and produce an audit chain that ties
  back to that human. Per documentation-rigor.md §3.2.5 row 28.
enforcement_status: advisory-until-2026-09-30-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet delegated-agent-token-issuance-coverage
  - cloud-ci/Rust gate packet delegated-agent-attestation-chain
  - cloud-ci/Rust gate packet delegated-agent-cross-tenant-block
  - cloud-ci/Rust gate packet delegated-agent-scope-inheritance
  - cloud-ci/Rust gate packet delegated-agent-audit-linkage
  - cloud-ci/Rust gate packet delegated-agent-bot-defence-allow-path
naming_justifications:
  - name: oya-shared-agent-authority
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.agent-authority
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the delegated-agent-token issuer trait +
      attestation-chain validator trait + scope-inheritance evaluator
      trait + cross-tenant-block enforcer trait + audit-linkage
      emitter trait belongs at the shared layer. Naming
      `oya-shared-agent-authority` keeps the single-concern flat
      layout per ADR-0131 and avoids any "suite" packaging per
      ADR-0132.
  - name: oya-governance-delegated-agent-token-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.delegated-agent-token-coverage
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies
      every µservice that accepts delegated-agent requests declares
      a token-issuance path + attestation-chain validation +
      Cedar fragment integration. Lane naming follows the canonical
      `oya-governance-<concern>` shape consistent with ADR-0297
      sibling lanes.
  - name: oya-governance-agent-attestation-chain
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.agent-attestation-chain
    justification: >
      CI fitness lane per ADR-0212; verifies the attestation chain
      is cryptographically linkable from the delegated agent's token
      back to the authorizing human's primary credential per ADR-
      0188 passkey/WebAuthn.
  - name: oya-governance-cross-tenant-delegation-block
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.cross-tenant-delegation-block
    justification: >
      CI fitness lane per ADR-0212; verifies no delegated-agent
      token can authorize cross-tenant actions even with explicit
      attestation chain.
  - name: oya-governance-agent-scope-inheritance
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.agent-scope-inheritance
    justification: >
      CI fitness lane per ADR-0212; verifies delegated-agent tokens
      inherit only the authorizing human's effective scope at
      delegation time + cannot escalate.
  - name: oya-governance-agent-audit-linkage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.agent-audit-linkage
    justification: >
      CI fitness lane per ADR-0212; verifies every delegated-agent
      action emits an audit-event-class with cryptographic linkage
      back to the authorizing human.
  - name: oya-governance-delegated-agent-authority
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.delegated-agent-authority
    justification: >
      Aggregate fitness lane per ADR-0212; rolls up the child lanes
      into a single advisory/BLOCKER gate per the keystone-bundle
      2026-05-20 promotion-gate model.
  - name: X-Oya-Delegated-Agent-Token
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Delegated-Agent-Token
    justification: >
      Custom HTTP request header carrying the delegated-agent token
      issued by the authorizing human to the delegate. Namespace
      prefix `X-Oya-` reserves the platform's header surface;
      avoids collision with `Authorization` (which carries the
      delegate's own credential).
  - name: X-Oya-Attestation-Chain
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Attestation-Chain
    justification: >
      Custom HTTP request header carrying the JWS-signed attestation
      chain that links the delegated agent's token to the authorizing
      human's primary credential.
  - name: X-Oya-Agent-Class
    layer: N/A (HTTP request header naming)
    bnf_segments: X-Oya.Agent-Class
    justification: >
      Custom HTTP request header declaring the delegate's class
      (`llm_agent`, `webhook`, `workflow_step`, `cli_tool`,
      `mobile_app`, `desktop_app`); used by bot-defence per ADR-0297
      to set the appropriate allow-path.
  - name: DelegatedAgentTokenIssued
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AgentAuthority.DelegatedAgentTokenIssued
    justification: >
      Audit-event-class emitted whenever a delegated-agent token is
      issued by an authorizing human. Registered per ADR-0263.
  - name: DelegatedAgentTokenInvoked
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AgentAuthority.DelegatedAgentTokenInvoked
    justification: >
      Audit-event-class emitted whenever a delegated-agent token is
      used to authorize an action. Registered per ADR-0263.
  - name: DelegatedAgentTokenRevoked
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AgentAuthority.DelegatedAgentTokenRevoked
    justification: >
      Audit-event-class emitted whenever a delegated-agent token is
      revoked by the authorizing human or by the substrate. Registered
      per ADR-0263.
  - name: CrossTenantDelegationBlocked
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AgentAuthority.CrossTenantDelegationBlocked
    justification: >
      Audit-event-class emitted when a delegated-agent token attempts
      cross-tenant action. Registered per ADR-0263.
  - name: AgentScopeEscalationAttempted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: AgentAuthority.AgentScopeEscalationAttempted
    justification: >
      Audit-event-class emitted when a delegated-agent token attempts
      action outside the inherited scope. Registered per ADR-0263.
  - name: policy/delegated-agent.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.delegated-agent
    justification: >
      Canonical filename for the per-µservice delegated-agent Cedar
      fragment under the µservice's `policy/` directory per ADR-0246
      + ADR-0243 fragment-lifecycle conventions; single-concern
      naming keeps the policy directory's contract-by-name invariant.
  - name: iac/<env>-delegated-agent.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.delegated-agent
    justification: >
      Canonical filename for per-µservice + per-env delegated-agent
      IaC manifest declaring per-route allow-classes + per-scope
      restrictions.
  - name: DELEGATED_AGENT
    layer: N/A (Principal.principal_type enum value per ADR-0244)
    bnf_segments: principal_type.DELEGATED_AGENT
    justification: >
      Principal.principal_type enum extension per ADR-0244 §D-3;
      identifies principals that are delegated agents (LLM agents,
      webhooks, workflow steps, etc.) acting on behalf of an
      authorizing human; distinct from `HUMAN` and `WORKLOAD`
      principal types.
---

# ADR-0305: Delegated-Agent Authority Chain Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-cluster-delegated-agent-authority-
chain** keystone, closing the gap identified in
`docs/standards/documentation-rigor.md` §3.2.5 row 28 of the
critical-path edge-case coverage matrix. The standard already
codifies row 28 handling requirements (per-tenant
`delegated_agent_token` model; tenant-attested delegation chain;
bot-mgmt sees attestation and allows; delegated agent inherits
tenant scope; cross-tenant delegation blocked; audit chains
delegated principal back to authorizing human); this ADR is the
binding ADR the standard's row 28 cites.

Enforcement is `advisory-until-2026-09-30-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes promote to
BLOCKER on 2026-09-30 to give per-µservice integration time to land.
Until 2026-09-30, validators emit findings without failing CI;
post-2026-09-30, the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why delegated-agent authority is a substrate primitive

Modern hyperscaler platforms — Microsoft, Anthropic, OpenAI, Google,
Salesforce, Atlassian, Slack, Notion, Zapier — all ship delegated-
agent authority chains as *first-class substrate primitives*. The
pattern is unambiguous across the named industry references:

- **Microsoft Copilot + Microsoft Graph delegated permissions.** Per
  Microsoft Graph documentation 2024 + the Entra ID v2.0 OAuth flow,
  a Copilot agent operates with `act_on_behalf_of` claims tied to
  the authorizing user's MSAL token. The agent's authority is
  inherited from the user; cross-tenant scope is blocked at the
  graph-API gate; every action emits an audit-log entry linking the
  agent's session to the user's identity. Microsoft's pattern is
  the canonical enterprise-scale reference.
- **Anthropic Claude API + computer-use + agent SDK.** Per
  Anthropic's 2024-2025 documentation (Claude 3.5 Sonnet computer-
  use, Claude 4 agent SDK, Model Context Protocol MCP, Claude
  Workspace), an agent acting on a user's behalf uses an API key
  scoped to that user's workspace; the API logs every model
  invocation; the agent cannot reach data outside the user's
  workspace.
- **OpenAI Assistants API + GPT Actions + ChatGPT plugins.** Per
  OpenAI's 2024 documentation, assistants are bound to the
  authorizing user's API key + a per-assistant scope; OAuth-based
  plugins use the user's third-party token; the agent's actions
  are logged in the OpenAI usage dashboard tied to the user's
  account.
- **Google Workspace add-ons + Apps Script + Vertex AI agents.** Per
  Google Workspace add-ons documentation, add-ons use OAuth 2.0
  with the user's consented scope; the user explicitly grants per-
  scope permissions; the add-on cannot exceed those scopes;
  audit-logs are linked to the user via Cloud Audit Logs.
- **Salesforce Einstein + Slack apps + Atlassian Forge.** Per their
  respective 2024 documentation, every app uses an OAuth 2.0
  delegated-permissions model; the user explicitly grants per-scope
  permissions; cross-org actions are blocked; audit-logs link the
  app's session back to the user's identity.
- **Zapier + IFTTT + n8n + Make.com workflows.** Per their respective
  documentation, every workflow step uses a per-user OAuth token
  for the third-party integration; the workflow cannot exceed the
  user's authorized scope; the user can revoke the token; audit-logs
  show which workflow step performed which action.
- **GitHub Apps + GitHub Actions + Dependabot.** Per GitHub's 2024
  documentation, GitHub Apps use a per-installation `installation_id`
  + scoped permissions; GitHub Actions use a workflow-specific
  `GITHUB_TOKEN` with minimum-required scope; Dependabot operates
  as a service user with bounded permissions.

The corollary: **every internet-facing surface oyatie ships MUST
inherit delegated-agent authority chain from the substrate, not
author it per-µservice.** A µservice that authors its own delegated-
agent token issuance, its own attestation-chain validation, its own
scope-inheritance logic, its own cross-tenant block is duplicating
substrate primitives that the shared `oya-shared-agent-authority`
crate already serves. That duplication is a
`feedback_no_silent_regression` violation; a
`feedback_quality_performance_scalability_bar` violation; and a
`feedback_autonomous_implementation_artifacts` violation.

The ADR-0305 delegated-agent authority chain doctrine closes this
gap.

### §A.1. The delegated-agent landscape 2026 — the surface the substrate serves

The 2026 delegated-agent landscape is qualitatively richer than any
prior era:

- **LLM agents (Anthropic, OpenAI, Google).** Anthropic Claude
  4-class agents executing computer-use, MCP tool-use, code-
  execution, and multi-step workflows. OpenAI Assistants + GPT
  Actions invoking external APIs on behalf of users. Google Vertex
  AI agents + Gemini-powered Workspace add-ons. Per Anthropic's
  2024 Q4 disclosures, ~150M+ Claude API calls per day; per
  OpenAI's 2024-Q4 disclosures, ~3.5B daily ChatGPT messages —
  scale demands substrate primitives, not per-tenant code.
- **Webhook integrations (IFTTT, Zapier, n8n, Make, Pipedream,
  Tray.io, Workato).** Per Zapier's 2024 disclosures, ~5 million
  Zapier users, ~30 million daily Zap executions. n8n's 2024 self-
  hosted base ~150k+ instances. The volume + diversity of webhook
  authorities demands substrate primitives.
- **Workflow steps (oyatie's Workflow Studio + n8n + Temporal +
  Airflow + Prefect + Dagster).** Per the oyatie Workflow Studio
  PRD (M-CC-P11 substrate scope) + the n8n/Temporal precedent, a
  workflow step is a delegated agent invoking external services
  + internal substrate primitives on behalf of the workflow
  initiator.
- **Bot-mgmt false-positive concern.** Per ADR-0297, the substrate
  ships bot-defence at planetary edge. Legitimate delegated agents
  (LLM agents, webhooks, workflow steps) are at high risk of
  false-positive bot-blocking because they exhibit bot-like
  behaviour (high request rate, predictable patterns, programmatic
  User-Agent). The substrate MUST provide an attestation path
  that bot-mgmt sees + allows.
- **Cross-tenant delegation risk.** A delegated agent authorized
  by user A in tenant X must not be able to act on tenant Y. The
  substrate enforces cross-tenant blocking even with explicit
  attestation chain.
- **Scope-escalation risk.** A delegated agent must not exceed the
  authorizing user's scope at delegation time. If user A grants
  a read-only delegation, the delegate cannot write.
- **Token-revocation lag.** When a user revokes a delegation, the
  substrate must propagate the revocation across cells within
  bounded time + audit the revocation.

The substrate baseline MUST be sized to this 2026 landscape. The bar
is not "OAuth tokens work"; the bar is "operate a per-tenant
delegation chain with cryptographic linkage back to the authorizing
human, with per-cell ≤2s revocation propagation, with bot-defence
allow-path attestation, and with deterministic cross-tenant blocking
at every gate."

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate primitive

The keystone bundle's foundational ADRs intersect delegated-agent
authority as follows:

- **ADR-0188 (passkey/WebAuthn canonical auth).** The authorizing
  human's primary credential is a passkey. The delegated-agent
  token's attestation chain links back to the passkey.
- **ADR-0242 (oyatie-is-a-tenant).** The platform's own automation
  agents (e.g., observability scrapers, foundry CI runners) are
  tenant-scoped delegated agents acting on behalf of the platform-
  tenant.
- **ADR-0243 (Cedar universal gate).** Every delegated-agent
  decision composes as a Cedar fragment. The attestation-chain
  validity + scope-inheritance + cross-tenant block all enter
  Cedar evaluation.
- **ADR-0244 (tenant scoping primitive).** Delegated-agent tokens
  are tenant-scoped; cross-tenant delegation blocked at the gate.
- **ADR-0245 (substrate vs product).** Delegated-agent authority is
  substrate; the per-product agent surfaces (LLM agent UI, workflow
  studio agent UI, webhook integration UI) consume the substrate.
- **ADR-0247 (self-modification doctrine).** Foundry agents (the
  self-modification surface) operate as delegated agents under
  Cedar; the substrate's `oyatie.foundry.*` principal namespace
  follows ADR-0305's authority chain.
- **ADR-0248 (Amazon-shape cellular architecture).** Delegated-
  agent token issuance + attestation cache is cell-local; cross-
  cell propagation via the audit-chain only.
- **ADR-0251 (compliance packs).** Per-pack rules constrain
  delegated-agent scope (e.g., HIPAA forbids PHI delegation
  outside business-associate agreements; GDPR Art. 28 requires
  data-processor contracts).
- **ADR-0253 (HTTP/3 + QUIC).** Delegated-agent tokens are
  presented in HTTP headers over HTTP/3+QUIC.
- **ADR-0255 (intelligence two-layer).** Intelligence-layer LLM
  agents (the AI Substrate) operate as delegated agents under
  ADR-0305.
- **ADR-0263 (observability emission contract).** Every delegated-
  agent action emits an audit-event-class linking back to the
  authorizing human.
- **ADR-0292 (minor user doctrine).** Minor users have restricted
  delegation; COPPA/KOSA/AADC packs forbid delegation of minor-
  PII to third-party agents without parental consent.
- **ADR-0295 (bootstrap CI SPIFFE).** Workload identity per SPIFFE
  is distinct from delegated-agent identity; SPIFFE SVIDs are
  workload-to-workload; delegated-agent tokens are human-to-agent.
- **ADR-0296 (library-first credential sidecar).** Delegated-agent
  tokens are stored + retrieved via the credential sidecar.
- **ADR-0297 (abuse-defence baseline).** Bot-defence consults the
  delegated-agent attestation; legitimate agents allowed, illegitimate
  agents (no attestation) bot-defended.
- **ADR-0298 (emergency-services bypass).** Emergency-services
  paths bypass delegated-agent restrictions where life-safety
  requires; per ADR-0298 doctrine.
- **ADR-0299 (account-recovery resilience).** Post-recovery, all
  prior delegated-agent tokens are invalidated; new delegations
  require re-authorization.
- **ADR-0303 (decision-resilience).** Delegated agents invoking
  consequential mutations on behalf of the user trigger the
  user's cooling-off + trusted-contact alerts.
- **ADR-0304 (cross-jurisdiction conflict resolution).** Per-pack
  delegation constraints (e.g., GDPR Art. 28 data-processor
  contracts, HIPAA business-associate agreements) compose with
  delegated-agent authority.

The bundle cannot land without the delegated-agent authority chain
articulated explicitly. The promotion gate for the 2026-05-20 bundle
is: *the substrate MUST authorize legitimate delegated agents
without false-positive bot-blocking, while enforcing tenant-scope
inheritance + cross-tenant blocking + cryptographic audit linkage.*
This ADR is the binding articulation.

### §A.3. What this ADR explicitly does NOT do

- This ADR does not specify per-vendor LLM agent SDK shape — that
  is the per-vendor SDK (Anthropic, OpenAI, Google, etc.). This
  ADR specifies the substrate's token + attestation + audit shape.
- This ADR does not redefine OAuth 2.0 — that is RFC 6749 + RFC
  8628. This ADR specifies how the substrate's tokens compose
  with OAuth 2.0.
- This ADR does not displace the per-µservice SPIFFE workload
  identity per ADR-0295. Workloads are distinct from delegated
  agents.
- This ADR does not specify the per-tenant agent-management UI;
  that is the tenancy substrate per ADR-0244 + the per-product
  surface.
- This ADR does not redefine Cedar fragment authoring conventions —
  that is ADR-0243 + ADR-0294. This ADR specifies the *content*
  of `policy/delegated-agent.cedar`.
- This ADR does not specify the per-pack legal-contract templates
  (GDPR Art. 28 DPA, HIPAA BAA, etc.); those are per-pack legal-
  council axis responsibilities.

## Decision

### §B. Five core primitives at three layers

The delegated-agent authority chain is **five core primitives**
(token issuance; attestation chain; scope inheritance; cross-tenant
block; audit linkage) wired at **three layers** (Tier-0 shared
crate, per-µservice gate, Cedar policy fragment). The 5×3 matrix
produces fifteen cells; each cell has a defined primitive.

```
                    Tier-0 shared             Per-µservice            Cedar policy
                    -------------             -------------           -------------
Token issuance      Token issuer +            Per-route consumer +   forbid when
                    JWS/COSE signing           token validation       token invalid

Attestation chain   Chain validator +         Per-µservice chain     forbid when
                    primary-credential         lookup                  chain_break_detected
                    cache

Scope inheritance   Effective-scope           Per-resource scope     forbid when
                    evaluator                  check                   agent_scope ⊄
                                                                        authorizer_scope

Cross-tenant block  Tenant-scope enforcer     Per-µservice tenant    forbid when
                                               attestation check       agent_tenant ≠
                                                                        target_tenant

Audit linkage       Per-action audit emit +   Per-µservice audit     permit but emit
                    Merkle-anchor              passthrough             AgentInvocation
                                                                       AuditedEvent
```

The five primitives are **interdependent**:

- **Token issuance** is the entry point. The authorizing human (via
  passkey re-auth per ADR-0188) issues a delegated-agent token to a
  named delegate (LLM agent, webhook URL, workflow-step ID, etc.).
- **Attestation chain** binds the token to the authorizing human's
  primary credential cryptographically. Every action carrying the
  token MUST present the attestation chain.
- **Scope inheritance** restricts the delegate's effective authority
  to a subset of the authorizing human's effective scope at
  delegation time. The delegate cannot escalate.
- **Cross-tenant block** prevents the delegate's token from being
  used to act on a different tenant — even if the authorizing
  human has access to multiple tenants, the delegation is per-
  tenant-scoped.
- **Audit linkage** ensures every delegated-agent action emits an
  audit-event-class that links back to the authorizing human + the
  delegation event + the delegate's session.

The three layers are **complementary**:

- **Tier-0 shared crate** centralizes the token issuer, the
  attestation-chain validator, the scope-inheritance evaluator,
  the cross-tenant enforcer, the audit-linkage emitter.
- **Per-µservice gate** sees the µservice-local request context
  (route, resource, action). The gate validates the token +
  consults the attestation cache.
- **Cedar policy fragment** composes the substrate + µservice +
  per-tenant + per-pack signals into a permit/forbid decision per
  ADR-0243 + ADR-0263.

### §B.1. Token shape — JWS-signed delegation token

The canonical delegated-agent token is a JWS (RFC 7515) using EdDSA
with Ed25519 keys per ADR-0188 cryptographic doctrine. The token
payload:

```json
{
  "iss": "https://issuer.oyatie.com/v1",
  "sub": "agent:llm:claude:workspace-fe3b2c01",
  "aud": "https://api.oyatie.com",
  "iat": 1716200000,
  "exp": 1716286400,
  "nbf": 1716200000,
  "jti": "8e7c4d2a-1b3f-4956-9c0e-7a8d2b3e4f5a",

  "oya": {
    "version": 1,
    "principal_type": "DELEGATED_AGENT",
    "agent_class": "llm_agent",
    "agent_vendor": "anthropic",
    "agent_product": "claude-3-5-sonnet",

    "authorizing_principal": {
      "type": "HUMAN",
      "id": "user:tenant-abc:user-456",
      "tenant_id": "tenant-abc",
      "primary_credential_kind": "passkey",
      "primary_credential_handle_hash": "sha256:f3a1b2c3d4..."
    },

    "tenant_scope": "tenant-abc",
    "effective_scope": [
      "read:resources:notes:*",
      "write:resources:notes:project-X",
      "read:resources:files:project-X"
    ],
    "scope_floor": "subset_of_authorizer_at_delegation_time",

    "attestation_chain_id": "att-chain-9c8b7a6d5e4f3a2b1c0d",
    "delegation_event_id": "del-event-1234abcd",

    "audit_linkage": {
      "audit_chain_anchor": "merkle:0xabcd1234...",
      "emit_on_every_action": true
    },

    "compliance_packs": ["pack-eu-gdpr", "pack-us-ccpa"],
    "cross_tenant_block": true,
    "scope_escalation_block": true
  }
}
```

**Signing key:**

The token is signed by the issuer's EdDSA Ed25519 signing key.
The key pair is per-cell-local + rotated per ADR-0294 lifecycle
(90-day rotation, signed publication).

**Token expiry:**

- Default expiry: 24 hours (`exp = iat + 86400`).
- LLM-agent expiry: 1 hour (`exp = iat + 3600`).
- Webhook expiry: 7 days (`exp = iat + 604800`).
- Workflow-step expiry: per-workflow declared, max 30 days.
- Mobile-app / desktop-app expiry: 90 days, rotated via refresh-
  token.
- High-privilege scope expiry: 1 hour absolute ceiling regardless
  of agent class.

**Token revocation:**

Tokens are revocable via:

1. The authorizing user revokes via the tenancy substrate per
   ADR-0244.
2. The substrate revokes on detected abuse (per ADR-0297 +
   ADR-0303 override-fatigue).
3. Account recovery per ADR-0299 invalidates all prior tokens.
4. Tenant-admin revocation via the tenancy substrate.

Revocation propagation is ≤ 2 seconds across all cells via the
substrate's revocation broadcast bus.

### §B.2. Attestation chain — full mechanics

The attestation chain cryptographically links the delegated-agent
token back to the authorizing human's primary credential. The
chain is a JWS-signed sequence:

```
Attestation Chain (JWS, multi-signature):

  Signature 1: Authorizing human's passkey (Ed25519, ADR-0188)
    Payload: {
      "principal_id": "user:tenant-abc:user-456",
      "delegate_id": "agent:llm:claude:workspace-fe3b2c01",
      "delegate_scope": [...],
      "delegation_event_id": "del-event-1234abcd",
      "issued_at": 1716200000,
      "expires_at": 1716286400
    }

  Signature 2: Tenant substrate's tenant-attestation key
    Payload: {
      "delegation_event_id": "del-event-1234abcd",
      "tenant_id": "tenant-abc",
      "tenant_audience_type": "B2C_CONSUMER"
    }

  Signature 3: Substrate issuer's signing key (the token issuer)
    Payload: {
      "delegation_event_id": "del-event-1234abcd",
      "token_jti": "8e7c4d2a-1b3f-4956-9c0e-7a8d2b3e4f5a",
      "compliance_packs_applied": [...]
    }
```

**Validation algorithm:**

On every request bearing a delegated-agent token + attestation
chain header:

1. Validate the JWS signatures on the chain (passkey + tenant +
   issuer).
2. Verify the chain's `delegation_event_id` matches the token's
   `delegation_event_id`.
3. Verify the authorizing human's passkey is registered + active
   per ADR-0188.
4. Verify the chain has not been revoked (per the revocation
   broadcast).
5. Verify the chain's `expires_at` is in the future.
6. Verify the tenant attestation matches the token's
   `tenant_scope`.
7. Verify the issuer signature matches a known cell-local issuer
   public key.

If any step fails, the substrate emits `AttestationChainInvalid` +
refuses the request with 401.

### §B.3. Scope inheritance — the no-escalation invariant

The delegated agent inherits a **subset** of the authorizing human's
effective scope at delegation time. The no-escalation invariant:

> The delegate's effective scope MUST be a subset of the authorizing
> human's effective scope at delegation time. The delegate cannot
> exceed; if the human's scope is subsequently increased, the
> delegate's scope is NOT auto-elevated.

In practice:

- **Subset-only delegation.** The user explicitly chooses which
  scopes to delegate. The substrate enforces subset-only at
  delegation time.
- **No transitive escalation.** A delegate cannot create sub-
  delegates with broader scope. If LLM-agent A creates a workflow-
  step B, B's scope ⊆ A's scope ⊆ user's scope.
- **No tenant-scope elevation.** A delegate authorized by a user
  who has roles in multiple tenants is scoped to a single tenant
  per delegation event. Acting on a different tenant requires a
  separate delegation event.
- **No principal-class elevation.** A delegate cannot impersonate
  the authorizing human's passkey or primary credential. The
  delegated-agent token represents the agent's class, not the
  human's class.
- **Scope-floor invariant.** Per-pack regulator floor (e.g.,
  HIPAA's BAA requirement) cannot be evaded by delegation. If a
  scope requires a BAA, the delegate must be in a BAA-attested
  cohort + the delegation event records the BAA-attestation.

### §B.4. Cross-tenant block — the boundary invariant

The cross-tenant block is enforced at every gate. The boundary
invariant:

> A delegated-agent token issued for tenant X cannot authorize an
> action on tenant Y, even if the authorizing human has access to
> both tenants.

In practice:

- **Per-tenant delegation event.** Each delegation event names a
  single tenant. A user with multi-tenant access creates separate
  delegation events per tenant.
- **Token tenant_scope is total.** The token's `oya.tenant_scope`
  is a single string; not an array.
- **Per-route tenant validation.** Every per-µservice route extracts
  the target tenant from the request + verifies it equals the
  token's `tenant_scope`. Mismatch → 403 + `CrossTenantDelegation
  Blocked` event class.
- **Cross-tenant query block.** Aggregate queries that span tenants
  are forbidden for delegated agents. The agent must issue a
  per-tenant query.

### §B.5. Bot-defence attestation-aware allow path

Per ADR-0297, the substrate ships bot-defence at planetary edge.
Legitimate delegated agents (LLM agents, webhooks, workflow steps)
are at high risk of false-positive bot-blocking. The substrate
MUST provide an attestation-aware allow path.

**Bot-defence flow with delegated-agent attestation:**

1. Request arrives at Tier-0 edge bearing `X-Oya-Delegated-Agent-
   Token` + `X-Oya-Attestation-Chain` headers.
2. Edge bot-mgmt computes bot-score per ADR-0297.
3. Edge bot-mgmt checks the attestation header:
   - If absent → standard bot-defence per ADR-0297 (high bot-score
     → challenge or refuse).
   - If present → fast-path validate the attestation chain via
     cell-local cache + the `agent_class` header → allow per-class
     rate-limit.
4. If attestation valid + `agent_class = llm_agent` → apply
   LLM-agent rate-limit (per-tenant configurable; default 1000
   req/min per agent).
5. If attestation valid + `agent_class = webhook` → apply webhook
   rate-limit (default 10 req/sec per webhook).
6. If attestation valid + `agent_class = workflow_step` → apply
   per-tenant workflow rate-limit (default 100 req/sec per
   workflow).
7. If attestation invalid → bot-defence proceeds standard;
   substrate emits `AttestationChainInvalid` event class.

This integration is the substrate's resolution to row 28 of the
critical-path matrix: legitimate automation gets through; illegitimate
automation gets bot-defended.

### §B.6. The audit-linkage invariant — every action traceable

Every delegated-agent action emits an audit-event-class that
cryptographically links back to the authorizing human + the
delegation event + the agent's session.

**Audit linkage shape:**

```json
{
  "event_class": "DelegatedAgentTokenInvoked",
  "request_id": "req-xyz123",
  "delegate_principal_id": "agent:llm:claude:workspace-fe3b2c01",
  "delegate_class": "llm_agent",
  "delegate_vendor": "anthropic",
  "authorizing_principal_id": "user:tenant-abc:user-456",
  "delegation_event_id": "del-event-1234abcd",
  "attestation_chain_id": "att-chain-9c8b7a6d5e4f3a2b1c0d",
  "action": "write_note",
  "resource": "note:project-X:note-001",
  "tenant_id": "tenant-abc",
  "audit_chain_anchor": "merkle:0xabcd1234...",
  "audit_chain_predecessor": "merkle:0xfedc4321..."
}
```

**Query surfaces:**

- **By authorizing human.** "Show all actions performed by any
  agent on my behalf in the last 30 days." Returns every event
  where `authorizing_principal_id = user:X`.
- **By delegation event.** "Show all actions performed under
  delegation event Y." Returns every event where
  `delegation_event_id = Y`.
- **By delegate.** "Show all actions performed by this LLM agent
  session." Returns every event where `delegate_principal_id = Z`.
- **By tenant admin.** "Show all delegated-agent activity in my
  tenant." Returns aggregated stats; per-user detail requires
  per-user query consent.

## §C. Consequences

### §C.1. Maintainability dimension

The delegated-agent authority chain baseline serves an unbounded
diversity of delegate classes. Maintainability invariants:

- **Per-agent-class config is data, not code.** Each agent-class
  (`llm_agent`, `webhook`, `workflow_step`, `cli_tool`, etc.)
  declares its rate-limit ceiling, expiry default, allowed
  `agent_vendor` set, and bot-defence allow-path config in a
  YAML registry. No code change to add a new agent-class.
- **Per-µservice declaration is configuration.** Each µservice
  declares its delegated-agent posture in `ARCHITECTURE.md
  §delegated-agent` + `iac/<env>-delegated-agent.yaml` +
  `policy/delegated-agent.cedar`.
- **Per-tenant tuning is configuration.** Tenants tune per-agent-
  class rate-limits + allow-vendor sets via the tenancy substrate.
- **Versioning policy.** The Cedar fragment + IaC manifest follow
  ADR-0294 + ADR-0258. Token format SemVer at `oya.version`.
- **Deprecation cadence.** Token format major-version bumps
  follow 12-month deprecation cadence; old tokens accepted in
  parallel during the transition.
- **Single-concern crate.** The shared crate is single-concern per
  ADR-0131. It does NOT absorb workload-identity (that is ADR-0295
  SPIFFE) or auth-flow-engineering (that is ADR-0188 passkey).
- **Tests as inheritance proof.** Every µservice that accepts
  delegated-agent requests MUST ship contract tests per the
  shared crate's fixtures.
- **Documentation density.** Each µservice's PRD MUST cite which
  agent-classes are accepted, which rate-limits apply, and which
  audit-event-classes are emitted.

### §C.2. Observability dimension

Per ADR-0263:

- **Audit-event-classes:**
  - `DelegatedAgentTokenIssued`
  - `DelegatedAgentTokenInvoked`
  - `DelegatedAgentTokenRevoked`
  - `DelegatedAgentTokenExpired`
  - `AttestationChainInvalid`
  - `CrossTenantDelegationBlocked`
  - `AgentScopeEscalationAttempted`
  - `AgentAttestationFastPathAllowed`
  - `AgentRateLimitExceeded`
  - `AgentAuditQueryEmitted`
- **Metrics:**
  - `oya_agent_authority_token_issue_counter` — tokens issued.
    Dimensions: agent_class, agent_vendor, tenant_bucket.
  - `oya_agent_authority_token_invoke_counter` — invocations.
    Dimensions: agent_class, tenant_bucket, µservice.
  - `oya_agent_authority_token_revoke_counter` — revocations.
    Dimensions: agent_class, revocation_cause.
  - `oya_agent_authority_attestation_validate_latency_histogram` —
    attestation validation latency.
  - `oya_agent_authority_cross_tenant_block_counter` — cross-
    tenant blocks.
  - `oya_agent_authority_scope_escalation_attempt_counter` —
    escalation attempts.
  - `oya_agent_authority_rate_limit_exceeded_counter` — rate-limit
    exceeded events.
- **Dashboards:** Each µservice that accepts delegated agents
  ships `dashboards/delegated-agent.json` with the canonical 10-
  panel layout.

### §C.3. Scalability dimension

The substrate scales to ~150M+ Claude-class agent invocations per
day across the platform's eventual scale:

- **Cell-local attestation cache.** O(active_delegations) state
  per cell; bounded ≤ 100M per cell typical; ≤ 1 KB per cache
  entry; per-cell ≤ 100 GB — within ADR-0263 cardinality budget.
- **Per-tenant token issuance.** Per-tenant issuance rate
  bounded; default 1k issuances/sec per tenant; configurable.
- **Hot-path performance.** Attestation validation is cell-local
  JWS verification + cache lookup; O(1); target p99 latency ≤
  3 ms.
- **Revocation propagation.** Substrate-wide via the kill-switch
  broadcast bus per ADR-0295; bounded ≤ 2 seconds end-to-end.
- **Burst capacity.** LLM-agent burst (e.g., a Claude session
  generating 10k tool-calls in 60s) accommodated by per-agent
  rate-limit + per-tenant pool.

### §C.4. Performance dimension

- **Attestation validation latency.** p50 ≤ 1 ms; p99 ≤ 3 ms;
  p99.9 ≤ 10 ms.
- **Token issuance latency.** p50 ≤ 20 ms (includes passkey
  re-auth challenge); p99 ≤ 100 ms.
- **Revocation propagation.** p50 ≤ 0.5 s; p99 ≤ 2 s end-to-end.
- **CPU budget per request.** ≤ 30 μs CPU including attestation
  validation + Cedar evaluation.
- **Memory budget.** ≤ 1 KB per active attestation cache entry;
  ≤ 100 GB per cell for 100M cached entries.

### §C.5. Optimization dimension

- **Cell-local attestation cache.** Cached attestations avoid
  repeated JWS verification.
- **Pre-computed agent-class allow-paths.** Bot-defence consults
  pre-computed allow-paths per agent-class; no per-request
  computation.
- **Batched revocation.** Multiple revocations within a 100ms
  window are batched into a single broadcast.
- **Cross-µservice attestation reuse.** Attestation validated at
  the api-gateway is propagated via a request-scoped header so
  downstream µservices need not re-validate.
- **Per-tenant token pool.** Tenants with high agent-volume can
  pre-issue token pools that the substrate amortizes issuance
  cost over.

### §C.6. Code quality dimension

- **Single ingress trait.** `DelegatedAgentGate::validate_or_deny()`;
  no µservice authors its own attestation logic.
- **No `#[cfg(test)]` bypass.** Cedar evaluates in test as in prod.
- **Mandatory documentation.** Every µservice with delegated-agent
  acceptance MUST include `compliance.md §delegated-agent-edge-
  cases`.
- **Deterministic test fixtures.** The shared crate ships
  fixtures for canonical agent classes.
- **No magic numbers.** All rate-limits, expiry defaults declared
  in `iac/<env>-delegated-agent.yaml`.
- **Audit-event-class registration enforcement.**
- **Property-based test coverage.** ≥ 85% per ADR-0212.

## §D. Detailed mechanics

### §D-1. Worked example — Anthropic Claude LLM agent

Scenario: a user in tenant `tenant-abc` uses Claude Desktop with
the oyatie MCP integration. The user authorizes Claude to read
their notes + create new notes in project-X.

**Step 1 — User initiates delegation.**

User clicks "Authorize Claude" in the oyatie Tenancy UI. The UI
displays the scope grant:

- Read all notes in project-X.
- Create new notes in project-X.

The user re-authenticates with their passkey per ADR-0188.

**Step 2 — Substrate issues delegation event.**

The Tenancy substrate emits a `DelegationEvent`:

```json
{
  "delegation_event_id": "del-event-abc123",
  "authorizing_principal_id": "user:tenant-abc:user-456",
  "delegate_principal_id": "agent:llm:anthropic:claude-desktop-user-456",
  "delegate_class": "llm_agent",
  "delegate_vendor": "anthropic",
  "delegate_product": "claude-desktop",
  "tenant_scope": "tenant-abc",
  "effective_scope": [
    "read:resources:notes:project-X",
    "write:resources:notes:project-X"
  ],
  "expires_at": 1716286400,
  "passkey_attestation_id": "passkey-att-xyz",
  "compliance_packs": ["pack-us-ccpa"]
}
```

**Step 3 — Substrate issues token + attestation chain.**

The substrate issues:

- A JWS-signed delegated-agent token per §B.1.
- An attestation chain per §B.2 (3 signatures: user passkey,
  tenant attestation, issuer).

The token + chain are returned to Claude Desktop via the OAuth-
class consent-redirect flow.

**Step 4 — Claude invokes oyatie API.**

Claude makes a request to `https://api.oyatie.com/v1/notes/project-X`
with:

- `Authorization: Bearer <claude's OAuth token>` (its own).
- `X-Oya-Delegated-Agent-Token: <delegation token>`.
- `X-Oya-Attestation-Chain: <attestation chain JWS>`.
- `X-Oya-Agent-Class: llm_agent`.

**Step 5 — Edge bot-defence (per ADR-0297 + §B.5).**

Edge sees `X-Oya-Agent-Class: llm_agent` + valid attestation chain.
Bot-defence applies LLM-agent allow-path: rate-limit 1000 req/min;
request proceeds to api-gateway.

**Step 6 — api-gateway validates attestation.**

api-gateway invokes `DelegatedAgentGate::validate_or_deny()`. The
gate validates the chain, the tenant-scope match, the expiry.

**Step 7 — Per-µservice scope check.**

Notes µservice receives the request + the validated attestation.
The Cedar fragment evaluates `policy/delegated-agent.cedar` +
`policy/notes.cedar` together. Both permit: the agent has
`write:resources:notes:project-X` scope; the action is permitted.

**Step 8 — Audit emission.**

Notes µservice emits `DelegatedAgentTokenInvoked` with the audit-
linkage payload per §B.6.

**Step 9 — Response returned.**

The note is created. Claude's request returns successfully.

### §D-2. Worked example — IFTTT webhook

Scenario: a user creates an IFTTT recipe: "When my Twitter mentions
my brand, post a note in oyatie."

**Step 1 — User authorizes IFTTT.**

User installs the oyatie IFTTT applet. IFTTT redirects to oyatie's
OAuth consent flow. User re-authenticates with passkey + grants:

- Create notes in project "Brand-mentions."

**Step 2 — Substrate issues delegation event.**

```json
{
  "delegation_event_id": "del-event-ifttt-789",
  "authorizing_principal_id": "user:tenant-xyz:user-789",
  "delegate_principal_id": "agent:webhook:ifttt:user-789-applet-1234",
  "delegate_class": "webhook",
  "delegate_vendor": "ifttt",
  "tenant_scope": "tenant-xyz",
  "effective_scope": ["write:resources:notes:brand-mentions"],
  "expires_at": 1716286400,
  "compliance_packs": ["pack-us-ccpa"]
}
```

**Step 3 — IFTTT triggers.**

A Twitter mention occurs. IFTTT invokes the oyatie webhook with the
delegation token + attestation chain. Bot-defence sees
`X-Oya-Agent-Class: webhook` + valid attestation; allows.

**Step 4 — Notes µservice writes the note.**

Audit emits `DelegatedAgentTokenInvoked` linking back to user-789.

### §D-3. Worked example — n8n workflow step

Scenario: a tenant ops team uses an n8n workflow that pulls data
from oyatie + processes + writes back.

**Step 1 — Workflow author authorizes the workflow.**

Tenant-admin (or workflow author with delegation rights) creates the
n8n integration. Substrate issues delegation event:

```json
{
  "delegation_event_id": "del-event-n8n-456",
  "authorizing_principal_id": "user:tenant-abc:admin-001",
  "delegate_principal_id": "agent:workflow_step:n8n:tenant-abc-workflow-789",
  "delegate_class": "workflow_step",
  "delegate_vendor": "n8n",
  "tenant_scope": "tenant-abc",
  "effective_scope": [
    "read:resources:metrics:*",
    "write:resources:notes:ops-summary"
  ],
  "expires_at": 1716290000,
  "compliance_packs": ["pack-us-ccpa", "pack-us-sox"]
}
```

**Step 2 — n8n executes workflow.**

n8n schedules the workflow. Each step invokes the oyatie API with
the delegation token. Substrate validates + permits per the
inherited scope.

**Step 3 — Audit emitted per step.**

Each invocation emits `DelegatedAgentTokenInvoked` with the
workflow-step-id + step-execution-id for traceability.

### §D-4. Worked example — Zapier task

Scenario: a tenant uses Zapier to sync oyatie notes to Notion.

**Step 1 — User authorizes Zapier.**

User installs the oyatie Zap. OAuth flow; substrate issues
delegation event with `agent_class = workflow_step, agent_vendor =
zapier`. Effective scope: `read:resources:notes:*`. Expires per
Zapier's documented 24h refresh-token cycle.

**Step 2 — Zap fires.**

A note is created; Zapier invokes oyatie API; substrate validates +
permits; emits `DelegatedAgentTokenInvoked`.

### §D-5. Cedar policy fragment — `policy/delegated-agent.cedar`

```cedar
// policy/delegated-agent.cedar
// Per-µservice Cedar fragment per ADR-0305 + ADR-0243 +
// ADR-0294 fragment-lifecycle.

// Default-deny: delegated-agent action refused unless attestation
// + scope + cross-tenant + audit invariants hold.

forbid (
  principal,
  action,
  resource
)
when {
  principal.principal_type == "DELEGATED_AGENT" &&
  (
    // Predicate 1: attestation chain invalid
    !context.attestation_chain_valid ||
    // Predicate 2: scope escalation attempted
    !context.action_within_inherited_scope ||
    // Predicate 3: cross-tenant block
    context.agent_tenant_scope != context.target_tenant_id ||
    // Predicate 4: token expired
    context.token_exp <= context.now ||
    // Predicate 5: token revoked
    context.token_revoked == true ||
    // Predicate 6: rate-limit exceeded for agent-class
    context.agent_class_rate_limit_exceeded == true
  )
};

// Per-pack delegation prohibition (e.g., HIPAA without BAA)
forbid (
  principal,
  action,
  resource
)
when {
  principal.principal_type == "DELEGATED_AGENT" &&
  resource.data_class == "PHI" &&
  context.applicable_packs.contains("pack-us-hipaa") &&
  !context.baa_attestation_present
};

// Minor-PII delegation prohibition without parental consent
forbid (
  principal,
  action,
  resource
)
when {
  principal.principal_type == "DELEGATED_AGENT" &&
  resource.data_class == "MINOR_PII" &&
  !context.parental_consent_attested
};

// Per-tenant agent-class allow-list
forbid (
  principal,
  action,
  resource
)
when {
  principal.principal_type == "DELEGATED_AGENT" &&
  !context.tenant_allow_agent_class_set.contains(principal.agent_class)
};

// Audit emission required
permit (
  principal,
  action,
  resource
)
when {
  principal.principal_type == "DELEGATED_AGENT" &&
  context.audit_event_will_emit == true
};
```

### §D-6. Per-cell-tier variants

Per ADR-0248:

- **Tier-0 cells (edge POPs).** Bot-defence attestation fast-path.
  No token issuance; no full validation; defer to Tier-1.
- **Tier-1 cells (regional control planes).** Token issuance +
  attestation cache + Cedar gate.
- **Tier-2 cells (data plane regions).** Per-µservice gate;
  consume token; emit audit.
- **Tier-3 cells (compliance-isolated).** Same as Tier-2 with
  per-pack overlays (HIPAA BAA, GDPR Art. 28, etc.).
- **Tier-4 cells (sovereign-cloud).** Delegated-agent restricted
  per per-jurisdiction sovereignty pack.

### §D-7. Observability — metrics, dashboards, audit-event-classes

Per ADR-0263:

**Audit-event-classes:** see §C.2.

**Metrics:** see §C.2.

**Dashboard:** 10-panel canonical layout.

### §D-8. Per-tenant audience-type tuning

| Audience type | Default agent-classes | Default agent-vendors |
|---|---|---|
| `B2C_CONSUMER` | llm_agent, webhook, mobile_app | per-tenant config |
| `B2B_TENANT` | all classes | per-tenant allow-list |
| `SENIOR_PROTECTED` | refused | n/a |
| `MINOR_PII` | refused (parental consent required) | n/a |
| `HIGH_RISK_USER` | webhook only with explicit attestation | n/a |
| `SOVEREIGN_GOV_TENANT` | per-pack restricted | per-pack allow-list |
| `FRIENDLY_CRAWLER_PARTNER` | crawler-specific | per-tenant |

### §D-9. Compliance interactions

- **GDPR Article 28 (data processor).** Delegated agents acting
  on personal data require a data-processor contract; substrate
  surfaces the requirement at delegation time.
- **HIPAA Business-Associate Agreement (BAA).** Delegated agents
  accessing PHI require an executed BAA; substrate enforces.
- **EU AI Act Article 14 (human oversight).** Per-pack rules
  constrain LLM-agent autonomy; substrate composes.
- **EU AI Act Article 50 (transparency).** AI-system-generated
  content must be marked as such; substrate emits via the
  delegate's audit metadata.
- **COPPA + KOSA + AADC.** Minor-PII delegation forbidden without
  parental consent.
- **CCPA + CPRA.** Consumer right to know which agents have
  delegation grants; substrate provides query surface.
- **SOC 2 CC6.6.** Delegated-agent access monitoring; substrate's
  audit events satisfy.

## §E. Implementation footprint

### §E.1. New crate

```
oya-shared-agent-authority/
├── Cargo.toml                            # workspace crate, single-concern
├── src/
│   ├── lib.rs                            # DelegatedAgentGate trait
│   ├── token/
│   │   ├── mod.rs                        # token submodule
│   │   ├── issuer.rs                     # JWS signer
│   │   ├── validator.rs                  # JWS validator
│   │   ├── claims.rs                     # token claims struct
│   │   └── revocation.rs                 # revocation bus client
│   ├── attestation/
│   │   ├── mod.rs                        # attestation submodule
│   │   ├── chain_validator.rs            # JWS multi-sig validator
│   │   ├── cache.rs                      # cell-local cache
│   │   └── passkey_link.rs               # ADR-0188 passkey integration
│   ├── scope/
│   │   ├── mod.rs                        # scope submodule
│   │   ├── inheritance.rs                # effective-scope evaluator
│   │   └── escalation_detector.rs        # no-escalation enforcer
│   ├── tenant_block/
│   │   ├── mod.rs                        # cross-tenant submodule
│   │   ├── enforcer.rs                   # boundary enforcer
│   │   └── tenant_extractor.rs           # request-tenant extractor
│   ├── audit_linkage/
│   │   ├── mod.rs                        # audit-linkage submodule
│   │   ├── emitter.rs                    # per-action emitter
│   │   ├── query_surface.rs              # by-human / by-event / by-delegate queries
│   │   └── merkle_anchor.rs              # ADR-0028 anchor integration
│   ├── bot_defence/
│   │   ├── mod.rs                        # bot-defence integration submodule
│   │   ├── allow_path.rs                 # per-class allow-path
│   │   └── rate_limit.rs                 # per-class rate-limit
│   ├── agent_class/
│   │   ├── mod.rs                        # agent-class registry
│   │   └── registry.rs                   # YAML loader
│   ├── cedar_fragment/
│   │   ├── mod.rs                        # Cedar fragment helpers
│   │   ├── context_builder.rs
│   │   └── evaluator.rs
│   ├── audit/
│   │   ├── mod.rs                        # audit event emission
│   │   ├── event_class.rs
│   │   └── emit.rs
│   ├── observability/
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   └── tracing.rs
│   ├── tenancy/
│   │   ├── mod.rs                        # tenancy substrate integration
│   │   └── principal_type.rs             # PrincipalType enum extension
│   └── error.rs
├── tests/
│   ├── llm_agent_anthropic.rs
│   ├── llm_agent_openai.rs
│   ├── webhook_ifttt.rs
│   ├── workflow_step_n8n.rs
│   ├── workflow_step_zapier.rs
│   ├── workflow_step_workflow_studio.rs
│   ├── attestation_chain_property.rs
│   ├── scope_inheritance_property.rs
│   ├── cross_tenant_block_property.rs
│   ├── audit_linkage_property.rs
│   ├── revocation_propagation.rs
│   └── fixtures/
│       ├── agent_class_fixtures.rs
│       ├── attestation_fixtures.rs
│       └── delegation_fixtures.rs
└── docs/
    ├── README.md
    ├── ARCHITECTURE.md
    ├── usage.md
    ├── agent-class-onboarding.md
    └── attestation-chain-format.md
```

### §E.2. New µservice extensions

Every µservice that accepts delegated-agent requests extends with:

```
microservices/<name>/
├── policy/
│   ├── delegated-agent.cedar
│   └── delegated-agent-overlays/
│       ├── pack-us-hipaa.cedar
│       ├── pack-eu-gdpr.cedar
│       ├── pack-us-ccpa.cedar
│       └── pack-eu-aiact.cedar
├── iac/
│   ├── dev-delegated-agent.yaml
│   ├── staging-delegated-agent.yaml
│   └── prod-delegated-agent.yaml
├── docs/
│   ├── ARCHITECTURE.md                  # +§delegated-agent
│   ├── PRD.md                           # +§delegated-agent-edge-cases
│   ├── compliance.md                    # +§delegated-agent per §3.2.5 row 28
│   └── runbooks/
│       ├── delegated-agent-token-revocation-lag.md
│       ├── delegated-agent-attestation-chain-failure.md
│       └── delegated-agent-cross-tenant-block-investigation.md
├── tests/
│   └── delegated_agent_contract.rs
├── dashboards/
│   └── delegated-agent.json
└── slos/
    ├── attestation-validate-latency.openslo.yaml
    └── revocation-propagation-latency.openslo.yaml
```

### §E.3. New runbooks

- `delegated-agent-token-revocation-lag.md`
- `delegated-agent-attestation-chain-failure.md`
- `delegated-agent-cross-tenant-block-investigation.md`
- `delegated-agent-scope-escalation-investigation.md`
- `delegated-agent-rate-limit-exhaustion.md`

### §E.4. New CI lanes

- `oya-governance-delegated-agent-token-coverage`
- `oya-governance-agent-attestation-chain`
- `oya-governance-cross-tenant-delegation-block`
- `oya-governance-agent-scope-inheritance`
- `oya-governance-agent-audit-linkage`
- `oya-governance-delegated-agent-authority` (aggregate)

### §E.5. Vendor selection rationale

- **JWS implementation** — `josekit` Rust crate (Ed25519 +
  EdDSA) + JWS specification RFC 7515.
- **OAuth flows** — RFC 6749 + RFC 8628 device-flow for CLI tools.
- **Passkey integration** — `webauthn-rs` per ADR-0188.
- **Audit-chain anchor** — Merkle per ADR-0028.
- **Revocation broadcast** — substrate's kill-switch bus per
  ADR-0295.

## §F. Migration

### §F.1. Per-µservice rollout sequenced by delegation-exposure

| Wave | Cohort | µservices | Window |
|---:|---|---|---|
| 1 | High-delegation-exposure | api-gateway, identity, tenancy, intelligence | 2026-05-30 → 2026-07-15 |
| 2 | Workflow-engine | workflow-studio, foundry, governance | 2026-07-15 → 2026-08-31 |
| 3 | Data-plane | notes, mail, social, ontology, marketplace | 2026-08-31 → 2026-09-30 |
| 4 | Cleanup + audit | all remaining | 2026-09-30 onward |

### §F.2. Per-µservice migration playbook

1. Add `oya-shared-agent-authority` workspace dependency.
2. Author `policy/delegated-agent.cedar`.
3. Author `iac/<env>-delegated-agent.yaml`.
4. Add `§delegated-agent` to `ARCHITECTURE.md`.
5. Add `§delegated-agent-edge-cases` to `PRD.md` + `compliance.md`.
6. Add `dashboards/delegated-agent.json` + SLOs.
7. Add contract test.
8. Pass `oya-governance-delegated-agent-authority`.
9. Soak ≥ 60s; promote.

### §F.3. Per-vendor agent-class onboarding

- **Anthropic Claude** — onboarded 2026-06-01 (Wave 1).
- **OpenAI Assistants** — onboarded 2026-06-15 (Wave 1).
- **Google Vertex AI agents** — onboarded 2026-07-01 (Wave 2).
- **IFTTT** — onboarded 2026-07-01 (Wave 2).
- **Zapier** — onboarded 2026-07-15 (Wave 2).
- **n8n** — onboarded 2026-07-15 (Wave 2).
- **Workflow Studio** — onboarded 2026-08-01 (Wave 2).
- **Make.com + Pipedream + Tray.io** — onboarded 2026-08-15 (Wave 3).
- **GitHub Apps + GitHub Actions** — onboarded 2026-09-01 (Wave 3).

### §F.4. What is NOT migrated

- SPIFFE workload identity per ADR-0295 (distinct).
- Per-tenant agent-management UI is the tenancy substrate per
  ADR-0244.
- Per-vendor SDK shape is the vendor's responsibility.

### §F.5. Rollback path

- Cell-tier rollback: `oya policy revert delegated-agent-v1`.
- µservice rollback: revert `policy/delegated-agent.cedar`.
- Soft-disable: `delegated_agent_enabled = false` in IaC; existing
  tokens accepted until expiry; no new issuances.
- Hard-disable: drop the workspace dependency.

## §G. References

### §G.1. Hyperscaler precedents

- Microsoft Graph delegated-permissions documentation 2024.
- Microsoft Copilot for Microsoft 365 documentation 2024.
- Anthropic Claude API + Computer-Use + MCP documentation 2024-2025.
- OpenAI Assistants API + GPT Actions documentation 2024.
- Google Workspace add-ons + Vertex AI Agents documentation 2024.
- Salesforce Einstein + Slack apps + Atlassian Forge documentation
  2024.
- Zapier + IFTTT + n8n + Make.com documentation 2024.
- GitHub Apps + Actions + Dependabot documentation 2024.

### §G.2. Standards + RFCs

- RFC 7515 — JSON Web Signature (JWS).
- RFC 7519 — JSON Web Token (JWT).
- RFC 7517 — JSON Web Key (JWK).
- RFC 8037 — CFRG ECDH and ECDSA in JOSE (EdDSA / Ed25519).
- RFC 6749 — OAuth 2.0.
- RFC 8628 — OAuth 2.0 Device Authorization Grant.
- RFC 9068 — JWT Profile for OAuth 2.0 Access Tokens.
- RFC 7009 — OAuth 2.0 Token Revocation.
- Model Context Protocol (Anthropic) specification 2024.
- OpenID 1.0.
- W3C WebAuthn Level 3 (per ADR-0188).
- SPIFFE SPEC v1.6.0 (distinct from delegated-agent).

### §G.3. Legal + compliance

- GDPR Article 28 — Data Processor obligations.
- HIPAA — Business Associate Agreement requirements.
- EU AI Act Articles 14, 50 — human oversight + AI transparency.
- COPPA — Children's Online Privacy Protection Act.
- KOSA — Kids Online Safety Act.
- EU AADC — Age Appropriate Design Code.
- CCPA + CPRA — California Consumer Privacy Act.
- SOC 2 CC6.6 — Delegated access monitoring.

### §G.4. Internal portfolio ADRs

- ADR-0028 Audit Chain (Merkle-sealed).
- ADR-0044 Service Mesh + mTLS.
- ADR-0099 Data Class Registry.
- ADR-0105 Thirteen-Layer Canonical Enum.
- ADR-0131 Per-µservice Flat Layout.
- ADR-0140 Cedar Policy Enforcement.
- ADR-0145 Inter-Microservice Communication Reform.
- ADR-0188 Passkey/WebAuthn Canonical Auth.
- ADR-0212 Buildability Doctrine.
- ADR-0242 Oyatie is a Tenant Doctrine.
- ADR-0243 Cedar as Universal Gate.
- ADR-0244 Tenant as Universal Scoping Primitive.
- ADR-0245 Substrate vs Product Layering.
- ADR-0246 Policy Engine Substrate Promotion.
- ADR-0247 Self-Modification Doctrine.
- ADR-0248 Amazon-Shape Cellular Architecture.
- ADR-0251 Compliance Pack — Cell Certification Levels.
- ADR-0253 Network Topology — Edge + Service Mesh.
- ADR-0255 Intelligence Two-Layer Substrate.
- ADR-0258 API Versioning + SemVer Policy.
- ADR-0263 Observability Emission Contract.
- ADR-0272 Cookie Consent per Purpose.
- ADR-0292 Minor User Doctrine.
- ADR-0294 Cedar Fragment Lifecycle.
- ADR-0295 Bootstrap CI SPIFFE + Kill-Switch.
- ADR-0296 Library-First Credential Sidecar.
- ADR-0297 Abuse-Defence Baseline.
- ADR-0298 Emergency-Services Bypass Doctrine.
- ADR-0299 Account-Recovery Resilience.
- ADR-0300 Whistleblower + Press-Freedom Anonymity.
- ADR-0301 Survivor-Safety Domestic-Abuse Mode.
- ADR-0302 Deceased-User Inheritance Doctrine.
- ADR-0303 Cognitive-Impairment Decision-Resilience.
- ADR-0304 Cross-Jurisdiction Conflict Resolution.
- ADR-0306 Disaster-Mode + Cell Resilience.

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.5 row 28.
- `docs/standards/doc-style.md`.
- `docs/templates/adr-template-v2.md`.

### §G.6. Auto-memory feedback (related)

- feedback_quality_performance_scalability_bar
- feedback_clean_architecture_requirements
- feedback_no_silent_regression
- feedback_autonomous_implementation_artifacts
- feedback_canonical_base_localization
- feedback_oyatie_is_a_tenant_doctrine
- feedback_cedar_as_universal_gate
- feedback_amazon_shape_cellular_architecture
- feedback_compliance_pack_primitive
- feedback_naming_justification
- feedback_intelligence_two_layer_substrate
- feedback_self_modification_doctrine

## §H. Change log

- **2026-05-20** — Initial proposal. Bundled with keystone-bundle
  2026-05-20 foundational doctrine synthesis as the critical-path-
  cluster-delegated-agent-authority-chain keystone. Closes
  documentation-rigor.md §3.2.5 row 28. Enforcement advisory until
  2026-09-30, BLOCKER thereafter.

---

End of ADR-0305.
