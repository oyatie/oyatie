# `cloud/` REORG-DRAIN — non-authoritative pointer

> **Non-authoritative.** This file is a pointer only. The sole live plan authority for
> repository state, sequencing, and outstanding work is
> [`specs/masterplan.json`](../specs/masterplan.json#masterplan_v2); agents and consumers must
> resolve plan state there, not here. This document exists only to name the receipt and
> disposition records that carry the details.

## Records

- **Purge receipt (done):** `cloud/evidence/purge-cloud-os-20260811.md` — `cloud/cloud-os/**`
  deleted 2026-08-11 after dest-verify on `os/`; see the receipt for resolved-vs-remaining debt.
- **Kernel disposition (keep rule):** `cloud/cloud-kernel/manifest.json` +
  [`../evidence/reorg/rr-cloud-kernel-disposition-20260806.md`](../evidence/reorg/rr-cloud-kernel-disposition-20260806.md)
  — `cloud/cloud-kernel/**` holds unique
  kuberos bytes; `#1659` was Asterinas ABI absorb only, not a kuberos absorb. Do not delete until
  the S4/S5 zero-crate residual rehome lands — staged in the governed envelope
  [`../specs/integ-branch-envelopes.json`](../specs/integ-branch-envelopes.json) (`cloud/cloud-kernel/`
  `reorg_now` row, `judgment_status: done`) and in the disposition record above; the keep-rule is
  registered in the live plan as the `P-OWNED-STACK-KERNEL` rung-0 source anchor `cloud/cloud-kernel`
  in `specs/masterplan.json#masterplan_v2`. No S4/S5 rehome work item is minted in
  `masterplan_v2.work_items` yet; that registration remains genuinely-outstanding debt (requires a
  founder-ratified sequencing re-derivation before a dispatchable work item can land).
