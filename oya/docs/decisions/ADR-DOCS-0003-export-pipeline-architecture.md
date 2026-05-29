---
id: ADR-DOCS-0003
title: Export pipeline architecture — Pandoc 3.x + WeasyPrint 62.x default + Chromium-headless opt-in inside gVisor sandbox
microservice: docs
status: Accepted
date: 2026-05-17
owner: axis-docs + ops-security
deciders: council-architecture, axis-docs, ops-security, ops-sre-reliability
supersedes: []
superseded_by: []
related: [ADR-0117, ADR-0131, ADR-0133, ADR-DOCS-0002, ADR-DOCS-0006]
related_artifacts:
  - microservices/docs/PRD.md (FR-09, FR-10, AC-03, AC-09, AC-10, AC-11)
  - microservices/docs/IP-011-export-import.md
  - microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md
purpose: |
  Settle the export + import pipeline backend choice + the sandbox architecture
  for handling untrusted document content. Closes PRD AC-09 (gVisor sandbox)
  + AC-10 (PDF/A archival) + AC-03 (OOXML round-trip) + FR-09 (export) +
  FR-10 (import).
doc_status: published
---

# ADR-DOCS-0003: Export + import pipeline architecture

## Status

Accepted — 2026-05-17.

## Context

The docs µservice ships with eight export formats (PDF / PDF/A-1b / PDF/A-2u / DOCX / Markdown / HTML / EPUB 3 / LaTeX) and four import formats (DOCX / Markdown / HTML / Google-Docs format) per PRD FR-09 + FR-10. Per PRD AC-03, OOXML round-trip must preserve ≥ 95% of features on the Microsoft test corpus. Per AC-10, PDF export must pass PDF/A-1b validation (veraPDF). Per AC-11, every export must pass WCAG 2.2 AA accessibility validation (axe-core + Pa11y).

The pipeline must:
1. Convert the canonical block-tree (per ADR-DOCS-0002) to each target format with high fidelity.
2. Handle untrusted import content safely (XXE, archive bombs, embedded macros, oversized files).
3. Stay within performance budget (PDF 50-page p99 ≤ 3s; DOCX p99 ≤ 2s).
4. Be SLSA L3 supply-chain auditable.
5. Run in isolation from the rest of the µservice's network surface (any sandbox escape must be contained).

Three production-grade pipeline candidates exist for the format conversion layer:

1. **Pandoc** (Haskell, GPL-2.0+ but compiled binary has commercial-use carve-out per copyright FAQ). Converts between DOCX / Markdown / HTML / EPUB / LaTeX / many more formats. Industry-standard for document conversion; ONLYOFFICE, Collabora, HackMD use Pandoc internally.

2. **Custom Rust converters per format**. Hand-roll DOCX writer (with `docx-rs` crate), Markdown writer (with `pulldown-cmark`), etc.

3. **LibreOffice headless** (Mozilla Public License 2.0). Powers Collabora Online; converts any-to-any via the LibreOffice runtime; very high fidelity but heavyweight (~500 MB image).

For the PDF rendering layer specifically, two production-grade backends exist:

1. **WeasyPrint** (BSD-3-Clause; Python). Pure-Python HTML+CSS-to-PDF; sandboxable; deterministic; emits PDF/A natively. Used by GitLab, ReportLab competitor.
2. **Chromium-headless** (BSD-3-Clause). Use the same rendering engine as the Chrome browser; highest visual fidelity; supports modern CSS/JS; large attack surface; large memory footprint.

For the sandbox layer, three options exist:
1. **gVisor** (Apache-2.0). User-space kernel; intercepts syscalls; strong isolation; small overhead.
2. **Firecracker** (Apache-2.0; AWS). MicroVM; KVM-based; very strong isolation; higher overhead.
3. **Container with seccomp-bpf + AppArmor**. No additional runtime; weaker isolation than gVisor.

## Decision

Adopt the following four-part architecture:

### Part 1 — Format conversion: Pandoc 3.x for DOCX / Markdown / HTML / EPUB / LaTeX

Pandoc 3.6.0 LTS pinned. Handles all non-PDF formats. Industry-leader for document conversion; mature; large user-contributed corpus for round-trip testing. License acceptable for binary use (Pandoc copyright FAQ permits commercial distribution).

Concrete bindings:
- Crate: `oya-docs-export-import-adapter-pandoc` (backend-qualified per ADR-0105 Amendment 3).
- IaC: `microservices/docs/iac/helm/values.yaml` `exportImport.pandocVersion: "3.6.0"`.

### Part 2 — PDF rendering default: WeasyPrint 62.x

WeasyPrint 62.3 LTS pinned as the default PDF backend. Sandboxable; deterministic; emits PDF/A-1b + PDF/A-2u natively. Performance acceptable (50-page p99 ≤ 3s per PRD).

Concrete bindings:
- Crate: `oya-docs-export-import-adapter-weasyprint`.
- IaC: `exportImport.weasyprintVersion: "62.3"`.

### Part 3 — PDF rendering high-fidelity opt-in: Chromium-headless

Chromium-headless 120.x LTS pinned as opt-in high-fidelity alternative for tenants who need visual fidelity beyond WeasyPrint (e.g., complex CSS3 features, custom fonts not in the WeasyPrint pool). Larger attack surface; per-tenant opt-in via Cedar policy.

Concrete bindings:
- Crate: `oya-docs-export-import-adapter-chromium`.
- IaC: `exportImport.chromiumVersion: "120.0.6099.224"`.
- Enabled per-tenant via `policy/pdf-backend-tenant-override.cedar`.

### Part 4 — Sandbox: gVisor

All export + import workers run under the `runsc` runtime class (gVisor) inside Kubernetes. Per-job tmpfs (1 GiB by default). No network egress per `microservices/docs/iac/helm/templates/networkpolicy.yaml` `oya-docs-export-import-no-egress` policy. Per-job pod restart for memory hygiene. veraPDF 1.26.x runs alongside as the PDF/A validator.

Concrete bindings:
- IaC: `runtimeClassName: gvisor` on the export-import Deployment.
- gVisor runtime pinned at `2024.10.0` LTS.
- LEAN check `oya-governance-export-sandbox-conformance` validates sandbox config.

## Alternatives Considered

### Alternative A — Custom Rust converters per format

- **Pros**:
  - Single Rust runtime end-to-end.
  - Tight control over schema mapping.
- **Cons**:
  - Reinventing Pandoc costs ~12 engineer-months of work for DOCX alone; OOXML is ECMA-376 + multi-thousand-page spec with edge cases that take years to cover.
  - No external corpus advantage (Pandoc has a 20-year-old user-tested corpus).
  - Maintenance burden falls entirely on axis-docs forever.
- **Rejected reason**: build-vs-buy unfavorable; Pandoc is industry-standard for a reason.

### Alternative B — LibreOffice headless for all formats including PDF

- **Pros**:
  - Highest OOXML fidelity (best-in-class).
  - Single backend covers DOCX + ODF + PDF + HTML + Markdown.
- **Cons**:
  - ~500 MB image; ~3-5x larger than Pandoc + WeasyPrint combined.
  - LibreOffice headless requires a fake X server for some operations; container complexity.
  - Cold-start ~2-3s vs Pandoc ~200ms.
  - Sandbox escape vectors are broader (more code surface).
- **Rejected reason**: image size + cold-start incompatible with PRD performance budget; sandbox surface concern.

### Alternative C — Chromium-headless for ALL PDF (no WeasyPrint default)

- **Pros**:
  - Highest visual fidelity for all tenants.
  - Single PDF backend.
- **Cons**:
  - Chromium is the largest attack surface in oyatie's deployment footprint; sandbox escape risk is the dominant concern.
  - Chromium memory footprint dwarfs WeasyPrint (~500 MB vs ~50 MB per job).
  - Non-deterministic output (font rendering varies by GPU + system config).
  - PDF/A emission requires post-processing (Chromium doesn't natively emit PDF/A).
- **Rejected reason**: attack surface + non-determinism + PDF/A friction.

### Alternative D — Firecracker microVM sandbox (instead of gVisor)

- **Pros**:
  - Strongest isolation (KVM-based; isolated kernel).
  - AWS-validated at production scale.
- **Cons**:
  - Requires bare-metal nodes (no nested virtualisation on most managed K8s); incompatible with OCI's default node config.
  - Higher per-job overhead (~200 MB memory baseline per microVM).
  - Cold-start ~500ms vs gVisor ~50ms.
  - Operational complexity (microVM lifecycle differs from container).
- **Rejected reason**: bare-metal requirement incompatible with managed Kubernetes; cold-start budget.

### Alternative E — seccomp-bpf + AppArmor only (no gVisor)

- **Pros**:
  - Zero runtime overhead.
  - Native container deploy.
- **Cons**:
  - Significantly weaker isolation than gVisor; many CVE-class escapes possible.
  - No user-space kernel barrier; syscall surface broader.
- **Rejected reason**: insufficient isolation for handling untrusted document content (DOCX import is a notorious attack vector).

## Consequences

### Architectural

- Three adapter crates: `-adapter-pandoc`, `-adapter-weasyprint`, `-adapter-chromium`.
- gVisor runtime class registered cluster-wide; export-import workers exclusively use it.
- Per-job tmpfs (1 GiB) + no network egress + per-job pod restart.
- veraPDF 1.26.x validates PDF/A output.

### Downstream impact

1. **PRD AC-09** (gVisor sandbox) — directly satisfied.
2. **PRD AC-10** (PDF/A) — directly satisfied by WeasyPrint native PDF/A emission + veraPDF validation.
3. **ADR-DOCS-0006 fidelity matrix** — Pandoc-bounded fidelity (per ADR-DOCS-0006); best-effort fidelity with named edge-case test matrix.
4. **runbooks/export-pipeline-failure-pandoc-rollback.md** — operational guidance for backend rollback.
5. **failure-modes.md FM-04 + FM-12** — covered failure modes (Pandoc regression; gVisor seccomp escape).
6. **cost-budget.md** — export workers sized at $0.018 / PDF job (50-page); gVisor pre-warm pool of 10 sandboxes contributes $600/cell/mo.

### SLOs gaining new dimensions

- `docs.export_pdf_seconds` — p99 ≤ 3s per AC-10.
- `docs.gvisor_seccomp_violation_total` — Sev-1 alert if non-zero.
- `docs.export_pdfa_validation_pass_ratio` — per-PDF/A-variant pass rate.
- `docs.ooxml_import_fidelity_ratio` — per ADR-DOCS-0006.

### Supply-chain + security

- Pandoc + WeasyPrint + Chromium + gVisor + veraPDF all pinned to LTS versions; cargo deny / image-scan allowlist enforced.
- Major-version upgrades require: (a) 100-doc round-trip reference corpus drill, (b) AC-09 gVisor escape corpus replay, (c) PDF/A validation green, (d) WCAG 2.2 AA validation green.
- gVisor + Pandoc + WeasyPrint + Chromium each have an Ed25519-signed advisory feed subscription.

### Risk register

- **Risk**: Pandoc upstream cools / abandons. **Mitigation**: Pandoc has 20-year track record + active maintainer; alternate path is custom Rust converters or LibreOffice headless (both ADR successor-IPs).
- **Risk**: gVisor regression in a Kubernetes minor version. **Mitigation**: pinned LTS; pre-deploy escape corpus replay.
- **Risk**: Chromium CVE chain. **Mitigation**: tenant opt-in only; default to WeasyPrint; per-job pod restart bounds blast radius.
- **Risk**: WeasyPrint cannot render a specific font / CSS feature. **Mitigation**: per-tenant Chromium opt-in; runbook fallback path.

## References

- PRD `microservices/docs/PRD.md` FR-09, FR-10, AC-03, AC-09, AC-10, AC-11.
- ADR-DOCS-0002 (block-type system).
- ADR-DOCS-0006 (DOCX import fidelity).
- Pandoc — `pandoc.org`; copyright FAQ `pandoc.org/faqs.html`.
- WeasyPrint — `weasyprint.org`.
- Chromium release notes — `chromiumdash.appspot.com/`.
- gVisor — `gvisor.dev`; security model `gvisor.dev/docs/architecture_guide/security/`.
- Firecracker — `firecracker-microvm.github.io`.
- veraPDF — `verapdf.org`.
- ISO 19005-1 (PDF/A-1b); ISO 19005-2 (PDF/A-2u).
- ECMA-376 (OOXML).
- LibreOffice — `libreoffice.org` (rejected alternative reference).
