const HR_COMPLIANCE_WORKFLOW_TOPIC: &str = "workflow.hr.compliance.dispatch";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrWorkflowDispatchEnvelope {
    pub topic: data_boundary_kernel::Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: data_boundary_kernel::Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: data_boundary_kernel::Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub workflow_ref: data_boundary_kernel::Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub obligation_kind: data_boundary_kernel::Classified<LaborComplianceObligationKind>, // data_class: INTERNAL_ONLY
    pub jurisdiction: data_boundary_kernel::Classified<Jurisdiction>, // data_class: INTERNAL_ONLY
    pub required_steps: data_boundary_kernel::Classified<Vec<LaborComplianceWorkflowStep>>, // data_class: INTERNAL_ONLY
    pub evidence_refs: data_boundary_kernel::Classified<Vec<AuditEvidenceRef>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: data_boundary_kernel::Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: data_boundary_kernel::Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaborWorkflowPlanOutcome {
    pub obligations: Vec<LaborComplianceObligation>, // data_class: INTERNAL_ONLY
    pub workflow_dispatches: Vec<HrWorkflowDispatchEnvelope>, // data_class: INTERNAL_ONLY
}

pub fn plan_labor_compliance_workflows(
    snapshot: LegalEntityWorkforceSnapshot,
) -> Result<LaborWorkflowPlanOutcome, HrAppError> {
    let obligations = evaluate_labor_compliance(snapshot)?;
    let workflow_dispatches = obligations.iter().map(workflow_dispatch).collect();

    Ok(LaborWorkflowPlanOutcome {
        obligations,
        workflow_dispatches,
    })
}

fn workflow_dispatch(obligation: &LaborComplianceObligation) -> HrWorkflowDispatchEnvelope {
    HrWorkflowDispatchEnvelope {
        topic: internal(HR_COMPLIANCE_WORKFLOW_TOPIC.to_owned()),
        tenant_id: internal(obligation.tenant_id.value.clone()),
        legal_entity_id: internal(obligation.legal_entity_id.value.clone()),
        workflow_ref: internal(obligation.workflow_ref.value.clone()),
        obligation_kind: internal(obligation.kind.value),
        jurisdiction: internal(obligation.jurisdiction.value),
        required_steps: internal(obligation.workflow_steps.value.clone()),
        evidence_refs: internal(obligation.evidence_paths.value.clone()),
        idempotency_key: internal(obligation.idempotency_key.value.clone()),
        schema_version: public(1),
    }
}
