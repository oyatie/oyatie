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


## A. Problem
`IP-012: import + export pipelines` is not a generic implementation packet; it closes the `012 import export pipelines` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Portability is implemented as format-specific import/export adapters with canonical Markdown/frontmatter and JSON Canonical roundtrips as acceptance evidence. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/import-pipeline-failure.md` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `012 import export pipelines` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
