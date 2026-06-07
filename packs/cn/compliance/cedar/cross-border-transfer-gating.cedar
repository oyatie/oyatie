// CN-PIPL-2021 Cedar Fragment: cross-border-transfer-gating
// fragment_id: pack-cn-pipl-2021-cross-border-transfer-gating
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Article 38 — cross-border transfer of personal information requires one of:
//   (a) CAC security assessment passed (mandatory for CIIOs and above-threshold processors),
//   (b) personal information protection certification from a CAC-accredited institution, OR
//   (c) standard contractual clauses (standard contract) filed with the provincial CAC office.
//
// Hyperscaler precedent:
//   AWS China (Sinnet/NWCD) and Microsoft Azure China (21Vianet) require tenants to complete
//   the CAC security assessment before enabling any cross-region data replication to non-CN
//   endpoints. Neither provider allows cross-border replication to be configured without the
//   attestation on file.

namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Rule 1: Forbid cross-border data transfer unless one of the three pathways
//         has been attested in the audit chain.
// -----------------------------------------------------------------------
forbid(
  principal is oyatie::Tenant,
  action == oyatie::Action::"data::cross-border-transfer",
  resource
)
when {
  principal.installed_packs.contains("CN-PIPL-2021") &&
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.cac_security_assessment_completed != true &&
  context.personal_information_protection_certification_held != true &&
  context.standard_contractual_clauses_filed != true
}
advice {
  "reason": "Cross-border transfer of PI_CN_PIPL or PI_CN_SENSITIVE data requires one of: (1) CAC security assessment completed and passed, (2) personal information protection certification from a CAC-accredited institution, or (3) standard contractual clauses filed with the provincial CAC office.",
  "audit_event_class": "CnPiplCrossBorderTransferDeny",
  "regulation_reference": "PIPL Article 38; CAC Measures on Security Assessment for Cross-Border Data Transfer (2022-09-01); CAC Standard Contract Measures (2023-06-01)"
};

// -----------------------------------------------------------------------
// Rule 2: Permit cross-border transfer when pathway attestation is on file.
// -----------------------------------------------------------------------
permit(
  principal is oyatie::Tenant,
  action == oyatie::Action::"data::cross-border-transfer",
  resource
)
when {
  principal.installed_packs.contains("CN-PIPL-2021") &&
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  (
    context.cac_security_assessment_completed == true ||
    context.personal_information_protection_certification_held == true ||
    context.standard_contractual_clauses_filed == true
  )
}
advice {
  "audit_event_class": "CnPiplCrossBorderTransferPermit",
  "regulation_reference": "PIPL Article 38 satisfied via attested pathway"
};
