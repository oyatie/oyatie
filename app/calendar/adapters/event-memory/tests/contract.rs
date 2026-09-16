#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use calendar_event_api::CalendarStore;
use calendar_event_domain::{CalendarEvent, EventError};
use calendar_event_memory::MemoryCalendar;
use calendar_event_usecase::{list_for_tenant, schedule};

fn event(tenant: &str, id: &str, start: i64, end: i64) -> CalendarEvent {
    CalendarEvent::new(id, tenant, start, end).unwrap()
}

#[test]
fn empty_calendar_lists_no_events() {
    let store = MemoryCalendar::new();
    assert!(store.list_events("tenant:t").unwrap().is_empty());
}

#[test]
fn put_then_list_is_tenant_scoped_and_id_ordered() {
    let mut store = MemoryCalendar::new();
    store.put_event(event("tenant:a", "event:z", 1, 2)).unwrap();
    store.put_event(event("tenant:b", "event:a", 1, 2)).unwrap();
    store.put_event(event("tenant:a", "event:a", 3, 4)).unwrap();
    assert_eq!(
        store.list_events("tenant:a").unwrap(),
        vec![
            event("tenant:a", "event:a", 3, 4),
            event("tenant:a", "event:z", 1, 2),
        ]
    );
}

#[test]
fn list_rejects_unscoped_tenant() {
    let store = MemoryCalendar::new();
    assert_eq!(
        store.list_events("person:u"),
        Err(EventError::MissingTenantScope)
    );
}

#[test]
fn usecase_schedules_through_memory() {
    let mut store = MemoryCalendar::new();
    let scheduled = schedule(&mut store, "event:e", "tenant:t", 10, 20).unwrap();
    assert_eq!(
        list_for_tenant(&store, "tenant:t").unwrap(),
        vec![scheduled]
    );
}

#[test]
fn usecase_does_not_store_an_inverted_range() {
    let mut store = MemoryCalendar::new();
    assert_eq!(
        schedule(&mut store, "event:e", "tenant:t", 20, 10),
        Err(EventError::InvertedRange)
    );
    assert!(store.list_events("tenant:t").unwrap().is_empty());
}
