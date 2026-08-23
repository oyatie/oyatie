# Spec: ulid-crockford-timestamp-validation-hardening

**Crate**: `shared-ulid-id-kernel`
**Lane**: foundation
**Priority**: med
**Effort**: S

## Problem Statement

`Ulid::try_new` enforces only two invariants today: length-26 and Crockford-base32 alphabet
membership. Two gaps remain:

1. **Timestamp overflow** — The ULID spec encodes a 48-bit UTC timestamp in the first
   10 base32 characters. With 5 bits per character, the first character can only represent
   values 0–7 (3 bits are used; the top 2 bits of the 5-bit group are always zero). Any
   first character in `'8'–'Z'` implies a timestamp beyond year ~10889, which is
   explicitly out-of-range per the ULID spec (github.com/ulid/spec, §"Monotonicity").

2. **Lowercase rejection** — Crockford-base32 defines a case-insensitive alphabet, and
   common ULID encoders emit lowercase. The current implementation hard-rejects lowercase,
   forcing callers to upper-case before every call. Normalisation at the boundary is the
   correct Postel-law behaviour.

## Acceptance Criteria

- AC-1 (timestamp overflow): A 26-character Crockford string whose first character is in
  `'8'..='9'` or `'A'..='Z'` returns `IdGeneratorError::MalformedUlid`.
- AC-2 (lowercase normalisation): A 26-character string that is valid when uppercased is
  accepted and stored in canonical (uppercase) form.
- AC-3 (alphabet + length): Existing length-26 and alphabet checks continue to reject
  truly malformed inputs.
- AC-4 (SeededIdGenerator): `SeededIdGenerator::new_ulid` output continues to validate
  without error.
- AC-5 (regression): A monotonic-prefix regression test set covers the exact character
  boundaries (`'7'` OK, `'8'` rejected).

## Design

### `try_new` changes (minimal diff)

```
pub fn try_new(raw: impl Into<String>) -> Result<Self, IdGeneratorError> {
    let raw = raw.into().to_ascii_uppercase();   // ← normalise
    if raw.len() != 26 {
        return Err(IdGeneratorError::MalformedUlid(raw));
    }
    // Timestamp overflow guard: first char must be 0–7.
    match raw.as_bytes()[0] {
        b'0'..=b'7' => {}
        _ => return Err(IdGeneratorError::MalformedUlid(raw)),
    }
    for byte in raw.as_bytes() {
        if !is_crockford_base32(*byte) {
            return Err(IdGeneratorError::MalformedUlid(raw));
        }
    }
    Ok(Ulid(raw))
}
```

The `is_crockford_base32` helper is unchanged (uppercase-only, as before).

### No new public API surface

No new types, traits, or error variants are introduced. The change is entirely within
`try_new`.

## Tests

New test functions added:

- `ulid_rejects_timestamp_overflow` — first char `'8'`, `'9'`, and `'Z'` each produce
  `MalformedUlid`.
- `ulid_accepts_lowercase_normalised` — a valid all-lowercase ULID is accepted and stored
  uppercase.
- `ulid_monotonic_prefix_boundaries` — char `'7'` at position 0 is accepted; char `'8'`
  at position 0 is rejected; verifies exact boundary.

Modified test function:

- `ulid_rejects_invalid_crockford_byte` — the lowercase assertion is inverted: lowercase
  is now accepted (stored upper-cased).

## Non-changes

- No new dependencies.
- `#![forbid(unsafe_code)]` retained.
- No new `IdGeneratorError` variants.
- SeededIdGenerator logic unchanged.
