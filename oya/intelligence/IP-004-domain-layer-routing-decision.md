---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-004-domain-layer-routing-decision
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-004: Domain layer — RoutingDecision + provider catalog

## Intent

`RoutingDecision` entity + `ProviderCatalog` value type in
`oya-intelligence-model-routing-domain`. The catalog enumerates per-pack-permitted providers
with per-modality coverage and per-tenant credential kind.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-model-routing-domain/src/routing_decision.rs` | create |
| `.../oya-intelligence-model-routing-domain/src/provider_catalog.rs` | create |
| `.../oya-intelligence-model-routing-domain/src/provider.rs` | create (enum) |
| `.../oya-intelligence-model-routing-domain/src/pack.rs` | create (enum) |

## Code shape

```rust
pub struct RoutingDecision {
    pub envelope_id: Ulid,
    pub provider: Provider,
    pub model: ModelId,
    pub region: Region,
    pub credential_kind: SecretReferenceKind,
    pub fallback_count: u8,
    pub decided_at: SystemTime,
}

pub struct ProviderCatalog { entries: Vec<ProviderCatalogEntry> }

pub struct ProviderCatalogEntry {
    pub pack: Pack,
    pub provider: Provider,
    pub models: Vec<ModelId>,
    pub modalities: Vec<Modality>,
    pub regions: Vec<Region>,
    pub credential_kinds: Vec<SecretReferenceKind>,
    pub compliance_flags: ComplianceFlags,
    pub baa_signed: bool,
    pub fedramp_high: bool,
    pub cn_resident: bool,
}

impl ProviderCatalog {
    pub fn permitted_for(&self, pack: Pack, modality: Modality) -> Vec<&ProviderCatalogEntry> { ... }
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-domain
buck2 build //:quality-lane-registry-authority-check # lane=provider-catalog-pack-coverage --microservice intelligence
```

## Test plan

- All 14 packs × all 16 providers covered or explicitly excluded with reason.
- `permitted_for` returns subset for known (pack, modality).

## Next IP

[`IP-005-domain-layer-eval-record.md`](IP-005-domain-layer-eval-record.md)

## References

- `microservices/intelligence/ARCHITECTURE.md` §6.
- `microservices/intelligence/multi-region.md`.
- `microservices/intelligence/policy/provider-routing.cedar`.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-004-domain-layer-routing-decision.md` matched `multi-region`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.
