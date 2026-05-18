---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-008-redaction-bc
status: pending
owner: council-privacy + axis-recordings
acceptance_lanes: [port-location, recordings-redaction-overlay-immutability]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-008-redaction-bc
depends_on_changesets: [CS-RECORDINGS-IP-006-transcript-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-007-search-bc, CS-RECORDINGS-IP-009-chapter-summary-bcs]
enables: [CS-RECORDINGS-IP-012-export-ediscovery-bcs]
acceptance_status: ga
load_bearing: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Redaction overlay table is insert-only — zero UPDATE statements in source | `rg "UPDATE\s+redaction_overlay" microservices/recordings/src` exits with no hits |
| AC-02 | Source media never mutated; overlay applied at playback | `cargo nextest run -p oya-recordings-redaction-domain -- source_immutability` |
| AC-03 | Auto-PII candidates emitted from transcription → overlay rows audit-chain sealed | `cargo nextest run -p oya-recordings-redaction-usecase -- auto_pii_seal` |
| AC-04 | Manual compliance-officer overlay requires Cedar `redact` action grant | `cargo nextest run -p oya-recordings-redaction-domain -- cedar_grant_required` |
| AC-05 | `oya gate validate recordings-redaction-overlay-immutability` exits 0 | governance lane |

## Build Sequence

1. Kernel: `RedactionOverlayStore`, `AutoPiiDetector`, `OverlayApplicator` ports.
2. Domain: `OverlayRow` (insert-only), `RedactionType` (audio/video/transcript), `Justification`.
3. Usecase: `EmitAutoPii`, `ApplyManualRedaction`, `ResolveOverlayAtPlayback`.
4. Adapter: `-adapter-postgres` enforcing insert-only via trigger + RLS.
5. `cargo run -p oya-dev-cli -- gate validate recordings-redaction-overlay-immutability`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-09 (manual redaction), FR-04 (auto-redaction) |
| PRD-recordings AC | AC-04 (no source mutation) |
| ADR | ADR-RECORDINGS-0003 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Developer adds UPDATE on overlay table by mistake | CI lane refuses any UPDATE on `redaction_overlay` |
| Overlay miss at playback exposes PII | Test `e2e_overlay_applied_at_playback` |
| Court-ordered redaction released early | Compliance-officer override via two-person rule (ADR-0215) |

## References

- ADR-RECORDINGS-0003.
- GDPR Art. 25 (Data protection by design and by default).
- HIPAA Safe Harbor — §164.514 (De-identification of protected health information).
- NIST SP 800-188 — De-identification of Personal Information.

## Next IP

[`IP-009-chapter-summary-bcs.md`](IP-009-chapter-summary-bcs.md)
