// NOTE: This IP lives in the ontology µservice directory because it extends ontology's projection
// model. It is owned jointly by axis-ontology + axis-consent-graph. Authored as part of
// PR #143 (consent-graph PHASE-01) per ADR-0214 §verification.

# IP-CT-001: Cross-tenant projection topic model (ontology extension)

- Microservice: ontology (extension)
- Bounded context: cross-tenant-projection (new)
- Layer: kernel
- Crate: `oya-ontology-cross-tenant-projection-kernel`
- Acceptance status: ga
- Authority: ADR-0214 §2.5 + ADR-SVC-CG-004; ADR-0058 ontology projection model.
- Depends on: `oya-ontology-projection-kernel` (existing intra-tenant projection model),
  `oya-consent-graph-projection-gateway-kernel` (cross-tenant types).

## 1. Goal

Extend ontology's existing projection model to support **cross-tenant** projections gated by
consent-graph. Today, ontology emits projections to tenant-local Pulsar topics; this IP adds a typed
cross-tenant projection topic model where:
- Topic resides in grantor's tenant + region (sovereignty).
- Grantee subscribes cross-tenant via consent-graph-minted JWT.
- Payload conforms to consent-graph's `ProjectionEvent` shape (already kernel-defined).

## 2. Scope

In:
- New `CrossTenantProjectionTarget` value object — extends `ProjectionTarget` with `(grantor_tenant,
  grantee_tenant, agreement_id, sovereignty_pin)`.
- New `CrossTenantProjectionEmitter` port — implemented by IP-CT-002's Pulsar adapter.
- Extension of ontology's `EmissionPipeline` to consult consent-graph for cross-tenant routing
  decisions per entity emission.

Out:
- Pulsar tenant-aware ACL impl (→ IP-CT-002).
- Scope-narrowing impl (→ IP-CT-003; reuses consent-graph's `ScopeNarrower`).
- Aggregate mode impl (→ IP-CT-004).
- Zero-copy contract enforcement (→ IP-CT-005).

## 3. Types

```rust
pub struct CrossTenantProjectionTarget {
    pub base: ProjectionTarget,                          // existing intra-tenant target
    pub grantor_tenant: TenantId,
    pub grantee_tenant: TenantId,
    pub agreement_id: AgreementId,                       // from consent-graph
    pub sovereignty_pin: Region,                         // MUST equal grantor's region
    pub topic_name: TopicName,                           // canonical from IP-010
    pub mode: SharingMode,                               // Projection | Aggregate | AttestedQuery
}

pub trait CrossTenantProjectionRouter: Send + Sync {
    /// Given an entity event, returns the set of cross-tenant projection targets it should fan out to.
    /// Calls consent-graph::agreement-sdk::list_active_by_grantor (cached) + filters by entity_type.
    async fn route(&self, entity: &OntologyRow)
        -> Result<Vec<CrossTenantProjectionTarget>, RouteError>;
}

pub trait CrossTenantProjectionEmitter: Send + Sync {
    async fn emit(&self, target: &CrossTenantProjectionTarget, event: &ProjectionEvent)
        -> Result<(), EmitError>;
}
```

## 4. Integration

Ontology's existing emission pipeline gains a hook **after** intra-tenant projection emission:

```rust
async fn emit_pipeline(&self, entity: &OntologyRow) -> Result<(), Error> {
    // 4.1 existing intra-tenant projections
    self.intra_tenant_emitter.emit(entity).await?;

    // 4.2 NEW: cross-tenant projections
    let targets = self.cross_tenant_router.route(entity).await?;
    for target in targets {
        // 4.2.1 enforcement check (consent-graph::enforcement-sdk)
        let allowed = self.enforcement.check_project_emit(target.agreement_id, entity).await;
        if !allowed { continue; }  // audit emitted by consent-graph; we just skip

        // 4.2.2 narrow + redact (IP-CT-003)
        let narrowed = self.narrower.narrow(entity, &target).await?;

        // 4.2.3 emit
        let event = ProjectionEvent::for_target(&target, &narrowed)?;
        self.cross_tenant_emitter.emit(&target, &event).await?;
    }
    Ok(())
}
```

## 5. Tests (kernel-pure)

- `cross_tenant_target_sovereignty_pin_required` — kernel rejects target where `sovereignty_pin != grantor.region`.
- `cross_tenant_target_unique_per_pair_entity` — for any (grantor, grantee, entity), at most one target.
- `route_returns_empty_when_no_active_agreement` — entity with no matching agreement → empty list.
- `route_filters_by_entity_type` — only agreements matching `entity_type` returned.

## 6. Dependencies

- `oya-ontology-projection-kernel`
- `oya-consent-graph-projection-gateway-kernel`
- `oya-consent-graph-agreement-sdk` (port; consumed by router impl in IP-CT-002)
- `oya-consent-graph-enforcement-sdk` (port; consumed by router impl)
- `serde`, `thiserror`, `ulid`

## 7. Verification

- `cargo build` + `cargo test` clean.
- `oya-check-layer-bnf-conformance` clean (kernel pure).
- Property test: random entity emissions + random agreement state → route function produces only
  valid cross-tenant targets.

## 8. Risk

- **R**: Routing decision becomes hot-path bottleneck.
  **M**: Agreement-SDK cache on consent-graph side; router cache on ontology side (60s TTL); enforcement
  check at emit time (not at route time) cached for 200ms freshness.
- **R**: Stale agreement list → emit to wrong grantee.
  **M**: Revocation Pulsar subscriber in ontology cross-tenant emitter invalidates router cache on
  revocation event (per IP-008 fan-out).

## 9. Public surface

- `CrossTenantProjectionTarget` value object
- `CrossTenantProjectionRouter` port
- `CrossTenantProjectionEmitter` port
- `RouteError`, `EmitError`

## 10. Cross-references

- ADR-0214 §2.5 sovereignty
- ADR-SVC-CG-004 grantor-region authority
- microservices/consent-graph/IP-009 (projection-gateway-kernel)
- microservices/ontology/specs/projection-model.md (existing intra-tenant model)


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
