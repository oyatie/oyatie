// Cedar policy — T1-webhook-delivery-event ingest authorization.
// ADR anchor: ADR-0201, ADR-0183.

// Only the configured provider service-account may POST to
// /v1/webhooks/{provider}. HMAC signature verification is the
// transport-layer authentication; this policy enforces the
// principal/role match at the application layer.

permit (
    principal,
    action == Action::"comms-email:webhook-ingest",
    resource
)
when {
    principal has role &&
    principal.role == "comms-email-webhook-ingester" &&
    resource has provider &&
    [ "ses", "postal", "mailgun" ].contains(resource.provider)
};

// Webhook ingest from a provider that is not in the active
// adapter set is forbidden — even if HMAC checks pass — so
// we cannot accidentally onboard a provider via webhooks.
forbid (
    principal,
    action == Action::"comms-email:webhook-ingest",
    resource
)
when {
    resource has provider &&
    !([ "ses", "postal", "mailgun" ].contains(resource.provider))
};
