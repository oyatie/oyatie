use shared_resource_provider_contract_kernel::{
    ListEntry, Page, PageRequest, ProviderError, ProviderFuture, ResourceName,
};

use super::support::{Document, ReferenceProvider};

pub(super) fn get<'a>(
    provider: &'a ReferenceProvider,
    name: &'a ResourceName,
) -> ProviderFuture<'a, Document> {
    Box::pin(async move {
        provider
            .items
            .get(&name.to_string())
            .cloned()
            .ok_or_else(|| ProviderError::NotFound {
                name: name.to_string(),
            })
    })
}

pub(super) fn list<'a>(
    provider: &'a ReferenceProvider,
    collection: &'a str,
    request: &'a PageRequest,
) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
    Box::pin(async move {
        let prefix = format!("{collection}/");
        let start_at = request
            .page_token
            .as_ref()
            .map(|token| token.as_str().to_owned());
        let mut items = Vec::new();
        let mut next_page_token = None;
        for (key, value) in &provider.items {
            if !key.starts_with(&prefix) {
                continue;
            }
            if let Some(start) = &start_at
                && key < start
            {
                continue;
            }
            if items.len() as u32 == request.page_size {
                next_page_token = Some(
                    shared_resource_provider_contract_kernel::PageToken::new(key.clone()).map_err(
                        |e| ProviderError::Internal {
                            message: e.to_string(),
                        },
                    )?,
                );
                break;
            }
            let name =
                ResourceName::try_from(key.clone()).map_err(|e| ProviderError::Internal {
                    message: e.to_string(),
                })?;
            items.push(ListEntry {
                name,
                resource: value.clone(),
            });
        }
        Ok(Page {
            items,
            next_page_token,
        })
    })
}
