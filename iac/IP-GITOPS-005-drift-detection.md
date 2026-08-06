# IP-GITOPS-005 — Drift detection and reporting

> ADR anchor: ADR-0202, ADR-0145.
> Owner: `axis-cloud-iac + ops-observability`.
> Scope: `iac` only.

## Goal

Define drift detection against the real cloud-iac contracts: desired-state
render/apply records, OpenTofu plan results, drift reports, AsyncAPI
`DriftDetected` events, and operator/auditor read surfaces.

## Real service paths

| Path | Drift role |
|---|---|
| `iac/contracts/openapi/cloud-iac.yaml` | REST `getDriftReport` surface and `DriftReport` schema |
| `iac/contracts/proto/cloud-iac.proto` | gRPC `GetDriftReport` and `DriftReport` schema |
| `iac/contracts/asyncapi/cloud-iac-events.yaml` | `workflow-events/drift.detected` / `DriftDetectedPayload` |
| `iac/policy/auditor-scope.cedar` | auditor read access to drift reports |
| `iac/policy/ci-scope.cedar` | validator-worker permit to `write_drift_report` |
| `iac/runbooks/drift-remediation.md` | operator remediation steps |
| `iac/dashboards/drift-coverage.json` | drift coverage dashboard |

## Implementation contract

1. The validator worker owns drift report writes and drift event emission.
2. REST and gRPC reads must expose `microservice`, `pack`, `environment`,
   `drift_score`, `detected_at`, and `drift_items`.
3. Drift events publish through `workflow-events/drift.detected`; incidents and
   metrics are downstream counterpart actions, not hidden side effects.
4. The drift path must not mutate desired state. Remediation happens through
   the normal `planPreview` and `triggerApply` path.

## Counterpart refs

- `iac/cross-microservice-handoffs.md` inbound row from
  `audit-chain` reads drift reports for audit.
- `iac/cross-microservice-handoffs.md` inbound row from
  `observability` reads drift report projections for metrics.
- `iac/cross-microservice-handoffs.md` outbound row to
  `ops-dashboard-control-center` opens drift incidents.

## Acceptance criteria

- `DriftReport` fields align across OpenAPI and proto contracts.
- `DriftDetectedPayload` exists in AsyncAPI and carries drift metadata.
- Auditor policy permits read-only drift access and forbids drift writes.
- The runbook names the operator path after drift detection.

## Validation commands

```bash
rg "DriftReport|getDriftReport" iac/contracts/openapi/cloud-iac.yaml
rg "DriftReport|GetDriftReport" iac/contracts/proto/cloud-iac.proto
rg "DriftDetected|drift.detected" iac/contracts/asyncapi/cloud-iac-events.yaml
rg "write_drift_report|read_drift_report" iac/policy
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-GITOPS-005-drift-detection.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `iac/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `iac/manifest.json`, `iac/IP-GITOPS-005-drift-detection.md`.
