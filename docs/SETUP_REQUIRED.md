# Manual Setup Required

Due to GitHub App permission limitations, the following setup steps must be performed manually by repository maintainers.

## Required GitHub Actions Workflows

### 1. Create Workflow Files

Copy the following workflow files from `docs/workflows/examples/` to `.github/workflows/`:

```bash
mkdir -p .github/workflows
cp docs/workflows/examples/ci.yml .github/workflows/
cp docs/workflows/examples/security-scan.yml .github/workflows/
cp docs/workflows/examples/release.yml .github/workflows/
```

### 2. Repository Settings

#### Branch Protection Rules
Navigate to Settings → Branches and configure:

**Main Branch Protection:**
- Require pull request reviews before merging (2 reviewers)
- Require status checks to pass before merging
- Required status checks:
  - `Code Quality`
  - `Security Scan`
  - `Test Suite`
  - `Build and Container`
- Require branches to be up to date before merging
- Require conversation resolution before merging
- Restrict pushes that create files larger than 100MB

**Develop Branch Protection:**
- Require pull request reviews before merging (1 reviewer)
- Require status checks to pass before merging
- Same status checks as main branch

#### Repository Secrets

Configure the following secrets in Settings → Secrets and variables → Actions:

```yaml
# Container Registry
DOCKERHUB_USERNAME: "your-dockerhub-username"
DOCKERHUB_TOKEN: "your-dockerhub-access-token"

# Kubernetes Deployment
KUBECONFIG_STAGING: "base64-encoded-kubeconfig-for-staging"
KUBECONFIG_PRODUCTION: "base64-encoded-kubeconfig-for-production"

# Python Package Publishing
PYPI_TOKEN: "your-pypi-api-token"

# Notifications
SLACK_WEBHOOK: "your-slack-webhook-url"

# Security Scanning (if using external services)
SNYK_TOKEN: "your-snyk-token"
SONAR_TOKEN: "your-sonarcloud-token"
```

#### Environment Configuration

Create environments in Settings → Environments:

**Staging Environment:**
- Deployment protection rules: None
- Environment secrets:
  - `KUBECONFIG_STAGING`
  - `API_BASE_URL: "https://api-staging.company.com"`

**Production Environment:**
- Deployment protection rules:
  - Required reviewers: 2 from engineering team
  - Wait timer: 5 minutes
- Environment secrets:
  - `KUBECONFIG_PRODUCTION`
  - `API_BASE_URL: "https://api.company.com"`

### 3. Repository Variables

Configure repository variables in Settings → Secrets and variables → Actions:

```yaml
# Application Configuration
APP_NAME: "fhe-llm-proxy"
CONTAINER_REGISTRY: "ghcr.io"
PYTHON_PACKAGE_NAME: "fhe-llm-proxy-client"

# Testing Configuration
TEST_TIMEOUT: "3600"  # 1 hour
BENCHMARK_THRESHOLD: "1.1"  # 10% performance regression threshold

# Security Configuration
SECURITY_SCAN_SCHEDULE: "0 2 * * *"  # Daily at 2 AM UTC
VULNERABILITY_THRESHOLD: "high"  # Fail on high/critical vulnerabilities
```

## Required GitHub App Permissions

To enable full CI/CD functionality, the GitHub App needs these permissions:

### Repository Permissions
- **Actions**: Write (to manage workflow runs)
- **Contents**: Write (to create releases and tags)
- **Metadata**: Read (to access repository metadata)
- **Pull requests**: Write (to create and manage PRs)
- **Issues**: Write (to create issues for planning)
- **Deployments**: Write (to manage deployments)
- **Environments**: Write (to manage deployment environments)
- **Pages**: Write (to deploy documentation)
- **Security events**: Write (to create security advisories)

### Organization Permissions
- **Members**: Read (to assign reviewers)
- **Team membership**: Read (to manage team-based reviews)

## Additional Setup Steps

### 1. Code Quality Tools Configuration

#### Pre-commit Hooks
```bash
# Install pre-commit hooks
pre-commit install
pre-commit install --hook-type commit-msg
```

#### IDE Integration
- Install recommended VS Code extensions from `.vscode/extensions.json`
- Configure Rust-analyzer settings from `.vscode/settings.json`

### 2. Security Tools Setup

#### Secret Scanning Baseline
```bash
# Generate baseline for detect-secrets
detect-secrets scan --all-files --baseline .secrets.baseline
```

#### Dependency Scanning
```bash
# Initialize cargo-deny configuration
cargo deny init
```

### 3. Documentation Setup

#### GitHub Pages
1. Go to Settings → Pages
2. Set source to "GitHub Actions"
3. Configure custom domain if needed

#### Documentation Automation
```bash
# Setup documentation build
cd docs
pip install -r requirements.txt
make html
```

### 4. Monitoring Integration

#### Observability Stack
```bash
# Deploy monitoring stack to Kubernetes
kubectl apply -f k8s-manifests/monitoring.yaml
```

#### Alert Configuration
```bash
# Configure AlertManager
kubectl create secret generic alertmanager-config \
  --from-file=alertmanager.yml=monitoring/alertmanager.yml
```

## Verification Steps

After completing the setup, verify everything works:

### 1. Test CI/CD Pipeline
```bash
# Create a test branch and PR
git checkout -b test-ci-setup
echo "# Test CI" >> TEST_CI.md
git add TEST_CI.md
git commit -m "test: verify CI/CD pipeline setup"
git push -u origin test-ci-setup
# Create PR through GitHub UI
```

### 2. Verify Security Scanning
- Check that security scans run on schedule
- Verify vulnerability alerts are created
- Test secret detection on commit

### 3. Test Release Process
```bash
# Test release workflow (manual trigger)
# Go to Actions → Release → Run workflow
# Specify version like "v0.1.0-test"
```

### 4. Validate Deployment
- Check staging deployment works
- Verify production deployment requires approval
- Test rollback procedures

## Troubleshooting

### Common Issues

#### Workflow Permissions
If workflows fail with permission errors:
1. Check GitHub App permissions
2. Verify repository secrets are configured
3. Ensure environment protection rules allow deployment

#### Container Registry Access
If container builds fail:
1. Verify `DOCKERHUB_TOKEN` is valid
2. Check container registry permissions
3. Ensure `GITHUB_TOKEN` has package write permissions

#### Deployment Failures
If Kubernetes deployments fail:
1. Verify `KUBECONFIG_*` secrets are base64 encoded correctly
2. Check cluster connectivity and permissions
3. Ensure deployment manifests are valid

### Support Contacts

- **Infrastructure Team**: infra@company.com
- **Security Team**: security@company.com
- **DevOps Support**: devops@company.com

## Maintenance

### Regular Tasks
- **Weekly**: Review failed workflows and security alerts
- **Monthly**: Update dependency scanning baselines
- **Quarterly**: Review and update branch protection rules
- **Annually**: Audit GitHub App permissions and secrets

### Updates
- Keep workflow templates updated with latest GitHub Actions versions
- Update security scanning tools and baselines regularly
- Review and refresh secrets and tokens before expiration