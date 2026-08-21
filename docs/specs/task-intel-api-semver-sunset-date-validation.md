# Spec: intel-api-semver-sunset-date-validation

**Crate:** `intelligence-api-semver-domain`
**Lane:** intelligence
**Priority:** med
**Effort:** S

## Summary
Tighten the `validate_api_semver` sunset field validation to accept only the literal `"none"` or a well-formed RFC 3339 calendar date (YYYY-MM-DD with valid month/day ranges). Any other shape must be rejected with a new `ApiSemverError::InvalidSunset` variant.

## Error Variant
```rust
InvalidSunset {
    artifact_path: String,
    metadata_path: String,
    sunset: String,
}
```

## Validation Rules
- `"none"` (case-sensitive, exact match) -> valid
- `YYYY-MM-DD` where:
  - YYYY is any 4-digit year
  - MM is 01-12
  - DD is 01 to max days in MM (accounting for leap years: Feb has 29 days max)
- Anything else -> `Err(ApiSemverError::InvalidSunset { ... })`

## Acceptance Tests (minimum 3 new)
1. `"none"` -> accepted
2. `"2026-01-15"` -> accepted
3. `"soon"` -> rejected with `InvalidSunset`
4. `"2026-13-01"` -> rejected (invalid month)
5. `"2026/01/01"` -> rejected (wrong separator)
6. `"2026-00-01"` -> rejected (month out of range)
7. `"2026-01-32"` -> rejected (day out of range)

## Implementation Notes
- Pure string parsing, no external dependencies, no I/O
- Validation added inline after the existing `validate_required_field` call for `sunset`
- Helper function `is_valid_sunset(s: &str) -> bool`
