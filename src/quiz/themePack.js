import { zipSync, unzipSync, strToU8 } from "fflate";
import { normalizeTheme, themeFiles, renameThemeFiles, slug } from "./themeFormat";

// A theme pack is a .zip with theme.json and the files it refers to (assets/…), or theme.json alone.
// Imported into a quiz, the files become quiz media named theme-<file>, so they travel with the quiz
// (zip export, Drive sync) and never collide with question media.

const baseName = (path) => String(path).split("/").pop();
const isJunk = (path) => path.startsWith("__MACOSX/") || path.includes("/__MACOSX/") || baseName(path).startsWith(".");

export class ThemePackError extends Error {}

// files: the chosen File objects. Returns { theme, problems, assets: [{ name (path in the pack), blob }] };
// throws ThemePackError("no_theme") when there is no theme.json, ("bad_json") when it cannot be read.
export async function readThemePack(files) {
  const entries = [];
  for (const file of files) {
    if (/\.zip$/i.test(file.name)) {
      const unzipped = unzipSync(new Uint8Array(await file.arrayBuffer()));
      for (const [path, data] of Object.entries(unzipped)) {
        if (!path.endsWith("/") && !isJunk(path)) entries.push({ path, blob: new Blob([data]) });
      }
    } else {
      entries.push({ path: file.webkitRelativePath || file.name, blob: file });
    }
  }
  const meta = entries.find((e) => baseName(e.path).toLowerCase() === "theme.json") || (entries.length === 1 && /\.json$/i.test(entries[0].path) ? entries[0] : null);
  if (!meta) throw new ThemePackError("no_theme");
  let raw;
  try {
    raw = JSON.parse(await meta.blob.text());
  } catch {
    throw new ThemePackError("bad_json");
  }
  // Paths as written in theme.json are relative to it
  const root = meta.path.includes("/") ? meta.path.slice(0, meta.path.lastIndexOf("/") + 1) : "";
  const assets = entries.filter((e) => e !== meta).map((e) => ({ name: e.path.startsWith(root) ? e.path.slice(root.length) : e.path, blob: e.blob }));
  const { theme, problems } = normalizeTheme(raw, { assets: assets.map((a) => ({ name: a.name, size: a.blob.size })) });
  if (!theme) throw new ThemePackError("no_theme");
  const used = new Set(themeFiles(theme));
  return { theme, problems, assets: assets.filter((a) => used.has(a.name)) };
}

// The pack's files as quiz media: { theme (referring to the media names), media: [{ name, blob }] }
export function themeForQuizMedia({ theme, assets }) {
  const names = new Map();
  const taken = new Set();
  for (const a of assets) {
    const plain = baseName(a.name).replace(/^theme-/, ""); // files from My themes already carry it
    let name = `theme-${plain}`;
    for (let i = 2; taken.has(name.toLowerCase()); i++) name = `theme-${i}-${plain}`;
    taken.add(name.toLowerCase());
    names.set(a.name, name);
  }
  return {
    theme: renameThemeFiles(theme, (path) => names.get(path) ?? path),
    media: assets.map((a) => ({ name: names.get(a.name), blob: a.blob })),
  };
}

// A theme as a pack: getFile(name) gives the Blob of each file it refers to (null when missing)
export async function exportThemePack(theme, getFile) {
  const files = {};
  const rename = new Map();
  for (const name of themeFiles(theme)) {
    const blob = await getFile(name);
    if (!blob) continue;
    const path = `assets/${baseName(name).replace(/^theme-/, "")}`;
    rename.set(name, path);
    files[path] = new Uint8Array(await blob.arrayBuffer());
  }
  const packed = renameThemeFiles(theme, (name) => rename.get(name) ?? null);
  files["theme.json"] = strToU8(JSON.stringify(packed, null, 2));
  return new Blob([zipSync(files, { level: 0 })], { type: "application/zip" });
}

export const themePackName = (theme) => `theme-${slug(theme.id) || "theme"}.zip`;
