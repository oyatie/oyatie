---
id: ADR-NOTES-0006
status: Accepted
date: 2026-05-17
microservice: notes
deciders: axis-notes, council-privacy, council-architecture, gtm-product-marketing
owner: axis-notes
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-NOTES-0001
  - ADR-NOTES-0002
related_artifacts:
  - microservices/notes/PRD.md (FR-15, FR-16; AC-07, AC-08, AC-16)
  - microservices/notes/catalog/oya-notes-import-pipeline-kernel.yaml
  - microservices/notes/catalog/oya-notes-export-pipeline-kernel.yaml
  - microservices/notes/runbooks/import-pipeline-failure.md
purpose: Define the canonical portable format for notes export/import and the supported source formats for import, satisfying GDPR Art. 20 portability + Open-Standard-Notes spirit + cross-tool migration use cases.
---

# ADR-NOTES-0006: Markdown + YAML frontmatter is canonical export; JSON Canonical (RFC 8785) for byte-identical roundtrip; supported imports = Apple Notes / Evernote ENEX / OneNote / Notion / Bear / Obsidian vault

## Status

Accepted — 2026-05-17.

## Context

The PRD §FR-15 + FR-16 + AC-07/AC-08/AC-16 require:
- Import from major incumbents.
- Export to multiple formats including a byte-identical-roundtrip format for archival.
- GDPR Art. 20 portability obligation must be honoured.

Three sub-questions:

1. **Canonical format**: what does an "oyatie notes export" look like?
2. **Roundtrip format**: how does export → import → export produce byte-identical output (for verifiable archival)?
3. **Supported source formats**: which incumbents do we promise import-day-one?

The PRD's user research and the master-plan's "data portability is real, not lock-in" doctrine point to **Markdown + YAML frontmatter** as the canonical format (matches Obsidian + Bear + Logseq + Hugo + Astro + many static-site generators) augmented by **JSON Canonical (RFC 8785)** for the round-trip property (Markdown is whitespace-tolerant, so round-trip via Markdown alone won't be byte-identical).

For source imports, the seven user-research-top incumbents (Apple Notes, Evernote, OneNote, Notion, Bear, Obsidian, Standard Notes/Joplin) cover ~95 % of likely migration scenarios.

## Decision

oyatie notes adopts a **dual-format export model** with **six supported import sources at minimum-shippable-tier**:

1. **Canonical export format = Markdown + YAML frontmatter.**
   - Body is CommonMark + GFM extensions + KaTeX math.
   - Frontmatter is YAML 1.2:
     ```yaml
     ---
     note_id: 01HGZX...
     title: Foo
     context_kind: Personal | Professional
     created_at: 2026-05-17T10:30:00Z
     edited_at: 2026-05-17T11:00:00Z
     tags: [tag-a, nested/sub-tag]
     backlinks_to: [01HGZA..., 01HGZB...]
     backlinks_from: [01HGZC...]
     daily_note_date: 2026-05-17
     template_id: meeting-notes
     ---
     # Foo

     Markdown body…
     ```
   - File-per-note; ULID-derived filename `01HGZX-foo.md`; sanitised title slug.
   - Attachments referenced as `attachments/<sha256>.<ext>` (preserved alongside notes).

2. **Roundtrip format = JSON Canonical (RFC 8785).**
   - JSON document per note containing: frontmatter fields + body string + attachment refs + version-history pointers.
   - JSON Canonical guarantees byte-identical serialisation regardless of insertion order, allowing `export → import → export` to produce identical bytes.
   - File-per-note `01HGZX-foo.json`.
   - Used for archival + verifiable roundtrip (AC-16).
   - Cryptographic digest of canonical JSON included as `content_hash: sha256:...` in the frontmatter; verifiable on re-import.

3. **Both formats emitted in a single export bundle** (zip or tarball):
   ```
   notes-export-<ulid>.tar.zst
   ├── manifest.json              # bundle metadata
   ├── markdown/
   │   ├── 01HGZX-foo.md
   │   └── …
   ├── canonical/
   │   ├── 01HGZX-foo.json
   │   └── …
   ├── attachments/
   │   └── <sha256>.<ext>
   ├── audit-chain-segment.json   # Professional-tier only
   └── README.md                  # human-readable export documentation
   ```

4. **PDF as derived output**: PDF export is a *projection* (one-way) for printing / sharing; not the canonical archival form.

5. **Personal-tier (E2E) export**:
   - Client-side decryption: client SDK pulls ciphertext + keys (which only the client has), decrypts locally, emits the bundle.
   - Server NEVER builds an export bundle of plaintext for Personal-tier notes.
   - Per-user export job in `oya-notes-export-pipeline-worker` is restricted to Professional-tier; Personal-tier export is a *client-only* code path.

6. **Supported import sources (M02 ship)**:

   | Source format | Detection | Pipeline path |
   |---|---|---|
   | Obsidian vault (folder of `*.md` + `.obsidian/` dir) | `.obsidian/` dir presence | path-most-direct: Markdown + frontmatter ingest with `[[wikilink]]` resolution |
   | Evernote ENEX (XML) | `.enex` file | per-note XML → Markdown via `htmltomarkdown` + tag preservation + attachment extract |
   | Apple Notes archive (`.html` + `iCloud Notes Export`) | manifest format | iCloud HTML → Markdown sanitised; attachment extract; lockable-notes skipped if no user-supplied key |
   | OneNote (`.one` + `.onepkg`) | binary signature | OneNote → Markdown via `pandoc` + section/page hierarchy → notebook/folder |
   | Notion Markdown export (zip with Markdown + CSV for databases) | zip + database CSV detection | Markdown + frontmatter ingest; tags from database properties; subset-import (databases not full) |
   | Bear (`.bearbk` bundle) | `.bearbk` signature | Bear → Markdown + hashtag preservation |

7. **Additional sources tracked as successor-IP (subsequent-to-M02-completion)**:
   - Joplin JEX format.
   - Logseq export.
   - Standard Notes JSON backup (handles E2E with user-provided key).
   - Roam Research JSON / EDN export.

8. **Import safety**: every import runs in a sandboxed worker; Markdown sanitised (no script tags; no inline event-handlers); attachment scan via OPSWAT or ClamAV; CSP enforced for any rendered HTML preview.

9. **Conflict resolution**: if imported note has matching ULID-or-derived-id with an existing note, default behaviour is **merge-with-suffix** (`-imported-<n>.md`); user can change to overwrite in import UX.

10. **Audit-chain on import (Professional-tier only)**: `ImportJobCompleted` event emitted with seal of input manifest hash + count + tenant + user.

## Alternatives Considered

### A. Proprietary opaque export format (oyatie-only)
- Pros: simpler internal model; control.
- Cons: violates portability doctrine; users feel locked-in; GDPR Art. 20 awkward.
- Rejected.

### B. Markdown-only (no JSON Canonical)
- Pros: simplest; cleanest user-readable export.
- Cons: Markdown whitespace-tolerant means roundtrip not byte-identical; AC-16 unachievable.
- Rejected.

### C. JSON-only (no Markdown)
- Pros: roundtrip-clean; structured.
- Cons: not human-readable; users expect Markdown files; portability with sibling tools harms.
- Rejected.

### D. Markdown + JSON Canonical (this ADR's choice)
- Pros: human-readable canonical + byte-identical archival; matches Obsidian + Bear + Logseq + Hugo expectations; GDPR Art. 20 honoured cleanly.
- Accepted.

### E. ProseMirror JSON or HAST as canonical
- Pros: rich-text-native.
- Cons: not human-readable; not Markdown-native (oyatie notes is Markdown-first); sibling-tool import-painless mismatched.
- Rejected.

### F. ENEX (Evernote) as canonical export
- Pros: industry standard for notes exchange.
- Cons: XML-heavy + verbose; Evernote-flavoured; not aligned with oyatie's Markdown-first stance.
- Rejected for canonical; supported as import only.

### G. PDF as canonical
- Pros: universally readable.
- Cons: not editable / re-importable; lossy for structured data (tags, links, frontmatter).
- Rejected for canonical; supported as derived output only.

## Consequences

### Positive

- GDPR Art. 20 portability honoured by Markdown + JSON Canonical bundle.
- Users can switch to Obsidian / Bear / Hugo / Astro tomorrow without conversion.
- Day-one import from six top incumbents covers ~95 % of migration scenarios.
- AC-16 (byte-identical roundtrip via JSON Canonical) achievable.
- Personal-tier E2E posture preserved (client-side export only).

### Negative

- Two emission formats in each export bundle (Markdown + JSON Canonical) doubles bundle size. Acceptable; archives compress well via zstd.
- Import pipeline must support six formats; per-format adapter complexity. Mitigated by per-format Cargo crate with shared `ImportSourceParser` port trait.
- Notion database structures aren't full-fidelity in Markdown-only model. Documented; users notified at import-UX.

### Operational

- Crate `oya-notes-import-pipeline-{kernel,domain,usecase,api,adapter,worker,sdk,app}` enumerated.
- Crate `oya-notes-export-pipeline-{kernel,domain,usecase,api,adapter,worker,sdk,app}` enumerated.
- Per-source-format adapter crates: `oya-notes-import-pipeline-adapter-obsidian`, `…-adapter-enex`, `…-adapter-apple-notes`, `…-adapter-onenote`, `…-adapter-notion`, `…-adapter-bear`.
- Runbook `runbooks/import-pipeline-failure.md`.
- Audit-chain integration on Professional-tier imports.

## References

- RFC 8785 — JSON Canonicalisation Scheme.
- CommonMark + GFM specs.
- Obsidian vault format (`obsidian.md/help/vault`).
- Evernote ENEX format (publicly documented).
- Apple iCloud Notes export format.
- Microsoft OneNote `.one` format.
- Notion Markdown export format.
- Bear `.bearbk` bundle format.
- pandoc Universal Document Converter.
- GDPR Art. 20 (right to data portability).
- ADR-NOTES-0001 (E2E posture; client-side export for Personal-tier).
- ADR-NOTES-0002 (backlink storage; informs `backlinks_to` / `backlinks_from` in frontmatter).
- `microservices/notes/PRD.md` FR-15 + FR-16 + AC-07 + AC-08 + AC-16.
- `microservices/notes/runbooks/import-pipeline-failure.md`.
