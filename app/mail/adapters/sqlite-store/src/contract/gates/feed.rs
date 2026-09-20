//! Gate 5 — a stalled or poisoned tenant cannot starve another: tenant B's
//! D = 3C dirty accounts are fully acknowledged within ⌈D/C⌉ + 1 rounds
//! however deep tenant A's backlog is; a round's reads grow with the dirty
//! accounts it serves, not with the dirty set; more perpetually-dirty
//! accounts than C still reach the tail. The retention half of the gate is
//! `feed_retention.rs`.
use super::{append, at};
use crate::contract::Counting;
use crate::contract::suite::{AccountSpec, Fixture, Gate, GateFailure};
use mail_api::{
    BlobStore, ChangeFeed, Consumer, FeedRead, MetadataStore, Precondition, Resume, Store,
};
use mail_kernel::{Command, Error, Retention, RetentionPolicy};

pub(super) const FOUNDRY: Consumer = Consumer::FoundryRecords;
const C: usize = 2;
const D: usize = 3 * C;

pub(super) fn accounts() -> Vec<AccountSpec<'static>> {
    let mut specs = vec![("a", "t", "alice@example.org")];
    specs.extend([
        ("a1", "ta", "a1@x.org"),
        ("a2", "ta", "a2@x.org"),
        ("a3", "ta", "a3@x.org"),
        ("c1", "tc", "c1@x.org"),
    ]);
    specs.extend([
        ("b1", "tb", "b1@x.org"),
        ("b2", "tb", "b2@x.org"),
        ("b3", "tb", "b3@x.org"),
        ("b4", "tb", "b4@x.org"),
        ("b5", "tb", "b5@x.org"),
        ("b6", "tb", "b6@x.org"),
    ]);
    specs
}

/// One committed change on `account`.
pub(super) fn touch<S: MetadataStore + BlobStore>(store: &S, account: &str) -> Result<u64, Error> {
    let revision = store.account(account)?.revision;
    let body = store.append(
        account,
        vec!["inbox".into()],
        b"Subject: t\r\n\r\nx",
        vec![],
        1,
    )?;
    Ok(store
        .execute(account, Precondition::Observed(revision), vec![body])?
        .revision)
}

/// A consumer round: serve up to `per_tenant` accounts per tenant, ack the
/// served accounts `acks` admits at their tail (a stalled account is one
/// the consumer never acknowledges). Returns the accounts served.
fn round<S: ChangeFeed + MetadataStore>(
    store: &S,
    resume: &mut Resume,
    per_tenant: usize,
    acks: impl Fn(&str) -> bool,
) -> Result<Vec<String>, Error> {
    let served = store.dirty(FOUNDRY, resume, 0, per_tenant, 3 * per_tenant)?;
    for dirty in served.iter().filter(|d| acks(&d.account)) {
        if let FeedRead::Changes { page, .. } = store.changes(FOUNDRY, &dirty.account, 100)? {
            store.acknowledge(FOUNDRY, &dirty.account, page.revision, 0, 0)?;
        }
    }
    Ok(served.into_iter().map(|d| d.account).collect())
}

fn isolation<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = fixture.open(&[FOUNDRY], &accounts());
    // Tenant A: a1 poisoned with a deep backlog, a2 and a3 stalled (dirty,
    // never acknowledged). Tenant B: D dirty accounts.
    for _ in 0..20 {
        at(gate, "fixture", touch(&store, "a1"))?;
    }
    at(
        gate,
        "fixture",
        store.poison(FOUNDRY, "a1", "consumer refused"),
    )?;
    for account in ["a2", "a3", "b1", "b2", "b3", "b4", "b5", "b6"] {
        at(gate, "fixture", touch(&store, account))?;
    }
    let mut resume = Resume::default();
    let mut acked = 0;
    let mut rounds = 0;
    while acked < D && rounds < D + 1 {
        rounds += 1;
        let served = at(
            gate,
            "tenant isolation",
            round(&store, &mut resume, C, |a| a.starts_with('b')),
        )?;
        acked += served.iter().filter(|a| a.starts_with('b')).count();
    }
    gate.measure("rounds to acknowledge D = 3C", rounds as u64);
    gate.check(
        "tenant isolation",
        acked == D && rounds <= D.div_ceil(C) + 1,
        format!("{acked} of {D} tenant-B accounts acknowledged in {rounds} rounds"),
    )?;
    // Nothing of tenant A was skipped: a2 and a3 are still dirty, a1 poison.
    let left = at(
        gate,
        "tenant isolation",
        store.dirty(FOUNDRY, &mut Resume::default(), 0, 10, 10),
    )?;
    let mut left: Vec<_> = left.into_iter().map(|d| d.account).collect();
    left.sort();
    gate.check(
        "tenant isolation",
        left == ["a2", "a3"],
        format!("dirty after the rounds: {left:?}"),
    )?;
    let poisoned = at(gate, "tenant isolation", store.poisoned(FOUNDRY))?;
    gate.check(
        "tenant isolation",
        poisoned.len() == 1 && poisoned.first().is_some_and(|p| p.0.account == "a1"),
        "poison was not kept",
    )?;
    // The resume position rotates tenants: with room for one account per
    // round and nothing acknowledged, three rounds serve three tenants.
    at(gate, "fixture", touch(&store, "c1"))?;
    at(gate, "fixture", touch(&store, "b1"))?;
    let mut resume = Resume::default();
    let mut tenants = std::collections::BTreeSet::new();
    for _ in 0..3 {
        for dirty in at(
            gate,
            "resume rotates tenants",
            store.dirty(FOUNDRY, &mut resume, 0, 1, 1),
        )? {
            tenants.insert(dirty.tenant);
        }
    }
    gate.check(
        "resume rotates tenants",
        tenants.len() == 3,
        format!("{tenants:?} served in three rounds of one"),
    )
}

fn reads_and_rotation<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = Counting::new(fixture.open(&[FOUNDRY], &accounts()));
    let all = ["b1", "b2", "b3", "b4", "b5", "b6"];
    for account in &all[..3] {
        at(gate, "fixture", touch(store.inner(), account))?;
    }
    let mut resume = Resume::default();
    store.reset();
    at(
        gate,
        "reads follow the served set",
        store.dirty(FOUNDRY, &mut resume, 0, 3, 3),
    )?;
    let at_three = store.reads();
    for account in &all[3..] {
        at(gate, "fixture", touch(store.inner(), account))?;
    }
    for account in ["a1", "a2", "a3"] {
        at(gate, "fixture", touch(store.inner(), account))?;
    }
    store.reset();
    at(
        gate,
        "reads follow the served set",
        store.dirty(FOUNDRY, &mut Resume::default(), 0, 3, 3),
    )?;
    let at_nine = store.reads();
    gate.measure("dirty() rows at 9 dirty accounts, limit 3", at_nine);
    gate.check(
        "reads follow the served set",
        // Rows are the tenants listed plus the accounts served: one more
        // tenant here, the same three accounts.
        at_nine <= at_three + 1,
        format!("{at_nine} rows with 9 dirty accounts vs {at_three} with 3"),
    )?;
    // 2C perpetually dirty accounts in one tenant: each served account is
    // acknowledged at its tail and touched again; within two rounds every
    // account has been served once.
    let mut resume = Resume::default();
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..2 {
        let served = at(
            gate,
            "tail accounts served",
            round(store.inner(), &mut resume, C, |_| true),
        )?;
        for account in served.iter().filter(|a| a.starts_with('b')) {
            seen.insert(account.clone());
            at(gate, "fixture", touch(store.inner(), account))?;
        }
    }
    gate.check(
        "tail accounts served",
        seen.len() >= 2 * C,
        format!(
            "{} distinct tenant-B accounts served in two rounds of {C}",
            seen.len()
        ),
    )
}

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    isolation(fixture, gate)?;
    reads_and_rotation(fixture, gate)?;
    super::feed_retention::run(fixture, gate)
}
