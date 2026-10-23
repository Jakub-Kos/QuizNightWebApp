import { describe, it, expect } from "vitest";
import { zipSync, strToU8 } from "fflate";
import { findPackUrl, readPack, report, md } from "../../scripts/workshop.mjs";
import { THEME_FORMAT } from "./themeFormat";

const pack = (theme, files = {}) => zipSync({ "theme.json": strToU8(JSON.stringify({ format: THEME_FORMAT, version: 1, ...theme })), ...files });

describe("the submission check", () => {
  it("finds the attached zip in an issue body, and nothing else", () => {
    const body = "### Theme file\n\n[theme-neon.zip](https://github.com/user-attachments/files/123456/theme-neon.zip)\n\nsee https://evil.example.com/x.zip";
    expect(findPackUrl(body)).toBe("https://github.com/user-attachments/files/123456/theme-neon.zip");
    expect(findPackUrl("https://evil.example.com/theme.zip")).toBeNull();
    expect(findPackUrl("")).toBeNull();
  });

  it("refuses what is not a theme pack, and packs that unpack to too much", () => {
    expect(() => readPack(new Uint8Array([1, 2, 3]))).toThrow("not a .zip");
    expect(() => readPack(zipSync({ "a.txt": strToU8("x") }))).toThrow("no theme.json");
    const huge = new Uint8Array(31 * 1024 * 1024);
    expect(() => readPack(zipSync({ "theme.json": strToU8("{}"), "big.jpg": huge }))).toThrow("too much");
  });

  it("reports a passing theme with the command to add it", () => {
    const text = report({ source: "https://github.com/user-attachments/files/1/t.zip", pack: readPack(pack({ id: "calm", name: "Calm", author: "Ann" })) });
    expect(text).toContain("<!-- workshop-check -->");
    expect(text).toContain("✅");
    expect(text).toContain("npm run workshop:add https://github.com/user-attachments/files/1/t.zip");
  });

  it("lists what would be dropped and gives no command", () => {
    const text = report({ source: "x.zip", pack: readPack(pack({ id: "x", name: "X", colors: { accent: "red" } })) });
    expect(text).toContain("❌");
    expect(text).toContain("`color`: accent");
    expect(text).not.toContain("workshop:add");
  });

  it("keeps text from the submission from formatting the comment or mentioning people", () => {
    expect(md("@owner <img src=x> | **bold** [link](u)")).toBe("@​owner \\<img src=x\\> \\| \\*\\*bold\\*\\* \\[link\\](u)");
    const text = report({ source: "x.zip", pack: readPack(pack({ id: "x", name: "Hi @everyone", author: "<b>me</b>" })) });
    expect(text).not.toContain("@everyone");
    expect(text).not.toContain("<b>");
  });
});
