//! Shell capability registry — the ADR-0061 production registry sourced from
//! the locked shell-BFF contract family (`shared-platform-contracts-kernel::shell_bff`).
//!
//! Module visibility is deny-by-default: a module card renders only when the
//! operator context carries the capability's required PDP action. The full
//! registry never ships to the browser as a catalog — the server derives the
//! permitted envelope per context and serializes only that.
//!
//! Context grants are expressed as contract-source data in this crate so the
//! shell, SSR endpoint, and token broker all consume one validated source; a
//! future remote BFF adapter must preserve this same contract shape.

use std::collections::BTreeSet;

use shared_platform_contracts_kernel::shell_bff::{
    CapabilityRegistryEntry, ModuleRouteRegistration, NavigationSurface, validate_registry,
};

use crate::render_envelope::{ModuleCard, OperatorContext};

const SHELL_CONTRACT_SOURCE_ID: &str = "shell-bff-contract-source:v1";
const SHELL_CONTRACT_AUTHORITY: &str =
    "ADR-0393 production portal-shell + shared-platform-contracts-kernel::shell_bff";

/// One production module surface: the locked-contract registry row plus the
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

const PRODUCTION_MODULES: &[RegisteredModule] = &[
    RegisteredModule {
        capability_id: "tenant-admin",
        display_name: "Tenant Admin",
        module_id: "tenancy",
        required_action: "tenancy.administer",
        route_prefix: "/tenancy",
        upstream_service: "tenancy",
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
        upstream_service: "cloud-compute",
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
        upstream_service: "cloud-network",
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
        upstream_service: "finops",
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
        upstream_service: "audit",
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
        upstream_service: "workspace",
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
        upstream_service: "accounting",
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
        upstream_service: "human-resources",
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
        upstream_service: "approvals",
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
        upstream_service: "workflow-studio",
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
        upstream_service: "clinical",
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
        upstream_service: "patient-schedule",
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
        upstream_service: "care-workflows",
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
        upstream_service: "secure-messenger",
        group: "Healthcare",
        description: "Team communication with care-context labels",
        action_label: "Open messages",
    },
];

#[derive(Clone, Copy, Debug)]
pub struct ContextActionGrant {
    context: OperatorContext,
    actions: &'static [&'static str],
}

const CONTEXT_ACTION_GRANTS: &[ContextActionGrant] = &[
    ContextActionGrant {
        context: OperatorContext::TenantAdmin,
        actions: &[
            "tenancy.administer",
            "compute.operate",
            "network.operate",
            "finops.review",
            "workflow.design",
            "audit.inspect",
        ],
    },
    ContextActionGrant {
        context: OperatorContext::CorporateOffice,
        actions: &[
            "workspace.use",
            "accounting.close",
            "hr.operate",
            "approvals.review",
            "workflow.design",
        ],
    },
    ContextActionGrant {
        context: OperatorContext::HealthcareClinician,
        actions: &[
            "care.home",
            "care.schedule",
            "care.workflows",
            "care.message",
            "workflow.design",
        ],
    },
];

/// Production shell-BFF contract source consumed by the shell crate.
///
/// This intentionally centralizes the registry rows, route registrations, and
/// per-context grants behind the same locked contract types so callers do not
/// depend on a separate UI catalog.
#[derive(Clone, Debug)]
pub struct ShellContractSource {
    source_id: &'static str,
    authority: &'static str,
    entries: Vec<CapabilityRegistryEntry>,
    routes: Vec<ModuleRouteRegistration>,
    context_grants: &'static [ContextActionGrant],
}

impl ShellContractSource {
    pub fn source_id(&self) -> &'static str {
        self.source_id
    }

    pub fn authority(&self) -> &'static str {
        self.authority
    }

    pub fn entries(&self) -> &[CapabilityRegistryEntry] {
        &self.entries
    }

    pub fn routes(&self) -> &[ModuleRouteRegistration] {
        &self.routes
    }

    pub fn granted_actions(&self, context: OperatorContext) -> BTreeSet<&'static str> {
        grants_for(self.context_grants, context)
    }
}

/// Resolve a context's granted actions directly from static grant data, with no
/// registry allocation. `permitted_module_cards` (SSR render path) uses this so
/// it never builds the full `ShellContractSource` just to read the grant set.
fn context_action_grants(context: OperatorContext) -> BTreeSet<&'static str> {
    grants_for(CONTEXT_ACTION_GRANTS, context)
}

fn grants_for(grants: &[ContextActionGrant], context: OperatorContext) -> BTreeSet<&'static str> {
    grants
        .iter()
        .find(|grant| grant.context == context)
        .map(|grant| grant.actions.iter().copied().collect())
        .unwrap_or_default()
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
    let source = production_shell_contract_source();
    (source.entries().to_vec(), source.routes().to_vec())
}

/// The production shell-BFF contract source in locked-contract form.
pub fn production_shell_contract_source() -> ShellContractSource {
    let entries = PRODUCTION_MODULES
        .iter()
        .map(|module| CapabilityRegistryEntry {
            capability_id: module.capability_id.to_owned(),
            display_name: module.display_name.to_owned(),
            module_id: module.module_id.to_owned(),
            required_action: module.required_action.to_owned(),
            navigation_surface: NavigationSurface::PrimaryNav,
        })
        .collect();
    let routes = PRODUCTION_MODULES
        .iter()
        .map(|module| ModuleRouteRegistration {
            module_id: module.module_id.to_owned(),
            route_prefix: module.route_prefix.to_owned(),
            upstream_service: module.upstream_service.to_owned(),
            capability_ids: vec![module.capability_id.to_owned()],
        })
        .collect();
    ShellContractSource {
        source_id: SHELL_CONTRACT_SOURCE_ID,
        authority: SHELL_CONTRACT_AUTHORITY,
        entries,
        routes,
        context_grants: CONTEXT_ACTION_GRANTS,
    }
}

/// Module cards the given context is permitted to see, derived deny-by-default
/// from the capability registry: no grant, no card — never a greyed-out one.
pub fn permitted_module_cards(context: OperatorContext) -> Vec<ModuleCard> {
    let granted = context_action_grants(context);
    PRODUCTION_MODULES
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
    fn production_contract_source_is_the_registry_authority() {
        let source = production_shell_contract_source();
        validate_registry(source.entries(), source.routes())
            .expect("production shell contract source must satisfy the locked contract");

        assert_eq!(source.source_id(), "shell-bff-contract-source:v1");
        assert!(
            source
                .authority()
                .contains("shared-platform-contracts-kernel::shell_bff")
        );
        assert_eq!(source.entries().len(), source.routes().len());
    }

    #[test]
    fn every_granted_action_resolves_to_a_registered_capability() {
        let source = production_shell_contract_source();
        let entries = source.entries();
        let known: BTreeSet<_> = entries
            .iter()
            .map(|entry| entry.required_action.clone())
            .collect();
        for context in OperatorContext::ALL {
            for action in source.granted_actions(context) {
                assert!(known.contains(action), "{action} grants nothing registered");
            }
        }
    }

    #[test]
    fn every_context_grants_at_least_one_action() {
        // Deny-by-default must not silently degrade into deny-everything: a
        // context added to OperatorContext::ALL without a grant entry would
        // render a blank shell, and the resolves-to-registered test above would
        // pass vacuously. Require every context to carry a non-empty grant set.
        let source = production_shell_contract_source();
        for context in OperatorContext::ALL {
            assert!(
                !source.granted_actions(context).is_empty(),
                "{context:?} grants no actions; it would render an empty shell"
            );
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
