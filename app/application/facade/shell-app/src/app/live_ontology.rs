use leptos::prelude::*;

#[component]
pub(super) fn LiveOntologyExplorer() -> impl IntoView {
    view! {
        <section id="live-ontology-explorer" class="verified-ontology-card live-ontology-explorer" aria-labelledby="live-ontology-title" aria-describedby="live-ontology-status" hidden>
            <h3 id="live-ontology-title">"Native Ontology explorer"</h3>
            <p id="live-ontology-identity"></p>
            <p>"Live reads for this verified account. The sample dashboard below has a separate sample context. Reads do not change objects."</p>
            <form id="live-ontology-query" class="live-ontology-query">
                <label for="live-ontology-type">"Entity type"</label><select id="live-ontology-type" required disabled></select>
                <label for="live-ontology-revision">"Requested schema revision"</label><input id="live-ontology-revision" type="number" min="0" max="4294967295" value="1" required />
                <label for="live-ontology-limit">"Objects per page"</label><input id="live-ontology-limit" type="number" min="1" max="1000" value="25" required />
                <label for="live-ontology-property">"Filter property (optional)"</label><input id="live-ontology-property" type="text" autocomplete="off" />
                <label for="live-ontology-comparison">"Comparison"</label><select id="live-ontology-comparison"><option value="none">"No filter"</option><option value="equals">"Equals"</option><option value="range">"Inclusive range"</option></select>
                <label for="live-ontology-kind">"Filter value kind"</label><select id="live-ontology-kind"><option value="text">"Text"</option><option value="int">"Integer"</option><option value="bool">"Boolean"</option></select>
                <label for="live-ontology-from">"Value / range start"</label><input id="live-ontology-from" type="text" aria-describedby="live-ontology-filter-help" />
                <label for="live-ontology-to">"Range end"</label><input id="live-ontology-to" type="text" aria-describedby="live-ontology-filter-help" />
                <p id="live-ontology-filter-help">"Text filters accept ASCII values without spaces, &, #, or quotes. Boolean values are true or false. Unsupported filters are refused."</p>
                <button type="submit" id="live-ontology-read" disabled>"Read objects"</button>
            </form>
            <div class="sign-in-actions"><button type="button" id="live-ontology-types-refresh">"Refresh schemas"</button><button type="button" id="live-ontology-audit-read">"Read tenant audit"</button></div>
            <p id="live-ontology-status" role="status" aria-live="polite">"Sign in to read your tenant's Ontology."</p>
            <section id="live-ontology-schema" aria-labelledby="live-ontology-schema-title" data-live-results="true" hidden>
                <h4 id="live-ontology-schema-title">"Native entity schema"</h4>
                <p>"The current accepted declaration is shown for the selected type. Requested object revisions are set separately."</p>
                <p id="live-ontology-schema-summary"></p><div id="live-ontology-schema-properties"></div>
            </section>
            <div id="live-ontology-page" data-live-results="true"></div>
            <div class="sign-in-actions"><button type="button" id="live-ontology-first" hidden>"First page"</button><button type="button" id="live-ontology-next" hidden>"Next page"</button></div>
            <section id="live-ontology-detail" aria-labelledby="live-ontology-detail-title" data-live-results="true" hidden>
                <h4 id="live-ontology-detail-title" tabindex="-1">"Object detail"</h4><div id="live-ontology-properties"></div>
                <button type="button" id="live-ontology-history-read">"Read object history"</button><div id="live-ontology-history"></div>
            </section>
            <section id="live-ontology-audit" aria-labelledby="live-ontology-audit-title" data-live-results="true" hidden><h4 id="live-ontology-audit-title">"Native tenant audit"</h4><div id="live-ontology-audit-rows"></div></section>
        </section>
    }
}
