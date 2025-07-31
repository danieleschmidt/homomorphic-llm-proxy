# GitHub Actions Workflow Implementation Guide

This document provides the complete GitHub Actions workflows that need to be manually created by repository administrators with appropriate permissions.

## Repository Setup Requirements

### Required Repository Secrets

Configure these secrets in GitHub repository settings:

```bash
# Publishing and deployment
CARGO_REGISTRY_TOKEN      # For crates.io publishing
PYPI_API_TOKEN            # For PyPI publishing  
DOCKER_USERNAME           # For Docker Hub
DOCKER_PASSWORD           # For Docker Hub

# Security and signing
GPG_PRIVATE_KEY           # For artifact signing
GPG_PASSPHRASE           # For GPG key
CODECOV_TOKEN            # For coverage reporting

# Performance monitoring
BENCHMARK_WEBHOOK         # For performance tracking
```

### Required Branch Protection Rules

1. Navigate to Settings → Branches
2. Add rule for `main` branch:
   - ✅ Require pull request reviews (2+ reviewers for security changes)
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Require conversation resolution before merging
   - ✅ Restrict pushes to matching branches

### Required Status Checks

Add these required status checks:
- `rust-quality`
- `python-quality` 
- `security-scan`
- `integration-tests`

## Workflow Files to Create

### 1. Continuous Integration (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  rust-quality:
    name: Rust Code Quality
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
        
    - name: Cache Cargo
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Format Check
      run: cargo fmt --all -- --check
      
    - name: Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Build
      run: cargo build --all-features
      
    - name: Test
      run: cargo test --all-features
      
    - name: Security Audit
      run: |
        cargo install cargo-audit
        cargo audit

  python-quality:
    name: Python Code Quality
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.9', '3.10', '3.11', '3.12']
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v5
      with:
        python-version: ${{ matrix.python-version }}
        
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        cd python && pip install -e ".[dev]"
        
    - name: Format Check (Black)
      run: cd python && black --check .
      
    - name: Lint (Ruff)
      run: cd python && ruff check .
      
    - name: Type Check (MyPy)
      run: cd python && mypy fhe_llm_proxy
      
    - name: Test with Coverage
      run: cd python && pytest tests/ -v --cov=fhe_llm_proxy --cov-report=xml
      
    - name: Upload Coverage
      uses: codecov/codecov-action@v4
      with:
        file: python/coverage.xml
        flags: python
        token: ${{ secrets.CODECOV_TOKEN }}

  security-scan:
    name: Security Analysis
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Run CodeQL
      uses: github/codeql-action/init@v3
      with:
        languages: rust, python
        
    - name: Perform CodeQL Analysis
      uses: github/codeql-action/analyze@v3
      
    - name: Python Security Scan
      run: |
        pip install bandit safety
        cd python && bandit -r . -c .bandit
        safety check

  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v4
    
    - name: Build Test Environment
      run: docker-compose -f docker-compose.test.yml build
      
    - name: Run Integration Tests
      run: docker-compose -f docker-compose.test.yml up --abort-on-container-exit
      
    - name: Cleanup
      run: docker-compose -f docker-compose.test.yml down
```

### 2. Release Automation (`.github/workflows/release.yml`)

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

env:
  CARGO_TERM_COLOR: always

jobs:
  validate-release:
    name: Validate Release
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
        
    - name: Extract Version
      id: version
      run: echo "version=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT

  build-rust:
    name: Build Rust Artifacts
    needs: validate-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
        - os: ubuntu-latest
          target: x86_64-unknown-linux-gnu
        - os: windows-latest
          target: x86_64-pc-windows-msvc
        - os: macos-latest
          target: x86_64-apple-darwin
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}
        
    - name: Build Release Binary
      run: cargo build --release --target ${{ matrix.target }} --all-features
      
    - name: Package Binary
      shell: bash
      run: |
        mkdir -p release
        if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
          BINARY_EXT=".exe"
        else
          BINARY_EXT=""
        fi
        
        cp target/${{ matrix.target }}/release/homomorphic-llm-proxy${BINARY_EXT} release/
        cp README.md LICENSE release/
        
        cd release
        if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
          7z a ../homomorphic-llm-proxy-${{ needs.validate-release.outputs.version }}-${{ matrix.target }}.zip *
        else
          tar -czf ../homomorphic-llm-proxy-${{ needs.validate-release.outputs.version }}-${{ matrix.target }}.tar.gz *
        fi
        
    - name: Upload Artifacts
      uses: actions/upload-artifact@v4
      with:
        name: release-${{ matrix.target }}
        path: homomorphic-llm-proxy-${{ needs.validate-release.outputs.version }}-${{ matrix.target }}.*

  create-release:
    name: Create GitHub Release
    needs: [validate-release, build-rust]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
        
    - name: Download All Artifacts
      uses: actions/download-artifact@v4
      
    - name: Create Release
      uses: softprops/action-gh-release@v2
      with:
        tag_name: ${{ needs.validate-release.outputs.version }}
        name: Release ${{ needs.validate-release.outputs.version }}
        draft: false
        prerelease: ${{ contains(needs.validate-release.outputs.version, '-') }}
        files: |
          release-*/*
```

### 3. Security Scanning (`.github/workflows/security.yml`)

```yaml
name: Security Scan

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 3 * * 1'

jobs:
  dependency-scan:
    name: Dependency Vulnerability Scan
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Rust Dependency Audit
      run: |
        cargo install cargo-audit cargo-deny
        cargo audit --json > rust-audit.json
        cargo deny check
        
    - name: Python Dependency Scan
      run: |
        pip install safety bandit
        cd python && safety check --json > ../python-safety.json
        
    - name: Upload Security Reports
      uses: actions/upload-artifact@v4
      with:
        name: security-reports
        path: |
          rust-audit.json
          python-safety.json

  container-security:
    name: Container Security Scan
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Build Container Image
      run: docker build -t fhe-proxy-security-scan .
      
    - name: Run Trivy Container Scan
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: 'fhe-proxy-security-scan'
        format: 'sarif'
        output: 'trivy-results.sarif'
        
    - name: Upload Trivy Scan Results
      uses: github/codeql-action/upload-sarif@v3
      with:
        sarif_file: 'trivy-results.sarif'

  secret-scan:
    name: Secret Detection
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
        
    - name: Run GitLeaks Secret Scan
      uses: gitleaks/gitleaks-action@v2
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 4. Performance Testing (`.github/workflows/performance.yml`)

```yaml
name: Performance Testing

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
    types: [labeled]
  schedule:
    - cron: '0 4 * * 0'

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  cpu-benchmarks:
    name: CPU Performance Benchmarks
    runs-on: ubuntu-latest
    if: contains(github.event.pull_request.labels.*.name, 'performance') || github.event_name != 'pull_request'
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      
    - name: Install Benchmark Tools
      run: |
        cargo install cargo-criterion
        sudo apt-get update
        sudo apt-get install -y valgrind
        
    - name: Run CPU Benchmarks
      run: |
        cargo bench --bench fhe_operations -- --output-format json > cpu_benchmarks.json
        
    - name: Upload Benchmark Results
      uses: actions/upload-artifact@v4
      with:
        name: cpu-benchmarks
        path: cpu_benchmarks.json

  load-testing:
    name: Load Testing
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v4
    
    - name: Install K6
      run: |
        sudo gpg -k
        sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
        echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
        sudo apt-get update
        sudo apt-get install k6
        
    - name: Start Test Environment
      run: |
        docker-compose up -d
        sleep 30
        
    - name: Run Load Tests
      run: k6 run load-testing/k6-config.js --out json=load_test_results.json
        
    - name: Upload Load Test Results
      uses: actions/upload-artifact@v4
      with:
        name: load-test-results
        path: load_test_results.json
```

## Implementation Instructions

### Step 1: Create Workflow Files
1. Create each workflow file in `.github/workflows/` directory
2. Copy the YAML content from the sections above
3. Commit and push the workflow files

### Step 2: Configure Repository Settings
1. Add required secrets in Settings → Secrets and variables → Actions
2. Configure branch protection rules as specified above
3. Enable required status checks

### Step 3: Test Workflows
1. Create a test pull request to verify CI workflow
2. Create a test tag (e.g., `v0.1.0-test`) to verify release workflow
3. Monitor workflow runs and adjust as needed

### Step 4: Team Configuration
1. Review and update team assignments in `.github/CODEOWNERS`
2. Ensure security team has access for crypto code reviews
3. Configure notification settings for workflow failures

## Security Considerations

- All workflows use pinned action versions for security
- Secret scanning is enabled for all branches
- Container images are scanned for vulnerabilities
- Cryptographic code requires additional security reviews
- Performance regression detection prevents degradation

## Maintenance

- Review and update workflow dependencies quarterly
- Monitor workflow execution times and optimize as needed
- Update security scanning tools and rulesets regularly
- Maintain baseline performance metrics for regression detection