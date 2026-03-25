import { describe, it, expect } from "vitest";
import { buildArgs, getBinaryPath } from "../src/binary.js";

describe("buildArgs", () => {
  it("returns only url for url-only call", () => {
    expect(buildArgs({ url: "https://example.com" })).toEqual([
      "https://example.com",
    ]);
  });

  it("adds --max-pages flag with value", () => {
    expect(buildArgs({ url: "https://example.com", max_pages: 5 })).toEqual([
      "https://example.com",
      "--max-pages",
      "5",
    ]);
  });

  it("adds --enable-js flag when true", () => {
    expect(
      buildArgs({ url: "https://example.com", enable_js: true })
    ).toEqual(["https://example.com", "--enable-js"]);
  });

  it("adds --vitals flag when true", () => {
    expect(buildArgs({ url: "https://example.com", vitals: true })).toEqual([
      "https://example.com",
      "--vitals",
    ]);
  });

  it("adds --fail-under flag with value", () => {
    expect(
      buildArgs({ url: "https://example.com", fail_under: 80 })
    ).toEqual(["https://example.com", "--fail-under", "80"]);
  });

  it("adds --beauty flag when true", () => {
    expect(buildArgs({ url: "https://example.com", beauty: true })).toEqual([
      "https://example.com",
      "--beauty",
    ]);
  });

  it("produces correct array with all flags", () => {
    expect(
      buildArgs({
        url: "https://example.com",
        max_pages: 5,
        enable_js: true,
        vitals: true,
        fail_under: 80,
        beauty: true,
      })
    ).toEqual([
      "https://example.com",
      "--max-pages",
      "5",
      "--enable-js",
      "--vitals",
      "--fail-under",
      "80",
      "--beauty",
    ]);
  });

  it("does not add boolean flags when false", () => {
    expect(
      buildArgs({ url: "https://example.com", enable_js: false })
    ).toEqual(["https://example.com"]);
  });

  it("does not add boolean flags when undefined", () => {
    expect(
      buildArgs({ url: "https://example.com", vitals: undefined })
    ).toEqual(["https://example.com"]);
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
