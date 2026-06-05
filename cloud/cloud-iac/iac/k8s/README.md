# cloud-iac Kubernetes desired state

First-party Kubernetes desired state is CUE/KRM plus Buck2 validation and trusted Rust/Prow `oya-ci-required` evidence. Helm is not a first-party authority here; use it only as an external chart compatibility adapter after normalizing policy-critical state into CUE/KRM.
