//! Size ceilings. Each is lower than the last because `normalize` and the
//! outbound signer both add bytes after the size check has passed.

pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;

/// Headers the outbound signature covers. The facade lists each twice in
/// `h=`; `normalize` refuses a message carrying any twice. Both are required:
/// `mail-auth` writes one `h=` entry per occurrence found, so without the
/// refusal the signature has no upper bound.
pub const SIGNED_HEADERS: &[&str] = &[
    "From",
    "To",
    "Cc",
    "Subject",
    "Date",
    "Message-ID",
    "MIME-Version",
    "Content-Type",
    "Content-Transfer-Encoding",
];

/// Measured worst case 1182 bytes, at the longest domain and selector
/// validation permits. Remeasure if `SIGNED_HEADERS` grows.
pub const SIGNATURE_ALLOWANCE: usize = 2048;

/// Unconditional: a ceiling that moved with facade configuration would make
/// `core` depend on state it cannot see.
pub const MAX_SUBMISSION_BYTES: usize = MAX_MESSAGE_BYTES - SIGNATURE_ALLOWANCE;

/// Measured worst case 370 bytes of `Date` and `Message-ID`.
const HEADER_INSERTION_ALLOWANCE: usize = 512;

/// Submission only. Inbound keeps `MAX_MESSAGE_BYTES`, since it neither
/// normalizes nor signs.
pub const MAX_DATA_BYTES: usize = MAX_SUBMISSION_BYTES - HEADER_INSERTION_ALLOWANCE;
