#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const { existsSync, mkdirSync } = require("node:fs");
const { homedir } = require("node:os");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const packageJson = require(path.join(root, "package.json"));
const dockerImage = `practicode-sandbox:${packageJson.version}`;
const exe = path.join(
  root,
  "target",
  "release",
  process.platform === "win32" ? "practicode.exe" : "practicode",
);
const args = process.argv.slice(2);

function rustInstallCommand() {
  if (process.platform === "win32") {
    return "winget install -e --id Rustlang.Rustup";
  }
  if (process.platform === "darwin") {
    return "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh";
  }
  return "sudo apt update && sudo apt install -y curl build-essential && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh";
}

function printRustInstallHelp() {
  console.error("practicode: Rust/Cargo is required to build the binary on first run.");
  console.error(`Install Rust: ${rustInstallCommand()}`);
  console.error("Then restart your terminal and run practicode again.");
  console.error("More options: https://www.rust-lang.org/tools/install");
}

function dockerInstallCommand() {
  if (process.platform === "win32") {
    return "winget install -e --id Docker.DockerDesktop";
  }
  if (process.platform === "darwin") {
    return "brew install --cask docker";
  }
  return "Ubuntu: https://docs.docker.com/engine/install/ubuntu/ | Debian: https://docs.docker.com/engine/install/debian/";
}

function printDockerHelp() {
  console.error("practicode: Docker is required for --docker sandbox mode.");
  console.error(`Install Docker: ${dockerInstallCommand()}`);
  console.error("Start Docker, then run practicode --docker again.");
  console.error("More options: https://docs.docker.com/get-docker/");
}

function runDockerSandbox(forwardedArgs) {
  const version = spawnSync("docker", ["version"], { stdio: "ignore" });
  if (version.error || version.status !== 0) {
    if (version.error) {
      console.error(`practicode: failed to run docker: ${version.error.message}`);
    }
    printDockerHelp();
    process.exit(1);
  }

  const build = spawnSync("docker", ["build", "-t", dockerImage, root], {
    stdio: "inherit",
  });
  if (build.error) {
    console.error(`practicode: failed to build Docker sandbox: ${build.error.message}`);
    printDockerHelp();
    process.exit(1);
  }
  if (build.status !== 0) {
    console.error("practicode: Docker sandbox build failed.");
    printDockerHelp();
    process.exit(build.status ?? 1);
  }

  const dataHome = path.resolve(
    process.env.PRACTICODE_HOME || path.join(homedir(), ".practicode"),
  );
  mkdirSync(dataHome, { recursive: true });

  const runArgs = [
    "run",
    "--rm",
    process.stdin.isTTY ? "-it" : "-i",
    "--init",
    "--network",
    "none",
    "--cpus",
    "2",
    "--memory",
    "1g",
    "--pids-limit",
    "256",
    "--cap-drop",
    "ALL",
    "--security-opt",
    "no-new-privileges",
    "--read-only",
    "--tmpfs",
    "/tmp:rw,nosuid,nodev,size=256m,mode=1777",
    "--mount",
    `type=bind,source=${process.cwd()},target=/workspace`,
    "--mount",
    `type=bind,source=${dataHome},target=/data`,
    "-w",
    "/workspace",
    "-e",
    `TERM=${process.env.TERM || "xterm-256color"}`,
    "-e",
    "HOME=/tmp",
    "-e",
    "PRACTICODE_HOME=/data",
    "-e",
    "PRACTICODE_NO_UPDATE_CHECK=1",
  ];
  if (process.env.COLORTERM) {
    runArgs.push("-e", `COLORTERM=${process.env.COLORTERM}`);
  }
  if (typeof process.getuid === "function" && typeof process.getgid === "function") {
    runArgs.push("--user", `${process.getuid()}:${process.getgid()}`);
  }
  runArgs.push(dockerImage, ...forwardedArgs);

  const run = spawnSync("docker", runArgs, {
    cwd: process.cwd(),
    stdio: "inherit",
  });
  if (run.error) {
    console.error(`practicode: failed to run Docker sandbox: ${run.error.message}`);
    printDockerHelp();
    process.exit(1);
  }
  process.exit(run.status ?? 1);
}

const dockerIndex = args.indexOf("--docker");
if (dockerIndex !== -1) {
  args.splice(dockerIndex, 1);
  runDockerSandbox(args);
}

if (!existsSync(exe)) {
  const build = spawnSync(
    "cargo",
    ["build", "--release", "--locked", "--manifest-path", path.join(root, "Cargo.toml")],
    { stdio: "inherit" },
  );
  if (build.error) {
    console.error(`practicode: failed to run cargo: ${build.error.message}`);
    printRustInstallHelp();
    process.exit(1);
  }
  if (build.status !== 0) {
    printRustInstallHelp();
    process.exit(build.status ?? 1);
  }
}

const run = spawnSync(exe, args, {
  cwd: process.cwd(),
  stdio: "inherit",
});

if (run.error) {
  console.error(`practicode: failed to run binary: ${run.error.message}`);
  process.exit(1);
}
process.exit(run.status ?? 1);
