---
adr_id: ADR-PAS-0004
title: "Vetting trust verdicts (retired four-label ladder) determined by structured rubric, not human discretion"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0004: Vetting trust verdicts (retired four-label ladder) determined by structured rubric, not human discretion

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Vetting trust verdicts (retired four-label ladder) determined by structured rubric, not human discretion.

## Decision

Vetting trust verdict computed deterministically from rubric (code-coverage, performance budget headroom, accessibility score, install count, rating average). Human reviewer can override with audit trail; auto-tier is the baseline.

## Alternatives considered

### Pure human discretion (REJECTED)

Inconsistent across reviewers; gameable by relationship; not auditable.

## Consequences

Tenant trust signal is calibrated; appeals process via runbook documented.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
