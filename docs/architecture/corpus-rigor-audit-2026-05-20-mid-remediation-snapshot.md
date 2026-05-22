---
audit_id: CORPUS-RIGOR-AUDIT-MID-REMEDIATION-SNAPSHOT-2026-05-20
audit_only: true
created_by: codex-corpus-rigor-audit-mid
created_on: 2026-05-20
source_prior_audit: docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md
output_path: docs/architecture/corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md
vcs_claim: ./bin/oya vcs claim --agent codex-corpus-rigor-audit-mid --intent corpus-rigor-audit-mid-remediation-snapshot docs/architecture
---

# Corpus Rigor Audit - 2026-05-20 Mid Remediation Snapshot

## Final Summary

Verdict: **REVISE-CORPUS-WIDE-AGAIN**.

Basis: the corpus made material progress after the prior post-Wave-3-G audit. Contracts are now fully version-clean by local scan, doctrine loop text is gone, every microservice clears the 100-artifact floor, capability-tier registry files exist, manifest audit fields are broad, and reverse cross-reference density is high for the sampled load-bearing ADRs. The corpus is still not rigor-complete because ADR-0321 declares 165 vendor dossiers while only 85 sections exist, only 58/165 meet the strict substance heuristic, 51/70 sampled microservice IP sets fail the requested 200-line/10-ADR bar, only 33/70 services have authored per-service ADR files, and most persona dossiers still lack the requested 2026-05-20 substance marker.

### Top-10 Highest-Leverage P0/P1 Findings

| Rank | Severity | Confidence | Finding | Evidence |
| --- | --- | --- | --- | --- |
| 1 | P0 BLOCKING | HIGH | ADR-0321 declares 165 vendor dossiers but exposes only D-001..D-085; D-086..D-165 are absent, so the declared corpus cannot be approved. | `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md; missing range D-086..D-165` |
| 2 | P1 NEEDS-FIX | HIGH | ADR-0321 substance progress is 35.2% (58/165 complete). D-001..D-050 are bespoke but only 44/50 meet the strict 120-line floor. | `D-006, D-009, D-010, D-011, D-012, D-013 are below 120 lines` |
| 3 | P1 NEEDS-FIX | HIGH | D-051..D-165 is not merely scaffold: D-051..D-064 are substantive, D-065..D-085 are thin scaffold, and D-086..D-165 are missing. | `D-051..D-064 complete; D-065..D-085 12-19 lines; D-086..D-165 absent` |
| 4 | P1 NEEDS-FIX | HIGH | IP slice substance is uneven: only 19/70 services pass the seeded two-IP sample. | `seed=20260520; pass requires >=200 lines, bespoke signal, >=10 unique ADR refs` |
| 5 | P1 NEEDS-FIX | HIGH | Per-microservice decision suites are incomplete: 33/70 services have authored decisions/ ADR files. | `microservices/*/decisions scan` |
| 6 | P1 NEEDS-FIX | HIGH | Doc-suite surfaces are incomplete even though artifact counts are healthy: 22/70 services have all 8 requested surfaces. | `surfaces: capability-tiers/onboarding/faqs/tutorials/benchmarks/migration-playbooks/reference-implementations/decisions` |
| 7 | P1 NEEDS-FIX | HIGH | Journey substance is partial after j161: j151 is under the 200-line bar, j162 is partial, and j163..j175 directories are empty or have zero audited files. | `docs/user-journeys/j151..j175` |
| 8 | P1 NEEDS-FIX | HIGH | Persona marker pass meets the absolute 30+ threshold but not corpus-wide coverage: 60/129 persona dossiers carry the exact marker. | `docs/personas/*.md` |
| 9 | P1 NEEDS-FIX | MED | BYOK authority docs are clean, but 112/572 BYOK-bearing artifacts lack a local provider/encryption/ADR disambiguation signal by heuristic scan. | `ADR-0255 §D-4 and ADR-0251 §D-10 pass; downstream docs/IPs still need local clarity` |
| 10 | P1 NEEDS-FIX | HIGH | Manifest audit fields landed broadly, but naming_justifications are not 100%: 69/70; detection is the miss. | `microservices/detection/manifest.json` |

### Workstream Completion Snapshot

| Workstream | Completion | Severity | Confidence | Notes |
| --- | ---: | --- | --- | --- |
| ADR-0321 vendor dossiers | 35.2% | P0 BLOCKING | HIGH | 58/165 complete; 85/165 present; missing D-086..D-165 |
| D-001..D-050 bespoke substance bar | 88.0% | P1 NEEDS-FIX | HIGH | 44/50 pass strict line+anchor bar; all 50 carry bespoke anchors |
| Long-form doctrine clause-loop cleanup | 100.0% | P2 IMPROVE | HIGH | Both target docs have zero Thesis clause and zero Problem clause hits |
| Microservice artifact floor | 100.0% | P2 IMPROVE | HIGH | All 70 service dirs have >=100 artifacts |
| Microservice all-surface doc suite | 31.4% | P1 NEEDS-FIX | HIGH | Only services with all 8 requested sub-surfaces pass |
| Per-microservice authored ADRs | 47.1% | P1 NEEDS-FIX | HIGH | Decision subdir with at least one markdown ADR |
| IP sample substance | 27.1% | P1 NEEDS-FIX | HIGH | Seeded sample of 2 IP files per service |
| Journey j151..j158 substance | 87.5% | P1 NEEDS-FIX | HIGH | j151 is below line/file bar; j152..j158 pass |
| Journey j159+ substance | 17.6% | P1 NEEDS-FIX | HIGH | j159..j161 pass; j162 partial; j163..j175 empty |
| Persona substance-marker coverage | 46.5% | P1 NEEDS-FIX | HIGH | Absolute 30+ threshold met, corpus-wide marker coverage incomplete |
| Capability-tier registry | 100.0% | P2 IMPROVE | HIGH | Registry exists with tier definition files plus microservice and vendor mappings |
| Manifest naming_justifications | 98.6% | P1 NEEDS-FIX | HIGH | 69/70 pass |
| Manifest audit fields | 100.0% | P2 IMPROVE | HIGH | 70/70 pass |
| OpenAPI 3.2.0 conformance | 100.0% | P2 IMPROVE | HIGH | 96/96 pass |
| AsyncAPI 3.1.0 conformance | 100.0% | P2 IMPROVE | HIGH | 106/106 pass |
| proto3 conformance | 100.0% | P2 IMPROVE | HIGH | 112/112 pass |
| BYOK local disambiguation | 80.4% | P1 NEEDS-FIX | MED | Primary authority docs pass; downstream artifacts need local disambiguators |
| Reverse cross-reference web sample | 100.0% | P2 IMPROVE | HIGH | 8/8 sampled load-bearing ADRs have >=8 reverse refs |

## Method

- Audit-only scope: no source remediation was performed; the only permitted repository write is this audit document.
- Claim evidence: `./bin/oya vcs claim --agent codex-corpus-rigor-audit-mid --intent corpus-rigor-audit-mid-remediation-snapshot docs/architecture` returned `oya vcs claim accepted`.
- Deterministic IP sampling: Python random seed `20260520`, two IP markdown files per microservice where available.
- ADR-0321 substance heuristic: complete means section length >=120 lines and at least 8 of 10 bespoke anchors: vendor data model, primary APIs, UX, Cedar, ontology, workflow, migration, failure modes, capability tier, naming justification.
- Contract conformance heuristic: OpenAPI file must declare `openapi: 3.2.0`; AsyncAPI file must declare `asyncapi: 3.1.0`; proto must declare `syntax = "proto3"`.
- BYOK heuristic confidence is MED because keyword scans can overcount historical audit docs and schema examples; primary ADR/runbook separation was also checked directly.

## Findings Register

| ID | Severity | Confidence | Workstream | Finding | Required correction |
| --- | --- | --- | --- | --- | --- |
| F-001 | P0 BLOCKING | HIGH | ADR-0321 | D-086..D-165 are absent while frontmatter declares 165 dossiers. | Author or explicitly retire D-086..D-165; do not leave count/declaration drift. |
| F-002 | P1 NEEDS-FIX | HIGH | ADR-0321 | Six D-001..D-050 sections are bespoke but below strict line bar. | Backfill D-006, D-009, D-010, D-011, D-012, D-013 to >=120 lines without templating. |
| F-003 | P1 NEEDS-FIX | HIGH | ADR-0321 | D-065..D-085 are thin scaffold despite having section headings. | Backfill each to the 10-anchor long-form pattern or mark as intentionally out-of-scope. |
| F-004 | P1 NEEDS-FIX | HIGH | Doctrine docs | Clause-loop cleanup passes; no blocking issue remains. | Keep zero-hit grep guard in future doctrine waves. |
| F-005 | P1 NEEDS-FIX | HIGH | Microservice doc suite | 48/70 services are missing at least one requested doc-suite surface. | Fill missing surface directories/files using service-specific content; prioritize decisions plus capability/onboarding docs. |
| F-006 | P1 NEEDS-FIX | HIGH | Per-service ADRs | 37/70 services do not have authored decisions/ ADR files. | Add service-local decision records or explicitly map each service to central ADR ownership. |
| F-007 | P1 NEEDS-FIX | HIGH | IP substance | 51/70 services fail the seeded IP sample. | Bring non-journey IPs to >=200 lines and >=10 ADR citations or split scaffold files out of substance claims. |
| F-008 | P1 NEEDS-FIX | HIGH | Journeys | j151 is under line/file bar; j162 partial; j163..j175 empty. | Backfill j151 and j162; author or remove empty j163+ shells from readiness claims. |
| F-009 | P1 NEEDS-FIX | HIGH | Personas | Only 60/129 persona dossiers carry the required marker. | Mark and substantiate remaining persona dossiers, or scope the marker to a named subset. |
| F-010 | P2 IMPROVE | HIGH | Capability tiers | Registry exists and maps services/vendors; file name `tier-defs` is not literal but definitions are present. | Optional: add a tier-defs alias/index if validators expect that exact term. |
| F-011 | P1 NEEDS-FIX | HIGH | Manifests | detection manifest lacks naming_justifications. | Patch detection manifest with naming justifications matching the audit field pattern. |
| F-012 | P2 IMPROVE | HIGH | Contracts | OpenAPI, AsyncAPI, and proto scans are 100% conformant. | Keep contract version regression guard active. |
| F-013 | P1 NEEDS-FIX | MED | BYOK | 112 BYOK-bearing artifacts lack local provider/encryption/ADR disambiguation signals. | Replace bare BYOK references with provider-BYOK or encryption-BYOK plus ADR-0255/ADR-0251 citation. |
| F-014 | P2 IMPROVE | HIGH | Cross-reference web | 8/8 sampled load-bearing ADRs have strong reverse reference counts. | Keep reverse-link maintenance; no sampled blocker. |

## Appendix A - ADR-0321 Vendor Dossier Detail

- ADR file: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Total lines: 10025
- Declared vendor_dossier_count: 165
- Present sections: 85/165
- Complete sections: 58/165 (35.2%)
- Missing ranges: D-086..D-165

### Appendix A.001 D-001
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Sales Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:81` through line 203; 123 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.002 D-002
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Service Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:204` through line 332; 129 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.003 D-003
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Marketing Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:333` through line 456; 124 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.004 D-004
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Pardot (Marketing Cloud Account Engagement)
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:457` through line 576; 120 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.005 D-005
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Tableau
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:577` through line 708; 132 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.006 D-006
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: MuleSoft Anypoint Platform
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:709` through line 825; 117 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.007 D-007
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Slack
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:826` through line 955; 130 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.008 D-008
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Heroku
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:956` through line 1080; 125 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.009 D-009
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Salesforce Commerce Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1081` through line 1198; 118 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.010 D-010
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Salesforce Industries (Vlocity / OmniStudio / Industries CPQ)
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1199` through line 1295; 97 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.011 D-011
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Salesforce Financial Services Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1296` through line 1402; 107 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.012 D-012
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Salesforce Health Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1403` through line 1520; 118 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.013 D-013
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Salesforce Field Service (formerly Field Service Lightning)
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1521` through line 1637; 117 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: below strict line/anchor bar.

### Appendix A.014 D-014
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Experience Cloud (formerly Communities)
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1638` through line 1764; 127 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.015 D-015
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Salesforce Trailhead
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1765` through line 1890; 126 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.016 D-016
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow ITSM
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1891` through line 2022; 132 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.017 D-017
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow Customer Service Management
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2023` through line 2146; 124 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.018 D-018
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow HR Service Delivery
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2147` through line 2276; 130 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.019 D-019
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow Now Platform
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2277` through line 2423; 147 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.020 D-020
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow CMDB
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2424` through line 2562; 139 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.021 D-021
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow IT Operations Management
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2563` through line 2695; 133 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.022 D-022
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow Security Operations
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2696` through line 2833; 138 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.023 D-023
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow GRC
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2834` through line 2987; 154 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.024 D-024
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow Field Service
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:2988` through line 3132; 145 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.025 D-025
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: ServiceNow Strategic Portfolio Management
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3133` through line 3288; 156 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.026 D-026
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday HCM
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3289` through line 3444; 156 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.027 D-027
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Financials
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3445` through line 3594; 150 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.028 D-028
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Adaptive Planning
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3595` through line 3737; 143 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.029 D-029
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Recruiting
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3738` through line 3878; 141 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.030 D-030
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Talent Management
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3879` through line 4021; 143 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.031 D-031
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Learning
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4022` through line 4169; 148 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.032 D-032
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Procurement
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4170` through line 4315; 146 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.033 D-033
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Workday Expenses
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4316` through line 4455; 140 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.034 D-034
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Jira Software
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4456` through line 4615; 160 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.035 D-035
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Jira Service Management
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4616` through line 4775; 160 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.036 D-036
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Confluence
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4776` through line 4931; 156 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.037 D-037
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Bitbucket Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4932` through line 5084; 153 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.038 D-038
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Trello
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5085` through line 5235; 151 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.039 D-039
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Atlassian Open DevOps
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5236` through line 5373; 138 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.040 D-040
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Dynamics 365
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5374` through line 5522; 149 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.041 D-041
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Power Apps
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5523` through line 5666; 144 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.042 D-042
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Power BI
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5667` through line 5827; 161 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.043 D-043
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Power Automate
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5828` through line 5980; 153 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.044 D-044
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Power Pages
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5981` through line 6137; 157 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.045 D-045
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft 365
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:6138` through line 6324; 187 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.046 D-046
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Azure DevOps
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:6325` through line 6510; 186 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.047 D-047
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Defender XDR
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:6511` through line 6693; 183 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.048 D-048
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Sentinel
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:6694` through line 6869; 176 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.049 D-049
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Purview
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:6870` through line 7055; 186 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.050 D-050
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Intune
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7056` through line 7228; 173 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.051 D-051
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Entra ID
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7229` through line 7402; 174 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.052 D-052
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Microsoft Viva
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7403` through line 7571; 169 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.053 D-053
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Marketo Engage
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7572` through line 7751; 180 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.054 D-054
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Experience Manager
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7752` through line 7926; 175 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.055 D-055
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Campaign
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7927` through line 8089; 163 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.056 D-056
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Analytics
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8090` through line 8256; 167 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.057 D-057
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Real-Time CDP
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8257` through line 8429; 173 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.058 D-058
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Journey Optimizer
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8430` through line 8593; 164 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.059 D-059
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Sign
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8594` through line 8757; 164 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.060 D-060
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: Adobe Creative Cloud
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8758` through line 8936; 179 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.061 D-061
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: HubSpot Marketing Hub
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8937` through line 9115; 179 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.062 D-062
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: HubSpot Sales Hub
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9116` through line 9298; 183 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.063 D-063
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: HubSpot Service Hub
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9299` through line 9476; 178 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.064 D-064
- Severity: P2 IMPROVE
- Confidence: HIGH
- Status: PASS
- Title: HubSpot CMS Hub
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9477` through line 9653; 177 lines; 10/10 anchors.
- Missing anchors: none.
- Substance judgment: complete.

### Appendix A.065 D-065
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: HubSpot Operations Hub
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9654` through line 9671; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.066 D-066
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Zendesk Support
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9672` through line 9689; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.067 D-067
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Zendesk Chat
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9690` through line 9707; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.068 D-068
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Zendesk Talk
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9708` through line 9725; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.069 D-069
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Intercom
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9726` through line 9743; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.070 D-070
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Snowflake
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9744` through line 9762; 19 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.071 D-071
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Databricks
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9763` through line 9781; 19 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.072 D-072
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Google BigQuery
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9782` through line 9800; 19 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.073 D-073
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: AWS Redshift
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9801` through line 9819; 19 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.074 D-074
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: dbt
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9820` through line 9837; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.075 D-075
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Fivetran
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9838` through line 9855; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.076 D-076
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Airbyte
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9856` through line 9873; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.077 D-077
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Segment
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9874` through line 9891; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.078 D-078
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: RudderStack
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9892` through line 9904; 13 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.079 D-079
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Looker
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9905` through line 9922; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.080 D-080
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Hex
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9923` through line 9940; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.081 D-081
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Mode
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9941` through line 9958; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.082 D-082
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: Sigma
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9959` through line 9976; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.083 D-083
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: ThoughtSpot
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9977` through line 9994; 18 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.084 D-084
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: PagerDuty
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9995` through line 10013; 19 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.085 D-085
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Status: FAIL
- Title: OpsGenie
- Evidence: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10014` through line 10025; 12 lines; 7/10 anchors.
- Missing anchors: vendor_data_model, primary_apis, capability_tier.
- Substance judgment: below strict line/anchor bar.

### Appendix A.086 D-086
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-086` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.087 D-087
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-087` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.088 D-088
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-088` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.089 D-089
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-089` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.090 D-090
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-090` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.091 D-091
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-091` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.092 D-092
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-092` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.093 D-093
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-093` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.094 D-094
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-094` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.095 D-095
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-095` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.096 D-096
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-096` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.097 D-097
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-097` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.098 D-098
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-098` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.099 D-099
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-099` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.100 D-100
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-100` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.101 D-101
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-101` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.102 D-102
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-102` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.103 D-103
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-103` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.104 D-104
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-104` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.105 D-105
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-105` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.106 D-106
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-106` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.107 D-107
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-107` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.108 D-108
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-108` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.109 D-109
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-109` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.110 D-110
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-110` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.111 D-111
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-111` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.112 D-112
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-112` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.113 D-113
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-113` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.114 D-114
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-114` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.115 D-115
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-115` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.116 D-116
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-116` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.117 D-117
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-117` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.118 D-118
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-118` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.119 D-119
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-119` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.120 D-120
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-120` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.121 D-121
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-121` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.122 D-122
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-122` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.123 D-123
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-123` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.124 D-124
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-124` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.125 D-125
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-125` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.126 D-126
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-126` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.127 D-127
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-127` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.128 D-128
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-128` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.129 D-129
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-129` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.130 D-130
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-130` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.131 D-131
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-131` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.132 D-132
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-132` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.133 D-133
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-133` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.134 D-134
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-134` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.135 D-135
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-135` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.136 D-136
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-136` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.137 D-137
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-137` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.138 D-138
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-138` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.139 D-139
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-139` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.140 D-140
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-140` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.141 D-141
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-141` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.142 D-142
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-142` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.143 D-143
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-143` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.144 D-144
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-144` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.145 D-145
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-145` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.146 D-146
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-146` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.147 D-147
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-147` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.148 D-148
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-148` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.149 D-149
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-149` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.150 D-150
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-150` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.151 D-151
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-151` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.152 D-152
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-152` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.153 D-153
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-153` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.154 D-154
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-154` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.155 D-155
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-155` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.156 D-156
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-156` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.157 D-157
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-157` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.158 D-158
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-158` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.159 D-159
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-159` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.160 D-160
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-160` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.161 D-161
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-161` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.162 D-162
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-162` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.163 D-163
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-163` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.164 D-164
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-164` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

### Appendix A.165 D-165
- Severity: P0 BLOCKING
- Confidence: HIGH
- Status: MISSING
- Evidence: no `### Section D-165` heading in `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Required correction: author bespoke vendor dossier or remove the declared 165-dossier claim.
- Substance bar: FAIL, 0 lines, 0/10 anchors.

## Appendix B - Long-Form Doctrine Clause-Loop Audit

### docs/architecture/unified-ecosystem-thesis-2026-05-21.md
- Severity: P2 IMPROVE
- Confidence: HIGH
- Exists: yes
- Lines: 2511
- `Thesis clause` hits: 0
- `Problem clause` hits: 0
- Verdict: PASS

### docs/architecture/training-cost-doctrine-2026-05-21.md
- Severity: P2 IMPROVE
- Confidence: HIGH
- Exists: yes
- Lines: 1250
- `Thesis clause` hits: 0
- `Problem clause` hits: 0
- Verdict: PASS

## Appendix C - Per-Microservice Doc-Suite Artifacts and Surfaces

| Microservice | Artifacts | Doc artifacts | Surface count | Decisions | Severity | Confidence | Missing surfaces |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| analytics | 137 | 125 | 8/8 | 5 | P2 IMPROVE | HIGH | none |
| api-gateway | 134 | 125 | 7/8 | 0 | P1 NEEDS-FIX | HIGH | decisions |
| application | 126 | 121 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| audit-chain | 196 | 192 | 8/8 | 0 | P1 NEEDS-FIX | HIGH | none |
| calendar | 127 | 123 | 1/8 | 6 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| cell | 139 | 135 | 7/8 | 0 | P1 NEEDS-FIX | HIGH | decisions |
| cloud-iac | 166 | 155 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| cloud-k8s | 121 | 117 | 7/8 | 1 | P1 NEEDS-FIX | HIGH | reference-implementations |
| cloud-secrets | 125 | 120 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| comms-email | 135 | 125 | 8/8 | 5 | P2 IMPROVE | HIGH | none |
| community | 199 | 191 | 8/8 | 6 | P2 IMPROVE | HIGH | none |
| compliance | 186 | 175 | 1/8 | 6 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| connect | 183 | 173 | 7/8 | 0 | P1 NEEDS-FIX | HIGH | decisions |
| consent-graph | 135 | 131 | 8/8 | 5 | P2 IMPROVE | HIGH | none |
| contact-center | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| contract-lifecycle-management | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| crm | 137 | 126 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| data-pipeline | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| data-warehouse | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| design-collaboration | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| detection | 129 | 112 | 8/8 | 1 | P2 IMPROVE | HIGH | none |
| developer-sdk | 129 | 120 | 1/8 | 7 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| docs | 120 | 116 | 1/8 | 7 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| drive | 159 | 155 | 1/8 | 8 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| feature-flags | 131 | 117 | 7/8 | 0 | P1 NEEDS-FIX | HIGH | decisions |
| financial-planning | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| finops-portal | 157 | 147 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| forms | 144 | 140 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| foundry | 576 | 547 | 1/8 | 1 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| global-trade | 137 | 126 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| governance | 195 | 188 | 1/8 | 2 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| healthcare-integration | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| identity | 223 | 218 | 1/8 | 6 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| incident-management | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| intelligence | 165 | 155 | 8/8 | 0 | P1 NEEDS-FIX | HIGH | none |
| itsm | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| learning-management | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| mail | 195 | 187 | 1/8 | 6 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| marketing-automation | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| marketplace | 123 | 115 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| meet | 138 | 133 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| messenger | 156 | 150 | 1/8 | 6 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| network | 125 | 121 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| notes | 159 | 150 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| observability | 198 | 194 | 1/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| ontology | 150 | 143 | 8/8 | 1 | P2 IMPROVE | HIGH | none |
| ops-dashboard-control-center | 147 | 128 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| payments | 190 | 181 | 8/8 | 0 | P1 NEEDS-FIX | HIGH | none |
| performance-management | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| plant-maintenance | 139 | 128 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| plugin-app-store | 140 | 131 | 1/8 | 7 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| production-planning | 139 | 128 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| quality-management | 139 | 128 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| real-estate | 129 | 118 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| recordings | 127 | 122 | 8/8 | 8 | P2 IMPROVE | HIGH | none |
| sheets | 125 | 121 | 8/8 | 8 | P2 IMPROVE | HIGH | none |
| shorts | 122 | 118 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| sites | 123 | 119 | 8/8 | 8 | P2 IMPROVE | HIGH | none |
| slides | 128 | 124 | 8/8 | 9 | P2 IMPROVE | HIGH | none |
| social | 144 | 134 | 1/8 | 7 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| supply-chain-planning | 137 | 126 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| tasks | 122 | 116 | 8/8 | 7 | P2 IMPROVE | HIGH | none |
| tenancy | 181 | 174 | 1/8 | 1 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations |
| translate | 120 | 116 | 7/8 | 7 | P1 NEEDS-FIX | HIGH | reference-implementations |
| treasury | 139 | 128 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| warehouse | 129 | 118 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| whiteboard | 152 | 141 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |
| workflow-engine | 222 | 218 | 8/8 | 0 | P1 NEEDS-FIX | HIGH | none |
| workflow-studio | 221 | 208 | 8/8 | 6 | P2 IMPROVE | HIGH | none |
| workplace-integration | 124 | 116 | 0/8 | 0 | P1 NEEDS-FIX | HIGH | capability-tiers, onboarding, faqs, tutorials, benchmarks, migration-playbooks, reference-implementations, decisions |

### Appendix C - analytics
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 137; below-100 flag: no.
- Doc artifact count: 125.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 5.
- Decision evidence: `microservices/analytics/decisions/ADR-AN-001-ttl-policy.md`.
- Decision evidence: `microservices/analytics/decisions/ADR-AN-002-partition-strategy.md`.
- Decision evidence: `microservices/analytics/decisions/ADR-AN-003-row-level-tenant-isolation.md`.
- Decision evidence: `microservices/analytics/decisions/ADR-AN-004-query-budget-tier.md`.
- Decision evidence: `microservices/analytics/decisions/ADR-AN-005-materialized-view-cadence.md`.

### Appendix C - api-gateway
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 134; below-100 flag: no.
- Doc artifact count: 125.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - application
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 126; below-100 flag: no.
- Doc artifact count: 121.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - audit-chain
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 196; below-100 flag: no.
- Doc artifact count: 192.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - calendar
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 127; below-100 flag: no.
- Doc artifact count: 123.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/calendar/decisions/ADR-CAL-0001-caldav-server-backend-selection.md`.
- Decision evidence: `microservices/calendar/decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md`.
- Decision evidence: `microservices/calendar/decisions/ADR-CAL-0003-jmap-vs-caldav-frontend-priority.md`.
- Decision evidence: `microservices/calendar/decisions/ADR-CAL-0004-tzdb-refresh-and-pinning-policy.md`.
- Decision evidence: `microservices/calendar/decisions/ADR-CAL-001-icalendar-rfc5545-rfc7986-freebusy-acl.md`.
- Decision evidence: `microservices/calendar/decisions/README.md`.

### Appendix C - cell
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 139; below-100 flag: no.
- Doc artifact count: 135.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - cloud-iac
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 166; below-100 flag: no.
- Doc artifact count: 155.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - cloud-k8s
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 121; below-100 flag: no.
- Doc artifact count: 117.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 1.
- Decision evidence: `microservices/cloud-k8s/decisions/ADR-CK-001-cilium-cni-selection.md`.

### Appendix C - cloud-secrets
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 125; below-100 flag: no.
- Doc artifact count: 120.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - comms-email
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 135; below-100 flag: no.
- Doc artifact count: 125.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 5.
- Decision evidence: `microservices/comms-email/decisions/SVC-ADR-001-dkim-cadence.md`.
- Decision evidence: `microservices/comms-email/decisions/SVC-ADR-002-suppression-list-policy.md`.
- Decision evidence: `microservices/comms-email/decisions/SVC-ADR-003-webhook-retry-policy.md`.
- Decision evidence: `microservices/comms-email/decisions/SVC-ADR-004-tenant-domain-onboard-flow.md`.
- Decision evidence: `microservices/comms-email/decisions/SVC-ADR-005-mjml-liquid-canonical.md`.

### Appendix C - community
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 199; below-100 flag: no.
- Doc artifact count: 191.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md`.
- Decision evidence: `microservices/community/decisions/ADR-COMM-0002-voting-engine-tie-breaking-and-decay.md`.
- Decision evidence: `microservices/community/decisions/ADR-COMM-0003-kb-article-versioning-and-fork-merge.md`.
- Decision evidence: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md`.
- Decision evidence: `microservices/community/decisions/ADR-COMM-0005-graph-of-discussions-and-replies.md`.
- Decision evidence: `microservices/community/decisions/README.md`.

### Appendix C - compliance
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 186; below-100 flag: no.
- Doc artifact count: 175.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/compliance/decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md`.
- Decision evidence: `microservices/compliance/decisions/ADR-compliance-001-evidence-retention-policy.md`.
- Decision evidence: `microservices/compliance/decisions/ADR-compliance-002-dsar-sla.md`.
- Decision evidence: `microservices/compliance/decisions/ADR-compliance-003-auditor-access-cedar-policy.md`.
- Decision evidence: `microservices/compliance/decisions/ADR-compliance-004-cross-tenant-kernel-invariant.md`.
- Decision evidence: `microservices/compliance/decisions/ADR-compliance-005-replace-drata-vanta-with-in-house.md`.

### Appendix C - connect
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 183; below-100 flag: no.
- Doc artifact count: 173.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - consent-graph
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 135; below-100 flag: no.
- Doc artifact count: 131.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 5.
- Decision evidence: `microservices/consent-graph/decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md`.
- Decision evidence: `microservices/consent-graph/decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md`.
- Decision evidence: `microservices/consent-graph/decisions/ADR-SVC-CG-003-three-sharing-modes.md`.
- Decision evidence: `microservices/consent-graph/decisions/ADR-SVC-CG-004-grantor-region-topic-ownership.md`.
- Decision evidence: `microservices/consent-graph/decisions/ADR-SVC-CG-005-self-revocation-b2c.md`.

### Appendix C - contact-center
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - contract-lifecycle-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - crm
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 137; below-100 flag: no.
- Doc artifact count: 126.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - data-pipeline
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - data-warehouse
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - design-collaboration
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - detection
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 129; below-100 flag: no.
- Doc artifact count: 112.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 1.
- Decision evidence: `microservices/detection/decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md`.

### Appendix C - developer-sdk
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 129; below-100 flag: no.
- Doc artifact count: 120.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0001-ed25519-signing-keys-via-openbao-transit-engine-only;-privat.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0004-payout-substrate-uses-iso-20022-pain.001-for-sepa-and-nacha-.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0005-tax-form-emission-triggered-at-year-end-regenerated-on-deman.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0006-kyc-pipeline-in-house;-no-external-kyc-saas-(onfido-persona-.md`.
- Decision evidence: `microservices/developer-sdk/decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md`.

### Appendix C - docs
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 120; below-100 flag: no.
- Doc artifact count: 116.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0001-crdt-library-selection.md`.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0002-block-type-system.md`.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0003-export-pipeline-architecture.md`.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0004-acl-granularity-per-block.md`.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0005-ai-writing-assist-bounds.md`.
- Decision evidence: `microservices/docs/decisions/ADR-DOCS-0006-import-fidelity-policy.md`.
- Decision evidence: `microservices/docs/decisions/README.md`.

### Appendix C - drive
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 159; below-100 flag: no.
- Doc artifact count: 155.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 8.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0003-share-link-security-model.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0004-encryption-at-rest-and-e2e.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-0006-immutability-and-worm-policy.md`.
- Decision evidence: `microservices/drive/decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`.
- Decision evidence: `microservices/drive/decisions/README.md`.

### Appendix C - feature-flags
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 131; below-100 flag: no.
- Doc artifact count: 117.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - financial-planning
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - finops-portal
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 157; below-100 flag: no.
- Doc artifact count: 147.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-001-focus-spec-version.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-002-cost-attribution-label-strategy.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-003-tenant-billing-export-cadence.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-004-credit-ledger-append-only.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-005-grafana-iframe-embed.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-006-cedar-residency-double-guard.md`.
- Decision evidence: `microservices/finops-portal/decisions/ADR-finops-portal-007-ed25519-quarterly-key.md`.

### Appendix C - forms
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 144; below-100 flag: no.
- Doc artifact count: 140.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0001-form-definition-schema.md`.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0002-captcha-and-anti-spam.md`.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0003-pii-column-encryption-and-residency.md`.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0004-conditional-logic-and-branching-engine.md`.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md`.
- Decision evidence: `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md`.
- Decision evidence: `microservices/forms/decisions/README.md`.

### Appendix C - foundry
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 576; below-100 flag: no.
- Doc artifact count: 547.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 1.
- Decision evidence: `microservices/foundry/decisions/SVC-ADR-WASM-001-wasmtime-canonical-foundry.md`.

### Appendix C - global-trade
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 137; below-100 flag: no.
- Doc artifact count: 126.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - governance
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 195; below-100 flag: no.
- Doc artifact count: 188.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 2.
- Decision evidence: `microservices/governance/decisions/ADR-GOV-001-audit-event-aggregation-pack-retention.md`.
- Decision evidence: `microservices/governance/decisions/SVC-ADR-WASM-001-envoy-wasm-canonical-governance.md`.

### Appendix C - healthcare-integration
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - identity
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 223; below-100 flag: no.
- Doc artifact count: 218.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/identity/decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md`.
- Decision evidence: `microservices/identity/decisions/ADR-identity-001-jwks-rotation-cadence.md`.
- Decision evidence: `microservices/identity/decisions/ADR-identity-002-passkey-attestation-policy.md`.
- Decision evidence: `microservices/identity/decisions/ADR-identity-003-scim-rate-limits.md`.
- Decision evidence: `microservices/identity/decisions/ADR-identity-004-session-class-tiers.md`.
- Decision evidence: `microservices/identity/decisions/ADR-identity-005-jit-it-approval-protocol.md`.

### Appendix C - incident-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - intelligence
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 165; below-100 flag: no.
- Doc artifact count: 155.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - itsm
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - learning-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - mail
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 195; below-100 flag: no.
- Doc artifact count: 187.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/mail/decisions/ADR-MAIL-0001-personal-mail-key-recovery.md`.
- Decision evidence: `microservices/mail/decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md`.
- Decision evidence: `microservices/mail/decisions/ADR-MAIL-0003-sdk-launch-order.md`.
- Decision evidence: `microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md`.
- Decision evidence: `microservices/mail/decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`.
- Decision evidence: `microservices/mail/decisions/README.md`.

### Appendix C - marketing-automation
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - marketplace
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 123; below-100 flag: no.
- Doc artifact count: 115.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - meet
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 138; below-100 flag: no.
- Doc artifact count: 133.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0001-sfu-substrate-selection.md`.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0002-recording-and-transcription-pipeline.md`.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0003-e2e-encryption-for-meetings.md`.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0004-live-streaming-egress-policy.md`.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0005-large-audience-and-webinar-architecture.md`.
- Decision evidence: `microservices/meet/decisions/ADR-MEET-0006-ai-feature-bounds.md`.
- Decision evidence: `microservices/meet/decisions/README.md`.

### Appendix C - messenger
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 156; below-100 flag: no.
- Doc artifact count: 150.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md`.
- Decision evidence: `microservices/messenger/decisions/ADR-MSGR-0001-huddles-placement.md`.
- Decision evidence: `microservices/messenger/decisions/ADR-MSGR-0002-e2e-personal-dm-key-escrow.md`.
- Decision evidence: `microservices/messenger/decisions/ADR-MSGR-0003-search-backend-selection.md`.
- Decision evidence: `microservices/messenger/decisions/ADR-MSGR-0004-federation-posture.md`.
- Decision evidence: `microservices/messenger/decisions/README.md`.

### Appendix C - network
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 125; below-100 flag: no.
- Doc artifact count: 121.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/network/decisions/ADR-NET-0001-professional-graph-storage.md`.
- Decision evidence: `microservices/network/decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md`.
- Decision evidence: `microservices/network/decisions/ADR-NET-0003-inmail-bridge-to-messenger.md`.
- Decision evidence: `microservices/network/decisions/ADR-NET-0004-jobs-handoff-to-ats.md`.
- Decision evidence: `microservices/network/decisions/ADR-NET-0005-endorsement-chain-integrity.md`.
- Decision evidence: `microservices/network/decisions/ADR-NET-0006-profile-portability-and-export.md`.
- Decision evidence: `microservices/network/decisions/README.md`.

### Appendix C - notes
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 159; below-100 flag: no.
- Doc artifact count: 150.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0001-e2e-encryption-default-personal-tier.md`.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0002-bidirectional-link-and-graph-storage.md`.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0003-crdt-library-for-optional-collab.md`.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0004-search-architecture-respecting-e2e.md`.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md`.
- Decision evidence: `microservices/notes/decisions/ADR-NOTES-0006-portable-export-and-import-format.md`.
- Decision evidence: `microservices/notes/decisions/README.md`.

### Appendix C - observability
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 198; below-100 flag: no.
- Doc artifact count: 194.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - ontology
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 150; below-100 flag: no.
- Doc artifact count: 143.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 1.
- Decision evidence: `microservices/ontology/decisions/README.md`.

### Appendix C - ops-dashboard-control-center
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 147; below-100 flag: no.
- Doc artifact count: 128.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - payments
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 190; below-100 flag: no.
- Doc artifact count: 181.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - performance-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - plant-maintenance
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 139; below-100 flag: no.
- Doc artifact count: 128.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - plugin-app-store
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 140; below-100 flag: no.
- Doc artifact count: 131.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0001-per-plugin-cedar-policy-materialization-at-install-time-not-.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0002-vetting-pipeline-ordered-stage-execution-never-parallelized.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0003-wasmtime-engine-per-tenant-plugin-installation-not-per-plugi.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0004-vetting-badge-tiers-(bronze/silver/gold/platinum)-determined.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0005-per-installation-rate-limit-default-100-req/s;-per-plugin-ov.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0006-subscription-billing-aggregator-runs-nightly-not-real-time.md`.
- Decision evidence: `microservices/plugin-app-store/decisions/ADR-PAS-0007-per-plugin-action-audit-trail-seals-via-audit-chain-µservice.md`.

### Appendix C - production-planning
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 139; below-100 flag: no.
- Doc artifact count: 128.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - quality-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 139; below-100 flag: no.
- Doc artifact count: 128.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - real-estate
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 129; below-100 flag: no.
- Doc artifact count: 118.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - recordings
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 127; below-100 flag: no.
- Doc artifact count: 122.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 8.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0001-transcription-and-diarization-pipeline.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0003-redaction-and-pii-policy.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0004-playback-and-cdn-strategy.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0005-storage-substrate-tiered.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0006-ai-feature-bounds.md`.
- Decision evidence: `microservices/recordings/decisions/ADR-RECORDINGS-0007-multi-source-ingest-contract.md`.
- Decision evidence: `microservices/recordings/decisions/README.md`.

### Appendix C - sheets
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 125; below-100 flag: no.
- Doc artifact count: 121.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 8.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0001-crdt-library-selection.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0002-formula-engine-conformance-target.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0003-large-sheet-storage-substrate.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0004-recalc-engine-architecture.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0005-ai-formula-and-smart-fill-bounds.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0006-per-range-acl-granularity.md`.
- Decision evidence: `microservices/sheets/decisions/ADR-SHEETS-0007-export-fidelity-policy.md`.
- Decision evidence: `microservices/sheets/decisions/README.md`.

### Appendix C - shorts
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 122; below-100 flag: no.
- Doc artifact count: 118.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0001-video-transcode-pipeline.md`.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0002-copyright-claim-system.md`.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0003-content-moderation-classifier-bounds.md`.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0004-drm-substrate-tenant-tier.md`.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0005-feed-ranking-algorithm.md`.
- Decision evidence: `microservices/shorts/decisions/ADR-SHORTS-0006-minor-protection-and-age-gate.md`.
- Decision evidence: `microservices/shorts/decisions/README.md`.

### Appendix C - sites
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 123; below-100 flag: no.
- Doc artifact count: 119.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 8.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0001-crdt-library-selection.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0003-cdn-substrate-and-cache-strategy.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0004-acme-and-custom-domain-flow.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0005-cms-collection-data-model.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0006-ai-page-build-bounds.md`.
- Decision evidence: `microservices/sites/decisions/ADR-SITES-0007-image-and-asset-pipeline.md`.
- Decision evidence: `microservices/sites/decisions/README.md`.

### Appendix C - slides
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 128; below-100 flag: no.
- Doc artifact count: 124.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 9.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0001-crdt-library-selection.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0002-rendering-canvas-substrate.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0003-export-pipeline-fidelity.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0006-ai-design-and-content-generation-bounds.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0007-per-slide-acl-granularity.md`.
- Decision evidence: `microservices/slides/decisions/ADR-SLIDES-0008-chart-live-link-to-sheets.md`.

### Appendix C - social
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 144; below-100 flag: no.
- Doc artifact count: 134.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0001-feed-ranking-algorithm.md`.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0002-follow-graph-storage.md`.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md`.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0004-federation-posture.md`.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0005-dual-context-feed-isolation.md`.
- Decision evidence: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md`.
- Decision evidence: `microservices/social/decisions/README.md`.

### Appendix C - supply-chain-planning
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 137; below-100 flag: no.
- Doc artifact count: 126.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - tasks
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 122; below-100 flag: no.
- Doc artifact count: 116.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0001-task-data-model-and-custom-fields.md`.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0002-dependency-graph-and-cycle-prevention.md`.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0003-recurring-task-engine.md`.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0004-view-engine-and-board-realtime.md`.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0005-automation-engine-cross-microservice.md`.
- Decision evidence: `microservices/tasks/decisions/ADR-TASKS-0006-ai-auto-assign-and-eu-ai-act-bounds.md`.
- Decision evidence: `microservices/tasks/decisions/README.md`.

### Appendix C - tenancy
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 181; below-100 flag: no.
- Doc artifact count: 174.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 1.
- Decision evidence: `microservices/tenancy/decisions/ADR-TEN-001-tenant-lifecycle-parent-child-cedar-permit.md`.

### Appendix C - translate
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 120; below-100 flag: no.
- Doc artifact count: 116.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: missing.
- Surface `decisions`: present.
- Per-service ADR files: 7.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0001-mt-engine-routing-and-fallback.md`.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0002-translation-memory-and-leverage-model.md`.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md`.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0004-data-residency-bound-inference.md`.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0005-document-round-trip-fidelity.md`.
- Decision evidence: `microservices/translate/decisions/ADR-TRANSLATE-0006-real-time-translation-stream-architecture.md`.
- Decision evidence: `microservices/translate/decisions/README.md`.

### Appendix C - treasury
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 139; below-100 flag: no.
- Doc artifact count: 128.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - warehouse
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 129; below-100 flag: no.
- Doc artifact count: 118.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - whiteboard
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 152; below-100 flag: no.
- Doc artifact count: 141.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - workflow-engine
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 222; below-100 flag: no.
- Doc artifact count: 218.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

### Appendix C - workflow-studio
- Severity: P2 IMPROVE
- Confidence: HIGH
- Artifact count: 221; below-100 flag: no.
- Doc artifact count: 208.
- Surface `capability-tiers`: present.
- Surface `onboarding`: present.
- Surface `faqs`: present.
- Surface `tutorials`: present.
- Surface `benchmarks`: present.
- Surface `migration-playbooks`: present.
- Surface `reference-implementations`: present.
- Surface `decisions`: present.
- Per-service ADR files: 6.
- Decision evidence: `microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md`.
- Decision evidence: `microservices/workflow-studio/decisions/ADR-WS-0002-dsl-canonical-form.md`.
- Decision evidence: `microservices/workflow-studio/decisions/ADR-WS-0003-leptos-wasm-substrate.md`.
- Decision evidence: `microservices/workflow-studio/decisions/ADR-WS-0004-jurisdiction-overlay-renderer.md`.
- Decision evidence: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md`.
- Decision evidence: `microservices/workflow-studio/decisions/README.md`.

### Appendix C - workplace-integration
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- Artifact count: 124; below-100 flag: no.
- Doc artifact count: 116.
- Surface `capability-tiers`: missing.
- Surface `onboarding`: missing.
- Surface `faqs`: missing.
- Surface `tutorials`: missing.
- Surface `benchmarks`: missing.
- Surface `migration-playbooks`: missing.
- Surface `reference-implementations`: missing.
- Surface `decisions`: missing.
- Per-service ADR files: 0.
- Decision evidence: none found under `decisions/`.

## Appendix D - Per-Microservice ADR Authorship

| Microservice | Authored ADRs? | Count | Severity | Confidence |
| --- | --- | ---: | --- | --- |
| analytics | yes | 5 | P2 IMPROVE | HIGH |
| api-gateway | no | 0 | P1 NEEDS-FIX | HIGH |
| application | no | 0 | P1 NEEDS-FIX | HIGH |
| audit-chain | no | 0 | P1 NEEDS-FIX | HIGH |
| calendar | yes | 6 | P2 IMPROVE | HIGH |
| cell | no | 0 | P1 NEEDS-FIX | HIGH |
| cloud-iac | no | 0 | P1 NEEDS-FIX | HIGH |
| cloud-k8s | yes | 1 | P2 IMPROVE | HIGH |
| cloud-secrets | no | 0 | P1 NEEDS-FIX | HIGH |
| comms-email | yes | 5 | P2 IMPROVE | HIGH |
| community | yes | 6 | P2 IMPROVE | HIGH |
| compliance | yes | 6 | P2 IMPROVE | HIGH |
| connect | no | 0 | P1 NEEDS-FIX | HIGH |
| consent-graph | yes | 5 | P2 IMPROVE | HIGH |
| contact-center | no | 0 | P1 NEEDS-FIX | HIGH |
| contract-lifecycle-management | no | 0 | P1 NEEDS-FIX | HIGH |
| crm | no | 0 | P1 NEEDS-FIX | HIGH |
| data-pipeline | no | 0 | P1 NEEDS-FIX | HIGH |
| data-warehouse | no | 0 | P1 NEEDS-FIX | HIGH |
| design-collaboration | no | 0 | P1 NEEDS-FIX | HIGH |
| detection | yes | 1 | P2 IMPROVE | HIGH |
| developer-sdk | yes | 7 | P2 IMPROVE | HIGH |
| docs | yes | 7 | P2 IMPROVE | HIGH |
| drive | yes | 8 | P2 IMPROVE | HIGH |
| feature-flags | no | 0 | P1 NEEDS-FIX | HIGH |
| financial-planning | no | 0 | P1 NEEDS-FIX | HIGH |
| finops-portal | yes | 7 | P2 IMPROVE | HIGH |
| forms | yes | 7 | P2 IMPROVE | HIGH |
| foundry | yes | 1 | P2 IMPROVE | HIGH |
| global-trade | no | 0 | P1 NEEDS-FIX | HIGH |
| governance | yes | 2 | P2 IMPROVE | HIGH |
| healthcare-integration | no | 0 | P1 NEEDS-FIX | HIGH |
| identity | yes | 6 | P2 IMPROVE | HIGH |
| incident-management | no | 0 | P1 NEEDS-FIX | HIGH |
| intelligence | no | 0 | P1 NEEDS-FIX | HIGH |
| itsm | no | 0 | P1 NEEDS-FIX | HIGH |
| learning-management | no | 0 | P1 NEEDS-FIX | HIGH |
| mail | yes | 6 | P2 IMPROVE | HIGH |
| marketing-automation | no | 0 | P1 NEEDS-FIX | HIGH |
| marketplace | no | 0 | P1 NEEDS-FIX | HIGH |
| meet | yes | 7 | P2 IMPROVE | HIGH |
| messenger | yes | 6 | P2 IMPROVE | HIGH |
| network | yes | 7 | P2 IMPROVE | HIGH |
| notes | yes | 7 | P2 IMPROVE | HIGH |
| observability | no | 0 | P1 NEEDS-FIX | HIGH |
| ontology | yes | 1 | P2 IMPROVE | HIGH |
| ops-dashboard-control-center | no | 0 | P1 NEEDS-FIX | HIGH |
| payments | no | 0 | P1 NEEDS-FIX | HIGH |
| performance-management | no | 0 | P1 NEEDS-FIX | HIGH |
| plant-maintenance | no | 0 | P1 NEEDS-FIX | HIGH |
| plugin-app-store | yes | 7 | P2 IMPROVE | HIGH |
| production-planning | no | 0 | P1 NEEDS-FIX | HIGH |
| quality-management | no | 0 | P1 NEEDS-FIX | HIGH |
| real-estate | no | 0 | P1 NEEDS-FIX | HIGH |
| recordings | yes | 8 | P2 IMPROVE | HIGH |
| sheets | yes | 8 | P2 IMPROVE | HIGH |
| shorts | yes | 7 | P2 IMPROVE | HIGH |
| sites | yes | 8 | P2 IMPROVE | HIGH |
| slides | yes | 9 | P2 IMPROVE | HIGH |
| social | yes | 7 | P2 IMPROVE | HIGH |
| supply-chain-planning | no | 0 | P1 NEEDS-FIX | HIGH |
| tasks | yes | 7 | P2 IMPROVE | HIGH |
| tenancy | yes | 1 | P2 IMPROVE | HIGH |
| translate | yes | 7 | P2 IMPROVE | HIGH |
| treasury | no | 0 | P1 NEEDS-FIX | HIGH |
| warehouse | no | 0 | P1 NEEDS-FIX | HIGH |
| whiteboard | no | 0 | P1 NEEDS-FIX | HIGH |
| workflow-engine | no | 0 | P1 NEEDS-FIX | HIGH |
| workflow-studio | yes | 6 | P2 IMPROVE | HIGH |
| workplace-integration | no | 0 | P1 NEEDS-FIX | HIGH |

## Appendix E - IP Slice Substance Seeded Samples

- Random seed: `20260520`
- Pass rule: >=200 lines, bespoke service signal, zero generic placeholder hits, and >=10 unique ADR citations.

### Appendix E - analytics
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 10
- Sample verdict: PASS
- Sample path: `microservices/analytics/IP-journey-j95-iso27001-soc2-annual-audit.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/analytics/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - api-gateway
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 32
- Sample verdict: FAIL
- Sample path: `microservices/api-gateway/IP-010-rate-limit-adapter-redis.md`
  - Lines: 15
  - Unique ADR citations: 0
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md`
  - Lines: 20
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - application
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 27
- Sample verdict: FAIL
- Sample path: `microservices/application/IP-journey-j93-in-dpdpa-rbi-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/application/IP-015-application-openslo-and-hg-app.md`
  - Lines: 122
  - Unique ADR citations: 3
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - audit-chain
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 96
- Sample verdict: FAIL
- Sample path: `microservices/audit-chain/IP-001-storage-backend-iac.md`
  - Lines: 68
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/audit-chain/IP-002-self-slo-manifest.md`
  - Lines: 151
  - Unique ADR citations: 3
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - calendar
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 34
- Sample verdict: FAIL
- Sample path: `microservices/calendar/IP-011-contracts-openapi-asyncapi-proto.md`
  - Lines: 68
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/calendar/IP-journey-j35-work-freebusy.md`
  - Lines: 420
  - Unique ADR citations: 36
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - cell
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 41
- Sample verdict: FAIL
- Sample path: `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`
  - Lines: 94
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/tenancy/ARCHITECTURE.md#cell-assignment`
  - Lines: 430
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - cloud-iac
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 38
- Sample verdict: PASS
- Sample path: `microservices/cloud-iac/IP-journey-j93-in-dpdpa-rbi-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/cloud-iac/IP-journey-j91-us-msb-mtl-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - cloud-k8s
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/cloud-k8s/IP-015-observability-slo-and-authority-cohesion.md`
  - Lines: 155
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/cloud-k8s/IP-journey-j95-iso27001-soc2-annual-audit.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - cloud-secrets
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 32
- Sample verdict: FAIL
- Sample path: `microservices/cloud-secrets/IP-journey-j94-sox404-public-company-controls.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/cloud-secrets/IP-007-resolver-rest-and-sdk-rust.md`
  - Lines: 77
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - comms-email
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 36
- Sample verdict: FAIL
- Sample path: `microservices/comms-email/IP-journey-j94-sox404-public-company-controls.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/comms-email/IP-026-unsubscribe-async-emit.md`
  - Lines: 25
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - community
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 66
- Sample verdict: PASS
- Sample path: `microservices/community/IP-journey-j110-talent-and-trust-surface.md`
  - Lines: 861
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/community/IP-journey-j65-community-export.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - compliance
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 90
- Sample verdict: FAIL
- Sample path: `microservices/compliance/IP-journey-j46-rx-overlay.md`
  - Lines: 420
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/compliance/IP-journey-j93-in-dpdpa-rbi-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - connect
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 51
- Sample verdict: FAIL
- Sample path: `microservices/connect/IP-journey-j120-bank-liquidity-provider-adapter.md`
  - Lines: 430
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/connect/IP-002-connector-catalog-domain-kernel.md`
  - Lines: 87
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - consent-graph
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 34
- Sample verdict: FAIL
- Sample path: `microservices/consent-graph/IP-journey-j94-sox404-public-company-controls.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/consent-graph/IP-014-partner-directory-handshake.md`
  - Lines: 188
  - Unique ADR citations: 3
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - contact-center
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/contact-center/IP-009-credential-sidecar-binding.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/contact-center/IP-001-tenant-scope-kernel.md`
  - Lines: 113
  - Unique ADR citations: 6
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - contract-lifecycle-management
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 30
- Sample verdict: PASS
- Sample path: `microservices/contract-lifecycle-management/IP-024-threat-model-control-map.md`
  - Lines: 306
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/contract-lifecycle-management/IP-015-data-residency-pack-overlays.md`
  - Lines: 306
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - crm
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 23
- Sample verdict: FAIL
- Sample path: `microservices/crm/IP-003-domain-layer-for-quote.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/crm/IP-014-rest-grpc-and-worker-surfaces-for-crm.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - data-pipeline
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 30
- Sample verdict: PASS
- Sample path: `microservices/data-pipeline/IP-010-multi-region-cell-layout.md`
  - Lines: 242
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/data-pipeline/IP-019-sdk-client-generation.md`
  - Lines: 256
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - data-warehouse
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/data-warehouse/IP-003-ontology-projection.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/data-warehouse/IP-022-chaos-drill-pack.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - design-collaboration
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 30
- Sample verdict: PASS
- Sample path: `microservices/design-collaboration/IP-002-cedar-default-deny.md`
  - Lines: 205
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/design-collaboration/IP-029-asset-license-provenance.md`
  - Lines: 205
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - detection
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 24
- Sample verdict: FAIL
- Sample path: `microservices/detection/IP-008-rules-engine-rest.md`
  - Lines: 75
  - Unique ADR citations: 12
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/detection/IP-005-feature-store-domain.md`
  - Lines: 75
  - Unique ADR citations: 12
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - developer-sdk
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 11
- Sample verdict: PASS
- Sample path: `microservices/developer-sdk/IP-journey-j97-sg-pdpa-mas-tenant.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/developer-sdk/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - docs
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/docs/IP-010-sharing-and-permissions.md`
  - Lines: 44
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/docs/IP-009-version-history.md`
  - Lines: 44
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - drive
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 61
- Sample verdict: PASS
- Sample path: `microservices/drive/IP-journey-j123-shared-asset-vault.md`
  - Lines: 430
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/drive/IP-journey-j57-starter-pack.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - feature-flags
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 37
- Sample verdict: FAIL
- Sample path: `microservices/feature-flags/IP-005-flag-adapter-postgres.md`
  - Lines: 51
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/feature-flags/IP-016-python-sdk.md`
  - Lines: 55
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - financial-planning
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/financial-planning/IP-023-dpia-evidence-packet.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/financial-planning/IP-029-vena-pigment-board-scenario-displacement.md`
  - Lines: 204
  - Unique ADR citations: 5
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - finops-portal
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 29
- Sample verdict: FAIL
- Sample path: `microservices/finops-portal/IP-journey-j42-spend-attribution.md`
  - Lines: 420
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/finops-portal/IP-journey-j96-ksa-uae-mena-onboarding.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - forms
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 31
- Sample verdict: PASS
- Sample path: `microservices/forms/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/forms/IP-journey-j54-quote-request.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - foundry
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 115
- Sample verdict: FAIL
- Sample path: `microservices/foundry/IP-010-runtime-capability-executor-sdk.md`
  - Lines: 136
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/foundry/IP-066-guardrails-autonomy-tier-gate-kernel-and-cedar-adapter.md`
  - Lines: 144
  - Unique ADR citations: 5
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - global-trade
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 23
- Sample verdict: FAIL
- Sample path: `microservices/global-trade/IP-010-usecase-layer-for-trade-document.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/global-trade/IP-002-domain-layer-for-sanctions-screening.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - governance
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 42
- Sample verdict: FAIL
- Sample path: `microservices/governance/IP-journey-j67-legal-authority.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/governance/IP-015-runbooks-iac-finalization.md`
  - Lines: 146
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - healthcare-integration
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 30
- Sample verdict: PASS
- Sample path: `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md`
  - Lines: 230
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/healthcare-integration/IP-024-threat-model-control-map.md`
  - Lines: 307
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - identity
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 129
- Sample verdict: FAIL
- Sample path: `microservices/identity/IP-journey-j140-internal-audit-dlp-egress-principal-context.md`
  - Lines: 425
  - Unique ADR citations: 9
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/identity/IP-journey-j149-cedar-limited-task-count-principal.md`
  - Lines: 430
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - incident-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/incident-management/IP-026-pagerduty-event-orchestration-displacement.md`
  - Lines: 209
  - Unique ADR citations: 5
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/incident-management/IP-007-grpc-internal-surface.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - intelligence
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 64
- Sample verdict: FAIL
- Sample path: `microservices/intelligence/IP-journey-j144-filter-and-drafter-consumer-tier.md`
  - Lines: 425
  - Unique ADR citations: 9
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/intelligence/IP-journey-j150-brand-safety-and-caption-assist.md`
  - Lines: 430
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - itsm
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/itsm/IP-007-grpc-internal-surface.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/itsm/IP-017-cost-budget-enforcer.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - learning-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/learning-management/IP-010-multi-region-cell-layout.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/learning-management/IP-007-grpc-internal-surface.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - mail
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 93
- Sample verdict: FAIL
- Sample path: `microservices/mail/IP-journey-j76-notice-delivery.md`
  - Lines: 430
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/mail/IP-journey-j98-au-privacy-apra-cps234.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - marketing-automation
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/marketing-automation/IP-003-ontology-projection.md`
  - Lines: 107
  - Unique ADR citations: 5
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/marketing-automation/IP-017-cost-budget-enforcer.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - marketplace
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 40
- Sample verdict: FAIL
- Sample path: `microservices/marketplace/IP-journey-j112-deal-settlement-ledger.md`
  - Lines: 862
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/marketplace/ip/IP-021-multi-region-replay.md`
  - Lines: 54
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - meet
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 36
- Sample verdict: FAIL
- Sample path: `microservices/meet/IP-013-contracts-openapi-asyncapi-proto.md`
  - Lines: 59
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/meet/IP-010-webinar-and-breakouts.md`
  - Lines: 90
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - messenger
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 63
- Sample verdict: FAIL
- Sample path: `microservices/messenger/IP-journey-j74-plugin-channel-actions.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/messenger/IP-001-iac-bootstrap.md`
  - Lines: 82
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - network
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 29
- Sample verdict: PASS
- Sample path: `microservices/network/IP-journey-j91-us-msb-mtl-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/network/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - notes
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 45
- Sample verdict: FAIL
- Sample path: `microservices/notes/IP-010-search-and-graph-view.md`
  - Lines: 94
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/notes/IP-journey-j45-record-correction-request.md`
  - Lines: 420
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - observability
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 77
- Sample verdict: PASS
- Sample path: `microservices/observability/IP-journey-j27-schedule-conflict-metrics.md`
  - Lines: 420
  - Unique ADR citations: 36
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/observability/IP-journey-j98-au-privacy-apra-cps234.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - ontology
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 53
- Sample verdict: FAIL
- Sample path: `microservices/ontology/IP-002-object-type-registry-kernel-domain.md`
  - Lines: 83
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/ontology/IP-003-link-action-function-type-registry.md`
  - Lines: 70
  - Unique ADR citations: 6
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - ops-dashboard-control-center
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 40
- Sample verdict: FAIL
- Sample path: `microservices/ops-dashboard-control-center/IP-008-step-up-auth-flow.md`
  - Lines: 50
  - Unique ADR citations: 4
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/ops-dashboard-control-center/IP-002-incident-command-workflows.md`
  - Lines: 27
  - Unique ADR citations: 0
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - payments
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 88
- Sample verdict: PASS
- Sample path: `microservices/payments/IP-journey-j52-buyer-charge-and-seller-settlement.md`
  - Lines: 430
  - Unique ADR citations: 17
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/payments/IP-journey-j120-per-currency-ledger-posting.md`
  - Lines: 430
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - performance-management
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 30
- Sample verdict: FAIL
- Sample path: `microservices/performance-management/IP-006-async-event-surface.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/performance-management/IP-011-observability-audit-events.md`
  - Lines: 55
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - plant-maintenance
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 25
- Sample verdict: PASS
- Sample path: `microservices/plant-maintenance/IP-015-integration-tests-for-plant-maintenance.md`
  - Lines: 344
  - Unique ADR citations: 14
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/plant-maintenance/IP-008-usecase-layer-for-maintenance-plan.md`
  - Lines: 305
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - plugin-app-store
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 21
- Sample verdict: PASS
- Sample path: `microservices/plugin-app-store/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/plugin-app-store/IP-journey-j92-br-lgpd-us-parent-dsar.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - production-planning
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 25
- Sample verdict: PASS
- Sample path: `microservices/production-planning/IP-014-rest-grpc-and-worker-surfaces-for-production-planning.md`
  - Lines: 333
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/production-planning/IP-016-mrp-explosion-to-supply-chain-planning-handoff.md`
  - Lines: 383
  - Unique ADR citations: 18
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - quality-management
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 25
- Sample verdict: PASS
- Sample path: `microservices/quality-management/IP-013-adapter-integrations-for-quality-management.md`
  - Lines: 269
  - Unique ADR citations: 12
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/quality-management/IP-025-fmea-workspace-with-rpn-scoring.md`
  - Lines: 287
  - Unique ADR citations: 12
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - real-estate
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 15
- Sample verdict: FAIL
- Sample path: `microservices/real-estate/IP-006-domain-layer-for-facility-service-request.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/real-estate/IP-014-rest-grpc-and-worker-surfaces-for-real-estate.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - recordings
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 27
- Sample verdict: FAIL
- Sample path: `microservices/recordings/IP-journey-j96-ksa-uae-mena-onboarding.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/recordings/IP-009-chapter-summary-bcs.md`
  - Lines: 84
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - sheets
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 25
- Sample verdict: FAIL
- Sample path: `microservices/sheets/IP-008-formatting-pivot-charts-data-validation.md`
  - Lines: 83
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/sheets/IP-003-formula-engine-kernel-domain-400-functions.md`
  - Lines: 98
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - shorts
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 29
- Sample verdict: FAIL
- Sample path: `microservices/shorts/IP-007-audio-track-library-and-attribution-bc.md`
  - Lines: 59
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/shorts/IP-journey-j79-short-video-surface.md`
  - Lines: 430
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - sites
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 25
- Sample verdict: FAIL
- Sample path: `microservices/sites/IP-003-page-bc-kernel.md`
  - Lines: 49
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/sites/IP-007-domain-binding-acme.md`
  - Lines: 49
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - slides
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 25
- Sample verdict: FAIL
- Sample path: `microservices/slides/IP-003-slide-layout-text-box-shape-kernel-domain.md`
  - Lines: 113
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/slides/IP-journey-j100-pack-rollout-first-action.md`
  - Lines: 400
  - Unique ADR citations: 13
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - social
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 32
- Sample verdict: FAIL
- Sample path: `microservices/social/IP-journey-j95-iso27001-soc2-annual-audit.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/social/IP-009-trending-topics-bc.md`
  - Lines: 62
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - supply-chain-planning
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 23
- Sample verdict: FAIL
- Sample path: `microservices/supply-chain-planning/IP-020-ctp-compute-with-shop-floor-sync.md`
  - Lines: 483
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/supply-chain-planning/IP-004-domain-layer-for-replenishment-plan.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - tasks
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 25
- Sample verdict: FAIL
- Sample path: `microservices/tasks/IP-journey-j98-au-privacy-apra-cps234.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/tasks/IP-013-rest-and-websocket-api-surface.md`
  - Lines: 89
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - tenancy
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 87
- Sample verdict: FAIL
- Sample path: `microservices/tenancy/IP-012-branch-protection-and-release-pointers.md`
  - Lines: 72
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/tenancy/IP-journey-j134-3-tenant-engagement-scope.md`
  - Lines: 425
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - translate
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 26
- Sample verdict: FAIL
- Sample path: `microservices/translate/IP-009-document-translation-stack.md`
  - Lines: 84
  - Unique ADR citations: 1
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/translate/IP-002-translate-router-kernel.md`
  - Lines: 288
  - Unique ADR citations: 2
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - treasury
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 25
- Sample verdict: FAIL
- Sample path: `microservices/treasury/IP-022-multi-currency-revaluation-policy.md`
  - Lines: 328
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/treasury/IP-011-usecase-layer-for-fx-exposure.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - warehouse
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 15
- Sample verdict: FAIL
- Sample path: `microservices/warehouse/IP-014-rest-grpc-and-worker-surfaces-for-warehouse.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/warehouse/IP-004-domain-layer-for-picking-wave.md`
  - Lines: 80
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - whiteboard
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 30
- Sample verdict: PASS
- Sample path: `microservices/whiteboard/IP-024-threat-model-control-map.md`
  - Lines: 299
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/whiteboard/IP-005-rest-contract-surface.md`
  - Lines: 389
  - Unique ADR citations: 15
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - workflow-engine
- Severity: P2 IMPROVE
- Confidence: HIGH
- IP file count: 111
- Sample verdict: PASS
- Sample path: `microservices/workflow-engine/IP-journey-j08-cooloff-state-machine.md`
  - Lines: 802
  - Unique ADR citations: 10
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS
- Sample path: `microservices/workflow-engine/IP-journey-j93-in-dpdpa-rbi-overlay.md`
  - Lines: 400
  - Unique ADR citations: 11
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: PASS

### Appendix E - workflow-studio
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 42
- Sample verdict: FAIL
- Sample path: `microservices/workflow-studio/IP-013-observability-slo-manifests.md`
  - Lines: 118
  - Unique ADR citations: 3
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/workflow-studio/IP-journey-j36-manager-review-console.md`
  - Lines: 420
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

### Appendix E - workplace-integration
- Severity: P1 NEEDS-FIX
- Confidence: HIGH
- IP file count: 41
- Sample verdict: FAIL
- Sample path: `microservices/workplace-integration/IP-journey-j37-clock-in-geofence.md`
  - Lines: 420
  - Unique ADR citations: 8
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL
- Sample path: `microservices/workplace-integration/ip/IP-025-load-and-failure-fixtures.md`
  - Lines: 54
  - Unique ADR citations: 7
  - Bespoke signal: yes
  - Generic-hit count: 0
  - Verdict: FAIL

## Appendix F - Journey Substance

### j151..j158 Required Range
- j151: severity=P1 NEEDS-FIX; confidence=HIGH; exists=yes; files=1; md_lines=175; adr_citations=12; verdict=FAIL; dir=`docs/user-journeys/j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow`
- j152: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1777; adr_citations=15; verdict=PASS; dir=`docs/user-journeys/j152-ahmad-hassan-construction-site-incident-bilingual`
- j153: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1309; adr_citations=14; verdict=PASS; dir=`docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year`
- j154: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=2031; adr_citations=16; verdict=PASS; dir=`docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch`
- j155: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1996; adr_citations=15; verdict=PASS; dir=`docs/user-journeys/j155-stefan-kovacs-college-night-shift-and-finals-week`
- j156: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1620; adr_citations=18; verdict=PASS; dir=`docs/user-journeys/j156-carlos-reyes-ii-maintenance-emergency-after-hours`
- j157: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1724; adr_citations=12; verdict=PASS; dir=`docs/user-journeys/j157-diana-lazar-print-operator-batch-defect-and-quality-recall`
- j158: severity=P2 IMPROVE; confidence=HIGH; exists=yes; files=10; md_lines=1686; adr_citations=15; verdict=PASS; dir=`docs/user-journeys/j158-print-shop-cell-rebalance-shorts-creator-spike`

### j159+ Follow-On Range
- j159: severity=P2 IMPROVE; confidence=HIGH; files=10; md_lines=2099; adr_citations=16; verdict=PASS; dir=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work`
- j160: severity=P2 IMPROVE; confidence=HIGH; files=10; md_lines=2048; adr_citations=15; verdict=PASS; dir=`docs/user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard`
- j161: severity=P2 IMPROVE; confidence=HIGH; files=10; md_lines=2151; adr_citations=14; verdict=PASS; dir=`docs/user-journeys/j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination`
- j162: severity=P1 NEEDS-FIX; confidence=HIGH; files=1; md_lines=202; adr_citations=12; verdict=FAIL; dir=`docs/user-journeys/j162-print-operator-diana-lazar-night-shift-onboarding`
- j163: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j163-av-coordinator-jordan-park-board-meeting-cross-time-zone`
- j164: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j164-retired-hiroshi-tanaka-yearly-tax-and-pension`
- j165: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j165-cco-naveen-iyer-board-quarterly-compliance-report`
- j166: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j166-cso-mira-goldberg-strategic-acquisition-go-no-go`
- j167: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j167-cto-diego-vargas-platform-major-version-cutover`
- j168: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief`
- j169: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j169-cmo-felix-ng-multi-country-launch-with-locale-pack`
- j170: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j170-aiko-brown-sustainability-report-and-scope-3-supply-chain`
- j171: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege`
- j172: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j172-lev-kahn-investor-relations-shareholder-meeting-livestream`
- j173: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure`
- j174: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j174-sven-eriksson-treasury-eod-position-reconciliation`
- j175: severity=P1 NEEDS-FIX; confidence=HIGH; files=0; md_lines=0; adr_citations=0; verdict=FAIL; dir=`docs/user-journeys/j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution`

## Appendix G - Persona Substance Marker Coverage

- Dossier count: 129
- With exact marker: 60
- Without marker: 69
- Absolute 30+ marker requirement: PASS

| Persona dossier | Lines | Marker | Severity | Confidence |
| --- | ---: | --- | --- | --- |
| `docs/personas/accountant-ravi-iyer.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/ahmad-hassan.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/aiyana-singh.md` | 408 | yes | P2 IMPROVE | HIGH |
| `docs/personas/anya-mironova.md` | 407 | yes | P2 IMPROVE | HIGH |
| `docs/personas/apprentice-jakob-bauer.md` | 456 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/auditor-it-specialist-jakub-nowak.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/av-coordinator-jordan-park.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/bank-compliance-officer-rishi-bhattacharya.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/bank-ops-officer-olamide-adebanjo.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/bank-risk-manager-anders-pedersen.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/banker-external-hideki-watanabe.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/benefits-specialist-aoife-murphy.md` | 396 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/board-director-patrick-oreilly.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/board-secretary-florence-akinsanya.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/business-analyst-aditya-verma.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/cafeteria-manager-soyeon-kim.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/captain-chen-pilot.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/captain-olufemi.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/carlos-martinez-forklift.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cco-naveen-iyer.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ceo-aoki-tanaka.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cfo-helena-brandt.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/channel-partner-tomas-pieter.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/chris-volkov.md` | 407 | yes | P2 IMPROVE | HIGH |
| `docs/personas/chro-linda-foster.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ciso-yuki-park.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cleaning-supervisor-tomas-horak.md` | 466 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cmo-felix-ng.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/co-op-student-liam-murphy.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/coach-park.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/commercial-banker-frederik-hartmann.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/communications-specialist-charlotte-dubois.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/compliance-analyst-yui-hayashi.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/compliance-officer-tunde-bello.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/consultant-adekunle-adebayo.md` | 456 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/coo-akira-watanabe.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/corp-dev-senior-analyst-saanvi-mehta.md` | 462 | yes | P2 IMPROVE | HIGH |
| `docs/personas/corporate-relations-director-soo-yeon-han.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/credit-analyst-hina-mori.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cs-ic-lin-chen.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/cso-mira-goldberg.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/cto-diego-vargas.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/customer-champion-akemi-sato.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/customer-success-manager-sofia-rezende.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/d-and-i-director-maya-okoroafor.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/data-analyst-felipe-andrade.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/data-scientist-yu-chen.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/devon-williams.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/devops-engineer-olukayode-adejumo.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/devops-manager-pavel-korsak.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/diana-reyes.md` | 404 | yes | P2 IMPROVE | HIGH |
| `docs/personas/dr-tanaka-surgeon.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/engineering-manager-aisha-ali.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/executive-assistant-olivia-reyes.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/external-auditor-dimitri-volkov.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/external-auditor-hyo-jin-lee.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/father-lopez-priest.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/fellow-dr-tobias-klein.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/finance-director-mei-ling-wu.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/financial-analyst-wendy-lee.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/hiroshi-tanaka.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/hr-specialist-aoife-murphy.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/hrbp-jamal-carter.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/intern-manager-felicia-adamou.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/internal-comms-lead-ji-ho-yoon.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/investment-banker-yuna-ahn.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/investor-lp-aanya-kapoor.md` | 470 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ir-manager-lev-kahn.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ir-specialist-unnamed.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/it-manager-jamie-o-connor.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/jordan-lee.md` | 463 | yes | P2 IMPROVE | HIGH |
| `docs/personas/leave-specialist-margarethe-reinhart.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/legal-counsel-anika-mehta.md` | 466 | yes | P2 IMPROVE | HIGH |
| `docs/personas/legal-operations-stephen-park.md` | 456 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/mailroom-hae-won-kim.md` | 462 | yes | P2 IMPROVE | HIGH |
| `docs/personas/maintenance-tech-carlos-reyes-ii.md` | 466 | yes | P2 IMPROVE | HIGH |
| `docs/personas/marcus-chen.md` | 408 | yes | P2 IMPROVE | HIGH |
| `docs/personas/maria-santos.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/marketing-manager-olu-adeyemi.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/marketing-specialist-riya-sharma.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/medical-resident-dr-sun-mi-kim.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ms-patel-teacher.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/office-coordinator-phoebe-lin.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/office-manager-priya-ramanathan.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/officer-rodriguez-police.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ombudsperson-felix-tan.md` | 462 | yes | P2 IMPROVE | HIGH |
| `docs/personas/outside-counsel-wei-yi-chen.md` | 396 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/paralegal-tomas-novak.md` | 456 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/pr-firm-beatriz-fernandez.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/pr-manager-helena-sato.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/print-operator-diana-lazar.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/priya-krishnan.md` | 407 | yes | P2 IMPROVE | HIGH |
| `docs/personas/procurement-manager-wei-liu.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/procurement-specialist-beata-kowalski.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/product-designer-akihiro-sato.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/product-manager-lily-chang.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/project-manager-soo-jin-park.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/public-affairs-director-carlos-mendez.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/receptionist-daria-volkova.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/recruiter-marcus-iv.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/recruiting-manager-hina-suzuki.md` | 467 | yes | P2 IMPROVE | HIGH |
| `docs/personas/regulator-inspector-sergei-petrov.md` | 395 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/retail-banker-sebastian-vega.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/retirement-plan-admin-bryce-williams.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/returning-intern-jia-han.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/sales-ae-maya-lindqvist.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/sales-manager-anthony-costa.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/sam-okafor.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/sarah-kim-delivery.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/sdr-kofi-asante.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/security-analyst-anna-petrova.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/security-guard-stefan-kovacs.md` | 469 | yes | P2 IMPROVE | HIGH |
| `docs/personas/software-engineer-hugo-tanaka.md` | 452 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/strategic-advisor-rita-almeida.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/summer-intern-priscilla-sharma.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/support-rep-nadia-hassani.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/sustainability-officer-aiko-brown.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/tax-analyst-ji-sung-park.md` | 458 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/tomas-garcia-jr-farmer.md` | 406 | yes | P2 IMPROVE | HIGH |
| `docs/personas/tomas-garcia.md` | 408 | yes | P2 IMPROVE | HIGH |
| `docs/personas/total-rewards-manager-nilufer-demir.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/trader-mei-lin.md` | 405 | yes | P2 IMPROVE | HIGH |
| `docs/personas/training-specialist-mehmet-yilmaz.md` | 456 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/treasury-ops-sven-eriksson.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/ux-researcher-adaeze-nwosu.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/venture-partner-lucas-muller.md` | 459 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/wealth-manager-aamir-khan.md` | 468 | yes | P2 IMPROVE | HIGH |
| `docs/personas/wellness-program-manager-akira-sato.md` | 457 | no | P1 NEEDS-FIX | HIGH |
| `docs/personas/yejin-park.md` | 412 | yes | P2 IMPROVE | HIGH |

## Appendix H - Capability-Tier Registry

- Registry path: `registry/capability-tiers`
- Exists: yes
- Tier definitions present: yes
- Microservice tier mapping present: yes
- Vendor tier mapping present: yes
- Registry file: `registry/capability-tiers/bronze.json`
- Registry file: `registry/capability-tiers/checkpoint.json`
- Registry file: `registry/capability-tiers/gold.json`
- Registry file: `registry/capability-tiers/index.json`
- Registry file: `registry/capability-tiers/microservice-tier-mapping.yaml`
- Registry file: `registry/capability-tiers/platinum.json`
- Registry file: `registry/capability-tiers/silver.json`
- Registry file: `registry/capability-tiers/vendor-tier-mapping.yaml`

## Appendix I - Manifest Fields

| Microservice | Manifest | naming_justifications | audit field | Severity | Confidence |
| --- | --- | --- | --- | --- | --- |
| analytics | `microservices/analytics/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| api-gateway | `microservices/api-gateway/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| application | `microservices/application/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| audit-chain | `microservices/audit-chain/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| calendar | `microservices/calendar/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| cell pattern successors | `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md` | yes | yes | P2 IMPROVE | HIGH |
| cloud-iac | `microservices/cloud-iac/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| cloud-k8s | `microservices/cloud-k8s/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| cloud-secrets | `microservices/cloud-secrets/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| comms-email | `microservices/comms-email/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| community | `microservices/community/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| compliance | `microservices/compliance/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| connect | `microservices/connect/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| consent-graph | `microservices/consent-graph/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| contact-center | `microservices/contact-center/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| contract-lifecycle-management | `microservices/contract-lifecycle-management/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| crm | `microservices/crm/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| data-pipeline | `microservices/data-pipeline/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| data-warehouse | `microservices/data-warehouse/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| design-collaboration | `microservices/design-collaboration/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| detection | `microservices/detection/manifest.json` | no | yes | P1 NEEDS-FIX | HIGH |
| developer-sdk | `microservices/developer-sdk/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| docs | `microservices/docs/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| drive | `microservices/drive/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| feature-flags | `microservices/feature-flags/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| financial-planning | `microservices/financial-planning/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| finops-portal | `microservices/finops-portal/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| forms | `microservices/forms/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| foundry | `microservices/foundry/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| global-trade | `microservices/global-trade/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| governance | `microservices/governance/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| healthcare-integration | `microservices/healthcare-integration/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| identity | `microservices/identity/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| incident-management | `microservices/incident-management/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| intelligence | `microservices/intelligence/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| itsm | `microservices/itsm/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| learning-management | `microservices/learning-management/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| mail | `microservices/mail/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| marketing-automation | `microservices/marketing-automation/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| marketplace | `microservices/marketplace/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| meet | `microservices/meet/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| messenger | `microservices/messenger/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| network | `microservices/network/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| notes | `microservices/notes/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| observability | `microservices/observability/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| ontology | `microservices/ontology/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| ops-dashboard-control-center | `microservices/ops-dashboard-control-center/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| payments | `microservices/payments/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| performance-management | `microservices/performance-management/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| plant-maintenance | `microservices/plant-maintenance/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| plugin-app-store | `microservices/plugin-app-store/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| production-planning | `microservices/production-planning/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| quality-management | `microservices/quality-management/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| real-estate | `microservices/real-estate/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| recordings | `microservices/recordings/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| sheets | `microservices/sheets/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| shorts | `microservices/shorts/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| sites | `microservices/sites/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| slides | `microservices/slides/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| social | `microservices/social/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| supply-chain-planning | `microservices/supply-chain-planning/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| tasks | `microservices/tasks/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| tenancy | `microservices/tenancy/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| translate | `microservices/translate/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| treasury | `microservices/treasury/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| warehouse | `microservices/warehouse/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| whiteboard | `microservices/whiteboard/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| workflow-engine | `microservices/workflow-engine/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| workflow-studio | `microservices/workflow-studio/manifest.json` | yes | yes | P2 IMPROVE | HIGH |
| workplace-integration | `microservices/workplace-integration/manifest.json` | yes | yes | P2 IMPROVE | HIGH |

## Appendix J - Contract Conformance Inventory

### OpenAPI Required Version 3.2.0
- Severity: P2 IMPROVE
- Confidence: HIGH
- Pass: 96/96 (100.0%)
- OpenAPI file: `microservices/analytics/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/application/contracts/openapi/application.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/application/contracts/openapi/tenant-admin-console.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/audit-chain/contracts/openapi/audit-chain.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/calendar/contracts/openapi/calendar.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI authority moved to successor contracts per `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md`; retired cell OpenAPI verdict is historical.
- OpenAPI file: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/comms-email/contracts/openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/community/contracts/openapi/community.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/compliance/contracts/openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/connect/contracts/connect-retirement.openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/connect/contracts/openapi/connect-integration.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/connect/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/consent-graph/contracts/openapi/consent-graph.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/contact-center/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/contact-center/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/crm/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/data-pipeline/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/data-pipeline/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/data-warehouse/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/data-warehouse/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/design-collaboration/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/design-collaboration/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/detection/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/developer-sdk/contracts/openapi/developer-sdk.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/developer-sdk/contracts/openapi/oya-ecosystem.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/docs/contracts/openapi/docs.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/drive/contracts/openapi/drive.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/feature-flags/contracts/feature-flags.openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/feature-flags/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/financial-planning/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/financial-planning/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/finops-portal/contracts/tenant-invoice-public.openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/forms/contracts/openapi/forms.openapi.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/eval-eval-runner.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/guardrails-guardrails.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/providers-provider-router.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/foundry/contracts/openapi/supervisor-foundry-supervisor.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/global-trade/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/governance/contracts/openapi/governance.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/healthcare-integration/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/identity/contracts/openapi/identity.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/identity/contracts/openapi/multi-context-split.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/incident-management/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/incident-management/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/intelligence/contracts/openapi/intelligence.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/itsm/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/itsm/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/learning-management/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/learning-management/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/mail/contracts/openapi/mail.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/marketing-automation/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/marketing-automation/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/marketplace/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/meet/contracts/openapi/meet.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/messenger/contracts/openapi/messenger.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/network/contracts/openapi/network.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/notes/contracts/openapi/notes.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/observability/contracts/openapi/slo-engine.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/ontology/contracts/openapi/ontology.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/ops-dashboard-control-center/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/payments/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/performance-management/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/performance-management/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/plant-maintenance/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/plugin-app-store/contracts/openapi/plugin-app-store.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/production-planning/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/quality-management/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/real-estate/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/recordings/contracts/openapi/recordings.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/sheets/contracts/openapi/sheets.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/shorts/contracts/openapi/shorts.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/sites/contracts/openapi/sites.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/slides/contracts/openapi/slides.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/social/contracts/openapi/social.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/supply-chain-planning/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/tasks/contracts/openapi/tasks.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/tenancy/contracts/openapi/tenancy.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/translate/contracts/openapi/translate.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/treasury/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/warehouse/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/whiteboard/contracts/local-openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/whiteboard/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`; version=3.2.0; verdict=PASS.
- OpenAPI file: `microservices/workplace-integration/contracts/openapi-v1.yaml`; version=3.2.0; verdict=PASS.

### AsyncAPI Required Version 3.1.0
- Severity: P2 IMPROVE
- Confidence: HIGH
- Pass: 106/106 (100.0%)
- AsyncAPI file: `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j152-ahmad-hassan-construction-site-incident-bilingual/schemas/asyncapi-crane-telemetry.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/schemas/asyncapi-payments-ledger.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j91-us-state-money-transmitter-licensing/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j92-br-lgpd-dsar-with-us-parent/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j94-sox-404-public-company-controls/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/schemas/asyncapi-overlay-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/analytics/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/application/contracts/asyncapi/application-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI authority moved to tenancy, cloud-iac, observability, api-gateway, and audit-chain events per `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md`; retired cell event verdict is historical.
- AsyncAPI file: `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/comms-email/contracts/asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/community/contracts/asyncapi/community-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/compliance/contracts/asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/connect/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/connect/contracts/connect-retirement.asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/contact-center/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/contact-center/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/crm/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/data-pipeline/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/data-warehouse/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/design-collaboration/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/detection/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/developer-sdk/contracts/asyncapi/developer-sdk-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/docs/contracts/asyncapi/docs-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/drive/contracts/asyncapi/drive-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/feature-flags/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/financial-planning/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/finops-portal/contracts/focus-export-internal.asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/global-trade/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/governance/contracts/asyncapi/governance-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/healthcare-integration/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/identity/contracts/asyncapi/identity-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/identity/contracts/asyncapi/multi-context-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/incident-management/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/incident-management/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/itsm/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/itsm/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/learning-management/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/learning-management/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/mail/contracts/asyncapi/mail-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/marketing-automation/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/marketplace/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/meet/contracts/asyncapi/meet-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/messenger/contracts/asyncapi/messenger-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/network/contracts/asyncapi/network-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/notes/contracts/asyncapi/notes-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/ops-dashboard-control-center/contracts/asyncapi/ops-dashboard-control-center-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/ops-dashboard-control-center/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/payments/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/performance-management/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/performance-management/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/plant-maintenance/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/plugin-app-store/contracts/asyncapi/plugin-app-store-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/production-planning/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/quality-management/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/real-estate/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/sheets/contracts/asyncapi/sheets-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/shorts/contracts/asyncapi/shorts-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/sites/contracts/asyncapi/sites-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/slides/contracts/asyncapi/slides-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/social/contracts/asyncapi/social-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/supply-chain-planning/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/tenancy/contracts/asyncapi/tenant-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/translate/contracts/asyncapi/translate-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/treasury/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/warehouse/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/whiteboard/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`; version=3.1.0; verdict=PASS.
- AsyncAPI file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`; version=3.1.0; verdict=PASS.

### proto Required Version proto3
- Severity: P2 IMPROVE
- Confidence: HIGH
- Pass: 112/112 (100.0%)
- proto file: `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j152-ahmad-hassan-construction-site-incident-bilingual/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j155-stefan-kovacs-college-night-shift-and-finals-week/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j156-carlos-reyes-ii-maintenance-emergency-after-hours/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j157-diana-lazar-print-operator-batch-defect-and-quality-recall/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j158-print-shop-cell-rebalance-shorts-creator-spike/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j91-us-state-money-transmitter-licensing/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j92-br-lgpd-dsar-with-us-parent/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j94-sox-404-public-company-controls/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/schemas/journey-messages.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/analytics/contracts/analytics.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/api-gateway/contracts/api_gateway.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/application/contracts/proto/application.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/audit-chain/contracts/proto/audit-chain.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/calendar/contracts/proto/calendar.proto`; version=proto3; verdict=PASS.
- Proto authority moved to successor service contracts per `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md`; retired cell proto verdict is historical.
- proto file: `microservices/cloud-iac/contracts/proto/cloud-iac.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/comms-email/contracts/comms_email.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/community/contracts/proto/community.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/compliance/contracts/compliance.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/connect/contracts/connect_retirement.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/connect/contracts/proto/connect_integration.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/consent-graph/contracts/proto/consent-graph.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/contact-center/contracts/contact-center-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/contact-center/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/contract-lifecycle-management/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/crm/contracts/crm-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/data-pipeline/contracts/data-pipeline-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/data-pipeline/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/data-warehouse/contracts/data-warehouse-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/data-warehouse/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/design-collaboration/contracts/design-collaboration-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/design-collaboration/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/detection/contracts/detection-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/developer-sdk/contracts/proto/developer-sdk.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/docs/contracts/proto/docs.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/drive/contracts/proto/drive.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/feature-flags/contracts/feature-flags-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/feature-flags/contracts/feature_flags.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/financial-planning/contracts/financial-planning-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/financial-planning/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/finops-portal/contracts/cost-allocation-policy-internal.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/forms/contracts/proto/forms.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/eval-eval_runner.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/evidence-foundry-evidence.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/guardrails-guardrails.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/providers-provider-invoke.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/runtime-foundry-runtime.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/foundry/contracts/proto/supervisor-foundry-supervisor.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/global-trade/contracts/global-trade-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/governance/contracts/proto/governance.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/healthcare-integration/contracts/healthcare-integration-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/healthcare-integration/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/identity/contracts/proto/identity.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/identity/contracts/proto/multi_context_split.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/incident-management/contracts/incident-management-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/incident-management/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/intelligence/contracts/proto/intelligence-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/intelligence/contracts/proto/intelligence.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/itsm/contracts/itsm-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/itsm/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/learning-management/contracts/learning-management-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/learning-management/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/mail/contracts/proto/mail.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/marketing-automation/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/marketing-automation/contracts/marketing-automation-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/marketplace/contracts/marketplace-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/meet/contracts/proto/meet.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/messenger/contracts/proto/messenger.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/network/contracts/proto/network.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/notes/contracts/proto/notes.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/observability/contracts/proto/slo-engine.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/ontology/contracts/proto/ontology.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/ops-dashboard-control-center/contracts/proto/ops_dashboard_control_center.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/payments/contracts/payments-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/performance-management/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/performance-management/contracts/performance-management-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/plant-maintenance/contracts/plant-maintenance-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/plugin-app-store/contracts/proto/plugin-app-store.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/production-planning/contracts/production-planning-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/quality-management/contracts/quality-management-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/real-estate/contracts/real-estate-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/recordings/contracts/proto/recordings.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/sheets/contracts/proto/sheets.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/shorts/contracts/proto/shorts.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/sites/contracts/proto/sites.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/slides/contracts/proto/slides.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/social/contracts/proto/social.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/supply-chain-planning/contracts/supply-chain-planning-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/tasks/contracts/proto/tasks.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/tenancy/contracts/proto/tenancy.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/translate/contracts/proto/translate.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/treasury/contracts/treasury-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/warehouse/contracts/warehouse-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/whiteboard/contracts/local-operations-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/whiteboard/contracts/whiteboard-v1.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/workflow-engine/contracts/proto/workflow-engine.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/workflow-studio/contracts/proto/workflow-studio.proto`; version=proto3; verdict=PASS.
- proto file: `microservices/workplace-integration/contracts/workplace-integration-v1.proto`; version=proto3; verdict=PASS.

## Appendix K - BYOK Disambiguation

- Authority check: ADR-0255 §D-4 explicitly defines provider-BYOK and distinguishes it from encryption-BYOK.
- Authority check: ADR-0251 §D-10 explicitly defines encryption-BYOK and distinguishes it from provider-BYOK.
- Runbook check: provider rotation and encryption rotation runbooks are separate and cross-link each other.
- Corpus keyword files: 572; unambiguous by heuristic: 460; ambiguous by heuristic: 112.

### K.1 Ambiguous BYOK Artifacts by Heuristic
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/architecture/keystone-bundle-audit-report.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/architecture/keystone-bundle-reading-order.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/architecture/corpus-rigor-audit-2026-05-20.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/README.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/handshake.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/story.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/ux-flow.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/integration-test-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard/integration-test-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/README.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/integration-test-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/README.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/story.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/schemas/asyncapi-payments-ledger.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/schemas/JobSearchFilterSpec.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/schemas/openapi-dual-tenant-mba.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`docs/user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/schemas/mba-application-state-machine.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/payments/manifest.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/api-gateway/manifest.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/failure-modes.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/dpia.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/capacity-model.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/CHANGELOG.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/PHASE-02-CONSUMER-BRAND-SURFACE.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/PHASE-01-INTELLIGENCE-TWO-LAYER-MVP.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/cost-budget.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/IP-013-adapter-google-vertex.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-011-usecase-layer-for-production-order.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-010-usecase-layer-for-routing-step.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-019-sop-horizon-monthly-cycle-with-executive-signoff-gate.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-012-usecase-layer-for-shop-floor-release.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-023-alternative-routing-engagement-decision-engine.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-024-mes-handshake-bidirectional-event-flow-isa-95.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-022-long-term-planning-versus-short-term-planning-split.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-018-ddmrp-buffer-profile-authoring-and-daf-recalc.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-021-capacity-leveling-finite-scheduling-forward-backward-bottleneck.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-020-production-version-selection-with-co-product-yield-variance.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-015-integration-tests-for-production-planning.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/production-planning/IP-025-production-line-balancing-takt-time-workstation-load-smoothing.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/feature-flags/backfill-replay.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/feature-flags/dpia.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-017-permit-to-work-issuance-workflow.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-012-usecase-layer-for-downtime-window.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-008-usecase-layer-for-maintenance-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-002-domain-layer-for-maintenance-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-001-domain-layer-for-equipment-master.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-022-mtbf-weibull-fitting-reliability-analytics.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-011-usecase-layer-for-technician-dispatch.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-007-usecase-layer-for-equipment-master.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-004-domain-layer-for-spare-part-reservation.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-024-maintenance-strategy-cycle-generation-due-date-calculator.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-025-equipment-hierarchy-class-characteristic-schema-and-relocation.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-003-domain-layer-for-work-order.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-021-reliability-centered-maintenance-decision-logic.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-023-maintenance-kpi-scorecard-oee-mttr-firsttimefix.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-009-usecase-layer-for-work-order.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-016-safety-loto-9-state-machine-with-audit-chain.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-019-spare-parts-mrp-linkage.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-010-usecase-layer-for-spare-part-reservation.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-005-domain-layer-for-technician-dispatch.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/plant-maintenance/IP-006-domain-layer-for-downtime-window.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/cloud-secrets/sdk-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/sdk-plan.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/README.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/competitor-parity-matrix.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/capabilities/oauth-grant-initiate.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/contracts/asyncapi-v1.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/contracts/openapi-v1.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/stripe.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/opsgenie.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/jira.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/mailgun.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/launchdarkly.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/shopify.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/trello.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/sendgrid.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/kakaopay.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/pagerduty.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/toss-payments.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/sentry.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/postgres-direct.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/bigquery.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/snowflake.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/segment.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/mixpanel.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/twilio.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/datadog.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/catalog/connectors/linear.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/contracts/asyncapi/connect-integration-events.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/connect/contracts/openapi/connect-integration.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/social/iac/openbao-policy.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/ontology/iac/openbao-policy.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/feature-flags/runbooks/audit-replay.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/tenancy/iac/secret-bindings.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/mail/runbooks/phi-leak-recovery.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/meet/onboarding/realtime-engineer-first-week.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/capabilities/dispatch.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/capabilities/routing.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/catalog/oya-intelligence-providers-adapter-openai.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/catalog/oya-intelligence-credential-resolver-usecase.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/catalog/oya-intelligence-providers-adapter-anthropic.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/dashboards/finops-cost-attribution.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/dashboards/byok-vs-platform-default-mix.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/policy/tenant-isolation.md`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/intelligence/contracts/openapi/intelligence-v1.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/api-gateway/scorecards/overrides.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/finops-portal/iac/secret-bindings.yaml`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`microservices/comms-email/iac/secret-bindings.yaml`; provider_signals=1; encryption_signals=1; finding=bare or locally ambiguous BYOK reference.
- Severity: P1 NEEDS-FIX; Confidence: MED; path=`specs/tenant-model.json`; provider_signals=0; encryption_signals=0; finding=bare or locally ambiguous BYOK reference.

### K.2 Disambiguated BYOK Artifact Sample
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/runbooks/byok-rotation-encryption-tenant-duress.md`; provider_signals=0; encryption_signals=62; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/runbooks/meta-trust-root-recovery.md`; provider_signals=0; encryption_signals=20; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/runbooks/provider-credential-leak-response.md`; provider_signals=2; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/runbooks/byok-rotation-provider-tenant-duress.md`; provider_signals=3; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`; provider_signals=0; encryption_signals=10; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`; provider_signals=36; encryption_signals=21; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`; provider_signals=3; encryption_signals=39; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md`; provider_signals=0; encryption_signals=42; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`; provider_signals=0; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0296-library-first-credential-sidecar.md`; provider_signals=6; encryption_signals=9; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`; provider_signals=0; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`; provider_signals=7; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0293-foundry-meta-trust-root.md`; provider_signals=0; encryption_signals=53; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0312-court-warrant-scoped-piercing.md`; provider_signals=0; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0253-amendment-http3-fallback-strict-tls-ech-pqc.md`; provider_signals=0; encryption_signals=18; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0284-platform-owner-name-indirection.md`; provider_signals=1; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0045-database-tier-strategy.md`; provider_signals=0; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md`; provider_signals=0; encryption_signals=18; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0243-cedar-as-universal-gate.md`; provider_signals=1; encryption_signals=20; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md`; provider_signals=0; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md`; provider_signals=2; encryption_signals=7; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0255-amendment-library-first-network-opt-in-clarification.md`; provider_signals=3; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0253-network-topology-edge-service-mesh.md`; provider_signals=0; encryption_signals=5; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0245-substrate-vs-product-layering.md`; provider_signals=0; encryption_signals=8; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0246-policy-engine-substrate-promotion.md`; provider_signals=1; encryption_signals=19; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md`; provider_signals=3; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0254-deployment-model-spectrum.md`; provider_signals=0; encryption_signals=7; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`; provider_signals=4; encryption_signals=23; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/personas/MASTER-ROSTER-2026-05-21.md`; provider_signals=1; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-stories/b2b-work-surfaces.md`; provider_signals=0; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/unified-ecosystem-thesis-2026-05-21.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/keystone-bundle-2026-05-20-lessons-learned.md`; provider_signals=4; encryption_signals=8; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md`; provider_signals=1; encryption_signals=4; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/microservices-corpus-line-audit-2026-05-21.md`; provider_signals=3; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/memory-spec-runbook-audit-2026-05-21.md`; provider_signals=1; encryption_signals=5; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md`; provider_signals=0; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/ip-corpus-line-audit-2026-05-21.md`; provider_signals=1; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/adr-corpus-line-audit-2026-05-21.md`; provider_signals=5; encryption_signals=9; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/training-cost-doctrine-2026-05-21.md`; provider_signals=0; encryption_signals=4; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md`; provider_signals=0; encryption_signals=5; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/wave-3-g-executive-briefing-2026-05-21.md`; provider_signals=0; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/hyperscaler-pattern-attribution.md`; provider_signals=0; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/architecture/keystone-bundle-2026-05-20-synthesis.md`; provider_signals=4; encryption_signals=16; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/standards/multispectrum-review-v2.4.0-cadence.md`; provider_signals=0; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/standards/documentation-rigor.md`; provider_signals=2; encryption_signals=16; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/standards/fips-hsm-substrate-root-signing.md`; provider_signals=2; encryption_signals=149; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/standards/messenger-e2e-encryption-mls.md`; provider_signals=0; encryption_signals=53; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/handshake.md`; provider_signals=43; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/story.md`; provider_signals=4; encryption_signals=12; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/handshake.md`; provider_signals=43; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/story.md`; provider_signals=4; encryption_signals=11; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/handshake.md`; provider_signals=45; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/handshake.md`; provider_signals=44; encryption_signals=45; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/handshake.md`; provider_signals=43; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/story.md`; provider_signals=4; encryption_signals=12; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j126-government-auditor-3pao-conducts-fedramp-audit/story.md`; provider_signals=0; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/handshake.md`; provider_signals=43; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/story.md`; provider_signals=4; encryption_signals=12; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/story.md`; provider_signals=1; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/handshake.md`; provider_signals=43; encryption_signals=44; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/handshake.md`; provider_signals=43; encryption_signals=44; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/handshake.md`; provider_signals=44; encryption_signals=45; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/handshake.md`; provider_signals=43; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/story.md`; provider_signals=4; encryption_signals=12; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/handshake.md`; provider_signals=43; encryption_signals=44; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/handshake.md`; provider_signals=44; encryption_signals=45; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/handshake.md`; provider_signals=44; encryption_signals=45; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/README.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/handshake.md`; provider_signals=45; encryption_signals=46; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/story.md`; provider_signals=4; encryption_signals=6; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/README.md`; provider_signals=1; encryption_signals=3; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/handshake.md`; provider_signals=44; encryption_signals=47; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/story.md`; provider_signals=4; encryption_signals=12; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/ux-flow.md`; provider_signals=1; encryption_signals=2; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/integration-test-plan.md`; provider_signals=30; encryption_signals=31; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/schemas/kr-csap-audit-pull.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/schemas/ai-act-credit-appeal.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/schemas/nis2-breach-cadence.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/schemas/gdpr-dsar-cascade.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/schemas/uk-aadc-minor-ux.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/schemas/fedramp-il5-airgap.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/schemas/dsa-transparency-report.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/schemas/ccpa-do-not-sell-cascade.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/schemas/hipaa-phi-workflow.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/schemas/cn-pipl-cac-assessment.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j21-personal-signup-passkey-first-dm/schemas/principal-created-event.json`; provider_signals=1; encryption_signals=0; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/schemas/kr-pipa-research-transfer.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/schemas/kr-fss-fraud-freeze.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j84-jp-appi-elder-user-consent/schemas/jp-appi-elder-consent.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j88-au-irap-protected-tenant/schemas/au-irap-protected-tenant.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/schemas/pci-tokenized-payment.json`; provider_signals=1; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`docs/products/cloud/PRD.md`; provider_signals=0; encryption_signals=44; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j89-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j77-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j78-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/ARCHITECTURE.md`; provider_signals=20; encryption_signals=4; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j82-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j85-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j90-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j83-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/compliance.md`; provider_signals=15; encryption_signals=7; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j79-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j76-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j88-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j87-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j84-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/workflow-engine/IP-journey-j80-cadence-orchestrator.md`; provider_signals=49; encryption_signals=49; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/payments/PRD.md`; provider_signals=10; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/payments/ARCHITECTURE.md`; provider_signals=4; encryption_signals=1; finding=local disambiguation signal present.
- Severity: P2 IMPROVE; Confidence: MED; path=`microservices/payments/threat-model.md`; provider_signals=1; encryption_signals=4; finding=local disambiguation signal present.

## Appendix L - Reverse Cross-Reference Web Sample

- Sample rationale: these eight ADRs are load-bearing because they govern tenant doctrine, Cedar gating, universal tenant scoping, substrate/product layering, policy-engine substrate, BYOK/encryption certification, provider-BYOK intelligence substrate, and B2B industry-leader coverage.
### ADR-0242
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md`
- Reverse references found: 1296
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/machine-readable/decisions.json`
- Reverse-ref sample: `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md`
- Reverse-ref sample: `docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse-ref sample: `docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`
- Reverse-ref sample: `docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`
- Reverse-ref sample: `docs/decisions/ADR-0293-foundry-meta-trust-root.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`
- Reverse-ref sample: `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0312-court-warrant-scoped-piercing.md`
- Reverse-ref sample: `docs/decisions/ADR-0306-disaster-mode-cell-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0253-amendment-http3-fallback-strict-tls-ech-pqc.md`

### ADR-0243
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0243-cedar-as-universal-gate.md`
- Reverse references found: 2186
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/performance-budgets/README.md`
- Reverse-ref sample: `docs/performance-budgets/deployment-model-slo-budgets.md`
- Reverse-ref sample: `docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md`
- Reverse-ref sample: `docs/performance-budgets/cedar-hot-path-1ms-p99.md`
- Reverse-ref sample: `docs/performance-budgets/edge-first-byte-50ms-p99.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-encryption-tenant-duress.md`
- Reverse-ref sample: `docs/runbooks/self-modification-rollback.md`
- Reverse-ref sample: `docs/runbooks/compliance-pack-emergency-suspension.md`
- Reverse-ref sample: `docs/runbooks/meta-trust-root-recovery.md`
- Reverse-ref sample: `docs/runbooks/provider-credential-leak-response.md`
- Reverse-ref sample: `docs/runbooks/tenant-data-residency-violation.md`
- Reverse-ref sample: `docs/runbooks/bootstrap-ci-compromise.md`
- Reverse-ref sample: `docs/runbooks/compliance-pack-revocation.md`
- Reverse-ref sample: `docs/runbooks/shamir-share-loss-or-coercion.md`
- Reverse-ref sample: `docs/runbooks/cedar-fragment-emergency-rollback.md`
- Reverse-ref sample: `docs/runbooks/cell-evacuation.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md`

### ADR-0244
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`
- Reverse references found: 3733
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-encryption-tenant-duress.md`
- Reverse-ref sample: `docs/runbooks/provider-credential-leak-response.md`
- Reverse-ref sample: `docs/runbooks/tenant-data-residency-violation.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md`
- Reverse-ref sample: `docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse-ref sample: `docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`
- Reverse-ref sample: `docs/decisions/ADR-0319-front-middle-back-office-information-barrier.md`
- Reverse-ref sample: `docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`
- Reverse-ref sample: `docs/decisions/ADR-0293-foundry-meta-trust-root.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`
- Reverse-ref sample: `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`

### ADR-0245
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0245-substrate-vs-product-layering.md`
- Reverse references found: 366
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse-ref sample: `docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`
- Reverse-ref sample: `docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`
- Reverse-ref sample: `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0312-court-warrant-scoped-piercing.md`
- Reverse-ref sample: `docs/decisions/ADR-0306-disaster-mode-cell-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0305-delegated-agent-authority-chain.md`
- Reverse-ref sample: `docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md`
- Reverse-ref sample: `docs/decisions/ADR-0284-platform-owner-name-indirection.md`
- Reverse-ref sample: `docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md`
- Reverse-ref sample: `docs/decisions/ADR-0300-whistleblower-press-freedom-anonymity.md`

### ADR-0246
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`, `docs/decisions/ADR-0246-policy-engine-substrate-promotion.md`
- Reverse references found: 600
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/performance-budgets/README.md`
- Reverse-ref sample: `docs/performance-budgets/deployment-model-slo-budgets.md`
- Reverse-ref sample: `docs/performance-budgets/cedar-hot-path-1ms-p99.md`
- Reverse-ref sample: `docs/performance-budgets/edge-first-byte-50ms-p99.md`
- Reverse-ref sample: `docs/runbooks/meta-trust-root-recovery.md`
- Reverse-ref sample: `docs/runbooks/shamir-share-loss-or-coercion.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md`
- Reverse-ref sample: `docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse-ref sample: `docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`
- Reverse-ref sample: `docs/decisions/ADR-0293-foundry-meta-trust-root.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`
- Reverse-ref sample: `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0312-court-warrant-scoped-piercing.md`

### ADR-0251
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse references found: 1213
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-encryption-tenant-duress.md`
- Reverse-ref sample: `docs/runbooks/compliance-pack-emergency-suspension.md`
- Reverse-ref sample: `docs/runbooks/provider-credential-leak-response.md`
- Reverse-ref sample: `docs/runbooks/tenant-data-residency-violation.md`
- Reverse-ref sample: `docs/runbooks/compliance-pack-revocation.md`
- Reverse-ref sample: `docs/runbooks/shamir-share-loss-or-coercion.md`
- Reverse-ref sample: `docs/runbooks/cell-evacuation.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0303-cognitive-impairment-decision-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`
- Reverse-ref sample: `docs/decisions/ADR-0302-deceased-user-inheritance-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0299-account-recovery-resilience.md`
- Reverse-ref sample: `docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md`
- Reverse-ref sample: `docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`
- Reverse-ref sample: `docs/decisions/ADR-0319-front-middle-back-office-information-barrier.md`
- Reverse-ref sample: `docs/decisions/ADR-0293-foundry-meta-trust-root.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`

### ADR-0255
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`, `docs/decisions/ADR-0255-amendment-library-first-network-opt-in-clarification.md`
- Reverse references found: 341
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/runbooks/provider-credential-leak-response.md`
- Reverse-ref sample: `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- Reverse-ref sample: `docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md`
- Reverse-ref sample: `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`
- Reverse-ref sample: `docs/decisions/ADR-0307-detection-substrate-streaming-batch.md`
- Reverse-ref sample: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`
- Reverse-ref sample: `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`
- Reverse-ref sample: `docs/decisions/ADR-0250-build-ahead-of-certification-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0296-library-first-credential-sidecar.md`
- Reverse-ref sample: `docs/decisions/ADR-0246-amendment-library-first-network-opt-in-clarification.md`
- Reverse-ref sample: `docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md`
- Reverse-ref sample: `docs/decisions/ADR-0293-foundry-meta-trust-root.md`
- Reverse-ref sample: `docs/decisions/ADR-0252-time-coordination-distributed-consistency.md`
- Reverse-ref sample: `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0305-delegated-agent-authority-chain.md`
- Reverse-ref sample: `docs/decisions/ADR-0284-platform-owner-name-indirection.md`
- Reverse-ref sample: `docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md`
- Reverse-ref sample: `docs/decisions/ADR-0243-cedar-as-universal-gate.md`
- Reverse-ref sample: `docs/decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md`
- Reverse-ref sample: `docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md`
- Reverse-ref sample: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Reverse-ref sample: `docs/decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md`
- Reverse-ref sample: `docs/decisions/ADR-0253-network-topology-edge-service-mesh.md`
- Reverse-ref sample: `docs/decisions/ADR-0245-substrate-vs-product-layering.md`

### ADR-0321
- Severity: P2 IMPROVE
- Confidence: HIGH
- Own files: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Reverse references found: 965
- Verdict: PASS
- Reverse-ref sample: `docs/GLOSSARY.md`
- Reverse-ref sample: `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`
- Reverse-ref sample: `docs/personas/corporate-relations-director-soo-yeon-han.md`
- Reverse-ref sample: `docs/personas/communications-specialist-charlotte-dubois.md`
- Reverse-ref sample: `docs/personas/apprentice-jakob-bauer.md`
- Reverse-ref sample: `docs/personas/cs-ic-lin-chen.md`
- Reverse-ref sample: `docs/personas/security-analyst-anna-petrova.md`
- Reverse-ref sample: `docs/personas/intern-manager-felicia-adamou.md`
- Reverse-ref sample: `docs/personas/co-op-student-liam-murphy.md`
- Reverse-ref sample: `docs/personas/fellow-dr-tobias-klein.md`
- Reverse-ref sample: `docs/personas/receptionist-daria-volkova.md`
- Reverse-ref sample: `docs/personas/devops-manager-pavel-korsak.md`
- Reverse-ref sample: `docs/personas/retail-banker-sebastian-vega.md`
- Reverse-ref sample: `docs/personas/channel-partner-tomas-pieter.md`
- Reverse-ref sample: `docs/personas/product-designer-akihiro-sato.md`
- Reverse-ref sample: `docs/personas/ir-specialist-unnamed.md`
- Reverse-ref sample: `docs/personas/coach-park.md`
- Reverse-ref sample: `docs/personas/board-secretary-florence-akinsanya.md`
- Reverse-ref sample: `docs/personas/customer-success-manager-sofia-rezende.md`
- Reverse-ref sample: `docs/personas/legal-operations-stephen-park.md`
- Reverse-ref sample: `docs/personas/sustainability-officer-aiko-brown.md`
- Reverse-ref sample: `docs/personas/investor-lp-aanya-kapoor.md`
- Reverse-ref sample: `docs/personas/software-engineer-hugo-tanaka.md`
- Reverse-ref sample: `docs/personas/training-specialist-mehmet-yilmaz.md`
- Reverse-ref sample: `docs/personas/mailroom-hae-won-kim.md`

## Appendix M - Evidence Ledger and Reproduction Commands

- Reproduction command 1: `wc -l docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md`
- Reproduction command 2: `rg -n 'Thesis clause|Problem clause' docs/architecture/unified-ecosystem-thesis-2026-05-21.md docs/architecture/training-cost-doctrine-2026-05-21.md`
- Reproduction command 3: `rg -n '^### Section D-' docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- Reproduction command 4: `find microservices -mindepth 1 -maxdepth 1 -type d | sort`
- Reproduction command 5: `find registry/capability-tiers -type f | sort`
- Reproduction command 6: `rg -n 'provider-BYOK|encryption-BYOK|BYOK' docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md docs/runbooks/byok-rotation-provider-tenant-duress.md docs/runbooks/byok-rotation-encryption-tenant-duress.md`
- Reproduction command 7: `rg -n 'openapi:|asyncapi:|syntax = "proto' microservices docs/user-journeys specs`

## Appendix N - Surface Presence Ledger

### Surface ledger - analytics
- Artifacts: 137; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `analytics/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/analytics/decisions/ADR-AN-001-ttl-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/analytics/decisions/ADR-AN-002-partition-strategy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/analytics/decisions/ADR-AN-003-row-level-tenant-isolation.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/analytics/decisions/ADR-AN-004-query-budget-tier.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/analytics/decisions/ADR-AN-005-materialized-view-cadence.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - api-gateway
- Artifacts: 134; severity=P1 NEEDS-FIX; confidence=HIGH.
- `api-gateway/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `api-gateway/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - application
- Artifacts: 126; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `application/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - audit-chain
- Artifacts: 196; severity=P1 NEEDS-FIX; confidence=HIGH.
- `audit-chain/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `audit-chain/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - calendar
- Artifacts: 127; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `calendar/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/ADR-CAL-0001-caldav-server-backend-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/ADR-CAL-0003-jmap-vs-caldav-frontend-priority.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/ADR-CAL-0004-tzdb-refresh-and-pinning-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/ADR-CAL-001-icalendar-rfc5545-rfc7986-freebusy-acl.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/calendar/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - cell
- Artifacts: 139; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cell/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cell/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - cloud-iac
- Artifacts: 166; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-iac/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - cloud-k8s
- Artifacts: 121; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-k8s/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `cloud-k8s/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-k8s/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/cloud-k8s/decisions/ADR-CK-001-cilium-cni-selection.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - cloud-secrets
- Artifacts: 125; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `cloud-secrets/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - comms-email
- Artifacts: 135; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `comms-email/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/comms-email/decisions/SVC-ADR-001-dkim-cadence.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/comms-email/decisions/SVC-ADR-002-suppression-list-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/comms-email/decisions/SVC-ADR-003-webhook-retry-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/comms-email/decisions/SVC-ADR-004-tenant-domain-onboard-flow.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/comms-email/decisions/SVC-ADR-005-mjml-liquid-canonical.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - community
- Artifacts: 199; severity=P2 IMPROVE; confidence=HIGH.
- `community/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `community/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/ADR-COMM-0002-voting-engine-tie-breaking-and-decay.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/ADR-COMM-0003-kb-article-versioning-and-fork-merge.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/ADR-COMM-0005-graph-of-discussions-and-replies.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/community/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - compliance
- Artifacts: 186; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `compliance/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-compliance-001-evidence-retention-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-compliance-002-dsar-sla.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-compliance-003-auditor-access-cedar-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-compliance-004-cross-tenant-kernel-invariant.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/compliance/decisions/ADR-compliance-005-replace-drata-vanta-with-in-house.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - connect
- Artifacts: 183; severity=P1 NEEDS-FIX; confidence=HIGH.
- `connect/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `connect/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - consent-graph
- Artifacts: 135; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `consent-graph/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/consent-graph/decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/consent-graph/decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/consent-graph/decisions/ADR-SVC-CG-003-three-sharing-modes.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/consent-graph/decisions/ADR-SVC-CG-004-grantor-region-topic-ownership.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/consent-graph/decisions/ADR-SVC-CG-005-self-revocation-b2c.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - contact-center
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contact-center/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - contract-lifecycle-management
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `contract-lifecycle-management/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - crm
- Artifacts: 137; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `crm/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - data-pipeline
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-pipeline/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - data-warehouse
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `data-warehouse/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - design-collaboration
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `design-collaboration/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - detection
- Artifacts: 129; severity=P2 IMPROVE; confidence=HIGH.
- `detection/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `detection/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/detection/decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - developer-sdk
- Artifacts: 129; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `developer-sdk/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0001-ed25519-signing-keys-via-openbao-transit-engine-only;-privat.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0004-payout-substrate-uses-iso-20022-pain.001-for-sepa-and-nacha-.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0005-tax-form-emission-triggered-at-year-end-regenerated-on-deman.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0006-kyc-pipeline-in-house;-no-external-kyc-saas-(onfido-persona-.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/developer-sdk/decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - docs
- Artifacts: 120; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `docs/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0001-crdt-library-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0002-block-type-system.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0003-export-pipeline-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0004-acl-granularity-per-block.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0005-ai-writing-assist-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/ADR-DOCS-0006-import-fidelity-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/docs/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - drive
- Artifacts: 159; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `drive/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0003-share-link-security-model.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0004-encryption-at-rest-and-e2e.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-0006-immutability-and-worm-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/drive/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - feature-flags
- Artifacts: 131; severity=P1 NEEDS-FIX; confidence=HIGH.
- `feature-flags/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `feature-flags/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - financial-planning
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `financial-planning/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - finops-portal
- Artifacts: 157; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `finops-portal/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-001-focus-spec-version.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-002-cost-attribution-label-strategy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-003-tenant-billing-export-cadence.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-004-credit-ledger-append-only.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-005-grafana-iframe-embed.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-006-cedar-residency-double-guard.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/finops-portal/decisions/ADR-finops-portal-007-ed25519-quarterly-key.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - forms
- Artifacts: 144; severity=P2 IMPROVE; confidence=HIGH.
- `forms/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `forms/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0001-form-definition-schema.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0002-captcha-and-anti-spam.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0003-pii-column-encryption-and-residency.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0004-conditional-logic-and-branching-engine.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/forms/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - foundry
- Artifacts: 576; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `foundry/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/foundry/decisions/SVC-ADR-WASM-001-wasmtime-canonical-foundry.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - global-trade
- Artifacts: 137; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `global-trade/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - governance
- Artifacts: 195; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `governance/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/governance/decisions/ADR-GOV-001-audit-event-aggregation-pack-retention.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/governance/decisions/SVC-ADR-WASM-001-envoy-wasm-canonical-governance.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - healthcare-integration
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `healthcare-integration/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - identity
- Artifacts: 223; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `identity/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-identity-001-jwks-rotation-cadence.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-identity-002-passkey-attestation-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-identity-003-scim-rate-limits.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-identity-004-session-class-tiers.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/identity/decisions/ADR-identity-005-jit-it-approval-protocol.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - incident-management
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `incident-management/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - intelligence
- Artifacts: 165; severity=P1 NEEDS-FIX; confidence=HIGH.
- `intelligence/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `intelligence/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - itsm
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `itsm/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - learning-management
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `learning-management/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - mail
- Artifacts: 195; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `mail/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/ADR-MAIL-0001-personal-mail-key-recovery.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/ADR-MAIL-0003-sdk-launch-order.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/mail/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - marketing-automation
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketing-automation/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - marketplace
- Artifacts: 123; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `marketplace/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - meet
- Artifacts: 138; severity=P2 IMPROVE; confidence=HIGH.
- `meet/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `meet/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0001-sfu-substrate-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0002-recording-and-transcription-pipeline.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0003-e2e-encryption-for-meetings.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0004-live-streaming-egress-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0005-large-audience-and-webinar-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/ADR-MEET-0006-ai-feature-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/meet/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - messenger
- Artifacts: 156; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `messenger/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/ADR-MSGR-0001-huddles-placement.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/ADR-MSGR-0002-e2e-personal-dm-key-escrow.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/ADR-MSGR-0003-search-backend-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/ADR-MSGR-0004-federation-posture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/messenger/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - network
- Artifacts: 125; severity=P2 IMPROVE; confidence=HIGH.
- `network/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `network/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0001-professional-graph-storage.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0003-inmail-bridge-to-messenger.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0004-jobs-handoff-to-ats.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0005-endorsement-chain-integrity.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/ADR-NET-0006-profile-portability-and-export.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/network/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - notes
- Artifacts: 159; severity=P2 IMPROVE; confidence=HIGH.
- `notes/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `notes/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0001-e2e-encryption-default-personal-tier.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0002-bidirectional-link-and-graph-storage.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0003-crdt-library-for-optional-collab.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0004-search-architecture-respecting-e2e.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/ADR-NOTES-0006-portable-export-and-import-format.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/notes/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - observability
- Artifacts: 198; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `observability/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - ontology
- Artifacts: 150; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `ontology/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/ontology/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - ops-dashboard-control-center
- Artifacts: 147; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `ops-dashboard-control-center/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - payments
- Artifacts: 190; severity=P1 NEEDS-FIX; confidence=HIGH.
- `payments/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `payments/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - performance-management
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `performance-management/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - plant-maintenance
- Artifacts: 139; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plant-maintenance/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - plugin-app-store
- Artifacts: 140; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `plugin-app-store/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0001-per-plugin-cedar-policy-materialization-at-install-time-not-.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0002-vetting-pipeline-ordered-stage-execution-never-parallelized.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0003-wasmtime-engine-per-tenant-plugin-installation-not-per-plugi.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0004-vetting-badge-tiers-(bronze/silver/gold/platinum)-determined.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0005-per-installation-rate-limit-default-100-req/s;-per-plugin-ov.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0006-subscription-billing-aggregator-runs-nightly-not-real-time.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/plugin-app-store/decisions/ADR-PAS-0007-per-plugin-action-audit-trail-seals-via-audit-chain-µservice.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - production-planning
- Artifacts: 139; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `production-planning/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - quality-management
- Artifacts: 139; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `quality-management/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - real-estate
- Artifacts: 129; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `real-estate/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - recordings
- Artifacts: 127; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `recordings/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0001-transcription-and-diarization-pipeline.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-and-legal-hold-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0003-redaction-and-pii-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0004-playback-and-cdn-strategy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0005-storage-substrate-tiered.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0006-ai-feature-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/ADR-RECORDINGS-0007-multi-source-ingest-contract.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/recordings/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - sheets
- Artifacts: 125; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sheets/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0001-crdt-library-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0002-formula-engine-conformance-target.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0003-large-sheet-storage-substrate.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0004-recalc-engine-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0005-ai-formula-and-smart-fill-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0006-per-range-acl-granularity.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/ADR-SHEETS-0007-export-fidelity-policy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sheets/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - shorts
- Artifacts: 122; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `shorts/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0001-video-transcode-pipeline.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0002-copyright-claim-system.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0003-content-moderation-classifier-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0004-drm-substrate-tenant-tier.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0005-feed-ranking-algorithm.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/ADR-SHORTS-0006-minor-protection-and-age-gate.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/shorts/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - sites
- Artifacts: 123; severity=P2 IMPROVE; confidence=HIGH.
- `sites/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `sites/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0001-crdt-library-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0003-cdn-substrate-and-cache-strategy.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0004-acme-and-custom-domain-flow.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0005-cms-collection-data-model.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0006-ai-page-build-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/ADR-SITES-0007-image-and-asset-pipeline.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/sites/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - slides
- Artifacts: 128; severity=P2 IMPROVE; confidence=HIGH.
- `slides/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `slides/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0001-crdt-library-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0002-rendering-canvas-substrate.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0003-export-pipeline-fidelity.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0006-ai-design-and-content-generation-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0007-per-slide-acl-granularity.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/ADR-SLIDES-0008-chart-live-link-to-sheets.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/slides/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - social
- Artifacts: 144; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `social/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0001-feed-ranking-algorithm.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0002-follow-graph-storage.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0004-federation-posture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0005-dual-context-feed-isolation.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/social/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - supply-chain-planning
- Artifacts: 137; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `supply-chain-planning/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - tasks
- Artifacts: 122; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `tasks/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0001-task-data-model-and-custom-fields.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0002-dependency-graph-and-cycle-prevention.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0003-recurring-task-engine.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0004-view-engine-and-board-realtime.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0005-automation-engine-cross-microservice.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/ADR-TASKS-0006-ai-auto-assign-and-eu-ai-act-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tasks/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - tenancy
- Artifacts: 181; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `tenancy/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/tenancy/decisions/ADR-TEN-001-tenant-lifecycle-parent-child-cedar-permit.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - translate
- Artifacts: 120; severity=P1 NEEDS-FIX; confidence=HIGH.
- `translate/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `translate/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `translate/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0001-mt-engine-routing-and-fallback.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0002-translation-memory-and-leverage-model.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0004-data-residency-bound-inference.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0005-document-round-trip-fidelity.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/ADR-TRANSLATE-0006-real-time-translation-stream-architecture.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/translate/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - treasury
- Artifacts: 139; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `treasury/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - warehouse
- Artifacts: 129; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `warehouse/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - whiteboard
- Artifacts: 152; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `whiteboard/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - workflow-engine
- Artifacts: 222; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workflow-engine/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-engine/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

### Surface ledger - workflow-studio
- Artifacts: 221; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/capability-tiers`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/onboarding`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/faqs`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/tutorials`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/benchmarks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/migration-playbooks`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/reference-implementations`: present; severity=P2 IMPROVE; confidence=HIGH.
- `workflow-studio/decisions`: present; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/ADR-WS-0002-dsl-canonical-form.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/ADR-WS-0003-leptos-wasm-substrate.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/ADR-WS-0004-jurisdiction-overlay-renderer.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md`; severity=P2 IMPROVE; confidence=HIGH.
- authored decision file: `microservices/workflow-studio/decisions/README.md`; severity=P2 IMPROVE; confidence=HIGH.

### Surface ledger - workplace-integration
- Artifacts: 124; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/capability-tiers`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/onboarding`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/faqs`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/tutorials`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/benchmarks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/migration-playbooks`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/reference-implementations`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- `workplace-integration/decisions`: missing; severity=P1 NEEDS-FIX; confidence=HIGH.
- authored decision file: none; severity=P1 NEEDS-FIX; confidence=HIGH.

## Final Verdict

**REVISE-CORPUS-WIDE-AGAIN.**

The remediation waves materially improved contract conformance, long-form doctrine, artifact counts, manifest audit fields, and reverse cross-reference density. The corpus remains below the requested rigor bar because dossier completion, IP substance, per-service ADR authorship, journey continuation, persona marker coverage, and downstream BYOK disambiguation are still incomplete.
