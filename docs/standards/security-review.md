# Oyatie — Security Review Standard

> **Owner:** `ops-security`. Per-change-class checklist + threat-model triggers.
> **Companion:** [SECURITY-PROGRAM.md](../SECURITY-PROGRAM.md), [`templates/threat-model-template.md`](../templates/threat-model-template.md), [`checklists/incident-response.md`](../checklists/incident-response.md).

## 1. When security review is required

| Change class | Reviewer | Trigger |
|---|---|---|
| Auth / identity changes | `ops-security` + `platform-tenancy-identity` | per ADR-0007 |
| Secrets / KMS / HSM changes | `ops-security` | per ADR-0043 |
| Sandbox / plugin / capability changes | `ops-security` + `axis-foundry` | per ADR-0023 + ADR-0036 |
| Cross-tenant / cross-cell changes | `ops-security` + `axis-cloud` | per ADR-0009 |
| Public API surface changes | `ops-security` + `platform-api-sdk` | per ADR-0037 |
| New dependency adoption | `ops-security` | per ADR-0013 + ADR-0014 |
| New per-region pack onboarding | `ops-security` co-sign | per ADR-0010 |
| Supply-chain / signing / SBOM changes | `ops-security` | per ADR-0039 |

## 2. Per-PR checklist

1. ☐ Threat surface change identified
2. ☐ Threat-model addendum if surface materially changes
3. ☐ Per-data-class annotation verified per ADR-0008
4. ☐ Per-capability autonomy ceiling verified per ADR-0022
5. ☐ Audit-chain emission wired per ADR-0003
6. ☐ License-tier check passes per ADR-0013
7. ☐ Secrets handling per ADR-0043
8. ☐ mTLS via Istio Ambient per ADR-0044
9. ☐ Per-cell isolation evidence (if cross-cell) per ADR-0009
10. ☐ DSR cascade impact verified per ADR-0038

## 3. Triggers for full threat model refresh

- Material change to architecture diagram of a service
- New trust boundary added
- New external integration / partner
- Per-incident postmortem identifies model gap
- Per-quarter scheduled cadence per [SECURITY-PROGRAM §2 control 11](../SECURITY-PROGRAM.md)
- Per-regulator expectation (PCI-DSS Req 6.4 / 11.4 / 12.10; HIPAA Security Rule §164.308 risk analysis annual)

## 4. Per-incident security review

- Sev 1/2 with security class triggers automatic threat-model refresh
- Per-incident postmortem per [`templates/incident-postmortem-template.md`](../templates/incident-postmortem-template.md) includes security findings
- Mechanical prevention per [`docs/standards/prevention-doctrine.md`](../standards/prevention-doctrine.md) within 30d

## 5. Bypass policy

- Never bypass security review for: cross-tenant data flow; new auth surface; new secrets handling; new sandbox surface; new public API; new dependency
- Bypass for: dev-only tools (per ADR-0013 carve-out); cosmetic doc; non-product internal tooling
- Per-bypass `# review-bypass: <reason>` logged + audit-emitted

## 6. Sources
[SECURITY-PROGRAM.md](../SECURITY-PROGRAM.md), ADR-0003/0007/0008/0009/0010/0013/0014/0022/0023/0036/0037/0038/0039/0043/0044, OWASP Top 10 + LLM Top 10, MITRE ATT&CK, CIS Controls v8, ISO 27001, NIST CSF, PCI-DSS v4.0, HIPAA Security Rule.
