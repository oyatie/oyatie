---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-009-document-translation-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, sandbox-isolation, fuzzing-corpus]
---

# IP-009: Document Translation stack (`oya-translate-doc-*`)

## Intent

Format-preserving document round-trip for DOCX / PPTX / XLSX / PDF / HTML / Markdown / PO / XLIFF / ARB / .strings / .resx / .properties per ADR-TRANSLATE-0005. Pandoc 3.x + LibreOffice 24.x in **gVisor** sandbox with seccomp + no-network + read-only-rootfs.

## ChangeSet boundary

Crates: `oya-translate-doc-{kernel, domain, usecase, api, adapter-pandoc, adapter-libreoffice, adapter-s3, rest, worker, sdk, app}`.

## Sandbox Architecture (per ADR-TRANSLATE-0005)

```text
doc-translate-worker pod (gVisor RuntimeClass):
├── main container (Rust; orchestrates extract → translate → merge)
├── pandoc sidecar (no-network; read-only rootfs; seccomp profile pandoc-strict.json)
├── libreoffice sidecar (no-network; read-only rootfs; seccomp profile lo-strict.json; --headless)
└── tmpfs for in-flight files (cleared on pod restart; max 500 MiB per pod)
```

Sandbox enforcement:
- gVisor (per `cell` posture).
- Seccomp profiles in `iac/helm/translate-router/seccomp/{pandoc,lo}-strict.json` — minimal syscall set.
- `network: none` (no egress).
- `readOnlyRootFilesystem: true`.
- `runAsNonRoot: true; runAsUser: 65532`.
- File-size cap: 100 MiB per document.
- Time-bound per job: 60 s for 10-page DOCX; 300 s for 100-page.

## Round-Trip Fidelity Tiers (ADR-TRANSLATE-0005)

| Tier | Formats | Guarantees |
|---|---|---|
| Tier-1 (best-effort high) | DOCX (Pandoc + LibreOffice) | Paragraph + list + table + image-anchor + style preserved; comments + tracked-changes preserved |
| Tier-1 | XLIFF 2.1 | Lossless segment-level; placeholders preserved |
| Tier-1 | TMX 1.4 | Lossless TU |
| Tier-2 (high) | PPTX | Slide + text-frame + image-anchor + theme preserved; SmartArt best-effort |
| Tier-2 | XLSX | Sheet + cell + formula + format preserved; chart best-effort |
| Tier-2 | HTML | Markup-preserved; CSS-class preserved |
| Tier-2 | Markdown | Markup-preserved; code-fence preserved |
| Tier-3 (best-effort) | PDF | Text-flow preserved; layout best-effort |
| Tier-1 | PO / ARB / .strings / .resx / .properties | Lossless key-value round-trip |

## Test Plan

| Test | Verifies |
|---|---|
| `test_docx_extract_paragraph_count_preserved` | Tier-1 |
| `test_xlsx_formula_preserved` | Tier-2 |
| `test_pdf_text_flow_preserved` | Tier-3 |
| `test_xliff_lossless_round_trip` | Tier-1 |
| `test_po_msgid_msgstr_lossless` | Tier-1 |
| `test_sandbox_no_network_egress` | gVisor enforcement |
| `test_sandbox_seccomp_blocks_disallowed_syscall` | seccomp |
| `test_sandbox_readonly_rootfs` | enforcement |
| `tests/security/fuzz_malicious_docx.rs` | T-06 fuzz corpus |
| `tests/security/fuzz_malicious_pdf.rs` | T-06 fuzz corpus |
| `tests/security/fuzz_xliff_xxe_attempts.rs` | F-01 / F-03 |
| `tests/security/sandbox_escape_attempt.rs` | FM-43 covered |
| `tests/load/document_translate_10page_docx_p95_under_8s.rs` | AC-09 |

## Halt Conditions

- Sandbox observed making network call.
- Seccomp violation observed.
- Any fuzz seed causes worker pod crash > 0 in steady state.

## Next IP

[`IP-010-bulk-translate-stack.md`](IP-010-bulk-translate-stack.md)
