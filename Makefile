SHELL := /bin/sh
.DEFAULT_GOAL := help

TOFU ?= tofu
CARGO ?= cargo
OCI_DIR := infra/oci
CLOUDFLARE_DIR := infra/cloudflare

.PHONY: help bootstrap install plan apply tofu-init tofu-fmt-check verify verify-deploy-contract ops check-tofu

help:
	@printf '%s\n' 'Oyatie deployment entrypoints (OpenTofu + Rust gates; no SSH troubleshooting)'
	@printf '%s\n' ''
	@printf '%s\n' '  make bootstrap              Verify contract + initialize OpenTofu roots'
	@printf '%s\n' '  make install                Apply OpenTofu desired state for OCI + Cloudflare'
	@printf '%s\n' '  make plan                   Preview OCI + Cloudflare OpenTofu changes'
	@printf '%s\n' '  make apply                  Apply OCI + Cloudflare OpenTofu changes'
	@printf '%s\n' '  make ops                    Show day-2 ops surface'
	@printf '%s\n' '  make verify                 Run deployment contract gate + OpenTofu fmt check'

bootstrap: verify-deploy-contract check-tofu tofu-init

install: apply

plan: check-tofu verify-deploy-contract
	$(TOFU) -chdir=$(OCI_DIR) plan -input=false
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) plan -input=false

apply: check-tofu verify-deploy-contract
	$(TOFU) -chdir=$(OCI_DIR) apply -input=false
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) apply -input=false

tofu-init: check-tofu
	$(TOFU) -chdir=$(OCI_DIR) init -input=false
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) init -input=false

tofu-fmt-check: check-tofu
	$(TOFU) -chdir=$(OCI_DIR) fmt -check -recursive
	$(TOFU) -chdir=$(CLOUDFLARE_DIR) fmt -check -recursive

verify: verify-deploy-contract tofu-fmt-check

verify-deploy-contract:
	$(CARGO) run -p oya-dev-cli -- gate validate deployment-ops-contract

ops:
	@printf '%s\n' 'Day-2 operations surface: https://ops.oyatie.com'
	@printf '%s\n' 'Route drift repair, incident work, and desired-state changes through ops/OpenTofu/Rust controllers.'
	@printf '%s\n' 'Direct host troubleshooting is intentionally not a supported deployment path.'

check-tofu:
	@command -v '$(TOFU)' >/dev/null 2>&1 || { \
		printf '%s\n' "OpenTofu binary '$(TOFU)' not found. Install via official OpenTofu packages or set TOFU=/path/to/tofu."; \
		printf '%s\n' 'Install docs: https://opentofu.org/docs/intro/install/'; \
		exit 127; \
	}
