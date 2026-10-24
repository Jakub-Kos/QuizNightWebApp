import { describe, it, expect } from "vitest";
import { zipSync, strToU8 } from "fflate";
import { normalizeTheme, themeFiles, readabilityIssues, THEME_FORMAT } from "./themeFormat";
import { readThemePack, exportThemePack, themeForQuizMedia, ThemePackError } from "./themePack";
import { THEMES, themeStyle, themeForQuiz, themeChoices, CUSTOM_THEME_ID } from "./themes";

const base = (fields) => ({ format: THEME_FORMAT, version: 1, id: "spooky", name: "Spooky", ...fields });
const codes = (problems) => problems.map((p) => p.code);

describe("normalizeTheme", () => {
  it("refuses what is not a theme", () => {
    expect(normalizeTheme({ title: "Quiz" }).theme).toBeNull();
    expect(normalizeTheme(null).theme).toBeNull();
  });

  it("keeps the built-in themes exactly as they are", () => {
    for (const theme of THEMES) {
      const { theme: again, problems } = normalizeTheme(theme);
      expect(problems).toEqual([]);
      expect(again).toEqual(theme);
    }
  });

  it("keeps known values and drops the rest, saying what it dropped", () => {
    const { theme, problems } = normalizeTheme(base({
      colors: { primary: "#F80", accent: "orange", panel: "#123456", css: "body{}" },
      fonts: { title: { family: "comic sans" }, body: { family: "grenze" } },
      background: { ornament: "volcano", glows: Array.from({ length: 8 }, () => ({ color: "#ffffff", x: 500, opacity: -1 })) },
      decoration: { type: "<script>" },
      script: "alert(1)",
    }));
    expect(theme.colors).toEqual({ primary: "#ff8800", panel: "#123456" });
    expect(theme.fonts).toEqual({ body: { family: "Grenze" } });
    expect(theme.background.glows).toHaveLength(6);
    expect(theme.background.glows[0]).toEqual({ color: "#ffffff", x: 100, y: 50, size: 50, opacity: 0 });
    expect(theme.background.ornament).toBeNull();
    expect(theme.decoration.type).toBeNull();
    expect(theme).not.toHaveProperty("script");
    expect(codes(problems)).toEqual(expect.arrayContaining(["color", "font", "glows", "ornament", "decoration"]));
  });

  it("names a theme in every language from one string and makes a safe id", () => {
    const { theme } = normalizeTheme(base({ id: "../Strašidelná Noc!", name: "  Strašidelná noc  " }));
    expect(theme.id).toBe("strasidelna-noc");
    expect(theme.name).toEqual({ en: "Strašidelná noc", cs: "Strašidelná noc", sk: "Strašidelná noc" });
  });

  it("accepts files only when they are in the pack, of the right type and size", () => {
    const assets = [{ name: "assets/bg.jpg", size: 1000 }, { name: "assets/huge.png", size: 9e6 }, { name: "assets/f.woff2", size: 100 }, { name: "assets/notes.txt", size: 5 }];
    const { theme, problems } = normalizeTheme(base({
      background: { image: "assets/BG.jpg" },
      fonts: { title: { file: "assets/f.woff2" }, body: { file: "assets/missing.woff2" } },
    }), { assets });
    expect(theme.background.image).toBe("assets/bg.jpg");
    expect(theme.fonts).toEqual({ title: { file: "assets/f.woff2" } });
    expect(problems).toEqual(expect.arrayContaining([
      { code: "missing_file", detail: "assets/missing.woff2" },
      { code: "unused_file", detail: "assets/huge.png" },
      { code: "unused_file", detail: "assets/notes.txt" },
    ]));
    const big = normalizeTheme(base({ background: { image: "assets/huge.png" } }), { assets });
    expect(codes(big.problems)).toContain("file_size");
    const wrong = normalizeTheme(base({ background: { image: "assets/notes.txt" } }), { assets });
    expect(codes(wrong.problems)).toContain("file_type");
  });
});

describe("theme packs", () => {
  const file = (name, data) => new File([data], name);

  it("reads a zip with theme.json in a folder, keeping only the files it uses", async () => {
    const zip = zipSync({
      "night/theme.json": strToU8(JSON.stringify(base({ background: { image: "assets/bg.jpg" } }))),
      "night/assets/bg.jpg": new Uint8Array([1, 2, 3]),
      "night/assets/extra.png": new Uint8Array([4]),
      "__MACOSX/night/._bg.jpg": new Uint8Array([0]),
    });
    const pack = await readThemePack([file("night.zip", zip)]);
    expect(pack.theme.background.image).toBe("assets/bg.jpg");
    expect(pack.assets.map((a) => a.name)).toEqual(["assets/bg.jpg"]);
    expect(codes(pack.problems)).toEqual(["unused_file"]);
  });

  it("reads theme.json alone and explains files that are not themes", async () => {
    const pack = await readThemePack([file("x.json", JSON.stringify(base({ colors: { accent: "#00ff00" } })))]);
    expect(pack.theme.colors.accent).toBe("#00ff00");
    await expect(readThemePack([file("quiz.json", JSON.stringify({ title: "Quiz" }))])).rejects.toThrow(ThemePackError);
    await expect(readThemePack([file("theme.json", "{oops")])).rejects.toThrow("bad_json");
    await expect(readThemePack([file("a.csv", "x")])).rejects.toThrow("no_theme");
  });

  it("stores the files as theme-… media and exports them back as the same pack", async () => {
    const zip = zipSync({
      "theme.json": strToU8(JSON.stringify(base({ background: { image: "assets/bg.jpg" }, fonts: { title: { file: "fonts/bg.jpg.woff2" } } }))),
      "assets/bg.jpg": new Uint8Array([1, 2, 3]),
      "fonts/bg.jpg.woff2": new Uint8Array([9]),
    });
    const stored = themeForQuizMedia(await readThemePack([file("t.zip", zip)]));
    expect(themeFiles(stored.theme)).toEqual(["theme-bg.jpg.woff2", "theme-bg.jpg"]);
    expect(stored.media.map((m) => m.name).sort()).toEqual(["theme-bg.jpg", "theme-bg.jpg.woff2"]);

    const blobs = new Map(stored.media.map((m) => [m.name, m.blob]));
    const exported = await exportThemePack(stored.theme, (name) => blobs.get(name));
    const again = await readThemePack([file("again.zip", new Uint8Array(await exported.arrayBuffer()))]);
    expect(again.problems).toEqual([]);
    expect(again.theme.background.image).toBe("assets/bg.jpg");
    expect(again.theme.fonts.title).toEqual({ file: "assets/bg.jpg.woff2" });
  });

  it("leaves out files it cannot find when exporting", async () => {
    const theme = normalizeTheme(base({ background: { image: "theme-bg.jpg" }, fonts: { body: { file: "theme-f.woff2" } } }), { stored: true }).theme;
    const exported = await exportThemePack(theme, () => null);
    const again = await readThemePack([file("e.zip", new Uint8Array(await exported.arrayBuffer()))]);
    expect(again.theme.background.image).toBeNull();
    expect(again.theme.fonts).toEqual({});
  });
});

describe("themes in a quiz", () => {
  it("picks the built-in theme, or the quiz's own one", () => {
    const own = normalizeTheme(base({ colors: { accent: "#00ff00" } })).theme;
    expect(themeForQuiz({ settings: { theme: "christmas" } }).id).toBe("christmas");
    expect(themeForQuiz({ settings: {} }).id).toBe("default");
    expect(themeForQuiz({ settings: { theme: CUSTOM_THEME_ID }, customTheme: own })).toBe(own);
    expect(themeForQuiz({ settings: { theme: CUSTOM_THEME_ID } }).id).toBe("default");
    expect(themeChoices({ customTheme: own }).map((c) => c.key)).toEqual(["default", "halloween", "christmas", CUSTOM_THEME_ID]);
  });

  it("changes no colour for Classic and fills every shade for others", () => {
    expect(themeStyle(THEMES[0])).toEqual({});
    const style = themeStyle(THEMES[1]);
    expect(Object.keys(style).filter((k) => k.startsWith("--color-primary-"))).toHaveLength(11);
    expect(style["--glow-accent"]).toBe("255 122 26");
    expect(style["--theme-font"]).toContain("Grenze");
  });
});

describe("readability", () => {
  it("passes Classic and the built-in themes", () => {
    for (const theme of THEMES) expect(readabilityIssues(theme)).toEqual([]);
  });

  it("flags light backgrounds, dark highlights, light main colours and loud pictures", () => {
    expect(readabilityIssues({ colors: { ground: "#888888", panel: "#999999", accent: "#333333", primary: "#ffff66" } }))
      .toEqual(["text_ground", "text_panel", "accent_text", "primary_text"]);
    expect(readabilityIssues({ colors: {}, background: { image: "x.jpg", imageOpacity: 0.9, imageBlur: 0 } })).toEqual(["image"]);
    expect(readabilityIssues({ colors: {}, background: { image: "x.jpg", imageOpacity: 0.9, imageBlur: 8 } })).toEqual([]);
  });
});

describe("My themes", () => {
  it("keeps the names of files that are already theme files", () => {
    const theme = normalizeTheme(base({ background: { image: "theme-bg.jpg" } }), { stored: true }).theme;
    const stored = themeForQuizMedia({ theme, assets: [{ name: "theme-bg.jpg", blob: new Blob(["x"]) }] });
    expect(stored.theme.background.image).toBe("theme-bg.jpg");
    expect(stored.media.map((m) => m.name)).toEqual(["theme-bg.jpg"]);
  });
});

describe("the workshop", () => {
  it("has only themes that pass the check without dropping anything", async () => {
    const { readFileSync, readdirSync, statSync } = await import("node:fs");
    const dir = new URL("../../public/workshop/", import.meta.url);
    const index = JSON.parse(readFileSync(new URL("index.json", dir), "utf8"));
    const folders = readdirSync(new URL("themes/", dir)).filter((id) => statSync(new URL(`themes/${id}`, dir)).isDirectory());
    expect(index.themes.map((e) => e.theme.id).sort()).toEqual(folders.sort());
    for (const entry of index.themes) {
      const files = themeFiles(entry.theme).map((name) => ({ name, size: statSync(new URL(`${entry.path}${name}`, dir)).size }));
      const { theme, problems } = normalizeTheme(entry.theme, { assets: files });
      expect(problems).toEqual([]);
      expect(theme).toEqual(entry.theme);
      expect(readabilityIssues(theme)).toEqual([]);
    }
  });
});

describe("theme upgrades", () => {
  it("keeps known style, celebration, sounds and layout values and drops unknown ones", () => {
    const { theme, problems } = normalizeTheme(base({
      style: { corners: "sharp", cards: "wood", headings: "plain", effects: "neon", extra: "x" },
      celebration: { type: "fireworks" },
      sounds: { pack: "dubstep" },
      scenes: { leaderboard: { layout: "podium" }, question: { layout: "fancy" } },
    }));
    expect(theme.style).toEqual({ corners: "sharp", headings: "plain", effects: "neon" });
    expect(theme.celebration).toEqual({ type: "fireworks" });
    expect(theme.sounds).toEqual({ pack: null });
    expect(theme.scenes).toEqual({ leaderboard: { layout: "podium" } });
    const more = normalizeTheme(base({ scenes: { leaderboard: { layout: "medals" }, dashboard: { layout: "timeline" }, round: { layout: "curtain" }, welcome: { intro: "polaroid" } } })).theme;
    expect(more.scenes).toEqual({ leaderboard: { layout: "medals" }, dashboard: { layout: "timeline" }, round: { layout: "curtain" }, welcome: { intro: "polaroid" } });
    const bad = normalizeTheme(base({ scenes: { round: { layout: "explode" }, welcome: { intro: 3 } } }));
    expect(bad.theme.scenes).toEqual({});
    expect(bad.problems.map((p) => p.code)).toEqual(["layout", "layout"]);
    expect(problems).toEqual(expect.arrayContaining([{ code: "style", detail: "wood" }, { code: "sounds", detail: "dubstep" }]));
  });

  it("gives a theme from an older version the new fields with nothing chosen", () => {
    const { theme, problems } = normalizeTheme(base({}));
    expect(problems).toEqual([]);
    expect(theme).toMatchObject({ style: {}, celebration: { type: null }, sounds: { pack: null }, scenes: {} });
  });
});
