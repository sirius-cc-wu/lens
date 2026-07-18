import { spawn } from "node:child_process";
import { once } from "node:events";

export default async function globalSetup() {
  const child = spawn(
    "cargo",
    ["build", "--locked", "--bin", "lens", "--message-format=json-render-diagnostics"],
    { stdio: ["ignore", "pipe", "inherit"] },
  );
  let output = "";
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    output += chunk;
  });

  const [status, signal] = await once(child, "close");
  if (status !== 0) {
    throw new Error(`Cargo failed to build Lens (status ${status}, signal ${signal})`);
  }

  const executable = output
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line))
    .find(
      (message) =>
        message.reason === "compiler-artifact" &&
        message.target.name === "lens" &&
        message.target.kind.includes("bin") &&
        message.executable,
    )?.executable;
  if (!executable) {
    throw new Error("Cargo did not report the Lens executable path");
  }
  process.env.LENS_BROWSER_TEST_BINARY = executable;
}
