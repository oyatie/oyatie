# Connect Retirement Threat Model

## Assets

- Retirement status, sub-service readiness evidence, and deletion criteria for the Connect umbrella.

## Threats

- New product runtime scope lands under the retiring umbrella.
- Retirement status claims drift from sub-service evidence.
- Tenant data ownership is accidentally reintroduced into the umbrella.

## Mitigations

- Cedar policy forbids new runtime scope under `connect`.
- Contracts expose read-only retirement status.
- Data residency policy delegates user data to first-class sub-services.
