use super::*;

pub(super) fn hub_items(
    messages: &[MessageItem],
    communities: &[CommunityItem],
    drafts: &[LocalDraft],
    surface: ProductSurface,
) -> Vec<HubItem> {
    let mut items = drafts
        .iter()
        .filter(|draft| draft.surface == surface)
        .map(|draft| HubItem {
            surface,
            source: "Local draft".to_string(),
            title: draft.title.clone(),
            body: draft.body.clone(),
            meta: "Stored in WASM island state only".to_string(),
        })
        .collect::<Vec<_>>();

    match surface {
        ProductSurface::Messenger => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| !item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Thread preview; no external message sent".to_string(),
                    }),
            );
            items.extend([
                HubItem {
                    surface,
                    source: "Workflow bot".to_string(),
                    title: "Tenant change approval output routes".to_string(),
                    body: "Messenger post links payroll delta, cloud rollback, Mail brief, Community note, and REC-WF-7741 evidence.".to_string(),
                    meta: "Evidence-linked workflow route".to_string(),
                },
                HubItem {
                    surface,
                    source: "Payroll desk".to_string(),
                    title: "4대보험 delta owner needed".to_string(),
                    body: "Park Seo-jun tier change is blocking close; finance owner can review from Action Inbox or Evidence Spine.".to_string(),
                    meta: "Unread operations thread".to_string(),
                },
                HubItem {
                    surface,
                    source: "Cloud SRE".to_string(),
                    title: "Mesh rollback note ready".to_string(),
                    body: "us-east-2 split has runbook, resource inventory, and audit-chain previews attached for reviewer context.".to_string(),
                    meta: "Evidence-linked incident thread".to_string(),
                },
            ]);
        }
        ProductSurface::Mail => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Mail preview; compose is local only".to_string(),
                    }),
            );
            items.extend([
                HubItem {
                    surface,
                    source: "Draft".to_string(),
                    title: "Finance approval brief".to_string(),
                    body: "Formal mail for CFO approval includes payroll close, HomeTax transport, vendor threshold, and receipt links.".to_string(),
                    meta: "Draft · requires human send".to_string(),
                },
                HubItem {
                    surface,
                    source: "Review".to_string(),
                    title: "HomeTax attestation request".to_string(),
                    body: "118 employees validated; 사업자등록번호 confirmation and CFO attestation remain before filing preview.".to_string(),
                    meta: "Evidence-linked mail route".to_string(),
                },
                HubItem {
                    surface,
                    source: "Policy".to_string(),
                    title: "Vendor approval route exception".to_string(),
                    body: "Stripe renewal can move to one-step approval below policy threshold; Procurement and Audit are copied.".to_string(),
                    meta: "Draft · policy copy".to_string(),
                },
            ]);
        }
        ProductSurface::Community => {
            items.extend(communities.iter().map(|item| HubItem {
                surface,
                source: item.space.clone(),
                title: item.topic.clone(),
                body: item.activity.clone(),
                meta: "Community post preview; no backend write".to_string(),
            }));
            items.extend([
                HubItem {
                    surface,
                    source: "Governance council".to_string(),
                    title: "April close governance digest".to_string(),
                    body: "Summarizes payroll blocker, withholding filing readiness, cloud rollback evidence, and reviewer assignments.".to_string(),
                    meta: "Draft post · council review".to_string(),
                },
                HubItem {
                    surface,
                    source: "Policy forum".to_string(),
                    title: "Approval threshold clarification".to_string(),
                    body: "Procurement threshold note is ready for role-aware publication with Mail and Audit references attached.".to_string(),
                    meta: "Evidence-linked community note".to_string(),
                },
            ]);
        }
        ProductSurface::Workflow => {}
    }

    items
}

pub(super) fn hub_item_kind(item: &HubItem, index: usize) -> &'static str {
    let haystack =
        format!("{} {} {} {}", item.source, item.title, item.body, item.meta).to_ascii_lowercase();
    if haystack.contains("draft") || haystack.contains("brief") || haystack.contains("send") {
        "draft"
    } else if haystack.contains("evidence")
        || haystack.contains("receipt")
        || haystack.contains("rec-")
    {
        "evidence"
    } else if index < 2 || haystack.contains("unread") || haystack.contains("blocking") {
        "unread"
    } else {
        "review"
    }
}

pub(super) fn hub_item_chip_class(kind: &str) -> &'static str {
    match kind {
        "draft" => "status-chip ai",
        "evidence" => "status-chip success",
        "unread" => "status-chip warning",
        _ => "status-chip",
    }
}

pub(super) fn selected_hub_item(items: &[HubItem], selected_index: usize) -> Option<HubItem> {
    items.get(selected_index).or_else(|| items.first()).cloned()
}
