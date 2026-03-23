import fs from "node:fs";
import path from "node:path";
import url from "node:url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");
const source = path.join(projectRoot, "src");
const target = path.join(projectRoot, "dist");
const tauriBuildRoot = path.join(projectRoot, "src-tauri", "target", "debug", "build");

fs.rmSync(target, { recursive: true, force: true });
fs.mkdirSync(target, { recursive: true });

for (const entry of fs.readdirSync(source, { withFileTypes: true })) {
  if (!entry.isFile()) {
    continue;
  }

  fs.copyFileSync(path.join(source, entry.name), path.join(target, entry.name));
}

const globalApiPointer = findFile(tauriBuildRoot, "__global-api-script.js");
if (globalApiPointer) {
  const pointerContent = fs.readFileSync(globalApiPointer, "utf8").trim();
  const [globalApiSource] = JSON.parse(pointerContent);
  if (globalApiSource && fs.existsSync(globalApiSource)) {
    fs.copyFileSync(globalApiSource, path.join(target, "tauri-global.js"));
  }
}

console.log(`Copied static frontend to ${target}`);

function findFile(dir, fileName) {
  if (!fs.existsSync(dir)) {
    return null;
  }

  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const nested = findFile(entryPath, fileName);
      if (nested) {
        return nested;
      }
      continue;
    }

    if (entry.isFile() && entry.name === fileName) {
      return entryPath;
    }
  }

  return null;
}

