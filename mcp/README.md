# geodaddy-mcp

[MCP](https://modelcontextprotocol.io/) server for [geodaddy](https://github.com/borabiricik/geodaddy-cli) — run GEO/SEO analysis from AI assistants like Claude Desktop, Claude Code, and Cursor.

## Quick Setup

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

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

### Claude Code

```bash
claude mcp add geodaddy -- npx -y geodaddy-mcp
```

### Cursor

Go to **Settings > MCP Servers > Add**, then set command to `npx -y geodaddy-mcp`.

## Tool: `analyze_url`

Run geodaddy GEO/SEO analysis on a URL. Returns a JSON report with overall score (0-100), per-category scores, per-page results, and actionable fix recommendations.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | yes | URL to analyze (supports `http://localhost`) |
| `max_pages` | number | no | Enable site crawling, stop after N pages |
| `enable_js` | boolean | no | Enable JavaScript rendering (downloads Chromium on first use) |
| `vitals` | boolean | no | Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) |
| `fail_under` | number | no | Error if overall score is below threshold (0-100) |
| `beauty` | boolean | no | Return human-readable output instead of JSON |

## How It Works

The MCP server wraps the geodaddy Rust binary as a subprocess. When installed via npm, a postinstall script automatically downloads the correct platform binary from GitHub releases.

Supported platforms: macOS (Intel & ARM), Linux (x64 & ARM), Windows (x64).

## Development

```bash
# Build the CLI binary
cargo build --release

# Build and test the MCP server
cd mcp
npm install
npm run build
npx vitest run
```

The server looks for the geodaddy binary in:
1. `mcp/bin/geodaddy` — downloaded by postinstall
2. `target/release/geodaddy` — local dev build

## License

MIT
