import { describe, it, expect } from "vitest";
import Papa from "papaparse";
import { parseQuestionsCsv } from "./parse";
import {
  readSheet, emptyDraft, planAdd, planDelete, planSave, planMove, findBlock, stampColumns, toModelRows, sheetWrites,
} from "./sheetModel";

const META = { author: "Tester", changed: "2026-10-23 12:00", headers: { author: "Autor", changed: "Upraveno" } };
const TAB = { sheetId: 7, stamps: { author: 10, changed: 11 } };

// A questions tab like the template: a title row, the header on row 2, a round with three questions
// and prepared rows (nothing but a Q#) up to 10, then a full second round
function tab() {
  const rows = [
    ["Kvíz 2026"],
    ["Kolo", "Q#", "Otázka", "Typ Odpovědi", "A", "B", "C", "D", "Správná odpověď", "Zdroj"],
    ["Kolo 1 - Zeměpis", "1", "Hlavní město Francie?", "Written", "", "", "", "", "Paříž"],
    ["", "2", "Kolik nohou má pavouk?", "Numeric", "", "", "", "", "8"],
    ["", "3", "Největší planeta?", "ABCD", "Mars", "Jupiter", "Venuše", "Merkur", "B"],
  ];
  for (let n = 4; n <= 10; n++) rows.push(["", String(n)]);
  rows.push(["Kolo 2 - Hudba", "1", "Kdo zpívá?", "Audio", "", "", "", "", "Queen", "pisen.mp3"]);
  rows.push(["", "2", "Pět největších zemí", "Top5", "", "", "", "", "Rusko"]);
  ["Kanada", "Čína", "USA", "Brazílie"].forEach((a, i) => rows.push(["", String(3 + i), "", "", "", "", "", "", a]));
  return rows;
}
const sheetOf = (rows) => readSheet(toModelRows(rows, stampColumns(rows)));
const draft = (text, answer = "x") => ({ ...emptyDraft("Written"), text, answer });

describe("readSheet", () => {
  it("groups questions exactly like the show's parser", () => {
    const rows = tab();
    const sheet = sheetOf(rows);
    const parsed = parseQuestionsCsv(Papa.unparse(rows));
    expect(sheet.headerRow).toBe(1);
    expect(sheet.rounds.map((r) => r.questions.map((b) => b.question.text))).toEqual(parsed.map((r) => r.questions.map((q) => q.text)));
    expect(sheet.rounds[1].questions[1].rows).toHaveLength(5); // the Top5 takes its answer rows
  });
});

describe("planAdd", () => {
  it("fills the round's first prepared row instead of inserting one", () => {
    const sheet = sheetOf(tab());
    const plan = planAdd(sheet, 0, draft("Nová otázka"), META);
    expect(plan.start).toBe(5); // the row with Q# 4
    expect(plan.deleteCount).toBe(1);
    expect(plan.next).toHaveLength(sheet.rows.length);
    expect(sheetWrites(TAB, "'Q'", plan).structure).toEqual([]);
    expect(plan.next[5].slice(1, 3)).toEqual(["4", "Nová otázka"]);
    expect(plan.next[5][10]).toBe("Tester");
  });

  it("inserts rows only for what does not fit in the prepared rows", () => {
    const rows = tab().filter((r) => !(r.length === 2 && Number(r[1]) >= 4 && Number(r[1]) <= 9)); // one prepared row left
    const sheet = sheetOf(rows);
    const top = { ...emptyDraft("Top5"), text: "Top list", answers: ["a", "b", "c"] };
    const plan = planAdd(sheet, 0, top, META);
    expect(plan.deleteCount).toBe(1);
    expect(plan.insert).toHaveLength(3);
    const { structure } = sheetWrites(TAB, "'Q'", plan);
    expect(structure).toHaveLength(1);
    expect(structure[0].insertDimension.range).toMatchObject({ startIndex: plan.start + 1, endIndex: plan.start + 3 });
    // The second round is untouched, only pushed down
    const after = readSheet(plan.next);
    expect(after.rounds[1].questions.map((b) => b.question.text)).toEqual(sheet.rounds[1].questions.map((b) => b.question.text));
  });

  it("inserts a row after the last question of a round with no prepared rows", () => {
    const sheet = sheetOf(tab());
    const plan = planAdd(sheet, 1, draft("Další"), META);
    expect(plan.start).toBe(sheet.rounds[1].questions.at(-1).end + 1);
    expect(plan.deleteCount).toBe(0);
    expect(readSheet(plan.next).rounds[1].questions.at(-1).question.text).toBe("Další");
  });
});

describe("planDelete", () => {
  it("keeps every row of the round and leaves the freed row prepared", () => {
    const sheet = sheetOf(tab());
    const plan = planDelete(sheet, 0, 0, META);
    expect(plan.insert).toHaveLength(plan.deleteCount);
    expect(sheetWrites(TAB, "'Q'", plan).structure).toEqual([]);
    const after = readSheet(plan.next);
    expect(plan.next).toHaveLength(sheet.rows.length);
    expect(after.rounds[0].marker).toBe("Kolo 1 - Zeměpis");
    expect(after.rounds[0].questions.map((b) => b.question.text)).toEqual(["Kolik nohou má pavouk?", "Největší planeta?"]);
    // The round marker stays on its row, Q# is 1, 2 again, the old third row is now prepared
    expect(plan.next[2][0]).toBe("Kolo 1 - Zeměpis");
    expect(plan.next.slice(2, 5).map((r) => r[1])).toEqual(["1", "2", "3"]);
    expect(plan.next[4].slice(2).every((c) => !c)).toBe(true);
  });

  it("frees all rows of a Top5 without removing any", () => {
    const sheet = sheetOf(tab());
    const plan = planDelete(sheet, 1, 1, META);
    expect(plan.insert).toHaveLength(plan.deleteCount);
    expect(plan.next).toHaveLength(sheet.rows.length);
    expect(readSheet(plan.next).rounds[1].questions).toHaveLength(1);
  });
});

describe("planSave and planMove", () => {
  it("keeps the round marker on the round's first question", () => {
    const sheet = sheetOf(tab());
    const plan = planSave(sheet, 0, 0, draft("Změněná", "Lyon"), META);
    expect(plan.next[2].slice(0, 3)).toEqual(["Kolo 1 - Zeměpis", "1", "Změněná"]);
    expect(plan.next[2][8]).toBe("Lyon");
  });

  it("swaps neighbours and keeps the marker on top", () => {
    const sheet = sheetOf(tab());
    const plan = planMove(sheet, 0, 0, 1, META);
    const after = readSheet(plan.next);
    expect(after.rounds[0].questions.slice(0, 2).map((b) => b.question.text)).toEqual(["Kolik nohou má pavouk?", "Hlavní město Francie?"]);
    expect(plan.next[2][0]).toBe("Kolo 1 - Zeměpis");
    expect(plan.next[3][0]).toBe("");
  });
});

describe("findBlock", () => {
  it("finds a question again after renumbering or a moved marker", () => {
    const sheet = sheetOf(tab());
    const block = sheet.rounds[0].questions[1];
    const rows = tab();
    rows[3][1] = "7";
    rows[3][0] = "something";
    expect(findBlock(sheetOf(rows), block.rows, { ri: 0, qi: 1 })).toEqual({ ri: 0, qi: 1 });
  });

  it("reports nothing when the question itself changed", () => {
    const sheet = sheetOf(tab());
    const rows = tab();
    rows[3][2] = "Edited in the Sheet";
    expect(findBlock(sheetOf(rows), sheet.rounds[0].questions[1].rows)).toBeNull();
  });
});

describe("author and time columns", () => {
  it("are K and L in a Sheet that uses nothing past J", () => {
    expect(stampColumns(tab())).toEqual({ author: 10, changed: 11 });
  });

  it("are found by their headers wherever they are, in any language", () => {
    const rows = tab();
    rows[1][10] = "Poznámky";
    rows[1][12] = "Szerző";
    rows[1][13] = "Módosítva";
    expect(stampColumns(rows)).toEqual({ author: 12, changed: 13 });
  });

  it("skip columns already in use", () => {
    const rows = tab();
    rows[3][10] = "check this one";
    expect(stampColumns(rows)).toEqual({ author: 11, changed: 12 });
  });

  it("are the only columns past J that a save writes", () => {
    const rows = tab();
    rows[1][10] = "Notes";
    rows[2][10] = "moderator note";
    const stamps = stampColumns(rows);
    const sheet = readSheet(toModelRows(rows, stamps));
    expect(sheet.rounds[0].questions[0].author).toBe("");
    const plan = planSave(sheet, 0, 0, draft("Změněná"), META);
    const { raw, entered } = sheetWrites({ sheetId: 7, stamps }, "'Q'", plan);
    const columns = [...raw, ...entered].map((r) => r.range.match(/!([A-Z]+)\d/)[1]);
    expect(columns).not.toContain("K");
    expect(raw).toContainEqual({ range: "'Q'!L3:L3", values: [["Tester"]] });
    expect(raw).toContainEqual({ range: "'Q'!M3:M3", values: [["2026-10-23 12:00"]] });
    // The headers are written into the found columns too
    expect(raw).toContainEqual({ range: "'Q'!L2", values: [["Autor"]] });
    expect(raw).toContainEqual({ range: "'Q'!A3:J3", values: [plan.insert[0].slice(0, 10)] });
    expect(entered).toContainEqual({ range: "'Q'!B3", values: [["1"]] });
  });
});
