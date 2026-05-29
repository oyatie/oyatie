# `workplace-integration` µservice — HRIS Engineer FAQ

20 real questions raised against the µservice that unifies HRIS + ATS + payroll + e-sign + shift scheduling at Oyatie.

---

**Q1. Why one µservice instead of separate HRIS, ATS, payroll, e-sign?**

ADR-0221: the employee entity is one canonical record. Fragmenting it across SaaS forces costly reconciliation, double-entry,
and audit gaps. One µservice with port abstractions to per-jurisdiction partners is cleaner than five SaaS integrations.

---

**Q2. Does `workplace-integration` actually run payroll, or does it call a partner?**

Both, per tenant_class:
- demo_trial: no native payroll; CSV export only.
- paid with per_seat billing_component: native US + UK payroll (calculation + withholding + filing).
- paid with per_usage billing_component: native multi-country payroll for 13+ jurisdictions; partners for the rest.
- paid with compliance_pack gating: regulator-cleared partners for every jurisdiction.

The native calculation engine lives in `crates/oya-workplace-integration-payroll-*` and is rule-driven.

---

**Q3. What e-sign levels are supported?**

Four (legally meaningful):
1. **Simple electronic signature** (eIDAS simple, US ESIGN+UETA, similar global). Click-to-sign with audit trail.
2. **Advanced electronic signature** (eIDAS advanced). Unique to signer, identifies signer, signer's sole control of signing data,
   linked to signed data such that tampering is detectable.
3. **Qualified electronic signature** (eIDAS qualified). Advanced + QSCD (Qualified Signature Creation Device, typically HSM-backed).
4. **FDA 21 CFR Part 11** (US healthcare). Specific audit-trail + biometric + cryptographic requirements.

Plus KR PKI, JEDI (Japan), Aadhaar (India), ECT Act (South Africa) — supported at paid with per_usage billing_component.

---

**Q4. How is clock-in attestation done?**

Three signals combined:
1. **Geofence**: GPS within X meters of declared work location.
2. **Wi-Fi MAC**: device sees an expected work-network BSSID.
3. **Device monotonic clock**: tamper-evidence via signed timestamp from device's secure enclave.

demo_trial uses 1 + 2; paid with per_seat billing_component adds 3. tenant_class adoption matrix details the accuracy guarantees.

---

**Q5. How are work permits / right-to-work handled?**

Per jurisdiction:
- US: E-Verify integration; I-9 form within 3 days of hire.
- UK: Right to Work check via Home Office API.
- EU: Work permit verification via national systems (varies by country).
- KR: HiKorea visa status check (sovereign tenants only; paid with compliance_pack gating).

The check is a mandatory step in the onboarding state machine; you cannot mark an employee Active without it.

---

**Q6. What about gig workers / 1099 contractors?**

`Employee::employment_type` is a closed enum: `W2` (US), `T4` (Canada), `PAYE` (UK/IE), `Form16` (India), `Independent_1099` (US),
`Independent_Other`. Each type triggers different onboarding paths + payroll calculations + tax reporting. Gig workers spanning
multiple platforms get `Employee::cross_platform_link` for canonical-identity tracking.

---

**Q7. How does the EU AI Act apply to hiring?**

Per ADR-0251 + EU AI Act Article 6 + Annex III: hiring algorithms are high-risk AI systems. When `intelligence` is used to rank
candidates, the EU AI Act pack must be active. The pack enforces:
- Decision logging with feature contributions.
- Human-in-the-loop for final decisions.
- Bias monitoring against EEOC / EU NDR categories.
- Right to explanation for rejected candidates.

Hiring without the pack is Cedar-forbidden for EU candidates.

---

**Q8. How is multi-state tax handled?**

The payroll engine tracks `work_state` + `residence_state` + special-case rules (CA-NV reciprocity, NY-NJ commuter, etc). State
withholding follows work_state in most cases; residence_state handles non-work-state credits. The rule engine enforces
state-by-state cases via `WithholdResult::MultiStateSplit`.

---

**Q9. What's the data retention for payroll records?**

Per US IRS: 4 years from due date of tax. Per EU GDPR: lawful basis is usually "legal obligation" — keep for required tax period
(country-dependent, often 7-10 years), then delete. paid with compliance_pack gating tenants in healthcare may need HIPAA's 6-year retention.

The retention policy lives in the active compliance pack; the µservice enforces it.

---

**Q10. How does benefits enrollment work?**

`workplace-integration` exposes a benefits-enrollment workflow that aggregates per-jurisdiction partner offerings:
- US: health (Justworks Health partners), dental, vision, FSA/HSA, 401(k).
- KR: GHS + private health add-ons.
- AU: super.
- SG: CPF.

The enrollment is a workflow in `workflow-engine`; status syncs into the canonical `Employee` record.

---

**Q11. Can we run multiple e-sign CA backends?**

Yes, per tenant_class. paid with per_seat billing_component: DocuSign, AdobeSign, HelloSign as legacy partners; Oyatie's native sign service for new docs. The choice is
per-tenant per-document-type. We always re-sign + chain-anchor every document in our audit chain regardless of CA.

---

**Q12. How are PTO + leave tracked?**

`Employee::balances` tracks per-jurisdiction-mandated leave (US: FMLA, CA, MA PFML, NY PFL, etc; EU: per-country statutory; AU:
annual + personal/carer + long service). Custom policies (unlimited PTO, anniversary lumps) layer on top.

---

**Q13. What's the time-clock approval flow?**

Manager sees daily timecards; auto-approves if within tolerance (e.g. ± 10 min from scheduled); flagged otherwise. Disputes raised
within tenant_class-specific windows are resolved via the `workflow-engine` dispute workflow.

---

**Q14. How does offboarding work?**

State machine: Active → Notice → Offboarding → Separated. Each transition triggers:
- Notice: notify benefits broker (for COBRA / equivalent), trigger equity vesting acceleration if applicable.
- Offboarding: SaaS account de-provisioning, hardware return checklist, knowledge transfer document gen.
- Separated: final paycheck calc + delivery, final benefits notice, PTO payout.

Audit chain captures every state transition.

---

**Q15. What about contractors paid via 1099?**

Different workflow: `Independent_1099` contractors get a simpler onboarding (W-9, ACH info, no benefits), invoice-based payment
(no payroll withholding), 1099-NEC annual reporting. The same µservice handles them; just different `employment_type` flow.

---

**Q16. How is sovereign deployment done?**

Per ADR-0244 + sovereign region pattern: a sovereign tenant runs in sovereign cells. For workplace-integration that means:
- Employee PII stays in-country.
- Payroll partners are regulator-cleared in-country.
- E-sign uses regulator-cleared CA in-country (KR FSS-cleared CAs, EU eIDAS QSCD-listed, etc).

---

**Q17. Can we self-host the µservice on-prem?**

paid with compliance_pack gating-only. The µservice runs on Cloud Hypervisor + Kata as per ADR-0254; the on-prem profile uses bare-metal Cloud Hypervisor
on customer hardware. The CA partnership still requires regulator-cleared external CAs unless the tenant has its own qualified CA.

---

**Q18. How do we handle ATS — applicant flow into hire?**

The ATS surface tracks candidates from application → screening → interview → offer → hire. On `Stage::OfferAccepted`, the candidate
graduates to `Employee` (one canonical record); the ATS data becomes part of the employee history. No duplicate records.

---

**Q19. What's the audit chain story for HR data?**

Every employee-record mutation, every signature, every clock-in, every payroll run writes to `audit-chain`. Auditors can verify the
chain. Sensitive data (SSN, salary) is hashed in audit events; the audit chain proves the event occurred without exposing PII.

---

**Q20. How does this integrate with `crm`?**

`crm` owns customer/contact records; `workplace-integration` owns employee records. They share the `ontology::Person` entity but
view different facets. No cross-µservice writes; updates flow via the canonical `ontology` µservice.
