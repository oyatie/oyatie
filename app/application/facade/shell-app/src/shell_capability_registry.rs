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
use crate::shell_context_grants::{
    CONTEXT_ACTION_GRANTS, ContextActionGrant, context_action_grants, grants_for,
};
use crate::shell_modules::PRODUCTION_MODULES;

const SHELL_CONTRACT_SOURCE_ID: &str = "shell-bff-contract-source:v1";
const SHELL_CONTRACT_AUTHORITY: &str =
    "ADR-0393 production portal-shell + shared-platform-contracts-kernel::shell_bff";

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
            admin_names.contains(&"Ontology"),
            "tenant admin holds foundry.ontology.use and must see the Ontology card"
        );
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

        for context in [
            OperatorContext::CorporateOffice,
            OperatorContext::HealthcareClinician,
        ] {
            let names: Vec<_> = permitted_module_cards(context)
                .into_iter()
                .map(|card| card.name)
                .collect();
            assert!(
                !names.iter().any(|name| name == "Ontology"),
                "{context:?} holds no foundry.ontology.use grant and must not see the Ontology card"
            );
        }
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
