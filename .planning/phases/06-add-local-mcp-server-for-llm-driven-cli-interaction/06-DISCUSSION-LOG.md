# Phase 6: Add local MCP server for LLM-driven CLI interaction - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 06-add-local-mcp-server-for-llm-driven-cli-interaction
**Areas discussed:** Transport & language, Tool design, Output handling, Distribution

---

## Transport & Language

### Language Choice

| Option | Description | Selected |
|--------|-------------|----------|
| TypeScript (Recommended) | Use official @modelcontextprotocol/sdk. Fastest path. Calls geodaddy as subprocess. | ✓ |
| Rust native | Write MCP server in Rust. Requires lib crate extraction or community MCP crate. | |
| Python | Use official Python mcp SDK. Calls geodaddy as subprocess. | |

**User's choice:** TypeScript (Recommended)
**Notes:** Official SDK support, fastest implementation path

### Transport Protocol

| Option | Description | Selected |
|--------|-------------|----------|
| stdio (Recommended) | Standard for local MCP servers. Natively supported by all major clients. | ✓ |
| HTTP/SSE | Runs on local port. More flexible but requires port management. | |
| Both | stdio default + optional --http flag. Maximum compatibility. | |

**User's choice:** stdio (Recommended)
**Notes:** None

### Binary Discovery

| Option | Description | Selected |
|--------|-------------|----------|
| PATH lookup (Recommended) | Assume geodaddy on PATH. Simple, works with cargo install. | |
| Configurable path | GEODADDY_PATH env var with PATH fallback. | |
| Bundled binary | Ship geodaddy binary alongside MCP server. Self-contained. | ✓ |

**User's choice:** Bundled binary
**Notes:** User prefers self-contained distribution

---

## Tool Design

### Tool Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Single analyze tool (Recommended) | One 'analyze_url' tool with all flags as params. Mirrors CLI. | ✓ |
| Granular tools | Separate tools per analyzer type. More discoverable. | |
| Layered | Main analyze tool + helper tools for context. | |

**User's choice:** Single analyze tool (Recommended)
**Notes:** None

### Parameter Exposure

| Option | Description | Selected |
|--------|-------------|----------|
| All flags (Recommended) | 1:1 mapping with CLI flags. LLM can use any combination. | ✓ |
| Curated subset | Only essential flags. Simpler schema. | |
| All flags + extras | CLI flags plus MCP-specific params like summary_mode. | |

**User's choice:** All flags (Recommended)
**Notes:** None

---

## Output Handling

### Result Format

| Option | Description | Selected |
|--------|-------------|----------|
| Raw JSON (Recommended) | Pass geodaddy JSON directly. No information loss. Simplest. | ✓ |
| Summarized text | Parse and generate human-readable summary. | |
| Structured + summary | Both raw JSON and text summary. | |

**User's choice:** Raw JSON (Recommended)
**Notes:** None

### Error Handling

| Option | Description | Selected |
|--------|-------------|----------|
| MCP error response (Recommended) | isError: true with stderr message. LLM sees tool error. | ✓ |
| Error in content | Always success, error details in content text. | |

**User's choice:** MCP error response (Recommended)
**Notes:** None

---

## Distribution

### Code Location

| Option | Description | Selected |
|--------|-------------|----------|
| mcp/ directory (Recommended) | Separate mcp/ at repo root. Clean separation. | |
| Inside cli/mcp/ | Nested under CLI project. Keeps everything together. | ✓ |
| Separate repo | Maximum independence. Harder to sync. | |

**User's choice:** Inside cli/mcp/
**Notes:** User prefers keeping MCP server nested under CLI project

### Installation Method

| Option | Description | Selected |
|--------|-------------|----------|
| npx (Recommended) | Published to npm, standard MCP distribution pattern. | ✓ |
| Manual setup | Clone repo, npm install, point config to local path. | |
| Both | npm publish + local dev path support. | |

**User's choice:** npx (Recommended)
**Notes:** None

---

## Claude's Discretion

- TypeScript project setup details (tsconfig, build tooling)
- Exact MCP tool schema definition (parameter types, descriptions)
- Subprocess spawning and output parsing implementation
- npm package naming and configuration

## Deferred Ideas

None
