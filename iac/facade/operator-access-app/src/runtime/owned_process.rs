use super::*;

pub(super) struct OwnedProcess(Option<Child>);

impl OwnedProcess {
    pub(super) fn new(child: Child) -> Self {
        Self(Some(child))
    }

    pub(super) fn child(&mut self) -> Result<&mut Child, AccessError> {
        self.0.as_mut().ok_or(AccessError::DependencyFailed)
    }

    pub(super) fn exited(&self) -> Result<bool, AccessError> {
        let child = self.0.as_ref().ok_or(AccessError::DependencyFailed)?;
        // SAFETY: waitid observes only our owned child. WNOWAIT deliberately
        // retains the leader (including a zombie) so its PID/PGID cannot be
        // reused before all pipes and the process group have been handled.
        unsafe {
            let mut info: libc::siginfo_t = std::mem::zeroed();
            let result = libc::waitid(
                libc::P_PID,
                child.id() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            );
            if result != 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    return Ok(false);
                }
                return Err(AccessError::DependencyFailed);
            }
            Ok(info.si_pid() != 0)
        }
    }

    pub(super) fn terminate(&mut self) -> Result<std::process::ExitStatus, AccessError> {
        let mut child = self.0.take().ok_or(AccessError::DependencyFailed)?;
        // SAFETY: the leader has never been reaped; PGID is the PID of this
        // process-group(0) child, not an unowned or potentially reused process.
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
        child.wait().map_err(|_| AccessError::DependencyFailed)
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if self.0.is_some() {
            let _ = self.terminate();
        }
    }
}
