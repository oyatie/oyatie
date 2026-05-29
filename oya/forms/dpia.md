---
doc_class: DPIA
microservice: forms
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-forms + ops-security
reviewers: council-legal-compliance, ops-sre-reliability
review_cadence: annually + on every (new pack | new field-type | LLM-provider change | breach)
doc_status: published
gdpr_art_35_triggered: true
ai_act_art_27_triggered: true
---

# Forms — Data Protection Impact Assessment (GDPR Art. 35)

This DPIA is mandatory because Forms processes:
1. Special-category data (GDPR Art. 9) when tenants author health / political / sexual-orientation fields.
2. Large-scale processing of identifying data (any tenant collecting > 10k responses).
3. AI-form-build (T2 capability) is potentially Annex III §4 high-risk under EU AI Act 2024/1689.
4. Cross-border transfers when tenants operate forms across packs.
5. Systematic monitoring (analytics + funnel) of submitter behaviour.

## §1 Description of Processing

### Nature

Tenant authors a form via the Leptos-WASM builder; submitter completes via embedded iframe / hosted link / pre-filled link; response captured + PII-encrypted at rest; downstream actions (workflow, export, webhook, bulk-distribute) triggered.

### Scope

| Dimension | Detail |
|---|---|
| Subjects | submitter (individual completing a form); per-tenant scope |
| Data classes | PII_IDENTIFYING; PII_QUASI_IDENTIFIER; SENSITIVE_GDPR_ART9; PHI (pack-us-healthcare); BEHAVIORAL_TENANT_PRODUCT; FINANCIAL (when payment); AUDIT; SECRET (signing keys) |
| Geographic | 11 packs per `policy/data-residency.md` |
| Volume | Forecast: 100M responses / year at GA |
| Duration | Per-pack retention table (e.g., pack-eu: bounded by purpose; pack-us-hc: 6y per HIPAA) |
| Frequency | Continuous |
| Categories of recipients | Tenant + workflow-engine + sheets + mail + messenger + drive + audit-chain + ontology |
| International transfers | Cross-border only with SCC; LLM-assist routed to pack-resident provider |
| Retention | Per `policy/data-residency.md` |

### Context

Tenants range from individual SaaS users to enterprise (HR, healthcare, finance). Submitters include employees, customers, patients, citizens. Power imbalance is highest for employment-screening and patient-intake forms.

## §2 Necessity & Proportionality

| Principle | Assessment |
|---|---|
| Lawful basis (Art. 6) | Contract / consent / legitimate-interest depending on tenant; explicit consent required for Art. 9 fields (Art. 9(2)(a)) |
| Purpose specification | Each form declares purpose at authoring; published-form spec includes purpose statement |
| Data minimisation | AI-form-build emits warnings on non-essential PII; builder blocks publish on missing data-class declaration |
| Accuracy | Submitter rectification path always-on |
| Storage limitation | Auto-purge at retention TTL; no silent extension |
| Transparency | Submitter-facing notice rendered above form; lists controllers + purposes + recipients + transfers + retention + rights |

## §3 Risks to Subjects

| ID | Risk | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-tenant PII leak via Citus RLS bypass | Low | High | M |
| R-02 | Submitter IP correlatable across forms via session/cookie | Med | Med | M |
| R-03 | AI-form-build emits form that captures more PII than needed | Med | Med | M |
| R-04 | Cross-border transfer of EU PII to non-adequate jurisdiction | Low | High | M |
| R-05 | Special-category field captured without explicit Art. 9 consent | Med | High | H |
| R-06 | Submitter unaware AI processed their response (analytics, clustering) | Med | Med | M |
| R-07 | PHI form opened by non-BAA tenant | Low | High | M |
| R-08 | Webhook delivery to insecure tenant endpoint | Med | Med | M |
| R-09 | DSR-erased response remains in derivative aggregate | Low | Med | L |
| R-10 | Embed iframe XSS leaks submitter session | Low | High | M |
| R-11 | Captcha provider sees submitter IP + cookie cross-site | Med | Low | L (mitigated by hCaptcha privacy posture) |
| R-12 | Bulk-distribute email exposes recipient list via Reply-All | Low | Med | L |
| R-13 | E-signature non-repudiation challenged because tier mismatch | Low | High | M |
| R-14 | LLM (AI-form-build) provider stores prompt + completion beyond zero-retention | Low | High | M |
| R-15 | Submitter wears employment/credit lens unknowingly (Annex III §4) | Med | High | H |

## §4 Mitigations

| Risk | Mitigation |
|---|---|
| R-01 | Citus RLS + Cedar default-deny + per-tenant DEK + cross-tenant adversarial test corpus (AC-28); chaos quarterly |
| R-02 | submitter_hash per-form-salted; IP HMAC'd (not raw); cookieless mode opt-in per pack |
| R-03 | Data-minimisation lint at publish; AI-form-build emits warnings; builder blocks `data_class=NORMAL` on inferred-PII fields |
| R-04 | SCC mandatory per `policy/data-residency.md`; pack-pinning default |
| R-05 | Builder requires `consent_notice_art9_text` on any `data_class=SENSITIVE_GDPR_ART9` field; submitter explicit-consent checkbox (non-pre-ticked) per Art. 7 |
| R-06 | AI-Act Art. 50 transparency: form metadata banner when AI-authored; analytics-aware consent |
| R-07 | pack-us-healthcare requires `tenant.healthcare.baa_signed=true`; Cedar gate; license-gate-cedar enforces |
| R-08 | Tenant must declare webhook target with mTLS; non-mTLS rejected at config time |
| R-09 | DSR runner cascades to derivative aggregates (analytics roll-ups); ledger of erasure |
| R-10 | CSP strict; Trusted Types; Renderer escapes all tenant-authored strings |
| R-11 | hCaptcha (default for pack-eu, pack-kr, pack-us-hc) per ADR-FORMS-0002; never reCAPTCHA in those packs |
| R-12 | Bulk-distribute personalises per-recipient; To: contains only the recipient; BCC: forbidden in compliance mode |
| R-13 | E-signature tier validation: tenant requesting QES-level form must have entitlement; otherwise downgrade rejected |
| R-14 | BYO-LLM with zero-retention contract attestation; provider compliance reviewed per `legal/sub-processors.md` |
| R-15 | ADR-FORMS-0005 high-risk classification: builder asks "is this form used for employment / credit / insurance screening?" — if yes, mandatory DPIA + Annex III §4 conformity |

## §5 Residual Risk

After mitigations, residual risk is **Low** for R-01..R-14 and **Low-Medium** for R-15 (mitigation is procedural, depends on tenant honesty in the classification question; ADR-FORMS-0005 §"Risk register" tracks this).

## §6 Consultation

- council-privacy reviewed: 2026-05-17.
- council-legal-compliance reviewed: 2026-05-17.
- ops-security reviewed: 2026-05-17.
- External DPO consultation: scheduled before GA cutover.
- DPA-class consultation (for high-risk-classified tenants): per-tenant at onboarding.

## §7 Decision

Processing is **proportionate** subject to:
1. Per-pack residency mandatory.
2. Per-tenant DEK envelope encryption at rest.
3. AI-form-build T2 gated per ADR-FORMS-0005.
4. Audit-chain seal on every response.
5. DSR cascade SLA per pack.
6. Annual DPIA review + on any trigger event.

## §8 Action Register

| Action | Owner | Due | Status |
|---|---|---|---|
| Per-tenant DPA template signed for every new tenant | council-legal-compliance | onboarding | per-tenant |
| Annex III §4 classification answer captured for every form | builder lint | publish-time | enforced |
| Quarterly chaos drill for R-01 | ops-sre-reliability | every quarter | scheduled |
| Sub-processor list updated per LLM provider change | council-privacy | event-driven | tracked |
| External DPO sign-off | council-privacy | before GA | pending |

## References

- GDPR Art. 35.
- EU AI Act Art. 27 (deployer FRIA).
- KR PIPA Art. 33-2 (PIA equivalent).
- HIPAA 45 CFR §164.308(a)(1) risk analysis.
- `compliance.md`, `threat-model.md`, `policy/data-residency.md`.
- ADR-FORMS-0001..0006.
