# IP-001 — control-plane-host foundation (kernel + port + adapters + app)

**Acceptance status:** ga (this lane)
**Authority:** ADR-0376; layered per ADR-0105/0131; dependency-seam per ADR-0092;
error-handling Tier-3 per ADR-0083.

## Goal

Land the buildable, tested foundation of `oya-managed-k8s-control-plane-host`:
the pure kernel, the shared `ControlPlaneProvisioning` port, both adapters
(kube-rs CAPI — live reconcile honest-deferred — and the deterministic
in-memory fake), and the composition-root app (axum admin/status API +
fail-closed `[[bin]]` main).

## Changesets (this IP)

1. **kernel** (`oya-managed-k8s-control-plane-host-kernel`) — `ControlPlaneTier`,
   `ControlPlaneStatus` state machine (with `can_transition_to`/`transition`/
   `IllegalTransition`), `DatastoreClass`; all enums `as_str`/`parse`; std+serde;
   `#![forbid(unsafe_code)]`. Exhaustive unit tests for both tier branches +
   illegal transitions + terminal states.
2. **api** (`-api`) — the `ControlPlaneProvisioning` port + DTOs (`ClusterRef`,
   `ProvisionRequest`, `ControlPlaneRef`, `ControlPlaneStatusReport`,
   `ProvisioningError`, `Unimplemented`). Object-safe boxed-future port.
3. **adapter-inmemory** (`-adapter-inmemory`) — deterministic fake driving both
   tiers through the kernel state machine; idempotent teardown.
4. **adapter-capi** (`-adapter-capi`) — kube-rs Client + Kamaji
   `TenantControlPlane` dynamic descriptor; every port method returns
   `Unimplemented::KamajiProviderLiveIntegration` (honest-deferred). kube-rs +
   k8s-openapi isolated here only.
5. **app** (`-app`) — composition root: `build_state_in_memory` /
   `build_state_capi`, `build_router`, `serve`, fail-closed
   `mgmt_kubeconfig_path_from_env`; axum admin/status API; `[[bin]]` main;
   acceptance suite exercising both tiers over a loopback socket.
6. **dependency-seam** — `kube`/`kube-runtime`/`k8s-openapi` added to
   `registry/dependency-blessed-allowlist.json` +
   `registry/dependency-rationales.json` (adapter-only, ADR-0092) + root
   `[workspace.dependencies]`; isolated to `-adapter-capi`.
7. **registry** — per-crate catalog records under `registry/catalog/`; the five
   crates registered in the root `Cargo.toml [workspace].members`.
8. **placeholder-debt** — `kamaji-provider-live-integration` entry added to
   `registry/placeholder-debt/adr-follow-ups.yaml`.

## Out of scope (named follow-ons)

- The live Kamaji `TenantControlPlane` / Talos control-plane CRD reconcile
  (`kamaji-provider-live-integration`).
- Billing / SLA / DPIA / external GA (`oya-managed-k8s-commercial-ga`, per
  ADR-0376).
- The sibling microservices that CONSUME the port (`cluster-lifecycle`,
  `tenant-quota`, `sla-observability`).

## Verification (this lane)

`cargo fmt --check`, `cargo clippy --all-targets -D warnings`, and
`cargo nextest run` over the five crates; plus `oya gate validate` for
architecture-boundaries, dependency-seam, data-class, honest-claims,
cedar-fragment-coverage, design-spec-maturity-claims, and cross-tenant-access-fuzz.
