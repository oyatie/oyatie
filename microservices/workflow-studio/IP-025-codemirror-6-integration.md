---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-studio
milestone: M03-studio-preview
phase: P02-native-canvas-shells
impl_plan_id: IP-025-codemirror-6-integration
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-frontend
co_owners: [axis-a11y]
date: 2026-05-18
related_adrs: [ADR-0205, ADR-0207]
acceptance_lanes: [a11y-axe-zero-violations, code-editor-correctness, oya-governance-promotion-readiness]
depends_on: [IP-016]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-025 — CodeMirror 6 integration (in-product code surfaces)

## Goal

Per ADR-0205, CodeMirror 6 is canonical for every in-product code surface (custom-code step body, scratch SQL, JSON editor, Cedar policy fragment editor). Build a thin adapter `CodeMirrorAdapter.ts` that wraps CM6 with the language packs, LSP integration (via IP-026), themes (light/dark/AAA high-contrast), and the WCAG 2.2 AA gate. The adapter is reused by every shell-specific embedding (web SvelteKit, plus WebView hosts in native shells).

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `clients/web-sveltekit/lib/editor/cm6/CodeMirrorAdapter.ts` | create | ~260 LoC; wraps `EditorView` + extensions + LSP + a11y |
| `clients/web-sveltekit/lib/editor/cm6/languages.ts` | create | ~80 LoC; per-language extension factory (`@codemirror/lang-javascript`, `-rust`, `-python`, `-sql`, `-yaml`, `-json`) |
| `clients/web-sveltekit/lib/editor/cm6/themes.ts` | create | ~140 LoC; light + dark + AAA high-contrast theme defs |
| `clients/web-sveltekit/lib/editor/cm6/a11y-extension.ts` | create | ~120 LoC; ARIA live region + screen-reader announcements |
| `clients/web-sveltekit/lib/editor/cm6/svelte-binding.svelte` | create | ~100 LoC; Svelte 5 runes binding |
| `clients/web-sveltekit/package.json` | edit | pin `@codemirror/state`, `@codemirror/view`, `@codemirror/commands`, `@codemirror/lang-*`, `codemirror-languageserver` |
| `clients/web-sveltekit/tests/editor-cm6.spec.ts` | create | ~200 LoC; 5 integration tests |
| `clients/web-sveltekit/tests/editor-a11y.spec.ts` | create | ~120 LoC; axe-core + keyboard navigation tests |
| `microservices/workflow-studio/runbooks/cm6-editor-debug.md` | create | ~80 LoC playbook |
| `microservices/workflow-studio/decisions/ADR-0205.md` | append §"CM6 adapter shipped" | +6 LoC |

## Code shape

`CodeMirrorAdapter.ts` (excerpt):

```ts
export class CodeMirrorAdapter {
  private view: EditorView;
  constructor(opts: AdapterOpts) {
    const extensions = [
      basicSetup,
      languageForId(opts.languageId),
      themeFor(opts.themePreference),
      a11yExtension({ liveRegion: opts.liveRegion }),
      lspClientFor(opts.lspEndpoint),
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
    ];
    this.view = new EditorView({ doc: opts.initialDoc, parent: opts.parent, extensions });
  }
  getValue(): string { return this.view.state.doc.toString(); }
  setValue(doc: string) { this.view.dispatch({ changes: { from: 0, to: this.view.state.doc.length, insert: doc } }); }
  destroy() { this.view.destroy(); }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `cm6_loads_typescript_pack_and_highlights` | editor-cm6.spec.ts | TS code renders with syntax tokens |
| `cm6_lsp_completion_triggers_within_300ms` | editor-cm6.spec.ts | Completion popup appears within 300ms p95 |
| `cm6_theme_dark_meets_aa_contrast` | editor-cm6.spec.ts | Computed colors satisfy WCAG 2.2 AA (4.5:1 normal, 3:1 large) |
| `cm6_theme_high_contrast_meets_aaa` | editor-cm6.spec.ts | AAA threshold (7:1 normal, 4.5:1 large) |
| `cm6_cedar_grammar_pack_recognizes_permit_block` | editor-cm6.spec.ts | Cedar `permit(principal,...)` highlights as keyword |
| `cm6_keyboard_nav_arrow_keys_move_caret` | editor-a11y.spec.ts | Arrow keys move caret; Home/End respected |
| `cm6_axe_core_zero_violations` | editor-a11y.spec.ts | axe-core scan returns 0 violations |
| `cm6_screen_reader_live_region_announces_diagnostics` | editor-a11y.spec.ts | LSP diagnostic emits ARIA-live announcement |

Minimum 5 required; 8 specified.

## Evidence to emit

- `evidence/microservices/workflow-studio/cm6-adapter-correctness-{date}.json`
- `evidence/microservices/workflow-studio/cm6-a11y-axe-{date}.json` — axe-core report
- Audit-chain seal: `oya audit-chain seal --kind code-editor-a11y --ms workflow-studio --window 30d`
- Metrics: `oya_workflow_studio_cm6_completion_latency_ms_bucket`, `oya_workflow_studio_cm6_keystroke_latency_ms_bucket`

## Rollback procedure

1. Revert ChangeSet for `clients/web-sveltekit/lib/editor/cm6/`.
2. Flip feature flag `workflow_studio_code_editor=disabled` → fall back to plain `<textarea>` (no syntax highlight, no LSP, banner displayed).
3. Remove pinned CM6 packages from `package.json`.
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-016 — bundles CM6 into SvelteKit build pipeline.
- IP-026 — LSP bridge (consumed for code completion).
- IP-027 — Cedar grammar (consumed as language pack).
- ADR-0205 — code editor canonical.
- ADR-0207 — a11y bar (axe-core threshold).

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate a11y-axe-zero-violations --target cm6
cargo run -p oya-dev-cli -- gate validate code-editor-correctness --target cm6
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice workflow-studio
pnpm --filter web-sveltekit test:integration editor-cm6
```

## Halt conditions

- Any axe-core violation: STOP, file a11y-defect IP.
- LSP completion latency p95 > 500ms: STOP, perf regression.
- Theme contrast below WCAG AA: STOP, security/compliance critical.

## Exit criteria

1. All 8 tests green on CI.
2. `a11y-axe-zero-violations` + `code-editor-correctness` + `oya-governance-promotion-readiness` lanes green.
3. Evidence ledger sealed.
4. Runbook published.
5. ADR-0205 status updated.

## Next IP

[`IP-026-lsp-bridge.md`](IP-026-lsp-bridge.md)

## References

- ADR-0205 — code editor canonical.
- ADR-0207 — a11y bar.
- IP-026 — LSP bridge.
- IP-027 — Cedar grammar.
- CodeMirror 6 docs — `https://codemirror.net/docs/`.
- axe-core — `https://github.com/dequelabs/axe-core`.
- WCAG 2.2 — Contrast (Minimum) 1.4.3, Contrast (Enhanced) 1.4.6.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-025-codemirror-6-integration.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
