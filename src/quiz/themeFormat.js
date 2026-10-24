// The theme file format. A theme is data only: colours, fonts, a background and a decoration picked from
// the ones the app draws itself, never CSS or code, so a theme shared by a stranger cannot do anything
// but change the look. normalizeTheme is the only way into the app: it keeps what it knows, in the
// ranges it allows, and reports everything it dropped.
//
// theme.json (version 1):
// {
//   "format": "quiznight-theme", "version": 1,
//   "id": "spooky-night",                       a-z, 0-9 and dashes
//   "name": { "en": "…", "cs": "…", "sk": "…" },  or one string for every language
//   "author": "…", "description": { … } or "…", "tags": ["halloween"],
//   "colors": { "primary", "accent", "groundTop", "ground", "groundBottom", "panel" },   "#rrggbb"
//   "fonts": { "title": { "family": "Rubik Wet Paint" } or { "file": "assets/font.woff2" }, "body": … },
//   "background": {
//     "image": "assets/bg.jpg", "imageOpacity": 0–1, "imageBlur": 0–20 (px on the 2400 px wide stage),
//     "glows": [{ "color", "x": 0–100, "y": 0–100, "size": 5–150, "opacity": 0–1 }],   at most 6
//     "ornament": "graveyard" | "lights" | "skyline" | "waves" | "meadow" | null
//   },
//   "decoration": { "type": "bats" | "snow" | "leaves" | "petals" | "hearts" | "confetti" | "bubbles" | "stars" | null },
//   "style": {                                   the shape of every screen (missing = Classic)
//     "corners": "sharp" | "rounded" | "round", "cards": "solid" | "glass" | "outline",
//     "headings": "spaced" | "plain", "effects": "normal" | "neon" | "flat"
//   },
//   "celebration": { "type": "confetti" | "fireworks" | "decoration" | null },   when the final standings appear
//   "sounds": { "pack": "chimes" | "spooky" | "festive" | "arcade" | null },     sounds the app makes itself
//   "scenes": {                                  per-scene looks
//     "leaderboard": { "layout": "table" | "podium" | "race" | "spotlight" | "medals" },
//     "dashboard": { "layout": "columns" | "tiles" | "timeline" },
//     "round": { "layout": "classic" | "curtain" | "slam" | "marquee" },        a round's title card
//     "welcome": { "intro": "flip" | "broadcast" | "spotlight" | "polaroid" }  the team wall's intros
//   }
// }
// Missing colours keep the Classic ones; files are paths inside the theme pack (themePack.js).

export const THEME_FORMAT = "quiznight-theme";
export const THEME_VERSION = 1;

// Fonts the app bundles (quiz/themes.js loads them), all with Czech and Slovak letters; others come as
// files in the pack. Display fonts are for headings, text fonts (with bold weights) for everything.
export const FONT_GROUPS = {
  display: ["Rubik Wet Paint", "Berkshire Swash", "Bebas Neue", "Pacifico", "Bungee", "Monoton", "Special Elite", "Audiowide"],
  text: ["Grenze", "Playfair Display", "Montserrat", "Nunito", "Oswald"],
};
export const BUNDLED_FONTS = [...FONT_GROUPS.display, ...FONT_GROUPS.text];
export const ORNAMENTS = ["graveyard", "lights", "skyline", "waves", "meadow"];
export const DECORATIONS = ["bats", "snow", "leaves", "petals", "hearts", "confetti", "bubbles", "stars"];
export const STYLE_OPTIONS = {
  corners: ["rounded", "sharp", "round"],
  cards: ["solid", "glass", "outline"],
  headings: ["spaced", "plain"],
  effects: ["normal", "neon", "flat"],
};
export const CELEBRATIONS = ["confetti", "fireworks", "decoration"];
export const SOUND_PACKS = ["chimes", "spooky", "festive", "arcade"];
export const LEADERBOARD_LAYOUTS = ["table", "podium", "race", "spotlight", "medals"];
export const DASHBOARD_LAYOUTS = ["columns", "tiles", "timeline"];
export const ROUND_LAYOUTS = ["classic", "curtain", "slam", "marquee"];
export const WELCOME_INTROS = ["flip", "broadcast", "spotlight", "polaroid"];
export const COLOR_KEYS = ["primary", "accent", "groundTop", "ground", "groundBottom", "panel"];
export const LANGS = ["en", "cs", "sk"];

export const IMAGE_TYPES = ["jpg", "jpeg", "png", "webp", "avif"];
export const FONT_TYPES = ["woff2", "woff", "ttf", "otf"];
export const LIMITS = { image: 8 * 1024 * 1024, font: 2 * 1024 * 1024, glows: 6, tags: 8, text: 80, description: 300 };

const isObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
const ext = (name) => String(name).split(".").pop().toLowerCase();
const clamp = (v, min, max, fallback) => (Number.isFinite(Number(v)) && v !== null && v !== "" ? Math.min(max, Math.max(min, Number(v))) : fallback);
const HEX = /^#[0-9a-f]{6}$/i;
const hex = (v) => {
  const s = String(v ?? "").trim();
  if (/^#[0-9a-f]{3}$/i.test(s)) return `#${[...s.slice(1)].map((c) => c + c).join("")}`.toLowerCase();
  return HEX.test(s) ? s.toLowerCase() : null;
};
const text = (v, max = LIMITS.text) => String(v ?? "").replace(/[\u0000-\u001f]/g, " ").trim().slice(0, max); // eslint-disable-line no-control-regex
export const slug = (v) => String(v ?? "").toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40);

// { en, cs, sk } from an object or one string
function texts(v, max) {
  if (typeof v === "string") {
    const s = text(v, max);
    return s ? Object.fromEntries(LANGS.map((l) => [l, s])) : {};
  }
  if (!isObject(v)) return {};
  return Object.fromEntries(LANGS.map((l) => [l, text(v[l], max)]).filter(([, s]) => s));
}

// raw: parsed theme.json; assets: [{ name, size }] in the pack (paths as written in the file). A theme
// already stored with a quiz (quiz.json, quiznight.json on Drive) refers to quiz media instead: stored
// accepts any file name of the right type, since such names only ever resolve to the quiz's own media.
// Returns { theme, problems: [{ code, detail }] }; theme is null when raw is not a theme at all.
export function normalizeTheme(raw, { assets = [], stored = false } = {}) {
  const problems = [];
  const problem = (code, detail = "") => problems.push({ code, detail: String(detail) });
  if (!isObject(raw) || raw.format !== THEME_FORMAT) return { theme: null, problems: [{ code: "format", detail: "" }] };
  if (Number(raw.version) > THEME_VERSION) problem("version", raw.version);

  const byName = new Map(assets.map((a) => [String(a.name).toLowerCase(), a]));
  const used = new Set();
  // A file the theme refers to: in the pack, of the right type and size
  const file = (path, types, limit) => {
    if (path == null || path === "") return null;
    const asset = stored ? { name: text(path, 200), size: 0 } : byName.get(String(path).toLowerCase());
    const code = !asset ? "missing_file" : !types.includes(ext(path)) ? "file_type" : asset.size > limit ? "file_size" : null;
    if (code) {
      problem(code, path);
      return null;
    }
    used.add(asset.name);
    return asset.name;
  };

  const name = texts(raw.name);
  const theme = {
    format: THEME_FORMAT,
    version: THEME_VERSION,
    id: slug(raw.id) || slug(name.en || name.cs || name.sk) || "theme",
    name: Object.keys(name).length ? name : { en: "Theme", cs: "Motiv", sk: "Téma" },
    author: text(raw.author),
    description: texts(raw.description, LIMITS.description),
    tags: (Array.isArray(raw.tags) ? raw.tags : []).map((t) => slug(t)).filter(Boolean).slice(0, LIMITS.tags),
    colors: {},
    fonts: {},
    background: { image: null, imageOpacity: 0.5, imageBlur: 0, glows: [], ornament: null },
    decoration: { type: null },
    style: {},
    celebration: { type: null },
    sounds: { pack: null },
    scenes: {},
  };

  const colors = isObject(raw.colors) ? raw.colors : {};
  for (const key of COLOR_KEYS) {
    if (colors[key] == null) continue;
    const value = hex(colors[key]);
    if (value) theme.colors[key] = value;
    else problem("color", key);
  }

  const fonts = isObject(raw.fonts) ? raw.fonts : {};
  for (const role of ["title", "body"]) {
    const f = fonts[role];
    if (!isObject(f)) continue;
    if (f.file != null) {
      const found = file(f.file, FONT_TYPES, LIMITS.font);
      if (found) theme.fonts[role] = { file: found };
    } else if (f.family != null) {
      const family = BUNDLED_FONTS.find((b) => b.toLowerCase() === String(f.family).trim().toLowerCase());
      if (family) theme.fonts[role] = { family };
      else problem("font", f.family);
    }
  }

  const bg = isObject(raw.background) ? raw.background : {};
  theme.background.image = file(bg.image, IMAGE_TYPES, LIMITS.image);
  theme.background.imageOpacity = clamp(bg.imageOpacity, 0, 1, 0.5);
  theme.background.imageBlur = clamp(bg.imageBlur, 0, 20, 0);
  const glows = Array.isArray(bg.glows) ? bg.glows : [];
  if (glows.length > LIMITS.glows) problem("glows", glows.length);
  theme.background.glows = glows.slice(0, LIMITS.glows).filter(isObject).flatMap((g) => {
    const color = hex(g.color);
    if (!color) {
      problem("color", "glow");
      return [];
    }
    return [{ color, x: clamp(g.x, 0, 100, 50), y: clamp(g.y, 0, 100, 50), size: clamp(g.size, 5, 150, 50), opacity: clamp(g.opacity, 0, 1, 0.2) }];
  });
  if (bg.ornament != null) {
    if (ORNAMENTS.includes(bg.ornament)) theme.background.ornament = bg.ornament;
    else problem("ornament", bg.ornament);
  }

  const deco = isObject(raw.decoration) ? raw.decoration : {};
  if (deco.type != null) {
    if (DECORATIONS.includes(deco.type)) theme.decoration.type = deco.type;
    else problem("decoration", deco.type);
  }

  // One of a list, or left out (with a problem when something unknown was asked for)
  const pick = (value, options, code) => {
    if (value == null) return undefined;
    if (options.includes(value)) return value;
    problem(code, value);
    return undefined;
  };
  const style = isObject(raw.style) ? raw.style : {};
  for (const [key, options] of Object.entries(STYLE_OPTIONS)) {
    const value = pick(style[key], options, "style");
    if (value) theme.style[key] = value;
  }
  theme.celebration.type = pick(isObject(raw.celebration) ? raw.celebration.type : null, CELEBRATIONS, "celebration") ?? null;
  theme.sounds.pack = pick(isObject(raw.sounds) ? raw.sounds.pack : null, SOUND_PACKS, "sounds") ?? null;
  const layout = pick(isObject(raw.scenes) && isObject(raw.scenes.leaderboard) ? raw.scenes.leaderboard.layout : null, LEADERBOARD_LAYOUTS, "layout");
  if (layout) theme.scenes.leaderboard = { layout };
  const board = pick(isObject(raw.scenes) && isObject(raw.scenes.dashboard) ? raw.scenes.dashboard.layout : null, DASHBOARD_LAYOUTS, "layout");
  if (board) theme.scenes.dashboard = { layout: board };
  const title = pick(isObject(raw.scenes) && isObject(raw.scenes.round) ? raw.scenes.round.layout : null, ROUND_LAYOUTS, "layout");
  if (title) theme.scenes.round = { layout: title };
  const intro = pick(isObject(raw.scenes) && isObject(raw.scenes.welcome) ? raw.scenes.welcome.intro : null, WELCOME_INTROS, "layout");
  if (intro) theme.scenes.welcome = { intro };

  if (!stored) for (const a of assets) if (!used.has(a.name)) problem("unused_file", a.name);
  return { theme, problems };
}

// Every file a theme refers to (paths in its pack, or media names once imported into a quiz)
export const themeFiles = (theme) => [theme?.fonts?.title?.file, theme?.fonts?.body?.file, theme?.background?.image].filter(Boolean);

// The same theme with its file references renamed (rename: old -> new, or null to leave the file out)
export function renameThemeFiles(theme, rename) {
  const fonts = Object.fromEntries(Object.entries(theme.fonts)
    .map(([role, f]) => [role, f.file ? { file: rename(f.file) } : f])
    .filter(([, f]) => f.family || f.file));
  return { ...theme, fonts, background: { ...theme.background, image: (theme.background.image && rename(theme.background.image)) || null } };
}

// Readability on a TV seen from across a room: WCAG contrast of the text the scenes put on the theme's
// colours. Returns codes for the editor: "text_ground" (white text on the background), "text_panel"
// (white text on cards), "accent_text" (black text on highlight buttons), "primary_text" (white text on
// main-colour buttons), "image" (a very visible background picture).
const CLASSIC = { primary: "#3b82f6", accent: "#eab308", groundTop: "#111111", ground: "#050505", groundBottom: "#000000", panel: "#0a0a0a" };
const luminance = (color) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(color.slice(i, i + 2), 16) / 255)
    .map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};
export const contrast = (a, b) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};
const shade = (color, amount) => `#${[1, 3, 5].map((i) => Math.round(parseInt(color.slice(i, i + 2), 16) * (1 - amount)).toString(16).padStart(2, "0")).join("")}`;

export function readabilityIssues(theme) {
  const c = { ...CLASSIC, ...theme?.colors };
  const issues = [];
  if ([c.groundTop, c.ground, c.groundBottom].some((g) => contrast("#ffffff", g) < 7)) issues.push("text_ground");
  if (contrast("#ffffff", c.panel) < 7) issues.push("text_panel");
  if (contrast("#000000", c.accent) < 4.5) issues.push("accent_text");
  if (contrast("#ffffff", shade(c.primary, 0.16)) < 3) issues.push("primary_text");
  if (theme?.background?.image && theme.background.imageOpacity > 0.6 && theme.background.imageBlur < 4) issues.push("image");
  return issues;
}
