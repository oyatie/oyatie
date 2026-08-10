// Treasury abuse defence
// Cedar v4.2 LTS fragment
// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315
// SAP code: TRM
// Benchmarks: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank

forbid (principal, action, resource);

permit (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES"
};

permit (principal, action, resource)
when {
  context has bot_score &&
  context.bot_score < 40 &&
  context has scrape_score &&
  context.scrape_score < 30 &&
  context has spoof_score &&
  context.spoof_score < 20 &&
  context has default_path_latency_ms &&
  context.default_path_latency_ms <= 2
};

forbid (principal, action, resource)
when {
  context has bot_score &&
  context.bot_score >= 80 &&
  !(context has audience_type && context.audience_type == "EMERGENCY_SERVICES")
};
