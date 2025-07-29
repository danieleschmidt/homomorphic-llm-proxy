# Justfile for FHE LLM Proxy - Modern command runner
# See https://github.com/casey/just for installation and usage

# Default recipe to display available commands
default:
    @just --list

# Build commands
build:
    cargo build --release --features gpu

build-debug:
    cargo build --features gpu

build-docker:
    docker build -t fhe-llm-proxy:latest .

build-python:
    cd python && python -m pip install -e .

# Testing commands  
test:
    cargo nextest run --all-features --profile ci

test-gpu:
    cargo nextest run --profile gpu

test-python:
    cd python && python -m pytest tests/ -v --cov=fhe_llm_proxy

test-all: test test-python

# Code quality commands
fmt:
    cargo fmt --all
    cd python && python -m black .

lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cd python && python -m ruff check .

audit:
    cargo audit --config .cargo/audit.toml
    cargo deny check

# Security commands  
security-scan:
    cargo audit
    cd python && python -m bandit -r . -c .bandit

sbom:
    cargo sbom --config sbom.toml

vuln-check:
    cargo deny check advisories

# Performance commands
bench:
    cargo bench --all-features

bench-quick:
    cargo bench --bench fhe_operations -- --quick

flamegraph binary="fhe-llm-proxy":
    cargo flamegraph --bin {{binary}} -- --bench-mode

perf-test:
    k6 run load-testing/k6-config.js

# Coverage commands
coverage:
    cargo tarpaulin --out html --output-dir target/coverage

coverage-python:
    cd python && python -m pytest --cov=fhe_llm_proxy --cov-report=html --cov-report=term

mutation-test:
    cargo mutants --config mutation-testing.toml

# Development commands
dev:
    cargo run --features gpu -- --config config/development.toml

dev-docker:
    docker run --rm -it --gpus all -p 8080:8080 -p 9090:9090 fhe-llm-proxy:latest

dev-setup:
    rustup component add rustfmt clippy
    cargo install cargo-nextest cargo-audit cargo-deny cargo-tarpaulin cargo-flamegraph cargo-mutants
    cd python && python -m pip install -e ".[dev]"

# Monitoring commands
logs:
    docker logs -f fhe-llm-proxy

metrics:
    curl -s http://localhost:9090/metrics | grep fhe_

health:
    curl -s http://localhost:8081/health | jq .

# Database/Storage commands  
backup-keys:
    ./scripts/backup-keys.sh

restore-keys backup_id:
    ./scripts/restore-keys.sh {{backup_id}}

# Deployment commands
deploy-local:
    docker-compose up --build

deploy-k8s:
    kubectl apply -f k8s-manifests/

deploy-monitoring:
    kubectl apply -f k8s-manifests/monitoring.yaml

# Documentation commands
docs:
    cargo doc --all-features --no-deps --open

docs-python:
    cd python && python -m sphinx-build -b html docs docs/_build

serve-docs port="8000":
    cd target/doc && python -m http.server {{port}}

# Cleanup commands
clean:
    cargo clean
    cd python && rm -rf build/ dist/ *.egg-info/
    docker system prune -f

clean-all: clean
    rm -rf target/
    rm -rf python/build/ python/dist/ python/*.egg-info/
    docker system prune -af

# CI/CD commands
ci-check:
    just fmt
    just lint  
    just test
    just audit
    just build

ci-full: ci-check
    just coverage
    just bench-quick
    just security-scan

# Release commands
release-prep version:
    ./scripts/release-prep.sh {{version}}

release-build:
    cargo build --release --all-features
    cd python && python -m build

release-publish:
    cargo publish
    cd python && python -m twine upload dist/*

# Utility commands
deps:
    cargo tree --all-features

deps-outdated:
    cargo outdated

size:
    cargo bloat --release --crates

profile binary="fhe-llm-proxy":
    perf record --call-graph=dwarf target/release/{{binary}}
    perf report

gpu-info:
    nvidia-smi
    nvcc --version

system-info:
    @echo "=== System Information ==="
    @echo "OS: $(uname -a)"
    @echo "CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
    @echo "Memory: $(free -h | grep Mem | awk '{print $2}')"
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Python: $(python --version 2>&1)"

# Git hooks
install-hooks:
    pre-commit install
    pre-commit install --hook-type commit-msg

run-hooks:
    pre-commit run --all-files

# Environment setup
setup-dev: dev-setup install-hooks
    @echo "Development environment setup complete!"

setup-ci:
    rustup component add rustfmt clippy
    cargo install cargo-nextest cargo-audit cargo-deny
    @echo "CI environment setup complete!"

# Advanced testing
test-stress:
    cargo nextest run --profile stress --test-threads 1

test-security:
    cargo nextest run --profile security

test-integration:
    docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit

# Configuration validation
validate-config:
    ./scripts/validate-config.sh

validate-k8s:
    kubectl apply --dry-run=client -f k8s-manifests/

# Troubleshooting
debug-gpu:
    nvidia-debugdump --list
    nvidia-smi -q

debug-memory:
    valgrind --tool=memcheck --leak-check=full target/debug/fhe-llm-proxy

debug-performance:
    perf stat -e cycles,instructions,cache-references,cache-misses target/release/fhe-llm-proxy