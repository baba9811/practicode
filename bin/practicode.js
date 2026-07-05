#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const exe = path.join(
  root,
  "target",
  "release",
  process.platform === "win32" ? "practicode.exe" : "practicode",
);

if (!existsSync(exe)) {
  const build = spawnSync(
    "cargo",
    ["build", "--release", "--locked", "--manifest-path", path.join(root, "Cargo.toml")],
    { stdio: "inherit" },
  );
  if (build.error) {
    console.error(`practicode: failed to run cargo: ${build.error.message}`);
    process.exit(1);
  }
  if (build.status !== 0) {
    process.exit(build.status ?? 1);
  }
}

const run = spawnSync(exe, process.argv.slice(2), {
  cwd: process.cwd(),
  stdio: "inherit",
});

if (run.error) {
  console.error(`practicode: failed to run binary: ${run.error.message}`);
  process.exit(1);
}
process.exit(run.status ?? 1);
