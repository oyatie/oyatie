---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0005
title: Document round-trip fidelity (DOCX / PPTX / XLSX / PDF / HTML / Markdown / PO / XLIFF)
status: Accepted
deciders: council-architecture, axis-translate, ops-security, council-privacy
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0135, ADR-0131, ADR-TRANSLATE-0001]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/threat-model.md
  - microservices/translate/IP-009-document-translation-stack.md
  - microservices/translate/runbooks/document-round-trip-corruption.md
doc_status: published
---

# ADR-TRANSLATE-0005 — Document round-trip fidelity

## Context

Document translation — translate a DOCX/PPTX/XLSX/PDF/HTML/Markdown/PO/XLIFF/ARB/.strings/.resx/.properties file end-to-end while preserving format — is a core enterprise TMS surface. Competitors (DeepL Pro document, Google Cloud Translation document, Microsoft Translator Document Translation, Smartling, Crowdin, Phrase) all ship this. Three orthogonal axes drive the design:

1. **Format fidelity** — how much of the original formatting survives the round-trip.
2. **Security posture** — translated files come from tenants and may contain malicious content (CVE-class exploits of Office parsers; OWASP File Upload threats).
3. **Performance** — 10-page DOCX p95 ≤ 8 s (PRD).

The canonical extract-translate-merge pipeline depends on:

- **Pandoc 3.x** for Markdown / HTML / DOCX / Org / LaTeX / EPUB inter-conversion.
- **LibreOffice 24.x** (community LTS) for DOCX / PPTX / XLSX / ODT round-trip.
- Direct format parsers for XLIFF (OASIS 2.1) / TMX (LISA OSCAR 1.4) / TBX (ISO 30042) / PO (GNU gettext) / ARB (Flutter) / .strings (Apple) / .resx (.NET) / .properties (Java).
- **PDF**: text-flow extraction via `pdftotext` (poppler) or Pandoc; layout-preserving PDF re-generation via LibreOffice `--convert-to pdf`.

Industry references:

- **OOXML ECMA-376** (DOCX/PPTX/XLSX standard).
- **OASIS XLIFF 2.1**.
- **LISA OSCAR TMX 1.4**.
- **ISO 30042 (TBX)**.
- **GNU gettext PO format**.
- **Flutter ARB format**.
- **Apple .strings format**.
- **.NET .resx format**.
- **Java .properties format**.
- **gVisor** (`gvisor.dev/`) — application kernel for sandboxing untrusted code.
- **OWASP File Upload Cheat Sheet**.
- **CVE history for LibreOffice + Pandoc** (per-quarter refresh).
- **DeepL Pro Document API** (closed; surface observation).
- **Google Cloud Translation Document API**.

## Decision

### 1. Tiered fidelity contract

Documents are translated with explicit per-format fidelity tier (per `IP-009-document-translation-stack.md`):

| Tier | Formats | Guarantees |
|---|---|---|
| Tier-1 (best-effort high) | DOCX (Pandoc + LibreOffice), XLIFF 2.1, TMX 1.4, PO, ARB, .strings, .resx, .properties | Paragraph + list + table + image-anchor + style preserved; comments + tracked-changes preserved (DOCX); placeholder-preserved (PO/ARB/etc.) |
| Tier-2 (high) | PPTX, XLSX, HTML, Markdown | Slide + text-frame + image-anchor + theme (PPTX); sheet + cell + formula + format (XLSX); markup-preserved (HTML + Markdown); CSS-class preserved; code-fence preserved |
| Tier-3 (best-effort) | PDF | Text-flow preserved; layout best-effort (PDFs lose semantic structure on extract) |

Tenants see the tier label per format on the API response (per `contracts/openapi/translate-files.yaml`).

### 2. gVisor sandbox for all parsers

All document parsing (Pandoc + LibreOffice + format-specific parsers) runs inside a **gVisor** sandbox container with:

- **Seccomp profile** — minimal syscall allowlist (per `iac/helm/translate-router/seccomp/{pandoc,lo}-strict.json`).
- **No network** — pods configured with `network: none`; no DNS; no outbound.
- **Read-only root filesystem** — only tmpfs mount for in-flight files.
- **Non-root user** — `runAsNonRoot: true; runAsUser: 65532`.
- **gVisor RuntimeClass** — applied per pod via `runtimeClassName: gvisor`.
- **Per-pod tmpfs cap** — 500 MiB; cleared on pod restart.
- **Per-job time bound** — 60 s for 10-page; 300 s for 100-page.
- **Per-job file-size cap** — 100 MiB per document.

This satisfies threat-model T-06 (malicious doc) and T-07 (XLIFF XXE) defense.

### 3. Pandoc + LibreOffice pinning

- Pandoc 3.x stable; pin specific minor version (e.g., 3.1.x) per release.
- LibreOffice 24.x community LTS; pin specific patch version per release.
- Per-quarter CVE refresh + sandbox image rebuild per ADR-0133.
- Sigstore attestation on sandbox image.

### 4. XML parsing hardening (XLIFF / TMX / TBX)

All XML parsing uses `quick-xml` (Rust) with:

- **Entity resolution DISABLED** (prevents XXE).
- **DTD loading DISABLED** (prevents billion-laughs).
- **External entity reference DISABLED**.
- **Schema validation** against OASIS XLIFF 2.1 / LISA TMX 1.4 / ISO TBX 3.0 official schemas.
- **Streaming parse** for > 10 MiB inputs.

### 5. Placeholder + variable preservation per format

| Format | Placeholders preserved |
|---|---|
| ICU MessageFormat | Yes (re-parse target; assert same names + arms) |
| Mustache `{{var}}` | Yes (allow-list-based extraction + diff) |
| `%s` / `%d` / `%1$s` (C-style + positional) | Yes (allow-list extraction + diff) |
| `${var}` (shell-style) | Yes |
| `<var>name</var>` (XLIFF inline tags) | Yes (XLIFF 2.1 inline-tag preservation per spec) |
| Per-format escape sequences | Preserved (e.g., `\n` in .strings; `\\u` in JSON) |

### 6. Fidelity regression test corpus

`tests/fidelity/corpus/` contains representative documents per format with:

- Golden source file.
- Expected target structure (per-tier fidelity assertion).
- Per-release regression test: DOCX → XLIFF → MT (test engine) → DOCX → assert paragraph count + table count + style count preserved.

### 7. PDF caveat: round-trip is best-effort

PDF lacks semantic structure; extract-translate-merge → re-render via LibreOffice produces a "translated PDF" but layout will differ from original. The API response carries `fidelity_tier: 3` + `caveats: ["layout-best-effort", "text-flow-preserved"]` per PDF translation. Tenants opt in to PDF with this caveat documented.

## Alternatives Considered

### Alternative A — Use only Pandoc (no LibreOffice)

- **Pros**: single dependency; smaller image; faster.
- **Cons**: Pandoc DOCX support is good but not perfect for tables + complex styles + embedded objects; LibreOffice fills the gap.
- **Verdict**: rejected. Pandoc + LibreOffice together cover the matrix.

### Alternative B — Use only LibreOffice (no Pandoc)

- **Pros**: maximal DOCX/PPTX/XLSX fidelity.
- **Cons**: LibreOffice on Markdown / HTML / PO / XLIFF is poor or non-existent; Pandoc fills the gap.
- **Verdict**: rejected. Pandoc + LibreOffice together cover the matrix.

### Alternative C — No sandbox; trust the parsers

- **Pros**: simpler; faster (no gVisor overhead).
- **Cons**: every CVE class (LibreOffice + Pandoc + poppler) is an exploitation path; cluster compromise risk.
- **Verdict**: rejected. gVisor + seccomp + no-network mandatory.

### Alternative D — Use vendor document-translation API (DeepL Pro Document / Google Document)

- **Pros**: outsourced fidelity engineering.
- **Cons**: tenant data leaves oyatie; residency invariant broken (ADR-TRANSLATE-0004); cost; vendor-bound feature ceiling.
- **Verdict**: rejected per residency posture; in-house preferred for sovereign packs; external vendor available for tenants who explicitly DPA-permit.

### Alternative E — Native MS Office API on a Windows server

- **Pros**: best DOCX/PPTX/XLSX fidelity (native).
- **Cons**: per-license cost; Windows server ops; cross-platform OCI deploy unfriendly; sandboxing harder than gVisor.
- **Verdict**: rejected.

### Alternative F — WebAssembly-sandboxed parsers (WASI)

- **Pros**: lighter than gVisor; portable.
- **Cons**: LibreOffice not WASM-compatible (massive C++ codebase); Pandoc partial; not viable M01.
- **Verdict**: rejected M01; tracked as future research.

## Consequences

### positive

1. **Format coverage matches industry leaders** — DOCX/PPTX/XLSX/PDF/HTML/Markdown/PO/XLIFF/ARB/.strings/.resx/.properties; competitor parity per `competitor-parity-matrix.md`.
2. **Sandboxed parser execution** — gVisor + seccomp + no-network + read-only-rootfs makes parser CVEs non-exploitable as cluster-compromise vectors; no competitor isolates at this granularity.
3. **Tiered fidelity contract** — tenant expectations set per format; per-tier guarantees explicit in API.
4. **Placeholder + plural preservation across format families** — ICU + Mustache + C-style + shell-style + per-format escape sequences uniformly handled.

### negative

1. **gVisor overhead ~10 %** per `gvisor.dev/docs/architecture_guide/performance/` — folded into capacity-model.md.
2. **Pandoc + LibreOffice operational maintenance** — per-quarter CVE refresh; sandbox image rebuild; some toil.
3. **PDF fidelity is best-effort** — tenant expectations must be set; ops cost for tenant communication.

### neutral

1. **Per-format adapter (Pandoc vs LibreOffice routing)** is internal; transparent to tenant.
2. **Cold-start sandbox ~ 200 ms** — pre-warmed pool of 8 per worker pod mitigates.
3. **XLIFF / TMX / TBX schema validation** is the same posture as foundry-providers' provider-events.yaml validation; uniform.

## Validation

- `tests/fidelity/corpus/<format>/round_trip.rs` — per-format fidelity regression.
- `tests/security/fuzz_malicious_<format>.rs` — fuzz corpus per format.
- `tests/security/sandbox_escape_attempt.rs` — FM-43 covered.
- `tests/load/document_translate_10page_docx_p95_under_8s.rs` — performance.
- Per-quarter CVE refresh + sandbox image rebuild verified by lane.

## References

- OOXML ECMA-376 (DOCX/PPTX/XLSX).
- OASIS XLIFF 2.1 — `docs.oasis-open.org/xliff/xliff-core/v2.1/`.
- LISA OSCAR TMX 1.4 — `www.gala-global.org/tmx-14b`.
- ISO 30042:2019 (TBX).
- GNU gettext (PO).
- Flutter ARB.
- Apple .strings (Foundation NSLocalizedString).
- .NET .resx.
- Java .properties.
- Pandoc — `pandoc.org/`.
- LibreOffice — `www.libreoffice.org/`.
- gVisor — `gvisor.dev/`.
- OWASP File Upload Cheat Sheet.
- ADR-0135 — connect super-app expansion (parent ADR).
- ADR-0131 — per-microservice flat layout.
- ADR-TRANSLATE-0001 — engine routing.
- ICU MessageFormat — `unicode-org.github.io/icu/userguide/format_parse/messages/`.
- `microservices/translate/threat-model.md` T-06 + T-07 + F-01..F-05.
