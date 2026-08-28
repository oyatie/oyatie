---
doc_class: Owner-ADR
owner: iam
status: Accepted
date: 2026-08-27
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
related:
  - docs/decisions/ADR-0710-kubernetes-admission-substrate-is-the-api-server.md
---

# IAM decisions in force

This file specializes ADR-0719 for `iam/`. It records the destination owner
boundary and the gates on removing incompatible inventory. It does not claim
that every destination feature has landed, and it proposes no amendment to
ADR-0719.

<current_state>

## Current evidence

At base `8489b29bce609b8ee3a3e5874f1d3013672d20c9`, canonical IAM behavior
already exists in the identity domain, identity use case, identity API,
OIDC/JWKS verifier, SCIM kernel/store, workload-identity domain, and device
attestation surfaces. Those implementations include typed human and workload
principals, issuer/audience/key validation, tenant binding, identity lifecycle,
and SCIM persistence behavior.

The same tree also contains 39 `tenant-rbac-*` crates and four hand-authored
tenant-rbac OpenSLO files. That cone exposes six REST/JSON route descriptions
but no listener, composes HR, Payroll, and Accounting packages in process, and
turns route counts, manifests, and review assertions into readiness evidence.
It is migration inventory, not a live IAM product or precedent.

</current_state>

<identity_authority>

## Decision: IAM proves who

- **achieves:** one portable authority for principal identity and posture
  without absorbing authorization decisions or tenant applications.
- **origin:** identity behavior and the tenant-rbac vertical product were
  colocated, obscuring the boundary between proving a principal and deciding
  whether that principal may act.
- **rule:** IAM MUST own human and workload principals, passkeys, SCIM into an
  existing tenant, federation consumption, workload-identity consumption,
  device-attestation context, and a role store that deterministically compiles
  role state into Cedar input. A Cognito-class user-token issuer may exist only
  as a separately sold IAM facade, never as an implicit kernel responsibility.
- **ensure:** core remains transport- and I/O-free; ports describe identity,
  federation, role-store, compilation, and evidence boundaries; adapters own
  protocols and persistence; contract tests prove tenant binding, token
  validation, idempotency, and deterministic compilation.
- **overturn_when:** a founder-accepted five-field owner decision reallocates a
  named identity responsibility and amends every affected owner in the same
  change.

</identity_authority>

<decision_separation>

## Decision: Policy decides may; Secrets issues trust material

- **achieves:** identity, authorization, and key authority retain independent
  failure domains and cannot silently authorize one another.
- **origin:** IAM contains Cedar/PDP and SVID-issuance fossils even though
  ADR-0719 assigns those engines to Policy and Secrets.
- **rule:** IAM MUST emit an authenticated, tenant-bound principal and role
  compilation output but MUST NOT evaluate Cedar or traverse ReBAC. Policy owns
  the PDP, snapshots, tuples, and in-process `Check`. Secrets owns SVID,
  certificate, signing-key, and secret-material issuance. IAM may consume
  workload identity and key-verification material only through ports.
- **ensure:** dependency review rejects PDP engines and key-issuance engines in
  IAM; authorization call sites consume Policy decisions; identity tests never
  treat an IAM role lookup as an allow decision; issuance tests resolve to the
  Secrets owner.
- **overturn_when:** a founder-accepted five-field decision proves a combined
  owner reduces blast radius while preserving fail-closed evaluation,
  independent key custody, and in-process hit-path authorization.

</decision_separation>

<tenant_boundary>

## Decision: IAM is not an HR, Payroll, Accounting, or Kubernetes shell

- **achieves:** first-party applications exercise the same sold identity
  boundary as external tenants and Kubernetes admission remains owned by the
  managed-cluster product.
- **origin:** `tenant-rbac-local-runtime-composition` and
  `tenant-rbac-local-inmemory-harness` import HR, Payroll, and Accounting
  packages directly, while an IAM admission-contract crate is cited as a live
  Kubernetes control.
- **rule:** IAM MUST NOT import or compose an `app/*` core, port, adapter, or
  facade; application products consume IAM through their own adapters to a
  sold public facade. IAM MUST NOT own Kubernetes VAP/CEL, RBAC, PSA,
  admission manifests, or conformance control. Caller-sensitive Kubernetes
  admission is a `k8s/` feature evaluated in the API server without a remote
  IAM or Policy lookup.
- **ensure:** reverse-dependency checks reject IAM-to-app edges; contract tests
  contain no HR, Payroll, or Accounting routes, stores, topics, or workflow
  plans; Kubernetes admission artifacts and failure injection are reviewed by
  the k8s owner.
- **overturn_when:** a founder-accepted five-field decision reallocates a named
  application or Kubernetes responsibility and proves the result still uses
  the public tenant boundary with no in-process privilege path.

</tenant_boundary>

<retirement_gate>

## Decision: tenant-rbac retirement is atomic and founder-gated

- **achieves:** remove the entire false REST/review/evidence product without
  deleting an abstract admission invariant that an accepted ADR still binds to
  one of its paths.
- **origin:** ADR-0719 classifies the tenant-rbac evidence farm, REST/JSON
  surface, and vertical-app composition as removal inventory, while accepted
  ADR-0710 D-9 falsely calls
  `iam/core/tenant-rbac-tenant-admission-policy` a live instance of
  identity-bound Kubernetes admission.
- **rule:** deletion of the tenant-rbac cone MUST remain NON-DISPATCHABLE until
  the founder accepts an ADR-0710 amendment that corrects that concrete live-
  instance claim while preserving the abstract fail-closed, in-process
  VAP/CEL plus RBAC and PSA invariant. After that amendment, one structural
  lane MUST delete all 39 tenant-rbac crate directories and all four named
  hand-authored OpenSLO files, with only the resulting mechanical `Cargo.lock`
  change. It MUST NOT leave a partial 37-crate state, move or salvage a crate,
  split a manifest first, retain REST compatibility, add a fake `main`, or
  create a rename wrapper. `iam/BUCK` is a separate systemic D-17 cleanup and
  is outside that deletion write set.
- **ensure:** the deletion dispatcher verifies founder acceptance and the
  complete inventory before assigning work; review rejects any strict subset;
  negative path scans, Cargo metadata, target-scoped Buck, and protected
  workspace Cargo evidence prove both absence and survivor integrity.
- **overturn_when:** the founder accepts a five-field amendment that names a
  different complete disposition, preserves Kubernetes fail-closed admission,
  and updates both the IAM and k8s owner laws in the same change.

</retirement_gate>

<future_facade>

## Decision: a genuine IAM Connect facade is a new feature

- **achieves:** the eventual sold identity service has one generated contract
  and a real process rather than inheriting the tenant-rbac compatibility
  surface.
- **origin:** `tenant-rbac-app` is a library-only REST/JSON router description;
  renaming it or adding an empty `main` would satisfy layout without producing
  the product ADR-0719 specifies.
- **rule:** a future IAM facade MUST begin from an owner-approved protobuf
  identity contract and accepted Connect code-generation/runtime targets, then
  provide a real `src/main.rs`, listener, bounded request handling, H3 public
  service with H2 Connect fallback, and default-deny Policy integration. It
  MUST be implemented as a separately reviewed new feature, independent of the
  retirement lane, with no standing REST/gRPC/transcode surface and no
  dependency on the tenant-rbac cone.
- **ensure:** structural admission and behavior land in separately reviewable
  lanes; generated bindings are the wire authority; protocol tests reject
  REST, gRPC trailers, malformed framing, cross-tenant principals, unbounded
  bodies, and authorization bypass.
- **overturn_when:** an accepted five-field protocol decision replaces
  ADR-0719's one-Connect-wire rule and supplies an equivalent single-contract,
  fail-closed migration.

</future_facade>

<operational_truth>

## Decision: evidence comes from behavior, not review artifacts

- **achieves:** IAM promotion reflects measured identity behavior and safe
  failure instead of route counts, booleans, or hand-authored SLO declarations.
- **origin:** the tenant-rbac cone treats manifests, expected totals, and
  review-only evidence plans as readiness even though no listener or observed
  workload exists.
- **rule:** IAM promotion MUST use executable contract, integration, recovery,
  and failure-injection tests plus generated SLO material from the platform IR.
  A missing, stale, unavailable, or unverifiable dependency MUST yield a typed
  refusal or unavailable result, never an allow or a green readiness claim.
- **ensure:** the PRD names measurable objectives; the SPEC binds each objective
  to an executable failure; hand-authored OpenSLO, frozen counts, and evidence-
  only crates remain inadmissible.
- **overturn_when:** a founder-accepted five-field evidence decision provides a
  stronger observed mechanism without reintroducing a static census as the
  product.

</operational_truth>

## Authority relationship

ADR-0719 already supplies the owner split, Connect protocol, app-as-tenant
boundary, structural-lane separation, and graph-not-census doctrine. This owner
law specializes those decisions and proposes no ADR-0719 amendment. The only
required apex correction is the founder-gated ADR-0710 concrete-instance
amendment described above.
