# oya/governance — Routing Investigation (READ-ONLY audit)

Investigator: workflow-subagent
Date: 2026-06-06
Source tree: `/Users/jasonlee/Developer/source/oya/governance`
Method: `ls -R`, read README/manifest/PRD/ARCH/contracts, sampled catalog rows + IP code-shapes + Cedar/IaC artifacts. No files edited.

---

## TL;DR verdict

`oya/governance` is a **documentation/spec-stage microservice, NOT a working code implementation.** It is the *declared* policy/quality/fitness AUTHORITY (consistent with ADR-0363 "stays its own service" and ADR-0347 foundry-fitness→governance rename), but on disk it is **decision-debt / scaffolding**: a large, internally-consistent body of specs, ADRs, PRDs, IPs, contracts, Cedar fragments, IaC, and per-crate *catalog descriptors* — with **zero implemented Rust**.

- **Real `.rs` files: 0. Cargo.toml: 0. `src/crates/` directory: does not exist** (README/catalog *reference* `microservices/governance/src/crates/...` paths, but nothing is there).
- The "crates" exist **only as catalog YAML rows** (41 catalog files) describing crates that are `status: scaffolded` or `status: migrating`. The IP "Code Shape" Rust blocks are explicit `todo!()` stubs.
- File breakdown on disk: 117 yaml, 90 md, 8 json, 6 cedar, 1 proto, 1 Dockerfile template, 1 Helm `.tpl`, 1 Jenkinsfile. No source code, no build.

So: **a genuinely-*designed* service, but not yet a genuinely-*live* one.** Its validity is as an authoritative SPEC; as running software it is a shell.

## On the abuse-defence / acl-enforcement / admission crates (the prompt's named examples)

These do **not exist as crates here** — not even as catalog rows. There are **no `oya-governance-abuse-defence`, `-acl-enforcement`, or `-admission` crate dirs or catalog files.** The only literal hit is the string `oya-governance-abuse-defence-ux-floor` appearing inside prose in `ARCHITECTURE.md` / `compliance.md` / a journey doc — i.e. a referenced *lane name*, not an implemented crate. The actual catalogued bounded contexts are only four: `lane-runtime`, `policy-engine`, `evidence-emitter`, `aggregation-indexer` (plus a `bundled-check-crate` BC). The prompt's example names appear to be aspirational/other-service lane identifiers, not governance's real surface.

## What it IS (declared purpose)

Per `PRD.md` and `README.md`: the **CI-Fitness Substrate + Industry-Best-Practice Conformance Engine.** It is described as:
- the **"enforcement origin"** of the ADR-0133 6-axis conformance program (pipeline / directory / naming / standards / practices / policies), and
- the **"execution origin" of every CI fitness lane that gates every other oyatie µservice's PRs.**
- It bundles the ~50 historical `oya-check-*` crates (per ADR-0131 §governance + ADR-0132 bundle-dissolution) into one µservice. Owner: `axis-governance` (catalog rows still say legacy `axis-foundry`).
- Explicitly **"shared substrate, not a hero product"**; consumed by every other µservice (each PR runs through governance lanes) and by tenants only indirectly via a published conformance posture.

## What it DOES (intended responsibilities / primitives)

Four bounded contexts (all currently `crates: []` in manifest — i.e. declared, unbuilt):
1. **policy-engine** — rule-pack decision engine; Cedar-style allow/forbid composition; 6-axis per-axis aggregation; baseline-pin authority; **admission verdicts** ("is PR #N admissible against `dev`?"); industry-baseline drift detection. (Domain layer spec: pure decision logic, zero I/O.)
2. **lane-runtime** — runs the ~50 fitness lanes against a working tree; reads each µservice's `{PRD,catalog,slos,policy,contracts,specs}`.
3. **evidence-emitter** — emits signed, replayable Findings per violation; audit-chain seal events; auditor (SOC2/ISO27001/GDPR) query+replay surface.
4. **aggregation-indexer** — deterministically regenerates central indices (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`) from per-µservice sources; refuses hand-edits.

**Actually-present machine-readable artifacts** (the real, non-Rust substance):
- **Cedar policy fragments** (`cedar/policies.cedar`, `policy/*.cedar`): genuine ABAC policy code — tenant-scope / auditor-scope / ci-scope / public-read isolation for Finding+evidence reads, and permit/forbid rules for sharding-automation actions (ADR-0348). These are real, evaluatable policy text (intended for Envoy ext_authz), the closest thing to "working" here.
- **Contracts**: OpenAPI 3.2.0 (`governance.yaml` — lane status / findings / lane-runs / evidence replay / admission verdicts), AsyncAPI 3.1.0, proto3. Spec only, no server.
- **IaC**: Helm / Kustomize / Terraform-wrapper stubs, Kata runtime-classes, Istio waypoint policies, a distroless-rust Dockerfile, a Jenkinsfile. Declarative, no built image.
- Heavy compliance/governance documentation: threat-model, DPIA, capacity/cost models, failure-modes, 11-region regulatory packs, ~20 journey IPs.

## governance vs oya-ci — relationship (the core routing question)

**They are complementary, NOT redundant — clean control/execution split:**
- **governance DEFINES the gates** and the verdict semantics: it is the *authority* that says what "production-ready" means (the ~50 lanes, the 6-axis rules, severities/BLOCKER, baseline citations, admission permit/forbid logic). It owns the *policy-engine* that decides "is this PR admissible against `dev`?" and the *evidence-emitter* that signs/replays the audit trail.
- **oya-ci (and GitHub Actions / Jenkins / ArgoCD) RUN the gates**: per ARCH ADR-0346/0349, the local mirror `oya verify --ci-required` and CI must execute `oya gate run-all` (cargo fmt/check/clippy/nextest + governance lanes); Jenkins LTS augments GitHub Actions in self-hosted contexts, ArgoCD does GitOps CD. governance does not replace the runner; it is the rule/verdict/evidence authority the runner invokes.
- Net: **governance = the policy/quality AUTHORITY (defines lanes, rules, admission verdicts, signed evidence); oya-ci = the execution substrate (runs lanes, mirrors CI matrix).** ADR-0363 keeping governance "its own service" is coherent with this — it is the SSOT for *what* is enforced, separate from *where* it runs. Caveat: today governance is the *spec* of that authority; the actual lane-execution logic is unbuilt, so at runtime the boundary is still notional.

## Does it hold the AI-agent-platform primitives? NO.

There are **no agent-runtime / agent-sandbox / tool-execution / LLM / model-serving / inference primitives here.** The only "agent/agentic" hits are:
- "agentic-dev-team-optimization" and "agentic SLO-gated promotion" (ADR-0139) — i.e. governance gating PRs that may be *authored by* agents, and SLO-gated promotion of agent-produced changes. This is *governance OF agentic development*, not an agent execution platform.
- FR-01 mentions "PR author (agent or human)" — again the gated subject, not a hosted capability.

The AI-agent-platform primitives do **not** live in governance. governance is purely the policy/fitness/evidence authority.

## Crate inventory (real-vs-shell)

No crate is a real Rust implementation. Counts are of **catalog descriptors** (the only crate-level artifacts on disk):

| BC / group | Catalog rows | On-disk Rust | Status (per catalog/manifest) | Verdict |
|---|---|---|---|---|
| policy-engine (kernel,domain,usecase,adapter,rest,worker,sdk,app,api) | 9 | none | `scaffolded`, manifest `crates: []` | SHELL (spec only) |
| lane-runtime (×9 layers) | 9 | none | `scaffolded`, manifest `crates: []` | SHELL (spec only) |
| evidence-emitter (×9 layers) | 9 | none | `scaffolded`, manifest `crates: []` | SHELL (spec only) |
| aggregation-indexer (×9 layers) | 9 | none | `scaffolded`, manifest `crates: []` | SHELL (spec only) |
| bundled `oya-check-*` (data-class, lean-a1, lean-a2, license-policy, supply-chain) | 5 | none | `migrating-to-microservices-governance`; `prior_path: crates/oya-check-*` (live impl, if any, lives in the OLD path, not here) | SHELL here; possible real impl at legacy path outside this dir |
| **Total catalog rows** | **41** | **0 .rs / 0 Cargo.toml** | — | **0 real / 41 shell** |

Note: manifest declares 4 BC × 9 layers = 36 umbrella crates + 3 bundled-check (IP-001..IP-011 all marked `acceptance_status: ga`), but `ga` here means **doc/spec acceptance**, not shipped code — ARCH states outright: *"no Rust code, crate metadata, OpenTofu body, Helm chart, ArgoCD Application, or live infrastructure apply is part of this propagation."* The 3 bundled-check rows' `prior_path` points to `crates/oya-check-*` (a legacy location outside this microservice dir), so any working check logic predates this service and was not migrated in.

## Evidence anchors

- `manifest.json` — every BC `crates: []`; `version 0.1.0`; `owner axis-governance`.
- `README.md` lines 56–61 — references `src/crates/` with "~50 + 36 crates" that **do not exist on disk**.
- `catalog/oya-governance-policy-engine-domain.yaml` — `status: scaffolded`.
- `catalog/oya-check-supply-chain.yaml` — `status: migrating-to-microservices-governance`, `prior_path: crates/oya-check-supply-chain`.
- `IP-006-policy-engine-kernel-domain.md` — "Code Shape" Rust uses `todo!()` bodies (plan, not impl).
- `ARCH.md` (ADR-0339 + final "Implementation boundary") — "no Rust code ... is part of this propagation."
- `AUDIT-FINDINGS-2026-05-18.json` — verdict `THIN_IPS_ONLY` (the service's own audit flags its IPs as thin).
- `cedar/policies.cedar`, `policy/tenant-scope.cedar` — real Cedar ABAC fragments (the genuine machine-readable substance).
- `contracts/openapi/governance.yaml` — admission/findings/evidence API spec (3.2.0), no server.
- Grep: 0 hits for agent-runtime/sandbox/tool-exec/llm/inference primitives; "agentic" = dev-team/PR-promotion governance only.
