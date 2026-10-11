use super::*;

pub(super) fn catalog_group_slug(group: &str) -> &'static str {
    match group {
        "Control" => "control",
        "Cloud" => "cloud",
        "Operations" => "operations",
        "Trust" => "trust",
        "Corporate" => "corporate",
        "Daily" => "daily",
        "Workflow" => "workflow",
        "No-code" => "no-code",
        "Healthcare" => "healthcare",
        _ => "other",
    }
}

pub(super) fn catalog_state_for(group: &str, name: &str) -> &'static str {
    if name.contains("Audit") {
        "sealed"
    } else if group == "Cloud" && name.contains("Network") {
        "attention"
    } else if matches!(group, "Operations" | "Workflow") {
        "review"
    } else {
        "ready"
    }
}

pub(super) fn catalog_state_label(state: &str) -> &'static str {
    match state {
        "attention" => "attention",
        "review" => "review",
        "sealed" => "sealed",
        _ => "ready",
    }
}

pub(super) fn catalog_owner_for(group: &str, name: &str) -> &'static str {
    if name.contains("Workflow") || group == "No-code" {
        "Automation council"
    } else {
        match group {
            "Control" => "Tenant admin",
            "Cloud" => "Infrastructure operations",
            "Operations" => "FinOps lead",
            "Trust" => "Security reviewer",
            "Corporate" => "Corporate ops",
            "Daily" => "Work home owner",
            "Workflow" => "Approval owner",
            "Healthcare" => "Clinical ops",
            _ => "Module owner",
        }
    }
}

pub(super) fn catalog_owner_avatar(owner: &str) -> &'static str {
    match owner {
        "Tenant admin" => "TA",
        "Infrastructure operations" => "PO",
        "FinOps lead" => "FO",
        "Security reviewer" => "SR",
        "Corporate ops" => "CO",
        "Work home owner" => "WH",
        "Automation council" => "AC",
        "Approval owner" => "AO",
        "Clinical ops" => "CL",
        _ => "MO",
    }
}

pub(super) fn catalog_criticality_for(group: &str, name: &str) -> &'static str {
    if name.contains("Network") || name.contains("Tenant Admin") {
        "P0"
    } else if matches!(group, "Cloud" | "Trust" | "Operations") {
        "P1"
    } else {
        "P2"
    }
}

pub(super) fn catalog_route_for(name: &str) -> &'static str {
    if name.contains("Workflow") || name.contains("Approvals") {
        "#workflow-studio"
    } else if name.contains("Cloud") || name.contains("FinOps") {
        "#cloud-ops-cockpit"
    } else if name.contains("Audit") {
        "#audit-ledger"
    } else if name.contains("Human") || name.contains("Clinical") || name.contains("Patient") {
        "#identity-employees"
    } else {
        "#work-hub"
    }
}

pub(super) fn catalog_dependency_for(group: &str, name: &str) -> &'static str {
    if name.contains("FinOps") {
        "Ledger"
    } else if name.contains("Audit") {
        "Evidence"
    } else if name.contains("Workflow") || group == "Workflow" {
        "Messenger/Mail"
    } else if group == "Cloud" {
        "Cloud Ops"
    } else if matches!(group, "Corporate" | "Daily") {
        "Work hub"
    } else if group == "Healthcare" {
        "Care hub"
    } else {
        "Policy"
    }
}

pub(super) fn catalog_code_for(name: &str) -> String {
    let code = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .map(|part| part.to_ascii_uppercase())
        .collect::<Vec<_>>()
        .join("-");
    format!("OYATIE-{code}")
}
