//! Inline thread merges are bounded per commit: the largest thread survives
//! and at most `MERGE_LIMIT` members are re-threaded across all appends of
//! one batch, so a commit's history stays pageable.
use mail_api::{MetadataStore, Precondition};
use mail_kernel::{Account, Command};
use mail_sqlite_store::SqliteStore;

#[test]
fn a_merge_keeps_the_larger_thread_and_leaves_a_side_beyond_the_limit_separate() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "a", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let message = |id: &str, refs: &str| Command::Append {
        mailboxes: vec!["inbox".into()],
        keywords: vec![],
        received_at: 1,
        raw: format!("Message-ID: <{id}>\r\nReferences: {refs}\r\nSubject: T\r\n\r\nbody")
            .into_bytes(),
    };
    // Thread A: 1001 members; thread B: 1001 members; thread C: one member.
    let mut commands = vec![];
    for root in ["a", "b"] {
        commands.push(message(&format!("{root}0@t"), ""));
        commands
            .extend((1..=1000).map(|n| message(&format!("{root}{n}@t"), &format!("<{root}0@t>"))));
    }
    commands.push(message("c0@t", ""));
    db.execute("a", Precondition::Require(0), commands).unwrap();
    let before = db.account("a").unwrap();
    let thread_of = |acct: &Account, id: &str| {
        acct.messages
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .thread_id()
            .to_owned()
    };
    let (a, b, c) = (
        thread_of(&before, "e1"),
        thread_of(&before, "e1002"),
        thread_of(&before, "e2003"),
    );
    assert!(a != b && b != c);
    // A bridge referencing all three: C (1 member) merges into the largest,
    // B (1001 members) exceeds the per-commit merge limit and stays separate.
    db.execute(
        "a",
        Precondition::Require(before.revision),
        vec![message("bridge@t", "<a0@t> <b0@t> <c0@t>")],
    )
    .unwrap();
    let after = db.account("a").unwrap();
    let survivor = thread_of(&after, "e2004");
    assert_eq!(survivor, a);
    assert_eq!(thread_of(&after, "e2003"), a, "the small side merged");
    assert_eq!(thread_of(&after, "e1002"), b, "the large side stayed");
    assert_eq!(
        after.messages.iter().filter(|m| m.thread_id() == b).count(),
        1001
    );
    let page = db.history("a", before.revision, 10_000).unwrap();
    assert!(
        !page.has_more && !page.rows.is_empty(),
        "the commit is one pageable revision"
    );
}

#[test]
fn the_merge_budget_spans_the_whole_commit_not_each_append() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "a", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let message = |id: &str, refs: &str| Command::Append {
        mailboxes: vec!["inbox".into()],
        keywords: vec![],
        received_at: 1,
        raw: format!("Message-ID: <{id}>\r\nReferences: {refs}\r\nSubject: T\r\n\r\nbody")
            .into_bytes(),
    };
    // Survivor B: 601 members; sides A1 and A2: 600 each. Two bridges in one
    // batch may merge only one side within the 1000-member commit budget.
    for (root, n) in [("b", 601), ("a1", 600), ("a2", 600)] {
        let commands = (0..n)
            .map(|i| {
                let refs = if i == 0 {
                    String::new()
                } else {
                    format!("<{root}0@t>")
                };
                message(&format!("{root}{i}@t"), &refs)
            })
            .collect();
        db.execute("a", Precondition::Observed(0), commands)
            .unwrap();
    }
    let before = db.account("a").unwrap();
    let thread_of = |acct: &Account, id: &str| {
        acct.messages
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .thread_id()
            .to_owned()
    };
    let (b, a1, a2) = (
        thread_of(&before, "e1"),
        thread_of(&before, "e602"),
        thread_of(&before, "e1202"),
    );
    db.execute(
        "a",
        Precondition::Require(before.revision),
        vec![
            message("x@t", "<b0@t> <a10@t>"),
            message("y@t", "<b0@t> <a20@t>"),
        ],
    )
    .unwrap();
    let after = db.account("a").unwrap();
    let members = |t: &str| after.messages.iter().filter(|m| m.thread_id() == t).count();
    assert_eq!(
        members(&b),
        601 + 600 + 2,
        "one side and both bridges joined the survivor"
    );
    assert!(
        members(&a1) == 600 || members(&a2) == 600,
        "the other side stayed separate"
    );
    assert!(db.history("a", before.revision, 10_000).unwrap().rows.len() <= 10_000);
}
