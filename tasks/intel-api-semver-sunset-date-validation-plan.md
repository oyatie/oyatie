# Plan: intel-api-semver-sunset-date-validation

## Objective
Tighten `validate_api_semver` sunset handling in `oya-intelligence-api-semver-domain`.

## Acceptance Criteria
- New `ApiSemverError::InvalidSunset` variant with fields: `artifact_path`, `metadata_path`, `sunset`
- `"none"` accepted as valid sunset
- Well-formed RFC 3339 calendar dates (YYYY-MM-DD, valid month 01-12, valid day 01-28/29/30/31 per month) accepted
- Malformed values (`"soon"`, `"2026-13-01"`, `"2026/01/01"`, empty-ish) rejected with `InvalidSunset`
- 3+ new test cases
- Existing tests remain green
- Pure string parsing, no I/O

## Steps
1. [x] Add `InvalidSunset` variant to `ApiSemverError`
2. [x] Add `is_valid_sunset` pure parsing function
3. [x] Call sunset validation after `validate_required_field` for `sunset`
4. [x] Add 3+ new test cases
5. [x] `cargo check -p oya-intelligence-api-semver-domain --all-targets`
6. [x] `cargo nextest run -p oya-intelligence-api-semver-domain`
