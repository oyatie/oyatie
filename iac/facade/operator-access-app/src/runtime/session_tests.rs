use super::*;
use std::cell::RefCell;
use std::rc::Rc;

type Hook = Box<dyn FnMut(&[&str], bool) -> Result<Value, AccessError>>;
thread_local! { static OCI: RefCell<Option<Hook>> = RefCell::new(None); }

pub(super) fn intercept(args: &[&str], cleanup: bool) -> Option<Result<Value, AccessError>> {
    OCI.with(|hook| hook.borrow_mut().as_mut().map(|f| f(args, cleanup)))
}

struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        OCI.with(|h| h.borrow_mut().take());
    }
}

#[test]
fn connect_accepted_create_interruption_closes_only_correlated_session() {
    for failure in [
        AccessError::Cancelled,
        AccessError::Timeout,
        AccessError::DependencyFailed,
    ] {
        let p = crate::tests::profile();
        let name = Rc::new(RefCell::new(String::new()));
        let deleted = Rc::new(RefCell::new(Vec::new()));
        let observed = deleted.clone();
        let expected = crate::tests::profile();
        OCI.with(|h| *h.borrow_mut() = Some(Box::new(move |args, cleanup| {
            match args[2] {
                "create-port-forwarding" => {
                    assert!(!cleanup);
                    let at = args.iter().position(|a| *a == "--display-name").unwrap();
                    *name.borrow_mut() = args[at + 1].to_string();
                    Err(failure)
                }
                "list" => {
                    assert!(cleanup);
                    Ok(json!({"data": [
                        {"display-name": "another-operator", "id": "not-ours"},
                        {"display-name": *name.borrow(), "id": "ocid1.bastionsession.oc1.ap-chuncheon-1.abc",
                         "bastion-id": expected.bastion,
                         "target-resource-details": {"target-resource-id": expected.instance,
                            "target-resource-private-ip-address": expected.private_ip,
                            "target-resource-port": 50000}}
                    ]}))
                }
                "delete" => {
                    assert!(cleanup);
                    observed.borrow_mut().push(args[4].to_string());
                    Ok(json!({}))
                }
                "get" => { assert!(cleanup); Ok(json!({"data": {"lifecycle-state": "DELETED"}})) }
                _ => panic!("unexpected OCI call"),
            }
        })));
        let _reset = Reset;
        let mut sessions = Sessions {
            oci: Oci(&p),
            ids: Vec::new(),
            tunnels: Vec::new(),
            attempts: Vec::new(),
        };
        assert_eq!(sessions.connect(50000), Err(failure));
        assert_eq!(sessions.attempts.len(), 1);
        assert_eq!(sessions.close(), Ok(()));
        assert_eq!(
            *deleted.borrow(),
            ["ocid1.bastionsession.oc1.ap-chuncheon-1.abc"]
        );
        assert!(sessions.ids.is_empty() && sessions.attempts.is_empty());
    }
}
