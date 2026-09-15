#[cfg(any(target_os = "linux", target_os = "macos"))]
fn provider_exited(child: &Child) -> io::Result<bool> {
    use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
    let pid = Pid::from_raw(child.id() as i32)
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    waitid(
        WaitId::Pid(pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
    )
    .map(|status| status.is_some())
    .map_err(io::Error::from)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn provider_exited(_child: &Child) -> io::Result<bool> {
    Err(io::ErrorKind::Unsupported.into())
}

#[cfg(unix)]
fn configure_provider_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_provider_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_provider_group(
    run: QualificationRun,
    child: &Child,
) -> Result<(), CandidateHeadQualificationFailure> {
    use rustix::process::{Pid, Signal, kill_process_group};
    let pid = Pid::from_raw(child.id() as i32).ok_or(
        CandidateHeadQualificationFailure::ProviderKill {
            run,
            kind: io::ErrorKind::InvalidInput,
        },
    )?;
    match kill_process_group(pid, Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        #[cfg(target_os = "macos")]
        Err(rustix::io::Errno::PERM) if only_exited_provider_remains(child) => Ok(()),
        Err(error) => Err(CandidateHeadQualificationFailure::ProviderKill {
            run,
            kind: io::Error::from(error).kind(),
        }),
    }
}

#[cfg(not(unix))]
fn terminate_provider_group(
    _run: QualificationRun,
    _child: &Child,
) -> Result<(), CandidateHeadQualificationFailure> {
    Err(CandidateHeadQualificationFailure::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
fn only_exited_provider_remains(child: &Child) -> bool {
    // WNOWAIT pins the leader and PGID until terminate_child reaps it. XNU
    // inventories group membership under proc_list_lock, including zombies.
    // A singleton in a buffer with room for two cannot conceal truncation.
    provider_exited(child).unwrap_or(false)
        && libproc::processes::pids_by_type(libproc::processes::ProcFilter::ByProgramGroup {
            pgrpid: child.id(),
        })
        .is_ok_and(|members| members.capacity() >= 2 && members.as_slice() == [child.id()])
}

#[cfg(all(test, target_os = "macos"))]
mod process_group_tests {
    use super::*;

    struct Group {
        child: Child,
        finished: bool,
    }

    impl Group {
        fn spawn(script: &str) -> Self {
            let mut command = Command::new("/bin/sh");
            command
                .args(["-c", script])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            configure_provider_group(&mut command);
            Self {
                child: command.spawn().expect("isolated test group must start"),
                finished: false,
            }
        }

        fn await_exit_unreaped(&self) {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !provider_exited(&self.child).expect("leader must remain waitable") {
                assert!(Instant::now() < deadline, "test leader failed to exit");
                thread::sleep(Duration::from_millis(10));
            }
        }

        fn finish(&mut self) -> ExitStatus {
            self.finished = true;
            terminate_child(QualificationRun::First, &mut self.child).unwrap()
        }
    }

    impl Drop for Group {
        fn drop(&mut self) {
            if !self.finished {
                let _ = terminate_child(QualificationRun::First, &mut self.child);
            }
        }
    }

    #[test]
    fn live_provider_cannot_use_exited_singleton_exception() {
        let group = Group::spawn("exec /bin/sleep 30");
        assert!(!provider_exited(&group.child).unwrap());
        assert!(!only_exited_provider_remains(&group.child));
    }

    #[test]
    fn exited_provider_with_live_descendant_cannot_use_singleton_exception() {
        let mut group = Group::spawn("/bin/sleep 30 & exit 0");
        group.await_exit_unreaped();
        let members =
            libproc::processes::pids_by_type(libproc::processes::ProcFilter::ByProgramGroup {
                pgrpid: group.child.id(),
            })
            .unwrap();
        assert!(members.contains(&group.child.id()));
        assert!(members.len() >= 2, "fixture must have a live descendant");
        assert!(!only_exited_provider_remains(&group.child));
        assert!(group.finish().success());
    }

    #[test]
    fn exited_singleton_is_proven_without_reaping_its_leader() {
        let mut group = Group::spawn("exit 0");
        group.await_exit_unreaped();
        assert!(only_exited_provider_remains(&group.child));
        assert!(
            provider_exited(&group.child).unwrap(),
            "proof must preserve WNOWAIT"
        );
        assert!(group.finish().success());
    }
}
