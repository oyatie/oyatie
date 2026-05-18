---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-012-export-ediscovery-bcs
status: pending
owner: ops-compliance + axis-recordings + ops-security
acceptance_lanes: [audit-chain-integrity, ediscovery-merkle-seal]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Export BC + eDiscovery BC (chain-of-custody Merkle seal)

## Intent

Land MP4/MP3/WAV/VTT/SRT/PDF/DOCX export bundle + eDiscovery export with
chain-of-custody Merkle seal per ADR-RECORDINGS-0002. Pandoc 3.x for
transcript-to-PDF/DOCX.

## Concrete crates

- `oya-recordings-export-{kernel,domain,usecase,api,adapter-ffmpeg,adapter-pandoc,worker,sdk,app}`
- `oya-recordings-ediscovery-{kernel,domain,usecase,api,adapter-postgres,worker,sdk,app}`

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate ediscovery-merkle-seal
# Bundle Merkle root verified independently
cargo nextest run -p oya-recordings-ediscovery-worker -- bundle_merkle_round_trip
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-012-export-ediscovery-bcs
depends_on_changesets: [CS-RECORDINGS-IP-008-redaction-bc, CS-RECORDINGS-IP-010-retention-legal-hold-bcs, CS-RECORDINGS-IP-011-playback-share-link-watermark-bcs]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-013-translation-bc]
enables: []
acceptance_status: ga
load_bearing: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Export bundle emits MP4 + transcript VTT + redaction overlay + audit-chain seal | `cargo nextest run -p oya-recordings-export-usecase -- bundle_shape` |
| AC-02 | eDiscovery bundle Merkle root verified independently | `cargo nextest run -p oya-recordings-ediscovery-worker -- bundle_merkle_round_trip` |
| AC-03 | Pandoc 3.x emits transcript PDF + DOCX with Unicode coverage | `cargo nextest run -p oya-recordings-export-adapter-pandoc -- unicode_coverage` |
| AC-04 | Export-worker SPIFFE Ed25519 signs the bundle | `cargo nextest run -p oya-recordings-ediscovery-worker -- spiffe_signed` |
| AC-05 | `oya gate validate ediscovery-merkle-seal` exits 0 | governance lane |

## Build Sequence

1. Export kernel + ffmpeg + pandoc adapters + worker.
2. eDiscovery kernel + postgres adapter + worker.
3. Bundle Merkle seal via blake3 tree (RFC 9162-style).
4. SPIFFE Ed25519 signer integration.
5. `cargo run -p oya-dev-cli -- gate validate ediscovery-merkle-seal`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-06 (export), FR-10 (eDiscovery) |
| PRD-recordings Tenant Outcome | TO-5 |
| ADR | ADR-RECORDINGS-0002 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Bundle tampered post-emit | Merkle root verified at re-ingest |
| SPIFFE key compromise | Short-TTL SPIFFE identity + rotation |
| Pandoc CVE on malformed transcript | Pandoc in gVisor sandbox |

## References

- ADR-RECORDINGS-0002.
- FRCP Rule 26(f), Rule 34 (US Federal Rules of Civil Procedure).
- The Sedona Conference Cooperation Proclamation.
- ISO/IEC 27037:2012 (Digital evidence identification, collection, acquisition and preservation).
- SPIFFE / SPIRE specification (`spiffe.io/docs`).
- Pandoc User's Guide (`pandoc.org`).
- RFC 9162 (Certificate Transparency 2.0) — Merkle tree reference.

## Next IP

[`IP-013-translation-bc.md`](IP-013-translation-bc.md)
