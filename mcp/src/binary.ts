import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";

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
  checkExists: (path: string) => boolean = existsSync
): string {
  const ext = process.platform === "win32" ? ".exe" : "";
  const binaryName = "geodaddy" + ext;

  // First check: installed via postinstall (bin/ directory next to dist/)
  const binPath = resolve(__dirname, "..", "bin", binaryName);
  if (checkExists(binPath)) {
    return binPath;
  }

  // Second check: local dev via cargo build (target/release/ relative to cli/)
  const devPath = resolve(__dirname, "..", "..", "target", "release", binaryName);
  if (checkExists(devPath)) {
    return devPath;
  }

  throw new Error(
    "geodaddy binary not found. Run 'cargo build --release' or reinstall the package."
  );
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
  return execFileAsync(getBinaryPath(), args, {
    timeout: 120_000,
    maxBuffer: 10 * 1024 * 1024,
  });
}
