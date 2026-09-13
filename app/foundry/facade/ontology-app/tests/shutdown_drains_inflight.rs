#![cfg(unix)]

#[path = "process_support/mod.rs"]
mod process;

use std::io::Write;
use std::time::Duration;

use process::{
    Store, body_of, connect, free_address, get, json_of, post_head, read_response, spawn_serving,
    write_body,
};

/// A request whose body is only half sent when the signal arrives must be
/// completed and acknowledged; no new connection may be accepted meanwhile;
/// the process then exits 0; the acknowledged write is durable.
fn drains_then_exits(signal: &str) {
    let store = Store::new(&format!("drain{signal}"));
    let address = free_address();
    let mut process = spawn_serving(&store, address);

    let body = write_body("ent_alpha", "idem_1", "Ada");
    let (first, rest) = body.split_at(20);
    let mut inflight = connect(address).expect("the process accepts a connection");
    inflight
        .write_all(format!("{}{first}", post_head(body.len())).as_bytes())
        .expect("the head and part of the body are writable");
    std::thread::sleep(Duration::from_millis(200));

    process.signal(signal);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        process.is_running(),
        "a process with a request in flight must not exit on the signal"
    );
    assert!(
        connect(address).is_none(),
        "after the signal the listener must refuse new connections"
    );

    inflight
        .write_all(rest.as_bytes())
        .expect("the rest of the body is writable");
    let response = read_response(&mut inflight);
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "the in-flight write is completed, not dropped: {response}"
    );
    assert_eq!(
        json_of(&response)["outcome"],
        "applied",
        "{}",
        body_of(&response)
    );

    let status = process.wait_exit();
    assert!(status.success(), "the drain ends with exit 0: {status}");
    assert!(
        store.stdout_contains("drained"),
        "the process logs that it drained"
    );

    let mut reopened = spawn_serving(&store, free_address());
    let history = get(reopened.address, "/v1/objects/ent_alpha/history");
    assert_eq!(
        json_of(&history).as_array().map(Vec::len),
        Some(1),
        "the write acknowledged during the drain is on the durable log: {history}"
    );
    assert!(reopened.is_running());
}

#[test]
fn sigterm_completes_the_request_in_flight_then_exits_zero() {
    drains_then_exits("-TERM");
}

#[test]
fn sigint_completes_the_request_in_flight_then_exits_zero() {
    drains_then_exits("-INT");
}
