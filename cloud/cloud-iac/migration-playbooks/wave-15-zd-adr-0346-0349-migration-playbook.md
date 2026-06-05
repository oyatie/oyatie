# Cloud IaC Active Substrate Migration Playbook

Status: active authority cleanup for the Buck2/Prow/native release-conveyor direction.

Current direction:

1. Buck2 target output plus the trusted Rust/Prow `oya-ci-required` context is merge authority.
2. Native release-conveyor reconciliation over CUE/KRM desired state is delivery authority.
3. GitHub/GitHub Actions are temporary publication and shadow-CI adapters only.
4. Jenkins, ArgoCD, first-party Helm charts, retired local `oya` gate/dev CLI commands, and Cargo command evidence are not active cloud-iac authority.

Migration steps:

1. Keep cloud-iac desired state in Rust/Buck2-validated OpenTofu and CUE/KRM packages.
2. Enter changes through isolated GitHub PR lanes against `dev`.
3. Require green `oya-ci-required`/Buck2/Prow evidence before merge.
4. Preserve only reference value from retired bridge artifacts, then delete active Jenkins/ArgoCD/Helm surfaces once native seams cover the capability.
5. Record evidence under `evidence/multispectrum/` and keep guards in Rust/Buck2, not ad-hoc shell or Python.

Rollback:

- Revert the PR through GitHub if the native guard rejects a false positive.
- Do not restore Jenkins, ArgoCD, first-party Helm, retired local `oya` CLI gates, or Cargo command authority as a rollback path.
