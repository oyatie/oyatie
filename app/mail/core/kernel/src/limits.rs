//! What the store keeps and what the wire can carry.

pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;

/// The header names the outbound signature covers, read at both ends of a
/// coupling neither crate can name in types: the facade's signer
/// (`facade/app/src/signing.rs`) lists each of these twice in `h=`, and
/// submission (`core/service/src/submission.rs`) refuses a message that
/// carries any of them twice. The refusal is what makes the signature
/// bounded -- `mail-auth` writes one `h=` entry per occurrence it finds --
/// so the two lists have to be one list. Change this and both sites move.
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

/// Room kept below `MAX_MESSAGE_BYTES` for the `DKIM-Signature` header the
/// outbound path prepends after the queue. Measured against this build's
/// signer at the 253-byte domain and selector its validation allows: 1182
/// bytes, for an `h=` of eighteen entries -- `SIGNED_HEADERS` listed twice,
/// which is every entry `mail-auth` can write, because it emits one per
/// occurrence found plus one per configured name never found, and a name it
/// found matched case-insensitively so it is the configured name's length.
/// Without the room, a message accepted at the full ceiling is refused by
/// outbound wire validation once signed, which is a `554` and a permanent
/// bounce rather than the delivery the sender was promised.
pub const SIGNATURE_ALLOWANCE: usize = 2048;

/// The largest message the outbound queue may hold, so that what is accepted
/// is what can still be delivered once signed. Unconditional rather than
/// raised when no key is configured: one ceiling is one thing to reason
/// about, and 2 KiB of 25 MiB buys the certainty.
pub const MAX_SUBMISSION_BYTES: usize = MAX_MESSAGE_BYTES - SIGNATURE_ALLOWANCE;

/// Room kept below `MAX_SUBMISSION_BYTES` for what `normalize` inserts into a
/// submission that arrived without it. Worst case on this build: 48 bytes of
/// `Date` (`mail-builder` formats into a 40-byte buffer and writes no more),
/// 320 of `Message-ID` (the name, three 16-byte hex fields and two
/// separators, a 253-byte domain and the brackets) and a 2-byte terminator,
/// so 370, reserved at 512. Without the room, a payload at exactly the
/// advertised `SIZE` carrying neither header is read to the last octet and
/// then refused `552`, which is the one refusal `SIZE` exists to prevent.
const HEADER_INSERTION_ALLOWANCE: usize = 512;

/// What the submission transports advertise, check `MAIL FROM SIZE=` against
/// and stop the DATA reader at, so that what is advertised is what is
/// accepted and what is accepted still delivers once normalized and signed.
/// Inbound advertises `MAX_MESSAGE_BYTES`: it signs nothing and normalizes
/// nothing, so lowering it there would only refuse mail taken before.
pub const MAX_DATA_BYTES: usize = MAX_SUBMISSION_BYTES - HEADER_INSERTION_ALLOWANCE;
