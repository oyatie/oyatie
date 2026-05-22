---
doc_class: SDK-Plan
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0258
companion_docs:
  - microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml
  - microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# SDK Plan — ops-dashboard-control-center

## §1 SDK surface

This is an **internal ops surface**. No public-facing SDK is planned. The SDK surface exists for:

1. **Internal tooling integration**: CLI tool `oya-ops` wraps the admin control-plane REST API for scriptable operator workflows. Useful for bulk operations (e.g., batch evidence-pack export, scripted on-call handoff for automated rotation systems).
2. **Foundry pipeline integration**: Foundry uses the gRPC proto surface (`contracts/proto/ops_dashboard_control_center.proto`) to pull scorecard and SLO state. The proto client is the "SDK" for Foundry.
3. **Monitoring integration**: Prometheus metrics scrape via `/metrics` endpoint. Grafana dashboards consume directly.

## §2 `oya-ops` CLI SDK (internal)

| Command | Maps to | Step-up required |
|---|---|---|
| `oya-ops incident declare --severity SEV1 --title "..."` | `POST /ops/v1/incidents` | T2 (TOTP/passkey) |
| `oya-ops deployment approve --deployment-id <id> --rationale "..."` | `POST /ops/v1/deployments/{id}/approvals` | T3 (hardware key) |
| `oya-ops rollback execute --deployment-id <id>` | `POST /ops/v1/deployments/{id}/rollback` | T3 (hardware key) |
| `oya-ops evidence-pack export --tenant-id <id>` | `POST /ops/v1/tenants/{id}/evidence-packs` | T2 |
| `oya-ops oncall handoff --incoming <operator-id>` | `POST /ops/v1/oncall/handoffs` | T2 |

CLI is Rust binary; wraps the OpenAPI client generated from `contracts/openapi/ops-dashboard-control-center.yaml`. Step-up auth integrated via `oya-bao token lookup` before each T2/T3 command.

## §3 Versioning

Per ADR-0258: SDK version tracks API SemVer. `oya-ops` CLI `0.x` while API is `0.x`. No stability guarantees until `1.0`. Deprecation cadence: ≥90d notice for breaking CLI changes.

## §4 Not planned

- Public REST SDK (npm/PyPI/crates.io): internal surface only.
- Webhook SDK: internal ops surface does not emit webhooks to external consumers.
