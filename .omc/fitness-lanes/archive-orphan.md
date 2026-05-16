
# Fitness Lane: archive-orphan

- status: Retired
- date: 2026-05-12
- retired_on: 2026-05-16
- retired_by: `docs/decisions/ADR-0118-retire-archive-orphan-fitness-lane.md`
- retirement_reason: The one-time pre-cutover archive boundary has served its purpose; the archived payload is removed, and ADR-0116 makes the Foundry pipeline (M-CC-P11) the canonical concurrent-work substrate.
- former_scope: Verified Bominal ultragoal orchestration-glue ARCHIVE rows under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/`, absent active originals, and zero living references outside authority/provenance docs.
- replacement: Foundry pipeline admission + projected-merge-state + conflict-kernel under M-CC-P11, with ADR-0116 retiring grit/rtk/icm/vox-era coordination surfaces.
- former_kernel_crate: `oya-foundry-fitness-archive-orphan-kernel` — retired; removed from workspace members.
- former_runner_path: `tools/oya-foundry-fitness-archive-orphan-app` — retired; removed from workspace members.
- naming_justification: `archive-orphan` remains only as a retired lane id because IP-008 and prior evidence used that exact one-time archive-boundary name.
