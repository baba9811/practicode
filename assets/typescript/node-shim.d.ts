declare const process: {
  stdout: { write(value: string): void };
};

declare function require(id: "node:fs"): {
  readFileSync(fd: number, encoding: "utf8"): string;
};
