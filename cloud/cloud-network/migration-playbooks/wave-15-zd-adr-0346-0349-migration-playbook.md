# cloud-network Wave 15-ZD active substrate migration playbook

Audience: an Oyatie operator or migration owner preparing `cloud-network` for parallel lane work without reviving retired local CLI, Jenkins, ArgoCD, or first-party Helm authority.

## Active authority

1. Buck2 is the build/test/check authority for Rust and documentation hygiene.
2. ADR-0513 oya-ci/Prow owns the trusted `oya-ci-required` required context.
3. GitHub/GitHub Actions are temporary PR/publication and shadow-evidence adapters only.
4. First-party Kubernetes desired state is CUE/KRM; Helm is external chart compatibility only after CUE/KRM normalization.
5. Durable promotion is native, release-conveyor-like, and audit-chain emitting.

## Retired authority not to reintroduce

- Local `./bin/oya` verifier, VCS, or gate commands.
- Cargo command snippets as merge, release, or incident evidence.
- Jenkinsfiles, Jenkins controller state, or Jenkins parity lanes.
- ArgoCD Application ownership, Argo sync IDs, or Argo refresh commands.
- Service-owned Helm chart directories as first-party deployment inputs.

## Migration procedure

1. Create one plain-git worktree branch per lane from `github-mirror/dev`.
2. Keep cloud-network edits inside `cloud/cloud-network/**`, the Rust hygiene guard/test, and lane-specific evidence.
3. Replace retired authority language with Buck2/Prow/CUE-KRM/native-promotion wording.
4. Delete service-owned Helm charts; add `iac/k8s/README.md` pointing to CUE/KRM.
5. Run the Rust repo-hygiene guard and `buck2 build //:repo-hygiene-automation-check` before opening a PR.
6. Let GitHub checks prove the temporary adapter path while Prow/Buck2 remains the canonical required context.

## Acceptance evidence

- Rust hygiene test rejects retired Cargo/local-CLI/Jenkins/ArgoCD/Helm authority for `cloud-network`.
- Targeted scans under `cloud/cloud-network` show no active retired authority phrases.
- `cloud/cloud-network/iac/k8s/helm` is absent.
- OpenAPI, OpenSLO, and manifest artifacts parse.
- PR checks pass before merge; post-merge verification repeats from the merge commit.
