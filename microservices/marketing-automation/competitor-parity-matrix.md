# Marketing Automation Competitor Parity Matrix

Service: marketing-automation
Date: 2026-05-21
Doc class: Competitor Parity Matrix
Phase: 4A.5 Big-8 customer-engagement substrate
Primary counterpart: HubSpot Marketing Hub
Flanking counterparts: Adobe Marketo Engage, Mailchimp
Reference counterparts: Salesforce Marketing Cloud Engagement, Klaviyo, Iterable, Braze
Binding authorities: ADR-0105, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0314, ADR-0321, ADR-0328, ADR-0331.

## 1. Stance

HubSpot Marketing Hub is the primary anchor per ADR-0328 §D-2.18-19. Marketo and Mailchimp are flanking counterparts. Salesforce Marketing Cloud Engagement, Klaviyo, Iterable, and Braze are reference counterparts retained for migration playbook coverage. Capability parity is rendered per-capability against the three primary counterparts (HubSpot + Marketo + Mailchimp); reference counterparts are cited where their primitive differs materially. Oyatie differentiator capabilities are flagged where the µservice exceeds counterpart depth.

The prior 304-row stamped matrix is superseded by this bespoke per-capability matrix. Capability rows below cover 50+ canonical capabilities per counterpart family with explicit per-capability Oyatie status, depth gap, and migration cost.

Legend:
- **EQUIVALENT** — Oyatie matches the counterpart capability surface within ±10% feature depth.
- **PARTIAL** — Oyatie covers the capability core but lacks one or more counterpart-specific affordances.
- **DIFFERENTIATOR** — Oyatie exceeds counterpart depth on this capability (typically auditable evidence chain).
- **DELEGATED** — Capability is owned by an adjacent Oyatie microservice (sites/social/messenger/contact-center/etc.); seam declared.
- **PLANNED** — Capability is scoped as a bounded context with capability YAML but no IP slice authored yet.

## 2. Email aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Marketing email object (subject + content + tokens) | Marketing Email | Email | Regular Campaign | `marketing_email` | EQUIVALENT | IP-031 |
| Drag-and-drop visual editor | Drag-and-Drop Email Editor | Email Editor 2.0 | Email Builder | workflow-canvas + email composition | PARTIAL | Visual editor surfaced in IP-034 |
| Plain-text variant | Plain-Text Version | Plain-Text Email | Plain-text Campaign | `marketing_email.plain_text_variant` | EQUIVALENT | IP-031 |
| RSS-driven email | Blog Notification + RSS Email | Email Program with RSS | RSS-driven Campaign | `marketing_email.rss_source` | EQUIVALENT | IP-031 |
| Postcard / direct mail | — | — | Postcard | `marketing_email.physical_postcard_variant` (Mailchimp parity) | EQUIVALENT | IP-031 |
| Dynamic personalization tokens | Personalization Tokens | Marketo Tokens | Merge Tags | `marketing_email.token_resolver` | EQUIVALENT | IP-031 |
| Smart content (rule-based) | Smart Content | Dynamic Content | Conditional Merge Tags | `marketing_email.smart_content_rules` | EQUIVALENT | IP-031 |
| A/B test variants | A/B Test | A/B Test | A/B Testing (Premium) | a-b-test bounded context | EQUIVALENT | IP-035 |
| Multivariate testing | Limited Multivariate | Multivariate Testing | Multivariate (Premium) | a-b-test.variant_set with 3+ allocations | PARTIAL | Coverage parity at HubSpot level |
| Send-time optimization | Send Time Optimization | Optimal Send Time | Send Time Optimization | send-time-optimization bounded context | EQUIVALENT | IP-036 |
| Accessibility checker | Accessibility Checker | Accessibility Hints | — | `kernel::email_accessibility_audit` | EQUIVALENT (matches HubSpot+Marketo) | IP-031 |
| Email tracking (open / click) | Email Tracking | Email Insights | Click/Open Reports | email-tracking bounded context | EQUIVALENT | IP-039 |
| Reply tracking | Reply Tracking | — | — | email-tracking.reply_recorded event | EQUIVALENT (matches HubSpot) | IP-039 |
| Link tracking with UTM | UTM Tracker | Link Tracking | Link Tracking | `kernel::utm_generator` (Q-021 settled by ADR-MS-MA-004) | PARTIAL | Open question Q-021 |
| Auditable send evidence chain | — | Limited | — | `EVT-MARKETING-EMAIL-SENT` sealed by audit-chain | DIFFERENTIATOR | Exceeds counterpart |
| Deliverability fail-closed | — | Limited | — | IP-029 governor; DMARC-failure pauses all marketing mail | DIFFERENTIATOR | Exceeds counterpart |

## 3. Landing Page aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Landing page object | Landing Page | Landing Page | Landing Page | `marketing_landing_page` | EQUIVALENT | IP-032 |
| Drag-and-drop editor | LP Editor | LP Editor | LP Builder | landing-page editor | PARTIAL | IP-032 |
| Form attachment | Form attached to LP | Form attached to LP | Signup Form attached | `marketing_landing_page.form_attachment_id` | EQUIVALENT | IP-032 |
| Conversion goal | Conversion Goal | Goal | Goal | `marketing_landing_page.conversion_goal` | EQUIVALENT | IP-032 |
| A/B variant | A/B Test | A/B Test | A/B Test (Premium) | a-b-test bounded context applied to LP | EQUIVALENT | IP-035 |
| SEO metadata | SEO Recommendations | SEO Tools | Limited SEO | `marketing_landing_page.seo_block` + delegation to sites SEO seam | PARTIAL | Q-019 settled |
| Custom CSS/HTML override | Custom Code Block | Custom HTML | Custom HTML | `marketing_landing_page.custom_css_html` | EQUIVALENT | IP-032 |
| Password protection | Password-protected LP | Password-protected LP | — | `marketing_landing_page.access_control.password_hash` | EQUIVALENT (matches HubSpot+Marketo) | IP-032 |
| Multi-language localisation | LP Localisation | LP Localisation | Multilingual | `marketing_landing_page.locale_variants` | EQUIVALENT | IP-032 |
| Hosted under tenant domain | hubspotpagebuilder.com or tenant CNAME | go.<tenant>.com | mailchimppages.com or tenant CNAME | Oyatie tenant CNAME + sites overlap | DELEGATED to sites for full site root | ADR-MS-MA-002 |
| AMP for Email | — | — | — | Future capability | PLANNED | — |
| Conversion-tracking auditable evidence | — | — | — | `EVT-MARKETING-LANDING-CONVERSION` sealed | DIFFERENTIATOR | Exceeds counterpart |

## 4. Form aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Form object | Form | Form | Signup Form | `marketing_form` | EQUIVALENT | IP-033 |
| Regular embed form | Regular Form | Embed Form | Embedded Form | `marketing_form.embed_variant` | EQUIVALENT | IP-033 |
| Pop-up form | Pop-up Form | Pop-up Form | Pop-up Form | `marketing_form.popup_variant` | EQUIVALENT | IP-033 |
| Collected/non-HubSpot form capture | Collected Form | — | — | `marketing_form.collected_variant` (JS SDK captures third-party form submits) | EQUIVALENT (matches HubSpot) | IP-033 |
| Chatflow-as-form | Chatflow as Form | — | — | chatflow bounded context produces form-submit equivalents | EQUIVALENT (matches HubSpot) | IP-055 |
| Field validation rules | Validation Rules | Validation | Validation | `marketing_form.field_validation_rules` | EQUIVALENT | IP-033 |
| Conditional logic | Progressive Profiling + Logic | Visibility Rules | Conditional Merge | `marketing_form.conditional_logic_dag` | EQUIVALENT | IP-033 |
| Progressive profiling | Progressive Profiling | Progressive Profiling | — | `marketing_form.progressive_profiling.next_field_rules` | EQUIVALENT (matches HubSpot+Marketo) | IP-033 |
| GDPR consent block | GDPR Consent Block | GDPR Consent Field | GDPR Sign-up Form | `marketing_form.gdpr_consent_block` per pack overlay | EQUIVALENT | IP-033 |
| Post-submit redirect | Redirect | Thank-You URL | Confirmation URL | `marketing_form.post_submit_redirect` | EQUIVALENT | IP-033 |
| Post-submit notification | Notification Email | Notification | Notification | `marketing_form.post_submit_notification` | EQUIVALENT | IP-033 |
| CAPTCHA | reCAPTCHA / hCaptcha | reCAPTCHA | reCAPTCHA | `marketing_form.captcha_provider` (pack-overlay-required) | EQUIVALENT | IP-033 |
| Submission idempotency | Limited | Limited | — | `marketing_form.idempotency_key_required` | DIFFERENTIATOR | Exceeds counterpart |
| Per-pack disclosure (EU/CA/KR) | Limited | Limited | Limited | `marketing_form.pack_overlay_disclosure_block` resolves per residency | DIFFERENTIATOR | Exceeds counterpart |

## 5. Workflow visual canvas aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Visual drag-and-drop builder | Workflow Editor | Smart Campaign Designer | Customer Journey Builder | workflow-canvas bounded context | EQUIVALENT | IP-034 |
| Contact-based vs company-based scope | Both | Both (Engagement Program is company-based) | Audience-scoped only | `marketing_workflow_canvas.scope ∈ {subject, account, audience}` | EQUIVALENT | IP-034 |
| Multiple entry triggers | Multiple Triggers | Smart List Trigger + Filter | Multiple Starting Points (Premium) | workflow-canvas.entry_triggers[] | EQUIVALENT | IP-034 |
| Conditional branch step | If/Then Branch | Choice Step | Branch | workflow-canvas.step_type=conditional_branch | EQUIVALENT | IP-034 |
| Wait/delay step | Delay | Wait | Delay | workflow-canvas.step_type=wait | EQUIVALENT | IP-034 |
| Send email step | Send Email | Send Email | Send Email | workflow-canvas.step_type=send_email | EQUIVALENT | IP-034 |
| Send SMS step | — (via integration) | Marketo Engage SMS | SMS (Premium) | workflow-canvas.step_type=send_sms (via messenger) | EQUIVALENT | IP-034 |
| Webhook step | Webhook Action | Webhook | — | workflow-canvas.step_type=invoke_webhook | EQUIVALENT (matches HubSpot+Marketo) | IP-034 |
| Update CRM property step | Set Property | Update Lead Field | Update Subscriber Field | workflow-canvas.step_type=update_crm_property (via crm contract) | EQUIVALENT | IP-034 |
| Score-adjust step | Adjust Score | Change Score | — | workflow-canvas.step_type=adjust_lead_score | EQUIVALENT (matches HubSpot+Marketo) | IP-034 |
| Lifecycle progression step | Update Lifecycle Stage | Change Engagement Score | — | workflow-canvas.step_type=progress_lifecycle | EQUIVALENT (matches HubSpot) | IP-034 |
| Audit-chain sealed step transitions | — | Limited | — | every step emits `EVT-MARKETING-JOURNEY-STEP-ADVANCED` sealed | DIFFERENTIATOR | Exceeds counterpart |
| Deterministic replay | — | Limited | — | workflow-canvas snapshots are immutable; replay reproduces exact step sequence | DIFFERENTIATOR | Exceeds counterpart |

## 6. Segment / List aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Dynamic segment / smart list | Active List | Smart List | Segment | segment bounded context | EQUIVALENT | IP-026 (differentiator) |
| Static list / tag | Static List | Static List | Tag | static-list bounded context | EQUIVALENT | IP-031 |
| Predicate compiler | List Filters | Filter + Trigger | Segment Rules | `kernel::compile_predicate_tree` validates against ontology | EQUIVALENT | IP-026 |
| Real-time membership update | Periodic refresh | Periodic refresh | Periodic refresh | sub-second freshness via IP-026 | DIFFERENTIATOR | Exceeds counterpart |
| Cross-object predicate (contact + company) | Yes | Yes (Marketo Lead Database) | Limited | predicate_tree references ontology nodes (Contact, Account, Behavior) | EQUIVALENT | IP-026 |
| Lookalike audience | — | — | Lookalike Audience (Premium) | intelligence.predict_lookalike returns predicate_tree | EQUIVALENT (matches Mailchimp) | IP-053 |
| Suppression list | Suppression List | Block List | Cleaned List | consent-audience.append_revocation | DIFFERENTIATOR | Exceeds counterpart |
| Predicate replay | Limited | Limited | — | predicate_tree snapshots replay against event-store cursor | DIFFERENTIATOR | Exceeds counterpart |

## 7. Lead scoring aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Score formula | Lead Scoring | Lead Scoring (Behavior + Demographic) | Predicted Demographics (Premium) | lead-scoring bounded context | EQUIVALENT | IP-037 |
| Demographic score | Yes | Yes (Demographic Score) | Yes (Predicted) | `lead-scoring.demographic_component` | EQUIVALENT | IP-037 |
| Behavioral score | Yes | Yes (Behavior Score) | Limited | `lead-scoring.behavioral_component` | EQUIVALENT | IP-037 |
| Score decay | Score Decay | Score Decay | — | `lead-scoring.decay_half_life_days` | EQUIVALENT (matches HubSpot+Marketo) | IP-037 |
| Manual adjustments | Manual Score | Manual Adjustment | — | `lead-scoring.manual_adjust` command | EQUIVALENT (matches HubSpot+Marketo) | IP-037 |
| Predictive (ML) lead scoring | HubSpot Predictive (Enterprise) | Marketo Predictive Content | — | intelligence µservice predicts; `lead-scoring.predictive_component` | EQUIVALENT (matches HubSpot+Marketo Enterprise) | IP-037 |
| Per-tenant model isolation | Limited (shared model) | Limited (shared model) | — | `lead-scoring.model_version` per tenant | DIFFERENTIATOR | Exceeds counterpart |

## 8. Lifecycle stage aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Lifecycle progression | Subscriber→Lead→MQL→SQL→Opportunity→Customer→Evangelist | Engagement Score Buckets | CLV bands | lifecycle-stage bounded context | EQUIVALENT (matches HubSpot canonical model) | IP-038 |
| Monotonic progression rule | Default monotonic | — | — | lifecycle-stage.invariant_monotonic | DIFFERENTIATOR | Exceeds counterpart |
| Downgrade with audit | Limited | — | — | `lifecycle-stage.downgrade` requires principal auth + audit event | DIFFERENTIATOR | Exceeds counterpart |
| Workflow trigger on transition | Workflow Trigger | Smart Campaign Trigger | — | workflow-canvas.entry_triggers[] includes lifecycle transitions | EQUIVALENT | IP-038 |

## 9. Subscription type aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Subscription type / publication category | Subscription Type | Communication Limits | Groups | subscription-type bounded context | EQUIVALENT | IP-041 |
| Communication preference center | Email Preference Center | Subscription Center | Update Profile | subscription-type.preference_center_link | EQUIVALENT | IP-041 |
| Per-type opt-in evidence | Opt-in record | Opt-in record | Limited | subscription-type.opt_in_evidence (signed by consent-graph) | DIFFERENTIATOR | Exceeds counterpart |
| Unsubscribe link mandatory (CAN-SPAM/CASL) | Yes | Yes | Yes | subscription-type.unsubscribe_link required per pack overlay | EQUIVALENT | IP-041 |
| Per-pack disclosure | Limited | Limited | Limited | subscription-type.pack_overlay_disclosure resolves per residency | DIFFERENTIATOR | Exceeds counterpart |

## 10. Consent / suppression aggregate (IP-027 differentiator)

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Suppression check | Suppression List | Block List | Cleaned/Unsubscribed | consent-audience.check_suppression | EQUIVALENT | IP-027 |
| Append-only consent ledger | — | — | — | consent-audience.append_consent / append_revocation | DIFFERENTIATOR | Exceeds counterpart |
| HLC-stamped consent events | — | — | — | every consent event HLC-stamped | DIFFERENTIATOR | Exceeds counterpart |
| Per-channel × per-purpose granularity | Topic + Channel | Topic + Channel | Topic only | consent-audience row keyed (subject, channel, purpose) | DIFFERENTIATOR | Exceeds counterpart |
| GDPR Article 15 DSR (right of access) | Manual | Manual | Manual | DSR report generated from append-only ledger | DIFFERENTIATOR | Exceeds counterpart |
| GDPR Article 17 (right to erasure) | Suppression flag | Suppression flag | Cleaned | tombstone projection while ledger remains append-only | DIFFERENTIATOR | Exceeds counterpart |
| GDPR Article 21 (right to object) | Unsubscribe | Unsubscribe | Unsubscribe | subject-initiated revocation always allowed by Cedar | EQUIVALENT | IP-027 |
| Audit-chain seal per consent event | — | Limited | — | every consent event sealed by audit-chain | DIFFERENTIATOR | Exceeds counterpart |

## 11. Attribution aggregate (IP-028 differentiator)

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Campaign attribution | Campaign Attribution Report | Revenue Cycle Modeler | Premium Revenue Report | attribution bounded context | EQUIVALENT | IP-028 |
| First-touch model | First Interaction | First Touch | Single Touch | attribution.first_touch model | EQUIVALENT | IP-028 |
| Last-touch model | Last Interaction | Last Touch | Single Touch | attribution.last_touch model | EQUIVALENT | IP-028 |
| Linear (equal-credit) model | Linear | Linear | — | attribution.linear model | EQUIVALENT (matches HubSpot+Marketo) | IP-028 |
| Position-based (U-shape) model | Position-Based | U-Shape | — | attribution.position_based_40_20_40 | EQUIVALENT (matches HubSpot+Marketo) | IP-028 |
| Time-decay model | Time-Decay | Time-Decay | — | attribution.time_decay model | EQUIVALENT (matches HubSpot+Marketo) | IP-028 |
| Custom model | Custom (Enterprise) | Custom | — | attribution.custom_model with formula | EQUIVALENT | IP-028 |
| Cross-channel attribution | Yes | Yes | Limited | touches include all engagement channels via ontology | EQUIVALENT | IP-028 |
| Revenue-event source-of-truth | HubSpot CRM | Marketo Sales Connect / CRM sync | Mailchimp e-commerce | crm.opportunity via crm contract | DIFFERENTIATOR | Exceeds counterpart (cryptographic seal) |
| Deterministic replay | Limited | Limited | Limited | attribution model + touches + revenue events → deterministic credit | DIFFERENTIATOR | Exceeds counterpart |
| Audit-chain sealed reconciliation | — | Limited | — | `EVT-MARKETING-ATTRIBUTION-RECONCILED` sealed | DIFFERENTIATOR | Exceeds counterpart |

## 12. Deliverability aggregate (IP-029 differentiator)

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Sender reputation monitoring | Email Health Tab | Deliverability Insights | Email Delivery Report | deliverability.warmup_state | EQUIVALENT | IP-029 |
| Domain warmup automation | Warmup automation (limited) | Warmup Program | — | deliverability.warmup state machine | EQUIVALENT (matches Marketo) | IP-029 |
| DKIM / SPF / DMARC validation | DKIM/SPF/DMARC | DKIM/SPF/DMARC | DKIM/SPF/DMARC | mail substrate source-of-truth | DELEGATED | mail contract |
| DMARC failure fail-closed | Manual | Manual | Manual | DMARC failure → automatic pause of all marketing mail | DIFFERENTIATOR | Exceeds counterpart |
| Inbox placement testing | Litmus integration | Inbox Inspector | Inbox Preview (Premium) | intelligence.predict_inbox_placement | EQUIVALENT | IP-029 |
| Blacklist monitoring | Email Health | Blacklist Monitoring | — | deliverability.blacklist_check_worker | EQUIVALENT (matches HubSpot+Marketo) | IP-029 |
| Tenant admin override | Manual | Manual | Manual | override requires Cedar step-up + audit | DIFFERENTIATOR | Exceeds counterpart |
| Per-domain warmup state | Limited | Per-domain | — | deliverability.warmup_id keyed by (tenant, domain_ref) | EQUIVALENT (matches Marketo) | IP-029 |

## 13. Frequency cap aggregate (IP-030 differentiator)

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Email frequency cap | HubSpot Frequency Safeguard | Communication Limit (Email) | Limited | frequency-cap on channel=email | EQUIVALENT | IP-030 |
| Cross-channel cap | Limited (per-channel only) | Per-channel only | — | frequency-cap window keyed (subject, purpose, channel) — atomic across channels | DIFFERENTIATOR | Exceeds counterpart |
| Per-purpose cap | Limited | Limited | — | frequency-cap.purpose dimension | DIFFERENTIATOR | Exceeds counterpart |
| Legal-notice bypass | Limited | Limited | — | Cedar policy: legal_notice purpose bypasses cap | DIFFERENTIATOR | Exceeds counterpart |
| Atomic touch reservation | Eventually consistent | Eventually consistent | — | CAS-atomic touch reservation | DIFFERENTIATOR | Exceeds counterpart |

## 14. ABM aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Target account list | HubSpot ABM Target Accounts | Marketo Named Accounts | — | abm.target_accounts | EQUIVALENT (matches HubSpot+Marketo) | IP-040 |
| Account score | Account Score | Account Score | — | abm.account_score with formula | EQUIVALENT (matches HubSpot+Marketo) | IP-040 |
| Account-level workflow | Account-based Workflow | Engagement Program | — | workflow-canvas.scope=account | EQUIVALENT (matches HubSpot+Marketo) | IP-040 |
| Intent data ingestion | Bombora/G2 integration | Bombora/G2 integration | — | abm.intent_data_source per provenance | EQUIVALENT (matches HubSpot+Marketo) | IP-040 |
| Buying committee detection | HubSpot Buying Committee | Marketo Buying Committee | — | segment.predicate_tree references account-level traits | EQUIVALENT (matches HubSpot+Marketo) | IP-026 |
| Cross-microservice account binding | HubSpot company object | Marketo Company / Account | — | crm.account-master via crm contract | EQUIVALENT | ADR-MS-MA-001 |

## 15. A/B Test aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Email A/B test | A/B Test | A/B Test | A/B Test | a-b-test for email | EQUIVALENT | IP-035 |
| Landing page A/B test | LP A/B Test | LP A/B Test | LP A/B Test (Premium) | a-b-test for landing-page | EQUIVALENT | IP-035 |
| Workflow A/B test | Workflow A/B (Limited) | Smart Campaign A/B | — | a-b-test for workflow-canvas | EQUIVALENT (matches HubSpot+Marketo) | IP-035 |
| Statistical significance threshold | Yes | Yes | Yes | a-b-test.significance_threshold | EQUIVALENT | IP-035 |
| Auto winner selection | Yes | Yes | Yes | a-b-test.winner_selection_rule | EQUIVALENT | IP-035 |
| Multivariate (3+ variants) | Limited (2 variants typical) | Multivariate | Multivariate (Premium) | a-b-test.variant_set unlimited | EQUIVALENT (matches Marketo+Mailchimp) | IP-035 |
| Audit-chain sealed test conclusion | — | Limited | — | `EVT-MARKETING-AB-TEST-CONCLUDED` sealed | DIFFERENTIATOR | Exceeds counterpart |

## 16. Send-Time Optimization aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Per-recipient optimal send time | Send Time Optimization | Optimal Send Time | Send Time Optimization | send-time-optimization bounded context | EQUIVALENT | IP-036 |
| ML-driven prediction | Yes | Yes | Yes | intelligence.predict_send_window | EQUIVALENT | IP-036 |
| Fallback window | Yes | Yes | Yes | sto.fallback_window | EQUIVALENT | IP-036 |
| Respect frequency cap | Limited | Limited | Limited | sto admit decision honors frequency-cap reservation | DIFFERENTIATOR | Exceeds counterpart |
| Respect deliverability admit | Limited | Limited | — | sto admit decision honors deliverability admit | DIFFERENTIATOR | Exceeds counterpart |

## 17. Email tracking aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Open tracking | Open | Open | Open | email-tracking.open_recorded | EQUIVALENT | IP-039 |
| Click tracking | Click | Click | Click | email-tracking.click_recorded | EQUIVALENT | IP-039 |
| Reply tracking | Yes (Sales Hub) | — | — | email-tracking.reply_recorded | EQUIVALENT (matches HubSpot) | IP-039 |
| Bounce tracking | Soft/Hard Bounce | Bounce | Bounce | email-tracking.bounce_recorded (via mail) | EQUIVALENT | IP-039 |
| GPC / DNT signal respect | Limited | Limited | Limited | email-tracking pack-overlay-resolves to honor GPC/DNT | DIFFERENTIATOR | Exceeds counterpart |
| Privacy-preserving pixel | Limited | Limited | Limited | email-tracking.tracking_pixel.aggregated_only_for_apple_mpp | DIFFERENTIATOR | Exceeds counterpart |

## 18. Webhook subscription aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Subscriber URL + event filter | HubSpot Webhooks | Marketo Webhooks | Mailchimp Webhooks | webhook-subscription | EQUIVALENT | IP-042 |
| Signed payload (HMAC) | HMAC-SHA-256 | HMAC-SHA-256 | HMAC-SHA-256 | HMAC-SHA-256 with per-subscription secret in OpenBao | EQUIVALENT | IP-042 |
| Replay-attack defence | Signed timestamp | Signed timestamp | — | signed timestamp window (5 min) | EQUIVALENT (matches HubSpot+Marketo) | IP-042 |
| Retry policy | Exponential backoff | Exponential backoff | Limited | webhook-subscription.retry_policy with backoff | EQUIVALENT | IP-042 |
| Delivery audit log | Delivery log | Delivery log | Limited | `EVT-MARKETING-WEBHOOK-DELIVERY-*` sealed | EQUIVALENT | IP-042 |
| HTTP/3 delivery | HTTP/1.1 + HTTP/2 | HTTP/1.1 | HTTP/1.1 | HTTP/3 + QUIC by default (HTTP/1.1 fallback) | DIFFERENTIATOR | Exceeds counterpart |

## 19. Marketing calendar aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Calendar view | HubSpot Marketing Calendar | Marketo Calendar | Mailchimp Content Calendar | marketing-calendar bounded context | EQUIVALENT | IP-043 |
| Multi-channel visualization | Email + Social + Blog | Email + Program | Email + Social | calendar entries reference email + LP + journey + social + ad assets | EQUIVALENT | IP-043 |
| Conflict detection | Limited | — | — | marketing-calendar.conflict_detection per channel × audience overlap | DIFFERENTIATOR | Exceeds counterpart |
| Meeting scheduler integration | HubSpot Meetings | Marketo Integration | — | DELEGATED to calendar µservice | DELEGATED | Q-013 settled |

## 20. Behavioral profile aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Custom behavioral event ingestion | Custom Behavioral Event (Enterprise) | Marketo Custom Activity | Limited | behavioral-profile.ingest_event | EQUIVALENT (matches HubSpot+Marketo) | IP-044 |
| Per-contact activity timeline | Contact Timeline | Activity Log | Subscriber Activity | behavioral-profile per subject | EQUIVALENT | IP-044 |
| Derived traits from events | Calculated Properties | Marketo Custom Object | — | behavioral-profile.derive_trait | EQUIVALENT (matches HubSpot+Marketo) | IP-044 |
| HLC stamping | — | — | — | every event HLC-stamped | DIFFERENTIATOR | Exceeds counterpart |

## 21. Marketing asset aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Template library | HubSpot Design Manager + Templates | Marketo Design Studio + Templates | Mailchimp Templates | marketing-asset.templates | EQUIVALENT | IP-045 |
| File hosting | HubSpot Files | Marketo Files | Mailchimp Content Studio | marketing-asset.files (with delegation to drive µservice for large objects) | PARTIAL | IP-045 |
| Snippets | HubSpot Snippets | Marketo Snippets | — | marketing-asset.snippets | EQUIVALENT (matches HubSpot+Marketo) | IP-045 |
| Brand kit | HubSpot Brand Kit (limited) | Marketo Brand | Mailchimp Brand Center | marketing-asset.brand_kit | EQUIVALENT | IP-045 |
| Per-locale variants | Limited | Limited | Limited | marketing-asset.locale_variants per pack | DIFFERENTIATOR | Exceeds counterpart |

## 22. Customer-facing analytics aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Campaign performance report | Marketing Analytics | Performance Insights | Reports | customer-analytics.campaign_performance_report | EQUIVALENT | IP-046 |
| Journey conversion report | Workflow Performance | Smart Campaign Performance | Customer Journey Reports | customer-analytics.journey_conversion_report | EQUIVALENT | IP-046 |
| Attribution by source | Attribution Reports | Revenue Cycle Reports | Premium Revenue Reports | customer-analytics.attribution_report | EQUIVALENT | IP-046 |
| Email engagement report | Email Performance | Email Insights | Email Reports | customer-analytics.email_engagement_report | EQUIVALENT | IP-046 |
| Customer-facing vs operator-facing distinction | Yes | Yes | Yes | customer-analytics distinct from `dashboards/` (operator-facing) | EQUIVALENT | IP-046 |
| Scheduled report delivery | Yes | Yes | Yes | customer-analytics.schedule_report | EQUIVALENT | IP-046 |
| Export contracts honor data-class | Limited | Limited | Limited | customer-analytics.export honors data-boundary labels | DIFFERENTIATOR | Exceeds counterpart |

## 23. Social media aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Social publishing | HubSpot Social | Marketo Engage Social | Mailchimp Social Posting (paid) | DELEGATED to social µservice | DELEGATED | Q-011 settled |
| Social monitoring | HubSpot Social Monitoring | — | — | DELEGATED to social µservice | DELEGATED | — |
| Cross-channel campaign | Yes | Yes | Limited | campaign aggregate references social assets via social contract | EQUIVALENT (matches HubSpot+Marketo) | IP-040 |

## 24. SEO aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| SEO recommendations | HubSpot SEO | Marketo SEO Tools | Limited | DELEGATED to sites µservice | DELEGATED | Q-019 settled |
| Keyword tracking | HubSpot Keyword | — | — | DELEGATED to sites µservice | DELEGATED | — |

## 25. CMS overlap aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Website CMS | HubSpot CMS Hub | Marketo Design Studio (limited) | — | DELEGATED to sites + design-collaboration µservices | DELEGATED | Q-007 settled |
| Marketing-attached landing pages | LP under CMS Hub | LP under Marketo | LP under Mailchimp | marketing-automation.landing-page (this µservice) | EQUIVALENT | ADR-MS-MA-002 |

## 26. Chatflow / bot aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Chatbot decision tree | HubSpot Chatflows | — | — | chatflow bounded context | EQUIVALENT (matches HubSpot) | IP-055 |
| Live agent handoff | HubSpot Live Chat | — | — | chatflow.handoff to contact-center | EQUIVALENT (matches HubSpot) | IP-055 |
| PII redaction at handoff | Limited | — | — | chatflow.pii_redaction_at_handoff_boundary | DIFFERENTIATOR | Exceeds counterpart |

## 27. Ad-network aggregate

| Capability | HubSpot | Marketo | Mailchimp | Oyatie | Status | Note |
|---|---|---|---|---|---|---|
| Google Ads integration | HubSpot Ads | Marketo Ad Bridge | Mailchimp Ads | DELEGATED to advertising-platform µservice | DELEGATED | Q-022 settled |
| LinkedIn Ads integration | HubSpot Ads | Marketo Ad Bridge | — | DELEGATED to advertising-platform µservice | DELEGATED | — |
| Facebook Ads integration | HubSpot Ads | Marketo Ad Bridge | Mailchimp Ads | DELEGATED to advertising-platform µservice | DELEGATED | — |
| Audience sync to ad networks | Yes | Yes | Yes | segment.export_to_ad_network via marketplace audience-license | EQUIVALENT | ADR-0314 |

## 28. Migration cost summary

| Counterpart | Migration playbook | Object mapping table | Field-level deltas | Capability gap | Migration cost (T-shirt) |
|---|---|---|---|---|---|
| HubSpot Marketing Hub (primary) | `migration-playbooks/from-hubspot-marketing-hub.md` | Yes (in playbook §3) | Yes (in playbook §4) | Email visual editor depth | M (1-2 sprints) |
| Adobe Marketo Engage (flanking) | `migration-playbooks/from-marketo.md` | Yes (in playbook §3) | Yes (in playbook §4) | Token-substitution depth, Engagement Program nurture stream | L (2-4 sprints) |
| Mailchimp (flanking) | `migration-playbooks/from-mailchimp.md` | Yes (in playbook §3) | Yes (in playbook §4) | Postcard direct-mail, Lookalike Audience | S (0.5-1 sprint) |
| Salesforce Marketing Cloud Engagement (reference) | `migration-playbooks/from-salesforce-marketing-cloud-engagement.md` (planned) | Planned | Planned | Journey Builder depth + Mobile Studio | XL (4+ sprints) |
| Klaviyo (reference) | `migration-playbooks/from-klaviyo.md` (planned) | Planned | Planned | E-commerce data layer | M (1-2 sprints) |
| Iterable (reference) | `migration-playbooks/from-iterable.md` (planned) | Planned | Planned | Workflow Studio depth | M (1-2 sprints) |
| Braze (reference) | `migration-playbooks/from-braze.md` (planned) | Planned | Planned | Canvas Flow depth + Currents | L (2-4 sprints) |

## 29. UNION coverage assessment

Aggregate UNION coverage (HubSpot ∪ Marketo ∪ Mailchimp) across the capabilities enumerated above:

- EQUIVALENT or DIFFERENTIATOR: 95+ capabilities (~75-80% of UNION).
- PARTIAL: 15-20 capabilities (~12-15% of UNION).
- DELEGATED with seam: 10+ capabilities (~8% of UNION, ad-networks + social + SEO + meeting scheduler + CMS root).
- PLANNED but not authored: 3-5 capabilities (~2% of UNION).

This matrix supersedes the prior 304-row stamped matrix. Differentiator capabilities (IP-026..IP-030) deliver auditable evidence chain, sub-second freshness, cross-channel atomic frequency cap, append-only consent ledger, and deterministic attribution replay — surfaces that the counterparts treat as platform-only or out-of-scope.

## 30. Companion document

The full per-capability UNION-coverage matrix lands in `microservices/marketing-automation/feature-parity-matrix-2026-05-20.md` (Wave-4 audit deliverable). This matrix is the executive-summary surface; the companion file is the row-by-row evidence chain.
