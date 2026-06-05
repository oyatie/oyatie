SHELL := /bin/sh
.DEFAULT_GOAL := help

TOFU ?= tofu
BUCK2 ?= buck2
# OpenTofu now owns only the Cloudflare edge. The cluster fleet is provisioned by
# Cluster API + Talos bootstrap plus Oyatie-native CUE/KRM desired-state,
# release-conveyor, and controller reconciliation. This Makefile is a
# pointer-thin edge convenience surface, not a deployment orchestrator.
CLOUDFLARE_DIR := infra/cloudflare

.PHONY: help bootstrap install plan apply tofu-init tofu-fmt-check verify verify-deploy-contract ops fleet check-tofu

help:
	@printf '%s\n' 'Oyatie deployment entrypoints (OpenTofu Cloudflare edge only; Buck2/Prow + CUE/KRM authority)'
	@printf '%s\n' ''
	@printf '%s\n' '  make bootstrap              Verify contract + initialize the OpenTofu edge root'
	@printf '%s\n' '  make install                Apply OpenTofu desired state (Cloudflare edge)'
	@printf '%s\n' '  make plan                   Preview Cloudflare edge changes'
	@printf '%s\n' '  make apply                  Apply Cloudflare edge changes'
	@printf '%s\n' '  make fleet                  Show native fleet desired-state references'
	@printf '%s\n' '  make ops                    Show day-2 ops surface'
	@printf '%s\n' '  make verify                 Run Buck2/Prow hygiene checks + OpenTofu fmt check'

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

verify: verify-deploy-contract tofu-fmt-check

verify-deploy-contract:
	$(BUCK2) build //:repo-hygiene-automation-check //:kubernetes-native-anti-pattern-check //:cloud-cell-elasticity-policy-check

# Cluster fleet is declarative desired state and controller reconciliation, not a Makefile concern.
fleet:
	@printf '%s\n' 'Fleet authority: specs/deployment-ops-contract.json + specs/kubernetes-native-anti-patterns.json'
	@printf '%s\n' 'Desired state: service-owned CUE/KRM packages validated by Buck2/Prow required-check evidence'
	@printf '%s\n' 'Application: native release-conveyor/KRM and Rust controllers converge state through operation ledgers'
	@printf '%s\n' 'Bootstrap: Talos/CAPI hardware-gated prelude remains documentation-only until native cloud-cell controllers own it'
	@printf '%s\n' 'This target intentionally prints references only; it does not deploy, template charts, or mutate Kubernetes.'

ops:
	@printf '%s\n' 'Day-2 operations surface: https://ops.oyatie.com'
	@printf '%s\n' 'Route drift repair, incident work, and desired-state changes through ops, native release-conveyor/KRM, CUE/KRM packages, OpenTofu edge-only changes, and Rust controllers.'
	@printf '%s\n' 'Direct host troubleshooting is intentionally not a supported deployment path.'

check-tofu:
	@command -v '$(TOFU)' >/dev/null 2>&1 || { \
		printf '%s\n' "OpenTofu binary '$(TOFU)' not found. Install via official OpenTofu packages or set TOFU=/path/to/tofu."; \
		printf '%s\n' 'Install docs: https://opentofu.org/docs/intro/install/'; \
		exit 127; \
	}
