/// Integration tests for geodaddy Phase 04: site-wide crawling.
///
/// These tests automate the 5 human-verification scenarios from 04-VERIFICATION.md
/// plus 2 additional edge cases. All tests use mockito to serve a real localhost
/// HTTP server — no external network calls are made.
///
/// Mockito note: `mockito::Server::new()` is synchronous in v1. Use it directly.
/// The server stays alive as long as the `server` variable is in scope.
use assert_cmd::Command;
use mockito::Server;
use serde_json::Value;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Minimal sitemap XML pointing at the mock server root URL.
fn sitemap_body(base_url: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>{}/</loc><priority>1.0</priority></url></urlset>"#,
        base_url
    )
}

/// Minimal valid HTML that passes the needs_js_rendering check
/// (has >= 1 heading and >= 1 paragraph, so JS rendering is NOT triggered).
fn minimal_html() -> &'static str {
    "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World content here.</p></body></html>"
}

/// robots.txt that allows all crawlers and sets Crawl-delay: 0
/// so tests don't wait 1 second between pages.
fn robots_txt() -> &'static str {
    "User-agent: *\nAllow: /\nCrawl-delay: 0\nContent-Signal: search=yes, ai-input=yes, ai-train=no\n"
}

/// Mock the site-wide agent-category well-known endpoints so that
/// mock_site passes the most impactful agent-readiness checks. Without
/// these, the agent category scores near zero and drags the overall
/// average below the thresholds asserted by compare tests.
fn mock_agent_endpoints(server: &mut Server) {
    let mocks = [
        ("/.well-known/mcp/server-card.json", r#"{"mcpServers":[]}"#),
        ("/.well-known/agent-card.json", r#"{"name":"mock","capabilities":["payment"]}"#),
        ("/.well-known/agent-skills/index.json", r#"{"skills":[]}"#),
        (
            "/.well-known/api-catalog",
            r#"{"linkset":[{"anchor":"https://example.com","service":[]}]}"#,
        ),
        (
            "/.well-known/oauth-authorization-server",
            r#"{"issuer":"https://example.com","authorization_endpoint":"https://example.com/auth"}"#,
        ),
        (
            "/.well-known/oauth-protected-resource",
            r#"{"resource":"https://example.com"}"#,
        ),
    ];
    for (path, body) in mocks {
        let m = server
            .mock("GET", path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        std::mem::forget(m);
    }
}

// ── Test 1: JSON output has score, categories, and pages ──────────────────────
//
// Verification scenario 1 from 04-VERIFICATION.md:
//   "confirm JSON contains top-level score, categories, and pages fields"

#[test]
fn test_json_output_has_score_categories_pages() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {}\nstdout was:\n{}", e, stdout));

    assert!(
        json["score"].is_number(),
        "expected json[\"score\"] to be a number, got: {}",
        json["score"]
    );
    assert!(
        json["categories"]["technical"].is_number(),
        "expected json[\"categories\"][\"technical\"] to be a number"
    );
    assert!(
        json["categories"]["content"].is_number(),
        "expected json[\"categories\"][\"content\"] to be a number"
    );
    assert!(
        json["categories"]["geo"].is_number(),
        "expected json[\"categories\"][\"geo\"] to be a number"
    );
    assert!(
        json["pages"].is_array(),
        "expected json[\"pages\"] to be an array"
    );
    assert!(
        json["pages"].as_array().unwrap().len() >= 1,
        "expected at least 1 page in pages array"
    );
}

// ── Test 2: stdout is pure JSON ───────────────────────────────────────────────
//
// Verification scenario 3 from 04-VERIFICATION.md:
//   "progress lines must NOT appear in stdout"

#[test]
fn test_stdout_is_pure_json() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Must parse as JSON — this is the primary assertion
    let _: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {}\nstdout was:\n{}", e, stdout));

    // stdout must not contain progress lines (those belong on stderr)
    assert!(
        !stdout.contains("Crawling page"),
        "progress text leaked to stdout: {}",
        stdout
    );
    assert!(
        !stdout.contains("[1/"),
        "progress text leaked to stdout: {}",
        stdout
    );
}

// ── Test 3: sitemap-driven progress goes to stderr ────────────────────────────
//
// Verification scenario 2 from 04-VERIFICATION.md:
//   "Lines in format [1/N] https://..." (sitemap-driven)

#[test]
fn test_progress_to_stderr_sitemap_format() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .arg("--max-pages")
        .arg("10")
        .output()
        .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();

    // Sitemap-driven format: "[1/1] http://..."
    assert!(
        stderr.contains("[1/"),
        "expected \"[1/\" in stderr for sitemap-driven progress, got:\n{}",
        stderr
    );
}

// ── Test 4: BFS fallback when sitemap returns 404 ─────────────────────────────
//
// Verification scenario 2 from 04-VERIFICATION.md (BFS path):
//   "No sitemap.xml found" message and "Crawling page 1..." format

#[test]
fn test_progress_to_stderr_bfs_fallback() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    // Sitemap returns 404 — triggers BFS fallback
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(404)
        .with_body("Not Found")
        .create();

    // Root page with no links (BFS depth 2, but only root needed)
    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .arg("--max-pages")
        .arg("10")
        .output()
        .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        stderr.contains("No sitemap"),
        "expected \"No sitemap\" message in stderr for BFS fallback, got:\n{}",
        stderr
    );
    assert!(
        stderr.contains("Crawling page 1"),
        "expected \"Crawling page 1\" in stderr for BFS fallback, got:\n{}",
        stderr
    );
}

// ── Test 5: --max-pages caps the number of pages in the report ────────────────
//
// Verification scenario 4 from 04-VERIFICATION.md:
//   "2 pages (or fewer if site has fewer pages)"

#[test]
fn test_max_pages_caps_result() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    // Sitemap with 3 URLs
    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>{}/</loc><priority>1.0</priority></url><url><loc>{}/page2</loc><priority>0.9</priority></url><url><loc>{}/page3</loc><priority>0.8</priority></url></urlset>"#,
        server.url(),
        server.url(),
        server.url()
    );

    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let _m_page2 = server
        .mock("GET", "/page2")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let _m_page3 = server
        .mock("GET", "/page3")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .arg("--max-pages")
        .arg("2")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {}\nstdout was:\n{}", e, stdout));

    let page_count = json["pages"].as_array().unwrap().len();
    assert!(
        page_count <= 2,
        "expected at most 2 pages with --max-pages 2, got {}",
        page_count
    );
}

// ── Test 6: --fail-under 0 exits 0 ───────────────────────────────────────────
//
// Verification scenario 5 from 04-VERIFICATION.md:
//   "exit: 0" (score will never be below 0)

#[test]
fn test_fail_under_zero_exits_zero() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .arg("--fail-under")
        .arg("0")
        .output()
        .unwrap();

    let exit_code = output.status.code().unwrap();
    assert_eq!(
        exit_code, 0,
        "--fail-under 0 should always exit 0 (score is never below 0)"
    );
}

// ── Test 8: --vitals not passed → performance is null in JSON ─────────────────
//
// D-05: performance must be null (not missing, not 100.0) when --vitals not passed

#[test]
fn test_no_vitals_performance_null() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(server.url())
        .output()
        .unwrap();

    assert!(output.status.success(), "geodaddy should exit 0");

    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be valid JSON");

    // D-05: performance must be null (not missing, not 100.0) when --vitals not passed
    assert!(
        json["categories"]["performance"].is_null(),
        "categories.performance must be null when --vitals is not passed, got: {}",
        json["categories"]["performance"]
    );

    // Also verify pages[0].categories.performance is null
    assert!(
        json["pages"][0]["categories"]["performance"].is_null(),
        "pages[0].categories.performance must be null when --vitals is not passed"
    );
}

// ── Test 9: --vitals flag is recognized by the CLI ────────────────────────────
//
// Verifies that --vitals is a recognized CLI flag (not rejected as unknown argument).
// Marked #[ignore] because the actual vitals measurement requires Chromium.
// Run manually with: cargo test -- --ignored test_vitals_flag_accepted

/// Verifies that --vitals is a recognized CLI flag (not rejected as unknown argument).
/// Marked #[ignore] because the actual vitals measurement requires Chromium.
/// Run manually with: cargo test -- --ignored test_vitals_flag_accepted
#[test]
#[ignore]
fn test_vitals_flag_accepted() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    let sitemap = sitemap_body(&server.url());
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(server.url())
        .arg("--vitals")
        .output()
        .unwrap();

    // The flag must be recognized — exit code may be non-zero if Chromium unavailable,
    // but stderr must NOT contain "unexpected argument '--vitals'"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument '--vitals'"),
        "geodaddy should recognize --vitals flag, stderr: {}",
        stderr
    );
}

// ── Test 7: non-HTML URLs in sitemap are not included in pages ────────────────
//
// Edge case: sitemap contains .rss and .xml URLs — they must be filtered out
// by is_html_url() inside fetch_sitemap_urls().

#[test]
fn test_non_html_urls_not_in_pages() {
    let mut server = Server::new();

    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();

    // Sitemap contains /, /feed.rss, /sitemap-posts.xml
    // Only / should survive is_html_url filtering in fetch_sitemap_urls
    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>{}/</loc><priority>1.0</priority></url><url><loc>{}/feed.rss</loc><priority>0.5</priority></url><url><loc>{}/sitemap-posts.xml</loc><priority>0.5</priority></url></urlset>"#,
        server.url(),
        server.url(),
        server.url()
    );

    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();

    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(minimal_html())
        .create();

    // These are mocked but should NOT be fetched (filtered before crawl loop)
    let _m_rss = server
        .mock("GET", "/feed.rss")
        .with_status(200)
        .with_body("<rss></rss>")
        .create();

    let _m_xml = server
        .mock("GET", "/sitemap-posts.xml")
        .with_status(200)
        .with_body("<urlset></urlset>")
        .create();

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg(&server.url())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {}\nstdout was:\n{}", e, stdout));

    let pages = json["pages"].as_array().unwrap();

    for page in pages {
        let url = page["url"].as_str().unwrap_or("");
        assert!(
            !url.ends_with(".rss"),
            "pages should not contain .rss URLs, found: {}",
            url
        );
        assert!(
            !url.ends_with(".xml"),
            "pages should not contain .xml URLs, found: {}",
            url
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Phase 8 — Competitor comparison integration tests (Wave 0 stubs, red).
//
// These tests exercise the `compare` subcommand. Wave 0 lands only the stub
// that prints "not yet implemented" + exits 2 — so every test below FAILS in
// Wave 0 and passes in Wave 1 once compare_sites is implemented.
// ════════════════════════════════════════════════════════════════════════════

fn mock_site(server: &mut Server) {
    let sitemap = sitemap_body(&server.url());
    let _m_robots = server
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();
    let _m_sitemap = server
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap)
        .create();
    let _m_root = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_header("link", "</sitemap.xml>; rel=\"sitemap\"")
        .with_body(minimal_html())
        .create();
    // Leak mocks for test lifetime
    std::mem::forget(_m_robots);
    std::mem::forget(_m_sitemap);
    std::mem::forget(_m_root);
    mock_agent_endpoints(server);
}

#[test]
fn test_compare_requires_two_urls() {
    // clap MUST reject `geodaddy compare <single-url>` because num_args = 2..
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("https://only-one.example.com")
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(0), "single-URL compare must NOT succeed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required")
            || stderr.contains("usage")
            || stderr.contains("2 values")
            || stderr.contains("minimum"),
        "expected clap error about needing 2 URLs, got stderr: {}",
        stderr
    );
}

#[test]
fn test_compare_shares_http_client() {
    // Success criterion: both mocks hit (proxied by successful CompareReport.sites having 2 entries).
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {} — stdout: {}", e, stdout));
    assert_eq!(json["sites"].as_array().map(|v| v.len()), Some(2));
}

#[test]
fn test_compare_max_pages_per_target() {
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("--max-pages")
        .arg("1")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("JSON: {} / {}", e, stdout));
    // Each site gets its own --max-pages=1 budget.
    assert_eq!(json["sites"][0]["pages"].as_array().map(|v| v.len()), Some(1));
    assert_eq!(json["sites"][1]["pages"].as_array().map(|v| v.len()), Some(1));
}

#[test]
fn test_compare_beauty_prints_table() {
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("--beauty")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Competitor Comparison"), "beauty mode header missing");
    assert!(stdout.contains("Overall Score"), "overall score row missing");
    assert!(stdout.contains("Winners"), "winners section missing");
}

#[test]
fn test_compare_json_schema_stable() {
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(json["schema_version"], "1");
    assert!(json["compared_at"].is_string());
    assert!(json["sites"].is_array());
    assert!(json["winners"].is_object());
    assert!(json["check_diff"].is_array());
    assert!(json["errors"].is_array());
}

#[test]
fn test_compare_fail_under_first_url() {
    // First URL's score is low → threshold 99 forces exit 1.
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("--fail-under")
        .arg("99.0")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "first URL below threshold → exit 1");
}

#[test]
fn test_compare_competitor_low_score_ignored() {
    // First URL = high-scoring site; competitor = bare HTML (low score). Threshold 50 → first passes.
    let mut a = Server::new();
    mock_site(&mut a); // a is first; threshold 50 OK
    let mut b = Server::new();
    let _mb_robots = b
        .mock("GET", "/robots.txt")
        .with_status(200)
        .with_body(robots_txt())
        .create();
    let _mb_sitemap = b
        .mock("GET", "/sitemap.xml")
        .with_status(200)
        .with_body(sitemap_body(&b.url()))
        .create();
    let _mb_root = b
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body></body></html>")
        .create();
    std::mem::forget(_mb_robots);
    std::mem::forget(_mb_sitemap);
    std::mem::forget(_mb_root);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("--fail-under")
        .arg("50.0")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    // First URL is above 50, competitor below → must NOT exit 1.
    assert_ne!(output.status.code(), Some(1), "competitor failure must not trigger --fail-under");
}

#[test]
fn test_compare_continues_on_per_url_error() {
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    // Intentionally use one unreachable URL in the middle.
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg("http://127.0.0.1:1")
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(json["sites"].as_array().map(|v| v.len()), Some(2), "a and b succeed");
    assert_eq!(json["errors"].as_array().map(|v| v.len()), Some(1), "middle URL surfaces as error");
}

#[test]
fn test_compare_first_url_failure_exit_2() {
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg("http://127.0.0.1:1")
        .arg(b.url())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "first URL fails → exit 2");
}

#[test]
fn test_compare_dedupes_duplicate_urls() {
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(json["sites"].as_array().map(|v| v.len()), Some(2), "deduped to 2 unique");
    assert!(stderr.to_lowercase().contains("duplicate"), "stderr must warn about duplicate URL");
}

#[test]
fn test_compare_winners_populated() {
    // Covers COMP-05 end-to-end: winners object present with expected keys.
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{}", e));
    for key in &["overall", "technical", "content", "geo", "performance"] {
        assert!(json["winners"].get(*key).is_some(), "winners.{} must exist", key);
    }
}

#[test]
fn test_compare_check_diff_populated() {
    // Covers COMP-06 end-to-end: check_diff has entries shaped as { check, results: [{url, status}] }.
    let mut a = Server::new();
    mock_site(&mut a);
    let mut b = Server::new();
    mock_site(&mut b);
    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("compare")
        .arg(a.url())
        .arg(b.url())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{}", e));
    let diff = json["check_diff"].as_array().expect("check_diff array");
    assert!(!diff.is_empty(), "check_diff must not be empty");
    let first = &diff[0];
    assert!(first["check"].is_string());
    assert!(first["results"].is_array());
    assert_eq!(first["results"].as_array().unwrap().len(), 2, "one entry per site");
}

// ════════════════════════════════════════════════════════════════════════════
// Quick task 260508-g4j — `llms-txt` subcommand integration tests.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn test_llms_txt_subcommand_produces_spec_compliant_output() {
    let mut server = Server::new();
    mock_site(&mut server);

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("llms-txt")
        .arg(server.url())
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got {:?} — stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("# "),
        "expected H1 at start, got: {:?}",
        &stdout[..stdout.len().min(80)]
    );
    assert!(
        stdout.lines().next().unwrap_or("").contains("Test"),
        "expected H1 to contain mock site title 'Test', got: {:?}",
        stdout.lines().next()
    );
    assert!(
        stdout.lines().any(|l| l.starts_with("## ")),
        "expected at least one H2 section, got stdout:\n{}",
        stdout
    );
    let link_line = stdout
        .lines()
        .find(|l| l.starts_with("- ["))
        .unwrap_or_else(|| panic!("expected at least one link line, got:\n{}", stdout));
    assert!(
        link_line.contains(&server.url()),
        "expected link line to reference mock URL {}, got: {}",
        server.url(),
        link_line
    );
}

#[test]
fn test_llms_txt_writes_to_output_file_when_o_flag() {
    let mut server = Server::new();
    mock_site(&mut server);

    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("geodaddy-llms-txt-{}-{}.txt", pid, nanos));

    let output = Command::cargo_bin("geodaddy")
        .unwrap()
        .arg("llms-txt")
        .arg(server.url())
        .arg("-o")
        .arg(&tmp)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0, got {:?} — stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    // stdout should be empty (no llms.txt body to stdout when -o is set)
    assert!(
        output.stdout.is_empty()
            || String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "expected empty stdout when -o is used, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let body = std::fs::read_to_string(&tmp)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", tmp.display(), e));
    assert!(
        body.starts_with("# "),
        "expected H1 in file, got: {:?}",
        &body[..body.len().min(80)]
    );

    let _ = std::fs::remove_file(&tmp);
}
