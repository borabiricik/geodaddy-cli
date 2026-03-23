#!/usr/bin/env bash
BINARY="$(dirname "$0")/../target/release/geodaddy"

PASS=0
FAIL=0

run_test() {
  local name="$1"
  local result="$2"
  if [ "$result" = "PASS" ]; then
    echo "PASS: $name"
    PASS=$((PASS + 1))
  else
    echo "FAIL: $name"
    FAIL=$((FAIL + 1))
  fi
}

# Test 1: --help
OUTPUT=$("$BINARY" --help 2>&1)
EXIT=$?
[[ $EXIT -eq 0 && "$OUTPUT" == *"URL to analyze"* && "$OUTPUT" == *"fail-under"* ]] \
  && run_test "--help exits 0 and contains docs" "PASS" \
  || run_test "--help exits 0 and contains docs" "FAIL"

# Test 2+3: JSON structure for real URL
OUTPUT=$("$BINARY" http://httpbin.org/get 2>/dev/null)
EXIT=$?
SCHEMA_VER=$(echo "$OUTPUT" | jq -r '.schema_version' 2>/dev/null)
PAGES_LEN=$(echo "$OUTPUT" | jq '.pages | length' 2>/dev/null)
PAGE_HAS_URL=$(echo "$OUTPUT" | jq -r '.pages[0].url' 2>/dev/null)
PAGE_HAS_RB=$(echo "$OUTPUT" | jq '.pages[0].robots_blocked' 2>/dev/null)
PAGE_HAS_RESULTS=$(echo "$OUTPUT" | jq '.pages[0].results | length' 2>/dev/null)
[[ $EXIT -eq 0 && "$SCHEMA_VER" == "1" && "$PAGES_LEN" == "1" && \
   "$PAGE_HAS_URL" != "null" && "$PAGE_HAS_RB" != "null" && "$PAGE_HAS_RESULTS" == "0" ]] \
  && run_test "JSON structure: schema_version, pages[0] with url/robots_blocked/results" "PASS" \
  || run_test "JSON structure: schema_version, pages[0] with url/robots_blocked/results" "FAIL"

# Test 4: --fail-under > 0 exits 1
"$BINARY" --fail-under 50 http://httpbin.org/get 2>/dev/null >/dev/null
EXIT=$?
[[ $EXIT -eq 1 ]] \
  && run_test "--fail-under 50 exits 1 (score=0.0 < 50)" "PASS" \
  || run_test "--fail-under 50 exits 1 (score=0.0 < 50)" "FAIL"

# Test 5: --fail-under 0 exits 0
"$BINARY" --fail-under 0 http://httpbin.org/get 2>/dev/null >/dev/null
EXIT=$?
[[ $EXIT -eq 0 ]] \
  && run_test "--fail-under 0 exits 0 (score=0.0 >= 0)" "PASS" \
  || run_test "--fail-under 0 exits 0 (score=0.0 >= 0)" "FAIL"

# Test 6: stdout is clean JSON (no tracing noise)
OUTPUT=$("$BINARY" http://httpbin.org/get 2>/dev/null)
echo "$OUTPUT" | jq . >/dev/null 2>&1 \
  && run_test "stdout is valid JSON (no tracing noise)" "PASS" \
  || run_test "stdout is valid JSON (no tracing noise)" "FAIL"

# Test 7: localhost with no server — graceful failure (connection refused = robots allow-all)
OUTPUT=$("$BINARY" http://localhost:19999/path/to/deep/page 2>/dev/null)
EXIT=$?
RB=$(echo "$OUTPUT" | jq '.pages[0].robots_blocked' 2>/dev/null)
[[ $EXIT -eq 0 && "$RB" == "false" ]] \
  && run_test "localhost with no server: robots_blocked=false (graceful)" "PASS" \
  || run_test "localhost with no server: robots_blocked=false (graceful)" "FAIL"

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]] && exit 0 || exit 1
