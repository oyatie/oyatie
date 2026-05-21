---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-027-cedar-grammar-impl
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-security]
date: 2026-05-18
related_adrs: [ADR-0183, ADR-0205]
acceptance_lanes: [grammar-correctness, oya-vcs-promotion-readiness]
depends_on: [IP-025]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-027 — Cedar grammar (Lezer-based CodeMirror 6 pack)

## Goal

Cedar is oyatie's authz language (ADR-0183). Foundry tool definitions and Workflow Studio steps may attach Cedar policy fragments. We need a first-class CodeMirror 6 language pack — `@oya/cedar-cm6-grammar` — built on Lezer for syntax highlighting, bracket matching, fold ranges, and structural lint. The Cedar core's official AST is reused as the source-of-truth grammar definition (per ADR-0183) so any drift between editor and evaluator is impossible.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/cedar-cm6-grammar/package.json` | create | ~60 LoC; `@oya/cedar-cm6-grammar`, peer deps on `@codemirror/language`, `@lezer/highlight` |
| `clients/cedar-cm6-grammar/src/cedar.grammar` | create | ~180 LoC; Lezer grammar covering `permit`/`forbid`/`when`/`unless`/principal/action/resource clauses, attribute access, comparison operators |
| `clients/cedar-cm6-grammar/src/index.ts` | create | ~80 LoC; `cedar()` extension factory returning `LanguageSupport` |
| `clients/cedar-cm6-grammar/src/highlight.ts` | create | ~60 LoC; HighlightStyle mapping token classes |
| `clients/cedar-cm6-grammar/src/lint.ts` | create | ~120 LoC; structural lints (unused principal, unreachable forbid, etc.) |
| `clients/cedar-cm6-grammar/src/fold.ts` | create | ~40 LoC; fold ranges for block statements |
| `clients/cedar-cm6-grammar/tests/parse.test.ts` | create | ~220 LoC; 6 parser tests covering real Cedar fragments |
| `clients/cedar-cm6-grammar/tests/lint.test.ts` | create | ~120 LoC; 4 lint tests |
| `clients/cedar-cm6-grammar/tests/fixtures/oya-policies.cedar` | create | curated real-world Cedar fragments from `microservices/foundry/specs/cedar/` |
| `microservices/workflow-studio/decisions/ADR-0183.md` | append §"CM6 Cedar grammar pack landed" | +6 LoC |

## Code shape

`src/cedar.grammar` (excerpt):

```text
@top Policy { (PolicyStatement)* }

PolicyStatement {
  Effect "(" PrincipalClause "," ActionClause "," ResourceClause ")" Condition? ";"
}

Effect { @specialize<identifier, "permit" | "forbid"> }

Condition { (When | Unless)+ }
When { @specialize<identifier, "when"> Block }
Unless { @specialize<identifier, "unless"> Block }
Block { "{" Expression "}" }

@tokens {
  identifier { @asciiLetter (@asciiLetter | @digit | "_")* }
  String { '"' (![\\\n"] | "\\" _)* '"' }
  Number { @digit+ }
  LineComment { "//" ![\n]* }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `cedar_parses_permit_block` | tests/parse.test.ts | `permit(principal,action,resource);` parses with no errors |
| `cedar_parses_forbid_when_unless_chain` | tests/parse.test.ts | `forbid(...)when{...}unless{...};` parses correctly |
| `cedar_parses_attribute_access` | tests/parse.test.ts | `principal.role == "admin"` recognized |
| `cedar_highlights_keywords_as_keyword_class` | tests/parse.test.ts | `permit`, `forbid`, `when`, `unless` → keyword class |
| `cedar_recognizes_string_literals` | tests/parse.test.ts | `"admin"` → string token |
| `cedar_real_world_oya_policies_parse_clean` | tests/parse.test.ts | All fixtures under `tests/fixtures/oya-policies.cedar` parse without errors |
| `cedar_lint_warns_on_unreachable_forbid` | tests/lint.test.ts | Unreachable forbid → warning diagnostic |
| `cedar_lint_warns_on_unused_principal` | tests/lint.test.ts | Unused principal binding → warning diagnostic |
| `cedar_fold_ranges_match_blocks` | tests/parse.test.ts | Each `when {}` block produces one fold range |
| `cedar_grammar_matches_cedar_core_ast` | tests/parse.test.ts | Round-trip via `@oya/cedar-core` AST builder is identity |

Minimum 5 required; 10 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/cedar-grammar-correctness-{date}.json`
- Audit-chain seal: `oya audit-chain seal --kind grammar-correctness --pkg cedar-cm6-grammar --window 30d`
- Metrics: `oya_cedar_grammar_parse_errors_total`, `oya_cedar_grammar_lint_warnings_total{rule}`

## Rollback procedure

1. Revert ChangeSet for `clients/cedar-cm6-grammar/`.
2. Unpin the package from `clients/web-sveltekit/package.json` and any Workflow Studio integrations.
3. CM6 falls back to plain-text rendering for Cedar fragments (highlight lost; editor still usable).
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-025 — CodeMirror adapter (consumer).
- ADR-0183 — Cedar canonical.
- ADR-0205 — code editor canonical.

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate grammar-correctness --pkg cedar-cm6-grammar
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-studio
pnpm --filter @oya/cedar-cm6-grammar test
```

## Halt conditions

- Any real-world Cedar fixture fails to parse: STOP — grammar incomplete.
- Round-trip via Cedar core AST is non-identity: STOP — grammar drift.
- Highlighting misclassifies keywords: STOP — visual regression.

## Exit criteria

1. All 10 tests green.
2. `grammar-correctness` + `oya-vcs-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Pack published to internal npm registry.
5. ADR-0183 Cedar surface map updated to reference the grammar pack.

## Next IP

[`IP-017-leptos-canvas-scaffold.md`](IP-017-leptos-canvas-scaffold.md)

## References

- ADR-0183 — Cedar canonical.
- ADR-0205 — code editor.
- Cedar language spec — `https://docs.cedarpolicy.com/`.
- Lezer parser generator — `https://lezer.codemirror.net/`.
- @oya/cedar-core internal docs.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-027-cedar-grammar-impl.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
