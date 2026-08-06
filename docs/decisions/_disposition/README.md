# ADR disposition workspace (2026-08-06)

## What this is

Full-corpus audit of every ADR on `origin/dev` at tip `c7f60a9db` (~448 ADRs), plus the **only** bulk mutation that is safe without per-ADR substance review:

| Change | Safe bulk? | This PR |
|--------|------------|---------|
| Normalize `status:` case (`accepted`→`Accepted`, etc.) | Yes | **Done** (67 files) |
| Fill missing `status` | No — needs read of body | Queued |
| Accept Proposed | No — needs evidence + door | Queued by admission class |
| Supersede Accepted | No — needs successor ADR | Queued |
| Fix Accepted→Proposed plan-lag edges | Per-edge Accept or frontmatter amend | Queued |
| Fill `superseded_by` when empty | Per-ADR research | Queued (5) |

## Policy (binding)

1. **Proposed is not implement authority.** Never mass-Accept.
2. **Superseded is not implement authority.** Follow successor.
3. **Accepted with `amended_by`** must be read with later peers (e.g. 0562 + 0615 + 0635).
4. **Plan-lag** (Accepted `depends_on`/`amends` still-Proposed) is a separate defect from “stale execute.”
5. Mechanical case normalize does **not** change meaning of status.

## Artifacts

| File | Content |
|------|---------|
| `2026-08-06-mechanical-status-case.json` | List of case-normalized files |
| `2026-08-06-full-disposition-summary.json` | Histograms + plan-lag + missing status + superseded gaps |
| `ADR-FULL-DISPOSITION-AUDIT.md` | Human queues |

## Next waves (not this PR)

### Wave D1 — Missing status (26)

Triage each body; set `Accepted` / `Proposed` / `Superseded` with evidence note in PR body. Prefer dual-critic if any become Accepted with planning impact.

### Wave D2 — Plan-lag edges (10 Accepted parents)

Parents include 0565, 0614, 0616, 0619, 0630, 0635–0639. For each cited Proposed: **Accept** (if lived law), **waive as design-input-only** in parent frontmatter note, or **repoint** depends_on.

### Wave D3 — Superseded without successor (5)

0057, 0097, 0101, 0102, 0138 — research successor (often 0335/0363 foundry retirement cluster).

### Wave D4 — Proposed admission queue (500–639 + activation)

0612 RE, 0560 CAS, 0556 warmth, security gates 0605–0608, etc. Accept only when next implement slice requires and evidence exists.

### Wave D5 — Broad Proposed review (<500)

Park / supersede / accept in batches of ≤8 with ownership-coherence audit (historical wave batch rule).

## Non-goals

- Accepting all Proposed ADRs in one PR
- Changing decision bodies while only normalizing status case
- Hand-editing `*.generated.json`
