---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: S-014
related_packs: [sox-404]
date: 2026-05-21
---

# Obligation Due-Basis Grammar

Obligation due dates are computed from contract anchor dates plus offsets. This document specifies the canonical due-basis expression grammar parsed by IP-027.

## Grammar (EBNF)

```
DueBasis      ::= AnchorRef Offset? AdjustmentList? Calendar?
AnchorRef     ::= ContractField | ContractMilestone | NamedEvent | LiteralDate
ContractField ::= "contract.effective_date"
               | "contract.execution_date"
               | "contract.expiration_date"
               | "contract.signature_date"
               | "contract.term_end_date"
               | "contract.first_invoice_date"
               | "contract.delivery_date"
ContractMilestone ::= "milestone." Identifier
NamedEvent    ::= "event." Identifier
LiteralDate   ::= ISO8601Date
Offset        ::= ("+" | "-") Quantity TimeUnit
Quantity      ::= Integer
TimeUnit      ::= "day" | "days" | "week" | "weeks" | "month" | "months"
               | "year" | "years" | "business_day" | "business_days"
AdjustmentList ::= "adjusted_for" Adjustment ("and" Adjustment)*
Adjustment    ::= "weekend_forward"
               | "weekend_backward"
               | "holiday_forward(" CalendarRef ")"
               | "holiday_backward(" CalendarRef ")"
               | "month_end"
               | "quarter_end"
Calendar      ::= "in_calendar(" CalendarRef ")"
CalendarRef   ::= "US_FEDERAL"
               | "UK_BANK_HOLIDAYS"
               | "EU_TARGET2"
               | "JP_GOV_HOLIDAYS"
               | "KR_GOV_HOLIDAYS"
               | "DE_GOV_HOLIDAYS"
               | "FR_GOV_HOLIDAYS"
               | tenant_calendar.Identifier
```

## Canonical examples

| Clause text | Due basis expression | Computed |
|---|---|---|
| "Payment due 30 days from invoice" | `event.invoice_issued + 30 days` | invoice + 30 |
| "Termination notice required 90 days before expiration" | `contract.expiration_date - 90 days` | exp - 90 |
| "Net-30" | `event.invoice_issued + 30 days` | invoice + 30 |
| "Net-60 end of month" | `event.invoice_issued + 60 days adjusted_for month_end` | invoice + 60, then month-end |
| "Within 5 business days of execution" | `contract.execution_date + 5 business_days in_calendar(US_FEDERAL)` | exec + 5 BD |
| "Quarterly report due 15 days after quarter end" | `event.quarter_end + 15 days` | qtr_end + 15 |
| "Annual report due 60 days after fiscal year end" | `contract.fiscal_year_end + 60 days` | FYE + 60 |
| "Notice within 10 days of breach discovery" | `event.breach_discovered + 10 days` | discovery + 10 |
| "Renewal notice not later than 60 days nor earlier than 120 days before expiration" | `contract.expiration_date - 60 days` (latest) + window | window: exp-120..exp-60 |
| "지급기일은 송장 발행일로부터 30일 이내" (KR) | `event.invoice_issued + 30 days` | invoice + 30 |
| "Fälligkeit 60 Tage nach Rechnungsdatum" (DE) | `event.invoice_issued + 60 days` | invoice + 60 |

## Calendar handling

Business-day computation honours the calendar specified in the contract or default tenant calendar. Calendars include:

- US Federal Holidays (5 USC § 6103).
- UK Bank Holidays (Banking and Financial Dealings Act 1971).
- EU TARGET2 (European Central Bank settlement calendar).
- JP Government Holidays (国民の祝日に関する法律).
- KR Government Holidays (관공서의 공휴일에 관한 규정).
- DE Federal + state-specific holidays.
- FR Government + bank calendar.

Tenant-defined calendars are also supported (e.g. internal-company observances).

## Window computation

For obligations with a window (e.g. "not later than 60 nor earlier than 120 days before"):

- `window_start`: anchor + earlier_offset.
- `window_end`: anchor + later_offset.
- Action is valid within the window only.

## Recurring obligations

Recurring obligations specify a cadence:

```
RecurringDueBasis ::= DueBasis "recurring" Cadence Until?
Cadence           ::= "monthly" | "quarterly" | "annually" | "every" Quantity TimeUnit
Until             ::= "until" (ContractField | LiteralDate | "termination")
```

Example: "Quarterly compliance report due 15 days after quarter end, until contract termination" →
`event.quarter_end + 15 days recurring quarterly until termination`.

## Computation engine

The due-basis expression is parsed by a Rust pest grammar (in `crates/oya-clm-due-basis-kernel/`) into an AST. The AST is evaluated against:

- Contract anchor dates (resolved when contract executed).
- Event timestamps (resolved when event occurs).
- Calendar (loaded at evaluation time for currency).

The computation is deterministic and idempotent; the same expression + same inputs yields the same date.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"DueBasisExpressionEval",
  resource is Obligation
) when {
  resource.due_basis_expression contains_unparseable_token
};
```

## Audit events

- `oya.contract.lifecycle.management.obligation.due_basis_resolved`
- `oya.contract.lifecycle.management.obligation.due_basis_recomputed_on_event`
- `oya.contract.lifecycle.management.obligation.window_entered`
- `oya.contract.lifecycle.management.obligation.window_expired`

## Standards references

- ISO 8601 (date and time formats).
- 5 USC § 6103 (US federal holidays).
- Banking and Financial Dealings Act 1971 (UK).
- TARGET2 calendar (ECB).
