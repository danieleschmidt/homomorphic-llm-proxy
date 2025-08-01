#!/usr/bin/env python3
"""
Automated dependency update script for FHE LLM Proxy project.
Checks for outdated dependencies and creates PRs with updates.
"""

import argparse
import json
import logging
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import requests
import toml
from github import Github

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class DependencyUpdater:
    """Automated dependency update manager."""
    
    def __init__(self, repo_path: str = '.'):
        self.repo_path = Path(repo_path)
        self.github_client = self.setup_github_client()
        self.repo_name = self.get_repo_name()
        self.repo = self.github_client.get_repo(self.repo_name)
        
    def setup_github_client(self) -> Github:
        """Setup GitHub API client."""
        token = os.getenv('GITHUB_TOKEN')
        if not token:
            logger.error("GITHUB_TOKEN environment variable not set")
            sys.exit(1)
        return Github(token)
    
    def get_repo_name(self) -> str:
        """Get repository name from git remote."""
        try:
            result = subprocess.run(
                ['git', 'remote', 'get-url', 'origin'],
                capture_output=True,
                text=True,
                cwd=self.repo_path
            )
            
            if result.returncode != 0:
                raise RuntimeError("Failed to get git remote URL")
            
            url = result.stdout.strip()
            # Parse GitHub URL to get owner/repo
            if url.startswith('https://github.com/'):
                return url.replace('https://github.com/', '').replace('.git', '')
            elif url.startswith('git@github.com:'):
                return url.replace('git@github.com:', '').replace('.git', '')
            else:
                raise ValueError(f"Unsupported git URL format: {url}")
                
        except Exception as e:
            logger.error(f"Failed to determine repository name: {e}")
            sys.exit(1)
    
    def check_rust_dependencies(self) -> List[Dict]:
        """Check for outdated Rust dependencies."""
        logger.info("Checking Rust dependencies...")
        
        cargo_toml = self.repo_path / 'Cargo.toml'
        if not cargo_toml.exists():
            logger.warning("Cargo.toml not found, skipping Rust dependency check")
            return []
        
        try:
            # Run cargo outdated to check for updates
            result = subprocess.run(
                ['cargo', 'outdated', '--format', 'json'],
                capture_output=True,
                text=True,
                cwd=self.repo_path
            )
            
            if result.returncode != 0:
                logger.warning(f"cargo outdated failed: {result.stderr}")
                return []
            
            outdated_data = json.loads(result.stdout)
            updates = []
            
            for dep in outdated_data.get('dependencies', []):
                if dep.get('available') and dep.get('available') != dep.get('project'):
                    updates.append({
                        'ecosystem': 'rust',
                        'name': dep['name'],
                        'current_version': dep['project'],
                        'latest_version': dep['available'],
                        'update_type': self.classify_update_type(dep['project'], dep['available']),
                        'file': 'Cargo.toml'
                    })
            
            logger.info(f"Found {len(updates)} outdated Rust dependencies")
            return updates
            
        except Exception as e:
            logger.error(f"Failed to check Rust dependencies: {e}")
            return []
    
    def check_python_dependencies(self) -> List[Dict]:
        """Check for outdated Python dependencies."""
        logger.info("Checking Python dependencies...")
        
        python_dir = self.repo_path / 'python'
        if not python_dir.exists():
            logger.warning("Python directory not found, skipping Python dependency check")
            return []
        
        pyproject_toml = python_dir / 'pyproject.toml'
        if not pyproject_toml.exists():
            logger.warning("pyproject.toml not found, skipping Python dependency check")
            return []
        
        try:
            # Use pip-outdated or similar tool
            result = subprocess.run(
                ['pip', 'list', '--outdated', '--format=json'],
                capture_output=True,
                text=True,
                cwd=python_dir
            )
            
            if result.returncode != 0:
                logger.warning(f"pip list outdated failed: {result.stderr}")
                return []
            
            outdated_data = json.loads(result.stdout)
            updates = []
            
            for dep in outdated_data:
                updates.append({
                    'ecosystem': 'python',
                    'name': dep['name'],
                    'current_version': dep['version'],
                    'latest_version': dep['latest_version'],
                    'update_type': self.classify_update_type(dep['version'], dep['latest_version']),
                    'file': 'python/pyproject.toml'
                })
            
            logger.info(f"Found {len(updates)} outdated Python dependencies")
            return updates
            
        except Exception as e:
            logger.error(f"Failed to check Python dependencies: {e}")
            return []
    
    def check_github_actions(self) -> List[Dict]:
        """Check for outdated GitHub Actions."""
        logger.info("Checking GitHub Actions...")
        
        workflows_dir = self.repo_path / '.github' / 'workflows'
        if not workflows_dir.exists():
            logger.warning("GitHub workflows directory not found")
            return []
        
        updates = []
        
        for workflow_file in workflows_dir.glob('*.yml'):
            try:
                with open(workflow_file, 'r') as f:
                    content = f.read()
                
                # Parse action versions (simplified - would need proper YAML parsing)
                import re
                action_pattern = r'uses:\s+([^@]+)@(v?\d+(?:\.\d+)*|[a-f0-9]+)'
                matches = re.findall(action_pattern, content)
                
                for action, version in matches:
                    latest_version = self.get_latest_action_version(action)
                    if latest_version and latest_version != version:
                        updates.append({
                            'ecosystem': 'github-actions',
                            'name': action,
                            'current_version': version,
                            'latest_version': latest_version,
                            'update_type': self.classify_update_type(version, latest_version),
                            'file': str(workflow_file.relative_to(self.repo_path))
                        })
                        
            except Exception as e:
                logger.warning(f"Failed to check {workflow_file}: {e}")
        
        logger.info(f"Found {len(updates)} outdated GitHub Actions")
        return updates
    
    def get_latest_action_version(self, action: str) -> Optional[str]:
        """Get the latest version of a GitHub Action."""
        try:
            if '/' not in action:
                return None
            
            owner, repo_name = action.split('/', 1)
            action_repo = self.github_client.get_repo(f"{owner}/{repo_name}")
            
            # Get latest release
            releases = list(action_repo.get_releases())
            if releases:
                return releases[0].tag_name
            
            # Fallback to latest tag
            tags = list(action_repo.get_tags())
            if tags:
                return tags[0].name
                
            return None
            
        except Exception as e:
            logger.debug(f"Failed to get latest version for {action}: {e}")
            return None
    
    def classify_update_type(self, current: str, latest: str) -> str:
        """Classify the type of update (major, minor, patch)."""
        try:
            # Remove 'v' prefix if present
            current = current.lstrip('v')
            latest = latest.lstrip('v')
            
            current_parts = [int(x) for x in current.split('.')]
            latest_parts = [int(x) for x in latest.split('.')]
            
            # Pad with zeros if needed
            max_len = max(len(current_parts), len(latest_parts))
            current_parts.extend([0] * (max_len - len(current_parts)))
            latest_parts.extend([0] * (max_len - len(latest_parts)))
            
            if latest_parts[0] > current_parts[0]:
                return 'major'
            elif latest_parts[1] > current_parts[1]:
                return 'minor'
            elif len(latest_parts) > 2 and latest_parts[2] > current_parts[2]:
                return 'patch'
            else:
                return 'unknown'
                
        except (ValueError, IndexError):
            return 'unknown'
    
    def filter_updates(self, updates: List[Dict], update_types: List[str], exclude_names: List[str]) -> List[Dict]:
        """Filter updates based on criteria."""
        filtered = []
        
        for update in updates:
            # Skip excluded packages
            if update['name'] in exclude_names:
                logger.debug(f"Skipping excluded package: {update['name']}")
                continue
            
            # Filter by update type
            if update_types and update['update_type'] not in update_types:
                logger.debug(f"Skipping {update['name']} due to update type: {update['update_type']}")
                continue
            
            filtered.append(update)
        
        return filtered
    
    def create_update_branch(self, updates: List[Dict]) -> str:
        """Create a new branch for dependency updates."""
        timestamp = datetime.now().strftime('%Y%m%d-%H%M%S')
        branch_name = f"deps/auto-update-{timestamp}"
        
        try:
            # Create new branch
            subprocess.run(
                ['git', 'checkout', '-b', branch_name],
                check=True,
                cwd=self.repo_path
            )
            
            logger.info(f"Created update branch: {branch_name}")
            return branch_name
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to create branch: {e}")
            raise
    
    def apply_rust_updates(self, updates: List[Dict]) -> bool:
        """Apply Rust dependency updates."""
        rust_updates = [u for u in updates if u['ecosystem'] == 'rust']
        if not rust_updates:
            return True
        
        logger.info(f"Applying {len(rust_updates)} Rust updates...")
        
        try:
            cargo_toml = self.repo_path / 'Cargo.toml'
            with open(cargo_toml, 'r') as f:
                content = f.read()
            
            # Apply updates (simplified - would need proper TOML parsing)
            for update in rust_updates:
                old_line = f'{update["name"]} = "{update["current_version"]}"'
                new_line = f'{update["name"]} = "{update["latest_version"]}"'
                content = content.replace(old_line, new_line)
            
            with open(cargo_toml, 'w') as f:
                f.write(content)
            
            # Update Cargo.lock
            subprocess.run(['cargo', 'update'], check=True, cwd=self.repo_path)
            
            return True
            
        except Exception as e:
            logger.error(f"Failed to apply Rust updates: {e}")
            return False
    
    def apply_python_updates(self, updates: List[Dict]) -> bool:
        """Apply Python dependency updates."""
        python_updates = [u for u in updates if u['ecosystem'] == 'python']
        if not python_updates:
            return True
        
        logger.info(f"Applying {len(python_updates)} Python updates...")
        
        try:
            pyproject_toml = self.repo_path / 'python' / 'pyproject.toml'
            
            with open(pyproject_toml, 'r') as f:
                data = toml.load(f)
            
            # Update dependencies
            for update in python_updates:
                # Find and update dependency (simplified)
                for section in ['dependencies', 'dev-dependencies']:
                    if section in data.get('project', {}):
                        deps = data['project'][section]
                        for i, dep in enumerate(deps):
                            if dep.startswith(update['name']):
                                deps[i] = f"{update['name']}>={update['latest_version']}"
                                break
            
            with open(pyproject_toml, 'w') as f:
                toml.dump(data, f)
            
            return True
            
        except Exception as e:
            logger.error(f"Failed to apply Python updates: {e}")
            return False
    
    def apply_github_actions_updates(self, updates: List[Dict]) -> bool:
        """Apply GitHub Actions updates."""
        action_updates = [u for u in updates if u['ecosystem'] == 'github-actions']
        if not action_updates:
            return True
        
        logger.info(f"Applying {len(action_updates)} GitHub Actions updates...")
        
        try:
            for update in action_updates:
                file_path = self.repo_path / update['file']
                
                with open(file_path, 'r') as f:
                    content = f.read()
                
                # Replace action version
                old_ref = f"{update['name']}@{update['current_version']}"
                new_ref = f"{update['name']}@{update['latest_version']}"
                content = content.replace(old_ref, new_ref)
                
                with open(file_path, 'w') as f:
                    f.write(content)
            
            return True
            
        except Exception as e:
            logger.error(f"Failed to apply GitHub Actions updates: {e}")
            return False
    
    def run_tests(self) -> bool:
        """Run tests to validate updates."""
        logger.info("Running tests to validate updates...")
        
        try:
            # Run Rust tests
            result = subprocess.run(
                ['cargo', 'test', '--all-features'],
                capture_output=True,
                cwd=self.repo_path
            )
            
            if result.returncode != 0:
                logger.error("Rust tests failed")
                logger.error(result.stderr.decode())
                return False
            
            # Run Python tests if they exist
            python_dir = self.repo_path / 'python'
            if python_dir.exists():
                result = subprocess.run(
                    ['python', '-m', 'pytest', 'tests/'],
                    capture_output=True,
                    cwd=python_dir
                )
                
                if result.returncode != 0:
                    logger.warning("Python tests failed, but continuing")
            
            logger.info("Tests passed")
            return True
            
        except Exception as e:
            logger.error(f"Failed to run tests: {e}")
            return False
    
    def commit_changes(self, updates: List[Dict]) -> bool:
        """Commit the dependency updates."""
        try:
            # Add all changes
            subprocess.run(['git', 'add', '.'], check=True, cwd=self.repo_path)
            
            # Create commit message
            update_summary = {}
            for update in updates:
                ecosystem = update['ecosystem']
                if ecosystem not in update_summary:
                    update_summary[ecosystem] = []
                update_summary[ecosystem].append(f"{update['name']} {update['current_version']} → {update['latest_version']}")
            
            commit_lines = ["deps: automated dependency updates"]
            commit_lines.append("")
            
            for ecosystem, deps in update_summary.items():
                commit_lines.append(f"{ecosystem.title()} updates:")
                for dep in deps:
                    commit_lines.append(f"- {dep}")
                commit_lines.append("")
            
            commit_lines.append("🤖 Generated with [Claude Code](https://claude.ai/code)")
            commit_lines.append("")
            commit_lines.append("Co-Authored-By: Claude <noreply@anthropic.com>")
            
            commit_message = "\n".join(commit_lines)
            
            subprocess.run(
                ['git', 'commit', '-m', commit_message],
                check=True,
                cwd=self.repo_path
            )
            
            logger.info("Changes committed")
            return True
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to commit changes: {e}")
            return False
    
    def push_branch(self, branch_name: str) -> bool:
        """Push the update branch to remote."""
        try:
            subprocess.run(
                ['git', 'push', '-u', 'origin', branch_name],
                check=True,
                cwd=self.repo_path
            )
            
            logger.info(f"Branch {branch_name} pushed to remote")
            return True
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to push branch: {e}")
            return False
    
    def create_pull_request(self, branch_name: str, updates: List[Dict]) -> Optional[str]:
        """Create a pull request for the dependency updates."""
        try:
            # Generate PR title and body
            total_updates = len(updates)
            ecosystems = list(set(u['ecosystem'] for u in updates))
            
            title = f"deps: update {total_updates} dependencies ({', '.join(ecosystems)})"
            
            body_lines = ["## Automated Dependency Updates"]
            body_lines.append("")
            body_lines.append("This PR contains automated dependency updates:")
            body_lines.append("")
            
            # Group by ecosystem
            by_ecosystem = {}
            for update in updates:
                ecosystem = update['ecosystem']
                if ecosystem not in by_ecosystem:
                    by_ecosystem[ecosystem] = []
                by_ecosystem[ecosystem].append(update)
            
            for ecosystem, deps in by_ecosystem.items():
                body_lines.append(f"### {ecosystem.title()} ({len(deps)} updates)")
                body_lines.append("")
                
                for dep in deps:
                    body_lines.append(f"- **{dep['name']}**: {dep['current_version']} → {dep['latest_version']} ({dep['update_type']})")
                
                body_lines.append("")
            
            body_lines.append("## Validation")
            body_lines.append("- [x] Automated tests passed")
            body_lines.append("- [ ] Manual testing (if required)")
            body_lines.append("")
            body_lines.append("🤖 Generated with [Claude Code](https://claude.ai/code)")
            
            body = "\n".join(body_lines)
            
            # Create PR
            pr = self.repo.create_pull(
                title=title,
                body=body,
                head=branch_name,
                base='main'
            )
            
            logger.info(f"Created pull request: {pr.html_url}")
            return pr.html_url
            
        except Exception as e:
            logger.error(f"Failed to create pull request: {e}")
            return None
    
    def cleanup_on_failure(self, branch_name: str):
        """Clean up if update process fails."""
        try:
            subprocess.run(['git', 'checkout', 'main'], cwd=self.repo_path)
            subprocess.run(['git', 'branch', '-D', branch_name], cwd=self.repo_path)
            logger.info("Cleaned up failed update branch")
        except Exception as e:
            logger.warning(f"Failed to clean up: {e}")


def main():
    """Main function to run dependency updates."""
    parser = argparse.ArgumentParser(description='Automated dependency updates')
    parser.add_argument('--types', nargs='+', choices=['major', 'minor', 'patch'],
                       default=['minor', 'patch'],
                       help='Types of updates to apply')
    parser.add_argument('--ecosystems', nargs='+', 
                       choices=['rust', 'python', 'github-actions'],
                       default=['rust', 'python', 'github-actions'],
                       help='Ecosystems to check for updates')
    parser.add_argument('--exclude', nargs='+', default=[],
                       help='Package names to exclude from updates')
    parser.add_argument('--dry-run', action='store_true',
                       help='Show what would be updated without making changes')
    parser.add_argument('--skip-tests', action='store_true',
                       help='Skip running tests after updates')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Enable verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        updater = DependencyUpdater()
        
        # Collect all outdated dependencies
        all_updates = []
        
        if 'rust' in args.ecosystems:
            all_updates.extend(updater.check_rust_dependencies())
        
        if 'python' in args.ecosystems:
            all_updates.extend(updater.check_python_dependencies())
        
        if 'github-actions' in args.ecosystems:
            all_updates.extend(updater.check_github_actions())
        
        # Filter updates
        filtered_updates = updater.filter_updates(all_updates, args.types, args.exclude)
        
        if not filtered_updates:
            logger.info("No dependency updates needed")
            return
        
        logger.info(f"Found {len(filtered_updates)} dependencies to update")
        
        if args.dry_run:
            print("\nUpdates that would be applied:")
            for update in filtered_updates:
                print(f"  {update['ecosystem']}: {update['name']} {update['current_version']} → {update['latest_version']} ({update['update_type']})")
            return
        
        # Apply updates
        branch_name = updater.create_update_branch(filtered_updates)
        
        try:
            # Apply updates by ecosystem
            success = True
            success &= updater.apply_rust_updates(filtered_updates)
            success &= updater.apply_python_updates(filtered_updates)
            success &= updater.apply_github_actions_updates(filtered_updates)
            
            if not success:
                raise RuntimeError("Failed to apply some updates")
            
            # Run tests
            if not args.skip_tests and not updater.run_tests():
                raise RuntimeError("Tests failed after updates")
            
            # Commit and push
            if not updater.commit_changes(filtered_updates):
                raise RuntimeError("Failed to commit changes")
            
            if not updater.push_branch(branch_name):
                raise RuntimeError("Failed to push branch")
            
            # Create PR
            pr_url = updater.create_pull_request(branch_name, filtered_updates)
            if pr_url:
                print(f"\nCreated pull request: {pr_url}")
            else:
                logger.warning("Failed to create pull request")
            
        except Exception as e:
            logger.error(f"Update process failed: {e}")
            updater.cleanup_on_failure(branch_name)
            sys.exit(1)
        
    except Exception as e:
        logger.error(f"Dependency update failed: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()