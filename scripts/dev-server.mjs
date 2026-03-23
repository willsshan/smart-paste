import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import url from "node:url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");
const root = path.join(projectRoot, "src");
const tauriBuildRoot = path.join(projectRoot, "src-tauri", "target", "debug", "build");
const port = 1420;

const contentTypes = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml"
};

const server = http.createServer((req, res) => {
  const pathname = req.url === "/" ? "/index.html" : req.url ?? "/index.html";

  if (pathname === "/tauri-global.js") {
    const globalApiFile = resolveTauriGlobalApiFile();
    if (!globalApiFile) {
      res.writeHead(404);
      res.end("Tauri global bridge not found");
      return;
    }

    fs.readFile(globalApiFile, (error, data) => {
      if (error) {
        res.writeHead(500);
        res.end("Failed to read Tauri global bridge");
        return;
      }

      res.writeHead(200, {
        "Content-Type": contentTypes[".js"]
      });
      res.end(data);
    });
    return;
  }

  const filePath = path.join(root, pathname);

  if (!filePath.startsWith(root)) {
    res.writeHead(403);
    res.end("Forbidden");
    return;
  }

  fs.readFile(filePath, (error, data) => {
    if (error) {
      res.writeHead(404);
      res.end("Not found");
      return;
    }

    const ext = path.extname(filePath);
    res.writeHead(200, {
      "Content-Type": contentTypes[ext] ?? "text/plain; charset=utf-8"
    });
    res.end(data);
  });
});

server.listen(port, "127.0.0.1", () => {
  console.log(`AI Paste dev server running at http://127.0.0.1:${port}`);
});

function resolveTauriGlobalApiFile() {
  const pointerFile = findFile(tauriBuildRoot, "__global-api-script.js");
  if (!pointerFile) {
    return null;
  }

  try {
    const pointerContent = fs.readFileSync(pointerFile, "utf8").trim();
    const [globalApiSource] = JSON.parse(pointerContent);
    const normalized = globalApiSource?.replace(/^\\\\\?\\/, "");
    if (normalized && fs.existsSync(normalized)) {
      return normalized;
    }
  } catch {
    return null;
  }

  return null;
}

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
