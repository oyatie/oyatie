---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-014-router-rest-worker-app
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

# IP-014: Router REST + worker + app

## Intent

REST surface + worker (engine-health monitor + cost roll-up + TM/termbase sync) + composition-root app binary. Wires every crate together; produces deployable images.

## ChangeSet boundary

Three new Rust crates:

- `oya-translate-router-rest` — Axum REST + OpenAPI generation
- `oya-translate-router-worker` — engine-health monitor + per-tenant cost roll-up + Mimir scrape endpoint
- `oya-translate-router-app` — composition root binary

## REST Surface

Per `contracts/openapi/translate.yaml`:

```
POST   /v1/translate                       — single-segment translate
POST   /v1/translate/batch                 — batch translate (≤ 100 segments)
POST   /v1/translate/decide                — dry-run routing decision
POST   /v1/detect-language                 — language detection
POST   /v1/tm/lookup                       — TM leverage query
POST   /v1/qe/score                        — quality estimation
GET    /v1/engines/health                  — per-engine health snapshot
GET    /v1/engines/capabilities            — capability catalogue
GET    /v1/policies/tenant/{tenant}        — tenant policy
PUT    /v1/policies/tenant/{tenant}        — tenant policy
```

Per `contracts/openapi/translate-files.yaml`:

```
POST   /v1/files/translate                 — document translate (multipart or signed-URL)
GET    /v1/files/translate/{job_id}        — doc job state
GET    /v1/files/translate/{job_id}/output — signed-URL to S3
POST   /v1/bulk/jobs                       — bulk-translate submit
GET    /v1/bulk/jobs/{job_id}              — bulk job state
DELETE /v1/bulk/jobs/{job_id}              — cancel
POST   /v1/termbase/import                 — TBX import
GET    /v1/termbase/export                 — TBX export
POST   /v1/tm/import                       — TMX import
GET    /v1/tm/export                       — TMX export
POST   /v1/files/import-xliff              — XLIFF 2.1 import
GET    /v1/files/export-xliff              — XLIFF 2.1 export
```

Per `contracts/openapi/translate-stream.yaml`:

```
WS     /v1/stream/translate                — real-time caption stream (per-session)
```

## Worker Surface

- Subscribes to `oya.foundry.providers.provider-health-changed` to update local engine-health snapshots.
- Computes per-tenant rolling cost roll-up; emits to Mimir.
- Periodic TM minhash recompute job (per-tenant scheduled).
- Periodic Meilisearch sync job (per-tenant + per-project).
- Bulk-job queue worker (per-job fan-out).
- Document-job queue worker (per-doc; spawn gVisor pods).

## Composition Root (App)

`oya-translate-router-app` `main.rs` wires:

1. OpenBao agent socket → `cloud-secrets-adapter`.
2. Postgres pool → `tm-adapter-postgres` + `termbase-adapter-postgres` + `bulk-adapter-postgres`.
3. Redis pool → `bulk-adapter-redis` + `stream-adapter-redis`.
4. Meilisearch client → `tm-adapter-meilisearch`.
5. S3 client → `doc-adapter-s3` + `bulk-adapter-s3`.
6. foundry-providers SDK → engine adapters (anthropic + openai + google + deepl + foundry-runtime).
7. NATS publisher → audit-chain emission.
8. Cedar policy engine → `policy/translate-tenant-scope.cedar` + `policy/ai-act-overlay.cedar`.
9. OTel exporter → observability.

## Test Plan

| Test | Verifies |
|---|---|
| `test_rest_openapi_spec_matches_handlers` | drift detection |
| `test_rest_oidc_required_on_all_paths` | auth |
| `test_rest_residency_violation_returns_403` | residency gate at REST |
| `test_rest_ratelimit_429_per_tenant_per_engine` | T-10 |
| `test_worker_engine_health_subscribed` | event flow |
| `test_app_composition_wires_all_crates` | composition root |
| `tests/load/router_decision_p99_5ms.rs` | AC-06 |
| `tests/load/translation_request_p95_250ms.rs` | AC-07 |
| `tests/e2e/full_translate_round_trip.rs` | end-to-end |
| `tests/e2e/pack_cn_stub_refuses_external_engine.rs` | residency in deployable artifact |

## Halt Conditions

- Any handler bypasses Cedar policy.
- Composition root misses a port wire-up (compile-time impossible if traits used properly).
- OpenAPI drift not caught by CI.

## Next IP

[`IP-015-hg-translate-gate-registration.md`](IP-015-hg-translate-gate-registration.md)
