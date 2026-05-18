---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-008-redaction-bc
status: pending
owner: council-privacy + axis-recordings
acceptance_lanes: [port-location, recordings-redaction-overlay-immutability]
---

# IP-008: Redaction BC — overlay model (no source mutation)

## Intent

Land the redaction-overlay model per ADR-RECORDINGS-0003. Auto-PII at
transcription time + manual compliance-officer overlay; insert-only rows
with audit-chain seal.

## Concrete crates

`oya-recordings-redaction-{kernel,domain,usecase,api,adapter-postgres,adapter-ffmpeg,rest,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate recordings-redaction-overlay-immutability
# No UPDATE statements on redaction_overlay table
rg "UPDATE\s+redaction_overlay" microservices/recordings/src   # expect zero
```

## Next IP

[`IP-009-chapter-summary-bcs.md`](IP-009-chapter-summary-bcs.md)
