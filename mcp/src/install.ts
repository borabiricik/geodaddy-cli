import { installBinary } from "./download.js";

// postinstall is only a head start: package managers that block install
// scripts (npm 12, bun, --ignore-scripts) get the binary on first tool call.
installBinary().catch((error: Error) => {
  console.error(
    `geodaddy-mcp: binary download skipped (${error.message}). It will be retried on first use.`
  );
});
