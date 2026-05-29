---
doc_class: SdkPlan
template_id: TPL-SDK-PLAN
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + axis-developer-experience
related_adrs: [ADR-0145, ADR-0170, ADR-0187]
---

# SDK Plan — identity µservice

Client SDKs for the 5 canonical stacks. Each SDK wraps the per-pack OIDC + WebAuthn + SCIM endpoints + step-up flow. All SDKs:

- Verify JWTs locally using cached JWKS (24h TTL; auto-refresh on `kid` miss).
- Honour `iss`, `aud`, `exp`, `nbf`, `tenant_id`, `acr`, `purpose`, `data_class` claims.
- Implement PKCE for public clients (RFC 7636).
- Implement device authorization grant (RFC 8628) for headless / CLI flows.
- Surface step-up redirects via a callback the host application implements.
- Never log raw tokens (canonical `Secret<Token>` wrapper).

## SDK-1 — Rust (`oya-shared-oidc-client-kernel` consumer SDK)

Surface: `OidcClient` trait already in `oya-shared-oidc-client-kernel`.

Additional packages:
- `oya-sdk-identity-axum` — `axum::middleware` that extracts bearer, verifies, attaches `OidcClaims` to request extensions.
- `oya-sdk-identity-tower` — `tower::Layer` for tonic gRPC services.
- `oya-sdk-identity-reqwest` — auto-injects bearer into outbound calls; refreshes JWKS on 401.

Maturity year-1: GA. Used by every oyatie µservice.

## SDK-2 — TypeScript (`@oyatie/identity-sdk`)

Targets browser + Node.js. Browser flow includes WebAuthn ceremony via `navigator.credentials`.

API:
```ts
import { IdentitySdk } from '@oyatie/identity-sdk';

const sdk = new IdentitySdk({ packEndpoint: 'https://identity-eu.oyatie.com' });
const token = await sdk.signInWithPasskey({ mediation: 'conditional' });
const me = sdk.verify(token); // { tenantId, acr, ... }
await sdk.stepUp('sensitive', '/secrets/rotate');
```

Browser:
- Uses WebAuthn `navigator.credentials.get({ mediation: 'conditional' })` for autofill.
- Stores ID-token in `httpOnly` cookie (SameSite=Strict; Secure; per-pack domain).
- Refresh-token rotation via `BroadcastChannel` to coordinate cross-tab.

Node.js:
- Service-account device-code flow.
- Token caching via OS keychain integration (libsecret / Keychain / Credential Manager).

Maturity year-1: GA. Workflow Studio SPA + Ops Portal consume it.

## SDK-3 — Swift (`OyatieIdentity` SPM package)

Targets iOS 16+ / macOS 13+ (Passkey API minimum).

API:
```swift
import OyatieIdentity

let sdk = IdentitySdk(packEndpoint: URL(string: "https://identity-eu.oyatie.com")!)
let token = try await sdk.signInWithPasskey()
let claims = try sdk.verify(token: token)
try await sdk.stepUp(required: .sensitive, returnTo: URL(...))
```

- AuthenticationServices framework integration for Passkey (`ASAuthorizationPlatformPublicKeyCredentialProvider`).
- Cross-device sign-in (caBLE) via `ASAuthorizationSecurityKeyPublicKeyCredentialProvider`.
- Refresh-token in iOS Keychain.

Maturity year-2: GA. Mobile shell µservice consumer.

## SDK-4 — Kotlin (`com.oyatie:identity-sdk`)

Targets Android 13+ + JVM.

API:
```kotlin
val sdk = IdentitySdk(packEndpoint = "https://identity-eu.oyatie.com")
val token = sdk.signInWithPasskey()
val claims = sdk.verify(token)
sdk.stepUp(StepUpAcr.SENSITIVE, returnTo = "..."")
```

- Android: CredentialManager API (Android 13+) for Passkey.
- JVM: Used by enterprise Java consumers; device-code flow.
- Refresh-token in Android Keystore / JVM via `jks` + OS-encrypted store.

Maturity year-2: GA.

## SDK-5 — C# (.NET 8+) (`Oyatie.Identity` NuGet)

Targets ASP.NET Core + Windows desktop.

API:
```csharp
var sdk = new IdentitySdk(packEndpoint: "https://identity-eu.oyatie.com");
var token = await sdk.SignInWithPasskeyAsync();
var claims = sdk.Verify(token);
await sdk.StepUpAsync(StepUpAcr.Sensitive, returnTo: "...");
```

- ASP.NET Core middleware adapter: `AddOyatieIdentity()` + `[Authorize(Acr = "sensitive")]` attribute.
- Windows: WebAuthn API via Microsoft.Identity.Client interop.
- Credentials in DPAPI on Windows; gnome-keyring on Linux .NET.

Maturity year-3: Preview → GA after pen test.

## Cross-SDK contracts

All SDKs implement:

- `signInWithPasskey(opts)` → returns ID-token + refresh-token.
- `verify(token)` → returns `OidcClaims` or throws.
- `stepUp(requiredAcr, returnTo)` → orchestrates redirect / device-code flow.
- `revokeSession()` → server-side revoke + client-side credential clear.
- `meetsAcr(claims, floor)` → boolean.

All SDKs MUST pass the conformance test set `evidence/sdk-conformance/identity-v1.json`.

## Versioning

Per ADR-0177 internal-external-api-surface-separation:

- SDK semver tracks the OpenAPI surface version, NOT this µservice's internal version.
- Major version bump requires migration guide.
- Backwards compatibility: SDK N can talk to server N or N-1; server can serve SDK N, N-1, or N-2.

## Distribution

| SDK | Registry | Cadence |
|---|---|---|
| Rust | private crates registry (per ADR-0170 dev-portal) | per-merge to main |
| TypeScript | private npm registry | per-merge to main |
| Swift | private SPM | weekly tag |
| Kotlin | private Maven Central mirror | weekly tag |
| C# | private NuGet | weekly tag |

## Deprecation lane

Per ADR-0173-vendor-lock-in §"open-source SDK exit optionality": every SDK is OSS-licensed (Apache-2.0) and the source published under `oyatie/oya-identity-sdks-*` repositories. Customers retain the right to fork.
