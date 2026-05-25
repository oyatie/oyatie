//! Speculative merge-queue algorithm (ADR-0360 O6, implements ADR-0111
//! projected-state). Prototype of the always-green-trunk ("Not Rocket Science
//! Rule") speculative queue: test candidates against a *projected* trunk
//! (trunk + all earlier candidates assumed to pass) in parallel, land the green
//! prefix, eject the first culprit, re-project the rest, and adapt the window
//! (grow on success, halve on failure — Zuul's TCP-window analogy).
//!
//! This is the pure, deterministic algorithm with unit tests; the production
//! queue (webhook-driven, CI-backed) wraps this decision core. The test oracle
//! is injected so the logic is verifiable without real CI.

/// Outcome of one speculative round over the queue head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RoundResult {
    /// Candidates that land on trunk this round (the green projected prefix).
    pub landed: Vec<String>,
    /// The first culprit ejected this round (if any).
    pub ejected: Option<String>,
    /// Candidates remaining in the queue for the next round (culprit removed),
    /// re-projected against the new trunk tip.
    pub requeued: Vec<String>,
    /// Window size for the next round (grows on clean success, halves on a
    /// failure), clamped to >= 1.
    pub next_window: usize,
}

/// Run one speculative round.
///
/// `queue` is the ordered candidate list. `window` bounds how many head
/// candidates are speculated this round. `passes(projected)` reports whether the
/// projected stack `trunk + queue[..=i]` is green (the candidate ids in order).
///
/// Because a culprit poisons every projected stack that includes it, the first
/// failing index IS the culprit: everything before it lands, the culprit is
/// ejected, and everything after re-queues for re-projection.
pub(crate) fn speculative_round<F>(queue: &[String], window: usize, passes: F) -> RoundResult
where
    F: Fn(&[String]) -> bool,
{
    let window = window.max(1);
    let batch_len = window.min(queue.len());

    let mut landed: Vec<String> = Vec::new();
    let mut culprit_index: Option<usize> = None;
    for i in 0..batch_len {
        let projected = &queue[..=i];
        if passes(projected) {
            // green up to i; provisionally landed (confirmed once no earlier fail)
        } else {
            culprit_index = Some(i);
            break;
        }
    }

    match culprit_index {
        None => {
            // Whole speculated batch is green: land it; grow the window.
            landed.extend_from_slice(&queue[..batch_len]);
            let requeued = queue[batch_len..].to_vec();
            RoundResult {
                landed,
                ejected: None,
                requeued,
                next_window: window + 1,
            }
        }
        Some(j) => {
            // queue[..j] are green and land; queue[j] is the culprit; the rest
            // re-queue (to be re-projected without the culprit). Halve window.
            landed.extend_from_slice(&queue[..j]);
            let ejected = Some(queue[j].clone());
            let requeued = queue[j + 1..].to_vec();
            RoundResult {
                landed,
                ejected,
                requeued,
                next_window: (window / 2).max(1),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn all_green_lands_whole_batch_and_grows_window() {
        let queue = ids(&["A", "B", "C"]);
        let r = speculative_round(&queue, 3, |_| true);
        assert_eq!(r.landed, ids(&["A", "B", "C"]));
        assert_eq!(r.ejected, None);
        assert_eq!(r.requeued, Vec::<String>::new());
        assert_eq!(r.next_window, 4); // grew
    }

    #[test]
    fn middle_culprit_lands_prefix_ejects_and_requeues_rest() {
        // B is the culprit: any projected stack containing B fails.
        let queue = ids(&["A", "B", "C", "D"]);
        let r = speculative_round(&queue, 4, |proj| !proj.iter().any(|c| c == "B"));
        assert_eq!(r.landed, ids(&["A"])); // A lands
        assert_eq!(r.ejected, Some("B".to_string())); // B ejected
        assert_eq!(r.requeued, ids(&["C", "D"])); // re-projected next round
        assert_eq!(r.next_window, 2); // halved from 4
    }

    #[test]
    fn first_candidate_fails_lands_nothing() {
        let queue = ids(&["A", "B"]);
        let r = speculative_round(&queue, 2, |proj| !proj.iter().any(|c| c == "A"));
        assert_eq!(r.landed, Vec::<String>::new());
        assert_eq!(r.ejected, Some("A".to_string()));
        assert_eq!(r.requeued, ids(&["B"]));
        assert_eq!(r.next_window, 1); // halved + clamped
    }

    #[test]
    fn window_bounds_the_batch() {
        let queue = ids(&["A", "B", "C", "D"]);
        let r = speculative_round(&queue, 2, |_| true);
        assert_eq!(r.landed, ids(&["A", "B"])); // only window=2 speculated
        assert_eq!(r.requeued, ids(&["C", "D"]));
        assert_eq!(r.next_window, 3);
    }

    #[test]
    fn empty_queue_is_a_noop() {
        let r = speculative_round(&[], 4, |_| true);
        assert_eq!(r.landed, Vec::<String>::new());
        assert_eq!(r.ejected, None);
        assert_eq!(r.requeued, Vec::<String>::new());
    }

    #[test]
    fn draining_a_queue_with_one_culprit_converges_to_always_green_trunk() {
        // Simulate rounds until empty; B is permanently bad. Trunk only ever
        // gains green changes (NRSR invariant), B never lands.
        let mut queue = ids(&["A", "B", "C"]);
        let mut window = 2;
        let mut trunk: Vec<String> = Vec::new();
        let mut rounds = 0;
        while !queue.is_empty() && rounds < 10 {
            let r = speculative_round(&queue, window, |proj| !proj.iter().any(|c| c == "B"));
            trunk.extend(r.landed);
            queue = r.requeued;
            window = r.next_window;
            rounds += 1;
        }
        assert_eq!(trunk, ids(&["A", "C"])); // both good changes landed
        assert!(!trunk.iter().any(|c| c == "B")); // culprit never on trunk
    }
}
