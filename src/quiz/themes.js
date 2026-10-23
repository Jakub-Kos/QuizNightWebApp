import { createContext, useContext } from "react";

// The look of a quiz's show: colours, title font, background and decorations. A theme is data only
// (never CSS or code), so themes shared by other people are safe to open. The scenes use named colours
// (Tailwind tokens in index.css): primary-50…950 (the main colour, blue by default), accent-50…950
// (highlights and buttons, yellow by default), ground / ground-top / ground-bottom (the background
// gradient) and panel (cards). Gold for first place, green and red for right and wrong, and the
// host's controls keep their own colours. ThemeScope sets the variables for a theme; scenes draw their
// background with SceneBackdrop, the place a theme can later change a scene's whole layout.
//
// theme: {
//   id, name: { en, cs, sk },
//   primary, accent        base colours (500), the other shades are mixed from them; omitted = default
//   ground: { top, base, bottom }, panel
//   font: { title, body }  font families for headings and for all other show text (numbers keep their
//                          monospace font); fonts in FONT_LOADERS are bundled
//   glows: [{ color, x, y, size, opacity }]   soft coloured light behind the scenes (x, y, size in %)
//   ornament               "graveyard" | "lights" | null, drawn by SceneBackdrop on the calm scenes
//   decoration             "bats" | "snow" | null, the overlay running over every scene
// }

export const DEFAULT_THEME_ID = "default";

export const THEMES = [
  {
    id: "default",
    name: { en: "Classic", cs: "Klasický", sk: "Klasszikus" },
    glows: [],
    ornament: null,
    decoration: null,
  },
  {
    id: "halloween",
    name: { en: "Halloween", cs: "Halloween", sk: "Halloween" },
    primary: "#8b3dff",
    accent: "#ff7a1a",
    ground: { top: "#1d0c30", base: "#0b0614", bottom: "#050208" },
    panel: "#120a1d",
    font: { title: "Rubik Wet Paint", body: "Grenze" },
    glows: [
      { color: "#7c2bd9", x: 8, y: 92, size: 70, opacity: 0.24 },
      { color: "#ff6a00", x: 70, y: 105, size: 60, opacity: 0.14 },
      { color: "#3dd68c", x: 100, y: 40, size: 40, opacity: 0.05 },
    ],
    ornament: "graveyard",
    decoration: "bats",
  },
  {
    id: "christmas",
    name: { en: "Christmas", cs: "Vánoce", sk: "Karácsony" },
    primary: "#e0303a",
    accent: "#f5c542",
    ground: { top: "#0f2a1d", base: "#06150e", bottom: "#020805" },
    panel: "#0a1d14",
    font: { title: "Berkshire Swash", body: "Playfair Display" },
    glows: [
      { color: "#e0303a", x: 0, y: 100, size: 60, opacity: 0.14 },
      { color: "#f5c542", x: 100, y: 0, size: 50, opacity: 0.1 },
    ],
    ornament: "lights",
    decoration: "snow",
  },
];

export const themeById = (id) => THEMES.find((theme) => theme.id === id) || THEMES[0];

// Bundled fonts, loaded only when a theme uses them; all cover Czech and Slovak letters. Body fonts
// bring the weights the scenes use (regular, bold, black).
const FONT_LOADERS = {
  "Rubik Wet Paint": () => import("@fontsource/rubik-wet-paint/400.css"),
  "Berkshire Swash": () => import("@fontsource/berkshire-swash/400.css"),
  Grenze: () => Promise.all([import("@fontsource/grenze/400.css"), import("@fontsource/grenze/700.css"), import("@fontsource/grenze/900.css")]),
  "Playfair Display": () => Promise.all([import("@fontsource/playfair-display/400.css"), import("@fontsource/playfair-display/700.css"), import("@fontsource/playfair-display/900.css")]),
};
export function loadThemeFonts(theme) {
  for (const family of [theme?.font?.title, theme?.font?.body]) {
    if (family && FONT_LOADERS[family]) FONT_LOADERS[family]().catch(() => {}); // offline: the usual font
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
const rgbTriple = (hex) => {
  const h = hex.replace("#", "");
  const full = h.length === 3 ? [...h].map((c) => c + c).join("") : h;
  return [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16)).join(" ");
};
const darken = (hex, amount) => rgbTriple(hex).split(" ").map((v) => Math.round(v * (1 - amount))).join(" ");

// The CSS variables a theme changes (the default theme changes none)
export function themeStyle(theme) {
  const style = {};
  if (theme.primary) Object.assign(style, shades("primary", theme.primary), { "--glow-primary": rgbTriple(theme.primary), "--glow-primary-deep": darken(theme.primary, 0.55) });
  if (theme.accent) Object.assign(style, shades("accent", theme.accent), { "--glow-accent": rgbTriple(theme.accent) });
  if (theme.ground) Object.assign(style, { "--color-ground": theme.ground.base, "--color-ground-top": theme.ground.top, "--color-ground-bottom": theme.ground.bottom });
  if (theme.panel) style["--color-panel"] = theme.panel;
  if (theme.font?.body) style["--theme-font"] = `"${theme.font.body}", "League Spartan"`;
  if (theme.font?.title) style["--theme-title-font"] = `"${theme.font.title}", var(--theme-font, "League Spartan")`;
  return style;
}

export const ThemeContext = createContext(THEMES[0]);
export const useTheme = () => useContext(ThemeContext);
