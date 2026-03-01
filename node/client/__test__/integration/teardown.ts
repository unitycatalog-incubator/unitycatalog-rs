import * as fs from "node:fs";
import * as path from "node:path";

const STATE_FILE = path.join(__dirname, ".server-state.json");

export default async function globalTeardown() {
  if (!fs.existsSync(STATE_FILE)) return;

  try {
    const state = JSON.parse(fs.readFileSync(STATE_FILE, "utf-8"));
    if (state.pid) {
      process.kill(state.pid, "SIGTERM");
      // Give it a moment to shut down gracefully
      await new Promise((r) => setTimeout(r, 500));
    }
  } catch {
    // Process may already be gone
  } finally {
    try {
      fs.unlinkSync(STATE_FILE);
    } catch {
      // ignore
    }
  }
}
