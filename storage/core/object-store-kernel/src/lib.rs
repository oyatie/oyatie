//! Deprecated source-compatibility crate for the pre-P0 CAS package identity.
//!
//! New storage code imports [`storage_domain::cas`]. This crate remains during
//! the advertised P0 compatibility window and contains no independent engine.

#![forbid(unsafe_code)]

pub use storage_domain::cas::*;

#[cfg(test)]
mod tests {
    use super::TenantId;

    #[test]
    fn legacy_package_reexports_the_cas_contract() {
        let tenant = TenantId::parse("ten_compat").expect("canonical tenant id parses");
        assert_eq!(tenant.as_str(), "ten_compat");
    }
}
