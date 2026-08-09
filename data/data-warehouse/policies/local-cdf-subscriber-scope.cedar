// local-cdf-subscriber-scope.cedar
// Wave-15A — Defect closure F-D4-L-06
// IP-037 binding
//
// Default-deny on ChangeDataFeed::subscribe.
// Required conditions:
// - subscriber is the producer tenant's principal, OR
// - subscriber is a signed SHARE_CONSUMER attached to a share that
//   has CDF enabled
// - producer tenant_class == "paid" (CDF emission is a paid-only feature)

@id("data-warehouse-cdf-producer-paid-only")
forbid (
  principal,
  action == Action::"ChangeDataFeed::subscribe",
  resource
)
when {
  resource.producer.tenant_class != "paid"
};

@id("data-warehouse-cdf-subscriber-must-be-same-tenant-or-share-consumer")
forbid (
  principal,
  action == Action::"ChangeDataFeed::subscribe",
  resource
)
unless {
  principal.tenant_id == resource.producer.tenant_id
  ||
  (
    principal.audience_type == "SHARE_CONSUMER"
    &&
    principal.share_id in resource.producer.cdf_enabled_share_ids
  )
};

@id("data-warehouse-cdf-streaming-billing-required")
forbid (
  principal,
  action == Action::"ChangeDataFeed::subscribe",
  resource
)
unless {
  "streaming_ingest_events" in resource.producer.billing_components
};
