use super::StoreAccessError;
use std::fs::File;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) fn lock(file: &File, exclusive: bool) -> Result<(), StoreAccessError> {
    // SQLite locks near 1GiB; byte zero is the separate native ownership protocol.
    lock_range(file, exclusive, 0, 1)
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(super) fn lock(_: &File, _: bool) -> Result<(), StoreAccessError> {
    Err(StoreAccessError::Unsupported)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn lock_range(
    file: &File,
    exclusive: bool,
    start: libc::off_t,
    length: libc::off_t,
) -> Result<(), StoreAccessError> {
    use std::os::fd::AsRawFd;
    let mut request = libc::flock {
        l_type: if exclusive {
            libc::F_WRLCK
        } else {
            libc::F_RDLCK
        } as _,
        l_whence: libc::SEEK_SET as _,
        l_start: start,
        l_len: length,
        l_pid: 0,
    };
    syscall(file.as_raw_fd(), &mut request)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[allow(unsafe_code)]
fn syscall(descriptor: libc::c_int, request: &mut libc::flock) -> Result<(), StoreAccessError> {
    // SAFETY: native initialized flock has a live mutable pointer for this call;
    // the borrowed descriptor stays open, and fcntl does not transfer ownership.
    let result = unsafe { libc::fcntl(descriptor, libc::F_OFD_SETLK, request) };
    if result == 0 {
        Ok(())
    } else {
        Err(refusal(std::io::Error::last_os_error()))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn refusal(error: std::io::Error) -> StoreAccessError {
    match error.raw_os_error() {
        Some(code) if code == libc::EAGAIN || code == libc::EACCES => StoreAccessError::Busy,
        Some(code)
            if code == libc::EINVAL
                || code == libc::ENOSYS
                || code == libc::EOPNOTSUPP
                || code == libc::ENOTSUP =>
        {
            StoreAccessError::Unsupported
        }
        _ => StoreAccessError::Io(error.to_string()),
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    #[test]
    fn nonblocking_failures_are_typed_and_never_fall_back() {
        for code in [libc::EAGAIN, libc::EACCES] {
            assert_eq!(
                refusal(std::io::Error::from_raw_os_error(code)),
                StoreAccessError::Busy
            );
        }
        for code in [libc::EINVAL, libc::ENOSYS, libc::ENOTSUP] {
            assert_eq!(
                refusal(std::io::Error::from_raw_os_error(code)),
                StoreAccessError::Unsupported
            );
        }
        let mut request = libc::flock {
            l_type: libc::F_WRLCK as _,
            l_whence: libc::SEEK_SET as _,
            l_start: 0,
            l_len: 1,
            l_pid: 0,
        };
        assert!(matches!(
            syscall(-1, &mut request),
            Err(StoreAccessError::Io(_))
        ));
        assert!(matches!(
            refusal(std::io::Error::from_raw_os_error(libc::EINTR)),
            StoreAccessError::Io(_)
        ));
    }
    #[test]
    fn ownership_is_exactly_one_byte_and_never_changes_file_contents() {
        use crate::tests::Directory;
        let directory = Directory::new();
        let path = directory.path("range");
        std::fs::write(&path, b"database header remains unchanged").unwrap();
        let set =
            crate::LocalStoreSet::exclusive(&[(path.clone(), crate::StoreOpen::Existing)]).unwrap();
        let other = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        assert_eq!(lock_range(&other, true, 0, 1), Err(StoreAccessError::Busy));
        lock_range(&other, true, 1, 1).unwrap();
        lock_range(&other, true, 0x40000000, 512).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"database header remains unchanged"
        );
        drop(other);
        drop(set);
    }
}
