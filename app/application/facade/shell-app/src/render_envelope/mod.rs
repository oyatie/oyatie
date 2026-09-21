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

#[cfg(any(feature = "ssr", test))]
pub fn server_derived_envelope(context: OperatorContext) -> TenantRenderEnvelope {
    permitted_envelope_snapshot(context)
}

#[cfg(any(feature = "ssr", test))]
pub fn permitted_envelope_snapshot(context: OperatorContext) -> TenantRenderEnvelope {
    match context {
        OperatorContext::TenantAdmin => tenant_admin_envelope(),
        OperatorContext::CorporateOffice => corporate_office_envelope(),
        OperatorContext::HealthcareClinician => healthcare_clinician_envelope(),
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatorContext, server_derived_envelope};

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
