// ordering-provider-can-view-own.cedar
// Ordering providers may view lab/pathology orders and results they ordered or are covering.

permit (
  principal in Role::"ordering-provider",
  action in [
    Action::"lab-order.read",
    Action::"lab-result.read",
    Action::"pathology-case.read",
    Action::"DiagnosticReport.read",
    Action::"Observation.read"
  ],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  (resource.ordering_provider_id == principal.id ||
   principal.id in resource.covering_provider_ids) &&
  context.purpose_of_use == "treatment"
};
