---
doc_class: Reference
shape: index
length_cap: 600
authority_tier: 3
status: Accepted
date: 2026-05-12
purpose: |
  Concrete inventory of every workspace crate (140 total as of 2026-05-12),
  classified GREEN / AMBER / RED against
  `docs/standards/crate-naming-convention.md` §2 grammar.
  Companion to the rename plan `docs/plans/rename-plan-2026-05-12.md` and the
  fitness lane spec `.omc/governance-lanes/architecture-conventions.md`.
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/plans/rename-plan-2026-05-12.md
  - .omc/governance-lanes/architecture-conventions.md
related_adrs:
  - ADR-0015
  - ADR-0017
doc_status: published
---

# Convention Audit — 2026-05-12

## 0. Method

Inventory snapshot: `ls crates/` at workspace root, 2026-05-12,
140 directory entries. Each name parsed by the grammar in
[`docs/standards/crate-naming-convention.md`](../standards/crate-naming-convention.md)
§2. Compound features cross-checked against the registry in §6 of the same
standard. No source files read; the audit is purely on package names and
on the per-crate `Cargo.toml` shape (sampled at 8 crates spanning kernel /
api / app / adapter / runtime / tooling).

Bucket definitions (verbatim from the standard):

- **GREEN** — grammar fully satisfied: 4 or 5 segments, role token present,
  capability tail iff `role = adapter`, single-token feature OR
  ADR-registered compound feature, name matches directory.
- **AMBER** — grammar satisfied **except** one advisory-only deviation:
  6-segment crate with a registered compound feature, OR a registered
  compound feature that has not yet been formally added to a per-crate
  `[package.metadata.oya]` block. Lane surfaces an advisory PR comment;
  does not block merge.
- **RED** — one or more BLOCKER violations: missing role token, headless
  adapter, kernel-with-capability, multi-token feature absent from the
  compound registry, 3-segment name (no feature), capability tail
  >2 tokens, or compound feature itself >2 tokens. Lane blocks merge.

## 1. Totals (n = 140)

| Bucket | Count | Share |
|---|---:|---:|
| GREEN | 81 | 57.9 % |
| AMBER | 22 | 15.7 % |
| RED | 37 | 26.4 % |

The 37-RED total reflects the **strict compound-feature gate**: any
multi-token feature MUST appear in the workspace compound-feature registry
(crate-naming-convention §6 / §7.1). Many foundry-fitness kernels carry
two-token names that read as a single semantic unit ("constitution-cite",
"runbook-freshness", "supply-chain") but have never been registered as
compounds. The rename plan offers two paths to drain the RED bucket: (a)
admit each into the registry by ADR (smallest disruption); (b) collapse
the foundry-fitness family under a single `fitness` feature with a
capability tail (largest semantic gain). See
[`docs/plans/rename-plan-2026-05-12.md`](../plans/rename-plan-2026-05-12.md).

## 2. GREEN inventory (n = 81)

These crates pass the grammar without changes. They still need a
`[package.metadata.oya]` block per the standard §7 — that work is tracked
as AMBER-metadata in the rename plan, not as a per-crate rename row.

### 2.1 Cloud (23 GREEN)

```
oya-cloud-billing-app                oya-cloud-billing-kernel
oya-cloud-capacity-kernel            oya-cloud-cell-app
oya-cloud-compute-kernel             oya-cloud-data-kernel
oya-cloud-dcops-kernel               oya-cloud-finops-api
oya-cloud-finops-kernel              oya-cloud-iam-api
oya-cloud-iam-kernel                 oya-cloud-kms-api
oya-cloud-kms-kernel                 oya-cloud-marketplace-kernel
oya-cloud-network-kernel             oya-cloud-observability-api
oya-cloud-observability-kernel       oya-cloud-region-api
oya-cloud-region-kernel              oya-cloud-resource-kernel
oya-cloud-storage-kernel             oya-cloud-surface-kernel
```

### 2.2 Foundry (17 GREEN)

```
oya-foundry-adapter-kernel           oya-foundry-bypass-kernel
oya-foundry-capability-kernel        oya-foundry-catalog-kernel
oya-foundry-eval-app                 oya-foundry-eval-kernel
oya-foundry-evidence-adapter-file    oya-foundry-evidence-kernel
oya-intelligence-mdbook-kernel            oya-foundry-openapi-kernel
oya-intelligence-policy-api               oya-foundry-policy-kernel
oya-intelligence-rag-api                  oya-intelligence-registry-api
oya-foundry-run-adapter-file         oya-foundry-run-kernel
oya-foundry-step-adapter-file        oya-foundry-step-kernel
```

### 2.3 Platform (18 GREEN)

```
oya-platform-cell-kernel             oya-platform-dsr-app
oya-platform-dsr-kernel              oya-platform-eventing-adapter-file
oya-platform-eventing-app            oya-platform-eventing-kernel
oya-platform-identity-api            oya-platform-identity-app
oya-platform-identity-kernel         oya-platform-metering-app
oya-platform-metering-kernel         oya-platform-observability-adapter-tracing
oya-platform-observability-kernel    oya-platform-residency-kernel
oya-platform-secrets-adapter-file    oya-platform-secrets-kernel
oya-platform-tenant-api              oya-platform-tenant-kernel
```

### 2.4 Workspace (23 GREEN)

```
oya-workspace-calendar-kernel        oya-workspace-chat-api
oya-workspace-chat-kernel            oya-workspace-collab-runtime-kernel
oya-workspace-dlp-kernel             oya-workspace-docs-kernel
oya-workspace-drive-api              oya-workspace-drive-kernel
oya-retention-dsr-kernel             oya-workspace-ediscovery-kernel
oya-workspace-forms-api              oya-workspace-forms-kernel
oya-workspace-mail-kernel            oya-workspace-meet-api
oya-workspace-meet-kernel            oya-workspace-notes-kernel
oya-workspace-recordings-kernel      oya-workspace-retention-kernel
oya-workspace-sheets-kernel          oya-workspace-sites-kernel
oya-workspace-slides-kernel          oya-workspace-tasks-kernel
oya-workspace-translate-kernel
```

> Note: `oya-workspace-collab-runtime-kernel` parses as feature
> `collab-runtime` + role `kernel` (registered compound). It is **NOT** a
> `role = runtime` crate; the token `runtime` is part of the feature noun
> (collaboration-runtime substrate). This is exactly the kind of compound
> the registry exists to disambiguate.

## 3. AMBER inventory (n = 22)

AMBER crates pass grammar but use a registered compound feature; they
inherit a `compound = true` metadata requirement and an ADR-cite obligation
in their `[package.metadata.oya]` block. The lane surfaces an advisory
PR comment; no rename.

| Crate | Compound feature | ADR to cite |
|---|---|---|
| `oya-cloud-billing-tax-app` | `billing-tax` | ADR-CLD-001 (cloud sub-features) |
| `oya-cloud-compute-functions-api` | `compute-functions` | ADR-CLD-001 |
| `oya-cloud-compute-k8s-api` | `compute-k8s` | ADR-CLD-001 |
| `oya-cloud-compute-vm-api` | `compute-vm` | ADR-CLD-001 |
| `oya-cloud-network-dns-api` | `network-dns` | ADR-CLD-001 |
| `oya-cloud-network-lb-api` | `network-lb` | ADR-CLD-001 |
| `oya-cloud-network-vpc-api` | `network-vpc` | ADR-CLD-001 |
| `oya-cloud-storage-block-api` | `storage-block` | ADR-CLD-001 |
| `oya-cloud-storage-object-api` | `storage-object` | ADR-CLD-001 |
| `oya-foundry-cargo-prefix-kernel` | `cargo-prefix` | ADR-FND-007 |
| `oya-platform-audit-chain-adapter-file` | `audit-chain` + 6 segments | ADR-GOV-002 (audit-chain) |
| `oya-platform-audit-chain-app` | `audit-chain` | ADR-GOV-002 |
| `oya-platform-audit-chain-kernel` | `audit-chain` | ADR-GOV-002 |
| `oya-platform-object-graph-api` | `object-graph` | ADR-PLT-004 |
| `oya-platform-object-graph-kernel` | `object-graph` | ADR-PLT-004 |
| `oya-platform-policy-cedar-api` | `policy-cedar` | ADR-PLT-005 |
| `oya-platform-policy-cedar-kernel` | `policy-cedar` | ADR-PLT-005 |
| `oya-platform-regional-pack-kernel` | `regional-pack` | ADR-PLT-006 |
| `oya-platform-regulatory-pack-api` | `regulatory-pack` | ADR-PLT-006 |
| `oya-workspace-address-book-kernel` | `address-book` | ADR-WSP-002 |
| `oya-workspace-document-format-kernel` | `document-format` | ADR-WSP-002 |
| `oya-workspace-trust-portal-kernel` | `trust-portal` | ADR-WSP-002 |

The lone 6-segment crate `oya-platform-audit-chain-adapter-file` is admitted
under the "audit-chain is a registered compound noun" rule (governance
substrate), but the standard's §2.1 strongly prefers ≤5 segments — see the
rename plan for the alternative `oya-platform-audit-adapter-chain-file`.

## 4. RED inventory (n = 37)

Each row is a BLOCKER until either (a) the crate is renamed per the plan,
or (b) the workspace compound-feature registry is extended by ADR. See the
rename plan for the recommended path per row.

| # | Current name | Violation | Class |
|---:|---|---|---|
| 1 | `oya-foundation-app` | 3 segments (no feature segment) | TOOSHORT |
| 2 | `oya-intelligence-api` | 3 segments (no feature segment) | TOOSHORT |
| 3 | `oya-intelligence-api-semver-kernel` | role token `api` precedes capability; tail = `semver-kernel` is 2 tokens, but `kernel` is a role, not a capability | ROLE-AS-CAP |
| 4 | `oya-foundry-adr-citation-kernel` | compound feature `adr-citation` not registered | NEW-COMPOUND |
| 5 | `oya-foundry-adr-index-kernel` | compound feature `adr-index` not registered | NEW-COMPOUND |
| 6 | `oya-foundry-authority-cohesion-kernel` | compound feature `authority-cohesion` not registered | NEW-COMPOUND |
| 7 | `oya-foundry-brand-residue-kernel` | compound feature `brand-residue` not registered | NEW-COMPOUND |
| 8 | `oya-foundry-claim-ceiling-kernel` | compound feature `claim-ceiling` not registered | NEW-COMPOUND |
| 9 | `oya-foundry-cloud-mutation-kernel` | compound feature `cloud-mutation` not registered | NEW-COMPOUND |
| 10 | `oya-foundry-codeowners-mirror-kernel` | compound feature `codeowners-mirror` not registered | NEW-COMPOUND |
| 11 | `oya-foundry-cohesion-fitness-kernel` | compound feature `cohesion-fitness` not registered | NEW-COMPOUND |
| 12 | `oya-foundry-constitution-cite-kernel` | compound feature `constitution-cite` not registered | ~~NEW-COMPOUND~~ SUNSET 2026-05-15 — crate deleted in commit `526e4bf` (strike: retire docs/CONSTITUTION.md and its enforcement crate); row preserved for historical audit integrity |
| 13 | `oya-foundry-cost-budget-kernel` | compound feature `cost-budget` not registered | NEW-COMPOUND |
| 14 | `oya-foundry-data-class-fitness-kernel` | 3-token feature exceeds cap | LONG-FEATURE |
| 15 | `oya-foundry-doc-catalog-kernel` | compound feature `doc-catalog` not registered | NEW-COMPOUND |
| 16 | `oya-foundry-documentation-system-kernel` | compound feature `documentation-system` not registered | NEW-COMPOUND |
| 17 | `oya-foundry-glossary-coverage-kernel` | compound feature `glossary-coverage` not registered | NEW-COMPOUND |
| 18 | `oya-foundry-glossary-vocabulary-kernel` | compound feature `glossary-vocabulary` not registered | NEW-COMPOUND |
| 19 | `oya-foundry-license-policy-kernel` | compound feature `license-policy` not registered | NEW-COMPOUND |
| 20 | `oya-foundry-mcp-gateway-kernel` | compound feature `mcp-gateway` not registered | NEW-COMPOUND |
| 21 | `oya-foundry-mobile-native-kernel` | compound feature `mobile-native` not registered | NEW-COMPOUND |
| 22 | `oya-foundry-placeholder-debt-kernel` | compound feature `placeholder-debt` not registered | NEW-COMPOUND |
| 23 | `oya-foundry-pr-traceability-kernel` | compound feature `pr-traceability` not registered | NEW-COMPOUND |
| 24 | `oya-foundry-pre-push-kernel` | compound feature `pre-push` not registered | NEW-COMPOUND |
| 25 | `oya-foundry-quality-lane-kernel` | compound feature `quality-lane` not registered | NEW-COMPOUND |
| 26 | `oya-foundry-raci-team-coverage-kernel` | 3-token feature exceeds cap | LONG-FEATURE |
| 27 | `oya-foundry-readme-doc-coverage-kernel` | 3-token feature exceeds cap | LONG-FEATURE |
| 28 | `oya-foundry-release-evidence-pack-kernel` | 3-token feature exceeds cap | LONG-FEATURE |
| 29 | `oya-foundry-runbook-freshness-kernel` | compound feature `runbook-freshness` not registered | NEW-COMPOUND |
| 30 | `oya-foundry-runbook-index-kernel` | compound feature `runbook-index` not registered | NEW-COMPOUND |
| 31 | `oya-foundry-slo-coverage-kernel` | compound feature `slo-coverage` not registered | NEW-COMPOUND |
| 32 | `oya-foundry-supply-chain-kernel` | compound feature `supply-chain` not registered | NEW-COMPOUND |
| 33 | `oya-foundry-typescript-workspace-kernel` | compound feature `typescript-workspace` not registered | NEW-COMPOUND |
| 34 | `oya-foundry-vendor-contract-recency-kernel` | 3-token feature exceeds cap | LONG-FEATURE |
| 35 | `oya-platform-data-boundary-kernel` | compound feature `data-boundary` not registered | NEW-COMPOUND |
| 36 | `oya-tooling-agent-read` | no role token (terminal `read` is not a role) | NO-ROLE |
| 37 | `oya-tooling-cli-dev-runtime` | `cli` + `dev` + `runtime` ⇒ role `cli` (or `runtime`) but with 2-token capability tail | LONG-CAPTAIL |

### 4.1 Class summary

| Class | Count | Resolution path |
|---|---:|---|
| NEW-COMPOUND | 28 | Either extend compound-feature registry (low effort, 1 ADR), OR rename per plan (high effort, breaks dep graph). |
| LONG-FEATURE | 4 | MUST rename — registry cannot admit 3-token features. |
| TOOSHORT | 2 | Insert feature segment (`oya-foundation-composition-app`, `oya-intelligence-policy-api`-style); foundation is the singleton composition root. |
| ROLE-AS-CAP | 1 | Re-parse: `oya-intelligence-api-semver-kernel` → role `kernel`, feature `api-semver`. The lane error is a parser artifact (eager role match on `api`); registry-admit `api-semver` and refine parser. |
| NO-ROLE | 1 | `oya-tooling-agent-read` needs `role = cli` and capability `read`: `oya-tooling-agent-cli-read`. |
| LONG-CAPTAIL | 1 | `oya-tooling-cli-dev-runtime` — drop `dev` from capability, keep as `oya-tooling-cli-runtime` OR re-parse as feature=`cli-dev`+role=`runtime` and admit `cli-dev` to compound registry. |

### 4.2 ROLE-AS-CAP parser caveat

`oya-intelligence-api-semver-kernel` exposes a real ambiguity in the grammar's
left-to-right role match: `api` is both a role AND a feature-segment of
"api-semver". The pragmatic fix is to make the BNF **prefer the rightmost
role token over the leftmost** when both parses validate. The standard
will note this; the lane uses the rightmost-role-token parser. Under
that parse, `api-semver` becomes the feature and is added to the
compound-feature registry. Marking this row as still RED for now (until
the parser refinement lands) is intentional.

## 5. Per-crate `Cargo.toml` shape audit (sampled, n = 8)

All 8 sampled `Cargo.toml` files (one each from kernel / api / app /
adapter / runtime / tooling — see "Method") use
`edition.workspace = true` / `version.workspace = true` /
`rust-version.workspace = true` correctly. None contain a
`[package.metadata.oya]` block. **140 crates** therefore carry an
AMBER-metadata obligation: add the block before the lane's metadata sub-check
is turned to BLOCKER (proposed cutover: 2026-Q3).

Sample manifest gaps observed:

- `oya-tooling-agent-read/Cargo.toml` omits `license`, `publish = false`,
  and `rust-version.workspace = true`. Treat as a separate row in the
  rename plan rather than folding into the naming RED bucket.
- All adapter crates correctly carry the capability suffix in the
  `[lib] name = "..."` snake-case form (`oya_foundry_evidence_adapter_file`).

## 6. Biggest tension surfaced

The compound-feature registry is the single most consequential design
decision in the standard. Two viable policies survive review:

- **Policy A — Strict registry.** Every multi-token feature requires an
  ADR row and a `[workspace.metadata.oya] compound_features` entry. The
  audit shows 37 RED today; an aggressive single-ADR pass admits 28 of
  them and reduces RED to 9. The remaining 9 are LONG-FEATURE (4) +
  TOOSHORT (2) + NO-ROLE (1) + LONG-CAPTAIL (1) + ROLE-AS-CAP (1) —
  these MUST be renamed; the count is low enough for manual treatment.
- **Policy B — Fitness-feature umbrella.** Collapse the foundry-fitness
  kernel family under a single `fitness` feature with a capability tail
  per check (`oya-governance-kernel` library +
  `oya-governance-{constitution-cite|runbook-freshness|...}-kernel`
  capability crates, or a single multi-bin lane runner). This destroys
  the 28-row NEW-COMPOUND tail at the cost of a one-time large refactor.
  See the rename plan for the cost-benefit.

The audit recommends **Policy A for the current cutover**, with Policy B
queued as a follow-on consolidation ADR (ADR-FND-008, proposed). Policy A
ships a single ADR that registers 28 compound features; Policy B is a
multi-PR refactor touching every foundry-fitness consumer (e.g.
`oya-tooling-cli-dev-runtime` imports each one explicitly).

## 7. Open question for user adjudication

The audit defers a single decision to the user before the rename plan
gets executed:

> **Q.** Should the foundry-fitness kernel family stay as discrete
> `oya-foundry-<noun>-kernel` crates (28 of them, admitted to the compound
> registry by a single ADR) — or be collapsed under a `fitness` feature
> umbrella (`oya-governance-<noun>-kernel`) with no compound feature?
> The grammar accepts either; the cost is registry-extension vs. one-time
> mass-rename + import-rewrite.

The open question is also written to
[`/Users/jasonlee/oyatie/.omc/plans/open-questions.md`](../../.omc/plans/open-questions.md).

## 8. Cross-references

- Grammar: [`docs/standards/crate-naming-convention.md`](../standards/crate-naming-convention.md)
- Layering rules: [`docs/standards/clean-architecture.md`](../standards/clean-architecture.md)
- Rename plan: [`docs/plans/rename-plan-2026-05-12.md`](../plans/rename-plan-2026-05-12.md)
- Lane spec: [`.omc/governance-lanes/architecture-conventions.md`](../../.omc/governance-lanes/architecture-conventions.md)
- Hyperscaler reference: [`docs/research/hyperscaler-best-practices-2026-05-12.md`](../research/hyperscaler-best-practices-2026-05-12.md)
- LTS pins: [`docs/research/lts-versions-verified-2026-05-12.md`](../research/lts-versions-verified-2026-05-12.md)
