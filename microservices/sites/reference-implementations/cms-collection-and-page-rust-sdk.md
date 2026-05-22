---
doc_class: ReferenceImplementation
microservice: sites
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Site + CMS-collection + page lifecycle via the sites Rust SDK

A runnable example showing how to provision a site, define a CMS-collection, author a page, run the WCAG accessibility gate, and publish — using `oya-sites-client` (target API; once IP-002 + IP-003 + IP-009 + IP-012 land).

## Cargo.toml

```toml
[package]
name = "sites-publish-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-sites-client = { path = "../../crates/oya-sites-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::Utc;
use oya_cedar_client::CedarPrincipal;
use oya_sites_client::{
    AccessibilityVerdict, Block, BlockKind, CmsCollectionSchema, CmsEntryCreate, CmsField,
    CmsFieldType, PageCreateRequest, PublishRequest, RenderingMode, RetentionLevel,
    SitesClient, SitesClientConfig, SiteCreateRequest, Visibility,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("SITES_PRINCIPAL_JWT")?;
    let config = SitesClientConfig {
        cell_endpoint: std::env::var("SITES_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(30),
    };
    let client = SitesClient::connect(config).await?;

    // 1. Create the site.
    let site = client
        .site_create(SiteCreateRequest {
            name: "acme-marketing".into(),
            display_name: "ACME Marketing".into(),
            default_language: "en-US".into(),
            visibility: Visibility::Public,
        })
        .await?;
    info!(site_id = %site.site_id, subdomain = %site.default_subdomain, "site created");

    // 2. Define a CMS-collection for blog posts.
    let collection = client
        .cms_collection_define(CmsCollectionSchema {
            site_id: site.site_id.clone(),
            entity_type: "BlogPost".into(),
            fields: vec![
                CmsField {
                    name: "title".into(),
                    field_type: CmsFieldType::Text { max_length: Some(200) },
                    required: true,
                    unique: false,
                },
                CmsField {
                    name: "slug".into(),
                    field_type: CmsFieldType::Text { max_length: Some(100) },
                    required: true,
                    unique: true,
                },
                CmsField {
                    name: "body".into(),
                    field_type: CmsFieldType::RichText,
                    required: true,
                    unique: false,
                },
                CmsField {
                    name: "published_date".into(),
                    field_type: CmsFieldType::Date,
                    required: true,
                    unique: false,
                },
            ],
            url_pattern: "/blog/[slug]".into(),
        })
        .await?;
    info!(collection_id = %collection.collection_id, "CMS-collection defined");

    // 3. Seed a blog post.
    let _entry = client
        .cms_entry_create(CmsEntryCreate {
            site_id: site.site_id.clone(),
            entity_type: "BlogPost".into(),
            fields: serde_json::json!({
                "title": "Welcome to ACME Marketing",
                "slug": "welcome",
                "body": "<p>Welcome to the new ACME Marketing site. We're excited to share product updates, customer stories, and engineering deep-dives here.</p>",
                "published_date": "2026-05-20"
            }),
        })
        .await?;
    info!("blog post seeded");

    // 4. Author the blog listing page.
    let page = client
        .page_create(PageCreateRequest {
            site_id: site.site_id.clone(),
            path: "/blog".into(),
            title: "ACME Blog".into(),
            description: Some("Updates, customer stories, and engineering deep-dives".into()),
            rendering: RenderingMode::Isr {
                revalidate_seconds: 60,
            },
            blocks: vec![
                Block {
                    kind: BlockKind::Heading {
                        level: 1,
                        text: "ACME Blog".into(),
                    },
                },
                Block {
                    kind: BlockKind::Paragraph {
                        text: "Updates, customer stories, and engineering deep-dives from the ACME team."
                            .into(),
                    },
                },
                Block {
                    kind: BlockKind::CmsCollectionRender {
                        collection_name: "BlogPost".into(),
                        sort_by: "published_date".into(),
                        sort_order: oya_sites_client::SortOrder::Desc,
                        limit: 20,
                        item_template: "blog-post-card".into(),
                    },
                },
            ],
            seo_meta: oya_sites_client::SeoMeta {
                meta_title: "ACME Blog — Updates and Engineering".into(),
                meta_description: "The latest from ACME engineering and customer success teams."
                    .into(),
                canonical: None, // auto-derived from path
                og_image: Some("https://www.acme.example/og/blog.jpg".into()),
            },
        })
        .await?;
    info!(page_id = %page.page_id, "blog listing page created");

    // 5. Run the WCAG 2.2 AA accessibility check.
    let accessibility = client
        .page_accessibility_check(&page.page_id)
        .await?;

    match accessibility.verdict {
        AccessibilityVerdict::Pass => {
            info!("accessibility check passed");
        }
        AccessibilityVerdict::Fail { violations } => {
            warn!(
                violations_count = violations.len(),
                "accessibility check failed; publish will be blocked"
            );
            for v in &violations {
                warn!(
                    criterion = %v.criterion,
                    level = %v.level,
                    message = %v.message,
                    "accessibility violation"
                );
            }
            return Ok(()); // do not attempt to publish
        }
    }

    // 6. Publish the page.
    let publish_receipt = client
        .page_publish(PublishRequest {
            page_id: page.page_id.clone(),
            site_id: site.site_id.clone(),
            wcag_target: oya_sites_client::WcagLevel::AA,
            invalidate_cdn: true,
        })
        .await?;
    info!(
        published_at = %publish_receipt.published_at,
        cdn_invalidated = publish_receipt.cdn_invalidated,
        "page published"
    );

    // 7. Optional: bind a custom domain (would require DNS access).
    // ... see runbook for the ACME-DNS-01 flow ...

    Ok(())
}
```

## Expected log output

```
INFO site created site_id=site-7f3a9b2c subdomain=acme-marketing.drill-acme.sites.drill-syd-1.oyatie.local
INFO CMS-collection defined collection_id=col-blogpost-acme
INFO blog post seeded
INFO blog listing page created page_id=page-blog-listing-1
INFO accessibility check passed
INFO page published published_at=2026-05-20T13:42:00Z cdn_invalidated=true
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --site acme-marketing
```

Expected events:

- `site_created`
- `cms_collection_defined`
- `cms_entry_created`
- `page_created`
- `accessibility_check_completed` (verdict=pass)
- `seo_meta_check_completed`
- `page_published`
- `cdn_invalidated`

## Direct HTTP alternative

```sh
# Create site
curl -X POST https://sites.drill-syd-1.oyatie.local/v1/sites \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "name": "acme-marketing",
        "display_name": "ACME Marketing",
        "default_language": "en-US",
        "visibility": "public"
    }'

# Publish page
curl -X POST https://sites.drill-syd-1.oyatie.local/v1/pages/{page_id}/publish \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "wcag_target": "AA",
        "invalidate_cdn": true
    }'
```

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `wcag_publish_block` | No | Fix violations + re-attempt. |
| `seo_meta_check_failed` | No | Fix the meta + re-attempt. |
| `custom_domain_not_provisioned` | No | Wait for ACME provisioning. |
| `cms_field_validation_failed` | No | Fix the entry to match the schema. |
| `cdn_invalidation_failed` | Yes (transient) | SDK retries. |
| `cell_unavailable` | Yes (circuit-breaker) | Cell down; SDK fails after 3 retries; opens for 30 s. |

## Where this file lives

`microservices/sites/reference-implementations/cms-collection-and-page-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/sites/reference-implementations/publish-example/` once IP-002 + IP-003 + IP-009 + IP-012 land.
