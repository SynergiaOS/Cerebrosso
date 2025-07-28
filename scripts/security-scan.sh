#!/bin/bash

# 🔒 Cerberus Phoenix v2.0 - Security Scan Script
# Comprehensive security audit using Snyk and additional tools

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Create security reports directory
REPORTS_DIR="security-reports/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$REPORTS_DIR"

log "🔒 Starting Cerberus Phoenix v2.0 Security Audit"
log "📁 Reports will be saved to: $REPORTS_DIR"

# Function to scan Rust project with Snyk
scan_rust_project() {
    local project_path=$1
    local project_name=$2
    
    log "🦀 Scanning Rust project: $project_name"
    
    if [ ! -f "$project_path/Cargo.toml" ]; then
        error "Cargo.toml not found in $project_path"
        return 1
    fi
    
    cd "$project_path"
    
    # Generate Cargo.lock if it doesn't exist
    if [ ! -f "Cargo.lock" ]; then
        log "📦 Generating Cargo.lock for $project_name"
        cargo generate-lockfile
    fi
    
    # Run Snyk test
    log "🔍 Running Snyk vulnerability scan for $project_name"
    if snyk test --severity-threshold=medium --json > "../../../$REPORTS_DIR/${project_name}-vulnerabilities.json" 2>/dev/null; then
        success "✅ No medium+ vulnerabilities found in $project_name"
    else
        warning "⚠️  Vulnerabilities found in $project_name - check report"
    fi
    
    # Generate human-readable report
    snyk test --severity-threshold=low > "../../../$REPORTS_DIR/${project_name}-report.txt" 2>/dev/null || true
    
    # Monitor project for future vulnerabilities
    log "📊 Setting up monitoring for $project_name"
    snyk monitor --project-name="cerberus-$project_name" > "../../../$REPORTS_DIR/${project_name}-monitor.txt" 2>/dev/null || true
    
    cd - > /dev/null
}

# Function to scan for secrets
scan_secrets() {
    log "🔐 Scanning for exposed secrets and API keys"
    
    # Common secret patterns
    local secret_patterns=(
        "api[_-]?key"
        "secret[_-]?key"
        "private[_-]?key"
        "password"
        "token"
        "auth"
        "credential"
    )
    
    for pattern in "${secret_patterns[@]}"; do
        log "🔍 Searching for pattern: $pattern"
        grep -r -i --include="*.rs" --include="*.toml" --include="*.json" --include="*.env*" \
            "$pattern" . > "$REPORTS_DIR/secrets-scan-$pattern.txt" 2>/dev/null || true
    done
    
    # Check for hardcoded Solana addresses and keys
    log "🔍 Scanning for hardcoded Solana addresses"
    grep -r --include="*.rs" -E "[1-9A-HJ-NP-Za-km-z]{32,44}" . > "$REPORTS_DIR/solana-addresses.txt" 2>/dev/null || true
    
    success "🔐 Secret scanning completed"
}

# Function to analyze dependencies
analyze_dependencies() {
    log "📦 Analyzing dependency security"
    
    # Find all Cargo.toml files
    find . -name "Cargo.toml" -not -path "./target/*" > "$REPORTS_DIR/cargo-files.txt"
    
    # Extract all dependencies
    log "📋 Extracting dependency list"
    {
        echo "# Cerberus Phoenix v2.0 - All Dependencies"
        echo "# Generated on $(date)"
        echo ""
        
        while IFS= read -r cargo_file; do
            echo "## Dependencies from $cargo_file"
            grep -A 100 "^\[dependencies\]" "$cargo_file" | grep -E "^[a-zA-Z0-9_-]+ = " || true
            echo ""
        done < "$REPORTS_DIR/cargo-files.txt"
    } > "$REPORTS_DIR/all-dependencies.md"
    
    success "📦 Dependency analysis completed"
}

# Function to check for outdated dependencies
check_outdated() {
    log "📅 Checking for outdated dependencies"
    
    for project in "services/hft-ninja" "services/cerebro-bff"; do
        if [ -d "$project" ]; then
            log "🔍 Checking outdated deps in $project"
            cd "$project"
            cargo outdated > "../../$REPORTS_DIR/$(basename $project)-outdated.txt" 2>/dev/null || {
                warning "cargo-outdated not installed. Install with: cargo install cargo-outdated"
            }
            cd - > /dev/null
        fi
    done
}

# Function to generate security summary
generate_summary() {
    log "📊 Generating security summary"
    
    {
        echo "# 🔒 Cerberus Phoenix v2.0 - Security Audit Summary"
        echo "Generated on: $(date)"
        echo ""
        
        echo "## 📋 Scan Results"
        echo ""
        
        # Count vulnerabilities
        local vuln_count=0
        for file in "$REPORTS_DIR"/*-vulnerabilities.json; do
            if [ -f "$file" ]; then
                local count=$(jq '.vulnerabilities | length' "$file" 2>/dev/null || echo "0")
                vuln_count=$((vuln_count + count))
                echo "- $(basename "$file" -vulnerabilities.json): $count vulnerabilities"
            fi
        done
        
        echo ""
        echo "**Total vulnerabilities found: $vuln_count**"
        echo ""
        
        if [ $vuln_count -eq 0 ]; then
            echo "✅ **No vulnerabilities found - Good security posture!**"
        elif [ $vuln_count -lt 5 ]; then
            echo "⚠️  **Low risk - Few vulnerabilities found**"
        elif [ $vuln_count -lt 15 ]; then
            echo "🚨 **Medium risk - Several vulnerabilities found**"
        else
            echo "🔥 **High risk - Many vulnerabilities found - Immediate action required!**"
        fi
        
        echo ""
        echo "## 📁 Report Files"
        echo ""
        ls -la "$REPORTS_DIR"/ | tail -n +2 | while read -r line; do
            echo "- $line"
        done
        
        echo ""
        echo "## 🎯 Next Steps"
        echo ""
        echo "1. Review vulnerability reports in detail"
        echo "2. Update vulnerable dependencies"
        echo "3. Implement security patches"
        echo "4. Setup continuous monitoring"
        echo "5. Review and rotate any exposed secrets"
        
    } > "$REPORTS_DIR/SECURITY-SUMMARY.md"
    
    success "📊 Security summary generated"
}

# Main execution
main() {
    log "🚀 Starting comprehensive security audit"
    
    # Check if Snyk is authenticated
    if ! snyk auth --check > /dev/null 2>&1; then
        error "Snyk not authenticated. Please run: snyk auth"
        exit 1
    fi
    
    # Scan Rust projects
    scan_rust_project "services/hft-ninja" "hft-ninja"
    scan_rust_project "services/cerebro-bff" "cerebro-bff"
    
    # Additional security checks
    scan_secrets
    analyze_dependencies
    check_outdated
    
    # Generate summary
    generate_summary
    
    success "🎉 Security audit completed!"
    log "📁 All reports saved to: $REPORTS_DIR"
    log "📊 Check SECURITY-SUMMARY.md for overview"
    
    # Display summary
    echo ""
    cat "$REPORTS_DIR/SECURITY-SUMMARY.md"
}

# Run main function
main "$@"
