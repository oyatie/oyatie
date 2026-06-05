# registry/vcs/

Status: **retired historical evidence only**.

This directory is not an active SCM, queue, webhook, merge, review, CI, or
promotion substrate. It is preserved so old decisions and evidence that cite
the former agentic VCS event-log/router shape still resolve during audits.

Current authority lives elsewhere:

- Native SCM direction: `/specs/gitops-vcs-replacement.json`.
- P00/P0 sequencing: `/specs/masterplan.json`.
- CI required-context direction: `/specs/oya-ci-prow-capability-parity.json`.
- Retired external substrate tombstones:
  `/specs/retired-external-substrate-registry.json`.
- Repo hygiene automation:
  `buck2 build //:repo-hygiene-automation-check`.
- Kubernetes-native anti-pattern guard:
  `buck2 build //:kubernetes-native-anti-pattern-check`.

Operational rules:

1. Do not add rows or consumers under this directory.
2. Do not use these files as a merge queue, CI router, or agent dispatcher.
3. Do not revive the retired CLI wrapper or deleted app crates that used to
   consume these files.
4. New native SCM work must use Rust libraries/services, Buck2 targets, Prow
   jobs, and Git/GitHub adapters only as explicit compatibility seams.

Files:

- `changeset-event-log.json` — empty historical event-log seed.
- `webhook-delivery-log.json` — empty historical delivery-log seed.
- `event-router.yaml` — frozen historical router rows; data rows are provenance,
  not executable routing authority.
- `concurrent-safe-paths.yaml` — frozen historical path-overlap seed; not an
  active admission-gate registry.
