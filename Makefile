# Jigen Build System
.PHONY: all build test bench doc dashboard clean help

# Default target
all: build test

# Build the Rust project
build:
	cargo build --release

# Run tests
test:
	cargo test --lib
	cargo test --test dsl_parser_tests
	cargo test --test scene_graph_tests
	cargo test --test physics_tests
	cargo test --test renderer_tests
	cargo test --test integration_tests
	cargo test --test edge_case_tests

# Run benchmarks
bench:
	cargo bench

# Build documentation dashboard
dashboard:
	cd apps/doc && npm run build
	mkdir -p docs
	cp -r apps/doc/dist/* docs/

# Serve documentation dashboard locally
serve-dashboard:
	cd apps/doc && npm run dev

# Run full CI pipeline
ci: test bench dashboard

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/
	rm -rf apps/doc/dist/
	rm -rf docs/

# Performance analysis
perf: bench dashboard

# Development workflow
dev:
	cargo watch -x 'test --lib'

# Help
help:
	@echo "Jigen Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all             - Build and test everything"
	@echo "  build           - Build the Rust project"
	@echo "  test            - Run all tests"
	@echo "  bench           - Run performance benchmarks"
	@echo "  dashboard       - Build documentation dashboard"
	@echo "  serve-dashboard - Serve dashboard locally for development"
	@echo "  ci              - Run full CI pipeline (test + bench + dashboard)"
	@echo "  perf            - Run performance analysis (bench + dashboard)"
	@echo "  dev             - Run tests in watch mode during development"
	@echo "  clean           - Clean all build artifacts"
	@echo "  help            - Show this help message"
