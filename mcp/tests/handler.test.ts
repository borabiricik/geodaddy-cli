import { describe, it, expect, vi } from "vitest";
import { z } from "zod";
import { analyzeUrl, analyzeUrlInputSchema, MAX_PAGES_LIMIT } from "../src/tool.js";

const schema = z.object(analyzeUrlInputSchema);

describe("analyze_url input validation", () => {
  it("accepts http and https URLs", () => {
    expect(schema.safeParse({ url: "https://example.com" }).success).toBe(true);
    expect(schema.safeParse({ url: "http://localhost:3000" }).success).toBe(true);
  });

  it.each(["--version", "--help", "app", "file:///etc/hosts", "ftp://example.com"])(
    "rejects %s",
    (url) => {
      expect(schema.safeParse({ url }).success).toBe(false);
    }
  );

  it("caps max_pages", () => {
    expect(schema.safeParse({ url: "https://example.com", max_pages: MAX_PAGES_LIMIT }).success).toBe(true);
    expect(schema.safeParse({ url: "https://example.com", max_pages: MAX_PAGES_LIMIT + 1 }).success).toBe(false);
  });
});

describe("analyzeUrl", () => {
  it("returns the report on success", async () => {
    const run = vi.fn(async () => ({ stdout: '{"score":80}', stderr: "" }));
    const result = await analyzeUrl({ url: "https://example.com" }, run);
    expect(result).toEqual({ content: [{ type: "text", text: '{"score":80}' }] });
    expect(run).toHaveBeenCalledWith(["--", "https://example.com"]);
  });

  it("keeps the report when the score is below fail_under", async () => {
    const run = vi.fn(async () => {
      throw Object.assign(new Error("Command failed"), {
        code: 1,
        stdout: '{"score":42}',
        stderr: "\x1b[32m INFO\x1b[0m geodaddy: Crawling page 1...",
      });
    });
    const result = await analyzeUrl({ url: "https://example.com", fail_under: 90 }, run);
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toContain("below the fail_under threshold of 90");
    expect(result.content[0].text).toContain('{"score":42}');
  });

  it("returns stderr without ANSI colors on other failures", async () => {
    const run = vi.fn(async () => {
      throw Object.assign(new Error("Command failed"), {
        code: 1,
        stdout: "",
        stderr: "\x1b[31mError:\x1b[0m Cannot connect to 'https://unreachable.test'",
      });
    });
    const result = await analyzeUrl({ url: "https://unreachable.test" }, run);
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toBe("Error: Cannot connect to 'https://unreachable.test'");
  });
});
