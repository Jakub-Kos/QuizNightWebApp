import Papa from "papaparse";
import { HELP } from "../data/help";

// Sample files shipped with the demo in public/source/
export const EXAMPLE_MEDIA = { Image: "pad_BM2.webp", PImage: "1944paris.jpg", Audio: "audio.m4a", Video: "video.mp4", PVideo: "video.mp4", Handout: "handout.jpg" };

// Header of the questions sheet template (public/templates)
export const SHEET_HEAD = ["Kolo", "Q#", "Otázka", "Typ Odpovědi", "A", "B", "C", "D", "Správná odpověď", "Zdroj"];

// The sheet rows that make up a type's example, written the way the help table tells users to
export function exampleRows(type, example) {
  const options = ["A", "B", "C", "D"].map((o) => example.options?.[o] ?? "");
  const row = (number, text, typeCell, opts, answer, media) => ["", String(number), text, typeCell, ...opts, answer, media];
  const none = ["", "", "", ""];
  if (type === "Top5") {
    const [first, ...rest] = example.answer;
    return [row(1, example.text, type, none, first, ""), ...rest.map((answer, i) => row(i + 2, "", "", none, answer, ""))];
  }
  if (type === "Sort") {
    return example.items.map((item, i) => row(i + 1, i ? "" : example.text, i ? "" : type, [item, "", "", ""], example.letters?.[i] ?? "", ""));
  }
  return [row(1, example.text, type, options, example.answer ?? "", EXAMPLE_MEDIA[type] || "")];
}

// A new quiz's first round: the help page's example of every question type, in the quiz language,
// numbered straight down like the template. media: the files in public/source/ it uses (a handout's
// answer column holds its answer picture).
export function exampleQuestions(lang, title) {
  const section = (HELP[lang] || HELP.en).sections.find((s) => s.id === "types");
  const rows = section.table.rows
    .flatMap(([type]) => exampleRows(type.replace(/`/g, ""), section.examples[type.replace(/`/g, "")] || {}))
    .map((row, i) => [i ? "" : `${lang === "cs" ? "Kolo" : "Round"} 1 - ${title}`, String(i + 1), ...row.slice(2)]);
  const media = new Set(rows.flatMap((row) => [row[9], row[3] === "Handout" ? row[8] : ""]).filter(Boolean));
  return { csv: Papa.unparse([SHEET_HEAD, ...rows]), rows, media: [...media] };
}
