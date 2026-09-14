#![cfg(unix)]

#[path = "process_support/mod.rs"]
mod process;

use std::io::Write;
use std::time::{Duration, Instant};

use process::{Store, connect, free_address, spawn, spawn_serving};

fn not_a_database(path: &std::path::Path) {
    let mut file = std::fs::File::create(path).expect("the store path is writable");
    for _ in 0..256 {
        file.write_all(b"this is not a database\n")
            .expect("garbage is writable");
    }
}

/// Spawn against a store that is not a database and watch the whole life
/// of the process: it must exit 1, say why, and never answer on its port.
fn refuses_before_serving(store: &Store) {
    let address = free_address();
    let mut process = spawn(store, address);
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        assert!(
            connect(address).is_none(),
            "a process refusing boot must never accept a connection"
        );
        if let Some(status) = process.child.try_wait().expect("the child can be polled") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "the process neither served nor exited"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(status.code(), Some(1), "a boot refusal exits 1: {status}");
    assert!(
        store.stdout_contains("boot refused"),
        "the refusal is logged as a boot refusal"
    );
    assert!(
        connect(address).is_none(),
        "the port stays closed after the refusal"
    );
}

#[test]
fn an_action_log_that_is_not_a_database_refuses_boot_before_the_listener_binds() {
    let store = Store::new("corrupt-action");
    not_a_database(&store.action);
    refuses_before_serving(&store);
    assert!(
        store.stdout_contains("action log"),
        "the refusal names the action log"
    );
}

#[test]
fn a_denial_trail_that_is_not_a_database_refuses_boot_before_the_listener_binds() {
    let store = Store::new("corrupt-denial");
    not_a_database(&store.denial);
    refuses_before_serving(&store);
    assert!(
        store.stdout_contains("denial"),
        "the refusal names the denial trail"
    );
}

#[test]
fn a_projection_store_that_is_not_a_database_refuses_boot_before_the_listener_binds() {
    let store = Store::new("corrupt-projection");
    not_a_database(&store.projection);
    refuses_before_serving(&store);
    assert!(
        store.stdout_contains("projection store"),
        "the refusal names the projection store"
    );
}

/// The control for the three refusals above. `Store::new` leaves all three
/// paths absent and the arms differ from this test only by the garbage they
/// then write, so a spawn that writes none must serve and log no refusal.
#[test]
fn a_spawn_that_garbages_no_store_serves_and_refuses_nothing() {
    let store = Store::new("nothing-garbaged");
    let _process = spawn_serving(&store, free_address());
    assert!(
        !store.stdout_contains("boot refused"),
        "a boot with nothing garbaged refuses nothing"
    );
}
