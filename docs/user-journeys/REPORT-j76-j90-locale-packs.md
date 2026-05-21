# j76-j90 Locale-Pack Overlay Delivery Report

| Journey | Pack | Services | Regulator pattern |
|---|---|---:|---|
| j76 | EU-GDPR-2018-baseline | 11 | Berlin DPA |
| j77 | EU-AI-ACT-HIGH-RISK | 10 | EU AI Office + national market surveillance authority |
| j78 | EU-NIS2 | 11 | EU CSIRT + competent authority |
| j79 | EU-DSA | 11 | Digital Services Coordinator |
| j80 | KR-PIPA + KR-CSAP | 12 | PIPC + KISA |
| j81 | KR-CSAP-v3.1 | 11 | KISA |
| j82 | KR-FSS | 11 | FSS + KoFIU |
| j83 | CN-PIPL-2021 | 12 | CAC + MIIT |
| j84 | JP-APPI | 10 | Japan PPC |
| j85 | HIPAA-2024 | 13 | HHS OCR |
| j86 | PCI-DSS-L1-v4 | 12 | PCI SSC + QSA |
| j87 | FedRAMP-High + DoD-IL5/IL6 | 13 | FedRAMP PMO + DoD AO |
| j88 | AU-IRAP-PROTECTED | 12 | ASD IRAP assessor + OAIC + APRA when applicable |
| j89 | UK-AADC + UK-Online-Safety-Act | 12 | ICO + Ofcom |
| j90 | US-CCPA-CPRA-2023 | 13 | California Privacy Protection Agency |

Line count: 117730 generated lines across 15 journey directories, schemas, per-service IP slices, and this report.
Conflict-resolution patterns: higher-restriction pack wins; residency hard-stop beats convenience; regulator deadline survives degraded mode; appeal remains available after denial; audit-chain is append-only even during rollback.
Activation cascade patterns: identity resolves principal context, tenancy pins pack and cell, consent-graph verifies purpose, workflow-engine fans out, each service applies Cedar, audit-chain seals, compliance assembles regulator evidence, and user notices localize without changing legal semantics.
Protected sources were not edited: ADRs, standards, existing PRDs, and existing ARCHITECTURE.md files remain untouched.
