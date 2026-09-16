#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub use calendar_event_domain::{CalendarEvent, EventError, require_tenant_scope};

pub trait CalendarStore: Send + Sync {
    fn put_event(&mut self, event: CalendarEvent) -> Result<(), EventError>;
    fn list_events(&self, tenant_scope_ref: &str) -> Result<Vec<CalendarEvent>, EventError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Empty;

    impl CalendarStore for Empty {
        fn put_event(&mut self, event: CalendarEvent) -> Result<(), EventError> {
            event.validate()
        }

        fn list_events(&self, tenant_scope_ref: &str) -> Result<Vec<CalendarEvent>, EventError> {
            require_tenant_scope(tenant_scope_ref)?;
            Ok(Vec::new())
        }
    }

    #[test]
    fn trait_lists_nothing_until_an_adapter_stores() {
        let store = Empty;
        assert!(store.list_events("tenant:t").unwrap().is_empty());
    }
}
