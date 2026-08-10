# ADR live source of truth (clean)

**Accepted topic-apex ADRs live here, plus any GATED PROPOSED apex** — 11 files: ADR-0700…0709 Accepted, ADR-0710 Proposed. All historical ADRs (former Accepted members, Superseded, Rejected) are in [`docs/adr-archive/`](../adr-archive/).

A **gated Proposed apex** is a new topic apex deliberately not yet Accepted because a named clause waits on named evidence. It lives here so agents resolve it at step 1 instead of missing it, and it is **not implement authority while Proposed**. That is enforced, not promised: the live-resolution rule in [`_disposition/2026-08-06-live-resolution-rule.json`](_disposition/2026-08-06-live-resolution-rule.json) ranks Proposed/Deprecated/Rejected as "not implement authority", and the `adr-citation-closure` gate fails closed under `adr_citation_rejected_authority` when any of the three authority surfaces cites one. **Location is discoverability; status is authority** — see [`_disposition/END-STATE-POLICY.md`](_disposition/END-STATE-POLICY.md) for the four conditions a gated Proposed apex must meet.

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
| [ADR-0710](ADR-0710-kubernetes-admission-substrate-is-the-api-server.md) **(Proposed — gated)** | Kubernetes admission substrate — gated on clause D-8's workload-boundary evidence |

**Redirect map (old number → live apex):** [`_disposition/adr-redirect.v1.json`](_disposition/adr-redirect.v1.json)

**Agent rule:** read apex first; resolve old `ADR-NNNN` via redirect map; full text only in archive/git history when needed for provenance.

**Census:** P3 selector includes only direct `docs/decisions/ADR-*.md` children — now the 11 apex files (plus this README is not selected).
