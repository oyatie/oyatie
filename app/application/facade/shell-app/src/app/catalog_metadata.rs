use super::*;

#[derive(Clone)]
pub(super) struct CatalogRecord {
    pub(super) id: &'static str,
    pub(super) module: ModuleCard,
    pub(super) state: &'static str,
    pub(super) owner: &'static str,
    pub(super) dependency: &'static str,
    pub(super) priority: &'static str,
    pub(super) destination: &'static str,
    pub(super) channel: &'static str,
}

impl CatalogRecord {
    pub(super) fn fields(&self) -> Vec<(String, String)> {
        [
            ("Capability", self.id),
            ("Module", &self.module.name),
            ("Category", &self.module.group),
            ("Description", &self.module.description),
            ("Action", &self.module.action_label),
            ("Sample health", catalog_state_label(self.state)),
            ("Sample owner", self.owner),
            ("Sample dependency", self.dependency),
            ("Sample priority", self.priority),
            ("Destination", self.destination),
            ("Channel", self.channel),
            (
                "Availability",
                if self.destination.is_empty() {
                    "Dedicated product unavailable; local sample inspection only"
                } else {
                    "Existing anonymous shell sample only; no live product operation"
                },
            ),
        ]
        .into_iter()
        .map(|(label, value)| (label.to_owned(), value.to_owned()))
        .collect()
    }
}

pub(super) fn catalog_record(module: &ModuleCard) -> Option<CatalogRecord> {
    let mut matches = crate::shell_modules::PRODUCTION_MODULES
        .iter()
        .filter(|entry| entry.display_name == module.name && entry.group == module.group);
    let entry = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let (destination, channel) = catalog_route_for(entry.capability_id);
    Some(CatalogRecord {
        id: entry.capability_id,
        module: module.clone(),
        state: catalog_state_for(&module.group, &module.name),
        owner: catalog_owner_for(&module.group, &module.name),
        dependency: catalog_dependency_for(&module.group, &module.name),
        priority: catalog_criticality_for(&module.group, &module.name),
        destination,
        channel,
    })
}

pub(super) fn catalog_route_for(id: &str) -> (&'static str, &'static str) {
    match id {
        "tenant-admin" => ("identity-panel-auth", ""),
        "cloud-compute" => ("resource-panel-inventory", ""),
        "cloud-network" => ("cockpit-panel-topology", ""),
        "finops" => ("cockpit-panel-finops", ""),
        "audit-chain" => ("resource-panel-audit", ""),
        "work-home" => ("daily-execution", ""),
        "mail" => ("work-hub", "Mail"),
        "accounting" => ("finance-panel-ledger", ""),
        "human-resources" => ("identity-panel-employees", ""),
        "approvals" => ("command-center-workbench", ""),
        "workflow-studio" => ("workflow-studio", ""),
        "foundry-ontology" => ("ontology-command-console", ""),
        _ => ("", ""),
    }
}

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
        "Data" => "data",
        _ => "other",
    }
}

pub(super) fn catalog_state_for(group: &str, name: &str) -> &'static str {
    if name == "Audit Chain" {
        "receipt"
    } else if name == "Cloud Network" {
        "attention"
    } else if matches!(group, "Operations" | "Workflow") {
        "review"
    } else {
        "sample"
    }
}

pub(super) fn catalog_state_label(state: &str) -> &'static str {
    match state {
        "attention" => "Attention example",
        "review" => "Review example",
        "receipt" => "Receipt example",
        _ => "Sample only",
    }
}

pub(super) fn catalog_owner_for(group: &str, name: &str) -> &'static str {
    if name.contains("Workflow") || group == "No-code" {
        return "Automation council";
    }
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
    if matches!(name, "Cloud Network" | "Tenant Admin") {
        "P0 example"
    } else if matches!(group, "Cloud" | "Trust" | "Operations") {
        "P1 example"
    } else {
        "P2 example"
    }
}

pub(super) fn catalog_dependency_for(group: &str, name: &str) -> &'static str {
    if name == "FinOps" {
        "Ledger"
    } else if name == "Audit Chain" {
        "Evidence"
    } else if name.contains("Workflow") || group == "Workflow" {
        "Messenger/Mail"
    } else if group == "Cloud" {
        "Cloud Ops"
    } else if matches!(group, "Corporate" | "Daily") {
        "Work hub"
    } else if group == "Healthcare" {
        "Care hub (unavailable)"
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
        .map(str::to_ascii_uppercase)
        .collect::<Vec<_>>()
        .join("-");
    format!("OYATIE-{code}")
}
