#!/usr/bin/env node

// Shebang above is helpful so it can run as an executable on Unix-like systems

const HELP_MESSAGE = `
Logseq Gardener Patch Manager (lgpm) v0.1.0

Usage:
  lgpm [command]

Available Commands:
  help    Show this help message
  version Show version info

For now, this is a placeholder CLI.
`;

function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === "help") {
    console.log(HELP_MESSAGE);
  } else if (args[0] === "version") {
    console.log("logseq-patch-manager version 0.1.0");
  } else {
    console.log("Unknown command:", args[0]);
    console.log("Try 'lpm help'.");
  }
}

main();
