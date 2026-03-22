import { type ChildProcess, execSync, spawn } from "node:child_process";
import * as fs from "node:fs";
import * as net from "node:net";
import * as path from "node:path";

const SERVER_READY_TIMEOUT_MS = 180_000;
const READY_PATTERN = /Listening on: .+:(\d+)/;

const STATE_FILE = path.join(__dirname, ".server-state.json");

function findAvailablePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.listen(0, "127.0.0.1", () => {
      const addr = srv.address();
      if (addr && typeof addr !== "string") {
        const port = addr.port;
        srv.close(() => resolve(port));
      } else {
        srv.close(() => reject(new Error("Could not determine port")));
      }
    });
    srv.on("error", reject);
  });
}

function resolveServerBinary(workspaceRoot: string): {
  command: string;
  args: string[];
} {
  if (process.env.UC_SERVER_BINARY) {
    console.log(
      `[setup] Using UC_SERVER_BINARY=${process.env.UC_SERVER_BINARY}`,
    );
    return { command: process.env.UC_SERVER_BINARY, args: [] };
  }

  // Try to locate the cargo target directory and use a pre-built binary
  try {
    const out = execSync("cargo metadata --format-version=1 --no-deps", {
      cwd: workspaceRoot,
      stdio: ["pipe", "pipe", "pipe"],
      timeout: 30_000,
      maxBuffer: 10 * 1024 * 1024,
    }).toString();
    const meta = JSON.parse(out);
    const binary = path.join(meta.target_directory, "debug", "uc");
    if (fs.existsSync(binary)) {
      console.log(`[setup] Found pre-built binary: ${binary}`);
      return { command: binary, args: [] };
    }
    console.log(`[setup] Binary not found at ${binary}, building...`);

    // Build the binary first
    execSync("cargo build --bin uc", {
      cwd: workspaceRoot,
      stdio: "inherit",
      timeout: 180_000,
    });

    if (fs.existsSync(binary)) {
      console.log(`[setup] Built binary: ${binary}`);
      return { command: binary, args: [] };
    }
  } catch (err) {
    console.log(
      `[setup] Binary resolution failed, falling back to cargo run: ${err}`,
    );
  }

  return { command: "cargo", args: ["run", "--bin", "uc", "--"] };
}

export default async function globalSetup() {
  const port = await findAvailablePort();
  const workspaceRoot = path.resolve(__dirname, "../../../..");
  const { command, args } = resolveServerBinary(workspaceRoot);

  console.log(
    `[setup] Starting server on port ${port}: ${command} ${args.join(" ")}`,
  );

  const proc: ChildProcess = spawn(
    command,
    [...args, "server", "--rest", "--port", String(port)],
    {
      cwd: workspaceRoot,
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, RUST_LOG: "uc=info,warn" },
    },
  );

  const baseUrl = await waitForServer(proc);

  console.log(`[setup] Server ready at ${baseUrl} (pid: ${proc.pid})`);

  fs.writeFileSync(STATE_FILE, JSON.stringify({ pid: proc.pid, baseUrl }));

  // Detach so the process doesn't keep Node alive during teardown
  proc.unref();
  (proc.stdout as NodeJS.ReadableStream & { unref?: () => void })?.unref?.();
  (proc.stderr as NodeJS.ReadableStream & { unref?: () => void })?.unref?.();
}

function waitForServer(proc: ChildProcess): Promise<string> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      proc.kill("SIGTERM");
      reject(
        new Error(`Server did not start within ${SERVER_READY_TIMEOUT_MS}ms`),
      );
    }, SERVER_READY_TIMEOUT_MS);

    let stderrBuf = "";
    let stdoutBuf = "";

    const tryMatch = (data: string, source: string) => {
      if (source === "stdout") stdoutBuf += data;
      else stderrBuf += data;

      const combined = stdoutBuf + stderrBuf;
      const match = combined.match(READY_PATTERN);
      if (match) {
        clearTimeout(timeout);
        const port = match[1];
        resolve(`http://127.0.0.1:${port}/api/2.1/unity-catalog`);
      }
    };

    proc.stdout?.on("data", (chunk: Buffer) =>
      tryMatch(chunk.toString(), "stdout"),
    );
    proc.stderr?.on("data", (chunk: Buffer) =>
      tryMatch(chunk.toString(), "stderr"),
    );

    proc.on("error", (err) => {
      clearTimeout(timeout);
      reject(new Error(`Failed to spawn server: ${err.message}`));
    });

    proc.on("exit", (code) => {
      clearTimeout(timeout);
      reject(
        new Error(
          `Server exited prematurely with code ${code}\nstdout: ${stdoutBuf}\nstderr: ${stderrBuf}`,
        ),
      );
    });
  });
}
