# cloud-iac desired state

Active cloud-iac desired state is Rust/Buck2-validated OpenTofu plus CUE/KRM packages reconciled by the native release conveyor and guarded by Rust/Prow `oya-ci-required`.

Retired active paths in this substrate:

- `iac/cue-krm-packages/**`
- `iac/k8s/helm/**`
- provider-local `argocd/**` modules
- provider-local `jenkins/**` modules

Helm, ArgoCD, and Jenkins may appear only as historical reference or external compatibility input after policy-critical state has been normalized through CUE/KRM and the native CI/CD seams.
