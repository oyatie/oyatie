# oya-governance-cedar-coverage

Scaffolds the ADR-0243 CI gate for public API Cedar policy coverage.

## Rule

Every public API endpoint must have a corresponding Cedar policy in `policies/*.cedar`.

## Trigger

The gate triggers when public API endpoints or Cedar policy files are added or changed.

## Compliant

A compliant endpoint has an explicit policy binding whose Cedar policy exists in the policy directory and can be matched back to the endpoint inventory.
