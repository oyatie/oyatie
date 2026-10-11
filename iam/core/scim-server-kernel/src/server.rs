use super::*;

impl<U: UserStore, G: GroupStore, I: IdGen> ScimServer for ReferenceScimServer<U, G, I> {
    fn list_users<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<User>, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let all = self.users.list(tenant).await;
            let filtered = apply_user_filter(&all, q.filter.as_deref())?;
            let total = filtered.len();
            let per_page = q.items_per_page.min(self.max_items_per_page).max(1);
            let start = q.start_index.saturating_sub(1);
            let slice: Vec<User> = filtered.into_iter().skip(start).take(per_page).collect();
            Ok(ListResponse {
                schemas: vec![ListResponse::<User>::SCHEMA.to_owned()],
                total_results: total,
                resources: slice,
                items_per_page: per_page,
                start_index: q.start_index,
            })
        })
    }

    fn get_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            self.users
                .get(tenant, id)
                .await
                .ok_or_else(|| ScimError::new(404, None, format!("user '{}' not found", id.0)))
        })
    }

    fn create_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        self.create_user_with_authority(tenant, input, now_unix, false)
    }

    fn replace_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        self.replace_user_with_authority(tenant, id, input, now_unix, false)
    }

    fn patch_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        self.patch_user_with_authority(tenant, id, op, now_unix, false)
    }

    fn delete_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>> {
        self.delete_user_with_authority(tenant, id, false)
    }

    fn list_groups<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<Group>, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let all = self.groups.list(tenant).await;
            let total = all.len();
            let per_page = q.items_per_page.min(self.max_items_per_page).max(1);
            let start = q.start_index.saturating_sub(1);
            let slice: Vec<Group> = all.into_iter().skip(start).take(per_page).collect();
            Ok(ListResponse {
                schemas: vec![ListResponse::<Group>::SCHEMA.to_owned()],
                total_results: total,
                resources: slice,
                items_per_page: per_page,
                start_index: q.start_index,
            })
        })
    }
    fn get_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            self.groups
                .get(tenant, id)
                .await
                .ok_or_else(|| ScimError::new(404, None, format!("group '{}' not found", id.0)))
        })
    }
    fn create_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewGroup,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            if input.display_name.is_empty() {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "displayName is required",
                ));
            }
            let id = self.ids.next_id();
            let etag = format!("{}{}", id.0, now_unix);
            let group = Group {
                schemas: vec![Group::CORE_SCHEMA.to_owned()],
                id: id.clone(),
                display_name: input.display_name,
                members: input.members,
                meta: self.group_meta(tenant, &id, now_unix, &etag),
            };
            self.groups.put(&group, tenant).await?;
            Ok(group)
        })
    }
    fn patch_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let mut group =
                self.groups.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("group '{}' not found", id.0))
                })?;
            for o in &op.operations {
                apply_patch_group(&mut group, o)?;
            }
            let etag = format!("{}{}", id.0, now_unix);
            group.meta.last_modified = rfc3339(now_unix);
            group.meta.version = format!("W/\"{etag}\"");
            self.groups.put(&group, tenant).await?;
            Ok(group)
        })
    }
    fn delete_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>> {
        Box::pin(async move {
            if self.groups.get(tenant, id).await.is_none() {
                return Err(ScimError::new(
                    404,
                    None,
                    format!("group '{}' not found", id.0),
                ));
            }
            self.groups.delete(tenant, id).await?;
            Ok(())
        })
    }
}
