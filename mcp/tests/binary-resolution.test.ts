import { describe, it, expect, vi } from "vitest";
import { resolve } from "node:path";
import { getBinaryPath, resolveBinaryPath } from "../src/binary.js";
import { getArchiveUrl, getPlatformTarget } from "../src/download.js";

const binaryName = process.platform === "win32" ? "geodaddy.exe" : "geodaddy";

describe("getBinaryPath with GEODADDY_BIN", () => {
  it("uses GEODADDY_BIN when the file exists", () => {
    const path = getBinaryPath((p) => p === "/opt/geodaddy", { GEODADDY_BIN: "/opt/geodaddy" });
    expect(path).toBe("/opt/geodaddy");
  });

  it("throws a specific error when GEODADDY_BIN points nowhere", () => {
    expect(() => getBinaryPath(() => false, { GEODADDY_BIN: "/missing" })).toThrow(
      "GEODADDY_BIN is set to '/missing'"
    );
  });
});

describe("resolveBinaryPath", () => {
  it("does not download when a binary is already present", async () => {
    const install = vi.fn(async () => "/downloaded");
    const path = await resolveBinaryPath(install, () => true, {});
    expect(path).toMatch(/geodaddy(\.exe)?$/);
    expect(install).not.toHaveBeenCalled();
  });

  it("downloads on first use when install scripts were blocked", async () => {
    const install = vi.fn(async () => "/downloaded/geodaddy");
    const path = await resolveBinaryPath(install, () => false, {});
    expect(path).toBe("/downloaded/geodaddy");
    expect(install).toHaveBeenCalledTimes(1);
  });

  it("shares one download between concurrent calls", async () => {
    const install = vi.fn(
      () => new Promise<string>((done) => setTimeout(() => done("/downloaded/geodaddy"), 10))
    );
    const paths = await Promise.all([
      resolveBinaryPath(install, () => false, {}),
      resolveBinaryPath(install, () => false, {}),
    ]);
    expect(paths).toEqual(["/downloaded/geodaddy", "/downloaded/geodaddy"]);
    expect(install).toHaveBeenCalledTimes(1);
  });

  it("falls back to geodaddy on PATH when the download fails", async () => {
    const onPath = resolve("/usr/local/bin", binaryName);
    const install = vi.fn(async () => {
      throw new Error("offline");
    });
    const path = await resolveBinaryPath(install, (p) => p === onPath, {
      PATH: "/usr/local/bin",
    });
    expect(path).toBe(onPath);
  });

  it("explains how to fix it when nothing works", async () => {
    const install = vi.fn(async () => {
      throw new Error("offline");
    });
    await expect(resolveBinaryPath(install, () => false, { PATH: "" })).rejects.toThrow(
      /download failed \(offline\).*GEODADDY_BIN/
    );
  });

  it("does not download when GEODADDY_BIN is set but wrong", async () => {
    const install = vi.fn(async () => "/downloaded/geodaddy");
    await expect(
      resolveBinaryPath(install, () => false, { GEODADDY_BIN: "/missing" })
    ).rejects.toThrow("GEODADDY_BIN");
    expect(install).not.toHaveBeenCalled();
  });
});

describe("release asset naming", () => {
  it("maps supported platforms to release targets", () => {
    expect(getPlatformTarget("darwin", "arm64")).toBe("aarch64-apple-darwin");
    expect(getPlatformTarget("linux", "x64")).toBe("x86_64-unknown-linux-musl");
    expect(getPlatformTarget("win32", "x64")).toBe("x86_64-pc-windows-msvc");
    expect(() => getPlatformTarget("freebsd", "x64")).toThrow("Unsupported platform");
  });

  it("builds the archive URL used by release.yml", () => {
    expect(getArchiveUrl("v0.8.0", "aarch64-apple-darwin")).toBe(
      "https://github.com/borabiricik/geodaddy-cli/releases/download/v0.8.0/geodaddy-v0.8.0-aarch64-apple-darwin.tar.gz"
    );
    expect(getArchiveUrl("v0.8.0", "x86_64-pc-windows-msvc")).toMatch(/\.zip$/);
  });
});
