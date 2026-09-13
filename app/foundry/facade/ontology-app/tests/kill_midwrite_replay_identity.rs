#![cfg(unix)]

#[path = "process_support/mod.rs"]
mod process;

use std::collections::BTreeMap;
use std::time::Duration;

use process::{Store, body_of, free_address, get, json_of, post_action, spawn_serving, write_body};

const OBJECTS: usize = 5;

/// Writes acknowledged before the process was killed, keyed by the ordinal
/// the process returned for each, plus the write that was in flight when
/// the connection died: the kill may have landed after its commit and
/// before its acknowledgement.
struct Acknowledged {
    by_ordinal: BTreeMap<u64, (String, String)>,
    in_flight: Option<(String, String)>,
}

/// Keep writing until the connection dies under us; every 200 is recorded
/// with the ordinal the process committed to before answering.
fn write_until_killed(address: std::net::SocketAddr) -> Acknowledged {
    let mut by_ordinal = BTreeMap::new();
    let mut sequence = 0u64;
    loop {
        sequence += 1;
        let object_ref = format!("ent_{}", sequence % OBJECTS as u64);
        let key = format!("idem_{sequence}");
        let name = format!("v{sequence}");
        let in_flight = Some((object_ref.clone(), name.clone()));
        let Some(response) = post_action(address, &write_body(&object_ref, &key, &name)) else {
            return Acknowledged {
                by_ordinal,
                in_flight,
            };
        };
        // A status line without a parseable body is a response the kill cut
        // off, which is the in-flight case, not an acknowledgement.
        let ordinal = serde_json::from_str::<serde_json::Value>(body_of(&response))
            .ok()
            .filter(|_| response.starts_with("HTTP/1.1 200"))
            .and_then(|body| body["ordinal"].as_u64());
        let Some(ordinal) = ordinal else {
            return Acknowledged {
                by_ordinal,
                in_flight,
            };
        };
        by_ordinal.insert(ordinal, (object_ref, name));
    }
}

#[test]
fn every_acknowledged_write_survives_a_sigkill_and_refolds_without_poison() {
    let store = Store::new("sigkill");
    let mut process = spawn_serving(&store, free_address());
    let address = process.address;

    let writer = std::thread::spawn(move || write_until_killed(address));
    std::thread::sleep(Duration::from_millis(400));
    // SIGKILL: no handler runs, no drain, whatever was mid-flight is lost.
    process.child.kill().expect("SIGKILL is deliverable");
    let status = process.wait_exit();
    assert!(
        !status.success(),
        "a killed process does not exit cleanly: {status}"
    );
    let acked = writer.join().expect("the writer thread ends");
    let (&max_acked, _) = acked
        .by_ordinal
        .last_key_value()
        .expect("at least one write was acknowledged before the kill");

    let mut reopened = spawn_serving(&store, free_address());
    let audit = get(reopened.address, "/v1/audit");
    let rows = json_of(&audit);
    let rows = rows.as_array().expect("the audit view is a list");
    let applied: BTreeMap<u64, &str> = rows
        .iter()
        .map(|row| {
            (
                row["ordinal"].as_u64().expect("an ordinal"),
                row["disposition"].as_str().expect("a disposition"),
            )
        })
        .collect();
    for (ordinal, (object_ref, _)) in &acked.by_ordinal {
        assert_eq!(
            applied.get(ordinal),
            Some(&"applied"),
            "acknowledged ordinal {ordinal} ({object_ref}) must replay as applied: {audit}"
        );
    }
    let head = applied.keys().last().copied().unwrap_or(0);
    assert!(
        head == max_acked || head == max_acked + 1,
        "the log may hold at most one write the kill cut off before its acknowledgement \
         (head {head}, last acknowledged {max_acked})"
    );
    assert!(
        applied
            .values()
            .all(|disposition| *disposition == "applied"),
        "a kill leaves no poisoned entry behind: {audit}"
    );

    let mut latest: BTreeMap<&str, (u64, &str)> = BTreeMap::new();
    for (ordinal, (object_ref, name)) in &acked.by_ordinal {
        latest.insert(object_ref, (*ordinal, name));
    }
    if head == max_acked + 1 {
        // The write cut off between its commit and its acknowledgement is
        // on the log at the head, so its object reads back at that value.
        let (object_ref, name) = acked
            .in_flight
            .as_ref()
            .expect("a head past the last acknowledgement means a write was in flight");
        latest.insert(object_ref, (head, name));
    }
    for (object_ref, (ordinal, name)) in latest {
        // The projection is the refold, not the log: the object must read
        // back at its last committed value.
        let pinned = get(
            reopened.address,
            &format!("/v1/objects/{object_ref}?revision=1"),
        );
        assert!(pinned.starts_with("HTTP/1.1 200"), "{pinned}");
        assert_eq!(
            json_of(&pinned)["properties"]["name"]["value"],
            name,
            "{object_ref} refolds to its last committed write: {pinned}"
        );
        let history = get(
            reopened.address,
            &format!("/v1/objects/{object_ref}/history"),
        );
        let ordinals: Vec<u64> = json_of(&history)
            .as_array()
            .expect("history is a list")
            .iter()
            .map(|row| row["ordinal"].as_u64().expect("an ordinal"))
            .collect();
        assert!(
            ordinals.contains(&ordinal),
            "{object_ref}'s last committed write (ordinal {ordinal}) is in its history: {history}"
        );
    }

    let status = json_of(&get(reopened.address, "/statusz"));
    assert_eq!(status["poisoned_entries"], 0, "{status}");
    assert_eq!(status["projection_lag"], 0, "{status}");
    assert!(reopened.is_running());
}
