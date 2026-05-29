# IP-018 — Cedar Policy Fragments

**microservice**: feature-flags
**bc**: policy
**layer**: policy
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0183, ADR-0243, ADR-0244, ADR-0246, ADR-0247, ADR-0292, ADR-0293, ADR-0294, ADR-0295, ADR-0297, ADR-0298
**companion_ips**: IP-006, IP-010, IP-017

## Scope

All Cedar policy fragments for feature-flags: flag-mutation-authorization, experiment-design-authorization, safety-killswitch-authorization, pack-flag-override, pack-overlay-authorization, abuse-defence, auditor-scope, ci-scope, emergency-services-bypass, tenant-targeting.

## Deliverables

| # | Fragment | File | Key Gates |
|---|----------|------|-----------|
| 1 | flag-mutation-authorization | `policy/flag-mutation-authorization.cedar` | Create (step-up A), Update live (step-up B), Archive (step-up B), Delete archived (step-up C), Undo 15s window |
| 2 | experiment-design-authorization | `policy/experiment-design-authorization.cedar` | Activate requires `has_sample_size_estimate`; MINOR_TARGETED requires compliance pack; EMERGENCY_SERVICES requires platform-safety-officer |
| 3 | safety-killswitch-authorization | `policy/safety-killswitch-authorization.cedar` | sre-oncall + step-up C; life-safety FORBID list; cross-tenant FORBID |
| 4 | pack-flag-override | `policy/pack-flag-override.cedar` | pack-overlay-agent only; FORBID pack override disabling EMERGENCY_SERVICES flags |
| 5 | pack-overlay-authorization | `policy/pack-overlay-authorization.cedar` | Foundry attestation + cosign signature required; FORBID without attestation |
| 6 | abuse-defence | `policy/abuse-defence.cedar` | EMERGENCY_SERVICES bypass first; bot_score≥95 block; mutation rate >60/min FORBID; honeypot FORBID |
| 7 | auditor-scope | `policy/auditor-scope.cedar` | Compliance officer (tenant-scoped); QSA (time-bounded); regulator (warrant); FORBID delete |
| 8 | ci-scope | `policy/ci-scope.cedar` | Foundry CI read + rollout advance/rollback; FORBID mutations + kill-switch |
| 9 | emergency-services-bypass | `policy/emergency-services-bypass.cedar` | LIFE-SAFETY; NENA-i3/NCMEC/FEMA/crisis-line; FORBID pack-override via emergency bypass |
| 10 | tenant-targeting | `policy/tenant-targeting.cedar` | Tenant-scoped read; default-deny cross-tenant |

## Soak Window Requirement (ADR-0294)

Every Cedar fragment MUST pass through `oya-foundry-cedar-fragment-activator` which enforces a ≥60s soak window between upload and activation. The `targeting-kernel` (IP-006) also enforces this at runtime.

## Cosign Attestation (ADR-0293)

Policy fragments authored by `pack-overlay-agent` MUST be cosign-signed under the meta-trust-root. The `pack-overlay-authorization.cedar` fragment enforces this at evaluation time: `context.pack_overlay_agent_attestation.cosign_verified == true`.

## Definition of Done

- `cedar validate --schema policy/schema.cedarschema` passes for all 10 fragments
- Soak window: uploading a fragment and immediately activating returns `TooFresh` error
- Life-safety FORBID: `NENA_I3_ROUTING` disengage without warrant → `Deny`
- Abuse-defence: EMERGENCY_SERVICES `audience_type` bypasses all rate limits
- CI gate `lean-a7-cedar-fragments` green
