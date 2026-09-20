//! The contract suite on the SQLite tier is green, and the same suite on a
//! deliberately broken store is red on exactly Gates 2, 3, 4, 5 and 6 — the
//! suite notices each violation it exists to notice.
use mail_api::Consumer;
use mail_kernel::Account;
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::{AccountSpec, Broken, run_suite};

fn sqlite(consumers: &[Consumer], accounts: &[AccountSpec<'_>]) -> SqliteStore {
    let db = SqliteStore::open(":memory:")
        .unwrap()
        .with_consumers(consumers);
    for (id, tenant, address) in accounts {
        let owner = address.split('@').next().unwrap();
        db.provision(
            Account::new(id, tenant, owner, address).unwrap(),
            &id.repeat(32),
        )
        .unwrap();
    }
    db
}

#[test]
fn the_sqlite_store_passes_every_gate() {
    let report = run_suite(&sqlite).unwrap_or_else(|failures| {
        panic!(
            "{}",
            failures
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    for measure in &report.measures {
        println!("gate {} {}: {}", measure.gate, measure.case, measure.value);
    }
    assert!(
        report
            .measures
            .iter()
            .any(|m| m.gate == 5 && m.case.starts_with("rounds"))
    );
}

#[test]
fn a_broken_store_fails_gates_two_through_six_and_no_other() {
    let broken = |consumers: &[Consumer], accounts: &[AccountSpec<'_>]| {
        Broken::new(sqlite(consumers, accounts))
    };
    let failures = run_suite(&broken).expect_err("the broken store passed");
    let mut gates: Vec<u8> = failures.iter().map(|f| f.gate).collect();
    gates.sort_unstable();
    gates.dedup();
    for failure in &failures {
        println!("{failure}");
    }
    assert_eq!(gates, [2, 3, 4, 5, 6]);
}
