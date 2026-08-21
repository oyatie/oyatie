//! Audit-chain query domain: pure validation, an opaque pagination cursor
//! codec, page assembly, and auditor-engagement authorization gating.
//!
//! This crate has exactly one dependency, `audit-query-api` (the DTO port),
//! and performs no I/O: no clock, no datastore, no crypto library. Everything
//! here is deterministic, std-only, and independently unit tested.
//!
//! ## Scope and the policy fragments it encodes
//!
//! The rules below are a pure-domain subset of four Cedar policy fragments
//! under `audit/policy/`. This crate enforces the parts that are checkable
//! from the DTOs alone; the parts that need a live PDP, a SPIFFE identity, or
//! a `scoped_tenants` / `scoped_packs` list from an auditor's JIT token are
//! out of reach for a dependency-free domain crate and are called out below
//! by name so nobody mistakes this crate's checks for the whole picture.
//!
//! - **`tenant-scope.cedar`** ("FORBID — Cross-tenant query refusal
//!   (defence-in-depth)"): a query must never be satisfiable against a
//!   different tenant than the one it was scoped to. This crate enforces the
//!   pagination-cursor half of that invariant: [`decode_cursor`] refuses a
//!   cursor whose embedded tenant does not match the tenant presented on the
//!   follow-up query ([`QueryDomainError::InvalidCursor`]), and, one scope
//!   narrower, refuses a cursor whose embedded [`CursorScope`] fingerprint
//!   (pack/period/event_type/principal/entity/limit) does not match the
//!   query it is presented against — see section (b). Both tenant legs of an
//!   auditor engagement are validated with the same identifier rule before
//!   [`authorize_auditor_engagement`] compares them
//!   ([`QueryDomainError::EmptyTenantId`]), and it refuses an engagement
//!   bound to a different tenant than the query
//!   ([`QueryDomainError::EngagementTenantMismatch`]). The live per-row ACL
//!   check (`resource.tenant_id == principal.tenant_id`) happens at the PDP,
//!   not here.
//! - **`auditor-scope.cedar`** ("PERMIT 1 ... `principal.valid_from <=
//!   context.now && context.now <= principal.valid_to`"): an auditor's read
//!   is only valid inside their engagement window.
//!   [`authorize_auditor_engagement`] enforces the upper bound
//!   (`expires_at`) against a comparison instant. `AuditorEngagement` (the
//!   port DTO) carries no `valid_from`, no `scoped_tenants`, no
//!   `scoped_packs`, and no `audit_framework`; those legs of PERMIT 1 are
//!   enforced by the PDP adapter that issues the JIT token, not by this
//!   crate.
//! - **`data-residency-enforcement.cedar`** / **`region-local-pi-read.cedar`**
//!   (both key every rule off `resource.pack` / `principal.bound_pack` /
//!   `principal.scoped_packs`): residency is pack-scoped, so an
//!   [`AuditQuery::pack`] that is present but blank, whitespace-only, or made
//!   entirely of invisible characters would let a downstream residency
//!   router silently fail open. [`ValidatedAuditQuery`] rejects such a value
//!   as [`QueryDomainError::InvalidPack`]. The actual `bound_pack ==
//!   resource.pack` / `pack in scoped_packs` match is enforced by the PDP,
//!   which is the only layer that knows the caller's bound pack.
//!
//! ## (a) Query validation
//!
//! [`validate_query`] rejects a blank `tenant_id`
//! ([`QueryDomainError::EmptyTenantId`]), a present-but-blank `pack`
//! ([`QueryDomainError::InvalidPack`]), and a malformed `period`
//! ([`QueryDomainError::InvalidPeriodWindow`]) or one whose span exceeds
//! [`MAX_QUERY_WINDOW_DAYS`] ([`QueryDomainError::WindowTooLarge`]). Every
//! identifier check uses [`validate_identifier`]'s CONTAINS rule (see below)
//! rather than a bare `.trim().is_empty()`.
//!
//! `period`, when present, accepts three shapes, all normalized to the same
//! [`PeriodWindow`]:
//! - `"<start>/<end>"`, an explicit range with each side an ISO `YYYY-MM-DD`
//!   calendar date and `end >= start` — the only shape that can express a
//!   window wider than one calendar unit, so it is what
//!   [`MAX_QUERY_WINDOW_DAYS`] bounds;
//! - a bare `YYYY-MM-DD` date, treated as a one-day window (`start == end`,
//!   `span_days() == 1`);
//! - a bare `YYYY-MM` month, treated as that whole calendar month
//!   (`span_days()` = the number of days in that month).
//!
//! The port DTO (`audit-query-api::AuditQuery::period`) mandates no format —
//! its own doc comment says only "Full DTO schema in IP-012", and that IP
//! does not exist: `audit/IPs/` holds only `IP-ADR-0339` and
//! `IP-WAVE-15-ZD`. This crate's own format decision therefore rests on this
//! doc paragraph, not on that dangling citation. Two sibling artifacts in
//! this same capability
//! already mint bare period identifiers of exactly the bare-date and
//! bare-month shapes above: `sealing-domain` uses `period_id: "2026-08-15"`
//! (`tests/seal_lifecycle.rs`) and `period_id: "2026-08"` (`src/status.rs`),
//! and this crate's own sibling field, `QueryRow::period_id`, carries the
//! identical convention. A caller filtering to the period of a row it just
//! paged over — the single most obvious use of this field — supplies
//! exactly one of those bare shapes, so this crate accepts them directly
//! rather than forcing every caller to wrap a single period id into a
//! redundant `"id/id"` range. The range shape remains the only way to ask
//! for a window wider than a single day or month; this crate mints that
//! range shape itself and validates it strictly on the way back in, same as
//! before (per L9). `event_type`, `principal`, and `entity` are free-text
//! search filters, not identity-key legs, and are intentionally left
//! unvalidated here — validating an arbitrary caller-chosen search string
//! would be arbitrary, not a rule this crate owns.
//!
//! `limit` is checked against an explicit page-size ceiling
//! ([`MAX_PAGE_SIZE`]). **Decision: an over-cap `limit` is REJECTED, not
//! clamped** ([`QueryDomainError::LimitExceedsMaximum`]) — silently shrinking
//! a caller's requested page size is a surprising behaviour change for an
//! auditor mid-export; a `Some(0)` limit is also rejected
//! ([`QueryDomainError::ZeroLimit`]) since a zero-row page can never make
//! progress. `None` falls back to [`DEFAULT_PAGE_SIZE`].
//!
//! [`MAX_PAGE_SIZE`] (1000) and [`DEFAULT_PAGE_SIZE`] (100) are pinned to
//! this capability's own published query contract:
//! `audit/contracts/openapi/audit-chain.yaml` line ~207 declares
//! `QueryRequest.limit: {type: integer, minimum: 1, maximum: 1000, default:
//! 100}` on the audit-chain `/query` endpoint. That `QueryRequest` schema is
//! a different, unwired shape from this crate's `AuditQuery` — it has
//! `time_range`/`page_token`/filter-array fields with no Rust binding
//! anywhere in this repo, so it is not *proven* to be the literal same wire
//! field as `AuditQuery::limit` — but a pure domain crate that hard-rejects a
//! contract-legal `limit` (or silently halves the contract's advertised
//! default) is a defect, not a "second number for the same concept" the
//! contract would tolerate. `tests/query_validation.rs` asserts the literal
//! values `1000` / `100` directly, so the constants and this citation cannot
//! drift apart silently (per L5).
//!
//! Every identifier field is stored EXACTLY as the caller supplied it (never
//! trimmed-then-stored): [`validate_identifier`] requires `value ==
//! value.trim()`, so a padded value is a hard reject rather than being
//! silently normalized. Trimming on the way in and storing the untrimmed
//! original would let `" tenant"`, `"tenant"`, and `"tenant\n"` collide or
//! diverge unpredictably downstream — this crate picks "reject
//! non-normalized input" and documents it here.
//!
//! ## (b) The cursor codec
//!
//! [`encode_cursor`] / [`decode_cursor`] implement a length-prefixed,
//! hex-encoded cursor with NO external crate — the same
//! length-prefix-then-bytes domain-separation scheme `chain-domain` uses for
//! its hash inputs, applied here to pagination state instead of hash
//! preimages. A cursor encodes exactly four fields (`version`, `tenant_id`,
//! `offset`, `scope_fingerprint`) followed by an 8-byte FNV-1a checksum over
//! the encoded payload.
//!
//! **`scope_fingerprint` binds the cursor to the query it was minted under,**
//! not just to the tenant. An `offset` is only meaningful relative to the
//! exact result set that produced it: [`CursorScope`] captures every filter
//! leg that can change which rows a query matches or how many rows land on a
//! page (`pack`, `period`, `event_type`, `principal`, `entity`, and the
//! resolved `limit`), and [`decode_cursor`] refuses a cursor whose scope
//! fingerprint does not match the scope of the query it is presented against
//! — with the same [`QueryDomainError::InvalidCursor`] the cross-tenant case
//! already used. Without this, a cursor minted while paging `pack: "eu"`
//! would be silently honoured as a resume position on a `pack: "us"` query
//! with an unrelated result set, applying the old offset to new rows and
//! dropping whichever rows sat before it (this crate's residency-hazard
//! reasoning above for a blank `pack` extends identically to a *changed*
//! `pack` left unbound from pagination state).
//!
//! **The checksum and the fingerprint are NOT security mechanisms.** FNV-1a
//! is a fast non-cryptographic hash; nothing in this crate can stop a
//! determined holder of tenant A's own cursor from re-deriving tenant A's
//! other cursors or scopes, and that is fine, because a cursor is not a
//! bearer capability — every follow-up query is independently re-authorized
//! by the PDP against the caller's own credentials (`tenant-scope.cedar`).
//! The checksum exists only to fail closed on accidental corruption or
//! truncation before the value reaches decoding logic that would otherwise
//! misparse it; the scope fingerprint exists only to fail closed on a cursor
//! replayed against a *different* query than the one that minted it.
//!
//! [`QueryDomainError::InvalidCursor`] is reachable for:
//! - a malformed cursor (not valid hex, or valid hex that decodes to garbage
//!   that fails the length-prefix framing or the checksum);
//! - a truncated cursor (a valid cursor with trailing hex characters cut
//!   off);
//! - a non-canonical offset field: [`decode_cursor`] only accepts the exact
//!   digit string [`encode_cursor`] mints (no `+` sign, no leading zero other
//!   than a bare `"0"`) — per L9, this crate is the cursor format's only
//!   minter, so a syntactically-different-but-numerically-equal offset field
//!   (`"+7"`, `"0007"`) is not a cursor this crate produced and is refused
//!   rather than normalized;
//! - **the security-relevant case**: a cursor minted for tenant A, presented
//!   on a query for tenant B. [`decode_cursor`] checks the embedded tenant
//!   against the tenant passed in by the caller and fails closed on any
//!   mismatch — it never falls back to honouring the embedded tenant instead;
//! - **the scope-relevant case**: a cursor minted under one [`CursorScope`],
//!   presented against a query with a different scope (changed `pack`,
//!   `period`, `event_type`, `principal`, `entity`, or `limit`) — see above.
//!
//! Per L4 (fail closed, never saturate): the length prefix inside a cursor is
//! read as a `u64` and converted to `usize` with `usize::try_from(..)`, which
//! is propagated as [`QueryDomainError::InvalidCursor`] on failure rather
//! than saturated to `usize::MAX` (which would widen acceptance instead of
//! refusing).
//!
//! ## (c) Pagination
//!
//! [`paginate`] slices a caller-supplied row set at `[offset, offset +
//! effective_limit)` and returns `next_cursor: None` exactly when that slice
//! reaches the end of the row set. An `offset` decoded from a syntactically
//! valid cursor that nonetheless points **at or past** the end of the
//! current row set is also treated as [`QueryDomainError::InvalidCursor`] (a
//! stale or forged position — a row set can shrink between two pages of the
//! same walk, e.g. a retention cascade redacting rows), rather than silently
//! returning an empty page — except that `offset == 0` against an empty row
//! set is a legitimate first (and only) page, not a stale cursor, since no
//! cursor was needed to reach it.
//!
//! ## (d) Auditor-engagement validity
//!
//! [`authorize_auditor_engagement`] takes the comparison instant
//! (`now_epoch_seconds: i64`, Unix epoch seconds) as a CALLER-SUPPLIED
//! parameter rather than reading a clock (per L8): this is a pure domain
//! crate with no clock port, and no dependency may be added to reach one, so
//! the composition-root / adapter layer — which owns a real clock — supplies
//! the instant. This crate's job is solely to compare it against
//! `expires_at` and the query's `tenant_id`; it cannot and does not assert
//! that the caller supplied a trustworthy instant.
//!
//! Per L7 (validate every leg of an identity tuple): both tenant legs —
//! `engagement.tenant_id` and the `query_tenant_id` parameter — are run
//! through [`validate_identifier`] before the equality comparison between
//! them. Comparing two unvalidated strings for equality is not the same as
//! validating either one: an empty, whitespace-only, or zero-width-only
//! tenant on *both* sides compares equal and would otherwise authorize the
//! engagement against a blank tenant scope.
//!
//! `expires_at` is a plain `String` on the port DTO with no format mandated
//! by the port; this crate is the only consumer that parses it, so
//! [`authorize_auditor_engagement`] validates it strictly as `YYYY-MM-DDTHH:MM:SSZ`
//! (UTC, no fractional seconds, no non-`Z` offsets) and fails closed with
//! [`QueryDomainError::InvalidExpiresAt`] on anything else, per L9.
//!
//! **Boundary note:** `auditor-scope.cedar`'s PERMIT 1, cited above, uses an
//! *inclusive* upper bound (`context.now <= principal.valid_to`). This crate
//! is deliberately one second stricter: `now_epoch_seconds >= expires_at`
//! (an *exclusive* upper bound) is treated as expired, so the boundary
//! instant itself — which the PDP would permit — is denied here. That is an
//! intentional fail-closed divergence, not an attempt to mirror the PDP
//! exactly, and is called out here so a reader reconciling this function
//! against the cedar file it cites is not misled by the apparent match.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

pub use audit_query_api::{AuditQuery, AuditorEngagement, QueryResult, QueryRow, ResultSealState};

/// Maximum inclusive span, in calendar days, that a `period` window may
/// cover. Chosen as 366 days (one calendar year, inclusive of a leap day) —
/// wide enough for a routine annual audit pull in one request, narrow enough
/// to bound the size of a single query. Callers who need a longer historical
/// range page across multiple `period` windows.
pub const MAX_QUERY_WINDOW_DAYS: u32 = 366;

/// Maximum accepted `AuditQuery::limit`. An over-cap `limit` is a hard
/// [`QueryDomainError::LimitExceedsMaximum`] error (see the module docs for
/// why this crate rejects rather than clamps). Pinned to
/// `audit/contracts/openapi/audit-chain.yaml` line ~207
/// (`QueryRequest.limit.maximum: 1000`) — see the module docs, section (a),
/// for why this crate does not invent a second bound for the same concept.
pub const MAX_PAGE_SIZE: u32 = 1000;

/// Effective page size used when `AuditQuery::limit` is `None`. Pinned to
/// `audit/contracts/openapi/audit-chain.yaml` line ~207
/// (`QueryRequest.limit.default: 100`).
pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Cursor wire-format version. Bumped to `v2` when the payload gained the
/// `scope_fingerprint` field (see [`CursorScope`], module docs section (b));
/// a `v1`-tagged cursor from before that change fails closed as
/// [`QueryDomainError::InvalidCursor`] (its shorter payload runs out of
/// bytes when a fourth field is decoded), it is never misparsed.
const CURSOR_VERSION: &str = "v2";

/// Pure validation / pagination / auditor-engagement error for an audit
/// query. See the module docs for exactly which policy fragment or lesson
/// each variant enforces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryDomainError {
    /// `tenant_id` is blank, whitespace-padded, or made only of invisible
    /// characters (see [`validate_identifier`]).
    EmptyTenantId,
    /// `period`, once parsed, spans more than [`MAX_QUERY_WINDOW_DAYS`].
    WindowTooLarge,
    /// A `cursor` is malformed, truncated, was minted for a different tenant
    /// than the one presented, or was minted under a different
    /// [`CursorScope`] (pack/period/event_type/principal/entity/limit) than
    /// the query it is presented against — see the module docs, section (b).
    InvalidCursor,
    /// `pack` is present but blank, padded, or content-free.
    InvalidPack,
    /// `period` is present but is not a bare `YYYY-MM-DD` date, a bare
    /// `YYYY-MM` month, or an explicit `"<start>/<end>"` range of two valid
    /// `YYYY-MM-DD` dates with `end >= start`.
    InvalidPeriodWindow { period: String },
    /// `limit` was explicitly `Some(0)`.
    ZeroLimit,
    /// `limit` exceeds [`MAX_PAGE_SIZE`].
    LimitExceedsMaximum { limit: u32, max: u32 },
    /// An `AuditorEngagement::engagement_id` is blank, padded, or
    /// content-free.
    EmptyEngagementId,
    /// An `AuditorEngagement::expires_at` is not a valid
    /// `YYYY-MM-DDTHH:MM:SSZ` UTC timestamp.
    InvalidExpiresAt { expires_at: String },
    /// The engagement's `tenant_id` does not match the query's `tenant_id`.
    EngagementTenantMismatch {
        engagement_tenant_id: String,
        query_tenant_id: String,
    },
    /// The comparison instant is at or past the engagement's `expires_at`.
    EngagementExpired { expires_at: String },
}

/// A parsed, validated, inclusive calendar-day span (`period`'s decoded
/// form). Fields are private; the only way to obtain one is through
/// [`validate_query`] / [`parse_period_window`], which enforce the format
/// and the maximum-span rule before a value can exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeriodWindow {
    start_day: i64,
    end_day: i64,
    span_days: u32,
}

impl PeriodWindow {
    /// Inclusive span in calendar days (`end - start + 1`).
    pub fn span_days(&self) -> u32 {
        self.span_days
    }

    /// Start date as a day number relative to the 1970-01-01 civil epoch
    /// (negative for dates before it).
    pub fn start_day_number(&self) -> i64 {
        self.start_day
    }

    /// End date (inclusive) as a day number relative to the 1970-01-01
    /// civil epoch.
    pub fn end_day_number(&self) -> i64 {
        self.end_day
    }
}

/// A validated [`AuditQuery`], with `pack` well-formed-if-present, `period`
/// parsed into a [`PeriodWindow`] and bounds-checked, `limit` resolved to a
/// concrete `effective_limit`, and `cursor` (if any) decoded into a starting
/// `offset` already checked against the presented `tenant_id`.
///
/// Fields are private with accessors (this type's whole purpose is that
/// holding one means validation already ran; a public-field struct literal
/// would let a caller construct an unvalidated instance and defeat that
/// guarantee).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAuditQuery {
    tenant_id: String,
    pack: Option<String>,
    event_type: Option<String>,
    principal: Option<String>,
    entity: Option<String>,
    period_window: Option<PeriodWindow>,
    offset: usize,
    effective_limit: u32,
    scope_fingerprint: u64,
}

impl ValidatedAuditQuery {
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn pack(&self) -> Option<&str> {
        self.pack.as_deref()
    }

    pub fn event_type(&self) -> Option<&str> {
        self.event_type.as_deref()
    }

    pub fn principal(&self) -> Option<&str> {
        self.principal.as_deref()
    }

    pub fn entity(&self) -> Option<&str> {
        self.entity.as_deref()
    }

    pub fn period_window(&self) -> Option<&PeriodWindow> {
        self.period_window.as_ref()
    }

    /// Row offset to resume from (`0` when the query had no cursor).
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Resolved page size: `AuditQuery::limit` if present and in range, else
    /// [`DEFAULT_PAGE_SIZE`].
    pub fn effective_limit(&self) -> u32 {
        self.effective_limit
    }
}

/// Validate an [`AuditQuery`] end to end: `tenant_id`, `pack`, `period`,
/// `limit`, and (if present) `cursor` — decoded here against `tenant_id` so
/// a cross-tenant cursor is refused as part of validation rather than later.
/// See the module docs, section (a), for the exact rule for each field.
pub fn validate_query(query: &AuditQuery) -> Result<ValidatedAuditQuery, QueryDomainError> {
    if !validate_identifier(&query.tenant_id) {
        return Err(QueryDomainError::EmptyTenantId);
    }
    if let Some(pack) = &query.pack
        && !validate_identifier(pack)
    {
        return Err(QueryDomainError::InvalidPack);
    }

    let period_window = match &query.period {
        Some(period) => Some(parse_period_window(period)?),
        None => None,
    };
    if let Some(window) = &period_window
        && window.span_days > MAX_QUERY_WINDOW_DAYS
    {
        return Err(QueryDomainError::WindowTooLarge);
    }

    let effective_limit = match query.limit {
        None => DEFAULT_PAGE_SIZE,
        Some(0) => return Err(QueryDomainError::ZeroLimit),
        Some(limit) if limit > MAX_PAGE_SIZE => {
            return Err(QueryDomainError::LimitExceedsMaximum {
                limit,
                max: MAX_PAGE_SIZE,
            });
        }
        Some(limit) => limit,
    };

    // Bind the cursor to the exact query scope it must resume, not just the
    // tenant — see the module docs, section (b), and `CursorScope`.
    let scope = CursorScope {
        pack: query.pack.as_deref(),
        period: query.period.as_deref(),
        event_type: query.event_type.as_deref(),
        principal: query.principal.as_deref(),
        entity: query.entity.as_deref(),
        limit: effective_limit,
    };
    let scope_fingerprint = compute_scope_fingerprint(&scope);

    let offset = match &query.cursor {
        Some(cursor) => decode_cursor_raw(cursor, &query.tenant_id, scope_fingerprint)?,
        None => 0,
    };

    Ok(ValidatedAuditQuery {
        tenant_id: query.tenant_id.clone(),
        pack: query.pack.clone(),
        event_type: query.event_type.clone(),
        principal: query.principal.clone(),
        entity: query.entity.clone(),
        period_window,
        offset,
        effective_limit,
        scope_fingerprint,
    })
}

/// Build a [`QueryResult`] page from `rows`, starting at `validated`'s
/// resolved offset and taking `validated`'s effective limit. `next_cursor`
/// is `Some` exactly when more rows remain past this page, and `None` on the
/// final page.
///
/// An `offset` at or past the end of `rows` (a stale or forged cursor
/// position referencing a row set that has since shrunk) is rejected as
/// [`QueryDomainError::InvalidCursor`] rather than silently treated as "no
/// more rows" — a caller-visible empty page and a caller-visible error are
/// very different signals, and only the latter is honest about a cursor that
/// no longer corresponds to a real position. The one exception is `offset ==
/// 0` against an already-empty `rows`: that is a legitimate first page (no
/// cursor was needed to reach `offset == 0`), not a stale one.
pub fn paginate(
    rows: &[QueryRow],
    validated: &ValidatedAuditQuery,
) -> Result<QueryResult, QueryDomainError> {
    let offset = validated.offset;
    // The exception named in the doc above is `offset == 0` against an
    // already-empty `rows` — NOT "any offset against empty rows". The
    // previous guard here was `offset >= rows.len() && !rows.is_empty()`,
    // whose `!rows.is_empty()` carve-out let EVERY offset through once
    // `rows` was empty (0 >= 0 is true, but the old guard never evaluated
    // it because the second half short-circuited), so a stale non-zero
    // offset replayed against a row set that had shrunk to nothing fell
    // through to the slice index below and panicked instead of returning
    // `InvalidCursor` — see the regression tests
    // `paginate_rejects_nonzero_offset_against_a_now_empty_row_set` and
    // `paginate_rejects_stale_offset_against_a_row_set_shrunk_to_empty`.
    // `offset != 0` is the correct, narrow carve-out: it excludes only the
    // legitimate offset-0-over-empty-rows first page, and still rejects
    // every other offset that is `>= rows.len()`, empty or not.
    if offset != 0 && offset >= rows.len() {
        return Err(QueryDomainError::InvalidCursor);
    }
    let limit = validated.effective_limit as usize;
    // `.min(rows.len())` already bounds `end`; `saturating_add` is pure
    // slice-arithmetic safety (limit is capped at MAX_PAGE_SIZE and offset is
    // already known <= rows.len(), or exempted as offset==0-over-empty-rows,
    // by the guard above), not a security widening — the acceptance
    // decision was made by the guard above.
    let end = offset.saturating_add(limit).min(rows.len());
    let page_rows = rows[offset..end].to_vec();
    let next_cursor = if end < rows.len() {
        Some(encode_cursor_raw(
            &validated.tenant_id,
            end,
            validated.scope_fingerprint,
        ))
    } else {
        None
    };
    Ok(QueryResult {
        rows: page_rows,
        next_cursor,
    })
}

/// The subset of an [`AuditQuery`]'s filter legs that a pagination cursor is
/// scoped to. Two cursors are only interchangeable when every field here is
/// identical between the query that minted the cursor and the query it is
/// presented against — see the module docs, section (b), for why an
/// `offset` alone is not a safe resume position across a changed scope.
///
/// `limit` is included as `effective_limit` (the resolved value, not the
/// caller's raw `Option<u32>`), since it is `effective_limit` that
/// determined how far the minting page advanced.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CursorScope<'a> {
    pub pack: Option<&'a str>,
    pub period: Option<&'a str>,
    pub event_type: Option<&'a str>,
    pub principal: Option<&'a str>,
    pub entity: Option<&'a str>,
    pub limit: u32,
}

/// Encode an opaque pagination cursor binding `tenant_id`, `offset`, and
/// `scope`. See the module docs, section (b), for the wire format and what
/// the checksum and scope fingerprint do and do not guarantee.
pub fn encode_cursor(tenant_id: &str, offset: usize, scope: CursorScope<'_>) -> String {
    encode_cursor_raw(tenant_id, offset, compute_scope_fingerprint(&scope))
}

/// Decode a cursor previously produced by [`encode_cursor`], REQUIRING its
/// embedded tenant to equal `tenant_id` AND its embedded scope fingerprint
/// to equal the fingerprint of `scope`. Every failure mode — malformed hex,
/// a truncated payload, a checksum mismatch, an unrecognized version, a
/// non-canonical offset field, a tenant mismatch, or a scope mismatch —
/// fails closed as [`QueryDomainError::InvalidCursor`]; none of them fall
/// back to a default offset.
pub fn decode_cursor(
    cursor: &str,
    tenant_id: &str,
    scope: CursorScope<'_>,
) -> Result<usize, QueryDomainError> {
    decode_cursor_raw(cursor, tenant_id, compute_scope_fingerprint(&scope))
}

/// Fingerprint a [`CursorScope`] with the same length-prefix
/// domain-separation scheme used for the rest of the cursor payload, so that
/// e.g. `pack: Some("a"), event_type: Some("bc")` cannot collide with
/// `pack: Some("ab"), event_type: Some("c")`, and `None` cannot collide with
/// `Some("")` (a presence tag byte precedes every optional field).
fn compute_scope_fingerprint(scope: &CursorScope<'_>) -> u64 {
    let mut buffer = Vec::new();
    encode_optional_field(&mut buffer, scope.pack);
    encode_optional_field(&mut buffer, scope.period);
    encode_optional_field(&mut buffer, scope.event_type);
    encode_optional_field(&mut buffer, scope.principal);
    encode_optional_field(&mut buffer, scope.entity);
    encode_field(&mut buffer, &scope.limit.to_be_bytes());
    fnv1a64(&buffer)
}

fn encode_optional_field(buffer: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(v) => {
            buffer.push(1);
            encode_field(buffer, v.as_bytes());
        }
        None => buffer.push(0),
    }
}

/// Encode the four-field cursor payload (`version`, `tenant_id`, `offset`,
/// `scope_fingerprint`) plus its checksum. Shared by [`encode_cursor`] and
/// [`paginate`] (which mints `next_cursor` under the already-validated
/// query's own scope fingerprint without recomputing it).
fn encode_cursor_raw(tenant_id: &str, offset: usize, scope_fingerprint: u64) -> String {
    let mut payload = Vec::new();
    encode_field(&mut payload, CURSOR_VERSION.as_bytes());
    encode_field(&mut payload, tenant_id.as_bytes());
    encode_field(&mut payload, offset.to_string().as_bytes());
    encode_field(&mut payload, &scope_fingerprint.to_be_bytes());
    let checksum = fnv1a64(&payload);
    let mut full = payload;
    full.extend_from_slice(&checksum.to_be_bytes());
    encode_hex(&full)
}

/// Decode a cursor previously produced by [`encode_cursor_raw`], checking
/// `tenant_id` and `scope_fingerprint` against the already-computed values.
/// Shared by [`decode_cursor`] and `validate_query` (which needs the decoded
/// offset before a [`ValidatedAuditQuery`] — with its own stored
/// `scope_fingerprint` — exists to hand to the public wrapper).
fn decode_cursor_raw(
    cursor: &str,
    tenant_id: &str,
    scope_fingerprint: u64,
) -> Result<usize, QueryDomainError> {
    let bytes = decode_hex(cursor)?;
    if bytes.len() < 8 {
        return Err(QueryDomainError::InvalidCursor);
    }
    let split_at = bytes.len() - 8;
    let (payload, checksum_bytes) = bytes.split_at(split_at);
    let mut checksum_arr = [0_u8; 8];
    checksum_arr.copy_from_slice(checksum_bytes);
    let expected_checksum = u64::from_be_bytes(checksum_arr);
    if fnv1a64(payload) != expected_checksum {
        return Err(QueryDomainError::InvalidCursor);
    }

    let mut pos = 0_usize;
    let version = decode_field(payload, &mut pos)?;
    let cursor_tenant = decode_field(payload, &mut pos)?;
    let offset_field = decode_field(payload, &mut pos)?;
    let scope_field = decode_field(payload, &mut pos)?;
    if pos != payload.len() {
        // Trailing bytes after the four expected fields: not a cursor this
        // crate minted.
        return Err(QueryDomainError::InvalidCursor);
    }
    if version != CURSOR_VERSION.as_bytes() {
        return Err(QueryDomainError::InvalidCursor);
    }
    // The security-relevant check: a cursor minted for a different tenant is
    // refused outright, never silently honoured.
    if cursor_tenant != tenant_id.as_bytes() {
        return Err(QueryDomainError::InvalidCursor);
    }
    // The scope-relevant check: a cursor minted under a different query scope
    // (pack/period/event_type/principal/entity/limit) is refused outright,
    // never honoured against an unrelated result set.
    if scope_field.len() != 8 {
        return Err(QueryDomainError::InvalidCursor);
    }
    let mut scope_arr = [0_u8; 8];
    scope_arr.copy_from_slice(scope_field);
    if u64::from_be_bytes(scope_arr) != scope_fingerprint {
        return Err(QueryDomainError::InvalidCursor);
    }
    parse_canonical_offset(offset_field)
}

/// Parse an offset field that must be byte-for-byte the canonical decimal
/// digit string `usize::to_string()` produces: no leading `+`, and no
/// leading zero other than a bare `"0"`. Per L9, [`encode_cursor_raw`] is
/// this format's only minter and never emits a non-canonical form (`"+7"`,
/// `"0007"`), so one is not a cursor this crate minted.
fn parse_canonical_offset(field: &[u8]) -> Result<usize, QueryDomainError> {
    if field.is_empty() || !field.iter().all(u8::is_ascii_digit) {
        return Err(QueryDomainError::InvalidCursor);
    }
    if field.len() > 1 && field[0] == b'0' {
        return Err(QueryDomainError::InvalidCursor);
    }
    let offset_str = std::str::from_utf8(field).map_err(|_| QueryDomainError::InvalidCursor)?;
    offset_str
        .parse::<usize>()
        .map_err(|_| QueryDomainError::InvalidCursor)
}

/// Authorize an [`AuditorEngagement`] for `query_tenant_id` at
/// `now_epoch_seconds` (Unix epoch seconds, caller-supplied — see the module
/// docs, section (d), for why this crate cannot read a clock itself).
///
/// Rejects a blank/padded/content-free `engagement_id`
/// ([`QueryDomainError::EmptyEngagementId`]), a blank/padded/content-free
/// tenant on EITHER side ([`QueryDomainError::EmptyTenantId`] — per L7, an
/// equality comparison between two unvalidated strings is not the same as
/// validating either one), a tenant mismatch
/// ([`QueryDomainError::EngagementTenantMismatch`]), a malformed
/// `expires_at` ([`QueryDomainError::InvalidExpiresAt`]), and an engagement
/// whose window has already closed at `now_epoch_seconds`
/// ([`QueryDomainError::EngagementExpired`]) — `now_epoch_seconds >=
/// expires_at` is treated as expired (the boundary instant itself is not
/// valid; see the module docs, section (d), for how this compares to
/// `auditor-scope.cedar`'s inclusive bound).
pub fn authorize_auditor_engagement(
    engagement: &AuditorEngagement,
    query_tenant_id: &str,
    now_epoch_seconds: i64,
) -> Result<(), QueryDomainError> {
    if !validate_identifier(&engagement.engagement_id) {
        return Err(QueryDomainError::EmptyEngagementId);
    }
    // L7: validate BOTH tenant legs before comparing them. Without this, an
    // empty/whitespace-only/zero-width-only tenant on both sides compares
    // equal and would authorize against a blank tenant scope.
    if !validate_identifier(&engagement.tenant_id) || !validate_identifier(query_tenant_id) {
        return Err(QueryDomainError::EmptyTenantId);
    }
    if engagement.tenant_id != query_tenant_id {
        return Err(QueryDomainError::EngagementTenantMismatch {
            engagement_tenant_id: engagement.tenant_id.clone(),
            query_tenant_id: query_tenant_id.to_string(),
        });
    }
    let expires_epoch = parse_rfc3339_utc_seconds(&engagement.expires_at).ok_or_else(|| {
        QueryDomainError::InvalidExpiresAt {
            expires_at: engagement.expires_at.clone(),
        }
    })?;
    if now_epoch_seconds >= expires_epoch {
        return Err(QueryDomainError::EngagementExpired {
            expires_at: engagement.expires_at.clone(),
        });
    }
    Ok(())
}

/// An identifier is acceptable when it CONTAINS at least one alphanumeric
/// character and is already normalized (`value == value.trim()`).
///
/// Per L3: `str::trim` strips only `White_Space` codepoints, so U+200B
/// (ZWSP), U+FEFF (BOM), U+2060, and NUL all survive it — an identifier made
/// purely of those would pass a bare `.trim().is_empty()` check. Requiring
/// at least one `char::is_alphanumeric()` codepoint catches that case
/// because none of those codepoints are alphanumeric.
///
/// Per L2: this function does not trim-and-accept; a padded value
/// (`value != value.trim()`) is rejected outright rather than normalized, so
/// the caller's stored value is always exactly what was validated.
fn validate_identifier(value: &str) -> bool {
    value == value.trim() && value.chars().any(char::is_alphanumeric)
}

/// Parse `period` into a [`PeriodWindow`], accepting the three shapes
/// documented in the module docs, section (a): a bare `YYYY-MM-DD` day, a
/// bare `YYYY-MM` month, or an explicit `"<start>/<end>"` range. Tried in
/// that order: `split_once('/')` finds the range delimiter first (a bare
/// date/month never contains `/`), and a 10-byte value with no `/` is tried
/// as a day before the 7-byte month shape is tried (the two shapes have
/// disjoint lengths, so there is no ambiguity between them).
fn parse_period_window(period: &str) -> Result<PeriodWindow, QueryDomainError> {
    let malformed = || QueryDomainError::InvalidPeriodWindow {
        period: period.to_string(),
    };
    if let Some((start_str, end_str)) = period.split_once('/') {
        let (start_year, start_month, start_day) =
            parse_date_ymd(start_str).ok_or_else(malformed)?;
        let (end_year, end_month, end_day) = parse_date_ymd(end_str).ok_or_else(malformed)?;
        let start_day_number = days_from_civil(start_year, start_month, start_day);
        let end_day_number = days_from_civil(end_year, end_month, end_day);
        if end_day_number < start_day_number {
            return Err(malformed());
        }
        let span_days =
            u32::try_from(end_day_number - start_day_number + 1).map_err(|_| malformed())?;
        return Ok(PeriodWindow {
            start_day: start_day_number,
            end_day: end_day_number,
            span_days,
        });
    }
    if let Some((year, month, day)) = parse_date_ymd(period) {
        let day_number = days_from_civil(year, month, day);
        return Ok(PeriodWindow {
            start_day: day_number,
            end_day: day_number,
            span_days: 1,
        });
    }
    if let Some((year, month)) = parse_year_month(period) {
        let first_day_number = days_from_civil(year, month, 1);
        let days = days_in_month(year, month);
        let last_day_number = days_from_civil(year, month, days);
        return Ok(PeriodWindow {
            start_day: first_day_number,
            end_day: last_day_number,
            span_days: days,
        });
    }
    Err(malformed())
}

/// Parse a strict `YYYY-MM` calendar month (no day component).
fn parse_year_month(value: &str) -> Option<(i64, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return None;
    }
    let year = parse_ascii_digits(&value[0..4])?;
    let month = parse_ascii_digits(&value[5..7])?;
    if !(1..=12).contains(&month) {
        return None;
    }
    Some((year as i64, month))
}

/// Parse a strict `YYYY-MM-DD` calendar date, range-checking the day against
/// the actual number of days in that month/year (so `2026-02-30` is
/// rejected, not silently accepted).
fn parse_date_ymd(value: &str) -> Option<(i64, u32, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = parse_ascii_digits(&value[0..4])?;
    let month = parse_ascii_digits(&value[5..7])?;
    let day = parse_ascii_digits(&value[8..10])?;
    if !(1..=12).contains(&month) {
        return None;
    }
    if day == 0 || day > days_in_month(year as i64, month) {
        return None;
    }
    Some((year as i64, month, day))
}

/// Parse a strict `YYYY-MM-DDTHH:MM:SSZ` UTC timestamp (no fractional
/// seconds, no non-`Z` offset) into Unix epoch seconds.
fn parse_rfc3339_utc_seconds(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 20 || bytes[10] != b'T' || bytes[19] != b'Z' {
        return None;
    }
    let (year, month, day) = parse_date_ymd(&value[0..10])?;
    if bytes[13] != b':' || bytes[16] != b':' {
        return None;
    }
    let hour = parse_ascii_digits(&value[11..13])?;
    let minute = parse_ascii_digits(&value[14..16])?;
    let second = parse_ascii_digits(&value[17..19])?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let day_number = days_from_civil(year, month, day);
    Some(day_number * 86_400 + i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second))
}

fn parse_ascii_digits(value: &str) -> Option<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days since the 1970-01-01 civil (proleptic Gregorian) epoch. Pure integer
/// arithmetic — Howard Hinnant's `days_from_civil` algorithm
/// (public domain), reproduced here rather than pulled in as a dependency.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let year_of_era = y - era * 400; // [0, 399]
    let month_prime = (i64::from(month) + 9) % 12; // [0, 11]
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1; // [0, 365]
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year; // [0, 146096]
    era * 146_097 + day_of_era - 719_468
}

fn encode_field(buffer: &mut Vec<u8>, value: &[u8]) {
    buffer.extend_from_slice(&(value.len() as u64).to_be_bytes());
    buffer.extend_from_slice(value);
}

/// Read one length-prefixed field starting at `*pos`, advancing `*pos` past
/// it. Fails closed (never saturates) on every malformed shape: a length
/// prefix cut short by truncation, a declared length wider than `usize` on
/// this platform (`usize::try_from`, not `.unwrap_or(usize::MAX)` — per L4),
/// or a declared length that runs past the end of `bytes`.
fn decode_field<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<&'a [u8], QueryDomainError> {
    let length_end = pos.checked_add(8).ok_or(QueryDomainError::InvalidCursor)?;
    if length_end > bytes.len() {
        return Err(QueryDomainError::InvalidCursor);
    }
    let mut length_bytes = [0_u8; 8];
    length_bytes.copy_from_slice(&bytes[*pos..length_end]);
    let declared_length = u64::from_be_bytes(length_bytes);
    let declared_length =
        usize::try_from(declared_length).map_err(|_| QueryDomainError::InvalidCursor)?;
    let field_end = length_end
        .checked_add(declared_length)
        .ok_or(QueryDomainError::InvalidCursor)?;
    if field_end > bytes.len() {
        return Err(QueryDomainError::InvalidCursor);
    }
    let field = &bytes[length_end..field_end];
    *pos = field_end;
    Ok(field)
}

/// FNV-1a 64-bit. Non-cryptographic; see the module docs, section (b), for
/// exactly what this checksum guarantees (corruption detection) and does not
/// (forgery resistance).
fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Decode a lower-hex string. Rejects odd length, uppercase, and any
/// non-hex-digit byte — this crate only ever EMITS lowercase hex, so
/// anything else is not a cursor this crate minted (per L9).
fn decode_hex(hex: &str) -> Result<Vec<u8>, QueryDomainError> {
    let bytes = hex.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(QueryDomainError::InvalidCursor);
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        out.push((high << 4) | low);
    }
    Ok(out)
}

fn hex_value(byte: u8) -> Result<u8, QueryDomainError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(QueryDomainError::InvalidCursor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_identifier (L2 / L3) ──────────────────────────────────

    #[test]
    fn validate_identifier_rejects_empty_string() {
        assert!(!validate_identifier(""));
    }

    #[test]
    fn validate_identifier_rejects_whitespace_only() {
        assert!(!validate_identifier("   "));
    }

    #[test]
    fn validate_identifier_rejects_padded_value_instead_of_trimming() {
        // L2: reject non-normalized input rather than silently trimming and
        // storing a different string than the caller supplied.
        assert!(!validate_identifier(" tenant"));
        assert!(!validate_identifier("tenant "));
        assert!(!validate_identifier("tenant\n"));
    }

    #[test]
    fn validate_identifier_rejects_invisible_characters_only() {
        // L3: ZWSP, BOM, U+2060, NUL all survive `.trim()`.
        assert!(!validate_identifier("\u{200B}"));
        assert!(!validate_identifier("\u{FEFF}"));
        assert!(!validate_identifier("\u{2060}"));
        assert!(!validate_identifier("\0"));
        assert!(!validate_identifier("\u{200B}\u{FEFF}\0"));
    }

    #[test]
    fn validate_identifier_accepts_normalized_content() {
        assert!(validate_identifier("tenant-alpha"));
        assert!(validate_identifier("pack-eu"));
    }

    // ── date / period parsing ───────────────────────────────────────────

    #[test]
    fn days_from_civil_matches_known_unix_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(days_from_civil(2026, 1, 1), 20_454);
    }

    #[test]
    fn parse_date_ymd_rejects_invalid_calendar_day() {
        assert_eq!(parse_date_ymd("2026-02-30"), None); // Feb never has 30 days
        assert_eq!(parse_date_ymd("2026-13-01"), None); // month 13
        assert_eq!(parse_date_ymd("2026-00-01"), None); // month 0
        assert_eq!(parse_date_ymd("2026-01-00"), None); // day 0
    }

    #[test]
    fn parse_date_ymd_accepts_leap_day() {
        assert_eq!(parse_date_ymd("2024-02-29"), Some((2024, 2, 29)));
        assert_eq!(parse_date_ymd("2023-02-29"), None); // 2023 is not a leap year
    }

    #[test]
    fn parse_date_ymd_rejects_malformed_shape() {
        assert_eq!(parse_date_ymd("2026/01/01"), None);
        assert_eq!(parse_date_ymd("26-01-01"), None);
        assert_eq!(parse_date_ymd("2026-01-01x"), None);
        assert_eq!(parse_date_ymd(""), None);
    }

    // ── period window grammar (findings B/#4/#6) ─────────────────────────

    #[test]
    fn parse_period_window_accepts_bare_day() {
        let window = parse_period_window("2026-08-15").unwrap();
        assert_eq!(window.span_days(), 1);
        assert_eq!(window.start_day_number(), window.end_day_number());
    }

    #[test]
    fn parse_period_window_accepts_bare_month() {
        // August 2026 has 31 days.
        let window = parse_period_window("2026-08").unwrap();
        assert_eq!(window.span_days(), 31);
        assert_eq!(window.end_day_number() - window.start_day_number() + 1, 31);
    }

    #[test]
    fn parse_period_window_still_accepts_explicit_range() {
        let window = parse_period_window("2026-01-01/2026-01-03").unwrap();
        assert_eq!(window.span_days(), 3);
    }

    #[test]
    fn parse_period_window_rejects_garbage() {
        assert!(matches!(
            parse_period_window("not-a-period"),
            Err(QueryDomainError::InvalidPeriodWindow { .. })
        ));
    }

    // ── canonical cursor offset field (finding #9) ────────────────────────

    #[test]
    fn parse_canonical_offset_accepts_the_exact_form_usize_to_string_produces() {
        assert_eq!(parse_canonical_offset(b"0"), Ok(0));
        assert_eq!(parse_canonical_offset(b"7"), Ok(7));
        assert_eq!(parse_canonical_offset(b"12345"), Ok(12_345));
    }

    #[test]
    fn parse_canonical_offset_rejects_leading_zero() {
        assert_eq!(
            parse_canonical_offset(b"0007"),
            Err(QueryDomainError::InvalidCursor)
        );
    }

    #[test]
    fn parse_canonical_offset_rejects_leading_plus_sign() {
        assert_eq!(
            parse_canonical_offset(b"+7"),
            Err(QueryDomainError::InvalidCursor)
        );
    }

    #[test]
    fn parse_canonical_offset_rejects_empty_field() {
        assert_eq!(
            parse_canonical_offset(b""),
            Err(QueryDomainError::InvalidCursor)
        );
    }

    // ── scope fingerprint domain separation ───────────────────────────────

    #[test]
    fn compute_scope_fingerprint_distinguishes_field_boundary_shifts() {
        let a = CursorScope {
            pack: None,
            period: None,
            event_type: Some("ab"),
            principal: Some("c"),
            entity: None,
            limit: 10,
        };
        let b = CursorScope {
            pack: None,
            period: None,
            event_type: Some("a"),
            principal: Some("bc"),
            entity: None,
            limit: 10,
        };
        assert_ne!(compute_scope_fingerprint(&a), compute_scope_fingerprint(&b));
    }

    #[test]
    fn compute_scope_fingerprint_distinguishes_none_from_present_empty() {
        let none_scope = CursorScope {
            pack: None,
            ..CursorScope::default()
        };
        let empty_scope = CursorScope {
            pack: Some(""),
            ..CursorScope::default()
        };
        assert_ne!(
            compute_scope_fingerprint(&none_scope),
            compute_scope_fingerprint(&empty_scope)
        );
    }

    #[test]
    fn compute_scope_fingerprint_distinguishes_limit() {
        let a = CursorScope {
            limit: 10,
            ..CursorScope::default()
        };
        let b = CursorScope {
            limit: 20,
            ..CursorScope::default()
        };
        assert_ne!(compute_scope_fingerprint(&a), compute_scope_fingerprint(&b));
    }

    // ── hex codec ───────────────────────────────────────────────────────

    #[test]
    fn hex_roundtrip() {
        let bytes = vec![0_u8, 1, 254, 255, 16, 128];
        assert_eq!(decode_hex(&encode_hex(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn decode_hex_rejects_odd_length() {
        assert_eq!(decode_hex("abc"), Err(QueryDomainError::InvalidCursor));
    }

    #[test]
    fn decode_hex_rejects_uppercase() {
        // This crate only ever emits lowercase hex.
        assert_eq!(decode_hex("AB"), Err(QueryDomainError::InvalidCursor));
    }

    #[test]
    fn decode_hex_rejects_non_hex_digit() {
        assert_eq!(decode_hex("zz"), Err(QueryDomainError::InvalidCursor));
    }

    // ── field framing ──────────────────────────────────────────────────

    #[test]
    fn decode_field_reads_back_encoded_value() {
        let mut buffer = Vec::new();
        encode_field(&mut buffer, b"hello");
        let mut pos = 0;
        assert_eq!(decode_field(&buffer, &mut pos).unwrap(), b"hello");
        assert_eq!(pos, buffer.len());
    }

    #[test]
    fn decode_field_rejects_truncated_length_prefix() {
        let mut pos = 0;
        assert_eq!(
            decode_field(&[0, 0, 0], &mut pos),
            Err(QueryDomainError::InvalidCursor)
        );
    }

    #[test]
    fn decode_field_rejects_length_past_end_of_buffer() {
        // Declares a field of length 100 but supplies none of it.
        let buffer = 100_u64.to_be_bytes().to_vec();
        let mut pos = 0;
        assert_eq!(
            decode_field(&buffer, &mut pos),
            Err(QueryDomainError::InvalidCursor)
        );
    }

    // ── fnv1a64 sanity ──────────────────────────────────────────────────

    #[test]
    fn fnv1a64_is_deterministic_and_order_dependent() {
        assert_eq!(fnv1a64(b"abc"), fnv1a64(b"abc"));
        assert_ne!(fnv1a64(b"abc"), fnv1a64(b"acb"));
        assert_ne!(fnv1a64(b""), fnv1a64(b"a"));
    }
}
