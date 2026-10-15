use leptos::prelude::*;

#[component]
pub(super) fn NamedValueInspector() -> impl IntoView {
    view! {
        <section id="live-ontology-value" aria-labelledby="live-ontology-value-title" data-live-results="true" hidden>
            <h4 id="live-ontology-value-title" tabindex="-1">"Reusable value type"</h4>
            <div id="live-ontology-value-properties"></div>
            <p>"A reusable string type restricts a root property to its exact members. Its accepted definition is immutable. Arrays and struct fields cannot adopt a reusable type."</p>
        </section>
    }
}

#[component]
pub(super) fn NamedValueForms() -> impl IntoView {
    view! {
        <form id="live-ontology-value-edit" class="foundry-editor-form" hidden>
            <label for="live-ontology-value-id">"Value type ID (vty_)"</label><input id="live-ontology-value-id" type="text" maxlength="4096" />
            <label for="live-ontology-value-revision">"Value type revision"</label><input id="live-ontology-value-revision" type="number" min="0" max="4294967295" />
            <label for="live-ontology-value-name">"Value type display name"</label><input id="live-ontology-value-name" type="text" maxlength="4096" />
            <div id="live-ontology-value-members" class="foundry-value-form" aria-describedby="live-ontology-value-member-help"></div>
            <p id="live-ontology-value-member-help">"Enter each allowed value separately. Spaces, line breaks and case are significant. Values must be unique and contain text. New definitions use revision 1. Save and validate to check native rules."</p>
            <p id="live-ontology-value-compatibility"></p><button type="submit">"Apply value type changes"</button>
            <button id="live-ontology-value-restore" type="button" hidden>"Restore published definition"</button>
            <button id="live-ontology-value-remove" type="button" hidden>"Remove duplicate definition"</button>
        </form>
        <details class="foundry-add-declarations"><summary>"Create reusable value type"</summary>
            <form id="live-ontology-value-add" class="foundry-editor-form">
                <label for="live-ontology-new-value-id">"New value type ID (vty_)"</label><input id="live-ontology-new-value-id" type="text" maxlength="4096" />
                <label for="live-ontology-new-value-name">"New value type display name"</label><input id="live-ontology-new-value-name" type="text" maxlength="4096" />
                <div id="live-ontology-new-value-members" class="foundry-value-form"></div>
                <p>"Create once, then choose this exact ID and revision on new optional properties. Accepted definitions and published property types remain fixed."</p><button type="submit">"Add reusable value type"</button>
            </form>
        </details>
    }
}
