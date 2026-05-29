---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-006-domain-layer-attribution
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-006: Domain layer — Attribution + citation schema

## Intent

`Citation` + `AttributionGraph` value objects in `oya-intelligence-attribution-domain`.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-attribution-domain/Cargo.toml` | create |
| `.../oya-intelligence-attribution-domain/src/lib.rs` | create |
| `.../oya-intelligence-attribution-domain/src/citation.rs` | create |
| `.../oya-intelligence-attribution-domain/src/attribution_graph.rs` | create |

## Code shape

```rust
pub struct Citation {
    pub source_uri: SourceUri,
    pub span: TextSpan,
    pub confidence: Confidence,        // newtype [0.0, 1.0]
    pub attribution_kind: AttributionKind,
}

pub enum AttributionKind {
    DirectQuote,
    Paraphrase,
    FactualClaim,
}

pub struct AttributionGraph {
    pub envelope_id: Ulid,
    pub citations: Vec<Citation>,
    pub provenance_chain: Vec<ProvenanceNode>,
}

pub struct TextSpan {
    pub start: usize,
    pub end: usize,
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-attribution-domain
```

## Test plan

- TextSpan well-formed: `start < end`.
- Confidence ∈ [0, 1].
- Round-trip serialise/deserialise.

## Next IP

[`IP-007-kernel-model-router.md`](IP-007-kernel-model-router.md)

## References

- `microservices/intelligence/capabilities/attribution.yaml`.

## Wave 15 substance conversion — AttributionGraph domain

### §A Problem

The PRD promises context-aware retrieval and advisory drafts with citations, but a raw string response cannot prove
which source supported which claim.
This IP closes the domain model gap for transparent citation rendering and auditable attribution.

### §B Approach

Define `Citation`, `TextSpan`, `AttributionKind`, and `AttributionGraph` as pure domain value objects.
Adapters and retrieval usecases populate them; Layer-B renders them in `CitationCard`.

### §C Deliverables

- `crates/oya-intelligence-attribution-domain/src/citation.rs`
- `attribution_graph.rs`, `source_uri.rs`, and `confidence.rs`
- tests for span bounds, confidence bounds, and serialization

### §D Implementation

1. Require every citation to carry source URI, span, confidence, and kind.
2. Validate `start < end` and source URI scheme allowlist.
3. Bind citations to `envelope_id` for audit correlation.
4. Represent direct quote, paraphrase, and factual claim separately.
5. Keep RAG document bodies outside the graph.
6. Round-trip through JSON for OpenAPI/proto alignment.

### §E Acceptance

Nextest must reject malformed spans and confidence values and prove `AttributionGraph` survives serialization for
Layer-B and audit tap.

### §F Evidence

Local anchors: `capabilities/attribution.yaml`, `contracts/openapi/intelligence-v1.yaml`, and
`runbooks/rag-retrieval-quality-regression.md`.

### §G Counterparts

OpenAI retrieval annotations, Anthropic source citations, and Google grounding expose similar citation UX; oyatie
adds explicit domain objects and audit correlation.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-006-domain-layer-attribution.md` matched `attribution`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
