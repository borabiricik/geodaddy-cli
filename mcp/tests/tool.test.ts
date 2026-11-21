import { describe, it, expect } from "vitest";
import { buildArgs, getBinaryPath } from "../src/binary.js";

describe("buildArgs", () => {
  const url = "https://example.com";

  it("puts the url after -- for a url-only call", () => {
    expect(buildArgs({ url })).toEqual(["--", url]);
  });

  it("adds --max-pages flag with value", () => {
    expect(buildArgs({ url, max_pages: 5 })).toEqual(["--max-pages", "5", "--", url]);
  });

  it("adds --enable-js flag when true", () => {
    expect(buildArgs({ url, enable_js: true })).toEqual(["--enable-js", "--", url]);
  });

  it("adds --vitals flag when true", () => {
    expect(buildArgs({ url, vitals: true })).toEqual(["--vitals", "--", url]);
  });

  it("adds --fail-under flag with value", () => {
    expect(buildArgs({ url, fail_under: 80 })).toEqual(["--fail-under", "80", "--", url]);
  });

  it("adds --beauty flag when true", () => {
    expect(buildArgs({ url, beauty: true })).toEqual(["--beauty", "--", url]);
  });

  it("produces correct array with all flags", () => {
    expect(
      buildArgs({
        url,
        max_pages: 5,
        enable_js: true,
        vitals: true,
        fail_under: 80,
        beauty: true,
      })
    ).toEqual([
      "--max-pages",
      "5",
      "--enable-js",
      "--vitals",
      "--fail-under",
      "80",
      "--beauty",
      "--",
      url,
    ]);
  });

  it("does not add boolean flags when false", () => {
    expect(buildArgs({ url, enable_js: false })).toEqual(["--", url]);
  });

  it("does not add boolean flags when undefined", () => {
    expect(buildArgs({ url, vitals: undefined })).toEqual(["--", url]);
  });

  it("keeps a flag-like url positional", () => {
    expect(buildArgs({ url: "--version" })).toEqual(["--", "--version"]);
  });
});

describe("getBinaryPath", () => {
  it("throws when binary not found at any location", () => {
    // Inject a checkExists that always returns false
    expect(() => getBinaryPath(() => false)).toThrow(
      "geodaddy binary not found"
    );
  });

  it("returns a path ending with geodaddy when bin/ path exists", () => {
    // Inject a checkExists that returns true for the first call (bin/ path)
    const path = getBinaryPath(() => true);
    expect(path).toMatch(/geodaddy(\.exe)?$/);
    expect(path).toContain("bin");
  });

  it("falls back to target/release/ when bin/ path does not exist", () => {
    let callCount = 0;
    const path = getBinaryPath(() => {
      callCount++;
      // First call (bin/ path) returns false, second call (target/release/) returns true
      return callCount >= 2;
    });
    expect(path).toMatch(/geodaddy(\.exe)?$/);
    expect(path).toContain("target");
    expect(path).toContain("release");
  });
});
