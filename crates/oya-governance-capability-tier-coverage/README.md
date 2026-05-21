# oya-governance-capability-tier-coverage

Scaffolds the ADR-0316 CI gate for capability-tier coverage.

## Rule

Every microservice must have an entry in `registry/capability-tiers/microservice-tier-mapping.yaml`.

## Trigger

The gate triggers when the canonical microservice inventory changes or when the capability-tier mapping file changes.

## Compliant

A compliant change keeps the mapping file in one-to-one coverage with the microservice inventory and reports any intentionally excluded service through the later enforcement surface.
