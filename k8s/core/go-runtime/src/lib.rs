//! Go concurrency semantics, hand-ported once, for source the port engine translates.
//!
//! # Why this is a library and not a rule corpus
//!
//! `docs/programs/k8s-port/census/concurrency.md` measured the concurrency surface of the pinned
//! Kubernetes tree and its conclusion is the reason this crate exists: **five hand-ported runtime
//! libraries are worth more than any rule in the table above, and they SHRINK the rule corpus
//! rather than adding to it**. The concentration is not a hope — `wait.*` alone is 108 of 400
//! named goroutine launches (27%) and 65% of the S1 background-loop shape, so one audited
//! [`wait::until`] collapses 108 launch sites into 108 calls to something already correct.
//!
//! # Threads, not futures — and why
//!
//! A goroutine is a stackful, preemptively scheduled thread whose blocking operations block. The
//! closest thing the target has is a real thread, and that is what this crate uses.
//!
//! The alternative is an async runtime, and rejecting it is a DECISION with a reason rather than a
//! default. Translating blocking Go into `async` requires colouring every function that
//! transitively blocks — the fixpoint the census names as one of its two largest open questions
//! (§4.1, §7 item 1) and could not close. Colouring wrong is not a performance bug: a `MutexGuard`
//! held across an `.await` is a deadlock the compiler will not always catch, and the census records
//! a proxy population of 55–107 sites where that is at risk.
//!
//! Two smaller facts point the same way. An unbuffered Go channel is a RENDEZVOUS — the sender
//! blocks until a receiver takes the value — and 239 of 400 created channels (59.8%) are
//! unbuffered; `tokio::sync::mpsc::channel(0)` does not exist, while a rendezvous is expressible
//! directly here. And `close(ch)` is a BROADCAST that wakes every receiver, which no
//! sender-disconnection model reproduces on its own.
//!
//! **The cost is stated rather than hidden**: one OS thread per goroutine is heavier than Go's
//! scheduler, and a corpus that launches thousands of short-lived goroutines will feel it. That is
//! a scheduling decision behind this crate's API, so it can be revisited without changing a line of
//! emitted code — which is the whole reason the emitted code calls a library instead of open-coding
//! a runtime.
//!
//! # Lock poisoning is deliberately ignored
//!
//! Go has no poisoning: a goroutine that panics while holding a mutex does not mark the mutex as
//! carrying suspect data. Every lock here recovers the guard out of a `PoisonError` rather than
//! propagating it, because propagating would introduce a failure mode the ported source has no
//! handling for and never had. What the source DOES have — a lock that is never released because
//! its holder died — is preserved by construction, since the guard is still dropped on unwind.
//!
//! # Where this crate lives
//!
//! `governance/capability-registry.json` is closed, and its `base/` admission rule is exact: a
//! crate enters `base/` only if depended on by **at least three** capabilities and strictly below
//! all of them. Exactly one capability depends on this today, so `base/` is not open to it, and
//! `specs/k8s-port/scope.json` names `k8s` as the port's destination capability. It belongs here
//! until a second and a third capability port Go source, at which point the registry's own rule —
//! not a preference — moves it.

#![forbid(unsafe_code)]

mod chan;
mod group;
mod signal;
mod spawn;
pub mod wait;

pub use chan::{Chan, Receiver, RecvError, SendError, Sender};
pub use group::{Once, WaitGroup};
pub use signal::{Signal, Trigger, Waiter};
pub use spawn::{Goroutine, spawn};
