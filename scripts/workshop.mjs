// The theme workshop catalogue: public/workshop/themes/<id>/ (theme.json + assets/…) and the index
// the app's gallery reads, public/workshop/index.json.
//
//   node scripts/workshop.mjs index                checks every theme and writes the index (run by the build,
//                                                  which fails when a theme does not pass the check)
//   node scripts/workshop.mjs add <zip or URL>     adds a submitted theme pack after review (the URL of the
//                                                  .zip attached to the GitHub submission works);
//                                                  --replace overwrites a theme with the same id
//   node scripts/workshop.mjs check <zip or URL>   prints a review report (Markdown) without adding anything;
//                         check --from-issue       finds the .zip attached to a submission in $ISSUE_BODY
//                                                  (the theme-check workflow comments the report on the issue)
//
// Every theme goes through normalizeTheme, the same check as an import in the app, and must pass it
// without dropping anything, so what the gallery shows is exactly what was reviewed.
import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync, rmSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
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

// Downloads stop at this size, and a pack may not unpack to more (a zip can claim huge contents)
const MAX_PACK = 30 * 1024 * 1024;

// The .zip attached to a submission: GitHub puts attachments in the body as links to user-attachments
export function findPackUrl(body) {
  const links = String(body || "").match(/https:\/\/(?:github\.com\/user-attachments\/files|(?:[\w-]+\.)?githubusercontent\.com)\/[^\s)"'<>`]+\.zip/gi);
  return links?.[0] ?? null;
}

class PackError extends Error {}

async function download(url) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new PackError(`the download failed (HTTP ${res.status})`);
  if (Number(res.headers.get("content-length")) > MAX_PACK) throw new PackError("the file is too big");
  const bytes = new Uint8Array(await res.arrayBuffer());
  if (bytes.length > MAX_PACK) throw new PackError("the file is too big");
  return bytes;
}

// A theme pack's bytes -> { theme, problems, assets: [{ name, data }] }; PackError when it is not one
export function readPack(bytes) {
  let entries;
  let total = 0;
  try {
    const files = unzipSync(bytes, { filter: (f) => (total += f.originalSize) <= MAX_PACK });
    if (total > MAX_PACK) throw new PackError("the pack unpacks to too much data");
    entries = Object.entries(files).filter(([path]) => !path.endsWith("/") && !path.includes("__MACOSX/") && !path.split("/").pop().startsWith("."));
  } catch (err) {
    throw err instanceof PackError ? err : new PackError("this is not a .zip file");
  }
  const meta = entries.find(([path]) => path.split("/").pop().toLowerCase() === "theme.json");
  if (!meta) throw new PackError("the pack has no theme.json");
  const base = meta[0].includes("/") ? meta[0].slice(0, meta[0].lastIndexOf("/") + 1) : "";
  const assets = entries.filter(([path]) => path !== meta[0]).map(([path, data]) => ({ name: path.startsWith(base) ? path.slice(base.length) : path, data }));
  let raw;
  try {
    raw = JSON.parse(new TextDecoder().decode(meta[1]));
  } catch {
    throw new PackError("theme.json is not valid JSON");
  }
  const { theme, problems } = normalizeTheme(raw, { assets: assets.map((a) => ({ name: a.name, size: a.data.length })) });
  if (!theme) throw new PackError("theme.json is not a theme");
  return { theme, problems, assets };
}

const loadPack = async (source) => readPack(/^https?:\/\//.test(source) ? await download(source) : new Uint8Array(readFileSync(source)));
const blocking = (problems) => problems.filter((p) => p.code !== "unused_file");

async function add(source, replace) {
  if (!source) fail("give the theme pack: a .zip file or its URL");
  let pack;
  try {
    pack = await loadPack(source);
  } catch (err) {
    fail(err instanceof PackError ? err.message : String(err));
  }
  const { theme, problems, assets } = pack;
  const dropped = blocking(problems);
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

const READABILITY = {
  text_ground: "white text is hard to read on the background",
  text_panel: "white text is hard to read on the cards",
  accent_text: "black text is hard to read on highlight buttons",
  primary_text: "white text is hard to read on main-colour buttons",
  image: "the background picture is very visible behind text",
};
const kb = (n) => `${Math.max(1, Math.round(n / 1024))} KB`;
// Text from a submission, safe in a comment: no Markdown or HTML, no table breaks, no @-mentions
export const md = (text) => String(text).replace(/[\\`*_[\]<>|#~]/g, (c) => `\\${c}`).replace(/@/g, "@\u200b").replace(/\s+/g, " ");

// The review report for a submission, as Markdown (marker: the workflow updates its own comment)
export function report({ source, pack, error, exists }) {
  const lines = ["<!-- workshop-check -->", "### Theme check", ""];
  if (!source) return [...lines, "❌ No theme file found. Drag the downloaded `theme-….zip` into the issue (edit it), and the check runs again."].join("\n");
  if (error) return [...lines, `❌ The attached file could not be checked: ${error}.`, "", "Download the theme again from the theme editor (Share to the workshop) and attach the new file."].join("\n");
  const { theme, problems, assets } = pack;
  const dropped = blocking(problems);
  const readability = readabilityIssues(theme);
  const used = new Set(themeFiles(theme));
  lines.push(dropped.length ? "❌ **Does not pass.** These parts would be left out:" : readability.length ? "⚠️ **Passes, but may be hard to read on a TV.**" : "✅ **Passes the check.**", "");
  dropped.forEach((p) => lines.push(`- \`${p.code}\`${p.detail ? `: ${md(p.detail)}` : ""}`));
  if (dropped.length) lines.push("");
  lines.push("| | |", "| --- | --- |",
    `| Name | ${Object.entries(theme.name).map(([l, n]) => `${md(n)} (${l})`).join(", ")} |`,
    `| Author | ${theme.author ? md(theme.author) : "—"} |`,
    `| Id | \`${theme.id}\`${exists ? " ⚠️ already in the workshop: adding it replaces that theme" : ""} |`,
    `| Tags | ${theme.tags.map((t) => `#${t}`).join(" ") || "—"} |`,
    `| Fonts | ${["title", "body"].map((r) => theme.fonts[r] ? `${r}: ${theme.fonts[r].family || `file ${md(theme.fonts[r].file)}`}` : null).filter(Boolean).join(", ") || "Classic"} |`,
    `| Scenery, decoration | ${theme.background.ornament || "—"}, ${theme.decoration.type || "—"} |`,
    `| Files | ${assets.filter((a) => used.has(a.name)).map((a) => `${md(a.name)} (${kb(a.data.length)})`).join(", ") || "—"} |`, "");
  if (readability.length) lines.push(`Readability: ${readability.map((c) => READABILITY[c]).join("; ")}.`, "");
  const unused = problems.filter((p) => p.code === "unused_file");
  if (unused.length) lines.push(`Not used by the theme (left out when added): ${unused.map((p) => md(p.detail)).join(", ")}.`, "");
  if (!dropped.length) lines.push("To review it, download the file and import it in a quiz (Look → Import theme). To add it:", "", "```", `npm run workshop:add ${source}${exists ? " --replace" : ""}`, "```");
  return lines.join("\n");
}

async function check(source) {
  if (!source) return console.log(report({ source: null }));
  try {
    const pack = await loadPack(source);
    console.log(report({ source, pack, exists: existsSync(join(themesDir, pack.theme.id)) }));
  } catch (err) {
    console.log(report({ source, error: err instanceof PackError ? err.message : String(err) }));
  }
}

// Run as a command (tests import the helpers above)
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, ...args] = process.argv.slice(2);
  const source = args.find((a) => !a.startsWith("--"));
  if (command === "index") writeIndex();
  else if (command === "add") await add(source, args.includes("--replace"));
  else if (command === "check") await check(args.includes("--from-issue") ? findPackUrl(process.env.ISSUE_BODY) : source);
  else fail("usage: node scripts/workshop.mjs index | add <zip or URL> [--replace] | check <zip or URL> | check --from-issue");
}
