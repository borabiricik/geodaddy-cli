import { createWriteStream, mkdirSync, existsSync, chmodSync, unlinkSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";
import { pipeline } from "node:stream/promises";
import https from "node:https";
import http from "node:http";

const REPO = "borabiricik/geodaddy-cli";
const __dirname = dirname(fileURLToPath(import.meta.url));
const pkg = JSON.parse(readFileSync(resolve(__dirname, "..", "package.json"), "utf-8"));
const VERSION = `v${pkg.version}`;

function getPlatformTarget(): string {
  const map: Record<string, string> = {
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-musl",
    "linux-arm64": "aarch64-unknown-linux-musl",
    "win32-x64": "x86_64-pc-windows-msvc",
  };
  const key = `${process.platform}-${process.arch}`;
  const target = map[key];
  if (!target) {
    throw new Error(
      `Unsupported platform: ${key}. Supported: ${Object.keys(map).join(", ")}`
    );
  }
  return target;
}

function getArchiveUrl(target: string): string {
  if (target.includes("windows")) {
    return `https://github.com/${REPO}/releases/download/${VERSION}/geodaddy-${VERSION}-${target}.zip`;
  }
  return `https://github.com/${REPO}/releases/download/${VERSION}/geodaddy-${VERSION}-${target}.tar.gz`;
}

function download(url: string, dest: string, redirectCount = 0): Promise<void> {
  if (redirectCount > 5) {
    return Promise.reject(new Error("Too many redirects"));
  }

  const client = url.startsWith("https") ? https : http;

  return new Promise((resolve, reject) => {
    client.get(url, (response) => {
      const status = response.statusCode ?? 0;

      if (status === 301 || status === 302) {
        const location = response.headers.location;
        if (!location) {
          reject(new Error("Redirect with no location header"));
          return;
        }
        response.resume();
        download(location, dest, redirectCount + 1).then(resolve, reject);
        return;
      }

      if (status !== 200) {
        response.resume();
        reject(new Error(`Download failed with status ${status}`));
        return;
      }

      const fileStream = createWriteStream(dest);
      pipeline(response, fileStream).then(resolve, reject);
    }).on("error", reject);
  });
}

function extractBinary(
  archivePath: string,
  binDir: string,
  isWindows: boolean
): void {
  if (isWindows) {
    execSync(
      `powershell -command "Expand-Archive -Path '${archivePath}' -DestinationPath '${binDir}' -Force"`
    );
  } else {
    execSync(`tar -xzf ${archivePath} -C ${binDir}`);
  }
  unlinkSync(archivePath);
}

async function main(): Promise<void> {
  const binDir = resolve(__dirname, "..", "bin");
  mkdirSync(binDir, { recursive: true });

  const isWindows = process.platform === "win32";
  const binaryName = isWindows ? "geodaddy.exe" : "geodaddy";
  const binaryPath = resolve(binDir, binaryName);

  if (existsSync(binaryPath)) {
    console.error("geodaddy binary already exists, skipping download");
    return;
  }

  const target = getPlatformTarget();
  const archiveUrl = getArchiveUrl(target);
  const archiveExt = isWindows ? ".zip" : ".tar.gz";
  const archivePath = resolve(binDir, "geodaddy-archive" + archiveExt);

  console.error(`Downloading geodaddy ${VERSION} for ${target}...`);
  await download(archiveUrl, archivePath);

  extractBinary(archivePath, binDir, isWindows);

  if (!isWindows) {
    chmodSync(resolve(binDir, "geodaddy"), 0o755);
  }

  console.error(`geodaddy ${VERSION} installed successfully`);
}

main().catch((error: Error) => {
  console.error(
    "Warning: Failed to download geodaddy binary: " + error.message
  );
  console.error("You can build from source with: cargo build --release");
  process.exit(0);
});
