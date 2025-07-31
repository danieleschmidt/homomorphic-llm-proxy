# GitHub Actions Workflow Setup

This document provides the required GitHub Actions workflows for the FHE LLM Proxy project.

## Required Workflows

### 1. Continuous Integration (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run tests
      run: cargo test --verbose
    
    - name: Run integration tests
      run: cargo test --test integration
    
    - name: Check documentation
      run: cargo doc --no-deps --document-private-items

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Security audit
      uses: actions-rs/audit-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Run cargo deny
      uses: EmbarkStudios/cargo-deny-action@v1

  build:
    name: Build
    runs-on: ubuntu-latest
    needs: [test, security]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    
    - name: Build release
      run: cargo build --release --verbose
```

### 2. Security Scanning (`.github/workflows/security.yml`)

```yaml
name: Security

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday

jobs:
  dependency-scan:
    name: Dependency Vulnerability Scan
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        scan-type: 'fs'
        scan-ref: '.'
        format: 'sarif'
        output: 'trivy-results.sarif'
    
    - name: Upload Trivy scan results to GitHub Security tab
      uses: github/codeql-action/upload-sarif@v3
      with:
        sarif_file: 'trivy-results.sarif'

  codeql:
    name: CodeQL Analysis
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        language: [ 'rust' ]
    
    steps:
    - name: Checkout repository
      uses: actions/checkout@v4
    
    - name: Initialize CodeQL
      uses: github/codeql-action/init@v3
      with:
        languages: ${{ matrix.language }}
    
    - name: Autobuild
      uses: github/codeql-action/autobuild@v3
    
    - name: Perform CodeQL Analysis
      uses: github/codeql-action/analyze@v3
```

### 3. Release Management (`.github/workflows/release.yml`)

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Create Release
      id: create_release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ github.ref }}
        release_name: Release ${{ github.ref }}
        draft: false
        prerelease: false

  build-binaries:
    name: Build Release Binaries
    needs: create-release
    runs-on: ${{ matrix.os }}
    
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
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
    
    - name: Build release binary
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Package binary (Unix)
      if: matrix.os != 'windows-latest'
      run: |
        cd target/${{ matrix.target }}/release
        tar czf ../../../fhe-llm-proxy-${{ matrix.target }}.tar.gz fhe-llm-proxy
    
    - name: Package binary (Windows)
      if: matrix.os == 'windows-latest'
      run: |
        cd target/${{ matrix.target }}/release
        7z a ../../../fhe-llm-proxy-${{ matrix.target }}.zip fhe-llm-proxy.exe
    
    - name: Upload Release Asset (Unix)
      if: matrix.os != 'windows-latest'
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ needs.create-release.outputs.upload_url }}
        asset_path: ./fhe-llm-proxy-${{ matrix.target }}.tar.gz
        asset_name: fhe-llm-proxy-${{ matrix.target }}.tar.gz
        asset_content_type: application/gzip
    
    - name: Upload Release Asset (Windows)
      if: matrix.os == 'windows-latest'
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ needs.create-release.outputs.upload_url }}
        asset_path: ./fhe-llm-proxy-${{ matrix.target }}.zip
        asset_name: fhe-llm-proxy-${{ matrix.target }}.zip
        asset_content_type: application/zip
```

### 4. Docker Build (`.github/workflows/docker.yml`)

```yaml
name: Docker

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]
  pull_request:
    branches: [ main ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    
    permissions:
      contents: read
      packages: write
    
    steps:
    - name: Checkout repository
      uses: actions/checkout@v4
    
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=semver,pattern={{major}}.{{minor}}
    
    - name: Build and push Docker image
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./Dockerfile
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
```

## Setup Instructions

1. **Create workflow files**: Copy the above YAML content into the corresponding files in `.github/workflows/`
2. **Repository secrets**: Set up required secrets in GitHub repository settings:
   - `GITHUB_TOKEN` (automatically provided)
   - Add any additional API keys or secrets as needed
3. **Branch protection**: Configure branch protection rules for `main` branch requiring:
   - Status checks to pass
   - Pull request reviews
   - Up-to-date branches
4. **Security settings**: Enable:
   - Dependency graph
   - Dependabot alerts and security updates
   - Secret scanning
   - Code scanning with CodeQL

## Workflow Features

- **Automated testing** on every PR and push
- **Security scanning** for vulnerabilities and secrets
- **Automated releases** on version tags
- **Docker image building** and publishing
- **Cross-platform binary builds**
- **Code quality checks** (formatting, linting)
- **Dependency auditing**

These workflows provide comprehensive CI/CD coverage for the FHE LLM Proxy project.