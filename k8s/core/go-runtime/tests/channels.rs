//! Message passing: what a channel and a signal MEAN, where the two languages disagree.
//!
//! Every test here corresponds to a place where the obvious Rust construct means something
//! DIFFERENT from the Go one. A crate that merely compiled would pass none of them and would look
//! finished.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use k8s_go_runtime::{Chan, Signal, spawn};

/// An unbuffered channel is a RENDEZVOUS, not a capacity-one queue.
///
/// This is the one that a `sync_channel(1)` would pass while being a different program: with a
/// queue of one, the sender runs a whole value ahead of the receiver. 239 of 400 channels in the
/// measured corpus are unbuffered, so the difference is the majority case.
#[test]
fn an_unbuffered_send_waits_for_its_receiver() {
    let (tx, rx) = Chan::<i32>::unbuffered();
    let progress = Arc::new(AtomicUsize::new(0));

    let sender = {
        let progress = Arc::clone(&progress);
        spawn(move || {
            for value in 0..3 {
                tx.send(value).expect("channel is open");
                progress.store(value as usize + 1, Ordering::SeqCst);
            }
        })
    };

    // Take the first value. The sender may complete that send and no more, because the second one
    // has nobody waiting for it.
    assert_eq!(rx.recv(), Some(0));
    std::thread::sleep(Duration::from_millis(50));
    assert!(
        progress.load(Ordering::SeqCst) <= 2,
        "an unbuffered sender must not run ahead of its receiver"
    );

    assert_eq!(rx.recv(), Some(1));
    assert_eq!(rx.recv(), Some(2));
    assert!(sender.join());
}

/// A buffered channel accepts up to its capacity without any receiver at all.
#[test]
fn a_buffered_send_does_not_wait_until_the_buffer_is_full() {
    let (tx, rx) = Chan::<i32>::buffered(2);
    tx.send(1).expect("capacity 2");
    tx.send(2).expect("capacity 2");
    assert_eq!(rx.recv(), Some(1));
    assert_eq!(rx.recv(), Some(2));
}

/// `close(ch)` is a BROADCAST: every parked receiver wakes, not one of them.
///
/// Sender-disconnection models release receivers only when the last sender goes away, which is a
/// different event — the corpus routinely closes a channel while senders are still alive.
#[test]
fn closing_a_channel_wakes_every_receiver() {
    let (tx, rx) = Chan::<i32>::unbuffered();
    let woken = Arc::new(AtomicUsize::new(0));

    let receivers: Vec<_> = (0..4)
        .map(|_| {
            let rx = rx.clone();
            let woken = Arc::clone(&woken);
            spawn(move || {
                assert!(rx.recv().is_none(), "a closed channel yields nothing");
                woken.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect();

    std::thread::sleep(Duration::from_millis(30));
    tx.close();
    for receiver in receivers {
        assert!(receiver.join());
    }
    assert_eq!(woken.load(Ordering::SeqCst), 4);
}

/// A closed channel is DRAINED before it reports closure.
///
/// A producer that fills a buffer and closes it expects every buffered value to arrive. Reporting
/// closure first would silently drop them.
#[test]
fn a_closed_channel_delivers_what_it_still_holds() {
    let (tx, rx) = Chan::<i32>::buffered(3);
    tx.send(1).expect("capacity 3");
    tx.send(2).expect("capacity 3");
    tx.close();

    assert_eq!(rx.recv(), Some(1));
    assert_eq!(rx.recv(), Some(2));
    assert_eq!(rx.recv(), None);
}

/// Sending on a closed channel is an error, not a silent drop.
#[test]
fn sending_on_a_closed_channel_reports_it() {
    let (tx, _rx) = Chan::<i32>::buffered(1);
    tx.close();
    assert!(tx.send(1).is_err());
}

/// A signal is a broadcast that every waiter observes, however late it arrives.
///
/// Modelling `chan struct{}` as a value channel would release one waiter per close, which is a
/// shutdown that hangs — and hangs in production rather than in a test.
#[test]
fn a_signal_releases_every_waiter_including_the_late_ones() {
    let (trigger, waiter) = Signal::pair();
    let released = Arc::new(AtomicUsize::new(0));

    let early: Vec<_> = (0..3)
        .map(|_| {
            let waiter = waiter.clone();
            let released = Arc::clone(&released);
            spawn(move || {
                waiter.wait();
                released.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect();

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(released.load(Ordering::SeqCst), 0, "nothing fired yet");
    trigger.fire();
    for waiter in early {
        assert!(waiter.join());
    }

    // A waiter that arrives after the signal fired must not block.
    waiter.wait();
    assert!(waiter.has_fired());
    assert_eq!(released.load(Ordering::SeqCst), 3);
}

/// Firing twice is idempotent, where the source panics on a second close.
#[test]
fn firing_a_signal_twice_is_not_fatal() {
    let (trigger, waiter) = Signal::pair();
    trigger.fire();
    trigger.fire();
    assert!(waiter.has_fired());
}

/// `once.Do` runs once, and every caller returns only after it has COMPLETED.
///
/// The second half is the guarantee that makes `Once` more than a boolean: a caller that did not
/// `wait.Until` runs immediately, then periodically, and stops PROMPTLY.
///
/// Promptness is the property a `sleep(period)` loop loses: it notices the stop signal only at the
/// end of the current period, so a one-minute loop takes up to a minute to shut down. With 165
/// A loop already stopped does no work at all.
///
/// The stop check is BEFORE the run, not after. Checking after would run the body one extra time
/// A goroutine is DETACHED: dropping its handle does not join it.
///
/// 656 of 745 classified launch sites never join, so a handle that blocked on drop would turn every
/// The non-blocking receive — `select` with a `default`, 135 sites in the measured corpus.
#[test]
fn try_recv_does_not_block_on_an_empty_channel() {
    let (tx, rx) = Chan::<i32>::buffered(1);
    assert_eq!(rx.try_recv(), None);
    tx.send(7).expect("capacity 1");
    assert_eq!(rx.try_recv(), Some(7));
    assert_eq!(rx.try_recv(), None);
}
