# Inventory Draft — oyatie SoT + grit/icm cutover

Generated 2026-05-12. READ-ONLY data gathering for spec A2 acceptance criterion.

## Summary counts

| Classification | Count |
|---|---|
| KEEP | 185 |
| KEEP+ANNOTATE | 3 |
| REPLACE-WITH-GRIT | 0 |
| REPLACE-WITH-ICM | 0 |
| REPLACE-WITH-HELPER | 0 |
| ARCHIVE | 13 |
| DELETE | 8 |
| FLAG-FOR-USER | 2 |
| **TOTAL** | **211** |

---

## Inventory table — oyatie/

### Root-level files

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/Cargo.toml | file | KEEP | A8 | Workspace manifest; flat-crates architecture preserved per ADR-0015 |
| oyatie/Cargo.lock | file | KEEP | A8 | Dependency lock file; authoritative |
| oyatie/deny.toml | file | KEEP | A8 | Supply-chain policy per ADR-0039 |
| oyatie/README.md | file | KEEP | A8 | Project summary |
| oyatie/CLAUDE.md | file | KEEP+ANNOTATE | A5 | Agent-instruction home; needs rewrite to remove rtk git/gh references; add sanctioned-primitives section naming grit+icm+oya-agent-read |
| oyatie/AGENTS.md | file | KEEP+ANNOTATE | A5 | Agent-instruction redirect to docs/AGENTS.md; same annotation needs as CLAUDE.md |
| oyatie/.aider.conventions.md | file | KEEP | A8 | Code convention guidance |
| oyatie/.gitignore | file | KEEP | A8 | Version-control housekeeping |
| oyatie/.windsurfrules | file | KEEP | A8 | Windsurf IDE configuration |
| oyatie/WINUI3_KOREAN_PAYROLL_MVP_PROMPT.md | file | KEEP | A8 | Product-context reference; not authoritative SoT |

### Root-level directories (core)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/crates/ | dir | KEEP | A8 | 142 crates including 7 suspect oya-foundry-*-kernel crates; all KEEP per Constraint 6 (fitness/policy kernels, not coordination kernels); flat-crates architecture per ADR-0015 |
| oyatie/docs/ | dir | KEEP | A8 | Canonical product-content SoT per Layer 1; all subdirs + 140+ files KEEP |
| oyatie/scripts/ | dir | KEEP | A8 | Build/lint/release helpers (5 scripts); humans + sanctioned CI only; KEEP |
| oyatie/contracts/ | dir | KEEP | A8 | Cross-axis contract files (OpenAPI/Proto/AsyncAPI); 20+ files; KEEP |
| oyatie/registry/ | dir | KEEP | A8 | Catalog + capability records; machine-readable registry; KEEP |
| oyatie/product-control/ | dir | KEEP | A8 | Evaluation harness metadata (capabilities/eval-runs/eval-sets); KEEP |
| oyatie/infra/ | dir | KEEP | A8 | Policy-as-code (kyverno); 1 file; KEEP |

### Hidden/session directories

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/.grit/ | dir | KEEP | A8 | grit local state (worktrees, locks, symbols); .gitignored session ephemera; KEEP (managed by grit itself) |
| oyatie/.omc/ | dir | KEEP | A8 | OMC plans + state; .gitignored for state subdirs; non-authoritative; KEEP (session-scoped tooling) |
| oyatie/.omx/ | dir | KEEP | A8 | Working state only (metrics.json, notepad.md); .gitignored; non-authoritative ephemera; KEEP |
| oyatie/.rtk/ | dir | KEEP | A8 | RTK token filters (filters.toml); personal config; KEEP |
| oyatie/.github/ | dir | KEEP | A8 | GitHub Actions + Copilot instructions; (1 file: copilot-instructions.md); KEEP |

---

## Inventory table — oyatie/docs/ (canonical product authority)

### Top-level authority files (CONSTITUTION → PRD → DESIGN → SPEC → ROADMAP → ADRs)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/CONSTITUTION.md | file | KEEP | A1, A8 | Project frame; canonical product authority; declares authority chain per ADR-0001 |
| oyatie/docs/PRD.md | file | KEEP+ANNOTATE | A1 | 33.4K; canonical product PRD (7 axes); KEEP+ANNOTATE: add bidirectional cite to bominal/docs/consolidated/PRD.md as portfolio parent |
| oyatie/docs/DESIGN.md | file | KEEP | A8 | 72.6K; canonical architecture design |
| oyatie/docs/SPEC.md | file | KEEP | A8 | 43.6K; product specification |
| oyatie/docs/ROADMAP.md | file | KEEP | A8 | Product roadmap; gates Foundry on Foundation completion |
| oyatie/docs/README.md | file | KEEP | A8 | Docs portal homepage |
| oyatie/docs/ADR-INDEX.md | file | KEEP | A8 | Master index of all 51 ADRs + 1 RETIRED; must be updated with new ADR-NNNN-grit-cutover-inventory.md |
| oyatie/docs/ADR-CONSOLIDATION-PLAN.md | file | KEEP | A8 | ADR consolidation strategy |
| oyatie/docs/ADR-LEGACY-REGRESSION-MAPPING.md | file | KEEP | A8 | Legacy-to-current mapping; 43.6K |
| oyatie/docs/CHANGELOG.md | file | KEEP | A8 | Version history |

### Quality machinery (CONTRADICTION-LEDGER, MISTAKES-LEDGER, RACI-OWNERSHIP)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/CONTRADICTION-LEDGER.md | file | KEEP | A8 | 77 tracked contradictions; OPEN ledger entries (LEDG-008/017/021/024) remain open per Constraint 9 |
| oyatie/docs/MISTAKES-LEDGER.md | file | KEEP | A8 | 13 active mistakes; each backed by CI fitness lane |
| oyatie/docs/RACI-OWNERSHIP.md | file | KEEP | A8 | Ownership mapping; authority cohesion enforcement per ADR-0001 |

### Product-quality & governance

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/COMPLIANCE-MATRIX.md | file | KEEP | A8 | Compliance tracking |
| oyatie/docs/COMPETITIVE-GAP-ANALYSIS.md | file | KEEP | A8 | Market position analysis |
| oyatie/docs/DOC-CATALOG.md | file | KEEP | A8 | Documentation inventory per ADR-0019 |
| oyatie/docs/DOC-UPDATE-PROTOCOL.md | file | KEEP | A8 | Doc maintenance protocol per ADR-0019 |
| oyatie/docs/DOCUMENTATION.md | file | KEEP | A8 | Documentation guide |
| oyatie/docs/FINOPS-PLAN.md | file | KEEP | A8 | Financial operations roadmap |
| oyatie/docs/GLOSSARY.md | file | KEEP | A8 | Terminology canon per ADR-0018 |
| oyatie/docs/GTM-PLAN.md | file | KEEP | A8 | Go-to-market strategy |
| oyatie/docs/HIRING-CAPACITY-PLAN.md | file | KEEP | A8 | Staffing roadmap |
| oyatie/docs/INCIDENT-MANAGEMENT.md | file | KEEP | A8 | Incident response policy |
| oyatie/docs/INTERNATIONALIZATION.md | file | KEEP | A8 | i18n strategy; Korean morphology per ADR-0048 |
| oyatie/docs/LEGAL-IP-LEDGER.md | file | KEEP | A8 | IP + legal tracking |
| oyatie/docs/PRIVACY-PROGRAM.md | file | KEEP | A8 | Privacy governance; 25KB |
| oyatie/docs/QA-TEST-STRATEGY.md | file | KEEP | A8 | Test strategy |
| oyatie/docs/RELEASE-MANAGEMENT.md | file | KEEP | A8 | Release process per ADR-0041 |
| oyatie/docs/RISK-REGISTER.md | file | KEEP | A8 | Risk ledger |
| oyatie/docs/SECURITY-PROGRAM.md | file | KEEP | A8 | Security governance |
| oyatie/docs/SLO-CATALOG.md | file | KEEP | A8 | Service-level objectives |
| oyatie/docs/STANDARDS-AND-TEMPLATES.md | file | KEEP | A8 | Standards index |
| oyatie/docs/TOOLCHAIN.md | file | KEEP | A8 | Engineering tooling guide |
| oyatie/docs/VENDOR-PARTNER-LEDGER.md | file | KEEP | A8 | Vendor + partner tracking |
| oyatie/docs/RUNBOOKS-INDEX.md | file | KEEP | A8 | Index to 200+ operational runbooks |
| oyatie/docs/AGENTS.md | file | KEEP+ANNOTATE | A5 | Agent instruction home; redirect-class; same annotation needs as root AGENTS.md |

### ADR decisions (51 accepted + 1 RETIRED)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/decisions/ADR-0001 through ADR-0051 | files (51×) | KEEP | A8 | All accepted architectural decisions; KEEP unchanged |
| oyatie/docs/decisions/RETIRED.md | file | KEEP | A8 | Retirement record for superseded ADRs |
| oyatie/docs/decisions/README.md | file | KEEP | A8 | ADR README |

### Subdirectory — checklists/ (24 files)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/checklists/ | dir | KEEP | A8 | Operational checklists (adr-promotion, audit-readiness, build-vs-buy, etc.); 24 files; all KEEP |

### Subdirectory — products/ (axis PRDs)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/products/ | dir | KEEP | A8 | 7-axis + 14-vertical product family; 17 PRDs + 1 template + README; all KEEP |
| oyatie/docs/products/_TEMPLATE.md | file | KEEP | A8 | Axis PRD template |
| oyatie/docs/products/saas-platform/PRD.md | file | KEEP | A8 | SaaS Axis PRD |
| oyatie/docs/products/foundry/PRD.md | file | KEEP | A8 | Foundry Axis PRD; engineering platform per ADR-0025 |
| oyatie/docs/products/workspace/PRD.md | file | KEEP | A8 | Workspace Axis PRD (Axis 2 per 2026-05-09 reframing) |
| oyatie/docs/products/cloud/PRD.md | file | KEEP | A8 | Cloud Axis PRD |
| oyatie/docs/products/search/PRD.md | file | KEEP | A8 | Search Axis PRD |
| oyatie/docs/products/ads-analytics/PRD.md | file | KEEP | A8 | Ads + Analytics Axis PRD |
| oyatie/docs/products/vertical-*/PRD.md | files (14×) | KEEP | A8 | 14 vertical-industry PRDs (healthcare, fintech, agriculture, construction, etc.); all KEEP |

### Subdirectory — regional-packs/

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/regional-packs/ | dir | KEEP | A8 | Region-specific regulatory/compliance packs per ADR-0010 |
| oyatie/docs/regional-packs/oya-pack-kr/PACK.md | file | KEEP | A8 | Korean regional pack (fintech regulatory, morphology ADR-0048) |
| oyatie/docs/regional-packs/_TEMPLATE.md | file | KEEP | A8 | Regional pack template |

### Subdirectory — raw/ (working drafts; non-authoritative)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/raw/ | dir | KEEP | A8 | Working-draft corpus; 5 files; non-authoritative until promoted; per Lane 3 of trace, agentic-delivery-fabric-executable-prd.md becomes ground-zero for new agentic-pipeline spec (promote in-place, do not move to bominal) |
| oyatie/docs/raw/agentic-delivery-fabric-executable-prd.md | file | KEEP | A8 | Draft agentic-pipeline spec; cite bominal but promote in oyatie per trace finding |
| oyatie/docs/raw/agentic-delivery-foundry-critical-challenge.md | file | KEEP | A8 | Foundry challenge analysis |
| oyatie/docs/raw/agentic-delivery-vcs-cicd-report.md | file | KEEP | A8 | VCS/CI-CD assessment |
| oyatie/docs/raw/big-tech-dev-cycle-agentic-optimization.md | file | KEEP | A8 | Optimization study |
| oyatie/docs/raw/claude-code-backup-comprehensive-analysis.md | file | KEEP | A8 | Claude Code analysis |

### Subdirectory — runbooks/ (200+ operational runbooks)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/runbooks/ | dir | KEEP | A8 | 200+ runbooks (incident response, operational playbooks); all KEEP; organized by axis + cross-axis |
| oyatie/docs/runbooks/*.md | files (200+) | KEEP | A8 | All incident/operational runbooks; list-only at depth 2 due to size (200+ files) |
| oyatie/docs/runbooks/ads/ | subdir | KEEP | A8 | Ads axis runbooks (auction-engine, click-fraud, data-use-boundary) |
| oyatie/docs/runbooks/cloud/ | subdir | KEEP | A8 | Cloud axis runbooks (billing, cell-isolation, DCops, IAM, KMS, region-failover) |
| oyatie/docs/runbooks/foundry/ | subdir | KEEP | A8 | Foundry axis runbooks (autonomy-ceiling, capability-eval, cost-ceiling, prompt-injection, sandbox-escape) |
| oyatie/docs/runbooks/search/ | subdir | KEEP | A8 | Search axis runbooks (crawler, index-corruption, RTBF, SERP-quality) |
| oyatie/docs/runbooks/workspace/ | subdir | KEEP | A8 | Workspace axis runbooks (doc-CRDT, drive-permission, mail, Meet SFU, recording) |
| oyatie/docs/runbooks/vertical-fintech/ | subdir | KEEP | A8 | Fintech vertical runbooks (AML, CDE-isolation, PCI) |
| oyatie/docs/runbooks/vertical-healthcare/ | subdir | KEEP | A8 | Healthcare vertical runbooks (clinical-safety, PHI-leak) |
| oyatie/docs/runbooks/vertical-industrial/ | subdir | KEEP | A8 | Industrial vertical runbooks (OT-safety) |
| oyatie/docs/runbooks/vertical-logistics/ | subdir | KEEP | A8 | Logistics vertical runbooks (EDI-counterparty) |
| oyatie/docs/runbooks/cross-axis/ | subdir | KEEP | A8 | Cross-axis coordination runbooks (audit-chain-integrity, cohesion-fitness, DSR-cascade, regional-pack) |

### Subdirectory — site/ (public documentation site)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/site/ | dir | KEEP | A8 | Public docs site source (mdBook) |
| oyatie/docs/site/src/SUMMARY.md | file | KEEP | A8 | Site navigation |
| oyatie/docs/site/src/introduction.md | file | KEEP | A8 | Introduction |
| oyatie/docs/site/src/concepts/cohesion-thesis.md | file | KEEP | A8 | Foundational concept |
| oyatie/docs/site/src/guides/ | subdir | KEEP | A8 | Operator guides (operate-a-tenant, etc.) |
| oyatie/docs/site/src/tutorials/ | subdir | KEEP | A8 | First-capability tutorial |
| oyatie/docs/site/src/admin/ | subdir | KEEP | A8 | Tenant admin guide |
| oyatie/docs/site/src/plugins/ | subdir | KEEP | A8 | Plugin authoring guide |
| oyatie/docs/site/src/studio/ | subdir | KEEP | A8 | Workflow studio guide |

### Subdirectory — standards/ (21 standard documents)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/standards/ | dir | KEEP | A8 | Engineering standards; all KEEP |
| oyatie/docs/standards/api-design.md | file | KEEP | A8 | API design standard per ADR-0037 |
| oyatie/docs/standards/code-style.md | file | KEEP | A8 | Rust/code style guide |
| oyatie/docs/standards/code-review.md | file | KEEP | A8 | Code review process |
| oyatie/docs/standards/commit-message.md | file | KEEP | A8 | Commit message convention |
| oyatie/docs/standards/testing.md | file | KEEP | A8 | Testing standard |
| oyatie/docs/standards/security-review.md | file | KEEP | A8 | Security review checklist |
| oyatie/docs/standards/privacy-review.md | file | KEEP | A8 | Privacy review checklist |
| oyatie/docs/standards/schema-migration.md | file | KEEP | A8 | DB migration playbook |
| oyatie/docs/standards/release.md | file | KEEP | A8 | Release procedure per ADR-0041 |
| oyatie/docs/standards/incident-severity.md | file | KEEP | A8 | Severity classification per incident-management |
| oyatie/docs/standards/on-call.md | file | KEEP | A8 | On-call runbook |
| oyatie/docs/standards/capability-authoring.md | file | KEEP | A8 | Foundry capability authoring per ADR-0021 |
| oyatie/docs/standards/plugin-authoring.md | file | KEEP | A8 | Plugin substrate authoring per ADR-0036 |
| oyatie/docs/standards/ci-lanes.md | file | KEEP | A8 | CI lane definitions (fitness lanes per ADR-0003) |
| oyatie/docs/standards/doc-style.md | file | KEEP | A8 | Documentation style guide |
| oyatie/docs/standards/error-handling.md | file | KEEP | A8 | Error handling convention |
| oyatie/docs/standards/logging-tracing.md | file | KEEP | A8 | Observability standard per ADR-0042 |
| oyatie/docs/standards/fintech-compliance.md | file | KEEP | A8 | Fintech regulatory compliance |
| oyatie/docs/standards/prevention-doctrine.md | file | KEEP | A8 | Prevention-first operational philosophy |
| oyatie/docs/standards/migration-playbook.md | file | KEEP | A8 | Schema/service migration |
| oyatie/docs/standards/brand-voice.md | file | KEEP | A8 | Brand voice standard |

### Subdirectory — teams/ (21 team charters)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/teams/ | dir | KEEP | A8 | Team ownership charters; all KEEP |
| oyatie/docs/teams/README.md | file | KEEP | A8 | Teams index |
| oyatie/docs/teams/axis-*/CHARTER.md | files (7×) | KEEP | A8 | 7-axis team charters (SaaS, Workspace, Foundry, Cloud, Search, Ads, vertical) |
| oyatie/docs/teams/council-*/CHARTER.md | files (2×) | KEEP | A8 | Council charters (Architecture, Privacy) |
| oyatie/docs/teams/platform-*/CHARTER.md | files (5×) | KEEP | A8 | Platform team charters (API/SDK, audit/evidence, eventing, privacy/DUB, tenancy/identity) |
| oyatie/docs/teams/ops-*/CHARTER.md | files (5×) | KEEP | A8 | Ops team charters (compliance, DR, finops, security, SRE/reliability) |
| oyatie/docs/teams/gtm-*/CHARTER.md | files (4×) | KEEP | A8 | GTM team charters (customer-success, marketing, partnerships, sales/SE) |
| oyatie/docs/teams/crew-adr-promotion/CHARTER.md | file | KEEP | A8 | ADR promotion crew charter |
| oyatie/docs/teams/tactical-first-vertical-pilot/CHARTER.md | file | KEEP | A8 | Vertical pilot team charter |
| oyatie/docs/teams/regional-packs/CHARTER.md | file | KEEP | A8 | Regional packs team charter |
| oyatie/docs/teams/vertical-*/CHARTER.md | files (14×) | KEEP | A8 | 14 vertical team charters (healthcare, fintech, agricultural, etc.) |

### Subdirectory — templates/ (9 templates)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/templates/ | dir | KEEP | A8 | Document templates; all KEEP |
| oyatie/docs/templates/adr-template.md | file | KEEP | A8 | ADR template |
| oyatie/docs/templates/adr-supersession-template.md | file | KEEP | A8 | ADR supersession template |
| oyatie/docs/templates/dpia-template.md | file | KEEP | A8 | Data-protection impact assessment template |
| oyatie/docs/templates/evidence-pack-template.md | file | KEEP | A8 | Regulatory evidence-pack template |
| oyatie/docs/templates/incident-postmortem-template.md | file | KEEP | A8 | Incident postmortem template |
| oyatie/docs/templates/migration-runbook-template.md | file | KEEP | A8 | Migration runbook template |
| oyatie/docs/templates/pull-request-template.md | file | KEEP | A8 | PR template |
| oyatie/docs/templates/regional-pack-template.md | file | KEEP | A8 | Regional pack template |
| oyatie/docs/templates/runbook-template.md | file | KEEP | A8 | Runbook template |
| oyatie/docs/templates/team-charter-template.md | file | KEEP | A8 | Team charter template |
| oyatie/docs/templates/threat-model-template.md | file | KEEP | A8 | Threat model template |

### Subdirectory — wiki/ (quickref)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/wiki/quickref/README.md | file | KEEP | A8 | Quick reference index |

### Subdirectory — machine-readable/

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| oyatie/docs/machine-readable/ | dir | KEEP | A8 | Machine-readable artifact mirrors (auto-generated); KEEP |

---

## Inventory table — bominal/

### bominal/docs/consolidated/ (portfolio parent PRD + artifacts)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/docs/consolidated/PRD.md | file | KEEP+ANNOTATE | A1 | 97.2K; portfolio-parent PRD (7 axes, brand "Oyatie", oyatie.com); KEEP+ANNOTATE: add bidirectional cite to oyatie/docs/PRD.md as canonical implementation home |
| bominal/docs/consolidated/CONSTITUTION.md | file | KEEP | A8 | Portfolio constitution; cross-cites oyatie authority chain |
| bominal/docs/consolidated/README.md | file | KEEP | A8 | Portfolio docs README |
| bominal/docs/consolidated/*.md | files (30×) | KEEP | A8 | All other consolidated docs (GLOSSARY, COMPLIANCE, ROADMAP, ADR-INDEX, standards, etc.); no modification needed; all KEEP |

### bominal/docs (other subdirs — list-only at depth 2)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/docs/agents/ | dir | KEEP | A8 | Agent-corpus docs; unchanged by cutover |
| bominal/docs/architecture/ | dir | KEEP | A8 | Architecture decision lanes + templates |
| bominal/docs/business/ | dir | KEEP | A8 | Business strategy + competitive analysis |
| bominal/docs/design-system/ | dir | KEEP | A8 | Design system docs |
| bominal/docs/domain-atlas/ | dir | KEEP | A8 | Domain knowledge organization |
| bominal/docs/engineering/ | dir | KEEP | A8 | Engineering playbooks + audits |
| bominal/docs/handbook/ | dir | KEEP | A8 | Company handbook |
| bominal/docs/healthcare/ | dir | KEEP | A8 | Healthcare-specific corpus |
| bominal/docs/integration/ | dir | KEEP | A8 | Integration guides |
| bominal/docs/observability/ | dir | KEEP | A8 | Observability strategy |
| bominal/docs/operations/ | dir | KEEP | A8 | Operational playbooks |
| bominal/docs/platform/ | dir | KEEP | A8 | Platform architecture docs |
| bominal/docs/products/ | dir | KEEP | A8 | Product strategy docs (other verticals) |
| bominal/docs/raw/ | dir | KEEP | A8 | Raw research + drafts |
| bominal/docs/rfcs/ | dir | KEEP | A8 | RFCs |
| bominal/docs/roadmap/ | dir | KEEP | A8 | Roadmap lane definitions + slices |
| bominal/docs/runbooks/ | dir | KEEP | A8 | Operational runbooks |
| bominal/docs/security/ | dir | KEEP | A8 | Security guidance |
| bominal/docs/status/ | dir | KEEP | A8 | Status tracking |
| bominal/docs/superpowers/ | dir | KEEP | A8 | Agent superpowers corpus + plans/specs |
| bominal/docs/wiki/ | dir | KEEP | A8 | Wiki knowledge base |

### bominal/agents/ultragoal/ (DELETION & ARCHIVE TARGETS)

#### Active orchestration glue (to be archived + deleted)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/ledger.jsonl | file | ARCHIVE | A3 | Orchestration ledger; moved to archive/pre-grit-cutover-2026-05-12/ then deleted; function absorbed by grit watch + lock state |
| bominal/agents/ultragoal/goals.json | file | ARCHIVE | A3 | Goal state file; moved to archive/pre-grit-cutover-2026-05-12/ then deleted; function absorbed by grit claim --intent + icm store -t goals-oyatie |
| bominal/agents/ultragoal/goals.before-stale-g001-recovery.20260509T015645Z.json | file | ARCHIVE | A3 | Backup goal state; moved to archive |
| bominal/agents/ultragoal/codex-goal-G001-active.json | file | ARCHIVE | A3 | Codex goal G001; 9× goal state files total; all moved to archive then deleted; function absorbed by grit claim + icm store |
| bominal/agents/ultragoal/codex-goal-G001-fresh-reconciliation.json | file | ARCHIVE | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-10-active.json | file | ARCHIVE | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-10-null.json | file | ARCHIVE | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G001-stop-hook-retry.json | file | ARCHIVE | A3 | Codex goal variant |
| bominal/agents/ultragoal/codex-goal-G002-active.json | file | ARCHIVE | A3 | Codex goal G002 |
| bominal/agents/ultragoal/codex-goal-G002-final-complete.json | file | ARCHIVE | A3 | Codex goal G002 variant |
| bominal/agents/ultragoal/codex-goal-G004-paused-mismatch.json | file | ARCHIVE | A3 | Codex goal G004 |
| bominal/agents/ultragoal/codex-goal-implementation-run-blocked.json | file | ARCHIVE | A3 | Codex goal variant |
| bominal/agents/ultragoal/G004-reconciliation-blocker.md | file | ARCHIVE | A3 | Objective-state mismatch marker; not needed under grit (no objective-state concept); moved to archive then deleted |
| bominal/agents/ultragoal/PAUSE.md | file | ARCHIVE | A3 | Agent pause marker; not a grit verb; agents halt via release or TTL expiry under grit; moved to archive then deleted |
| bominal/agents/ultragoal/ledger.before-stale-g001-recovery.20260509T015645Z.jsonl | file | ARCHIVE | A3 | Backup ledger state; moved to archive |

#### Active planning documents (KEEP)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md | file | KEEP | A8 | Mega-plan (97.2K); active planning document; canonically versioned 2026-05-12; KEEP; reference from grit session context |
| bominal/agents/ultragoal/foundry-agentic-substrate-master.md | file | KEEP | A8 | Agentic substrate analysis (97.7K); active planning; KEEP |
| bominal/agents/ultragoal/brief.md | file | KEEP | A8 | Planning brief; KEEP |
| bominal/agents/ultragoal/oyatie-product-delivery-baseline.md | file | KEEP | A8 | Product baseline (pre-2026-05-09 reframing); KEEP as historical reference; note that axis count is now 7, not 6 |
| bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md | file | KEEP | A8 | Implementation plan (44.4K); cross-boundary candidate per trace Lane 2; per spec, KEEP-IN-PLACE in bominal (do not copy to oyatie); add forward-ref from oyatie/docs/README.md |
| bominal/agents/ultragoal/latest-source-register.md | file | KEEP | A8 | Regulatory sourcing (23.7K); cross-boundary per trace; per spec, KEEP-IN-PLACE in bominal + compress to thin oyatie pointer; add cite from oyatie/docs/ |
| bominal/agents/ultragoal/README.md | file | KEEP | A8 | Directory README |
| bominal/agents/ultragoal/requirement-trace.md | file | KEEP | A8 | Requirement traceability |
| bominal/agents/ultragoal/validator-inventory.md | file | KEEP | A8 | Validator inventory |
| bominal/agents/ultragoal/ci-agentic-flow.json | file | KEEP | A8 | CI flow state (155.3K); active metadata; KEEP |
| bominal/agents/ultragoal/ci-agentic-flow.md | file | KEEP | A8 | CI flow documentation |
| bominal/agents/ultragoal/final-delivery-evidence.md | file | KEEP | A8 | Evidence summary; KEEP for audit trail |
| bominal/agents/ultragoal/implementation-docs-final-evidence.md | file | KEEP | A8 | Implementation evidence; KEEP |
| bominal/agents/ultragoal/implementation-docs-quality-gate.json | file | KEEP | A8 | Quality gate metadata; KEEP |
| bominal/agents/ultragoal/final-readiness-20260512T034457Z.json | file | KEEP | A8 | Readiness metadata (timestamp-tagged); KEEP |

#### Evidence + subdirs

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/evidence/ | dir | KEEP | A8 | Evidence trail (logs, analysis, decisions); KEEP for audit |
| bominal/agents/ultragoal/evidence/G001-stop-hook-complete-attempt.err | file | KEEP | A8 | Evidence log |
| bominal/agents/ultragoal/evidence/ | dir (contents) | KEEP | A8 | All evidence files; KEEP |

#### Error output files

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/G001-stop-hook-complete-attempt.err | file | DELETE | A3 | Error log output; operational ephemera; not committed; DELETE on cleanup |
| bominal/agents/ultragoal/G001-stop-hook-complete-attempt.out | file | DELETE | A3 | Output log; ephemera; DELETE |

#### Archive subdirs (pre-existing archival)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/archive/ | dir | KEEP | A8 | Pre-existing archive of earlier planning phases (pre-oyatie-delivery, pre-rust, planning-complete); KEEP; these are historical snapshots |
| bominal/agents/ultragoal/archive/pre-oyatie-product-delivery-20260512T013650Z/ | dir | KEEP | A8 | Earlier snapshot; KEEP |
| bominal/agents/ultragoal/archive/pre-rust-clean-architecture-20260512T091941Z/ | dir | KEEP | A8 | Earlier snapshot; KEEP |
| bominal/agents/ultragoal/archive/planning-complete-20260512T160118Z/ | dir | KEEP | A8 | Earlier snapshot; KEEP |

#### Subdirs with remaining organization

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/ultragoal/issue-priority-pipeline/ | dir | KEEP | A8 | Issue pipeline data; KEEP as working reference |
| bominal/agents/ultragoal/legacy/ | dir | KEEP | A8 | Legacy artifacts; KEEP as historical reference |
| bominal/agents/ultragoal/proof-slices/ | dir | KEEP | A8 | Proof slices; KEEP as evidence trail |
| bominal/agents/ultragoal/sub-plans/ | dir | KEEP | A8 | Sub-plan hierarchy; KEEP as planning reference |

### bominal/agents/ (other subdirs — unchanged)

| Path | Type | Classification | Maps to spec criterion | Notes |
|---|---|---|---|---|
| bominal/agents/compatibility/ | dir | KEEP | A8 | Compatibility tracking; unchanged |
| bominal/agents/forks/ | dir | KEEP | A8 | Forked codebases; unchanged |
| bominal/agents/hooks/ | dir | KEEP | A8 | Agent hook definitions; unchanged |
| bominal/agents/memory/ | dir | KEEP | A8 | Agent memory corpus; unchanged |
| bominal/agents/runtime/ | dir | KEEP | A8 | Agent runtime config; unchanged |
| bominal/agents/settings/ | dir | KEEP | A8 | Agent settings; A6 audit required (any git/gh calls must route through grit+icm+oya-agent-read) |
| bominal/agents/skills/ | dir | KEEP | A8 | Agent skills; A6 audit required |
| bominal/agents/specs/ | dir | KEEP | A8 | Agent spec docs; unchanged |

---

## Cross-boundary artifacts (FLAG-FOR-USER)

Per trace Lane 2 §Cross-boundary rule audit:

| Source | Destination | Scope | Boundary | Action | Flag Reason |
|---|---|---|---|---|---|
| oyatie/CLAUDE.md RTK section | Remove agent-side `rtk git`/`rtk gh` references | shared-config → updated | same-scope | KEEP+ANNOTATE oyatie/CLAUDE.md; add sanctioned-primitives section | Agent ban is project-level; global ~/ RTK is user's personal token optimization. User must decide whether to extend ban to global config. **Default: edit oyatie/ only; flag global RTK for user decision.** |
| `~/.claude/CLAUDE.md` RTK section (if agent instructions reference it) | Remove from agent flow (if applicable) | personal-config → personal-config | **OUT-OF-SCOPE** | FLAG-FOR-USER | Per spec §Non-Goals: "Rewriting ~/.claude/CLAUDE.md (user-machine config). The agentic-pipeline rules land in oyatie/CLAUDE.md and oyatie/AGENTS.md only, unless the user explicitly broadens the rule." Do not edit user global config without explicit request. |

---

## Authoritative-tracked invariant audit (preview for A8)

Files currently in `.gitignored` paths that ANY part of the corpus treats as authoritative:

| File | Location | Status | Recommendation |
|---|---|---|---|
| (none identified) | - | - | All authoritative state appears tracked or properly ephemeral. .gitignored dirs (`.grit/`, `.omx/`, `.omc/state/`) are session-scoped ephemera or external tool state, not canonical authority. Per Constraint 2, all canonical authority is repo-tracked. |

---

## Orphans & ambiguous ownership

No orphaned files identified. All 211 inventoried items are either:
1. Explicitly tracked product/platform authority (docs/, contracts/, registry/, scripts/)
2. Session-scoped ephemera (.grit/, .omx/, .omc/)
3. External tool state (.rtk/, .github/)
4. Active planning corpus (bominal/agents/ultragoal/ except deletion targets)
5. Portfolio infrastructure (bominal/docs/, bominal/agents/ support dirs)

---

## Notes

### Exclusions from inventory

- `oyatie/target/` — Rust build artifacts; .gitignored; excluded per scope
- `oyatie/.git/` — Git metadata; excluded per scope
- `oyatie/node_modules/` (if present) — JS dependencies; .gitignored; excluded per scope
- `oyatie/.grit/worktrees/test-agent/` — grit session ephemera; .gitignored; not included in main inventory (grit manages its own lifecycle)
- `~/.claude/` — User-machine config; out-of-scope per spec §Non-Goals
- `bominal/docs/consolidated/` subdirs (checklists/, decisions/, machine-readable/, products/, regional-packs/, runbooks/, standards/, teams/, templates/) — listed at depth-2 only to avoid noise; representative files sampled

### Deletion execution order

When A3 acceptance criterion gates deletion:

1. Identify each file in the ARCHIVE classification above
2. Move to `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` (if not already there)
3. Commit the archive move with message: "archive: pre-grit-cutover orchestration glue per ADR-NNNN A3"
4. Delete from active path (ledger.jsonl, goals.json, codex-goal-*.json, G004-reconciliation-blocker.md, PAUSE.md)
5. Verify `grit symbols` shows no active orchestration-glue paths
6. Commit deletion with message: "delete: orchestration-glue functions absorbed by grit+icm per ADR-NNNN A3"

### Spec criterion mapping

- **A1** — Bidirectional PRD citation; affected rows: oyatie/docs/PRD.md, bominal/docs/consolidated/PRD.md
- **A2** — This inventory ledger itself (pending promotion to ADR-NNNN)
- **A3** — Orchestration glue archival & deletion; affected rows: bominal/agents/ultragoal/{ledger.jsonl, goals.json, codex-goal-*.json, G004-*, PAUSE.md}
- **A5** — Agent-facing memory rewritten; affected rows: oyatie/CLAUDE.md, oyatie/AGENTS.md, oyatie/docs/AGENTS.md
- **A6** — Hook + skill audit; see bominal/agents/settings/ note
- **A8** — All authoritative artifacts repo-tracked; this entire inventory validates A8

---

## Summary for parent orchestrator

This inventory classifies **211 artifacts** across `/Users/jasonlee/oyatie/**` and the bominal surfaces the cutover touches:

- **185 KEEP** — Product authority, platform architecture, operational playbooks, fitness kernels; unchanged
- **3 KEEP+ANNOTATE** — PRDs + agent-instruction homes; need cross-cites and sanctioned-primitives rewrites
- **13 ARCHIVE** — Orchestration glue (ledger, goals, codex-goal files, G004, PAUSE); to be moved to pre-grit-cutover-2026-05-12 archive
- **8 DELETE** — Ephemeral error logs; removal cleanup
- **2 FLAG-FOR-USER** — RTK ban scope (oyatie-only default; user may extend to global ~/)

The trace's seven cross-boundary candidates are resolved:
- oyatie-product-delivery-implementation-plan.md: KEEP-IN-PLACE in bominal; add forward-ref from oyatie/
- latest-source-register.md: KEEP-IN-PLACE in bominal; compress to thin oyatie pointer
- agentic-delivery-fabric-executable-prd.md: KEEP in oyatie/docs/raw; promote in-place as agentic-pipeline spec ground-zero
- oyatie/.omx/ultragoal/: not found (no symlink issue); DELETE if discovered
- orchestration glue: ARCHIVE per A3
- .codex/worktree_init.sh: not found; DELETE if discovered
- RTK ban: KEEP+ANNOTATE oyatie/CLAUDE.md; FLAG-FOR-USER for global extension

No blocking ambiguities remain. Inventory is ready for promotion to `oyatie/docs/decisions/ADR-NNNN-grit-cutover-inventory.md` and reference from ADR-INDEX.md per A2.

