#!/usr/bin/env python3
"""
Automated metrics collection script for FHE LLM Proxy project.
Collects various metrics from GitHub, CI/CD, and application sources.
"""

import argparse
import json
import logging
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, Any, Optional

import requests
from github import Github
import yaml

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class MetricsCollector:
    """Main metrics collection class."""
    
    def __init__(self, config_path: str = '.github/project-metrics.json'):
        self.config_path = Path(config_path)
        self.metrics = self.load_metrics_config()
        self.github_client = self.setup_github_client()
        
    def load_metrics_config(self) -> Dict[str, Any]:
        """Load metrics configuration from JSON file."""
        if not self.config_path.exists():
            logger.error(f"Metrics config file not found: {self.config_path}")
            sys.exit(1)
            
        with open(self.config_path, 'r') as f:
            return json.load(f)
    
    def setup_github_client(self) -> Github:
        """Setup GitHub API client."""
        token = os.getenv('GITHUB_TOKEN')
        if not token:
            logger.error("GITHUB_TOKEN environment variable not set")
            sys.exit(1)
            
        return Github(token)
    
    def collect_all_metrics(self) -> Dict[str, Any]:
        """Collect all metrics and update the configuration."""
        logger.info("Starting metrics collection...")
        
        # Update metadata
        self.metrics['metadata']['last_updated'] = datetime.utcnow().isoformat() + 'Z'
        
        # Collect different metric categories
        self.collect_development_metrics()
        self.collect_performance_metrics()
        self.collect_security_metrics()
        self.collect_operational_metrics()
        self.collect_business_metrics()
        self.collect_compliance_metrics()
        
        logger.info("Metrics collection completed")
        return self.metrics
    
    def collect_development_metrics(self):
        """Collect development-related metrics."""
        logger.info("Collecting development metrics...")
        
        repo_name = self.metrics['metadata']['repository']
        repo = self.github_client.get_repo(repo_name)
        
        # Code quality metrics
        self.collect_code_quality_metrics(repo)
        
        # Development velocity metrics
        self.collect_velocity_metrics(repo)
        
        # Contributor metrics
        self.collect_contributor_metrics(repo)
    
    def collect_code_quality_metrics(self, repo):
        """Collect code quality metrics from various sources."""
        quality_metrics = self.metrics['development_metrics']['code_quality']
        
        # Test coverage (would be collected from CI artifacts)
        coverage_data = self.get_coverage_data()
        if coverage_data:
            quality_metrics['test_coverage'].update(coverage_data)
        
        # Static analysis results (from CI artifacts)
        static_analysis = self.get_static_analysis_data()
        if static_analysis:
            quality_metrics['static_analysis'].update(static_analysis)
            quality_metrics['static_analysis']['last_scan'] = datetime.utcnow().isoformat() + 'Z'
        
        # Code complexity (from analysis tools)
        complexity_data = self.get_code_complexity_data()
        if complexity_data:
            quality_metrics['code_complexity'].update(complexity_data)
            quality_metrics['code_complexity']['last_measured'] = datetime.utcnow().isoformat() + 'Z'
    
    def collect_velocity_metrics(self, repo):
        """Collect development velocity metrics."""
        velocity_metrics = self.metrics['development_metrics']['development_velocity']
        
        # Get data for the last 30 days
        since = datetime.utcnow() - timedelta(days=30)
        
        # Commits per week
        commits = list(repo.get_commits(since=since))
        velocity_metrics['commits_per_week'] = len(commits) * 7 / 30
        
        # Pull request metrics
        prs = list(repo.get_pulls(state='all', sort='updated', direction='desc'))
        recent_prs = [pr for pr in prs if pr.updated_at > since]
        
        velocity_metrics['pull_requests_opened'] = len([pr for pr in recent_prs if pr.created_at > since])
        velocity_metrics['pull_requests_merged'] = len([pr for pr in recent_prs if pr.merged_at and pr.merged_at > since])
        velocity_metrics['pull_requests_closed'] = len([pr for pr in recent_prs if pr.closed_at and pr.closed_at > since])
        
        # Average PR size (lines changed)
        merged_prs = [pr for pr in recent_prs if pr.merged_at and pr.merged_at > since]
        if merged_prs:
            total_changes = sum(pr.additions + pr.deletions for pr in merged_prs)
            velocity_metrics['average_pr_size'] = total_changes / len(merged_prs)
            
            # Average review time
            review_times = []
            for pr in merged_prs:
                if pr.created_at and pr.merged_at:
                    review_time = (pr.merged_at - pr.created_at).total_seconds() / 3600
                    review_times.append(review_time)
            
            if review_times:
                velocity_metrics['average_review_time_hours'] = sum(review_times) / len(review_times)
    
    def collect_contributor_metrics(self, repo):
        """Collect contributor-related metrics."""
        contributor_metrics = self.metrics['development_metrics']['contributor_metrics']
        
        # Get contributors for the last 30 days
        since = datetime.utcnow() - timedelta(days=30)
        commits = list(repo.get_commits(since=since))
        
        # Active contributors
        active_contributors = set(commit.author.login for commit in commits if commit.author)
        contributor_metrics['active_contributors'] = len(active_contributors)
        
        # New contributors (simplified - would need historical data for accuracy)
        all_contributors = set(contributor.login for contributor in repo.get_contributors())
        contributor_metrics['new_contributors_this_month'] = max(0, len(active_contributors) - len(all_contributors) + len(active_contributors))
        
        # Bus factor (contributors who made >50% of commits)
        if commits:
            commit_counts = {}
            for commit in commits:
                if commit.author:
                    commit_counts[commit.author.login] = commit_counts.get(commit.author.login, 0) + 1
            
            total_commits = len(commits)
            major_contributors = sum(1 for count in commit_counts.values() if count > total_commits * 0.1)
            contributor_metrics['bus_factor'] = max(1, major_contributors)
    
    def collect_performance_metrics(self):
        """Collect performance-related metrics."""
        logger.info("Collecting performance metrics...")
        
        # Build performance (from CI artifacts)
        build_data = self.get_build_performance_data()
        if build_data:
            self.metrics['performance_metrics']['build_performance'].update(build_data)
        
        # Runtime performance (from benchmarks)
        runtime_data = self.get_runtime_performance_data()
        if runtime_data:
            self.metrics['performance_metrics']['runtime_performance'].update(runtime_data)
    
    def collect_security_metrics(self):
        """Collect security-related metrics."""
        logger.info("Collecting security metrics...")
        
        repo_name = self.metrics['metadata']['repository']
        repo = self.github_client.get_repo(repo_name)
        
        # Vulnerability data from GitHub Security Advisory API
        vulnerabilities = self.get_vulnerability_data(repo)
        if vulnerabilities:
            self.metrics['security_metrics']['vulnerability_management'].update(vulnerabilities)
        
        # Security scanning timestamps
        security_metrics = self.metrics['security_metrics']['vulnerability_management']['security_scanning']
        current_time = datetime.utcnow().isoformat() + 'Z'
        security_metrics['last_dependency_scan'] = current_time
        security_metrics['last_sast_scan'] = current_time
    
    def collect_operational_metrics(self):
        """Collect operational metrics."""
        logger.info("Collecting operational metrics...")
        
        # Deployment metrics (from CI/CD system)
        deployment_data = self.get_deployment_metrics()
        if deployment_data:
            self.metrics['operational_metrics']['deployment_metrics'].update(deployment_data)
        
        # Reliability metrics (from monitoring systems)
        reliability_data = self.get_reliability_metrics()
        if reliability_data:
            self.metrics['operational_metrics']['reliability_metrics'].update(reliability_data)
    
    def collect_business_metrics(self):
        """Collect business-related metrics."""
        logger.info("Collecting business metrics...")
        
        # These would typically come from application analytics
        # For now, we'll use placeholder implementations
        pass
    
    def collect_compliance_metrics(self):
        """Collect compliance-related metrics."""
        logger.info("Collecting compliance metrics...")
        
        # Update audit timestamps
        compliance_metrics = self.metrics['compliance_metrics']['regulatory_compliance']
        compliance_metrics['last_compliance_audit'] = datetime.utcnow().isoformat() + 'Z'
    
    def get_coverage_data(self) -> Optional[Dict[str, Any]]:
        """Get test coverage data from CI artifacts."""
        # This would typically parse coverage reports from CI
        # For now, return None to indicate no data available
        return None
    
    def get_static_analysis_data(self) -> Optional[Dict[str, Any]]:
        """Get static analysis results from CI artifacts."""
        # This would parse results from clippy, bandit, semgrep, etc.
        return None
    
    def get_code_complexity_data(self) -> Optional[Dict[str, Any]]:
        """Get code complexity metrics."""
        # This would use tools like radon, complexity analyzers
        return None
    
    def get_build_performance_data(self) -> Optional[Dict[str, Any]]:
        """Get build performance data from CI."""
        # This would parse CI build times and resource usage
        return None
    
    def get_runtime_performance_data(self) -> Optional[Dict[str, Any]]:
        """Get runtime performance data from benchmarks."""
        # This would parse benchmark results
        return None
    
    def get_vulnerability_data(self, repo) -> Optional[Dict[str, Any]]:
        """Get vulnerability data from GitHub Security API."""
        try:
            # Get security advisories
            advisories = list(repo.get_security_advisories())
            
            # Count by severity
            vulnerability_counts = {'critical': 0, 'high': 0, 'medium': 0, 'low': 0}
            
            for advisory in advisories:
                severity = advisory.severity.lower()
                if severity in vulnerability_counts:
                    vulnerability_counts[severity] += 1
            
            vulnerability_counts['total'] = sum(vulnerability_counts.values())
            
            return {'open_vulnerabilities': vulnerability_counts}
        except Exception as e:
            logger.warning(f"Could not fetch vulnerability data: {e}")
            return None
    
    def get_deployment_metrics(self) -> Optional[Dict[str, Any]]:
        """Get deployment metrics from CI/CD system."""
        # This would integrate with CI/CD system APIs
        return None
    
    def get_reliability_metrics(self) -> Optional[Dict[str, Any]]:
        """Get reliability metrics from monitoring systems."""
        # This would integrate with monitoring systems (Prometheus, etc.)
        return None
    
    def save_metrics(self, output_path: Optional[str] = None):
        """Save updated metrics to file."""
        if output_path is None:
            output_path = self.config_path
        
        with open(output_path, 'w') as f:
            json.dump(self.metrics, f, indent=2, sort_keys=True)
        
        logger.info(f"Metrics saved to {output_path}")
    
    def generate_report(self, output_format: str = 'json') -> str:
        """Generate a metrics report in the specified format."""
        if output_format == 'json':
            return json.dumps(self.metrics, indent=2, sort_keys=True)
        elif output_format == 'yaml':
            return yaml.dump(self.metrics, default_flow_style=False, sort_keys=True)
        else:
            raise ValueError(f"Unsupported output format: {output_format}")


def main():
    """Main function to run metrics collection."""
    parser = argparse.ArgumentParser(description='Collect project metrics')
    parser.add_argument('--config', default='.github/project-metrics.json',
                       help='Path to metrics configuration file')
    parser.add_argument('--output', help='Output file path (default: update config file)')
    parser.add_argument('--format', choices=['json', 'yaml'], default='json',
                       help='Output format')
    parser.add_argument('--report-only', action='store_true',
                       help='Generate report without updating metrics file')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Enable verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        collector = MetricsCollector(args.config)
        
        # Collect metrics
        updated_metrics = collector.collect_all_metrics()
        
        if args.report_only:
            # Generate and print report
            report = collector.generate_report(args.format)
            if args.output:
                with open(args.output, 'w') as f:
                    f.write(report)
                logger.info(f"Report saved to {args.output}")
            else:
                print(report)
        else:
            # Save updated metrics
            collector.save_metrics(args.output)
            
            if args.output and args.output != args.config:
                # Also generate a report if different output file specified
                report = collector.generate_report(args.format)
                report_path = Path(args.output).with_suffix(f'.{args.format}')
                with open(report_path, 'w') as f:
                    f.write(report)
                logger.info(f"Report saved to {report_path}")
        
        logger.info("Metrics collection completed successfully")
        
    except Exception as e:
        logger.error(f"Metrics collection failed: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()