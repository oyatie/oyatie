//! Service level objectives as typed IR, and the pure renderer that turns
//! them into the checked-in OpenSLO payloads.
//!
//! The payloads under `app/foundry/observability/slos/` are OUTPUTS. They are
//! byte-compared against this renderer by the suite, so a hand-edited payload
//! fails rather than drifting from the objective the code believes it serves
//! — an SLO that disagrees with its own definition is worse than no SLO,
//! because it reports confidently.
//!
//! Rendering is string assembly with no serialization dependency, following
//! the platform's own precedent for hand-rendered exposition formats.

use std::collections::BTreeSet;

/// One objective, in the shape the corpus grammar expects.
pub struct SloSpec {
    /// Kebab file stem; the payload is `<name>.generated.openslo.yaml`.
    pub name: &'static str, // data_class: INTERNAL_ONLY
    pub display_name: &'static str, // data_class: INTERNAL_ONLY
    pub sli_class: &'static str,    // data_class: INTERNAL_ONLY
    pub description: &'static str,  // data_class: INTERNAL_ONLY
    /// Prometheus expression for the numerator.
    pub good_query: &'static str, // data_class: INTERNAL_ONLY
    /// Prometheus expression for the denominator.
    pub total_query: &'static str, // data_class: INTERNAL_ONLY
    pub target: &'static str,       // data_class: INTERNAL_ONLY
    pub objective_display: &'static str, // data_class: INTERNAL_ONLY
    /// Whether the ratio's sources are cumulative counters. False for a
    /// gauge-backed objective: declaring a gauge cumulative tells a consumer
    /// to `rate()` something that does not monotonically increase.
    pub counter: bool, // data_class: INTERNAL_ONLY
}

impl SloSpec {
    /// The metrics this objective names, SCANNED OUT OF THE QUERIES rather
    /// than declared beside them. An earlier revision kept a hand-maintained
    /// list: changing a query while leaving the list stale produced an
    /// objective naming a deleted metric with the suite still green, because
    /// the check validated its own input instead of the thing it existed to
    /// guarantee.
    pub fn referenced_metrics(&self) -> BTreeSet<String> {
        const PREFIX: &str = "foundry_";
        let mut found = BTreeSet::new();
        for query in [self.good_query, self.total_query] {
            let mut consumed = 0;
            while let Some(at) = query[consumed..].find(PREFIX) {
                let start = consumed + at;
                // A hit must begin a token. Without this, the recording-rule
                // idiom `job:foundry_read_served_total:rate5m` yields the
                // exported substring and the objective passes while naming a
                // series the process never emits. Rejecting it leaves that
                // rule UNVALIDATED rather than validated: this scanner can
                // only check names the process itself exports, and a rule
                // lives in the evaluator's config, not here. An objective
                // built entirely from rules therefore recovers nothing and
                // fails the emptiness check, which is the honest answer.
                let preceded_by_token_char = query[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':');
                let tail = &query[start..];
                let end = tail
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(tail.len());
                if !preceded_by_token_char {
                    // Nor may it be a prefix of a longer recording-rule name.
                    let followed_by_colon = tail[end..].starts_with(':');
                    if !followed_by_colon {
                        found.insert(tail[..end].to_owned());
                    }
                }
                consumed = start + end;
            }
        }
        found
    }
}

const SERVICE: &str = "foundry-ontology";

/// The objectives this vertical declares. Each names only metrics
/// `metrics::exported_metric_names` reports.
/// NO FRESHNESS OBJECTIVE, deliberately.
///
/// `foundry_projection_lag` cannot currently take a non-zero value: it is
/// `head - applied_ordinal` where `head` reads the per-tenant `entries`
/// mirror, and that mirror is built once in `compose` and never appended to
/// afterwards — `write_handles` lends out the log, the denial trail and the
/// projection, never the mirror. So `head` is frozen at the boot ordinal
/// while `applied_ordinal` only grows, and the saturating subtraction is
/// identically zero. An objective over it would be declared coverage
/// providing none, which is the failure this module exists to prevent; it
/// would simply never breach instead of permanently breaching.
///
/// A real freshness objective needs the lag measured against the DURABLE
/// log head rather than the boot mirror. That is a change to what lag
/// means process-wide, which belongs with the lane that owns readiness and
/// the boot decision table, not to a lane about what the exposition counts.
pub static SLOS: &[SloSpec] = &[
    SloSpec {
        name: "ontology-submit-availability",
        display_name: "foundry-ontology — Action submission availability",
        sli_class: "availability",
        description: "A submission is good when the writer accepted it into the log. A \
                      policy refusal counts against this objective deliberately: from the \
                      caller's side an Action it was entitled to submit and could not is \
                      an outage, and hiding refusals here would make the number flatter \
                      than the service.",
        good_query: "sum(rate(foundry_action_submit_served_total[5m]))",
        total_query: "sum(rate(foundry_action_submit_served_total[5m])) + \
                      sum(rate(foundry_action_submit_refused_total[5m]))",
        target: "0.99",
        objective_display: "99% of submissions accepted over 30d",
        counter: true,
    },
    SloSpec {
        name: "ontology-read-availability",
        display_name: "foundry-ontology — read surface availability",
        sli_class: "availability",
        description: "A read is good when the projection answered it. Refusals — absent \
                      credential, policy denial, unusable revision pin — are counted, so \
                      a surface that refuses everything cannot report itself available.",
        good_query: "sum(rate(foundry_read_served_total[5m]))",
        total_query: "sum(rate(foundry_read_served_total[5m])) + \
                      sum(rate(foundry_read_refused_total[5m]))",
        target: "0.995",
        objective_display: "99.5% of reads answered over 30d",
        counter: true,
    },
];

/// Render one objective as its OpenSLO payload. Pure: the same spec always
/// produces the same bytes, which is what makes the golden comparison a
/// test rather than a snapshot.
pub fn render_openslo(spec: &SloSpec) -> String {
    format!(
        "# Generated from `app/foundry/facade/ontology-app/src/slo.rs`. Do not hand-edit:\n\
         # the suite compares this file byte-for-byte against the renderer.\n\
         apiVersion: openslo/v1\n\
         kind: SLO\n\
         metadata:\n  \
           name: {name}\n  \
           displayName: \"{display}\"\n  \
           labels:\n    \
             microservice: {service}\n    \
             sli_class: {sli_class}\n    \
             plane: app\n    \
             owner_team: foundry\n\
         spec:\n  \
           service: {service}\n  \
           description: |\n{description}\n  \
           indicator:\n    \
             metadata:\n      \
               name: {name}-indicator\n    \
             spec:\n      \
               ratioMetric:\n        \
                 counter: {counter}\n        \
                 good:\n          \
                   metricSource:\n            \
                     type: Prometheus\n            \
                     spec:\n              \
                       query: |\n                {good}\n        \
                 total:\n          \
                   metricSource:\n            \
                     type: Prometheus\n            \
                     spec:\n              \
                       query: |\n                {total}\n  \
           objectives:\n    \
             - target: {target}\n      \
               displayName: \"{objective_display}\"\n  \
           timeWindow:\n    \
             - duration: 30d\n      \
               isRolling: true\n  \
           budgetingMethod: Occurrences\n",
        name = spec.name,
        display = spec.display_name,
        service = SERVICE,
        sli_class = spec.sli_class,
        description = wrap_block(spec.description),
        counter = spec.counter,
        good = spec.good_query,
        total = spec.total_query,
        target = spec.target,
        objective_display = spec.objective_display,
    )
}

/// Indent a description into a YAML literal block at four spaces.
fn wrap_block(text: &str) -> String {
    text.split_whitespace()
        .fold(Vec::<String>::new(), |mut lines, word| {
            match lines.last_mut() {
                Some(line) if line.len() + 1 + word.len() <= 72 => {
                    line.push(' ');
                    line.push_str(word);
                }
                _ => lines.push(format!("    {word}")),
            }
            lines
        })
        .join("\n")
}
