---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-003-formula-engine-kernel-domain-400-functions
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, oya-governance-sheets-formula-engine-correctness]
depends_on: [IP-002]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: formula-engine — kernel + domain + usecase + api + adapter + sdk (≥400 functions; LibreOffice Calc reference corpus)

## Intent

Author the `formula-engine` BC's full crate set: ≥400-function library covering math, logical, lookup, statistical, financial, text, date, array categories per ADR-SHEETS-0002. Parser + evaluator are pure Rust; deterministic. Conformance against the LibreOffice Calc reference corpus is the load-bearing AC-11 invariant.

## ChangeSet boundary

Six crates:
- `oya-sheets-formula-engine-{kernel,domain,usecase,api,adapter,sdk}`

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/sheets/src/crates/oya-sheets-formula-engine-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-formula-engine-domain/{Cargo.toml,src/lib.rs,src/parser.rs,src/evaluator.rs,src/functions/{math,logical,lookup,statistical,financial,text,date,array}.rs,tests/excel_reference_corpus.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-formula-engine-usecase/{Cargo.toml,src/lib.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-formula-engine-api/{Cargo.toml,src/lib.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-formula-engine-adapter/{Cargo.toml,src/lib.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-formula-engine-sdk/{Cargo.toml,src/lib.rs}` | create |
| `microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl` | create (corpus seeded from LibreOffice Calc behaviour matrix) |

## Code Shape

`formula-engine-domain/tests/excel_reference_corpus.rs`:

```rust
#[test]
fn test_excel_reference_corpus() {
    let corpus = load_corpus("microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl");
    let mut mismatches = vec![];
    for case in corpus {
        let result = oya_sheets_formula_engine_domain::evaluator::eval(&case.formula, &case.context);
        if result != case.expected {
            mismatches.push((case.formula.clone(), case.expected.clone(), result));
        }
    }
    assert!(mismatches.is_empty(), "Formula-engine corpus mismatch: {} cases failed", mismatches.len());
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-formula-engine-kernel -p oya-sheets-formula-engine-domain \
  -p oya-sheets-formula-engine-usecase -p oya-sheets-formula-engine-api \
  -p oya-sheets-formula-engine-adapter -p oya-sheets-formula-engine-sdk
cargo nextest run -p oya-sheets-formula-engine-domain --test excel_reference_corpus
buck2 build //:quality-lane-registry-authority-check # lane=sheets-formula-engine-correctness --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_excel_reference_corpus` | ≥400 functions; LibreOffice Calc reference behaviour matched per ADR-SHEETS-0002 |
| `test_math_functions` | SUM, AVG, COUNT, ROUND, etc. (50 fns) |
| `test_logical_functions` | IF, AND, OR, NOT, IFS, SWITCH, etc. (15 fns) |
| `test_lookup_functions` | VLOOKUP, HLOOKUP, INDEX, MATCH, XLOOKUP, etc. (30 fns) |
| `test_statistical_functions` | STDEV, VAR, CORREL, PERCENTILE, etc. (80 fns) |
| `test_financial_functions` | NPV, IRR, PMT, FV, PV, RATE, etc. (50 fns) |
| `test_text_functions` | CONCAT, TRIM, SUBSTITUTE, REGEX*, etc. (60 fns) |
| `test_date_functions` | TODAY, NOW, DATE, DATEDIF, etc. (40 fns) |
| `test_array_functions` | TRANSPOSE, FILTER, SORT, UNIQUE, etc. (50 fns) |
| `test_error_propagation` | #DIV/0!, #VALUE!, #REF!, #NAME?, #NUM!, #N/A, #NULL!, #CIRCULAR, #SLOW! |
| `test_no_eval_no_exec_no_shell` | No `process::Command`, no eval-like primitives (LEAN check) |

## Halt Conditions

- Corpus mismatch on any function — STOP. ADR-SHEETS-0002 load-bearing.
- LEAN check `oya-governance-editor-execution-forbidden` fails — STOP.

## Next IP

[`IP-004-recalc-engine-dep-graph-parallel.md`](IP-004-recalc-engine-dep-graph-parallel.md)

## References

- PRD AC-11.
- ADR-SHEETS-0002 (formula-engine conformance).
- LibreOffice Calc behaviour matrix — `documentation.libreoffice.org`.
- OOXML ECMA-376 — `ecma-international.org/publications-and-standards/standards/ecma-376/`.
- Microsoft Excel function reference — `support.microsoft.com/en-us/office/excel-functions-by-category`.
