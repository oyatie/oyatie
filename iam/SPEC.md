---
doc_class: Owner-SPEC
owner: iam
status: Active
date: 2026-08-27
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - iam/ADR.md
  - iam/PRD.md
---

# IAM behavior and contract

<classification>

## Current artifact classification

| Class | Behavior or artifact | Disposition |
|---|---|---|
| Retain and mature | Identity domain/use cases, identity API semantics, OIDC/JWKS verification, SCIM semantics/store, workload-principal consumption, device-attestation boundary | Preserve behavior in canonical IAM core/ports/adapters; repair structure only in separately authorized lanes |
| Transfer by owner | IAM-resident Cedar/PDP evaluation | Policy owner extracts it; tenant-rbac deletion does not rehome it |
| Transfer by owner | IAM-resident SVID/certificate/key issuance | Secrets owner extracts it; tenant-rbac deletion does not rehome it |
| Remove after founder gate | All 39 `tenant-rbac-*` crate directories and four named tenant-rbac OpenSLO files | One atomic structural deletion; no partial state or compatibility replacement |
| Build later | Real IAM identity Connect facade | New generated-protobuf feature with a listener and process main |
| Build later | Caller-sensitive Kubernetes VAP/CEL/RBAC admission | New k8s-owner feature; no remote IAM or Policy lookup in admission |
| Repair separately | Root `iam/BUCK` census loader | Systemic D-17 graph cleanup outside the tenant-rbac deletion set |

</classification>

<principal_contract>

## Principal projection

A verified principal is a transport-neutral value with:

- opaque principal identifier and principal kind;
- canonical tenant identifier;
- lifecycle state;
- credential/federation source and verification-material version;
- issued/not-before/expiry bounds where applicable;
- authenticated assurance and device-posture facts;
- scopes or role references as identity facts, never as an allow decision;
- correlation and evidence identifiers without raw credentials.

Projection is total over its supported inputs: every invalid condition maps to a
stable typed refusal. Validation precedes domain use-case execution. Tenant,
issuer, audience, signature, key/algorithm binding, time, token-type, and
required-claim checks cannot be reordered after mutation or authorization.

</principal_contract>

<role_contract>

## Role storage and Cedar compilation

Role state is tenant-scoped, versioned, idempotent, and durably ordered. The
compiler consumes one canonical snapshot and produces:

- the tenant and input version;
- deterministic bounded Cedar entities/policy input;
- compiler/schema version;
- a content digest over canonical output;
- issuance/freshness metadata for Policy consumption.

Identical canonical input produces identical output and digest. The compiler
does not evaluate an action/resource request, traverse relationships, or emit
allow/deny. Policy validates freshness and owns every `Check` result.

</role_contract>

<scim_contract>

## SCIM and identity mutation

SCIM operations bind authenticated tenant, path tenant, and body tenant before
store access. Create/update/delete plus idempotency outcome and audit/outbox
intent commit atomically before acknowledgement. A retry with the same key and
canonical request returns the recorded result; reuse with different canonical
input is a typed conflict. SCIM may reference only a tenant that Tenancy has
already created.

</scim_contract>

<dependency_contract>

## Allowed dependency direction

```text
facade -> core + ports + adapters
adapters -> ports + external protocol/storage libraries
core -> core-local domain/use-case modules
policy PEP consumer -> IAM principal + Policy Check
app adapter -> sold IAM Connect facade
```

Forbidden edges include IAM to `app/*`, IAM core to transport/storage, apps to
IAM core/ports, IAM to Policy engine internals, IAM to Secrets issuance
internals, and IAM to Kubernetes admission implementation. An IAM-generated
principal is input to Policy; it is not a Policy decision.

</dependency_contract>

<retirement_state_machine>

## Tenant-rbac retirement gate

```text
blocked_on_false_live_instance
  -> founder_amends_adr_0710
  -> complete_inventory_reverified
  -> atomic_structural_deletion
  -> cargo_and_buck_survivors_verified
```

There is no `partially_removed`, `compatibility_rest`, `renamed_wrapper`,
`manifest_split`, or `fake_main` state. Before founder acceptance, the only
valid state is the complete current inventory. After acceptance, the only valid
terminal state is absence of every path enumerated in `iam/PLAN.md`.

The ADR-0710 amendment corrects only the false concrete assertion that the IAM
contract crate is a live admission instance. It preserves:

- API-server in-process VAP/CEL validation;
- RBAC checks using request-carried caller identity where needed;
- PSA as the pod-security baseline;
- `failurePolicy: Fail` and `parameterNotFoundAction: Deny` where applicable;
- no network lookup to IAM or the Cedar/ReBAC PDP on the admission path.

No ADR-0719 amendment is proposed or needed.

</retirement_state_machine>

<future_connect_contract>

## Future identity facade admission criteria

The future facade is not dispatchable until owner law names its protobuf
package, RPC methods, generated Rust/Connect targets, runtime, bounded-message
limits, and Cargo/Buck output contract. It then lands as a new feature with:

- one protobuf contract and generated Connect client/server symbols;
- a real facade process and listener;
- H3 at the public door and the same Connect framing over H2 fallback;
- authentication before decoding sensitive mutation bodies where the runtime
  supports header-first rejection;
- tenant binding before store mutation;
- Policy `Check` through the accepted in-process boundary;
- bounded bodies, deadlines, cancellation, backpressure, structured errors,
  correlation, metrics, traces, and audit intent;
- no REST/JSON compatibility service, gRPC envelope, transcode, or dependency
  on a tenant-rbac artifact.

</future_connect_contract>

<verification_contract>

## Evidence contract

Behavior evidence is target-scoped during leaf work and protected at workspace
integration. The deletion lane records exact command, exit code, and tip SHA.

Target-scoped Buck evidence after the separate `iam/BUCK` cleanup:

```text
buck2 targets //iam/...
buck2 build //iam/...
buck2 test //iam/...
```

Protected serialized Cargo evidence after the mechanical lockfile update:

```text
cargo metadata --locked --offline --format-version 1
cargo fmt --all --check
cargo nextest run --locked --workspace --profile ci
cargo clippy --workspace --all-targets -- -D warnings
```

Required negative evidence:

```text
git ls-files | rg '^iam/(core|ports|adapters|facade)/tenant-rbac-'
git ls-files iam/observability/slos/tenant-rbac
rg -n 'tenant-rbac|tenant_rbac' iam Cargo.toml Cargo.lock
```

The first two commands return no paths after deletion. The final search is
reviewed: package/lock references are absent, while historical citations in
owner law may remain. Target discovery alone is not build evidence. A failed
Buck graph, changed lockfile beyond removed packages, missing test, or skipped
failure campaign is a refusal, not partial success.

</verification_contract>

<promotion_failures>

## Executable safety cases

Promotion binds the PRD objectives to these tests:

- token validation rejects algorithm substitution, bad signature, key-source
  injection, unknown/expired key, wrong issuer/audience/type, and invalid time;
- tenant mismatch fails before read/write and cannot emit a success response;
- storage crash/reopen at every acknowledgement boundary preserves atomicity
  and idempotent replay;
- role compilation shuffle/retry produces deterministic output, while stale or
  absent output cannot authorize;
- attestation expiry/downgrade removes assurance rather than preserving it;
- Policy outage cannot be converted into an IAM allow;
- removal-set fault injection omits one crate or one SLO and proves the
  inventory gate refuses dispatch;
- forbidden-edge fixtures prove IAM-to-app and app-to-IAM-core dependencies are
  rejected.

</promotion_failures>
