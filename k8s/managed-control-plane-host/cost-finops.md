# Cost / FinOps — `managed-k8s-control-plane-host`

**Authority:** ADR-0376 (control-plane economics is the deciding force behind the
two-tier model). non_claim: this is the cost MODEL the product layer reasons
about; metering/billing is deferred to `managed-k8s-commercial-ga`.

## The economic thesis (ADR-0376)

The two tiers exist precisely because of control-plane cost:

- **Dedicated (Talos spoke):** each tenant carries a standing
  dedicated-control-plane tax — ~$73/tenant/month for the 3-control-plane-node +
  own-etcd footprint ALONE (ADR-0376). Strongest isolation + only credible
  sovereign/air-gapped story, but does not scale to dense multi-tenant economics.
- **Hosted (Kamaji):** tenant control planes run as PODS in the shared management
  cluster, collapsing the per-tenant standing tax. Density + seconds-scale
  provisioning is the GKE/EKS/OKE commodity model. The marginal cost of one more
  hosted control plane is a slice of management-cluster capacity + its datastore,
  not a fresh 3-node control plane.

## Cost drivers this service influences

| Driver | Hosted | Dedicated |
|--------|--------|-----------|
| Standing control-plane footprint | shared mgmt-cluster slice | 3 dedicated CP nodes + etcd / tenant |
| Datastore | per-tenant etcd OR pooled relational (`DatastoreClass`) | own etcd on CP nodes |
| Provisioning time → time-to-value | seconds | minutes (full spoke bring-up) |
| Management-cluster capacity planning | a standing concern (density) | n/a |

The `DatastoreClass` choice is itself a cost/isolation knob: `EtcdPerTenant`
(stronger isolation, higher per-tenant cost) vs `PooledRelational` (denser,
lower cost, logical separation).

## Deferred

- Metering of managed clusters, the per-tenant cost attribution, and the billing
  components → `managed-k8s-commercial-ga` (ADR-0376). This lane emits NO
  billing components and makes NO pricing claim.
