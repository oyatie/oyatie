---
doc_class: ImplementationPlan
impl_plan_id: IP-012-import-export-pipelines
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

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

## Next IP

[`IP-013-ai-assist-and-e2e-refusal.md`](IP-013-ai-assist-and-e2e-refusal.md)
