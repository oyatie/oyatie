SHELL := /bin/sh
.DEFAULT_GOAL := help

TOFU ?= tofu
CARGO ?= cargo
# OpenTofu now owns only the Cloudflare edge. The cluster fleet is provisioned by
# Cluster API + Talos (installation-media zero-touch) + per-cell Argo CD — see infra/capi, infra/talos/installation-media,
# infra/gitops (ADR-0375, supersedes the OCI/on-prem deployment model of ADR-0120/0121).
CLOUDFLARE_DIR := infra/cloudflare

.PHONY: help bootstrap install plan apply tofu-init tofu-fmt-check verify-deploy verify-deploy-contract ops fleet check-tofu

help:
	@printf '%s\n' 'Oyatie deployment entrypoints (OpenTofu edge + CAPI/Talos fleet; no SSH troubleshooting)'
	@printf '%s\n' ''
	@printf '%s\n' '  make bootstrap              Verify contract + initialize the OpenTofu edge root'
	@printf '%s\n' '  make install                Apply OpenTofu desired state (Cloudflare edge)'
	@printf '%s\n' '  make plan                   Preview Cloudflare edge changes'
	@printf '%s\n' '  make apply                  Apply Cloudflare edge changes'
	@printf '%s\n' '  make fleet                  Show the Talos/CAPI fleet bring-up entrypoints'
	@printf '%s\n' '  make ops                    Show day-2 ops surface'
	@printf '%s\n' '  make verify-deploy          Run deployment contract gate + OpenTofu fmt check'
	@printf '%s\n' ''
	@printf '%s\n' '  cargo verify is NOT a Make target. Merge-path verify is:'
	@printf '%s\n' '    cargo fmt --all --check'
	@printf '%s\n' '    cargo clippy --workspace --all-targets -- -D warnings'
	@printf '%s\n' '    cargo test --workspace'

bootstrap: verify-deploy-contract check-tofu tofu-init

install: apply

plan: check-tofu verify-deploy-contract
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) plan -input=false

apply: check-tofu verify-deploy-contract
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) apply -input=false

tofu-init: check-tofu
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) init -input=false

tofu-fmt-check: check-tofu
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) fmt -check -recursive

# Named verify-deploy so it cannot be mistaken for README/AGENTS cargo verify (ADR-0716).
verify-deploy: verify-deploy-contract tofu-fmt-check

verify-deploy-contract:
	$(CARGO) run -p marketplace-dev-cli -- gate validate deployment-ops-contract

# Cluster fleet is declarative + git-driven (CAPI/Talos/Argo CD), not a Makefile concern.
fleet:
	@printf '%s\n' 'Talos + Cluster API + Argo CD fleet (ADR-0375):'
	@printf '%s\n' '  Control-plane media : CONTROLPLANE_ENDPOINT=https://<cp-ip>:6443 infra/talos/installation-media/gen-media.sh control-plane'
	@printf '%s\n' '  Node media    : CONFIG_URL=https://join.oyatie.dev/config infra/talos/installation-media/gen-media.sh node'
	@printf '%s\n' '  CAPI install  : KUBECONFIG=<control-plane> infra/capi/init.sh   (then infra/capi/crs/render.sh)'
	@printf '%s\n' '  Spokes        : add cells to infra/capi/clusters/values.yaml, then'
	@printf '%s\n' '                  helm template oya-spokes infra/capi/clusters -f <cells>.yaml | kubectl apply -f - ; CAPI reconciles'
	@printf '%s\n' '  NOTE: fleet bring-up is hardware-gated + multi-step (boot media on real nodes); this target only PRINTS the sequence.'

ops:
	@printf '%s\n' 'Day-2 operations surface: https://ops.oyatie.com'
	@printf '%s\n' 'Route drift repair, incident work, and desired-state changes through ops/OpenTofu/Argo CD/Rust controllers.'
	@printf '%s\n' 'Direct host troubleshooting is intentionally not a supported deployment path.'

check-tofu:
	@command -v '$(TOFU)' >/dev/null 2>&1 || { \
		printf '%s\n' "OpenTofu binary '$(TOFU)' not found. Install via official OpenTofu packages or set TOFU=/path/to/tofu."; \
		printf '%s\n' 'Install docs: https://opentofu.org/docs/intro/install/'; \
		exit 127; \
	}
