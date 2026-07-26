//! Condition combinators: all-of and any-of.
//!
//! Mirrors Talos's `conditions.WaitForAll(...)` (in `pkg/conditions/all.go`),
//! which combines several conditions into one that is satisfied only when every
//! member is. The boot sequencer uses this to gate a task on a *set* of
//! dependencies (e.g. "network ready AND machine config present AND time
//! synced"). We additionally provide an any-of combinator, useful for
//! "ready when either path A or path B appears".
//!
//! The combinators borrow their members as `&dyn Condition` so heterogeneous
//! conditions (file / service / network) can be combined without boxing.

use crate::condition::{Condition, Poll};

/// A condition satisfied only when *all* members are satisfied.
///
/// Analogue of `conditions.WaitForAll`. A permanent failure in any member is
/// propagated immediately (the whole group can never succeed).
pub struct All<'c> {
    members: Vec<&'c dyn Condition>,
}

impl<'c> All<'c> {
    /// Combine `members` into an all-of condition.
    pub fn new(members: Vec<&'c dyn Condition>) -> Self {
        All { members }
    }

    /// Number of member conditions.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// True when there are no members (and thus the condition is trivially
    /// satisfied, matching Go's empty `WaitForAll`).
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

impl Condition for All<'_> {
    fn poll(&self) -> Poll {
        let mut pending: Vec<String> = Vec::new();
        for m in &self.members {
            match m.poll() {
                Poll::Ready => {}
                Poll::Pending(s) => pending.push(s),
                Poll::Failed(e) => return Poll::Failed(e),
            }
        }
        if pending.is_empty() {
            Poll::Ready
        } else {
            Poll::Pending(format!("all of: still waiting on {:?}", pending))
        }
    }

    fn describe(&self) -> String {
        let parts: Vec<String> = self.members.iter().map(|m| m.describe()).collect();
        format!("all of {:?}", parts)
    }
}

/// A condition satisfied when *any* member is satisfied.
///
/// If every member fails permanently, the group fails permanently.
pub struct Any<'c> {
    members: Vec<&'c dyn Condition>,
}

impl<'c> Any<'c> {
    /// Combine `members` into an any-of condition.
    pub fn new(members: Vec<&'c dyn Condition>) -> Self {
        Any { members }
    }

    /// Number of member conditions.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// True when there are no members. An empty any-of can never be satisfied.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

impl Condition for Any<'_> {
    fn poll(&self) -> Poll {
        if self.members.is_empty() {
            // No member can ever satisfy it; treat as a hard failure so callers
            // don't spin forever.
            return Poll::Failed(os_kernel::Error::invalid("any-of has no members"));
        }
        let mut pending: Vec<String> = Vec::new();
        let mut all_failed = true;
        for m in &self.members {
            match m.poll() {
                Poll::Ready => return Poll::Ready,
                Poll::Pending(s) => {
                    all_failed = false;
                    pending.push(s);
                }
                Poll::Failed(_) => {}
            }
        }
        if all_failed {
            Poll::Failed(os_kernel::Error::invalid_state(
                "any-of: all members failed",
            ))
        } else {
            Poll::Pending(format!("any of: still waiting on {:?}", pending))
        }
    }

    fn describe(&self) -> String {
        let parts: Vec<String> = self.members.iter().map(|m| m.describe()).collect();
        format!("any of {:?}", parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{None, Poll, Poller, SimClock};
    use os_kernel::Error;

    struct Fixed(Poll, &'static str);
    impl Condition for Fixed {
        fn poll(&self) -> Poll {
            self.0.clone()
        }
        fn describe(&self) -> String {
            self.1.to_string()
        }
    }

    #[test]
    fn all_empty_is_ready() {
        let all = All::new(vec![]);
        assert!(all.is_empty());
        assert_eq!(all.poll(), Poll::Ready);
    }

    #[test]
    fn all_ready_when_every_member_ready() {
        let a = None;
        let b = None;
        let all = All::new(vec![&a, &b]);
        assert_eq!(all.len(), 2);
        assert_eq!(all.poll(), Poll::Ready);
    }

    #[test]
    fn all_pending_when_one_pending() {
        let ready = None;
        let pending = Fixed(Poll::Pending("waiting on x".into()), "x");
        let all = All::new(vec![&ready, &pending]);
        match all.poll() {
            Poll::Pending(s) => assert!(s.contains("waiting on x")),
            other => panic!("expected pending, got {:?}", other),
        }
    }

    #[test]
    fn all_propagates_failure() {
        let ready = None;
        let failed = Fixed(Poll::Failed(Error::not_found("boom")), "f");
        let all = All::new(vec![&ready, &failed]);
        match all.poll() {
            Poll::Failed(e) => assert_eq!(e.kind(), "not_found"),
            other => panic!("expected failed, got {:?}", other),
        }
    }

    #[test]
    fn any_ready_when_one_ready() {
        let pending = Fixed(Poll::Pending("p".into()), "p");
        let ready = None;
        let any = Any::new(vec![&pending, &ready]);
        assert_eq!(any.poll(), Poll::Ready);
    }

    #[test]
    fn any_pending_when_none_ready_but_some_pending() {
        let p1 = Fixed(Poll::Pending("p1".into()), "p1");
        let f = Fixed(Poll::Failed(Error::not_found("x")), "f");
        let any = Any::new(vec![&p1, &f]);
        assert!(matches!(any.poll(), Poll::Pending(_)));
    }

    #[test]
    fn any_fails_when_all_fail() {
        let f1 = Fixed(Poll::Failed(Error::not_found("a")), "f1");
        let f2 = Fixed(Poll::Failed(Error::not_found("b")), "f2");
        let any = Any::new(vec![&f1, &f2]);
        match any.poll() {
            Poll::Failed(e) => assert_eq!(e.kind(), "invalid_state"),
            other => panic!("expected failed, got {:?}", other),
        }
    }

    #[test]
    fn any_empty_fails() {
        let any = Any::new(vec![]);
        assert!(any.is_empty());
        assert!(matches!(any.poll(), Poll::Failed(_)));
    }

    #[test]
    fn all_drives_to_ready_via_poller() {
        let a = None;
        let b = None;
        let all = All::new(vec![&a, &b]);
        let clock = SimClock::new(0);
        let report = all.wait(&clock, Poller::new(3, 5)).unwrap();
        assert_eq!(report.attempts, 1);
        // describe should mention "all of"
        assert!(all.describe().starts_with("all of"));
    }
}
