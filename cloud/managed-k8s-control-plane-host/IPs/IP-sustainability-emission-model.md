# IP: Managed-K8s control-plane-host emission model hold

This card intentionally does not add `sustainability_emission_model` to `cloud/managed-k8s-control-plane-host/manifest.json`.

The service has real control-plane-host authority and audit rows for tier selection, datastore binding, provisioned, and torn-down states. However, the current manifest still lacks the ADR-0340 `capacity_model` and `pod_runtime_tier` fields required by the ADR-0344 schema. Authoring power coefficients before those fields are source-authorized would convert an inventory gap into a false FinOps readiness signal.

Required follow-up before declaration:
1. Add or review a service-owned capacity model for hosted Kamaji and dedicated Talos control-plane operations.
2. Bind the pod runtime tier to the management-cluster-only execution boundary.
3. Decide whether provider/SKU pricing tracks hosted control-plane pod minutes, dedicated Talos spoke references, or both.
4. Add a deterministic fixture that maps provision/status/teardown audit rows to watt-hours without creating a live Kubernetes or provider mutation claim.

This hold preserves the non-claims in the existing foundation IP: no real sandbox/live management-cluster execution proof, no billing/SLA/DPIA/external GA claim, no provider-live provisioning, no OpenCost/FOCUS export, no regulator evidence pack, and no live FinOps/cost-readiness claim in this lane.
