---
id: ADR-compliance-001
status: Accepted
deciders: axis-compliance, axis-security, council-architecture
date: 2026-05-18
related_adrs: [ADR-0184, ADR-0209]
---

# ADR-compliance-001 — Evidence retention policy (hot 90 days / cold 7 years)

## Context

Per IP-009, retention windows must satisfy SOC 2 (7 years), HIPAA (6 years), GDPR (varies), PCI-DSS (3 years), while honoring GDPR Art. 17 erasure where applicable.

## Decision

| Tier | Window | Action |
|---|---|---|
| Hot | 0-90 days | full-fidelity SeaweedFS hot bucket; WORM |
| Cold | 90 days - 7 years | gzip + cosign re-seal; 3-way replicated; off-site backup |
| Archive | 7+ years (PCI) / 6+ years (HIPAA) | reviewed quarterly; statutory-only retention |

GDPR Art. 17 erasure pseudonymizes subject-linked artifacts at 30 days; non-subject-linked SOC 2 evidence retained 7 years.

## Consequences

Storage cost stable; auditor accessibility preserved for the longest statutory window.
