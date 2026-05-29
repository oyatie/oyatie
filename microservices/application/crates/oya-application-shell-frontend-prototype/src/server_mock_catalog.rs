#![cfg(feature = "ssr")]

use crate::render_envelope::{DemoContext, TenantRenderEnvelope, permitted_envelope_snapshot};

pub const SERVER_ONLY_CATALOG_SENTINEL: &str =
    "SERVER_ONLY_CATALOG_SENTINEL_DO_NOT_SHIP_TO_BROWSER_WASM";

#[derive(Clone, Debug)]
struct CatalogModule {
    name: &'static str,
    requires_healthcare_accreditation: bool,
}

const SERVER_ONLY_FULL_CATALOG: &[CatalogModule] = &[
    CatalogModule {
        name: "Tenant Admin",
        requires_healthcare_accreditation: false,
    },
    CatalogModule {
        name: "Cloud Compute",
        requires_healthcare_accreditation: false,
    },
    CatalogModule {
        name: "Cloud Network",
        requires_healthcare_accreditation: false,
    },
    CatalogModule {
        name: "Accounting",
        requires_healthcare_accreditation: false,
    },
    CatalogModule {
        name: "Human Resources",
        requires_healthcare_accreditation: false,
    },
    CatalogModule {
        name: "Clinical Home",
        requires_healthcare_accreditation: true,
    },
    CatalogModule {
        name: "Patient Schedule",
        requires_healthcare_accreditation: true,
    },
    CatalogModule {
        name: "Care Workflows",
        requires_healthcare_accreditation: true,
    },
];

pub fn derive_tenant_render_envelope(context: DemoContext) -> TenantRenderEnvelope {
    let envelope = permitted_envelope_snapshot(context);

    debug_assert!(SERVER_ONLY_FULL_CATALOG.iter().any(|module| {
        module.requires_healthcare_accreditation && module.name == "Clinical Home"
    }));
    debug_assert!(!SERVER_ONLY_CATALOG_SENTINEL.is_empty());

    envelope
}
