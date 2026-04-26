//! Agent-readiness checks — discovery of standards that make a site usable as
//! a resource by autonomous AI agents. Inspired by Cloudflare's
//! "Is it Agent Ready?" framework and intersecting standards like MCP, A2A,
//! llms.txt, OAuth, and emerging bot-auth / content-signals specifications.
//!
//! Every check lives under the `agent-` check-id prefix and contributes to the
//! "agent" category score. Missing emerging-standard endpoints return Warn
//! (half-credit) rather than Fail so that non-implementing sites are nudged,
//! not crushed.
//!
//! Checks implemented:
//! - agent-link-headers          — HTTP `Link` header inspection
//! - agent-markdown-negotiation  — `Accept: text/markdown` content negotiation
//! - agent-content-signals       — robots.txt `Content-Signal:` directive
//! - agent-web-bot-auth          — `.well-known/http-message-signatures-directory`
//! - agent-mcp-server-card       — `.well-known/mcp/*.json` variants
//! - agent-a2a-card              — `.well-known/agent-card.json`
//! - agent-skills                — `.well-known/agent-skills/index.json` or `.well-known/skills/index.json`
//! - agent-webmcp                — `navigator.modelContext` / WebMCP script detection
//! - agent-api-catalog           — `.well-known/api-catalog`
//! - agent-oauth-discovery       — `.well-known/oauth-authorization-server` or `.well-known/openid-configuration`
//! - agent-oauth-protected-resource — `.well-known/oauth-protected-resource`
//! - agent-x402                  — x402 payment protocol
//! - agent-mpp                   — MPP payment discovery via `/openapi.json`
//! - agent-ucp                   — `.well-known/ucp`
//! - agent-acp                   — `.well-known/acp.json`
//! - agent-ap2                   — AP2 via A2A Agent Card

use crate::crawling::{fetch_text_capped_with_builder, MAX_BODY_BYTES};
use crate::scoring::{AnalysisResult, Status};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, LINK};
use scraper::{Html, Selector};
use url::Url;

/// Collected site-wide resources needed by agent-category analyzers.
/// Fetched once per analysis run and passed to individual checks to avoid
/// redundant HTTP requests.
#[derive(Debug, Clone, Default)]
pub struct AgentResources {
    pub markdown_negotiation: Option<MarkdownNegotiation>,
    pub web_bot_auth_found: bool,
    pub mcp_server_card_url: Option<String>,
    pub a2a_agent_card_body: Option<String>,
    pub agent_skills_url: Option<String>,
    pub api_catalog_found: bool,
    pub oauth_discovery_url: Option<String>,
    pub oauth_protected_resource_found: bool,
    pub ucp_found: bool,
    pub acp_found: bool,
    pub openapi_body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarkdownNegotiation {
    pub response_content_type: String,
}

/// Fetch all site-wide agent resources concurrently. Returns AgentResources
/// with each field populated for hits and None/false for misses.
pub async fn fetch_agent_resources(
    client: &reqwest::Client,
    base_url: &Url,
) -> AgentResources {
    // Build absolute URLs for each well-known candidate
    let md_url = base_url.clone();
    let web_bot_auth_url = join_path(base_url, "/.well-known/http-message-signatures-directory");
    let mcp_variants = [
        "/.well-known/mcp/server-cards.json",
        "/.well-known/mcp/server-card.json",
        "/.well-known/mcp.json",
    ];
    let mcp_urls: Vec<Url> = mcp_variants
        .iter()
        .map(|p| join_path(base_url, p))
        .collect();
    let a2a_url = join_path(base_url, "/.well-known/agent-card.json");
    let skills_variants = [
        "/.well-known/agent-skills/index.json",
        "/.well-known/skills/index.json",
    ];
    let skills_urls: Vec<Url> = skills_variants
        .iter()
        .map(|p| join_path(base_url, p))
        .collect();
    let api_catalog_url = join_path(base_url, "/.well-known/api-catalog");
    let oauth_auth_url = join_path(base_url, "/.well-known/oauth-authorization-server");
    let oidc_url = join_path(base_url, "/.well-known/openid-configuration");
    let oauth_urls = [oauth_auth_url, oidc_url];
    let oauth_resource_url = join_path(base_url, "/.well-known/oauth-protected-resource");
    let ucp_url = join_path(base_url, "/.well-known/ucp");
    let acp_url = join_path(base_url, "/.well-known/acp.json");
    let openapi_url = join_path(base_url, "/openapi.json");

    // Launch all fetches in parallel
    let (
        md_nego,
        web_bot_auth_found,
        mcp_hit,
        a2a_body,
        skills_hit,
        api_catalog_found,
        oauth_disc_hit,
        oauth_res_found,
        ucp_found,
        acp_found,
        openapi_body,
    ) = tokio::join!(
        fetch_markdown_negotiation(client, &md_url),
        fetch_ok(client, &web_bot_auth_url, None),
        fetch_first_ok(client, &mcp_urls),
        fetch_body_if_ok(client, &a2a_url, None),
        fetch_first_ok(client, &skills_urls),
        fetch_ok(
            client,
            &api_catalog_url,
            Some(("Accept", "application/linkset+json, application/json")),
        ),
        fetch_first_ok(client, &oauth_urls),
        fetch_ok(client, &oauth_resource_url, None),
        fetch_ok(client, &ucp_url, None),
        fetch_ok(client, &acp_url, None),
        fetch_body_if_ok(client, &openapi_url, None),
    );

    AgentResources {
        markdown_negotiation: md_nego,
        web_bot_auth_found,
        mcp_server_card_url: mcp_hit,
        a2a_agent_card_body: a2a_body,
        agent_skills_url: skills_hit,
        api_catalog_found,
        oauth_discovery_url: oauth_disc_hit,
        oauth_protected_resource_found: oauth_res_found,
        ucp_found,
        acp_found,
        openapi_body,
    }
}

fn join_path(base: &Url, path: &str) -> Url {
    let mut u = base.clone();
    u.set_path(path);
    u.set_query(None);
    u.set_fragment(None);
    u
}

async fn fetch_ok(
    client: &reqwest::Client,
    url: &Url,
    header: Option<(&str, &str)>,
) -> bool {
    let mut req = client.get(url.as_str());
    if let Some((k, v)) = header {
        req = req.header(k, v);
    }
    match req.send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

async fn fetch_body_if_ok(
    client: &reqwest::Client,
    url: &Url,
    header: Option<(&str, &str)>,
) -> Option<String> {
    let mut req = client.get(url.as_str());
    if let Some((k, v)) = header {
        req = req.header(k, v);
    }
    // Body-capped variant so an unbounded /openapi.json or agent-card.json
    // (several competitor products publish multi-MB OpenAPI specs) cannot
    // push the worker into GB-scale string allocations.
    let (body, _) = fetch_text_capped_with_builder(req, MAX_BODY_BYTES).await;
    if body.is_empty() { None } else { Some(body) }
}

async fn fetch_first_ok(client: &reqwest::Client, urls: &[Url]) -> Option<String> {
    for u in urls {
        if fetch_ok(client, u, None).await {
            return Some(u.to_string());
        }
    }
    None
}

async fn fetch_markdown_negotiation(
    client: &reqwest::Client,
    url: &Url,
) -> Option<MarkdownNegotiation> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("text/markdown"));
    match client.get(url.as_str()).headers(headers).send().await {
        Ok(resp) => {
            let ct = resp
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            Some(MarkdownNegotiation {
                response_content_type: ct,
            })
        }
        Err(_) => None,
    }
}

// ── Individual checks ────────────────────────────────────────────────────────

/// `Link` HTTP header inspection. Sites that expose sitemap / canonical /
/// describedby via Link headers give agents a second discovery channel
/// beyond crawling the HTML.
pub fn analyze_link_headers(response_headers: &HeaderMap) -> AnalysisResult {
    let link_values: Vec<&HeaderValue> = response_headers.get_all(LINK).iter().collect();
    if link_values.is_empty() {
        return AnalysisResult {
            check: "agent-link-headers",
            status: Status::Fail,
            message: "No Link HTTP header present on the homepage response.".to_string(),
            recommendation:
                "Expose a Link header such as `</sitemap.xml>; rel=\"sitemap\"` or `</llms.txt>; rel=\"describedby\"` so agents discover canonical resources without parsing HTML."
                    .to_string(),
        };
    }
    let joined = link_values
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>()
        .join(", ");
    AnalysisResult {
        check: "agent-link-headers",
        status: Status::Pass,
        message: format!("Link header present: {}", truncate(&joined, 140)),
        recommendation: String::new(),
    }
}

/// Markdown content negotiation. Cloudflare's "Markdown for Agents"
/// convention — when an agent requests `Accept: text/markdown`, the origin
/// should return a markdown representation rather than HTML.
pub fn analyze_markdown_negotiation(res: &AgentResources) -> AnalysisResult {
    let neg = match &res.markdown_negotiation {
        Some(n) => n,
        None => {
            return AnalysisResult {
                check: "agent-markdown-negotiation",
                status: Status::Fail,
                message: "Homepage fetch with Accept: text/markdown failed.".to_string(),
                recommendation:
                    "Ensure the homepage responds successfully when an agent sends Accept: text/markdown and return a markdown representation."
                        .to_string(),
            };
        }
    };
    let ct = neg.response_content_type.to_ascii_lowercase();
    if ct.contains("text/markdown") {
        AnalysisResult {
            check: "agent-markdown-negotiation",
            status: Status::Pass,
            message: format!("Server returned {} for Accept: text/markdown.", neg.response_content_type),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-markdown-negotiation",
            status: Status::Fail,
            message: format!(
                "Server returned content-type `{}`, not text/markdown — no markdown representation for agents.",
                neg.response_content_type
            ),
            recommendation:
                "Implement content negotiation: when Accept includes text/markdown, respond with a markdown version of the page body. See https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/"
                    .to_string(),
        }
    }
}

/// robots.txt `Content-Signal:` directive. Cloudflare-backed proposal that
/// lets publishers express allow/deny policy per AI use-case
/// (search, ai-input, ai-train).
pub fn analyze_content_signals(robots_body: &str) -> AnalysisResult {
    let lower = robots_body.to_ascii_lowercase();
    if lower.contains("content-signal:") {
        AnalysisResult {
            check: "agent-content-signals",
            status: Status::Pass,
            message: "robots.txt contains Content-Signal directive.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-content-signals",
            status: Status::Fail,
            message: "No Content-Signal directive found in robots.txt.".to_string(),
            recommendation:
                "Add Cloudflare Content Signals (e.g. `Content-Signal: search=yes, ai-input=yes, ai-train=no`) to robots.txt to express granular AI usage preferences. See https://blog.cloudflare.com/content-signals/"
                    .to_string(),
        }
    }
}

/// Web Bot Auth directory. Cloudflare spec for cryptographic bot identity
/// verification. Informational — missing is not a defect for content sites.
pub fn analyze_web_bot_auth(res: &AgentResources) -> AnalysisResult {
    if res.web_bot_auth_found {
        AnalysisResult {
            check: "agent-web-bot-auth",
            status: Status::Pass,
            message: "/.well-known/http-message-signatures-directory is served.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-web-bot-auth",
            status: Status::Warn,
            message:
                "Web Bot Auth directory not found (informational — mostly relevant to bot operators)."
                    .to_string(),
            recommendation:
                "If you operate or host AI agents, publish an HTTP Message Signatures directory at /.well-known/http-message-signatures-directory so origins can verify crypto-signed requests. See https://blog.cloudflare.com/web-bot-auth/"
                    .to_string(),
        }
    }
}

/// MCP Server Card. Advertises MCP servers hosted by the origin.
pub fn analyze_mcp_server_card(res: &AgentResources) -> AnalysisResult {
    match &res.mcp_server_card_url {
        Some(url) => AnalysisResult {
            check: "agent-mcp-server-card",
            status: Status::Pass,
            message: format!("MCP Server Card served at {}.", url),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "agent-mcp-server-card",
            status: Status::Fail,
            message: "No MCP Server Card at /.well-known/mcp/server-card(s).json or /.well-known/mcp.json.".to_string(),
            recommendation:
                "If your site exposes MCP servers, publish a Server Card at /.well-known/mcp/server-cards.json listing them so AI assistants can discover and connect. See https://modelcontextprotocol.io"
                    .to_string(),
        },
    }
}

/// A2A Agent Card. Google's Agent2Agent protocol — `.well-known/agent-card.json`.
pub fn analyze_a2a_card(res: &AgentResources) -> AnalysisResult {
    match &res.a2a_agent_card_body {
        Some(_body) => AnalysisResult {
            check: "agent-a2a-card",
            status: Status::Pass,
            message: "A2A Agent Card served at /.well-known/agent-card.json.".to_string(),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "agent-a2a-card",
            status: Status::Fail,
            message: "No A2A Agent Card at /.well-known/agent-card.json.".to_string(),
            recommendation:
                "If your site hosts an A2A-compatible agent, publish an Agent Card at /.well-known/agent-card.json describing its capabilities and endpoints."
                    .to_string(),
        },
    }
}

/// Agent Skills index — agentskills.io spec.
pub fn analyze_agent_skills(res: &AgentResources) -> AnalysisResult {
    match &res.agent_skills_url {
        Some(url) => AnalysisResult {
            check: "agent-skills",
            status: Status::Pass,
            message: format!("Agent Skills index served at {}.", url),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "agent-skills",
            status: Status::Fail,
            message:
                "No Agent Skills index at /.well-known/agent-skills/index.json or /.well-known/skills/index.json."
                    .to_string(),
            recommendation:
                "Publish an Agent Skills catalog describing the structured capabilities your site exposes to agents. See https://agentskills.io"
                    .to_string(),
        },
    }
}

/// WebMCP — in-page tool registration via `navigator.modelContext` or
/// a `<script type=\"webmcp\">` tag.
pub fn analyze_webmcp(html_doc: &Html) -> AnalysisResult {
    // Per-script walk instead of serialising the entire DOM to a string:
    // `Html::root_element().html()` re-materialises every byte of the page
    // in a fresh allocation, which for a 2–5MB JS-heavy site is 2–5MB of
    // transient RAM per analyzer call (and this analyzer plus x402 both
    // did it — compounding under concurrent load).
    let script_sel = Selector::parse("script").expect("valid");
    let mut has_webmcp_type = false;
    let mut has_nav_mc = false;
    let mut has_register_tool = false;

    for el in html_doc.select(&script_sel) {
        if let Some(t) = el.value().attr("type") {
            if t.eq_ignore_ascii_case("webmcp") {
                has_webmcp_type = true;
            }
        }
        // Only scan the text nodes inside the script — we don't need to
        // serialise attributes or child elements.
        for txt in el.text() {
            let lower = txt.to_ascii_lowercase();
            if !has_nav_mc && lower.contains("navigator.modelcontext") {
                has_nav_mc = true;
            }
            if !has_register_tool
                && lower.contains("registertool")
                && lower.contains("modelcontext")
            {
                has_register_tool = true;
            }
            if has_nav_mc && has_register_tool {
                break;
            }
        }
        if has_webmcp_type && has_nav_mc && has_register_tool {
            break;
        }
    }

    if has_webmcp_type || has_nav_mc || has_register_tool {
        AnalysisResult {
            check: "agent-webmcp",
            status: Status::Pass,
            message: "Page exposes WebMCP tool registrations.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-webmcp",
            status: Status::Fail,
            message: "No WebMCP tools detected in page markup or scripts.".to_string(),
            recommendation:
                "Expose in-page agent tools via WebMCP: register them through navigator.modelContext or a <script type=\"webmcp\"> tag. See https://webmcp.org"
                    .to_string(),
        }
    }
}

/// `.well-known/api-catalog` (IETF draft) — machine-readable catalogue of APIs.
pub fn analyze_api_catalog(res: &AgentResources) -> AnalysisResult {
    if res.api_catalog_found {
        AnalysisResult {
            check: "agent-api-catalog",
            status: Status::Pass,
            message: "/.well-known/api-catalog is served.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-api-catalog",
            status: Status::Fail,
            message: "No API Catalog at /.well-known/api-catalog.".to_string(),
            recommendation:
                "If your site exposes APIs, publish a linkset JSON catalogue at /.well-known/api-catalog so agents can discover endpoints. See the IETF httpapi-api-catalog draft."
                    .to_string(),
        }
    }
}

/// OAuth / OIDC discovery — either `oauth-authorization-server` (RFC 8414)
/// or `openid-configuration`.
pub fn analyze_oauth_discovery(res: &AgentResources) -> AnalysisResult {
    match &res.oauth_discovery_url {
        Some(url) => AnalysisResult {
            check: "agent-oauth-discovery",
            status: Status::Pass,
            message: format!("OAuth/OIDC discovery metadata served at {}.", url),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "agent-oauth-discovery",
            status: Status::Fail,
            message:
                "No OAuth/OIDC discovery at /.well-known/oauth-authorization-server or /.well-known/openid-configuration."
                    .to_string(),
            recommendation:
                "If your site fronts an OAuth authorization server, publish RFC 8414 metadata at /.well-known/oauth-authorization-server. Agents use this for authenticated API access."
                    .to_string(),
        },
    }
}

/// OAuth Protected Resource Metadata (RFC 9728).
pub fn analyze_oauth_protected_resource(res: &AgentResources) -> AnalysisResult {
    if res.oauth_protected_resource_found {
        AnalysisResult {
            check: "agent-oauth-protected-resource",
            status: Status::Pass,
            message: "/.well-known/oauth-protected-resource is served.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-oauth-protected-resource",
            status: Status::Fail,
            message: "No OAuth Protected Resource Metadata at /.well-known/oauth-protected-resource.".to_string(),
            recommendation:
                "If your site is an OAuth-protected resource, publish RFC 9728 metadata at /.well-known/oauth-protected-resource so agents know how to authenticate."
                    .to_string(),
        }
    }
}

/// x402 payment protocol — HTTP 402 Payment Required with a discovery payload.
pub async fn analyze_x402(
    client: &reqwest::Client,
    base_url: &Url,
    html_doc: &Html,
) -> AnalysisResult {
    let candidates = [
        base_url.clone(),
        join_path(base_url, "/api"),
        join_path(base_url, "/api/v1"),
    ];
    for u in &candidates {
        if let Ok(resp) = client.get(u.as_str()).send().await {
            let status = resp.status().as_u16();
            if status == 402 {
                return AnalysisResult {
                    check: "agent-x402",
                    status: Status::Pass,
                    message: format!("x402 payment challenge detected at {}.", u),
                    recommendation: String::new(),
                };
            }
        }
    }
    // Sniff HTML for x402 markers without serialising the whole DOM. We only
    // care about meta tags, link tags, and inline script text — a full
    // root_element().html() walk reallocates the entire document for a
    // couple of substring checks, which is a meaningful memory spike on
    // JS-heavy sites.
    let mut saw_x402 = false;
    let mut saw_payment = false;

    let meta_sel = Selector::parse("meta").expect("valid");
    for el in html_doc.select(&meta_sel) {
        for attr in ["name", "property", "content"].iter() {
            if let Some(v) = el.value().attr(attr) {
                let lower = v.to_ascii_lowercase();
                if !saw_x402 && lower.contains("x402") { saw_x402 = true; }
                if !saw_payment && lower.contains("payment") { saw_payment = true; }
                if saw_x402 && saw_payment { break; }
            }
        }
        if saw_x402 && saw_payment { break; }
    }

    if !(saw_x402 && saw_payment) {
        let link_sel = Selector::parse("link[rel], link[href]").expect("valid");
        for el in html_doc.select(&link_sel) {
            for attr in ["rel", "href"].iter() {
                if let Some(v) = el.value().attr(attr) {
                    let lower = v.to_ascii_lowercase();
                    if !saw_x402 && lower.contains("x402") { saw_x402 = true; }
                    if !saw_payment && lower.contains("payment") { saw_payment = true; }
                }
            }
            if saw_x402 && saw_payment { break; }
        }
    }

    if !(saw_x402 && saw_payment) {
        let script_sel = Selector::parse("script").expect("valid");
        'outer: for el in html_doc.select(&script_sel) {
            for txt in el.text() {
                let lower = txt.to_ascii_lowercase();
                if !saw_x402 && lower.contains("x402") { saw_x402 = true; }
                if !saw_payment && lower.contains("payment") { saw_payment = true; }
                if saw_x402 && saw_payment { break 'outer; }
            }
        }
    }

    if saw_x402 && saw_payment {
        return AnalysisResult {
            check: "agent-x402",
            status: Status::Pass,
            message: "x402 payment advertisement detected in page content.".to_string(),
            recommendation: String::new(),
        };
    }
    AnalysisResult {
        check: "agent-x402",
        status: Status::Warn,
        message: "x402 payment protocol not detected (neutral for non-commerce sites).".to_string(),
        recommendation:
            "If you monetize API access, implement x402: return HTTP 402 with a payment discovery payload so agents can pay and retry. See https://www.x402.org"
                .to_string(),
    }
}

/// MPP — Micropayment Protocol. Detection via `/openapi.json` carrying MPP
/// extensions or a dedicated MPP payment discovery document.
pub fn analyze_mpp(res: &AgentResources) -> AnalysisResult {
    if let Some(body) = &res.openapi_body {
        let lower = body.to_ascii_lowercase();
        if lower.contains("x-mpp") || lower.contains("mpp-payment") || lower.contains("micropayment") {
            return AnalysisResult {
                check: "agent-mpp",
                status: Status::Pass,
                message: "MPP payment metadata found in /openapi.json.".to_string(),
                recommendation: String::new(),
            };
        }
    }
    AnalysisResult {
        check: "agent-mpp",
        status: Status::Warn,
        message: "MPP payment discovery not detected (neutral for non-commerce sites).".to_string(),
        recommendation:
            "To accept agentic micropayments, publish MPP metadata in /openapi.json or a dedicated discovery document. See https://mpp.dev"
                .to_string(),
    }
}

/// UCP — Universal Commerce Protocol profile at `/.well-known/ucp`.
pub fn analyze_ucp(res: &AgentResources) -> AnalysisResult {
    if res.ucp_found {
        AnalysisResult {
            check: "agent-ucp",
            status: Status::Pass,
            message: "/.well-known/ucp profile is served.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-ucp",
            status: Status::Warn,
            message: "UCP profile not found (neutral for non-commerce sites).".to_string(),
            recommendation:
                "If you sell products/services to agents, publish a UCP profile at /.well-known/ucp. See https://ucp.dev"
                    .to_string(),
        }
    }
}

/// ACP — Agentic Commerce Protocol discovery document at `/.well-known/acp.json`.
pub fn analyze_acp(res: &AgentResources) -> AnalysisResult {
    if res.acp_found {
        AnalysisResult {
            check: "agent-acp",
            status: Status::Pass,
            message: "/.well-known/acp.json is served.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-acp",
            status: Status::Warn,
            message: "ACP discovery document not found (neutral for non-commerce sites).".to_string(),
            recommendation:
                "To accept agentic commerce transactions, publish an ACP discovery document at /.well-known/acp.json. See https://agenticcommerce.dev"
                    .to_string(),
        }
    }
}

/// AP2 — Agentic Payment Protocol. Relies on presence of an A2A Agent Card
/// with payment capability metadata.
pub fn analyze_ap2(res: &AgentResources) -> AnalysisResult {
    let body = match &res.a2a_agent_card_body {
        Some(b) => b,
        None => {
            return AnalysisResult {
                check: "agent-ap2",
                status: Status::Warn,
                message: "AP2 not detected — requires an A2A Agent Card.".to_string(),
                recommendation:
                    "AP2 builds on A2A. If you accept agentic payments, first publish an A2A Agent Card, then add AP2 payment capability metadata to it."
                        .to_string(),
            };
        }
    };
    let lower = body.to_ascii_lowercase();
    if lower.contains("ap2") || (lower.contains("payment") && lower.contains("capabilities")) {
        AnalysisResult {
            check: "agent-ap2",
            status: Status::Pass,
            message: "AP2 payment capability declared in A2A Agent Card.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "agent-ap2",
            status: Status::Warn,
            message: "A2A Agent Card present but no AP2 payment capability declared.".to_string(),
            recommendation:
                "Extend your A2A Agent Card with AP2 payment capability metadata to accept agentic payments."
                    .to_string(),
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderName, HeaderValue};

    fn mk_headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    #[test]
    fn link_headers_missing_is_fail() {
        let r = analyze_link_headers(&HeaderMap::new());
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.check, "agent-link-headers");
    }

    #[test]
    fn link_headers_present_is_pass() {
        let h = mk_headers(&[("Link", "</sitemap.xml>; rel=\"sitemap\"")]);
        let r = analyze_link_headers(&h);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn markdown_negotiation_html_is_fail() {
        let res = AgentResources {
            markdown_negotiation: Some(MarkdownNegotiation {
                response_content_type: "text/html; charset=utf-8".into(),
            }),
            ..Default::default()
        };
        let r = analyze_markdown_negotiation(&res);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn markdown_negotiation_markdown_is_pass() {
        let res = AgentResources {
            markdown_negotiation: Some(MarkdownNegotiation {
                response_content_type: "text/markdown".into(),
            }),
            ..Default::default()
        };
        let r = analyze_markdown_negotiation(&res);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn content_signals_absent_is_fail() {
        let r = analyze_content_signals("User-agent: *\nAllow: /\n");
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn content_signals_present_is_pass() {
        let body = "User-agent: *\nAllow: /\nContent-Signal: search=yes, ai-train=no\n";
        let r = analyze_content_signals(body);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn web_bot_auth_missing_is_warn() {
        let res = AgentResources::default();
        let r = analyze_web_bot_auth(&res);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn web_bot_auth_found_is_pass() {
        let res = AgentResources {
            web_bot_auth_found: true,
            ..Default::default()
        };
        let r = analyze_web_bot_auth(&res);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn mcp_server_card_missing_is_fail() {
        let r = analyze_mcp_server_card(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn mcp_server_card_found_is_pass() {
        let res = AgentResources {
            mcp_server_card_url: Some("https://x/.well-known/mcp.json".into()),
            ..Default::default()
        };
        let r = analyze_mcp_server_card(&res);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn a2a_card_missing_is_fail() {
        let r = analyze_a2a_card(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn agent_skills_missing_is_fail() {
        let r = analyze_agent_skills(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn webmcp_detection_on_nav_modelcontext() {
        let html = Html::parse_document(
            "<html><body><script>navigator.modelContext.registerTool({})</script></body></html>",
        );
        let r = analyze_webmcp(&html);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn webmcp_missing_is_fail() {
        let html = Html::parse_document("<html><body>hi</body></html>");
        let r = analyze_webmcp(&html);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn api_catalog_missing_is_fail() {
        let r = analyze_api_catalog(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn oauth_discovery_missing_is_fail() {
        let r = analyze_oauth_discovery(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn oauth_discovery_found_is_pass() {
        let res = AgentResources {
            oauth_discovery_url: Some("https://x/.well-known/oauth-authorization-server".into()),
            ..Default::default()
        };
        let r = analyze_oauth_discovery(&res);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn oauth_protected_resource_missing_is_fail() {
        let r = analyze_oauth_protected_resource(&AgentResources::default());
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn mpp_no_openapi_is_warn() {
        let r = analyze_mpp(&AgentResources::default());
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn mpp_with_marker_is_pass() {
        let res = AgentResources {
            openapi_body: Some(r#"{"x-mpp":{"price":1}}"#.into()),
            ..Default::default()
        };
        let r = analyze_mpp(&res);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn ucp_missing_is_warn() {
        let r = analyze_ucp(&AgentResources::default());
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn acp_missing_is_warn() {
        let r = analyze_acp(&AgentResources::default());
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn ap2_without_a2a_is_warn() {
        let r = analyze_ap2(&AgentResources::default());
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("requires an A2A Agent Card"));
    }

    #[test]
    fn ap2_with_a2a_missing_payment_is_warn() {
        let res = AgentResources {
            a2a_agent_card_body: Some(r#"{"name":"x"}"#.into()),
            ..Default::default()
        };
        let r = analyze_ap2(&res);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn ap2_with_payment_capability_is_pass() {
        let res = AgentResources {
            a2a_agent_card_body: Some(
                r#"{"name":"x","capabilities":["payment"]}"#.into(),
            ),
            ..Default::default()
        };
        let r = analyze_ap2(&res);
        assert_eq!(r.status, Status::Pass);
    }
}
