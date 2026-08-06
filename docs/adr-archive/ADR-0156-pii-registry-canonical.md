---
id: ADR-0156
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0156: PII Registry Canonical (Cross-Cutting Data Classification)

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, council-privacy, axis-tenancy
- Tier-A hyperscaler pattern: AWS Macie + GCP DLP + GDPR Art. 30

## Context

GDPR Article 30 ("Records of processing activities"), CPRA, and the
Korea Personal Information Protection Act all require a centrally
queryable register of WHICH personal-data categories WHICH µservice
processes. AWS Macie + GCP DLP encode the same idea as a per-account
data-class inventory.

oyatie has the per-bounded-context `data_classes_owned` field but no
aggregated cross-µservice registry. Without one, a DSR (Data Subject
Request) "delete my data" cascade cannot be authored because the
caller has no machine-readable list of µservices that hold which
PII categories.

## Decision

Adopt a cross-cutting PII registry consolidating per-µservice
data-class processing.

1. Every µservice's `manifest.json` gains a top-level
   `data_classes_processed` array (the UNION of per-BC
   `data_classes_owned`).
2. `specs/microservices/pii-registry.json` aggregates the per-µservice
   `data_classes_processed` into a cross-µservice index by data-class.
3. DSR cascade machinery queries the registry to fan out the
   delete/export/access request to every µservice that holds the
   requested category.
4. New `oya-check-data-class` rules (extending the existing gate)
   validate the per-µservice + registry coherence.

## Consequences

Positive:
- DSR cascade has a machine-readable plan.
- GDPR Art. 30 compliance evidence in one artifact.
- AWS Macie / GCP DLP-equivalent inventory.

Negative:
- Every µservice manifest update.
- Registry maintained as a derived artifact (kept in sync via gate).

## Alternatives considered

- Per-µservice ad-hoc declarations — REJECTED, no aggregate view.
- Tag-based ad-hoc PII discovery (Macie-style) — DEFERRED; registry
  ships first.

## References

- GDPR Art. 30 — Records of Processing Activities.
- AWS Macie — automated PII discovery.
- GCP DLP — data classification.
- specs/microservices/pii-registry.json.
- ADR-0008-data-use-boundary.
