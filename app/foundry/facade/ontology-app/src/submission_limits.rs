use std::io::{self, Write};

use foundry_submission_draft::{MAX_SUBMISSION_BYTES, SubmitError, SubmitRequest};

/// Count canonical wire bytes without allocating a second request or its
/// escaped strings. Serialization stops as soon as the budget is exhausted.
pub(super) fn check(request: &SubmitRequest) -> Result<(), SubmitError> {
    serde_json::to_writer(Budget(MAX_SUBMISSION_BYTES), request).map_err(|_| SubmitError::Surface {
        cause: "the submission exceeds the request byte limit",
    })
}

struct Budget(usize);

impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "submission byte limit"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
