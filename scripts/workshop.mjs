// The theme workshop catalogue: public/workshop/themes/<id>/ (theme.json + assets/…) and the index
// the app's gallery reads, public/workshop/index.json.
//
//   node scripts/workshop.mjs index                checks every theme and writes the index (run by the build,
//                                                  which fails when a theme does not pass the check)
//   node scripts/workshop.mjs add <zip or URL>     adds a submitted theme pack after review (the URL of the
//                                                  .zip attached to the GitHub submission works);
//                                                  --replace overwrites a theme with the same id
//
// Every theme goes through normalizeTheme, the same check as an import in the app, and must pass it
// without dropping anything, so what the gallery shows is exactly what was reviewed.
import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync, rmSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { unzipSync } from "fflate";
import { normalizeTheme, themeFiles, readabilityIssues } from "../src/quiz/themeFormat.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "workshop");
const themesDir = join(root, "themes");

const fail = (message) => {
  console.error(`✗ ${message}`);
  process.exit(1);
};
const describe = (problems) => problems.map((p) => `  - ${p.code}${p.detail ? `: ${p.detail}` : ""}`).join("\n");

// Every file below a folder, with paths relative to it
function listFiles(dir, prefix = "") {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? listFiles(path, `${prefix}${name}/`) : [`${prefix}${name}`];
  });
}

function checkFolder(id) {
  const dir = join(themesDir, id);
  const files = listFiles(dir).filter((f) => f !== "theme.json");
  let raw;
  try {
    raw = JSON.parse(readFileSync(join(dir, "theme.json"), "utf8"));
  } catch (err) {
    fail(`${id}: theme.json cannot be read (${err.message})`);
  }
  const { theme, problems } = normalizeTheme(raw, { assets: files.map((name) => ({ name, size: statSync(join(dir, name)).size })) });
  if (!theme) fail(`${id}: theme.json is not a theme`);
  if (problems.length) fail(`${id}: the theme does not pass the check:\n${describe(problems)}`);
  if (theme.id !== id) fail(`${id}: the folder must be named after the theme's id (${theme.id})`);
  const readability = readabilityIssues(theme);
  if (readability.length) console.warn(`! ${id}: hard to read: ${readability.join(", ")}`);
  return theme;
}

function writeIndex() {
  mkdirSync(themesDir, { recursive: true });
  const previous = existsSync(join(root, "index.json")) ? JSON.parse(readFileSync(join(root, "index.json"), "utf8")) : { themes: [] };
  const addedAt = new Map(previous.themes.map((entry) => [entry.theme.id, entry.addedAt]));
  const themes = readdirSync(themesDir).filter((id) => statSync(join(themesDir, id)).isDirectory()).sort().map((id) => {
    const theme = checkFolder(id);
    return { path: `themes/${id}/`, addedAt: addedAt.get(id) ?? new Date().toISOString().slice(0, 10), theme };
  });
  themes.sort((a, b) => b.addedAt.localeCompare(a.addedAt) || a.theme.id.localeCompare(b.theme.id));
  writeFileSync(join(root, "index.json"), `${JSON.stringify({ version: 1, themes }, null, 2)}\n`);
  console.log(`✓ ${themes.length} workshop theme${themes.length === 1 ? "" : "s"} checked, index written`);
}

async function add(source, replace) {
  if (!source) fail("give the theme pack: a .zip file or its URL");
  const bytes = /^https?:\/\//.test(source)
    ? new Uint8Array(await (await fetch(source).then((res) => (res.ok ? res : fail(`download failed: HTTP ${res.status}`)))).arrayBuffer())
    : new Uint8Array(readFileSync(source));
  let entries;
  try {
    entries = Object.entries(unzipSync(bytes)).filter(([path]) => !path.endsWith("/") && !path.includes("__MACOSX/") && !path.split("/").pop().startsWith("."));
  } catch {
    fail("this is not a .zip file");
  }
  const meta = entries.find(([path]) => path.split("/").pop().toLowerCase() === "theme.json");
  if (!meta) fail("the pack has no theme.json");
  const base = meta[0].includes("/") ? meta[0].slice(0, meta[0].lastIndexOf("/") + 1) : "";
  const assets = entries.filter(([path]) => path !== meta[0]).map(([path, data]) => ({ name: path.startsWith(base) ? path.slice(base.length) : path, data }));
  let raw;
  try {
    raw = JSON.parse(new TextDecoder().decode(meta[1]));
  } catch {
    fail("theme.json is not valid JSON");
  }
  const { theme, problems } = normalizeTheme(raw, { assets: assets.map((a) => ({ name: a.name, size: a.data.length })) });
  if (!theme) fail("theme.json is not a theme");
  const dropped = problems.filter((p) => p.code !== "unused_file");
  if (dropped.length) fail(`the theme does not pass the check (ask the author to fix it, or fix theme.json yourself):\n${describe(dropped)}`);

  const dir = join(themesDir, theme.id);
  if (existsSync(dir)) {
    if (!replace) fail(`a theme with the id "${theme.id}" is already in the workshop (add --replace to overwrite it)`);
    rmSync(dir, { recursive: true });
  }
  const used = new Set(themeFiles(theme));
  for (const a of assets.filter((asset) => used.has(asset.name))) {
    mkdirSync(dirname(join(dir, a.name)), { recursive: true });
    writeFileSync(join(dir, a.name), a.data);
  }
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "theme.json"), `${JSON.stringify(theme, null, 2)}\n`);
  console.log(`✓ added "${theme.name.en || theme.id}" by ${theme.author || "(no author)"} as public/workshop/themes/${theme.id}/`);
  writeIndex();
}

const [command, ...args] = process.argv.slice(2);
if (command === "index") writeIndex();
else if (command === "add") await add(args.find((a) => !a.startsWith("--")), args.includes("--replace"));
else fail("usage: node scripts/workshop.mjs index | add <zip or URL> [--replace]");
