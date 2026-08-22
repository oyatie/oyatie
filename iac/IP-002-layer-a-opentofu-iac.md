---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-002-layer-a-opentofu-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [helm-lint, terraform-validate, governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Layer-A OpenTofu self-hosted runner + per-pack state-bucket IaC

## Intent

Helm chart for OpenTofu self-hosted runner (Terraform-Cloud-equivalent) + per-pack S3 state-bucket OpenTofu config under `microservices/cloud-iac/iac/`. Provides the OpenTofu execution surface used by iac-renderer (`-adapter-opentofu`) and iac-applier.

## ChangeSet boundary

One ChangeSet: 1 Helm chart for OpenTofu runner + 1 OpenTofu module for per-pack state buckets + Kustomize base patches.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-iac/iac/helm/opentofu/Chart.yaml` | create | OpenTofu runner chart |
| `microservices/cloud-iac/iac/helm/opentofu/values.yaml` | create | LTS-pinned OpenTofu version; HA runner replicas |
| `microservices/cloud-iac/iac/terraform/per-pack-state-bucket.tf` | create | S3-compatible state bucket per pack; SSE-KMS; versioned; WORM |
| `microservices/cloud-iac/iac/terraform/state-bucket-iam.tf` | create | per-pack IAM policy for state-bucket access |
| `microservices/cloud-iac/iac/kustomize/base/kustomization.yaml` | update | add opentofu chart to shared base |

## Code Shape

```yaml
# opentofu/values.yaml
opentofu:
  image:
    registry: docker.io
    repository: opentofu/opentofu
    tag: "1.8.0"  # LTS pin
  runner:
    replicas: 3  # HA min per capacity-model.md
    resources:
      requests: {cpu: "2", memory: "4Gi"}
      limits: {cpu: "4", memory: "8Gi"}
  state:
    backend: s3
    encryption:
      enabled: true
      kms_key_id: "${openbao:secret/cloud-iac/opentofu/kms-key-id}"
```

```hcl
// per-pack-state-bucket.tf
resource "oci_objectstorage_bucket" "cloud_iac_state" {
  for_each = var.active_packs  // {"pack-kr": "ap-seoul-1", ...}

  compartment_id = var.compartment_id
  name           = "cloud-iac-state-${each.key}"
  namespace      = var.namespace

  versioning = "Enabled"
  object_events_enabled = false  // no public bucket events

  retention_rules {  // WORM
    display_name = "compliance-mode"
    duration {
      time_amount = 6
      time_unit   = "YEARS"
    }
  }
}
```

## Acceptance Gates

```bash
helm lint microservices/cloud-iac/iac/helm/opentofu
tofu validate microservices/cloud-iac/iac/terraform/
cloud-ci/ci governance gate `per-microservice-layout` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

- IaC class: helm-install smoke + OpenTofu plan dry-run.
- E2E: spin up kind; apply OpenTofu chart; verify runner Ready; submit a trivial OpenTofu plan; verify state stored in test bucket.

## Halt Conditions

- OpenTofu LTS drift — escalate.
- KMS key reference resolution failure — block.

## Next IP

[`IP-003-iac-renderer-kernel.md`](IP-003-iac-renderer-kernel.md)

## References

- PRD-cloud-iac §"Layer-A".
- ADR-0117.
- OpenTofu deployment — `opentofu.org/docs/`.
- `microservices/cloud-iac/policy/data-residency.md`.
