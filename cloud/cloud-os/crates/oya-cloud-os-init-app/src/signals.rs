//! PID 1 signal handling model.
//!
//! PID 1 is special to the kernel: signals with default actions are NOT applied
//! to it (the kernel will not auto-kill init), and unhandled `SIGINT`/`SIGTERM`
//! are dropped unless init installs an explicit handler. Talos' init installs
//! handlers for the lifecycle-relevant signals:
//!
//! * `SIGCHLD` — a child exited; run the reaper.
//! * `SIGTERM` / `SIGINT` — begin orderly shutdown (poweroff).
//! * `SIGUSR1` — halt instead of poweroff.
//! * `SIGPWR` — power-failure; emergency poweroff.
//! * `SIGHUP` — reload/no-op (logged).
//!
//! This module maps a received signal to the [`InitAction`] PID 1 should take.
//! It is pure and host-testable; the Linux binary wires the real `sigaction`/
//! `signalfd` to drive [`dispatch`].

/// The standard POSIX signal numbers init cares about (Linux values).
pub mod sig {
    pub const SIGHUP: i32 = 1;
    pub const SIGINT: i32 = 2;
    pub const SIGKILL: i32 = 9;
    pub const SIGUSR1: i32 = 10;
    pub const SIGUSR2: i32 = 12;
    pub const SIGTERM: i32 = 15;
    pub const SIGCHLD: i32 = 17;
    pub const SIGPWR: i32 = 30;
}

/// What PID 1 should do in response to a signal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InitAction {
    /// Reap exited children (SIGCHLD).
    Reap,
    /// Begin orderly shutdown then power off (SIGTERM/SIGINT).
    PowerOff,
    /// Begin orderly shutdown then halt (SIGUSR1).
    Halt,
    /// Reboot the machine (SIGINT under ctrl-alt-del wiring, SIGUSR2).
    Reboot,
    /// Emergency power-off on power failure (SIGPWR).
    EmergencyPowerOff,
    /// Reload configuration / no-op other than logging (SIGHUP).
    Reload,
    /// Signal not handled by init — ignore it (the kernel default for PID 1).
    Ignore,
}

impl InitAction {
    /// True if this action ends the boot lifecycle (process should not return).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            InitAction::PowerOff
                | InitAction::Halt
                | InitAction::Reboot
                | InitAction::EmergencyPowerOff
        )
    }
}

/// Map a received signal number to the init action. Unknown/uncaught signals
/// map to [`InitAction::Ignore`], reflecting PID 1's "no default action" rule.
pub fn dispatch(signum: i32) -> InitAction {
    match signum {
        sig::SIGCHLD => InitAction::Reap,
        sig::SIGTERM | sig::SIGINT => InitAction::PowerOff,
        sig::SIGUSR1 => InitAction::Halt,
        sig::SIGUSR2 => InitAction::Reboot,
        sig::SIGPWR => InitAction::EmergencyPowerOff,
        sig::SIGHUP => InitAction::Reload,
        // SIGKILL/SIGSTOP cannot be caught; everything else is ignored by PID 1.
        _ => InitAction::Ignore,
    }
}

/// The set of signals init explicitly installs handlers for (i.e. the mask it
/// blocks and consumes via signalfd). SIGKILL/SIGSTOP are intentionally absent
/// because they are uncatchable.
pub fn handled_signals() -> &'static [i32] {
    &[
        sig::SIGCHLD,
        sig::SIGTERM,
        sig::SIGINT,
        sig::SIGUSR1,
        sig::SIGUSR2,
        sig::SIGPWR,
        sig::SIGHUP,
    ]
}

/// True if `signum` is one init installs a handler for.
pub fn is_handled(signum: i32) -> bool {
    handled_signals().contains(&signum)
}

/// Drive a sequence of received signals through the dispatcher, stopping at the
/// first terminal action. Returns the actions taken (the last is terminal if
/// any terminal signal was seen). Models init's main signal loop.
pub fn run_signal_loop(signals: &[i32]) -> Vec<InitAction> {
    let mut actions = Vec::new();
    for &s in signals {
        let action = dispatch(s);
        if action == InitAction::Ignore {
            continue;
        }
        let terminal = action.is_terminal();
        actions.push(action);
        if terminal {
            break;
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigchld_reaps() {
        assert_eq!(dispatch(sig::SIGCHLD), InitAction::Reap);
        assert!(!InitAction::Reap.is_terminal());
    }

    #[test]
    fn sigterm_and_sigint_power_off() {
        assert_eq!(dispatch(sig::SIGTERM), InitAction::PowerOff);
        assert_eq!(dispatch(sig::SIGINT), InitAction::PowerOff);
        assert!(InitAction::PowerOff.is_terminal());
    }

    #[test]
    fn sigusr1_halts_and_usr2_reboots() {
        assert_eq!(dispatch(sig::SIGUSR1), InitAction::Halt);
        assert_eq!(dispatch(sig::SIGUSR2), InitAction::Reboot);
        assert!(InitAction::Halt.is_terminal());
        assert!(InitAction::Reboot.is_terminal());
    }

    #[test]
    fn sigpwr_emergency() {
        assert_eq!(dispatch(sig::SIGPWR), InitAction::EmergencyPowerOff);
        assert!(InitAction::EmergencyPowerOff.is_terminal());
    }

    #[test]
    fn sighup_reloads_nonterminal() {
        assert_eq!(dispatch(sig::SIGHUP), InitAction::Reload);
        assert!(!InitAction::Reload.is_terminal());
    }

    #[test]
    fn unknown_signal_ignored() {
        assert_eq!(dispatch(sig::SIGKILL), InitAction::Ignore);
        assert_eq!(dispatch(99), InitAction::Ignore);
        assert!(!InitAction::Ignore.is_terminal());
    }

    #[test]
    fn handled_set_excludes_uncatchable() {
        assert!(is_handled(sig::SIGTERM));
        assert!(is_handled(sig::SIGCHLD));
        assert!(!is_handled(sig::SIGKILL));
        assert_eq!(handled_signals().len(), 7);
    }

    #[test]
    fn loop_reaps_then_powers_off() {
        let actions = run_signal_loop(&[sig::SIGCHLD, sig::SIGCHLD, sig::SIGTERM]);
        assert_eq!(
            actions,
            vec![InitAction::Reap, InitAction::Reap, InitAction::PowerOff]
        );
    }

    #[test]
    fn loop_stops_at_first_terminal() {
        // The SIGHUP after SIGTERM must never be processed.
        let actions = run_signal_loop(&[sig::SIGTERM, sig::SIGHUP, sig::SIGUSR1]);
        assert_eq!(actions, vec![InitAction::PowerOff]);
    }

    #[test]
    fn loop_skips_ignored_signals() {
        let actions = run_signal_loop(&[sig::SIGKILL, sig::SIGCHLD, 99]);
        assert_eq!(actions, vec![InitAction::Reap]);
    }

    #[test]
    fn loop_with_no_terminal_runs_all_nonterminal() {
        let actions = run_signal_loop(&[sig::SIGHUP, sig::SIGCHLD, sig::SIGHUP]);
        assert_eq!(
            actions,
            vec![InitAction::Reload, InitAction::Reap, InitAction::Reload]
        );
    }
}
