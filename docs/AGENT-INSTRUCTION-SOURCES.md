---
doc_class: AuditInventory
parent: .omc/plans/milestones/M01-foundation/phases/P01-agentic-pipeline-cutover/IP-007-hook-skill-audit.md
id: AGENT-INSTRUCTION-SOURCES
status: Accepted
generated_at: 2026-05-14 12:49:00+00:00
purpose: "This inventory enumerates repo-local files that contain exact `agent-instructions` fences after M01-P08-IP-007."
doc_status: published
---
# Agent Instruction Sources


| Path | Fences | Start lines | End lines | Rewrite verified |
|---|---:|---|---|---|
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-001-provider-account-pool-kernel.md` | 1 | `107` | `115` | yes |
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-002-anthropic-compat-adapter.md` | 1 | `100` | `108` | yes |
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-003-openai-compat-adapter.md` | 1 | `91` | `99` | yes |
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-004-oauth-subscription-capture.md` | 1 | `95` | `103` | yes |
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-005-upstream-api-drift-lane.md` | 1 | `99` | `107` | yes |
| `.omc/plans/milestones/M02-foundry-preview/phases/P02-multi-subscription-pool/IP-006-tos-policy-audit-chain.md` | 1 | `118` | `126` | yes |
| `.omc/standards/agent-instructions-discipline.md` | 1 | `39` | `41` | yes |
| `/templates/checklists/agent-completion-checklist.md` | 1 | `22` | `74` | yes |
| `/templates/checklists/agent-kickoff-checklist.md` | 1 | `21` | `60` | yes |
| `/templates/checklists/escalation-checklist.md` | 1 | `59` | `72` | yes |
| `/templates/checklists/inventory-update-checklist.md` | 1 | `65` | `69` | yes |
| `/templates/implementation-plan-template.md` | 1 | `71` | `79` | yes |
| `/templates/pull-request-template.md` | 1 | `30` | `35` | yes |
| `/templates/runbook-template.md` | 1 | `67` | `72` | yes |
| `AGENTS.md` | 1 | `9` | `22` | yes |
| `CLAUDE.md` | 1 | `9` | `22` | yes |
| `docs/AGENTS.md` | 1 | `145` | `164` | yes |
| `templates/checklists/agent-completion-checklist.md` | 1 | `26` | `78` | yes |
| `templates/checklists/agent-kickoff-checklist.md` | 1 | `25` | `64` | yes |
| `templates/checklists/escalation-checklist.md` | 1 | `63` | `76` | yes |
| `templates/checklists/inventory-update-checklist.md` | 1 | `69` | `73` | yes |
| `docs/standards/agent-instructions-discipline.md` | 1 | `43` | `45` | yes |
| `docs/templates/implementation-plan-template.md` | 1 | `75` | `83` | yes |
| `docs/templates/pull-request-template-v2.md` | 1 | `37` | `42` | yes |
| `docs/templates/runbook-template-v2.md` | 1 | `73` | `78` | yes |

Inventory total: 25 files; 25 fenced sections.


Validation command: `cargo run -p oya-governance-banned-primitives`. In this workspace the cargo invocation is still blocked by the pre-existing missing `crates/oya-tenancy-kernel/Cargo.toml`; standalone rustc validation for the lane is recorded in the IP-007 evidence bundle.
