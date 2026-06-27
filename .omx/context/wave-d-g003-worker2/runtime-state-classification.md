# G003-B repo hygiene/runtime-state classification

Task: worker-1 continuation for task 2 / G003-B repo hygiene/runtime-state classification.

## Scope

Owned write scope used:

- `.omx/context/wave-d-g003-worker2/**`
- `specs/workspace-hygiene.json` only, after announcing the single-spec policy fix.

Avoided `.codex/**`, `.claude/**`, `.omx/ultragoal/**`, generated files, workflow files, and `specs/repo-hygiene-automation.json`.

## Classification

Current checkout drift is mostly ignored runtime/build state, not tracked product source drift:

- `git status --short --untracked-files=all` was clean before this task's evidence/policy edits.
- `git status --ignored --short --untracked-files=all` showed ignored runtime/build outputs, including `.omx/logs/...` and `buck-out/...` from local OMX/Buck2 activity.
- `.omx/context/wave-d-g003-worker2/` was absent before this evidence note.

Existing policy already covered root ephemeral artifacts and build artifacts broadly, but the repo scan surface did not explicitly inventory hidden repo-runtime state roots such as `.omx/context` and `.omx/logs`. The smallest preventive policy change is the new `repo-runtime-state` scan surface in `specs/workspace-hygiene.json`.

## Outcome

Added explicit inventory-only policy coverage for repo-local runtime state roots while exempting `.omx/ultragoal/**` from cleanup-oriented interpretation. This prevents recurrence of unclassified root runtime-state drift without deleting or rewriting runtime state.
