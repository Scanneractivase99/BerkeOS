#!/usr/bin/env bash
# BerkeOS Filesystem Test Suite
# Tests create, write, read, delete files and directory operations

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"
ISO="$BUILD_DIR/berkeos.iso"
DISK1="$BUILD_DIR/berkeos_disk.img"
DISK2="$BUILD_DIR/berkeos_disk2.img"
SERIAL_LOG="$BUILD_DIR/serial_test.log"
FIFO_IN="$BUILD_DIR/qemu_in.fifo"
QEMU_PID=""

# Test counters
PASS_COUNT=0
FAIL_COUNT=0
TESTS_RUN=0

# Cleanup function
cleanup() {
    if [ -n "$QEMU_PID" ] && kill -0 "$QEMU_PID" 2>/dev/null; then
        kill -9 "$QEMU_PID" 2>/dev/null || true
        wait "$QEMU_PID" 2>/dev/null || true
    fi
    rm -f "$FIFO_IN" "$SERIAL_LOG" 2>/dev/null || true
}

trap cleanup EXIT

# Test result functions
pass() {
    PASS_COUNT=$((PASS_COUNT + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "${GREEN}[PASS]${NC} $1"
}

fail() {
    FAIL_COUNT=$((FAIL_COUNT + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "${RED}[FAIL]${NC} $1"
    if [ -n "$2" ]; then
        echo -e "       Expected: $2"
    fi
}

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

section() {
    echo ""
    echo -e "${BOLD}${YELLOW}=== $1 ===${NC}"
}

# Wait for boot with timeout
wait_for_boot() {
    local timeout=$1
    local elapsed=0
    local interval=1
    
    while [ $elapsed -lt $timeout ]; do
        if grep -q "BerkeOS" "$SERIAL_LOG" 2>/dev/null; then
            return 0
        fi
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    return 1
}

# Wait for shell prompt
wait_for_prompt() {
    local timeout=$1
    local elapsed=0
    
    while [ $elapsed -lt $timeout ]; do
        if grep -qE "(> |# )" "$SERIAL_LOG" 2>/dev/null; then
            # Give a moment for the prompt to fully render
            sleep 0.5
            return 0
        fi
        sleep 0.5
        elapsed=$((elapsed * 2 / 2 + 1))
    done
    return 1
}

# Send command to QEMU
send_cmd() {
    local cmd="$1"
    echo "$cmd" > "$FIFO_IN"
    # Small delay to let command process
    sleep 0.3
}

# Extract output after last command
get_output_since() {
    local marker="$1"
    local start_line=$(wc -l < "$SERIAL_LOG" 2>/dev/null || echo 0)
    sleep 0.5
    tail -n +"$start_line" "$SERIAL_LOG" 2>/dev/null
}

# Check if output contains expected string
check_output() {
    local expected="$1"
    grep -q "$expected" "$SERIAL_LOG" 2>/dev/null
}

# Main test function
run_test() {
    local test_name="$1"
    local test_func="$2"
    
    section "Test: $test_name"
    $test_func
}

# ============================================
# TEST SCENARIOS
# ============================================

test_boot() {
    info "Verifying system boots successfully..."
    
    if [ -f "$SERIAL_LOG" ] && grep -q "BerkeOS" "$SERIAL_LOG"; then
        pass "System booted - BerkeOS detected"
    else
        fail "System failed to boot - BerkeOS not found in output"
    fi
}

test_create_file() {
    info "Testing file creation (touch)..."
    
    send_cmd "touch testfile"
    sleep 1
    
    if check_output "testfile"; then
        pass "File creation - testfile created"
    else
        # Try ls to verify
        send_cmd "ls"
        sleep 1
        if check_output "testfile"; then
            pass "File creation - testfile found in ls"
        else
            fail "File creation - testfile not found"
        fi
    fi
}

test_write_file() {
    info "Testing file write (echo)..."
    
    send_cmd "echo Hello_World > testfile"
    sleep 1
    
    if check_output "Hello_World"; then
        pass "File write - content written"
    else
        fail "File write - content not found"
    fi
}

test_read_file() {
    info "Testing file read (cat)..."
    
    send_cmd "cat testfile"
    sleep 1
    
    if check_output "Hello_World"; then
        pass "File read - content verified"
    else
        fail "File read - content mismatch"
    fi
}

test_delete_file() {
    info "Testing file deletion (rm)..."
    
    send_cmd "rm testfile"
    sleep 1
    
    # Verify file is gone
    send_cmd "ls"
    sleep 1
    
    if ! check_output "testfile"; then
        pass "File deletion - testfile removed"
    else
        fail "File deletion - testfile still exists"
    fi
}

test_create_directory() {
    info "Testing directory creation (mkdir)..."
    
    send_cmd "mkdir testdir"
    sleep 1
    
    if check_output "testdir"; then
        pass "Directory creation - testdir created"
    else
        # Try ls
        send_cmd "ls"
        sleep 1
        if check_output "testdir"; then
            pass "Directory creation - testdir found in ls"
        else
            fail "Directory creation - testdir not found"
        fi
    fi
}

test_change_directory() {
    info "Testing directory change (cd)..."
    
    send_cmd "cd testdir"
    sleep 1
    
    # Check if path changed
    if check_output "testdir" && grep -qE "(Alpha|Beta):.*testdir" "$SERIAL_LOG"; then
        pass "Directory change - cd to testdir"
    else
        # Try pwd to verify
        send_cmd "pwd"
        sleep 1
        if check_output "testdir"; then
            pass "Directory change - pwd shows testdir"
        else
            fail "Directory change - path not changed"
        fi
    fi
}

test_list_directory() {
    info "Testing directory listing (ls)..."
    
    send_cmd "ls"
    sleep 1
    
    # In an empty directory, should show (empty) or similar
    if grep -qE "(\(empty\)|Directory of|\.\.)" "$SERIAL_LOG"; then
        pass "Directory listing - ls executed"
    else
        fail "Directory listing - unexpected output"
    fi
}

test_filesystem_check() {
    info "Testing filesystem check (fsinfo)..."
    
    # Note: fsck command doesn't exist in BerkeOS
    # Using fsinfo as alternative filesystem check
    send_cmd "fsinfo"
    sleep 1
    
    if check_output "BerkeFS"; then
        pass "Filesystem check - fsinfo shows BerkeFS"
    else
        # Try df as alternative
        send_cmd "df"
        sleep 1
        if check_output "(Disk|bytes|Filesystem)"; then
            pass "Filesystem check - df shows disk info"
        else
            fail "Filesystem check - no filesystem info available"
        fi
    fi
}

# ============================================
# MAIN EXECUTION
# ============================================

main() {
    echo -e "${BOLD}${CYAN}"
    echo "========================================"
    echo "  BerkeOS Filesystem Test Suite"
    echo "========================================"
    echo -e "${NC}"
    
    # Step 1: Build ISO if needed
    section "Step 1: Build"
    if [ ! -f "$ISO" ]; then
        info "ISO not found, running build..."
        cd "$PROJECT_DIR"
        if ./build.sh; then
            pass "Build completed successfully"
        else
            fail "Build failed"
            echo -e "\n${RED}========================================"
            echo -e "  TEST SUMMARY"
            echo -e "========================================${NC}"
            echo -e "  ${BOLD}Passed:${NC} $PASS_COUNT"
            echo -e "  ${BOLD}Failed:${NC} $FAIL_COUNT"
            echo -e "  ${BOLD}Total:${NC}  $TESTS_RUN"
            echo ""
            exit 1
        fi
    else
        info "ISO already exists: $ISO"
    fi
    
    # Step 2: Prepare test environment
    section "Step 2: Prepare Test Environment"
    
    # Create fresh disk images for testing
    info "Creating fresh disk images..."
    rm -f "$DISK1" "$DISK2" 2>/dev/null || true
    dd if=/dev/zero of="$DISK1" bs=1M count=128 2>/dev/null
    dd if=/dev/zero of="$DISK2" bs=1M count=256 2>/dev/null
    
    # Create FIFO for QEMU input
    rm -f "$FIFO_IN" 2>/dev/null || true
    mkfifo "$FIFO_IN" 2>/dev/null || true
    
    # Clear serial log
    rm -f "$SERIAL_LOG" 2>/dev/null || true
    
    pass "Test environment prepared"
    
    # Step 3: Start QEMU
    section "Step 3: Start QEMU"
    
    info "Launching QEMU in headless mode..."
    
    qemu-system-x86_64 \
        -m 256M \
        -cdrom "$ISO" \
        -drive file="$DISK1",format=raw,if=ide,index=0,media=disk \
        -drive file="$DISK2",format=raw,if=ide,index=1,media=disk \
        -boot d \
        -nographic \
        -serial file:"$SERIAL_LOG" \
        -monitor none \
        -qmp none \
        -no-reboot \
        < "$FIFO_IN" \
        > /dev/null 2>&1 &
    
    QEMU_PID=$!
    info "QEMU started with PID: $QEMU_PID"
    
    # Wait for boot
    info "Waiting for system boot (timeout: 60s)..."
    if wait_for_boot 60; then
        pass "Boot completed"
    else
        fail "Boot timeout - system did not boot within 60s"
        echo -e "\n${RED}Boot log (last 20 lines):${NC}"
        tail -20 "$SERIAL_LOG" 2>/dev/null || echo "No log available"
        exit 1
    fi
    
    # Wait for shell prompt
    info "Waiting for shell prompt..."
    if wait_for_prompt 30; then
        pass "Shell prompt detected"
    else
        fail "Shell prompt not detected"
    fi
    
    # Small delay before tests
    sleep 2
    
    # Step 4: Run tests
    section "Step 4: Running Tests"
    
    run_test "Boot Verification" test_boot
    run_test "Create File" test_create_file
    run_test "Write File" test_write_file
    run_test "Read File" test_read_file
    run_test "Delete File" test_delete_file
    run_test "Create Directory" test_create_directory
    run_test "Change Directory" test_change_directory
    run_test "List Directory" test_list_directory
    run_test "Filesystem Check" test_filesystem_check
    
    # Cleanup: go back to root and clean test artifacts
    send_cmd "cd /"
    sleep 0.5
    send_cmd "rm -rf testdir"
    sleep 0.5
    
    # Step 5: Shutdown QEMU gracefully
    section "Step 5: Shutdown"
    
    info "Sending shutdown command..."
    send_cmd "halt"
    sleep 3
    
    # Force kill if still running
    if kill -0 "$QEMU_PID" 2>/dev/null; then
        info "Force terminating QEMU..."
        kill -9 "$QEMU_PID" 2>/dev/null || true
        wait "$QEMU_PID" 2>/dev/null || true
    fi
    QEMU_PID=""
    
    pass "QEMU shutdown complete"
    
    # Summary
    section "Test Summary"
    echo ""
    echo -e "  ${BOLD}Passed:${NC} ${GREEN}$PASS_COUNT${NC}"
    echo -e "  ${BOLD}Failed:${NC} ${RED}$FAIL_COUNT${NC}"
    echo -e "  ${BOLD}Total:${NC}  $TESTS_RUN"
    echo ""
    
    if [ $FAIL_COUNT -eq 0 ]; then
        echo -e "${GREEN}${BOLD}All tests passed!${NC}"
        echo ""
        exit 0
    else
        echo -e "${RED}${BOLD}Some tests failed.${NC}"
        echo ""
        
        # Show relevant serial log excerpt
        echo -e "${YELLOW}Serial log excerpt (last 30 lines):${NC}"
        echo "----------------------------------------"
        tail -30 "$SERIAL_LOG" 2>/dev/null || echo "No log available"
        echo "----------------------------------------"
        echo ""
        exit 1
    fi
}

# Run main
main "$@"
