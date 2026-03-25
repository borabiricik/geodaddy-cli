# Phase 6: Add local MCP server for LLM-driven CLI interaction - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a local MCP (Model Context Protocol) server that exposes geodaddy CLI capabilities as tools for LLM clients (Claude Desktop, Claude Code, Cursor, etc.). The server runs as a stdio process, calls geodaddy as a subprocess, and returns raw JSON results via MCP protocol.

</domain>

<decisions>
## Implementation Decisions

### Transport & Language
- **D-01:** MCP server written in TypeScript using the official `@modelcontextprotocol/sdk`
- **D-02:** stdio transport only (standard for local MCP servers, natively supported by Claude Desktop, Claude Code, Cursor)
- **D-03:** geodaddy binary bundled with the MCP server package (self-contained distribution)

### Tool Design
- **D-04:** Single `analyze_url` MCP tool that mirrors the CLI interface
- **D-05:** All CLI flags exposed as tool parameters: `url` (required), `max_pages`, `enable_js`, `vitals`, `fail_under`, `beauty` — 1:1 mapping with CLI flags
- **D-06:** LLM decides which parameters to pass based on user's request context

### Output Handling
- **D-07:** Raw JSON output from geodaddy passed directly as MCP tool result content — no transformation or summarization
- **D-08:** Errors return MCP error response (`isError: true`) with geodaddy's stderr message — LLM sees it as tool error and can retry or inform user

### Distribution
- **D-09:** MCP server code lives in `cli/mcp/` directory (nested under CLI project)
- **D-10:** Published to npm, users install/configure via npx (standard MCP server distribution pattern)

### Claude's Discretion
- TypeScript project setup details (tsconfig, build tooling)
- Exact MCP tool schema definition (parameter types, descriptions)
- Subprocess spawning and output parsing implementation
- npm package naming and configuration

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above. Research should investigate:
- `@modelcontextprotocol/sdk` TypeScript SDK documentation
- MCP stdio transport specification
- Claude Desktop MCP server configuration format

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/main.rs` — CLI entry point with clap derive macros, defines all flags and their behavior
- JSON output format already well-defined (schema_version, url, crawled_at, pages[], scores)

### Established Patterns
- All output is JSON to stdout, errors to stderr — clean separation for subprocess capture
- Exit codes: 0=success, 1=fail (with --fail-under) — useful for error detection
- Single binary distribution via `cargo build --release`

### Integration Points
- MCP server calls `geodaddy <url> [flags]` as child process
- Captures stdout (JSON report) and stderr (errors)
- Maps exit code to MCP success/error response

</code_context>

<specifics>
## Specific Ideas

- The MCP server should be a thin wrapper — all analysis logic stays in the Rust binary
- LLM clients should be able to analyze any URL by just calling the tool with a URL parameter
- The bundled binary approach means the npm package is self-contained (no separate geodaddy install required)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-add-local-mcp-server-for-llm-driven-cli-interaction*
*Context gathered: 2026-03-25*
