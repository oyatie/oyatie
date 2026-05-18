---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: recordings
deprecated_artifact: oya-connect-recordings-domain crate (single bundled crate)
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-RECORDINGS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005, ADR-RECORDINGS-0006, ADR-RECORDINGS-0007]
related_specs: [/specs/microservices/recordings.json]
owner_team: axis-recordings
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-recordings-domain` crate

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-recordings-*` crate family under `microservices/recordings/src/crates/`
per ADR-0131 — 22 bounded contexts split across the canonical 13-layer enum.
See **`microservices/recordings/migration-from-connect.md`** for the full
import-path map, Hyrum's-Law-bound surface callouts, configuration delta,
runbook continuity table, and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-RECORDINGS
accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #2).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter soak +
Phase 3 canary), the indicative advisory removal date is **2026-11-17**,
gated on the SLO trigger.

## Reason

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a brand-layer
   concept and must not appear in crate names.
2. **ADR-0130 — agentic SLO-gated promotion.** Recordings needs independent
   SLO targets per surface (recording-list, playback-start, transcript-search,
   transcript-render, redaction-render, export-mp4, export-transcript-pdf,
   legal-hold-engagement + zero-tolerance load-bearing invariants for
   retention-policy-correctness and legal-hold-chain-of-custody-correctness).
3. **ADR-0131 — per-µservice flat layout.** Recordings's 8 runbooks, threat-
   model, DPIA, multi-region plan, sdk-plan all need to live under one
   folder.
4. **ADR-RECORDINGS-0001..0007** are net-new clean-replacement-boundary
   capabilities that did not exist in the legacy surface; the new µservice
   ships them, the legacy did not.

## Migration Guide pointer

→ **`microservices/recordings/migration-from-connect.md`**

Includes: import-path map (legacy single bundled crate → 22 BCs × N layers);
net-new BC enumeration (transcript / redaction / legal-hold load-bearing /
ediscovery / watermarking / video-encode-ladder / multi-source ingest /
tiered-storage / cross-µservice translate); concrete `use` and `Cargo.toml`
rewrites; configuration delta; runbook continuity table (8 net-new);
Hyrum's-Law surface callouts (HLS manifest byte stability, transcript JSON
field ordering, redaction-overlay coordinate inclusivity, signed-URL TTL
behaviour, retention purge cliff, KMS-shred ordering, RecordingsSurfaceStaging
shape retirement); 5-step migration recipe; 6-phase Strangler timeline;
verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-recordings-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-recordings-domain` | splits per BC + per layer → 22 BCs × {kernel, domain, usecase, api, adapter-*, rest, worker, sdk, app} crates under `oya-recordings-<bc>-<layer>` |

The single legacy crate hosts these shapes which split into N replacement
shapes (see `migration-from-connect.md` import-path map):

- `RecordingArchiveEntry` → `oya_recordings_recording_kernel::Recording`
- `ArchiveRetentionPolicy` → `oya_recordings_retention_policy_kernel::RetentionPolicy`
- `RecordingVariant` → splits per concern into
  `oya_recordings_media_segment_kernel::MediaSegment` (HLS) and
  `oya_recordings_export_kernel::ExportVariant` (export-bundle)
- `RecordingVariantFormat` → splits per concern into `SegmentFormat`
  (HLS/DASH/CMAF) and `ExportFormat` (mp4/mp3/wav/vtt/srt/pdf/docx)
- `RecordingsSurfaceStaging` → retired; ingest-side shape is
  `oya_recordings_recording_ingest_kernel::RecordingIngestRequest` per
  ADR-RECORDINGS-0007

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-recordings-*` crates ship in parallel | 1 | No (additive) | — |
| `oya-recordings-transcript-*` / `-redaction-*` / `-legal-hold-*` / `-ediscovery-*` / `-watermarking-*` / `-video-encode-ladder-*` net-new crates | 1 | No (net-new; no legacy counterpart) | — |
| `oya-connect-recordings-migration-adapter` shim authored | 2 | No (preserves legacy surface) | — |
| Feature-flagged canary 10→50→100 % | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-recordings-domain` crate removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (loud + immediate + CI-detectable).
- **ADR-0134** (migration policy).
- **Version bump.** Per semver on each consumer's `Cargo.toml`.
- **Sunset schedule.** 6-month advisory window from 2026-05-17.
- **Owning-axis migration ChangeSets.** axis-recordings ships migration
  ChangeSets for every internal consumer per the Churn Rule before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases —
  HG-RECORDINGS gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4
  commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5
  commands.
- [ ] No references to the deprecated system remain — `rg
  "oya_connect_recordings" --type rust` produces zero hits outside
  historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-RECORDINGS-0001..0007.
- `microservices/recordings/migration-from-connect.md` — full migration guide.
- `microservices/recordings/PRD.md` — target-state product definition.
- `microservices/recordings/runbooks/*.md` — 8 runbooks.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
