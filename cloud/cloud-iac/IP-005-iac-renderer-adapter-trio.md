---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-005-iac-renderer-adapter-trio
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: iac-renderer backend-qualified adapter trio (helm, kustomize, opentofu)

## Intent

Implement the three backend-qualified adapter crates per ADR-0105 Amendment 3 `*-adapter-<backend>` pattern:
- `-adapter-helm`: Helm CLI / SDK wrapper for chart rendering.
- `-adapter-kustomize`: kustomize binary wrapper for overlay resolution.
- `-adapter-opentofu`: OpenTofu CLI wrapper for plan computation.

Plus the generic `-adapter` crate for protocol-neutral port impls (event emitter, etc.).

## ChangeSet boundary

Four new crates per ADR-0105: `-adapter`, `-adapter-helm`, `-adapter-kustomize`, `-adapter-opentofu`. Catalog rows added.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-adapter/{Cargo.toml,src/lib.rs,src/event_emitter.rs}` | create |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-adapter-helm/{Cargo.toml,src/lib.rs,src/helm_client.rs}` | create |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-adapter-kustomize/{Cargo.toml,src/lib.rs,src/kustomize_client.rs}` | create |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-adapter-opentofu/{Cargo.toml,src/lib.rs,src/opentofu_client.rs}` | create |
| `microservices/cloud-iac/catalog/oya-cloud-iac-iac-renderer-adapter*.yaml` | create (4 rows) |

## Code Shape

```rust
// adapter-helm/src/helm_client.rs
pub struct HelmAdapter {
    helm_binary_path: PathBuf,
    cosign_verifier: Arc<dyn CosignVerifier>,
}

#[async_trait]
impl ChartSourceReader for HelmAdapter {
    async fn read(&self, microservice: &str, sha: &str) -> Result<Vec<ChartSource>, RenderError> {
        // Walk microservices/<ms>/iac/helm/* at the given SHA
        // For each Chart.yaml: parse, verify Cosign signature on dependencies, compute digest
        ...
    }
}
```

```rust
// adapter-opentofu/src/opentofu_client.rs
pub struct OpenTofuAdapter {
    runner_url: Url,
    auth_token: SecretString,
}

#[async_trait]
impl TerraformPlanComputer for OpenTofuAdapter {
    async fn plan(&self, module: &ModuleSource, env: &Environment) -> Result<TerraformPlan, RenderError> {
        // POST to OpenTofu runner; parse plan output; signed by runner SPIFFE identity
        ...
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cloud-iac-iac-renderer-adapter -p oya-cloud-iac-iac-renderer-adapter-helm -p oya-cloud-iac-iac-renderer-adapter-kustomize -p oya-cloud-iac-iac-renderer-adapter-opentofu --all-features
cargo nextest run -p oya-cloud-iac-iac-renderer-adapter -p oya-cloud-iac-iac-renderer-adapter-helm -p oya-cloud-iac-iac-renderer-adapter-kustomize -p oya-cloud-iac-iac-renderer-adapter-opentofu --all-features
cloud-ci/oya-ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

Per PHASE-01 adapter class: 1 test per port-impl method + ≥ 2 integration against real backend (Helm CLI + kustomize binary + OpenTofu runner containers). Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_helm_adapter_chart_read` | adapter reads chart text + computes digest |
| `test_helm_adapter_cosign_verify` | adapter refuses unsigned chart |
| `test_kustomize_adapter_overlay_merge` | overlay merge result is deterministic |
| `test_opentofu_adapter_plan` | plan request returns parsed TerraformPlan |
| `integration_helm_real_chart` | against `bitnami/postgresql` reference chart |
| `integration_opentofu_real_module` | against in-tree test module |

## Halt Conditions

- Adapter exposes internal Helm/Kustomize/OpenTofu types — must adapt to kernel types.
- Cosign verification skipped or made optional — refuse.

## Next IP

[`IP-006-iac-validator-kernel-domain-usecase.md`](IP-006-iac-validator-kernel-domain-usecase.md)

## References

- ADR-0105 Amendment 3 (`*-adapter-<backend>` pattern).
- Helm CLI — `helm.sh/docs/`.
- kustomize — `kubectl.docs.kubernetes.io/references/kustomize/`.
- OpenTofu CLI — `opentofu.org/docs/cli/`.
- Sigstore Cosign — `docs.sigstore.dev/cosign/`.
