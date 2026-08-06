# R3 post-merge product-completion packet — TEMPLATE

**Lane:** `R3-postmerge-trunk`  
**Authority:** Not merge authority. Records product-complete evidence **after** squash to `origin/dev`.  
**Canon:** AGENTS.md `required_sequence` → post-merge product-completion packet; D19/D20 in `templates/checklists/done-definition-checklist.md`.

---

## Hard rule (read first)

| Claim | Allowed? |
|-------|----------|
| PR-head `oya-ci-required` green | **Not enough** for this packet |
| Squash merge to `origin/dev` only | **Not enough** without post-merge green on promoted SHA |
| Promoted commit on `dev` + **exact-head** post-merge `oya-ci-required` success | **Required** before any field below is treated as evidence |

**Fill only after squash to `origin/dev`.** Pre-merge admission, draft CI, and PR-head green are pre-merge surfaces (R2 / review). They must not be copied into promoted fields as if they were trunk evidence.

**Automation (not merge authority; never claims ultragoal complete):**

```sh
.grok/bin/mm-packet --pr N --program <program> [--dry-run]
.grok/bin/mm-drive packet --pr N --program <program>
# or: --sha <promoted_sha>
```

Fail-closed unless PR is MERGED and trunk `oya-ci-required` is SUCCESS on the exact promoted SHA. See `harness/DRIVE.md` § D5.

**Example filled packet (docs-only #1559):**  
[`../cas-fabric/evidence/1559-post-merge-completion-packet.json`](../cas-fabric/evidence/1559-post-merge-completion-packet.json)

**Related draft (pre-merge scaffolding only — not complete):**  
[`../k8s-port/evidence/G001-post-merge-packet-DRAFT.md`](../k8s-port/evidence/G001-post-merge-packet-DRAFT.md)

---

## Metadata

| Field | Value |
|-------|--------|
| `packet_type` | `post_merge_product_completion` |
| `status` | `DRAFT` → flip to `COMPLETE` only when fields 1–8 have real post-merge evidence |
| `pr` | `_PR number_` |
| `title` | `_PR title_` |
| `branch` | `dev` (promoted trunk only) |
| `program_id` / ultragoal / beads | `_optional tracking ids_` |
| `recorded_at` | `_ISO-8601 after post-merge green_` |
| `completion_packet_closed` | `false` until fields 1–8 verified |

---

## Template fields (AGENTS post-merge product gate)

### 1. Promoted commit

| Field | Value |
|-------|--------|
| `promoted_commit` | `_full SHA of squash commit on origin/dev_` |
| `promoted_at` | `_merge time_` |
| `merge_method` | `squash` (required by governance sequence) |
| `pr_state` | `_must be MERGED_` |
| `origin_dev_tip_equals_promoted` | `_true only after requery_` |

Verify: `git fetch origin dev && git rev-parse origin/dev` equals `promoted_commit`.

### 2. Post-merge `oya-ci-required` (promoted tip)

| Field | Value |
|-------|--------|
| `status` | `_must be success_` |
| `run_url` | `_https://github.com/.../actions/runs/<id> on promoted SHA_` |
| `head_sha` | `_must equal promoted_commit_` |
| `completed_at` | `_run completion time_` |

**Reject if:** run is on PR head, merge_group only without matching trunk tip, `queued`/`in_progress`, or `head_sha ≠ promoted_commit`.

### 3. Rollout verification

| Field | Value |
|-------|--------|
| `class` | `_runtime | docs-only | governance | build-time | other_` |
| `verification` | `_what was checked on trunk (deploy, tip presence, gate selectability, …)_` |
| `result` | `_pass/fail + brief note_` |

### 4. Rollback note

| Field | Value |
|-------|--------|
| `rollback` | `_how to undo promoted SHA; migrations/data/infra notes_` |
| `blast_radius` | `_what reverts with the commit_` |

### 5. Observability check

| Field | Value |
|-------|--------|
| `check` | `_golden signals / SLOs / dashboards, or explicit N/A + why_` |
| `evidence` | `_link or N/A rationale_` |

### 6. Browser / user-story evidence

| Field | Value |
|-------|--------|
| `evidence` | `_UX path / story verification, or explicit N/A + why_` |

### 7. Release-governance / release-note impact

| Field | Value |
|-------|--------|
| `impact` | `_Release Please / release note / changelog impact, or N/A when no live product release surface applies_` |

### 8. Agent-observation harvest

| Field | Value |
|-------|--------|
| `sources_reviewed` | `_chat | review notes | scratch | PR comments | Kanban | …_` |
| `cards_created_or_linked` | `_ids or none_` |
| `duplicates_documented` | `_ids or none_` |
| `rationale` | `_why create/link/no-action_` |

New/linked cards must include: source context, classification, affected artifact, acceptance criteria, verification path, suggested owner/profile, dependencies/conflict notes (D20).

---

## JSON shape (optional machine copy)

Prefer storing a filled packet next to the program as `evidence/<pr>-post-merge-completion-packet.json` (see #1559 example). Field names align with that example:

```json
{
  "packet_type": "post_merge_product_completion",
  "pr": 0,
  "title": "",
  "promoted_commit": "",
  "branch": "dev",
  "oya_ci_required": {
    "status": "success",
    "run_url": "",
    "head_sha": "",
    "completed_at": ""
  },
  "rollout_verification": "",
  "rollback_note": "",
  "observability_check": "",
  "browser_user_story_evidence": "",
  "release_governance_release_note_impact": "",
  "agent_observation_harvest": {
    "cards_created_or_linked": "",
    "rationale": ""
  },
  "completion_packet_closed": false,
  "recorded_at": "",
  "template_ref": "programs/hyperscaler-delivery-lanes/R3-postmerge-packet-template.md",
  "authority": "Not merge authority; post-merge product gate evidence only."
}
```

---

## Closeout checklist (after fields 1–8 are real)

- [ ] Squash merged to `origin/dev`; PR state MERGED  
- [ ] `promoted_commit` == `origin/dev` tip (re-queried)  
- [ ] Post-merge `oya-ci-required` **success** on **exact** promoted SHA (run URL recorded)  
- [ ] Rollout, rollback, observability, user-story, release-note fields filled (N/A only with rationale)  
- [ ] Agent-observation harvest recorded (cards or duplicate/no-action)  
- [ ] Packet path linked from program `PROGRAM.json` / SESSION-BACKLOG / Beads as applicable  
- [ ] Ultragoal / beads closed **only** if this packet is COMPLETE — never on PR-head green alone  

---

## Hard stops

- No baseline / cache-writer identity from PR head (R3: trusted `dev` push only)  
- No secrets in packet or git  
- No false ultragoal complete  
- Do not treat this template file as a filled packet  
- Do not couple R3 automation PRs to #1558 / #1561 product diffs  

---

## Lane board

- Board: [`LANES.md`](LANES.md) → **R3-postmerge-trunk**  
- Session ledger: [`../SESSION-BACKLOG.md`](../SESSION-BACKLOG.md) → `S-R3-POSTMERGE`  
- Example COMPLETE: [`../cas-fabric/evidence/1559-post-merge-completion-packet.json`](../cas-fabric/evidence/1559-post-merge-completion-packet.json)
