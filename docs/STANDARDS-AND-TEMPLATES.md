---
purpose: Oyatie — Standards, Templates, Hooks, Skills, Tools, Checklists
doc_status: published
---

# Oyatie — Standards, Templates, Hooks, Skills, Tools, Checklists

> **Status:** Draft v0.1 — 2026-05-09. Authored per user directive: every contributor (human or agent) reaches into this doc for the canonical template / checklist / hook / skill / tool when starting common work. The goal: zero bespoke artifact authoring; everyone copies from a templated, standardized source.
> **Owner:** `axis-foundry` (engineering platform surface) + `council-architecture` (cross-cutting standards).
> **Companion:** [TOOLCHAIN.md](TOOLCHAIN.md), [DOC-CATALOG.md](DOC-CATALOG.md), [DOCUMENTATION.md](DOCUMENTATION.md).

---

## 1. Reading guide

This catalog is divided into seven kinds of artifact:

| Kind | Used by | Lives at |
|---|---|---|
| **Templates** | Author starting a new artifact (PR, ADR, capability, runbook, …) | `docs/templates/<artifact>.md` (or `.yaml`) |
| **Checklists** | Author finishing or gating an artifact | `templates/checklists/<task>.md` |
| **Hooks** | Claude Code / Codex / Gemini agent harnesses + git hooks | `.claude/hooks/`, `.git/hooks/`, `scripts/hooks/` |
| **Skills** | Agents at runtime (slash commands) | Installed runtime catalogs (Codex: `~/.codex/skills`; project `.claude/skills/<id>/SKILL.md` / `.codex/skills/` when checked in); optional local `.grok/` mm-delivery kit. `.omc/skills/` is retired residual, not live authority (ADR-0619). |
| **Tools (CLIs)** | Humans + agents at runtime | `oya <persona> <subcommand>` per the persona-split CLI in [TOOLCHAIN §3](TOOLCHAIN.md) |
| **Standardized guidance** | Anyone authoring (style / pattern / norm) | `docs/standards/<topic>.md` |
| **Requirements + Specs** | Per-product or per-axis surface contracts | `products/<product>/PRD.md` + `contracts/openapi/**/*.yaml` |

The whole catalog is mirrored at `machine-readable/standards.json` for agent consumption.

---

## 2. Templates index

| Template | Path | Used when | Required sections | Validator |
|---|---|---|---|---|
| Pull request | [`templates/pull-request-template.md`](templates/pull-request-template.md) | Every PR | Issue / Summary / Verification / `## Code Review` | Local validator retired; merge contract requires `presubmit` plus independent reviewer approval (`F-PR5-06` bounds cloud enforcement) |
| ADR | [`templates/adr-template.md`](templates/adr-template.md) | New decision | Status / Supersedes / Superseded-by / Context / Decision / Consequences / Alternatives / Open-questions / References | `adr-template-coverage` CI lane |
| Capability record | [`templates/capability-record-template.yaml`](templates/capability-record-template.yaml) | Foundry capability publish | id / namespace / inputs / outputs / autonomy_tier / data_classes / evidence_topic / regulatory_packs / cost_profile / sunset | `capability-schema-validator` |
| Catalog record (per crate) | [`templates/catalog-record-template.yaml`](templates/catalog-record-template.yaml) | Every flat-crate | name (=package) / role / context / plane / contracts_consumed / contracts_exposed / regulatory_packs_consumed / lifecycle.state / traceability.github_issue / allowed_dependency_edges | `oya catalog validate` |
| Runbook | [`templates/runbook-template.md`](templates/runbook-template.md) | New operational procedure | Trigger / Severity / Pre-checks / Steps / Rollback / Verification / Post-incident updates | `runbook-discoverability` lane |
| Incident postmortem | [`templates/incident-postmortem-template.md`](templates/incident-postmortem-template.md) | Sev 1/2 closed | Summary / Timeline / Impact / Root cause / Lessons / Actions / Trust-portal entry | `incident-template-completeness` |
| Regional pack | [`templates/regional-pack-template.md`](templates/regional-pack-template.md) | New regional pack | Pack id / Region / Regulators / Compliance packs / i18n / Currency / Calendar / Tax / Identity providers / Payment rails / Address book / Ecosystem partners / Content safety / Ad policy / Industry data models / Vendor partners | `regional-pack-validator` |
| Per-product PRD | [`products/_TEMPLATE.md`](products/_TEMPLATE.md) (already exists) | New product | 13 sections per [products/README.md](products/README.md) | `prd-template-coverage` |
| Team charter | [`templates/team-charter-template.md`](templates/team-charter-template.md) | New team | Mission / Owned axes / In-scope / Out-of-scope / Dependencies / Success metrics / Cadence / Bandwidth / Norms / Risks | `raci-team-coverage` |
| Threat model (per service) | [`templates/threat-model-template.md`](templates/threat-model-template.md) | Per service quarterly | Actors / Surfaces / Trust boundaries / STRIDE table / Mitigations / Residual risks | `security-controls-coverage` |
| DPIA (per regulated capability) | [`templates/dpia-template.md`](templates/dpia-template.md) | Per regulated capability | Purpose / Lawful basis / Data classes / Risks / Mitigations / Residual risk / Council sign-off | `privacy-class-taxonomy-coverage` |
| Evidence pack (per regulator) | [`templates/evidence-pack-template.md`](templates/evidence-pack-template.md) | Per regulator (annual / per-audit) | Control mapping / Evidence type / Cadence / Owner / Latest evidence link | `compliance-evidence-recency` |
| Migration runbook (per migrating tenant) | [`templates/migration-runbook-template.md`](templates/migration-runbook-template.md) | Tenant migrating from legacy stack | Source / Target / Mapping table / Cutover steps / Rollback / Validation | `runbook-discoverability` |
| Capability eval set | [`templates/capability-eval-set-template.yaml`](templates/capability-eval-set-template.yaml) | New Foundry capability | golden_inputs / expected_outputs / eval_metric / pass_threshold | `capability-eval-coverage` |
| ADR rename / supersession | [`templates/adr-supersession-template.md`](templates/adr-supersession-template.md) | When superseding an ADR | Original ADR / What changes / Why / Migration path | `adr-supersession-graph` |

---

## 3. Checklists index

| Checklist | Path | Trigger | Owner | Validator |
|---|---|---|---|---|
| Pre-push | [`../templates/checklists/pre-push.md`](../templates/checklists/pre-push.md) | Before `git push` | Author | `oya verify` |
| Pre-merge | [`../templates/checklists/pre-merge.md`](../templates/checklists/pre-merge.md) | Before `gh pr merge` | Author + reviewer | `oya gate validate` |
| Wave-gate passing | [`../templates/checklists/wave-gate.md`](../templates/checklists/wave-gate.md) | At wave boundary | Wave-tactical team | `wave-gate-readiness` (per ADR-0040) |
| Vertical onboarding | [`../templates/checklists/vertical-onboarding.md`](../templates/checklists/vertical-onboarding.md) | New vertical Preview | Vertical team | per-vertical PRD §11 + COMPLIANCE-MATRIX |
| Regional-pack onboarding | [`../templates/checklists/regional-pack-onboarding.md`](../templates/checklists/regional-pack-onboarding.md) | New regional pack | `regional-packs` team | `regional-pack-validator` |
| Foundry capability publishing | [`../templates/checklists/foundry-capability-publishing.md`](../templates/checklists/foundry-capability-publishing.md) | New / updated capability | `axis-foundry` | `capability-schema-validator` + eval-set pass |
| New team onboarding | [`../templates/checklists/new-team-onboarding.md`](../templates/checklists/new-team-onboarding.md) | New team formed | `council-architecture` | `raci-team-coverage` |
| Audit readiness | [`../templates/checklists/audit-readiness.md`](../templates/checklists/audit-readiness.md) | Per audit cycle (annual + on-demand) | `ops-compliance` | `compliance-evidence-recency` |
| Incident response | [`../templates/checklists/incident-response.md`](../templates/checklists/incident-response.md) | Sev 1/2 detected | Incident manager | `incident-template-completeness` |
| DSR cascade | [`../templates/checklists/dsr-cascade.md`](../templates/checklists/dsr-cascade.md) | DSR received | `council-privacy` (DSR operator) | DSR queue dashboard |
| Brand-rename batch | [`../templates/checklists/brand-rename-batch.md`](../templates/checklists/brand-rename-batch.md) | Per-batch (17 batches per rename agent) | per-batch lead | `brand-residue-validator` |
| ADR promotion | [`../templates/checklists/adr-promotion.md`](../templates/checklists/adr-promotion.md) | Proposed → Accepted | `crew-adr-promotion` | `adr-supersession-graph` |
| Foundation-bypass renewal | [`../templates/checklists/foundation-bypass-renewal.md`](../templates/checklists/foundation-bypass-renewal.md) | Per bypass expiry | per-bypass owner | `bypass-expiry-monitor` |
| Cross-axis contract change | [`../templates/checklists/cross-axis-contract-change.md`](../templates/checklists/cross-axis-contract-change.md) | Any DESIGN §10 row change | All affected axis teams | `design-contracts-mirror` |
| License-tier review | [`../templates/checklists/license-tier-review.md`](../templates/checklists/license-tier-review.md) | New external dep / version bump | `ops-security` + `axis-foundry` | `governance-license` |
| Build-vs-buy decision | [`../templates/checklists/build-vs-buy.md`](../templates/checklists/build-vs-buy.md) | New surface authored | Owning axis + `council-architecture` | `build-vs-buy-decision-validator` |
| Tenant onboarding | [`../templates/checklists/tenant-onboarding.md`](../templates/checklists/tenant-onboarding.md) | New tenant | `gtm-customer-success` + per-vertical | `tenant-onboarding-evidence` |
| Trust-portal publish | [`../templates/checklists/trust-portal-publish.md`](../templates/checklists/trust-portal-publish.md) | Audit-evidence regen | `ops-compliance` + `gtm-marketing` | trust-portal verification |

---

## 4. Hooks index

Hooks are mechanical gates fired by harnesses or git. Defined under `.claude/hooks/`, `.git/hooks/`, `scripts/hooks/`. Key hooks:

| Hook | Event | Purpose | Path |
|---|---|---|---|
| `pre-commit-license` | git pre-commit | Refuses commits that add an external dep without a license-ledger entry | `scripts/hooks/pre-commit-license.sh` |
| `pre-commit-arch-boundary` | git pre-commit | Refuses commits that violate ADR-0015 dep direction (kernel←domain←app←api/worker/adapter←runtime) | `oya gate validate architecture-boundaries` |
| `pre-commit-data-class-annotation` | git pre-commit | Refuses commits that add a struct field without a `data_class` annotation when the file is in a kernel crate | `scripts/hooks/pre-commit-data-class.sh` |
| `pre-commit-yaml-date-quoted` | git pre-commit | Refuses unquoted YAML dates (per mistakes-and-fixes-ledger) | `scripts/hooks/pre-commit-yaml-date.sh` |
| `pre-commit-forward-ref` | git pre-commit | Refuses markdown links to paths not yet on origin/main (per Issue #1433) | `scripts/hooks/pre-commit-forward-ref.sh` |
| `pre-push` | git pre-push | Runs `oya verify` (cargo fmt --check, cargo clippy, cargo nextest, oya gate validate, arch-boundary) | `.git/hooks/pre-push` |
| `pre-tool-use-foundry-evidence` | Claude Code PreToolUse | Stamps every Foundry capability invocation with an evidence-emission event before tool runs | `.claude/hooks/pre-tool-use-foundry-evidence.sh` |
| `post-tool-use-cohesion-fitness` | Claude Code PostToolUse | Runs cross-axis contract drift detection after edits | `.claude/hooks/post-tool-use-cohesion.sh` |
| `session-start-doc-context` | Claude Code SessionStart | Loads consolidated docs into agent context | `.claude/hooks/session-start-doc-context.sh` |
| `user-prompt-submit-skill-routing` | Claude Code UserPromptSubmit | Routes magic-keyword prompts to the right skill | `.claude/hooks/user-prompt-submit-skill-routing.sh` |
| `stop-validation` | Claude Code Stop | Verifies no leftover incomplete tasks before yielding | `.claude/hooks/stop-validation.sh` |
| `audit-emission-on-capability-invoke` | runtime | Every capability invocation emits an audit-chain record per ADR-0003 | `crates/intelligence-evidence-*` |
| `cohesion-fitness-on-pr` | CI | Runs `governance-cohesion` on every PR | `.github/workflows/cohesion-fitness.yml` |
| `license-fitness-on-pr` | CI | Runs `cargo deny licenses` + Trivy `--scanners license` + custom container scan | `.github/workflows/license-fitness.yml` |

---

## 5. Skills index (slash-command surface)

Skills are agent-invocable workflows. Under `.claude/skills/<id>/SKILL.md`. Aliases for human use under `oya <persona>`.

| Skill | Path | Purpose | Persona-CLI alias |
|---|---|---|---|
| `dev-check` | `.claude/skills/dev-check/SKILL.md` | Run pre-push checks | `oya verify` |
| `adr-author` | `.claude/skills/adr-author/SKILL.md` | Draft a new ADR with all required sections | `oya catalog adr new` |
| `adr-promote` | `.claude/skills/adr-promote/SKILL.md` | Promote a Proposed → Accepted ADR with shipped-evidence verification | `oya catalog adr promote` |
| `intelligence-capability-author` | `.claude/skills/intelligence-capability-author/SKILL.md` | Scaffold a new capability YAML + eval set + adapter | `oya agent capability new` |
| `regional-pack-author` | `.claude/skills/regional-pack-author/SKILL.md` | Scaffold a new regional pack with all 14 sections | `oya pack new` |
| `vertical-onboard` | `.claude/skills/vertical-onboard/SKILL.md` | Onboard a new vertical end-to-end | (orchestrates several) |
| `runbook-author` | `.claude/skills/runbook-author/SKILL.md` | Scaffold a new runbook | `oya ops runbook new` |
| `postmortem-extractor` | `.claude/skills/postmortem-extractor/SKILL.md` | Extract postmortem from incident-management timeline | `oya ops incident postmortem` |
| `glossary-extractor` | `.claude/skills/glossary-extractor/SKILL.md` | Find new domain terms in PRs and propose GLOSSARY rows | (auto on PR) |
| `rustdoc-fixer` | `.claude/skills/rustdoc-fixer/SKILL.md` | Propose rustdoc fixes when CI flags missing/stale doc comments | (auto on PR) |
| `translation-drafter` | `.claude/skills/translation-drafter/SKILL.md` | Draft per-pack translations | `oya pack translate` |
| `cohesion-fitness-fix` | `.claude/skills/cohesion-fitness-fix/SKILL.md` | Propose fixes for cohesion-fitness violations | (auto on PR) |
| `evidence-pack-regenerator` | `.claude/skills/evidence-pack-regenerator/SKILL.md` | Regenerate per-regulator evidence pack | `oya admin compliance regenerate` |
| `dsr-cascade-runner` | `.claude/skills/dsr-cascade-runner/SKILL.md` | Execute a DSR cascade end-to-end | `oya admin privacy dsr` |

---

## 6. Tools index (CLIs)

See [TOOLCHAIN §3 Language-stack matrix](TOOLCHAIN.md) and [DESIGN §13.4 Persona-split CLI](DESIGN.md).

The 8 persona-CLIs:
- `oya dev` — engineer
- `oya admin` — tenant admin
- `oya build` — customer builder
- `oya agent` — Foundry agent ops
- `oya ops` — SRE/Ops
- `oya pack` — regional pack maintainer
- `oya catalog` — catalog + capability authoring
- `oya gate` — gates + bypasses + claim-ceiling

Plus the agent-discoverable equivalent: `mcp-server` exposing every CLI subcommand as an MCP tool (per [TOOLCHAIN §4.A](TOOLCHAIN.md)).

---

## 7. Standardized-guidance index

Cross-cutting standards docs (lives under `docs/standards/`):

| Standard | Path | Topic |
|---|---|---|
| Code style | [`standards/code-style.md`](standards/code-style.md) | rustfmt + clippy config; per-language style; forbidden patterns |
| Commit message | [`standards/commit-message.md`](standards/commit-message.md) | conventional commits + Refs / Closes / Blocks; signed commits |
| Cloud-native infrastructure automation | [`standards/cloud-native-infrastructure-automation.md`](standards/cloud-native-infrastructure-automation.md) | API-shaped Rust/config/controller/gate infrastructure automation; no new ad-hoc CLIs or Python/shell core infra behavior |
| API design | [`standards/api-design.md`](standards/api-design.md) | REST + gRPC + event-schema conventions; pagination; idempotency; semver |
| Schema migration | [`standards/schema-migration.md`](standards/schema-migration.md) | versioned, reversible, dry-run; backward-read for ≥ 2 versions |
| Error handling | [`standards/error-handling.md`](standards/error-handling.md) | Result<T,E> conventions; never panic at API boundaries; retryable vs terminal |
| Logging + tracing | [`standards/logging-tracing.md`](standards/logging-tracing.md) | OTel `gen_ai` semconv; structured JSON; mandatory fields per ADR-0045 |
| Testing | [`standards/testing.md`](standards/testing.md) | test pyramid; fixture discipline; fuzz; property tests; insta snapshots; nextest |
| Security review | [`standards/security-review.md`](standards/security-review.md) | per-change-class checklist; threat-model triggers |
| Privacy review | [`standards/privacy-review.md`](standards/privacy-review.md) | data-class annotations; consent receipts; DSR cascade tests |
| Code review | [`standards/code-review.md`](standards/code-review.md) | per-change-class reviewer agent; verdict format; bypass logging |
| Release | [`standards/release.md`](standards/release.md) | trunk-based + release branch; tag at cut; Argo Rollouts canary |
| On-call | [`standards/on-call.md`](standards/on-call.md) | rotation; escalation; comms templates |
| Incident severity | [`standards/incident-severity.md`](standards/incident-severity.md) | Sev 1-4 taxonomy; declared per service |
| Doc style | [`standards/doc-style.md`](standards/doc-style.md) | per-doc kind (Diátaxis quadrant) writing rules; voice; tone |
| Brand voice | [`standards/brand-voice.md`](standards/brand-voice.md) | Oyatie / oYa brand voice; KR + global |
| Migration playbook | [`standards/migration-playbook.md`](standards/migration-playbook.md) | tenant migration from competitor stacks |
| Plugin authoring | [`standards/plugin-authoring.md`](standards/plugin-authoring.md) | Plugin substrate conventions (Wasmtime + manifest + signing) |
| Capability authoring | [`standards/capability-authoring.md`](standards/capability-authoring.md) | Foundry capability conventions (eval-set + autonomy tier + class allowlist) |

---

## 8. Requirements + Specs index

| Where | What |
|---|---|
| [PRD.md](PRD.md) | Cross-product requirements |
| [products/<id>/PRD.md](products/) | Per-product requirements |
| [DESIGN.md](DESIGN.md) §10 | Cross-axis contracts |
| [SPEC.md](SPEC.md) (when authored) | Per-axis surface enumeration with one-line invariants |
| `contracts/openapi/**/*.yaml` | Public REST surfaces |
| `contracts/*.proto` | gRPC + event surfaces |
| `contracts/asyncapi/*.yaml` | Async event-bus surfaces |

---

## 9. Catalog of catalogs

This doc is the catalog OF the catalogs. The other catalogs are:

| Catalog | Path | Source of truth |
|---|---|---|
| Document catalog | [DOC-CATALOG.md](DOC-CATALOG.md) + `machine-readable/catalog.json` | every consolidated doc |
| Crate catalog | `registry/catalog/<crate>.yaml` | every flat-crate |
| Capability catalog | `registry/capability-templates/*.yaml` | every Foundry capability |
| Contract catalog | `contracts/` + `machine-readable/contracts.json` | every cross-axis contract |
| Regulatory pack catalog | `regional-packs/<pack>/PACK.md` + `machine-readable/regional-packs.json` | every regional pack |
| Risk register | [RISK-REGISTER.md](RISK-REGISTER.md) + `machine-readable/risks.json` | every risk |
| Compliance matrix | [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md) + `machine-readable/compliance.json` | every regulator × control |
| Contradiction ledger | [CONTRADICTION-LEDGER.md](CONTRADICTION-LEDGER.md) + `machine-readable/contradictions.json` | every contradiction |
| Vendor + partner ledger | [VENDOR-PARTNER-LEDGER.md](VENDOR-PARTNER-LEDGER.md) | every external dep + partner |
| Glossary | [GLOSSARY.md](GLOSSARY.md) + `machine-readable/glossary.json` | every domain term |
| Team charters | [teams/](teams/) | every team |
| Per-product PRDs | [products/](products/) | every product |
| Standards | `standards/` (planned per §7) | every cross-cutting standard |
| Templates | [templates/](templates/) | every artifact template |
| Checklists | [templates/checklists/](../templates/checklists/) | every common task |
| Skills | installed runtime + project `.claude/skills/` / `.codex/skills/` + optional `.grok/` (mm-delivery); not `.omc/skills/` | every agent skill |
| Hooks | `.claude/hooks/` + `.git/hooks/` + `scripts/hooks/` | every gate |

---

## 10. Authoring norm

When you start a new piece of work:
1. Read this doc (or its machine-readable mirror)
2. Find the matching template; copy it
3. Find the matching checklist; do every step
4. If you hit a hook block, fix it (don't `--no-verify`)
5. Output the artifact in the canonical location with the canonical structure
6. The `oya verify` and CI lanes will validate

This is the *contract*: zero bespoke artifacts. Standardization-first.

---

## 11. Sources scanned

- [DOC-CATALOG.md](DOC-CATALOG.md), [TOOLCHAIN.md](TOOLCHAIN.md), [DOCUMENTATION.md](DOCUMENTATION.md)
- ADR-0015 (repo structure), ADR-0050 (governance umbrella), ADR-0001 (deprecation), ADR-0040 (per-endpoint deprecation telemetry), ADR-0040 (launch readiness)
- CLAUDE.md (project memory), `.claude/skills/`, `.claude/hooks/`, `.github/workflows/`, `scripts/hooks/`
- All consolidated docs at `docs/`
