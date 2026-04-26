//! `geodaddy see <url>` — show what an AI crawler (non-JS) sees.
//!
//! Fetches the page with a plain HTTP client (what GPTBot/ClaudeBot/PerplexityBot
//! get) and renders it as readable markdown. When `--enable-js` is passed, also
//! fetches via headless Chromium and produces a diff of content hidden from AI
//! crawlers.

use anyhow::Result;
use chromiumoxide::Browser;
use colored::Colorize;
use ego_tree::NodeRef;
use scraper::{Html, Node, Selector};
use serde::Serialize;
use std::collections::HashSet;
use url::Url;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Serialize, Debug)]
pub struct SeeReport {
    pub url: String,
    pub fetched_at: String,
    pub mode: Mode,
    pub plain: View,
    pub rendered: Option<View>,
    pub diff: Option<Diff>,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Plain,
    Diff,
}

#[derive(Serialize, Debug)]
pub struct View {
    pub title: Option<String>,
    pub meta_description: Option<String>,
    /// Full markdown body — retained in memory for diff computation and for
    /// the `--beauty` renderer, but intentionally excluded from JSON output
    /// (callers wanting the text should use `--beauty` and redirect stdout).
    #[serde(skip_serializing)]
    pub markdown: String,
    pub stats: Stats,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Stats {
    pub word_count: usize,
    pub heading_count: usize,
    pub h1_count: usize,
    pub h2_count: usize,
    pub h3_plus_count: usize,
    pub link_count: usize,
    pub image_count: usize,
    pub images_without_alt: usize,
}

#[derive(Serialize, Debug)]
pub struct Diff {
    pub hidden_word_count: i64,
    pub hidden_word_percentage: f64,
    pub hidden_headings: Vec<String>,
    pub hidden_link_count: i64,
    pub hidden_image_count: i64,
}

// ── Entry points ─────────────────────────────────────────────────────────────

/// Build a `SeeReport` for the URL. Library-facing: performs HTTP (and
/// optionally CDP) I/O but does not touch stdout. Backends, tests, and
/// programmatic callers should use this.
///
/// If `browser` is `Some`, a JS-rendered fetch is performed in addition to
/// the plain HTTP fetch and the diff is computed. If the JS fetch fails,
/// the report falls back to plain-only mode (no error bubbled up) — this
/// matches the CLI's behaviour and keeps the endpoint robust against
/// flaky headless-browser runs.
pub async fn produce_report(
    url: &str,
    client: &reqwest::Client,
    browser: Option<&Browser>,
) -> Result<SeeReport> {
    let parsed = Url::parse(url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", url, e))?;

    // Plain fetch (what AI crawlers see)
    let plain_html = fetch_plain(client, &parsed).await?;
    let plain_doc = Html::parse_document(&plain_html);
    let plain_view = build_view(&plain_doc);

    // Optional JS-rendered fetch for diff
    let (mode, rendered, diff) = if let Some(b) = browser {
        match fetch_rendered(b, parsed.as_str()).await {
            Ok(rendered_html) => {
                let rendered_doc = Html::parse_document(&rendered_html);
                let rendered_view = build_view(&rendered_doc);
                let d = compute_diff(&plain_view, &rendered_view);
                (Mode::Diff, Some(rendered_view), Some(d))
            }
            Err(e) => {
                tracing::warn!("JS-rendered fetch failed: {} — falling back to plain-only", e);
                (Mode::Plain, None, None)
            }
        }
    } else {
        (Mode::Plain, None, None)
    };

    Ok(SeeReport {
        url: url.to_string(),
        fetched_at: chrono::Utc::now().to_rfc3339(),
        mode,
        plain: plain_view,
        rendered,
        diff,
    })
}

/// CLI entry for the `see` subcommand — builds the report and renders it.
///
/// Output format matches the project-wide convention:
///   - `beauty = false` (default) → JSON
///   - `beauty = true`            → human-readable markdown
pub async fn run_see(
    url: &str,
    beauty: bool,
    client: &reqwest::Client,
    browser: Option<&Browser>,
) -> Result<()> {
    let report = produce_report(url, client, browser).await?;

    if beauty {
        print_text_report(&report);
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

// ── Fetchers ─────────────────────────────────────────────────────────────────

async fn fetch_plain(client: &reqwest::Client, url: &Url) -> Result<String> {
    use crate::crawling::{fetch_text_capped, MAX_BODY_BYTES};
    let (body, _) = fetch_text_capped(client, url.as_str(), MAX_BODY_BYTES).await;
    Ok(body)
}

async fn fetch_rendered(browser: &Browser, url: &str) -> Result<String> {
    use crate::crawling::{truncate_utf8, MAX_BODY_BYTES};
    // Hard upper bound so a hung page cannot hold a blocking worker forever
    // and a runaway DOM cannot push a multi-GB string through CDP.
    let fetch = async {
        let page = browser.new_page(url).await?;
        let _ = page.wait_for_navigation().await;
        let content = page.content().await?;
        if let Err(e) = page.close().await {
            tracing::warn!("Failed to close JS page: {}", e);
        }
        Ok::<_, chromiumoxide::error::CdpError>(content)
    };
    let content = tokio::time::timeout(std::time::Duration::from_secs(30), fetch)
        .await
        .map_err(|_| anyhow::anyhow!("Headless fetch timed out"))?
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(truncate_utf8(&content, MAX_BODY_BYTES).to_string())
}

// ── View builder ─────────────────────────────────────────────────────────────

fn build_view(doc: &Html) -> View {
    let title = extract_title(doc);
    let meta_description = extract_meta_description(doc);
    let markdown = render_markdown(doc);
    let stats = compute_stats(doc, &markdown);

    View {
        title,
        meta_description,
        markdown,
        stats,
    }
}

fn extract_title(doc: &Html) -> Option<String> {
    let sel = Selector::parse("title").ok()?;
    doc.select(&sel)
        .next()
        .map(|el| collapse_ws(&el.text().collect::<String>()))
        .filter(|s| !s.is_empty())
}

fn extract_meta_description(doc: &Html) -> Option<String> {
    let sel = Selector::parse(r#"meta[name="description"]"#).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
        .map(|s| collapse_ws(&s))
        .filter(|s| !s.is_empty())
}

fn compute_stats(doc: &Html, markdown: &str) -> Stats {
    let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let mut h1 = 0;
    let mut h2 = 0;
    let mut h3p = 0;
    for el in doc.select(&heading_sel) {
        match el.value().name() {
            "h1" => h1 += 1,
            "h2" => h2 += 1,
            _ => h3p += 1,
        }
    }
    let heading_count = h1 + h2 + h3p;

    let link_sel = Selector::parse("a[href]").unwrap();
    let link_count = doc.select(&link_sel).count();

    let img_sel = Selector::parse("img").unwrap();
    let mut image_count = 0;
    let mut images_without_alt = 0;
    for el in doc.select(&img_sel) {
        image_count += 1;
        let alt = el.value().attr("alt").unwrap_or("").trim();
        if alt.is_empty() {
            images_without_alt += 1;
        }
    }

    let word_count = markdown
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphanumeric()))
        .count();

    Stats {
        word_count,
        heading_count,
        h1_count: h1,
        h2_count: h2,
        h3_plus_count: h3p,
        link_count,
        image_count,
        images_without_alt,
    }
}

// ── HTML → Markdown walker ───────────────────────────────────────────────────

fn render_markdown(doc: &Html) -> String {
    let body_sel = Selector::parse("body").unwrap();
    let root = doc.select(&body_sel).next();

    let mut out = String::new();
    match root {
        Some(body) => walk(*body, &mut out, false),
        None => walk(doc.tree.root(), &mut out, false),
    }
    cleanup_markdown(out)
}

fn walk(node: NodeRef<Node>, out: &mut String, in_pre: bool) {
    match node.value() {
        Node::Text(t) => {
            if in_pre {
                out.push_str(&t.text);
            } else {
                out.push_str(&t.text);
            }
        }
        Node::Element(el) => {
            let name = el.name();
            match name {
                // Hard skips — no content emitted, no recursion
                "script" | "style" | "noscript" | "template" | "svg" | "iframe"
                | "object" | "embed" | "canvas" | "audio" | "video" | "picture"
                | "source" | "track" | "map" | "area" | "head" => {}

                // Line break
                "br" => out.push('\n'),
                "hr" => out.push_str("\n\n---\n\n"),

                // Headings
                "h1" => heading(node, out, 1),
                "h2" => heading(node, out, 2),
                "h3" => heading(node, out, 3),
                "h4" => heading(node, out, 4),
                "h5" => heading(node, out, 5),
                "h6" => heading(node, out, 6),

                // Paragraph-like blocks
                "p" | "section" | "article" | "main" | "aside" | "header"
                | "footer" | "figure" | "figcaption" | "details" | "summary"
                | "dialog" | "address" => block(node, out, in_pre),

                // Nav is particularly important to include — it reveals the
                // site's information architecture as the LLM sees it.
                "nav" => block(node, out, in_pre),

                // Generic containers — no forced block boundary, just walk
                "div" | "span" => {
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                }

                // Lists
                "ul" | "ol" => {
                    ensure_blank_line(out);
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    ensure_blank_line(out);
                }
                "li" => {
                    ensure_line_start(out);
                    out.push_str("- ");
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    out.push('\n');
                }
                "dl" => {
                    ensure_blank_line(out);
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    ensure_blank_line(out);
                }
                "dt" => {
                    ensure_line_start(out);
                    out.push_str("**");
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    out.push_str("**\n");
                }
                "dd" => {
                    ensure_line_start(out);
                    out.push_str("  ");
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    out.push('\n');
                }

                // Inline formatting
                "strong" | "b" => wrap_inline(node, out, "**", "**", in_pre),
                "em" | "i" | "cite" | "var" => wrap_inline(node, out, "*", "*", in_pre),
                "code" => wrap_inline(node, out, "`", "`", true),
                "mark" => wrap_inline(node, out, "==", "==", in_pre),
                "del" | "s" | "strike" => wrap_inline(node, out, "~~", "~~", in_pre),

                // Pre-formatted
                "pre" => {
                    ensure_blank_line(out);
                    out.push_str("```\n");
                    for c in node.children() {
                        walk(c, out, true);
                    }
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("```");
                    ensure_blank_line(out);
                }

                // Blockquote
                "blockquote" => {
                    ensure_blank_line(out);
                    let mut inner = String::new();
                    for c in node.children() {
                        walk(c, &mut inner, in_pre);
                    }
                    let inner = cleanup_markdown(inner);
                    for line in inner.lines() {
                        out.push_str("> ");
                        out.push_str(line);
                        out.push('\n');
                    }
                    ensure_blank_line(out);
                }

                // Anchor — collect inner text, emit as markdown link
                "a" => {
                    let href = el.attr("href").unwrap_or("").trim().to_string();
                    let mut inner = String::new();
                    for c in node.children() {
                        walk(c, &mut inner, in_pre);
                    }
                    let text = collapse_ws(&inner);
                    match (text.as_str(), href.as_str()) {
                        ("", "") => {}
                        ("", h) => out.push_str(&format!("[{}]({})", h, h)),
                        (t, "") => out.push_str(t),
                        (t, h) => out.push_str(&format!("[{}]({})", t, h)),
                    }
                }

                // Image
                "img" => {
                    let src = el.attr("src").unwrap_or("").trim();
                    let alt = el.attr("alt").unwrap_or("").trim();
                    if !src.is_empty() {
                        out.push_str(&format!("![{}]({})", alt, src));
                    }
                }

                // Tables — render as lightweight markdown pipe tables
                "table" => {
                    ensure_blank_line(out);
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    ensure_blank_line(out);
                }
                "thead" | "tbody" | "tfoot" | "colgroup" | "col" | "caption" => {
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                }
                "tr" => {
                    ensure_line_start(out);
                    out.push_str("| ");
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    out.push('\n');
                }
                "th" | "td" => {
                    let mut inner = String::new();
                    for c in node.children() {
                        walk(c, &mut inner, in_pre);
                    }
                    let text = collapse_ws(&inner);
                    out.push_str(&text);
                    out.push_str(" | ");
                }

                // Form elements — emit placeholder/value text only if useful
                "button" => {
                    let mut inner = String::new();
                    for c in node.children() {
                        walk(c, &mut inner, in_pre);
                    }
                    let t = collapse_ws(&inner);
                    if !t.is_empty() {
                        out.push_str(&format!("[{}]", t));
                    }
                }
                "label" => {
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                    out.push(' ');
                }
                "input" | "textarea" | "select" | "option" | "form" | "fieldset" | "legend" => {
                    // Skip form controls entirely — their placeholders aren't content
                }

                // Default — recurse
                _ => {
                    for c in node.children() {
                        walk(c, out, in_pre);
                    }
                }
            }
        }
        _ => {
            for c in node.children() {
                walk(c, out, in_pre);
            }
        }
    }
}

fn heading(node: NodeRef<Node>, out: &mut String, level: u8) {
    ensure_blank_line(out);
    out.push_str(&"#".repeat(level as usize));
    out.push(' ');
    for c in node.children() {
        walk(c, out, false);
    }
    ensure_blank_line(out);
}

fn block(node: NodeRef<Node>, out: &mut String, in_pre: bool) {
    ensure_blank_line(out);
    for c in node.children() {
        walk(c, out, in_pre);
    }
    ensure_blank_line(out);
}

fn wrap_inline(node: NodeRef<Node>, out: &mut String, open: &str, close: &str, in_pre: bool) {
    let mut inner = String::new();
    for c in node.children() {
        walk(c, &mut inner, in_pre);
    }
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return;
    }
    out.push_str(open);
    out.push_str(trimmed);
    out.push_str(close);
}

fn ensure_blank_line(out: &mut String) {
    if out.is_empty() {
        return;
    }
    // Guarantee two trailing newlines (= one blank line separator)
    if out.ends_with("\n\n") {
        return;
    }
    if out.ends_with('\n') {
        out.push('\n');
    } else {
        out.push_str("\n\n");
    }
}

fn ensure_line_start(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

// ── Text utils ───────────────────────────────────────────────────────────────

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cleanup_markdown(input: String) -> String {
    // Collapse runs of whitespace inside each line, preserve blank-line structure
    let mut out = String::with_capacity(input.len());
    let mut blank_streak = 0usize;

    for line in input.split('\n') {
        let trimmed = line.trim_end();
        // For content lines, collapse internal runs of spaces/tabs but keep
        // leading markers (# prefixes, list "- ", blockquote "> ", etc.)
        let cleaned = collapse_inline_ws(trimmed);

        if cleaned.trim().is_empty() {
            blank_streak += 1;
            if blank_streak <= 1 {
                out.push('\n');
            }
        } else {
            blank_streak = 0;
            out.push_str(&cleaned);
            out.push('\n');
        }
    }

    out.trim().to_string()
}

fn collapse_inline_ws(line: &str) -> String {
    // Preserve leading whitespace (for nested list indentation) but collapse
    // runs of spaces/tabs inside the line to a single space.
    let leading: String = line
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let body = &line[leading.len()..];
    let body_collapsed = body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    format!("{}{}", leading, body_collapsed)
}

// ── Diff ─────────────────────────────────────────────────────────────────────

fn compute_diff(plain: &View, rendered: &View) -> Diff {
    let hidden_words =
        rendered.stats.word_count as i64 - plain.stats.word_count as i64;
    let hidden_percentage = if rendered.stats.word_count > 0 {
        (hidden_words as f64 / rendered.stats.word_count as f64) * 100.0
    } else {
        0.0
    };

    let plain_headings = extract_heading_texts(&plain.markdown);
    let rendered_headings = extract_heading_texts(&rendered.markdown);
    let plain_set: HashSet<_> = plain_headings.iter().cloned().collect();
    let hidden_headings: Vec<String> = rendered_headings
        .into_iter()
        .filter(|h| !plain_set.contains(h))
        .collect();

    Diff {
        hidden_word_count: hidden_words,
        hidden_word_percentage: (hidden_percentage * 10.0).round() / 10.0,
        hidden_headings,
        hidden_link_count: rendered.stats.link_count as i64 - plain.stats.link_count as i64,
        hidden_image_count: rendered.stats.image_count as i64 - plain.stats.image_count as i64,
    }
}

fn extract_heading_texts(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim_start();
            if !trimmed.starts_with('#') {
                return None;
            }
            let rest = trimmed.trim_start_matches('#').trim();
            if rest.is_empty() {
                None
            } else {
                Some(rest.to_string())
            }
        })
        .collect()
}

// ── Text (human-readable) rendering ─────────────────────────────────────────
//
// Minimal on purpose: users want "what the LLM sees", not a stats dashboard.
// The full structured data is available via `--json`.

fn print_text_report(r: &SeeReport) {
    // Diff summary goes to stderr (if any content is hidden), so stdout stays
    // pure markdown and `geodaddy see ... --beauty > page.md` produces a clean
    // file.
    if let (Mode::Diff, Some(diff)) = (r.mode, &r.diff) {
        if let Some(line) = diff_warning_line(diff) {
            eprintln!("{}", line.yellow());
        }
    }

    if r.plain.markdown.trim().is_empty() {
        eprintln!(
            "{}",
            "(no text content — site is likely JS-rendered; try --enable-js)".dimmed()
        );
    } else {
        println!("{}", r.plain.markdown);
    }
}

fn diff_warning_line(diff: &Diff) -> Option<String> {
    if diff.hidden_word_count <= 0 && diff.hidden_headings.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    if diff.hidden_word_count > 0 {
        parts.push(format!(
            "{} words hidden ({:.0}%)",
            diff.hidden_word_count, diff.hidden_word_percentage
        ));
    }

    if !diff.hidden_headings.is_empty() {
        let preview: Vec<&str> = diff
            .hidden_headings
            .iter()
            .take(3)
            .map(String::as_str)
            .collect();
        let extra = diff.hidden_headings.len().saturating_sub(preview.len());
        let mut headings = preview.join(", ");
        if extra > 0 {
            headings.push_str(&format!(" (+{} more)", extra));
        }
        parts.push(format!("missing headings: {}", headings));
    }

    Some(format!("⚠ AI crawlers miss: {}", parts.join(" · ")))
}
