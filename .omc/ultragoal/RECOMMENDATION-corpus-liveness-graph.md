# Recommendation: Corpus Liveness Graph (fundamental solution to decay)

**Problem (founder, 2026-06-10):** missed directives from accepted ADRs, directive drift, lossy context across sessions, staleness, dead code, dead files, unmaintained docs/code — all decaying with no fundamental mechanism. Founder has no answer; wants a recommendation. `/idea-refine` output.

**Reframe (the unlock):** every symptom is ONE graph pathology, not separate problems:
- dead code/file = **orphan node** (no inbound edge from entrypoint/test/product/ADR)
- stale doc / broken link = **broken/outdated edge** (doc→code whose content-hash moved)
- directive drift = **missing compliance edge** (work in a directive's scope not satisfying it)
- missed directive (#651) = **untraversed edge** (scoped work done without surfacing the governing directive)
- unmaintained = **stale node** (last-touch past freshness budget, never re-validated)
- lossy context = context not **generated from the graph**, so in-scope nodes dropped

**Recommended direction:** treat the WHOLE corpus (code symbols, files, docs, ADRs, directives, tests, product surfaces) as nodes in ONE content-addressed, AST-derived graph; make decay a family of FAIL-CLOSED CI invariants over it. The decisive, trailblazing move:

> **Docs and directives are BUILD ARTIFACTS with dependencies, not prose.** A doc referencing `oya/identity` is a build target whose input is `oya/identity`; when that input's content-hash changes, the doc is **dirty** and must be re-validated or it fails the gate. Dead code = a target nothing depends on (GC-able). Directive compliance = a required green edge. This is **buck2 dirty-tracking applied to the entire corpus** — which is WHY the owned content-addressed AST (W2) + buck2 fabric is the substrate, and why it runs in-process/incrementally (the cloud-native/full-Rust efficiency thesis).

**Pieces already in the repo that converge here:** W2 owned AST (content-addressed node identity) · 3-layer knowledge graph in `registry/` · total-accounting gate (partial no-orphan-file invariant) · authority-chain registry idea (FRIC-014, file→ADR edges) · generated-faces discipline (the graph face is CI-materialized, can't hand-rot).

**Enforcement layering (structural > gate > hook):**
- **Structural:** node identity = content hash; edges DERIVED by AST parse → can't forget a computed edge; graph face generated + CI-materialized → can't hand-rot. Decay impossible-by-construction for structural classes.
- **Gate (canonical, fail-closed in oya-ci-required):** no orphans; no broken/stale edges past freshness budget; no work-in-directive-scope without a compliance edge; **no accepted directive without an enforcement-mechanism-or-explicit-cultural-tag** (the #651-class hole); supersession TWO-OUTCOME (re-point or dispose, FRIC-014).
- **Hook:** local fast feedback only.

**Missed-directive specifically:** every accepted ADR/directive declares (or AST-derives) its SCOPE (glob/service/concern). Graph computes scope→work edges → PR touching a scope AUTO-SURFACES governing directives (context generated from graph = lossy-context fix), gate asserts compliance. MECHANIZABLE directives → compile to Cedar policies / gate predicates (drift-proof). NON-MECHANIZABLE → a **directive-coverage review axis** (the adversarial review generalized — the discipline that caught #655's hidden blockers). Honest split: pure mechanism can't judge "prose subtly wrong now"; the review axis covers the semantic residual.

**Assumptions to validate (before ratifying as ADR):**
1. Most decay is graph-expressible (true for refs/orphans/staleness; partial for semantic drift → review axis covers rest). Test: classify the 50 session frictions — graph-invariant vs needs-judgment.
2. Corpus-wide AST extraction tractable at 791 crates + docs incrementally (content-addressed → only changed nodes re-parse). Test: W2 spike on one service.
3. Freshness budgets authorable without noise. Test: sane default per node-class.

**Not doing:** per-symptom point gates (→ graph invariants); hand-maintained doc/authority/ownership indexes (→ generated faces); mechanizing 100% of semantic staleness (→ review axis); a separate KG product (this IS W2 AST + registry extended).

**Next:** log friction (done) → deep-research precedents (Bazel staleness, Google Tricorder/Rosie LSC, docs-as-tested-artifacts, semantic-diff drift) → author ADR (self-referential: the fix for directive-drift is itself a ratified, enforced directive). Founder ratifies the ADR (door:one-way).

## SCOPE EXPANSION (founder 2026-06-10): every granularity, every conformance class
The substrate operates at ALL granularities and covers ALL decay classes — one graph, nodes at every level, invariants per class:

**Granularities (node levels):** ADR/document → file → folder → module/crate → symbol (fn/type/const) → **line/statement** → token/format. The content-addressed AST gives sub-symbol (line/statement) identity, so "dead code LINE" (unreachable statement, unused local) is an orphan at the statement-node level — not just dead files.

**Conformance classes (invariant families) — each a failed edge/predicate on a node:**
1. **Liveness/orphan** — dead code (line/symbol/file), dead/orphan folder, orphan ADR (no inbound governs/cited-by edge). Node with no inbound reachability edge from an entrypoint/test/product/ADR.
2. **Reference integrity** — stale doc, broken link, dangling citation, doc→code whose content-hash moved. Broken/outdated edge.
3. **Format conformance** — formatting drift (rustfmt/treefmt not applied; compiles but violates format). A node whose token-form ≠ its canonical-format projection. (Today: separate format gates; in CLG it's one invariant class over format-nodes.)
4. **Template conformance** — template/scaffold drift: a file that must match a template (ADR frontmatter schema, manifest.json shape, IP-doc template, crate scaffold, BUCK shape) but drifted. A node whose structure ≠ its declared template's required-shape. (Today: partial — manifest-hygiene, bnf-suffix, adr-shape gates; in CLG it's one template-conformance invariant parameterized by the node's declared template.)
5. **Liveness/freshness** — unmaintained: stale node past freshness budget, never re-validated.
6. **Directive compliance** — drift/missed-directive: work-in-scope without a satisfied governing-directive edge.

The unifying claim to validate in the deep-dive: a SINGLE content-addressed AST graph + a parameterized invariant family (each class above = one invariant kind, parameterized per node-type) subsumes the entire current gate fleet (manifest-hygiene, bnf-suffix, format, total-accounting, adr-shape, staleness, brand-residue, …) — they are all special cases of {orphan, broken-edge, format-nonconformance, template-nonconformance, stale-node, missing-compliance-edge}. If true, the N-gates fragmentation collapses to ONE substrate + a rule table.
