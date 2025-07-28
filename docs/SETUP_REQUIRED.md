# Manual Setup Requirements

The following items require manual configuration by repository administrators.

## GitHub Repository Settings

### Branch Protection Rules
1. Go to Settings → Branches
2. Add rule for `main` branch:
   - Require pull request reviews (1+ reviewers)
   - Require status checks to pass before merging
   - Require branches to be up to date before merging
   - Restrict pushes to main branch

### Repository Topics
Add relevant topics in Settings → General:
- Language/framework tags
- Project category tags
- Organization tags

### Security Settings
1. Enable Dependency Alerts
2. Enable Dependabot security updates
3. Enable Secret scanning alerts

## GitHub Actions Workflows

Create in `.github/workflows/`:
- `ci.yml` - Continuous integration
- `quality.yml` - Code quality checks
- `docs.yml` - Documentation deployment

See [workflows documentation](workflows/README.md) for detailed requirements.

## External Integrations

Consider setting up:
- Code coverage reporting (Codecov)
- Security scanning (Snyk, CodeQL)
- Documentation hosting (GitHub Pages)