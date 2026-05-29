---
doc_class: FAQ
microservice: compliance
persona: compliance-engineer + privacy-engineer + dpo
related_adrs: [ADR-COMP-001, ADR-0304, ADR-0010, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Compliance Engineer FAQ — compliance

## Why are pack overlays immutable + versioned?

Per ADR-COMP-001 § Decision Constraint COMP-C10. Three reasons:

1. **Auditability**: historical compliance decisions can be replayed against the pack version that was active at the time. If a 2025 decision is challenged in 2027, we replay against 2025's pack.
2. **Hotfix safety**: emergency regulator updates create a new version (hotfix) without rewriting history. Old version stays queryable.
3. **Pack rule integrity**: if pack rules were mutable, an attacker (or honest mistake) could weaken historical evidence post-hoc.

Pack overlays are stored as content-addressable + Ed25519-signed bundles. Once published, they cannot be edited. Pack hotfixes supersede; the old version remains for historical decisions.

## What's the 6-step precedence (ADR-COMP-001 § Decision)?

Step-by-step:

1. **Absolute legal hard-stop wins**: e.g., "CSAM auto-remove" mandatory under 18 USC § 2258A; no pack can override.
2. **Data residency restriction wins over availability**: e.g., KR-PIPA Art 28 cross-border transfer ban beats general availability convenience.
3. **Higher restriction wins** for retention / breach / consent / export / automated-decisioning: this is the most-common path. HIPAA 6-y retention beats default 1-y.
4. **More specific jurisdiction wins** when stricter: California CCPA-specific rules beat general US privacy floor.
5. **Tenant explicit stricter policy can raise floor**: tenant says "always require 2-factor for admin"; even pack default allows single-factor, tenant rule wins.
6. **Product recommendation only when no pack covers**: fallback if no pack has a rule for a given primitive.

Cedar enforces this order. Pack rules carry `restriction_level` + `legal_basis` to drive step 3.

## Why can't tenant policy weaken regulator floor?

Per ADR-COMP-001 § Decision. A tenant cannot opt out of a pack rule once subscribed to the pack. Example:

- Tenant subscribes to HIPAA pack (covered entity status established via BAA).
- HIPAA Privacy Rule says minimum 6-y retention.
- Tenant cannot say "actually we want 1-y retention" — that would violate HIPAA + tenant might lose BAA + be fined.

Tenants CAN add stricter rules (raise the floor). They CAN'T lower the floor below pack requirements. Cedar `compliance::pack::weaken_regulator_floor` is **unconditionally forbidden** (per ADR-COMP-001).

## What's ADR-0304 and how does it relate to ADR-COMP-001?

Per ADR-0304 (cross-jurisdiction conflict resolution doctrine). ADR-0304 establishes:

- Cross-jurisdiction precedence (e.g., US-vs-EU conflict).
- Residency hard-stop semantics.
- Transparency-report obligation.

ADR-COMP-001 binds these into the compliance µservice's implementation. ADR-0304 is the **authority document** (doctrine); ADR-COMP-001 is the **implementation** that obeys it.

If you're authoring a cross-jurisdictional rule, cite both ADRs.

## How does DSAR automation work with multi-pack conflicts?

Per ADR-COMP-001 § Decision + IP-003-gdpr-dsar-automation-pipeline. DSAR (Data Subject Access Request) path:

1. User submits DSAR via tenant portal.
2. compliance µservice creates `DsarRequest` row.
3. Fan-out to product µservices (drive, messenger, mail, calendar, identity).
4. Each µservice collects data the subject has access to.
5. Conflict resolution runs:
   - GDPR Art 15 grants access.
   - HIPAA may restrict access to PHI if subject is not the patient OR is the patient but provider has clinical-care reasons.
   - SOX may restrict access to material non-public information.
6. Resolver applies 6-step precedence to each data_class.
7. Bundle assembled (granted parts) + transparency report (denied parts with legal basis).
8. User receives bundle + transparency report.

DSAR fulfillment SLO: 30 d (GDPR), 45 d (CCPA), 30 d (PIPL). oyatie typically completes in 14 d at paid on-prem-connected cell_topology.

## What's a DPIA and when is it required?

Per IP-DPIA-001 + GDPR Art 35. DPIA (Data Protection Impact Assessment) is required when processing is "likely to result in high risk to rights and freedoms". Examples:

- Automated decision-making (Art 22).
- Large-scale processing of special categories (Art 9: health, ethnicity, political, religious, etc.).
- Systematic monitoring of publicly accessible areas.
- Cross-border data transfers in high-risk jurisdictions.

oyatie's DPIA orchestration:

1. Tenant initiates DPIA for a planned processing activity.
2. System computes risk score per pack (GDPR risk, KR-PIPA risk, etc.).
3. Recommends safeguards.
4. Tenant DPO reviews + commits the DPIA.
5. DPIA stored as compliance evidence; updated annually or on processing changes.

## How does the EU AI Act Annex III refusal pipeline work?

Per ADR-COMP-001 + IP-EU-AI-Act-Annex-III + ADR-MAIL-0004 (precedent for spam classifier). EU AI Act 2024/1689 Annex III lists "high-risk AI systems":

- Biometric identification.
- Critical infrastructure.
- Educational/training assessment.
- Employment + worker management.
- Essential private + public services.
- Law enforcement.
- Migration + border control.
- Justice + democracy.

For tenants subject to EU AI Act:

- Identify in-scope AI features (e.g., spam classifier, behavioral risk scoring, automated decision systems).
- Disable or restrict to pack-allowed scope.
- Provide refusal evidence to regulator on request.
- Document conformity assessment under Art 26.

oyatie auto-flags features when tenant subscribes to EU AI Act pack.

## How is the effective policy projected efficiently?

Per ADR-COMP-001 § Implementation Notes capacity math. With 20 packs × 500 rules/pack = 10 000 candidate rules per tenant, naive evaluation is slow.

Optimizations:

1. **Pre-index by primitive + data_class + jurisdiction**: filter to relevant rules quickly.
2. **Cache `pack_set_hash`**: tenants with same pack set + version share computed effective policy.
3. **Incremental recompute**: only re-evaluate the affected portion when a pack changes.
4. **Async projection**: `compliance.effective-policy.changed.v1` event signals product µservices to refresh.

Target: p95 compute ≤ 100 ms (per ADR-COMP-001 verification).

## What's the difference between compliance + governance µservices?

- **compliance**: pack legal semantics + DSAR/DPIA pipeline + regulator-export + transparency reports. The substrate for "what must be true" by law.
- **governance**: lane runtime + evidence projection + retention execution. The substrate for "what must be verified continuously" by CI lanes.

compliance defines the rules (e.g., "HIPAA requires 6-y retention"). governance executes the retention (e.g., "delete this row at year 6 + 1 day").

compliance owns pack-overlay precedence + conflict resolution. governance owns evidence aggregation + retention execution + lane runtime.

## How is pack publishing audited?

Per ADR-COMP-001 § Decision + Cedar `compliance::pack::publish`. Pack publishing:

1. Submit signed pack schema + Cedar policy fragments + scorecard references.
2. Cedar permits `compliance::pack::publish` for compliance owners (typically the pack maintainer team).
3. Pack stored content-addressable in audit-chain-backed registry.
4. `compliance.pack.published.v1` event emitted (audit-chain-sealed).
5. Tenants opt-in via `compliance.pack.activated.v1`.
6. Hotfix uses `compliance::pack::hotfix` with emergency reason or 7-d soak.

Each version is queryable by historical date. Auditors can verify "which pack version was active on 2026-03-15".

## How is cross-jurisdictional data transfer handled?

Per ADR-COMP-001 § Implementation Notes sovereign path + ADR-0010. Cross-border:

- **GDPR**: Art 49 transfer mechanism evidence (SCCs, BCRs, adequacy decision) required.
- **PIPL Art 38**: Cross-border assessment by CAC (Cyberspace Administration of China) for "important data".
- **KR-PIPA Art 28**: Consent + transfer agreement; in some cases prior approval from PIPC.
- **UK DPA**: equivalent to GDPR + UK SCCs.

When a tenant has multi-jurisdiction packs (e.g., GDPR + PIPL), a query crossing borders requires transfer mechanism evidence. compliance µservice maintains evidence pointers (typically SCC documents stored in drive with WORM lock).

## What's the regulator portal (paid tenant_class)?

Per IP-007-auditor-readonly-portal. Each pack has a dedicated auditor view:

- Read-only access to tenant's effective pack policy for the auditor's pack scope.
- Per-rule evidence pointer (links to audit-chain events).
- DSAR queue + DSAR resolution timeline.
- Breach notification timeline.
- Sample evidence bundles ready to export.

Cedar `compliance::regulator_request::*` actions are gated. The auditor cannot modify policy or read non-scoped data.

## How is breach notification automated?

Per IP-014-manual-evidence-upload-flow + pack-specific breach rules. Per pack:

- GDPR: 72 h to notify DPA (Art 33).
- HIPAA: 60 d for breaches affecting 500+ individuals; immediate for 500+ in same jurisdiction.
- PCI DSS: notify card brand + affected individuals.
- CCPA: notify "without unreasonable delay" + within 30 d.
- KR-PIPA: notify PIPC within 24 h.

compliance µservice maintains a breach notification clock per pack. When tenant declares a breach:

1. Clock starts.
2. Cascade to legal + DPO via notification.
3. Auto-drafts regulator notification per pack template.
4. Tracks dispatch.
5. Audit-chain seals the timeline.

## How does migration from Drata/Vanta/Hyperproof work?

See `migration-playbooks/from-drata.md` (governance µservice) + `migration-playbooks/from-onetrust.md` (this µservice). compliance + governance jointly handle the migration:

- **governance**: imports continuous evidence + control mappings.
- **compliance**: imports pack subscriptions + conflict rules + DSARs + DPIAs.

Phased cutover: certification-by-certification (SOC 2 first; then ISO 27001; then HIPAA, etc.).
