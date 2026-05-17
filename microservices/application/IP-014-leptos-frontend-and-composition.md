---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-014-leptos-frontend-and-composition
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-build, cargo-nextest, lean-a1, composition-root-only, wasm-build]
---

# IP-014: Leptos frontend + composition-root binaries

## Intent

- Leptos WASM frontend crate `oya-application-shell-frontend` containing the
  shell UI (app-switcher, login page, admin portal scaffold, module-host
  iframe wrapper, Cedar-gated navigation).
- Composition-root binaries (one per BC) per ADR-0105 amendment:
  `oya-application-shell-routing-app`, `oya-application-tenant-context-app`,
  `oya-application-auth-gateway-app`, `oya-application-module-loader-app`,
  `oya-application-frontend-bundle-serve-app`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-shell-frontend/{Cargo.toml,src/{lib,app,routes,components/*}.rs}` | create — Leptos crate, `cdylib` + WASM |
| `microservices/application/src/crates/oya-application-shell-routing-app/{Cargo.toml,src/main.rs}` | create — composition root |
| `microservices/application/src/crates/oya-application-tenant-context-app/{Cargo.toml,src/main.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-app/{Cargo.toml,src/main.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-app/{Cargo.toml,src/main.rs}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-app/{Cargo.toml,src/main.rs}` | create |
| `microservices/application/src/crates/oya-application-shell-routing-sdk/{Cargo.toml,src/lib.rs}` | create — route registration SDK |
| 7 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// Leptos shell frontend (excerpt)
#[component]
pub fn AppShell(cx: Scope) -> impl IntoView {
    let session = use_context::<Session>(cx).expect("session in context");
    view! { cx,
        <Suspense fallback=move || view! { cx, <ShellSkeleton /> }>
            <AppSwitcher tenant_id=session.tenant_id />
            <Router>
                <Routes>
                    <Route path="/" view=Home />
                    <Route path="/admin/*any" view=AdminPortal />
                    <Route path="/:module/*rest" view=ModuleHost />
                </Routes>
            </Router>
        </Suspense>
    }
}

// composition root (shell-routing-app)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load_from_env_and_openbao().await?;
    let registry = oya_application_shell_routing_adapter::PostgresRouteRegistry::new(cfg.pg.clone());
    let resolve = oya_application_shell_routing_usecase::ResolveRouteUseCase::new(registry);
    oya_application_shell_routing_rest::serve(resolve, cfg.rest).await
}
```

## Acceptance Gates

```bash
cargo build --target wasm32-unknown-unknown -p oya-application-shell-frontend --release
cargo build --release -p oya-application-shell-routing-app
cargo build --release -p oya-application-auth-gateway-app
cargo build --release -p oya-application-tenant-context-app
cargo build --release -p oya-application-module-loader-app
cargo build --release -p oya-application-frontend-bundle-serve-app
cargo run -p oya-dev-cli -- gate validate composition-root-only --microservice application
cargo run -p oya-dev-cli -- gate validate wasm-bundle-size --crate oya-application-shell-frontend
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_compose_and_drop` | smoke; no panics |
| `test_main_with_failing_dependency` | mocked failing Postgres → app exits non-zero |
| `test_wasm_bundle_under_2mb_gzip` | bundle-size budget |
| `test_lighthouse_tti_under_2s` | synthetic Lighthouse |
| `test_csp_header_set` | runtime probe |
| `test_sri_in_shell_html` | hash in `<link>` |

Coverage: 60 % for app; 80 % for frontend components.

## Next IP

[`IP-015-application-openslo-and-hg-app.md`](IP-015-application-openslo-and-hg-app.md)
