---
id: ADR-0561
title: "Commission the workload-identity X.509-SVID issuance substrate + PDP mTLS caller-auth (G002 slice)"
status: Proposed
planning_impact: false
deciders: founder, agent-lane g02-workload-svid
date: 2026-06-13
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0510, ADR-0536, ADR-0553, ADR-0559]
amends: []
related: [ADR-0002, ADR-0083, ADR-0131, ADR-0148, ADR-0243, ADR-0295, ADR-0506, ADR-0547, ADR-0550]
related_specs: []
milestone: W0
---

# ADR-0561: Commission the workload-identity X.509-SVID issuance substrate + PDP mTLS caller-auth (G002 slice)

## Status

**Proposed — 2026-06-13 (G002 Trust Substrate slice; founder door. Door: two-way — the new
SVID kernel + trustd adapter + PDP mTLS PEP are additive; no consumer is forced onto them yet,
and deleting the new crates + the PDP `mtls` module restores the prior state.)**

## Context

ADR-0559 commissioned the cloud-iam Cedar PDP as a runnable service. Its `AuthorizeRequest`
carries `tenant_id` (proto field 2) and the gRPC handler binds it **verbatim** from the caller
body (`grpc.rs`: `tenant_id: request.tenant_id`), and the listeners bind a plain `TcpListener`
with **no TLS** (`server.rs`). The PDP therefore trusts the caller's claimed tenant with no
cryptographic proof: any caller can assert any tenant. This is the open gap tracked as the #717
closure and is **ADR-0559 D6 step 0** — the prerequisite that unblocks the G004 PDP slice 2
(per-tenant policy evaluation cannot be safe while the tenant is caller-asserted).

The platform already has the two halves needed to close it, on `dev`:

1. **A CA + issuance engine** (`cloud/cloud-os/crates/oya-cloud-os-trustd-domain`): a faithful
   Rust port of the Talos `trustd` PKI — `CertificateAuthority<S: SigningBackend>`, a
   join-token-gated `SecurityService::handle_certificate` minting path, an `IssuancePolicy` that
   already rejects CA-capable leaves, and a `TrustBundle` chain verifier. `SigningBackend` is the
   cutover seam (`InMemorySigner` today → cloud-kms later, ADR-0510).
2. **A SPIFFE + tenant model** (`oya/identity/crates/oya-identity-workload-domain`): `TenantId`,
   `WorkloadPrincipal`, and a **tenant-rooted** `TrustDomain` (`spiffe://<tenant>`).

This slice connects them: give the PDP **mTLS caller authentication via SPIFFE-style X.509-SVID,
fail-closed**, deriving the authorized tenant from a cryptographically-verified peer SVID rather
than the request body.

## Decision

Commission three additive units (clean-arch kernel/adapter/app split, mirroring the ADR-0559 PDP
split), plus the X.509 work in the trustd domain to carry a SPIFFE identity:

1. **`oya-identity-workload-svid-kernel`** (PURE, no-IO): the cell-rooted `SpiffeId` +
   `WorkloadPath` value types; the `WorkloadIdentityIssuer` / `SvidVerifier` / `TrustBundleSource`
   ports; and the fail-closed `bind_caller_tenant(&SpiffeId, &TenantId) -> Result<TenantId, …>`
   gate that derives the authorized tenant from a verified SVID path and rejects any caller-body
   mismatch (the #717 closure). Reuses `oya-identity-workload-domain::TenantId`.
2. **`oya-identity-workload-svid-trustd-adapter`**: implements the kernel ports over the trustd
   CA — issuance via `SecurityService::handle_certificate` with a new `for_workload` URI-SAN CSR
   shape; verification via `TrustBundle::verify_leaf` + single-URI-SAN extraction.
3. **PDP mTLS PEP** (`oya-cloud-iam-pdp-app::mtls`): `SpiffeCallerAuth` authenticates a caller by
   its verified peer SVID and binds the tenant **before** the entity-ref build, boot-refusing when
   the trust bundle is empty (mirroring the PDP's policy-bundle boot-refusal).
4. **Trustd X.509 SVID carrier**: `SubjectAltNames` gains a `uris` field (the
   `uniformResourceIdentifier` general-name that SPIFFE X.509-SVID §2 puts the identity in), signed
   into `Certificate::tbs_bytes` so the identity is bound, with a `for_workload` CSR mirroring
   `for_node`.

All paths are fail-closed: gRPC `PermissionDenied` / REST `403` (NEVER `404`, never a
fall-through) on no client cert, untrusted issuer, expired SVID, malformed SVID, tenant mismatch,
malformed request-tenant, and platform-SVID-asserting-a-tenant.

### Hyperscaler precedent

This is the SPIFFE/SPIRE workload-identity pattern (CNCF; Google ALTS / BeyondProd service
identity; Istio mesh SVID) reimplemented Rust-native per the founder "proven patterns, Rust
reimplementation" doctrine. The cell-rooted naming follows the existing ADR-0295 SPIRE binding
(`oya/api-gateway/iac/spire-trust-bundle.yaml`: `spiffe://oyatie.cell-${CELL_ID}/api-gateway/envoy`).
The `IssuancePolicy` CA-leaf rejection mirrors a real CA's basicConstraints `cA:FALSE` enforcement.

## D-sections

### D1 — Trust-domain naming: CELL-ROOTED authority + tenant-in-path (RESOLVED)

The PDP is a **platform** service: it serves all tenants and is owned by none. The existing
tenant-rooted model (`spiffe://<tenant>`, `oya-identity-workload-domain::TrustDomain::for_tenant`)
**cannot name it** — there is no single tenant to root its authority at. We adopt the cell-rooted
scheme already in production use (ADR-0295), encoding the tenant in the path:

- platform workload: `spiffe://oyatie.cell-<id>/platform/<service>`
  (e.g. `spiffe://oyatie.cell-7/platform/cloud-iam-pdp`)
- tenant workload: `spiffe://oyatie.cell-<id>/tenant/<ten_x>/<workload>`
  (e.g. `spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync`)

The trust-domain **authority** is the cell (`oyatie.cell-<id>`); the **tenant** is a path segment.
`bind_caller_tenant` derives the tenant from the `tenant/<ten_x>/…` path and refuses a
`platform/…` SVID outright (a platform identity must never assert it *is* a tenant). This is the
key conflict-resolution: the tenant-rooted invariant (`trust_domain == tenant`) is sound for
single-tenant workloads but is structurally unable to name a multi-tenant platform service, so the
slice introduces the cell-rooted authority alongside it rather than overloading the old shape.

**Additive, non-breaking**: slice-1 introduces a NEW `SpiffeId` parser in the new SVID kernel and
does **not** touch the existing tenant-rooted `TrustDomain` in `oya-identity-workload-domain`.
Existing consumers (oya-identity workload OIDC path, whose SVID trust domain is `spiffe://ten_acme`)
keep working unchanged. The two schemes coexist by construction.

### D2 — Legacy TrustDomain convergence (scheduled follow-up)

The two trust-domain shapes (tenant-rooted in `oya-identity-workload-domain`, cell-rooted in the
SVID kernel) are a deliberate, ledger-tracked transitional duality, not a permanent fork. A tracked
follow-up IP will converge the legacy `TrustDomain` onto the cell-rooted scheme (tenant workloads
re-expressed as `spiffe://oyatie.cell-<id>/tenant/<ten_x>/<workload>`), migrating the OIDC-path
consumers behind the same `SpiffeId` parser and retiring `TrustDomain::for_tenant`. Until then the
duality is absorbed at the kernel boundary (two parsers, one binding gate) and carries a friction
ledger row.

### D3 — Cedar gate binding (ADR-0243)

The bound tenant feeds `principal.tenant_id` on the Cedar authorization request (ADR-0243 universal
gate). With this slice the PDP's `tenant_id` is **SVID-derived**, so every per-tenant Cedar policy
condition (`principal.tenant_id == resource.tenant_id`-class rules, the G004 slice-2 surface) now
rests on a cryptographically-proven tenant instead of a caller-asserted string.

### D4 — Signer cutover (ADR-0510)

Issuance signs through the trustd `SigningBackend` seam. Slice-1 uses the deterministic in-memory
signer (the shape model); the cloud-kms signer swap is ADR-0510 transitional work and lands behind
the unchanged `SigningBackend` trait — the kernel ports and the PDP PEP do not change at cutover
(the ports model the W5 destination: real DER bytes in, a verified `SpiffeId` out).

### D5 — DEFERRED slice-1b (documented, NOT built in this slice)

Slice-1-core is the **in-process** issuance + verification + PDP tenant-binding logic, fully
testable without K8s. The following are explicitly deferred and design-of-record here:

- **Real transport mTLS**: a rustls `ServerConfig` requiring + verifying a client cert on the PDP
  listeners, handing the verified peer leaf DER to `SvidVerifier::verify_peer`. The trustd domain is
  a faithful *shape* model (its "DER" is a deterministic encoding, its signatures keyed hashes), so
  a real rustls handshake cannot interoperate with trustd-issued certs; wiring real rustls is part
  of the same cutover as the cloud-kms real-crypto swap and is deferred to avoid shipping a
  half-real, fragile transport. The slice-1 PEP consumes the post-handshake peer-leaf bytes a real
  rustls layer would hand it, via an adapter-owned leaf codec (the DER stand-in).
- **K8s cert-delivery**: an operator-reconciled projected `Secret` + init-container SVID fetch with
  a fetch-fail = deploy-fail gate; a CRD + operator mirroring the cloud-kms operator.
- **Per-cell sealing-root CA rooting**, **SPIRE node-attestation depth**, **mesh-wide rollout**, and
  the **cloud-kms signer swap**.

## Threat model (all fail-closed)

- **Spoofing (tenant impersonation, the #717 root)**: a caller asserts `tenant_id` it does not own.
  Mitigated: the tenant is derived from the verified SVID path; a request-body tenant that disagrees
  is `TenantMismatch` → DENY. The body tenant is only ever a cross-check input.
- **SVID theft / replay**: a stolen leaf is presented by an attacker. Bounded by short TTL (issuance
  caps TTL via `IssuancePolicy`) + expiry enforcement (`Expired` → DENY); full rotation + revocation
  ride the CRL already in trustd. Deeper binding (proof-of-possession) is slice-1b mesh work.
- **Bundle poisoning**: an attacker swaps the trust bundle for one trusting a rogue CA. Mitigated by
  boot-refusal on an empty/garbage bundle and, at cutover, signed bundle distribution (ADR-0536 D-2);
  an untrusted issuer is `UntrustedSvid` → DENY.
- **Fail-open**: any verification or binding error must DENY, never allow or fall through. Enforced:
  every reject path returns `PermissionDenied`/`403` (never `404`, never a default), and an
  undecodable/garbage leaf is an `UntrustedSvid` DENY, not a panic.
- **CA-capable leaf**: a workload requests a signing cert to mint its own identities. Mitigated by
  the `IssuancePolicy` CA-leaf rejection (regression-guarded by `issuance_policy_rejects_ca_leaf`).

## Commissioned + governed artifacts (structural-accounting justification)

This decision commissions and governs the following tracked artifacts; each
exact path is the structural-accounting `justification_ref` for its row
(ADR-0555 born-accounted-at-creation; the firewall's total-accounting gate
requires every NEW tracked path be ADR-justified, owned, and reachable):

New SVID kernel crate (owned by axis-identity, reachable via cargo-members):

- oya/identity/crates/oya-identity-workload-svid-kernel/src/lib.rs
- oya/identity/crates/oya-identity-workload-svid-kernel/Cargo.toml
- oya/identity/crates/oya-identity-workload-svid-kernel/BUCK
- oya/identity/crates/oya-identity-workload-svid-kernel/OWNERS
- oya/identity/crates/oya-identity-workload-svid-kernel/slos/workload-svid-issuance-availability.openslo.yaml

New SVID trustd adapter crate (owned by axis-identity, reachable via cargo-members):

- oya/identity/crates/oya-identity-workload-svid-trustd-adapter/src/lib.rs
- oya/identity/crates/oya-identity-workload-svid-trustd-adapter/src/leaf_codec.rs
- oya/identity/crates/oya-identity-workload-svid-trustd-adapter/Cargo.toml
- oya/identity/crates/oya-identity-workload-svid-trustd-adapter/BUCK
- oya/identity/crates/oya-identity-workload-svid-trustd-adapter/OWNERS

New PDP mTLS PEP module (owned by the existing cloud/cloud-iam OWNERS,
axis-cloud-platform; reachable via cargo-members):

- cloud/cloud-iam/crates/oya-cloud-iam-pdp-app/src/mtls.rs

The two new crate roots are owned by axis-identity (a born-at-creation OWNERS in
each crate directory per ADR-0555) and reachable via the globbed Cargo workspace
membership (cargo-members). The trustd X.509 extension
(cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/x509.rs,
.../src/certificate.rs, .../src/ca.rs) amends already-accounted files in place
and carries no new accounting rows.

## Consequences

- **Positive**: the PDP tenant is cryptographically bound; G004 slice-2 per-tenant policy is
  unblocked; the SVID substrate is reusable by every platform PEP; clean kernel/adapter/app seams.
- **Negative / tracked**: a transitional trust-domain duality (D2, ledgered); real transport mTLS +
  real crypto deferred to slice-1b (D5); the leaf codec is a DER stand-in retired at cutover.
- **Reversible**: two-way door — deleting the new crates + the PDP `mtls` module restores `dev`.
