---
id: ADR-0561
title: "Commission the workload-identity X.509-SVID issuance + PDP caller-tenant-binding substrate (G002 slice 1; live mTLS = slice-1b)"
status: Superseded
planning_impact: false
deciders: founder, agent-lane g02-workload-svid
date: 2026-06-13
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0701]
amended_by: []
depends_on: [ADR-0510, ADR-0536, ADR-0553, ADR-0559]
amends: []
related: [ADR-0002, ADR-0083, ADR-0131, ADR-0148, ADR-0243, ADR-0295, ADR-0506, ADR-0547, ADR-0550]
related_specs: []
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0561: Commission the workload-identity X.509-SVID issuance + PDP caller-tenant-binding substrate (G002 slice 1; live mTLS = slice-1b)

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

1. **A CA + issuance engine** (`os/core/trustd-domain`): a faithful
   Rust port of the Talos `trustd` PKI — `CertificateAuthority<S: SigningBackend>`, a
   join-token-gated `SecurityService::handle_certificate` minting path, an `IssuancePolicy` that
   already rejects CA-capable leaves, and a `TrustBundle` chain verifier. `SigningBackend` is the
   cutover seam (`InMemorySigner` today → cloud-kms later, ADR-0510).
2. **A SPIFFE + tenant model** (`iam/core/identity-workload-domain`): `TenantId`,
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
   mismatch (the #717 tenant-binding gate; logic only — live enforcement is slice-1b).
   Reuses `oya-identity-workload-domain::TenantId`.
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

### D5-bis — Slice-1b split: 1b-i (trustd REAL X.509) landed; 1b-ii (PDP rustls) still deferred

The original D5 bundled "real crypto" with the rustls transport cutover. Implementation experience
split the deferral into two independently-shippable slices, and **slice-1b-i is now delivered**
(FRIC-1781510000) while **slice-1b-ii remains deferred** (FRIC-1781490000 stays `queued`):

- **Slice-1b-i — trustd REAL X.509 (DELIVERED):** the trustd CA now mints **real** X.509 — a real
  ECDSA-P256 key pair (`rcgen` on the `aws-lc-rs` backend, ADR-0506; **ring FORBIDDEN** — `rcgen`'s
  default features pull ring, so it is configured `default-features = false` + the `aws_lc_rs`
  feature, and `x509-parser` is on `verify-aws`, NOT the ring-backed `verify`; the dep tree carries
  **no ring**) and a real ASN.1 DER `TBSCertificate` built from the existing `Certificate` fields,
  including the SPIFFE id as a real `uniformResourceIdentifier` `GeneralName`. The **`SigningBackend`
  trait is unchanged** — a new `EcdsaP256Signer` implements it, the in-memory keyed-hash signer is
  retained only for the shape-model unit tests, and issuance output is now real DER. The
  **URI-SAN-signature-bound invariant** is preserved by construction: `rcgen` signs the whole
  `TBSCertificate`, so the SPIFFE URI is inside the real signature exactly as it was inside the
  shape-model `tbs_bytes` — a post-issuance URI tamper breaks a **real signature**, not a MAC. The
  **SVID adapter** swaps its TSV1 `leaf_codec` stand-in for **real leaf-DER parsing via
  `x509-parser`** (parse the leaf, extract the single URI SAN → `SpiffeId`, verify the real signature
  against the trust-bundle CA's `SubjectPublicKeyInfo`, check validity). The **SVID kernel ports and
  `bind_caller_tenant` are byte-unchanged** (cutover litmus: only the trustd signer + the adapter
  codec gained real crypto; the kernel/port shapes that model the W5 destination did not move).

- **Slice-1b-ii — PDP rustls mTLS wiring (DELIVERED, 2026-06-13, FRIC-1781490000):** the live
  transport is built and RED-proven over real handshakes. A custom rustls `ClientCertVerifier`
  (`oya-cloud-iam-pdp-app/src/client_cert_verifier.rs`) on the `aws-lc-rs` provider (NO ring,
  ADR-0506) requires a client cert (`client_auth_mandatory`) and defers the leaf trust decision to
  `TrustdSvidVerifier::verify_peer` — the SAME real-DER verification the in-process PEP uses, so the
  transport check and the PEP check cannot diverge. Both listeners terminate the handshake via one
  `tokio_rustls::TlsAcceptor` (`oya-cloud-iam-pdp-app/src/mtls_transport.rs`): gRPC feeds the
  TLS-terminated streams to tonic's `serve_with_incoming` with a custom `Connected` impl surfacing the
  verified peer leaf as a request extension (NO tonic TLS feature — confined to the PDP crate); REST
  runs a manual `hyper-util` accept loop layering the peer leaf as an axum `Extension`. The
  `grpc.rs`/`rest.rs` call sites invoke `SpiffeCallerAuth::authenticate_caller` BEFORE deciding,
  binding the tenant from the SVID and replacing the verbatim `request.tenant_id` (the #717 closure,
  live); every reject is `PermissionDenied`/`403`, never `404`, never a fall-through. Boot is
  fail-closed: an empty trust bundle is `StartError::Mtls(TrustBundleEmpty)`. Five real-handshake E2E
  fixtures (`tests/mtls_live_socket.rs`) prove trusted-SVID ALLOW with SVID-tenant binding (REST +
  gRPC), rogue/untrusted-CA rejection, expired rejection, cross-tenant `403`/`PermissionDenied`,
  no-client-cert refusal, and the empty-bundle boot-refuse. The capability is RED-proven over real
  handshakes; the production boot wiring landed in slice-1b-iii-a/b (below).

- **Slice-1b-iii-a/b — production mTLS boot from a delivered cert mount (LANDED, 2026-06-13,
  FRIC-1781490000 STAYS OPEN):** `main()` no longer boots plain TCP. A new runtime source
  `MtlsContext::from_path(dir)` (`oya-cloud-iam-pdp-app/src/mtls_transport.rs`) builds the
  `MtlsContext` from the delivered kubernetes.io/tls Secret projection (`tls.crt`/`tls.key`/`ca.crt`,
  PEM), extracting **each CA's REAL `SubjectPublicKeyInfo` DER** (via `x509-parser`) into the trust
  anchor's `public_key_der` — the value the live rustls verify path consults
  (`TrustBundle::trusted_ca_spki_ders`); the anchor's attached signer + shape-model `signature` are
  inert on that path (the correctness finding). A new config knob
  `OYA_CLOUD_IAM_PDP_MTLS_CERT_DIR` (default `/etc/oya-cloud-iam-pdp/tls`) resolves the mount. The
  boot decision is extracted into `server::boot_from_config` — the SINGLE body both `main()` and the
  production-path closure E2E run (tested wiring IS production wiring): it does `from_path` +
  `start_with_mtls`, **fail-closed** — an absent/empty/malformed mount is a HARD `BootError` and
  `main` exits non-zero (BOOT REFUSAL), NEVER a downgrade to plain TCP. `start()` (plain TCP) remains
  the shared boot body / test helper but is unreachable from `main`. The Helm Deployment mounts the
  `oya-cloud-iam-pdp-svid` Secret read-only at `/etc/oya-cloud-iam-pdp/tls` and sets the env knob.
  The production-path closure E2E (`tests/main_boot_closure.rs`) boots through `boot_from_config`
  from a real operator-shaped mount and drives real rustls client handshakes (trusted SVID → ALLOW
  bound to the SVID tenant; cross-tenant → 403), plus a missing-cert RED fixture proving the boot
  helper refuses (no plain socket binds). **FRIC-1781490000 STAYS OPEN** until slice-1b-iii-c — the
  **live SVID operator** that PRODUCES the `oya-cloud-iam-pdp-svid` Secret in-cluster — is the
  delivery source, because genuine end-to-end production closure requires REAL cert delivery, not
  test-written material (the twice-burned overclaim rule, #722/#725). The **cloud-kms signer swap**
  also remains. Until iii-c lands, the prod pod fail-closes without the mount (which is correct), and
  a consumer (and the G004 PDP slice-2 author) MUST NOT treat the live PDP as cryptographically
  tenant-authenticated end-to-end.

The `SigningBackend` cutover seam (D4) is now exercised by a real asymmetric backend, confirming the
seam shape was already correct: the CA, `TrustBundle`, `SecurityService`, and the kernel ports did
not change when `InMemorySigner` was replaced by `EcdsaP256Signer` for production issuance.

## Threat model (all fail-closed)

- **Spoofing (tenant impersonation, the #717 root)**: a caller asserts `tenant_id` it does not own.
  Mitigated **by the slice-1 logic** (RED-proven in-process): the tenant is derived from the verified
  SVID path; a request-body tenant that disagrees is `TenantMismatch` → DENY; the body tenant is only
  ever a cross-check input. **NOT YET ENFORCED in production**: the `SpiffeCallerAuth` PEP is built and
  tested but unwired — `server.rs` still binds plain `TcpListener` and `grpc.rs` still trusts
  `request.tenant_id` verbatim. The live gap closure (real rustls mTLS transport invoking the PEP) is
  the deferred slice-1b and remains open, tracked as `FRIC-1781490000`.
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

- iam/core/identity-workload-svid-kernel/src/lib.rs
- iam/core/identity-workload-svid-kernel/Cargo.toml
- iam/core/identity-workload-svid-kernel/BUCK
- iam/core/identity-workload-svid-kernel/OWNERS
- iam/observability/slos/identity-workload-svid-kernel/workload-svid-issuance-availability.openslo.yaml

New SVID trustd adapter crate (owned by axis-identity, reachable via cargo-members):

- iam/adapters/identity-workload-svid-trustd/src/lib.rs
- iam/adapters/identity-workload-svid-trustd/src/leaf_codec.rs
- iam/adapters/identity-workload-svid-trustd/Cargo.toml
- iam/adapters/identity-workload-svid-trustd/BUCK
- iam/adapters/identity-workload-svid-trustd/OWNERS

New PDP mTLS PEP module (owned by the existing cloud/cloud-iam OWNERS,
axis-cloud-platform; reachable via cargo-members):

- iam/facade/cloud-pdp-app/src/mtls.rs

The two new crate roots are owned by axis-identity (a born-at-creation OWNERS in
each crate directory per ADR-0555) and reachable via the globbed Cargo workspace
membership (cargo-members). The trustd X.509 extension
(os/core/trustd-domain/src/x509.rs,
.../src/certificate.rs, .../src/ca.rs) amends already-accounted files in place
and carries no new accounting rows.

Slice-1b-i (FRIC-1781510000) adds two new source files:

- os/core/trustd-domain/src/der.rs — real ASN.1 DER
  issuance (rcgen on the aws-lc-rs backend). Its crate (oya-cloud-os-trustd-domain)
  was previously UNOWNED baseline debt (no OWNERS marker), so der.rs would be born
  unowned. Slice-1b-i therefore born-accounts the crate: it adds
  os/core/trustd-domain/OWNERS (owner: axis-cloud-platform,
  matching the sibling cloud/* substrate crates cloud-iam/cloud-kms/cloud-kernel),
  registered as reached in specs/reachability-registry.json — which owns der.rs and
  pays down the crate's prior unowned debt (12 source files newly owned, zero new
  unowned). This OWNERS marker is justified by its citation here.
- iam/adapters/identity-workload-svid-trustd/src/leaf_der.rs —
  real leaf-DER parsing + signature verification (x509-parser, verify-aws). Its crate
  is already owned (axis-identity OWNERS from slice-1), so leaf_der.rs inherits it.

The retired oya-identity-workload-svid-trustd-adapter/src/leaf_codec.rs (the TSV1
shape-model stand-in) is removed in the same change; its accounting row is
retired with it. The new third-party deps (rcgen, x509-parser, time) ride the
reindeer-generated third-party/BUCK and the workspace Cargo.toml/Cargo.lock.

Slice-1b-ii (FRIC-1781490000) adds two new source files + one E2E test, all
owned by the existing cloud/cloud-iam OWNERS (axis-cloud-platform) and reachable
via cargo-members (they inherit the crate's accounting; no new unowned rows):

- iam/facade/cloud-pdp-app/src/client_cert_verifier.rs — the
  rustls `ClientCertVerifier` deferring leaf trust to the SVID verifier (aws-lc-rs
  provider, NO ring; ADR-0506).
- iam/facade/cloud-pdp-app/src/mtls_transport.rs — the
  one-acceptor mTLS transport (`MtlsContext`, the gRPC `Connected` peer-cert
  stream, the REST hyper-util accept loop).
- iam/facade/cloud-pdp-app/tests/mtls_live_socket.rs — the
  five real-handshake RED fixtures + boot-refuse.

The new third-party deps (rustls, tokio-rustls, hyper, hyper-util, tower,
futures-core, async-stream — all aws-lc-rs/ring-free) ride the workspace
Cargo.toml/Cargo.lock and the third-party/BUCK aliases; no tonic TLS feature is
enabled (the gRPC peer-cert capture is confined to the PDP crate).

Slice-1b-iii-a/b (FRIC-1781490000) adds one new test file (the production-path
closure E2E + the fail-closed RED fixtures), owned by the existing
cloud/cloud-iam OWNERS (axis-cloud-platform) and reachable via cargo-members (it
inherits the PDP crate's accounting; no new unowned/OWNERS row). The from_path
runtime source + the OYA_CLOUD_IAM_PDP_MTLS_CERT_DIR knob + the main()/server.rs
boot switch + the Helm SVID mount amend already-accounted files in place:

- iam/facade/cloud-pdp-app/tests/main_boot_closure.rs — the
  production-path closure E2E (boot through `server::boot_from_config` from a real
  operator-shaped cert mount → trusted-SVID ALLOW + cross-tenant 403) plus the
  fail-closed RED fixtures (absent/empty mount → boot refusal; from_path unit
  RED: MountUnreadable / Empty / NoCaAnchors / MalformedPem).

The new dev-dep `base64` (operator-shaped PEM serialization in the closure
fixture) rides the workspace Cargo.toml/third-party//:base64; it is dev-only and
adds no library surface.

## Consequences

- **Positive**: the PDP tenant is cryptographically bound; G004 slice-2 per-tenant policy is
  unblocked; the SVID substrate is reusable by every platform PEP; clean kernel/adapter/app seams.
- **Negative / tracked**: a transitional trust-domain duality (D2, ledgered); real transport mTLS +
  real crypto deferred to slice-1b (D5); the leaf codec is a DER stand-in retired at cutover.
- **Reversible**: two-way door — deleting the new crates + the PDP `mtls` module restores `dev`.

## Governed surfaces

Slice-1b-iii-c (FRIC-1781490000, the cert-delivery dimension) commissions the
in-cluster SVID-delivery operator that PRODUCES the `oya-cloud-iam-pdp-svid`
Secret the PDP mTLS PEP boots from — the single missing producer that closes the
cert-delivery dimension of FRIC-1781490000 and unblocks G004. Each exact path
below is its row's structural-accounting `justification_ref` (ADR-0555
born-accounted-at-creation; total-accounting requires every NEW tracked path be
ADR-justified, owned, and reachable). The operator roots issuance on the trustd
`EcdsaP256` CA via the unchanged `SigningBackend` seam; the cloud-kms per-cell
sealing-root swap stays DEFERRED behind that seam (D4/D5) — this slice closes the
CERT-DELIVERY dimension only and makes no full-G002-completion claim.

New SVID operator kernel crate (owned by axis-identity, reachable via cargo-members):

- iam/core/identity-workload-svid-operator-kernel/src/lib.rs
- iam/core/identity-workload-svid-operator-kernel/tests/reconcile.rs
- iam/core/identity-workload-svid-operator-kernel/Cargo.toml
- iam/core/identity-workload-svid-operator-kernel/BUCK
- iam/core/identity-workload-svid-operator-kernel/OWNERS
- iam/observability/slos/identity-workload-svid-operator-kernel/svid-delivery-availability.openslo.yaml

New SVID operator kube-rs adapter crate (owned by axis-identity, reachable via cargo-members):

- iam/adapters/identity-workload-svid-operator-k8s/src/lib.rs
- iam/adapters/identity-workload-svid-operator-k8s/tests/adapter.rs
- iam/adapters/identity-workload-svid-operator-k8s/Cargo.toml
- iam/adapters/identity-workload-svid-operator-k8s/BUCK
- iam/adapters/identity-workload-svid-operator-k8s/OWNERS

New SVID operator app crate (owned by axis-identity, reachable via cargo-members):

- iam/facade/identity-workload-svid-operator-app/src/lib.rs
- iam/facade/identity-workload-svid-operator-app/src/main.rs
- iam/facade/identity-workload-svid-operator-app/tests/app.rs
- iam/facade/identity-workload-svid-operator-app/Cargo.toml
- iam/facade/identity-workload-svid-operator-app/BUCK
- iam/facade/identity-workload-svid-operator-app/OWNERS

The three new crate roots are owned by axis-identity (a born-at-creation OWNERS in
each crate directory per ADR-0555) and reachable via the globbed Cargo workspace
membership (the `iam/*/*` member glob). The operator-kernel SLO at
`iam/observability/slos/identity-workload-svid-operator-kernel/` is non-crate-resident
(ADR-0139 SLO-home convention), so it carries an explicit reachability seed in
`specs/reachability-registry.json` justified by this ADR.

The keystone closure extends an already-accounted test file in place (no new
accounting row): `iam/facade/cloud-pdp-app/tests/main_boot_closure.rs` gains the
`operator_produced_secret_boots_pdp_and_yields_real_allow_deny_handshake` fixture —
the PDP boots from OPERATOR-PRODUCED mTLS material and a caller SVID minted from
the SAME operator CA yields a real rustls ALLOW (bound to its SVID tenant) +
cross-tenant 403. The two new dev-deps (the operator k8s + kernel crates) ride the
PDP crate's `[dev-dependencies]`; they are dev-only and add no library surface.

The Helm operator Deployment + RBAC + PDB (Secret create/update/patch scoped to the
`oya-cloud-iam-pdp-svid` Secret in the cloud-iam namespace) amend the existing
cloud/cloud-iam Helm chart (owned by the existing cloud/cloud-iam OWNERS,
axis-cloud-platform); the new templates carry no new crate accounting rows but ARE
new tracked paths, so each is born-accounted here (this ADR is their
`justification_ref`) and seeded reachable in `specs/reachability-registry.json`
(the non-crate-resident Helm templates are NOT reached via cargo-members):

- cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-deployment.yaml
- cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-rbac.yaml
- cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-pdb.yaml
