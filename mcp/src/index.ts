#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { getBinaryPath, buildArgs, runGeodaddy } from "./binary.js";

const server = new McpServer({
  name: "geodaddy-mcp",
  version: "0.2.0",
});

server.registerTool(
  "analyze_url",
  {
    title: "Analyze URL for GEO/SEO",
    description:
      "Run geodaddy GEO/SEO analysis on a URL. Returns a JSON report with overall score (0-100), per-category scores (Technical, Content, GEO, Performance), per-page analysis results, and actionable fix recommendations for AI search engine optimization.",
    inputSchema: {
      url: z
        .string()
        .describe("URL to analyze (supports http://localhost)"),
      max_pages: z
        .number()
        .int()
        .positive()
        .optional()
        .describe("Enable site crawling, stop after N pages"),
      enable_js: z
        .boolean()
        .optional()
        .describe(
          "Enable JavaScript rendering (downloads Chromium on first use)"
        ),
      vitals: z
        .boolean()
        .optional()
        .describe(
          "Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via headless browser"
        ),
      fail_under: z
        .number()
        .min(0)
        .max(100)
        .optional()
        .describe(
          "Return error if overall score is below this threshold (0-100)"
        ),
      beauty: z
        .boolean()
        .optional()
        .describe(
          "Return colored human-readable output instead of JSON (note: disables JSON output)"
        ),
    },
  },
  async ({ url, max_pages, enable_js, vitals, fail_under, beauty }) => {
    const args = buildArgs({
      url,
      max_pages,
      enable_js,
      vitals,
      fail_under,
      beauty,
    });

    try {
      const { stdout } = await runGeodaddy(args);
      return {
        content: [{ type: "text" as const, text: stdout }],
      };
    } catch (error: unknown) {
      const err = error as { stderr?: string; message?: string };
      const message = err.stderr || err.message || "Unknown error";
      return {
        content: [{ type: "text" as const, text: message }],
        isError: true,
      };
    }
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
