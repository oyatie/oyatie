---
doc_class: IP
template_id: TPL-IP
ip_id: IP-017
microservice: identity
status: proposed
related_adrs: [ADR-0476, ADR-0506, ADR-0507, ADR-0508]
related_crates:
  - iam-identity-workload-oidc
  - oya-shared-webauthn-server-kernel
date: 2026-06-30
owner_team: axis-identity + council-architecture
---

# IP-017 — Bespoke identity/authn/crypto bridge tracking

## Goal

Track the first atomic IDENTITY-003 bridge slice from ADR-0476, ADR-0506,
ADR-0507, and ADR-0508 without turning it into a full identity implementation
epic. This IP records the Phase-1 `oya-identity` surface, the Phase-1 crypto and
WebAuthn/authenticator bridges, and the promotion/cutover gates downstream lanes
must satisfy before claiming product completion.

This slice is a tracking and sequencing artifact only. It does not add a new
runtime, does not vendor OpenSK source, does not introduce a new identity IdP
binary, and does not change dependency selection in Cargo/Buck.

## Atomic exit checklist

| Exit item | Tracking anchor | Done in this IP |
|---|---|---|
| `oya-identity` Phase-1 surface | ADR-0476 plus the surface table below | Yes: OIDC/OAuth2, WebAuthn, MFA, federation, Cedar, audit, and multi-region rows are enumerated with current/future anchors. |
| aws-lc-rs provider | ADR-0506, root `Cargo.toml`, `iam/adapters/identity-workload-oidc/src/lib.rs` | Yes: provider invariants and no-ring promotion gates are explicit. |
| webauthn-rs RP parity table | ADR-0507, `libs/oya-shared-webauthn-server-kernel`, IP-004/IP-005 | Yes: every ADR-0507 parity row has a tracked Oyatie surface and cutover gate. |
| OpenSK vendored reference | ADR-0508, `tools/opensk-vendored/README.md`, `oya/oya-authn-device-firmware/catalog.yaml` | Yes: reference-vendoring status, non-goals, and oya-authn-device gates are explicit. |
| Promotion/cutover gates | ADR-0476/0506/0507/0508 bridge sections | Yes: Phase-1 admission, bridge-retirement, and bespoke cutover gates are listed below. |

## Authority map

| Source | Binding decision | Current tracked surface | Non-claim |
|---|---|---|---|
| ADR-0476 | `oya-identity` is the bespoke Rust human identity destination; OIDC provider, OAuth 2.0 authorization server, WebAuthn/passkeys, TOTP/HOTP, tenant IdP federation, Cedar integration, and multi-region sessions are the Phase-1 surface. | `oya/identity/` IPs/catalog/contracts plus workload identity seams under `iam/`. | This IP does not claim the whole Phase-1 runtime is implemented. |
| ADR-0506 | aws-lc-rs is the canonical Phase-1 crypto provider; `ring` must not be activated in production/development identity crypto paths. | Workspace `aws-lc-rs` dependency; workload OIDC verifier uses `aws_lc_rs`; crypto-backend-purity gate is the no-ring enforcement anchor. | This IP does not move to bespoke `oya-crypto`. |
| ADR-0507 | webauthn-rs is the Phase-1 WebAuthn relying-party bridge; oya-webauthn is the Tier-2 bespoke destination. | `libs/oya-shared-webauthn-server-kernel` adapter boundary and IP-004/IP-005 ceremony/REST plans. | This IP does not add a concrete webauthn-rs adapter crate. |
| ADR-0508 | OpenSK is the Phase-1 authenticator-side reference; oya-authn-device is the Tier-3 bespoke hardware destination. | `tools/opensk-vendored/README.md` reference directory and `oya/oya-authn-device-firmware/catalog.yaml`. | This IP does not vendor upstream OpenSK source or build firmware. |

## `oya-identity` Phase-1 surface tracking

| Surface | ADR-0476 Phase-1 requirement | Current repo anchor | Promotion gate before claiming Phase-1 live |
|---|---|---|---|
| OIDC provider discovery | Per-tenant `/.well-known/openid-configuration` and issuer metadata. | `oya/identity/IP-002-oidc-issuer-kernel.md`, `oya/identity/contracts/openapi/identity.yaml`, `registry/catalog/iam-identity-oidc-issuer-kernel.yaml`. | Discovery tests prove issuer, JWKS URI, token endpoint, auth endpoint, and tenant realm URL match pack/tenant configuration. |
| OAuth 2.0 authorization server | Authorization and token endpoints with PKCE; implicit flow prohibited. | `oya/identity/IP-002-oidc-issuer-kernel.md`, `oya/identity/contracts/openapi/identity.yaml`. | Contract tests reject implicit flow, require PKCE, and emit audit on token issuance/denial. |
| JWKS verification / workload bridge | Workload and service consumers verify issuer tokens offline. | `iam/adapters/identity-workload-oidc/src/lib.rs`, `iam/ports/identity-workload-api`, `iam/core/identity-workload-domain`. | Issuer-published JWKS parsing + offline ES256/RS/EdDSA verification pass; policy default-deny remains 403. |
| WebAuthn / passkey RP | Registration and authentication ceremonies for first factor and step-up. | `libs/oya-shared-webauthn-server-kernel`, `oya/identity/IP-004-webauthn-relying-party-kernel.md`, `oya/identity/IP-005-webauthn-rest.md`. | The webauthn-rs-backed adapter verifies CBOR/COSE/attestation and the kernel tests continue passing. |
| TOTP/HOTP and ACR step-up | MFA and step-up gates for sensitive flows. | `oya/identity/IP-010-step-up-orchestrator.md`, `oya/identity/capabilities/webauthn-authenticate.yaml`. | ACR policy tests prove routine/elevated/sensitive/critical flows; audit events identify step-up grants and refusals. |
| Tenant IdP federation | Broker enterprise Okta/Auth0/AzureAD-style IdPs without storing federated credentials. | `oya/identity/IP-011-external-idp-federation.md`, `docs/runbooks/identity-provider-federation.md`. | Federation contract tests prove per-tenant issuer binding, no credential custody, JWKS refresh, and fail-closed discovery mismatch. |
| SCIM user/group lifecycle | Provision users/groups and deactivation flows. | `oya/identity/IP-007-scim-server-kernel.md`, `oya/identity/IP-008-scim-adapter-zitadel.md`, `iam/adapters/identity-scim-store-postgres`. | SCIM mutation tests prove tenant RLS, idempotency, deactivation, and audit-chain emission. |
| Cedar principal integration | Unified Cedar namespace for human and workload principals. | `oya/identity/policy/identity.cedar`, `iam/adapters/identity-workload-authz-cedar`, `libs/oya-shared-platform-contracts-kernel/src/identity.rs`. | Authorization tests prove human `User::"<sub>"` and workload principals share policy semantics with default deny. |
| Audit and observability | Identity issuance/authentication/authz events and SLO evidence. | `oya/identity/IP-012-audit-emitter.md`, `iam/observability/slos/identity/`, `oya/identity/dashboards/identity-overview.json`. | Audit completeness is 1.0 for registration, authentication, token issue, federation, SCIM, and denial events. |
| Multi-region sessions | No session affinity; replicated session state. | ADR-0476 D5; current repo has planning anchors but no live replication implementation in this slice. | Cross-region failover test proves RTO/RPO target and session continuity before any live claim. |

## aws-lc-rs provider invariants

| Invariant | Tracking anchor | Required evidence |
|---|---|---|
| `aws-lc-rs` is the identity crypto provider for Phase-1. | Root `Cargo.toml` workspace dependency; `iam/adapters/identity-workload-oidc/src/lib.rs`. | Targeted `cargo test -p iam-identity-workload-oidc --tests`; `cargo tree -i ring --target all` evidence remains zero activated ring. |
| JOSE/JWS verification must not select algorithms from untrusted headers alone. | `iam/adapters/identity-workload-oidc/src/lib.rs` algorithm binding checks. | Tests reject `alg:none`, HS confusion, unknown `kid`, mismatched `alg`, and untrusted `jku`/`x5u`. |
| Future identity TLS/http clients use aws-lc-rs rustls, not native-tls/OpenSSL/ring. | Root `Cargo.toml` `rustls`, `hyper-rustls`, `sqlx`, `reqwest` comments and ADR-0506. | Dependency diff review for any identity PR; crypto-backend-purity gate green. |
| The bespoke destination is `oya-crypto`, not a direct in-place rewrite of identity. | ADR-0506 cutover section. | kubers Phase-B kernel proofs, oya-crypto FIPS validation, and security-isolation conformance evidence exist before migration. |

## webauthn-rs RP parity tracking

| ADR-0507 parity row | Current Oyatie tracking surface | Bridge-cutover gate |
|---|---|---|
| Registration ceremony | `WebauthnServer::begin_registration` / `finish_registration` in `libs/oya-shared-webauthn-server-kernel`; IP-005 REST start/finish endpoints. | Concrete webauthn-rs adapter validates browser `navigator.credentials.create` response and kernel stores tenant/user-bound credential. |
| Authentication ceremony | `begin_authentication` / `finish_authentication` and sign-count replay defense. | Concrete adapter validates `navigator.credentials.get` assertion; replay regression test remains red/green. |
| Attestation formats | `AttestationConveyance`, AAGUID allowlist, pack-tier policy in the kernel. | webauthn-rs adapter proves packed, fido-u2f, none, tpm, android-key, android-safetynet, and apple support or records explicit unsupported rows. |
| Algorithms | COSE key is opaque in the kernel; crypto verification is adapter-owned. | Adapter evidence covers ES256/ES384/ES512/RS256/EdDSA or records an ADR-backed gap before claiming parity. |
| User verification | `UserVerification` and pack/ACR policy inputs in kernel and IP-005 REST. | Tests prove required/preferred/discouraged behavior and tenant policy override propagation. |
| Resident/discoverable credentials | Empty `allow_credentials` + `Mediation::Conditional` models conditional UI/discoverable credential flow. | Browser/virtual-authenticator conformance proves cross-tenant discovery is not possible. |
| Multi-credential per user | `exclude_credentials` prevents duplicate registration; credential store accepts multiple IDs per user. | REST/Postgres tests prove list, add, revoke, rotate, and duplicate prevention. |
| Backup eligibility / backup state | `Credential.backup_eligible` and `Credential.backup_state` are persisted fields. | Risk-policy tests consume BE/BS flags and emit audit on backup-state change. |
| Metadata service | AAGUID allowlist refresh is split to IP-006. | FIDO MDS3 refresh worker validates trust roots and pack-tier policy before regulated-pack promotion. |
| Replay defense | Sign-count monotonic-increase check rejects cloned/replayed assertions. | Existing kernel test plus adapter-level assertion replay test stay green. |
| Audit | IP-012 audit emitter is the identity audit anchor. | Every register/authenticate/revoke success and denial emits tenant-attributed audit event. |
| Multi-tenancy | TenantId binds challenges and credentials; tenant mismatch is a typed error. | Cross-tenant credential read/authentication tests return 403/deny without leaking existence. |

## OpenSK reference tracking

| Item | Current state | Gate |
|---|---|---|
| Vendored reference directory | `tools/opensk-vendored/README.md` exists and records the upstream, Apache-2.0 license, and deferred subtree command. | Before claiming vendored source, the follow-up IP runs `git subtree add --prefix tools/opensk-vendored/src https://github.com/google/OpenSK main --squash` and records upstream commit + license scan evidence. |
| Firmware service stub | `oya/oya-authn-device-firmware/catalog.yaml` binds ADR-0508 and ADR-0507. | Before firmware claim, a crate builds the nRF52840 reference target and records reproducible-build evidence. |
| Authenticator/RP loop | ADR-0507 + ADR-0508 pair webauthn-rs RP with OpenSK authenticator reference. | Virtual or hardware ceremony evidence proves OpenSK-issued assertion validates through the RP adapter before closed-loop claim. |
| Bespoke `oya-authn-device` cutover | ADR-0508 parity table is the Tier-3 bar. | FIDO2/CTAP2.1 parity, transport, attestation, resident-key, update, manufacturing, and lifecycle rows are independently green. |

## Promotion and cutover gates

### Phase-1 admission gate

A PR may claim this bridge slice is tracked when all are true:

1. This IP exists and cites ADR-0476, ADR-0506, ADR-0507, and ADR-0508.
2. The Phase-1 surface table names each identity/authn surface and current repo anchor.
3. The aws-lc-rs provider table names the no-ring enforcement path and identity verifier anchor.
4. The webauthn-rs parity table maps every ADR-0507 row to an Oyatie surface and cutover gate.
5. The OpenSK table records the vendored-reference directory, source-vendoring non-claim, and firmware stub gate.
6. `git diff --check` and targeted content validation pass.

### Bridge retirement gate

Do not retire the current Phase-1 IdP bridge or claim `oya-identity` live parity until:

1. OIDC/OAuth2, JWKS verification, WebAuthn, MFA/ACR, SCIM, federation, Cedar, audit, and multi-region session tests pass against the same tenant/pack fixtures.
2. Traffic shadowing shows no token, ceremony, SCIM, or federation regressions.
3. Audit-chain completeness remains 1.0 across success and denial events.
4. Rollback to the bridge is documented and tested.
5. `oya-ci-required` is green on the protected PR that promotes the cutover evidence.

### Bespoke provider cutover gates

| Bridge | Bespoke destination | Cutover authority |
|---|---|---|
| aws-lc-rs | oya-crypto | ADR-0506: kubers Phase-B proofs, oya-crypto FIPS 140-3 validation, and security-isolation gate evidence. |
| webauthn-rs | oya-webauthn | ADR-0507: parity table green, oya-identity Phase-2 gate, tenant opt-in period with no ceremony-success regression. |
| OpenSK | oya-authn-device | ADR-0508: Tier-3 parity table green, manufacturing run validated, OpenTitan path verified for Tier-4, hardware budget approved. |

## Non-goals for IDENTITY-003

- No new public API route or runtime process.
- No Cargo dependency change and no `Cargo.lock` churn.
- No OpenSK source subtree in this slice.
- No hardware provisioning workflow beyond tracking the reference and gates.
- No claim that the current Zitadel/Keycloak-era bridge is already retired.

## Acceptance evidence

Run these checks from the PR worktree:

```bash
python3 -m json.tool specs/root-hub-pointers.json >/dev/null
python3 - <<'PY'
from pathlib import Path
p = Path('oya/identity/IP-017-bespoke-identity-authn-crypto-bridge.md')
text = p.read_text()
required = [
    'ADR-0476', 'ADR-0506', 'ADR-0507', 'ADR-0508',
    'aws-lc-rs', 'webauthn-rs', 'OpenSK',
    'Phase-1 admission gate', 'Bridge retirement gate',
    'Bespoke provider cutover gates',
]
missing = [item for item in required if item not in text]
if missing:
    raise SystemExit(f'missing required tracking anchors: {missing}')
PY
git diff --check
```
