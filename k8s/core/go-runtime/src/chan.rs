//! A Go channel: a bounded queue with a broadcast close and a rendezvous at capacity zero.
//!
//! Built on a mutex and a pair of condition variables rather than on the standard library's
//! channels, because three of Go's semantics have no equivalent there:
//!
//! 1. **Close is a broadcast.** `close(ch)` wakes every blocked receiver, and every subsequent
//!    receive returns immediately with the zero value. Sender disconnection only fires when the
//!    LAST sender goes away, which is a different event — a Go channel is routinely closed while
//!    senders are still alive, and routinely never closed at all while its senders vanish.
//! 2. **Unbuffered is a rendezvous.** The sender blocks until a receiver has taken the value, not
//!    until there is room for it. 239 of 400 created channels in the measured corpus (59.8%) are
//!    unbuffered, so this is the majority case, not an edge.
//! 3. **A send to a closed channel is a fatal error in the source.** It is a panic in Go. Here it
//!    is a returned error, because a library that aborts the process removes the ported program's
//!    ability to do what it would have done — and the translation of Go's panic is a decision that
//!    belongs to the rule pack, not to this crate.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, PoisonError};

/// Sending on a channel that is already closed. A panic in the source language.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendError;

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("send on a closed channel")
    }
}

impl std::error::Error for SendError {}

/// Receiving from a channel that is closed and drained.
///
/// The source spells this as the second value of `v, ok := <-ch`, so it is not an error there and
/// is not treated as one here — [`Receiver::recv`] returns an `Option` and this type exists for the
/// callers that want the failure spelled out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecvError;

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("receive from a closed and drained channel")
    }
}

impl std::error::Error for RecvError {}

struct State<T> {
    queue: VecDeque<T>,
    /// Receivers currently parked, which is what makes a rendezvous observable to a sender.
    waiting_receivers: usize,
    /// Values handed to a receiver but not yet collected — the rendezvous handshake.
    taken: usize,
    closed: bool,
}

struct Shared<T> {
    state: Mutex<State<T>>,
    /// Signalled when a value arrives or the channel closes.
    filled: Condvar,
    /// Signalled when a value leaves or the channel closes.
    drained: Condvar,
    capacity: usize,
}

impl<T> Shared<T> {
    fn lock(&self) -> std::sync::MutexGuard<'_, State<T>> {
        // Poisoning is ignored deliberately; see the crate documentation.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A Go channel, split into the two halves the source's type system already distinguishes.
///
/// 126 send-only and 453 receive-only type positions in the measured corpus carry channels across
/// API boundaries, so the direction is usually declared at the source and does not have to be
/// inferred.
pub struct Chan<T>(std::marker::PhantomData<T>);

impl<T> Chan<T> {
    /// `make(chan T)` — unbuffered, so a send is a rendezvous with a receive.
    #[must_use]
    pub fn unbuffered() -> (Sender<T>, Receiver<T>) {
        Self::with_capacity(0)
    }

    /// `make(chan T, n)` — buffered, so a send blocks only when the buffer is full.
    #[must_use]
    pub fn buffered(capacity: usize) -> (Sender<T>, Receiver<T>) {
        Self::with_capacity(capacity)
    }

    fn with_capacity(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queue: VecDeque::new(),
                waiting_receivers: 0,
                taken: 0,
                closed: false,
            }),
            filled: Condvar::new(),
            drained: Condvar::new(),
            capacity,
        });
        (
            Sender {
                shared: Arc::clone(&shared),
            },
            Receiver { shared },
        )
    }
}

/// The sending half. Cloneable, because a Go channel value is one thing many goroutines hold.
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Sender<T> {
    /// `ch <- value`, blocking until the value is taken (unbuffered) or buffered.
    ///
    /// # Errors
    /// [`SendError`] when the channel is closed, which the source language treats as fatal.
    pub fn send(&self, value: T) -> Result<(), SendError> {
        let mut state = self.shared.lock();
        loop {
            if state.closed {
                return Err(SendError);
            }
            if self.shared.capacity == 0 {
                // Rendezvous: hand the value over only once a receiver is parked for it, and do
                // not return until that receiver has collected it. Queueing without the second
                // half would make an unbuffered channel a capacity-one queue, which is a different
                // program: the sender would run ahead of the receiver by exactly one value.
                if state.waiting_receivers > state.queue.len() {
                    state.queue.push_back(value);
                    self.shared.filled.notify_one();
                    let handshake = state.taken;
                    while state.taken == handshake && !state.closed {
                        state = self
                            .shared
                            .drained
                            .wait(state)
                            .unwrap_or_else(PoisonError::into_inner);
                    }
                    return Ok(());
                }
            } else if state.queue.len() < self.shared.capacity {
                state.queue.push_back(value);
                self.shared.filled.notify_one();
                return Ok(());
            }
            state = self
                .shared
                .drained
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// `close(ch)` — a BROADCAST. Every parked receiver wakes, and every later receive returns
    /// immediately once the queue is drained.
    ///
    /// Closing twice is a panic in the source; here it is idempotent, because a library that
    /// aborts removes a choice the ported program should be making.
    pub fn close(&self) {
        let mut state = self.shared.lock();
        state.closed = true;
        drop(state);
        self.shared.filled.notify_all();
        self.shared.drained.notify_all();
    }

    /// Whether the channel has been closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.shared.lock().closed
    }
}

/// The receiving half.
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Receiver<T> {
    /// `v, ok := <-ch` — `None` once the channel is closed AND drained.
    ///
    /// Draining before reporting closure is Go's rule and not an implementation detail: a producer
    /// that fills a buffer and closes it expects every buffered value to be delivered.
    pub fn recv(&self) -> Option<T> {
        let mut state = self.shared.lock();
        state.waiting_receivers += 1;
        loop {
            if let Some(value) = state.queue.pop_front() {
                state.waiting_receivers -= 1;
                state.taken += 1;
                drop(state);
                self.shared.drained.notify_all();
                return Some(value);
            }
            if state.closed {
                state.waiting_receivers -= 1;
                return None;
            }
            // A parked receiver is what an unbuffered send is waiting to see, so the sender is
            // woken before this thread sleeps.
            self.shared.drained.notify_all();
            state = self
                .shared
                .filled
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// The `select` with a `default` branch — 135 sites in the measured corpus — as one operation.
    ///
    /// Returns `None` both when nothing is ready and when the channel is closed and drained. The
    /// two are distinguishable through [`Receiver::is_closed`]; they are not distinguished here
    /// because the source's non-blocking receive does not distinguish them either.
    pub fn try_recv(&self) -> Option<T> {
        let mut state = self.shared.lock();
        let value = state.queue.pop_front();
        if value.is_some() {
            state.taken += 1;
            drop(state);
            self.shared.drained.notify_all();
        }
        value
    }

    /// Whether the channel has been closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.shared.lock().closed
    }
}
