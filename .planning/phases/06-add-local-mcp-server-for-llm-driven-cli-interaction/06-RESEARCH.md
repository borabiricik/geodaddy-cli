# Phase 06: Add Local MCP Server for LLM-Driven CLI Interaction - Research

**Researched:** 2026-03-25
**Domain:** MCP (Model Context Protocol) server, TypeScript, stdio transport, npm binary distribution
**Confidence:** HIGH

## Summary

This phase builds a TypeScript MCP server that wraps the geodaddy Rust binary as a subprocess, exposing a single `analyze_url` tool over stdio transport. The official `@modelcontextprotocol/sdk` (v1.27.1) provides `McpServer` and `StdioServerTransport` classes with Zod-based schema validation for tool parameters. The server calls the geodaddy binary, captures stdout JSON, and returns it as MCP tool content.

The primary complexity is binary distribution -- bundling platform-specific Rust binaries into an npm package. The established pattern (used by esbuild, Tailwind CSS, and others) uses `optionalDependencies` with platform-specific sub-packages, but for an MCP server this is overkill. A simpler approach downloads the correct binary from GitHub releases at postinstall time, since geodaddy already publishes 5 platform targets via the existing release workflow.

**Primary recommendation:** Use the official MCP SDK with stdio transport, download the geodaddy binary from GitHub releases during npm postinstall, and publish as a single npm package invokable via `npx`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: MCP server written in TypeScript using the official `@modelcontextprotocol/sdk`
- D-02: stdio transport only (standard for local MCP servers, natively supported by Claude Desktop, Claude Code, Cursor)
- D-03: geodaddy binary bundled with the MCP server package (self-contained distribution)
- D-04: Single `analyze_url` MCP tool that mirrors the CLI interface
- D-05: All CLI flags exposed as tool parameters: `url` (required), `max_pages`, `enable_js`, `vitals`, `fail_under`, `beauty` -- 1:1 mapping with CLI flags
- D-06: LLM decides which parameters to pass based on user's request context
- D-07: Raw JSON output from geodaddy passed directly as MCP tool result content -- no transformation or summarization
- D-08: Errors return MCP error response (`isError: true`) with geodaddy's stderr message -- LLM sees it as tool error and can retry or inform user
- D-09: MCP server code lives in `cli/mcp/` directory (nested under CLI project)
- D-10: Published to npm, users install/configure via npx (standard MCP server distribution pattern)

### Claude's Discretion
- TypeScript project setup details (tsconfig, build tooling)
- Exact MCP tool schema definition (parameter types, descriptions)
- Subprocess spawning and output parsing implementation
- npm package naming and configuration

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Project Constraints (from CLAUDE.md)

- Language: Rust for the CLI binary (already built); TypeScript for the MCP wrapper
- Distribution: Local CLI first, no cloud dependencies
- Output: JSON-only for v1 (the MCP server passes this through directly)
- The geodaddy binary is already built with 5 cross-platform targets via GitHub Actions release workflow

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @modelcontextprotocol/sdk | 1.27.1 | MCP server framework | Official SDK, locked decision D-01 |
| zod | 3.25+ or 4.x | Schema validation for tool params | Peer dependency of MCP SDK |
| typescript | 5.8+ | TypeScript compiler | Standard for MCP server projects |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @types/node | 22 | Node.js type definitions | Always (dev dependency) |
| vitest | 2.x | Test framework | Testing tool handler logic |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Postinstall binary download | optionalDependencies platform packages | optionalDependencies requires publishing 5+ sub-packages to npm; overkill for this project. Postinstall download from GitHub releases is simpler and geodaddy already has releases. |
| vitest | jest | vitest is faster, used by official MCP server-filesystem reference |
| tsc direct | tsup/esbuild bundler | tsc is sufficient for a small server; bundlers add complexity for no gain here |

**Installation:**
```bash
npm init -y
npm install @modelcontextprotocol/sdk zod
npm install -D typescript @types/node vitest
```

**Version verification:** Versions confirmed against npm registry on 2026-03-25:
- @modelcontextprotocol/sdk: 1.27.1 (latest)
- zod: 4.3.6 (latest, within SDK peer dep range ^3.25 || ^4.0)
- typescript: 6.0.2 (latest)

## Architecture Patterns

### Recommended Project Structure
```
cli/mcp/
  package.json        # npm package config with bin field
  tsconfig.json       # TypeScript configuration
  src/
    index.ts          # Entry point: McpServer + StdioServerTransport setup
    tool.ts           # analyze_url tool registration + handler
    binary.ts         # Binary path resolution + subprocess spawning
    install.ts        # Postinstall script: download binary from GitHub releases
  bin/
    geodaddy          # Downloaded binary (gitignored, created at install time)
  dist/               # Compiled JS output (gitignored)
  tests/
    tool.test.ts      # Tool handler unit tests
```

### Pattern 1: MCP Server with stdio Transport
**What:** Create McpServer, register tool, connect via StdioServerTransport
**When to use:** Always -- this is the only pattern for this phase
**Example:**
```typescript
// Source: https://ts.sdk.modelcontextprotocol.io/documents/server.html
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new McpServer({
  name: "geodaddy-mcp",
  version: "0.1.0",
});

server.registerTool(
  "analyze_url",
  {
    title: "Analyze URL",
    description: "Run geodaddy GEO/SEO analysis on a URL. Returns a JSON report with scores, per-page results, and actionable fix recommendations.",
    inputSchema: {
      url: z.string().url().describe("URL to analyze (supports http://localhost)"),
      max_pages: z.number().int().positive().optional().describe("Enable crawling, stop after N pages"),
      enable_js: z.boolean().optional().describe("Enable JavaScript rendering via headless browser"),
      vitals: z.boolean().optional().describe("Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT)"),
      fail_under: z.number().min(0).max(100).optional().describe("Fail if overall score below threshold (0-100)"),
      beauty: z.boolean().optional().describe("Output colored human-readable report instead of JSON"),
    },
  },
  async ({ url, max_pages, enable_js, vitals, fail_under, beauty }) => {
    // Build CLI args and spawn geodaddy binary
    // Return { content: [{ type: "text", text: stdout }] }
    // On error: return { content: [{ type: "text", text: stderr }], isError: true }
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
```

### Pattern 2: Subprocess Spawning
**What:** Spawn geodaddy binary, capture stdout/stderr, map exit code to MCP response
**When to use:** In the tool handler
**Example:**
```typescript
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

async function runGeodaddy(args: string[]): Promise<{ stdout: string; stderr: string }> {
  const binaryPath = getBinaryPath(); // resolves to cli/mcp/bin/geodaddy
  return execFileAsync(binaryPath, args, {
    timeout: 120_000, // 2 minute timeout for crawling
    maxBuffer: 10 * 1024 * 1024, // 10MB for large site reports
  });
}
```

### Pattern 3: Error Handling with isError
**What:** Return MCP error responses when geodaddy fails
**When to use:** When subprocess exits non-zero or throws
**Example:**
```typescript
try {
  const { stdout } = await runGeodaddy(args);
  return {
    content: [{ type: "text", text: stdout }],
  };
} catch (error: any) {
  // execFile rejects on non-zero exit code
  const stderr = error.stderr || error.message;
  return {
    content: [{ type: "text", text: stderr }],
    isError: true,
  };
}
```

### Pattern 4: Binary Download at Postinstall
**What:** Download platform-appropriate geodaddy binary from GitHub releases
**When to use:** npm postinstall hook
**Example:**
```typescript
// install.ts - runs as postinstall script
import { execSync } from "node:child_process";
import { createWriteStream, chmodSync, mkdirSync } from "node:fs";
import { pipeline } from "node:stream/promises";
import https from "node:https";

const REPO = "user/geodaddy"; // adjust to actual GitHub org/repo
const VERSION = "v0.1.1"; // pinned to CLI version

function getPlatformTarget(): string {
  const platform = process.platform; // darwin, linux, win32
  const arch = process.arch; // x64, arm64
  const map: Record<string, string> = {
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-musl",
    "linux-arm64": "aarch64-unknown-linux-musl",
    "win32-x64": "x86_64-pc-windows-msvc",
  };
  const target = map[`${platform}-${arch}`];
  if (!target) throw new Error(`Unsupported platform: ${platform}-${arch}`);
  return target;
}
```

### Anti-Patterns to Avoid
- **Writing to stdout in the MCP server:** stdio transport uses stdout for JSON-RPC messages. Any console.log() or process.stdout.write() will corrupt the protocol. Use console.error() or MCP logging for diagnostics.
- **Transforming geodaddy JSON output:** D-07 requires raw JSON passthrough. Do not parse, filter, or restructure the output.
- **Hardcoding binary path:** Use path resolution relative to the package directory, not cwd.
- **Omitting timeout on subprocess:** Crawling can be slow. Set a generous timeout (2+ minutes) to avoid premature kills.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP protocol handling | Custom JSON-RPC over stdio | @modelcontextprotocol/sdk McpServer + StdioServerTransport | Protocol is complex with negotiation, capabilities, etc. |
| Schema validation | Manual parameter checking | Zod schemas via SDK registerTool | SDK auto-validates and rejects malformed input |
| Binary platform detection | Custom platform mapping | Standard process.platform + process.arch mapping | Well-established Node.js pattern |

## Common Pitfalls

### Pitfall 1: stdout Corruption
**What goes wrong:** Any output written to stdout breaks the MCP JSON-RPC protocol
**Why it happens:** console.log() defaults to stdout; developers add debug logging
**How to avoid:** Never use console.log() in the server. Use console.error() for debug output. The MCP SDK handles all stdout communication.
**Warning signs:** Client reports "invalid JSON" or "parse error" from MCP server

### Pitfall 2: Binary Not Found After Install
**What goes wrong:** Postinstall script fails silently or binary path is wrong at runtime
**Why it happens:** npm postinstall can be disabled by users (--ignore-scripts), or path resolution fails
**How to avoid:** Check binary exists at startup, provide clear error message. Include a manual download fallback command.
**Warning signs:** "ENOENT" errors when spawning subprocess

### Pitfall 3: Windows cmd Wrapper Requirement
**What goes wrong:** npx-based MCP servers fail on Windows
**Why it happens:** Claude Desktop on Windows needs `cmd /c npx` not just `npx`
**How to avoid:** Document Windows configuration explicitly: `"command": "cmd", "args": ["/c", "npx", "-y", "geodaddy-mcp"]`
**Warning signs:** Server fails to start on Windows only

### Pitfall 4: Large Output Buffer Overflow
**What goes wrong:** execFile fails with "maxBuffer exceeded" for large site crawls
**Why it happens:** Default Node.js maxBuffer is 1MB; multi-page crawl reports can exceed this
**How to avoid:** Set maxBuffer to 10MB+ in execFile options
**Warning signs:** "Error: maxBuffer length exceeded" in stderr

### Pitfall 5: Package.json bin Shebang Missing
**What goes wrong:** npx fails to execute the compiled JS entry point
**Why it happens:** Compiled dist/index.js lacks `#!/usr/bin/env node` shebang
**How to avoid:** Add shebang as first line of src/index.ts: `#!/usr/bin/env node`
**Warning signs:** "Permission denied" or "not a recognized command" when running via npx

### Pitfall 6: beauty Flag in MCP Context
**What goes wrong:** LLM receives ANSI escape codes instead of parseable JSON
**Why it happens:** `--beauty` flag outputs colored terminal text, not JSON
**How to avoid:** Consider omitting beauty from MCP tool schema or documenting that it returns non-JSON. Per D-05 it should be exposed, but the tool description should warn that it disables JSON output.
**Warning signs:** LLM response contains garbled escape sequences

## Code Examples

### Complete MCP Server Entry Point
```typescript
// Source: https://ts.sdk.modelcontextprotocol.io/documents/server.html
// + https://github.com/modelcontextprotocol/servers/blob/main/src/filesystem/
#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const __dirname = dirname(fileURLToPath(import.meta.url));

function getBinaryPath(): string {
  const ext = process.platform === "win32" ? ".exe" : "";
  return resolve(__dirname, "..", "bin", `geodaddy${ext}`);
}

const server = new McpServer({
  name: "geodaddy-mcp",
  version: "0.1.0",
});

server.registerTool(
  "analyze_url",
  {
    title: "Analyze URL for GEO/SEO",
    description:
      "Run geodaddy analysis on a URL. Returns a JSON report with overall score, per-category scores (Technical, Content, GEO), per-page analysis results, and actionable fix recommendations for AI search engine optimization.",
    inputSchema: {
      url: z.string().describe("URL to analyze"),
      max_pages: z
        .number()
        .int()
        .positive()
        .optional()
        .describe("Enable site crawling, stop after N pages"),
      enable_js: z
        .boolean()
        .optional()
        .describe("Enable JavaScript rendering (downloads Chromium on first use)"),
      vitals: z
        .boolean()
        .optional()
        .describe("Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT)"),
      fail_under: z
        .number()
        .min(0)
        .max(100)
        .optional()
        .describe("Return error if overall score is below this threshold"),
      beauty: z
        .boolean()
        .optional()
        .describe("Return colored human-readable output instead of JSON"),
    },
  },
  async ({ url, max_pages, enable_js, vitals, fail_under, beauty }) => {
    const args: string[] = [url];
    if (max_pages !== undefined) args.push("--max-pages", String(max_pages));
    if (enable_js) args.push("--enable-js");
    if (vitals) args.push("--vitals");
    if (fail_under !== undefined) args.push("--fail-under", String(fail_under));
    if (beauty) args.push("--beauty");

    try {
      const { stdout } = await execFileAsync(getBinaryPath(), args, {
        timeout: 120_000,
        maxBuffer: 10 * 1024 * 1024,
      });
      return { content: [{ type: "text", text: stdout }] };
    } catch (error: any) {
      const message = error.stderr || error.message || "Unknown error";
      return { content: [{ type: "text", text: message }], isError: true };
    }
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
```

### Claude Desktop Configuration
```json
{
  "mcpServers": {
    "geodaddy": {
      "command": "npx",
      "args": ["-y", "geodaddy-mcp"]
    }
  }
}
```

### Windows Claude Desktop Configuration
```json
{
  "mcpServers": {
    "geodaddy": {
      "command": "cmd",
      "args": ["/c", "npx", "-y", "geodaddy-mcp"]
    }
  }
}
```

### Claude Code Configuration
```json
{
  "mcpServers": {
    "geodaddy": {
      "command": "npx",
      "args": ["-y", "geodaddy-mcp"],
      "type": "stdio"
    }
  }
}
```

### package.json Structure
```json
{
  "name": "geodaddy-mcp",
  "version": "0.1.0",
  "description": "MCP server for geodaddy GEO/SEO analysis",
  "license": "MIT",
  "type": "module",
  "bin": {
    "geodaddy-mcp": "dist/index.js"
  },
  "scripts": {
    "build": "tsc && chmod +x dist/index.js",
    "prepare": "npm run build",
    "postinstall": "node dist/install.js",
    "test": "vitest"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.27.1",
    "zod": "^4.0.0"
  },
  "devDependencies": {
    "@types/node": "^22",
    "typescript": "^5.8.0",
    "vitest": "^2.0.0"
  },
  "engines": {
    "node": ">=18"
  },
  "files": [
    "dist",
    "bin"
  ]
}
```

### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Node16",
    "moduleResolution": "Node16",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "declaration": true
  },
  "include": ["src"]
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| MCP SDK v0.x with Server class | MCP SDK v1.x with McpServer class | 2025-Q4 | McpServer has registerTool(), simpler API |
| SSE transport for local servers | stdio transport standard | 2025 | stdio is default for local MCP servers |
| Manual JSON-RPC implementation | SDK handles protocol | 2025 | No need to implement handshake, capabilities negotiation |

**Deprecated/outdated:**
- `Server` class from earlier SDK versions -- use `McpServer` instead
- SSE transport for local servers -- stdio is the standard
- `@modelcontextprotocol/sdk` v0.x API -- v1.x has breaking changes

## Open Questions

1. **npm Package Name Availability**
   - What we know: `geodaddy-mcp` follows the standard `{tool}-mcp` naming convention
   - What's unclear: Whether the name is available on npm
   - Recommendation: Check `npm view geodaddy-mcp` before publishing; fall back to `@geodaddy/mcp` if taken

2. **GitHub Repository Path for Binary Downloads**
   - What we know: Release workflow publishes to GitHub releases with format `geodaddy-{version}-{target}.tar.gz`
   - What's unclear: Exact GitHub org/repo path for download URLs
   - Recommendation: Use the actual repo path (discoverable from `git remote -v`); hardcode in install script

3. **Binary Version Pinning Strategy**
   - What we know: MCP package and CLI binary versions should stay in sync
   - What's unclear: Whether to pin exact version or allow range
   - Recommendation: Pin exact version in the install script; bump when CLI releases new version

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | MCP server runtime | Yes | 22.14.0 | -- |
| npm | Package management | Yes | 10.9.2 | -- |
| npx | MCP server execution | Yes | 10.9.2 | -- |
| TypeScript | Build | Yes (installable) | 6.0.2 (latest) | -- |
| geodaddy binary | Subprocess | Yes (local build) | 0.1.1 | cargo build --release |

**Missing dependencies with no fallback:** None
**Missing dependencies with fallback:** None

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 2.x |
| Config file | cli/mcp/vitest.config.ts (Wave 0) |
| Quick run command | `cd cli/mcp && npx vitest run` |
| Full suite command | `cd cli/mcp && npx vitest run --coverage` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-04 | Single analyze_url tool registered | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "registers analyze_url"` | No -- Wave 0 |
| D-05 | All CLI flags mapped as parameters | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "maps parameters"` | No -- Wave 0 |
| D-07 | Raw JSON passthrough | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "passes raw JSON"` | No -- Wave 0 |
| D-08 | Error returns isError: true | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "returns isError"` | No -- Wave 0 |
| D-02 | stdio transport connects | integration | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "stdio"` | No -- Wave 0 |
| D-03 | Binary resolution works | unit | `cd cli/mcp && npx vitest run tests/binary.test.ts` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cd cli/mcp && npx vitest run`
- **Per wave merge:** `cd cli/mcp && npx vitest run --coverage`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `cli/mcp/tests/tool.test.ts` -- covers D-04, D-05, D-07, D-08
- [ ] `cli/mcp/tests/binary.test.ts` -- covers D-03 binary resolution
- [ ] `cli/mcp/vitest.config.ts` -- test config
- [ ] Framework install: `cd cli/mcp && npm install` -- sets up all dev dependencies

## Sources

### Primary (HIGH confidence)
- npm registry: @modelcontextprotocol/sdk v1.27.1 -- version, dependencies, exports verified
- npm registry: zod v4.3.6 -- peer dependency compatibility verified
- [MCP TypeScript SDK docs](https://ts.sdk.modelcontextprotocol.io/documents/server.html) -- McpServer API, registerTool, StdioServerTransport
- [Official MCP server-filesystem](https://github.com/modelcontextprotocol/servers/blob/main/src/filesystem/) -- reference package.json, project structure
- [MCP Build Server Guide](https://modelcontextprotocol.io/docs/develop/build-server) -- official tutorial

### Secondary (MEDIUM confidence)
- [Packaging Rust for npm](https://blog.orhun.dev/packaging-rust-for-npm/) -- binary distribution patterns
- [Publishing binaries on npm](https://sentry.engineering/blog/publishing-binaries-on-npm) -- optionalDependencies vs postinstall
- [MCP Local Server Connection](https://modelcontextprotocol.io/docs/develop/connect-local-servers) -- Claude Desktop config format
- [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) -- Claude Code configuration

### Tertiary (LOW confidence)
- [Windows npx wrapper issue](https://github.com/SuperClaude-Org/SuperClaude_Framework/issues/390) -- cmd /c requirement on Windows

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- SDK version verified on npm, API confirmed from official docs
- Architecture: HIGH -- patterns follow official MCP server reference implementations
- Pitfalls: HIGH -- stdout corruption and Windows issues are well-documented in MCP ecosystem

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (MCP SDK is fast-moving; check for v2 stable release)
