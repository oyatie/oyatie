//! Live smoke against a FoundationDB reachable through `FDB_CLUSTER_FILE`.
//! Runs only in the live lane (`.github/scripts/live-fdb.sh`): every test is
//! `#[ignore]`, named `live_*`, and this file compiles only with `fdb`.
#![cfg(feature = "fdb")]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mail_store_fdb::ffi::{
    API_VERSION, Database, KeySelector, RangeOptions, Transaction, block_on, network,
};
use mail_store_fdb::key::{Key, KeyRange, read_u64};
use mail_store_fdb::{FdbConfig, FdbStore};

fn cluster_file() -> PathBuf {
    let path =
        std::env::var_os("FDB_CLUSTER_FILE").expect("FDB_CLUSTER_FILE names the live cluster");
    PathBuf::from(path)
}

fn open() -> FdbStore {
    let config = FdbConfig {
        cluster_file: cluster_file(),
        cell: "smoke".into(),
    };
    FdbStore::open(config).expect("store opens with the fdb feature")
}

/// Unique per process so concurrent lanes never share keys.
fn prefix() -> Key {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    Key::new(b"oyatie/mail/smoke/")
        .u64(u64::from(std::process::id()))
        .u64(nanos as u64)
}

fn commit(tx: &Transaction) -> i64 {
    block_on(tx.commit()).expect("commit");
    tx.committed_version().expect("committed version")
}

fn read_all(db: &Database, range: &KeyRange) -> Vec<(Vec<u8>, Vec<u8>)> {
    let tx = db.create_transaction().expect("transaction");
    let page = block_on(tx.get_range(
        KeySelector::first_greater_or_equal(&range.begin),
        KeySelector::first_greater_or_equal(&range.end),
        RangeOptions::default(),
    ))
    .expect("range read")
    .key_values()
    .expect("key-value array");
    assert!(!page.more, "smoke ranges fit one page");
    page.pairs
}

#[test]
#[ignore]
fn live_select_api_version_and_connect() {
    network::start().expect("API version 730 selected and network thread running");
    assert_eq!(API_VERSION, 730);
    let store = open();
    let tx = store.database().create_transaction().expect("transaction");
    let version = block_on(tx.get_read_version())
        .expect("read version")
        .int64()
        .expect("int64");
    assert!(version > 0, "a configured database hands out read versions");
}

#[test]
#[ignore]
fn live_set_get_clear_round_trip() {
    let store = open();
    let db = store.database();
    let key = prefix().tag(1).into_bytes();

    let tx = db.create_transaction().expect("transaction");
    tx.set(&key, b"hello");
    let written_at = commit(&tx);
    assert!(written_at > 0);

    let tx = db.create_transaction().expect("transaction");
    let got = block_on(tx.get(&key, false))
        .expect("get")
        .value()
        .expect("value");
    assert_eq!(got.as_deref(), Some(&b"hello"[..]));
    tx.clear(&key);
    assert!(commit(&tx) > written_at);

    let tx = db.create_transaction().expect("transaction");
    let got = block_on(tx.get(&key, false))
        .expect("get")
        .value()
        .expect("value");
    assert_eq!(got, None);
}

#[test]
#[ignore]
fn live_range_read_returns_keys_in_order() {
    let store = open();
    let db = store.database();
    let space = prefix().tag(2);
    let range = space.range();

    let tx = db.create_transaction().expect("transaction");
    for n in [3u64, 1, 2] {
        tx.set(space.clone().u64(n).as_bytes(), &n.to_be_bytes());
    }
    commit(&tx);

    let pairs = read_all(db, &range);
    let numbers: Vec<u64> = pairs
        .iter()
        .map(|(key, _)| read_u64(key, key.len() - 8).expect("u64 suffix"))
        .collect();
    assert_eq!(numbers, [1, 2, 3]);
    for (key, value) in &pairs {
        assert_eq!(&key[key.len() - 8..], value.as_slice());
    }

    let tx = db.create_transaction().expect("transaction");
    tx.clear_range(&range.begin, &range.end);
    commit(&tx);
    assert!(read_all(db, &range).is_empty());
}

#[test]
#[ignore]
fn live_fdb_build_has_no_static_refusal() {
    assert_eq!(FdbStore::refusal(), None);
}
