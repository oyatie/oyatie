//! Audit-chain emission domain: canonical envelope construction and
//! fingerprint verification.
//!
//! This crate owns the pure, I/O-free rules for turning a caller-supplied
//! `(pack, tenant_partition, period, event_id, payload_digest)` tuple into a
//! [`CanonicalEnvelope`]:
//!
//! 1. **Validated construction** — [`CanonicalEnvelope::build`] rejects an
//!    empty/whitespace-only `event_id`, an empty `pack`, an empty
//!    `tenant_partition`, an empty or malformed `period`, an empty
//!    `payload_digest`, and an empty/whitespace-only claimed fingerprint
//!    before any envelope is produced. `event_id`, `pack`, `tenant_partition`
//!    and `payload_digest` are stored in their *trimmed* form, so leading or
//!    trailing whitespace can never spell a shadow tenant partition, a
//!    shadow pack, or a duplicate-looking event id past dedup checks — two
//!    inputs that differ only in surrounding whitespace build the identical
//!    envelope.
//! 2. **Deterministic canonical preimage** — [`canonical_preimage`] encodes
//!    the validated tuple as an unambiguous byte string. Every field is
//!    length-prefixed (a big-endian `u64` byte count immediately before the
//!    field's bytes), so no two distinct field tuples can ever collide onto
//!    the same preimage the way naive separator-joined encodings can (e.g.
//!    `"ab" + "," + "c"` colliding with `"a" + "," + "bc"`).
//! 3. **Fingerprint verification, not computation** — this crate has no
//!    hash-crate dependency and never computes a digest itself. Hashing is
//!    expressed as the [`Fingerprinter`] port the domain owns; the caller
//!    supplies an implementation (typically a thin adapter over a real hash
//!    function) and a claimed fingerprint, and the domain's job is to run
//!    the preimage through that port and confirm the result matches what the
//!    caller claimed, returning [`EmissionDomainError::FingerprintMismatch`]
//!    on drift. This is the correct hexagonal shape for a `core/*-domain`
//!    crate under ADR-0562: a capability the domain cannot reach on its own
//!    is expressed as a port, never as a new dependency.
//! 4. **Period bucketing** — [`validate_period_id`] parses and validates a
//!    `YYYY-MM-DD` UTC-calendar-day period id (this crate's own bucketing
//!    convention, documented at [`validate_period_id`]) without depending on
//!    a date crate, and [`period_id_from_rfc3339`] derives one from an
//!    RFC3339 timestamp by normalizing the timestamp's local wall-clock
//!    reading onto the UTC calendar date first — two RFC3339 spellings of
//!    the *same instant* (e.g. `"2026-02-19T15:00:00Z"` and
//!    `"2026-02-20T00:00:00+09:00"`) always derive the *same* period id.
//! 5. **Envelope immutability** — [`CanonicalEnvelope`]'s fields are
//!    private; the only way to produce one is [`CanonicalEnvelope::build`],
//!    which enforces every rule above, and the only way to read one back is
//!    through its accessor methods ([`CanonicalEnvelope::coordinate`],
//!    [`CanonicalEnvelope::event_id`], [`CanonicalEnvelope::payload_digest`],
//!    [`CanonicalEnvelope::fingerprint`]). A caller can never construct or
//!    mutate an envelope in a way that bypasses validation or leaves the
//!    fingerprint stale.
#![allow(dead_code)]

use audit_emission_kernel::ChainCoordinate;

/// Domain-separation tag mixed into every [`canonical_preimage`]. Bumping
/// this string is a breaking change to the preimage format: it changes every
/// fingerprint computed against it.
const CANONICAL_PREIMAGE_DOMAIN: &str = "audit-emission-envelope-v1";

/// Canonical, validated envelope produced by [`CanonicalEnvelope::build`].
///
/// Every field has already passed the rules documented on `build`:
/// `event_id`, `payload_digest`, `coordinate.pack` and
/// `coordinate.tenant_partition` are non-empty and stored in their trimmed
/// form, `coordinate.period` is a well-formed period id, and `fingerprint`
/// has been confirmed (via a caller-supplied [`Fingerprinter`]) to be the
/// correct fingerprint of the canonical preimage over the other four
/// fields. Fields are private: the only public constructor is `build`, and
/// the only public reads are the accessor methods below, so no caller can
/// assemble or mutate a `CanonicalEnvelope` that skips these rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalEnvelope {
    coordinate: ChainCoordinate,
    event_id: String,       // data_class: INTERNAL_ONLY
    payload_digest: String, // data_class: INTERNAL_ONLY
    fingerprint: String,    // data_class: INTERNAL_ONLY
}

/// Closed set of domain-level failures for emission-envelope construction,
/// preimage handling, and period bucketing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmissionDomainError {
    /// `event_id` was empty, or contained only whitespace.
    EmptyEventId,
    /// `coordinate.pack` was empty, or contained only whitespace.
    EmptyPack,
    /// The fingerprint computed by the caller-supplied [`Fingerprinter`]
    /// over the canonical preimage did not match the caller's claimed
    /// fingerprint.
    FingerprintMismatch,
    /// `coordinate.tenant_partition` was empty, or contained only whitespace.
    EmptyTenantPartition,
    /// `coordinate.period` was empty, or contained only whitespace.
    EmptyPeriod,
    /// `coordinate.period` was non-empty but did not parse as a well-formed
    /// `YYYY-MM-DD` period id (wrong shape, non-digit characters, an
    /// out-of-range month, or a day out of range for its month/year).
    MalformedPeriod { period: String },
    /// `payload_digest` was empty, or contained only whitespace.
    EmptyPayloadDigest,
    /// A timestamp passed to [`period_id_from_rfc3339`] was not a
    /// well-formed RFC3339 timestamp.
    MalformedTimestamp { timestamp: String },
    /// The claimed fingerprint passed to [`CanonicalEnvelope::build`] was
    /// empty, or contained only whitespace. A degraded or misconfigured
    /// [`Fingerprinter`] adapter that returns an empty string on failure
    /// must not be able to produce a "verified" envelope simply because the
    /// caller also claims an empty fingerprint: this rule closes that gap
    /// before the port is ever invoked.
    EmptyFingerprint,
}

/// Port the domain owns for computing a fingerprint over an opaque byte
/// preimage. The domain never links a hash crate itself: implementations
/// (typically thin adapters wrapping a real cryptographic hash) live outside
/// this crate and are supplied by the caller. See the crate-level docs for
/// why this is a port rather than a dependency.
pub trait Fingerprinter {
    /// Compute a fingerprint string for `preimage`. Implementations must be
    /// deterministic: the same bytes must always produce the same output.
    fn fingerprint(&self, preimage: &[u8]) -> String;
}

impl CanonicalEnvelope {
    /// Validate `coordinate`, `event_id`, and `payload_digest`, then verify
    /// `claimed_fingerprint` against the canonical preimage of those fields
    /// using `fingerprinter`.
    ///
    /// `event_id`, `coordinate.pack`, `coordinate.tenant_partition`, and
    /// `payload_digest` are stored in their trimmed form: emptiness is
    /// judged after trimming, and the value actually stored (and fed into
    /// the canonical preimage) is the trimmed value, so two inputs that
    /// differ only in surrounding whitespace produce the identical
    /// envelope rather than two envelopes that silently diverge.
    ///
    /// # Errors
    ///
    /// - [`EmissionDomainError::EmptyEventId`] if `event_id` is empty or
    ///   whitespace-only.
    /// - [`EmissionDomainError::EmptyPack`] if `coordinate.pack` is empty or
    ///   whitespace-only.
    /// - [`EmissionDomainError::EmptyTenantPartition`] if
    ///   `coordinate.tenant_partition` is empty or whitespace-only.
    /// - [`EmissionDomainError::EmptyPeriod`] /
    ///   [`EmissionDomainError::MalformedPeriod`] if `coordinate.period`
    ///   fails [`validate_period_id`].
    /// - [`EmissionDomainError::EmptyPayloadDigest`] if `payload_digest` is
    ///   empty or whitespace-only.
    /// - [`EmissionDomainError::EmptyFingerprint`] if `claimed_fingerprint`
    ///   is empty or whitespace-only. Checked before `fingerprinter` is
    ///   invoked.
    /// - [`EmissionDomainError::FingerprintMismatch`] if the fingerprint
    ///   `fingerprinter` computes over the canonical preimage does not equal
    ///   `claimed_fingerprint`.
    ///
    /// Validation runs before fingerprint verification, so a malformed
    /// coordinate is reported precisely rather than surfacing as a spurious
    /// mismatch.
    pub fn build(
        coordinate: ChainCoordinate,
        event_id: impl Into<String>,
        payload_digest: impl Into<String>,
        claimed_fingerprint: impl Into<String>,
        fingerprinter: &dyn Fingerprinter,
    ) -> Result<Self, EmissionDomainError> {
        let mut event_id = event_id.into();
        let mut payload_digest = payload_digest.into();
        let claimed_fingerprint = claimed_fingerprint.into();
        let mut coordinate = coordinate;

        if event_id.trim().is_empty() {
            return Err(EmissionDomainError::EmptyEventId);
        }
        event_id = event_id.trim().to_string();

        if coordinate.pack.trim().is_empty() {
            return Err(EmissionDomainError::EmptyPack);
        }
        coordinate.pack = coordinate.pack.trim().to_string();

        if coordinate.tenant_partition.trim().is_empty() {
            return Err(EmissionDomainError::EmptyTenantPartition);
        }
        coordinate.tenant_partition = coordinate.tenant_partition.trim().to_string();

        validate_period_id(&coordinate.period)?;

        if payload_digest.trim().is_empty() {
            return Err(EmissionDomainError::EmptyPayloadDigest);
        }
        payload_digest = payload_digest.trim().to_string();

        if claimed_fingerprint.trim().is_empty() {
            return Err(EmissionDomainError::EmptyFingerprint);
        }

        let preimage = canonical_preimage(&coordinate, &event_id, &payload_digest);
        let expected_fingerprint = fingerprinter.fingerprint(&preimage);
        if expected_fingerprint != claimed_fingerprint {
            return Err(EmissionDomainError::FingerprintMismatch);
        }

        Ok(Self {
            coordinate,
            event_id,
            payload_digest,
            fingerprint: expected_fingerprint,
        })
    }

    /// The envelope's chain coordinate: `(pack, tenant_partition, period)`.
    pub fn coordinate(&self) -> &ChainCoordinate {
        &self.coordinate
    }

    /// The envelope's validated, trimmed event id.
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// The envelope's validated, trimmed payload digest.
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }

    /// The envelope's fingerprint, as confirmed against the canonical
    /// preimage of its other fields at construction time.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Re-verify `self.fingerprint` against the canonical preimage of
    /// `self`'s own fields using `fingerprinter`.
    ///
    /// `CanonicalEnvelope`'s fields are private and the only public
    /// constructor is [`build`](Self::build), which itself confirms the
    /// fingerprint before an envelope is ever produced — so within this
    /// crate's own type system, an already-built envelope's fields cannot
    /// drift out from under its fingerprint. `verify` exists for the case
    /// where the *fingerprinter* is what may have changed since
    /// construction (e.g. a hash algorithm or signing-key rotation on the
    /// adapter side): it re-derives the fingerprint with whatever
    /// `Fingerprinter` is supplied here and confirms it still agrees with
    /// the one recorded at build time.
    ///
    /// # Errors
    ///
    /// [`EmissionDomainError::FingerprintMismatch`] if the recomputed
    /// fingerprint does not equal `self.fingerprint`.
    pub fn verify(&self, fingerprinter: &dyn Fingerprinter) -> Result<(), EmissionDomainError> {
        let preimage = canonical_preimage(&self.coordinate, &self.event_id, &self.payload_digest);
        if fingerprinter.fingerprint(&preimage) != self.fingerprint {
            return Err(EmissionDomainError::FingerprintMismatch);
        }
        Ok(())
    }
}

/// Encode `(coordinate.pack, coordinate.tenant_partition, coordinate.period,
/// event_id, payload_digest)` as an unambiguous, injection-proof byte
/// preimage.
///
/// Each field is written as a big-endian `u64` byte length immediately
/// followed by the field's UTF-8 bytes, after a length-prefixed domain tag.
/// Because every field carries its own length, two distinct field tuples can
/// never encode to the same bytes — unlike a naive separator-joined encoding
/// (e.g. `pack + "," + tenant_partition`), where `("ab", "c")` and `("a",
/// "bc")` collide on `"ab,c"` vs `"a,bc"` only by coincidence of this
/// example, but where a field containing the separator itself (e.g. tenant
/// partition `"a,b"` paired with pack `"a"` vs pack `"a,b"` paired with an
/// empty-looking split) can genuinely collide. Length-prefixing removes the
/// possibility entirely: decoding the length always finds the true field
/// boundary, regardless of what bytes the field contains.
///
/// This function is a pure encoder: it does not trim or otherwise normalize
/// its inputs. [`CanonicalEnvelope::build`] is responsible for normalizing
/// (trimming) fields before they reach this function.
pub fn canonical_preimage(
    coordinate: &ChainCoordinate,
    event_id: &str,
    payload_digest: &str,
) -> Vec<u8> {
    let mut buf = Vec::new();
    write_length_prefixed_field(&mut buf, CANONICAL_PREIMAGE_DOMAIN);
    write_length_prefixed_field(&mut buf, &coordinate.pack);
    write_length_prefixed_field(&mut buf, &coordinate.tenant_partition);
    write_length_prefixed_field(&mut buf, &coordinate.period);
    write_length_prefixed_field(&mut buf, event_id);
    write_length_prefixed_field(&mut buf, payload_digest);
    buf
}

fn write_length_prefixed_field(buf: &mut Vec<u8>, field: &str) {
    let bytes = field.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    buf.extend_from_slice(bytes);
}

/// Validate that `period` is a well-formed period id.
///
/// This crate's period-id convention is a UTC calendar day, `YYYY-MM-DD`:
/// four digits, `-`, two digits (month, `01`-`12`), `-`, two digits (day,
/// valid for the given month and year, including leap-year February). This
/// function only checks that `period` has that shape; it does not itself
/// derive a period id from a timestamp — see [`period_id_from_rfc3339`] for
/// that, including how it accounts for the UTC offset.
///
/// # Errors
///
/// - [`EmissionDomainError::EmptyPeriod`] if `period` is empty or
///   whitespace-only.
/// - [`EmissionDomainError::MalformedPeriod`] if `period` is non-empty but
///   does not parse as `YYYY-MM-DD` with an in-range month and day.
pub fn validate_period_id(period: &str) -> Result<(), EmissionDomainError> {
    if period.trim().is_empty() {
        return Err(EmissionDomainError::EmptyPeriod);
    }
    if parse_period_components(period).is_some() {
        Ok(())
    } else {
        Err(EmissionDomainError::MalformedPeriod {
            period: period.to_string(),
        })
    }
}

/// Derive a `YYYY-MM-DD` UTC-calendar-day period id from an RFC3339
/// timestamp string, normalizing the timestamp's offset onto UTC first.
///
/// This performs a full structural validation of the timestamp (date, `T`
/// separator, time-of-day, optional fractional seconds, and a `Z` or
/// `±HH:MM` offset) without depending on a date crate, and then shifts the
/// local `(date, hour, minute)` reading by the offset so the returned
/// period id always names the UTC calendar day the instant falls on —
/// regardless of which offset the caller happened to spell the timestamp
/// with. Two RFC3339 strings that name the same instant always derive the
/// same period id: `"2026-02-19T15:00:00Z"` and
/// `"2026-02-20T00:00:00+09:00"` are the same instant and both derive
/// `"2026-02-19"`; `"2026-02-20T23:59:59+09:00"` (14:59:59 UTC) derives
/// `"2026-02-20"`, but `"2026-02-21T00:30:00+09:00"` (15:30:00 UTC the
/// *previous* day) derives `"2026-02-20"`, not `"2026-02-21"`.
///
/// # Errors
///
/// [`EmissionDomainError::MalformedTimestamp`] if `timestamp` is not a
/// well-formed RFC3339 timestamp, including when its own date component is
/// not a valid calendar date, or when the UTC-normalized result would fall
/// outside the representable `0000-01-01`..=`9999-12-31` range.
pub fn period_id_from_rfc3339(timestamp: &str) -> Result<String, EmissionDomainError> {
    let malformed = || EmissionDomainError::MalformedTimestamp {
        timestamp: timestamp.to_string(),
    };

    // Minimum well-formed length: "YYYY-MM-DDTHH:MM:SSZ" == 20 bytes.
    if timestamp.len() < 20 || !timestamp.is_char_boundary(10) || !timestamp.is_char_boundary(19) {
        return Err(malformed());
    }

    let date_part = &timestamp[0..10];
    let separator = timestamp.as_bytes()[10];
    if separator != b'T' && separator != b't' {
        return Err(malformed());
    }
    let (year, month, day) = parse_period_components(date_part).ok_or_else(malformed)?;

    let time_part = &timestamp[11..19];
    let time_bytes = time_part.as_bytes();
    if time_bytes[2] != b':' || time_bytes[5] != b':' {
        return Err(malformed());
    }
    let hour = parse_two_digits(&time_part[0..2]).ok_or_else(malformed)?;
    let minute = parse_two_digits(&time_part[3..5]).ok_or_else(malformed)?;
    // Seconds may reach 60 to tolerate a leap second.
    let second = parse_two_digits(&time_part[6..8]).ok_or_else(malformed)?;
    if hour > 23 || minute > 59 || second > 60 {
        return Err(malformed());
    }

    let rest = &timestamp[19..];
    let offset_minutes = parse_rfc3339_offset_minutes(rest).ok_or_else(malformed)?;

    let (utc_year, utc_month, utc_day) =
        shift_to_utc_date(year, month, day, hour, minute, offset_minutes).ok_or_else(malformed)?;
    if utc_year > 9999 {
        return Err(malformed());
    }

    Ok(format!("{utc_year:04}-{utc_month:02}-{utc_day:02}"))
}

/// Parse `"HH-MM-DD"`-shaped year/month/day digits and validate month and
/// day ranges (leap-year aware). Returns `None` on any malformed input.
fn parse_period_components(period: &str) -> Option<(u32, u32, u32)> {
    let bytes = period.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !period.is_char_boundary(4)
        || !period.is_char_boundary(5)
        || !period.is_char_boundary(7)
        || !period.is_char_boundary(8)
    {
        return None;
    }
    let year = parse_digits(&period[0..4])?;
    let month = parse_digits(&period[5..7])?;
    let day = parse_digits(&period[8..10])?;
    if !(1..=12).contains(&month) {
        return None;
    }
    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return None;
    }
    Some((year, month, day))
}

fn parse_digits(field: &str) -> Option<u32> {
    if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    field.parse::<u32>().ok()
}

fn parse_two_digits(field: &str) -> Option<u32> {
    if field.len() != 2 {
        return None;
    }
    parse_digits(field)
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u32, month: u32) -> u32 {
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

/// Parse the RFC3339 suffix after `HH:MM:SS`: an optional `.`-prefixed
/// fractional-seconds run of one or more digits, followed by either `Z`/`z`
/// or a `±HH:MM` offset with in-range hour/minute. Returns the offset from
/// UTC in minutes (`UTC = local − offset`; `Z` is `Some(0)`), or `None` if
/// the suffix is malformed.
fn parse_rfc3339_offset_minutes(rest: &str) -> Option<i32> {
    let bytes = rest.as_bytes();
    let mut index = 0;

    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        let fraction_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == fraction_start {
            return None; // "." with no digits after it.
        }
    }

    if !rest.is_char_boundary(index) {
        return None;
    }
    let offset = &rest[index..];
    if offset == "Z" || offset == "z" {
        return Some(0);
    }
    let offset_bytes = offset.as_bytes();
    if offset_bytes.len() != 6 {
        return None;
    }
    let sign: i32 = match offset_bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    if offset_bytes[3] != b':' {
        return None;
    }
    let offset_hour = parse_two_digits(&offset[1..3])?;
    let offset_minute = parse_two_digits(&offset[4..6])?;
    if offset_hour > 23 || offset_minute > 59 {
        return None;
    }
    Some(sign * (offset_hour as i32 * 60 + offset_minute as i32))
}

/// Shift a local `(year, month, day, hour, minute)` wall-clock reading
/// (already validated: `month`/`day` a real calendar date, `hour < 24`,
/// `minute < 60`) by `offset_minutes` (the reading's UTC offset — positive
/// east of UTC, so `UTC = local − offset_minutes`) onto the UTC calendar
/// date.
///
/// Seconds do not participate: at minute resolution the local
/// time-of-day is `0..=1439` minutes and a well-formed RFC3339 offset is
/// bounded to `±(23*60+59)` minutes (`±1439`), so the UTC time-of-day
/// (`local − offset`) can only ever land one calendar day before, on, or
/// one calendar day after the local date — never further.
///
/// Returns `None` only in the unrepresentable edge case of shifting
/// backward past `0000-01-01`.
fn shift_to_utc_date(
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    offset_minutes: i32,
) -> Option<(u32, u32, u32)> {
    let local_minutes_of_day = (hour * 60 + minute) as i32;
    let utc_minutes_of_day = local_minutes_of_day - offset_minutes;
    let day_shift = utc_minutes_of_day.div_euclid(1440);

    match day_shift {
        0 => Some((year, month, day)),
        1 => Some(next_day(year, month, day)),
        -1 => previous_day(year, month, day),
        // Unreachable for a validated local time-of-day (0..=1439) shifted
        // by a validated RFC3339 offset (±1439): kept as a defensive,
        // non-panicking fallback rather than an unwrap/assert.
        _ => None,
    }
}

/// The UTC calendar date one day after `(year, month, day)`, which must
/// already be a valid calendar date.
fn next_day(year: u32, month: u32, day: u32) -> (u32, u32, u32) {
    let max_day = days_in_month(year, month);
    if day < max_day {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    }
}

/// The UTC calendar date one day before `(year, month, day)`, which must
/// already be a valid calendar date. `None` only when `(year, month, day)`
/// is `0000-01-01`, which has no representable predecessor.
fn previous_day(year: u32, month: u32, day: u32) -> Option<(u32, u32, u32)> {
    if day > 1 {
        Some((year, month, day - 1))
    } else if month > 1 {
        let previous_month = month - 1;
        Some((year, previous_month, days_in_month(year, previous_month)))
    } else if year > 0 {
        Some((year - 1, 12, 31))
    } else {
        None
    }
}
