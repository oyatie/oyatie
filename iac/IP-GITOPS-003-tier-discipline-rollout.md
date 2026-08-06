# IP-GITOPS-003 — Cloud-IaC apply-scope discipline rollout

> ADR anchor: ADR-0202.
> Owner: `axis-cloud-iac + ops-security`.
> Scope: `iac` only.

## Goal

Replace retired tier-discipline language with the current cloud-iac
apply-scope discipline: render workers render, validator workers validate and
emit drift, applier workers mutate declared apply scope, rollback workers revert
declared apply scope, and registry workers maintain state/provenance records.

## Real service paths

| Path | Contract this IP tightens |
|---|---|
| `iac/policy/ci-scope.cedar` | worker action permits and cross-microservice apply forbid |
| `iac/policy/iac-isolation.md` | isolation invariants and reserved scopes |
| `iac/contracts/openapi/cloud-iac.yaml` | `triggerRender`, `planPreview`, `triggerApply`, `triggerRollback`, `getDriftReport` |
| `iac/contracts/proto/cloud-iac.proto` | gRPC equivalents for render/apply/rollback/drift |
| `iac/capabilities/iac-render.yaml` | render capability risk surface |
| `iac/capabilities/iac-apply.yaml` | apply capability risk surface |
| `iac/capabilities/iac-rollback.yaml` | rollback capability risk surface |

## Implementation contract

1. CI gates fail when an applier or rollback request lacks
   `apply_scope.microservice == resource.microservice`.
2. Drift detection and validation remain separate from mutation. Validator
   identities may emit `write_drift_report`; they may not apply or roll back.
3. Registry writes are append/provenance oriented and must not mutate ArgoCD
   applications.
4. The old "advisory then BLOCKER after tier migration window" framing is not
   used for this service. The enforcement unit is the current policy contract in
   `ci-scope.cedar`.

## Counterpart refs

- `iac/cross-microservice-handoffs.md` outbound rows to
  `cloud-k8s` define the only cluster mutation handoff.
- `iac/cross-microservice-handoffs.md` outbound row to
  `audit-chain` defines the audit emission dependency that must precede durable
  infrastructure mutation.

## Acceptance criteria

- This IP contains no retired rollout label or delayed-enforcement window
  language.
- `rg "cross-µservice apply|apply_scope|iac-applier-worker" iac/policy/ci-scope.cedar`
  confirms the policy source exists.
- REST and gRPC contracts both expose render, apply, rollback, and drift reads.

## Validation commands

```bash
rg "apply_scope|iac-applier-worker|iac-rollback-worker" iac/policy/ci-scope.cedar
rg "triggerApply|triggerRollback|getDriftReport" iac/contracts/openapi/cloud-iac.yaml
```

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-GITOPS-003-tier-discipline-rollout.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `iac/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `iac/manifest.json`, `iac/IP-GITOPS-003-tier-discipline-rollout.md`.
