---
doc_kind: implementation-plan
id: IP-017
title: Multi-context principal resolver
status: Scaffolded
owner_team: axis-identity
related_adrs: [ADR-0215, ADR-0242, ADR-0243, ADR-0244, ADR-0311]
related_capabilities:
  - microservices/identity/capabilities/multi-context-principal-resolve.yaml
related_contracts:
  - microservices/identity/contracts/openapi/multi-context-split.yaml
  - microservices/identity/contracts/asyncapi/multi-context-events.yaml
  - microservices/identity/contracts/proto/multi_context_split.proto
related_policy:
  - microservices/identity/policy/context-split.cedar
date: 2026-05-18
substance_scrubbed: 2026-05-21
---

# IP-017: Multi-context principal resolver

## A. Problem

The old 28-line version of this IP named the ADR-0215 surface but did not say
how identity resolves one human principal across personal, work, healthcare,
marketplace, and community contexts. That gap was not cosmetic: every
downstream µservice depends on identity to return a context envelope that Cedar
can evaluate before storage adapters see a request.

The concrete risk is cross-context authority bleed. A tenant admin must be able
to resolve a work context for an employee, but must never enumerate or switch
into that employee's personal tenant. A healthcare break-glass flow needs a
healthcare context with sovereignty metadata, while a marketplace seller flow
needs organization binding and counterparty-safe disclosure. Treating all of
those as one `principal_id` string would violate ADR-0242, ADR-0243, ADR-0244,
and ADR-0311.

This IP closes the resolver gap by turning
`capabilities/multi-context-principal-resolve.yaml` from a scaffolded capability
into an implementable substrate slice with a real OpenAPI operation, internal
proto RPC, sealed audit events, and a Cedar deny rule for personal-context
visibility.

## B. Approach

Implement a resolver pipeline around the existing contract surfaces:

1. `POST /principal-context/resolve` in
   `microservices/identity/contracts/openapi/multi-context-split.yaml` is the
   REST contract. It accepts `principal_id`, optional `requested_context_id`,
   and `purpose`.
2. `PrincipalContextResolver.ResolvePrincipalContext` in
   `microservices/identity/contracts/proto/multi_context_split.proto` is the
   internal RPC shape for other substrate services that should not call REST.
3. `IdentityContextResolved` and `IdentityContextSwitchRefused` in
   `microservices/identity/contracts/asyncapi/multi-context-events.yaml` are
   the only events this IP may emit.
4. `microservices/identity/policy/context-split.cedar` remains deny-wins:
   tenant admins are forbidden from personal contexts and unlisted context
   switches are refused.
5. The capability record
   `microservices/identity/capabilities/multi-context-principal-resolve.yaml`
   stays `maturity: scaffolded` until the implementation PR supplies tests and
   evidence; this IP defines the promotion gate, not a false GA claim.

The resolver should be deterministic: given the same principal state snapshot,
requested context, purpose, Cedar bundle, and request time, it returns the same
envelope or refusal reason. Hidden wall-clock reads are not allowed in the
kernel path; request time is passed through the usecase context.

## C. Deliverables

| Artifact | Required change |
|---|---|
| `crates/oya-identity-multi-context-principal-resolver-kernel/src/lib.rs` | Define `PrincipalContext`, `ContextType`, `ResolvePrincipalContextRequest`, `PrincipalContextEnvelope`, `ContextSwitchRefusal`, and `ContextStore` trait. |
| `crates/oya-identity-multi-context-principal-resolver-usecase/src/lib.rs` | Implement resolve order: load principal contexts, choose requested or default context, evaluate Cedar, emit envelope/refusal. |
| `crates/oya-identity-multi-context-principal-resolver-rest/src/lib.rs` | Bind OpenAPI operation `resolvePrincipalContext` to the usecase and map refusal to HTTP 403. |
| `crates/oya-identity-multi-context-principal-resolver-adapter/src/lib.rs` | Provide the storage/audit adapter boundary without defining domain terms in the adapter. |
| `microservices/identity/contracts/openapi/multi-context-split.yaml` | Keep required response fields aligned with the kernel envelope: `principal_id`, `active_context_id`, `context_type`, `sovereignty_region`, and `allowed_context_switches`. |
| `microservices/identity/contracts/asyncapi/multi-context-events.yaml` | Emit only `IdentityContextResolved` and `IdentityContextSwitchRefused`; both require `audit_chain_seal`. |
| `microservices/identity/contracts/proto/multi_context_split.proto` | Keep proto enum values aligned with REST enum values: personal, work, healthcare, marketplace, community. |
| `microservices/identity/policy/context-split.cedar` | Preserve tenant-admin personal-context deny and switch-allowlist deny. |
| `microservices/identity/capabilities/multi-context-principal-resolve.yaml` | Promote from `scaffolded` only after the tests and evidence below exist. |

## D. Implementation Steps

1. Add the kernel crate listed in `manifest.json` under
   `multi-context-principal-resolver`: define the closed `ContextType` enum that
   mirrors `multi_context_split.proto` and the OpenAPI enum.
2. Implement `ContextStore::list_contexts(principal_id)` and
   `ContextStore::load_context(context_id)` as traits. The kernel must not know
   the backing database, Zitadel organization model, or audit-chain transport.
3. Implement the usecase selection rule: if `requested_context_id` is present,
   attempt that context; otherwise choose the principal's default context for
   the supplied `purpose`. Ambiguous defaults return a typed refusal, not a
   guessed context.
4. Evaluate `Action::"ResolvePrincipalContext"` from
   `policy/context-split.cedar` before returning the envelope. Evaluate
   `Action::"SwitchPrincipalContext"` when the requested context differs from
   the stored active context.
5. Build the REST adapter for `resolvePrincipalContext`; response `200` returns
   `PrincipalContextEnvelope`, response `403` returns a stable refusal code and
   emits `IdentityContextSwitchRefused`.
6. Build the proto adapter for internal callers. It must not introduce a second
   field vocabulary; it maps directly to the same kernel request/envelope types.
7. Emit `IdentityContextResolved` with `principal_id`, `active_context_id`,
   `context_type`, and `audit_chain_seal` after Cedar permits. The event must
   be sealed before the REST/RPC response is considered committed.
8. Add fixtures for at least these cases: personal self-resolution, work
   tenant-admin resolution, tenant-admin denied personal context, healthcare
   break-glass context, marketplace seller context, expired context grant,
   unknown requested context, ambiguous default context, duplicate replay, and
   cross-region residency mismatch.
9. Update the capability record only when the implementation evidence exists:
   `maturity` may move from `scaffolded` to `preview` after contract tests and
   Cedar fixtures pass; GA requires production soak.

## E. Acceptance

- `microservices/identity/contracts/openapi/multi-context-split.yaml` validates
  and still exposes exactly one public operation: `resolvePrincipalContext`.
- `microservices/identity/contracts/proto/multi_context_split.proto` compiles
  and the generated enum maps one-for-one to the OpenAPI `context_type` enum.
- Cedar tests prove `tenant-admin` cannot resolve `context_type == "personal"`
  and that a principal cannot switch to a context not in `allowed_contexts`.
- REST contract tests cover HTTP 200 and HTTP 403 with stable refusal reasons.
- AsyncAPI tests or schema validation cover `IdentityContextResolved` and
  `IdentityContextSwitchRefused`, including required `audit_chain_seal`.
- Capability promotion is blocked until
  `microservices/identity/capabilities/multi-context-principal-resolve.yaml`
  can point at an eval set under
  `microservices/identity/capabilities/eval/` that exists in the repo.
- No downstream service may parse a context from a raw token string; downstream
  callers consume the envelope fields directly.

## F. Evidence

- `microservices/identity/PRD.md` classifies identity as the universal authn /
  authz substrate and requires `tenant_id`, `acr`, `purpose`, `data_class`,
  `age_class`, and `jurisdiction_code` claims for downstream Cedar evaluation.
- `microservices/identity/manifest.json` lists the
  `multi-context-principal-resolver` bounded context and its six planned crates.
- `microservices/identity/capabilities/multi-context-principal-resolve.yaml`
  currently marks the capability `maturity: scaffolded`, which is the correct
  state until this IP lands code and tests.
- `microservices/identity/contracts/openapi/multi-context-split.yaml`,
  `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, and
  `microservices/identity/contracts/proto/multi_context_split.proto` already
  define the public, event, and internal RPC shapes.
- `microservices/identity/policy/context-split.cedar` already contains the
  tenant-admin personal-context forbid and switch allowlist forbid needed for
  the first enforcement slice.
- `microservices/identity/REMEDIATION-NOTES-2026-05-21.md` records the prior
  Wave 15A finding that the 28-line IP did not justify GA maturity.

## G. Counterparts

| Counterpart | Relevant behavior | Oyatie delta this IP closes |
|---|---|---|
| Okta Workforce Identity | Strong org/user lifecycle, but personal/work context separation is tenant-centric rather than first-class across product surfaces. | Identity returns an explicit active-context envelope so personal and work contexts remain separately governable. |
| Auth0 Organizations | Organization membership scopes app access, but downstream apps often build their own context-switch semantics. | The resolver centralizes context switching and refusal evidence in the identity substrate. |
| Microsoft Entra ID | Tenant and guest-user boundaries are mature, but consumer personal context is outside the workforce tenant model. | Oyatie models personal, work, healthcare, marketplace, and community contexts in one closed enum with Cedar gates. |
| Keycloak / Zitadel | Realms/orgs can model tenancy, but application-specific context envelopes are left to integrators. | The IP makes the envelope a first-class contract across OpenAPI, proto, AsyncAPI, and Cedar. |
| Palantir Foundry | Strong ontology and access-control separation across organizations. | Oyatie adapts that explicit-context discipline to identity principal resolution before any µservice storage read. |

## H. Non-goals

- This IP does not implement WebAuthn, OIDC token signing, SCIM provisioning, or
  external IdP federation; those are IP-002 through IP-011.
- This IP does not create a new Cedar entity vocabulary beyond what
  `policy/context-split.cedar` already uses.
- This IP does not promote the capability to GA. It defines the work required
  for a future implementation PR to justify promotion.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
