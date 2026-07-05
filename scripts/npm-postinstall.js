const { spawnSync } = require("node:child_process");
const path = require("node:path");

if (process.env.PRACTICODE_SKIP_BUILD === "1") {
  process.exit(0);
}

const root = path.resolve(__dirname, "..");
const build = spawnSync("cargo", ["build", "--release", "--locked"], {
  cwd: root,
  stdio: "inherit",
});

if (build.error) {
  console.warn(`practicode: cargo build skipped: ${build.error.message}`);
  console.warn("practicode: install Rust/Cargo before first run, or set PRACTICODE_SKIP_BUILD=1.");
  process.exit(0);
}

process.exit(build.status ?? 1);
