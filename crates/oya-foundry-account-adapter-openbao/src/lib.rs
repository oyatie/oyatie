//! M02-P00-IP-003 — OpenBao SecretStorePort adapter (default reference impl).
//! In-memory store keyed by SecretReference. Network OpenBao integration deferred;
//! interface remains provider-agnostic — adapters in other crates can swap in.

use std::collections::HashMap;

use oya_foundry_account_domain::{
    SecretMaterial, SecretReference, SecretStoreError, SecretStorePort,
};

#[derive(Default)]
pub struct OpenBaoAdapter {
    store: HashMap<String, SecretMaterial>,
}

impl OpenBaoAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(sref: &SecretReference) -> String {
        format!("{sref:?}")
    }
}

impl SecretStorePort for OpenBaoAdapter {
    fn put(
        &mut self,
        sref: &SecretReference,
        material: SecretMaterial,
    ) -> Result<(), SecretStoreError> {
        if material.is_empty() {
            return Err(SecretStoreError::Backend("empty material rejected".into()));
        }
        // Key on Debug repr (which is redacted) — internal map is keyed by the
        // SecretReference identity, not by the secret value. We use a parallel
        // identity map to keep equality without exposing the inner string.
        self.store.insert(Self::key(sref), material);
        Ok(())
    }

    fn get(&self, sref: &SecretReference) -> Result<SecretMaterial, SecretStoreError> {
        self.store
            .get(&Self::key(sref))
            .cloned()
            .ok_or(SecretStoreError::NotFound)
    }

    fn rotate(
        &mut self,
        sref: &SecretReference,
        new_material: SecretMaterial,
    ) -> Result<(), SecretStoreError> {
        if !self.store.contains_key(&Self::key(sref)) {
            return Err(SecretStoreError::NotFound);
        }
        self.store.insert(Self::key(sref), new_material);
        Ok(())
    }

    fn delete(&mut self, sref: &SecretReference) -> Result<(), SecretStoreError> {
        self.store
            .remove(&Self::key(sref))
            .map(|_| ())
            .ok_or(SecretStoreError::NotFound)
    }
}

pub fn store_secret_reference(
    adapter: &mut OpenBaoAdapter,
    sref: &SecretReference,
    material: SecretMaterial,
) -> Result<(), SecretStoreError> {
    adapter.put(sref, material)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sref(s: &str) -> SecretReference {
        SecretReference::new(s.to_owned()).unwrap()
    }
    fn mat(bytes: &[u8]) -> SecretMaterial {
        SecretMaterial::new(bytes.to_vec())
    }

    #[test]
    fn put_then_get_roundtrip() {
        let mut a = OpenBaoAdapter::new();
        let r = sref("sref://aws-prod-key");
        a.put(&r, mat(b"top-secret-key-material")).unwrap();
        let got = a.get(&r).unwrap();
        assert_eq!(got.expose_for_provider_call(), b"top-secret-key-material");
    }

    #[test]
    fn get_missing_returns_not_found() {
        let a = OpenBaoAdapter::new();
        assert_eq!(
            a.get(&sref("sref://missing")),
            Err(SecretStoreError::NotFound)
        );
    }

    #[test]
    fn put_empty_material_rejected() {
        let mut a = OpenBaoAdapter::new();
        assert!(a.put(&sref("sref://k1"), mat(b"")).is_err());
    }

    #[test]
    fn rotate_replaces_material() {
        let mut a = OpenBaoAdapter::new();
        let r = sref("sref://gemini-key");
        a.put(&r, mat(b"old-key")).unwrap();
        a.rotate(&r, mat(b"new-key")).unwrap();
        assert_eq!(a.get(&r).unwrap().expose_for_provider_call(), b"new-key");
    }

    #[test]
    fn rotate_missing_returns_not_found() {
        let mut a = OpenBaoAdapter::new();
        assert_eq!(
            a.rotate(&sref("sref://gone"), mat(b"x")),
            Err(SecretStoreError::NotFound)
        );
    }

    #[test]
    fn delete_removes_entry() {
        let mut a = OpenBaoAdapter::new();
        let r = sref("sref://disposable");
        a.put(&r, mat(b"secret-value")).unwrap();
        a.delete(&r).unwrap();
        assert_eq!(a.get(&r), Err(SecretStoreError::NotFound));
    }

    #[test]
    fn delete_missing_returns_not_found() {
        let mut a = OpenBaoAdapter::new();
        assert_eq!(
            a.delete(&sref("sref://gone")),
            Err(SecretStoreError::NotFound)
        );
    }

    #[test]
    fn material_debug_is_redacted() {
        let m = mat(b"my-actual-secret-bytes");
        let dbg = format!("{m:?}");
        assert!(dbg.contains("[REDACTED"));
        assert!(!dbg.contains("my-actual-secret"));
    }

    #[test]
    fn sref_debug_is_redacted_in_store_too() {
        let r = sref("sref://very-secret-key-id");
        let dbg = format!("{r:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("very-secret"));
    }

    #[test]
    fn store_secret_reference_fn() {
        let mut a = OpenBaoAdapter::new();
        let r = sref("sref://test-key");
        store_secret_reference(&mut a, &r, mat(b"value")).unwrap();
        assert_eq!(a.get(&r).unwrap().expose_for_provider_call(), b"value");
    }
}
