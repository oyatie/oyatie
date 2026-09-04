use shared_resource_provider_contract_kernel::conformance::ConformanceFixture;

use super::support::{Document, ReferenceProvider};

pub(super) struct ReferenceFixture;

impl ConformanceFixture for ReferenceFixture {
    type Provider = ReferenceProvider;

    fn fresh_provider(&self) -> ReferenceProvider {
        ReferenceProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        Document {
            title: format!("Document {ordinal}"),
            revision: ordinal,
        }
    }
}
