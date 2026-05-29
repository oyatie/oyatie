# API Gateway Threat Model

## Assets

- Tenant request identity, route selection, cell residency metadata, and edge audit events.

## Threats

- Spoofed tenant identity through malformed JWT or partner certificate.
- Cross-cell routing attempt that bypasses residency policy.
- WAF rule evasion or denial amplification.
- Replay of admitted request identifiers.

## Mitigations

- JWT and mTLS validation happen before workload routing.
- Cedar coarse-scope policy rejects tenant or cell mismatch.
- WAF triggers and policy denials emit audit-chain events.
- Request IDs are propagated once and treated as replay-sensitive.
