// Cedar policy — suppression list operations.
// ADR anchor: ADR-0201, ADR-0183.

// Insertion: any of (IP-009 bounce/complaint handler service
// account, tenant-admin, substrate-authority operator,
// GDPR-erasure pipeline).
permit (
    principal,
    action == Action::"comms-email:suppress-insert",
    resource
)
when {
    principal has role &&
    [ "comms-email-bounce-handler",
      "tenant-admin",
      "substrate-authority",
      "gdpr-erasure-pipeline" ].contains(principal.role)
};

// Removal: substrate-authority ONLY. Tenant-admin cannot remove
// regulatory-opt-out or GDPR-erasure entries.
permit (
    principal,
    action == Action::"comms-email:suppress-remove",
    resource
)
when {
    principal has role &&
    principal.role == "substrate-authority"
};

forbid (
    principal,
    action == Action::"comms-email:suppress-remove",
    resource
)
when {
    resource has reason &&
    (resource.reason == "GdprErasure" || resource.reason == "RegulatoryOptOut") &&
    principal has role &&
    principal.role != "substrate-authority"
};
