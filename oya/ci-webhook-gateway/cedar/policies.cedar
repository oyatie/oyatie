// Cedar authorization policies for the CI webhook gateway.
//
// The gateway is a substrate service, not tenant-scoped. Authorization here is
// coarse: only the GitHub publication adapter principal may post webhook deliveries, and
// only the gateway service identity may publish the Prow-triggering event. The HMAC
// signature is the cryptographic authenticator; Cedar is the policy gate that
// scopes WHICH principals may exercise WHICH action on the gateway.

// Allow the GitHub publication adapter service principal to deliver webhooks.
permit (
    principal == Service::"github-publication-adapter",
    action == Action::"webhook.deliver",
    resource == Endpoint::"webhook/github"
);

// Allow the gateway service identity to publish the Prow-triggering event.
permit (
    principal == Service::"ci-webhook-gateway",
    action == Action::"prow.trigger",
    resource == Service::"oya-ci-prow-lane"
);

// Deny any cross-tenant data action — the gateway has no tenant data path.
forbid (
    principal,
    action == Action::"tenant.data.read",
    resource
);
