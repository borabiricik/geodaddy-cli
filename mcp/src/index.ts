#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { readFileSync } from "node:fs";
import { analyzeUrl, analyzeUrlInputSchema } from "./tool.js";

const { version } = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf-8")
) as { version: string };

const server = new McpServer(
  { name: "geodaddy-mcp", version },
  {
    instructions:
      "geodaddy analyzes public web pages for AI search optimization (GEO). Call analyze_url with an http(s) URL; it returns a JSON report with an overall score, per-category scores and per-check fix recommendations.",
  }
);

server.registerTool(
  "analyze_url",
  {
    title: "Analyze URL for GEO/SEO",
    description:
      "Run geodaddy GEO/SEO analysis on a URL. Returns a JSON report with overall score (0-100), per-category scores (Technical, Content, GEO, Agent, and Performance when vitals is on), per-page analysis results, and actionable fix recommendations for AI search engine optimization. Only reads the target site.",
    inputSchema: analyzeUrlInputSchema,
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      openWorldHint: true,
    },
  },
  (params) => analyzeUrl(params)
);

const transport = new StdioServerTransport();
await server.connect(transport);
