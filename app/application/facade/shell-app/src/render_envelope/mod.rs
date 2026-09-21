#[cfg(any(feature = "ssr", test))]
mod builders;
#[cfg(any(feature = "ssr", test))]
mod corporate_office;
#[cfg(any(feature = "ssr", test))]
mod healthcare_clinician;
#[cfg(any(feature = "ssr", test))]
mod product_activity;
#[cfg(any(feature = "ssr", test))]
mod tenant_admin;
mod types;

pub use types::*;

#[cfg(any(feature = "ssr", test))]
use self::{
    corporate_office::corporate_office_envelope,
    healthcare_clinician::healthcare_clinician_envelope, tenant_admin::tenant_admin_envelope,
};

/// Composition root for every served envelope: the Ontology card's source is
/// chosen here and nowhere else.
#[cfg(any(feature = "ssr", test))]
pub fn server_derived_envelope(context: OperatorContext) -> TenantRenderEnvelope {
    permitted_envelope_snapshot(
        context,
        &application_ontology_card_fake::FixtureOntologyCardSource,
    )
}

/// Only the context that holds the Ontology grant consults `ontology`.
#[cfg(any(feature = "ssr", test))]
pub fn permitted_envelope_snapshot(
    context: OperatorContext,
    ontology: &dyn application_ontology_card::OntologyCardSource,
) -> TenantRenderEnvelope {
    match context {
        OperatorContext::TenantAdmin => tenant_admin_envelope(ontology),
        OperatorContext::CorporateOffice => corporate_office_envelope(),
        OperatorContext::HealthcareClinician => healthcare_clinician_envelope(),
    }
}

#[cfg(test)]
mod tests {
    use application_ontology_card::{OntologyCardError, OntologyCardFacts, OntologyCardSource};
    use application_ontology_card_fake::FixtureOntologyCardSource;

    use super::{
        ModuleCard, OperatorContext, TenantRenderEnvelope, permitted_envelope_snapshot,
        server_derived_envelope,
    };

    const SENTINEL: &str = "sentinel-policy-9f3e";

    struct Sentinel;

    impl OntologyCardSource for Sentinel {
        fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
            Ok(OntologyCardFacts {
                policy_version: SENTINEL.to_owned(),
                served_tenants: 41,
                projection_lag: 5,
                poisoned_entries: 2,
            })
        }
    }

    struct Refusing;

    impl OntologyCardSource for Refusing {
        fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
            Err(OntologyCardError::Unavailable("no listener".to_owned()))
        }
    }

    fn ontology_card(envelope: &TenantRenderEnvelope) -> Option<&ModuleCard> {
        envelope.modules.iter().find(|card| card.name == "Ontology")
    }

    #[test]
    fn tenant_admin_ontology_card_numbers_come_through_the_port() {
        let envelope = permitted_envelope_snapshot(OperatorContext::TenantAdmin, &Sentinel);
        let card = ontology_card(&envelope).expect("tenant admin sees the Ontology card");
        assert!(card.description.contains(SENTINEL), "{}", card.description);
        assert!(
            card.description.contains("41 tenants"),
            "{}",
            card.description
        );
        assert!(card.description.contains("lag 5"), "{}", card.description);
        assert!(
            card.description.contains("poisoned 2"),
            "{}",
            card.description
        );
        for sibling in envelope
            .modules
            .iter()
            .filter(|card| card.name != "Ontology")
        {
            assert!(
                !sibling.description.contains(SENTINEL),
                "{} carries Ontology facts",
                sibling.name
            );
        }
    }

    #[test]
    fn contexts_without_the_ontology_grant_never_render_port_facts() {
        for context in [
            OperatorContext::CorporateOffice,
            OperatorContext::HealthcareClinician,
        ] {
            let envelope = permitted_envelope_snapshot(context, &Sentinel);
            assert!(ontology_card(&envelope).is_none(), "{context:?}");
            let json = serde_json::to_string(&envelope).expect("envelope serializes");
            assert!(!json.contains(SENTINEL), "{context:?} leaked port facts");
            assert!(
                !json.contains("41 tenants"),
                "{context:?} leaked port facts"
            );
        }
    }

    #[test]
    fn a_refusing_source_keeps_the_card_and_says_the_numbers_are_unavailable() {
        let envelope = permitted_envelope_snapshot(OperatorContext::TenantAdmin, &Refusing);
        let card = ontology_card(&envelope).expect("the grant, not the source, shows the card");
        assert!(
            card.description.contains("unavailable: no listener"),
            "{}",
            card.description
        );
        assert!(
            !card.description.contains("tenants"),
            "{}",
            card.description
        );
        assert!(
            !card.description.contains("policy "),
            "{}",
            card.description
        );
    }

    #[test]
    fn the_served_envelope_is_wired_to_the_fixture_fake() {
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let card = ontology_card(&envelope).expect("tenant admin sees the Ontology card");
        assert!(
            card.description
                .contains(FixtureOntologyCardSource::POLICY_VERSION),
            "{}",
            card.description
        );
        assert_eq!(
            envelope,
            server_derived_envelope(OperatorContext::TenantAdmin)
        );
    }

    #[test]
    fn regulated_care_surfaces_are_absent_from_unaccredited_contexts() {
        for context in [
            OperatorContext::TenantAdmin,
            OperatorContext::CorporateOffice,
        ] {
            let envelope = server_derived_envelope(context);
            let surface_names = envelope
                .modules
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>()
                .join("|");

            assert!(!envelope.accreditation.healthcare_enabled);
            assert!(!surface_names.contains("Patient"));
            assert!(!surface_names.contains("Clinical"));
            assert!(!surface_names.contains("Care Workflows"));
        }
    }

    #[test]
    fn accredited_healthcare_context_receives_care_modules() {
        let envelope = server_derived_envelope(OperatorContext::HealthcareClinician);
        let surface_names = envelope
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>()
            .join("|");

        assert!(envelope.accreditation.healthcare_enabled);
        assert!(surface_names.contains("Clinical Home"));
        assert!(surface_names.contains("Patient Schedule"));
        assert!(surface_names.contains("Care Workflows"));
    }

    #[test]
    fn every_context_has_daily_dashboard_primitives() {
        for context in OperatorContext::ALL {
            let envelope = server_derived_envelope(context);

            assert!(!envelope.daily_tasks.is_empty(), "{context:?} tasks");
            assert!(!envelope.schedule.is_empty(), "{context:?} schedule");
            assert!(!envelope.messages.is_empty(), "{context:?} messages");
            assert!(!envelope.community.is_empty(), "{context:?} community");
            assert!(!envelope.approvals.is_empty(), "{context:?} approvals");
            assert!(!envelope.modules.is_empty(), "{context:?} modules");
            assert!(
                envelope.workflow.nodes.len() >= 4,
                "{context:?} workflow nodes"
            );
        }
    }

    #[test]
    fn permitted_envelope_snapshot_is_local_and_cloneable_for_island_state() {
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let cloned_for_island = envelope.clone();

        assert_eq!(cloned_for_island.context, OperatorContext::TenantAdmin);
        assert_eq!(cloned_for_island.modules, envelope.modules);
        assert_eq!(cloned_for_island.workflow.nodes, envelope.workflow.nodes);
    }
}
