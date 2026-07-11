#!/usr/bin/env node

const { main } = require("./launcher.js");

main(process.argv.slice(2)).then(
  (status) => {
    process.exitCode = status;
  },
  (error) => {
    console.error(`practicode: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  },
);
