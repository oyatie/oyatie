//! The `go` statement.
//!
//! 751 launch sites in the measured corpus, of which the census resolves 745 into 7 shapes plus a
//! 23% residue whose shape is in the callee body. Every one of them starts here.
//!
//! A launched goroutine is DETACHED by default, because that is what the source means: `go f()`
//! returns immediately and nothing joins it. The handle exists for the shapes that do join — S3
//! fan-out and S4 parallel sub-task, 111 sites between them — and dropping it does not wait, which
//! is where this differs from `std::thread::scope` and matches the source instead.

use std::thread::JoinHandle;

/// A running goroutine.
///
/// Dropping it DETACHES, matching `go f()` with nothing joining. That is deliberately unlike a
/// scoped thread: 656 of 745 classified launch sites never join, and a handle that blocked on drop
/// would turn every one of them into a synchronous call.
pub struct Goroutine {
    handle: Option<JoinHandle<()>>,
}

impl Goroutine {
    /// Wait for the goroutine to finish. `false` when it ended by panicking.
    ///
    /// The source has no join on a bare `go` statement — this is for the shapes that build one out
    /// of a `WaitGroup` or a result channel, where the join is the point.
    pub fn join(mut self) -> bool {
        match self.handle.take() {
            Some(handle) => handle.join().is_ok(),
            None => true,
        }
    }
}

/// `go f()` — run `f` on its own thread and return immediately.
///
/// The closure is `'static` because a goroutine outlives the statement that launched it and the
/// source gives no lifetime that would bound it. Sharing state with one means sharing it the way
/// the source does: through a channel, or behind a lock.
pub fn spawn<F>(f: F) -> Goroutine
where
    F: FnOnce() + Send + 'static,
{
    Goroutine {
        handle: Some(std::thread::spawn(f)),
    }
}
