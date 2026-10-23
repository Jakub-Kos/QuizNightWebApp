import { normalizeTheme, themeFiles } from "./themeFormat";
import { WORKSHOP_REPO } from "../config";

// The theme workshop: themes shared by others, reviewed and added to public/workshop/ with
// scripts/workshop.mjs, listed in public/workshop/index.json ({ version, themes: [{ path, addedAt, theme }] }).
// The gallery checks every theme again, like an import, before showing it.

const BASE = `${import.meta.env.BASE_URL}workshop/`;
let loading = null;

// [{ path, addedAt, theme }], newest first
export function loadWorkshop() {
  loading ??= fetch(`${BASE}index.json`, { cache: "no-cache" })
    .then((res) => (res.ok ? res.json() : Promise.reject(new Error(`HTTP ${res.status}`))))
    .then((index) => (index.themes || []).flatMap((entry) => {
      const theme = normalizeTheme(entry.theme, { stored: true }).theme;
      return theme && typeof entry.path === "string" && /^themes\/[a-z0-9-]+\/$/.test(entry.path) ? [{ path: entry.path, addedAt: String(entry.addedAt || ""), theme }] : [];
    }))
    .catch((err) => {
      loading = null; // try again next time
      throw err;
    });
  return loading;
}

export const workshopFileUrl = (entry, name) => `${BASE}${entry.path}${name}`;

// The theme with its files, as a theme pack (themePack.js) to store in a quiz or in My themes
export async function fetchWorkshopPack(entry) {
  const assets = await Promise.all(themeFiles(entry.theme).map(async (name) => {
    const res = await fetch(workshopFileUrl(entry, name));
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return { name, blob: await res.blob() };
  }));
  return { theme: entry.theme, assets };
}

// The GitHub form for sharing a theme, filled in with what the theme says about itself
export function shareUrl(theme, lang) {
  const pick = (texts) => texts?.[lang] || texts?.en || Object.values(texts || {})[0] || "";
  const params = new URLSearchParams({
    template: "theme.yml",
    title: `Theme: ${pick(theme.name)}`,
    name: pick(theme.name),
    author: theme.author || "",
    description: pick(theme.description),
    tags: (theme.tags || []).join(", "),
  });
  return `https://github.com/${WORKSHOP_REPO}/issues/new?${params}`;
}
