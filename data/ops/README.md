# DATA-005 data substrate ops skeleton

This directory is the DATA-005 contract-only ops surface for the persistence substrate (`G003-g03-persistence-substrate-oya-data`). It contains static operator and GitOps manifests plus the paired OpenSLO entry under `data/observability/slos/data-substrate/`.

Scope and non-claims:

- The operator manifests are static Kubernetes skeletons for review and admission checks only.
- The Deployment is intentionally scaled to zero and uses a contract-only image reference; no controller runtime, database migration, Argo CD sync, or production/tenant workload capability is asserted here.
- The GitOps Application omits automated sync and is labelled as a static skeleton so ADR-0139 promotion remains blocked until evidence is attached.
- The OpenSLO file declares the evidence shape for future reconciliation latency measurement; it is not a measured SLO window.

Owned artifacts in this slice:

- `operator/`: namespace, service account, namespace-scoped RBAC, contract ConfigMap, and zero-replica operator Deployment skeleton.
- `gitops/`: Argo CD Application skeleton pointing at `data/ops/operator` without automated sync.
- `../observability/slos/data-substrate/operator-reconciliation-latency.openslo.yaml`: OpenSLO skeleton for ADR-0139 evidence-required gating.
