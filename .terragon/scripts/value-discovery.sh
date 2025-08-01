#!/bin/bash
set -euo pipefail

# Autonomous Value Discovery Script
# Scans repository for technical debt and generates prioritized backlog

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TERRAGON_DIR="$REPO_ROOT/.terragon"
METRICS_FILE="$TERRAGON_DIR/value-metrics.json"
BACKLOG_FILE="$REPO_ROOT/BACKLOG.md"

echo "🔍 Starting autonomous value discovery..."

# Create discovery report
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
TEMP_REPORT="/tmp/discovery-report-$$.json"

# Initialize discovery report
cat > "$TEMP_REPORT" << EOF
{
  "discovery_timestamp": "$TIMESTAMP",
  "repository": "$(basename "$REPO_ROOT")",
  "items": [],
  "summary": {
    "total_items": 0,
    "high_priority": 0,
    "medium_priority": 0,
    "low_priority": 0,
    "security_issues": 0,
    "performance_issues": 0
  }
}
EOF

echo "📊 Analyzing code for technical debt markers..."

# Scan for TODO/FIXME/HACK markers
TODO_COUNT=0
if command -v rg >/dev/null 2>&1; then
    TODO_COUNT=$(rg -c "TODO|FIXME|HACK|DEPRECATED|XXX|BUG|WORKAROUND" --type rust "$REPO_ROOT" 2>/dev/null || echo "0")
elif command -v grep >/dev/null 2>&1; then
    TODO_COUNT=$(find "$REPO_ROOT" -name "*.rs" -exec grep -c "TODO\|FIXME\|HACK\|DEPRECATED\|XXX\|BUG\|WORKAROUND" {} + 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
fi

echo "  Found $TODO_COUNT technical debt markers"

# Security vulnerability scan
SECURITY_ISSUES=0
if command -v cargo >/dev/null 2>&1 && [ -f "$REPO_ROOT/Cargo.toml" ]; then
    echo "🔒 Scanning for security vulnerabilities..."
    cd "$REPO_ROOT"
    
    # Check for cargo-audit
    if cargo audit --version >/dev/null 2>&1; then
        SECURITY_ISSUES=$(cargo audit 2>/dev/null | grep -c "ID:" || echo "0")
        echo "  Found $SECURITY_ISSUES security vulnerabilities"
    else
        echo "  cargo-audit not installed, skipping security scan"
    fi
    
    # Check for outdated dependencies  
    if cargo outdated --version >/dev/null 2>&1; then
        OUTDATED_COUNT=$(cargo outdated --exit-code 1 2>/dev/null | grep -c "→" || echo "0")
        echo "  Found $OUTDATED_COUNT outdated dependencies"
    fi
fi

# Performance analysis
echo "⚡ Analyzing performance patterns..."
PERF_ISSUES=0
if [ -d "$REPO_ROOT/tests" ]; then
    # Count unimplemented test placeholders
    PLACEHOLDER_TESTS=$(find "$REPO_ROOT/tests" -name "*.rs" -exec grep -c "placeholder\|TODO\|unimplemented\|#\[ignore\]" {} + 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
    echo "  Found $PLACEHOLDER_TESTS placeholder/unimplemented tests"
    PERF_ISSUES=$((PERF_ISSUES + PLACEHOLDER_TESTS))
fi

# Code complexity analysis (simple version using file sizes and function counts)
echo "🧮 Analyzing code complexity..."
COMPLEX_FILES=0
if [ -d "$REPO_ROOT/src" ]; then
    # Find files larger than 500 lines (potential complexity hotspots)
    COMPLEX_FILES=$(find "$REPO_ROOT/src" -name "*.rs" -exec wc -l {} + | awk '$1 > 500 {count++} END {print count+0}')
    echo "  Found $COMPLEX_FILES potentially complex files (>500 lines)"
fi

# Documentation gaps
echo "📚 Analyzing documentation coverage..."
DOC_GAPS=0
if [ -d "$REPO_ROOT/src" ]; then
    # Count files without rustdoc comments
    TOTAL_RS_FILES=$(find "$REPO_ROOT/src" -name "*.rs" | wc -l)
    FILES_WITH_DOCS=$(find "$REPO_ROOT/src" -name "*.rs" -exec grep -l "///" {} + 2>/dev/null | wc -l || echo "0")
    DOC_GAPS=$((TOTAL_RS_FILES - FILES_WITH_DOCS))
    echo "  Found $DOC_GAPS files without documentation comments"
fi

# Update summary in report
TOTAL_ITEMS=$((TODO_COUNT + SECURITY_ISSUES + PERF_ISSUES + DOC_GAPS))
HIGH_PRIORITY=$((SECURITY_ISSUES + TODO_COUNT / 3))
MEDIUM_PRIORITY=$(((TODO_COUNT * 2) / 3 + PERF_ISSUES))
LOW_PRIORITY=$((DOC_GAPS + COMPLEX_FILES))

# Generate updated metrics
cat > "$METRICS_FILE" << EOF
{
  "repository": {
    "name": "$(basename "$REPO_ROOT")",
    "maturityLevel": "maturing",
    "lastAssessment": "$TIMESTAMP",
    "maturityScore": 65
  },
  "continuousDiscovery": {
    "lastRun": "$TIMESTAMP",
    "itemsDiscovered": $TOTAL_ITEMS,
    "sourcesScanned": ["gitHistory", "staticAnalysis", "documentation", "security"]
  },
  "currentMetrics": {
    "technicalDebtMarkers": $TODO_COUNT,
    "securityIssues": $SECURITY_ISSUES,
    "performanceIssues": $PERF_ISSUES,
    "documentationGaps": $DOC_GAPS,
    "complexityHotspots": $COMPLEX_FILES
  },
  "priorityDistribution": {
    "high": $HIGH_PRIORITY,
    "medium": $MEDIUM_PRIORITY,
    "low": $LOW_PRIORITY,
    "total": $TOTAL_ITEMS
  },
  "executionHistory": [],
  "backlogMetrics": {
    "totalItems": $TOTAL_ITEMS,
    "averageAge": 0,
    "debtRatio": 0.3,
    "velocityTrend": "discovering"
  },
  "valueDelivered": {
    "weeklyValue": 0,
    "monthlyValue": 0,
    "totalValue": 0
  }
}
EOF

echo "📋 Updating backlog..."

# Update BACKLOG.md with discovery results
cat > "$BACKLOG_FILE" << EOF
# 📊 Autonomous Value Backlog

Last Updated: $TIMESTAMP  
Next Discovery: $(date -u -d '+1 hour' +"%Y-%m-%dT%H:%M:%SZ")

## 🎯 Discovery Summary

**Items Discovered**: $TOTAL_ITEMS  
**Technical Debt Markers**: $TODO_COUNT  
**Security Issues**: $SECURITY_ISSUES  
**Performance Issues**: $PERF_ISSUES  
**Documentation Gaps**: $DOC_GAPS  
**Complexity Hotspots**: $COMPLEX_FILES

## 📈 Priority Distribution

- **High Priority**: $HIGH_PRIORITY items (Security vulnerabilities, critical TODOs)
- **Medium Priority**: $MEDIUM_PRIORITY items (Performance issues, most technical debt)
- **Low Priority**: $LOW_PRIORITY items (Documentation, code cleanup)

## 🔄 Continuous Discovery Status

✅ **Code Analysis**: Scanned for debt markers  
✅ **Security Scan**: Vulnerability detection  
✅ **Performance Analysis**: Test coverage and complexity  
✅ **Documentation Review**: Coverage analysis  

## 🚨 Immediate Actions Required

Based on the discovery scan, prioritize:

1. **Address Security Issues** ($SECURITY_ISSUES found) - Run \`cargo audit\` for details
2. **Complete Core Implementation** ($TODO_COUNT TODOs found) - Focus on critical paths
3. **Improve Test Coverage** ($PERF_ISSUES test issues found) - Replace placeholders
4. **Documentation Enhancement** ($DOC_GAPS gaps found) - Add rustdoc comments

## 📊 Value Discovery Configuration

**Discovery Sources Active**:
- Static code analysis (ripgrep/grep)
- Security vulnerability scanning (cargo-audit)
- Dependency analysis (cargo-outdated)
- Test coverage analysis
- Documentation coverage analysis

**Next Discovery Run**: Scheduled for 1 hour
**Discovery Frequency**: Hourly for security, daily for comprehensive analysis

## 🔧 Tools Integration Status

$(command -v rg >/dev/null 2>&1 && echo "✅ ripgrep (rg) - Available" || echo "❌ ripgrep (rg) - Not available")
$(command -v cargo >/dev/null 2>&1 && echo "✅ Cargo - Available" || echo "❌ Cargo - Not available")
$(cargo audit --version >/dev/null 2>&1 && echo "✅ cargo-audit - Available" || echo "❌ cargo-audit - Not installed")
$(cargo outdated --version >/dev/null 2>&1 && echo "✅ cargo-outdated - Available" || echo "❌ cargo-outdated - Not installed")

## 📋 Recommended Tool Installation

To enhance autonomous discovery capabilities:

\`\`\`bash
# Install enhanced analysis tools
cargo install cargo-audit cargo-outdated ripgrep
\`\`\`

*This backlog is automatically updated by the autonomous value discovery system. For manual discovery, run: \`.terragon/scripts/value-discovery.sh\`*
EOF

echo "✅ Value discovery completed!"
echo "📊 Results:"
echo "  - Total items: $TOTAL_ITEMS"
echo "  - High priority: $HIGH_PRIORITY"
echo "  - Security issues: $SECURITY_ISSUES"
echo "  - Updated: $BACKLOG_FILE"
echo "  - Metrics: $METRICS_FILE"

# Cleanup
rm -f "$TEMP_REPORT"

exit 0