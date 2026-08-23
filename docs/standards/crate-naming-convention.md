---
purpose: "Canonical, machine-checkable grammar for every `oyatie-*` Cargo crate name and every `[package.metadata.oya]` block in the oyatie workspace."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 500
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Canonical, machine-checkable grammar for every `oyatie-*` Cargo crate name and
  every `[package.metadata.oya]` block in the oyatie workspace. Resolves the
  forward-reference left by `docs/standards/code-style-rust.md` §5 (naming
  conventions) and binds the `governance-naming-convention` lane.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-naming-convention
companion_docs:
  - docs/standards/code-style-rust.md
  - docs/standards/doc-style.md
  - docs/plans/rename-plan-2026-05-12.md
  - .omc/governance-lanes/naming-convention.md
  - docs/research/hyperscaler-best-practices-2026-05-12.md
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0053
  - ADR-0054
authority_chain_declaration: |
  /specs/decision-principles.json + /specs/forbidden-operations.json > docs/AGENTS.md > docs/standards/code-style-rust.md
  > THIS DOC > .omc/governance-lanes/naming-convention.md
---

# Crate Naming Convention

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

This standard operates within the [`decision-principles.json`](../../specs/decision-principles.json) + [`forbidden-operations.json`](../../specs/forbidden-operations.json)
frame (architecture decision principles, ADR-0015 flat crates, ADR-0017 `oyatie-` prefix) and
downstream of [`docs/standards/code-style-rust.md`](code-style-rust.md) §5.
Every `oyatie-*` crate path under `crates/` MUST conform to the grammar in §2.
The lane [`governance-naming-convention`](../../.omc/governance-lanes/naming-convention.md)
mechanically enforces it. Severity = **BLOCKER**.

This standard ports the convergent hyperscaler practice for crate naming:
AWS publishes [`aws-sdk-<service>`](https://github.com/awslabs/aws-sdk-rust) for
service SDKs and [`aws-smithy-<role>`](https://crates.io/crates/aws-smithy-runtime)
for framework-internal crates — a strict three-segment grammar with a
controlled role enum. Microsoft publishes [`azure_<service>`](https://github.com/Azure/azure-sdk-for-rust)
with a one-crate-per-service rule per the
[Azure SDK Rust Guidelines](https://azure.github.io/azure-sdk/rust_introduction.html).
Google's Rust workspace presence (`google-cloud-<service>`, e.g.
[`google-cloud-storage`](https://crates.io/crates/google-cloud-storage)) follows
the same shape. Oracle's OCI Rust surface is too narrow to constitute a precedent.
The shared pattern is: **fixed top-level prefix → product/context segment →
service/role segment → optional capability tail**. Oyatie adopts the same
discipline, expressed as a BNF below.

Sources scanned: [`hyperscaler-best-practices-2026-05-12.md`](../research/hyperscaler-best-practices-2026-05-12.md)
Domain 3 (workspace structure) and Domain 4 (CI/CD tooling);
[Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html);
[Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html);
[cargo-deny configuration](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html);
[cargo-semver-checks](https://crates.io/crates/cargo-semver-checks);
[AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust);
[Azure SDK for Rust](https://github.com/Azure/azure-sdk-for-rust);
[Google Cloud Rust](https://github.com/googleapis/google-cloud-rust).

## 1. Vocabulary

The keywords MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119)
and [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) when, and only when,
they appear in all capitals.

## 2. Canonical grammar (BNF) — v4.1 flat µservice (per ADR-0056 v4.1)

> **v4.1 supersedes v4.** This section is updated per ADR-0056 v4.1
> (accepted 2026-05-13). The v4 `shared|vertical` binary is **retired** — the flat
> µservice catalog makes every µservice independent; the `shared` literal is dropped
> from slot2. The v3 BNF (`oyatie-<context>-<feature>-<role>[-<capability>]`) is also
> retired. v4.1 is the canonical grammar for all `oyatie-*` crates from Shard 1 forward.

The crate-name grammar, evaluated left-to-right on the package-name kebab string:

```bnf
crate          ::= "oya" "-" microservice ( "-" bc-tokens )? "-" layer
                 | "oya" "-" "check" "-" rule-name

microservice   ::= kebab-token ( "-" kebab-token )*    (* 1..3 tokens; registry-validated *)

bc-tokens      ::= kebab-token ( "-" kebab-token )*    (* 0..N; OPTIONAL *)

layer          ::= "kernel" | "domain" | "usecase" | "app"
                 | "adapter" | "infrastructure"
                 | "cli" | "rest" | "grpc"
                 | "worker" | "sdk" | "api"

rule-name      ::= kebab-token ( "-" kebab-token )*    (* 1..4 tokens *)

kebab-token    ::= [a-z] [a-z0-9]*
```

**Parser rule**: split on `-`; LAST token = layer (one of 12); SECOND token
(after `oyatie-`) = registered µservice name; remaining middle tokens (if any) =
optional BC tokens. `check-*` crates are exempt from this grammar.

**BC optionality rule**: BC slot is OPTIONAL. Omit when the µservice has a single
concept at the layer (e.g., `medical-domain`, `tenancy-kernel`). Include
when the µservice has multiple BC-level splits at the same layer (e.g.,

Constraints (the lane `check-architecture-cli` verifies all of them):

1. **Segment count.** Total segments (counting `oya` as segment 1) MUST be `>=3`
   (microservice + layer minimum); NO upper bound.
2. **Slot-2 µservice enum.** MUST be a µservice name registered in
   `[workspace.metadata.oyatie.microservices]`. Adding a µservice is a **1-ADR action**.
   The literals `shared`, `platform`, `vertical` are **retired** and must NOT appear
   as slot-2 values.
3. **Layer enum (closed, 12 values).** LAST token MUST be one of 12 canonical
   values per ADR-0056 §"12-Value Layer Enum". Adding a layer is a **1-ADR action**.
4. **Capability tail.** REQUIRED for `role = adapter` (per ADR-0015 §3:
   adapters bind a kernel trait to *one* provider/capability, and the
   provider/capability identity belongs in the crate name). OPTIONAL for
   every other role. Forbidden for `role = kernel` (kernels are by
   definition capability-agnostic). The capability MUST be the *terminal*
   segments — no role token may follow a capability.
5. **Feature locality.** `feature` MUST be 1..3 kebab-tokens. Multi-token
   feature names (e.g. `audit-chain`, `policy-cedar`, `object-graph`,
   `data-class`, `release-evidence-pack`) are admitted as compound features
   ONLY when one of the following holds: (a) the feature is a proper
   noun the rest of `docs/` already cites (audit-chain, policy-cedar,
   object-graph); (b) the feature names a multi-token external referent
   (vendor-contract-recency, raci-team-coverage). A unilateral compound
   that fails both tests MUST be renamed; the lane flags it RED.
6. **Bin-only tooling exemption.** A crate that ships ONLY a `[[bin]]` and
   has no library surface MAY use `role = cli` or `role = runtime` with no
   capability tail, and MAY collapse the `feature` segment to a single token.
   See §5.

The package name (`[package] name`) MUST equal the directory name. The
`[lib]` `name` field, when present, MUST equal the package name with `-`
replaced by `_` (per Cargo's library-name rule).

## 3. Microservice registry — semantic table (selected examples)

The full registry lives in `[workspace.metadata.oyatie.microservices]` in the root
`Cargo.toml`. Examples of registered µservices:

| Microservice | Definition | Layer examples |
|---|---|---|
| `cloud` | Cloud-provider plane: compute, storage, network, IAM, KMS, billing, region, observability. | `cloud-compute-vm-rest`, `cloud-iam-kernel` |
| `ontology` | Palantir-Ontology-equivalent information adapter: typed entities + links + actions + functions, audit-chain, RLS, jurisdiction overlays. Replaces retired `object-graph`. | `ontology-entity-kernel`, `ontology-agent-gateway-rest` |
| `workflow` | Cross-µservice action/orchestration adapter: state machines, DAGs, approvals, SLA timers, escalations, handoffs. | `workflow-state-machine-domain`, `workflow-approvals-application` |
| `application` | B2B unified shell: tenants sign in; enable µservices à-la-carte. | `application-product-enablement-rest` |
| `messenger` | Concrete messaging µservice with strict personal/professional tenant and RBAC separation. | `messenger-domain`, `messenger-message-stream-usecase` |
| `mail` | Concrete mail µservice with strict personal/professional tenant and RBAC separation. | `mail-domain`, `mail-mailbox-store-usecase` |
| `community` | Concrete community µservice for groups, professional profile/graph, social/anonymous modes, and moderation. | `community-post-store-domain`, `community-social-domain` |
| `connector` | Integration-adapter substrate for external systems, OAuth/webhook brokerage, and connector catalog/runtime contracts. Not a product-group wrapper. | `connector-salesforce-adapter`, `connector-netsuite-adapter` |
| `tenancy` | Tenant lifecycle, multi-tenant isolation, RLS enforcement. | `tenancy-kernel`, `tenancy-adapter` |
| `identity` | Authentication, STS token issue, PKCE+nonce, SSO binding. | `identity-kernel`, `identity-rest` |
| `medical` | Electronic medical records, FHIR R5, clinical workflows. | `medical-encounter-domain` |
| `payroll` | Salary calculation, 4대보험 EDI, 연말정산, payroll journal. | `payroll-ledger-application` |
| `payments` | Payment rails, settlement, reconciliation. | `payments-ledger-application` |

Adding a µservice REQUIRES: (1) an ADR proposing the name + justification,
(2) a row in `[workspace.metadata.oyatie.microservices]`, (3) the lane regex updated
in the naming-convention fitness lane.

## 4. Role enum — semantic table

The canonical role taxonomy follows ADR-0105 + ADR-0106. Current inward dependency shape:

`kernel <- domain <- usecase <- app`, with `adapter`/`rest` as explicit ports/surfaces that an `app` may compose. `app -> app` is forbidden; shared orchestration belongs in `usecase`.

| Role | Layer | Surface | Capability tail | Imports allowed |
|---|---|---|---|---|
| `kernel` | innermost; pure-domain types, no I/O, no async, no provider deps | library only (`[lib]`) | **forbidden** | no project-internal crates except explicitly whitelisted base kernels |
| `domain` | business invariants; pure functions on kernel types | library only | optional | `kernel` |
| `usecase` | application/use-case orchestration over domain ports; no concrete adapters | library only | optional | `kernel`, `domain` |
| `app` | deployable/composition root wiring usecases, adapters, and surfaces | bin plus thin library shim when needed | optional | `kernel`, `domain`, `usecase`, `adapter`, `rest`; never another `app` |
| `adapter` | provider implementations bound to one capability | library only | **REQUIRED** | `kernel`, `domain`, `usecase`; never another adapter |
| `rest` / `grpc` / `api` | process-boundary inputs bound to a feature | library + optional bin | optional | `kernel`, `domain`, `usecase`, `app` only when the API intentionally calls the composition-root surface |
| `worker` | scheduled / queue-driven process | library + bin | optional | `kernel`, `domain`, `usecase`, `app` only when the worker intentionally calls the composition-root surface |
| `cli` | developer/agent terminal tool (not deployed) | bin | optional | lower layers plus explicit app surfaces when the CLI is an operator wrapper |
| `sdk` | externally-published client surface (consumer-facing) | library only | optional | generated/contract types only; no server app imports |
| `infrastructure` | framework/runtime support that is not a deployable app | library only | optional | `kernel`, `domain`, `usecase`, `adapter` as justified |

Legacy `application`, `runtime`, and `test` role records are transitional compatibility rows only. New records use `usecase`/`app` per ADR-0106.

## 5. Capability tail — required-when, forbidden-when

| Role | Capability tail | Examples |
|---|---|---|
| `kernel` | **forbidden** | `intelligence-evidence-kernel` (no tail) |
| `domain` | optional | n/a in workspace yet |
| `app` | optional | `cloud-billing-tax-app` (tail = `tax` on feature `billing`) |
| `api` | optional | `cloud-compute-vm-api` (tail = `vm` on feature `compute`) |
| `worker` | optional | n/a in workspace yet |
| `adapter` | **REQUIRED** | `intelligence-evidence-adapter-file` (file backend) |
| `runtime` | optional | `tooling-cli-dev-runtime` (tail = `dev`; see §6.1) |
| `cli` | optional | n/a in workspace yet |
| `sdk` | optional | n/a in workspace yet |

Compound capability tails (`-adapter-tracing`, `-adapter-file`) MUST be a
single token. Two-token capability tails are admitted ONLY when the
provider/backend name is itself multi-token externally (e.g.
`-adapter-azure-blob` if Azure Blob Storage support is added). The lane
flags 6-segment names AMBER and requires an ADR cite.

## 6. Compound / multi-token features

Workspace evidence (2026-05-12 inventory) records these multi-token
features as accepted compound nouns:

| Compound feature | Origin / referent | Crates using it |
|---|---|---|
| `audit-chain` | governance audit-chain doctrine (decision-principles.json DP-08) | kernel/app/adapter |
| `policy-cedar` | Cedar policy engine binding | kernel/api |
| `object-graph` | object-graph platform substrate | kernel/api |
| `regional-pack` | regional regulatory pack | kernel |
| `regulatory-pack` | regulatory-pack contract | api |
| `data-class` | data-classification fitness | (extended below) |
| `compute-vm` / `compute-k8s` / `compute-functions` | cloud-compute sub-surfaces | api |
| `storage-object` / `storage-block` | cloud-storage sub-surfaces | api |
| `network-vpc` / `network-dns` / `network-lb` | cloud-network sub-surfaces | api |
| `billing-tax` | cloud-billing tax sub-app | app |
| `address-book` / `document-format` / `trust-portal` / `collab-runtime` | workspace-axis sub-features | kernel |
| `agent-read` | tooling-agent read-only surface | bin-only tooling |
| `api-semver` | foundry fitness check on api semver | kernel |
| `cargo-prefix` / `cli-dev` | foundry / tooling sub-surfaces | kernel / runtime |
| `audit-chain-adapter` (= feature `audit-chain` + role `adapter`) | governance binding | adapter |

Rules:

1. A compound feature MUST appear in the above registry to count as GREEN.
   New compounds REQUIRE an ADR row plus a registry update.
2. Compound features that are simply a noun-phrase ("readme-doc-coverage",
   "vendor-contract-recency", "raci-team-coverage", "release-evidence-pack",
   "data-class-fitness") are AMBER if ≤5 segments total, RED if 6 segments.
   The recommended resolution is to fold the noun-phrase into a single
   token: `release-evidence-pack` → feature `release-pack`, capability
   `evidence`. See `docs/plans/rename-plan-2026-05-12.md`.
3. Compounds MUST NOT cross the role boundary: `governance-data-class-fitness-kernel`
   parses as feature=`data-class-fitness` (3 tokens) + role=`kernel`,
   exceeding the feature-token cap; rename per the plan.

### 6.1 The `tooling` exemption

The `tooling` context covers dev-time-only binaries with no library
surface published outside the workspace. The exemption is narrow:

- A `tooling` crate MAY ship ONLY a `[[bin]]`.
- A `tooling` crate MAY use feature=`cli` or feature=`agent` etc. without
  the compound-feature ADR requirement, because the audience is internal
  and the surface is non-load-bearing.
- A `tooling` crate that produces a multi-binary distribution (e.g.
  `tooling-cli-dev-runtime` ships two bins) MUST use `role = runtime`
  with a feature describing the runtime kind.

## 7. Cargo.toml requirements (per crate)

Every workspace member's `Cargo.toml` MUST include:

```toml
[package]
name             = "oyatie-<context>-<feature>-<role>[-<capability>]"
edition          .workspace = true
version          .workspace = true
rust-version     .workspace = true
license          = "Apache-2.0"
publish          = false                    # unless role = sdk

[lib]                                       # for non-bin-only crates
name = "<context>_<feature>_<role>[_<capability>]"   # underscored
path = "src/lib.rs"
doctest = false                              # unless explicitly opted in

[lints]
workspace = true                             # MANDATORY per workspace lints inheritance

[package.metadata.oya]
context     = "<context>"                   # MUST match name
role        = "<role>"                      # MUST match name
feature     = "<feature>"                   # MUST match name
capability  = "<capability>" | ""           # "" when none
layer       = <ADR-0015 layer enum>         # for cross-check with role
audit_chain = <bool>                        # true if crate emits to audit-chain
```

The `[package.metadata.oya]` block is the machine-readable inventory. The
lane parses it, cross-checks against the package name, and refuses any
mismatch.

### 7.1 Workspace-level registry

`Cargo.toml` (workspace root) MUST grow a `[workspace.metadata.oya]`
block that pins the closed enums:

```toml
[workspace.metadata.oya]
# microservices = open kebab registry; slot-2 of BNF v4.1.
# "shared", "platform", "vertical", "workspace" are RETIRED — do not add.
microservices = [
  "cloud", "foundry", "ontology", "workflow", "application",
  "mail", "messenger", "community", "connector", "tenant-rbac",
  "tenancy", "identity", "audit-chain", "eventing", "secrets",
  "observability", "kms", "policy", "search", "vector",
  "data-boundary", "finance-library", "capability-registry",
  "records", "ads", "analytics",
  "medical", "pharmacy", "healthcare-portal", "emergency", "clinical",
  "hr", "payroll", "accounting", "ats", "grc", "performance",
  "manufacturing", "logistics", "facility-ops", "procurement", "security",
  "payments", "insurance", "finance-quant",
  "dining", "cellar",
]
layers = ["kernel", "domain", "usecase", "app", "adapter", "infrastructure",
          "cli", "rest", "grpc", "worker", "sdk", "api"]
compound_bc_tokens = [
  "audit-chain", "policy-cedar", "regional-pack", "regulatory-pack",
  "compute-vm", "compute-k8s", "compute-functions",
  "storage-object", "storage-block",
  "network-vpc", "network-dns", "network-lb",
  "billing-tax", "address-book", "document-format",
  "trust-portal", "agent-read",
  "api-semver", "cargo-prefix", "cli-dev",
  "state-machine", "agent-gateway", "product-enablement",
]
```

Adding to any list REQUIRES an ADR cite in the workspace `CHANGELOG`.

## 8. Reserved suffixes (fixtures, not crates)

Fixture and test-support artifacts MUST be carried as either (a) `dev-dependencies`
inside the crate they support, or (b) integration-test directories
(`tests/`, `benches/`, `examples/`) inside the same crate. Standalone
fixture crates MUST use one of these terminal suffixes:

| Suffix | Use | Cargo placement |
|---|---|---|
| `-test` | shared test harness crate | `[dev-dependencies]` only |
| `-bench` | benchmark harness crate | `[dev-dependencies]` only |
| `-example` | example crate | `[[example]]` target on a host crate, preferred over a standalone crate |
| `-integration` | cross-crate integration-test scaffolding | `[dev-dependencies]` only |

Fixture crates MUST NOT appear in any production binary's dependency tree.
The lane fails the workspace if a fixture suffix appears as a normal
`[dependencies]` entry.

## 9. Hyperscaler practice mapping

| Practice | Source | Oyatie equivalent |
|---|---|---|
| `aws-sdk-<service>` strict three-segment grammar | [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust) | The BNF in §2 (analogous three-segment core, with `feature` + optional `capability` extension) |
| `aws-smithy-<role>` for framework-internal crates | [aws-smithy-runtime](https://crates.io/crates/aws-smithy-runtime) | The `foundry` context (engineering-platform crates) |
| `azure_<service>`, one crate per service | [Azure SDK Rust Guidelines](https://azure.github.io/azure-sdk/rust_introduction.html) | The `cloud` context's sub-features (`compute-vm`, `storage-object`, …) |
| `google-cloud-<service>` | [Google Cloud Rust](https://github.com/googleapis/google-cloud-rust) | The `cloud` context root-feature naming |
| `cargo-deny` `bans` for crate-name policy | [cargo-deny — bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html) | Lane mechanism in `.omc/governance-lanes/naming-convention.md` |
| `cargo-semver-checks` on every crate | [cargo-semver-checks](https://crates.io/crates/cargo-semver-checks) | Already in `code-style-rust.md` §8; complements naming lane |
| Rust API Guidelines — Naming | [API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html) | §10 in-crate identifier rules below |

## 10. In-crate identifier hygiene

Inside a crate, the standard tracks the
[Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html):

- Modules: `snake_case`, file-per-module (no `mod.rs`); enforced by
  `clippy::mod_module_files = deny` (already in `code-style-rust.md` §2.1).
- Types: `PascalCase`. Traits: noun phrases (`Provider`, `EventEmitter`),
  not `-able` adjectives.
- Functions / methods: `snake_case`, verb-first.
- Constants / statics: `SCREAMING_SNAKE_CASE`.
- Generic params: single capital (`T`, `E`, `R`) or descriptive PascalCase
  when ≥ 2 generics are in scope.
- Crate-root re-exports: `pub use` only the public surface; everything
  else is `pub(crate)`. Enforced by `unreachable_pub = warn` in `code-style-rust.md`.

## 11. Anti-patterns

1. **Adapter without capability tail.** A crate named `oyatie-*-adapter` is
   ambiguous (which provider?). Always include the capability tail:
   `-adapter-file`, `-adapter-tracing`, `-adapter-postgres`.
2. **Kernel with capability tail.** A kernel is by definition
   capability-agnostic; if the kernel needs a capability tail, it isn't a
   kernel — it's an adapter or an app.
3. **Headless adapter** (e.g. `intelligence-evidence-file`). The role token
   is missing; readers cannot tell whether the crate is a kernel with a
   file-format feature or an adapter binding evidence to a file backend.
4. **Compound feature without ADR.** Adding a fourth-token feature
   (`release-evidence-pack-kernel`, six segments total) bypasses the
   compound registry; the lane flags it RED.
5. **Crate name ≠ directory name.** Cargo allows it; the lane refuses it.
6. **Bin-only crate using `role = kernel`.** Kernels have no I/O. A
   bin-only crate is `cli` or `runtime`.
7. **Cross-context renames absent ADR.** Moving `platform-X-kernel` to
   `foundry-X-kernel` changes the data-class boundary and the
   audit-chain emission; an ADR is REQUIRED.

## 12. Sources scanned

- [`docs/research/hyperscaler-best-practices-2026-05-12.md`](../research/hyperscaler-best-practices-2026-05-12.md)
- [`docs/research/lts-versions-verified-2026-05-12.md`](../research/lts-versions-verified-2026-05-12.md)
- [`docs/standards/code-style-rust.md`](code-style-rust.md)
- [`docs/standards/doc-style.md`](doc-style.md)
- [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html)
- [Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Cargo Book — Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [cargo-deny — Bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html)
- [cargo-semver-checks](https://crates.io/crates/cargo-semver-checks)
- [cargo-public-api](https://github.com/cargo-public-api/cargo-public-api)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [aws-smithy-rs](https://github.com/smithy-lang/smithy-rs)
- [Azure SDK for Rust](https://github.com/Azure/azure-sdk-for-rust)
- [Azure SDK Rust Guidelines](https://azure.github.io/azure-sdk/rust_introduction.html)
- [Google Cloud Rust](https://github.com/googleapis/google-cloud-rust)
- [Firecracker](https://firecracker-microvm.github.io/) (AWS workspace structure precedent)
- [Hyperlight](https://opensource.microsoft.com/blog/2024/11/07/introducing-hyperlight-virtual-machine-based-security-for-functions-at-scale/) (Microsoft workspace structure precedent)
- ADR-0015 (flat crates), ADR-0017 (`oyatie-` prefix), ADR-0053, ADR-0054.
