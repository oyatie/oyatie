// Cedar authorization policies for the CI Webhook Gateway (ADR-0150/0243).
//
// The gateway is a substrate service, not tenant-scoped. Authorization here is
// coarse: only the Forgejo forge principal may post webhook deliveries, and
// only the gateway service identity may kick the Jenkins pipeline. The HMAC
// signature is the cryptographic authenticator; Cedar is the policy gate that
// scopes WHICH principals may exercise WHICH action on the gateway.

// Allow the Forgejo forge service principal to deliver webhooks.
permit (
    principal == Service::"forgejo-forge",
    action == Action::"webhook.deliver",
    resource == Endpoint::"webhook/forgejo"
);

// Allow the gateway service identity to kick the Jenkins CI lane.
permit (
    principal == Service::"ci-webhook-gateway",
    action == Action::"pipeline.kick",
    resource == Service::"jenkins-oya-ci-lane"
);

// Deny any cross-tenant data action — the gateway has no tenant data path.
forbid (
    principal,
    action == Action::"tenant.data.read",
    resource
);
