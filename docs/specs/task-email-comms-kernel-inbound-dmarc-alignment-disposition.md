# Spec: Inbound DMARC Alignment + Disposition Evaluator

**Crate**: `shared-email-comms-kernel`  
**Slug**: `email-comms-kernel-inbound-dmarc-alignment-disposition`  
**RFC reference**: RFC 7489 (DMARC), RFC 7208 (SPF), RFC 6376 (DKIM)

## Objective

Add a pure, deterministic inbound DMARC alignment evaluator to the kernel. Given the
RFC 5322 `From` header domain, an SPF authentication result domain, a DKIM `d=` domain, and
the sender's published DMARC policy, produce a pass/fail verdict and a concrete disposition.
No I/O, no DNS, no network — kernel-layer only.

## Contracts

### Public Types

```
DmarcAlignmentMode   — Strict | Relaxed
DmarcAlignmentInput  — { from_domain, spf_result_domain, dkim_result_domain, alignment_mode, policy }
DmarcEvalVerdict     — { aligned: bool, spf_aligned: bool, dkim_aligned: bool, disposition: DmarcDisposition }
DmarcDisposition     — Accept | Quarantine | Reject
```

### Public Function

```
pub fn evaluate_inbound_dmarc(input: &DmarcAlignmentInput) -> DmarcEvalVerdict
```

### Alignment Rules (RFC 7489 §3.1)

- **Strict**: SPF/DKIM domain == `from_domain` (case-insensitive).
- **Relaxed**: organizational domain of SPF/DKIM domain == organizational domain of `from_domain`.
  Organizational domain = strip leftmost label from a multi-label domain (`sub.example.com` → `example.com`).
  Single-label domains use exact match.
- DMARC **passes** if `spf_aligned OR dkim_aligned`.

### Disposition Mapping (RFC 7489 §4.2)

| DMARC result | Policy      | Disposition  |
|-------------|-------------|--------------|
| Pass        | any         | Accept       |
| Fail        | None        | Accept       |
| Fail        | Quarantine  | Quarantine   |
| Fail        | Reject      | Reject       |

## Mod Layout (flat-clean-arch per ADR-0509)

All code lives in `src/lib.rs` under section:
```
// ---------- Inbound DMARC alignment + disposition ----------
```

No new files, no new modules, no new workspace members.

## Testing Strategy

Hermetic unit tests in the `#[cfg(test)]` block of `src/lib.rs`:

| Test name                                     | Validates                                       |
|----------------------------------------------|-------------------------------------------------|
| `dmarc_spf_only_aligned_pass`                | SPF aligned, DKIM not → verdict pass, Accept    |
| `dmarc_dkim_only_aligned_pass`               | DKIM aligned, SPF not → verdict pass, Accept    |
| `dmarc_both_fail_none_policy_accept`         | Both fail, p=none → Accept                      |
| `dmarc_both_fail_quarantine_policy`          | Both fail, p=quarantine → Quarantine            |
| `dmarc_both_fail_reject_policy`              | Both fail, p=reject → Reject                    |
| `dmarc_strict_subdomain_fails`               | Strict mode: sub.example.com vs example.com → fail |
| `dmarc_relaxed_subdomain_passes`             | Relaxed mode: sub.example.com vs example.com → pass |
| `dmarc_relaxed_cross_org_fails`              | Relaxed mode: other.com vs example.com → fail  |
| `dmarc_both_aligned_pass`                    | Both aligned → pass, Accept                     |
| `dmarc_case_insensitive_alignment`           | Domain comparison is case-insensitive           |
| `dmarc_empty_spf_domain_not_aligned`         | Empty SPF domain → not aligned, no panic       |
| `dmarc_none_policy_pass_is_accept`           | p=none + pass → Accept                          |

## Observability / SLO

Callers (IP-016 inbound receiver) attach OTel span attributes from the returned `DmarcEvalVerdict`:
- `dmarc.aligned` (bool)
- `dmarc.spf_aligned` (bool)
- `dmarc.dkim_aligned` (bool)
- `dmarc.disposition` (string: "accept" | "quarantine" | "reject")

Existing SLO: `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml` covers this.

## Crate Boundary

All changes are confined to `crates/shared-email-comms-kernel/src/lib.rs`.
No new workspace member, no root `Cargo.toml` edit, no other crate touched.
