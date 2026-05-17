---
purpose: Oyatie — Security Program
doc_status: published
---

# Oyatie — Security Program

> **Status:** Draft v0.1 — 2026-05-09. Authoritative-deep.
> **Owner:** `ops-security`. Updates per [DOC-CATALOG.md `doc.security_program`](DOC-CATALOG.md).
> **Companion:** [PRIVACY-PROGRAM.md](PRIVACY-PROGRAM.md), [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md), [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md).

---

## 1. Threat model (cross-axis)

The Oyatie threat model assumes: a sophisticated, motivated adversary with credentials to one tenant; a supply-chain attacker who has compromised a transitive dep; an insider with elevated access to one axis; a regulator who can subpoena. Defense-in-depth across all 7 axes.

### 1.1 Threat actors

| Actor | Motivation | Capabilities | Mitigations |
|---|---|---|---|
| External APT | Tenant data exfil | Spear-phish; supply-chain; cloud-IAM enumeration | mTLS + per-cell isolation + audit chain + Trivy + Cosign + signed commits |
| Cybercriminal | Ransomware on tenant data | Phishing + lateral movement | Per-tenant cell isolation + KMS-shred + immutable backups + DR drills |
| Insider (engineer) | Accidental exposure / IP theft | Repo + cloud admin | Cedar policy + break-glass audit + foundation-bypass ledger + CODEOWNERS |
| Insider (operator) | Mis-config of tenant | Cloud control plane | M-of-N approval for high-risk ops + dry-run + rollback + audit |
| Foundry agent (rogue or hijacked) | Privilege escalation via tool use | Capability invocation | Autonomy ceiling + per-capability data-class allowlist + evidence emission + sandbox |
| Tenant adversary | Cross-tenant data access | Authenticated user | Engine-enforced row-level security per ADR-0006 + cell isolation + per-class data boundary |
| Plugin author (malicious) | Sandbox escape | Plugin runtime | Wasmtime sandbox per ADR-0023 + cap-gated PluginContext + Cosign signing per ADR-0039 |
| Regulator (subpoena) | Compelled disclosure | Legal process | Trust portal + per-tenant proof-of-erasure + jurisdictional minimization |
| State actor (KR / JP / US / EU / IN / etc.) | Sovereignty / lawful-intercept | Region-specific | Per-region data residency + per-pack regulator binding + transparent reporting |

### 1.2 Attack surfaces (per axis)

| Axis | Surface | Top threats |
|---|---|---|
| SaaS | Workflow / plugin runtime; tenant API | Plugin escape; cross-tenant access; injection |
| Workspace | Mail SMTP / IMAP receive; Doc CRDT; Drive object download; Meet WebRTC | Phishing; XSS in Doc; permission escalation; recording leak |
| Vertical | Per-vertical regulated data | Vertical-specific (PHI exfil, payment fraud, contract breach) |
| Foundry | Provider adapter (subscription cookies); capability invocation; tool sandbox | Subscription token theft; capability misuse; sandbox escape; prompt injection (taint zones) |
| Cloud | IAM; KMS; tenant compute; storage; network | IAM enumeration; key compromise; cell breach; lateral movement |
| Search | Crawler; index; SERP | Crawl-rate abuse; index poisoning; PII leak via query log |
| Ads | Auction; targeting; advertiser console | Click fraud; targeting bypass; advertiser-data exfil |

## 2. Security posture (12 controls)

1. **Defense-in-depth across all 7 axes.** Every cross-axis call must pass: identity check, RBAC/ABAC, data-class gate, cell-routing, audit emission.
2. **Zero trust** between cells, between tenants, between axes — no implicit trust.
3. **mTLS everywhere** via Istio Ambient per ADR-0044; service-to-service has authenticated peer.
4. **KMS-shred encryption** at rest per ADR-0043 envelope encryption; per-record DEK for HARD_DENY classes.
5. **Per-cell isolation evidence** with quarterly cross-tenant negative-access fuzz (per Issue #129).
6. **Supply-chain signing** Cosign keyless + Rekor + SBOM per ADR-0039; license-policy gate per drafted ADR.
7. **Signed commits + tags** per Issue #1299; merge-governance ruleset per #1295.
8. **Branch-protection-as-code** per #239; admission policy via Kyverno/OPA per #1306.
9. **Audit-chain emission** per ADR-0003 with 4-hour evidence-pack regeneration SLA.
10. **Break-glass** for every regulated capability; M-of-N approval (3-of-5 for catastrophic) + automated revoke after window.
11. **Quarterly threat-model refresh** per service (Issue #114).
12. **Quarterly red team + annual external pen test** per PCI-DSS Req 11 + ISO 27001 control set.

## 3. Per-axis security controls (top 5 each)

### 3.1 SaaS
- Per-tenant cell isolation with engine-enforced RLS (ADR-0006)
- Cedar authorization policy on every API call
- Webhook signing with rotating keys
- Plugin marketplace signature verification (ADR-0039)
- DSR cascade tested quarterly

### 3.2 Workspace
- Mail DLP + phishing classifier + sandboxed attachment scan
- Doc per-permission granular share with Cedar
- Drive per-object KMS-shred on delete
- Meet recording access only via trust portal with audit
- Workspace data-class respects tenant-class override (HIPAA Mail, PCI off, education-minor blocks)

### 3.3 Vertical
- Per-vertical regulator binding (HIPAA / MFDS / FSC / PIPA / GDPR …)
- Per-vertical hard-deny override (healthcare PHI, fintech PCI, education minors)
- Per-vertical control evidence emission (HIPAA controls, KISA controls, etc.)

### 3.4 Foundry
- Subscription token rotation + per-conversation isolation
- Per-capability data-class allowlist + Cedar gate
- Tool sandbox: Wasmtime default, Firecracker for higher-isolation
- Prompt-injection taint zones (untrusted content marked; downstream tools refuse)
- M-of-N break-glass for autonomy-tier T4 (auto-execute) operations

### 3.5 Cloud
- IAM Cedar + STS short-lived credentials; no long-lived API keys (per Issue #1298 OIDC)
- KMS HSM-backed (KCMVP for KR per ADR-0043)
- Per-cell network ACLs; no implicit east-west traffic
- IMDSv2 enforcement on all VMs; no IMDSv1 fallback
- WORM tier for compliance archives (Glacier-class with vault lock)

### 3.6 Search
- Per-tenant index segregation (engine-enforced row-level isolation)
- Crawl politeness + per-host budget; rate-limit per crawl-rights ledger
- SERP query-log DP-aggregated; per-user query NEVER cross-tenant linked
- Right-to-be-forgotten cascade + Cosign-signed proof-of-erasure

### 3.7 Ads
- Singleton ad-gate as the only sourcing service into ads from tenant data (per [PRIVACY-PROGRAM §2.2.4](PRIVACY-PROGRAM.md))
- Per-class data boundary enforced at runtime (HARD_DENY classes never enter auction features)
- Click fraud detection + IVT filtering
- Advertiser console MFA enforcement
- DP / k-anonymity wrappers on all aggregate exports

## 4. Continuous-control monitoring

Per [COMPLIANCE-MATRIX](COMPLIANCE-MATRIX.md), every control has an evidence cadence tracked by planned advisory lane `oya-foundry-fitness-control-monitoring`. Failures emit `EVT-CONTROL-EVIDENCE-MISSING` and trigger a runbook.

## 5. Sources scanned

CIS Controls v8, ISO 27001:2022, NIST SP 800-53, NIST CSF, OWASP Top 10 + LLM Top 10, KISA Security Assessment, Microsoft STRIDE, MITRE ATT&CK, ADR-0003 (audit chain), ADR-0006 (Object Graph isolation), ADR-0007 (Cedar policy), ADR-0008 (Data Use Boundary), ADR-0013 (license policy), ADR-0022 (autonomy ceiling), ADR-0023 (sandbox), ADR-0036 (plugin substrate), ADR-0037 (API stability), ADR-0039 (supply chain), ADR-0040 (progressive delivery), ADR-0042 (observability), ADR-0043 (secrets management), ADR-0044 (service mesh).
