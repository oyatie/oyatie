---
id: ADR-0102
title: Foundry Settings Template Canonical Rendering
status: Accepted
doc_status: published
owner: council-architecture
date: 2026-05-15
---

# ADR-0102: Foundry Settings Template Canonical Rendering

## Status
Accepted

## Context
Multiple providers and multiple accounts require consistent settings (hooks, skills, MCP servers). Manual maintenance leads to drift.

## Decision
We will use a canonical `SettingsTemplate` value type in `oya-foundry-settings-template-kernel` and per-provider `SettingsRenderer` implementations in `oya-foundry-settings-template-adapter`. 
1. Drift is verified at `AccountSnapshotProvider::snapshot()` time.
2. Rendering is atomic (tempfile + rename).
3. Secret references (`sref://`) are resolved at render/spawn time via `SecretStorePort`.

## Drivers
- **Multi-account Consistency:** Ensure N accounts stay in sync.
- **Cross-provider Parity:** Normalize different CLI configuration formats into one workspace template.
- **Drift-free Onboarding:** Automatically configure new accounts on first tick.

## Consequences
- Introduces `oya-foundry-settings-template-kernel` and `oya-foundry-settings-template-adapter`.
- 3 template payloads in `templates/foundry-supervisor/`.
- `lean-settings-drift` CI lane to enforce parity.
