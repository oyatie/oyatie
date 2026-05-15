# Foundry Supervisor Security

## Secret Management
All provider credentials and API keys are stored in OpenBao. The supervisor resolves `sref://` references at spawn time; raw secrets never enter the repo or persistent logs.

## Autonomy Ceilings
The supervisor enforces Cedar-based autonomy ceilings. Capabilities are classified into tiers (T1-T4), and the supervisor refuses to spawn a driver whose capability exceeds the tenant's current ceiling.

## Audit Chain
Every decision point (spawn, reject, quarantine, drift) emits a signed audit row to the audit-chain. This provides non-repudiable proof of compliance with routing and usage policies.
