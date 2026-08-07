// Cedar policy — tenant-bootstrap controller event publication authority.
// Authority: ADR-0007 Cedar, ADR-0145 inter-µservice communication.
//
// Only the tenancy µservice may publish onboarded/offboarded events on the
// `tenancy/tenant-events` channel; the analytics bootstrap controller
// consumes them. The controller, in turn, is the only authorized publisher
// of tenant_database.created / dropped / proof_of_erasure events.

@id("tenancy-publishes-tenant-events")
permit (
  principal == Microservice::"tenancy",
  action in [Action::"PublishEvent"],
  resource == EventChannel::"tenancy/tenant-events"
);

@id("analytics-publishes-tenant-database-lifecycle")
permit (
  principal == Microservice::"analytics",
  action in [Action::"PublishEvent"],
  resource == EventChannel::"analytics/tenant-database-lifecycle"
);

@id("analytics-publishes-proof-of-erasure")
permit (
  principal == Microservice::"analytics",
  action in [Action::"PublishEvent"],
  resource == EventChannel::"audit-chain/proof-of-erasure"
);

@id("default-deny-event-publication")
forbid (
  principal,
  action == Action::"PublishEvent",
  resource
) unless {
  principal in [Microservice::"tenancy", Microservice::"analytics", Microservice::"audit-chain", Microservice::"workflow-engine"]
};
