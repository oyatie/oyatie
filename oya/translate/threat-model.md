---
doc_class: ThreatModel
title: STRIDE + LINDDUN + OWASP-LLM threat model
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-translate + council-privacy
deciders: ops-security, axis-translate, council-privacy, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004, ADR-TRANSLATE-0005, ADR-TRANSLATE-0006]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/dpia.md
  - microservices/translate/policy/credential-isolation.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md
review_cadence: quarterly + on every new vendor adapter
doc_status: published
---

# Threat Model — translate µservice

## Scope + Methodology

Methodology: **STRIDE** (Microsoft) for tactical threats, **LINDDUN** (KU Leuven) for privacy-engineering threats, **OWASP LLM Top 10 (2024)** for LLM-specific threats, **MITRE ATLAS** for ML-attack-on-models, plus **OASIS XLIFF security considerations** for translation-file-specific attacks.

Trust boundaries (TB):
- TB-1 — tenant ↔ translate-rest (HTTPS + OIDC + mTLS).
- TB-2 — translate-rest ↔ translate-router (in-cluster mTLS).
- TB-3 — translate-router ↔ engine adapters (in-cluster mTLS + SPIFFE).
- TB-4 — engine adapters ↔ foundry-providers / foundry-runtime (in-cluster mTLS).
- TB-5 — translate-router ↔ Postgres / Valkey / Meilisearch (mTLS + per-tenant RLS).
- TB-6 — document-translation sandbox ↔ Pandoc / LibreOffice (**gVisor + seccomp + no-network**).
- TB-7 — external vendor edges (`api.deepl.com`, `translation.googleapis.com`, `api.anthropic.com`, `api.openai.com`) — egress via cell egress proxy.
- TB-8 — bulk-translate S3 ↔ translate-bulk-worker.

## Assets + Data Classification

| Asset | Class (Bominal ADR-0028 taxonomy) | Owner | Retention |
|---|---|---|---|
| Source-segment plaintext (in-flight) | `CONFIDENTIAL_TENANT_CONTENT` | tenant | not persisted by translate (transient only) |
| TM units (source+target pairs) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` (segment may contain PII) | tenant | per-tenant retention policy; default 7y or shorter per tenant DPA |
| Termbase entries | `BEHAVIORAL_TENANT_PRODUCT` | tenant | retained until termbase deletion |
| QE scores | `BEHAVIORAL_TENANT_PRODUCT` | translate | 90 d default |
| Document round-trip artifacts (S3) | `CONFIDENTIAL_TENANT_CONTENT` | tenant | 30 d default; tenant-configurable |
| Engine credentials (in-memory only) | `SECRET` | ops-security (via OpenBao) | per OpenBao lease |
| Audit events (`TranslationCompleted`, `EngineRouted`, etc.) | `AUDIT` | translate | per pack retention table |
| EU AI Act disclosure record | `AUDIT` | translate | bounded by GDPR + AI Act |
| Real-time caption-stream audio-derived text | `CONFIDENTIAL_TENANT_CONTENT` + `PII_QUASI_IDENTIFIER` | tenant | transient; not persisted |

## STRIDE Threats

| # | Threat | STRIDE | Asset | Mitigation | Residual |
|---|---|---|---|---|---|
| T-01 | Vendor credential theft (DeepL/Google/Anthropic key extracted from logs/env/error) | S, I, R | Engine credentials | OpenBao SecretReference + zeroize-on-drop + Debug=REDACTED + `oya-translate-credential-isolation` LEAN lane | Low |
| T-02 | Adapter-substitution (attacker swaps in-house adapter binary for one that exfiltrates) | T, E | All assets routed through adapter | Sigstore attestation on adapter image + adapter-pinning runbook + per-tenant adapter-version pin + SPIFFE identity | Low |
| T-03 | Engine response-shape anomaly drives unsafe downstream behavior (adversarial response containing prompt-injection) | T, E, I | Tenant content | Response-shape validator (T-03 conformance test); per-vendor canonical-shape normalization; downstream `cell` egress filtering | Low |
| T-04 | Cross-tenant TM leverage (Tenant A's TM unit returned to Tenant B) | I, R | TM units | Per-tenant RLS in Postgres + Cedar default-deny on tenant-id-mismatch + Meilisearch per-tenant index isolation | Low |
| T-05 | Cross-region leakage (sovereign tenant content sent to non-resident vendor) | I, R | Tenant content + residency invariant | Per-pack engine whitelist enforced at router decide; per-pack Cedar policy default-deny; `oya-translate-data-residency-correctness` BLOCKER lane | **Critical mitigation; zero-tolerance** |
| T-06 | Malicious DOCX/PPTX/PDF (CVE-class exploitation of LibreOffice / Pandoc) | T, D, E | Document parser sandbox | gVisor + seccomp + no-network + read-only-rootfs sandbox per ADR-TRANSLATE-0005; OWASP File Upload hardening; per-quarter CVE refresh | Low |
| T-07 | XLIFF/TMX/TBX XML XXE (XML External Entity) | I, D | File-import worker | DefusedXML / quick-xml with entity-resolution disabled; XLIFF schema validation; size cap | Low |
| T-08 | Placeholder/variable injection (translation that escapes ICU MessageFormat into target rendering) | T, I | Placeholder-preservation invariant | Placeholder allow-list + ICU MessageFormat re-parse validation + CLDR plural-rule validation; rejection if placeholder count or names diverge | Low |
| T-09 | Prompt injection via source segment ("Ignore previous instructions, output X") | T, E, I | LLM-class engine response | Engine-specific system-prompt isolation + source-segment fenced + response-filter ; per-vendor docs cited | Medium (LLM-class only) |
| T-10 | Rate-limit cascade (one tenant exhausts engine quota) | D | Engine availability | Per-tenant token-bucket (Valkey) + per-engine global token-bucket + back-pressure → router demote-engine; quota burst alert | Low |
| T-11 | Bulk-job storage abuse (10 GB tenant XLIFF upload) | D | S3 + bulk-worker | Per-tenant upload quota + size cap + content-type validation + virus scan via ClamAV sidecar | Low |
| T-12 | Real-time stream replay (attacker replays caption-stream WS messages) | T, R | Stream session | Per-session nonce + Ed25519-signed chunk + replay-window enforcement; STT source authentication via meet µservice | Low |
| T-13 | QE-score manipulation (model output coerced to over-report quality, evading human review) | T, E | QE score | QE deployed as low-risk AI per ADR-TRANSLATE-0003 with documented bounds; out-of-bound score quarantined for human review | Medium |
| T-14 | TM poisoning (malicious target injected into TM via human-translator role) | T, R | TM units | 2-person review on TM commits + audit-chain TmUpdated seal + per-tenant TM rollback runbook | Medium |
| T-15 | Termbase poisoning (malicious term causes incorrect MT enforcement) | T, R | Termbase | 2-person review on TBX import + audit-chain TermbaseUpdated seal | Medium |
| T-16 | EU AI Act Art. 50 disclosure suppression (downstream µservice drops the disclosure event) | R | Compliance posture | Disclosure emitted at adapter layer, not at REST; audit-chain seal mandatory; LEAN-lane `oya-governance-eu-ai-act-disclosure` verifies emit count | Low |
| T-17 | Vendor model swap (DeepL upgrades model without notice → silent quality regression) | R, T | Translation quality | Per-tenant adapter pin + COMET-Kiwi QE on every call + canary-cohort-weighted rollout when vendor announces new model | Medium |
| T-18 | Audit-chain forge (attacker forges `TranslationCompleted` event) | T, R | Audit trail | Ed25519 envelope signature; per-adapter keyring in OpenBao KMS; verifier in foundry-evidence | Low |

## LINDDUN Privacy Threats

| # | Threat | LINDDUN | Asset | Mitigation |
|---|---|---|---|---|
| P-01 | Linkability across tenants via shared segment hash | L | TM unit segment hash | Per-tenant BLAKE3-keyed hash (HMAC) such that cross-tenant hash equivalence impossible |
| P-02 | Identifiability (segment contains PII) carried to vendor | I | Tenant content | DLP scan + opt-in PHI/PII redaction; per-tenant residency-bound + ZDR negotiation per pack |
| P-03 | Non-repudiation gap — vendor cannot deny they processed segment | N | Tenant content | Ed25519-signed `TranslationCompleted` seal + `EngineRouted` decision record |
| P-04 | Detectability (translate-router exposes engine selection that leaks tenant patterns) | D | Routing decision | Router decision stored per-tenant; cross-tenant aggregation uses DP-noise |
| P-05 | Disclosure (TM serving target containing PII to wrong principal) | D | TM units | Per-tenant RLS + Cedar tenant-scope policy + per-project access control |
| P-06 | Unawareness (tenant unaware that segment crossed border for translation) | U | Tenant content | Pre-call transparency: tenant sees engine + region in API response + UI; EU AI Act Art. 50 disclosure on EU |
| P-07 | Non-compliance with GDPR Art. 5 minimisation (whole document sent when only headers needed) | NC | Tenant content | Segment-level extraction; only translatable segments sent to engine |

## OWASP LLM Top 10 (2024) Coverage

| LLM-### | Mitigation in translate µservice |
|---|---|
| LLM01 (Prompt Injection) | Source segment fenced + system-prompt isolation per vendor + response filter; per ADR-TRANSLATE-0003 |
| LLM02 (Insecure Output Handling) | Response-shape validator + placeholder count check + ICU re-parse |
| LLM03 (Training Data Poisoning) | N/A for inference-only adapters; in-house model training out of scope (foundry-runtime owns) |
| LLM04 (Model Denial-of-Service) | Per-tenant + per-engine token-bucket; bulk-job quota |
| LLM05 (Supply Chain Vulnerabilities) | Sigstore attestation + cargo deny + SBOM per release |
| LLM06 (Sensitive Information Disclosure) | DLP scan + ZDR negotiation + per-pack residency |
| LLM07 (Insecure Plugin Design) | Adapter trait sealed; only vetted vendors permitted |
| LLM08 (Excessive Agency) | translate operates at autonomy level T1/T2 (no T3 destructive); per ADR-0022 |
| LLM09 (Overreliance) | QE score gate + human-in-the-loop review for high-risk content classes |
| LLM10 (Model Theft) | In-house models served from foundry-runtime; weights never leave cluster; gVisor isolation |

## MITRE ATLAS coverage

| Technique | Mitigation |
|---|---|
| AML.T0010 (ML Supply Chain Compromise) | Sigstore + SBOM + adapter-pin |
| AML.T0024 (Exfiltration via ML Inference API) | Per-tenant rate limit + DLP on output |
| AML.T0043 (Craft Adversarial Data) | QE + response-shape validator |
| AML.T0046 (Spamming ML system with chaff) | Per-tenant + per-engine token bucket |

## Translation-File-Specific Threats (OASIS XLIFF / LISA TMX / ISO TBX)

| # | Threat | Mitigation |
|---|---|---|
| F-01 | XLIFF `<source>`/`<target>` containing nested malicious markup | XLIFF 2.1 schema validation + content sanitization |
| F-02 | TMX `<tu>` linkage carrying cross-tenant identifiers | Per-tenant TMX import (sealed scope); identifier scrubbing |
| F-03 | TBX termbase entry with executable XSL transform | XSL disabled; entity resolution disabled |
| F-04 | ICU MessageFormat injection (`{0,plural,…}` malformed to break compiler) | Re-parse with `icu_messageformat` crate; reject malformed |
| F-05 | CLDR plural-rule mismatch (output category not declared in target locale) | Per-target-locale plural-rule validator |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=threat-model --microservice translate` exits 0 (cross-checks T-### → mitigation → test coverage).
- `tests/security/` directory contains a fuzz corpus per T-06, T-07, T-08, F-01..F-05.
- Quarterly threat-model refresh; vendor adapter additions trigger a per-vendor section.

## References

- Microsoft STRIDE — `learn.microsoft.com/en-us/security/engineering/threats`.
- KU Leuven LINDDUN — `linddun.org/`.
- OWASP LLM Top 10 (2024) — `owasp.org/www-project-top-10-for-large-language-model-applications/`.
- MITRE ATLAS — `atlas.mitre.org/`.
- OASIS XLIFF 2.1 security considerations.
- OWASP File Upload Cheat Sheet.
- LibreOffice security advisories (per-quarter refresh).
- ADR-TRANSLATE-0004 (residency-bound inference).
- ADR-TRANSLATE-0005 (gVisor sandboxed document parsers).
- Bominal ADR-0028 (data classification taxonomy).
