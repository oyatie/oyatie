// Intelligence calls are scoped by tenant/personal context and explicit consent.
permit(
  principal,
  action in [
    Action::"IntelligenceAssistDraft",
    Action::"IntelligenceContextAwareRetrieval"
  ],
  resource
)
when {
  principal.context_id == resource.context_id &&
  principal.consent_grants.contains(resource.consent_grant_id) &&
  principal.ai_budget_remaining > 0
};

forbid(
  principal,
  action,
  resource
)
when {
  principal.context_id != resource.context_id
};
