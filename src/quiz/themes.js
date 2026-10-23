import { createContext, useContext } from "react";
import { THEME_FORMAT, THEME_VERSION } from "./themeFormat";

// The look of a quiz's show, in the theme file format (themeFormat.js). The scenes use named colours
// (Tailwind tokens in index.css): primary-50…950 (the main colour, blue by default), accent-50…950
// (highlights and buttons, yellow by default), ground / ground-top / ground-bottom (the background
// gradient) and panel (cards). Gold for first place, green and red for right and wrong, and the
// host's controls keep their own colours. ThemeScope sets the variables for a theme; scenes draw their
// background with SceneBackdrop, the place a theme can later change a scene's whole layout.
// settings.theme is a built-in theme's id, or "custom" for the quiz's own theme (quiz.customTheme,
// its files stored with the quiz's media).

export const DEFAULT_THEME_ID = "default";
export const CUSTOM_THEME_ID = "custom";

const builtin = (fields) => ({
  format: THEME_FORMAT, version: THEME_VERSION, author: "Quiz Night", description: {}, tags: [],
  colors: {}, fonts: {}, decoration: { type: null }, scenes: {}, ...fields,
  background: { image: null, imageOpacity: 0.5, imageBlur: 0, glows: [], ornament: null, ...fields.background },
});

export const THEMES = [
  builtin({ id: "default", name: { en: "Classic", cs: "Klasický", sk: "Klasszikus" } }),
  builtin({
    id: "halloween",
    name: { en: "Halloween", cs: "Halloween", sk: "Halloween" },
    tags: ["halloween"],
    colors: { primary: "#8b3dff", accent: "#ff7a1a", groundTop: "#1d0c30", ground: "#0b0614", groundBottom: "#050208", panel: "#120a1d" },
    fonts: { title: { family: "Rubik Wet Paint" }, body: { family: "Grenze" } },
    background: {
      glows: [
        { color: "#7c2bd9", x: 8, y: 92, size: 70, opacity: 0.24 },
        { color: "#ff6a00", x: 70, y: 100, size: 60, opacity: 0.14 },
        { color: "#3dd68c", x: 100, y: 40, size: 40, opacity: 0.05 },
      ],
      ornament: "graveyard",
    },
    decoration: { type: "bats" },
  }),
  builtin({
    id: "christmas",
    name: { en: "Christmas", cs: "Vánoce", sk: "Karácsony" },
    tags: ["christmas"],
    colors: { primary: "#e0303a", accent: "#f5c542", groundTop: "#0f2a1d", ground: "#06150e", groundBottom: "#020805", panel: "#0a1d14" },
    fonts: { title: { family: "Berkshire Swash" }, body: { family: "Playfair Display" } },
    background: {
      glows: [
        { color: "#e0303a", x: 0, y: 100, size: 60, opacity: 0.14 },
        { color: "#f5c542", x: 100, y: 0, size: 50, opacity: 0.1 },
      ],
      ornament: "lights",
    },
    decoration: { type: "snow" },
  }),
];

export const themeById = (id) => THEMES.find((theme) => theme.id === id) || THEMES[0];

// The theme a quiz shows with these settings
export const themeForQuiz = (quiz, settings = quiz?.settings) =>
  settings?.theme === CUSTOM_THEME_ID && quiz?.customTheme ? quiz.customTheme : themeById(settings?.theme);

// The themes a quiz can pick: [{ key (the settings value), theme }]
export const themeChoices = (quiz) => [
  ...THEMES.map((theme) => ({ key: theme.id, theme })),
  ...(quiz?.customTheme ? [{ key: CUSTOM_THEME_ID, theme: quiz.customTheme }] : []),
];

export const themeName = (theme, lang) => theme?.name?.[lang] || theme?.name?.en || Object.values(theme?.name || {})[0] || "";

// Bundled fonts, loaded only when a theme uses them; all cover Czech and Slovak letters. Body fonts
// bring the weights the scenes use (regular, bold, black).
const FONT_LOADERS = {
  "Rubik Wet Paint": () => import("@fontsource/rubik-wet-paint/400.css"),
  "Berkshire Swash": () => import("@fontsource/berkshire-swash/400.css"),
  Grenze: () => Promise.all([import("@fontsource/grenze/400.css"), import("@fontsource/grenze/700.css"), import("@fontsource/grenze/900.css")]),
  "Playfair Display": () => Promise.all([import("@fontsource/playfair-display/400.css"), import("@fontsource/playfair-display/700.css"), import("@fontsource/playfair-display/900.css")]),
  "Bebas Neue": () => import("@fontsource/bebas-neue/400.css"),
  Pacifico: () => import("@fontsource/pacifico/400.css"),
  Bungee: () => import("@fontsource/bungee/400.css"),
  Monoton: () => import("@fontsource/monoton/400.css"),
  "Special Elite": () => import("@fontsource/special-elite/400.css"),
  Audiowide: () => import("@fontsource/audiowide/400.css"),
  Montserrat: () => Promise.all([import("@fontsource/montserrat/400.css"), import("@fontsource/montserrat/700.css"), import("@fontsource/montserrat/900.css")]),
  Nunito: () => Promise.all([import("@fontsource/nunito/400.css"), import("@fontsource/nunito/700.css"), import("@fontsource/nunito/900.css")]),
  Oswald: () => Promise.all([import("@fontsource/oswald/400.css"), import("@fontsource/oswald/700.css")]),
};

// A font file gets a family name of its own, so two themes' files never mix
const hash = (s) => {
  let h = 2166136261;
  for (const ch of String(s)) h = Math.imul(h ^ ch.codePointAt(0), 16777619);
  return (h >>> 0).toString(36);
};
export const fontFamily = (font) => (font?.file ? `qn-font-${hash(font.file)}` : font?.family || null);

const loadedFiles = new Map(); // url -> family
// resolveAsset(name): a URL for a file of the theme (a quiz media name)
export function loadThemeFonts(theme, resolveAsset) {
  for (const font of [theme?.fonts?.title, theme?.fonts?.body]) {
    if (font?.family && FONT_LOADERS[font.family]) FONT_LOADERS[font.family]().catch(() => {}); // offline: the usual font
    const url = font?.file && resolveAsset?.(font.file);
    if (url && !loadedFiles.has(url) && typeof FontFace !== "undefined") {
      loadedFiles.set(url, fontFamily(font));
      new FontFace(fontFamily(font), `url(${url})`).load().then((face) => document.fonts.add(face)).catch(() => loadedFiles.delete(url));
    }
  }
}

// Shades 50…950 from a base colour, mixed towards white and black like Tailwind's palettes
const LIGHTER = { 50: 10, 100: 20, 200: 36, 300: 56, 400: 78 };
const DARKER = { 600: 16, 700: 32, 800: 46, 900: 58, 950: 72 };
function shades(name, base) {
  const vars = { [`--color-${name}-500`]: base };
  Object.entries(LIGHTER).forEach(([step, amount]) => { vars[`--color-${name}-${step}`] = `color-mix(in oklab, ${base} ${amount}%, white)`; });
  Object.entries(DARKER).forEach(([step, amount]) => { vars[`--color-${name}-${step}`] = `color-mix(in oklab, ${base}, black ${amount}%)`; });
  return vars;
}

// "#8b3dff" -> "139 61 255", for the glows written as rgb(var(--glow-…) / alpha)
const rgbTriple = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)).join(" ");
const darken = (hex, amount) => rgbTriple(hex).split(" ").map((v) => Math.round(v * (1 - amount))).join(" ");

// The CSS variables a theme changes (Classic changes none)
export function themeStyle(theme) {
  const c = theme.colors || {};
  const style = {};
  if (c.primary) Object.assign(style, shades("primary", c.primary), { "--glow-primary": rgbTriple(c.primary), "--glow-primary-deep": darken(c.primary, 0.55) });
  if (c.accent) Object.assign(style, shades("accent", c.accent), { "--glow-accent": rgbTriple(c.accent) });
  if (c.ground) style["--color-ground"] = c.ground;
  if (c.groundTop) style["--color-ground-top"] = c.groundTop;
  if (c.groundBottom) style["--color-ground-bottom"] = c.groundBottom;
  if (c.panel) style["--color-panel"] = c.panel;
  const body = fontFamily(theme.fonts?.body);
  const title = fontFamily(theme.fonts?.title);
  if (body) style["--theme-font"] = `"${body}", "League Spartan"`;
  if (title) style["--theme-title-font"] = `"${title}", var(--theme-font, "League Spartan")`;
  return style;
}

// The theme being shown, with imageUrl: its background picture resolved to a URL
export const ThemeContext = createContext({ ...THEMES[0], imageUrl: null });
export const useTheme = () => useContext(ThemeContext);
