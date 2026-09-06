use super::*;
use std::os::fd::AsRawFd;

pub(super) fn nonblocking(pipe: &impl AsRawFd) -> Result<(), AccessError> {
    // SAFETY: a live owned pipe supplies the fd. These calls only update its
    // status flags; no pointer or ownership is transferred to libc.
    unsafe {
        let flags = libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL);
        if flags < 0 || libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(AccessError::DependencyFailed);
        }
    }
    Ok(())
}

pub(super) fn read_step<T: Read>(
    pipe: &mut Option<T>,
    buffer: &mut Vec<u8>,
) -> Result<(), AccessError> {
    let Some(stream) = pipe else {
        return Ok(());
    };
    let mut chunk = Zeroizing::new([0u8; 16 * 1024]);
    match stream.read(chunk.as_mut()) {
        Ok(0) => {
            pipe.take();
        }
        Ok(size) => {
            if buffer.len() + size > process::LIMIT {
                return Err(AccessError::OutputLimit);
            }
            buffer.extend_from_slice(&chunk[..size]);
        }
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
            ) => {}
        Err(_) => return Err(AccessError::DependencyFailed),
    }
    Ok(())
}

pub(super) fn write_step<T: Write>(
    pipe: &mut Option<T>,
    input: &[u8],
    written: &mut usize,
) -> Result<(), AccessError> {
    if *written == input.len() {
        pipe.take();
        return Ok(());
    }
    let Some(stream) = pipe else {
        return Err(AccessError::DependencyFailed);
    };
    let end = (*written + 16 * 1024).min(input.len());
    match stream.write(&input[*written..end]) {
        Ok(0) => return Err(AccessError::DependencyFailed),
        Ok(size) => *written += size,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
            ) => {}
        Err(_) => return Err(AccessError::DependencyFailed),
    }
    Ok(())
}
