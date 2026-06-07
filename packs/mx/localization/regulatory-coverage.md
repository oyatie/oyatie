---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-REGULATORY-COVERAGE
title: Mexico Regulatory Coverage
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LFPDPPP.pdf
  - https://www.diputados.gob.mx/LeyesBiblio/ref/lfpdppp.htm
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20Instituciones%20de%20Tecnolog%C3%ADa%20Financiera.pdf
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.gob.mx/cnsf/documentos/circular-unica-de-seguros-y-fianzas
  - https://www.gob.mx/cre/que-hacemos
  - https://www.ift.org.mx/
  - https://www.gob.mx/atdt
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LMTR.pdf
---

# Mexico Regulatory Coverage

This document maps Mexico legal and regulator sources to Oyatie controls.
It is a coverage matrix, not legal advice.
It records article ids so later machine-readable policy work does not invent citations.
It treats LFPDPPP as the private-sector privacy floor.
It treats sector regulators as overlays that may make the floor stricter.
It treats INAI-era material as transition evidence where the current statutory authority moved in 2025.

## Authority Citations

- MX-REG-AUTH-001: LFPDPPP Art. 1 defines the law's purpose for private parties that process personal data.


- MX-REG-AUTH-002: LFPDPPP Art. 2 excludes credit-information entities governed by their special law and household-only processing.


- MX-REG-AUTH-003: LFPDPPP Art. 3(I) defines aviso de privacidad as the notice available to the data subject from collection.


- MX-REG-AUTH-004: LFPDPPP Art. 3(IV) defines consent as a free, specific, and informed manifestation of will.


- MX-REG-AUTH-005: LFPDPPP Art. 3(V) defines personal data as information concerning an identified or identifiable person.


- MX-REG-AUTH-006: LFPDPPP Art. 3(VI) defines sensitive personal data through intimate-sphere and discrimination-risk effects.


- MX-REG-AUTH-007: LFPDPPP Art. 3(VII) defines ARCO rights as access, rectification, cancellation, and opposition.


- MX-REG-AUTH-008: LFPDPPP Art. 3(XVIII) defines transfer as communication of data to a person other than controller or processor.


- MX-REG-AUTH-009: LFPDPPP Art. 6 supplies the primary principle list for processing controls.


- MX-REG-AUTH-010: LFPDPPP Art. 8 makes consent the ordinary processing basis unless the law supplies an exception.


- MX-REG-AUTH-011: LFPDPPP Art. 9 requires express consent for sensitive personal data.


- MX-REG-AUTH-012: LFPDPPP Art. 10 lists cases where consent is not necessary, including law, public sources, disassociation, contract, emergency, health, and judicial resolution paths.


- MX-REG-AUTH-013: LFPDPPP Art. 14 ties notice duties to data-subject information before treatment.


- MX-REG-AUTH-014: LFPDPPP Art. 15 requires controllers to make privacy notices available.


- MX-REG-AUTH-015: LFPDPPP Art. 16 sets privacy notice content, including identity, purposes, rights mechanisms, transfers, and changes.


- MX-REG-AUTH-016: LFPDPPP Art. 17 addresses presentation timing and collection modes for the privacy notice.


- MX-REG-AUTH-017: LFPDPPP Art. 19 requires security measures that protect personal data against damage, loss, alteration, destruction, or unauthorized use, access, or treatment.


- MX-REG-AUTH-018: LFPDPPP Art. 20 requires prompt notification to affected holders when a security vulnerability materially affects patrimonial or moral rights.


- MX-REG-AUTH-019: LFPDPPP Art. 22 grants ARCO rights to data subjects.


- MX-REG-AUTH-020: LFPDPPP Art. 23 requires controllers to provide access to personal data and treatment details.


- MX-REG-AUTH-021: LFPDPPP Art. 24 allows rectification of inaccurate or incomplete personal data.


- MX-REG-AUTH-022: LFPDPPP Art. 25 allows cancellation subject to legal retention and blocking logic.


- MX-REG-AUTH-023: LFPDPPP Art. 26 recognizes opposition to processing where legally appropriate.


- MX-REG-AUTH-024: LFPDPPP Art. 36 governs domestic and international transfer communication to recipients.


- MX-REG-AUTH-025: LFPDPPP Art. 37 lists transfer cases where consent is not required.


## Activated Cedar Policies

- MX-REG-CEDAR-001: `mx_scope_lfpdppp_private_person` checks whether a private person or legal entity decides processing means and purpose.


- MX-REG-CEDAR-002: `mx_scope_household_exclusion` denies business use of the household-only exclusion under LFPDPPP Art. 2.


- MX-REG-CEDAR-003: `mx_scope_credit_information_exclusion` routes credit-information companies to special-law review rather than generic pack allow.


- MX-REG-CEDAR-004: `mx_processing_principles_gate` requires every processing activity to map to LFPDPPP Art. 6 principles.


- MX-REG-CEDAR-005: `mx_privacy_notice_content_gate` verifies notice fields required by LFPDPPP Arts. 15-16.


- MX-REG-CEDAR-006: `mx_privacy_notice_presentation_gate` verifies collection-channel timing under LFPDPPP Art. 17.


- MX-REG-CEDAR-007: `mx_consent_gate` verifies consent or LFPDPPP Art. 10 exception before processing.


- MX-REG-CEDAR-008: `mx_sensitive_express_consent_gate` verifies express consent for LFPDPPP Art. 9 sensitive data.


- MX-REG-CEDAR-009: `mx_security_measures_gate` requires administrative, technical, and physical controls before storing Mexico personal data.


- MX-REG-CEDAR-010: `mx_breach_material_rights_gate` requires a rights-impact decision under LFPDPPP Art. 20.


- MX-REG-CEDAR-011: `mx_arco_access_gate` routes access requests through identity proof and treatment-detail disclosure.


- MX-REG-CEDAR-012: `mx_arco_rectification_gate` routes correction requests through evidence of inaccuracy or incompleteness.


- MX-REG-CEDAR-013: `mx_arco_cancellation_gate` routes erasure requests through cancellation, blocking, and retention checks.


- MX-REG-CEDAR-014: `mx_arco_opposition_gate` routes objection requests through purpose and legal-basis review.


- MX-REG-CEDAR-015: `mx_transfer_notice_gate` ensures recipients receive and accept notice limitations under LFPDPPP Art. 36.


- MX-REG-CEDAR-016: `mx_transfer_exception_gate` verifies each no-consent transfer against LFPDPPP Art. 37.


- MX-REG-CEDAR-017: `mx_secretaria_authority_gate` tracks post-2025 authority functions under LFPDPPP Arts. 38-39.


- MX-REG-CEDAR-018: `mx_legacy_inai_transition_gate` permits historical INAI references only when tagged as legacy or transition evidence.


- MX-REG-CEDAR-019: `mx_cnbv_supervised_financial_gate` applies CNBV privacy and cybersecurity controls for banking and fintech tenants.


- MX-REG-CEDAR-020: `mx_cnsf_supervised_insurance_gate` applies CNSF privacy evidence for insurance, bonding, agents, adjusters, and regulatory reports.


- MX-REG-CEDAR-021: `mx_cre_energy_regulated_gate` applies energy-sector controls for permit, metering, user, billing, and reliability records.


- MX-REG-CEDAR-022: `mx_lmtr_telecom_gate` applies LMTR and telecom user-rights controls for networks, subscribers, devices, complaints, and traffic.


- MX-REG-CEDAR-023: `mx_telecom_transition_gate` requires IFT, CRT, ATDT, or SICT authority mapping before telecom automation.


- MX-REG-CEDAR-024: `mx_sector_overlay_strictest_gate` chooses stricter duties over generic LFPDPPP permissions.


- MX-REG-CEDAR-025: `mx_regulatory_claim_denial_gate` blocks marketing or compliance claims without article-level evidence.


## Data Model Deltas

- MX-REG-DATA-001: `mx_authority_citation.article_id` stores exact article ids such as `LFPDPPP-ART-20`.


- MX-REG-DATA-002: `mx_authority_citation.url` stores the official source URL, not a secondary summary.


- MX-REG-DATA-003: `mx_authority_citation.snapshot_date` stores the pack citation date.


- MX-REG-DATA-004: `mx_regulator.code` supports `SECRETARIA`, `LEGACY_INAI`, `CNBV`, `CNSF`, `CRE`, `IFT`, `CRT`, `ATDT`, and `SICT`.


- MX-REG-DATA-005: `mx_regulator.transition_state` distinguishes active authority, archived authority, and unclear authority.


- MX-REG-DATA-006: `mx_obligation.kind` supports notice, consent, security, rights, transfer, breach, sector-reporting, and supervisory-record types.


- MX-REG-DATA-007: `mx_obligation.trigger` stores the event that makes a rule applicable.


- MX-REG-DATA-008: `mx_obligation.denial_reason` stores the reason a pack action cannot proceed.


- MX-REG-DATA-009: `mx_data_category` includes ordinary, sensitive, financial, insurance, energy, telecom, device, traffic, and digital-identity classes.


- MX-REG-DATA-010: `mx_processing_principle_map` links a processing activity to LFPDPPP Art. 6 principles.


- MX-REG-DATA-011: `mx_notice_requirement` captures identity, purpose, options, means, transfers, procedure, and notice-change fields.


- MX-REG-DATA-012: `mx_consent_basis` captures consent mode or statutory exception.


- MX-REG-DATA-013: `mx_security_control_profile` captures control evidence for LFPDPPP Art. 19.


- MX-REG-DATA-014: `mx_breach_rights_impact` captures property-rights and moral-rights impact assessment.


- MX-REG-DATA-015: `mx_arco_right_type` enumerates access, rectification, cancellation, and opposition.


- MX-REG-DATA-016: `mx_transfer_case` stores consented, same-corporate, legal, public-interest, contract, medical, judicial, and other Art. 37 categories.


- MX-REG-DATA-017: `mx_cnbv_overlay_profile` stores banking, securities, fintech, outsourcing, electronic-means, and cybersecurity markers.


- MX-REG-DATA-018: `mx_cnsf_overlay_profile` stores policyholder, beneficiary, claim, agent, adjuster, actuary, and supervisory-report markers.


- MX-REG-DATA-019: `mx_cre_overlay_profile` stores electricity, hydrocarbons, gas, permit, metering, reliability, and user markers.


- MX-REG-DATA-020: `mx_telecom_overlay_profile` stores subscriber, device, portability, accessibility, complaint, traffic, network, and satellite markers.


- MX-REG-DATA-021: `mx_regulatory_claim` stores product claims that cite Mexico compliance.


- MX-REG-DATA-022: `mx_regulatory_claim.evidence_refs` links every claim to official URLs and article ids.


- MX-REG-DATA-023: `mx_regulatory_conflict` stores conflicts across sector and privacy obligations.


- MX-REG-DATA-024: `mx_regulatory_conflict.resolution` stores strictest-wins, legal-review, or deny outcome.


- MX-REG-DATA-025: `mx_pack_no_touch_scope` records that only `/packs/mx-localization/` is modified by this slice.


## API Contract Deltas

- MX-REG-API-001: `GET /localization/mx/regulatory-coverage` returns obligation rows grouped by authority.


- MX-REG-API-002: `GET /localization/mx/regulatory-coverage/lfpdppp` returns LFPDPPP article mappings.


- MX-REG-API-003: `GET /localization/mx/regulatory-coverage/cnbv` returns banking, securities, and fintech mappings.


- MX-REG-API-004: `GET /localization/mx/regulatory-coverage/cnsf` returns insurance and bonding mappings.


- MX-REG-API-005: `GET /localization/mx/regulatory-coverage/cre` returns energy-sector mappings.


- MX-REG-API-006: `GET /localization/mx/regulatory-coverage/telecom` returns IFT, CRT, ATDT, and LMTR mappings.


- MX-REG-API-007: `POST /localization/mx/regulatory-coverage/applicability` evaluates tenant facts against coverage triggers.


- MX-REG-API-008: `POST /localization/mx/regulatory-coverage/conflicts` evaluates strictest-rule conflicts.


- MX-REG-API-009: `POST /localization/mx/regulatory-coverage/article-check` validates that an obligation row has an article id.


- MX-REG-API-010: `POST /localization/mx/regulatory-coverage/source-check` validates that the cited URL is an official authority URL.


- MX-REG-API-011: `POST /localization/mx/regulatory-coverage/transition-check` validates regulator transition state.


- MX-REG-API-012: `POST /localization/mx/regulatory-coverage/claim-check` validates compliance claims before product publication.


- MX-REG-API-013: `GET /localization/mx/regulatory-coverage/claims/{claim_id}` returns claim evidence and denial state.


- MX-REG-API-014: `POST /localization/mx/regulatory-coverage/sector-profile` stores CNBV, CNSF, CRE, or telecom profile.


- MX-REG-API-015: `GET /localization/mx/regulatory-coverage/sector-profile/{tenant_id}` returns active sector overlays.


- MX-REG-API-016: `POST /localization/mx/regulatory-coverage/no-touch-check` verifies no other geography pack changed.


- MX-REG-API-017: `GET /localization/mx/regulatory-coverage/audit-events` returns ADR-0263 event-class mappings.


- MX-REG-API-018: `GET /localization/mx/regulatory-coverage/failure-modes` returns denial and review-required reasons.


- MX-REG-API-019: `POST /localization/mx/regulatory-coverage/evidence-refresh` creates a refresh task for authority URLs.


- MX-REG-API-020: `GET /localization/mx/regulatory-coverage/snapshot` returns the current authority snapshot.


- MX-REG-API-021: `POST /localization/mx/regulatory-coverage/legacy-inai-use` records a legacy INAI citation with transition notes.


- MX-REG-API-022: `POST /localization/mx/regulatory-coverage/telecom-transition-use` records an IFT, CRT, ATDT, or SICT citation.


- MX-REG-API-023: `POST /localization/mx/regulatory-coverage/sector-override` requires legal-review evidence before overriding generic pack behavior.


- MX-REG-API-024: `GET /localization/mx/regulatory-coverage/authority/{authority_code}` returns mapped obligations by authority code.


- MX-REG-API-025: `POST /localization/mx/regulatory-coverage/deny` records denied coverage decisions with evidence.


## Audit Event Additions (per ADR-0263)

- MX-REG-AUDIT-001: `MxRegulatoryCoverageEvaluated` records tenant facts, authority rows, and coverage result.


- MX-REG-AUDIT-002: `MxLfpdpppArticleMapped` records article id, obligation kind, and control id.


- MX-REG-AUDIT-003: `MxCnbvAuthorityMapped` records CNBV source, financial overlay, and cybersecurity marker.


- MX-REG-AUDIT-004: `MxCnsfAuthorityMapped` records CNSF source, insurance overlay, and treatment inventory.


- MX-REG-AUDIT-005: `MxCreAuthorityMapped` records CRE source, energy overlay, and regulated activity.


- MX-REG-AUDIT-006: `MxTelecomAuthorityMapped` records IFT, CRT, ATDT, SICT, or LMTR source.


- MX-REG-AUDIT-007: `MxAuthorityTransitionTagged` records legacy, active, archived, or unclear transition state.


- MX-REG-AUDIT-008: `MxRegulatoryConflictDetected` records conflicting authorities and affected workflow.


- MX-REG-AUDIT-009: `MxRegulatoryConflictResolved` records strictest-wins, legal-review, or deny outcome.


- MX-REG-AUDIT-010: `MxRegulatoryClaimSubmitted` records claim text, product surface, and authority refs.


- MX-REG-AUDIT-011: `MxRegulatoryClaimApproved` records evidence refs and approving role.


- MX-REG-AUDIT-012: `MxRegulatoryClaimDenied` records missing article, nonofficial URL, or stale authority reason.


- MX-REG-AUDIT-013: `MxSectorOverlayActivated` records sector code and triggering tenant facts.


- MX-REG-AUDIT-014: `MxSectorOverlayDenied` records missing license, missing classification, or missing authority.


- MX-REG-AUDIT-015: `MxAuthoritySnapshotRefreshed` records URLs checked, changed hashes, and review outcome.


- MX-REG-AUDIT-016: `MxLegacyInaiCitationUsed` records legacy INAI source, current authority mapping, and transition note.


- MX-REG-AUDIT-017: `MxTelecomTransitionCitationUsed` records IFT or CRT source plus ATDT or LMTR context.


- MX-REG-AUDIT-018: `MxNoTouchScopeVerified` records that no other geography pack was modified.


- MX-REG-AUDIT-019: `MxCoverageExportGenerated` records redacted export of obligation rows.


- MX-REG-AUDIT-020: `MxCoverageAuditRejected` records unsanitized payload rejection under ADR-0263.


- MX-REG-AUDIT-021: `MxAuthorityArticleMissing` records a failed citation row missing article id.


- MX-REG-AUDIT-022: `MxAuthorityUrlRejected` records a nonofficial citation URL rejection.


- MX-REG-AUDIT-023: `MxCoverageDenyIssued` records final denial for a workflow.


- MX-REG-AUDIT-024: `MxCoverageLegalReviewRequired` records review checkpoint and unresolved question.


- MX-REG-AUDIT-025: `MxCoverageSnapshotFrozen` records the authority snapshot used for release.


## Failure Modes

- MX-REG-FAIL-001: A coverage row without an article id cannot support runtime policy.


- MX-REG-FAIL-002: A coverage row without an official authority URL cannot support compliance claims.


- MX-REG-FAIL-003: A coverage row citing legacy INAI without transition context may misstate current enforcement.


- MX-REG-FAIL-004: A telecom row citing IFT without CRT, ATDT, SICT, or LMTR context may misstate current authority.


- MX-REG-FAIL-005: A CNBV row that treats fintech cybersecurity provisions as all-bank rules overstates applicability.


- MX-REG-FAIL-006: A CNBV row that ignores banking privacy and outsourcing controls understates risk.


- MX-REG-FAIL-007: A CNSF row that treats insurance supervisory reports as ordinary CRM data loses regulatory context.


- MX-REG-FAIL-008: A CRE row that treats metering records as anonymous by default loses identifiability context.


- MX-REG-FAIL-009: A telecom row that treats traffic or device data as nonpersonal by default loses subscriber linkage.


- MX-REG-FAIL-010: A public-sector privacy rule copied into private-sector LFPDPPP workflow may produce false duties.


- MX-REG-FAIL-011: A private-sector LFPDPPP rule copied into a government controller workflow may omit LGPDPPSO duties.


- MX-REG-FAIL-012: A privacy notice rule without transfer language cannot support Art. 36 transfer behavior.


- MX-REG-FAIL-013: A consent rule without sensitive-data branch cannot support Art. 9 processing.


- MX-REG-FAIL-014: An ARCO rule without identity verification cannot safely disclose personal data.


- MX-REG-FAIL-015: A breach rule without rights-impact assessment cannot decide notification under Art. 20.


- MX-REG-FAIL-016: A sector overlay without tenant facts may overblock unrelated tenants.


- MX-REG-FAIL-017: A sector overlay without strictest-duty resolution may underblock regulated tenants.


- MX-REG-FAIL-018: A product claim saying Mexico compliant without source refs must be denied.


- MX-REG-FAIL-019: A runtime bundle using stale authority snapshots must require refresh before activation.


- MX-REG-FAIL-020: A doc slice that touches another geography violates the slice boundary.


- MX-REG-FAIL-021: A generated script or templater would violate this author's constraints.


- MX-REG-FAIL-022: An audit event containing raw personal data violates ADR-0263.


- MX-REG-FAIL-023: A regulator transition row with `unknown` state cannot auto-approve.


- MX-REG-FAIL-024: A CNBV, CNSF, CRE, or telecom conflict resolved by generic allow must be rejected.


- MX-REG-FAIL-025: A legal-review-required row cannot be converted into allowed by a tenant admin alone.


## Worked Examples

- MX-REG-EXAMPLE-001: A marketing email workflow maps to LFPDPPP Arts. 6, 8, 14, 15, 16, and ARCO Arts. 22-28.


- MX-REG-EXAMPLE-002: A biometric login workflow maps to LFPDPPP Art. 9 sensitive-data express consent.


- MX-REG-EXAMPLE-003: A customer support transfer to a foreign vendor maps to LFPDPPP Arts. 36-37 and cross-border review.


- MX-REG-EXAMPLE-004: A credential leak maps to LFPDPPP Arts. 19-20 and CNBV security overlay if financial.


- MX-REG-EXAMPLE-005: A fintech platform incident maps to CNBV ITF cybersecurity provisions and privacy breach analysis.


- MX-REG-EXAMPLE-006: A bank outsourcing workflow maps to CNBV sector review before generic transfer approval.


- MX-REG-EXAMPLE-007: A policyholder claim workflow maps to CNSF data-protection inventory and sensitive-data consent.


- MX-REG-EXAMPLE-008: An actuarial report maps to CNSF supervisory treatment rather than generic analytics.


- MX-REG-EXAMPLE-009: A smart-meter workflow maps to CRE activity context and LFPDPPP identifiability review.


- MX-REG-EXAMPLE-010: An electric-vehicle charging workflow maps to CRE electromobility context and personal-location review.


- MX-REG-EXAMPLE-011: A mobile subscriber complaint maps to telecom user-rights, device, and privacy controls.


- MX-REG-EXAMPLE-012: A portability workflow maps to telecom user-rights and personal-data portability handling.


- MX-REG-EXAMPLE-013: A telecom accessibility workflow maps to IFT/CRT materials and user-rights evidence.


- MX-REG-EXAMPLE-014: A satellite-service workflow maps to LMTR Arts. 4-5 federal-jurisdiction classification.


- MX-REG-EXAMPLE-015: A Llave MX login integration maps to ATDT digital identity, public-sector boundary, and segregation controls.


- MX-REG-EXAMPLE-016: A regulator report referencing INAI maps to legacy citation and current authority transition note.


- MX-REG-EXAMPLE-017: A privacy notice missing transfer statements fails transfer preflight.


- MX-REG-EXAMPLE-018: A cancellation request tied to legal retention maps to blocking rather than immediate deletion.


- MX-REG-EXAMPLE-019: A complaint report containing customer names must use ADR-0263 redaction before audit export.


- MX-REG-EXAMPLE-020: A Mexico pack marketing page must display authority snapshot and avoid claiming sector authorization.


- MX-REG-EXAMPLE-021: A tenant with both CNBV and telecom operations receives both overlays and strictest conflict resolution.


- MX-REG-EXAMPLE-022: A tenant using household data for personal use remains out of scope, but enterprise reuse re-enters scope.


- MX-REG-EXAMPLE-023: A processor responding to controller instructions logs role evidence and does not invent controller purpose.


- MX-REG-EXAMPLE-024: A sector regulator request for records must be logged with authority code, article refs, and redaction profile.


- MX-REG-EXAMPLE-025: A denied compliance claim creates an audit row with missing article or unsupported authority as reason.


## Cross-References

- MX-REG-XREF-001: `README.md` defines the pack identity and activation posture.


- MX-REG-XREF-002: `data-residency-and-cross-border.md` expands Art. 36 and Art. 37 transfer handling.


- MX-REG-XREF-003: `consent-and-data-subject-rights.md` expands Arts. 8-10 and 22-28 handling.


- MX-REG-XREF-004: `breach-notification-and-incident-response.md` expands Arts. 19-20 handling.


- MX-REG-XREF-005: `sectoral-overlays.md` expands CNBV, CNSF, CRE, telecom, and ATDT overlays.


- MX-REG-XREF-006: ADR-0243 binds every coverage row to a Cedar enforcement point.


- MX-REG-XREF-007: ADR-0244 binds every coverage row to tenant and sub-scope context.


- MX-REG-XREF-008: ADR-0251 binds every coverage row to compliance-pack bundle mechanics.


- MX-REG-XREF-009: ADR-0263 binds every coverage row to audit event emission and scrubbing.


- MX-REG-XREF-010: LFPDPPP Art. 1 is the first coverage gate for private-sector processing.


- MX-REG-XREF-011: LFPDPPP Art. 2 is the exclusion gate.


- MX-REG-XREF-012: LFPDPPP Art. 3 is the definition gate.


- MX-REG-XREF-013: LFPDPPP Art. 6 is the principles gate.


- MX-REG-XREF-014: LFPDPPP Arts. 8-10 are the consent and exception gates.


- MX-REG-XREF-015: LFPDPPP Arts. 14-18 are the privacy-notice gates.


- MX-REG-XREF-016: LFPDPPP Arts. 19-20 are the security and breach gates.


- MX-REG-XREF-017: LFPDPPP Arts. 22-28 are the ARCO gates.


- MX-REG-XREF-018: LFPDPPP Arts. 36-37 are the transfer gates.


- MX-REG-XREF-019: LFPDPPP Arts. 38-39 are the current federal authority gates.


- MX-REG-XREF-020: CNBV official sources are sector overlays, not generic law replacement.


- MX-REG-XREF-021: CNSF official sources are sector overlays for insurance and bonding supervision.


- MX-REG-XREF-022: CRE official sources are sector overlays for regulated energy activities.


- MX-REG-XREF-023: IFT and CRT sources are telecom continuity and transition sources.


- MX-REG-XREF-024: ATDT sources are transformation-digital and telecom transition context.


- MX-REG-XREF-025: LMTR official text is the telecom statutory reference for post-2025 coverage.

