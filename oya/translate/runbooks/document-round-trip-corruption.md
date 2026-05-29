---
doc_class: Runbook
title: Document translation round-trip — corruption / fidelity-class breach
microservice: translate
severity: "Sev-2 (single-doc / fidelity-class breach) / Sev-1 (systemic doc adapter failure)"
status: Accepted
owner_team: axis-translate + ops-sre-reliability + ops-security
date: 2026-05-18
related_artifacts:
  - microservices/translate/failure-modes.md (FM-40..FM-44)
  - microservices/translate/decisions/ADR-TRANSLATE-0005-document-round-trip-fidelity.md
  - microservices/translate/slos/document-translate-latency.openslo.yaml
  - microservices/translate/threat-model.md (T-DOC-*)
doc_status: published
---

# Runbook: Document translation round-trip — corruption

## Trigger

Any of:

- FM-40 (DOCX round-trip emits unreadable output: Word fails to open).
- FM-41 (PPTX round-trip drops slide-master / placeholder map).
- FM-42 (XLSX round-trip corrupts formula references).
- FM-43 (PDF round-trip loses tables / form fields beyond fidelity-class budget per ADR-TRANSLATE-0005).
- FM-44 (Pandoc / LibreOffice gVisor sandbox panic; document worker pod restarts).
- Tenant escalation: > 5 corrupted-document reports in 24 h on the same adapter version.
- Security alert: malicious DOCX/PDF exploits gVisor escape attempt detected.

## Severity

| Symptom | Severity |
|---|---|
| Single tenant single-format issue (e.g., DOCX only) | Sev-2 |
| Multi-format same-pack systemic | Sev-1 |
| gVisor escape attempt detected | Sev-1 (P0; ops-security) |
| Fidelity-class breach without corruption (recoverable post-edit) | Sev-3 |

## Symptoms

- `oya_translate_doc_roundtrip_failure_total{format="docx"}` rate increase.
- `oya_translate_doc_fidelity_class_breach_total{class="A"|"B"|"C"}` non-zero (per ADR-TRANSLATE-0005 classes).
- `oya_translate_doc_pandoc_panic_total` non-zero.
- `oya_translate_doc_libreoffice_panic_total` non-zero.
- Sandbox audit: `runtime.io.kubernetes.cri.runtime-class=gvisor` pod restart cycles.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via metrics + check format affected (DOCX / PPTX / XLSX / PDF / HTML / Markdown) | ≤ 3 min |
| 2 | Sample 3 affected output files; verify with `pandoc --check <file>` or LibreOffice `soffice --convert-to docx --headless` | ≤ 10 min |
| 3 | Pin Pandoc adapter to previous version: `cargo run -p oya-dev-cli -- translate pin-adapter --backend pandoc --version <prev>` | ≤ 5 min |
| 4 | Pin LibreOffice adapter (if affected): `cargo run -p oya-dev-cli -- translate pin-adapter --backend libreoffice --version <prev>` | ≤ 5 min |
| 5 | If gVisor escape attempt: isolate node `kubectl cordon <node>`; engage ops-security; preserve forensics | ≤ 5 min |
| 6 | Halt document-translate worker for affected format `cargo run -p oya-dev-cli -- translate halt-doc-format --format <fmt>` | ≤ 5 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Pandoc upstream version regression | timing matches adapter update | bisect upstream version |
| LibreOffice CVE-driven sandbox upgrade | gVisor seccomp profile change | verify seccomp profile matches ADR-TRANSLATE-0005 §"sandboxing" |
| Format-specific edge case (e.g., DOCX with embedded objects) | failures cluster on document metadata feature | reproduce in CI fixture; file format-specific regression test |
| Placeholder-preservation regression | ICU MessageFormat / variables broken after re-merge | rollback placeholder-preservation module |
| gVisor escape attempt | runtime-class violation in audit log | ops-security ownership; forensic snapshot |
| Per-tenant attack (oversized doc) | one tenant submits 1000 docs > 100 MB | rate-limit per `cost-budget.md` |

## Resolution Path

### Path A — Pin previous adapter

1. Identify previous adapter version: `helm get values translate -n translate | grep -E 'pandoc|libreoffice'`.
2. Pin: `cargo run -p oya-dev-cli -- translate pin-adapter --backend <pandoc|libreoffice> --version <prev>`.
3. Reapply Helm chart.
4. Verify `tests/integration/doc_roundtrip_fidelity_class_a.rs` re-runs green.
5. Emit `DocumentAdapterPinned` audit event.

### Path B — Format-specific halt + tenant comms

1. Halt the failing format only (DOCX/PPTX/XLSX/PDF/HTML/Markdown).
2. Other formats continue serving.
3. Surface "DOCX translate in degraded mode" banner to tenants.
4. Backfill engineering ticket; resume when fix ships.

### Path C — gVisor escape attempt (P0)

1. Cordon affected node; preserve forensics.
2. Spin replacement pod on different node.
3. Audit-chain emits `SandboxEscapeAttempt{node, pod, adapter, ts}`.
4. ops-security takes ownership; reverse-engineer attack vector.
5. Patch gVisor + seccomp + Pandoc/LibreOffice CVE chain.
6. Tenant comms IF data class exfiltrated.
7. Per `incident-response.md`: regulator notification within 72 h if user-data potentially exfiltrated (GDPR Art. 33).

## Fidelity Class Reference (ADR-TRANSLATE-0005)

| Class | Format | Round-trip guarantee |
|---|---|---|
| A — high-fidelity | DOCX, XLSX, PPTX, Markdown, HTML | 95 % structural fidelity; 100 % text fidelity |
| B — medium-fidelity | PDF (text-extract path) | 80 % structural fidelity; layout may shift; 100 % text fidelity |
| C — text-only | PDF (image-only with OCR), legacy DOC | Text extraction only; no layout preservation |

Fidelity class breach defined as: structural-fidelity falling below class floor → `oya_translate_doc_fidelity_class_breach_total` increments.

## Verification Commands

```bash
# Fidelity class A regression
cargo run -p oya-dev-cli -- translate verify-doc-fidelity --fidelity-class A --window 30m

# Sandbox panic zero
cargo run -p oya-dev-cli -- translate verify-doc-sandbox-stable --window 1h

# Sample round-trip on fixture set
cargo run -p oya-dev-cli -- translate fixture-roundtrip-suite --pack <p>
# expects: 100% green on 50-document fixture set
```

## Rollback Path

If Path A fails, fall back to text-only extraction (Class C path) and surface "document translation in text-only mode" banner. Tenants can request format-preserving re-translate when adapter is recovered.

## Post-Incident

- Postmortem within 5 business days.
- If P0 sandbox escape: external ops-security review + ADR amendment to gVisor hardening.
- If fidelity-class breach repeated: class-floor renegotiation with tenants; PRD update.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-us-healthcare | HIPAA — corrupted PHI documents may trigger HHS OCR breach review |
| pack-eu | GDPR Art. 33 if exfiltration suspected via sandbox escape |
| pack-kr | KR PIPA Art. 34 + KISA notification if KR-user data class affected |
| pack-cn-stub | document content boundary: in-house only; vendor escape impossible |

## Named Industry Sources

- Pandoc 3.x — `pandoc.org/`.
- LibreOffice 24.x security advisories — `www.libreoffice.org/about-us/security/advisories/`.
- gVisor — `gvisor.dev/docs/architecture_guide/security/`.
- OWASP File Upload Cheat Sheet — `cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html`.
- ECMA-376 (OOXML) — `www.ecma-international.org/publications-and-standards/standards/ecma-376/`.
- ISO 32000-2 (PDF 2.0).
- OASIS ODF 1.3.
- ICU MessageFormat 2 — `unicode-org.github.io/icu/userguide/format_parse/messages/`.
- NIST SP 800-53 SC-44 (sandboxing).

## References

- ADR-TRANSLATE-0005 (document round-trip fidelity).
- `microservices/translate/threat-model.md` T-DOC-*.
- `microservices/translate/slos/document-translate-latency.openslo.yaml`.
- `microservices/translate/iac/helm/translate/templates/networkpolicy.yaml` (sandbox egress lockdown).
