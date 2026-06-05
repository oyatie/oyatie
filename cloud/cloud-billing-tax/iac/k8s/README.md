# cloud-billing-tax Kubernetes desired state

First-party desired state for this substrate is CUE/KRM plus Buck2 validation and the trusted Rust/Prow `oya-ci-required` context. Helm is not a first-party authority here; use Helm only as an external chart compatibility adapter after normalizing policy-critical state through CUE/KRM.

The retired local Helm chart directory was removed to prevent agents from reintroducing ArgoCD/Jenkins-era deployment authority. Keep service-owned Kubernetes YAML here only as generated or compatibility evidence until the CUE package lands.
