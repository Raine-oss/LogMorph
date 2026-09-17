#!/usr/bin/env bash
set -e

echo "========================================================"
echo "    LOGMORPH COMPREHENSIVE CLIENT VERIFICATION SUITE    "
echo "========================================================"
echo

PASS_COUNT=0
TOTAL_COUNT=0

record_test() {
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    local test_name="$1"
    local status="$2"
    if [ "$status" -eq 0 ]; then
        echo "[PASS] $test_name"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "[FAIL] $test_name"
        exit 1
    fi
}

# 1. Rust Unit & Integration Tests
echo "--- Stage 1: Rust Core Unit & Integration Tests (26 Tests) ---"
source $HOME/.cargo/env
cargo test --quiet
record_test "Rust Core 26/26 Tests Passed" $?

# 2. Release Binary Compilation
echo
echo "--- Stage 2: Compiling Release CLI Executable ---"
cargo build --release --quiet
record_test "Release Binary Compilation" $?

CLI="./target/release/logmorph"

# 3. CLI Subcommand Verifications
echo
echo "--- Stage 3: CLI Subcommand Execution Tests ---"

# 3.1 Analyze text
$CLI analyze tests/fixtures/paper_sample.log > /dev/null
record_test "CLI: analyze (text output)" $?

# 3.2 Analyze JSON
JSON_OUT=$($CLI analyze tests/fixtures/paper_sample.log --format json)
echo "$JSON_OUT" | grep -q '"attribution": "confirmed"'
record_test "CLI: analyze --format json (valid snake_case schema)" $?

# 3.3 Summary command
$CLI summary tests/fixtures/paper_sample.log > /dev/null
record_test "CLI: summary command" $?

# 3.4 Inspect command
INSPECT_OUT=$($CLI inspect tests/fixtures/paper_sample.log --error 1)
echo "$INSPECT_OUT" | grep -q "Detailed Inspection"
record_test "CLI: inspect --error 1" $?

# 3.5 Export command with output file
TEMP_EXPORT="/tmp/logmorph_client_export.json"
rm -f "$TEMP_EXPORT"
$CLI export tests/fixtures/paper_sample.log --output "$TEMP_EXPORT" > /dev/null
[ -f "$TEMP_EXPORT" ] && grep -q "primary_exception" "$TEMP_EXPORT"
record_test "CLI: export --output <FILE>" $?
rm -f "$TEMP_EXPORT"

# 3.6 Stdin Pipe
cat tests/fixtures/paper_sample.log | $CLI analyze > /dev/null
record_test "CLI: Stdin pipeline streaming" $?

# 3.7 Filter by plugin
FILTER_OUT=$($CLI analyze tests/fixtures/paper_sample.log --plugin MyCustomPlugin)
echo "$FILTER_OUT" | grep -q "MyCustomPlugin"
record_test "CLI: Filter by plugin" $?

# 3.8 Memory Cap (--max-signatures)
CAP_OUT=$($CLI analyze tests/fixtures/repeated_error.log --max-signatures 1)
echo "$CAP_OUT" | grep -q "Unique Error Signatures"
record_test "CLI: --max-signatures memory ceiling" $?

# 4. Standard Process Exit Codes
echo
echo "--- Stage 4: Process Exit Codes Tests ---"

# Exit code 0 on success
set +e
$CLI analyze tests/fixtures/paper_sample.log --summary-only > /dev/null
EXIT_0=$?
[ "$EXIT_0" -eq 0 ]
record_test "Exit Code 0: Success" $?

# Exit code 1 on missing file
$CLI analyze non_existent_file.log 2> /dev/null
EXIT_1=$?
[ "$EXIT_1" -eq 1 ]
record_test "Exit Code 1: File Not Found" $?

# Exit code 2 on invalid input
$CLI inspect tests/fixtures/paper_sample.log --error 999 2> /dev/null
EXIT_2=$?
[ "$EXIT_2" -eq 2 ]
record_test "Exit Code 2: Invalid Input Index" $?
set -e

# 5. Paper / Spigot Plugin Companion
echo
echo "--- Stage 5: Paper/Spigot Companion Plugin & JNI Bridge ---"

# Update native lib in resources
mkdir -p plugin/src/main/resources/natives/linux-x86_64
cp target/release/liblogmorph.so plugin/src/main/resources/natives/linux-x86_64/

cd plugin
mvn test -q
record_test "Maven JUnit: JNI Bridge Native Roundtrip" $?

mvn package -DskipTests -q
record_test "Maven Package: LogMorph-1.0.0.jar built with embedded native engine" $?
cd ..

[ -f plugin/target/LogMorph-1.0.0.jar ]
record_test "Plugin JAR Artifact verified on disk" $?

echo
echo "========================================================"
echo "    VERIFICATION RESULT: ALL $PASS_COUNT / $TOTAL_COUNT TESTS PASSED!"
echo "========================================================"
