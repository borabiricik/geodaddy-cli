import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { resolve, dirname, delimiter } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";
import { installBinary } from "./download.js";

const execFileAsync = promisify(execFile);

const __dirname = dirname(fileURLToPath(import.meta.url));

export interface AnalyzeParams {
  url: string;
  max_pages?: number;
  enable_js?: boolean;
  vitals?: boolean;
  fail_under?: number;
  beauty?: boolean;
}

export function getBinaryPath(
  checkExists: (path: string) => boolean = existsSync,
  env: NodeJS.ProcessEnv = process.env
): string {
  const ext = process.platform === "win32" ? ".exe" : "";
  const binaryName = "geodaddy" + ext;

  // Explicit override, e.g. a binary installed with install.sh
  const override = env.GEODADDY_BIN;
  if (override) {
    if (checkExists(override)) {
      return override;
    }
    throw new Error(`GEODADDY_BIN is set to '${override}', but no file exists there.`);
  }

  // Installed by postinstall or by a previous on-demand download
  const binPath = resolve(__dirname, "..", "bin", binaryName);
  if (checkExists(binPath)) {
    return binPath;
  }

  // Local dev via cargo build (target/release/ relative to the repo root)
  const devPath = resolve(__dirname, "..", "..", "target", "release", binaryName);
  if (checkExists(devPath)) {
    return devPath;
  }

  throw new Error("geodaddy binary not found.");
}

function findOnPath(
  checkExists: (path: string) => boolean,
  env: NodeJS.ProcessEnv
): string | undefined {
  const binaryName = process.platform === "win32" ? "geodaddy.exe" : "geodaddy";
  return (env.PATH ?? "")
    .split(delimiter)
    .filter(Boolean)
    .map((dir) => resolve(dir, binaryName))
    .find((candidate) => checkExists(candidate));
}

let pendingInstall: Promise<string> | undefined;

/**
 * Resolves the geodaddy binary, downloading the matching release on first use
 * when it is missing (install scripts may have been blocked). Falls back to a
 * geodaddy on PATH if the download fails.
 */
export async function resolveBinaryPath(
  install: () => Promise<string> = installBinary,
  checkExists: (path: string) => boolean = existsSync,
  env: NodeJS.ProcessEnv = process.env
): Promise<string> {
  try {
    return getBinaryPath(checkExists, env);
  } catch (notFound) {
    if (env.GEODADDY_BIN) throw notFound;
  }

  pendingInstall ??= install().finally(() => {
    pendingInstall = undefined;
  });
  try {
    return await pendingInstall;
  } catch (error) {
    const onPath = findOnPath(checkExists, env);
    if (onPath) {
      return onPath;
    }
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(
      `geodaddy binary not found and the download failed (${reason}). ` +
        "Install the CLI (https://github.com/borabiricik/geodaddy-cli#installation) " +
        "or set GEODADDY_BIN to its path."
    );
  }
}

export function buildArgs(params: AnalyzeParams): string[] {
  const args: string[] = [params.url];

  if (params.max_pages !== undefined) {
    args.push("--max-pages", String(params.max_pages));
  }
  if (params.enable_js === true) {
    args.push("--enable-js");
  }
  if (params.vitals === true) {
    args.push("--vitals");
  }
  if (params.fail_under !== undefined) {
    args.push("--fail-under", String(params.fail_under));
  }
  if (params.beauty === true) {
    args.push("--beauty");
  }

  return args;
}

export async function runGeodaddy(
  args: string[]
): Promise<{ stdout: string; stderr: string }> {
  return execFileAsync(await resolveBinaryPath(), args, {
    timeout: 120_000,
    maxBuffer: 10 * 1024 * 1024,
  });
}
