# `cloud/` REORG-DRAIN — non-authoritative pointer

> **Non-authoritative.** This file is a pointer only. The sole live plan authority for
> repository state, sequencing, and outstanding work is
> [`specs/masterplan.json`](specs/masterplan.json#masterplan_v2); agents and consumers must
> resolve plan state there, not here. This document exists only to name the receipt and
> disposition records that carry the details.

## Records

- **Purge receipt (done):** `cloud/evidence/purge-cloud-os-20260811.md` — `cloud/cloud-os/**`
  deleted 2026-08-11 after dest-verify on `os/`; see the receipt for resolved-vs-remaining debt.
- **Kernel disposition (keep rule):** `cloud/cloud-kernel/manifest.json` +
  [`../evidence/reorg/rr-cloud-kernel-disposition-20260806.md`](../evidence/reorg/rr-cloud-kernel-disposition-20260806.md)
  — `cloud/cloud-kernel/**` holds unique
  kuberos bytes; `#1659` was Asterinas ABI absorb only, not a kuberos absorb. Do not delete until
  the zero-crate residual rehome tracked in the masterplan lands.
