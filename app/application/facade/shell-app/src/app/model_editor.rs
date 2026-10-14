use leptos::prelude::*;

#[component]
fn RelationshipCardinalityHelp(id: &'static str) -> impl IntoView {
    view! {
        <div id=id>
            <p>"For this relationship type in this tenant, From is the source and To is the target. No links are required."</p>
            <p>"One to one: each source can link to at most one target; each target can link from at most one source."</p>
            <p>"One to many: each source can link to many targets; each target can link from at most one source."</p>
            <p>"Many to many: each source can link to many targets; each target can link from many sources."</p>
        </div>
    }
}

#[component]
pub(super) fn ModelDraftControls() -> impl IntoView {
    view! {
        <div id="live-ontology-draft-controls" class="foundry-draft-controls" data-live-results="true" hidden>
            <label for="live-ontology-draft-choice">"Model version"</label><select id="live-ontology-draft-choice" disabled></select>
            <button id="live-ontology-draft-more" type="button" hidden>"Load more drafts"</button>
            <button id="live-ontology-draft-new" type="button" disabled>"Create draft"</button>
            <button id="live-ontology-draft-save" type="button" disabled>"Save draft"</button>
            <button id="live-ontology-draft-reload" type="button" disabled>"Reload saved"</button>
            <button id="live-ontology-draft-validate" type="button" disabled>"Validate saved changes"</button>
            <p id="live-ontology-draft-state" role="status" aria-live="polite"></p>
        </div>
    }
}
#[component]
pub(super) fn ModelDraftInspector() -> impl IntoView {
    view! {
        <section id="live-ontology-draft-inspector" class="foundry-draft-inspector" aria-labelledby="live-ontology-draft-inspector-title" data-live-results="true" hidden>
            <h4 id="live-ontology-draft-inspector-title" tabindex="-1">"Draft declaration"</h4>
            <p id="live-ontology-draft-selection"></p>
            <form id="live-ontology-entity-edit" class="foundry-editor-form" hidden>
                <label for="live-ontology-entity-name">"Type display name"</label><input id="live-ontology-entity-name" type="text" maxlength="4096" />
                <label for="live-ontology-entity-revision">"Declaration revision"</label><input id="live-ontology-entity-revision" type="number" min="1" max="4294967295" />
                <p>"Changed existing declarations need a new revision. Native validation checks compatibility."</p>
                <button type="submit">"Apply type changes"</button>
            </form>
            <label for="live-ontology-property-choice">"Property declaration"</label><select id="live-ontology-property-choice" disabled></select>
            <form id="live-ontology-property-edit" class="foundry-editor-form" hidden>
                <label for="live-ontology-property-name">"Property name"</label><input id="live-ontology-property-name" type="text" maxlength="4096" />
                <label for="live-ontology-property-kind">"Value declaration"</label><select id="live-ontology-property-kind"><option value="legacy">"Legacy String (unchanged)"</option><option value="string">"String"</option><option value="integer">"Integer"</option><option value="double">"Double"</option><option value="boolean">"Boolean"</option><option value="date">"Date"</option><option value="timestamp">"Timestamp"</option></select>
                <label for="live-ontology-property-required">"Presence"</label><select id="live-ontology-property-required"><option value="required">"Required"</option><option value="optional">"Optional"</option></select>
                <p id="live-ontology-property-compatibility"></p><button type="submit">"Apply property changes"</button>
            </form>
            <form id="live-ontology-property-add" class="foundry-editor-form" hidden>
                <label for="live-ontology-new-property-name">"New property name"</label><input id="live-ontology-new-property-name" type="text" maxlength="4096" />
                <label for="live-ontology-new-property-kind">"New property value declaration"</label><select id="live-ontology-new-property-kind"><option value="string">"String"</option><option value="integer">"Integer"</option><option value="double">"Double"</option><option value="boolean">"Boolean"</option><option value="date">"Date"</option><option value="timestamp">"Timestamp"</option></select>
                <label for="live-ontology-new-property-required">"New property presence"</label><select id="live-ontology-new-property-required"><option value="optional">"Optional"</option><option value="required">"Required"</option></select>
                <button type="submit">"Add property"</button>
            </form>
            <form id="live-ontology-link-edit" class="foundry-editor-form" hidden>
                <label for="live-ontology-link-from">"From type (source)"</label><select id="live-ontology-link-from"></select>
                <label for="live-ontology-link-to">"To type (target)"</label><select id="live-ontology-link-to"></select>
                <label for="live-ontology-link-cardinality">"Cardinality"</label><select id="live-ontology-link-cardinality" aria-describedby="live-ontology-link-cardinality-help"><option value="one_to_one">"One to one"</option><option value="one_to_many">"One to many"</option><option value="many_to_many">"Many to many"</option></select>
                <RelationshipCardinalityHelp id="live-ontology-link-cardinality-help" />
                <label for="live-ontology-link-revision">"Declaration revision"</label><input id="live-ontology-link-revision" type="number" min="1" max="4294967295" />
                <p>"Relationship editing stays within this tenant. Cross-tenant intent is preserved as read-only metadata."</p><button type="submit">"Apply relationship changes"</button>
            </form>
            <fieldset><legend>"Add declarations"</legend>
                <form id="live-ontology-entity-add" class="foundry-editor-form"><label for="live-ontology-new-entity-id">"New type ID (ety_)"</label><input id="live-ontology-new-entity-id" type="text" maxlength="4096" /><label for="live-ontology-new-entity-name">"Display name"</label><input id="live-ontology-new-entity-name" type="text" maxlength="4096" /><button type="submit">"Add type"</button></form>
                <form id="live-ontology-link-add" class="foundry-editor-form"><label for="live-ontology-new-link-id">"New relationship ID (lty_)"</label><input id="live-ontology-new-link-id" type="text" maxlength="4096" /><label for="live-ontology-new-link-from">"From type (source)"</label><select id="live-ontology-new-link-from"></select><label for="live-ontology-new-link-to">"To type (target)"</label><select id="live-ontology-new-link-to"></select><label for="live-ontology-new-link-cardinality">"New relationship cardinality"</label><select id="live-ontology-new-link-cardinality" aria-describedby="live-ontology-new-link-cardinality-help"><option value="one_to_many">"One to many"</option><option value="one_to_one">"One to one"</option><option value="many_to_many">"Many to many"</option></select><RelationshipCardinalityHelp id="live-ontology-new-link-cardinality-help" /><button type="submit">"Add relationship"</button></form>
            </fieldset>
        </section>
    }
}
#[component]
pub(super) fn ModelDraftTray() -> impl IntoView {
    view! {
        <section id="live-ontology-draft-tray" class="foundry-draft-tray" aria-labelledby="live-ontology-draft-tray-title" data-live-results="true" hidden>
            <h4 id="live-ontology-draft-tray-title">"Draft changes"</h4><p id="live-ontology-draft-status" role="status" aria-live="polite"></p>
            <div id="live-ontology-draft-diff"></div><div id="live-ontology-draft-diagnostics"></div>
            <p>"Draft edits do not change object records. Schema validation checks the exact saved draft. Publishing is not available yet."</p>
        </section>
    }
}
