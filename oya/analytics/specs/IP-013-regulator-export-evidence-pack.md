# IP-013 — Regulator-Export Evidence Pack

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + axis-compliance)
**Authority ADRs:** ADR-0038 DSR cascade, ADR-0003 audit chain, ADR-0193, ADR-0039 supply chain
**Depends on:** IP-008
**Status:** Planned

## Scope

Compliance-officer-facing endpoint that produces a regulator-grade evidence pack from the audit-log surface: bulk export with cryptographic chain-of-custody, per-axis grouping, evidence-ref pointers to underlying canonical entities. Used for:

- **GDPR Article 15** subject access requests (tenant exports their own data).
- **K-PIPA Article 35** access requests.
- **Regulator inspection** (court order; internal compliance officer with justification).
- **SOC 2** annual audit evidence.

The pack is **streamed** (multi-GB packs are common) and **cosign-signed** (per ADR-0039) to provide cryptographic chain-of-custody.

## Deliverables

1. `POST /v1/regulator-export` endpoint per OpenAPI 3.2.0 contract.
2. gRPC `StreamRegulatorExport` per proto3 contract.
3. Streaming response — large packs stream NDJSON over chunked-transfer-encoding.
4. Cosign-signed manifest (per ADR-0039) at the head of every pack.
5. Cedar policy `microservices/analytics/policy/regulator-export.cedar` (already authored — referenced here).
6. Idempotency via `Idempotency-Key` header per ADR-0150.
7. Deterministic output — re-running the same window produces a byte-identical pack.
8. Integration test verifying signature correctness.

## Acceptance criteria

- 1-month export for `tenant_test_tenant` produces a pack with a valid cosign signature verifiable against the OpenBao-bound public key.
- Pack manifest lists every audit event with `(event_id, emitted_at, evidence_ref, payload_hash)`.
- Re-running the same export window with the same `Idempotency-Key` returns the same `pack_id` AND streams a byte-identical body.
- p95 export latency ≤ 30 minutes for a 1-month tenant window at sizing target.
- Cedar denies tenant principals from exporting other tenants' data.
- Internal compliance officer can export any tenant under `context.justification_kind in ["court_order", "regulator_request"]`.
- Audit-chain entry per export start AND completion.

## Implementation tasks

### T1 — REST handler

File: `crates/oya-analytics-api/src/regulator_export.rs`

```rust
#[axum::debug_handler]
pub async fn post_regulator_export(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegulatorExportRequest>,
) -> Result<Response, ApiError> {
    let principal = extract_principal(&headers)?;
    let idempotency_key = extract_idempotency_key(&headers)?;
    let request_id = extract_request_id(&headers);

    // Cedar authorize before any work.
    app.cedar.check_action(
        &principal,
        "RegulatorExport",
        Resource::Tenant(req.tenant_id.clone()),
        cedar_ctx(&req),
    )?;

    // Idempotency: if this key seen in last 24h, return the same pack_id.
    if let Some(prior) = app.idempotency_store.get(&idempotency_key).await? {
        return stream_existing_pack(prior).await;
    }

    // Generate pack_id deterministically from (tenant_id, from, to, axes).
    let pack_id = derive_pack_id(&req);

    // Emit audit-chain entry for the export start.
    app.audit_chain.emit(
        "oya.analytics.regulator_export.started.v1",
        json!({ "pack_id": pack_id, "principal": principal.id, "tenant_id": req.tenant_id }),
    ).await?;

    // Stream NDJSON.
    let stream = build_export_stream(&app, pack_id.clone(), &req, &principal).await?;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/x-ndjson")
        .header("transfer-encoding", "chunked")
        .header("x-pack-id", pack_id.as_str())
        .body(Body::from_stream(stream))?)
}
```

### T2 — Streaming construction

File: `crates/oya-analytics-api/src/regulator_export_stream.rs`

```rust
async fn build_export_stream(
    app: &AppState,
    pack_id: String,
    req: &RegulatorExportRequest,
    principal: &Principal,
) -> Result<impl Stream<Item = Result<Bytes, ApiError>>, ApiError> {
    let manifest = generate_manifest(&pack_id, req).await?;
    let manifest_signature = app.cosign.sign_blob(&manifest_canonical_json(&manifest)).await?;

    let row_stream = app.olap_client.query_audit_log_streaming(
        &principal.tenant_id,
        &req.from,
        &req.to,
        &req.axes,
    ).await?;

    let stream = stream! {
        // Line 1: signed manifest.
        let mut manifest_with_sig = manifest;
        manifest_with_sig.cosign_signature = manifest_signature;
        yield Ok(ndjson_line(&manifest_with_sig));

        // Lines 2..N: audit log rows, deterministically ordered.
        for await row in row_stream {
            let row = row?;
            yield Ok(ndjson_line(&row));
        }
    };
    Ok(stream)
}
```

### T3 — Deterministic ordering

Per "byte-identical re-run" requirement, the streaming SELECT must be deterministic:

```sql
SELECT event_id, emitted_at, tenant_id, axis, event_type, principal_id, evidence_ref, payload_hash
FROM tenant_${tid}.audit_events
WHERE emitted_at BETWEEN {from:DateTime64} AND {to:DateTime64}
  AND axis IN ({axes:Array(String)})
ORDER BY emitted_at, event_id  -- stable
SETTINGS max_execution_time = 1800;  -- 30 min ceiling
```

The `ORDER BY emitted_at, event_id` (unique tuple) ensures deterministic output.

### T4 — Manifest schema

```json
{
  "pack_id": "regexp:^[a-z0-9-]{36}$",
  "tenant_id": "ten_acme",
  "axes": ["auth", "workflow"],
  "window": {"from": "2026-04-01T00:00:00Z", "to": "2026-05-01T00:00:00Z"},
  "generated_at": "2026-05-18T10:00:00Z",
  "generated_by": {"principal_id": "alice@oyatie", "role": "ComplianceOfficer"},
  "row_count_estimate": 12345678,
  "cosign_signature": "MEUCIQ...",
  "schema_version": "1.0"
}
```

The `cosign_signature` covers the canonical-JSON-serialized manifest with the signature field zeroed.

### T5 — Cedar policy wiring

The Cedar fragment is `microservices/analytics/policy/regulator-export.cedar` (already authored).

The handler emits:

```rust
let cedar_ctx = json!({
    "audit_event_will_emit": true,
    "justification_kind": req.justification_kind.as_deref().unwrap_or(""),
    "justification_ref": req.justification_ref.as_deref().unwrap_or(""),
    "purpose": "regulator_export",
});
```

### T6 — Pack storage (optional async retrieval)

For very large packs that exceed the streaming connection window, the handler can asynchronously persist the pack to S3 and return a presigned URL:

File: `crates/oya-analytics-api/src/regulator_export_async.rs`

For phase 1, we ship synchronous streaming only. Async-fetch is deferred to phase 2.

### T7 — Integration test

File: `crates/oya-analytics-api/tests/regulator_export.rs`

```rust
#[tokio::test]
async fn test_export_byte_identical_with_idempotency_key() {
    let app = setup_test_app().await;
    seed_audit_events(&app, "tenant_test", 1000).await;

    let key = uuid::Uuid::new_v4();
    let body_1 = post_export(&app, "tenant_test", key, ("2026-01-01", "2026-01-31")).await;
    let body_2 = post_export(&app, "tenant_test", key, ("2026-01-01", "2026-01-31")).await;
    assert_eq!(body_1, body_2);
}

#[tokio::test]
async fn test_export_cosign_signature_valid() {
    let app = setup_test_app().await;
    let body = post_export(&app, "tenant_test", uuid::Uuid::new_v4(), ("2026-01-01", "2026-01-31")).await;
    let first_line = body.lines().next().unwrap();
    let manifest: Manifest = serde_json::from_str(first_line).unwrap();
    assert!(cosign::verify_blob(&app.cosign_pubkey, &manifest_canonical_json(&manifest), &manifest.cosign_signature).is_ok());
}

#[tokio::test]
async fn test_export_cross_tenant_forbidden() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("ten_acme");
    let req = RegulatorExportRequest { tenant_id: "ten_bryan".into(), .. };
    let res = post_export_as(&app, &principal, req).await;
    assert_eq!(res.status(), 403);
}
```

## Out of scope

- Async pack persistence to S3 (deferred — phase 2).
- Cross-cell export aggregation (tenant data is residency-bound; not federated).
- Non-audit-log evidence packs (e.g., business KPI export — separate path).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Query times out (>30min) | ClickHouse error 159 | return 503 with `Retry-After`; tenant retries with smaller window |
| Cosign key unavailable | OpenBao timeout | return 503; alert |
| S3 backup of pack fails (when async-fetch added) | S3 5xx | retry with backoff |
| Tenant principal asks for other tenant | Cedar forbid | 403 + audit |
| Internal admin without justification | Cedar forbid | 403 + audit |

## SLO commitment (downstream IP-014)

- Success rate: ≥ 99.9% (see `slos/regulator-export-success.openslo.yaml`).
- p95 latency (1-month window): ≤ 30 min.
- Determinism: 100% byte-identical for idempotent retries.

## Rollback

- Endpoint is purely additive; rollback = disable Cedar policy permitting the action.
- No data is mutated by the export path.

## Evidence emission

- Per export start: `oya.analytics.regulator_export.started.v1`.
- Per export complete: `oya.analytics.regulator_export.completed.v1` with `(pack_id, row_count, duration, cosign_signature)`.
- Per export error: `oya.analytics.regulator_export.failed.v1`.
- Per Cedar denial: `oya.analytics.cedar.forbid.v1` with policy fragment id.

## References

- ADR-0038 DSR cascade.
- ADR-0003 audit chain.
- ADR-0039 supply chain (cosign).
- ADR-0150 cursor pagination + idempotency keys.
- `microservices/analytics/policy/regulator-export.cedar`.
- `microservices/analytics/contracts/openapi-v1.yaml` `/v1/regulator-export`.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/analytics/specs/IP-013-regulator-export-evidence-pack.md:260` - - `microservices/analytics/contracts/openapi-v1.yaml` `/v1/regulator-export`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `KR-PIPA-2023-amendment` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=semi-annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-013-regulator-export-evidence-pack.md:235` - ## SLO commitment (downstream IP-014).

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-013-regulator-export-evidence-pack.md:246` - ## Evidence emission.
