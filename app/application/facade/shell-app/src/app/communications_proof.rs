use super::*;

pub(super) fn comms_receipt_bridge() -> impl IntoView {
    view! {
        <section
            class="comms-receipt-bridge"
            data-comms-receipt-bridge="true"
            aria-label="Messenger Mail Community receipt bridge"
        >
            <div class="comms-bridge-head">
                <div>
                    <p class="screen-anchor">"COMMS RECEIPT BRIDGE"</p>
                    <h4>"Messenger, Mail, and Community return to one proof packet"</h4>
                    <span data-comms-bridge-status="true">
                        "Ops room, approval brief, council post, and audit receipt are staged as one local FD-001 workload packet."
                    </span>
                </div>
                <button type="button" data-comms-bridge-action="seal">"Seal handoff"</button>
            </div>
            <div class="comms-bridge-routes" aria-label="Communication proof routes">
                <button type="button" class="selected" data-comms-bridge-route="messenger" data-comms-bridge-title="Messenger ops room" data-comms-bridge-receipt="REC-COMMS-MSG-021" data-comms-bridge-target="Ops room → Mail brief → Community note">
                    <span>"01 · Messenger"</span><strong>"Ops room thread"</strong><em>"REC-COMMS-MSG-021"</em>
                </button>
                <button type="button" data-comms-bridge-route="mail" data-comms-bridge-title="Mail approval brief" data-comms-bridge-receipt="REC-COMMS-MAIL-022" data-comms-bridge-target="Formal approval → Evidence packet">
                    <span>"02 · Mail"</span><strong>"Approval brief"</strong><em>"REC-COMMS-MAIL-022"</em>
                </button>
                <button type="button" data-comms-bridge-route="community" data-comms-bridge-title="Community council note" data-comms-bridge-receipt="REC-COMMS-COMM-023" data-comms-bridge-target="Council post → Role-visible vote">
                    <span>"03 · Community"</span><strong>"Governance note"</strong><em>"REC-COMMS-COMM-023"</em>
                </button>
                <button type="button" data-comms-bridge-route="receipt" data-comms-bridge-title="Universal receipt packet" data-comms-bridge-receipt="REC-COMMS-HANDOFF-006" data-comms-bridge-target="Audit ledger → Receipt stitching console">
                    <span>"04 · Receipt"</span><strong>"Audit stitch"</strong><em>"REC-COMMS-HANDOFF-006"</em>
                </button>
            </div>
            <aside class="comms-bridge-detail" aria-label="Selected communication receipt detail">
                <dl>
                    <div><dt>"Selected"</dt><dd data-comms-bridge-detail-title="true">"Messenger ops room"</dd></div>
                    <div><dt>"Receipt"</dt><dd data-comms-bridge-detail-receipt="true">"REC-COMMS-MSG-021"</dd></div>
                    <div><dt>"Route"</dt><dd data-comms-bridge-detail-target="true">"Ops room → Mail brief → Community note"</dd></div>
                </dl>
                <div class="comms-bridge-actions" aria-label="Communication receipt bridge actions">
                    <button type="button" data-comms-bridge-action="workflow">"Workflow"</button>
                    <button type="button" data-comms-bridge-action="cloud">"Cloud"</button>
                    <button type="button" data-comms-bridge-action="audit">"Audit receipt"</button>
                    <button type="button" data-comms-bridge-action="draft">"Draft all"</button>
                </div>
            </aside>
        </section>
    }
}

pub(super) fn comms_product_board(surface: ProductSurface) -> impl IntoView {
    match surface {
        ProductSurface::Messenger => view! {
            <section class="comms-product-board messenger-board" data-comms-product-board="true" data-comms-board-surface="Messenger" aria-label="Messenger command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"MESSENGER COMMAND"</p><h4>"Ops room thread with FD-001 workload evidence"</h4><span>"Fast operational chat for dogfooding FD-001 microservices on Oyatie Cloud, with evidence links and action extraction."</span></div>
                    <div class="comms-board-actions"><span class="status-chip warning">"2 unread"</span><button type="button" data-comms-action="thread-escalate">"Escalate"</button><button type="button" data-comms-action="thread-to-mail">"Promote to Mail"</button><button type="button" data-comms-action="thread-receipt">"Attach receipt"</button></div>
                </div>
                <div class="comms-board-grid">
                    <article class="thread-transcript-card"><p class="screen-anchor">"LIVE THREAD"</p><ol class="comms-transcript"><li><strong>"Ops bot"</strong><span>"Kubernetes runtime tier drift detected in cell-us-east-2."</span><em>"09:18 · unread"</em></li><li class="mine"><strong>"Tenant admin"</strong><span>"Link rollback runbook and notify Finance before close."</span><em>"09:22 · local draft"</em></li><li><strong>"Security reviewer"</strong><span>"Need audit-chain evidence before promotion."</span><em>"09:24 · evidence"</em></li></ol></article>
                    <article><p class="screen-anchor">"ACTION EXTRACTION"</p><div class="comms-action-list"><button type="button" data-comms-action="create-task"><strong>"Create task"</strong><span>"Rollback evidence owner · due 2.1h"</span></button><button type="button" data-comms-action="link-workflow"><strong>"Link workflow"</strong><span>"PROC-PAYROLL-CLOSE critical path"</span></button><button type="button" data-comms-action="thread-to-mail"><strong>"Draft formal mail"</strong><span>"CFO + SRE approval brief"</span></button></div></article>
                    <article><p class="screen-anchor">"PARTICIPANTS"</p><div class="comms-presence-grid"><span><em>"OP"</em><strong>"Ops bot"</strong><small>"online"</small></span><span><em>"SR"</em><strong>"Security"</strong><small>"watching"</small></span><span><em>"FL"</em><strong>"Finance"</strong><small>"mail owner"</small></span><span><em>"GV"</em><strong>"Governance"</strong><small>"council"</small></span></div></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Mail => view! {
            <section class="comms-product-board mail-board" data-comms-product-board="true" data-comms-board-surface="Mail" aria-label="Mail command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"MAIL COMMAND"</p><h4>"Formal approval brief composer"</h4><span>"Structured mail draft with recipients, subject, FD-001 workload evidence attachments, Oyatie Cloud cell context, approvals, and send preview."</span></div>
                    <div class="comms-board-actions"><span class="status-chip ai">"draft"</span><button type="button" data-comms-action="mail-preview">"Preview"</button><button type="button" data-comms-action="mail-attach">"Attach packet"</button><button type="button" data-comms-action="send-preview">"Send preview"</button></div>
                </div>
                <div class="comms-mail-compose-grid">
                    <article class="mail-envelope-card"><p class="screen-anchor">"ENVELOPE"</p><dl><div><dt>"From"</dt><dd>"Finance lead · Oyatie"</dd></div><div><dt>"To"</dt><dd>"CFO, SRE reviewer"</dd></div><div><dt>"CC"</dt><dd>"Governance council, Audit"</dd></div><div><dt>"Subject"</dt><dd>"Approval needed: payroll close + cloud rollback evidence"</dd></div></dl></article>
                    <article class="mail-body-card"><p class="screen-anchor">"DRAFT BODY"</p><div class="mail-paper"><strong>"Please review the April close packet before 18:00."</strong><p>"Payroll delta, HomeTax readiness, vendor exception, and Oyatie Cloud rollback evidence are attached as read-only receipts for the FD-001 tenant workload. No external send is enabled before live integration."</p><ol><li>"REC-PAY-2026-04-PARK"</li><li>"REC-CLOUD-MESH-4182"</li><li>"REC-WF-7741"</li></ol></div></article>
                    <article><p class="screen-anchor">"APPROVAL CHECKS"</p><div class="mail-checks"><span class="done">"Human reviewer required"</span><span class="done">"PIPA-safe body"</span><span class="review">"CFO signoff pending"</span><span>"External delivery disabled"</span></div></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Community => view! {
            <section class="comms-product-board community-board" data-comms-product-board="true" data-comms-board-surface="Community" aria-label="Community command workspace">
                <div class="comms-board-head">
                    <div><p class="screen-anchor">"COMMUNITY COMMAND"</p><h4>"Governance council publication"</h4><span>"Role-aware community post, voting, pinned Oyatie Cloud cell context, and moderation state for FD-001 tenant-workload coordination."</span></div>
                    <div class="comms-board-actions"><span class="status-chip success">"role-aware"</span><button type="button" data-comms-action="community-pin">"Pin"</button><button type="button" data-comms-action="community-poll">"Open poll"</button><button type="button" data-comms-action="publish-note">"Publish local"</button></div>
                </div>
                <div class="community-feed-grid">
                    <article class="community-post-card"><p class="screen-anchor">"PINNED POST"</p><div class="community-post-preview"><span>"Governance council"</span><h5>"April close governance digest"</h5><p>"Payroll blocker, withholding filing readiness, Oyatie Cloud rollback evidence, and reviewer assignments are summarized for role-visible FD-001 review."</p><div><button type="button" data-comms-action="community-upvote">"▲ 24"</button><button type="button" data-comms-action="community-comment">"8 comments"</button><button type="button" data-comms-action="community-save">"Save"</button></div></div></article>
                    <article><p class="screen-anchor">"AUDIENCE"</p><div class="community-audience-grid"><span><strong>"Finance"</strong><em>"required"</em></span><span><strong>"SRE"</strong><em>"review"</em></span><span><strong>"People Ops"</strong><em>"visible"</em></span><span><strong>"Vendors"</strong><em>"blocked"</em></span></div></article>
                    <article><p class="screen-anchor">"MODERATION"</p><dl class="community-moderation"><div><dt>"Policy"</dt><dd>"PIPA-safe"</dd></div><div><dt>"Evidence"</dt><dd>"3 receipts"</dd></div><div><dt>"Publish"</dt><dd>"local only"</dd></div></dl></article>
                </div>
            </section>
        }.into_any(),
        ProductSurface::Workflow => view! {
            <section class="comms-product-board" data-comms-product-board="true"><p>"Workflow route selected."</p></section>
        }.into_any(),
    }
}
