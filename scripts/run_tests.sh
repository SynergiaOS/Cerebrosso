#!/bin/bash
# 🧪 Comprehensive Test Runner for Cerberus Phoenix v2.0

set -e

echo "🧪 Starting Comprehensive Test Suite for Cerberus Phoenix v2.0"
echo "================================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
UNIT_TESTS_PASSED=0
INTEGRATION_TESTS_PASSED=0
E2E_TESTS_PASSED=0
TOTAL_FAILURES=0

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "INFO")
            echo -e "${BLUE}ℹ️  $message${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠️  $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}❌ $message${NC}"
            ;;
    esac
}

# Function to run tests with error handling
run_test_suite() {
    local test_name=$1
    local test_command=$2
    local required_services=$3
    
    print_status "INFO" "Running $test_name..."
    
    if [ ! -z "$required_services" ]; then
        print_status "INFO" "Checking required services: $required_services"
        # Add service health checks here if needed
    fi
    
    if eval "$test_command"; then
        print_status "SUCCESS" "$test_name passed"
        return 0
    else
        print_status "ERROR" "$test_name failed"
        TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
        return 1
    fi
}

# Change to project root
cd "$(dirname "$0")/.."

print_status "INFO" "Project root: $(pwd)"

# 1. Unit Tests
echo ""
print_status "INFO" "🔬 Running Unit Tests"
echo "----------------------------------------"

# Test Cerebro-BFF unit tests
if run_test_suite "Cerebro-BFF Unit Tests" "cd services/cerebro-bff && cargo test --lib"; then
    UNIT_TESTS_PASSED=$((UNIT_TESTS_PASSED + 1))
fi

# Test HFT-Ninja Minimal unit tests (if any)
if [ -d "services/hft-ninja-minimal" ]; then
    if run_test_suite "HFT-Ninja Minimal Unit Tests" "cd services/hft-ninja-minimal && cargo test --lib"; then
        UNIT_TESTS_PASSED=$((UNIT_TESTS_PASSED + 1))
    fi
fi

# 2. Integration Tests
echo ""
print_status "INFO" "🔗 Running Integration Tests"
echo "----------------------------------------"

# Test Cerebro-BFF integration tests
if run_test_suite "Cerebro-BFF Integration Tests" "cd services/cerebro-bff && cargo test --test integration_tests"; then
    INTEGRATION_TESTS_PASSED=$((INTEGRATION_TESTS_PASSED + 1))
fi

# Test Context Engine specific tests
if run_test_suite "Context Engine Tests" "cd services/cerebro-bff && cargo test --test context_engine_tests"; then
    INTEGRATION_TESTS_PASSED=$((INTEGRATION_TESTS_PASSED + 1))
fi

# 3. End-to-End Tests (require running services)
echo ""
print_status "INFO" "🌐 Running End-to-End Tests"
echo "----------------------------------------"

# Check if services are running
print_status "INFO" "Checking if services are running..."

CEREBRO_RUNNING=false
HFT_NINJA_RUNNING=false

if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    CEREBRO_RUNNING=true
    print_status "SUCCESS" "Cerebro-BFF is running on port 3000"
else
    print_status "WARNING" "Cerebro-BFF is not running on port 3000"
fi

if curl -s http://localhost:8082/health > /dev/null 2>&1; then
    HFT_NINJA_RUNNING=true
    print_status "SUCCESS" "HFT-Ninja is running on port 8082"
else
    print_status "WARNING" "HFT-Ninja is not running on port 8082"
fi

if [ "$CEREBRO_RUNNING" = true ] && [ "$HFT_NINJA_RUNNING" = true ]; then
    print_status "INFO" "Both services are running, executing E2E tests"
    
    if run_test_suite "End-to-End Tests" "cd services/cerebro-bff && cargo test --test e2e_tests -- --ignored"; then
        E2E_TESTS_PASSED=$((E2E_TESTS_PASSED + 1))
    fi
else
    print_status "WARNING" "Services not running, skipping E2E tests"
    print_status "INFO" "To run E2E tests, start services with:"
    print_status "INFO" "  - Cerebro-BFF: cd services/cerebro-bff && cargo run"
    print_status "INFO" "  - HFT-Ninja: cd services/hft-ninja-minimal && PORT=8082 cargo run"
fi

# 4. Performance Tests (optional)
echo ""
print_status "INFO" "⚡ Running Performance Tests"
echo "----------------------------------------"

if [ "$CEREBRO_RUNNING" = true ]; then
    print_status "INFO" "Running basic performance benchmarks..."
    
    # Simple performance test using curl
    RESPONSE_TIME=$(curl -o /dev/null -s -w '%{time_total}' http://localhost:3000/health)
    print_status "INFO" "Health endpoint response time: ${RESPONSE_TIME}s"
    
    if (( $(echo "$RESPONSE_TIME < 1.0" | bc -l) )); then
        print_status "SUCCESS" "Performance test passed (< 1s)"
    else
        print_status "WARNING" "Performance test warning (>= 1s)"
    fi
else
    print_status "WARNING" "Cerebro-BFF not running, skipping performance tests"
fi

# 5. Test Summary
echo ""
echo "================================================================"
print_status "INFO" "📊 Test Summary"
echo "================================================================"

echo "Unit Tests Passed: $UNIT_TESTS_PASSED"
echo "Integration Tests Passed: $INTEGRATION_TESTS_PASSED"
echo "E2E Tests Passed: $E2E_TESTS_PASSED"
echo "Total Failures: $TOTAL_FAILURES"

if [ $TOTAL_FAILURES -eq 0 ]; then
    print_status "SUCCESS" "🎉 All tests passed!"
    echo ""
    print_status "SUCCESS" "Cerberus Phoenix v2.0 is ready for deployment!"
    exit 0
else
    print_status "ERROR" "❌ $TOTAL_FAILURES test suite(s) failed"
    echo ""
    print_status "ERROR" "Please fix the failing tests before deployment"
    exit 1
fi
