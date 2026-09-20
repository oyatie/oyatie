//! Per-context PDP action grants: which capability actions each operator
//! context carries. Deny-by-default: an action missing here renders no card.
//! This table is the seam a caller-verifier adapter replaces when grants come
//! from iam instead of contract-source data.

use std::collections::BTreeSet;

use crate::render_envelope::OperatorContext;

#[derive(Clone, Copy, Debug)]
pub struct ContextActionGrant {
    context: OperatorContext,
    actions: &'static [&'static str],
}

pub(crate) const CONTEXT_ACTION_GRANTS: &[ContextActionGrant] = &[
    ContextActionGrant {
        context: OperatorContext::TenantAdmin,
        actions: &[
            "tenancy.administer",
            "compute.operate",
            "network.operate",
            "finops.review",
            "workflow.design",
            "audit.inspect",
            "foundry.ontology.use",
        ],
    },
    ContextActionGrant {
        context: OperatorContext::CorporateOffice,
        actions: &[
            "workspace.use",
            "mail.use",
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

/// Resolve a context's granted actions directly from static grant data, with no
/// registry allocation. `permitted_module_cards` (SSR render path) uses this so
/// it never builds the full `ShellContractSource` just to read the grant set.
pub(crate) fn context_action_grants(context: OperatorContext) -> BTreeSet<&'static str> {
    grants_for(CONTEXT_ACTION_GRANTS, context)
}

pub(crate) fn grants_for(
    grants: &[ContextActionGrant],
    context: OperatorContext,
) -> BTreeSet<&'static str> {
    grants
        .iter()
        .find(|grant| grant.context == context)
        .map(|grant| grant.actions.iter().copied().collect())
        .unwrap_or_default()
}
