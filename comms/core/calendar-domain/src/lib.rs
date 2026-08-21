//! Workspace calendar kernel.
//!
//! Typed kernel records for the W-Workspace-Preview calendar surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. This crate owns only the
//! calendar aggregate and scheduling seam; protocol adapters such as CalDAV stay
//! outside the kernel per ADR-0015.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, DataClassification, PrivacyDataClass};

const CALENDAR_SCHEMA_VERSION: u32 = 1;
const EVENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarError {
    InvalidCalendarId,
    InvalidEventId,
    InvalidTenantId,
    InvalidRegion,
    InvalidTitle,
    InvalidTimeRange,
    InvalidAttendeeEmail,
    InvalidAttendeeRole,
    InvalidLocation,
    InvalidMeetSessionId,
    InvalidRecurrenceRule,
    InvalidDataClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub name: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEventCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub calendar_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub title: String,                        // data_class: PII_QUASI_IDENTIFIER
    pub start_epoch_seconds: u64,             // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,               // data_class: INTERNAL_ONLY
    pub attendees: Vec<Attendee>,             // data_class: PII_IDENTIFYING
    pub location: Option<String>,             // data_class: PII_QUASI_IDENTIFIER
    pub meet_session_id: Option<String>,      // data_class: INTERNAL_ONLY
    pub recurrence: Option<RecurrenceRule>,   // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Calendar {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub name: Classified<String>,                  // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarEvent {
    pub id: Classified<String>,                   // data_class: INTERNAL_ONLY
    pub calendar_id: Classified<String>,          // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub title: Classified<String>,                // data_class: PII_QUASI_IDENTIFIER
    pub start_epoch_seconds: Classified<u64>,     // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: Classified<u64>,       // data_class: INTERNAL_ONLY
    pub attendees: Classified<Vec<Attendee>>,     // data_class: PII_IDENTIFYING
    pub location: Classified<Option<String>>,     // data_class: PII_QUASI_IDENTIFIER
    pub meet_session_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub recurrence: Classified<Option<RecurrenceRule>>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attendee {
    pub email: Classified<String>, // data_class: PII_IDENTIFYING
    pub role: Classified<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurrenceRule {
    pub rule: Classified<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSlot {
    pub calendar_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

pub trait SlotPicker {
    fn find_slots(
        &self,
        tenant_id: &str,
        attendee_emails: &[String],
        earliest_start_epoch_seconds: u64,
        latest_end_epoch_seconds: u64,
        duration_seconds: u64,
    ) -> Result<Vec<CalendarSlot>, CalendarError>;
}

impl Calendar {
    pub fn new(input: CalendarCreate) -> Result<Self, CalendarError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_calendar_data_class());
        validate_non_empty(&input.id, CalendarError::InvalidCalendarId)?;
        validate_non_empty(&input.tenant_id, CalendarError::InvalidTenantId)?;
        validate_non_empty(&input.region, CalendarError::InvalidRegion)?;
        validate_non_empty(&input.name, CalendarError::InvalidTitle)?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            name: Classified::new(input.name, data_class),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(CALENDAR_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl CalendarEvent {
    pub fn new(input: CalendarEventCreate) -> Result<Self, CalendarError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_calendar_data_class());
        validate_non_empty(&input.id, CalendarError::InvalidEventId)?;
        validate_non_empty(&input.calendar_id, CalendarError::InvalidCalendarId)?;
        validate_non_empty(&input.tenant_id, CalendarError::InvalidTenantId)?;
        validate_non_empty(&input.title, CalendarError::InvalidTitle)?;
        validate_time_range(input.start_epoch_seconds, input.end_epoch_seconds)?;
        if input.attendees.is_empty() {
            return Err(CalendarError::InvalidAttendeeEmail);
        }
        for attendee in &input.attendees {
            if attendee.email.data_class != DataClassification::Privacy(attendee_email_data_class())
            {
                return Err(CalendarError::InvalidDataClass);
            }
        }
        if let Some(location) = input.location.as_deref() {
            validate_non_empty(location, CalendarError::InvalidLocation)?;
        }
        if let Some(meet_session_id) = input.meet_session_id.as_deref() {
            validate_non_empty(meet_session_id, CalendarError::InvalidMeetSessionId)?;
        }

        Ok(Self {
            id: internal(input.id),
            calendar_id: internal(input.calendar_id),
            tenant_id: internal(input.tenant_id),
            data_class: internal(data_class),
            title: Classified::new(input.title, data_class),
            start_epoch_seconds: internal(input.start_epoch_seconds),
            end_epoch_seconds: internal(input.end_epoch_seconds),
            attendees: Classified::new(input.attendees, attendee_email_data_class()),
            location: Classified::new(input.location, data_class),
            meet_session_id: internal(input.meet_session_id),
            recurrence: internal(input.recurrence),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(EVENT_SCHEMA_VERSION),
        })
    }

    pub fn duration_seconds(&self) -> u64 {
        self.end_epoch_seconds.value - self.start_epoch_seconds.value
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl Attendee {
    pub fn new(email: String, role: String) -> Result<Self, CalendarError> {
        validate_mail_address(&email)?;
        validate_non_empty(&role, CalendarError::InvalidAttendeeRole)?;
        Ok(Self {
            email: Classified::new(email, attendee_email_data_class()),
            role: internal(role),
        })
    }
}

impl RecurrenceRule {
    pub fn new(rule: String) -> Result<Self, CalendarError> {
        validate_non_empty(&rule, CalendarError::InvalidRecurrenceRule)?;
        Ok(Self {
            rule: internal(rule),
        })
    }
}

impl CalendarSlot {
    pub fn new(
        calendar_id: String,
        start_epoch_seconds: u64,
        end_epoch_seconds: u64,
    ) -> Result<Self, CalendarError> {
        validate_non_empty(&calendar_id, CalendarError::InvalidCalendarId)?;
        validate_time_range(start_epoch_seconds, end_epoch_seconds)?;
        Ok(Self {
            calendar_id: internal(calendar_id),
            start_epoch_seconds: internal(start_epoch_seconds),
            end_epoch_seconds: internal(end_epoch_seconds),
        })
    }
}

pub fn default_workspace_calendar_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn attendee_email_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_calendar_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, CalendarError> {
    PrivacyDataClass::new(data_class).map_err(|_| CalendarError::InvalidDataClass)
}

fn validate_time_range(
    start_epoch_seconds: u64,
    end_epoch_seconds: u64,
) -> Result<(), CalendarError> {
    if start_epoch_seconds >= end_epoch_seconds {
        Err(CalendarError::InvalidTimeRange)
    } else {
        Ok(())
    }
}

fn validate_mail_address(address: &str) -> Result<(), CalendarError> {
    let trimmed = address.trim();
    if trimmed != address || trimmed.chars().any(char::is_whitespace) {
        return Err(CalendarError::InvalidAttendeeEmail);
    }
    let Some((local, domain)) = trimmed.split_once('@') else {
        return Err(CalendarError::InvalidAttendeeEmail);
    };
    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(CalendarError::InvalidAttendeeEmail);
    }
    Ok(())
}

fn validate_non_empty(value: &str, error: CalendarError) -> Result<(), CalendarError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// M03-P06-IP-001 — workspace.calendar.caldav STAGING surface (RFC 4791).
// ---------------------------------------------------------------------------

const CALDAV_RFC: u32 = 4791;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSurfaceStaging {
    pub calendar_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub caldav_rfc_number: Classified<u32>, // data_class: INTERNAL_ONLY
    pub per_tenant_isolated: Classified<bool>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

impl CalendarSurfaceStaging {
    pub fn new(calendar_id: String, tenant_id: String) -> Result<Self, CalendarError> {
        validate_non_empty(&calendar_id, CalendarError::InvalidCalendarId)?;
        validate_non_empty(&tenant_id, CalendarError::InvalidTenantId)?;
        Ok(Self {
            calendar_id: internal(calendar_id),
            tenant_id: internal(tenant_id),
            caldav_rfc_number: internal(CALDAV_RFC),
            per_tenant_isolated: internal(true),
            schema_version: internal(1),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::OperationalDataClass;

    fn attendee() -> Attendee {
        Attendee::new("user@example.com".into(), "required".into()).unwrap()
    }

    fn calendar_input() -> CalendarCreate {
        CalendarCreate {
            id: "cal-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            name: "Team calendar".into(),
            data_class: None,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn event_input() -> CalendarEventCreate {
        CalendarEventCreate {
            id: "event-1".into(),
            calendar_id: "cal-1".into(),
            tenant_id: "tenant-1".into(),
            title: "Planning".into(),
            start_epoch_seconds: 1_700_000_100,
            end_epoch_seconds: 1_700_000_400,
            attendees: vec![attendee()],
            location: Some("Room A".into()),
            meet_session_id: Some("meet-1".into()),
            recurrence: Some(RecurrenceRule::new("FREQ=WEEKLY".into()).unwrap()),
            data_class: None,
            updated_at_epoch_seconds: 1_700_000_050,
        }
    }

    #[test]
    fn calendar_defaults_to_quasi_identifier_class_for_visible_name() {
        let calendar = Calendar::new(calendar_input()).unwrap();

        assert_eq!(
            calendar.privacy_data_class().data_class(),
            DataClass::PiiQuasiIdentifier
        );
        assert_eq!(
            calendar.name.data_class,
            DataClassification::Privacy(default_workspace_calendar_data_class())
        );
    }

    #[test]
    fn event_validates_time_order_and_computes_duration() {
        let event = CalendarEvent::new(event_input()).unwrap();
        assert_eq!(event.duration_seconds(), 300);

        let mut invalid = event_input();
        invalid.end_epoch_seconds = invalid.start_epoch_seconds;
        assert_eq!(
            CalendarEvent::new(invalid),
            Err(CalendarError::InvalidTimeRange)
        );
    }

    #[test]
    fn event_classifies_attendees_as_identifying_and_title_as_quasi() {
        let event = CalendarEvent::new(event_input()).unwrap();

        assert_eq!(
            event.title.data_class,
            DataClassification::Privacy(default_workspace_calendar_data_class())
        );
        assert_eq!(
            event.attendees.value[0].email.data_class,
            DataClassification::Privacy(attendee_email_data_class())
        );
    }

    #[test]
    fn attendee_rejects_invalid_email() {
        assert_eq!(
            Attendee::new("not-an-email".into(), "required".into()),
            Err(CalendarError::InvalidAttendeeEmail)
        );
    }

    #[test]
    fn surface_staging_pins_caldav_rfc_4791_and_per_tenant_isolation() {
        let staging = CalendarSurfaceStaging::new("cal-1".into(), "tenant-1".into()).unwrap();
        assert_eq!(staging.caldav_rfc_number.value, 4791);
        assert!(staging.per_tenant_isolated.value);
    }

    #[test]
    fn surface_staging_rejects_empty_identifiers() {
        assert_eq!(
            CalendarSurfaceStaging::new("".into(), "t".into()),
            Err(CalendarError::InvalidCalendarId)
        );
        assert_eq!(
            CalendarSurfaceStaging::new("c".into(), "".into()),
            Err(CalendarError::InvalidTenantId)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_calendar_data_class_from_legacy(DataClass::Audit),
            Err(CalendarError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
