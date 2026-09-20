//! Explicit, locked, backup-verified conversion of a `41ce37e61` database
//! (one JSON `state` column per account) to the relational schema.
mod account;
mod legacy;
mod lock;
mod step;
mod verify;
pub use legacy::{LegacyAccount, LegacyMessage};
pub use lock::{Conversion, ConvertError, Converter};
pub use step::has_step;
pub use verify::BackupCheck;

/// Rough duration printed by `serve` when it refuses a below-version file:
/// conversion parses every body once for threading.
pub fn estimate(accounts: u64, messages: u64) -> String {
    let seconds = (messages / 2_000).max(accounts / 50).max(1);
    if seconds < 120 {
        format!("{seconds} s")
    } else {
        format!("{} min", seconds.div_ceil(60))
    }
}
