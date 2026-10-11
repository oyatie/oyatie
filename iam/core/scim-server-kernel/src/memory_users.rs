use super::*;

#[derive(Default)]
pub struct InMemoryUserStore {
    inner: std::sync::Mutex<Vec<(TenantId, User)>>,
}

impl UserStore for InMemoryUserStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .map(|g| {
                    g.iter()
                        .filter(|(t, _)| t == tenant)
                        .map(|(_, u)| u.clone())
                        .collect()
                })
                .unwrap_or_default()
        })
    }
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, u)| t == tenant && u.id == *id)
                .map(|(_, u)| u.clone())
        })
    }
    fn put<'a>(
        &'a self,
        user: &'a User,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            if g.iter().any(|(t, existing)| {
                t == tenant && (existing.id == user.id || existing.user_name == user.user_name)
            }) {
                return Err(ScimStoreError::Conflict);
            }
            g.push((tenant.clone(), user.clone()));
            Ok(())
        })
    }
    fn compare_and_put<'a>(
        &'a self,
        expected: &'a User,
        proposed: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut rows = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            let (_, current) = rows
                .iter_mut()
                .find(|(t, user)| t == tenant && user.id == expected.id)
                .ok_or(ScimStoreError::Conflict)?;
            if current != expected {
                return Err(ScimStoreError::Conflict);
            }
            if !native_admin && protected_user_transition(current, proposed) {
                return Err(ScimStoreError::ProtectedUserChange);
            }
            *current = proposed.clone();
            Ok(())
        })
    }
    fn compare_and_delete<'a>(
        &'a self,
        expected: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut rows = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            let index = rows
                .iter()
                .position(|(t, user)| t == tenant && user.id == expected.id)
                .ok_or(ScimStoreError::Conflict)?;
            let current = &rows[index].1;
            if current != expected {
                return Err(ScimStoreError::Conflict);
            }
            if !native_admin && protected_user_deletion(current) {
                return Err(ScimStoreError::ProtectedUserChange);
            }
            rows.remove(index);
            Ok(())
        })
    }
    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            g.retain(|(t, u)| !(t == tenant && u.id == *id));
            Ok(())
        })
    }
    fn find_by_user_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, u)| t == tenant && u.user_name == user_name)
                .map(|(_, u)| u.clone())
        })
    }
}
