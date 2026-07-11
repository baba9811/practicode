const { spawnSync } = require("node:child_process");
const { createHash, randomBytes } = require("node:crypto");
const {
  chmodSync,
  createReadStream,
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} = require("node:fs");
const http = require("node:http");
const https = require("node:https");
const { homedir } = require("node:os");
const path = require("node:path");
const { Transform } = require("node:stream");
const { pipeline } = require("node:stream/promises");

const REQUEST_TIMEOUT_MS = 15_000;
const MAX_REDIRECTS = 5;
const MAX_MANIFEST_BYTES = 1024 * 1024;
const STALE_TEMPORARY_FILE_MS = 60 * 60 * 1000;
const RELEASE_ROOT = "https://github.com/baba9811/practicode/releases/download";

function assetFor(platform, arch) {
  const targets = {
    "darwin-arm64": "practicode-aarch64-apple-darwin",
    "darwin-x64": "practicode-x86_64-apple-darwin",
    "linux-arm64": "practicode-aarch64-unknown-linux-musl",
    "linux-x64": "practicode-x86_64-unknown-linux-musl",
    "win32-x64": "practicode-x86_64-pc-windows-msvc.exe",
  };
  const asset = targets[`${platform}-${arch}`];
  if (!asset) {
    throw new Error(
      `Unsupported platform: ${platform}/${arch}. ` +
        "Use cargo install practicode or practicode --docker on a supported Docker host.",
    );
  }
  return asset;
}

function cacheDirectory(env = process.env, platform = process.platform, home = homedir()) {
  if (env.PRACTICODE_CACHE_DIR) return path.resolve(env.PRACTICODE_CACHE_DIR);
  if (platform === "win32") {
    return path.join(env.LOCALAPPDATA || path.join(home, "AppData", "Local"), "practicode");
  }
  if (platform === "darwin") return path.join(home, "Library", "Caches", "practicode");
  return path.join(env.XDG_CACHE_HOME || path.join(home, ".cache"), "practicode");
}

function checksumFor(manifest, asset) {
  for (const line of manifest.split(/\r?\n/)) {
    const match = line.match(/^([0-9a-fA-F]{64})\s+\*?(.+)$/);
    if (match && match[2] === asset) return match[1].toLowerCase();
  }
  throw new Error(`SHA256SUMS does not contain ${asset}`);
}

function sha256File(filename) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const input = createReadStream(filename);
    input.on("data", (chunk) => hash.update(chunk));
    input.on("error", reject);
    input.on("end", () => resolve(hash.digest("hex")));
  });
}

function responseFor(rawUrl, timeoutMs, redirectsLeft = MAX_REDIRECTS) {
  return new Promise((resolve, reject) => {
    const url = new URL(rawUrl);
    const client = url.protocol === "https:" ? https : url.protocol === "http:" ? http : null;
    if (!client) {
      reject(new Error(`Unsupported release URL protocol: ${url.protocol}`));
      return;
    }
    const request = client.get(
      url,
      { headers: { "user-agent": "practicode-npm-launcher" } },
      (response) => {
        const status = response.statusCode || 0;
        if (status >= 300 && status < 400 && response.headers.location) {
          response.resume();
          if (redirectsLeft === 0) {
            reject(new Error(`Too many redirects while downloading ${rawUrl}`));
            return;
          }
          const redirected = new URL(response.headers.location, url);
          if (url.protocol === "https:" && redirected.protocol !== "https:") {
            reject(new Error(`Refused insecure redirect for ${rawUrl}`));
            return;
          }
          responseFor(redirected, timeoutMs, redirectsLeft - 1).then(resolve, reject);
          return;
        }
        if (status !== 200) {
          response.resume();
          reject(new Error(`HTTP ${status} while downloading ${rawUrl}`));
          return;
        }
        resolve(response);
      },
    );
    request.setTimeout(timeoutMs, () => request.destroy(new Error(`Download timed out: ${rawUrl}`)));
    request.on("error", reject);
  });
}

async function downloadText(url, timeoutMs) {
  const response = await responseFor(url, timeoutMs);
  const chunks = [];
  let bytes = 0;
  for await (const chunk of response) {
    bytes += chunk.length;
    if (bytes > MAX_MANIFEST_BYTES) {
      response.destroy();
      throw new Error(`Checksum manifest is unexpectedly large: ${url}`);
    }
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString("utf8");
}

async function downloadFile(url, filename, timeoutMs) {
  const response = await responseFor(url, timeoutMs);
  const hash = createHash("sha256");
  const tap = new Transform({
    transform(chunk, _encoding, callback) {
      hash.update(chunk);
      callback(null, chunk);
    },
  });
  await pipeline(response, tap, createWriteStream(filename, { flags: "wx", mode: 0o600 }));
  return hash.digest("hex");
}

function temporaryName(filename) {
  return `${filename}.tmp-${process.pid}-${randomBytes(6).toString("hex")}`;
}

function cleanTemporaryFiles(versionDir, asset) {
  if (!existsSync(versionDir)) return;
  const prefixes = [`${asset}.tmp-`, `${asset}.sha256.tmp-`];
  for (const name of readdirSync(versionDir)) {
    const filename = path.join(versionDir, name);
    try {
      if (
        prefixes.some((prefix) => name.startsWith(prefix)) &&
        Date.now() - statSync(filename).mtimeMs > STALE_TEMPORARY_FILE_MS
      ) {
        rmSync(filename, { force: true });
      }
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
}

async function cachedBinaryIsValid(binary, asset) {
  const checksumFile = `${binary}.sha256`;
  if (!existsSync(binary) || !existsSync(checksumFile)) return false;
  try {
    const expected = checksumFor(readFileSync(checksumFile, "utf8"), asset);
    return (await sha256File(binary)) === expected;
  } catch {
    return false;
  }
}

async function ensureBinary({
  version,
  platform = process.platform,
  arch = process.arch,
  cacheDir = cacheDirectory(),
  releaseBaseUrl = `${RELEASE_ROOT}/v${version}`,
  requestTimeoutMs = REQUEST_TIMEOUT_MS,
}) {
  const asset = assetFor(platform, arch);
  const versionDir = path.join(cacheDir, version);
  const binary = path.join(versionDir, asset);
  cleanTemporaryFiles(versionDir, asset);
  if (await cachedBinaryIsValid(binary, asset)) {
    if (platform !== "win32") chmodSync(binary, 0o755);
    return binary;
  }

  mkdirSync(versionDir, { recursive: true });
  const temporaryBinary = temporaryName(binary);
  const temporaryChecksum = temporaryName(`${binary}.sha256`);

  try {
    const manifest = await downloadText(`${releaseBaseUrl}/SHA256SUMS`, requestTimeoutMs);
    const expected = checksumFor(manifest, asset);
    const actual = await downloadFile(`${releaseBaseUrl}/${asset}`, temporaryBinary, requestTimeoutMs);
    if (actual !== expected) {
      throw new Error(`Checksum mismatch for ${asset}: expected ${expected}, received ${actual}`);
    }
    if (platform !== "win32") chmodSync(temporaryBinary, 0o755);
    writeFileSync(temporaryChecksum, `${expected}  ${asset}\n`, { flag: "wx", mode: 0o600 });
    try {
      renameSync(temporaryBinary, binary);
    } catch (error) {
      if (existsSync(binary) && (await sha256File(binary)) === expected) {
        rmSync(temporaryBinary, { force: true });
      } else {
        rmSync(binary, { force: true });
        renameSync(temporaryBinary, binary);
      }
    }
    try {
      renameSync(temporaryChecksum, `${binary}.sha256`);
    } catch (error) {
      const installedChecksum = `${binary}.sha256`;
      if (
        existsSync(installedChecksum) &&
        checksumFor(readFileSync(installedChecksum, "utf8"), asset) === expected
      ) {
        rmSync(temporaryChecksum, { force: true });
      } else {
        rmSync(installedChecksum, { force: true });
        renameSync(temporaryChecksum, installedChecksum);
      }
    }
    return binary;
  } catch (error) {
    rmSync(temporaryBinary, { force: true });
    rmSync(temporaryChecksum, { force: true });
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Unable to download a verified Practicode ${version} binary. ${detail}\n` +
        "If you are offline, reconnect once to populate the cache, use practicode --docker, " +
        "or install explicitly with cargo install practicode.",
    );
  }
}

function dockerInstallCommand(platform = process.platform) {
  if (platform === "win32") return "winget install -e --id Docker.DockerDesktop";
  if (platform === "darwin") return "brew install --cask docker";
  return "https://docs.docker.com/engine/install/";
}

function printDockerHelp() {
  console.error("practicode: Docker is required for --docker sandbox mode.");
  console.error(`Install Docker: ${dockerInstallCommand()}`);
  console.error("Start Docker, then run practicode --docker again.");
}

function runDockerSandbox(root, version, forwardedArgs) {
  const dockerImage = `practicode-sandbox:${version}`;
  const dockerVersion = spawnSync("docker", ["version"], { stdio: "ignore" });
  if (dockerVersion.error || dockerVersion.status !== 0) {
    if (dockerVersion.error) {
      console.error(`practicode: failed to run docker: ${dockerVersion.error.message}`);
    }
    printDockerHelp();
    return 1;
  }

  const build = spawnSync("docker", ["build", "-t", dockerImage, root], { stdio: "inherit" });
  if (build.error || build.status !== 0) {
    if (build.error) console.error(`practicode: failed to build Docker sandbox: ${build.error.message}`);
    else console.error("practicode: Docker sandbox build failed.");
    printDockerHelp();
    return build.status || 1;
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
    `type=bind,source=${process.cwd()},target=/workspace,readonly`,
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
  if (process.env.COLORTERM) runArgs.push("-e", `COLORTERM=${process.env.COLORTERM}`);
  if (typeof process.getuid === "function" && typeof process.getgid === "function") {
    runArgs.push("--user", `${process.getuid()}:${process.getgid()}`);
  }
  runArgs.push(dockerImage, ...forwardedArgs);

  const run = spawnSync("docker", runArgs, { cwd: process.cwd(), stdio: "inherit" });
  if (run.error) {
    console.error(`practicode: failed to run Docker sandbox: ${run.error.message}`);
    printDockerHelp();
    return 1;
  }
  return run.status ?? 1;
}

async function main(args, root = path.resolve(__dirname, "..")) {
  const packageJson = require(path.join(root, "package.json"));
  const dockerIndex = args.indexOf("--docker");
  if (dockerIndex !== -1) {
    const forwarded = [...args];
    forwarded.splice(dockerIndex, 1);
    return runDockerSandbox(root, packageJson.version, forwarded);
  }

  const binary = await ensureBinary({
    version: packageJson.version,
    cacheDir: cacheDirectory(),
    releaseBaseUrl:
      process.env.PRACTICODE_RELEASE_BASE_URL || `${RELEASE_ROOT}/v${packageJson.version}`,
  });
  const run = spawnSync(binary, args, { cwd: process.cwd(), stdio: "inherit" });
  if (run.error) {
    throw new Error(`Failed to run cached Practicode binary: ${run.error.message}`);
  }
  return run.status ?? 1;
}

module.exports = {
  assetFor,
  cacheDirectory,
  checksumFor,
  ensureBinary,
  main,
  sha256File,
};
