# Default locations
EXTENSION_NAME := gemini-marketplace
EXTENSION_BASE := $(if $(GEMINI_CONFIG),$(GEMINI_CONFIG),$(HOME)/.gemini)
EXTENSION_INSTALL_DIR := $(EXTENSION_BASE)/extensions/$(EXTENSION_NAME)

.PHONY: help fmt lint test check local-publish

help:
	@echo "Available commands:"
	@echo "  make fmt            # cargo fmt"
	@echo "  make lint           # cargo clippy --all-targets --all-features -D warnings"
	@echo "  make test           # cargo test"
	@echo "  make check          # cargo check"
	@echo "  make local-publish  # rebuild binary and sync into $(EXTENSION_INSTALL_DIR)"

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

check:
	cargo check

local-publish: fmt lint test
	@echo "Installing binary to Cargo bin directory..."
	cargo install --path . --force
	@echo "Publishing extension files to $(EXTENSION_INSTALL_DIR)"
	rm -rf "$(EXTENSION_INSTALL_DIR)"
	mkdir -p "$(EXTENSION_INSTALL_DIR)"
	find . -mindepth 1 -maxdepth 1 \
		! -name 'target' \
		! -name '.git' \
		! -name '.github' \
		! -name '.idea' \
		-exec cp -R {} "$(EXTENSION_INSTALL_DIR)/" \;
	printf '{\n  "source": "%s",\n  "type": "local"\n}\n' "$(EXTENSION_INSTALL_DIR)" > "$(EXTENSION_INSTALL_DIR)/.gemini-extension-install.json"
	@echo "Local publish complete. Run 'gemini extensions list' to confirm the update."
