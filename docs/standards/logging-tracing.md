---
purpose: Oyatie — Logging + Tracing Standard
doc_status: published
---

# Oyatie — Logging + Tracing Standard

> **Owner:** `ops-sre-reliability`.

## 1. Tracing crate

`tracing` + `tracing-subscriber`; structured JSON output to stdout; OTel exporter to the ADR-0383/ADR-0186 observability backplane plus audit chain. ADR-0042 instrumentation semantics are carried forward by ADR-0383; ADR-0042 is not storage-tier authority.

## 2. OpenTelemetry semantic conventions

- General: OTel resource + service.name + service.version + deployment.environment
- HTTP: `http.method`, `http.route`, `http.status_code`, `http.client_ip`
- DB: `db.system`, `db.statement` (redacted per data-class)
- Messaging: `messaging.system`, `messaging.destination`, `messaging.message.id`
- Cloud: `cloud.region`, `cloud.availability_zone`, `cloud.cell` (Oyatie extension)
- Tenancy: `oyatie.tenant.id`, `oyatie.tenant.region`, `oyatie.cell.id` (Oyatie extension)
- Foundry / GenAI: per OTel `gen_ai.*` semconv (`gen_ai.system`, `gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`, etc.)

## 3. Mandatory fields per span

- `trace_id` (W3C)
- `span_id` (W3C)
- `service.name`
- `oyatie.tenant.id` (when tenant-scoped)
- `oyatie.cell.id` (when cell-scoped)
- `oyatie.capability.id` (when Foundry capability)
- `oyatie.data_classes_touched` (per ADR-0008)
- `oyatie.autonomy_tier` (per ADR-0007/0022 when applicable)

## 4. Log levels

| Level | Use |
|---|---|
| `error!` | Operation failed; manual investigation may be needed |
| `warn!` | Operation degraded but completed; auto-alert if rate exceeds threshold |
| `info!` | Notable operation (capability invoke / tenant onboard / DSR cascade) |
| `debug!` | Diagnostic (off in production by default; tenant-tagged opt-in) |
| `trace!` | Verbose; off in production always |

## 5. Per-data-class redaction

Per [PRIVACY-PROGRAM.md §2.2.1](../PRIVACY-PROGRAM.md):
- Never log PHI / PCI / Sensitive-PIPA-Art23 even at debug
- PII_IDENTIFYING / PII_QUASI redacted with `[REDACTED]` in production
- Tenant-scoped fields hashed (k-anonymity) for cross-tenant aggregate

## 6. Tracing instrumentation

Per [code-style.md §1](code-style.md):
- `#[tracing::instrument(skip(secret_field))]` on every async public fn
- Per-handler span emits `http.*` fields
- Per-capability span emits `oyatie.capability.*` fields
- Per-saga span emits `oyatie.workflow.*` fields

## 7. Replay-safe traces

Per ADR-0024 Foundry replay:
- Per-step trace emits enough state to reconstruct invocation
- Provider-response captured (encrypted at rest per data-class)
- Replay endpoint reconstructs full agent run

## 8. Forwarding + storage

- Mimir for metrics, Loki for logs, Tempo for traces, and Grafana for visualization under the self-hosted AGPL-3 posture recorded by ADR-0383 / ADR-0186.
- No Grafana Cloud or external SaaS observability backend is introduced by this standard.
- Per-cell observability namespace
- Per-tenant audit-chain integration

## 9. Sources
OTel `gen_ai` semconv; `tracing` crate; ADR-0003 (audit chain); ADR-0008 (Data Use Boundary); ADR-0383 / ADR-0186 (observability storage and visualization backplane); ADR-0042 only for instrumentation semantics carried forward by ADR-0383.
