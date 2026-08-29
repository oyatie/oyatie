#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LifecycleWindowStateV1 {
    NotYetValid,
    Active,
    Expired,
}

pub(crate) fn lifecycle_window_state(
    starts_at: LifecycleTimestampV1,
    expires_at: LifecycleTimestampV1,
    evaluated_at: LifecycleTimestampV1,
) -> LifecycleWindowStateV1 {
    if evaluated_at < starts_at {
        LifecycleWindowStateV1::NotYetValid
    } else if evaluated_at > expires_at {
        LifecycleWindowStateV1::Expired
    } else {
        LifecycleWindowStateV1::Active
    }
}

pub(crate) fn checked_lifecycle_timestamp_add(
    timestamp: LifecycleTimestampV1,
    seconds: u64,
) -> Result<LifecycleTimestampV1, LifecycleFailureV1> {
    timestamp
        .unix_seconds()
        .checked_add(seconds)
        .map(LifecycleTimestampV1::from_unix_seconds)
        .ok_or_else(lifecycle_bounds)
}
