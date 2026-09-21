//! What the store keeps and what the wire can carry.

pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;

/// Room kept below `MAX_MESSAGE_BYTES` for the `DKIM-Signature` header the
/// outbound path prepends after the queue. Measured against this build's
/// signer: 655 bytes for the nine signed header names listed once behind a
/// 43-byte domain and a 29-byte selector, and 1182 for the worst case the
/// configuration permits -- every name listed twice, and the 253-byte domain
/// and selector its validation allows. Without the room, a message accepted
/// at the full ceiling is refused by outbound wire validation once signed,
/// which is a `554` and a permanent bounce rather than the delivery the
/// sender was promised.
pub const SIGNATURE_ALLOWANCE: usize = 2048;

/// The largest message submission accepts, so that what is advertised and
/// accepted is what can still be delivered once signed. Unconditional rather
/// than raised when no key is configured: one ceiling is one thing to reason
/// about, and 2 KiB of 25 MiB buys the certainty.
pub const MAX_SUBMISSION_BYTES: usize = MAX_MESSAGE_BYTES - SIGNATURE_ALLOWANCE;
