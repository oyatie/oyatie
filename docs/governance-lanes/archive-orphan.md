---
doc_status: retired
retired_by: docs/decisions/ADR-0700-ci-admission-live-apex.md
retired_on: 2026-05-16
---

# Fitness Lane: archive-orphan

- status: Retired
- date: 2026-05-12
- retired_on: 2026-05-16
- retired_by: `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- retirement_reason: The one-time pre-cutover archive boundary has served its purpose; the archived payload is removed, and ADR-0116 makes the Foundry pipeline (M01-P18) the canonical concurrent-work substrate.
- former_kernel_crate: `governance-archive-orphan-kernel` — retired; removed from workspace members.
- former_runner_path: `tools/governance-archive-orphan-app` — retired; removed from workspace members.
- naming_justification: `archive-orphan` remains only as a retired lane id because IP-008 and prior evidence used that exact one-time archive-boundary name.
