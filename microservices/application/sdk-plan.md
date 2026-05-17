---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-application + gtm-customer-success
deciders: axis-application, council-architecture
related_adrs: [ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/contracts/openapi/application.yaml
  - microservices/application/contracts/proto/application.proto
  - microservices/application/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (application µservice)

## Purpose

Product µservices need first-party SDKs to (a) register their routes and
module manifests with the Application Shell, (b) consume the active
session token in their own surfaces, and (c) subscribe to Workflow
events emitted by Application. This document specifies the SDK strategy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (primary; oyatie language) | First-party `oya-application-shell-routing-sdk` and `oya-application-module-loader-sdk` crates | axis-application |
| **TypeScript** | M03 (frontend integration for product modules) | First-party hand-authored (Leptos interop) + OpenAPI types generated; published to npm | axis-application |
| **Python** | M04 (for product backends in Python) | OpenAPI-generated; published to PyPI | axis-application + gtm |
| **Go** | M04 | gRPC-generated; published as go-module | axis-application + gtm |
| **JVM (Kotlin / Java)** | M05 | gRPC-generated; published to Maven Central | axis-application + gtm |
| **C# / .NET** | M05+ | OpenAPI-generated; published to NuGet | axis-application + gtm |

Prioritisation drivers: Rust + TypeScript (the languages oyatie's own
product µservices use); other languages added as tenant demand surfaces.

## Surfaces

### Module registration SDK (Rust)

`oya-application-module-loader-sdk`:

```rust
let client = ApplicationClient::new(opts);
let manifest = ModuleManifest::builder()
    .module("oya-hr-module")
    .version(env!("OYA_BUILD_SHA"))
    .sri_hash(env!("OYA_WASM_SRI"))
    .signer_key_id(env!("OYA_PUBLISHER_KEY_ID"))
    .routes(vec![...])
    .bundle_url(env!("OYA_BUNDLE_URL"))
    .build()?;
let signed = manifest.sign_with_openbao_key("hr-publisher")?;
client.publish_module_manifest(signed).await?;
```

Authentication: SPIFFE workload identity → OIDC service account.

### Route registration SDK (Rust)

`oya-application-shell-routing-sdk`:

```rust
let client = ApplicationClient::new(opts);
client.register_routes(vec![
    RouteRegistration {
        path: "/hr/dashboard",
        tenant_scope: TenantScope::TenantScoped,
        required_roles: vec!["employee", "hr-viewer"],
        pack_residency: PackResidency::InheritFromTenant,
        admin_scope: false,
        required_mfa: MfaFactor::None,
        csp_module_id: "oya-hr-module",
    },
    // ...
]).await?;
```

### Session-read SDK (TypeScript)

Product modules running inside the shell iframe can read the active
session via postMessage RPC; the SDK wraps this:

```typescript
import { ApplicationShellClient } from "@oyatie/application-shell-sdk";
const client = new ApplicationShellClient();
const session = await client.getSession();
// session: { user_id, tenant_id, mfa_factor, expires_at }
```

### Event subscription SDK (Rust)

Product µservices that need to react to session lifecycle:

```rust
let client = ApplicationClient::new(opts);
let mut stream = client.stream_session_events(SessionEventFilter {
    tenant_id: Some(tenant),
    event_kinds: vec![SessionEventKind::Ended],
}).await?;
while let Some(event) = stream.next().await {
    // handle SessionEnded for user provisioning teardown, etc.
}
```

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/application/src/crates/oya-application-*-sdk/`.

- Public surface: ergonomic builder + async client.
- Authentication: workload-identity via `oya-tenancy-sdk` re-export.
- Retry: exponential backoff for transient 5xx and 429.
- Streaming: gRPC server-streaming via tonic.
- Re-exports kernel types from `oya-application-*-kernel` for type
  consistency.
- `#![deny(unsafe_code)]`.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Pipeline in `microservices/application/sdk-generation/`:

1. Source of truth: `contracts/openapi/application.yaml` +
   `contracts/proto/application.proto`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific
   generator profile.
3. gRPC → language: `protoc` + language-specific plugin.
4. Wrappers: each language ships a thin ergonomic wrapper around generated
   client (retry, auth-injection, tenant header).

### Versioning

- SDK major version pinned to API major version (1.x SDK ↔ 1.x API).
- Deprecation policy: 6-month notice + dual-stack support; sunset emitted
  via OpenAPI `deprecated: true` + SDK runtime warning.

## Sunset policy

- Any SDK language < 1 % usage after 12 months: sunset notice published;
  6-month support window; archived.
- Final-version SDK kept available for security patches only.

## References

- ADR-0123 cross-product auth.
- `microservices/observability/sdk-plan.md` (precedent).
