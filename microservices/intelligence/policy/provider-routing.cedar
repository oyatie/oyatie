// intelligence µservice — provider-routing policy fragment
// Cedar v4
//
// Purpose: enforce per-pack provider routing; refuse cross-pack outbound; enforce HIPAA / FedRAMP
//          / KR-CSAP routing constraints; enforce CN-pack inbound-only invariant.
// Owner:   axis-intelligence + ops-security + council-privacy
// References: ARCHITECTURE.md §6 (Provider matrix); multi-region.md; compliance.md.

// =============================================================================
// Default deny — every routing decision must match a permit
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — pack-kr permitted providers
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-kr" &&
  resource has provider &&
  resource.provider in [
    "anthropic-eu", "openai-eu", "vllm-kr", "naver-hyperclova-x", "apple-foundation-on-device"
  ]
};

// =============================================================================
// PERMIT — pack-eu permitted providers
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-eu" &&
  resource has provider &&
  resource.provider in [
    "anthropic-eu", "openai-eu", "vertex-eu", "azure-openai-eu",
    "mistral-la-plateforme", "cohere-eu", "vllm-eu", "apple-foundation-on-device"
  ]
};

// =============================================================================
// PERMIT — pack-us permitted providers
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-us" &&
  resource has provider &&
  resource.provider in [
    "anthropic-us", "openai-us", "vertex-us", "bedrock-us", "azure-openai-us",
    "cohere-us", "vllm-us", "openrouter", "together", "groq", "replicate",
    "apple-foundation-on-device", "huggingface-inference"
  ]
};

// =============================================================================
// PERMIT — pack-us-healthcare (HIPAA): only BAA-signed providers
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-us-healthcare" &&
  resource has provider &&
  resource has baa_signed &&
  resource.baa_signed == true &&
  resource.provider in [
    "anthropic-via-bedrock-baa", "azure-openai-baa", "bedrock-titan-baa", "vllm-hipaa-eligible"
  ]
};

// =============================================================================
// PERMIT — pack-us-federal (FedRAMP High): Gov-cloud only
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-us-federal" &&
  resource has provider &&
  resource has fedramp_high &&
  resource.fedramp_high == true &&
  resource.provider in [
    "azure-openai-gov-cloud", "bedrock-gov-cloud"
  ]
};

// =============================================================================
// PERMIT — pack-cn (PIPL + Generative AI Provisions 2023): CN-resident only
// =============================================================================

permit (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-cn" &&
  resource has provider &&
  resource has cn_resident &&
  resource.cn_resident == true &&
  resource.provider in [
    "alibaba-qwen", "tencent-hunyuan", "baidu-ernie"
  ]
};

// =============================================================================
// FORBID — pack-cn outbound to non-CN providers
// =============================================================================

forbid (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  principal.pack == "pack-cn" &&
  resource has cn_resident &&
  resource.cn_resident == false
};

// =============================================================================
// PERMIT — pack-jp, pack-sg, pack-au, pack-in, pack-br, pack-ae, pack-ksa, pack-uk
// =============================================================================
//
// Pack-overlay-specific PERMIT fragments live at:
//   microservices/intelligence/policy/provider-routing-pack-<name>.cedar
// loaded conditionally via per-pack Kustomize overlay.

// =============================================================================
// FORBID — Cross-pack routing without SCC
// =============================================================================

forbid (
  principal,
  action == Action::"route_to_provider",
  resource in ProviderEndpoint::?p
)
when {
  principal has pack &&
  resource has provider_pack &&
  principal.pack != resource.provider_pack &&
  principal has scc_active &&
  principal.scc_active == false
};
