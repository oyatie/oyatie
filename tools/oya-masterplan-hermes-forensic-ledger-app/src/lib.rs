//! Owned-Rust forensic ledger builder for masterplan v2 Hermes done-card
//! completion claims.
//!
//! Seed contract (masterplan v2 consolidation, Sub-AC 3): every Hermes
//! done-card completion claim imports as `claimed-done-unverified`; evidence
//! references are attached where they already exist (in-repo evidence
//! artifacts referencing the card id, plus merged-PR/gate-run URLs recorded in
//! the card's own result text) and the remainder stay flagged; no claim may
//! carry a verified status without an attached evidence link. The board is
//! ingested read-only through the owned pure-Rust SQLite reader in
//! [`sqlite`] — no shell/python board extraction.
//!
//! This crate is the single writer of `masterplan_v2.hermes_done_card_imports`
//! and `masterplan_v2.hermes_done_card_import_summary` in
//! `/specs/masterplan.json`, and of the forensic ledger evidence artifact. It
//! carries no plan or merge authority.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod json;
pub mod ledger;
pub mod sha256;
pub mod sqlite;
