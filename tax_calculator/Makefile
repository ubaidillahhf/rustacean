.PHONY: run test build clean

# Default target
all: test

# Run the application
run:
	cargo run

# Run tests
test:
	cargo test -- --nocapture

# Run tests with coverage
coverage:
	cargo tarpaulin --ignore-tests --out Html
	@echo "Coverage report generated at: file://$(shell pwd)/tarpaulin-report.html"

# Build in release mode
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Format code
format:
	cargo fmt --all

# Check code style and common issues
check:
	cargo clippy -- -D warnings

# Install development dependencies
setup:
	rustup component add rustfmt
	rustup component add clippy
	cargo install cargo-watch

# Watch for changes and run tests
watch:
	cargo watch -x test

# Run with specific input (example: make run-args ARGS="1 6000000 1 0")
run-args:
	cargo run -- $(ARGS)
