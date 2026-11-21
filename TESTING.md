# geodaddy — Test Guide

## Prerequisites

- Rust toolchain installed (via `rustup`). Check: `rustc --version`
- `jq` installed (for formatting JSON output). macOS: `brew install jq`
- Internet connection (some tests hit `httpbin.org`)

---

## 1. Build the Binary

```bash
cargo build --release
```

The first build takes ~1-2 minutes (compiling dependencies). On success:

```
Finished `release` profile [optimized] target(s) in ...
```

Binary is at: `target/release/geodaddy`

---

## 2. Basic Usage

### --help

```bash
./target/release/geodaddy --help
```

Expected output (abridged):

```
GEO analysis tool — surface actionable AI search optimization issues

Usage: geodaddy [OPTIONS] [URL] [COMMAND]

Commands:
  compare      Compare 2+ URLs side-by-side with per-category diff and winner detection...
  see          Show what an AI crawler (GPTBot / ClaudeBot / PerplexityBot) sees when it visits the page...
  app          Analyze a mobile app's AI recommendability from its App Store or Play Store URL...
  app-compare  Compare 2+ apps side-by-side across the app score categories...
  llms-txt     Generate a spec-compliant llms.txt index from a site (per https://llmstxt.org)...
  help         Print this message or the help of the given subcommand(s)

Arguments:
  [URL]  URL to analyze (omit when using a subcommand like `compare`)

Options:
      --fail-under <SCORE>  Exit with code 1 if overall score is below this threshold (0-100)...
      --max-pages <N>       Enable crawling and stop after N pages...
      --enable-js           Enable JavaScript rendering for pages detected as JS-heavy...
      --vitals              Measure lab Web Vitals (LCP, FCP, CLS, TTFB, and TBT as a lab proxy for INP)...
      --beauty              Output a colored, human-readable report instead of JSON
  -h, --help                Print help
  -V, --version             Print version
```

---

## 3. Analyze a URL

### A real URL

```bash
./target/release/geodaddy https://example.com
```

Expected JSON output (stdout):

```json
{
  "schema_version": "1",
  "url": "https://example.com",
  "crawled_at": "2026-03-23T...",
  "pages": [
    {
      "url": "https://example.com/",
      "robots_blocked": false,
      "results": [...]
    }
  ]
}
```

### Formatted with jq

```bash
./target/release/geodaddy https://example.com | jq .
```

### A specific field

```bash
./target/release/geodaddy https://example.com | jq '.pages[0].robots_blocked'
```

---

## 4. Localhost Test

Should work without errors even when no local server is running:

```bash
./target/release/geodaddy http://localhost:3000
```

Expected: `robots_blocked: false`, exit code `0`

If a real app is running on localhost (e.g., a Next.js dev server):

```bash
./target/release/geodaddy http://localhost:3000/blog/post/1
```

---

## 5. robots.txt Behavior

### Site with a robots.txt

```bash
./target/release/geodaddy https://openai.com | jq '.pages[0].robots_blocked'
```

If the site's `robots.txt` blocks `GPTBot` but has no rule for `geodaddy` (or `*`), this returns `false`.

### Intentional block test

A site with `Disallow: /` in its `robots.txt` will return `robots_blocked: true` — but **the crawl still continues** (soft warn behavior).

---

## 6. --fail-under Flag

For CI/CD pipeline integration:

```bash
# Exits 1 if score is below 50
./target/release/geodaddy --fail-under 50 https://example.com
echo "Exit code: $?"   # expect 1

# Exits 0 if threshold is 0
./target/release/geodaddy --fail-under 0 https://example.com
echo "Exit code: $?"   # expect 0
```

---

## 7. Automated Test Suite

Run all tests at once:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

To also run integration tests (requires internet):

```bash
chmod +x tests/integration_test.sh
bash tests/integration_test.sh
```

Expected output:

```
PASS: --help exits 0 and contains docs
PASS: JSON structure: schema_version, pages[0] with url/robots_blocked/results
PASS: --fail-under 50 exits 1 (score=0.0 < 50)
PASS: --fail-under 0 exits 0 (score=0.0 >= 0)
PASS: stdout is valid JSON (no tracing noise)
PASS: localhost with no server: robots_blocked=false (graceful)

Results: 7 passed, 0 failed
```

> Tests 2–6 make requests to `http://httpbin.org/get`. They may fail without an internet connection.

---

## 8. JSON stdout Cleanliness (Pipe Test)

Verify that JSON output contains no tracing/log noise:

```bash
./target/release/geodaddy https://example.com 2>/dev/null | jq .
```

If `jq` parses without errors, stdout is clean.

To see verbose tracing:

```bash
RUST_LOG=debug ./target/release/geodaddy https://example.com 2>&1 | head -20
```

---

## 9. Error Cases

### Invalid URL

```bash
./target/release/geodaddy "not-a-url"
```

Expected: error message on stderr, exit code `1`

### Unreachable host

```bash
./target/release/geodaddy https://this-domain-does-not-exist-xyz.com
```

Expected: JSON output is produced, `robots_blocked: false`

---

## 10. CI/CD Integration

```bash
# Pass criterion: site reachable + threshold 0
./target/release/geodaddy --fail-under 0 https://example.com

# Recommended usage with a score gate:
./target/release/geodaddy --fail-under 70 https://mysite.com
```

---

## Summary

| Test | Command | Expected |
|------|---------|----------|
| Build | `cargo build --release` | Compiles successfully |
| Help | `geodaddy --help` | Usage docs printed |
| Basic analysis | `geodaddy https://example.com` | JSON output |
| Localhost | `geodaddy http://localhost:3000` | Exit 0, robots_blocked false |
| Exit code fail | `geodaddy --fail-under 50 <url>` | Exit 1 |
| Exit code pass | `geodaddy --fail-under 0 <url>` | Exit 0 |
| Automated tests | `bash tests/integration_test.sh` | 7/7 PASS |
