//! Identity and lifetime ownership evidence shared by native store ports.
#![forbid(unsafe_code)]

use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StoreIdentity {
    pub device: u64, // data_class: INTERNAL_ONLY
    pub inode: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct StoreAccess {
    identity: StoreIdentity,
    exclusive: bool,
    owner: Arc<()>,
}

impl StoreAccess {
    /// Adapters issue one token per acquired lifetime guard, never per connection clone.
    pub fn new(identity: StoreIdentity, exclusive: bool) -> Self {
        Self {
            identity,
            exclusive,
            owner: Arc::new(()),
        }
    }
    pub fn identity(&self) -> StoreIdentity {
        self.identity
    }
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }
    pub fn same_owner(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.exclusive == other.exclusive
            && Arc::ptr_eq(&self.owner, &other.owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_file_identity_is_not_ownership() {
        let id = StoreIdentity {
            device: 1,
            inode: 2,
        };
        let first = StoreAccess::new(id, true);
        assert!(first.same_owner(&first.clone()));
        assert!(!first.same_owner(&StoreAccess::new(id, true)));
        assert!(!first.same_owner(&StoreAccess::new(id, false)));
    }
}
