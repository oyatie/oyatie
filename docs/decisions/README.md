# ADR live source of truth (clean)

**Only Accepted topic-apex ADRs live here** (10 files). All historical ADRs (former Accepted members, Superseded, Rejected) are in [`docs/adr-archive/`](../adr-archive/).

| ID | Topic |
|----|--------|
| [ADR-0700](ADR-0700-ci-admission-live-apex.md) | CI admission / build / runners |
| [ADR-0701](ADR-0701-monorepo-capability-live-apex.md) | Monorepo capability / faces / reorg |
| [ADR-0702](ADR-0702-identity-authz-live-apex.md) | Identity / authz / secrets |
| [ADR-0703](ADR-0703-cas-cache-live-apex.md) | CAS / cache (RE activation fail-closed) |
| [ADR-0704](ADR-0704-k8s-port-live-apex.md) | K8s port / owned kernel |
| [ADR-0705](ADR-0705-product-protocol-live-apex.md) | Product protocols / APIs / comms |
| [ADR-0706](ADR-0706-observability-live-apex.md) | Observability / progressive delivery |
| [ADR-0707](ADR-0707-trust-safety-live-apex.md) | Trust / safety / resilience |
| [ADR-0708](ADR-0708-platform-foundations-live-apex.md) | Cells, residency, workflow, plugins, search |
| [ADR-0709](ADR-0709-general-live-apex.md) | General architecture remainder |

**Redirect map (old number → live apex):** [`_disposition/adr-redirect.v1.json`](_disposition/adr-redirect.v1.json)

**Agent rule:** read apex first; resolve old `ADR-NNNN` via redirect map; full text only in archive/git history when needed for provenance.

**Census:** P3 selector includes only direct `docs/decisions/ADR-*.md` children — now the 10 apex files (plus this README is not selected).
