use leptos::prelude::*;

#[component]
pub(super) fn FoundryWorkspace() -> impl IntoView {
    view! {
        <section id="foundry-workspace" class="foundry-workspace" aria-labelledby="foundry-workspace-title" tabindex="-1">
            <header class="foundry-workspace-header">
                <div><h2 id="foundry-workspace-title">"Ontology workspace"</h2><p id="live-ontology-identity"></p></div>
                <span id="live-ontology-access-badge" class="foundry-read-label">"Sign-in required"</span>
            </header>
            <p id="live-ontology-access-notice" class="foundry-access-notice">"Sign in to explore your tenant's native model. "<a href="#oyatie-sign-in">"Sign in with a passkey"</a>" or "<a href="#ontology-command-console">"explore the labeled sample graph"</a>"."</p>
            <section id="live-ontology-explorer" class="live-ontology-explorer" aria-labelledby="live-ontology-title" aria-describedby="live-ontology-status" hidden>
                <h3 id="live-ontology-title" class="sr-only">"Native Ontology explorer"</h3>
                <div id="live-ontology-module-strip" class="foundry-module-strip">
                    <div class="foundry-taskbar" role="group" aria-label="Ontology task controls">
                    <div class="foundry-mode-bar" role="group" aria-label="Ontology workspace mode"><button type="button" id="live-ontology-mode-schema" aria-pressed="true" aria-controls="live-ontology-model">"Model"</button><button type="button" id="live-ontology-mode-objects" aria-pressed="false" aria-controls="live-ontology-objects">"Objects"</button></div>
                    <span id="live-ontology-object-type-controls" class="foundry-object-type-controls"><label for="live-ontology-type">"Object read type"</label><select id="live-ontology-type" required disabled></select></span>
                    <button type="button" id="live-ontology-types-refresh">"Refresh model"</button>
                    </div>
                    <p id="live-ontology-status" role="status" aria-live="polite">"Sign in to read your tenant's Ontology."</p>
                    <nav class="foundry-pane-nav" aria-label="Ontology panes"><a id="live-ontology-pane-library" href="#live-ontology-library-title">"Library"</a><a id="live-ontology-pane-workpane" href="#live-ontology-workpane">"Workspace"</a><a id="live-ontology-pane-inspector" href="#live-ontology-inspector-title">"Inspector"</a></nav>
                    <details class="foundry-capability-note"><summary>"Workspace capabilities"</summary>
                        <p>"Available now: type, relationship and reusable string declarations, object reads, selected relationship traversal in a one-hop table, object history and tenant audit. Owned model drafts require separate native model access. Source mappings, publication, an instance graph canvas, multi-hop exploration and scenarios require their native workflows."</p>
                        <p id="live-ontology-graph-help">"Arrows follow the declared from → to direction. Select a type or relationship to inspect it. Scroll or swipe to pan; use arrow keys when the canvas is focused. + and − zoom; Home fits the model."</p>
                        <p>"Draft edits do not change object records. Schema validation checks the exact saved draft. Publishing is not available yet."</p>
                    </details>
                </div>
                <div class="foundry-workspace-body">
                    <details id="live-ontology-library" class="foundry-library" open>
                        <summary id="live-ontology-library-title">"Model library"</summary>
                        <label for="live-ontology-search">"Search types, properties, relationships and value types"</label><input id="live-ontology-search" type="search" autocomplete="off" placeholder="Search model library" />
                        <p id="live-ontology-library-summary" role="status" aria-live="polite"></p>
                        <h4>"Object types"</h4><div id="live-ontology-library-types" data-live-results="true"></div>
                        <h4>"Relationships"</h4><div id="live-ontology-library-links" data-live-results="true"></div>
                        <h4>"Reusable value types"</h4><p id="live-ontology-values-note"></p><div id="live-ontology-library-values" data-live-results="true"></div>
                        <p class="foundry-library-help">"The graph shows type declarations. Objects are instances; source lineage is a separate view."</p>
                    </details>
                    <div id="live-ontology-workpane" class="foundry-main" role="region" aria-label="Current Ontology workpane" tabindex="-1">
                        <section id="live-ontology-model" aria-labelledby="live-ontology-model-title">
                            <div class="foundry-canvas-toolbar"><div><h4 id="live-ontology-model-title">"Type graph"</h4><p id="live-ontology-model-summary"></p></div>
                                <div role="group" aria-label="Model representation"><button type="button" id="live-ontology-view-graph" aria-pressed="true" aria-controls="live-ontology-graph">"Graph"</button><button type="button" id="live-ontology-view-table" aria-pressed="false" aria-controls="live-ontology-model-table">"Table"</button></div>
                                <div id="live-ontology-graph-controls" role="group" aria-label="Type graph zoom"><button type="button" id="live-ontology-zoom-out" aria-label="Zoom out">"−"</button><output id="live-ontology-zoom" aria-live="polite">"100%"</output><button type="button" id="live-ontology-zoom-in" aria-label="Zoom in">"+"</button><button type="button" id="live-ontology-fit">"Fit to view"</button></div>
                            </div>
                            <div id="live-ontology-graph" class="foundry-graph" tabindex="0" aria-label="Native object type graph" aria-describedby="live-ontology-graph-help" data-live-results="true"><div id="live-ontology-stage" class="foundry-graph-stage"><div id="live-ontology-scene" class="foundry-graph-scene"></div></div></div>
                            <div id="live-ontology-model-table" class="foundry-model-table" data-live-results="true" hidden></div>
                            <p id="live-ontology-model-note" class="foundry-model-note">"Current schema and relationship declarations are loaded separately. This view is not an atomic model snapshot."</p>
                        </section>
                        <section id="live-ontology-objects" aria-label="Native object instances" hidden>
            <form id="live-ontology-query" class="live-ontology-query">
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
            <div class="sign-in-actions"><button type="button" id="live-ontology-audit-read">"Read tenant audit"</button></div>
            <div id="live-ontology-page" data-live-results="true"></div>
            <div class="sign-in-actions"><button type="button" id="live-ontology-first" hidden>"First page"</button><button type="button" id="live-ontology-next" hidden>"Next page"</button></div>
            <section id="live-ontology-detail" aria-labelledby="live-ontology-detail-title" data-live-results="true" hidden>
                <h4 id="live-ontology-detail-title" tabindex="-1">"Object detail"</h4><div id="live-ontology-properties"></div>
                <button type="button" id="live-ontology-history-read">"Read object history"</button><div id="live-ontology-history"></div>
                <section aria-labelledby="live-ontology-relationship-title">
                    <h5 id="live-ontology-relationship-title">"Related objects"</h5><p id="live-ontology-relationship-root"></p>
                    <div class="live-ontology-query">
                        <label for="live-ontology-relationship-direction">"Direction from this object"</label><select id="live-ontology-relationship-direction"><option value="outbound">"Outbound · from this object"</option><option value="inbound">"Inbound · to this object"</option></select>
                        <label for="live-ontology-relationship-type">"Accepted relationship"</label><select id="live-ontology-relationship-type" required></select>
                        <button type="button" id="live-ontology-relationship-read" disabled>"Read related objects"</button>
                    </div>
                    <p id="live-ontology-relationship-status" role="status" aria-live="polite"></p><div id="live-ontology-relationship-results" data-live-results="true"></div>
                    <p>"A bounded one-hop table of object instances. The Model canvas shows type declarations. Each read checks current access; choose Read related objects again for a fresh request."</p>
                </section>
            </section>
            <section id="live-ontology-audit" aria-labelledby="live-ontology-audit-title" data-live-results="true" hidden><h4 id="live-ontology-audit-title">"Native tenant audit"</h4><div id="live-ontology-audit-rows"></div></section>
                        </section>
                    </div>
                    <details id="live-ontology-inspector" class="foundry-inspector" open><summary id="live-ontology-inspector-title">"Inspector"<span id="live-ontology-inspector-context"><span id="live-ontology-context-selected"></span><span id="live-ontology-context-identity"></span><span id="live-ontology-context-name"></span></span></summary>
                        <details id="live-ontology-selection-details" class="foundry-selection-details" hidden><summary>"Full selection details"</summary><dl id="live-ontology-selection-values"></dl></details>
                        <super::model_editor::ModelDraftInspector />
                        <super::named_values::NamedValueInspector />
            <section id="live-ontology-schema" aria-labelledby="live-ontology-schema-title" data-live-results="true" hidden>
                <h4 id="live-ontology-schema-title">"Native entity schema"</h4>
                <p>"The selected type declaration is shown. Requested object revisions are set separately."</p>
                <p id="live-ontology-schema-summary"></p><div id="live-ontology-schema-properties"></div>
            </section>
                        <section id="live-ontology-link" aria-labelledby="live-ontology-link-title" data-live-results="true" hidden><h4 id="live-ontology-link-title">"Native relationship declaration"</h4><div id="live-ontology-link-properties"></div><p>"Cross-tenant intent describes the model. It does not grant permission to read another tenant."</p></section>
                        <p id="live-ontology-inspector-empty">"Select a type or relationship to inspect its declaration."</p>
                    </details>
                    <super::model_editor::ModelDraftTray />
                </div>
            </section>
        </section>
    }
}
