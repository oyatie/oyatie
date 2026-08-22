---
doc_class: IP
template_id: TPL-IP
ip_id: IP-001-identity-design
microservice: identity
bounded_context: workload-identity
status: proposed
related_adrs: [ADR-0002, ADR-0131, ADR-0183]
related_crates:
  - identity-workload-domain
  - identity-workload-oidc-adapter
  - identity-workload-authz-cedar-adapter
date: 2026-05-26
owner_team: axis-identity
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# IP-001 — Workload-Identity Design Implementation Plan

> Implements the design in `microservices/identity/workload-identity/PRD.md`.
> Distinct from `IP-001-zitadel-helm-per-pack.md` (human-identity Zitadel
> deployment); this IP lands the machine-to-machine workload-identity slice
> (ADR-0002 task T3). Grounded in `design/hyperscaler-best-practice-brief.md`.

## Goal

Land three crates that together provide workload-token validation and Cedar PARC
authorization for the fleet, behind stable swap-in traits, fail-closed, with an
immutable decision log:

1. `identity-workload-domain` — pure kernel.
2. `identity-workload-oidc-adapter` — `ring` OIDC/JWS validation (8-step pipeline).
3. `identity-workload-authz-cedar-adapter` — Cedar PARC PDP behind a trait.

## Sequenced slices

### Slice 1 — `identity-workload-domain` (pure kernel, zero deps)

| File | Purpose |
|---|---|
| `crates/identity-workload-domain/Cargo.toml` | no deps beyond std (matches catalog: pure kernel) |
| `crates/identity-workload-domain/src/lib.rs` | `WorkloadPrincipal`, lifecycle state machine, PARC decision types |
| `crates/identity-workload-domain/tests/workload_domain.rs` | lifecycle + decision-type tests |

Public surface (excerpt):

```rust
pub struct WorkloadPrincipal {
    pub principal_id: PrincipalId,   // SPIFFE-shaped, immutable, never reused
    pub trust_domain: TrustDomain,   // == tenant (brief §6)
    pub tenant_id: TenantId,
    pub state: LifecycleState,
    pub capabilities: Vec<Capability>,
    pub scopes: Vec<Scope>,
    pub claims: BTreeMap<String, ClaimValue>, // operational claims only
}

pub enum LifecycleState { Provision, Active, Suspended, Retired }

impl WorkloadPrincipal {
    /// Allowed: Provision->Active, Active<->Suspended, {Active,Suspended}->Retired.
    /// Retired is terminal. Returns Err(InvalidTransition) otherwise.
    pub fn transition(&mut self, to: LifecycleState) -> Result<(), LifecycleError>;
}

pub struct ParcRequest { pub principal: EntityRef, pub action: ActionRef, pub resource: EntityRef, pub context: Context }
pub enum Decision { Allow, Deny }
pub struct AuthzDecision { pub decision: Decision, pub determining_policies: Vec<PolicyId>, pub errors: Vec<DecisionError> }
```

Tests (≥8): provision→active; active→suspended→active; active→retired terminal;
suspended→retired; retired→active rejected; retired id non-reuse;
`AuthzDecision` implicit-deny shape (empty determining_policies + Deny); decision
error typing.

### Slice 2 — `identity-workload-oidc-adapter` (`ring` OIDC)

| File | Purpose |
|---|---|
| `crates/identity-workload-oidc-adapter/Cargo.toml` | deps: `ring`, `base64`, `serde`, `serde_json` (matches catalog) |
| `crates/identity-workload-oidc-adapter/src/lib.rs` | 8-step pipeline + RSA JWK (n/e) → PKCS#1 DER bridge for ring |
| `crates/identity-workload-oidc-adapter/tests/oidc_validation.rs` | the RFC 8725/9068 test matrix |

The 8-step pipeline (PRD §3.1) is implemented as an ordered, fail-fast function
returning `Result<Rfc9068Claims, DecisionError>`. The `kid`→alg binding table is
the sole source of the verification algorithm; the token header `alg` is parsed
only to be checked against the binding (mismatch → `AlgorithmMismatch`), never to
select the algorithm.

Tests (≥12, one per threat in `threat-model.md#workload-identity-threat-model`): well-formed ES256 ok;
well-formed RS256 ok; `alg:none` rejected; RS256→HS256 confusion rejected;
unknown `kid`; wrong issuer; key-not-belongs-to-issuer; wrong audience; expired
(outside skew); nbf in future; skew boundary (exactly 60s ok, 61s not);
`jku` not allowlisted; malformed compact JWS; missing `typ`.

### Slice 3 — `identity-workload-authz-cedar-adapter` (Cedar PARC)

| File | Purpose |
|---|---|
| `crates/identity-workload-authz-cedar-adapter/Cargo.toml` | in-crate evaluator; documents upstream `cedar-policy` swap (manifest `consumes_upstream_oss`) |
| `crates/identity-workload-authz-cedar-adapter/src/lib.rs` | `WorkloadAuthorizer` trait + faithful Cedar-semantics evaluator |
| `crates/identity-workload-authz-cedar-adapter/tests/cedar_authz.rs` | PARC + forbid-wins + lifecycle-precondition tests |

```rust
pub trait WorkloadAuthorizer: Send + Sync {
    fn authorize(&self, req: &ParcRequest, principal: &WorkloadPrincipal) -> AuthzDecision;
}
```

The evaluator enforces: default-deny; forbid-overrides-permit; the lifecycle
precondition (non-`Active` → deny via `forbid-suspended-principal`); tenant ==
trust-domain. It loads `microservices/identity/policy/identity.cedar`. The
upstream `cedar-policy` crate is the documented drop-in swap behind the same trait.

Tests (≥10): explicit permit; explicit forbid wins over permit; implicit deny
(empty determining_policies); suspended principal denied; retired denied;
cross-trust-domain denied; capability-scoped permit; scope-narrowed read;
sensitive-write-without-mfa denied; sensitive-write-with-mfa permitted.

## Contract + spec artifacts (this design package)

Already authored alongside this IP:

- `contracts/identity.openapi.yaml` (OpenAPI 3.2.0)
- `contracts/identity.asyncapi.yaml` (AsyncAPI 3.1.0)
- `contracts/identity.proto` (proto3)
- `policy/identity.cedar`
- `capabilities/identity.capabilities.yaml`
- `slos/{authorize-latency-p99,validation-availability,decision-correctness}.openslo.yaml`
- `runbooks/{jwks-fetch-failure,policy-store-unavailable}.md`
- `design/{threat-model,failure-modes,data-residency,cost-finops,audit-evidence-emission,tenant-isolation,operational-boundaries}.md`

## Acceptance gates

Maps to PRD §6 (AC-W-01 … AC-W-18). The slice is complete when:

- All crate tests above pass under `cargo nextest`.
- The architecture-boundaries gate confirms the domain kernel imports no peers
  (AC-W-16) and the trait swap is clean (AC-W-15).
- The honest-claims + design-spec-maturity gates accept this package (every doc
  is substantive and brief-grounded).
- The golden decision corpus exists and is wired to the correctness SLO prober.

## Out of scope for this IP

- Token issuance / minting; JWKS serving (OIDC-issuer context).
- Human auth (passkeys, SCIM, step-up).
- A policy-authoring UI.
- PDP-side decision caching (PEP-side only, per PRD §7).

## Test plan reference

Unit + contract + integration strategy mirrors the existing identity test-plans
(`microservices/identity/test-plans/`); the threat matrix in
`threat-model.md#workload-identity-threat-model` is the source of the verifier test cases.
