---
doc_class: User-Journey-README
journey_id: j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege
slice: ombudsperson-cross-tenant-mediation-need-to-know-cedar-privileged-evidence-merkle-anchored-regulator-compellable
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Felix Tan (white/middle-office; ombudsperson — confidential employee complaints)
audience_type: B2B_OMBUDSPERSON + INTERNAL_INVESTIGATIONS + WHISTLEBLOWER_INTAKE
microservice_count: 5
pack_overlay_anchor: EU-Whistleblower-Directive-2019-1937 + US-SOX-806 + KR-ACRC-Anti-Corruption-Act + EEO-Title-VII + GDPR-Article-9 + Attorney-Ombudsperson-Privilege
related_adrs:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0246-mls-rfc-9420-e2ee-personal-messenger
  - ADR-0251-compliance-pack-primitive
  - ADR-0263-observability-emission-contract
  - ADR-0253-http3-quic-default-protocol
  - ADR-0252-hlc-default-truetime-tier
---

# j171 — Ombudsperson Felix Tan handles a harassment allegation against a VP across employee + employer tenants over 14 days

## At a glance

Felix Tan (陳家樂 / Tan Ka-lok in Cantonese; Felix is his English given name) is a **44-year-old certified ombudsperson** (IOA — International Ombudsman Association — certified, OCO designation 2022-09) on staff at **Halberd-Mercer Holdings Limited**, a Singapore-headquartered conglomerate (SGX:HMH; ~31,400 employees across 14 countries; food + property + healthcare segments). Felix is Singapore-Chinese, born in Toa Payoh 1982, MA-Counseling NUS 2009, JD SMU 2014 (qualified Singapore Bar but does not practice externally), joined Halberd-Mercer's office of the ombudsperson in 2020-04. He reports administratively to the Board's Independent Audit & Risk Committee chair (Mrs. Sarojini Iyer-Krishnan, ID-1968) and operationally to no one — the IOA-recognized **independence + confidentiality + impartiality + informality** four standards apply.

It is **Monday May 3, 2027, 09:14 SGT (+08:00)**. Felix's secure intake channel received a confidential complaint over the weekend (Sunday 22:18 SGT) from a complainant identified only by her ombudsperson-channel handle `complainant-2027-Δ47` (real identity sealed). The complaint is a **harassment allegation against a named VP-level executive at a subsidiary**:

- **Complainant**: A 31-year-old female product manager working at **Halberd-Mercer Property Singapore Pte Ltd** (subsidiary; tenant `halberd-mercer-property-sg`). Her real name (sealed) is **Priscilla Lim Hui-min**, employee ID HMP-SG-2017-3082. She is a personal-tenant holder (her own oyatie personal tenant `priscilla-lim-personal-2018`) which she uses to file this complaint from her personal phone outside business hours.
- **Respondent**: **Mr. Aloysius Goh Kheng-Soon** (Managing Director-level, equivalent VP), Halberd-Mercer Property Singapore, employee ID HMP-SG-2009-0014. Sealed at complaint stage; visible only to Felix.
- **Allegation**: A pattern of sexualized comments + unwelcome physical contact during a 4-month period (2027-01 through 2027-04), culminating in a specific incident on 2027-04-22 at the Halberd-Mercer property leasing team offsite at Capella Sentosa where Aloysius placed his hand on Priscilla's lower back without consent in the corridor outside the dining room (~21:18 SGT), and a Whatsapp message at 23:14 SGT that night that contained the phrase 「你今天看起来很性感」 ("you look very sexy today").

The journey is a **cross-tenant mediation under ombudsperson legal privilege**: Priscilla files from her **personal tenant**; Felix's office sits inside the **employer tenant**; the Cedar privilege boundary keeps Aloysius's name + the substance of the allegation visible **only** to Felix until either (a) Priscilla escalates to formal HR/EEOC-class process, or (b) Felix invokes the **mandatory-reporter exception** for child safety / criminal threat / imminent harm (not applicable here), or (c) regulator compels under EU Whistleblower Directive Article 22 / SOX § 806 / KR ACRC Article 13.

Microservices: `messenger` (NEED-TO-KNOW privileged channel with MLS E2EE per ADR-0246), `drive` (WORM-locked evidence room for the Whatsapp screenshots + her own contemporaneous notes + the corridor camera capture if obtained later), `audit-chain` (Merkle anchor with **privileged-content tag** — the regulator can compel the anchor proof without ombudsperson seeing payload), `community` (channel for the moderator-decision-appeal handoff if Priscilla's anonymous post in #womenintech-halberd is removed and she wants to appeal), `governance` (the formal escalation gate if Priscilla decides to proceed to investigation, requiring Cedar permit + Audit & Risk Committee notification).

The journey covers Felix's **14 days** (May 3 → May 17) of:

1. **messenger** µservice — NEED-TO-KNOW MLS E2EE privileged channel; only Felix + the complainant principal pair join; Aloysius's name + Halberd-Mercer Property Sg leadership cannot see; Cedar denies enumeration; the channel is sealed under attorney-ombudsperson-privilege class
2. **drive** µservice — WORM-locked evidence room; Whatsapp screenshots, contemporaneous notes, corridor incident reconstruction; retention 7 years per Halberd-Mercer ombuds office records-retention rule; privileged-content tag
3. **audit-chain** µservice — Merkle anchor with **privileged-content tag**; regulator-compellable inclusion proof without payload disclosure; per ADR-0263 emission contract with payload-class redaction
4. **community** µservice — Priscilla's anonymous post in the Halberd-Mercer #womenintech community channel was removed by a moderator on 2027-05-02 (the act that triggered her decision to file the formal complaint); the moderator-decision-appeal handoff comes to Felix's office
5. **governance** µservice — the formal escalation gate; if Priscilla decides to proceed (she does NOT in this journey; she elects ombudsperson-mediated resolution); Cedar permit + ARC notification + investigation Cedar bundle on standby

Microservices: `messenger`, `drive`, `audit-chain`, `community`, `governance`. Secondary: `identity` (Priscilla's passkey both from personal-tenant and from employer-tenant; cross-tenant boundary), `tenancy` (personal ↔ employer cross-tenant), `notes` (Felix's working notes; privileged class), `compliance` (the pack-manifest assertion that the EU-WD/SOX/ACRC packs are active), `cell` (the EU+SG WORM cells for privileged retention), `observability` (with redaction; no PII in metrics).

## Why this journey matters

Felix Tan is **MASTER-ROSTER §5.7 row 318** — the canonical ombudsperson persona at a large multi-national B2B enterprise. This persona covers ~14,200 ombudsperson-class roles globally (BLS 2024 code 13-1041 narrowed to "Compliance Officer + Ombudsperson designation"). Ombudsperson-mediated complaint handling is the most privacy-sensitive workflow on the platform; a single payload leak destroys the IOA confidentiality standard + exposes the company to retaliation claims + can violate EU 2019/1937 Article 16 (confidentiality protection).

The journey closes:

- **Critical-path row 207** (Cross-tenant NEED-TO-KNOW Cedar permit class — complainant's personal tenant + employer ombuds office)
- **Critical-path row 208** (Ombudsperson-privileged channel — only the dyad can read; channel not enumerable; metadata redacted)
- **Critical-path row 209** (WORM-locked privileged evidence drive — 7-year retention, privileged-content tag, regulator-compellable)
- **Critical-path row 210** (Merkle anchor with privileged-content tag — inclusion proof without payload disclosure; ADR-0263 redacted emission)
- **Critical-path row 211** (Community moderator-decision-appeal handoff to ombuds office — the appeal class that bridges public community moderation and confidential ombuds intake)
- **Critical-path row 212** (Mandatory-reporter exception path — the override branch for child safety / criminal threat / imminent harm; not invoked here but the code path must exist)

Hyperscaler benchmark: traditional ombudsperson tools are paper + voicemail + dedicated email; specialized SaaS (Convercent + NAVEX + EthicsPoint) handle whistleblower intake but do not handle cross-tenant boundary (personal ↔ employer) or ombuds-privilege Cedar class. The cross-tenant privileged channel with regulator-compellable Merkle attestation is unique to oyatie's [[substrate-vs-product]] architecture + [[cedar-universal-gate]] doctrine.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 3 09:14 SGT → May 17 18:48 SGT across 14 days | Singapore weather + Cantonese/Hokkien dialogue + Whatsapp screenshots + Capella Sentosa offsite + named board chair + privilege boundary |
| `ux-flow.md` | Felix's ombuds intake console + Priscilla's mobile complaint composer + privileged channel + WORM evidence room + community appeal handoff + (latent) governance escalation | Per-screen Cedar permit; privilege boundary indicator; mandatory-reporter exception copy text |
| `handshake.md` | Per-µservice API; MLS E2EE channel send/receive; WORM-tag write; Merkle privileged anchor; community appeal handoff | Each row names cross-tenant boundary + Cedar permit + audit class; redaction policy |
| `integration-test-plan.md` | Cross-tenant boundary fuzz + Cedar deny coverage + WORM-tag immutability + Merkle proof without payload + community appeal handoff | Per-test seed + privilege-boundary invariant + mandatory-reporter exception test |
| `schemas/cedar-policy.cedar` | Ombuds privilege Cedar bundle | NEED-TO-KNOW permit + payload-class allowlist + mandatory-reporter exception + escalation gate |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Cantonese + Hokkien + Mandarin + Singapore-English preservation; privileged-content envelopes; redaction tags |
| `schemas/openapi-ombuds-intake.json` | OpenAPI for ombuds intake endpoints | Intake + privileged channel + WORM evidence + Merkle anchor + community appeal handoff |
| `schemas/openapi-cross-tenant-privilege.json` | OpenAPI for cross-tenant privilege boundary | Personal ↔ employer boundary; principal mapping; redaction emission |
| `schemas/ombuds-case-state-machine.yaml` | 7-state ombuds case lifecycle | intake → triage → privileged_channel_open → evidence_collected → mediation → resolution → archive (or → escalation_to_governance) |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `messenger` | NEED-TO-KNOW MLS E2EE privileged channel; only the Felix-Priscilla dyad joins; channel not enumerable | row 208 |
| `drive` | WORM-locked privileged evidence room; 7-year retention; privileged-content tag | row 209 |
| `audit-chain` | Merkle anchor with privileged-content tag; regulator-compellable inclusion proof without payload | row 210 |
| `community` | Moderator-decision-appeal handoff to ombuds office | row 211 |
| `governance` | Latent escalation gate; Cedar permit + ARC notification on Priscilla's escalation decision (not invoked) | row 207 + row 212 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Priscilla's passkey from personal tenant + employer tenant; cross-tenant boundary; Felix's YubiKey 5C NFC + Halberd-Mercer ombudsperson title attestation |
| `tenancy` | `priscilla-lim-personal-2018` ↔ `halberd-mercer-property-sg` ↔ `halberd-mercer-holdings-corporate-sg`; tri-tenant boundary; Felix's principal sits in the corporate ombudsperson office tenant |
| `notes` | Felix's working notes; privileged class; auto-redacted in observability emissions |
| `compliance` | Pack-manifest assertion (EU-WD + SOX-806 + KR-ACRC + EEO-Title-VII + GDPR-Art-9 + Attorney-Ombudsperson-Privilege); pack overlay validation |
| `cell` | EU-Frankfurt-tier-1-privileged-worm cell for regulator-compellable + SG-Singapore-tier-2-tenant cell for live intake |
| `observability` | Redacted emission; no payload in metrics; privileged-class emission rule per ADR-0263 |

## Pack overlays (6 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| EU-Whistleblower-Directive-2019-1937 | EU Directive applies to Halberd-Mercer's EU subsidiaries; ombuds protection extends to EU + EEA + non-EU complainants by Halberd-Mercer policy | `pack-eu-wd-2019-1937-v2` |
| US-SOX-806 | Halberd-Mercer Holdings is SGX-listed; SOX § 806 anti-retaliation extends to its dual-listed ADR class | `pack-sox-806-anti-retaliation-v3` |
| KR-ACRC-Anti-Corruption-Act | KR subsidiary Halberd-Mercer Property Korea Co., Ltd. is in scope; ACRC Article 13 ombudsperson independence | `pack-kr-acrc-art-13` |
| EEO-Title-VII | Halberd-Mercer has US affiliates; EEOC Title VII applies | `pack-eeo-title-vii-2027` |
| GDPR-Article-9 | Health + sexual-orientation special-category data may surface; Article 9 explicit consent required for downstream sharing | `pack-gdpr-article-9-special-category` |
| Attorney-Ombudsperson-Privilege | The IOA privilege class; ABA Model Rule 1.6 by analogy; Singapore evidence Act § 128 (privileged communication) | `pack-ombudsperson-privilege-ioa-v2` |

## Regulatory anchors

1. **EU Whistleblower Directive 2019/1937** — Article 16 (confidentiality protection), Article 22 (regulator compulsion exception)
2. **US Sarbanes-Oxley § 806** — 18 U.S.C. § 1514A — anti-retaliation
3. **KR Anti-Corruption and Civil Rights Commission Act** — Article 13 — ombudsperson independence
4. **EEOC Title VII** — 42 U.S.C. § 2000e — harassment and retaliation
5. **GDPR Article 9** — special category data
6. **Singapore Evidence Act § 128** — privileged communication
7. **Singapore Workplace Fairness Act 2024** — passed 2024-08, in force 2026-01-01 — anti-harassment framework
8. **IOA Standards of Practice** — confidentiality (§3), independence (§1), impartiality (§4), informality (§5)
9. **ABA Model Rule 1.6** — confidentiality of information (by analogy for ombudsperson)
10. **ADR-0243 + ADR-0244 + ADR-0245 + ADR-0246 + ADR-0251 + ADR-0263 + ADR-0253 + ADR-0252**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `sg-singapore-tier-2-tenant-halberd-mercer-property-sg` | Subsidiary tenant cell | Priscilla's employer-side artifacts |
| `sg-singapore-tier-1-corporate-halberd-mercer-holdings` | Corporate ombudsperson office cell | Felix's primary cell |
| `sg-singapore-tier-2-personal-priscilla-lim-2018` | Priscilla's personal-tenant cell | Filing from personal phone |
| `eu-frankfurt-tier-1-privileged-worm` | Privileged-class WORM retention cell | 7-year evidence retention; EU residency for EU-WD compliance |
| `external-transparency-log-batch-2027-05-17` | External transparency log | Privileged-content-tag inclusion proof |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"felix.tan@halberd-mercer-holdings-corporate-sg",
    action in [
        Action::"messenger.privileged_channel_open",
        Action::"messenger.privileged_channel_send",
        Action::"messenger.privileged_channel_receive",
        Action::"drive.privileged_worm_write",
        Action::"drive.privileged_worm_read",
        Action::"audit_chain.privileged_anchor_emit",
        Action::"community.moderator_appeal_handoff_receive",
        Action::"governance.escalation_gate_evaluate"
    ],
    resource is OmbudsCase
) when {
    principal.role_in_tenant("halberd-mercer-holdings-corporate-sg") == "ombudsperson_certified_ioa" &&
    resource.privilege_class == "ombudsperson_privileged" &&
    context.passkey_assertion_present == true &&
    context.title_attestation_ioa_oco_2022 == true
};

permit (
    principal,
    action == Action::"messenger.privileged_channel_send",
    resource is PrivilegedChannel
) when {
    resource.channel_class == "ombudsperson_privileged_dyad" &&
    resource.member_count == 2 &&
    principal in resource.permitted_principals &&
    context.payload_class in [
        "complainant_narrative",
        "complainant_evidence_attachment",
        "ombudsperson_clarification_question",
        "ombudsperson_mediation_option",
        "complainant_decision_intent"
    ] &&
    context.mls_e2ee_envelope_intact == true
};

forbid (
    principal,
    action in [
        Action::"messenger.channel_enumerate",
        Action::"messenger.channel_metadata_read"
    ],
    resource is PrivilegedChannel
) when {
    resource.privilege_class == "ombudsperson_privileged" &&
    !(principal in resource.permitted_principals) &&
    !(context.regulator_compulsion_order_id != "")
};

permit (
    principal == User::"regulator-compelled-principal",
    action == Action::"audit_chain.privileged_anchor_inclusion_proof_request",
    resource is MerkleAnchor
) when {
    context.regulator_compulsion_order_id != "" &&
    context.regulator_class in ["EU_WD_Article_22", "SOX_806_subpoena", "KR_ACRC_Article_13_demand", "SG_Court_Order"] &&
    resource.privilege_class == "ombudsperson_privileged"
    // proof only — payload is not disclosed via this permit
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J171-001 | Community moderator-decision-appeal handoff event arrives at ombuds office; Felix opens ombuds case; audit `EVT-J171-COMMUNITY-APPEAL-HANDOFF-001` |
| AC-J171-002 | Priscilla initiates intake from her personal tenant; cross-tenant principal mapping established; audit `EVT-J171-INTAKE-INITIATED-002` |
| AC-J171-003 | NEED-TO-KNOW privileged dyad channel opened (Felix + Priscilla only); MLS E2EE per ADR-0246; channel not enumerable; audit `EVT-J171-PRIVILEGED-CHANNEL-OPENED-003` |
| AC-J171-004 | 6 Whatsapp screenshots + 3 contemporaneous notes + 1 corridor incident reconstruction uploaded to WORM drive; privileged-content tag; 7-year retention; audit `EVT-J171-EVIDENCE-WORM-WRITTEN-004` |
| AC-J171-005 | Merkle anchor emitted with privileged-content tag; inclusion proof publicly available; payload not in proof; audit `EVT-J171-MERKLE-PRIVILEGED-ANCHOR-005` |
| AC-J171-006 | 14-day mediation period; 8 privileged channel exchanges; 2 mediation options proposed; complainant elects ombudsperson-mediated resolution (NOT formal escalation); audit `EVT-J171-MEDIATION-OPTIONS-006` |
| AC-J171-007 | Mediation outcome recorded: Aloysius receives a written reprimand from CEO via ombuds-channel; reassigned to a non-overlapping team; Priscilla receives written apology + 6-month workplace transfer support; audit `EVT-J171-MEDIATION-OUTCOME-007` |
| AC-J171-008 | Cedar deny coverage: enumeration attempts by Aloysius (3) + HR director (2) + IT admin (1) all denied; audit `EVT-J171-CEDAR-DENY-COVERAGE-008` |
| AC-J171-009 | Pack-manifest assertion: 6 packs active + cross-validated; audit `EVT-J171-PACK-MANIFEST-009` |
| AC-J171-010 | Mandatory-reporter exception NOT triggered; the code path is exercised in a deny-test only; audit `EVT-J171-MANDATORY-REPORTER-NOT-TRIGGERED-010` |
| AC-J171-011 | Observability emission redacted; no payload in metrics; only counters + redaction-flag set; audit `EVT-J171-OBSERVABILITY-REDACTED-011` |
| AC-J171-012 | Cantonese + Hokkien + Mandarin + Singapore-English + diacritic preservation byte-exact across all artifacts |

## Cross-references

- Persona dossier: `docs/personas/ombudsperson-felix-tan.md`
- MASTER-ROSTER §5.7 row 318
- Matrix §10 j171 recommendation
- Related: j05 (whistleblower anonymous ethics report), j129 (court warrant pierces personal tenant), j130 (auditor receives bribery via personal messenger), j127 (dual-tenant identity employee resigns)
- Pack roster: `packs/eu-wd-2019-1937-v2/`, `packs/sox-806-anti-retaliation-v3/`, `packs/kr-acrc-art-13/`, `packs/eeo-title-vii-2027/`, `packs/gdpr-article-9-special-category/`, `packs/ombudsperson-privilege-ioa-v2/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, the privileged channel remains enumerable only by Felix + Priscilla, the 10 evidence items are WORM-sealed in the EU privileged retention cell, the Merkle anchor proves inclusion without disclosing payload, the community-appeal handoff is recorded, the mediation outcome is sealed in audit-chain, and the mandatory-reporter exception is exercised in deny-tests without being triggered in the happy path.
