import {
  chmodSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { pipeline } from "node:stream/promises";
import https from "node:https";
import type { IncomingMessage } from "node:http";

const REPO = "borabiricik/geodaddy-cli";
const DOWNLOAD_TIMEOUT_MS = 120_000;
const MAX_REDIRECTS = 5;

const __dirname = dirname(fileURLToPath(import.meta.url));

export const PACKAGE_DIR = resolve(__dirname, "..");
export const BIN_DIR = resolve(PACKAGE_DIR, "bin");

function packageVersion(): string {
  const pkg = JSON.parse(
    readFileSync(resolve(PACKAGE_DIR, "package.json"), "utf-8")
  );
  return `v${pkg.version}`;
}

export function getPlatformTarget(
  platform: string = process.platform,
  arch: string = process.arch
): string {
  const map: Record<string, string> = {
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-musl",
    "linux-arm64": "aarch64-unknown-linux-musl",
    "win32-x64": "x86_64-pc-windows-msvc",
  };
  const key = `${platform}-${arch}`;
  const target = map[key];
  if (!target) {
    throw new Error(
      `Unsupported platform: ${key}. Supported: ${Object.keys(map).join(", ")}`
    );
  }
  return target;
}

export function getArchiveUrl(version: string, target: string): string {
  const ext = target.includes("windows") ? "zip" : "tar.gz";
  return `https://github.com/${REPO}/releases/download/${version}/geodaddy-${version}-${target}.${ext}`;
}

function get(url: string, signal: AbortSignal): Promise<IncomingMessage> {
  return new Promise((resolvePromise, reject) => {
    https.get(url, { signal }, resolvePromise).on("error", reject);
  });
}

async function download(url: string, dest: string, signal: AbortSignal): Promise<void> {
  let current = url;
  for (let redirects = 0; redirects <= MAX_REDIRECTS; redirects++) {
    if (!current.startsWith("https://")) {
      throw new Error(`Refusing non-HTTPS download URL: ${current}`);
    }
    const response = await get(current, signal);
    const status = response.statusCode ?? 0;
    if (status >= 300 && status < 400 && response.headers.location) {
      response.resume();
      current = new URL(response.headers.location, current).toString();
      continue;
    }
    if (status !== 200) {
      response.resume();
      throw new Error(`Download failed with status ${status}: ${current}`);
    }
    await pipeline(response, createWriteStream(dest), { signal });
    return;
  }
  throw new Error("Too many redirects");
}

async function downloadWithDeadline(url: string, dest: string): Promise<void> {
  // One deadline for the whole download, including redirects and a slow body.
  const signal = AbortSignal.timeout(DOWNLOAD_TIMEOUT_MS);
  try {
    await download(url, dest, signal);
  } catch (error) {
    if (signal.aborted) {
      throw new Error(`Download did not finish within ${DOWNLOAD_TIMEOUT_MS / 1000} s: ${url}`);
    }
    throw error;
  }
}

function extract(archivePath: string, destDir: string, isWindows: boolean): void {
  if (isWindows) {
    // powershell joins everything after -Command into one script, so paths are quoted inside it.
    const quote = (path: string) => `'${path.replace(/'/g, "''")}'`;
    execFileSync("powershell", [
      "-NoProfile",
      "-NonInteractive",
      "-Command",
      `Expand-Archive -LiteralPath ${quote(archivePath)} -DestinationPath ${quote(destDir)} -Force`,
    ]);
  } else {
    execFileSync("tar", ["-xzf", archivePath, "-C", destDir]);
  }
}

/**
 * Downloads the geodaddy release binary matching this package's version into
 * `bin/` and returns its path. Safe to call concurrently: the binary is
 * extracted into a temporary directory and moved into place in one step.
 */
export async function installBinary(binDir: string = BIN_DIR): Promise<string> {
  const isWindows = process.platform === "win32";
  const binaryName = isWindows ? "geodaddy.exe" : "geodaddy";
  const binaryPath = resolve(binDir, binaryName);
  if (existsSync(binaryPath)) {
    return binaryPath;
  }

  const version = packageVersion();
  const target = getPlatformTarget();
  mkdirSync(binDir, { recursive: true });
  const workDir = mkdtempSync(join(binDir, ".download-"));
  try {
    const archivePath = join(workDir, isWindows ? "archive.zip" : "archive.tar.gz");
    console.error(`Downloading geodaddy ${version} for ${target}...`);
    await downloadWithDeadline(getArchiveUrl(version, target), archivePath);
    extract(archivePath, workDir, isWindows);
    const extracted = join(workDir, binaryName);
    if (!isWindows) {
      chmodSync(extracted, 0o755);
    }
    try {
      renameSync(extracted, binaryPath);
    } catch (error) {
      // A concurrent install won the race (Windows rename does not overwrite).
      if (!existsSync(binaryPath)) throw error;
    }
    console.error(`geodaddy ${version} installed at ${binaryPath}`);
    return binaryPath;
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}
