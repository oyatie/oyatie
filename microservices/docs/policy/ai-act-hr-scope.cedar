// Cedar policy: ai-act-hr-scope
// Microservice: docs
// Purpose: refuse docs AI writing-assist capabilities when an HR/employment
// document workflow enters regulated-AI scope and no conformity-assessment
// evidence is on file.
// Owner: council-privacy + axis-docs + ops-legal
// Status: Accepted (2026-05-19)
// Related: ADR-DOCS-0005, capabilities/T1-assist.yaml, capabilities/T2-auto.yaml
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// REGULATED-AI HR DOCUMENT REFUSAL.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"DocsT1AutoSummary",
    Action::"DocsT1ExpandRewrite",
    Action::"DocsT2EnableAutoTranslate",
    Action::"DocsT2EnableAutoSummaryOnPublish"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.hr_document_workflow == true &&
  context.context_kind == "employment" &&
  !(context has ai_act_conformity_assessment_evidence_on_file &&
    context.ai_act_conformity_assessment_evidence_on_file == true)
};
