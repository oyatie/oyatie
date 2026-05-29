// local-secure-share-create.cedar
// Wave-15A — Defect closures F-D4-D-03
// IP-042 binding
//
// Default-deny on Share::publish.
// Required conditions:
// - producer tenant_class == "paid"
// - producer has enabled "share_consumer_events" billing component
// - if consumer is non-tenant (reader-account), a signed DealSet must
//   exist per ADR-0314
// - consumer principal class is in the allowed set

@id("data-warehouse-share-publish-paid-only")
forbid (
  principal,
  action == Action::"Share::publish",
  resource
)
when {
  resource.producer.tenant_class != "paid"
};

@id("data-warehouse-share-publish-billing-required")
forbid (
  principal,
  action == Action::"Share::publish",
  resource
)
unless {
  "share_consumer_events" in resource.producer.billing_components
};

@id("data-warehouse-share-publish-non-tenant-needs-dealset")
forbid (
  principal,
  action == Action::"Share::publish",
  resource
)
when {
  resource.consumer.kind == "reader_account"
  &&
  resource.dealset_id == null
};

@id("data-warehouse-share-publish-consumer-class-check")
forbid (
  principal,
  action == Action::"Share::publish",
  resource
)
unless {
  resource.consumer.audience_type == "SHARE_CONSUMER"
  ||
  resource.consumer.audience_type == "DATA_PLATFORM_OPERATOR"
};
