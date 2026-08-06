---
id: ADR-0014
status: Superseded
superseded_by: [ADR-0709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Build-vs-buy matrix — stack ownership

# ADR-0014: Build-vs-buy policy — per-microservice matrix (in-house obligatory / external acceptable / requires-review), decision flow chart, per-dep metadata (license tier + maturity + isolation + replacement plan + owning team), oya-governance-build-vs-buy CI lane

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture` + per-microservice leads
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0011, ADR-0013, ADR-0015, ADR-0019

---

## Context

PRD §3.1 commitment 4 sets the build-vs-buy posture: "in-house build over external dep wherever the dep is not as mature as `axum` / `tokio` / `serde` / a Postgres driver / OS kernel-grade tools." TOOLCHAIN §6 codifies the decision flow chart (existing → industry-standard with license + maturity → in-house). What's missing is a *per-microservice* matrix that tells a PR author whether a dep choice is normally OK, normally not OK, or requires review — and a CI gate that rejects deps that violate the matrix.

Without this ADR, every dep adoption is an ad-hoc judgment call. The license tier (ADR-0013) is necessary but not sufficient: an Apache-2 dep can still be the wrong choice if it duplicates an in-house obligatory surface (substrate kernels, audit chain, capability registry) or if the replacement plan is missing. The contradiction ledger LEDG-004 + LEDG-013 captured prior dep-choice drift; this ADR closes the protocol gap.

---

## Decision

We adopt a **per-microservice build-vs-buy matrix**, a **decision flow chart**, **per-dep metadata** in the catalog, and a CI lane that enforces the matrix.

### Per-axis matrix (in-house obligatory / external acceptable / requires-review)

| Axis surface | Default | Rationale |
|---|---|---|
| **Foundation kernels** (Tenant, Identity, Audit chain, Capability registry, Plane, Eventing, Policy/Cedar, Ontology) | **In-house obligatory** | Substrate forking forbidden (ADR-0001); cohesion + sovereignty |
| **Foundry runtime** (capability invocation, autonomy ceiling, evidence emission, RAG endpoint, sandbox kernel) | **In-house obligatory** | Same as above |
| **Provider adapters — API mode** (Anthropic / OpenAI / Gemini API) | **In-house** (direct HTTP + serde) | Avoid vendor SDK lock; license-clean |
| **Provider adapters — subscription mode** | **In-house wrapper around Chromiumoxide** | Headless browser is acceptable adapter-layer dep |
| **Cloud control plane** (IAM, region register, capacity, marketplace catalog) | **In-house** | Cohesion (ADR-0001) |
| **Cloud data plane — managed K8s wrapper** | **In-house wrapper around `kube-rs`** | `kube-rs` is Apache-2; wrapping for the IAM tie-in |
| **Cloud data plane — hypervisor** | **External acceptable** (QEMU / Firecracker; both Apache-2) | Best-of-breed |
| **Cloud data plane — storage primary** | **In-house** | KMS + DEK + audit integration |
| **Cloud data plane — networking dataplane** | **External acceptable** for Pingora-class LB / Envoy mesh; **in-house** for VPC + SG | License-clean OSS where mature |
| **Search indexer** | **External acceptable** (Tantivy, Apache-2) initially; **in-house extension** as scale demands | Tantivy is best-in-class |
| **Search vector index** | **External acceptable** (pgvector, Apache-2; FAISS via FFI) initially; **in-house** as scale demands | Per ADR-0044/0177 |
| **Search ranker — semantic** | **In-house serving + Python training** | Industry standard split |
| **Ads auction engine** | **In-house** | Sub-100 ms; no vendor matches |
| **Ads ML — smart bidding** | **External (PyTorch/xgboost) for training; in-house Rust for inference** | Industry standard split |
| **Analytics — OLAP store** | **External acceptable** (DataFusion / DuckDB Apache-2; ClickHouse Apache-2 with caveats) | Mature OSS |
| **Database — primary OLTP** | **External acceptable** (PostgreSQL + Citus per ADR-0045) | License-clean; mature |
| **Database — KV / cache** | **External acceptable** (Redis pre-7.4; DragonflyDB BSD; KeyDB BSD; Garnet MIT) | License-tier-aware (ADR-0013 forbids Redis 7.4+ SSPL) |
| **Message broker** | **External acceptable** (Apache Kafka per ADR-0005); **in-house schema registry** | Per ADR-0005 |
| **Container runtime** | **External** (containerd + runc; Apache-2) | Industry standard |
| **Service mesh** | **External** (Istio Ambient, Apache-2) | Per ADR-0044 |
| **API gateway** | **External** (Envoy, Apache-2) | Per ADR-0013 |
| **IaC** | **External** (OpenTofu, MPL-2; Pulumi where general-purpose code needed) | Per ADR-0044 |
| **CD** | **External** (Argo Rollouts + Argo CD, Apache-2) | Per ADR-0050 |
| **Container registry** | **External** (Harbor, Apache-2) | Per ADR-0028 (cloud-provider container registry) |
| **Supply-chain — signing** | **External** (Cosign + Rekor, Apache-2) | Per ADR-0039 |
| **Supply-chain — scanning** | **External** (Trivy, Apache-2) | Per ADR-0039 |
| **Secrets management** | **External** (OpenBao, MPL-2) | Per ADR-0043 |
| **Observability — metrics + traces + logs** | **External** (OpenTelemetry, VictoriaMetrics, Tempo, Loki — all Apache-2) | License-clean OSS |
| **Workflow visual editor host** | **External acceptable** (draw.io fork or in-house) | Either OK |
| **Plugin sandbox runtime** | **External** (Wasmtime + WASI Preview 2; both Apache-2) | Per ADR-0023 |
| **Notebook environment** | **External** (Jupyter, BSD) for ad-hoc; **in-house** when integrated with Foundry RAG | Pragmatic |

Anything not in the matrix defaults to **requires-review**.

### Decision flow chart

```
                     New dep need
                          ↓
          Is the surface "in-house obligatory" per the matrix?
                ├── yes → build in-house; no external dep
                └── no
                          ↓
          Does an existing in-house surface cover it?
                ├── yes → reuse
                └── no
                          ↓
          Does an external dep at Tier 1 license (ADR-0013) cover it
          AND meet the maturity bar (axum-class)
          AND have a clear replacement plan if abandoned?
                ├── yes → adopt; declare per-dep metadata
                └── no
                          ↓
          Build in-house; open ADR documenting choice + maintenance plan
                          ↓
          Add to catalog; CI gate verifies; vendor-partner-ledger row added
```

### Per-dep metadata

Every adopted external dep carries a row in `docs/VENDOR-PARTNER-LEDGER.md` AND a catalog annotation:

```yaml
# registry/catalog/<consuming-crate>.yaml
external_deps:
  - name: tantivy
    version: ">=0.22, <0.23"
    license_tier: tier1
    license: MIT
    maturity: production-ready              # production-ready | beta | alpha | research
    isolation: adapter-layer                # kernel-only | adapter-layer | runtime-only
    replacement_plan: in-house-extension-on-scale
    replacement_trigger: ">100M docs per cell, or >50 QPS sustained per cell"
    owning_team: axis-search
```

### CI lane: `oya-governance-build-vs-buy`

Runs on every PR that touches `Cargo.toml`, `package.json`, `requirements.txt`, `go.mod`, or any catalog record. It:

1. Resolves added deps against the per-microservice matrix.
2. Hard-fails any "in-house obligatory" surface that imports an external dep.
3. Emits `requires-review` label for any dep not in the matrix.
4. Verifies per-dep metadata is present in the catalog (license_tier + maturity + isolation + replacement_plan + owning_team).
5. Cross-checks license_tier against ADR-0013.

### Boundary

- Applies to: every external dep in any product crate (Tier 1, Tier 2, Tier 3 from ADR-0013); every WASM plugin published to the Marketplace.
- Does not apply to: dev-dependencies (per ADR-0013 carve-out); operationally consumed third-party SaaS.

---

## Consequences

### Positive

- Per-axis discipline becomes mechanical; "should we build or buy this?" is a matrix lookup, not a debate.
- Substrate forking (in-house obligatory surfaces) is impossible to violate accidentally.
- Per-dep metadata makes the replacement plan a first-class concern; "we'll deal with vendor lock later" becomes the explicit plan, not a hope.
- License-tier integration ensures Tier 3 deps are never adopted without review.

### Negative

- Per-axis matrix needs maintenance as new surfaces emerge; mitigated by quarterly council review.
- Some "external acceptable" deps require evaluation before adoption (maturity bar); slower than free-for-all.
- Per-dep metadata lift is real for the initial catalog migration.

### Operational

- On-call: `EVT-BUILD-VS-BUY-DENY` alerted weekly to council; `EVT-VENDOR-DEP-MATURITY-DROPPED` alert if a dep changes maturity tier.
- Runbooks: `runbooks/external-dep-onboarding.md`, `runbooks/in-house-replacement-trigger.md`, `runbooks/dep-replacement-execution.md`.
- CI: `oya-governance-build-vs-buy` is a P0 lane.
- Per-quarter audit: vendor-partner-ledger reviewed; deps with stale `replacement_trigger` re-evaluated.

---

## Alternatives considered

### Alternative A — License-only enforcement (ADR-0013), no build-vs-buy matrix

- **Pros:** simpler.
- **Cons:** an Apache-2 dep can still violate substrate-forking (ADR-0001); license is necessary but not sufficient.
- **Rejected because:** ADR-0001 cohesion requires substrate discipline.

### Alternative B — Per-PR build-vs-buy review (no matrix)

- **Pros:** no matrix maintenance.
- **Cons:** reviewer judgment varies; legacy corpus shows drift.
- **Rejected because:** failure mode demonstrated.

### Alternative C — Pure in-house (no external deps)

- **Pros:** maximum sovereignty.
- **Cons:** unrealistic; we cannot reimplement Postgres, Kafka, Envoy, Wasmtime, Cosign at scale.
- **Rejected because:** infeasible.

---

## Open questions

1. **Q1.** Maturity bar definition — concrete metrics (release cadence, contributor count, uptime in flagship deployments) or council judgment? Default: judgment with metric inputs. → owner: `council-architecture`.
2. **Q2.** Replacement-plan enforcement — do we require an issue-level tracking? Default: catalog-only initially; promote to issue when trigger nears. → ADR-0019.
3. **Q3.** "In-house obligatory" promotion of an "external acceptable" surface (e.g. when scale demands an in-house Kafka) — what's the protocol? Default: ADR amendment + per-dep replacement plan execution. → ADR-0019.
4. **Q4.** Per-pack vendor partners (ADR-0010) — does the matrix cover vendor partners, or only product-code deps? Default: product-code only; pack vendor partners covered by per-pack policy. → ADR-0010.

---

## References

- `docs/PRD.md` §3.1 commitment 4 (in-house build over external dep when the dep is not axum-class mature)
- `docs/TOOLCHAIN.md` §3 (language-stack matrix — agnostic of legacy), §6 (toolchain decision flow chart), §7 (license manifest)
- `docs/VENDOR-PARTNER-LEDGER.md` (per-dep metadata target)
- `docs/CONTRADICTION-LEDGER.md` LEDG-004 (license posture conflict), LEDG-013 (Foundry/Furnace mismatch as build-vs-buy edge case)
- ADR-0001 (cohesion), ADR-0011 (catalog hosts external_deps metadata), ADR-0013 (license tier feeds build-vs-buy decision), ADR-0015 (per-microservice crate roles inform matrix), ADR-0019 (catalog protocol)
