---
doc_class: DPIA
template_id: TPL-DPIA
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-foundry
deciders: council-privacy, ops-security, council-architecture, axis-foundry
methodology: ICO + CNIL DPIA methodology + GDPR Art. 35
related_adrs: [ADR-0028, ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/threat-model.md
  - microservices/governance/policy/data-residency.md
  - microservices/governance/compliance.md
review_cadence: annual + on every new data-class introduction + on every new pack onboarding
doc_status: published
---

# DPIA: governance µservice

## 1. Description of Processing

### Nature

The governance µservice processes per-PR metadata, lane execution telemetry, and Findings emitted by ~50 fitness lanes. Processing happens at PR-time (lane execution) and at quarterly cadence (industry-baseline refresh + audit replay). The µservice is the CI-fitness substrate for every oyatie µservice; without it no PR advances to `dev`.

### Scope

| Category | Detail |
|---|---|
| Data subjects | (a) Internal engineers + agents authoring PRs; (b) Tenant operators when they open PRs against their own µservice; (c) External auditors during JIT audit windows. Approx 50–200 distinct subjects at M01 launch tier; forecast 5000+ at L scale tier. |
| Data categories | PR-author identity (OIDC subject, email, name); commit metadata (sha, signature, message); lane verdicts; Finding records; evidence blobs (lane output transcripts; occasionally containing user-attributable data when lanes run over tenant artifacts) |
| Special categories | Generally none. Lane evidence may incidentally include PII (e.g., test fixture with email) — `oya-check-data-class` lane refuses real-PII fixtures; sanitiser pass per threat-model §T-I-03 |
| Geographic scope | KR (M01 launch; pack-kr); 11 packs at L tier |
| Volumes (M01 → L tier) | 1k PRs/month (M01) → 100k PRs/month (L); 50k Findings/month → 5M; 1 TB evidence/year → 100 TB |
| Retention | Findings + evidence: 7y (SOC 2 + ISO 27001 minimum); PR-author identity in non-AUDIT views: 2y; AUDIT view: 7y |

### Context

Processing is for **legitimate-interest** purpose (Art. 6(1)(f) GDPR / equivalent under each pack's law): operating a CI substrate for software-quality enforcement that protects the controller's customers, employees, and contractual obligations. The data subjects (internal engineers, agents, tenant PR authors) have a contractual relationship with the controller; processing is necessary and proportionate to the controller's legitimate interest in software safety and contractual SLA compliance.

### Purpose

| Primary purpose | Legal basis (GDPR Art. 6) | Pack-specific basis |
|---|---|---|
| Software-quality enforcement at PR-time | 6(1)(f) Legitimate interest | KR PIPA Art. 15(1)(2) Legitimate purpose; HIPAA §164.308 Administrative Safeguards; pack-eu GDPR Art. 6(1)(f); pack-us NIST CSF (legitimate operational purpose); pack-jp APPI Art. 17; pack-sg PDPA §13 Reasonable; pack-au Privacy Act APP 3.2; pack-in DPDPA §6 Legitimate use; pack-br LGPD Art. 7(IX); pack-ae UAE PDPL Art. 5(1)(g); pack-ksa PDPL Art. 4(4) |
| Audit-replayable evidence for external auditors (SOC 2 + ISO 27001 + SLSA) | 6(1)(c) Legal obligation (where audit is contractual or regulatory) + 6(1)(f) Legitimate interest | each pack: regulatory audit mandates (KR-ISMS-P; HIPAA §164.316; APRA-CPS 234; SAMA-CSF; etc.) |
| Industry-baseline drift detection (quarterly refresh) | 6(1)(f) Legitimate interest | as above |
| Aggregation-index generation | 6(1)(f) Legitimate interest | as above |

## 2. Necessity + Proportionality

### Necessity

Lane execution + Finding emission requires processing PR-author identity and commit metadata to: (a) attribute violations to author for remediation; (b) emit auditable audit-chain records per SLSA L3 source-provenance + SOC 2 CC6.3 + ISO 27001 A.5.16 non-repudiation requirements; (c) refuse spoofed PRs.

Without identity attribution: violations cannot be attributed to remediation owner; SLSA Source L3 cannot be claimed; audit-chain non-repudiation breaks. The minimum dataset is PR-author OIDC subject + commit SHA + signature + lane verdict. Email/name are retained because (a) GitHub already exposes them in the commit metadata, (b) audit reports require human-readable attribution.

### Proportionality

| Data | Necessity | Proportionality test |
|---|---|---|
| PR-author OIDC subject | Required for non-repudiation | Minimum identifier; cannot be hashed away because audit-chain seal must be replayable against GitHub's API |
| PR-author email + name | Required for human-readable audit reports | Recorded; access scoped to ops-security in AUDIT view; redacted in non-AUDIT views by default; quarterly review of access logs |
| Commit signature | Required for SLSA Source L3 | Stored; cryptographically necessary |
| Lane verdict | Core processing output | Necessary |
| Finding evidence (lane transcripts) | Required for replay + remediation | Sanitiser pass on write; per-pack retention overlay |
| External-auditor JIT identity | Required for audit window | Short-lived OIDC; ≤1h TTL; not stored after audit window closes |

Less-intrusive alternatives considered + rejected:
- **Anonymous PRs**: rejected — breaks SLSA Source L3, breaks remediation attribution, breaks audit non-repudiation.
- **Hash-only author identity**: rejected — auditor reports unreadable; controller cannot prove identity binding at audit time.
- **Findings without evidence**: rejected — breaks reproducibility, breaks SOC 2 CC7.4 evidence requirement.

## 3. Rights of Data Subjects

### Right to information (Arts. 13, 14)

- `docs/AGENTS.md` discloses governance lane execution and Finding emission to all agents authoring PRs.
- `CLAUDE.md` discloses to human contributors.
- PR template includes a footer notice (per `microservices/governance/specs/pr-template-consent-footer.json`) summarising data collection.
- Tenant operators receive notice via Application Shell during PR-author setup (per `microservices/application/PRD.md` Slice C onboarding).
- External auditors receive notice via the JIT-scope acceptance flow.

### Right of access (Art. 15)

- Internal engineers / agents: query via the governance evidence-query control-plane operation for `author=<subject>`.
- Tenant operators: read via Application Shell self-service.
- External auditors: read via JIT-scoped API.
- ETA: ≤1 working day; pack-eu / pack-kr 30-day statutory ceiling.

### Right to rectification (Art. 16)

- Findings record OBSERVED facts (lane verdicts); rectification of OBSERVED facts is generally not applicable.
- PR-author identity rectification: handled at GitHub level (vendor-managed); governance picks up corrected identity on next PR.

### Right to erasure (Art. 17)

- Generally **refused under Art. 17(3)(b) (legal obligation; audit-replayability)** and Art. 17(3)(e) (legal claims defence).
- Pack-eu: erasure available for non-AUDIT fields (e.g., display name in non-AUDIT views) after audit-window closes; AUDIT view retained per SOC 2 + ISO 27001 retention minimums.
- Pack-us-healthcare: HIPAA §164.530 requires retention regardless; PHI scrub at retention end.
- Pack-kr: KR PIPA Art. 21 retention end → cryptographic erasure via KMS key-destroy.

### Right to restrict processing (Art. 18)

- Available for non-AUDIT views.
- AUDIT view processing is necessary for legal-obligation purpose; restriction refused per Art. 18(2).

### Right to data portability (Art. 20)

- Internal engineers / agents: receive Findings + evidence as JSON via the governance evidence-export control-plane operation for `author=<subject>`.
- Format: canonical-JSON + signed bundle.
- ETA: ≤1 working day.

### Right to object (Art. 21)

- Available for legitimate-interest processing; controller balances per Art. 21(1).
- Outcome: typically refused because contractual + regulatory necessity is high. Documented per-objection.

### Right not to be subject to automated decision-making (Art. 22)

- **Applicability**: Governance lanes make automated decisions (PR admission / refusal) with **legal or similarly significant effects** on the data subject (PR author).
- **Mitigation**: Decisions are based on **PR content (code), not on data-subject characteristics**. Lane outcomes are deterministic and explained (every Finding cites the rule + the offending line/file).
- **Human-in-loop fallback**: Break-glass procedure per `runbooks/lane-bypass-emergency.md` provides ops-security override on emergency basis.
- **Right to contest**: PR author can request rule-pack review via PR comment; review by axis-foundry within 5 working days.

## 4. Risks to Data Subjects

Per LINDDUN section of `threat-model.md`; this section summarises only the high-risk items + adds residual posture.

| Risk | Source | Likelihood | Impact | Residual | Action |
|---|---|---|---|---|---|
| Cross-tenant Finding read | T-I-01 | M | H | L | Cedar tenant-scope + Postgres RLS + S3 IAM scoping. |
| PII leakage in evidence transcripts | P-I-01 / T-I-03 | M | M | M | Sanitiser pass + `oya-check-data-class` lane + monthly false-negative review. |
| Long-term retention profiling | P-D-01 | L | M | L | 7y bounded; profiling refused at policy level. |
| External-auditor JIT scope leak | P-D-02 | L | H | M | JIT scope bound to single window; tenant-scope claim enforced. |
| PR-author unawareness | P-U-01 | M | M | M | docs/AGENTS.md + CLAUDE.md + PR template footer. |
| Per-pack retention violation | P-NC-01 | M | H | M | Per-pack kustomize overlay tested at deploy time; quarterly compliance review. |
| Automated-decision (Art. 22) without notice | new | L | M | L | Notice in docs + PR template; contest procedure available. |

## 5. Pack-specific Privacy Posture

| Pack | Regulator | Applicable instrument | Posture |
|---|---|---|---|
| pack-kr | KCC + PIPC | KR PIPA Arts. 15, 17, 18, 21, 23, 24, 25, 28, 29 | Legitimate-purpose basis; in-KR processing; per-pack KMS in KR; retention per KR commercial-code 5y; sensitive-info (Art. 23) handling for RRN-class data refused at sanitiser stage |
| pack-us-healthcare | OCR | HIPAA §164.308, 310, 312, 314, 316; HITECH | BAA required with controller; PHI scrub on evidence; retention 6y; admin/physical/technical safeguards documented in `compliance.md` |
| pack-us | (sector-specific) | Various state laws (CCPA/CPRA, VCDPA, etc.) | Notice + access + deletion rights via Application Shell; sale of personal info: refused at policy level |
| pack-eu | EU DPA + national DPAs | GDPR Arts. 5, 6, 13, 14, 17, 22, 25, 30, 32, 33, 35; EDPB guidelines | This DPIA; ROPA in `compliance.md`; DPO consultation triggered for new packs |
| pack-jp | PPC | APPI Arts. 17/18/20/21/23/24/26-2 | Purpose-of-use notice; consent for cross-border transfer to non-equivalent jurisdictions; APPI Art. 26-2 for sensitive personal info |
| pack-sg | PDPC | PDPA §11-26 + §13 Consent | Reasonable purpose; notice; access rights; transfer-limitation per §26 |
| pack-au | OAIC | Privacy Act 1988 APP 1-13 (esp. APP 6, 8, 11) | Open + transparent management; cross-border disclosure protections per APP 8; security per APP 11 |
| pack-in | (forthcoming Board) | DPDPA 2023 §6-10 | Notice + consent (where applicable) + legitimate use; data fiduciary obligations; data principal rights |
| pack-br | ANPD | LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48 | Legitimate-interest basis; access + portability + deletion (within audit-retention bounds) |
| pack-ae | UAE Federal | UAE PDPL Federal Decree-Law No. 45/2021 | Notice + lawful-basis; cross-border per Art. 22 |
| pack-ksa | SDAIA | PDPL Royal Decree M/19/2021 | Lawful-basis; data-subject rights per Arts. 4-9 |

## 6. Cross-Border Transfers

| Transfer | Mechanism | Documentation |
|---|---|---|
| pack-kr → pack-eu (auditor read) | SCCs (EU Commission 2021/914) + supplementary measures (encryption-in-transit + encryption-at-rest) | per audit window; logged |
| pack-eu → pack-us (auditor read; pre-DPF) | EU-US Data Privacy Framework participant; or SCCs as fallback | per audit window |
| pack-eu → pack-jp | Adequacy decision (EU-Japan 2019/419) | no additional mechanism |
| pack-eu → pack-kr | Adequacy decision (EU-Korea 2021/2071) | no additional mechanism |
| pack-eu → pack-au / pack-sg / pack-in / pack-br / pack-ae / pack-ksa | SCCs + transfer-impact-assessment | TIA in `compliance.md` annex |
| pack-us-healthcare → other packs | refused by default (HIPAA-bound; no transfer absent BAA chain) | bucket-prefix lock per `iac/kustomize/overlays/pack-us-healthcare/` |

## 7. Data Subject Engagement

Pre-deployment consultation:
- Internal engineers + agents: consulted via `docs/AGENTS.md` review (open PR thread for comment; closed by 2026-05-31).
- Tenant operators: pre-onboarding notice + acknowledgement at first PR-author setup; per-tenant DPO consultation for regulated tenants.
- External auditors: notice + scope-acceptance flow at JIT issuance.

Post-deployment engagement:
- Quarterly transparency report at `evidence/audits/dpia-transparency/<quarter>.md`.
- Annual data-subject feedback survey via Application Shell.

## 8. Consultation with DPO + Supervisory Authorities

- DPO consultation: 2026-05-17 (this DPIA); next: annual + on new-pack onboarding.
- Supervisory authorities: prior-consultation per Art. 36 not required (residual risk Medium, not High; documented per-residual in `threat-model.md`).
- Pack-kr (KR PIPC): notification at first tenant onboarding per KR PIPA Art. 32.
- Pack-eu DPAs: ROPA notification per Art. 30 (in `compliance.md`).

## 9. Monitoring + Review

- Quarterly DPIA review by council-privacy.
- Annual external review (DPA / SOC 2 Type 2 examiner / ISO 27001 auditor as scheduled).
- Trigger for ad-hoc review: new data class added; new pack onboarded; new automated-decision lane; material privacy incident.

## 10. Sign-off

- council-privacy: 2026-05-17 (this version)
- ops-security: 2026-05-17
- council-architecture: 2026-05-17

## References

- GDPR Art. 35 + EDPB Guidelines on DPIA (WP248rev.01).
- ICO DPIA guidance — `ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/accountability-and-governance/data-protection-impact-assessments-dpias/`.
- CNIL DPIA methodology — `cnil.fr`.
- KR PIPA + PIPC guidance.
- HIPAA Privacy + Security Rules.
- `microservices/observability/dpia.md` (shape reference).
- `microservices/governance/threat-model.md` §LINDDUN.
- `microservices/governance/compliance.md` (ROPA + per-pack overlay).
