---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-012-export-ediscovery-bcs
status: pending
owner: ops-compliance + axis-recordings + ops-security
acceptance_lanes: [audit-chain-integrity, ediscovery-merkle-seal]
---

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

## Next IP

[`IP-013-translation-bc.md`](IP-013-translation-bc.md)
