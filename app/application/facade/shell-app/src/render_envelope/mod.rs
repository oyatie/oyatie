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

/// The anonymous shell serves a preview; it never consults an operator source.
#[cfg(any(feature = "ssr", test))]
pub fn server_derived_envelope(context: OperatorContext) -> TenantRenderEnvelope {
    let mut envelope = match context {
        OperatorContext::TenantAdmin => tenant_admin_envelope(),
        OperatorContext::CorporateOffice => corporate_office_envelope(),
        OperatorContext::HealthcareClinician => healthcare_clinician_envelope(),
    };
    envelope.server_derivation_note = "Anonymous preview only: this is sample UI. No tenant or role was verified, and live service status was not queried.".to_owned();
    if context == OperatorContext::TenantAdmin {
        if let Some(card) = envelope
            .modules
            .iter_mut()
            .find(|card| card.name == "Ontology")
        {
            card.description
                .push_str(" · Preview only · live ontology status requires a verified session");
        }
    }
    envelope
}

/// Contract-test snapshot only; no served route accepts an Ontology source.
#[cfg(test)]
pub fn permitted_envelope_snapshot(
    context: OperatorContext,
    ontology: &dyn application_ontology_card::OntologyCardSource,
) -> TenantRenderEnvelope {
    let mut envelope = match context {
        OperatorContext::TenantAdmin => tenant_admin_envelope(),
        OperatorContext::CorporateOffice => corporate_office_envelope(),
        OperatorContext::HealthcareClinician => healthcare_clinician_envelope(),
    };
    if context == OperatorContext::TenantAdmin {
        tenant_admin::with_ontology_status(&mut envelope, ontology);
    }
    envelope
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
    fn every_anonymous_context_is_a_preview_in_the_api_and_ssr() {
        for context in OperatorContext::ALL {
            let envelope = server_derived_envelope(context);
            assert!(
                envelope
                    .server_derivation_note
                    .contains("Anonymous preview only"),
                "{context:?}"
            );
            assert!(
                envelope
                    .server_derivation_note
                    .contains("No tenant or role was verified"),
                "{context:?}"
            );
            let json = crate::app::render_envelope_json(context.id()).expect("known context");
            assert!(json.contains("Anonymous preview only"), "{context:?}");
            assert!(!json.contains(SENTINEL), "{context:?}");
        }
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let card = ontology_card(&envelope).expect("tenant admin sees the Ontology card");
        assert!(
            card.description.contains("Preview only"),
            "{}",
            card.description
        );
        assert!(
            !card
                .description
                .contains(FixtureOntologyCardSource::POLICY_VERSION),
            "{}",
            card.description
        );
        assert_eq!(
            envelope,
            server_derived_envelope(OperatorContext::TenantAdmin)
        );
        let html = crate::app::static_dashboard_html();
        assert!(html.contains("Anonymous preview only"), "{html}");
        assert!(html.contains("Preview only"), "{html}");
        assert!(!html.contains(SENTINEL), "{html}");
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
