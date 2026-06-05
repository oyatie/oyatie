---
doc_class: Standard
status: Accepted
date: 2026-06-05
canonical_authority: specs/repo-hygiene-automation.json#hyperscaler_adoption_fitness_policy
planned_enforcement_ref: buck2 build //:repo-hygiene-automation-check
---

# Hyperscaler adoption fitness

## Decision

Every Oyatie methodology, reasoning pattern, technology choice, operation,
dependency, vendored tool, runtime, build path, and implementation approach must
be justified through a hyperscaler adoption lens before it becomes durable
architecture.

The question is not "is this tool popular?" The question is:

> Would Google, AWS, Azure, or a comparable hyperscaler plausibly operate this
> pattern at cloud-provider scale, and if not, what Oyatie-native seam makes it
> fit our ambition?

## Required classification

Each non-trivial choice is classified as one of:

| Class | Meaning |
|---|---|
| `adopt` | Use the external thing directly because it is mature, scalable, license-clean, and already fits our control model. |
| `wrap` | Use it behind an Oyatie-owned adapter because the ecosystem value is high but the semantics are not ours. |
| `shadow` | Keep it only as temporary compatibility or migration evidence. |
| `self_write` | Implement the strategic seam ourselves because sovereignty, scale, safety, performance, or product leverage requires it. |
| `reject` | Do not use it because it creates a scalability, security, licensing, operational, or architecture mismatch. |
| `historical` | Preserve only as provenance; do not use as current guidance. |

## Fitness dimensions

A decision must record evidence for these dimensions, or explain why a dimension
is not applicable:

1. **Scale and concurrency** — supports hyperscale fanout, parallel lanes,
   large repositories, many tenants, and high event volume without shared mutable
   bottlenecks.
2. **Isolation and blast radius** — supports cells, stamps, shuffle sharding,
   tenant isolation, account/project boundaries, and failure containment.
3. **Control-plane / data-plane split** — does not put tenant data-plane
   survival behind an unavailable control plane.
4. **Automation over toil** — supports controller reconciliation, hands-off
   safe deployment, rollback, and evidence capture rather than manual heroics.
5. **Security by default** — supports zero trust, workload identity,
   least-privilege, provenance, policy-as-code, encryption, and auditable
   access.
6. **Portability and exit** — has adapter seams, conformance tests, data/export
   paths, and no opaque lock-in that blocks self-hosted or multi-cloud goals.
7. **Performance and cost** — has measurable latency, throughput, cache,
   storage, and operational cost behavior under cold and warm paths.
8. **Operational maturity** — supports SLOs, metrics, logs, traces, readiness,
   health checks, incident recovery, and bounded failure modes.
9. **Compliance and auditability** — produces dual-attribution audit evidence,
   policy decisions, change records, and retention/deletion controls.
10. **Language/runtime fit** — defaults to memory-safe, compile-time checked,
    Rust-first implementation for durable Oyatie semantics, with explicit
    exceptions for ecosystem adapters, bootstrap seams, or standards tooling.
11. **Shared-surface discipline** — avoids high-conflict root files, shared
    helper libraries, and hand-edited canonical configs when lane-owned shards or
    generated aggregates can remove conflict.
12. **Strategic leverage** — advances Oyatie's cloud-provider ambition rather
    than importing a smaller-product operating model.

## Build/adopt rule

Adopt proven hyperscaler patterns first. Reimplement only when evidence shows
that an Oyatie-owned implementation gives better control, scale, safety,
performance, portability, tenant isolation, policy proof, or product leverage.

Do not cargo-cult Google, AWS, Azure, Kubernetes, Prow, Sapling, Piper, CUE,
Timoni, Helm, or any other system. Treat them as evidence and compatibility
interfaces, not automatic authorities.

## Backlog rule

If a decision cannot yet pass the fitness test but is still useful to unblock
parallel work, it must enter backlog as `shadow`, `wrap`, or `self_write`
instead of becoming first-class durable architecture.

Backlog entries must include:

- current role and allowed scope;
- hyperscaler evidence source or counterargument;
- blocking restriction or scalability concern;
- adapter/wrapper boundary;
- conformance or replacement path;
- cutover/removal condition;
- Buck2/Prow verification target.

## Source-driven basis

- Google SRE Release Engineering:
  https://sre.google/sre-book/release-engineering/
- Google SRE Eliminating Toil:
  https://sre.google/sre-book/eliminating-toil/
- AWS Builders Library, safe hands-off deployments:
  https://aws.amazon.com/builders-library/automating-safe-hands-off-deployments/
- AWS Builders Library, shuffle sharding:
  https://aws.amazon.com/builders-library/workload-isolation-using-shuffle-sharding/
- Azure Deployment Stamps pattern:
  https://learn.microsoft.com/en-us/azure/architecture/patterns/deployment-stamp
- AWS Well-Architected Framework:
  https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html
- Azure Well-Architected reliability and security:
  https://learn.microsoft.com/en-us/azure/well-architected/reliability/
  https://learn.microsoft.com/en-us/azure/well-architected/security/
