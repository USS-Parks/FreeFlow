import { readdir, readFile } from "node:fs/promises";
import { extname, join, relative } from "node:path";

const root = process.cwd();
const sourceRoot = join(root, "src");
const allowedLocalStorageFile = "src/lib/utils/theme.ts";
const violations: string[] = [];

const forbidden = [
  /@tauri-apps\/plugin-(?:fs|sql|store|clipboard-manager)/,
  /navigator\.clipboard/,
  /\bfetch\s*\(/,
];

async function visit(directory: string): Promise<void> {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await visit(path);
      continue;
    }
    if (![".ts", ".tsx"].includes(extname(entry.name))) continue;

    const projectPath = relative(root, path).replaceAll("\\", "/");
    if (projectPath === "src/bindings.ts") continue;
    const text = await readFile(path, "utf8");
    const lines = text.split(/\r?\n/);
    for (const [index, line] of lines.entries()) {
      if (forbidden.some((pattern) => pattern.test(line))) {
        violations.push(`${projectPath}:${index + 1}: ${line.trim()}`);
      }
      if (
        line.includes("localStorage") &&
        projectPath !== allowedLocalStorageFile
      ) {
        violations.push(`${projectPath}:${index + 1}: ${line.trim()}`);
      }
    }
  }
}

await visit(sourceRoot);

if (violations.length > 0) {
  console.error("Frontend service-boundary violations found:");
  for (const violation of violations) console.error(`  ${violation}`);
  process.exit(1);
}

console.log("Frontend service-boundary gate passed.");
