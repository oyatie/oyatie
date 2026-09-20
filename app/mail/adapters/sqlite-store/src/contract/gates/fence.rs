//! Gate 3 — fencing under contention, at the store: a failure injected at
//! each boundary is answered by a replacement that completes; a commit
//! carrying a stale `Require` is refused; two writers' unconditional
//! updates never fail; conditional writers retry within three attempts at
//! a 20 msg/s trickle (ten times the charter's rate). Written protocol
//! rows: the 1000-message MULTIAPPEND re-attempting `Busy`
//! (`protocol-imap/tests/wire/multiappend_review.rs`) and a session's STORE
//! racing a concurrent expunge (`wire/sequence.rs`). The 20 msg/s and
//! 100-session stress rows are not written (recorded in the S6 PR).
use super::{append, at, open};
use crate::contract::Faulty;
use crate::contract::suite::{Fixture, Gate, GateFailure};
use mail_api::{MetadataStore, Precondition};
use mail_kernel::{Command, Error};
use std::time::Duration;

fn keyword(id: &str, keyword: &str) -> Command {
    Command::Keywords {
        id: id.into(),
        keywords: vec![keyword.into()],
    }
}

/// Commit with `Require(current)`, re-reading and re-attempting on
/// `Conflict`; returns the attempts it took.
fn conditional<S: MetadataStore>(store: &S, command: impl Fn() -> Command) -> Result<u32, Error> {
    for attempt in 1..=10 {
        let revision = store.account("a")?.revision;
        match store.execute("a", Precondition::Require(revision), vec![command()]) {
            Ok(_) => return Ok(attempt),
            Err(Error::Conflict) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(Error::Busy)
}

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = Faulty::new(open(fixture, &[]));
    let mut commands = Vec::new();
    for n in 1..=4 {
        commands.push(at(
            gate,
            "fixture",
            append(store.inner(), "inbox", n, "", "F"),
        )?);
    }
    at(
        gate,
        "fixture",
        store.execute("a", Precondition::Observed(0), commands),
    )?;
    // Boundary faults: each one fails exactly once, persists nothing, and
    // the replacement attempt completes.
    for boundary in ["account", "execute"] {
        store.fail_at(boundary, Error::Unavailable);
        let attempt = match conditional(&store, || keyword("e1", "$seen")) {
            Ok(attempt) => attempt,
            Err(Error::Unavailable) => at(
                gate,
                "replacement completes",
                conditional(&store, || keyword("e1", "$seen")),
            )?,
            Err(error) => return Err(gate.store("replacement completes", error)),
        };
        gate.check(
            "replacement completes",
            attempt <= 2 && !store.armed(),
            format!("{boundary}: {attempt} attempts, armed {}", store.armed()),
        )?;
    }
    store.fail_at("messages", Error::Unavailable);
    let read = store.messages("a", &["e1".into()]);
    gate.check(
        "replacement completes",
        read == Err(Error::Unavailable),
        "the read boundary did not fail",
    )?;
    at(
        gate,
        "replacement completes",
        store.messages("a", &["e1".into()]),
    )?;
    // A stale conditional commit is refused, not re-applied.
    let head = at(gate, "stale commit", store.account("a"))?.revision;
    at(
        gate,
        "stale commit",
        store.execute(
            "a",
            Precondition::Require(head),
            vec![keyword("e2", "$seen")],
        ),
    )?;
    let stale = store.execute(
        "a",
        Precondition::Require(head),
        vec![keyword("e2", "$flagged")],
    );
    gate.check(
        "stale commit",
        stale == Err(Error::Conflict),
        format!("{stale:?}"),
    )?;
    let flags = at(gate, "stale commit", store.messages("a", &["e2".into()]))?;
    gate.check(
        "stale commit",
        flags
            .messages
            .first()
            .is_some_and(|m| !m.keywords.iter().any(|k| k == "$flagged")),
        "the stale commit was applied",
    )?;
    // Two writers, unconditional: zero failures. Conditional at a trickle:
    // every commit lands within three attempts.
    let inner = store.inner();
    let outcome = std::thread::scope(|scope| {
        let writers: Vec<_> = ["$a", "$b"]
            .into_iter()
            .map(|keyword_name| {
                scope.spawn(move || -> Result<(u32, u32), Error> {
                    let mut failures = 0;
                    let mut worst = 0;
                    for n in 0..20 {
                        let revision = inner.account("a")?.revision;
                        if inner
                            .execute(
                                "a",
                                Precondition::Observed(revision),
                                vec![keyword("e3", keyword_name)],
                            )
                            .is_err()
                        {
                            failures += 1;
                        }
                        let id = if n % 2 == 0 { "e4" } else { "e1" };
                        worst = worst.max(conditional(inner, || keyword(id, keyword_name))?);
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    Ok((failures, worst))
                })
            })
            .collect();
        // Join every writer before judging, so a second panic cannot escape
        // the scope as a test panic.
        let joined: Vec<_> = writers.into_iter().map(|w| w.join()).collect();
        joined
            .into_iter()
            .map(|w| w.map_err(|_| Error::Unavailable)?)
            .collect::<Result<Vec<_>, _>>()
    });
    let outcome = at(gate, "two writers", outcome)?;
    let failures: u32 = outcome.iter().map(|o| o.0).sum();
    let worst = outcome.iter().map(|o| o.1).max().unwrap_or(0);
    gate.measure("conditional attempts, worst of 40", u64::from(worst));
    gate.check(
        "two writers",
        failures == 0,
        format!("{failures} unconditional commits failed"),
    )?;
    gate.check(
        "conditional retry",
        worst <= 3,
        format!("a conditional commit took {worst} attempts"),
    )
}
