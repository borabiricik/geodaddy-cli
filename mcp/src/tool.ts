import { z } from "zod";
import { buildArgs, runGeodaddy, type AnalyzeParams } from "./binary.js";

// ~3 s per page plus startup was measured; 25 pages stays inside the 120 s subprocess timeout.
export const MAX_PAGES_LIMIT = 25;

function isHttpUrl(value: string): boolean {
  try {
    return /^https?:$/.test(new URL(value).protocol);
  } catch {
    return false;
  }
}

export const analyzeUrlInputSchema = {
  url: z
    .string()
    .url()
    .refine(isHttpUrl, { message: "Only http:// and https:// URLs are supported" })
    .describe("http(s) URL to analyze (supports http://localhost)"),
  max_pages: z
    .number()
    .int()
    .positive()
    .max(MAX_PAGES_LIMIT)
    .optional()
    .describe(
      `Enable site crawling, stop after N pages (max ${MAX_PAGES_LIMIT}; run the geodaddy CLI directly for larger crawls)`
    ),
  enable_js: z
    .boolean()
    .optional()
    .describe(
      "Enable JavaScript rendering (requires an installed Chrome or Chromium; set CHROME_PATH to override)"
    ),
  vitals: z
    .boolean()
    .optional()
    .describe(
      "Measure lab Web Vitals (LCP, FCP, CLS, TTFB, and TBT as a lab proxy for INP) via headless Chrome"
    ),
  fail_under: z
    .number()
    .min(0)
    .max(100)
    .optional()
    .describe(
      "Mark the result as an error if the overall score is below this threshold (0-100); the report is still returned"
    ),
  beauty: z
    .boolean()
    .optional()
    .describe(
      "Return human-readable text instead of JSON (note: disables JSON output)"
    ),
};

type ToolResult = {
  content: { type: "text"; text: string }[];
  isError?: boolean;
};

type Runner = (args: string[]) => Promise<{ stdout: string; stderr: string }>;

const ANSI_PATTERN = /\x1b\[[0-9;]*m/g;

function stripAnsi(text: string): string {
  return text.replace(ANSI_PATTERN, "");
}

export async function analyzeUrl(
  params: AnalyzeParams,
  run: Runner = runGeodaddy
): Promise<ToolResult> {
  try {
    const { stdout } = await run(buildArgs(params));
    return { content: [{ type: "text", text: stdout }] };
  } catch (error: unknown) {
    const err = error as {
      code?: number | string;
      stdout?: string;
      stderr?: string;
      message?: string;
    };

    // geodaddy prints the full report and exits 1 when --fail-under is not met.
    if (params.fail_under !== undefined && err.code === 1 && err.stdout?.trim()) {
      return {
        content: [
          {
            type: "text",
            text: `Overall score is below the fail_under threshold of ${params.fail_under}.\n\n${err.stdout}`,
          },
        ],
        isError: true,
      };
    }

    const message = stripAnsi(err.stderr?.trim() || err.message || "Unknown error");
    return { content: [{ type: "text", text: message }], isError: true };
  }
}
