# Team: Ops — Security

## Mission
This team owns Oyatie's security program: threat modeling, vulnerability management, penetration testing, supply-chain security (ADR-0039 Trivy + Cosign), security-incident response, and the security review gate for sensitive-path PRs (auth, secrets, payments, PHI). It exists because the cohesion thesis — one tenancy, one identity, one audit chain spanning every axis — creates a single threat surface that requires a coordinated security posture. A breach in any axis can undermine trust across the entire product. It does **not** own compliance evidence packs (→ `ops-compliance`) or the audit chain infrastructure (→ `platform-audit-evidence`).

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting security operations
- **Surfaces:**
  - Security program doc (`SECURITY-PROGRAM.md`)
  - Threat model registry (per-axis threat models, updated per wave)
  - Vulnerability management: Trivy SARIF pipeline, dependency CVE triage, patch SLAs
  - Supply-chain: Cosign image signing attestation, SBOM registry, license-policy gate (co-owned with `axis-foundry` for tooling)
  - Penetration testing program: internal red team + third-party pentest cadence
  - Security-reviewer agent gate: every PR touching auth/secrets/payment/PHI paths requires security-reviewer sign-off
  - Security-incident response: Sev-1 security-class incidents (classification, escalation, disclosure)
- **Cross-axis contracts (DESIGN §10):**
  - `Autonomy ceiling policy` (co-reviewer) — security review on all autonomy-ceiling policy changes
  - `IAM / SSO / SAML / OIDC IdP` (co-reviewer) — security review on all identity contract changes
- **Catalog records:** security tooling (Trivy pipeline, Cosign attestation in `axis-foundry` Foundry crates)
- **Runbooks:** `runbooks/security-incident-response.md`, `runbooks/cve-critical-patch.md`, `runbooks/supply-chain-compromise.md`, `runbooks/cedar-policy-breach.md`
- **ADRs:** ADR-0039 (supply-chain: Trivy + Cosign — co-owner with `axis-foundry`)

## In-scope work
- Threat modeling: per-axis threat models (STRIDE), updated at each wave gate; architectural threat review on cross-axis contract changes
- Vulnerability management: Trivy SARIF triage (4-layer: OS + lang + container + IaC), CVE severity classification, patch SLA enforcement (Critical: 7 days, High: 30 days)
- Supply-chain security: Cosign image signing attestation, SBOM publication per release, license-policy gate enforcement (AGPL/GPL hard-fail)
- Penetration testing: quarterly internal red-team exercises; annual third-party pentest; bug-bounty program at W-Cloud-Stable
- Security-reviewer agent: on-call security reviewer for auth/secrets/payment/PHI PRs; SLA: 1 business day
- Security-incident response: Sev-1 security-class — contain, investigate, disclose, remediate, audit-chain record
- OT security review: quarterly audit for `vertical-industrial` OT write paths
- Credential hygiene: secrets scanning (no secrets in git), key rotation policies, SecretProvider architecture review
- Security training: annual security awareness, phishing simulation

## Out-of-scope (anti-scope)
- Compliance evidence pack authorship (→ `ops-compliance` — security provides security controls evidence; compliance assembles the packs)
- Audit chain infrastructure (→ `platform-audit-evidence`)
- Physical security (→ cloud provider / colo operator)
- HR background checks (→ HR / `vertical-corporate`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-foundry` | Supply-chain tooling (Trivy pipeline, Cosign), Foundry gates | Per-release |
| `platform-audit-evidence` | Security incident audit chain records | Per incident |
| `platform-tenancy-identity` | Cedar policy threat model inputs | Quarterly |
| `ops-compliance` | Compliance evidence pack requirements (what controls need security evidence) | Monthly |
| `ops-sre-reliability` | Incident severity classification alignment | Per Sev-1 |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All axis + vertical teams | Security-reviewer sign-off on sensitive PRs | Per sensitive PR (SLA: 1 business day) |
| `axis-foundry` | Supply-chain ADR-0039 co-ownership, autonomy-ceiling threat model | Per policy change |
| `ops-compliance` | Security controls evidence for compliance packs | Monthly |
| `platform-tenancy-identity` | Cedar policy security review | Per policy change |
| `vertical-industrial` | OT write-path security audit | Quarterly |
| `vertical-healthcare` | PHI-path security review | Per PHI-adjacent PR |
| `vertical-fintech` | Payment/AML-path security review | Per payment PR |

## Success metrics
- **Critical CVEs unpatched > 7 days:** 0
- **High CVEs unpatched > 30 days:** 0
- **Supply-chain Cosign attestation coverage:** 100% of release artifacts
- **Security-reviewer SLA (1 business day):** ≥ 99% of sensitive PRs
- **SBOM published per release:** 100%
- **Annual third-party pentest:** completed; findings remediated per severity SLA
- **Secrets detected in git history:** 0 (secrets-scanning gate)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for cross-axis security architecture disputes
- Legal: founder + counsel for regulatory disclosure obligations (GDPR breach notification, PIPA breach notification)
- Founder: as last resort (Sev-1 security incident)

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — CVE triage queue, security-review SLA, supply-chain status
- Cross-team review: quarterly threat-model review with each axis lead; annual pentest debrief

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; all security-team PRs require peer review from another security team member
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; supply-chain ADR (ADR-0039) amendments are P0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Critical CVE in production unpatched | High | 7-day SLA; automated Trivy alert → PagerDuty |
| Supply-chain compromise via malicious dependency | Catastrophic | Trivy 4-layer scan; Cosign attestation; AGPL/GPL hard-gate; SBOM diff on each PR |
| Credential leaked in git | Catastrophic | Secrets-scanning pre-commit hook + CI gate; rotation runbook |
| Security-reviewer bottleneck delays sensitive PRs | Medium | On-call rotation; SLA monitoring; escalation to `ops-security` lead |

## Sources scanned
PRD.md §7 (risks: agent autonomy ceiling, PHI leak), DESIGN.md §10 (autonomy ceiling, IAM rows), ADR-0039, DOC-CATALOG.md §2.1 (doc.security_program owner), CLAUDE.md (security-reviewer agent trigger rules).
