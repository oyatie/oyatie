---
doc_class: LocalizationPack
pack_id: JP-PACK-1
title: Japan Financial Services Act, FIEA, Banking Act, and Payment Services Localization
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
citing_authority_url:
  - https://www.fsa.go.jp/en/laws_regulations/index.html
  - https://www.fsa.go.jp/en/policy/marketentry/guidebook/02.html
  - https://www.japaneselawtranslation.go.jp/en/laws/view/1911/en
  - https://www.japaneselawtranslation.go.jp/en/laws/view/4498
  - https://www.japaneselawtranslation.go.jp/en/laws/view/4477
---

# Japan Financial Services Act and Banking Act Localization Pack

PackLine-001: This document defines the JP-PACK-1 financial-services overlay for Oyatie services that touch regulated finance, banking, payment instruments, investment products, cryptoassets, or bank-linked data flows in Japan.
PackLine-002: It is a localization-control document, not a legal opinion; product teams must use it to encode gates, data contracts, audit events, and failure modes that require counsel and compliance owner review.
PackLine-003: The pack gives precedence to Japanese statute-specific licensing gates over generic global-finance defaults when the customer, user, branch, solicitation, account, asset, or settlement rail is Japan-linked.
PackLine-004: The controlling authorities are the Financial Services Agency, Japanese Law Translation Database System, Payment Services Act, Banking Act, and Financial Instruments and Exchange Act sources cited below.
PackLine-005: The localized service families include account onboarding, regulated-activity classification, product catalog publication, financial solicitation, wallet or stored-value flows, bank transfer flows, asset-management workflows, and cryptoasset exchange integrations.
PackLine-006: The pack assumes strict activity classification before feature activation because Japan separates bank license, financial instruments business registration, funds transfer registration, prepaid payment instrument registration, electronic payment instrument registration, and cryptoasset exchange registration.
PackLine-007: The pack must be activated before any Japan-facing financial product can be surfaced in a public catalog, API sandbox, partner integration, customer onboarding flow, or production workflow.
PackLine-008: The pack must not infer that a group affiliate license covers an unlicensed Oyatie tenant; authority, branch, registration number, Japan representative, and permitted business category must be explicit per tenant.
PackLine-009: The pack treats unregistered foreign solicitation into Japan as a high-risk failure mode, especially for financial instruments, cryptoasset exchange services, and electronic payment instruments.
PackLine-010: The pack preserves the Japanese law term 前払式支払手段 for prepaid payment instruments because product names such as gift balance, stored value, wallet credit, voucher, and merchant coin can mask the regulated substance.
PackLine-011: The pack treats "virtual currency" as a historical and translation-adjacent term only; current operational controls use cryptoasset / 暗号資産 unless a legacy authority source demands the older label.
PackLine-012: The pack treats Japanese bank activity under the Banking Act as a license-controlled domain, not a simple payment-service extension.
PackLine-013: The pack treats Financial Instruments Business under FIEA as category-controlled: Type I, Type II, Investment Management, Investment Advisory and Agency, Securities etc. Management, and related intermediary services.
PackLine-014: The pack treats the Payment Services Act as the primary control plane for prepaid payment instruments, funds transfer services, electronic payment instruments, and cryptoasset exchange services.
PackLine-015: The pack treats the FSA market-entry guidebook as interpretive routing support for registration categories, while statutory text and official registration conditions remain the enforcement source.
PackLine-016: The pack requires product teams to bind every Japan financial feature to a `jp_financial_activity_type` before workflow execution.
PackLine-017: The pack requires data teams to bind every stored financial instrument, money balance, token balance, wallet balance, or solicitation artifact to a statute and license basis.
PackLine-018: The pack requires audit teams to preserve registration decision inputs because regulators may inspect reports, books, documents, advertisements, user disclosures, and outsourced operations.
PackLine-019: The pack requires feature flags to fail closed when license metadata is absent, expired, disputed, scoped to another entity, scoped to another product category, or scoped outside Japan.
PackLine-020: The pack requires a separate Japan compliance owner acknowledgement for mixed products that combine investment advice, fund solicitation, stored value, cryptoasset custody, and banking integrations.

## Authority Citations

Authority-001: FSA Laws and Regulations index identifies Japanese Law Translation as the official full-text search path for financial laws and lists supervisory guidelines for banks and financial instruments business operators.
Authority-002: FSA market-entry guidebook names four Financial Instruments Business categories: Type I, Type II, Investment Management, and Investment Advisory and Agency.
Authority-003: FSA market-entry guidebook explains that managing customer assets or funds generally requires Investment Management Business registration.
Authority-004: FSA market-entry guidebook explains that advice limited to values of securities or investment decisions may map to Investment Advisory and Agency Business rather than Investment Management.
Authority-005: FSA market-entry guidebook explains that solicitation or sale of securities, including fund interests, may require Type I or Type II Financial Instruments Business registration.
Authority-006: Financial Instruments and Exchange Act Article 28 defines Type I Financial Instruments Business by reference to securities, derivatives, underwriting, and related categories.
Authority-007: Financial Instruments and Exchange Act Article 28 defines Type II Financial Instruments Business for specified securities and rights categories outside Type I scope.
Authority-008: Financial Instruments and Exchange Act Article 28 defines Investment Advisory and Agency Business for advisory and agency acts listed under the Act.
Authority-009: Financial Instruments and Exchange Act Article 28 defines Investment Management Business for discretionary and fund-management activities.
Authority-010: Financial Instruments and Exchange Act Article 29 requires Financial Instruments Business to be conducted only by persons registered by the Prime Minister.
Authority-011: Financial Instruments and Exchange Act Article 29-2 requires a registration application containing business category, capital, officers, office locations, other business types, and related matters.
Authority-012: Financial Instruments and Exchange Act Article 29-3 creates the registry of Financial Instruments Business Operators and public inspection of that registry.
Authority-013: Financial Instruments and Exchange Act Article 29-4 requires refusal of registration for disqualifying conditions, false statements, or missing important matters.
Authority-014: Financial Instruments and Exchange Act Article 43-2 requires segregated management of customer securities and money for covered securities-related business.
Authority-015: Financial Instruments and Exchange Act Article 46-2 requires Financial Instruments Business Operators to prepare and preserve books and business documents.
Authority-016: Financial Instruments and Exchange Act Article 51 gives supervisory authority to order necessary business-operation improvements when public interest or investor protection requires it.
Authority-017: Financial Instruments and Exchange Act Article 52 authorizes registration rescission or suspension for serious registration, solvency, law-violation, investor-harm, and wrongful-act conditions.
Authority-018: Financial Instruments and Exchange Act Article 56-2 authorizes reports, materials, and inspections connected to Financial Instruments Business Operators and related persons.
Authority-019: Banking Act Article 1 states the purpose of preserving bank-service credibility, depositor protection, smooth financial services, and sound national economic development.
Authority-020: Banking Act Article 2 defines a bank as a person engaging in banking under the license referred to in Article 4.
Authority-021: Banking Act Article 2 defines banking to include accepting deposits or installment savings together with lending or bill discounting.
Authority-022: Banking Act Article 2 defines banking to include dealing in funds transfer transactions.
Authority-023: Banking Act Article 4 requires a Prime Minister license before a person may engage in banking.
Authority-024: Banking Act Article 24 provides supervisory report and material submission authority over banks.
Authority-025: Banking Act Article 25 provides on-site inspection authority over banks and related business/property status.
Authority-026: Banking Act Article 26 provides suspension and business-improvement supervisory authority over banks.
Authority-027: Banking Act Article 27 provides license rescission authority for covered Banking Act violations and disqualifying conditions.
Authority-028: Banking Act Article 29 provides retained-assets-in-Japan authority under specified bank-supervision conditions.
Authority-029: Banking Act Chapter VII establishes foreign bank branch controls that must not be bypassed by cross-border product catalog publication.
Authority-030: Banking Act Article 52-61-2 requires registration for electronic payment services under the Banking Act chapter governing bank-linked electronic payment services.
Authority-031: Payment Services Act Article 1 states that the Act enforces registration and measures for prepaid payment instruments, funds transfer transactions by non-deposit-taking institutions, electronic payment instruments, cryptoassets, and related clearing functions.
Authority-032: Payment Services Act Article 7 requires a corporation registered by the Prime Minister before issuing prepaid payment instruments for third-party business.
Authority-033: Payment Services Act Article 8 requires a prepaid payment instrument registration application with prescribed particulars.
Authority-034: Payment Services Act Article 9 creates the public register of issuers of prepaid payment instruments for third-party business.
Authority-035: Payment Services Act Article 21-2 requires oversight of entrusted parties for prepaid payment instrument issuance.
Authority-036: Payment Services Act Article 21-3 requires prompt complaint processing measures for prepaid payment instrument users.
Authority-037: Payment Services Act Article 22 requires prepaid payment instrument issuers to prepare and maintain books and documents.
Authority-038: Payment Services Act Article 23 requires reports on prepaid payment instrument issuance, unused balance, and issuance security deposit.
Authority-039: Payment Services Act Article 27 authorizes registration revocation or suspension for third-party prepaid payment instrument issuers.
Authority-040: Payment Services Act Article 37 requires registration for funds transfer service providers.
Authority-041: Payment Services Act Article 49 requires information-security management for funds transfer service providers.
Authority-042: Payment Services Act Article 50 requires guidance and control over outsourced funds transfer services.
Authority-043: Payment Services Act Article 51 requires customer protection measures and anti-confusion disclosures for funds transfer services.
Authority-044: Payment Services Act Article 62-3 is the registration anchor for electronic payment instruments service providers.
Authority-045: Payment Services Act Article 63 prohibits unregistered foreign electronic payment instruments service providers from soliciting persons in Japan.
Authority-046: Payment Services Act Article 63-2 requires registration before providing a cryptoasset exchange service.
Authority-047: Payment Services Act Article 63-3 requires cryptoasset exchange registration applications to include trade name, address, capital, business office, directors, Japan representative for foreign providers, handled cryptoassets, service method, outsourcing, and other business types.
Authority-048: Payment Services Act Article 63-4 creates the public register of cryptoasset exchange service providers.
Authority-049: Payment Services Act Article 63-5 requires refusal of cryptoasset exchange registration for disqualifying conditions, false material statements, missing material statements, insufficient financial basis, or missing control systems.
Authority-050: Payment Services Act Article 63-7 prohibits name lending by cryptoasset exchange service providers.
Authority-051: Payment Services Act Article 63-8 requires information-security management for cryptoasset exchange service information.
Authority-052: Payment Services Act Article 63-9 requires outsourced-party guidance for cryptoasset exchange service outsourcing.
Authority-053: Payment Services Act Article 63-9-2 requires cryptoasset exchange advertising to identify trade name, registration number, and material cryptoasset characteristics.
Authority-054: Payment Services Act Article 63-9-3 prohibits misleading cryptoasset exchange contract and advertising conduct.
Authority-055: Payment Services Act Article 63-10 requires cryptoasset user-protection explanations, fee disclosures, terms disclosures, and service-stability measures.
Authority-056: Payment Services Act Article 63-11 requires segregation and trust/custody-style controls for users' money and cryptoassets under cryptoasset exchange services.
Authority-057: Payment Services Act Article 63-13 requires cryptoasset exchange books and accounts documents.
Authority-058: Payment Services Act Article 63-14 requires annual cryptoasset exchange reports and additional management reports for user money and cryptoasset custody.
Authority-059: FSA cryptoasset guidance notes the 2020 terminology shift from "virtual currency" to "cryptoassets" under amended Payment Services Act terminology.
Authority-060: FSA cryptoasset guidance states that domestic exchange service between cryptoassets and legal currency requires cryptoasset exchange registration.
Authority-061: FSA 2025 cryptoasset discussion paper records the 2016 amendment as establishing registration for fiat-crypto exchange providers and user-protection measures.
Authority-062: FSA 2025 cryptoasset discussion paper records the 2019 amendment as adding advance reporting for handled cryptoassets, cold-wallet expectations in principle, and advertising/solicitation regulations.
Authority-063: FSA 2025 cryptoasset discussion paper records the 2022 travel-rule layer under anti-money-laundering legislation for cryptoasset transfer identification.
Authority-064: FSA 2025 cryptoasset discussion paper records ongoing proposals for retained-assets-in-Japan and cryptoasset intermediary business changes; this pack marks them as watchlist, not current mandatory controls unless enacted and effective.
Authority-065: The pack cites official sources only for canonical control requirements and uses non-official industry commentary zero times as authority.
Authority-066: Where Japanese Law Translation versions differ, this pack records the cited law URL and requires legal review before treating a translation label as controlling over current Japanese statutory text.
Authority-067: Where a feature touches deposits, lending, bill discounting, or funds transfer transactions by a deposit-taking institution, Banking Act gates take precedence over Payment Services Act non-bank routing.
Authority-068: Where a feature touches non-bank funds transfers, prepaid payment instruments, electronic payment instruments, or cryptoasset exchange services, Payment Services Act gates must be evaluated before generic payment routing.
Authority-069: Where a feature touches securities, derivatives, collective investment schemes, investment advice, discretionary management, or fund solicitation, FIEA gates must be evaluated before generic marketplace routing.
Authority-070: Where a feature combines investment product distribution with payment or wallet functionality, all relevant statute gates must pass; one license category cannot silently substitute for another.

## Activated Cedar Policies

Policy-001: `jp_financial_activity_classified_before_activation` denies feature activation unless `jp_financial_activity_type` is present, reviewed, and mapped to a statute.
Policy-002: `jp_fiea_registration_required_for_financial_instruments_business` denies FIEA-scope acts without a tenant registration basis under Article 29 or an approved exemption record.
Policy-003: `jp_fiea_type_i_scope_gate` denies Type I securities, derivatives, underwriting, or securities-related management workflows unless the tenant registration category explicitly includes Type I.
Policy-004: `jp_fiea_type_ii_scope_gate` denies Type II fund interest, deemed-security, or non-Type-I securities workflows unless the tenant registration category explicitly includes Type II.
Policy-005: `jp_fiea_investment_management_scope_gate` denies discretionary management, delegated investment authority, or fund-management execution unless Investment Management registration is active.
Policy-006: `jp_fiea_investment_advisory_scope_gate` denies compensated investment advice or advisory agency workflows unless Investment Advisory and Agency registration is active or an approved no-substantial-remuneration exemption exists.
Policy-007: `jp_fiea_solicitation_scope_gate` denies Japan-facing fund or securities solicitation unless the activity is mapped to Type I, Type II, Investment Management, or approved exemption handling.
Policy-008: `jp_fiea_foreign_operator_representative_office_gate` denies representative-office mode if the workflow performs solicitation, advice, execution, management, order routing, or customer onboarding.
Policy-009: `jp_fiea_public_registry_required` denies production use when the registration number is missing from the tenant registration record for registered activities.
Policy-010: `jp_fiea_japan_office_required_for_type_i_foreign_entity` denies Type I foreign-entity workflows unless Japan office and representative metadata are present.
Policy-011: `jp_fiea_major_business_change_review` denies material category change until compliance approves new registration or amendment analysis.
Policy-012: `jp_fiea_customer_asset_segregation_required` denies custody or management of customer securities or money unless segregation controls are active.
Policy-013: `jp_fiea_books_documents_retention_required` denies regulated trade execution unless books-and-documents retention is enabled.
Policy-014: `jp_fiea_supervisory_order_freeze` denies new onboarding when the tenant has an unresolved FSA improvement order, suspension, or rescission notice.
Policy-015: `jp_fiea_disqualified_registration_actor_denied` denies registration-basis usage by actors marked disqualified under Article 29-4 review.
Policy-016: `jp_fiea_professional_investor_status_locked` denies professional-investor routing unless status, contract kind, effective date, and revocation route are recorded.
Policy-017: `jp_fiea_general_investor_protection_default` treats unknown investor status as general-investor status.
Policy-018: `jp_fiea_advertising_disclosure_gate` denies public material unless registration category, risk, fee, issuer, and product scope disclosures are attached.
Policy-019: `jp_fiea_cross_border_solicitation_gate` denies cross-border Japan solicitation by unregistered foreign financial instruments operators unless counsel-approved exemption evidence exists.
Policy-020: `jp_banking_license_required` denies deposit-taking, installment savings, lending-with-deposit, bill discounting with deposits, or banking funds-transfer business unless a Banking Act license basis is attached.
Policy-021: `jp_banking_deposit_label_lock` denies product labels that imply deposit, bank account, savings, or insured balance when the tenant is not a licensed bank.
Policy-022: `jp_banking_funds_transfer_by_bank_gate` routes deposit-taking institution funds-transfer activity through Banking Act and bank supervision gates, not through non-bank PSA routing alone.
Policy-023: `jp_banking_foreign_branch_gate` denies foreign-bank branch workflows unless Japan branch license, principal branch designation, and local representative metadata are present.
Policy-024: `jp_banking_bank_agent_gate` denies bank agency services unless principal bank, agent authorization, and customer-explanation controls are present.
Policy-025: `jp_banking_electronic_payment_service_registration_gate` denies Banking Act electronic payment service workflows unless Article 52-61-2 registration evidence exists.
Policy-026: `jp_banking_asset_retention_order_hold` prevents asset movement outside allowed boundaries when a retained-assets-in-Japan order or equivalent supervisory marker is active.
Policy-027: `jp_banking_report_material_preservation` denies deletion of bank-related reports, materials, logs, and books during active supervision, incident, inspection, or audit hold.
Policy-028: `jp_banking_license_rescission_freeze` denies new account creation or product enrollment when bank license rescission or suspension state exists.
Policy-029: `jp_banking_depositor_protection_priority` requires depositor-protection escalation before general commercial continuity when bank failure, transfer, merger, or dissolution workflows are triggered.
Policy-030: `jp_banking_misleading_bank_status_denied` denies UI, API, or marketing output that suggests bank status for a non-bank tenant.
Policy-031: `jp_psa_prepaid_third_party_registration_required` denies third-party prepaid payment instrument issuance unless Article 7 registration metadata is active.
Policy-032: `jp_psa_prepaid_own_business_unregistered_threshold_gate` blocks own-business prepaid flows when unused-balance threshold or reporting conditions require registration or notification review.
Policy-033: `jp_psa_prepaid_instrument_term_lock` requires the Japanese label 前払式支払手段 in compliance metadata for stored-value products treated as prepaid payment instruments.
Policy-034: `jp_psa_prepaid_unused_balance_report_gate` denies settlement close when base-date unused balance reporting data is incomplete.
Policy-035: `jp_psa_prepaid_security_deposit_gate` denies issuance when required issuance security deposit data is missing or stale.
Policy-036: `jp_psa_prepaid_outsourcing_supervision_gate` denies outsourced prepaid issuance operations unless entrusted-party controls are recorded.
Policy-037: `jp_psa_prepaid_complaint_processing_gate` denies launch unless complaint intake, SLA, owner, and resolution evidence fields are configured.
Policy-038: `jp_psa_prepaid_revocation_freeze` denies new issuance during revocation, suspension, unknown-office, or public-notice state.
Policy-039: `jp_psa_funds_transfer_registration_required` denies non-bank funds transfer services unless Article 37 registration evidence exists.
Policy-040: `jp_psa_funds_transfer_type_gate` requires Type I, Type II, or Type III funds-transfer classification before transfer execution.
Policy-041: `jp_psa_funds_transfer_user_funds_not_stored_gate` denies holding funds that are unlikely to be used for transfer where user-protection measures require non-retention.
Policy-042: `jp_psa_funds_transfer_bank_confusion_disclosure` requires disclosure that non-bank funds transfer services are not bank funds transfer transactions by deposit-taking institutions.
Policy-043: `jp_psa_funds_transfer_security_deposit_gate` denies transfer execution if security deposit, guarantee, or trust coverage is insufficient.
Policy-044: `jp_psa_funds_transfer_outsourcing_supervision_gate` denies outsourced transfer processing unless guidance and operational control evidence is attached.
Policy-045: `jp_psa_funds_transfer_information_security_gate` denies production operation unless leakage, loss, and damage prevention controls are active.
Policy-046: `jp_psa_electronic_payment_instruments_registration_required` denies electronic payment instrument exchange/service workflows unless Article 62-3 registration metadata exists.
Policy-047: `jp_psa_foreign_electronic_payment_instrument_solicitation_denied` denies Japan solicitation by unregistered foreign electronic payment instrument providers.
Policy-048: `jp_psa_cryptoasset_exchange_registration_required` denies cryptoasset exchange services unless Article 63-2 registration metadata exists.
Policy-049: `jp_psa_cryptoasset_name_registered` denies cryptoasset handling unless the handled cryptoasset name is included in approved registration metadata or change-control evidence.
Policy-050: `jp_psa_cryptoasset_foreign_representative_gate` denies foreign cryptoasset exchange provider activity unless Japan representative metadata exists.
Policy-051: `jp_psa_cryptoasset_name_lending_denied` denies delegated or white-label cryptoasset exchange operation in another party's name.
Policy-052: `jp_psa_cryptoasset_advertising_gate` requires trade name, registration number, non-currency disclaimer, material characteristics, and risk language before publication.
Policy-053: `jp_psa_cryptoasset_misleading_profit_solicitation_denied` denies advertising that promotes cryptoasset purchase or exchange solely for profit rather than payment/use purpose.
Policy-054: `jp_psa_cryptoasset_user_property_segregation_gate` denies custody unless user money and cryptoassets are segregated from the provider's property.
Policy-055: `jp_psa_cryptoasset_cold_storage_control_gate` requires custody architecture evidence for offline or low-risk storage handling when policy is enabled.
Policy-056: `jp_psa_cryptoasset_periodic_audit_gate` denies custody expansion unless periodic auditor evidence exists.
Policy-057: `jp_psa_cryptoasset_annual_report_gate` blocks annual close when Article 63-14 report data is incomplete.
Policy-058: `jp_psa_cryptoasset_travel_rule_watch_gate` requires AML transfer metadata review when cryptoasset transfer features are enabled.
Policy-059: `jp_multi_license_composite_product_gate` denies products combining banking, FIEA, PSA prepaid, funds transfer, electronic payment instruments, or cryptoasset functions unless every component gate passes.
Policy-060: `jp_financial_regulatory_hold_priority` gives regulator hold, inspection hold, incident hold, and license-suspension hold precedence over customer-configurable retention deletion.
Policy-061: `jp_financial_counsel_override_required` requires counsel override for any activity marked `uncertain_jp_financial_classification`.
Policy-062: `jp_financial_api_sandbox_not_public_solicitation` blocks sandbox invitations if they contain product-specific investment, wallet, bank, or crypto solicitation language before classification.
Policy-063: `jp_financial_test_tenant_denied_real_assets` denies real deposits, real securities, real cryptoassets, and real prepaid value in unlicensed test tenants.
Policy-064: `jp_financial_feature_flag_japan_region_lock` prevents Japan region exposure when global feature flags lack JP-PACK-1 policy evaluation.
Policy-065: `jp_financial_registration_number_immutable_audit` prevents mutable overwrite of registration number history; corrections require new version entries.
Policy-066: `jp_financial_officer_and_representative_snapshot` requires officer, director, and Japan representative snapshots for regulated application records.
Policy-067: `jp_financial_outsourcing_chain_visible` requires first-tier and known multi-tier outsourced-provider mapping for regulated operations.
Policy-068: `jp_financial_supervisory_notice_ingest_required` requires regulator notices to enter the compliance event queue before operational state can be changed.
Policy-069: `jp_financial_user_disclosure_locale_required` requires Japanese-language disclosure artifacts where a Japan retail user can see regulated product information.
Policy-070: `jp_financial_pack_precedence` makes this JP financial policy pack override generic global finance policies on conflict for Japan-linked activity.

## Data Model Deltas

Data-001: Add `jp_financial_activity_type` enum with values `banking`, `bank_agent`, `bank_electronic_payment_service`, `fiea_type_i`, `fiea_type_ii`, `investment_management`, `investment_advisory_agency`, `securities_management`, `funds_transfer`, `prepaid_payment_instrument`, `electronic_payment_instrument`, `cryptoasset_exchange`, `mixed_financial_product`, and `non_regulated_reference_only`.
Data-002: Add `jp_financial_statute_basis` enum with values `banking_act`, `financial_instruments_exchange_act`, `payment_services_act`, `act_on_provision_of_financial_services`, `anti_money_laundering_related`, `multi_statute`, and `not_applicable`.
Data-003: Add `jp_financial_registration_required` boolean to the product classification record.
Data-004: Add `jp_financial_registration_status` enum with values `not_required`, `required_not_started`, `application_in_progress`, `registered`, `suspended`, `rescinded`, `expired`, `withdrawn`, `exempted_by_review`, and `unknown`.
Data-005: Add `jp_financial_registration_number` string with immutable version history.
Data-006: Add `jp_financial_registration_authority` string defaulting to `Financial Services Agency / Prime Minister delegated authority` for FSA-controlled registrations.
Data-007: Add `jp_financial_registration_scope_description` text describing permitted products, categories, offices, and business methods.
Data-008: Add `jp_financial_registration_effective_at` timestamp for registration start.
Data-009: Add `jp_financial_registration_expires_at` nullable timestamp for registrations or approvals with an end date.
Data-010: Add `jp_financial_registration_last_verified_at` timestamp for evidence freshness.
Data-011: Add `jp_financial_registration_evidence_uri` array linking registry screenshots, official register references, counsel memo IDs, and regulator letters.
Data-012: Add `jp_financial_office_in_japan_required` boolean.
Data-013: Add `jp_financial_office_in_japan_id` foreign key to the regulated office table.
Data-014: Add `jp_financial_representative_in_japan_required` boolean for foreign juridical persons and foreign cryptoasset exchange providers.
Data-015: Add `jp_financial_representative_in_japan_person_id` foreign key.
Data-016: Add `jp_financial_officer_snapshot_id` for directors, officers, auditors, and equivalent foreign-role captures.
Data-017: Add `jp_financial_disqualification_review_status` enum with values `not_required`, `pending`, `passed`, `failed`, and `requires_counsel`.
Data-018: Add `jp_financial_disqualification_review_evidence` array to preserve Article 29-4 and Article 63-5 style registration refusal screening artifacts.
Data-019: Add `jp_fiea_business_category` enum with values `type_i`, `type_ii`, `investment_management`, `investment_advisory_agency`, `securities_management`, `registered_financial_institution_business`, `financial_instruments_intermediary`, and `none`.
Data-020: Add `jp_fiea_kind_of_contract` string for professional-investor status and activity-rule exceptions.
Data-021: Add `jp_fiea_investor_classification` enum with values `general_investor`, `professional_investor`, `qualified_institutional_investor`, `specified_investor`, `unknown`, and `not_applicable`.
Data-022: Add `jp_fiea_investor_status_evidence_id` foreign key to investor classification evidence.
Data-023: Add `jp_fiea_solicitation_target_region` enum with values `japan`, `outside_japan`, `mixed`, and `unknown`.
Data-024: Add `jp_fiea_product_security_classification` enum with values `listed_security`, `fund_interest`, `collective_investment_scheme`, `derivative`, `cryptoasset_derivative`, `non_security`, `unknown`, and `mixed`.
Data-025: Add `jp_fiea_customer_asset_control_model` enum with values `none`, `segregated_securities`, `segregated_money`, `trust_money`, `third_party_custodian`, `customer_direct`, and `mixed`.
Data-026: Add `jp_fiea_books_documents_profile_id` for retention configuration.
Data-027: Add `jp_fiea_public_disclosure_profile_id` for advertising, explanation documents, risk notices, and customer materials.
Data-028: Add `jp_fiea_representative_office_only` boolean; true forbids solicitation and execution workflows.
Data-029: Add `jp_banking_license_required` boolean.
Data-030: Add `jp_banking_license_id` foreign key to regulated bank license record.
Data-031: Add `jp_banking_business_kind` enum with values `deposit_lending`, `installment_savings_lending`, `bill_discounting_with_deposits`, `funds_transfer_by_bank`, `bank_agent`, `foreign_bank_branch`, `electronic_payment_service`, and `not_banking`.
Data-032: Add `jp_banking_deposit_taking_flag` boolean.
Data-033: Add `jp_banking_depositor_protection_profile_id` linking to depositor notice, payout, transfer, and protection rules.
Data-034: Add `jp_banking_foreign_branch_principal_id` for foreign bank branch mapping.
Data-035: Add `jp_banking_bank_agent_principal_bank_id` for bank agency services.
Data-036: Add `jp_banking_asset_retention_order_status` enum with values `none`, `notice_received`, `active`, `released`, and `unknown`.
Data-037: Add `jp_banking_supervisory_order_status` enum with values `none`, `report_requested`, `inspection_pending`, `business_improvement_order`, `suspension`, `license_rescission`, and `unknown`.
Data-038: Add `jp_banking_electronic_payment_service_registration_id` for Banking Act Article 52-61-2 electronic payment service registration.
Data-039: Add `jp_psa_service_type` enum with values `prepaid_own_business`, `prepaid_third_party`, `funds_transfer_type_i`, `funds_transfer_type_ii`, `funds_transfer_type_iii`, `electronic_payment_instrument`, `cryptoasset_exchange`, and `none`.
Data-040: Add `jp_psa_prepaid_instrument_label_jp` string defaulting to `前払式支払手段` for prepaid classification records.
Data-041: Add `jp_psa_prepaid_issuer_type` enum with values `own_business`, `third_party_business`, `high_value_electronically_transferable`, `unknown`, and `not_applicable`.
Data-042: Add `jp_psa_prepaid_unused_balance_base_date` date.
Data-043: Add `jp_psa_prepaid_unused_balance_amount_jpy` decimal.
Data-044: Add `jp_psa_prepaid_issuance_security_deposit_amount_jpy` decimal.
Data-045: Add `jp_psa_prepaid_report_submission_status` enum with values `not_due`, `due`, `submitted`, `late`, `blocked`, and `not_applicable`.
Data-046: Add `jp_psa_prepaid_complaint_process_profile_id`.
Data-047: Add `jp_psa_funds_transfer_type` enum with values `type_i`, `type_ii`, `type_iii`, `unknown`, and `not_applicable`.
Data-048: Add `jp_psa_funds_transfer_security_deposit_profile_id`.
Data-049: Add `jp_psa_funds_transfer_user_confusion_disclosure_id`.
Data-050: Add `jp_psa_funds_transfer_obligation_duration_limit` interval field.
Data-051: Add `jp_psa_funds_transfer_user_obligation_amount_limit_jpy` decimal for Type III limit controls where applicable.
Data-052: Add `jp_psa_electronic_payment_instrument_registration_id`.
Data-053: Add `jp_psa_foreign_electronic_payment_solicitation_status` enum with values `not_foreign`, `registered`, `unregistered_denied`, `counsel_review`, and `unknown`.
Data-054: Add `jp_psa_cryptoasset_exchange_registration_id`.
Data-055: Add `jp_psa_cryptoasset_handled_asset_names` array.
Data-056: Add `jp_psa_cryptoasset_japan_representative_person_id`.
Data-057: Add `jp_psa_cryptoasset_advertising_profile_id`.
Data-058: Add `jp_psa_cryptoasset_user_money_segregation_profile_id`.
Data-059: Add `jp_psa_cryptoasset_user_cryptoasset_segregation_profile_id`.
Data-060: Add `jp_psa_cryptoasset_cold_storage_profile_id`.
Data-061: Add `jp_psa_cryptoasset_periodic_audit_profile_id`.
Data-062: Add `jp_psa_cryptoasset_annual_report_status` enum with values `not_due`, `preparing`, `submitted`, `late`, `blocked`, and `not_applicable`.
Data-063: Add `jp_psa_cryptoasset_management_report_status` enum for managed user money or cryptoasset volume reports.
Data-064: Add `jp_financial_outsourcing_profile_id` linking first-tier and multi-tier entrusted providers.
Data-065: Add `jp_financial_information_security_profile_id` for leakage, loss, damage, access, encryption, and incident response controls.
Data-066: Add `jp_financial_customer_disclosure_language_profile` with Japanese required when retail Japan users are targeted.
Data-067: Add `jp_financial_regulatory_hold_id` nullable foreign key to inspection, report request, order, suspension, or incident hold.
Data-068: Add `jp_financial_counsel_review_id` for uncertain classifications and exemptions.
Data-069: Add `jp_financial_watchlist_status` enum with values `none`, `monitoring_2025_cryptoasset_reform`, `monitoring_payment_services_amendment`, `monitoring_fiea_reform`, and `other`.
Data-070: Add `jp_financial_pack_version` string set to `JP-PACK-1`.

## API Contract Deltas

API-001: `POST /jp/financial/classifications` creates a Japan financial activity classification record before any regulated feature can be enabled.
API-002: The classification request requires `tenant_id`, `product_id`, `jp_financial_activity_type`, `statute_basis`, `target_user_region`, `target_asset_type`, and `proposed_business_method`.
API-003: The classification response returns `classification_id`, `registration_required`, `required_license_path`, `blocked_until`, `policy_decisions`, and `next_review_owner`.
API-004: `GET /jp/financial/classifications/{classification_id}` returns the immutable classification record and linked evidence versions.
API-005: `PATCH /jp/financial/classifications/{classification_id}` may update only review status, evidence links, and watchlist notes; activity type changes require a new classification version.
API-006: `POST /jp/financial/registrations` registers metadata for Banking Act, FIEA, or PSA license/registration evidence.
API-007: The registration API requires `registration_authority`, `registration_number`, `registered_entity_legal_name`, `scope_description`, `effective_at`, and `evidence_uri`.
API-008: The registration API requires `business_categories` for FIEA records and rejects category-free FIEA registration evidence.
API-009: The registration API requires `office_in_japan` and `representative_in_japan` for foreign entities when the statute gate demands local presence.
API-010: `POST /jp/financial/registrations/{registration_id}/verify` records a fresh verification of registry status and scope.
API-011: `POST /jp/financial/registrations/{registration_id}/suspend` marks operational freeze after regulatory suspension, rescission, or internal invalidation.
API-012: `POST /jp/fiea/activity-check` checks whether a proposed security, fund, derivative, advisory, solicitation, or management activity is permitted.
API-013: The FIEA activity-check request requires `product_security_classification`, `business_category`, `investor_classification`, `solicitation_target_region`, `customer_asset_control_model`, and `registration_id`.
API-014: The FIEA activity-check response returns `allow`, `required_registration_categories`, `required_disclosures`, `segregation_controls`, and `books_documents_profile`.
API-015: FIEA activity-check denies unknown investor status for professional-investor exceptions and returns general-investor default treatment.
API-016: FIEA activity-check denies representative-office-only tenants when `proposed_action` is solicitation, order routing, advice, execution, or discretionary management.
API-017: `POST /jp/fiea/solicitation-materials/review` stores advertisement, pitch, disclosure, and explanation artifacts for FIEA review.
API-018: FIEA solicitation material review requires registration category, investor class, fee disclosure, issuer disclosure, risk disclosure, and Japan language profile.
API-019: `POST /jp/fiea/customer-assets/segregation-check` validates that customer securities and money are managed separately from operator property.
API-020: `POST /jp/banking/license-check` determines whether a product implies banking activity under the Banking Act.
API-021: Banking license-check request requires `accepts_deposits`, `accepts_installment_savings`, `lends_funds`, `discounts_bills`, `deals_in_funds_transfer`, `uses_bank_label`, and `entity_license_id`.
API-022: Banking license-check response returns `bank_license_required`, `license_present`, `forbidden_labels`, `depositor_protection_required`, and `supervisory_state`.
API-023: Banking license-check denies any product label using bank, deposit, account, savings, or insured balance semantics when a non-bank tenant proposes the label.
API-024: `POST /jp/banking/foreign-branch-check` validates foreign bank branch principal, licensed branch, representative, and office metadata.
API-025: `POST /jp/banking/bank-agent-check` validates principal bank, bank-agent authority, customer explanation status, and prohibited-act controls.
API-026: `POST /jp/banking/electronic-payment-service-check` validates Article 52-61-2 registration for bank-linked electronic payment services.
API-027: `POST /jp/banking/regulatory-holds` records report requests, inspection notices, improvement orders, suspension orders, license rescission, and retained-assets-in-Japan orders.
API-028: Banking regulatory-hold API requires `hold_type`, `authority_source`, `received_at`, `affected_assets`, `affected_services`, `release_condition`, and `evidence_uri`.
API-029: `POST /jp/psa/prepaid/classify` classifies stored value as own-business, third-party, high-value electronically transferable, or not prepaid.
API-030: Prepaid classify request requires `redeemable_merchants`, `issuer_entity`, `transferability`, `unused_balance`, `base_date`, `instrument_label`, and `settlement_model`.
API-031: Prepaid classify response returns `prepaid_status`, `registration_required`, `security_deposit_required`, `reporting_required`, and `complaint_profile_required`.
API-032: `POST /jp/psa/prepaid/reports` records Article 23 report data: issued amount, unused balance, security deposit, base date, attachments, and submission status.
API-033: `POST /jp/psa/funds-transfer/classify` classifies non-bank funds transfer services into Type I, Type II, Type III, or not applicable.
API-034: Funds-transfer classify request requires `transfer_amount`, `transfer_timing`, `funds_holding_duration`, `user_obligation_amount`, `deposit_taking_institution`, and `security_deposit_model`.
API-035: Funds-transfer classify response returns `funds_transfer_type`, `registration_required`, `security_deposit_profile`, `user_disclosure_profile`, and `duration_limit`.
API-036: `POST /jp/psa/electronic-payment-instruments/check` validates registration and solicitation status for electronic payment instruments service flows.
API-037: Electronic payment instruments check denies foreign provider solicitation into Japan without Article 62-3 registration evidence.
API-038: `POST /jp/psa/cryptoasset-exchange/check` checks cryptoasset exchange service registration before fiat-crypto exchange, crypto-crypto exchange, intermediation, brokerage, agency, or custody workflows.
API-039: Cryptoasset exchange check request requires `registration_id`, `handled_cryptoasset_name`, `service_method`, `custody_model`, `advertising_profile_id`, and `representative_in_japan`.
API-040: Cryptoasset exchange check response returns `allow`, `missing_registration_items`, `asset_name_registered`, `custody_controls_required`, `ad_disclosures_required`, and `annual_report_status`.
API-041: `POST /jp/psa/cryptoasset-advertisements/review` validates registration number, trade name, non-currency disclaimer, material characteristics, and misleading-profit-solicitation prohibition.
API-042: `POST /jp/psa/cryptoasset-custody/segregation-check` validates segregation of user money and user cryptoassets from provider property.
API-043: `POST /jp/psa/cryptoasset/reports` records annual report, managed-money report, managed-cryptoasset report, audit attachment, and submission state.
API-044: `POST /jp/financial/outsourcing/review` registers entrusted-party chains for FIEA, Banking Act, and PSA operations.
API-045: Outsourcing review request requires `regulated_service_type`, `outsourced_function`, `provider_legal_name`, `provider_region`, `multi_tier_allowed`, `control_evidence`, and `exit_plan`.
API-046: `POST /jp/financial/information-security/review` binds leakage, loss, damage, access-control, encryption, recovery, and incident controls to regulated services.
API-047: `POST /jp/financial/regulator-notices` ingests regulator notices into the compliance queue and attaches operational holds when necessary.
API-048: Regulator notice ingestion requires `authority`, `notice_type`, `received_at`, `deadline`, `affected_registration_id`, `summary`, and `source_uri`.
API-049: `GET /jp/financial/policy-decisions/{decision_id}` returns Cedar policy evaluation inputs, outputs, matched authority, and denial reasons.
API-050: `POST /jp/financial/feature-flags/evaluate` evaluates global or tenant feature flags against JP-PACK-1 financial gates before exposure.
API-051: Feature flag evaluation requires `feature_id`, `tenant_id`, `region_exposure`, `activity_classification_id`, and `registration_ids`.
API-052: Feature flag evaluation returns `jp_allow`, `jp_blockers`, `missing_evidence`, `required_reviews`, and `audit_event_ids`.
API-053: `POST /jp/financial/counsel-reviews` creates counsel review work items for uncertain activity classification or claimed exemptions.
API-054: Counsel review request requires `uncertain_terms`, `product_description`, `user_region`, `asset_region`, `registration_candidate`, and `risk_owner`.
API-055: `POST /jp/financial/customer-disclosures` stores Japanese and English disclosure variants with versioned product, registration, fee, risk, and complaint references.
API-056: Customer disclosure API rejects retail Japan disclosure bundles that lack Japanese-language content.
API-057: `POST /jp/financial/composite-products/check` validates multi-statute products and returns per-component statute decisions.
API-058: Composite product check denies launch until all component gates pass or a counsel-approved removal narrows product scope.
API-059: `GET /jp/financial/watchlist` exposes statutory reform watch items without treating pending proposals as active controls.
API-060: Watchlist entries require `source`, `proposal_name`, `status`, `potential_impact`, `not_effective_until`, and `owner`.
API-061: All JP financial APIs must emit `jp_financial_pack_version=JP-PACK-1` in responses.
API-062: All JP financial denial responses must include at least one `authority_reference` or `policy_reference`.
API-063: All JP financial approval responses must include `evidence_freshness_at` for registration and classification inputs.
API-064: All JP financial APIs must preserve immutable request and decision snapshots for regulator, auditor, and internal review.
API-065: All JP financial APIs must fail closed when tenant region, user region, solicitation region, or asset region is unknown and Japan cannot be excluded.
API-066: All JP financial APIs must treat production, beta, sandbox invitation, demo, and partner test modes as potentially regulated if real Japan users, real assets, or product-specific solicitation are present.
API-067: All JP financial APIs must reject generic `global_finance_ok` overrides when a JP-PACK-1 statute gate is unresolved.
API-068: All JP financial APIs must expose `manual_review_reason` when a compliance owner overrides a deny state.
API-069: All JP financial APIs must attach audit event IDs to externalized user-visible approvals, denials, and disclosure publications.
API-070: All JP financial APIs must prohibit silent downgrade from Banking Act, FIEA, or PSA classification to generic commerce classification after evidence is collected.

## Audit Event Additions

Audit-001: Emit `jp.financial.classification.created` when a financial activity classification is created.
Audit-002: Emit `jp.financial.classification.updated` when review status, evidence, or watchlist linkage changes.
Audit-003: Emit `jp.financial.classification.denied_unknown_region` when Japan cannot be excluded and classification is missing.
Audit-004: Emit `jp.financial.registration.created` when registration metadata is entered.
Audit-005: Emit `jp.financial.registration.verified` when official registry or authority evidence is freshly verified.
Audit-006: Emit `jp.financial.registration.suspended` when internal or regulator state freezes a registration.
Audit-007: Emit `jp.financial.registration.scope_mismatch` when a product requests an activity outside registration scope.
Audit-008: Emit `jp.financial.registration.number_changed` when a new immutable registration-number version is added.
Audit-009: Emit `jp.financial.disqualification.review_started` for Article 29-4, Article 63-5, or analogous refusal-condition review.
Audit-010: Emit `jp.financial.disqualification.review_failed` when disqualifying actor or false/missing material registration information is found.
Audit-011: Emit `jp.fiea.activity_check.performed` for each Financial Instruments Business classification decision.
Audit-012: Emit `jp.fiea.type_i.denied_missing_registration` when Type I workflow lacks registration.
Audit-013: Emit `jp.fiea.type_ii.denied_missing_registration` when Type II workflow lacks registration.
Audit-014: Emit `jp.fiea.investment_management.denied_missing_registration` when discretionary or fund-management activity lacks registration.
Audit-015: Emit `jp.fiea.investment_advisory.denied_missing_registration` when compensated advisory activity lacks registration or exemption.
Audit-016: Emit `jp.fiea.solicitation.denied_cross_border` when unregistered foreign solicitation into Japan is blocked.
Audit-017: Emit `jp.fiea.representative_office.blocked_operational_activity` when a representative-office-only tenant attempts regulated activity.
Audit-018: Emit `jp.fiea.investor_classification.defaulted_general` when unknown investor status is treated as general investor.
Audit-019: Emit `jp.fiea.customer_asset.segregation_verified` when securities or money segregation checks pass.
Audit-020: Emit `jp.fiea.customer_asset.segregation_failed` when securities or money segregation evidence is missing or insufficient.
Audit-021: Emit `jp.fiea.books_documents.profile_enabled` when retention profile is bound to the regulated workflow.
Audit-022: Emit `jp.fiea.advertisement.review_passed` when solicitation material receives approval.
Audit-023: Emit `jp.fiea.advertisement.review_failed` when registration category, fee, risk, or issuer disclosure is missing.
Audit-024: Emit `jp.fiea.supervisory_order.hold_applied` when FSA improvement, suspension, or rescission state blocks onboarding.
Audit-025: Emit `jp.banking.license_check.performed` when Banking Act license analysis runs.
Audit-026: Emit `jp.banking.license_missing.denied` when banking activity lacks Article 4 license basis.
Audit-027: Emit `jp.banking.misleading_bank_label.denied` when non-bank product language implies bank status.
Audit-028: Emit `jp.banking.foreign_branch.denied_missing_license` when foreign branch metadata is missing.
Audit-029: Emit `jp.banking.bank_agent.denied_missing_principal` when bank-agent services lack principal bank authority.
Audit-030: Emit `jp.banking.electronic_payment_service.denied_missing_registration` when Article 52-61-2 registration is missing.
Audit-031: Emit `jp.banking.depositor_protection.escalated` when failure, transfer, merger, or dissolution creates depositor-protection workflow.
Audit-032: Emit `jp.banking.asset_retention_order.applied` when assets are frozen under retained-assets-in-Japan control.
Audit-033: Emit `jp.banking.report_request.received` when regulator report/material request is ingested.
Audit-034: Emit `jp.banking.inspection_notice.received` when on-site inspection notice is ingested.
Audit-035: Emit `jp.psa.prepaid.classification.performed` when stored-value classification runs.
Audit-036: Emit `jp.psa.prepaid.third_party.denied_missing_registration` when Article 7 registration is missing.
Audit-037: Emit `jp.psa.prepaid.unused_balance.report_due` when base-date unused-balance reporting becomes due.
Audit-038: Emit `jp.psa.prepaid.security_deposit.missing` when issuance security deposit data is missing.
Audit-039: Emit `jp.psa.prepaid.outsourcing.denied_missing_controls` when entrusted-party control evidence is missing.
Audit-040: Emit `jp.psa.prepaid.complaint_profile.enabled` when complaint-processing measures are configured.
Audit-041: Emit `jp.psa.prepaid.revocation_hold.applied` when suspension, revocation, or unknown-office public notice freezes issuance.
Audit-042: Emit `jp.psa.funds_transfer.classification.performed` when non-bank funds transfer classification runs.
Audit-043: Emit `jp.psa.funds_transfer.denied_missing_registration` when Article 37 registration is missing.
Audit-044: Emit `jp.psa.funds_transfer.security_deposit.missing` when deposit, guarantee, or trust evidence is insufficient.
Audit-045: Emit `jp.psa.funds_transfer.bank_confusion_disclosure.missing` when non-bank transfer disclosure is absent.
Audit-046: Emit `jp.psa.funds_transfer.information_security_missing` when leakage/loss/damage controls are absent.
Audit-047: Emit `jp.psa.funds_transfer.outsourcing_missing` when entrusted-party controls are absent.
Audit-048: Emit `jp.psa.electronic_payment_instrument.denied_missing_registration` when Article 62-3 registration is missing.
Audit-049: Emit `jp.psa.foreign_electronic_payment_instrument.solicitation_denied` when unregistered foreign solicitation targets Japan.
Audit-050: Emit `jp.psa.cryptoasset.exchange_check.performed` when cryptoasset exchange registration analysis runs.
Audit-051: Emit `jp.psa.cryptoasset.denied_missing_registration` when Article 63-2 registration is missing.
Audit-052: Emit `jp.psa.cryptoasset.asset_name_not_registered` when handled asset is outside approved registration evidence.
Audit-053: Emit `jp.psa.cryptoasset.name_lending.denied` when another person would operate in the provider's name.
Audit-054: Emit `jp.psa.cryptoasset.advertisement.review_failed` when registration number, trade name, non-currency disclaimer, or material characteristics are missing.
Audit-055: Emit `jp.psa.cryptoasset.misleading_profit_solicitation.denied` when ad language encourages profit-only purchase or exchange.
Audit-056: Emit `jp.psa.cryptoasset.user_property.segregation_verified` when user money and cryptoasset segregation is verified.
Audit-057: Emit `jp.psa.cryptoasset.user_property.segregation_failed` when segregation evidence is missing.
Audit-058: Emit `jp.psa.cryptoasset.periodic_audit.missing` when custody audit evidence is not present.
Audit-059: Emit `jp.psa.cryptoasset.annual_report.due` when annual report status becomes due.
Audit-060: Emit `jp.psa.cryptoasset.management_report.due` when managed user money or cryptoasset report data becomes due.
Audit-061: Emit `jp.financial.outsourcing.review_passed` when entrusted-party control review passes.
Audit-062: Emit `jp.financial.outsourcing.review_failed` when first-tier or multi-tier provider evidence is incomplete.
Audit-063: Emit `jp.financial.information_security.review_passed` when leakage/loss/damage prevention controls are bound.
Audit-064: Emit `jp.financial.regulator_notice.ingested` for any regulator notice affecting JP financial services.
Audit-065: Emit `jp.financial.regulatory_hold.released` when a hold release condition is satisfied and approved.
Audit-066: Emit `jp.financial.counsel_review.created` when uncertain classification or exemption review starts.
Audit-067: Emit `jp.financial.counsel_review.approved` when counsel approves an exemption or classification.
Audit-068: Emit `jp.financial.counsel_review.rejected` when counsel rejects a proposed classification or exemption.
Audit-069: Emit `jp.financial.composite_product.denied_component_failure` when any component statute gate fails.
Audit-070: Emit `jp.financial.feature_flag.denied_jp_pack` when JP-PACK-1 blocks a global financial feature for Japan exposure.

## Failure Modes

Failure-001: A product team calls wallet credit "points" while allowing third-party redemption; the pack treats the product as potential 前払式支払手段 and blocks launch until prepaid classification is complete.
Failure-002: A tenant claims a global e-money approval but lacks Japan Article 7 prepaid payment instrument registration; third-party prepaid issuance remains denied.
Failure-003: A tenant launches a gift balance for its own merchant store and later enables marketplace redemption; the system must create a new third-party prepaid classification.
Failure-004: A base-date unused balance report is incomplete; issuance may continue only if compliance confirms no report is due, otherwise reporting block applies.
Failure-005: A prepaid issuer outsources ledger operation but lacks entrusted-party guidance evidence; launch is denied under outsourcing controls.
Failure-006: A prepaid issuer cannot identify its office or representative during a public-notice state; issuance is frozen under revocation-risk controls.
Failure-007: A non-bank tenant labels a stored balance "deposit"; the Banking Act misleading-label gate blocks the copy and returns replacement wording.
Failure-008: A non-bank tenant offers indefinite funds holding before transfer; funds transfer classification must evaluate user-protection restrictions and obligation duration.
Failure-009: A funds transfer provider does not distinguish Type I, Type II, and Type III service paths; transfer execution is denied until classification is complete.
Failure-010: A funds transfer product lacks security deposit, guarantee, or trust evidence; the transfer route is blocked.
Failure-011: A funds transfer product presents itself as bank remittance; the anti-confusion disclosure gate blocks publication.
Failure-012: A bank-owned tenant assumes Banking Act license covers a non-bank affiliate; the pack denies affiliate use unless registration basis names the acting entity.
Failure-013: A foreign bank branch workflow lacks principal branch and local representative metadata; Japan branch operation is denied.
Failure-014: A bank agent opens customer onboarding before principal bank authority is recorded; bank-agent workflow is denied.
Failure-015: A bank integration receives an inspection notice and tries to purge logs under customer deletion; regulatory hold precedence blocks deletion.
Failure-016: A retained-assets-in-Japan order applies to a bank or watched cryptoasset reform state; asset movement is blocked according to hold scope.
Failure-017: A financial product uses "safe savings" marketing without a bank license; the pack treats it as misleading bank-status language.
Failure-018: A securities product launches as "education" but includes product-specific purchase invitation to Japan users; FIEA solicitation review is required.
Failure-019: A fund marketplace sells partnership interests to Japan investors without Type II or applicable registration analysis; solicitation is denied.
Failure-020: An investment advisory feature gives compensated recommendations but claims it is only analytics; the advisory classification gate blocks the service until remuneration and advice status are reviewed.
Failure-021: A discretionary portfolio automation service lacks Investment Management registration; execution and account connection are denied.
Failure-022: A foreign investment manager serves only a registered Japanese investment manager under a narrow exemption; the system permits only the counsel-approved counterparty and scope.
Failure-023: A representative office tenant begins collecting leads for product sale; representative-office-only mode blocks solicitation.
Failure-024: A tenant claims professional-investor treatment without contract-kind status evidence; general-investor protection applies.
Failure-025: A customer asset custody workflow lacks segregation controls; FIEA and PSA custody gates deny launch.
Failure-026: A financial instruments operator is under business improvement order; new onboarding is blocked until regulatory hold release.
Failure-027: A registration application contains missing material office information; registration-basis record cannot be marked verified.
Failure-028: A disqualified officer remains attached to a regulated tenant; FIEA or PSA registration usage is denied until remediation.
Failure-029: A product wraps cryptoasset derivatives into investment strategy notes; both FIEA and PSA cryptoasset gates must be evaluated.
Failure-030: A cryptoasset exchange tenant handles a token not listed in registration evidence; asset activation is denied.
Failure-031: A cryptoasset exchange ad lacks registration number; publication is denied.
Failure-032: A cryptoasset exchange ad says cryptoassets are equivalent to currency; publication is denied.
Failure-033: A cryptoasset exchange ad encourages profit-only speculation; publication is denied.
Failure-034: A cryptoasset exchange custodies user money in an operating account; user money segregation fails.
Failure-035: A cryptoasset exchange custodies user cryptoassets without segregation evidence; custody route fails.
Failure-036: A cryptoasset exchange delegates operations to an unregistered brand using its own name; name-lending gate denies.
Failure-037: A foreign cryptoasset exchange provider lacks a Japan representative for required registration metadata; Japan launch is blocked.
Failure-038: A cryptoasset exchange provider misses annual report data; regulated reporting hold applies.
Failure-039: A cryptoasset exchange provider lacks periodic audit evidence for custody controls; custody expansion is denied.
Failure-040: A product team uses "virtual currency" in current customer copy; compliance copy review converts to cryptoasset terminology unless legacy citation context requires otherwise.
Failure-041: A foreign electronic payment instrument provider solicits Japan users without Article 62-3 registration; solicitation is denied.
Failure-042: An electronic payment instrument product is misclassified as prepaid value; composite product review must decide PSA chapter routing.
Failure-043: A stablecoin-like product is classified globally as payment token but not evaluated under Japan electronic payment instrument rules; launch is denied.
Failure-044: A bank-linked screen scraping or payment initiation service lacks Banking Act electronic payment service registration review; integration is denied.
Failure-045: A customer support workflow deletes complaint evidence for prepaid or cryptoasset users; retention and complaint-processing gates block deletion.
Failure-046: A service provider claims "licensed in Japan" but registration scope only covers one asset or service type; scope mismatch event blocks broader use.
Failure-047: A global feature flag exposes investment advice to Japan without JP-PACK-1 evaluation; feature flag evaluation denies Japan exposure.
Failure-048: A sandbox invites Japan retail users to test real investment product workflows; sandbox is treated as regulated exposure and blocked.
Failure-049: A demo account uses real cryptoasset custody; test-tenant real-assets gate blocks the scenario.
Failure-050: A partner integration stores FIEA books outside retention profile; execution is denied until retention is bound.
Failure-051: A partner integration routes funds transfer through an outsourced processor without multi-tier mapping; outsourcing review fails.
Failure-052: A regulator notice is received by email but not ingested; no operational state change can clear the hold until notice evidence is registered.
Failure-053: A team tries to remove a regulator hold through feature flag rollback; hold release requires explicit release condition and approval.
Failure-054: A compliance owner approves a classification orally; API denies until evidence URI and review ID are stored.
Failure-055: A product combines Banking Act deposit semantics and PSA prepaid value; composite product review blocks single-statute approval.
Failure-056: A product combines FIEA solicitation and cryptoasset exchange custody; both FIEA and PSA gates must pass.
Failure-057: A product combines investment management and bank transfer authority; investment management and banking/payment checks must both pass.
Failure-058: A product claims no Japan users but has Japanese language landing pages, yen pricing, or Japan affiliates; Japan cannot be excluded and classification is required.
Failure-059: A tenant moves registration evidence from one legal entity to another tenant; immutable entity binding blocks reuse.
Failure-060: A registration number is overwritten instead of versioned; immutable audit policy rejects the update.
Failure-061: A watchlist proposal is treated as active law without effective-date confirmation; watchlist policy requires non-active status until enactment and effectiveness are verified.
Failure-062: A legal translation label conflicts with current Japanese statutory terminology; counsel review must resolve before launch.
Failure-063: A financial disclosure bundle lacks Japanese for retail Japan users; customer disclosure publication is blocked.
Failure-064: A customer data export omits registration decision IDs; audit export fails because decisions must be reconstructable.
Failure-065: A tenant has a revoked registration but cached allow decision; policy evaluation must re-check registration freshness before execution.
Failure-066: A product is lowered from regulated classification to generic commerce after denial; downgrade is prohibited without counsel-reviewed new facts.
Failure-067: A third-party marketplace tries to rely on seller registrations while Oyatie performs regulated intermediation; activity is classified at the actor performing the regulated act.
Failure-068: A customer-facing FAQ says "FSA approved this investment" merely because registration exists; advertising review blocks implication of product merit approval.
Failure-069: A bank API product suggests deposit insurance when not applicable; bank disclosure review blocks the output.
Failure-070: A cryptoasset report lacks managed user money amounts; annual report submission remains incomplete.
Failure-071: A funds transfer product holds user money after transfer purpose expires; obligation-duration policy triggers operational exception and user-fund return workflow.
Failure-072: A prepaid issuer omits unused balance and security deposit fields from base-date ledger; report generation fails.
Failure-073: A foreign entity lacks brought-in capital metadata where required for Type I registration records; registration evidence remains incomplete.
Failure-074: A Japan office location changes without registration update review; regulated activity pauses for scope freshness.
Failure-075: A director change occurs without officer snapshot update; registration evidence goes stale.
Failure-076: A cryptoasset handled-asset change is configured after the fact; pack requires advance or counsel-reviewed change evidence based on current authority.
Failure-077: A compliance user approves a policy denial without reason; override is rejected because manual review reason is required.
Failure-078: A product uses the Payment Services Act route for an activity that is actually deposit-taking; Banking Act gate supersedes.
Failure-079: A product uses Banking Act affiliate status to bypass FIEA for securities solicitation; FIEA gate still applies.
Failure-080: A product uses FIEA registration to bypass PSA cryptoasset custody controls; PSA gate still applies.
Failure-081: A mixed product cannot identify dominant statute; composite product remains blocked until all plausible statute gates are evaluated.
Failure-082: A system stores only final allow/deny and not input facts; audit reconstruction fails.
Failure-083: A partner webhook reports regulatory suspension but tenant state remains active; regulator notice ingestion must force hold.
Failure-084: A customer support agent manually enables Japan exposure after denial; feature flag policy blocks operational path and logs override attempt.
Failure-085: A system relies on stale annual verification for registration evidence; freshness check blocks launch until reverified.
Failure-086: A cryptoasset exchange provider is missing certified association or equivalent internal-rule evidence where required by registration review; compliance review remains pending.
Failure-087: A funds transfer provider omits AML/travel-rule related metadata for cryptoasset-adjacent transfer; AML watch gate escalates.
Failure-088: A prepaid product becomes transferable electronically at high value but remains in low-risk profile; classification update is required.
Failure-089: An investment strategy newsletter becomes individualized advice; advisory classification changes from reference-only to regulated.
Failure-090: A bank merger or business transfer changes license basis; successor license and depositor-protection workflow must be verified before migration.

## Worked Examples

Example-001: A Japan tenant wants to offer a yen wallet redeemable only at its own store.
Example-002: The system runs `/jp/psa/prepaid/classify` with issuer, redeemable merchant, transferability, unused balance, and base date.
Example-003: If classification stays own-business prepaid and thresholds do not trigger registration/reporting duties, the product remains under prepaid monitoring rather than third-party registration.
Example-004: If the tenant later enables third-party merchants, `jp_psa_prepaid_third_party_registration_required` blocks issuance until Article 7 registration evidence is added.
Example-005: A marketplace wants to issue points redeemable across independent merchants.
Example-006: The pack treats the points as potential third-party 前払式支払手段.
Example-007: The classification response requires Article 7 registration, unused-balance reporting, issuance security deposit data, complaint-processing profile, and outsourcing controls.
Example-008: A SaaS team wants to call customer balances "deposits" in a non-bank wallet.
Example-009: `jp_banking_misleading_bank_label.denied` fires because non-bank product copy implies bank status.
Example-010: The remediation is not merely wording; product classification must still evaluate prepaid or funds transfer status.
Example-011: A non-bank remittance service transfers yen between Japan users.
Example-012: The system runs funds transfer classification and requires Article 37 registration evidence.
Example-013: The service must show disclosures preventing confusion with bank funds transfers.
Example-014: If the service holds user funds beyond the permitted transfer-processing period, the obligation-duration failure mode triggers.
Example-015: A licensed bank tenant offers deposit accounts and funds transfer transactions.
Example-016: Banking Act Article 4 license metadata is required, and deposit labels are allowed only within the licensed bank scope.
Example-017: Bank report requests, inspection notices, business improvement orders, or asset retention orders create regulatory holds that override ordinary deletion and transfer rules.
Example-018: A foreign bank wants to expose a Japan branch product.
Example-019: The pack requires Japan branch license metadata, principal branch designation, local representative, and office information before onboarding.
Example-020: A partner wants to act as a bank agent for an existing licensed bank.
Example-021: Bank-agent check requires principal bank authority, customer explanations, prohibited-act controls, and audit retention.
Example-022: A broker-dealer tenant wants to sell listed securities to Japan retail users.
Example-023: The system classifies the activity as FIEA Type I candidate and requires registration category evidence.
Example-024: Advertising review requires registration category, risk, fee, issuer, and Japanese disclosure bundle.
Example-025: A platform wants to sell fund interests in a collective investment scheme.
Example-026: The pack evaluates Type II, Investment Management, and solicitation roles separately because fund creation, management, and distribution can be different regulated acts.
Example-027: A Japan investment manager wants discretionary authority over customer portfolios.
Example-028: The pack requires Investment Management registration and customer asset controls.
Example-029: If a foreign group company performs sub-management, the Japan registrant must record entrustment and monitoring evidence where required.
Example-030: An analytics feature provides general market news without individualized advice and no remuneration.
Example-031: The feature may be classified as reference-only, but the classification record must preserve the facts supporting non-advisory status.
Example-032: If the same feature adds portfolio-specific buy/sell recommendations for compensation, it becomes Investment Advisory and Agency candidate activity.
Example-033: A foreign investment advisor wants to advise only registered Japanese investment management companies.
Example-034: The pack requires counsel-reviewed exemption evidence and restricts the allow decision to the named counterparties and scope.
Example-035: A representative office wants to collect market information in Japan.
Example-036: The pack may allow representative-office-only mode but denies solicitation, order routing, execution, advice, or management.
Example-037: A cryptoasset exchange tenant wants to list BTC/JPY exchange.
Example-038: The pack requires Article 63-2 registration metadata, handled cryptoasset name, service method, custody model, and advertisement profile.
Example-039: User money and cryptoassets must be segregated from provider property.
Example-040: Advertising must include trade name, registration number, non-currency disclaimer, and material characteristics.
Example-041: A cryptoasset exchange tenant wants to add a new token.
Example-042: `jp_psa_cryptoasset_asset_name_not_registered` blocks activation until handled-asset registration or change-control evidence is approved.
Example-043: A foreign cryptoasset exchange wants Japan users but lacks a Japan representative.
Example-044: Registration evidence remains incomplete, and solicitation is denied.
Example-045: A stablecoin-like instrument is offered as an electronic payment instrument.
Example-046: The pack routes the product to Payment Services Act electronic payment instrument registration analysis and blocks foreign unregistered solicitation.
Example-047: A bank-linked payment initiation service wants to connect customer accounts.
Example-048: The pack evaluates Banking Act electronic payment service registration, customer explanations, and bank integration controls.
Example-049: A composite app offers fund investing, cash transfer, and cryptoasset custody.
Example-050: Composite product check returns FIEA, PSA funds-transfer, and PSA cryptoasset component decisions.
Example-051: Launch is denied until every component passes or the product scope removes failed components.
Example-052: A global feature flag accidentally exposes cryptoasset marketing to Japan.
Example-053: Feature flag evaluation blocks Japan exposure because JP-PACK-1 has not approved cryptoasset advertisement disclosures.
Example-054: A test tenant wants to simulate bank deposits with real yen.
Example-055: Test-tenant real-assets policy denies real yen deposit handling without regulated production classification.
Example-056: A regulator sends a report request concerning a bank integration.
Example-057: Regulator notice ingestion emits a hold and preserves logs, reports, materials, and customer records under regulatory retention.
Example-058: A compliance owner wants to clear a hold after remediation.
Example-059: Hold release requires authority source, release condition, approval, and evidence URI.
Example-060: A product owner claims a registration number from another group company.
Example-061: Entity binding rejects the evidence unless the acting tenant legal entity is named or valid agency authority is recorded.
Example-062: A Japanese retail disclosure bundle is submitted in English only.
Example-063: Customer disclosure API blocks publication until Japanese-language content is added.
Example-064: A cryptoasset exchange annual reporting job lacks user money management amounts.
Example-065: The annual report remains incomplete and expansion is blocked.
Example-066: A prepaid issuer changes offices and fails to update registration evidence.
Example-067: Evidence freshness fails, and new issuance is blocked until office scope is reverified.
Example-068: A funds transfer service changes processor.
Example-069: Outsourcing review must update entrusted-party chain, control evidence, and exit plan before production migration.
Example-070: A FIEA operator changes directors.
Example-071: Officer snapshot must update, and disqualification review must pass before registration evidence is treated as fresh.
Example-072: A marketing page says "FSA approved investment returns."
Example-073: Advertisement review blocks the page because registration cannot be presented as merit approval.
Example-074: A prepaid product promises principal protection like a bank deposit.
Example-075: The Banking Act label check and prepaid disclosure review both block the language.
Example-076: A cryptoasset ad says "guaranteed profit from token exchange."
Example-077: Misleading profit solicitation is denied and logged.
Example-078: A foreign electronic payment instrument provider sends Japan-targeted email.
Example-079: If Article 62-3 registration is absent, solicitation is denied.
Example-080: A product remains ambiguous after classification.
Example-081: The system creates counsel review and blocks launch until classification is resolved.

## Cross-References

CrossRef-001: README.md defines JP-PACK-1 overview, scope, precedence, and activated microservice map.
CrossRef-002: appi-personal-information-protection.md governs personal-information handling, consent, cross-border transfer, opt-out transfers, and anonymized information controls.
CrossRef-003: my-number-act-individual-numbers.md governs Individual Number permitted purpose, cross-tenant prohibition, penalties, and daily-call controls.
CrossRef-004: telecommunications-business-act.md governs telecom registration, secrecy of communications, and data retention orders.
CrossRef-005: cybersecurity-basic-act-incident-response.md governs cyber incident response, critical infrastructure designation, NISC/METI timelines, and cross-border attack notification.
CrossRef-006: This financial-services document should be evaluated with APPI whenever customer identity, transaction, account, asset, or investor profile data is processed.
CrossRef-007: This financial-services document should be evaluated with My Number controls whenever tax, withholding, account-opening, or statutory identification workflows request Individual Numbers.
CrossRef-008: This financial-services document should be evaluated with Telecommunications Business Act controls when wallet, banking, or cryptoasset messaging uses telecom transmission metadata.
CrossRef-009: This financial-services document should be evaluated with cybersecurity incident controls when cyber incidents affect bank, securities, funds transfer, prepaid, electronic payment instrument, or cryptoasset operations.
CrossRef-010: JP-PACK-1 financial classification must run before product catalog publication in Japan.
CrossRef-011: JP-PACK-1 financial classification must run before partner sandbox invitation when product-specific financial functionality is visible to Japan users.
CrossRef-012: JP-PACK-1 financial classification must run before public marketing localization into Japanese for finance-related products.
CrossRef-013: JP-PACK-1 financial classification must run before enabling yen-denominated stored value.
CrossRef-014: JP-PACK-1 financial classification must run before enabling Japan bank-account connection.
CrossRef-015: JP-PACK-1 financial classification must run before enabling securities order routing.
CrossRef-016: JP-PACK-1 financial classification must run before enabling fund interest distribution.
CrossRef-017: JP-PACK-1 financial classification must run before enabling discretionary investment algorithms.
CrossRef-018: JP-PACK-1 financial classification must run before enabling compensated investment recommendations.
CrossRef-019: JP-PACK-1 financial classification must run before enabling cryptoasset custody or exchange.
CrossRef-020: JP-PACK-1 financial classification must run before enabling electronic payment instrument services.
CrossRef-021: FIEA registration evidence must not be reused for Banking Act deposit-taking activity.
CrossRef-022: Banking Act license evidence must not be reused for FIEA securities solicitation activity.
CrossRef-023: Payment Services Act prepaid registration must not be reused for cryptoasset exchange activity.
CrossRef-024: Payment Services Act cryptoasset exchange registration must not be reused for Banking Act deposit-taking.
CrossRef-025: Payment Services Act funds transfer registration must not be reused for Banking Act bank status.
CrossRef-026: Payment Services Act electronic payment instrument registration must not be reused for prepaid instruments without classification review.
CrossRef-027: A global compliance allow decision must not override a JP-PACK-1 denial.
CrossRef-028: A tenant-level allow decision must not override a product-level scope mismatch.
CrossRef-029: A group registration must not override legal-entity mismatch.
CrossRef-030: A sandbox label must not override real-asset restrictions.
CrossRef-031: A beta label must not override solicitation rules.
CrossRef-032: A demo label must not override advertising review when Japan retail users can view the demo.
CrossRef-033: A watchlist proposal must not be treated as active law until enactment and effective date are confirmed.
CrossRef-034: A translation term must not override current Japanese statutory terminology without counsel review.
CrossRef-035: Registration number history must be immutable across all JP financial records.
CrossRef-036: Registration scope descriptions must be preserved with every allow decision.
CrossRef-037: Officer snapshots must be linked to registration freshness.
CrossRef-038: Japan representative metadata must be linked to foreign provider registration freshness.
CrossRef-039: Office-in-Japan metadata must be linked to FIEA Type I and foreign-provider registration freshness where required.
CrossRef-040: Public registry verification should be refreshed before first production launch.
CrossRef-041: Public registry verification should be refreshed after regulatory notice ingestion.
CrossRef-042: Public registry verification should be refreshed after officer, office, product, or asset-scope changes.
CrossRef-043: Public registry verification should be refreshed before annual compliance certification.
CrossRef-044: Books-and-documents retention must be tied to FIEA execution workflows.
CrossRef-045: Books-and-documents retention must be tied to prepaid issuance workflows.
CrossRef-046: Books-and-documents retention must be tied to cryptoasset exchange workflows.
CrossRef-047: Books-and-documents retention must be tied to bank supervision workflows.
CrossRef-048: Complaint-processing profiles must be tied to prepaid payment instrument workflows.
CrossRef-049: Complaint-processing profiles should be tied to cryptoasset user protection workflows.
CrossRef-050: Customer disclosure profiles must be tied to funds transfer workflows.
CrossRef-051: Customer disclosure profiles must be tied to FIEA solicitation workflows.
CrossRef-052: Customer disclosure profiles must be tied to cryptoasset advertisement workflows.
CrossRef-053: Japanese-language disclosure is required when Japan retail users are targeted.
CrossRef-054: Professional-investor status must be contract-kind specific.
CrossRef-055: Unknown investor status defaults to general investor.
CrossRef-056: Unknown user region defaults to Japan-required review when Japan cannot be excluded.
CrossRef-057: Unknown asset region defaults to Japan-required review when Japan cannot be excluded.
CrossRef-058: Unknown solicitation region defaults to Japan-required review when Japan cannot be excluded.
CrossRef-059: Unknown legal entity blocks registration evidence reuse.
CrossRef-060: Unknown registration scope blocks regulated workflow activation.
CrossRef-061: Unknown office location blocks foreign provider activation where local office is required.
CrossRef-062: Unknown Japan representative blocks foreign cryptoasset exchange activation.
CrossRef-063: Unknown handled cryptoasset name blocks cryptoasset exchange asset activation.
CrossRef-064: Unknown prepaid merchant redemption scope blocks prepaid classification.
CrossRef-065: Unknown funds transfer type blocks transfer execution.
CrossRef-066: Unknown FIEA business category blocks securities or investment workflow.
CrossRef-067: Unknown bank license status blocks deposit-label and banking workflow.
CrossRef-068: Unknown outsourcing chain blocks regulated outsourced operation.
CrossRef-069: Unknown information-security profile blocks PSA funds transfer and cryptoasset workflows.
CrossRef-070: Unknown audit profile blocks customer asset custody workflows.
CrossRef-071: Regulator holds override customer-configurable deletion.
CrossRef-072: Regulator holds override tenant-configurable retention shortening.
CrossRef-073: Regulator holds override feature-flag reactivation.
CrossRef-074: Regulator holds require explicit release condition.
CrossRef-075: Regulator holds require evidence URI before release.
CrossRef-076: Regulatory notices must create audit events before operational state changes.
CrossRef-077: Counsel reviews must create audit events before exemption use.
CrossRef-078: Manual overrides must include reason, owner, timestamp, and evidence.
CrossRef-079: Manual overrides do not remove statutory classification history.
CrossRef-080: Manual overrides do not permit non-existent registration evidence.
CrossRef-081: Composite products require per-component statute decisions.
CrossRef-082: Composite products deny launch when any component statute gate fails.
CrossRef-083: Composite products may launch narrowed scope only after failed components are removed and new classification is recorded.
CrossRef-084: Bank deposit semantics and prepaid value semantics require separate analysis.
CrossRef-085: Bank transfer semantics and non-bank funds transfer semantics require separate analysis.
CrossRef-086: Investment management and investment advice require separate FIEA category analysis.
CrossRef-087: Fund management and fund solicitation require separate actor analysis.
CrossRef-088: Cryptoasset exchange and cryptoasset derivatives require PSA plus FIEA analysis.
CrossRef-089: Electronic payment instruments and prepaid payment instruments require separate PSA chapter analysis.
CrossRef-090: Bank-linked electronic payment services require Banking Act chapter analysis.
CrossRef-091: FSA market-entry guidebook may guide category routing but does not replace statute and registration evidence.
CrossRef-092: Japanese Law Translation should be cited with URL and reviewed for version freshness.
CrossRef-093: FSA cryptoasset guidance should be cited for terminology and operational policy background.
CrossRef-094: FSA 2025 cryptoasset discussion paper should be treated as watchlist or policy background unless enacted controls are confirmed.
CrossRef-095: JP financial data model deltas must be migration-scoped to pack activation and not silently alter other localization packs.
CrossRef-096: JP financial API deltas must return pack version for audit traceability.
CrossRef-097: JP financial audit events must preserve decision inputs.
CrossRef-098: JP financial failure modes must be regression-tested before launch of affected services.
CrossRef-099: JP financial Cedar policies must fail closed for missing registration evidence.
CrossRef-100: JP financial Cedar policies must fail closed for stale evidence when freshness is required.
CrossRef-101: JP financial Cedar policies must fail closed for unknown Japan exposure.
CrossRef-102: JP financial Cedar policies must fail closed for missing Japanese retail disclosure.
CrossRef-103: JP financial Cedar policies must fail closed for missing outsourced-party controls.
CrossRef-104: JP financial Cedar policies must fail closed for missing information-security controls.
CrossRef-105: JP financial Cedar policies must fail closed for missing customer asset segregation controls.
CrossRef-106: JP financial Cedar policies must fail closed for missing books-and-documents retention.
CrossRef-107: JP financial Cedar policies must fail closed for missing complaint-processing measures where required.
CrossRef-108: JP financial Cedar policies must fail closed for missing user-protection disclosure.
CrossRef-109: JP financial Cedar policies must fail closed for misleading bank, investment, prepaid, funds transfer, electronic payment instrument, or cryptoasset claims.
CrossRef-110: JP financial Cedar policies must fail closed for registration status `suspended`, `rescinded`, `expired`, `withdrawn`, or `unknown`.
CrossRef-111: JP financial workflows must preserve APPI lawful-basis and cross-border-transfer evidence for personal data used in financial onboarding.
CrossRef-112: JP financial workflows must preserve My Number permitted-purpose evidence when Individual Numbers are used for tax or statutory reporting.
CrossRef-113: JP financial workflows must preserve telecom secrecy controls when communications metadata is processed for financial messaging.
CrossRef-114: JP financial workflows must preserve cybersecurity incident timelines when regulated financial services are disrupted.
CrossRef-115: JP financial workflows must preserve cross-border attack notifications when cyber incidents involve foreign infrastructure or attackers.
CrossRef-116: JP financial workflows must preserve critical infrastructure designation if the tenant is classified as critical financial infrastructure.
CrossRef-117: JP financial workflows must include daily evidence freshness checks for high-risk registration states.
CrossRef-118: JP financial workflows must include at least annual evidence refresh for stable registrations unless policy sets a shorter period.
CrossRef-119: JP financial workflows must include immediate refresh after public registry changes.
CrossRef-120: JP financial workflows must include immediate refresh after regulator notice ingestion.
CrossRef-121: JP financial workflows must include immutable versioning for legal-entity name, registration number, office, representative, officer, and scope changes.
CrossRef-122: JP financial workflows must attach source URLs to authority citations in generated compliance evidence.
CrossRef-123: JP financial workflows must cite FSA and Japanese Law Translation sources before using non-official commentary.
CrossRef-124: JP financial workflows must not rely on scraped unofficial license lists when official registry evidence is available.
CrossRef-125: JP financial workflows must not conflate Bank of Japan examination role with FSA licensing authority.
CrossRef-126: JP financial workflows must not present JFSA registration as endorsement of product returns.
CrossRef-127: JP financial workflows must not present PSA registration as insurance of prepaid or cryptoasset balances.
CrossRef-128: JP financial workflows must not present FIEA registration as approval of investment merit.
CrossRef-129: JP financial workflows must not present Banking Act license as cover for unlicensed affiliate activity.
CrossRef-130: JP financial workflows must not present overseas license as equivalent to Japan registration without counsel review.
CrossRef-131: JP financial workflows must not permit unregistered foreign electronic payment instrument solicitation into Japan.
CrossRef-132: JP financial workflows must not permit unregistered foreign cryptoasset exchange solicitation into Japan.
CrossRef-133: JP financial workflows must not permit unregistered FIEA solicitation into Japan.
CrossRef-134: JP financial workflows must not permit non-bank deposit claims.
CrossRef-135: JP financial workflows must not permit user asset commingling.
CrossRef-136: JP financial workflows must not permit unchecked customer disclosures.
CrossRef-137: JP financial workflows must not permit unchecked regulator notice closure.
CrossRef-138: JP financial workflows must not permit unchecked feature flag exposure to Japan.
CrossRef-139: JP financial workflows must not permit documentation-only compliance without Cedar policy, data model, API, audit, and failure mode hooks.
CrossRef-140: Checkpoint: this document is complete when line count is at least 600, required headings are present, all authority URLs are represented in frontmatter, and retired VCS ratchet verify/done/promote has accepted `jp_pack_docs:6`.
