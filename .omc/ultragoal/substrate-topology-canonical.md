# Canonical substrate topology — cross-model-verified (Sol Ultra)

**Source:** codex gpt-5.6-sol/ultra cross-check vs my proposed shape, grounded in AWS cell-based architecture + Verified Permissions + Zanzibar + SPIRE/Nested-SPIRE + Nitro/Titan + Meta Shard Manager, cited against committed origin/dev (ADR-0280, platform-architecture.json, capability-registry.json). Verdict on my proposal: **DISAGREE (high confidence) — less wrong than both committed specs, but still a category error.** This is the corrected canonical topology to encode as the ADR-0280 amendment that lands FIRST (before the ADR-0562/0615 Accept).

## The category error (in BOTH committed specs AND my proposal)
**Equating a capability ownership boundary with one deployable bootstrap node.** That forces the false choice "cell is the leaf" vs "secrets is the leaf." Reality: a bare cell envelope is *below* the services hosted in it; hardware/boot trust is *below* the operational secrets service; cell lifecycle is *above* iam/policy; the router is a *separate* distributed data plane; and every critical control capability has BOTH a global-control face and a cell-local/runtime face. → The canonical model is a **face-aware, sharded, typed DAG across planes**, NOT a linear stratum map.

## Two kinds of roots
- **Security/integrity roots (EXTERNAL to the 24 caps):** hardware RoT + measured boot; offline/threshold org root key + immutable trust bundle; signed bootstrap manifest + artifact digests + initial cell identity; genesis/break-glass authority. (AWS Nitro / Google Titan analog.)
- **Liveness/hosting roots:** bare compute; bootstrap network/DNS/time; local durable boot storage; the minimal runtime that starts k8s + the first service.
- **Operational KMS/SPIRE is NOT irreducible** — it runs on the hosting substrate and is authenticated through IAM. `secrets.root-control` is the sole *in-graph cryptographic authority service*, but NOT the bootstrap leaf. The router is not a root at all.

## The corrected canonical topology — 4 planes + external root (not 2)
```
E0  external genesis roots: hardware RoT + signed boot/artifacts + org root key/quorum + bootstrap compute/network/storage facts
        |
B0  empty cell envelope: {network.bootstrap, compute.bootstrap, storage.bootstrap} -> k8s.bootstrap -> cell.envelope
        |
C0  cell-local trust & authz: secrets.cell/intermediate -> iam verifier -> tenancy/home-cell snapshot -> policy PDP + local versioned policy/ReBAC store
        |
C1  cell-local reliability & platform: audit append/seal, observability, data, messaging, per-cell gateway, local flags
        |
C2  cell workloads: intelligence, workflow, billing metering, marketplace, console APIs, compliance enforcement, comms

  in parallel:
G   logically-global, physically-PARTITIONED management plane: secrets root admin, cell registry/lifecycle/placement, IAM admin, tenancy directory, policy authoring/signing/distribution, fleet k8s/network/compute/storage, CI/IaC + catalogs/aggregation
        | (signed, versioned snapshots — one-way)
R   distributed routing data plane: network edge + gateway edge + cell.router — thin, cellularized, cached, NO live G-plane dependency (static stability)
```
**Generating rule:** push everything into the cell that *can* live in the cell; the global plane may be large but **cell runtime must never synchronously depend on it** (static stability — data plane survives control-plane outage). The router is a data plane, not the control plane, and must itself be cellularized (only layer knowing all cells).

## Five distinct graphs (a single `bootstrap_order` cannot encode all)
genesis · new-cell provisioning · steady-state request · control-data publication · failure/brownout propagation — different edge types, often opposite directions.

## Capability face-splits (the crux)
- **cell** → `cell.envelope` (B0 empty failure domain) · `cell.genesis` (break-glass, first mgmt cell) · `cell.lifecycle.cp` (G; depends on iam/policy/tenancy/audit) · `cell.router.dp` (R; signed cached snapshots, no sync tenancy/iam). Breaks the alleged cycle: genesis creates an empty cell ID first; lifecycle needs iam/policy later.
- **iam** → G identity admin · C0 local token/SVID/JWK validation.
- **policy** → G authoring/signing/distribution · C0 local PDP + versioned tenant policy/ReBAC store. (standalone ✓ but NOT a singleton global PDP.)
- **tenancy** → G lifecycle/home-cell directory · C0 signed cached tenant context + routing snapshot.
- **secrets** → external-root-backed G root/intermediate mgmt · C0 cell key partition + downstream SPIRE issuer.
Static-stability invariant: existing sessions/routes continue on cached state; only NEW identity/tenant/placement/migration ops may safely stop.

## Policy placement (standalone, cell-distributed)
G authoring/signing/distribution + home-cell authority (tenant policy + ReBAC tuples) + per-cell runtime PDP + last-known-good snapshot; PEPs at gateway + every protected service; router does routing (+ maybe cached token-sig check), NOT full authz. **Stale snapshot must DENY or route to the authoritative shard — never silently authorize.** (AWS AVP = store-per-tenant; Zanzibar = logically-global but physically-distributed-with-local-replicas, not one PDP instance.)

## 24-capability plane placement (summary)
cell[B0/G/R] · iam[G/C0] · policy[G/C0] · tenancy[G/C0] · secrets[E0-backed G/C0] · audit[C0 seal + G async agg] · observability[C0 collectors + G async fleet] · data[C1 + optional G schema] · storage[B0 + C1 + G capacity] · compute[B0 + C1 + G fleet] · k8s[B0/C0 per-cell + G fleet] · network[B0 + R edge + C0 mesh + G DNS/config] · gateway[R edge + C1 ingress/PEP] · messaging[C1 bus/outbox + async cross-cell + optional G schema] · intelligence[C2 + G registry] · workflow[C2 + G catalog] · ci[M mgmt, isolated runners — never a prod request dep] · iac[M desired-state — never a runtime dep] · billing[C1/C2 metering + G async rating/invoice] · marketplace[G catalog + C2 cache/fulfil] · console[G/R shell, APIs route into cells, no shared tenant datastore] · compliance[G authoring + C1 enforcement/evidence] · comms[C2 delivery shard + G templates] · flags[G authoring + C0/C1 LKG evaluator, no sync global lookup].

**`governance` is NOT one of the 24** — cross-cutting, implemented via ci/iac/policy/compliance/audit. Standalone would make 25. The registry's iam-owns-{identity+policy-engine} is superseded by the founder's policy-extract ruling → 24 caps ↔ 24 dag_nodes (policy now owns its own node).

## Consequence for move (1)
The ADR-0280 topology amendment is BIGGER than a spec pick: it must encode the E0/B0/C0/C1/C2 + G + R plane model, the 5 edge-typed graphs, and the per-capability face-splits — then platform-architecture.json + substrate-dependency-dag.json both DERIVE from it (single generated source). This lands FIRST, before the ADR-0562/0615 Accept batch.
