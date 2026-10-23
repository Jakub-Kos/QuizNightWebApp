import { describe, it, expect } from "vitest";
import { scoresTable, scoresWrites, reconcile } from "./driveConfig";

const NAME = "'Body'";

// A scores tab like the template: a title row, the header on row 2, teams from row 3, then rows the
// template prepared (formulas, no name)
const values = (teams, prepared = 2) => [
  ["Výsledky"],
  ["#", "Název týmu", "Kolo 1", "Kolo 2", "Celkem"],
  ...teams.map((t, i) => [String(i + 1), ...t]),
  ...Array.from({ length: prepared }, (_, i) => [String(teams.length + i + 1), "", "", "", "0"]),
];
const sheetFor = (v) => ({ tab: { sheetId: 3, title: "Body" }, ...scoresTable(v) });
const rowOf = (range) => Number(range.match(/(\d+)$/)[1]) - 1;
const NO_OPS = { ids: [], names: [], clears: [], appends: [] };

describe("scoresTable", () => {
  it("finds the header below a title row, and the teams and empty rows under it", () => {
    const table = scoresTable(values([["Sovy", "1", "2"], ["Lišky", "3", "1"]]));
    expect(table).toMatchObject({ header: 1, nameCol: 1, idCol: -1, end: 6 });
    expect(table.rows).toEqual([{ row: 2, name: "Sovy", id: null }, { row: 3, name: "Lišky", id: null }]);
    expect(table.empty).toEqual([4, 5]);
  });

  it("is null without a team name column", () => {
    expect(scoresTable([["Kolo", "Q#"]])).toBeNull();
  });
});

describe("scoresWrites", () => {
  it("adds the id column hidden, right of the table", () => {
    const sheet = sheetFor(values([["Sovy"]]));
    const { structure, cells } = scoresWrites(sheet, NAME, { ...NO_OPS, ids: [{ row: 2, id: "t1" }] });
    expect(cells).toContainEqual({ range: `${NAME}!F2`, values: [["quiznight-id"]] });
    expect(cells).toContainEqual({ range: `${NAME}!F3`, values: [["t1"]] });
    expect(structure[0].updateDimensionProperties).toMatchObject({ range: { startIndex: 5, endIndex: 6 }, properties: { hiddenByUser: true } });
  });

  it("puts a new team into the first prepared row, never above the header", () => {
    const sheet = sheetFor(values([["Sovy"]]));
    const { structure, cells } = scoresWrites(sheet, NAME, { ...NO_OPS, appends: [{ name: "Nový", id: "t9" }] });
    expect(structure.filter((r) => r.insertDimension)).toEqual([]);
    expect(cells).toContainEqual({ range: `${NAME}!B4`, values: [["Nový"]] });
    expect(cells.every((c) => rowOf(c.range) >= sheet.header)).toBe(true);
  });

  it("inserts rows right below the table when the prepared rows run out", () => {
    const sheet = sheetFor(values([["Sovy"]], 1));
    const { structure, cells } = scoresWrites(sheet, NAME, { ...NO_OPS, appends: [{ name: "A", id: "a" }, { name: "B", id: "b" }, { name: "C", id: "c" }] });
    const insert = structure.find((r) => r.insertDimension).insertDimension.range;
    expect(insert).toMatchObject({ startIndex: 4, endIndex: 6 });
    const names = cells.filter((c) => c.range.includes("!B")).map((c) => [rowOf(c.range), c.values[0][0]]);
    expect(names).toEqual([[3, "A"], [4, "B"], [5, "C"]]);
  });

  it("clears only the name and id of a removed team, keeping its row and points", () => {
    const v = values([["Sovy", "1"], ["Lišky", "2"]]);
    v[1].push("quiznight-id");
    v[2].push("", "", "", "t1");
    const sheet = sheetFor(v);
    const { structure, cells } = scoresWrites(sheet, NAME, { ...NO_OPS, clears: [2] });
    expect(structure).toEqual([]);
    expect(cells).toEqual([{ range: `${NAME}!B3`, values: [[""]] }, { range: `${NAME}!F3`, values: [[""]] }]);
  });
});

describe("reconcile", () => {
  const entry = (name, extra) => ({ name, motto: `${name}!`, ...extra });

  it("keeps a team's details when it is renamed in the Sheet", () => {
    const entries = { t1: entry("Sovy") };
    const result = reconcile({ rows: [{ row: 2, name: "Sovy noci", id: "t1" }], entries, base: entries, local: entries, canWrite: true });
    expect(result.entries.t1).toEqual({ name: "Sovy noci", motto: "Sovy!" });
    expect(result.ops).toEqual(NO_OPS);
  });

  it("archives a team whose row is gone, and reconnects it when its name is typed again", () => {
    const entries = { t1: entry("Sovy"), t2: entry("Lišky") };
    const gone = reconcile({ rows: [{ row: 2, name: "Lišky", id: "t2" }], entries, base: entries, local: entries, canWrite: true });
    expect(gone.entries.t1).toMatchObject({ name: "Sovy", removed: true });
    expect(gone.order).toEqual(["t2"]);

    const back = reconcile({ rows: [{ row: 2, name: "Lišky", id: "t2" }, { row: 3, name: "sovy", id: null }], entries: gone.entries, base: null, local: {}, canWrite: true });
    expect(back.order).toEqual(["t2", "t1"]);
    expect(back.entries.t1).toEqual({ name: "sovy", motto: "Sovy!" });
    expect(back.ops.ids).toEqual([{ row: 3, id: "t1" }]);
  });

  it("writes renames, removals and additions made in the app", () => {
    const base = { t1: entry("Sovy"), t2: entry("Lišky") };
    const local = { t1: entry("Noční sovy"), t3: entry("Nový") };
    const rows = [{ row: 2, name: "Sovy", id: "t1" }, { row: 3, name: "Lišky", id: "t2" }];
    const result = reconcile({ rows, entries: { ...base, ...local }, base, local, canWrite: true });
    expect(result.ops.names).toEqual([{ row: 2, name: "Noční sovy" }]);
    expect(result.ops.clears).toEqual([3]);
    expect(result.ops.appends).toEqual([{ name: "Nový", id: "t3" }]);
    expect(result.entries.t2.removed).toBe(true);
    expect(result.order).toEqual(["t1", "t3"]);
  });

  it("writes nothing for a viewer and never puts placeholder ids into the Sheet", () => {
    const local = { t1: entry("Sovy noci") };
    const result = reconcile({ rows: [{ row: 2, name: "Sovy", id: null }], entries: { "name:sovy": entry("Sovy") }, base: { t1: entry("Sovy") }, local, canWrite: false });
    expect(result.ops).toEqual(NO_OPS);
    expect(result.order).toEqual(["name:sovy"]);

    const writer = reconcile({ rows: [{ row: 2, name: "Sovy", id: null }], entries: { "name:sovy": entry("Sovy") }, base: null, local: {}, canWrite: true });
    expect(writer.ops.ids).toHaveLength(1);
    expect(writer.ops.ids[0].id.startsWith("name:")).toBe(false);
    expect(writer.entries[writer.ops.ids[0].id].motto).toBe("Sovy!");
  });
});
