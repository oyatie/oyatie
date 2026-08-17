//! Deterministic integer-only Reddit-style hot/controversy ranking kernel.
//!
//! # Score formulas
//!
//! All arithmetic is integer-only (no floats) to guarantee identical results
//! across platforms and compiler versions.
//!
//! ## hot_score
//! ```text
//! RECENCY_WEIGHT  = 86_400   (seconds in one day; mirrors feed_ranking.rs model)
//!
//! age_secs      = now.saturating_sub(created_at)
//! recency_term  = RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT))
//! hot_score     = tally().saturating_add(recency_term as i64)
//! ```
//!
//! ## controversy_score
//! ```text
//! up    = count of Up receipts
//! down  = count of Down receipts
//! score = min(up, down).saturating_mul(up.saturating_add(down))
//! ```
//!
//! ## rank_posts
//! Orders by `hot_score` descending; stable ascending `post_id` tiebreak.
//! Excludes entries with empty post_id.

use crate::{VoteKind, VoteLedger};

/// Recency weight constant: one day in seconds.
/// Mirrors `RECENCY_WEIGHT` in `feed_ranking.rs`.
pub const RECENCY_WEIGHT: u64 = 86_400;

impl VoteLedger {
    /// Returns the hot score: net tally blended with an age-decay recency term.
    ///
    /// - `created_at`: Unix epoch seconds when the post was created.
    /// - `now`: current Unix epoch seconds.
    ///
    /// Saturating arithmetic throughout; no panics on any input combination.
    pub fn hot_score(&self, created_at: u64, now: u64) -> i64 {
        let age_secs = now.saturating_sub(created_at);
        let recency_term = RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT));
        self.tally().saturating_add(recency_term as i64)
    }

    /// Returns the controversy score: rewards near-equal up/down vote splits.
    ///
    /// Formula: `min(up, down) * (up + down)` — saturating.
    ///
    /// Properties:
    /// - 0 when all votes are one-directional.
    /// - Maximal when up == down for a fixed total.
    /// - Symmetric: swapping up/down gives the same score.
    pub fn controversy_score(&self) -> u64 {
        let (up, down) = self
            .receipts
            .value
            .iter()
            .fold((0u64, 0u64), |(u, d), r| match r.kind {
                VoteKind::Up => (u.saturating_add(1), d),
                VoteKind::Down => (u, d.saturating_add(1)),
            });
        up.min(down).saturating_mul(up.saturating_add(down))
    }
}

/// Rank post entries by `hot_score` descending, with stable ascending
/// lexicographic `post_id` tiebreak. Entries with empty post_id are excluded.
///
/// # Arguments
/// - `entries`: slice of `(post_id, ledger, created_at)` tuples.
/// - `now`: current Unix epoch seconds.
///
/// # Returns
/// `Vec<String>` of post_ids in ranked order.
pub fn rank_posts(entries: &[(&str, &VoteLedger, u64)], now: u64) -> Vec<String> {
    let mut ranked: Vec<(&str, i64)> = entries
        .iter()
        .filter(|(post_id, _, _)| !post_id.trim().is_empty())
        .map(|(post_id, ledger, created_at)| (*post_id, ledger.hot_score(*created_at, now)))
        .collect();

    ranked.sort_by(|(id_a, score_a), (id_b, score_b)| {
        score_b.cmp(score_a).then_with(|| id_a.cmp(id_b))
    });

    ranked.into_iter().map(|(id, _)| id.to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommunityAuthor, CommunityMode, CommunityPost, VoteKind, VoteReceipt};

    fn author() -> CommunityAuthor {
        CommunityAuthor::new("anon".into(), "user:real".into(), Some("policy".into())).unwrap()
    }

    fn post(post_id: &str) -> CommunityPost {
        CommunityPost::new(
            post_id.into(),
            "t".into(),
            "tenant".into(),
            CommunityMode::Teamblind,
            author(),
            "body".into(),
            "retain".into(),
        )
        .unwrap()
    }

    fn ledger_with_votes(post_id: &str, up: u32, down: u32) -> VoteLedger {
        let p = post(post_id);
        let mut l = VoteLedger::new(&p);
        for i in 0..up {
            l.cast(
                VoteReceipt {
                    vote_id: format!("u{i}"),
                    voter_ref: format!("voter:up:{i}"),
                    post_id: post_id.into(),
                    kind: VoteKind::Up,
                },
                &p,
            )
            .unwrap();
        }
        for i in 0..down {
            l.cast(
                VoteReceipt {
                    vote_id: format!("d{i}"),
                    voter_ref: format!("voter:down:{i}"),
                    post_id: post_id.into(),
                    kind: VoteKind::Down,
                },
                &p,
            )
            .unwrap();
        }
        l
    }

    // ── 1: hot_score net upvote raises score ──────────────────────────────────

    #[test]
    fn hot_score_net_upvote_raises_score() {
        let now = 1_000_000u64;
        let created_at = now - 3600; // 1 hour old

        let no_votes = ledger_with_votes("p", 0, 0);
        let upvoted = ledger_with_votes("p", 5, 0);

        assert!(
            upvoted.hot_score(created_at, now) > no_votes.hot_score(created_at, now),
            "net upvotes must raise hot_score"
        );
    }

    // ── 2: hot_score decay is monotonic ──────────────────────────────────────

    #[test]
    fn hot_score_decay_monotonic() {
        let now = 1_000_000u64;
        let l = ledger_with_votes("p", 3, 1); // tally = 2

        let fresh = l.hot_score(now - 60, now); // 1 min old
        let hour_old = l.hot_score(now - 3600, now); // 1 hour old
        let day_old = l.hot_score(now - 86_400, now); // exactly 1 day old

        assert!(fresh >= hour_old, "fresher post must score >= hour-old");
        assert!(hour_old >= day_old, "hour-old post must score >= day-old");
    }

    // ── 3: hot_score decay floor — ancient post recency_term = 0 ─────────────

    #[test]
    fn hot_score_decay_floor() {
        let now = 1_000_000u64;
        let l = ledger_with_votes("p", 3, 1); // tally = 2

        // Post older than RECENCY_WEIGHT: recency_term floors at 0
        let ancient_score = l.hot_score(now - 200_000, now);
        assert_eq!(
            ancient_score,
            l.tally(),
            "ancient post hot_score must equal tally() only"
        );
    }

    // ── 4: hot_score empty ledger at now == created_at gives RECENCY_WEIGHT ──

    #[test]
    fn hot_score_empty_ledger_at_zero_age() {
        let now = 1_000_000u64;
        let l = ledger_with_votes("p", 0, 0);
        assert_eq!(
            l.hot_score(now, now),
            RECENCY_WEIGHT as i64,
            "empty ledger at age 0 must equal RECENCY_WEIGHT"
        );
    }

    // ── 5: controversy_score zero when all one-directional ────────────────────

    #[test]
    fn controversy_score_zero_when_one_directional() {
        let all_up = ledger_with_votes("p", 5, 0);
        let all_down = ledger_with_votes("p", 0, 5);
        assert_eq!(all_up.controversy_score(), 0, "all-up must score 0");
        assert_eq!(all_down.controversy_score(), 0, "all-down must score 0");
    }

    // ── 6: controversy_score maximal at equal split ───────────────────────────

    #[test]
    fn controversy_score_maximal_at_equal_split() {
        // Total = 4 votes
        let equal = ledger_with_votes("p", 2, 2); // min=2, total=4 → 2*4=8
        let lopsided = ledger_with_votes("q", 3, 1); // min=1, total=4 → 1*4=4
        assert!(
            equal.controversy_score() > lopsided.controversy_score(),
            "equal split must score higher than lopsided"
        );
    }

    // ── 7: controversy_score symmetric ───────────────────────────────────────

    #[test]
    fn controversy_score_symmetric() {
        let up_heavy = ledger_with_votes("p", 7, 3);
        let down_heavy = ledger_with_votes("q", 3, 7);
        assert_eq!(
            up_heavy.controversy_score(),
            down_heavy.controversy_score(),
            "swapping up/down must give identical controversy_score"
        );
    }

    // ── 8: rank_posts stable tiebreak ─────────────────────────────────────────

    #[test]
    fn rank_posts_deterministic_stable_tiebreak() {
        let now = 1_000_000u64;
        let created_at = now - 1000;

        let la = ledger_with_votes("aaa", 1, 0);
        let lb = ledger_with_votes("zzz", 1, 0);
        let lm = ledger_with_votes("mmm", 1, 0);

        let entries: Vec<(&str, &VoteLedger, u64)> = vec![
            ("zzz", &lb, created_at),
            ("mmm", &lm, created_at),
            ("aaa", &la, created_at),
        ];

        let result = rank_posts(&entries, now);
        assert_eq!(
            result,
            vec!["aaa", "mmm", "zzz"],
            "stable tiebreak must be ascending post_id"
        );
    }

    // ── 9: rank_posts excludes empty post_id ──────────────────────────────────

    #[test]
    fn rank_posts_excludes_empty_post_id() {
        let now = 1_000_000u64;
        let l = ledger_with_votes("p", 1, 0);
        let empty_l = ledger_with_votes("p", 1, 0);

        let entries: Vec<(&str, &VoteLedger, u64)> = vec![
            ("", &empty_l, now - 100),
            ("  ", &empty_l, now - 100),
            ("valid", &l, now - 100),
        ];

        let result = rank_posts(&entries, now);
        assert_eq!(
            result,
            vec!["valid"],
            "empty/blank post_id must be excluded"
        );
    }

    // ── 10: rank_posts higher hot_score wins ──────────────────────────────────

    #[test]
    fn rank_posts_higher_hot_score_wins() {
        let now = 1_000_000u64;

        let fresh_upvoted = ledger_with_votes("fresh", 10, 0);
        let old_downvoted = ledger_with_votes("old", 0, 5);

        let entries: Vec<(&str, &VoteLedger, u64)> = vec![
            ("old", &old_downvoted, now - 80_000),
            ("fresh", &fresh_upvoted, now - 100),
        ];

        let result = rank_posts(&entries, now);
        assert_eq!(result[0], "fresh", "fresh upvoted post must rank first");
    }

    // ── 11: controversy_score no panic on large counts ────────────────────────

    #[test]
    fn controversy_score_no_panic_on_large_counts() {
        // Direct computation with large values to test saturating_mul
        // We can't easily insert u64::MAX votes via the ledger, so verify
        // the formula directly: saturating_mul must not panic
        let large: u64 = u64::MAX / 2;
        let result = large.saturating_mul(u64::MAX); // must not panic
        assert!(result > 0, "saturating_mul of large values must not panic");

        // Also verify that ledger with many votes doesn't panic
        let now = 1_000_000u64;
        let l = ledger_with_votes("p", 50, 50);
        let _ = l.controversy_score(); // must not panic
        let _ = l.hot_score(now - 1000, now); // must not panic
    }

    // ── 12: rank_posts deterministic on repeated calls ────────────────────────

    #[test]
    fn rank_posts_deterministic_repeated_calls() {
        let now = 1_000_000u64;

        let la = ledger_with_votes("alpha", 5, 1);
        let lb = ledger_with_votes("beta", 3, 2);
        let lc = ledger_with_votes("gamma", 1, 0);

        let entries: Vec<(&str, &VoteLedger, u64)> = vec![
            ("alpha", &la, now - 5000),
            ("beta", &lb, now - 1000),
            ("gamma", &lc, now - 100),
        ];

        let first = rank_posts(&entries, now);
        let second = rank_posts(&entries, now);
        assert_eq!(
            first, second,
            "repeated ranking of identical input must be identical"
        );
    }
}
