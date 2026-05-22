# oya-governance-pack-overlay-completeness

Scaffolds the ADR-0251 CI gate for compliance pack overlay completeness.

## Rule

Every microservice must have `packs/` overlays for each applicable compliance pack.

## Trigger

The gate triggers when microservices, compliance pack applicability, or service pack overlays are added or changed.

## Compliant

A compliant service declares and ships the overlay files required by every applicable pack, with missing or non-applicable packs represented through the later enforcement metadata.
