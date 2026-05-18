// NOTE: ontology extension; jointly owned axis-ontology + axis-consent-graph.

# IP-CT-003: Projection scope narrowing (field-level redaction) — ontology extension

- Microservice: ontology (extension)
- Bounded context: cross-tenant-projection
- Layer: usecase
- Crate: `oya-ontology-cross-tenant-projection-usecase`
- Acceptance status: ga
- Authority: ADR-0214 §2.5; IP-CT-001; consent-graph IP-011 §3-4 (scope narrowing reference).

## 1. Goal

Apply consent-graph's `ProjectionScopeNarrower` (already kernel-defined in
`oya-consent-graph-projection-gateway-kernel`) to ontology's emission pipeline so that ontology
**never** emits a cross-tenant payload containing a field outside the agreement's scope.

The narrower is a **shared crate** (consent-graph defines, ontology consumes). This IP wires it into
ontology and adds ontology-specific tests + audit emission.

## 2. Scope

In:
- Adapter calling consent-graph's `ScopeNarrower::narrow` with ontology's raw row.
- PII classifier guard (consent-graph IP-011 §5).
- Audit emission: narrowed-event includes `redaction_applied: [...]` field listing redacted fields.
- Failure handling: if narrower returns Err (programming bug, schema mismatch), emission BLOCKED + P0
  audit + agreement auto-suspended.

Out:
- Narrower implementation (lives in consent-graph IP-011).
- Aggregate mode narrowing (IP-CT-004).

## 3. Integration

```rust
async fn narrow(&self, raw: &OntologyRow, target: &CrossTenantProjectionTarget)
    -> Result<JsonValue, NarrowError>
{
    // 3.1 fetch agreement (cached)
    let agreement = self.consent_graph_sdk.read(target.agreement_id, target.grantor_tenant).await?;

    // 3.2 narrow via shared consent-graph kernel
    let narrowed = consent_graph::projection_gateway::narrow(
        &raw.payload, &agreement.scope, &agreement.terms,
    )?;

    // 3.3 PII classifier guard
    self.pii_classifier.assert_no_residency_violation(&narrowed, &agreement.sovereignty)?;

    // 3.4 redaction applied summary for audit
    let redacted_fields = consent_graph::projection_gateway::redaction_summary(
        &raw.payload, &narrowed, &agreement.terms.redaction,
    )?;
    self.audit_bridge.emit_narrow_summary(target.agreement_id, redacted_fields).await?;

    Ok(narrowed)
}
```

## 4. Tests

- `narrow_excludes_out_of_scope_field` — entity has field "ssn"; scope.allow=["sku"]; output has only "sku".
- `narrow_redaction_mask_applied` — terms.redaction.email=Mask; output has "email":"****".
- `narrow_hash_deterministic_within_agreement` — same input + same agreement → same hash.
- `narrow_hash_differs_across_agreements` — agreement-specific salt verified.
- `pii_classifier_blocks_kr_pack_pii_in_eu_grantee` — KR pack agreement with EU grantee + cross-border-forbidden field → error.
- `narrow_failure_blocks_emit` — programmatic narrower error → caller receives error; no emission.

## 5. Performance

- p99 narrow latency ≤10ms (in-memory + JSON manipulation only).
- 1M ops/s capacity per pod.

## 6. Risk

- **R**: PII classifier false-negative → leak.
  **M**: Classifier is data-driven (`oya-shared-pii-classifier`); PR review required for changes;
  default = forbidden on unknown field.
- **R**: Schema drift — agreement created with old entity schema; ontology now uses new schema.
  **M**: schema_version on agreement; narrower uses agreement's schema version; PR review on
  schema migration.

## 7. Verification

- `cargo build` + `cargo test`.
- E2E test: emit 1K rows through narrower, verify zero out-of-scope fields leak.
- Property test: random schemas + random agreements; narrower output ⊆ agreement.scope.field_set.

## 8. Cross-references

- IP-CT-001 (kernel types)
- microservices/consent-graph/IP-011 (narrower impl)
- microservices/consent-graph/threat-model.md §5 (data leakage)
