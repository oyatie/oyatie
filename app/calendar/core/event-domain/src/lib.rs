#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventError {
    Invalid,
    MissingTenantScope,
    InvertedRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEvent {
    pub event_id: String,
    pub tenant_scope_ref: String,
    pub starts_at_unix: i64,
    pub ends_at_unix: i64,
}

impl CalendarEvent {
    pub fn new(
        event_id: impl Into<String>,
        tenant_scope_ref: impl Into<String>,
        starts_at_unix: i64,
        ends_at_unix: i64,
    ) -> Result<Self, EventError> {
        let event = Self {
            event_id: event_id.into(),
            tenant_scope_ref: tenant_scope_ref.into(),
            starts_at_unix,
            ends_at_unix,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), EventError> {
        require_identity(&self.event_id)?;
        require_tenant_scope(&self.tenant_scope_ref)?;
        if self.ends_at_unix <= self.starts_at_unix {
            return Err(EventError::InvertedRange);
        }
        Ok(())
    }
}

pub fn require_tenant_scope(value: &str) -> Result<(), EventError> {
    require_identity(value)?;
    if value.starts_with("tenant:") {
        Ok(())
    } else {
        Err(EventError::MissingTenantScope)
    }
}

fn require_identity(value: &str) -> Result<(), EventError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        Err(EventError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_requires_identity_tenant_and_forward_range() {
        assert_eq!(
            CalendarEvent::new("", "tenant:t", 1, 2),
            Err(EventError::Invalid)
        );
        assert_eq!(
            CalendarEvent::new("event:e", "person:u", 1, 2),
            Err(EventError::MissingTenantScope)
        );
        assert_eq!(
            CalendarEvent::new("event:e", "tenant:t", 2, 2),
            Err(EventError::InvertedRange)
        );
        let event = CalendarEvent::new("event:e", "tenant:t", 1, 2).unwrap();
        assert_eq!(event.event_id, "event:e");
        assert_eq!(event.starts_at_unix, 1);
        assert_eq!(event.ends_at_unix, 2);
    }
}
