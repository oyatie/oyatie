---
doc_class: ImplementationPlan
impl_plan_id: IP-012-import-export-pipelines
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: import + export pipelines

## Intent

Land `oya-notes-import-pipeline-*` (Apple Notes / Evernote ENEX / OneNote / Notion / Bear / Obsidian per ADR-NOTES-0006) + `oya-notes-export-pipeline-*` (Markdown + frontmatter + JSON Canonical + PDF).

## Per-Format Adapters

| Crate | Source format |
|---|---|
| `oya-notes-import-pipeline-adapter-obsidian` | Obsidian vault |
| `oya-notes-import-pipeline-adapter-enex` | Evernote ENEX |
| `oya-notes-import-pipeline-adapter-apple-notes` | Apple Notes archive |
| `oya-notes-import-pipeline-adapter-onenote` | OneNote `.one` + `.onepkg` |
| `oya-notes-import-pipeline-adapter-notion` | Notion Markdown zip |
| `oya-notes-import-pipeline-adapter-bear` | Bear `.bearbk` |

## Roundtrip Test

`tests/e2e/export-roundtrip-canonical.rs` (AC-16): export → import → re-export produces byte-identical JSON Canonical.

## Acceptance Gates

```bash
cargo check -p oya-notes-import-pipeline-kernel
cargo check -p oya-notes-export-pipeline-kernel
cargo test --test obsidian-vault-roundtrip
cargo test --test enex-import
cargo test --test export-roundtrip-canonical
```

## ChangeSet metadata

```yaml
changeset_id: CS-NOTES-IP-012-import-export-pipelines
depends_on_changesets: [CS-NOTES-IP-003-note-store-kernel-domain]
parallel_safe_with_changesets: [CS-NOTES-IP-011-collab-edit-loro]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Export → import → re-export produces byte-identical JSON Canonical (RFC 8785) | `cargo test --test export-roundtrip-canonical` |
| AC-02 | Obsidian vault roundtrip preserves wikilinks + frontmatter + tag-graph | `cargo test --test obsidian-vault-roundtrip` |
| AC-03 | Evernote ENEX import preserves note + tags + attachments + creation/mod timestamps | `cargo test --test enex-import` |
| AC-04 | Markdown export preserves frontmatter YAML key order + body | `cargo nextest run -p oya-notes-export-pipeline-domain -- md_canonical` |
| AC-05 | PDF export embeds Unicode CJK glyphs (KR + JP + ZH characters present) | `cargo nextest run -p oya-notes-export-pipeline-adapter-pdf -- cjk_glyphs` |

## Build Sequence

1. Kernel: `Importer`, `Exporter`, `FormatAdapter` ports.
2. Domain: `SourceFormat` enum (obsidian/enex/apple-notes/onenote/notion/bear), `TargetFormat` enum (md/json-canonical/pdf).
3. Per-format adapters per table above.
4. Roundtrip test fixtures at `tests/fixtures/import-export/`.
5. `cargo test --test export-roundtrip-canonical`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-notes FR | FR-15 (import), FR-16 (export) |
| PRD-notes AC | AC-16 (roundtrip) |
| ADR | ADR-NOTES-0006 (import format coverage) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| ENEX format ambiguity loses inline attachments | Test fixture covers attachment + cross-link cases |
| Apple Notes archive format private | Document parser at the format-version we support; refuse unknown |
| PDF export omits CJK glyphs | Bundled noto-cjk; CJK glyph test |

## References

- RFC 8785 (JSON Canonicalization Scheme).
- Evernote ENEX format reference (Evernote Developer docs — "Evernote XML Export Format").
- Obsidian vault format documentation (Obsidian Help — "Vault").
- Notion Markdown export reference (Notion Help — "Export your content").
- ADR-NOTES-0006.

## Next IP

[`IP-013-ai-assist-and-e2e-refusal.md`](IP-013-ai-assist-and-e2e-refusal.md)
