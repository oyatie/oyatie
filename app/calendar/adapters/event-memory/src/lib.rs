//! In-memory `CalendarStore`. Volatile process-local fake; not a durable store.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use calendar_event_api::CalendarStore;
use calendar_event_domain::{CalendarEvent, EventError, require_tenant_scope};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MemoryCalendar {
    events: BTreeMap<(String, String), CalendarEvent>,
}

impl MemoryCalendar {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CalendarStore for MemoryCalendar {
    fn put_event(&mut self, event: CalendarEvent) -> Result<(), EventError> {
        event.validate()?;
        let key = (event.tenant_scope_ref.clone(), event.event_id.clone());
        self.events.insert(key, event);
        Ok(())
    }

    fn list_events(&self, tenant_scope_ref: &str) -> Result<Vec<CalendarEvent>, EventError> {
        require_tenant_scope(tenant_scope_ref)?;
        Ok(self
            .events
            .range((tenant_scope_ref.to_owned(), String::new())..)
            .take_while(|((tenant, _), _)| tenant == tenant_scope_ref)
            .map(|(_, event)| event.clone())
            .collect())
    }
}
