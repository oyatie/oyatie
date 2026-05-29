# Plan: ulid-crockford-timestamp-validation-hardening

## Summary
Harden `Ulid::try_new` in `oya-shared-ulid-id-kernel` to enforce the full ULID spec:
1. Reject inputs whose first character exceeds '7' (timestamp overflow guard).
2. Normalize/accept lowercase Crockford symbols by uppercasing before storage.

## Steps

- [ ] Write spec doc at `docs/specs/task-ulid-crockford-timestamp-validation-hardening.md`
- [ ] Add red-phase tests (timestamp overflow, lowercase normalization, monotonic-prefix regression)
- [ ] Implement `is_crockford_base32_lc` helper and case-fold path in `try_new`
- [ ] Implement first-byte timestamp-overflow check in `try_new`
- [ ] Update existing `ulid_rejects_invalid_crockford_byte` test (lowercase now accepted)
- [ ] `cargo check -p oya-shared-ulid-id-kernel --all-targets` passes
- [ ] `cargo nextest run -p oya-shared-ulid-id-kernel` passes
- [ ] Commit and push, open PR

## ULID timestamp constraint rationale

A ULID is 128 bits: 48-bit timestamp + 80-bit random. Encoded as 26 Crockford-base32
characters (5 bits each = 130 bits; first character uses only 3 of its 5 bits because
the spec encodes the top character with the 2 MSBs unused).

The ULID spec (github.com/ulid/spec) states the most-significant character of the
26-character string represents the timestamp top bits. Because the 48-bit timestamp
max is 2^48-1 and Crockford-base32 encodes 5 bits per character, the first character
can only represent values 0–7 (3 bits). Characters '8'–'Z' in position 0 would imply
a timestamp beyond year 10889 and are explicitly rejected by the spec.
