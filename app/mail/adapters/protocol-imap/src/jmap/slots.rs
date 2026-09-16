use axum::http::StatusCode;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Default)]
pub(super) struct Slots<const LIMIT: usize>(Mutex<HashMap<(String, String), Weak<Semaphore>>>);
impl<const LIMIT: usize> Slots<LIMIT> {
    pub(super) fn acquire(
        &self,
        principal: &mail_api::Principal,
    ) -> Result<OwnedSemaphorePermit, StatusCode> {
        let mut slots = self.0.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        slots.retain(|_, v| v.strong_count() > 0);
        let key = (principal.tenant.clone(), principal.subject.clone());
        let semaphore = slots.get(&key).and_then(Weak::upgrade).unwrap_or_else(|| {
            let semaphore = Arc::new(Semaphore::new(LIMIT));
            slots.insert(key, Arc::downgrade(&semaphore));
            semaphore
        });
        semaphore
            .try_acquire_owned()
            .map_err(|_| StatusCode::TOO_MANY_REQUESTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_is_per_tenant_and_subject_and_recovers_after_cancellation() {
        let slots = Slots::<1>::default();
        let mut principal = mail_api::Principal {
            tenant: "t".into(),
            subject: "alice".into(),
            account: "a".into(),
        };
        let active = slots.acquire(&principal).unwrap();
        principal.account = "shared".into();
        assert_eq!(
            slots.acquire(&principal).unwrap_err(),
            StatusCode::TOO_MANY_REQUESTS
        );
        principal.tenant = "other".into();
        let unrelated = slots.acquire(&principal).unwrap();
        principal.tenant = "t".into();
        drop(active);
        assert!(slots.acquire(&principal).is_ok());
        drop(unrelated);
        let requests = Slots::<4>::default();
        let permits: Vec<_> = (0..4)
            .map(|_| requests.acquire(&principal).unwrap())
            .collect();
        assert_eq!(
            requests.acquire(&principal).unwrap_err(),
            StatusCode::TOO_MANY_REQUESTS
        );
        drop(permits);
        assert!(requests.acquire(&principal).is_ok());
    }
}
