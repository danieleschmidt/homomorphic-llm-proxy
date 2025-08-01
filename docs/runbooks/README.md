# Runbooks

This directory contains operational runbooks for common scenarios and troubleshooting procedures.

## Available Runbooks

### Incident Response
- [High Latency Incidents](incident-response/high-latency.md)
- [GPU Memory Issues](incident-response/gpu-memory.md)
- [Privacy Budget Exhaustion](incident-response/privacy-budget.md)
- [Key Rotation Failures](incident-response/key-rotation.md)

### Maintenance Procedures
- [Planned Maintenance](maintenance/planned-maintenance.md)
- [Scaling Operations](maintenance/scaling.md)
- [Backup and Recovery](maintenance/backup-recovery.md)
- [Configuration Updates](maintenance/config-updates.md)

### Monitoring and Alerting
- [Alert Response Procedures](monitoring/alert-response.md)
- [Dashboard Maintenance](monitoring/dashboard-maintenance.md)
- [Metrics Troubleshooting](monitoring/metrics-troubleshooting.md)

### Security Procedures
- [Security Incident Response](security/incident-response.md)
- [Key Management Procedures](security/key-management.md)
- [Access Control Updates](security/access-control.md)
- [Compliance Auditing](security/compliance-auditing.md)

## Using Runbooks

### Runbook Format
Each runbook follows a standard format:
- **Overview**: Problem description and impact
- **Detection**: How to identify the issue
- **Investigation**: Steps to diagnose root cause
- **Resolution**: Step-by-step fix procedures
- **Prevention**: Long-term prevention measures
- **References**: Related documentation and tools

### Severity Levels
- **SEV-1**: Critical - Service down or major security incident
- **SEV-2**: High - Significant performance degradation
- **SEV-3**: Medium - Minor issues or planned maintenance
- **SEV-4**: Low - Informational or future improvements

### Escalation Matrix
```
SEV-1: Immediate escalation to on-call engineer
SEV-2: Escalate within 15 minutes if not resolved
SEV-3: Handle during business hours
SEV-4: Address in next sprint planning
```

## Emergency Contacts

- **On-Call Engineer**: Use PagerDuty or Slack @oncall
- **Security Team**: security@company.com
- **Infrastructure Team**: infra@company.com
- **Management Escalation**: engineering-leads@company.com

## Quick Reference

### Common Commands
```bash
# Check service status
kubectl get pods -n fhe-proxy

# View recent logs
kubectl logs -f deployment/fhe-llm-proxy -n fhe-proxy --tail=100

# Check GPU status
nvidia-smi

# View metrics
curl http://localhost:9090/metrics | grep fhe_

# Health check
curl http://localhost:8080/health
```

### Important URLs
- **Grafana Dashboard**: http://grafana.company.com/d/fhe-proxy
- **Prometheus**: http://prometheus.company.com
- **AlertManager**: http://alertmanager.company.com
- **Documentation**: https://docs.company.com/fhe-proxy

## Runbook Maintenance

### Update Process
1. Create branch for runbook updates
2. Test procedures in staging environment
3. Review with team members
4. Update version and last-tested date
5. Merge to main branch

### Review Schedule
- **Monthly**: Review and update all runbooks
- **Quarterly**: Test emergency procedures
- **Annually**: Complete runbook audit and refresh

### Contributing
See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on adding new runbooks or updating existing ones.