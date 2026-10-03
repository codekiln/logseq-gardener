import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import mldoc from "mldoc";

const require = createRequire(import.meta.url);
const packageJson = require("mldoc/package.json");

const root = path.dirname(fileURLToPath(import.meta.url));
const fixtureRoot = path.join(root, "fixtures", "garden");
const snapshotPath = path.join(root, "mldoc-1.5.9.json");
const update = process.argv.length === 3 && process.argv[2] === "--update";

if (process.argv.length > (update ? 3 : 2)) {
  console.error("Usage: node compare.mjs [--update]");
  process.exit(2);
}

const config = {
  toc: false,
  parse_outline_only: false,
  heading_number: false,
  keep_line_break: true,
  format: "Markdown",
  heading_to_list: false,
};
const configJson = JSON.stringify(config);

async function markdownPaths(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await markdownPaths(fullPath)));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(fullPath);
    }
  }
  return files.sort();
}

const files = {};
for (const file of await markdownPaths(fixtureRoot)) {
  const relativePath = path.relative(fixtureRoot, file).split(path.sep).join("/");
  const source = await readFile(file);
  const markdown = source.toString("utf8");
  files[relativePath] = {
    sha256: createHash("sha256").update(source).digest("hex"),
    ast: JSON.parse(mldoc.Mldoc.parseJson(markdown, configJson)),
    references: JSON.parse(mldoc.Mldoc.getReferences(markdown, configJson)),
  };
}

const actual = {
  schema: 1,
  parser: { package: "mldoc", version: packageJson.version },
  config,
  files,
};
const serialized = `${JSON.stringify(actual, null, 2)}\n`;

if (update) {
  await writeFile(snapshotPath, serialized);
  console.log(`Updated ${path.basename(snapshotPath)} for ${Object.keys(files).length} fixtures.`);
} else {
  const expected = await readFile(snapshotPath, "utf8");
  if (expected !== serialized) {
    const expectedFiles = JSON.parse(expected).files;
    const changed = [...new Set([...Object.keys(expectedFiles), ...Object.keys(files)])]
      .filter((name) => JSON.stringify(expectedFiles[name]) !== JSON.stringify(files[name]));
    console.error(`mldoc baseline differs${changed.length ? `: ${changed.join(", ")}` : " in metadata"}. Run mise run parser:baseline-update from the repository root and review the diff.`);
    process.exitCode = 1;
  } else {
    console.log(`mldoc baseline matches ${Object.keys(files).length} fixtures.`);
  }
}
