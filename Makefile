# Makefile for FHE LLM Proxy
# Alternative to justfile for environments where just is not available

.PHONY: help build test clean docker deploy docs

# Default target
help:
	@echo "FHE LLM Proxy Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build release binary"
	@echo "  build-debug    - Build debug binary"
	@echo "  test           - Run test suite"
	@echo "  test-gpu       - Run GPU-specific tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run linters"
	@echo "  docker         - Build Docker container"
	@echo "  docker-test    - Build test container"
	@echo "  deploy         - Deploy with docker-compose"
	@echo "  docs           - Generate documentation"
	@echo "  security       - Run security checks"
	@echo "  bench          - Run benchmarks"
	@echo "  coverage       - Generate coverage report"
	@echo ""

# Build targets
build:
	cargo build --release --features gpu

build-debug:
	cargo build --features gpu

build-all: build build-python

build-python:
	cd python && python -m pip install -e .

# Test targets
test:
	cargo nextest run --all-features --profile ci

test-gpu:
	cargo nextest run --profile gpu

test-python:
	cd python && python -m pytest tests/ -v --cov=fhe_llm_proxy

test-all: test test-python

test-integration:
	docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit

# Code quality targets
fmt:
	cargo fmt --all
	cd python && python -m black .

lint:
	cargo clippy --all-targets --all-features -- -D warnings
	cd python && python -m ruff check .

audit:
	cargo audit --config .cargo/audit.toml
	cargo deny check

security: audit
	cargo audit
	cd python && python -m bandit -r . -c .bandit

# Docker targets
docker:
	docker build -t fhe-llm-proxy:latest .

docker-test:
	docker build -f Dockerfile.test -t fhe-llm-proxy:test .

docker-multi:
	docker buildx build --platform linux/amd64,linux/arm64 -t fhe-llm-proxy:multi .

# Deployment targets
deploy:
	docker-compose up --build

deploy-k8s:
	kubectl apply -f k8s-manifests/

deploy-monitoring:
	kubectl apply -f k8s-manifests/monitoring.yaml

# Documentation targets
docs:
	cargo doc --all-features --no-deps --open

docs-python:
	cd python && python -m sphinx-build -b html docs docs/_build

# Performance targets
bench:
	cargo bench --all-features

bench-quick:
	cargo bench --bench fhe_operations -- --quick

perf:
	k6 run load-testing/k6-config.js

# Coverage targets
coverage:
	cargo tarpaulin --out html --output-dir target/coverage

coverage-python:
	cd python && python -m pytest --cov=fhe_llm_proxy --cov-report=html --cov-report=term

mutation-test:
	cargo mutants --config mutation-testing.toml

# Development targets
dev:
	cargo run --features gpu -- --config config/development.toml

dev-setup:
	rustup component add rustfmt clippy
	cargo install cargo-nextest cargo-audit cargo-deny cargo-tarpaulin cargo-flamegraph cargo-mutants
	cd python && python -m pip install -e ".[dev]"

# Utility targets
clean:
	cargo clean
	cd python && rm -rf build/ dist/ *.egg-info/
	docker system prune -f

clean-all: clean
	rm -rf target/
	rm -rf python/build/ python/dist/ python/*.egg-info/
	docker system prune -af

install:
	cargo install --path . --features gpu

uninstall:
	cargo uninstall homomorphic-llm-proxy

# Release targets
release-prep:
	./scripts/release-prep.sh $(VERSION)

release-build:
	cargo build --release --all-features
	cd python && python -m build

release-publish:
	cargo publish
	cd python && python -m twine upload dist/*

# CI targets
ci: fmt lint test audit build

ci-full: ci coverage bench security

# Monitoring targets
logs:
	docker logs -f fhe-llm-proxy

metrics:
	curl -s http://localhost:9090/metrics | grep fhe_

health:
	curl -s http://localhost:8080/health | jq .

# Debug targets
debug-gpu:
	nvidia-debugdump --list
	nvidia-smi -q

debug-memory:
	valgrind --tool=memcheck --leak-check=full target/debug/homomorphic-llm-proxy

debug-performance:
	perf stat -e cycles,instructions,cache-references,cache-misses target/release/homomorphic-llm-proxy

# Configuration targets
validate-config:
	./scripts/validate-config.sh

validate-k8s:
	kubectl apply --dry-run=client -f k8s-manifests/

# Environment info
info:
	@echo "=== System Information ==="
	@echo "OS: $$(uname -a)"
	@echo "CPU: $$(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
	@echo "Memory: $$(free -h | grep Mem | awk '{print $$2}')"
	@echo "Rust: $$(rustc --version)"
	@echo "Cargo: $$(cargo --version)"
	@echo "Docker: $$(docker --version)"
	@if command -v nvidia-smi >/dev/null 2>&1; then echo "GPU: $$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)"; fi