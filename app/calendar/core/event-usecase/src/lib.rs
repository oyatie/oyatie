#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use calendar_event_api::CalendarStore;
use calendar_event_domain::{CalendarEvent, EventError};

pub fn schedule(
    store: &mut impl CalendarStore,
    event_id: impl Into<String>,
    tenant_scope_ref: impl Into<String>,
    starts_at_unix: i64,
    ends_at_unix: i64,
) -> Result<CalendarEvent, EventError> {
    let event = CalendarEvent::new(event_id, tenant_scope_ref, starts_at_unix, ends_at_unix)?;
    store.put_event(event.clone())?;
    Ok(event)
}

pub fn list_for_tenant(
    store: &impl CalendarStore,
    tenant_scope_ref: &str,
) -> Result<Vec<CalendarEvent>, EventError> {
    store.list_events(tenant_scope_ref)
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_event_api::CalendarStore;

    struct VecStore(Vec<CalendarEvent>);

    impl CalendarStore for VecStore {
        fn put_event(&mut self, event: CalendarEvent) -> Result<(), EventError> {
            event.validate()?;
            self.0.push(event);
            Ok(())
        }

        fn list_events(&self, tenant_scope_ref: &str) -> Result<Vec<CalendarEvent>, EventError> {
            calendar_event_domain::require_tenant_scope(tenant_scope_ref)?;
            Ok(self
                .0
                .iter()
                .filter(|event| event.tenant_scope_ref == tenant_scope_ref)
                .cloned()
                .collect())
        }
    }

    #[test]
    fn schedule_rejects_inverted_range_before_store() {
        let mut store = VecStore(Vec::new());
        assert_eq!(
            schedule(&mut store, "event:e", "tenant:t", 5, 1),
            Err(EventError::InvertedRange)
        );
        assert!(store.0.is_empty());
    }

    #[test]
    fn schedule_then_list_returns_the_tenant_event() {
        let mut store = VecStore(Vec::new());
        let event = schedule(&mut store, "event:e", "tenant:t", 1, 2).unwrap();
        assert_eq!(list_for_tenant(&store, "tenant:t").unwrap(), vec![event]);
        assert!(list_for_tenant(&store, "tenant:other").unwrap().is_empty());
    }
}
