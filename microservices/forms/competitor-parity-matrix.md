---
doc_class: CompetitorParityMatrix
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + council-product
review_cadence: quarterly
doc_status: published
---

# Forms — Competitor Parity Matrix

oyatie Forms competes head-to-head with the industry leaders below. This matrix is the contract; any "missing" cell is a backlog item or an explicit non-goal.

## Competitors

1. **Google Forms** (Google Workspace)
2. **Microsoft Forms** (Microsoft 365)
3. **Typeform**
4. **Jotform**
5. **Tally**
6. **Airtable Forms**
7. **SurveyMonkey**
8. **Wufoo**
9. **Formstack**
10. **Survicate**
11. **Qualtrics XM**
12. **HubSpot Forms**
13. **Mailchimp Forms** (Intuit)
14. **Hotjar Surveys** (Contentsquare)

## Capability Matrix

| Capability | Google Forms | MS Forms | Typeform | Jotform | Tally | Airtable Forms | SurveyMonkey | Wufoo | Formstack | Survicate | Qualtrics | HubSpot Forms | Mailchimp Forms | Hotjar Surveys | **oyatie Forms** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Conditional logic | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | **✓ + DAG cycle detection** |
| File upload | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | partial | ✗ | **✓ + ClamAV/OPSWAT scan inline** |
| Signature field | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | **✓ + eIDAS QES per ADR-FORMS-0006** |
| Payment field | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | **✓ + fintech bridge, PCI-DSS scope-reduction** |
| E-mail bulk distribute | ✗ (manual) | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓ + 10k/blast, GDPR-aware unsubscribe** |
| Embed iframe | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓ + CSP frame-ancestors per-tenant** |
| Embed JS widget | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓ + Trusted Types + per-tenant CSP** |
| REST submission API | ✗ | partial | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | **✓ + OpenAPI 3.2.0** |
| Webhooks on submit | partial | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓ + mTLS + HMAC + DLQ** |
| A/B variants | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | **✓ + statistical-significance gate** |
| Multi-language i18n | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓ + 14 locales incl. RTL** |
| WCAG 2.2 AA | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | **✓ + CI-gated** |
| Anonymous link | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✓** |
| Authenticated link (OIDC) | partial (Google SSO) | ✓ (M365 SSO) | ✓ | ✓ | ✗ | partial | ✓ | partial | ✓ | partial | ✓ | partial | partial | ✗ | **✓ + OAuth 2.1 + tenant IdP** |
| Pre-filled link | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | **✓ + HMAC-SHA-256 + TTL** |
| Captcha (privacy-preserving) | reCAPTCHA only | reCAPTCHA only | hCaptcha opt | hCaptcha | Turnstile | – | reCAPTCHA | reCAPTCHA | reCAPTCHA | – | reCAPTCHA | reCAPTCHA | reCAPTCHA | – | **hCaptcha/Turnstile/Friendly Captcha; reCAPTCHA forbidden in pack-eu/kr/us-hc** |
| AI form build | partial | partial (Copilot) | ✓ | ✓ | partial | ✗ | ✓ | ✗ | partial | partial | ✓ | partial | partial | partial | **✓ + T0/T1/T2 tier per ADR-FORMS-0005** |
| Per-field encryption-at-rest | ✗ | partial | ✗ | ✗ | ✗ | ✗ | partial | ✗ | partial | ✗ | ✓ | partial | ✗ | ✗ | **✓ + per-tenant DEK envelope** |
| HIPAA mode | ✗ | partial (E5+BAA) | ✗ | ✓ (BAA) | ✗ | ✗ | partial (BAA-eligible) | ✗ | ✓ (BAA) | ✗ | ✓ (BAA) | ✗ | ✗ | ✗ | **✓ + pack-us-healthcare** |
| GDPR DSR honoured | partial | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | **✓ + per-pack SLA** |
| Multi-page form | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | partial | partial | **✓ + branching DAG** |
| Likert / scale field | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✓ | partial | partial | ✓ | **✓** |
| Grid / matrix field | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | **✓** |
| Templates marketplace | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | partial | ✓ | ✓ | partial | partial | **✓ + per-pack signed** |
| Real-time analytics | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | **✓ + drop-off funnel + conversion** |
| Export CSV / XLSX | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✓ | ✓ / ✗ | **✓ + ≤ 5s for 100k rows** |
| Live sheet bridge | ✓ Google | ✓ Excel | ✓ | ✓ | ✓ | ✓ | ✓ | partial | ✓ | partial | ✓ | partial | partial | ✗ | **✓ → sheets µservice** |
| Workflow trigger | partial | ✓ (Flow) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | partial | partial | **✓ → workflow-engine via Workflow + Ontology adapter** |
| Audit-chain | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | partial | ✓ (HIPAA) | ✗ | ✓ | ✗ | ✗ | ✗ | **✓ + Ed25519 per response** |
| Regional residency | partial | ✓ | partial | partial | ✗ | ✗ | partial | partial | ✓ | ✗ | ✓ | partial | partial | ✗ | **✓ + 11 packs** |
| Self-hostable | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | **✓ (tenant-cell + on-prem available Tier-G+)** |

## Where we lead

- WCAG 2.2 AA enforced (every competitor is "partial"; we are CI-gated).
- Audit-chain Ed25519 seal on every response.
- Per-tenant DEK envelope encryption (most competitors lack column-level encryption).
- E-signature eIDAS QES (only Jotform + Wufoo + Qualtrics offer signatures, none at QES tier).
- Captcha privacy posture (reCAPTCHA forbidden in pack-eu/kr/us-hc).
- Workflow + Ontology adapter pattern (every cross-product flow is explicit + auditable).
- 11-pack residency footprint (only Qualtrics + Formstack + MS Forms come close).
- Self-hostable Tier-G (no competitor offers this).

## Where we accept "match"

- Conditional logic, multi-language, multi-page, Likert/grid fields, templates, analytics export: these are table-stakes and we match them.

## Where we explicitly do NOT lead

- "Free-tier" pricing — oyatie Forms uses tier-2 pricing minimum; we don't compete with Google Forms on free-tier user count.
- "Quick-poll-from-Slack-in-30-seconds" — Tally + Mailchimp Forms have better single-question-quick-poll UX; we focus on serious-form authoring.

## Forbidden claims (per `feedback_no_silent_regression.md`)

- We do NOT claim "100% spam-free" — captcha + rate-limit reduce but cannot eliminate.
- We do NOT claim "auto-detect special-category data" — we surface a builder warning but tenant declaration is authoritative.
- We do NOT claim "zero PHI breach risk" — HIPAA mode reduces but tenant misconfiguration can still create exposure.

## References

- Public product docs of each competitor (cited per-row in evidence ledger).
- ADR-FORMS-0001..0006.
- PRD.md.
- `compliance.md`.
