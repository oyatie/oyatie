#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::Error;
use messenger_policy_api::{Action, Policy};

pub async fn authorize(
    policy: &impl Policy,
    tenant: &str,
    subject: &str,
    action: Action,
    resource: &str,
) -> Result<(), Error> {
    if tenant.is_empty() || subject.is_empty() || resource.is_empty() {
        return Err(Error::Denied);
    }
    policy.authorize(tenant, subject, action, resource).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::sync::Mutex;
    use std::task::{Context, Poll, Waker};

    struct Script {
        result: Result<(), Error>,
        calls: Mutex<Vec<(String, String, Action, String)>>,
    }

    impl Script {
        fn new(result: Result<(), Error>) -> Self {
            Self {
                result,
                calls: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<(String, String, Action, String)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl Policy for Script {
        async fn authorize(
            &self,
            tenant: &str,
            subject: &str,
            action: Action,
            resource: &str,
        ) -> Result<(), Error> {
            self.calls.lock().unwrap().push((
                tenant.into(),
                subject.into(),
                action,
                resource.into(),
            ));
            self.result.clone()
        }
    }

    fn finished<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("policy future was pending"),
        }
    }

    #[test]
    fn allow_forwards_the_parc() {
        let policy = Script::new(Ok(()));
        assert_eq!(
            finished(authorize(
                &policy,
                "acme",
                "alice",
                Action::Send,
                "work-room"
            )),
            Ok(())
        );
        assert_eq!(
            policy.calls(),
            vec![(
                "acme".into(),
                "alice".into(),
                Action::Send,
                "work-room".into()
            )]
        );
    }

    #[test]
    fn deny_and_outage_stay_errors() {
        let denied = Script::new(Err(Error::Denied));
        assert_eq!(
            finished(authorize(
                &denied,
                "acme",
                "alice",
                Action::CreateRoom,
                "work-room"
            )),
            Err(Error::Denied)
        );
        let outage = Script::new(Err(Error::Unavailable("policy down".into())));
        assert!(matches!(
            finished(authorize(
                &outage,
                "acme",
                "alice",
                Action::ManageRoom,
                "work-room"
            )),
            Err(Error::Unavailable(_))
        ));
        assert_eq!(outage.calls().len(), 1);
    }

    #[test]
    fn empty_identity_is_denied_without_policy() {
        let policy = Script::new(Ok(()));
        for (tenant, subject, resource) in [
            ("", "alice", "work-room"),
            ("acme", "", "work-room"),
            ("acme", "alice", ""),
        ] {
            assert_eq!(
                finished(authorize(
                    &policy,
                    tenant,
                    subject,
                    Action::Invite,
                    resource
                )),
                Err(Error::Denied)
            );
        }
        assert!(policy.calls().is_empty());
    }
}
