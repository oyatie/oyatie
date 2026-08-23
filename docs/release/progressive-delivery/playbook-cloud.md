---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cloud-kernel rollouts (KMS / storage / network / billing / observability).
  Blue/green for KMS roots; canary for everything else.
planned_enforcement_ref:
  - governance-canary-required
  - governance-rollback-evidence
related_adrs: [ADR-0028, ADR-0043, ADR-0045, ADR-0049, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: Cloud-Axis Rollout


## 1. Surface

Cloud-axis kernels under `crates/cloud-*` ([ADR-0028](../../decisions/ADR-0028-cloud-provider-architecture.md)).

## 2. Default rail per sub-axis

| Sub-axis | Rail | Rationale |
|---|---|---|
| `cloud-kms-*` | **Blue/green** (mandatory) | KMS root rotation = atomic; HSM-backed; per [ADR-0043](../../decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md) |
| `cloud-storage-*` | Canary (BG for schema) | Per-cell canary; BG when block/object storage backends change |
| `cloud-network-*` | Canary, per-cell, **lockstep across regions** | Cross-region replication topology change → BG |
| `cloud-billing-*` | Canary + dark-launch (write-side) | Per [`dark-launch-spec.md`](dark-launch-spec.md) §2 — billing logic is high-risk |
| `cloud-observability-*` | Canary | OTel collector / metric source — provider-agnostic |
| `cloud-iam-*` | Canary + dark-launch (Cedar policy) | Per [ADR-0007](../../decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md) — auth logic = high-risk |
| `cloud-region-*` / `-cell-*` | Blue/green | Cell topology is stateful |
| `cloud-compute-*` | Canary | Compute fleet rollouts |
| `cloud-dcops-*` | Canary | DCIM software ([ADR-0032](../../decisions/ADR-0032-dcim-software-for-own-dc-ops.md)) |

## 3. KMS root rotation (special)

KMS root rotation runs blue/green with mandatory:

1. **Dual-encrypt window** — new envelope keys wrap both blue + green roots; readers tolerate both.
2. **Cohort-gated cutover** — stable cohort cuts last; per-vertical regulatory pack must approve.
3. **Per-cell HSM attestation** — each cell's HSM signs the new root before traffic-shift.
4. **Soak ≥ 7 days** on `canary-eligible` before stable cohort cut.
5. **Rollback path** — re-shift traffic to blue root; blue retains material ≥ 90 days post-cutover.

Planned advisory lane: `governance-rollback-evidence` (D14 mandate) + an existing KMS-rotation lane.

## 4. Cross-region rollout halting

Per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) §Open-question-Q4: a Sev-2 in region 1 halts rollout to region 2 by default. Cloud-axis playbook enforces this with no override below Sev-1-incident-commander.

## 5. Per-cell rollback unit

Default per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md). Cloud-axis cells correspond 1:1 to `cloud-cell-app` instances.

## 6. SLO targets (cloud-specific)

| Service | SLO target | Window | Notes |
|---|---|---|---|
| KMS unwrap | 99.999% | 30 d | Five nines; critical-path |
| Storage GET | 99.99% | 30 d | |
| Storage PUT | 99.95% | 30 d | |
| IAM auth | 99.99% | 30 d | |
| Billing meter | 99.95% | 30 d | |
| Compute control-plane | 99.95% | 30 d | |

Burn-rate alerts per service per [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md).

## 7. Provider-agnostic posture

All cloud kernels are provider-neutral per [Directive 4](../../../docs/MASTERPLAN.md). Provider-specific behaviour lives in `-adapter-aws-*`, `-adapter-oci-*`, `-adapter-azure-*`, `-adapter-gcp-*` (when those cells onboard). Canary stages a provider adapter independently.

## 8. Hyperscaler equivalents

AWS KMS GenerateDataKey rotation; Google Cloud KMS-Inline; Microsoft Azure Managed HSM; Oracle Vault rotation. All exercise the blue/green pattern; we adopt it identically.

## 9. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — Cloud axis cadence: bi-weekly staging → prod; `security-reviewer` re-affirms at gate 5 for control-plane paths.
