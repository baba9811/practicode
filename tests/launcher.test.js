const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const {
  chmodSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  utimesSync,
  writeFileSync,
} = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const {
  assetFor,
  ensureBinary,
  npmUpdateCommand,
} = require("../bin/launcher.js");

const root = path.resolve(__dirname, "..");
const version = require("../package.json").version;

function temporaryDirectory(t) {
  const directory = mkdtempSync(path.join(os.tmpdir(), "practicode-launcher-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}

function digest(data) {
  return createHash("sha256").update(data).digest("hex");
}

function listen(handler) {
  return new Promise((resolve) => {
    const server = http.createServer(handler);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      resolve({ server, origin: `http://127.0.0.1:${port}` });
    });
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
    server.closeAllConnections?.();
  });
}

function seedCachedBinary(cacheDir, body) {
  const asset = assetFor(process.platform, process.arch);
  const binary = path.join(cacheDir, version, asset);
  mkdirSync(path.dirname(binary), { recursive: true });
  writeFileSync(binary, body);
  if (process.platform !== "win32") chmodSync(binary, 0o755);
  writeFileSync(`${binary}.sha256`, `${digest(body)}  ${asset}\n`);
  return binary;
}

test("maps every published platform and rejects unsupported targets", () => {
  assert.equal(assetFor("darwin", "x64"), "practicode-x86_64-apple-darwin");
  assert.equal(assetFor("darwin", "arm64"), "practicode-aarch64-apple-darwin");
  assert.equal(assetFor("linux", "x64"), "practicode-x86_64-unknown-linux-musl");
  assert.equal(assetFor("linux", "arm64"), "practicode-aarch64-unknown-linux-musl");
  assert.equal(assetFor("win32", "x64"), "practicode-x86_64-pc-windows-msvc.exe");
  assert.throws(() => assetFor("freebsd", "x64"), /Unsupported platform/);
  assert.throws(() => assetFor("win32", "arm64"), /Unsupported platform/);
});

test("uses the platform command required by the npm shim", () => {
  assert.deepEqual(npmUpdateCommand("linux", {}), {
    command: "npm",
    args: ["update", "-g", "practicode"],
  });
  assert.deepEqual(npmUpdateCommand("win32", { ComSpec: "C:\\Windows\\cmd.exe" }), {
    command: "C:\\Windows\\cmd.exe",
    args: ["/d", "/s", "/c", "npm.cmd update -g practicode"],
  });
});

test("follows redirects, verifies SHA-256, and installs atomically", async (t) => {
  const cacheDir = temporaryDirectory(t);
  const asset = assetFor(process.platform, process.arch);
  const body = Buffer.from("fixture executable\n");
  const { server, origin } = await listen((request, response) => {
    if (request.url === "/release/SHA256SUMS") {
      response.writeHead(302, { location: "/assets/SHA256SUMS" }).end();
    } else if (request.url === `/release/${asset}`) {
      response.writeHead(307, { location: `/assets/${asset}` }).end();
    } else if (request.url === "/assets/SHA256SUMS") {
      response.end(`${digest(body)}  ${asset}\n`);
    } else if (request.url === `/assets/${asset}`) {
      response.end(body);
    } else {
      response.writeHead(404).end();
    }
  });
  t.after(() => close(server));

  const binary = await ensureBinary({
    version,
    cacheDir,
    releaseBaseUrl: `${origin}/release`,
  });

  assert.deepEqual(readFileSync(binary), body);
  assert.equal(readFileSync(`${binary}.sha256`, "utf8"), `${digest(body)}  ${asset}\n`);
  assert.equal(readdirSync(path.dirname(binary)).some((name) => name.includes(".tmp-")), false);
  if (process.platform !== "win32") {
    assert.notEqual(require("node:fs").statSync(binary).mode & 0o111, 0);
  }
});

test("concurrent installs share one verified cache without partial files", async (t) => {
  const cacheDir = temporaryDirectory(t);
  const asset = assetFor(process.platform, process.arch);
  const body = Buffer.from("concurrent fixture executable\n");
  const { server, origin } = await listen((request, response) => {
    if (request.url === "/release/SHA256SUMS") {
      response.end(`${digest(body)}  ${asset}\n`);
    } else if (request.url === `/release/${asset}`) {
      setTimeout(() => response.end(body), 20);
    } else {
      response.writeHead(404).end();
    }
  });
  t.after(() => close(server));

  const binaries = await Promise.all(
    Array.from({ length: 8 }, () =>
      ensureBinary({ version, cacheDir, releaseBaseUrl: `${origin}/release` }),
    ),
  );

  assert.equal(new Set(binaries).size, 1);
  assert.deepEqual(readFileSync(binaries[0]), body);
  assert.equal(
    readFileSync(`${binaries[0]}.sha256`, "utf8"),
    `${digest(body)}  ${asset}\n`,
  );
  assert.equal(
    readdirSync(path.dirname(binaries[0])).some((name) => name.includes(".tmp-")),
    false,
  );
});

test("removes a download whose checksum does not match", async (t) => {
  const cacheDir = temporaryDirectory(t);
  const asset = assetFor(process.platform, process.arch);
  const body = Buffer.from("tampered executable\n");
  const { server, origin } = await listen((request, response) => {
    if (request.url === "/release/SHA256SUMS") {
      response.end(`${"0".repeat(64)}  ${asset}\n`);
    } else if (request.url === `/release/${asset}`) {
      response.end(body);
    } else {
      response.writeHead(404).end();
    }
  });
  t.after(() => close(server));

  await assert.rejects(
    ensureBinary({ version, cacheDir, releaseBaseUrl: `${origin}/release` }),
    /checksum mismatch/i,
  );
  const versionDir = path.join(cacheDir, version);
  assert.equal(existsSync(path.join(versionDir, asset)), false);
  assert.equal(
    existsSync(versionDir) && readdirSync(versionDir).some((name) => name.includes(".tmp-")),
    false,
  );
});

test("removes a binary download that exceeds the size limit", async (t) => {
  const cacheDir = temporaryDirectory(t);
  const asset = assetFor(process.platform, process.arch);
  const body = Buffer.from("fixture executable that is too large\n");
  const { server, origin } = await listen((request, response) => {
    if (request.url === "/release/SHA256SUMS") {
      response.end(`${digest(body)}  ${asset}\n`);
    } else if (request.url === `/release/${asset}`) {
      response.end(body);
    } else {
      response.writeHead(404).end();
    }
  });
  t.after(() => close(server));

  await assert.rejects(
    ensureBinary({
      version,
      cacheDir,
      releaseBaseUrl: `${origin}/release`,
      maxBinaryBytes: 8,
    }),
    /unexpectedly large/i,
  );
  const versionDir = path.join(cacheDir, version);
  assert.equal(
    existsSync(versionDir) && readdirSync(versionDir).some((name) => name.includes(".tmp-")),
    false,
  );
});

test("reuses a verified cached binary without network access", async (t) => {
  const cacheDir = temporaryDirectory(t);
  const expected = seedCachedBinary(cacheDir, Buffer.from("cached executable\n"));
  const stale = `${expected}.tmp-interrupted`;
  writeFileSync(stale, "partial");
  const old = new Date(Date.now() - 2 * 60 * 60 * 1000);
  utimesSync(stale, old, old);
  if (process.platform !== "win32") chmodSync(expected, 0o644);
  const actual = await ensureBinary({
    version,
    cacheDir,
    releaseBaseUrl: "http://127.0.0.1:1/unreachable",
  });
  assert.equal(actual, expected);
  assert.equal(existsSync(stale), false);
  if (process.platform !== "win32") {
    assert.notEqual(require("node:fs").statSync(actual).mode & 0o111, 0);
  }
});

test("reports an actionable offline error when no verified cache exists", async (t) => {
  const cacheDir = temporaryDirectory(t);
  await assert.rejects(
    ensureBinary({
      version,
      cacheDir,
      releaseBaseUrl: "http://127.0.0.1:1/unreachable",
      requestTimeoutMs: 200,
    }),
    /offline|network|download/i,
  );
});

test("forwards arguments and never invokes Cargo on the native path", { skip: process.platform === "win32" }, (t) => {
  const directory = temporaryDirectory(t);
  const cacheDir = path.join(directory, "cache");
  const argsFile = path.join(directory, "args.txt");
  const cargoMarker = path.join(directory, "cargo-ran");
  const fakeBin = path.join(directory, "bin");
  mkdirSync(fakeBin);
  const cargo = path.join(fakeBin, "cargo");
  writeFileSync(cargo, `#!/bin/sh\ntouch '${cargoMarker}'\nexit 99\n`);
  chmodSync(cargo, 0o755);
  seedCachedBinary(
    cacheDir,
    Buffer.from("#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PRACTICODE_ARGS_FILE\"\n"),
  );

  const result = spawnSync(
    process.execPath,
    [path.join(root, "bin", "practicode.js"), "--smoke", "two words"],
    {
      cwd: directory,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${fakeBin}${path.delimiter}${process.env.PATH || ""}`,
        PRACTICODE_ARGS_FILE: argsFile,
        PRACTICODE_CACHE_DIR: cacheDir,
        PRACTICODE_RELEASE_BASE_URL: "http://127.0.0.1:1/unreachable",
      },
      timeout: 5_000,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(argsFile, "utf8"), "--smoke\ntwo words\n");
  assert.equal(existsSync(cargoMarker), false);
});

test("native launcher identifies the cached binary as an npm install", { skip: process.platform === "win32" }, (t) => {
  const directory = temporaryDirectory(t);
  const cacheDir = path.join(directory, "cache");
  const methodFile = path.join(directory, "install-method.txt");
  seedCachedBinary(
    cacheDir,
    Buffer.from(
      "#!/bin/sh\nprintf '%s' \"$PRACTICODE_INSTALL_METHOD\" > \"$PRACTICODE_METHOD_FILE\"\n",
    ),
  );

  const result = spawnSync(process.execPath, [path.join(root, "bin", "practicode.js")], {
    cwd: directory,
    encoding: "utf8",
    env: {
      ...process.env,
      PRACTICODE_CACHE_DIR: cacheDir,
      PRACTICODE_METHOD_FILE: methodFile,
      PRACTICODE_RELEASE_BASE_URL: "http://127.0.0.1:1/unreachable",
    },
    timeout: 5_000,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(methodFile, "utf8"), "npm");
});

test("reserved native exit runs npm update and returns its status", { skip: process.platform === "win32" }, (t) => {
  const directory = temporaryDirectory(t);
  const cacheDir = path.join(directory, "cache");
  const fakeBin = path.join(directory, "bin");
  const npmArgsFile = path.join(directory, "npm-args.txt");
  mkdirSync(fakeBin);
  const npm = path.join(fakeBin, "npm");
  writeFileSync(
    npm,
    "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PRACTICODE_NPM_ARGS_FILE\"\nexit 17\n",
  );
  chmodSync(npm, 0o755);
  seedCachedBinary(cacheDir, Buffer.from("#!/bin/sh\nexit 42\n"));

  const result = spawnSync(process.execPath, [path.join(root, "bin", "practicode.js")], {
    cwd: directory,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fakeBin}${path.delimiter}${process.env.PATH || ""}`,
      PRACTICODE_CACHE_DIR: cacheDir,
      PRACTICODE_NPM_ARGS_FILE: npmArgsFile,
      PRACTICODE_RELEASE_BASE_URL: "http://127.0.0.1:1/unreachable",
    },
    timeout: 5_000,
  });

  assert.equal(existsSync(npmArgsFile), true, result.stderr);
  assert.equal(readFileSync(npmArgsFile, "utf8"), "update\n-g\npracticode\n");
  assert.equal(result.status, 17, result.stderr);
});

test("docker sandbox mounts only its data home writable", { skip: process.platform === "win32" }, (t) => {
  const directory = temporaryDirectory(t);
  const fakeBin = path.join(directory, "bin");
  const dockerArgsFile = path.join(directory, "docker-args.txt");
  mkdirSync(fakeBin);
  const docker = path.join(fakeBin, "docker");
  writeFileSync(
    docker,
    `#!/bin/sh
if [ "$1" = version ] || [ "$1" = build ]; then exit 0; fi
printf '%s\\n' "$@" > "$PRACTICODE_DOCKER_ARGS_FILE"
`,
  );
  chmodSync(docker, 0o755);

  const result = spawnSync(
    process.execPath,
    [path.join(root, "bin", "practicode.js"), "--docker", "--smoke"],
    {
      cwd: directory,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${fakeBin}${path.delimiter}${process.env.PATH || ""}`,
        PRACTICODE_DOCKER_ARGS_FILE: dockerArgsFile,
        PRACTICODE_HOME: path.join(directory, "data"),
      },
      timeout: 5_000,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  const mounts = readFileSync(dockerArgsFile, "utf8")
    .split("\n")
    .filter((arg) => arg.startsWith("type=bind,"));
  const workspace = mounts.find((arg) => arg.includes("target=/workspace"));
  const data = mounts.find((arg) => arg.includes("target=/data"));
  assert.match(workspace, /,readonly$/);
  assert.doesNotMatch(data, /,readonly(?:,|$)/);
});

test("the packed npm launcher runs from a verified fixture cache", { skip: process.platform === "win32" }, (t) => {
  const directory = temporaryDirectory(t);
  const packDir = path.join(directory, "pack");
  const installDir = path.join(directory, "install");
  const cacheDir = path.join(directory, "cache");
  const argsFile = path.join(directory, "packed-args.txt");
  mkdirSync(packDir);

  const packed = spawnSync("npm", ["pack", "--json", "--pack-destination", packDir], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(packed.status, 0, packed.stderr);
  const filename = JSON.parse(packed.stdout)[0].filename;
  const installed = spawnSync(
    "npm",
    ["install", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", installDir, path.join(packDir, filename)],
    { encoding: "utf8" },
  );
  assert.equal(installed.status, 0, installed.stderr);

  seedCachedBinary(
    cacheDir,
    Buffer.from("#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PRACTICODE_ARGS_FILE\"\n"),
  );
  const launcher = path.join(installDir, "node_modules", "practicode", "bin", "practicode.js");
  const result = spawnSync(process.execPath, [launcher, "packed", "smoke"], {
    cwd: directory,
    encoding: "utf8",
    env: {
      ...process.env,
      PRACTICODE_ARGS_FILE: argsFile,
      PRACTICODE_CACHE_DIR: cacheDir,
    },
    timeout: 5_000,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(argsFile, "utf8"), "packed\nsmoke\n");
});
