# `dns` OpenTofu module

> ADR anchor: ADR-0202 (Tier B).

DNS zones + records. Publishes SPF / DKIM / DMARC records that
ADR-0201 comms-email µservice produces. Per-tenant from-domain
onboarding (IP-011) flows through here.
