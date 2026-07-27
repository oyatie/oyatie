//! containerd tasks: the running instantiation of a container.
//!
//! A container is static metadata + an OCI spec; a *task* is the actual running
//! process tree created from it. This mirrors containerd's `Task` interface
//! (`Start`, `Kill`, `Wait`, `Delete`) and its process state machine.

use os_kernel::error::{Error, Result};

/// POSIX-ish signal numbers used to stop tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// SIGTERM (15) — graceful stop.
    Term,
    /// SIGKILL (9) — forced stop.
    Kill,
    /// SIGHUP (1) — reload.
    Hup,
}

impl Signal {
    /// The numeric signal value.
    pub fn number(self) -> u8 {
        match self {
            Signal::Hup => 1,
            Signal::Kill => 9,
            Signal::Term => 15,
        }
    }

    /// Whether this signal terminates the process.
    pub fn is_terminating(self) -> bool {
        matches!(self, Signal::Term | Signal::Kill)
    }
}

/// The result of waiting on a stopped task or exec process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus {
    /// The exit code.
    pub code: i32,
}

impl ExitStatus {
    /// Whether the process exited cleanly (code 0).
    pub fn success(self) -> bool {
        self.code == 0
    }
}

/// An additional process run inside a task's container via `Exec`.
///
/// Mirrors containerd's `Process` created by `Task.Exec`: it has its own id and
/// pid but shares the container's namespaces. Talos uses execs for liveness/
/// readiness probes and `talosctl` debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecProcess {
    /// The exec id, unique within the task.
    pub exec_id: String,
    /// argv of the exec'd process.
    pub args: Vec<String>,
    /// Pid once started.
    pub pid: Option<u32>,
    /// State of the exec process.
    pub state: TaskState,
    /// Exit code once it stops.
    pub exit_code: Option<i32>,
}

/// State of a containerd task's process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task created from the spec but not yet started.
    Created,
    /// Process is running.
    Running,
    /// Process paused (frozen cgroup).
    Paused,
    /// Process exited; carries no exit code at the type level.
    Stopped,
}

/// A running (or created) task with its pid and exit status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// The owning container id.
    pub container_id: String,
    /// OS pid once started; `None` while only Created.
    pub pid: Option<u32>,
    /// Current process state.
    pub state: TaskState,
    /// Exit code once Stopped.
    pub exit_code: Option<i32>,
    /// Additional exec'd processes, keyed by exec id.
    pub execs: Vec<ExecProcess>,
}

impl Task {
    /// Create a task in the `Created` state.
    pub fn new(container_id: impl Into<String>) -> Self {
        Task {
            container_id: container_id.into(),
            pid: None,
            state: TaskState::Created,
            exit_code: None,
            execs: Vec::new(),
        }
    }

    /// Start the task, assigning a pid. Only valid from `Created`.
    pub fn start(&mut self, pid: u32) -> Result<()> {
        if self.state != TaskState::Created {
            return Err(Error::invalid_state("task already started"));
        }
        if pid == 0 {
            return Err(Error::invalid("pid must be non-zero"));
        }
        self.pid = Some(pid);
        self.state = TaskState::Running;
        Ok(())
    }

    /// Pause a running task.
    pub fn pause(&mut self) -> Result<()> {
        if self.state != TaskState::Running {
            return Err(Error::invalid_state("only running tasks can be paused"));
        }
        self.state = TaskState::Paused;
        Ok(())
    }

    /// Resume a paused task.
    pub fn resume(&mut self) -> Result<()> {
        if self.state != TaskState::Paused {
            return Err(Error::invalid_state("only paused tasks can be resumed"));
        }
        self.state = TaskState::Running;
        Ok(())
    }

    /// Deliver a signal. Term/Kill stop the task and record an exit code.
    pub fn kill(&mut self, signal: Signal) -> Result<()> {
        match self.state {
            TaskState::Running | TaskState::Paused => {
                if matches!(signal, Signal::Term | Signal::Kill) {
                    self.state = TaskState::Stopped;
                    // 128 + signal number is the conventional shell exit code.
                    self.exit_code = Some(128 + i32::from(signal.number()));
                }
                Ok(())
            }
            _ => Err(Error::invalid_state("task is not running")),
        }
    }

    /// Record a clean (or explicit-code) process exit. Valid while running or
    /// paused; mirrors containerd delivering an exit event for the init process.
    pub fn exit(&mut self, code: i32) -> Result<()> {
        match self.state {
            TaskState::Running | TaskState::Paused => {
                self.state = TaskState::Stopped;
                self.exit_code = Some(code);
                Ok(())
            }
            _ => Err(Error::invalid_state("task is not running")),
        }
    }

    /// Wait for the task, returning its exit status once stopped.
    pub fn wait(&self) -> Result<ExitStatus> {
        match (self.state, self.exit_code) {
            (TaskState::Stopped, Some(code)) => Ok(ExitStatus { code }),
            (TaskState::Stopped, None) => {
                Err(Error::invalid_state("stopped task missing exit code"))
            }
            _ => Err(Error::invalid_state("task has not exited")),
        }
    }

    /// Start an exec process inside the running task.
    pub fn exec(&mut self, exec_id: impl Into<String>, args: Vec<String>, pid: u32) -> Result<()> {
        if self.state != TaskState::Running {
            return Err(Error::invalid_state("can only exec into a running task"));
        }
        if pid == 0 {
            return Err(Error::invalid("pid must be non-zero"));
        }
        if args.is_empty() {
            return Err(Error::invalid("exec args must not be empty"));
        }
        let exec_id = exec_id.into();
        if self.execs.iter().any(|e| e.exec_id == exec_id) {
            return Err(Error::invalid_state("exec id already exists"));
        }
        self.execs.push(ExecProcess {
            exec_id,
            args,
            pid: Some(pid),
            state: TaskState::Running,
            exit_code: None,
        });
        Ok(())
    }

    /// Record the exit of an exec process.
    pub fn exec_exit(&mut self, exec_id: &str, code: i32) -> Result<()> {
        let e = self
            .execs
            .iter_mut()
            .find(|e| e.exec_id == exec_id)
            .ok_or_else(|| Error::not_found("exec process"))?;
        if e.state != TaskState::Running {
            return Err(Error::invalid_state("exec process is not running"));
        }
        e.state = TaskState::Stopped;
        e.exit_code = Some(code);
        Ok(())
    }

    /// Look up an exec process by id.
    pub fn exec_process(&self, exec_id: &str) -> Option<&ExecProcess> {
        self.execs.iter().find(|e| e.exec_id == exec_id)
    }

    /// Whether the task has terminated.
    pub fn is_stopped(&self) -> bool {
        self.state == TaskState::Stopped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_assigns_pid() {
        let mut t = Task::new("etcd");
        assert_eq!(t.state, TaskState::Created);
        t.start(4242).unwrap();
        assert_eq!(t.pid, Some(4242));
        assert_eq!(t.state, TaskState::Running);
        assert!(t.start(1).is_err());
    }

    #[test]
    fn pause_resume_cycle() {
        let mut t = Task::new("c");
        t.start(10).unwrap();
        t.pause().unwrap();
        assert_eq!(t.state, TaskState::Paused);
        assert!(t.pause().is_err());
        t.resume().unwrap();
        assert_eq!(t.state, TaskState::Running);
    }

    #[test]
    fn kill_records_exit_code() {
        let mut t = Task::new("c");
        t.start(10).unwrap();
        t.kill(Signal::Term).unwrap();
        assert!(t.is_stopped());
        assert_eq!(t.exit_code, Some(128 + 15));
        assert!(t.kill(Signal::Kill).is_err());
    }

    #[test]
    fn zero_pid_rejected() {
        let mut t = Task::new("c");
        assert!(t.start(0).is_err());
    }

    #[test]
    fn signal_numbers() {
        assert_eq!(Signal::Kill.number(), 9);
        assert_eq!(Signal::Term.number(), 15);
        assert!(Signal::Kill.is_terminating());
        assert!(!Signal::Hup.is_terminating());
    }

    #[test]
    fn exit_records_code_and_wait_returns_it() {
        let mut t = Task::new("c");
        assert!(t.wait().is_err()); // not started
        t.start(10).unwrap();
        assert!(t.wait().is_err()); // running, not exited
        t.exit(0).unwrap();
        assert!(t.is_stopped());
        let status = t.wait().unwrap();
        assert!(status.success());
        assert_eq!(status.code, 0);
        // Exit again fails.
        assert!(t.exit(1).is_err());
    }

    #[test]
    fn non_success_exit_status() {
        let mut t = Task::new("c");
        t.start(1).unwrap();
        t.exit(2).unwrap();
        let status = t.wait().unwrap();
        assert!(!status.success());
        assert_eq!(status.code, 2);
    }

    #[test]
    fn exec_lifecycle() {
        let mut t = Task::new("c");
        // Can't exec before running.
        assert!(t.exec("probe", vec!["/bin/true".to_string()], 100).is_err());
        t.start(10).unwrap();
        t.exec("probe", vec!["/bin/true".to_string()], 100).unwrap();
        assert_eq!(t.exec_process("probe").unwrap().state, TaskState::Running);
        // Duplicate exec id rejected.
        assert!(t.exec("probe", vec!["/bin/true".to_string()], 101).is_err());
        // Empty args / zero pid rejected.
        assert!(t.exec("p2", Vec::new(), 5).is_err());
        assert!(t.exec("p3", vec!["/bin/true".to_string()], 0).is_err());

        t.exec_exit("probe", 0).unwrap();
        assert_eq!(t.exec_process("probe").unwrap().exit_code, Some(0));
        // Exiting twice / unknown id fails.
        assert!(t.exec_exit("probe", 1).is_err());
        assert_eq!(t.exec_exit("nope", 0).unwrap_err().kind(), "not_found");
    }

    #[test]
    fn exit_status_helper() {
        assert!(ExitStatus { code: 0 }.success());
        assert!(!ExitStatus { code: 137 }.success());
    }
}
