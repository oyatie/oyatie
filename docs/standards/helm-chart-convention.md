# Helm chart convention (canonical)

Authority: ADR-0064 canonical-base + localization-packs (SWEEP-I Slice 2).
Library: `microservices/governance/iac/helm/_oyatie-helpers/`.

## Every µservice depends on `_oyatie-helpers` (library chart)

Every per-µservice `Chart.yaml` MUST declare:

```yaml
apiVersion: v2
name: oyatie-<microservice>-<chart>
type: application
version: 0.1.0
appVersion: "1.0.0"
dependencies:
  - name: helpers
    version: 0.1.0
    repository: "file://../../../../governance/iac/helm/_oyatie-helpers"
```

`helm dep update` resolves the library before `helm template` or `helm
install`.

## Every template uses helpers (no boilerplate per-µservice)

Per-µservice templates MUST consume canonical helpers via `include`:

| Helper | Use site |
|---|---|
| `oyatie.labels` | `metadata.labels` on every resource |
| `oyatie.selectorLabels` | `spec.selector.matchLabels` on Deployment/Service |
| `oyatie.serviceAccountName` | `spec.template.spec.serviceAccountName` on Deployment |
| `oyatie.securityContext.restricted` | container `securityContext` |
| `oyatie.networkPolicy.defaultDeny` | `spec` body of NetworkPolicy when no ingress/egress whitelist applies |
| `oyatie.networkPolicy.allowEgressToSubstrate` | first egress entries on every NetworkPolicy |
| `oyatie.prometheusRule.perSloBurnRate` | `groups.[].rules` of PrometheusRule |
| `oyatie.probes.standardLiveness` | `livenessProbe` on container |
| `oyatie.probes.standardReadiness` | `readinessProbe` on container |
| `oyatie.envFromOpenBao` | rendered env-var block over `.Values.secrets` |
| `oyatie.resourceRequests.tier{xs,s,m,l,xl}` | `resources` on container |
| `oyatie.runtimeClassName.gvisor` | pod spec for sandboxed workloads |

## What stays per-µservice (values.yaml)

- Image registry + repository + tag
- Replica count + HPA min/max
- BC list + tier + plane + sizing tier selection
- Per-µservice ENV vars (non-secret)
- NetworkPolicy egress allowlist for cross-µservice SDK consumers
- OpenBao secret references map
- PDB minAvailable
- ServiceMonitor scrape intervals
- PrometheusRule rule-file references

## What stays per-µservice (templates — structural exceptions)

Some µservices MUST diverge for legitimate reasons; structural exceptions
are documented at top of the per-µservice template:

- **cell** µservice — uses `StatefulSet` (per-cell stable identity needed)
- **meet** µservice — GPU `nodeSelector` for Whisper transcription worker
- **anonymous** µservice — gVisor `runtimeClassName` for blind-signature workers
- **foundry** µservice — per-BC distinct `runtimeClassName`
- **drive** µservice — large-storage `PersistentVolumeClaim` template

Other deviations require an ADR.

## What is removed by SWEEP-I

- Per-µservice label-set boilerplate (now `oyatie.labels`)
- Per-µservice selector-label boilerplate (now `oyatie.selectorLabels`)
- Per-µservice securityContext boilerplate (now `oyatie.securityContext.restricted`)
- Per-µservice DNS egress boilerplate (now `oyatie.networkPolicy.allowEgressToSubstrate`)
- Per-µservice probe boilerplate (now `oyatie.probes.standardLiveness/Readiness`)
- Per-µservice resource-block boilerplate (now `oyatie.resourceRequests.tier*`)
- Per-µservice burn-rate-alert templates (now `oyatie.prometheusRule.perSloBurnRate`)

## Validation

`retired CLI helm-structural-validator` validates every per-µservice template:

1. Chart.yaml depends on `helpers` library
2. `metadata.labels` uses `{{ include "oyatie.labels" $ }}` (or documented exception)
3. `securityContext` uses `{{ include "oyatie.securityContext.restricted" $ }}`
4. Probes use canonical helpers
5. NetworkPolicy includes `oyatie.networkPolicy.allowEgressToSubstrate`
6. PrometheusRule for SLO alerts uses `oyatie.prometheusRule.perSloBurnRate`

## References

- ADR-0064 canonical-base + localization-packs
- ADR-0131 per-microservice flat layout
- `microservices/governance/iac/helm/_oyatie-helpers/`
