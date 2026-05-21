---
doc_status: published
last_audited: 2026-05-20
---
# Personas Guide

Start with `MASTER-ROSTER-2026-05-21.md`. The roster is not a list of separate users; it is a graph of identity projections: one human can appear across tenant, role, workspace, locale, device, and skill-tier contexts while retaining the same identity root.

## How to Read MASTER-ROSTER

1. Read section 1 for the continuity-of-identity doctrine. It explains why personas are context projections rather than independent accounts.
2. Read the axes before opening individual persona files: collar color, workspace, skill tier, locale, device profile, tenant membership, Cedar permit class, and journey coverage.
3. Use `cross_context_personas[]` as the same-human bridge. When two personas represent the same person in different contexts, product and policy flows must preserve that linkage without leaking tenant data.
4. Treat persona dossiers as build inputs for PRDs, journeys, Cedar tests, and UX acceptance criteria. If a journey references a persona, that persona needs enough detail to test authority, accessibility, language, device, and privacy posture.

## Update Rules

Do not create a persona because a role sounds useful. Add or amend a persona only when a journey, ADR, PRD, or compliance pack needs that projection. Keep the slug stable, add supersession notes instead of deleting, and update the master roster before relying on a new persona in downstream docs.
