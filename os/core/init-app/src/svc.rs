//! `talos-svc` — a tiny static "node service" binary used to prove that
//! `talos-init` (PID 1) can spawn and supervise a real child process.
//!
//! The initramfs has no `/bin/sh` and no userland to speak of, so the only
//! thing init can `execve(2)` is another binary baked into the same initramfs.
//! This crate therefore ships a second `[[bin]]` target, `svc`, installed at
//! `/usr/bin/svc`. The boot sequencer's `StartServices` phase spawns it; init
//! observes it start, lets it run, then reaps it via `waitpid(2)`.
//!
//! Its job is intentionally trivial: print a few heartbeats to stdout (which
//! init has wired to the serial console) and exit 0. The heartbeat count is
//! kept small so the boot does not stall — init blocks reaping this child
//! before powering the machine off.

use std::io::Write;

/// Number of heartbeats to print before exiting cleanly.
const HEARTBEATS: u32 = 3;

fn main() {
    // stdout is inherited from init, which has dup'd /dev/console onto fds
    // 0/1/2, so these lines reach the serial port.
    for n in 1..=HEARTBEATS {
        println!("talos-svc: heartbeat {n}");
        // Best-effort flush so the line is visible even if we exit promptly.
        let _ = std::io::stdout().flush();
        // A short, real sleep so the heartbeats are observably spaced and init
        // genuinely waits on a live child rather than an instant exit.
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    println!("talos-svc: done, exiting 0");
    let _ = std::io::stdout().flush();
    std::process::exit(0);
}
