# Runbook — control-plane provision / teardown

**Service:** `managed-k8s-control-plane-host` (ADR-0376). non_claim: this
runbook describes the operator flow for the design-spec surface; the live
reconcile steps activate with `kamaji-provider-live-integration`.

## Preconditions

- The service is running in the MANAGEMENT cluster with `$OYATIE_MGMT_KUBECONFIG`
  set to the management-cluster kubeconfig (the `[[bin]]` fails closed otherwise).
- The caller is a platform / control-plane-operator principal (Cedar default-deny
  for tenants).

## Provision a tenant control plane

1. Decide the tier with the tenant: `hosted_kamaji` (default, dense) or
   `dedicated_talos_spoke` (sovereign/premium). For hosted, decide the
   `datastore_class` (`etcd_per_tenant` = strongest isolation; `pooled_relational`
   = denser).
2. Call `POST /admin/control-planes` with `{tenant_id, cluster_name, tier,
   datastore_class}`.
   - **201** → record the returned `handle` (needed for status/teardown).
   - **400** → malformed ref or unknown tier/datastore; fix the request.
   - **501** → the live CAPI reconcile is not yet wired
     (`kamaji-provider-live-integration`); the request is honestly rejected, not
     silently dropped. Until the follow-on lands, provisioning is exercised via
     the in-memory adapter only.
3. Poll `POST /admin/control-planes/status` with `{tenant_id, cluster_name, tier,
   handle}` until `status == active` and `endpoint` is populated. Expect
   seconds for hosted, minutes for dedicated.
4. Verify the audit chain recorded `tier-selected` → `datastore-bound` (hosted) →
   `provisioned`.

## Tear down a tenant control plane

1. Call `POST /admin/control-planes/teardown` with `{tenant_id, cluster_name,
   tier, handle}`.
   - **204** → accepted; the control plane drains then deletes. Idempotent —
     re-issuing on an already-deleted control plane is also a 204/no-op.
2. Poll status until `deleted`.
3. Verify the audit chain recorded `torn-down`.

## Failure handling

- **Management cluster unreachable (502 backend):** check management-cluster
  health; the service does NOT retry unboundedly (circuit-breaker posture). Do not
  point the service at a tenant cluster — it only ever talks to the management
  cluster (operational boundary).
- **501 on every call:** expected until `kamaji-provider-live-integration` lands.
  Track the follow-on ADR; do not attempt to force a provision.
- **Boot crash with `MissingMgmtKubeconfig`:** set `$OYATIE_MGMT_KUBECONFIG` to the
  management kubeconfig path and restart. Never work around the guard by switching
  to the in-memory build in production.
