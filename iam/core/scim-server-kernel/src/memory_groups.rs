use super::*;

#[derive(Default)]
pub struct InMemoryGroupStore {
    inner: std::sync::Mutex<Vec<(TenantId, Group)>>,
}

impl GroupStore for InMemoryGroupStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<Group>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .map(|g| {
                    g.iter()
                        .filter(|(t, _)| t == tenant)
                        .map(|(_, gr)| gr.clone())
                        .collect()
                })
                .unwrap_or_default()
        })
    }
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<Group>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, gr)| t == tenant && gr.id == *id)
                .map(|(_, gr)| gr.clone())
        })
    }
    fn put<'a>(
        &'a self,
        group: &'a Group,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "group store lock poisoned".to_owned(),
            })?;
            if let Some(slot) = g
                .iter_mut()
                .find(|(t, gr)| t == tenant && gr.id == group.id)
            {
                slot.1 = group.clone();
            } else {
                g.push((tenant.clone(), group.clone()));
            }
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
                detail: "group store lock poisoned".to_owned(),
            })?;
            g.retain(|(t, gr)| !(t == tenant && gr.id == *id));
            Ok(())
        })
    }
}
