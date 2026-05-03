.PHONY: help api api-step9 agent test-storage test-storage-impl rag analyze clean

# Default target
help:
	@echo "Available targets:"
	@echo "  make api                 - Run API server (original)"
	@echo "  make api-step9           - Run API server (Step 9.9 Framework)"
	@echo "  make agent               - Run agent example"
	@echo "  make test-storage        - Run test_og_storage binary"
	@echo "  make test-storage-impl   - Run test_og_storage_implement example"
	@echo "  make rag                 - Run RAG pipeline binary"
	@echo "  make agent      - Run analyze_contract.sh script"
	@echo "  make clean               - Clean build artifacts"

# API Servers
api:
	cargo run --bin api

api-step9:
	cargo run --bin api_step9_9

# Examples
agent:
	cargo run --example agent_example

test-storage-impl:
	cargo run --example test_og_storage_implement

# Binaries
test-storage:
	cargo run --bin test_og_storage

rag:
	cargo run --bin rag_pipeline

# Shell Scripts
agent:
	./agent_analyze.sh

# Utility
clean:
	cargo clean
