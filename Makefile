# Default locations
EXTENSION_NAME := gemini-marketplace
EXTENSION_BASE := $(if $(GEMINI_CONFIG),$(GEMINI_CONFIG),$(HOME)/.gemini)
EXTENSION_INSTALL_DIR := $(EXTENSION_BASE)/extensions/$(EXTENSION_NAME)
DIST_DIR := dist
EXTENSION_ARCHIVE := $(DIST_DIR)/gemini-marketplace-extension.tar.gz
DEMO_HOME := $(shell mktemp -d 2>/dev/null || mktemp -d -t gemini-marketplace-demo)
DEMO_FIXTURES := crates/marketplace-core/tests/data/marketplace/curated
DEMO_PORT := 8137

.PHONY: help fmt lint test check contract-lint demo demo-mcp local-publish extension-archive

help:
	@echo "Available commands:"
	@echo "  make fmt            # cargo fmt --all"
	@echo "  make lint           # cargo clippy --workspace --all-targets --all-features -D warnings"
	@echo "  make test           # cargo test --workspace"
	@echo "  make check          # cargo check --workspace"
	@echo "  make contract-lint  # lint the OpenAPI contract"
	@echo "  make demo           # run CLI walkthrough (list/search/status)"
	@echo "  make demo-mcp       # run MCP server + harness walkthrough"
	@echo "  make local-publish  # rebuild binary and sync into $(EXTENSION_INSTALL_DIR)"
	@echo "  make extension-archive  # build dist/gemini-marketplace-extension.tar.gz for remote installs"

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace

check:
	cargo check --workspace

contract-lint:
	@scripts/lint-openapi.sh

demo:
	@bash -c 'set -euo pipefail; \
	  rm -rf $(DEMO_HOME); \
	  mkdir -p $(DEMO_HOME); \
	  cp $(DEMO_FIXTURES)/*.json $(DEMO_HOME)/; \
	  printf "{\n  \"manifests\": [\n    \"http://127.0.0.1:$(DEMO_PORT)/curated-observability.json\",\n    \"http://127.0.0.1:$(DEMO_PORT)/curated-analysis-suite.json\"\n  ]\n}\n" > $(DEMO_HOME)/index.json; \
	  DEMO_SERVER_PID=$$(cd $(DEMO_HOME) && python3 -m http.server $(DEMO_PORT) --bind 127.0.0.1 >/dev/null 2>&1 & echo $$!); \
	  cleanup() { kill $$DEMO_SERVER_PID 2>/dev/null || true; }; \
	  trap cleanup EXIT INT TERM; \
	  export GEMINI_MARKETPLACE_HOME=$(DEMO_HOME); \
	  export GEMINI_MARKETPLACE_SOURCE_URL=http://127.0.0.1:$(DEMO_PORT)/index.json; \
	  echo "Running gemini marketplace CLI demo (isolated state at $(DEMO_HOME))..."; \
	  cargo run -p gemini-marketplace -- --help >/dev/null; \
	  cargo run -p gemini-marketplace -- list --json | head -n 5; \
	  cargo run -p gemini-marketplace -- search caching --json | head -n 5; \
	  cargo run -p gemini-marketplace -- status --json'

demo-mcp:
	@bash -c 'set -euo pipefail; \
	  rm -rf $(DEMO_HOME); \
	  mkdir -p $(DEMO_HOME); \
	  cp $(DEMO_FIXTURES)/*.json $(DEMO_HOME)/; \
	  printf "{\n  \"manifests\": [\n    \"http://127.0.0.1:$(DEMO_PORT)/curated-observability.json\",\n    \"http://127.0.0.1:$(DEMO_PORT)/curated-analysis-suite.json\"\n  ]\n}\n" > $(DEMO_HOME)/index.json; \
	  DEMO_SERVER_PID=$$(cd $(DEMO_HOME) && python3 -m http.server $(DEMO_PORT) --bind 127.0.0.1 >/dev/null 2>&1 & echo $$!); \
	  cleanup() { kill $$DEMO_SERVER_PID 2>/dev/null || true; }; \
	  trap cleanup EXIT INT TERM; \
	  export GEMINI_MARKETPLACE_HOME=$(DEMO_HOME); \
	  export GEMINI_MARKETPLACE_SOURCE_URL=http://127.0.0.1:$(DEMO_PORT)/index.json; \
	  echo "Running MCP server + harness demo..."; \
	  cargo build -p marketplace-mcp-server -p marketplace-mcp-cli >/dev/null; \
	  cargo run -p marketplace-mcp-cli -- --server-bin target/debug/marketplace-mcp-server -- list --json | head -n 5'

local-publish: fmt lint test
	@echo "Building release binary..."
	cargo build --release -p gemini-marketplace
	@echo "Updating cargo bin directory (optional)..."
	@if cargo install --path crates/marketplace-core --force >/dev/null 2>&1; then \
		printf "Updated gemini-marketplace binary via cargo install.\\n"; \
	else \
		printf "warning: cargo install failed (often due to permission issues); continuing with extension publish.\\n"; \
	fi
	@echo "Publishing extension manifest + commands to $(EXTENSION_INSTALL_DIR)"; \
	if [ -d "$(EXTENSION_INSTALL_DIR)" ]; then \
		if rm -rf "$(EXTENSION_INSTALL_DIR)"; then \
			printf "Removed previous install at $(EXTENSION_INSTALL_DIR).\\n"; \
		else \
			printf "\\nerror: unable to delete $(EXTENSION_INSTALL_DIR). Ensure you own the directory or set GEMINI_CONFIG to an alternate path (see README).\\n"; \
			exit 1; \
		fi; \
	fi; \
	mkdir -p "$(EXTENSION_INSTALL_DIR)"; \
	install -m 0644 gemini-extension.json "$(EXTENSION_INSTALL_DIR)/gemini-extension.json"; \
	cp -R commands "$(EXTENSION_INSTALL_DIR)/"; \
	printf '{\\n  "source": "%s",\\n  "type": "local"\\n}\\n' "$(EXTENSION_INSTALL_DIR)" > "$(EXTENSION_INSTALL_DIR)/.gemini-extension-install.json"
	@echo "Local publish complete. Run 'gemini extensions list' to confirm the update."

extension-archive: fmt lint test
	@echo "Building extension archive for remote installs..."
	rm -rf "$(DIST_DIR)"
	mkdir -p "$(DIST_DIR)/extension"
	install -m 0644 gemini-extension.json "$(DIST_DIR)/extension/gemini-extension.json"
	cp -R commands "$(DIST_DIR)/extension/"
	tar -czf "$(EXTENSION_ARCHIVE)" -C "$(DIST_DIR)" extension
	@echo "Created $(EXTENSION_ARCHIVE). Upload this artifact and instruct users to extract it into ~/.gemini/extensions/$(EXTENSION_NAME)."
