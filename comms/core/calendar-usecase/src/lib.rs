//! Calendar capability USECASE — cloud-agnostic application logic over the
//! `comms-calendar-domain` kernel and the `comms-calendar-api` port.
//!
//! This crate composes the calendar invariants the kernel cannot enforce alone:
//! fail-closed authorization at every entrypoint, tenant-isolation guards across
//! the request/aggregate boundary, attendee + recurrence well-formedness, and
//! the scheduling composition (free/busy slot -> validated event). It is pure
//! application logic: no persistence, cloud, identity, or CalDAV backend — those
//! are DEFERRED behind the port traits and supplied by adapters later. CalDAV
//! stays out of this crate and the kernel per ADR-0015.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use comms_calendar_api::{
    AuthorizedCalendarContext, CalendarApiError, CalendarStore, FreeBusyResolver, FreeBusyWindow,
};
use comms_calendar_domain::{
    Attendee, Calendar, CalendarCreate, CalendarError, CalendarEvent, CalendarEventCreate,
    CalendarSlot, RecurrenceRule,
};

/// Usecase-level failures: an authz/boundary failure from the port, a domain
/// invariant failure from the kernel, or a tenant-isolation violation caught
/// here at the application boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarUsecaseError {
    /// Port-boundary failure (authz deny, backend, not-found, malformed).
    Api(CalendarApiError),
    /// Kernel invariant failure (time range, attendee, recurrence, ids, class).
    Domain(CalendarError),
    /// The request's tenant does not match the authorized context tenant.
    TenantMismatch,
    /// No free slot satisfied the scheduling window.
    NoAvailableSlot,
}

impl From<CalendarApiError> for CalendarUsecaseError {
    fn from(error: CalendarApiError) -> Self {
        CalendarUsecaseError::Api(error)
    }
}

impl From<CalendarError> for CalendarUsecaseError {
    fn from(error: CalendarError) -> Self {
        CalendarUsecaseError::Domain(error)
    }
}

/// An attendee as it crosses the application boundary, before kernel validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttendeeInput {
    pub email: String,
    pub role: String,
}

/// Request to create a calendar within the authorized tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCalendarRequest {
    pub id: String,
    pub tenant_id: String,
    pub region: String,
    pub name: String,
    pub created_at_epoch_seconds: u64,
}

/// Request to create an event within the authorized tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateEventRequest {
    pub id: String,
    pub calendar_id: String,
    pub tenant_id: String,
    pub title: String,
    pub start_epoch_seconds: u64,
    pub end_epoch_seconds: u64,
    pub attendees: Vec<AttendeeInput>,
    pub location: Option<String>,
    pub meet_session_id: Option<String>,
    /// An RFC-5545 RRULE string (e.g. `FREQ=WEEKLY`). Validated for a supported
    /// FREQ before the kernel `RecurrenceRule` is built.
    pub recurrence_rule: Option<String>,
    pub updated_at_epoch_seconds: u64,
}

/// Request to schedule an event into the first free slot the resolver returns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleEventRequest {
    pub id: String,
    pub calendar_id: String,
    pub tenant_id: String,
    pub title: String,
    pub attendees: Vec<AttendeeInput>,
    pub location: Option<String>,
    pub meet_session_id: Option<String>,
    pub recurrence_rule: Option<String>,
    pub window: FreeBusyWindow,
    pub updated_at_epoch_seconds: u64,
}

/// Create + persist a calendar. Fail-closed: the context is validated and the
/// request tenant is matched against the authorized tenant BEFORE the kernel
/// aggregate is built or the store is touched.
pub fn create_calendar<S: CalendarStore>(
    store: &S,
    ctx: &AuthorizedCalendarContext,
    req: CreateCalendarRequest,
) -> Result<Calendar, CalendarUsecaseError> {
    ctx.validate()?;
    guard_tenant(ctx, &req.tenant_id)?;
    let calendar = Calendar::new(CalendarCreate {
        id: req.id,
        tenant_id: req.tenant_id,
        region: req.region,
        name: req.name,
        data_class: None,
        created_at_epoch_seconds: req.created_at_epoch_seconds,
    })?;
    store.put_calendar(ctx, &calendar)?;
    Ok(calendar)
}

/// Validate + persist an event. Composes the attendee and recurrence invariants
/// (below) over the kernel, after the fail-closed authz + tenant guard.
pub fn create_event<S: CalendarStore>(
    store: &S,
    ctx: &AuthorizedCalendarContext,
    req: CreateEventRequest,
) -> Result<CalendarEvent, CalendarUsecaseError> {
    ctx.validate()?;
    guard_tenant(ctx, &req.tenant_id)?;
    let event = build_event(
        req.id,
        req.calendar_id,
        req.tenant_id,
        req.title,
        req.start_epoch_seconds,
        req.end_epoch_seconds,
        req.attendees,
        req.location,
        req.meet_session_id,
        req.recurrence_rule,
        req.updated_at_epoch_seconds,
    )?;
    store.put_event(ctx, &event)?;
    Ok(event)
}

/// Schedule an event into the first slot the free/busy resolver returns for the
/// attendee set, then validate + persist it. Fail-closed authz + tenant guard
/// run first; the resolver is called with the SAME validated context.
pub fn schedule_event<S, R>(
    store: &S,
    resolver: &R,
    ctx: &AuthorizedCalendarContext,
    req: ScheduleEventRequest,
) -> Result<CalendarEvent, CalendarUsecaseError>
where
    S: CalendarStore,
    R: FreeBusyResolver,
{
    ctx.validate()?;
    guard_tenant(ctx, &req.tenant_id)?;
    let emails: Vec<String> = req.attendees.iter().map(|a| a.email.clone()).collect();
    let slot = resolver
        .find_slots(ctx, &emails, req.window)?
        .into_iter()
        .next()
        .ok_or(CalendarUsecaseError::NoAvailableSlot)?;
    let CalendarSlot {
        start_epoch_seconds,
        end_epoch_seconds,
        ..
    } = slot;
    let event = build_event(
        req.id,
        req.calendar_id,
        req.tenant_id,
        req.title,
        start_epoch_seconds.value,
        end_epoch_seconds.value,
        req.attendees,
        req.location,
        req.meet_session_id,
        req.recurrence_rule,
        req.updated_at_epoch_seconds,
    )?;
    store.put_event(ctx, &event)?;
    Ok(event)
}

/// The supported RFC-5545 RRULE `FREQ` values. CalDAV/ICS expansion is an
/// adapter concern (ADR-0015); the usecase only admits a well-formed recurrence
/// the kernel will store. Extending this set is a deliberate, reviewed change.
const SUPPORTED_FREQ: [&str; 5] = ["SECONDLY", "MINUTELY", "HOURLY", "DAILY", "WEEKLY"];
const SUPPORTED_FREQ_TAIL: [&str; 2] = ["MONTHLY", "YEARLY"];

/// Validate an attendee at the application boundary (delegating email + role
/// well-formedness to the kernel `Attendee::new`). Rejects an empty attendee
/// set: the kernel also rejects it, but failing here yields the precise
/// usecase error and keeps the invariant explicit.
pub fn validate_attendees(inputs: &[AttendeeInput]) -> Result<Vec<Attendee>, CalendarUsecaseError> {
    if inputs.is_empty() {
        return Err(CalendarUsecaseError::Domain(
            CalendarError::InvalidAttendeeEmail,
        ));
    }
    inputs
        .iter()
        .map(|a| {
            Attendee::new(a.email.clone(), a.role.clone()).map_err(CalendarUsecaseError::Domain)
        })
        .collect()
}

/// Validate an RFC-5545 RRULE for a supported `FREQ` before building the kernel
/// `RecurrenceRule`. `None` => no recurrence (a one-off event). A rule missing a
/// recognized `FREQ=` token is rejected as `InvalidRecurrenceRule`.
pub fn validate_recurrence(
    rule: Option<String>,
) -> Result<Option<RecurrenceRule>, CalendarUsecaseError> {
    match rule {
        None => Ok(None),
        Some(raw) => {
            if !rrule_has_supported_freq(&raw) {
                return Err(CalendarUsecaseError::Domain(
                    CalendarError::InvalidRecurrenceRule,
                ));
            }
            RecurrenceRule::new(raw)
                .map(Some)
                .map_err(CalendarUsecaseError::Domain)
        }
    }
}

fn rrule_has_supported_freq(raw: &str) -> bool {
    raw.split(';').any(|part| {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("").trim().to_ascii_uppercase();
        let value = kv.next().unwrap_or("").trim().to_ascii_uppercase();
        key == "FREQ"
            && (SUPPORTED_FREQ.contains(&value.as_str())
                || SUPPORTED_FREQ_TAIL.contains(&value.as_str()))
    })
}

#[allow(clippy::too_many_arguments)]
fn build_event(
    id: String,
    calendar_id: String,
    tenant_id: String,
    title: String,
    start_epoch_seconds: u64,
    end_epoch_seconds: u64,
    attendees: Vec<AttendeeInput>,
    location: Option<String>,
    meet_session_id: Option<String>,
    recurrence_rule: Option<String>,
    updated_at_epoch_seconds: u64,
) -> Result<CalendarEvent, CalendarUsecaseError> {
    let attendees = validate_attendees(&attendees)?;
    let recurrence = validate_recurrence(recurrence_rule)?;
    CalendarEvent::new(CalendarEventCreate {
        id,
        calendar_id,
        tenant_id,
        title,
        start_epoch_seconds,
        end_epoch_seconds,
        attendees,
        location,
        meet_session_id,
        recurrence,
        data_class: None,
        updated_at_epoch_seconds,
    })
    .map_err(CalendarUsecaseError::Domain)
}

/// Tenant-isolation guard: the request's tenant MUST equal the authorized
/// context tenant. A mismatch is a hard application-boundary deny, independent of
/// any backend RLS (defense in depth).
fn guard_tenant(
    ctx: &AuthorizedCalendarContext,
    request_tenant_id: &str,
) -> Result<(), CalendarUsecaseError> {
    if ctx.tenant_id() == request_tenant_id {
        Ok(())
    } else {
        Err(CalendarUsecaseError::TenantMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn ctx() -> AuthorizedCalendarContext {
        AuthorizedCalendarContext {
            principal_ref: "user:u".into(),
            tenant_scope_ref: "tenant:t".into(),
            policy_decision_ref: "cedar:allow:calendar".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    fn attendees() -> Vec<AttendeeInput> {
        vec![AttendeeInput {
            email: "user@example.com".into(),
            role: "required".into(),
        }]
    }

    #[derive(Default)]
    struct FakeStore {
        calendars: RefCell<Vec<Calendar>>,
        events: RefCell<Vec<CalendarEvent>>,
    }

    impl CalendarStore for FakeStore {
        fn put_calendar(
            &self,
            _ctx: &AuthorizedCalendarContext,
            calendar: &Calendar,
        ) -> Result<(), CalendarApiError> {
            self.calendars.borrow_mut().push(calendar.clone());
            Ok(())
        }
        fn get_calendar(
            &self,
            _ctx: &AuthorizedCalendarContext,
            calendar_id: &str,
        ) -> Result<Calendar, CalendarApiError> {
            self.calendars
                .borrow()
                .iter()
                .find(|c| c.id.value == calendar_id)
                .cloned()
                .ok_or(CalendarApiError::NotFound)
        }
        fn put_event(
            &self,
            _ctx: &AuthorizedCalendarContext,
            event: &CalendarEvent,
        ) -> Result<(), CalendarApiError> {
            self.events.borrow_mut().push(event.clone());
            Ok(())
        }
        fn list_events(
            &self,
            _ctx: &AuthorizedCalendarContext,
            calendar_id: &str,
        ) -> Result<Vec<CalendarEvent>, CalendarApiError> {
            Ok(self
                .events
                .borrow()
                .iter()
                .filter(|e| e.calendar_id.value == calendar_id)
                .cloned()
                .collect())
        }
    }

    struct FixedResolver {
        start: u64,
        end: u64,
    }

    impl FreeBusyResolver for FixedResolver {
        fn find_slots(
            &self,
            _ctx: &AuthorizedCalendarContext,
            _attendee_emails: &[String],
            _window: FreeBusyWindow,
        ) -> Result<Vec<CalendarSlot>, CalendarApiError> {
            Ok(vec![
                CalendarSlot::new("cal-1".into(), self.start, self.end)
                    .map_err(CalendarApiError::Domain)?,
            ])
        }
    }

    struct EmptyResolver;

    impl FreeBusyResolver for EmptyResolver {
        fn find_slots(
            &self,
            _ctx: &AuthorizedCalendarContext,
            _attendee_emails: &[String],
            _window: FreeBusyWindow,
        ) -> Result<Vec<CalendarSlot>, CalendarApiError> {
            Ok(vec![])
        }
    }

    fn event_req() -> CreateEventRequest {
        CreateEventRequest {
            id: "event-1".into(),
            calendar_id: "cal-1".into(),
            tenant_id: "t".into(),
            title: "Planning".into(),
            start_epoch_seconds: 1_700_000_100,
            end_epoch_seconds: 1_700_000_400,
            attendees: attendees(),
            location: Some("Room A".into()),
            meet_session_id: Some("meet-1".into()),
            recurrence_rule: Some("FREQ=WEEKLY".into()),
            updated_at_epoch_seconds: 1_700_000_050,
        }
    }

    #[test]
    fn unauthorized_context_is_refused_before_any_store_write() {
        let store = FakeStore::default();
        let mut bad = ctx();
        bad.policy_decision_ref = "".into();
        let err = create_event(&store, &bad, event_req()).unwrap_err();
        assert!(matches!(err, CalendarUsecaseError::Api(_)));
        assert!(store.events.borrow().is_empty());
    }

    #[test]
    fn cross_tenant_request_is_denied() {
        let store = FakeStore::default();
        let mut req = event_req();
        req.tenant_id = "other-tenant".into();
        assert_eq!(
            create_event(&store, &ctx(), req),
            Err(CalendarUsecaseError::TenantMismatch)
        );
        assert!(store.events.borrow().is_empty());
    }

    #[test]
    fn create_event_persists_with_valid_attendee_and_recurrence() {
        let store = FakeStore::default();
        let event = create_event(&store, &ctx(), event_req()).unwrap();
        assert_eq!(event.duration_seconds(), 300);
        assert_eq!(store.events.borrow().len(), 1);
    }

    #[test]
    fn empty_attendee_set_is_rejected() {
        let store = FakeStore::default();
        let mut req = event_req();
        req.attendees = vec![];
        assert_eq!(
            create_event(&store, &ctx(), req),
            Err(CalendarUsecaseError::Domain(
                CalendarError::InvalidAttendeeEmail
            ))
        );
    }

    #[test]
    fn invalid_attendee_email_is_rejected() {
        let mut req = event_req();
        req.attendees = vec![AttendeeInput {
            email: "not-an-email".into(),
            role: "required".into(),
        }];
        let store = FakeStore::default();
        assert_eq!(
            create_event(&store, &ctx(), req),
            Err(CalendarUsecaseError::Domain(
                CalendarError::InvalidAttendeeEmail
            ))
        );
    }

    #[test]
    fn unsupported_recurrence_freq_is_rejected() {
        let mut req = event_req();
        req.recurrence_rule = Some("FREQ=FORTNIGHTLY".into());
        let store = FakeStore::default();
        assert_eq!(
            create_event(&store, &ctx(), req),
            Err(CalendarUsecaseError::Domain(
                CalendarError::InvalidRecurrenceRule
            ))
        );
    }

    #[test]
    fn supported_monthly_recurrence_is_accepted() {
        assert!(validate_recurrence(Some("FREQ=MONTHLY;INTERVAL=2".into())).is_ok());
    }

    #[test]
    fn none_recurrence_is_a_one_off_event() {
        assert_eq!(validate_recurrence(None), Ok(None));
    }

    #[test]
    fn invalid_time_range_propagates_from_kernel() {
        let mut req = event_req();
        req.end_epoch_seconds = req.start_epoch_seconds;
        let store = FakeStore::default();
        assert_eq!(
            create_event(&store, &ctx(), req),
            Err(CalendarUsecaseError::Domain(
                CalendarError::InvalidTimeRange
            ))
        );
    }

    #[test]
    fn schedule_event_uses_first_free_slot() {
        let store = FakeStore::default();
        let resolver = FixedResolver {
            start: 1_700_000_100,
            end: 1_700_000_400,
        };
        let req = ScheduleEventRequest {
            id: "event-2".into(),
            calendar_id: "cal-1".into(),
            tenant_id: "t".into(),
            title: "Sync".into(),
            attendees: attendees(),
            location: None,
            meet_session_id: None,
            recurrence_rule: None,
            window: FreeBusyWindow {
                earliest_start_epoch_seconds: 1_700_000_000,
                latest_end_epoch_seconds: 1_700_100_000,
                duration_seconds: 300,
            },
            updated_at_epoch_seconds: 1_700_000_050,
        };
        let event = schedule_event(&store, &resolver, &ctx(), req).unwrap();
        assert_eq!(event.start_epoch_seconds.value, 1_700_000_100);
        assert_eq!(event.duration_seconds(), 300);
    }

    #[test]
    fn schedule_event_fails_when_no_slot_available() {
        let store = FakeStore::default();
        let resolver = EmptyResolver;
        let req = ScheduleEventRequest {
            id: "event-3".into(),
            calendar_id: "cal-1".into(),
            tenant_id: "t".into(),
            title: "Sync".into(),
            attendees: attendees(),
            location: None,
            meet_session_id: None,
            recurrence_rule: None,
            window: FreeBusyWindow {
                earliest_start_epoch_seconds: 1_700_000_000,
                latest_end_epoch_seconds: 1_700_100_000,
                duration_seconds: 300,
            },
            updated_at_epoch_seconds: 1_700_000_050,
        };
        assert_eq!(
            schedule_event(&store, &resolver, &ctx(), req),
            Err(CalendarUsecaseError::NoAvailableSlot)
        );
    }

    #[test]
    fn create_calendar_persists_within_tenant() {
        let store = FakeStore::default();
        let calendar = create_calendar(
            &store,
            &ctx(),
            CreateCalendarRequest {
                id: "cal-1".into(),
                tenant_id: "t".into(),
                region: "region-alpha1".into(),
                name: "Team calendar".into(),
                created_at_epoch_seconds: 1_700_000_000,
            },
        )
        .unwrap();
        assert_eq!(calendar.id.value, "cal-1");
        assert_eq!(store.calendars.borrow().len(), 1);
    }
}
