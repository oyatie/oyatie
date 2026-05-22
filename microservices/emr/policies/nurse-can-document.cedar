// Cedar policy — nurse.can-document
// Owning µservice: emr
// Owning BCs: note, vital, medication (administration)
// Authored Wave 15M-B 2026-05-21

// Permit RN / LPN to document vitals + medication administration + nursing notes
// for assigned patients during an active encounter.

permit (
    principal in [Group::"clinician.rn", Group::"clinician.lpn"],
    action in [
        Action::"emr.vital.record",
        Action::"emr.medication.administer",
        Action::"emr.note.draft",
        Action::"emr.note.autosave",
        Action::"emr.note.sign",
        Action::"emr.order.verify"
    ],
    resource
)
when {
    principal.tenant_id == resource.tenant_id &&
    context.care_team_member == true &&
    context.encounter_active == true &&
    context.purpose_of_use == "TREATMENT" &&
    context.session_assurance_level in ["AAL2", "AAL3"] &&
    "HIPAA-2024" in context.installed_packs
};

// Permit nurse to sign a note ONLY if they are the author
permit (
    principal,
    action == Action::"emr.note.sign",
    resource
)
when {
    principal.id == resource.author_id
};

// Forbid nurse from prescribing
forbid (
    principal in [Group::"clinician.rn", Group::"clinician.lpn"],
    action == Action::"emr.medication.prescribe",
    resource
);
