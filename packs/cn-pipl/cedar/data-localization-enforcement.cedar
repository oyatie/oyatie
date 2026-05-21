// CN-PIPL-2021 Cedar Fragment: data-localization-enforcement
// fragment_id: pack-cn-pipl-2021-data-localization-enforcement
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Article 40 — CIIOs and processors exceeding CAC volume threshold must store
//             personal information within mainland China; cross-border transfer requires CAC
//             security assessment or approved pathway.
//
// This fragment enforces:
//   1. Tenant data placement is forbidden outside cn-pipl-eligible cells.
//   2. Any cell-migration action that would move PI_CN_PIPL data out of mainland-China cells is denied.
//   3. Audit event CnPiplDataLocalizationDeny is emitted on every deny decision.
//
// Hyperscaler precedent:
//   AWS China (operated by Sinnet/NWCD in Beijing/Ningxia) and Alibaba Cloud (mainland regions)
//   enforce the same constraint via IAM SCPs that refuse CreateBucket/PutObject calls
//   targeting non-CN endpoints when the customer account is registered under their ICP license.

// -----------------------------------------------------------------------
// Namespace
// -----------------------------------------------------------------------
namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Entity types (imported from substrate schemas; declared here for clarity)
// -----------------------------------------------------------------------
// Cell: represents a deployment cell with its certification levels.
// Tenant: represents a tenant account with installed packs.
// Action: platform actions gated by Cedar.

// -----------------------------------------------------------------------
// Rule 1: Forbid tenant placement in any cell NOT carrying cn-pipl-eligible.
//
// A tenant with CN-PIPL-2021 installed attempting to activate in a cell that
// does not hold the "cn-pipl-eligible" certification level is denied.
// -----------------------------------------------------------------------
forbid(
  principal is oyatie::Tenant,
  action == oyatie::Action::"cell::activate",
  resource is oyatie::Cell
)
when {
  principal.installed_packs.contains("CN-PIPL-2021") &&
  !resource.certification_levels.contains("cn-pipl-eligible")
}
advice {
  "reason": "CN-PIPL-2021 pack requires cn-pipl-eligible cell certification per PIPL Article 40 and CAC Security Assessment Measures 2022. Cell does not hold this certification.",
  "audit_event_class": "CnPiplDataLocalizationDeny",
  "regulation_reference": "PIPL Article 40; CAC Measures on Security Assessment for Cross-Border Data Transfer, effective 2022-09-01"
};

// -----------------------------------------------------------------------
// Rule 2: Forbid data-plane cell migration that would egress PI_CN_PIPL
//         data to a non-CN cell.
//
// The cell::migrate action carries context attribute target_cell. If the
// target cell is not cn-pipl-eligible, migration of CN-PIPL tenants is denied.
// -----------------------------------------------------------------------
forbid(
  principal is oyatie::Tenant,
  action == oyatie::Action::"cell::migrate",
  resource is oyatie::Cell
)
when {
  principal.installed_packs.contains("CN-PIPL-2021") &&
  context.target_cell_certification_levels.contains("cn-pipl-eligible") == false
}
advice {
  "reason": "Cannot migrate CN-PIPL-2021 tenant to a cell that is not cn-pipl-eligible. Cross-border transfer of PI_CN_PIPL data requires CAC security assessment first.",
  "audit_event_class": "CnPiplDataLocalizationDeny",
  "regulation_reference": "PIPL Article 38; PIPL Article 40; CAC cross-border transfer measures 2022"
};

// -----------------------------------------------------------------------
// Rule 3: Permit intra-CN data-plane operations for cn-pipl-eligible cells.
//
// Explicitly permit so that intra-CN operations are not accidentally caught
// by broader deny rules from other packs.
// -----------------------------------------------------------------------
permit(
  principal is oyatie::Tenant,
  action in [
    oyatie::Action::"cell::activate",
    oyatie::Action::"cell::read",
    oyatie::Action::"cell::write"
  ],
  resource is oyatie::Cell
)
when {
  principal.installed_packs.contains("CN-PIPL-2021") &&
  resource.certification_levels.contains("cn-pipl-eligible")
}
advice {
  "audit_event_class": "CnPiplDataLocalizationPermit",
  "regulation_reference": "PIPL Article 40 satisfied: cell is cn-pipl-eligible"
};
