---
doc_status: published
---

# Threat Model: <Service / Surface / Capability> — <YYYY-MM-DD>

> **Owner:** `ops-security` co-author + per-axis team owner
> **Cadence:** quarterly per service per [security-program.json §2 control 11](../security-program/security-program.json)
> **Companion:** [security-program.json](../security-program/security-program.json), ADR-0008 + ADR-0022 + ADR-0039

---

## 1. Subject

What is being threat-modeled. One paragraph.

## 2. Architecture diagram (logical)

```
[outside trust boundary]
        ↓
[edge / gateway]
        ↓
[axis surface] ──→ [cross-axis contract] ──→ [other axis]
        ↓
[per-cell isolation]
        ↓
[per-store / per-KMS]
```

## 3. Trust boundaries

- (between user / tenant)
- (between tenant / Oyatie internal)
- (between cell / cell)
- (between region / region)
- (between Oyatie / external service-provider)
- (between human / agent)

## 4. Actors (per [security-program.json §1.1](../security-program/security-program.json))

| Actor | Motivation | Capabilities | Likelihood | Worst-case impact |
|---|---|---|---|---|
| External APT | tenant data exfil | spear-phish + supply-chain + cloud-IAM enum | (per quarterly assessment) | (per data class touched) |
| Cybercriminal | ransomware | phishing + lateral | | |
| Insider (engineer) | accidental / IP theft | repo + cloud admin | | |
| Insider (operator) | misconfig | cloud control plane | | |
| Intelligence agent (rogue or hijacked) | privilege escalation via tool use | capability invocation | | |
| Tenant adversary | cross-tenant access | authenticated user | | |
| Plugin author (malicious) | sandbox escape | plugin runtime | | |
| Regulator (subpoena) | compelled disclosure | legal process | | |
| State actor | sovereignty / lawful-intercept | region-specific | | |

## 5. STRIDE per surface

| Element | S (spoofing) | T (tampering) | R (repudiation) | I (info disclosure) | D (DoS) | E (elevation) |
|---|---|---|---|---|---|---|
| (per surface enumerated) | | | | | | |

## 6. Mitigations applied (cross-reference security-program.json)

- mTLS via service mesh per ADR-0044
- KMS-shred encryption at rest per ADR-0043
- Per-cell isolation per ADR-0009
- Audit-chain emission per ADR-0003
- Cedar policy + autonomy ceiling per ADR-0007/0022
- Supply-chain Trivy + Cosign + SBOM per ADR-0039
- Per-class data boundary per ADR-0008
- DSR cascade per ADR-0038
- Plugin sandbox per ADR-0036
- Sandbox per ADR-0023

## 7. Residual risks

For each risk in §5 not fully mitigated, list:
- Risk
- Why mitigation is partial / unsupported
- Compensating control
- Acceptance authority (council)

## 8. Open mitigation work

| # | Mitigation | Owner | Type | ETA |
|---|---|---|---|---|
| 1 | (mechanical) | (team) | mechanical | <date> |
| 2 | (process) | (team) | process | <date> |

## 9. Per-regulator alignment

Per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md):
- Affected regulators (PIPA / GDPR / HIPAA / PCI / etc.)
- Per-regulator control alignment

## 10. Council sign-off

- ☐ ops-security lead
- ☐ Per-axis owning team lead
- ☐ Privacy council (if data-class touched)
- ☐ Council architecture (if cross-axis contract impact)

## 11. Sources scanned

- [security-program.json](../security-program/security-program.json)
- ADR-0007/0008/0022/0023/0039/0043/0044
- Per-surface dependency ledger
- KISA + ISO 27001 + SOC 2 + NIST CSF + OWASP Top 10 + LLM Top 10

*Threat model refresh per quarter; per-incident updates triggered by Sev 1/2.*
