//! In-process change hint per account: a commit signals, an armed waiter
//! wakes and reads the store at once instead of at the polling floor. It is
//! a hint, never the source of truth — waiters still poll at the floor
//! (default 1 s, bounded 250 ms..30 s), so a missed signal costs one
//! interval, not a change. Process-wide on the SQLite tier; the hosted tier
//! replaces it with watches.
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::Notify;

pub const DEFAULT_FLOOR: Duration = Duration::from_secs(1);
pub const MIN_FLOOR: Duration = Duration::from_millis(250);
pub const MAX_FLOOR: Duration = Duration::from_secs(30);
static FLOOR: OnceLock<Duration> = OnceLock::new();

fn table() -> &'static Mutex<HashMap<String, Arc<Notify>>> {
    static TABLE: OnceLock<Mutex<HashMap<String, Arc<Notify>>>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ponytail: the table only grows, one entry per account ever waited on or
// signalled in this process; bound it if a node hosts millions of accounts.
fn handle(account: &str) -> Arc<Notify> {
    let mut table = table().lock().unwrap_or_else(|p| p.into_inner());
    Arc::clone(table.entry(account.to_owned()).or_default())
}

/// Wake every waiter armed on `account`. Nothing is stored: a waiter arms
/// itself (`Notified::enable`) before reading, so a commit during its read
/// is seen at once; an unarmed session sees it at its next floor.
pub fn signal(account: &str) {
    handle(account).notify_waiters();
}

/// The handle a session arms before each read and awaits after it.
pub fn subscribe(account: &str) -> Arc<Notify> {
    handle(account)
}

/// Fix the process-wide polling floor once, clamped to the bounds; later
/// calls are ignored. Without a call the default applies.
pub fn set_floor(requested: Duration) -> Duration {
    *FLOOR.get_or_init(|| requested.clamp(MIN_FLOOR, MAX_FLOOR))
}

pub fn floor() -> Duration {
    FLOOR.get().copied().unwrap_or(DEFAULT_FLOOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_armed_waiter_sees_a_signal_sent_before_it_awaits_and_an_unarmed_one_does_not() {
        let notify = subscribe("acct-armed");
        let armed = notify.notified();
        tokio::pin!(armed);
        armed.as_mut().enable();
        signal("acct-armed");
        tokio::time::timeout(Duration::from_millis(50), armed)
            .await
            .expect("armed before the signal");
        signal("acct-unarmed");
        let late = subscribe("acct-unarmed");
        assert!(
            tokio::time::timeout(Duration::from_millis(50), late.notified())
                .await
                .is_err(),
            "nothing is stored for a waiter that was not armed"
        );
        assert_eq!(set_floor(Duration::from_millis(1)), MIN_FLOOR);
        assert_eq!(set_floor(Duration::from_secs(90)), MIN_FLOOR, "set once");
        assert_eq!(floor(), MIN_FLOOR);
    }
}
