.PHONY: all test bench docs clean install help

# Default target
all: test

# Run all tests
test:
	@echo "Running all tests..."
	cargo test --all-targets

# Run tests with output
test-verbose:
	@echo "Running tests with verbose output..."
	cargo test -- --nocapture --test-threads=1

# Run specific test suite
test-predictive:
	cargo test --test predictive_tests

test-warming:
	cargo test --test warming_tests

test-tiering:
	cargo test --test tiering_tests

test-coherence:
	cargo test --test coherence_tests

# Run property-based tests
test-prop:
	@echo "Running property-based tests..."
	cargo test prop_ -- --test-threads=1

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	cargo bench

# Run specific benchmark
bench-predictive:
	cargo bench --bench predictive_cache_bench

bench-warming:
	cargo bench --bench cache_warming_bench

bench-tiering:
	cargo bench --bench dynamic_tiering_bench

bench-coherence:
	cargo bench --bench coherence_bench

bench-throughput:
	cargo bench --bench throughput_bench

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

# Build examples
examples:
	@echo "Building examples..."
	cargo build --examples

# Run example
run-example-basic:
	cargo run --example basic_optimizer

run-example-predictive:
	cargo run --example predictive_warming

run-example-tiering:
	cargo run --example dynamic_tiering

# Format code
fmt:
	cargo fmt

# Check code formatting
fmt-check:
	cargo fmt -- --check

# Run linter
clippy:
	cargo clippy -- -D warnings

# Run all checks
check: fmt-check clippy
	@echo "All checks passed!"

# Generate code coverage
coverage:
	@echo "Generating code coverage..."
	cargo tarpaulin --out Html --output-dir coverage/

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install dependencies
install:
	@echo "Installing dependencies..."
	cargo fetch

# Build release version
build-release:
	@echo "Building release version..."
	cargo build --release

# Run CI tests
ci: test
	@echo "CI tests passed!"

# Performance validation
perf-validate:
	@echo "Validating performance targets..."
	cargo test --release --test performance_tests

# Generate flamegraph
flamegraph:
	@echo "Generating flamegraph..."
	cargo bench --bench throughput_bench -- --profile-time=5

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit

# Update dependencies
update:
	@echo "Updating dependencies..."
	cargo update

# Show help
help:
	@echo "Available targets:"
	@echo "  make test          - Run all tests"
	@echo "  make test-verbose  - Run tests with verbose output"
	@echo "  make test-predictive - Run predictive tests"
	@echo "  make test-warming  - Run warming tests"
	@echo "  make test-tiering  - Run tiering tests"
	@echo "  make test-coherence - Run coherence tests"
	@echo "  make test-prop     - Run property-based tests"
	@echo "  make bench         - Run all benchmarks"
	@echo "  make bench-predictive - Run predictive benchmarks"
	@echo "  make docs          - Generate and open documentation"
	@echo "  make examples      - Build examples"
	@echo "  make fmt           - Format code"
	@echo "  make clippy        - Run linter"
	@echo "  make check         - Run all checks"
	@echo "  make coverage      - Generate code coverage"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make build-release - Build release version"
	@echo "  make flamegraph    - Generate flamegraph"
