---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-014-per-pack-iac-overlays
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-pack-routing-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Per-pack Kustomize overlays for cloud-iac substrate

## Intent

Author per-pack Kustomize overlays for the cloud-iac substrate. pack-kr is live (M01 launch). pack-eu / pack-us / pack-us-healthcare / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa are conditional (activated when first-tenant in pack signs DPA).

## ChangeSet boundary

11 per-pack overlay manifests under `iac/iac/kustomize/overlays/`. Per-pack secret references, KMS keyrings, retention overrides, region pinning.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/iac/kustomize/overlays/pack-kr/kustomization.yaml` | already created in IP-001; verified live |
| `iac/iac/kustomize/overlays/pack-eu/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-us/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-jp/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-sg/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-au/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-in/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-br/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-ae/kustomization.yaml` | create |
| `iac/iac/kustomize/overlays/pack-ksa/kustomization.yaml` | create |

## Code Shape

```yaml
# overlays/pack-eu/kustomization.yaml (representative)
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

bases:
  - ../../base

patches:
  - patch: |-
      - op: replace
        path: /spec/values/argo-cd/server/config/url
        value: "https://argocd-eu.oyatie.dev"
    target:
      kind: HelmRelease
      name: argocd
  - patch: |-
      - op: replace
        path: /spec/values/opentofu/state/backend
        value: s3
    target:
      kind: HelmRelease
      name: opentofu

configMapGenerator:
  - name: pack-residency
    literals:
      - PACK=pack-eu
      - REGION=eu-frankfurt-1
      - DR_REGION=eu-amsterdam-1
      - JURISDICTION=eu
      - OCI_OBJECTSTORAGE_ENDPOINT=https://objectstorage.eu-frankfurt-1.oraclecloud.com
      - KMS_KEY_RING=cloud-iac-eu-keyring
      - APPLY_AUDIT_RETENTION_YEARS=2
```

```yaml
# overlays/pack-us-healthcare/kustomization.yaml (HIPAA overrides)
configMapGenerator:
  - name: pack-residency
    literals:
      - PACK=pack-us-healthcare
      - REGION=us-ashburn-1
      - DR_REGION=us-phoenix-1
      - JURISDICTION=us-hc
      - APPLY_AUDIT_RETENTION_YEARS=6  # HIPAA §164.316(b)(2)
      - HIPAA_ELIGIBLE=true
```

## Acceptance Gates

```bash
for pack in pack-kr pack-eu pack-us pack-us-healthcare pack-jp pack-sg pack-au pack-in pack-br pack-ae pack-ksa; do
  kubectl --dry-run=client apply -k iac/iac/kustomize/overlays/$pack
done
cloud-ci/oya-ci governance gate `pack-routing-conformance` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `retention-conformance` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

- IaC class: per-pack kustomize render smoke; per-pack helm-install smoke against ephemeral kind cluster.
- Cross-pack: assert pack-kr overlay does NOT reference pack-eu resources (per residency contract).

## Halt Conditions

- Cross-pack resource reference in an overlay — refuse per residency.
- Pack overlay missing required HIPAA retention setting (pack-us-healthcare) — refuse.

## Next IP

[`IP-015-hg-cloud-iac-registration.md`](IP-015-hg-cloud-iac-registration.md)

## References

- ADR-0117; ADR-0131.
- `iac/policy/data-residency.md`.
- `iac/multi-region.md`.

## DR posture (per ADR-0343)

- Target source: `iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` with drill cadence `quarterly`.
- RTO/RPO target: RTO p99 <= `3600` seconds; RPO p99 <= `300` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `true`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/seaweedfs-volume-failover.md`, `iac/manifest.json`, `iac/IP-014-per-pack-iac-overlays.md`.
