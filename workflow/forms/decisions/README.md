---
doc_class: AdrIndex
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + council-architecture
doc_status: published
---

# forms service-scoped ADRs

This directory holds **service-scoped** Architecture Decision Records owned by the `forms` µservice per ADR-0131 §"Canonical folder shape". Repo-wide ADRs live at `docs/decisions/` (e.g., ADR-0105, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)).

Service-scoped ADRs are numbered `ADR-FORMS-####` (four-digit, sequential within this directory). The `FORMS` prefix prevents collision with the repo-wide `ADR-####` series and matches the convention adopted by sibling µservices migrating to ADR-0131 per-microservice flat layout (e.g., `ADR-WS-####` for workflow-studio).

## Index

| ADR | Title | Status |
|---|---|---|
| [ADR-FORMS-0001](ADR-FORMS-0001-form-definition-schema.md) | Form-definition schema — RFC 8785 JSON Canonicalization Scheme + form.v1 JSON-Schema profile | Accepted |
| [ADR-FORMS-0002](ADR-FORMS-0002-captcha-and-anti-spam.md) | Captcha selection — hCaptcha primary + Cloudflare Turnstile + Friendly Captcha fallback; reCAPTCHA forbidden in pack-eu/kr/us-hc | Accepted |
| [ADR-FORMS-0003](ADR-FORMS-0003-pii-column-encryption-and-residency.md) | PII column encryption — per-tenant DEK with envelope encryption (OpenBao root); per-pack residency | Accepted |
| [ADR-FORMS-0004](ADR-FORMS-0004-conditional-logic-and-branching-engine.md) | Conditional-logic engine — CEL (Google Common Expression Language) over declarative DAG; server-authoritative | Accepted |
| [ADR-FORMS-0005](ADR-FORMS-0005-ai-form-build-bounds.md) | AI-form-build T0/T1/T2 tier bounds — T0 + T1 intra-µservice; T2-cross gated by Cedar + ChangeSet review + reviewer-agent; Annex III §4 high-risk classification | Accepted |
| [ADR-FORMS-0006](ADR-FORMS-0006-e-signature-conformance.md) | E-signature conformance — eIDAS XAdES/PAdES/CAdES profiles; tenant-tier mapping (SES default, AES for Tier-D+, QES for Tier-G+) | Accepted |

## Cross-reference policy

- Every service-scoped ADR in this directory MUST reference the repo-wide ADRs it inherits from (e.g., ADR-0131 layout, ADR-0140 Cedar, ADR-0105 layer enum, ADR-0135 microservice naming, ADR-0139 SLO-gated promotion, ADR-0132 single-concern, ADR-0133 review cadence).
- Repo-wide ADRs MUST NOT depend on service-scoped ADRs; dependency direction is one-way (service-scoped depends on repo-wide).
- Service-scoped ADRs may reference each other freely within this directory.
- Supersession is recorded by adding `superseded_by:` to the old ADR's frontmatter and `supersedes:` to the new ADR's frontmatter; old ADRs are **never deleted** (per the documentation-and-adrs skill).

## Sibling µservice ADR directories

- `microservices/workflow-studio/decisions/` — workflow-studio service-scoped ADRs.
- `microservices/sheets/decisions/` — sheets service-scoped ADRs.
- `workflow/workflow-engine/decisions/` — workflow-engine service-scoped ADRs.
- (Other µservices acquire their own `decisions/` directory at the time they author their first service-scoped ADR.)

## Author + reviewer protocol

Per the documentation-and-adrs skill and ADR-0131:

1. Author a draft ADR under this directory using the structure: Status / Date / Context / Decision / Alternatives Considered (≥3 alternatives) / Consequences (≥3 downstream impacts).
2. Decision must be concrete (no TODO comments; no deferral within scope).
3. Consequences must list ≥3 downstream impacts (architecture / SLO / compliance / risk).
4. ADR must cross-reference (a) the repo-wide ADRs it inherits from, (b) named industry sources where applicable (RFCs, regulations, standards).
5. ChangeSet review per ADR-0110 with reviewer-agent APPROVE before merge to `dev`.
