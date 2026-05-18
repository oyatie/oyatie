---
id: ADR-FORMS-0004
title: Conditional-logic + branching engine — CEL (Google Common Expression Language) over a declarative DAG; server-authoritative
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + council-architecture + ops-security
deciders: council-architecture, axis-forms, ops-security, council-design-system, axis-sdk
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0140, ADR-FORMS-0001]
related_specs: [/specs/products/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-02 + FR-12 + AC-03 + AC-15
  - microservices/forms/contracts/openapi/forms.openapi.yaml /components/schemas/BranchPredicate
  - microservices/forms/contracts/proto/forms.proto message BranchPredicate
  - microservices/forms/threat-model.md §"T-T-01" + §"T-D-02"
doc_status: published
---

# ADR-FORMS-0004: Conditional-logic + branching engine — CEL over a declarative DAG; server-authoritative

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Conditional-logic + branching is the single biggest expressiveness lever for a form builder: every competitor (Typeform, Jotform, SurveyMonkey, Qualtrics) treats it as a top-3 feature. The engine must:

1. Be **declarative** (no Turing-complete user-authored code; LLM-assist outputs must be inspectable + linted).
2. Be **server-authoritative**: clients evaluate for UX, but the server is the truth on which fields are visible / required / persisted (per `threat-model.md` T-T-01 skip-logic bypass).
3. Be **bounded**: predicate evaluation must terminate quickly; cycle detection at authoring time (T-D-02 validation-cost DoS).
4. Support **per-field show/hide**, **page-flow next**, **required-only-if**, and **cross-field validation**.
5. Be **lintable + statically analysable** for: data-class flow (don't capture PII in a conditionally-hidden field that's still visible to the LLM-assist); reachability (no orphan pages); cycle freedom.
6. Be **identical client-side + server-side** (AC-03 parity over 1000-case corpus).
7. Support **i18n-aware** comparisons (e.g., string comparison with locale collation).

## Decision

Adopt **CEL (Common Expression Language)** as the predicate expression language, evaluated against a **declarative DAG** structure stored under `FormSpecV1.branching[]` + `FormSpecV1.cross_field_rules[]`.

### Why CEL

- Standardised; multi-language implementations (Go, C++, Java, Rust via `cel-rust`, TypeScript via `cel-js`).
- Bounded execution (no loops, no recursion); terminating by construction.
- Strongly typed (catch type errors at compile time at authoring).
- Industry-proven (Kubernetes admission controllers, Google APIs, Tetragon, Cilium).
- LLM-friendly (CEL grammar widely-trained; AI-form-build emits CEL natively).

### DAG model

```
FormSpecV1
├── pages[]
│   └── sections[]
│       └── fields[] (declared by field_id)
├── branching[]
│   ├── BranchPredicate
│   │   ├── when: CEL expression over visible fields
│   │   ├── show_field_ids[]
│   │   ├── hide_field_ids[]
│   │   └── next_page_id: optional jump-to-page
├── cross_field_rules[]
│   ├── CrossFieldRule
│   │   ├── rule: CEL expression returning bool
│   │   └── error_message_i18n: per-locale message
```

### Authoring-time invariants (enforced by `oya-forms-branching-static-analysis` lane)

- **No cycles**: branching DAG is acyclic; circular `next_page_id` rejected at publish.
- **Reachability**: every page reachable from `pages[0]` via at least one path.
- **Predicate cost cap**: ≤ 200 cross-field rules per form; ≤ 5000 CEL ops per evaluation; reject if exceeded (per `threat-model.md` T-D-02).
- **Data-class flow**: if a field with `data_class=PII_IDENTIFYING` is referenced in a branching predicate's `when`, the predicate's containing branch MUST NOT route a submitter to a path that conditionally hides + persists that PII value (i.e., we never persist PII a submitter believed to be hidden).
- **Type-soundness**: CEL expression's types match form spec; e.g., `fields["age"] >= 18` only valid if `age` declared as number.

### Runtime invariants

- **Server-authoritative**: client-side evaluation is advisory; server re-evaluates all branching + cross-field rules on submit, with server-fetched form spec. T-T-01 invariant: skip-logic bypass via tampered client state always rejected at server.
- **Identical results**: AC-03 1000-case corpus run on both `cel-rust` (server-side) and `cel-js` (client-side) MUST produce identical pass/fail per case.
- **i18n string compare**: ICU collation via fixed locale per submitter session.
- **Field-references closure**: a CEL predicate referencing `fields["foo"]` requires field "foo" to be in scope per the DAG visible-set; out-of-scope reference is null and the predicate result follows CEL null semantics (`fields["foo"] >= 18` with null = error → submit rejected with diagnostic).

## Alternatives Considered

### Alternative A — Imperative JavaScript predicates (eval()-style)

Allow tenant to embed JavaScript snippets as branch predicates; execute in a sandbox (e.g., QuickJS, V8 isolate).

- **Pros**
  - Maximum expressiveness.
  - Familiar to developers.
- **Cons**
  - Sandbox escape risk; JS-engine vulnerabilities are a continuous-threat surface.
  - Non-terminating predicates possible (`while(1)`); timeout enforcement adds DoS surface.
  - LLM (AI-form-build) often emits subtly wrong JS for arithmetic edge cases.
  - Static analysis (data-class flow, cycle freedom) requires AST parser.
  - Static type system absent.
- **Rejected reason**: security + boundedness > flexibility. CEL is structurally bounded; JS is not.

### Alternative B — JSONata (declarative JSON query language)

Use JSONata as the predicate expression language.

- **Pros**
  - Declarative; widely used for JSON transformations.
  - Bounded execution.
- **Cons**
  - Less type-safety than CEL; runtime errors common on missing fields.
  - Smaller ecosystem of multi-language implementations (mostly JS; Rust binding immature).
  - Tooling ecosystem smaller; less LLM training data.
- **Rejected reason**: ecosystem maturity. CEL has wider multi-language support + better LLM coverage.

### Alternative C — Custom DSL bespoke to forms

Define a forms-specific DSL ("when answer X equals 'yes' then show Y").

- **Pros**
  - Maximum control; can model forms-specific concepts directly.
- **Cons**
  - Bespoke parser + evaluator per language (Rust server-side, TypeScript client-side, Python SDK); divergence risk.
  - LLM-assist must learn the DSL.
  - No off-the-shelf static analyser; every check we want we must build.
  - Tenant operator learns yet another syntax.
- **Rejected reason**: bespoke DSL costs > benefits. CEL is good enough and has tooling.

### Alternative D — Pure declarative rules (no expression language)

Limit branching to simple key-value matches: `{"field": "x", "equals": "y", "show": "z"}`.

- **Pros**
  - Trivial to implement; no expression language at all.
- **Cons**
  - Cannot express `age >= 18` OR `lower(email).endsWith("@university.edu")` OR `dates_diff(start, end) >= 7`.
  - Forces tenant to flatten everything into rule sets; explodes rule count past T-D-02 cap.
  - Competitors offer expression language (Typeform Logic Jump expressions, Qualtrics survey flow logic, Jotform conditions).
- **Rejected reason**: insufficient expressiveness for competitive parity.

### Alternative E — Rego (Open Policy Agent's policy language)

Use Rego.

- **Pros**
  - Declarative; bounded; widely used in policy.
- **Cons**
  - Heavier than CEL; logic programming model has surprising semantics for non-Prolog-trained authors.
  - Smaller LLM training corpus on Rego than on CEL.
  - Better-fit for authorisation policy than form branching.
- **Rejected reason**: paradigm mismatch + LLM authoring quality.

## Consequences

### Architectural

- The `oya-forms-conditional-logic-domain` kernel exposes `Evaluate(form_spec, field_values) -> Visibility{ visible_field_ids[], required_field_ids[], next_page_id, validation_errors[] }`.
- Server-side uses `cel-rust`; client-side (Leptos WASM) uses `cel-js` compiled into the bundle.
- AC-03 parity test runs every release: 1000-case corpus over both implementations.
- The dsl-loader rejects CEL expressions referencing undeclared fields, non-terminating constructs (none in CEL), or operations exceeding the cost cap.

### Downstream µservices

1. **ontology**: `Form` entity exposes branching DAG as a queryable structure; consumers can statically analyse.
2. **foundry-runtime** (T1 field-suggest): LLM-assist emits CEL natively; PII-redactor + dsl-loader chain validates.
3. **workflow-engine**: form-submission event includes the visible-set the submitter saw; engine can replay deterministically.
4. **sheets**: response-bridge to sheets preserves only-visible-fields-at-submit-time (no skipped PII bleeding into sheets).
5. **observability**: SLI `oya-forms-conditional-logic-parity` over the 1000-case corpus; non-100% gate failure.

### SLOs and CI lanes affected

- `oya-forms-conditional-logic-parity` — server + client identical over 1000-case corpus (AC-03).
- `oya-forms-branching-static-analysis` — DAG acyclic, reachable, type-sound, cost-capped (AC-04 + T-D-02).
- `oya-forms-skip-logic-pii-correctness` — hidden-by-condition field has no `PII_*` value persisted (AC-15 + T-T-01).

### Compliance + audit

- GDPR Art. 5(1)(c) data minimisation: server-side enforcement of skip-logic prevents over-collection.
- GDPR Art. 25 DPbDD: default-deny on out-of-scope CEL field references.
- WCAG 2.2 AA: branching changes announced to screen readers via ARIA live region (renderer responsibility, not engine).

### Risk register

- **Risk**: CEL spec evolves; current evaluator drifts. **Mitigation**: pin `cel-spec` version per form spec; supersession ADR if upgrade required.
- **Risk**: `cel-rust` vs `cel-js` semantic divergence on edge cases. **Mitigation**: AC-03 parity test; common conformance corpus from the CEL project + oyatie additions.
- **Risk**: Tenant authors a 199-rule form that evaluates slowly under load. **Mitigation**: cost cap enforced at publish; SLI `oya-forms-field-validate-latency` budgeted at p99 50ms.
- **Risk**: LLM emits CEL referencing non-existent fields. **Mitigation**: dsl-loader rejects with precise json-pointer (per ADR-FORMS-0005 §"Decision").

## References

- CEL specification — `github.com/google/cel-spec`.
- `cel-rust` — `github.com/cel-rust/cel-rust`.
- `cel-js` — `github.com/yaml-language-server/cel-js`.
- Kubernetes Admission Controller (CEL-based ValidatingAdmissionPolicy) — `kubernetes.io/docs/reference/access-authn-authz/validating-admission-policy/`.
- JSONata — `jsonata.org/`.
- Rego / OPA — `openpolicyagent.org/`.
- Typeform Logic Jumps + Qualtrics Survey Flow + Jotform Conditions reference docs.
- `microservices/forms/PRD.md` AC-03 + AC-04 + AC-15.
- `microservices/forms/threat-model.md` T-T-01 + T-D-02.
- ADR-FORMS-0001 (form spec).
- ADR-0140 Cedar (separate policy engine; CEL here is for form-internal branching only).
- ADR-0131 per-microservice flat layout.
