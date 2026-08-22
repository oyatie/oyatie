# Implementation-ready acceptance criteria — `managed-k8s-control-plane-host`

Each criterion is verifiable against the code in this lane (kernel + api +
adapters + app) or against the named follow-on. ✅ = covered + tested in this lane.

## Kernel (state machine)

- ✅ **AC-K1** `ControlPlaneTier` and `DatastoreClass` round-trip through
  `as_str`/`parse`; unknown slugs parse to `None` (fail-closed). The product
  default tier is `HostedKamaji`.
- ✅ **AC-K2** The hosted happy path (`requested → datastore_bound → provisioning
  → endpoint_ready → active → draining → deleted`) is legal; the dedicated happy
  path (`… → media_formed → …`) is legal.
- ✅ **AC-K3** `requested` cannot jump straight to `active`/`provisioning`; an
  illegal `transition` returns `IllegalTransition` (never panics).
- ✅ **AC-K4** Any non-terminal status can transition to `failed`; terminal
  statuses (`deleted`, `failed`) have no outgoing transition.

## Port + adapters

- ✅ **AC-P1** The `ControlPlaneProvisioning` port is object-safe
  (`Arc<dyn ControlPlaneProvisioning>`) and exposes `provision`/`status`/`teardown`.
- ✅ **AC-P2** The in-memory adapter provisions BOTH tiers to `active` (with an
  endpoint), and `teardown` drives `active → draining → deleted` and is idempotent.
- ✅ **AC-P3** A malformed `ClusterRef` is rejected (`InvalidClusterRef`); an
  unknown handle `status` is `NotFound`.
- ✅ **AC-P4** The CAPI adapter returns
  `Unimplemented(KamajiProviderLiveIntegration)` from every port method (no fake
  success); it still validates the cluster ref before reporting the boundary.

## App (composition root)

- ✅ **AC-A1** `POST /admin/control-planes` returns 201 + the handle on success;
  400 on malformed ref / unknown tier; 501 when backed by the (deferred) CAPI
  adapter.
- ✅ **AC-A2** `POST /admin/control-planes/status` returns 200 + status (`active`
  after provision; `deleted` after teardown via the in-memory adapter).
- ✅ **AC-A3** `POST /admin/control-planes/teardown` returns 204 (idempotent).
- ✅ **AC-A4** Tier defaults to `hosted_kamaji` when omitted (ADR-0376 default).
- ✅ **AC-A5** `GET /healthz` returns 200 `ok`.
- ✅ **AC-A6** The production `[[bin]]` fails closed without `$OYATIE_MGMT_KUBECONFIG`
  (`BootError::MissingMgmtKubeconfig`); it never falls back to the in-memory fake.

## Registry / governance

- ✅ **AC-R1** All five crates are root `[workspace].members`; each has a
  `registry/catalog/*.yaml` record (passes architecture-boundaries).
- ✅ **AC-R2** `kube`/`kube-runtime`/`k8s-openapi` are in the dependency-blessed
  allowlist + rationales (adapter-only seam) and isolated to `-adapter-capi`
  (passes dependency-seam).
- ✅ **AC-R3** Cedar default-deny for tenants + permit only platform/control-plane
  operators (`cedar/policies.cedar`).
- ✅ **AC-R4** The deferred Kamaji reconcile is tracked at
  `registry/placeholder-debt/adr-follow-ups.yaml#kamaji-provider-live-integration`
  (passes honest-claims / design-spec-maturity).

## Deferred (named follow-ons — NOT this lane)

- **AC-D1** Live Kamaji `TenantControlPlane` / Talos control-plane reconcile +
  NetworkPolicy/datastore isolation enforcement (`kamaji-provider-live-integration`).
- **AC-D2** Billing/SLA/DPIA/external GA (`managed-k8s-commercial-ga`).

## Verification commands (this lane, targeted)

```
cargo fmt -p managed-k8s-control-plane-host-{kernel,api,adapter-capi,adapter-inmemory,app} --check
cargo clippy -p managed-k8s-control-plane-host-{kernel,api,adapter-capi,adapter-inmemory,app} --all-targets -- -D warnings
cargo nextest run -p managed-k8s-control-plane-host-{kernel,api,adapter-capi,adapter-inmemory,app}
./bin/oya gate validate architecture-boundaries
./bin/oya gate validate dependency-seam
./bin/oya gate validate data-class
./bin/oya gate validate honest-claims
./bin/oya gate validate cedar-fragment-coverage
./bin/oya gate validate design-spec-maturity-claims
./bin/oya gate validate cross-tenant-access-fuzz
```
