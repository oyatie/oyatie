//! Shell capability registry — the ADR-0061 registry seeded from the locked
//! shell-BFF contract family (`oya-shared-platform-contracts-kernel::shell_bff`).
//!
//! Module visibility is deny-by-default: a module card renders only when the
//! operator context carries the capability's required PDP action. The full
//! registry never ships to the browser as a catalog — the server derives the
//! permitted envelope per context and serializes only that.
//!
//! Until the live shell-BFF service lands (G05/G06 fan-in), the action grants
//! per context come from a transitional in-process table behind the same
//! contract types; the registry shape itself is the locked contract.

use std::collections::BTreeSet;

use oya_shared_platform_contracts_kernel::shell_bff::{
    CapabilityRegistryEntry, ModuleRouteRegistration, NavigationSurface, validate_registry,
};

use crate::render_envelope::{ModuleCard, OperatorContext};

/// One registered module surface: the locked-contract registry row plus the
/// shell display metadata used to render its module card.
struct RegisteredModule {
    capability_id: &'static str,
    display_name: &'static str,
    module_id: &'static str,
    required_action: &'static str,
    route_prefix: &'static str,
    upstream_service: &'static str,
    group: &'static str,
    description: &'static str,
    action_label: &'static str,
}

const REGISTERED_MODULES: &[RegisteredModule] = &[
    RegisteredModule {
        capability_id: "tenant-admin",
        display_name: "Tenant Admin",
        module_id: "tenancy",
        required_action: "tenancy.administer",
        route_prefix: "/tenancy",
        upstream_service: "oya-tenancy",
        group: "Control",
        description: "Users, roles, packs, residency, module enablement",
        action_label: "Review posture",
    },
    RegisteredModule {
        capability_id: "cloud-compute",
        display_name: "Cloud Compute",
        module_id: "cloud-compute",
        required_action: "compute.operate",
        route_prefix: "/cloud-compute",
        upstream_service: "oya-cloud-compute",
        group: "Cloud",
        description: "VMs, functions, Kubernetes workloads, and runtime tiers",
        action_label: "Open compute",
    },
    RegisteredModule {
        capability_id: "cloud-network",
        display_name: "Cloud Network",
        module_id: "cloud-network",
        required_action: "network.operate",
        route_prefix: "/cloud-network",
        upstream_service: "oya-cloud-network",
        group: "Cloud",
        description: "VPC, DNS, load balancing, ingress posture",
        action_label: "Open network",
    },
    RegisteredModule {
        capability_id: "finops",
        display_name: "FinOps",
        module_id: "finops",
        required_action: "finops.review",
        route_prefix: "/finops",
        upstream_service: "oya-finops",
        group: "Operations",
        description: "Cost allocation, budgets, sustainability views",
        action_label: "Review spend",
    },
    RegisteredModule {
        capability_id: "audit-chain",
        display_name: "Audit Chain",
        module_id: "audit",
        required_action: "audit.inspect",
        route_prefix: "/audit",
        upstream_service: "oya-audit",
        group: "Trust",
        description: "Sealed evidence and policy event review",
        action_label: "Inspect evidence",
    },
    RegisteredModule {
        capability_id: "work-home",
        display_name: "Work Home",
        module_id: "workspace",
        required_action: "workspace.use",
        route_prefix: "/workspace",
        upstream_service: "oya-workspace",
        group: "Daily",
        description: "Tasks, calendar, mail, messenger, and approvals",
        action_label: "Open home",
    },
    RegisteredModule {
        capability_id: "accounting",
        display_name: "Accounting",
        module_id: "accounting",
        required_action: "accounting.close",
        route_prefix: "/accounting",
        upstream_service: "oya-accounting",
        group: "Corporate",
        description: "Invoices, close tasks, budgets, and exceptions",
        action_label: "Review close",
    },
    RegisteredModule {
        capability_id: "human-resources",
        display_name: "Human Resources",
        module_id: "human-resources",
        required_action: "hr.operate",
        route_prefix: "/human-resources",
        upstream_service: "oya-human-resources",
        group: "Corporate",
        description: "Onboarding, policy acknowledgements, and payroll workflows",
        action_label: "Open HR",
    },
    RegisteredModule {
        capability_id: "approvals",
        display_name: "Approvals",
        module_id: "approvals",
        required_action: "approvals.review",
        route_prefix: "/approvals",
        upstream_service: "oya-approvals",
        group: "Workflow",
        description: "Plain-language approvals with policy context",
        action_label: "Review queue",
    },
    RegisteredModule {
        capability_id: "workflow-studio",
        display_name: "Workflow Studio",
        module_id: "workflow-studio",
        required_action: "workflow.design",
        route_prefix: "/workflow-studio",
        upstream_service: "oya-workflow-studio",
        group: "No-code",
        description: "Design approvals and operating workflows safely",
        action_label: "Open studio",
    },
    RegisteredModule {
        capability_id: "clinical-home",
        display_name: "Clinical Home",
        module_id: "clinical",
        required_action: "care.home",
        route_prefix: "/clinical",
        upstream_service: "oya-clinical",
        group: "Healthcare",
        description: "Care tasks, visits, and secure team messages",
        action_label: "Open home",
    },
    RegisteredModule {
        capability_id: "patient-schedule",
        display_name: "Patient Schedule",
        module_id: "patient-schedule",
        required_action: "care.schedule",
        route_prefix: "/patient-schedule",
        upstream_service: "oya-patient-schedule",
        group: "Healthcare",
        description: "Visit flow with compliance-safe placeholders",
        action_label: "Review schedule",
    },
    RegisteredModule {
        capability_id: "care-workflows",
        display_name: "Care Workflows",
        module_id: "care-workflows",
        required_action: "care.workflows",
        route_prefix: "/care-workflows",
        upstream_service: "oya-care-workflows",
        group: "Healthcare",
        description: "Accredited workflow templates for care coordination",
        action_label: "Open workflows",
    },
    RegisteredModule {
        capability_id: "secure-messenger",
        display_name: "Secure Messenger",
        module_id: "secure-messenger",
        required_action: "care.message",
        route_prefix: "/secure-messenger",
        upstream_service: "oya-secure-messenger",
        group: "Healthcare",
        description: "Team communication with care-context labels",
        action_label: "Open messages",
    },
    RegisteredModule {
        capability_id: "customer-support-advisory",
        display_name: "Customer Support",
        module_id: "support-advisory",
        required_action: "support.view",
        route_prefix: "/support",
        upstream_service: "oya-support-advisory",
        group: "Operations",
        description: "Support cases, diagnostic bundles, and trusted-advisor recommendations",
        action_label: "Open support",
    },
];

/// PDP actions granted to each operator context.
///
/// Transitional in-process grant table (ADR-0510 pattern): the shape a live
/// PDP/BFF answer takes — a set of allowed actions for the principal — so the
/// adapter swap at G05/G06 fan-in does not change this module's callers.
fn granted_actions(context: OperatorContext) -> BTreeSet<&'static str> {
    match context {
        OperatorContext::TenantAdmin => [
            "tenancy.administer",
            "compute.operate",
            "network.operate",
            "finops.review",
            "workflow.design",
            "audit.inspect",
            "support.view",
        ]
        .into_iter()
        .collect(),
        OperatorContext::CorporateOffice => [
            "workspace.use",
            "accounting.close",
            "hr.operate",
            "approvals.review",
            "workflow.design",
        ]
        .into_iter()
        .collect(),
        OperatorContext::HealthcareClinician => [
            "care.home",
            "care.schedule",
            "care.workflows",
            "care.message",
            "workflow.design",
        ]
        .into_iter()
        .collect(),
    }
}

/// Per-context display copy for capabilities whose card text differs by
/// context (the registry row itself is context-invariant).
fn contextual_copy(
    context: OperatorContext,
    capability_id: &str,
) -> Option<(&'static str, &'static str)> {
    match (context, capability_id) {
        (OperatorContext::CorporateOffice, "workflow-studio") => {
            Some(("Draft team workflows from templates", "Draft workflow"))
        }
        (OperatorContext::HealthcareClinician, "workflow-studio") => {
            Some(("Draft safe care coordination workflows", "Draft care flow"))
        }
        _ => None,
    }
}

/// The full registry in locked-contract form.
pub fn capability_registry() -> (Vec<CapabilityRegistryEntry>, Vec<ModuleRouteRegistration>) {
    let entries = REGISTERED_MODULES
        .iter()
        .map(|module| CapabilityRegistryEntry {
            capability_id: module.capability_id.to_owned(),
            display_name: module.display_name.to_owned(),
            module_id: module.module_id.to_owned(),
            required_action: module.required_action.to_owned(),
            navigation_surface: NavigationSurface::PrimaryNav,
        })
        .collect();
    let routes = REGISTERED_MODULES
        .iter()
        .map(|module| ModuleRouteRegistration {
            module_id: module.module_id.to_owned(),
            route_prefix: module.route_prefix.to_owned(),
            upstream_service: module.upstream_service.to_owned(),
            capability_ids: vec![module.capability_id.to_owned()],
        })
        .collect();
    (entries, routes)
}

/// Module cards the given context is permitted to see, derived deny-by-default
/// from the capability registry: no grant, no card — never a greyed-out one.
pub fn permitted_module_cards(context: OperatorContext) -> Vec<ModuleCard> {
    let granted = granted_actions(context);
    REGISTERED_MODULES
        .iter()
        .filter(|module| granted.contains(module.required_action))
        .map(|module| {
            let (description, action_label) = contextual_copy(context, module.capability_id)
                .unwrap_or((module.description, module.action_label));
            ModuleCard {
                name: module.display_name.to_owned(),
                group: module.group.to_owned(),
                description: description.to_owned(),
                action_label: action_label.to_owned(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_satisfies_locked_contract_invariants() {
        let (entries, routes) = capability_registry();
        validate_registry(&entries, &routes).expect("registry must satisfy the locked contract");
    }

    #[test]
    fn every_granted_action_resolves_to_a_registered_capability() {
        let (entries, _) = capability_registry();
        let known: BTreeSet<_> = entries
            .iter()
            .map(|entry| entry.required_action.clone())
            .collect();
        for context in OperatorContext::ALL {
            for action in granted_actions(context) {
                assert!(known.contains(action), "{action} grants nothing registered");
            }
        }
    }

    #[test]
    fn module_visibility_is_deny_by_default_across_contexts() {
        let admin_cards = permitted_module_cards(OperatorContext::TenantAdmin);
        let admin_names: Vec<_> = admin_cards.iter().map(|card| card.name.as_str()).collect();
        assert!(admin_names.contains(&"Tenant Admin"));
        assert!(admin_names.contains(&"Audit Chain"));
        assert!(
            !admin_names.contains(&"Clinical Home"),
            "unaccredited context must not receive healthcare capabilities"
        );

        let clinician_cards = permitted_module_cards(OperatorContext::HealthcareClinician);
        let clinician_names: Vec<_> = clinician_cards
            .iter()
            .map(|card| card.name.as_str())
            .collect();
        assert!(clinician_names.contains(&"Clinical Home"));
        assert!(
            !clinician_names.contains(&"Tenant Admin"),
            "clinician context must not receive tenancy administration"
        );
    }

    #[test]
    fn workflow_studio_copy_is_contextual_but_single_registry_row() {
        let (entries, _) = capability_registry();
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.capability_id == "workflow-studio")
                .count(),
            1
        );
        let corporate = permitted_module_cards(OperatorContext::CorporateOffice);
        let studio = corporate
            .iter()
            .find(|card| card.name == "Workflow Studio")
            .expect("corporate context includes Workflow Studio");
        assert_eq!(studio.action_label, "Draft workflow");
    }
}
