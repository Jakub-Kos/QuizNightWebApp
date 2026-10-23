import { describe, it, expect, vi } from "vitest";
import { writeTargets, createWriteGuard, WriteBlockedError, quizFileIds } from "./writeGuard";

const DRIVE = "https://www.googleapis.com/drive/v3/files";
const UPLOAD = "https://www.googleapis.com/upload/drive/v3/files";
const SHEETS = "https://sheets.googleapis.com/v4/spreadsheets";
const FOLDER = "application/vnd.google-apps.folder";
const json = JSON.stringify;

// The requests the app makes, as googleFetch sees them
describe("writeTargets", () => {
  it("lets reads through unchecked", () => {
    expect(writeTargets("GET", `${DRIVE}/abc?fields=id`)).toBeNull();
    expect(writeTargets(undefined, `${SHEETS}/s1/values/'Q'`)).toBeNull();
  });

  it("names the spreadsheet of Sheet edits", () => {
    expect(writeTargets("POST", `${SHEETS}/s1:batchUpdate`, "{}")).toEqual({ kind: "edit", ids: ["s1"] });
    expect(writeTargets("POST", `${SHEETS}/s1/values:batchUpdate`, "{}")).toEqual({ kind: "edit", ids: ["s1"] });
  });

  it("names the file of metadata and content updates and of sharing", () => {
    expect(writeTargets("PATCH", `${DRIVE}/s1?fields=id&supportsAllDrives=true`, json({ appProperties: {} }))).toEqual({ kind: "edit", ids: ["s1"] });
    expect(writeTargets("PATCH", `${UPLOAD}/c1?uploadType=media`, "{}")).toEqual({ kind: "edit", ids: ["c1"] });
    expect(writeTargets("POST", `${DRIVE}/f1/permissions?sendNotificationEmail=true`, "{}")).toEqual({ kind: "edit", ids: ["f1"] });
    expect(writeTargets("DELETE", `${DRIVE}/f1/permissions/p9?supportsAllDrives=true`)).toEqual({ kind: "edit", ids: ["f1"] });
  });

  it("names the parent folder of new files, also in a multipart upload", () => {
    expect(writeTargets("POST", `${UPLOAD}?uploadType=resumable`, json({ name: "a.jpg", parents: ["m1"] }))).toEqual({ kind: "create", ids: ["m1"] });
    const multipart = `--b\r\nContent-Type: application/json\r\n\r\n${json({ name: "quiznight.json", parents: ["f1"] })}\r\n--b\r\n\r\n{"x":1}\r\n--b--`;
    expect(writeTargets("POST", `${UPLOAD}?uploadType=multipart`, multipart)).toEqual({ kind: "create", ids: ["f1"] });
    expect(writeTargets("POST", `${DRIVE}/template/copy?fields=id`, json({ name: "Q", parents: ["new"] }))).toEqual({ kind: "create", ids: ["new"] });
  });

  it("allows an empty folder anywhere", () => {
    expect(writeTargets("POST", `${DRIVE}?fields=id,name`, json({ name: "Quiz", mimeType: FOLDER, parents: ["root"] }))).toEqual({ kind: "folder", ids: ["root"] });
  });

  it("refuses deleting files, files without a parent and unknown requests", () => {
    expect(writeTargets("DELETE", `${DRIVE}/s1`)).toEqual({ refuse: "delete" });
    expect(writeTargets("POST", `${UPLOAD}?uploadType=resumable`, json({ name: "a.jpg" }))).toEqual({ refuse: "no_parent" });
    expect(writeTargets("POST", "https://www.googleapis.com/drive/v3/changes/watch", "{}")).toEqual({ refuse: "unknown_url" });
    expect(writeTargets("DELETE", `${SHEETS}/s1`)).toEqual({ refuse: "sheet_method" });
  });

  it("tells trashing apart", () => {
    expect(writeTargets("PATCH", `${DRIVE}/f1`, json({ trashed: true }))).toEqual({ kind: "trash", ids: ["f1"] });
  });
});

// My Drive (root) > Other > Private.doc; My Drive > Quiz (folder q) > Zdroje (m) > kolo3 (k3) > cat.jpg (cat)
const PARENTS = { other: ["root"], private: ["other"], q: ["root"], sheet: ["q"], m: ["q"], k3: ["m"], cat: ["k3"], loose: ["root"] };
const QUIZ = { drive: { folderId: "q", spreadsheetId: "sheet", mediaFolderId: "m", configFileId: null } };

function guard(quizzes = [QUIZ]) {
  const parentsOf = vi.fn(async (id) => PARENTS[id] || []);
  const g = createWriteGuard({ roots: async () => quizzes.flatMap(quizFileIds), parentsOf, rootFolder: async () => "root" });
  vi.spyOn(console, "error").mockImplementation(() => {});
  return { g, parentsOf };
}
const edit = (id) => ["PATCH", `${DRIVE}/${id}`, "{}"];

describe("createWriteGuard", () => {
  it("allows the quiz's own files", async () => {
    const { g } = guard();
    await expect(g.check("POST", `${SHEETS}/sheet:batchUpdate`, "{}")).resolves.toMatchObject({ kind: "edit" });
    await expect(g.check(...edit("q"))).resolves.toBeTruthy();
  });

  it("allows files deeper inside the quiz folder, looking each up once", async () => {
    const { g, parentsOf } = guard();
    await expect(g.check(...edit("cat"))).resolves.toBeTruthy();
    const calls = parentsOf.mock.calls.length;
    await expect(g.check(...edit("cat"))).resolves.toBeTruthy();
    expect(parentsOf.mock.calls.length).toBe(calls);
    await expect(g.check("POST", `${UPLOAD}?uploadType=resumable`, json({ parents: ["k3"] }))).resolves.toBeTruthy();
  });

  it("refuses files outside every quiz", async () => {
    const { g } = guard();
    await expect(g.check(...edit("private"))).rejects.toBeInstanceOf(WriteBlockedError);
    await expect(g.check("POST", `${SHEETS}/elsewhere/values:batchUpdate`, "{}")).rejects.toBeInstanceOf(WriteBlockedError);
    await expect(g.check("POST", `${UPLOAD}?uploadType=multipart`, json({ parents: ["other"] }))).rejects.toBeInstanceOf(WriteBlockedError);
  });

  it("never treats My Drive as a quiz folder, even for a Sheet lying in it", async () => {
    const { g } = guard([{ drive: { folderId: "root", spreadsheetId: "loose" } }]);
    await expect(g.check(...edit("loose"))).resolves.toBeTruthy();
    await expect(g.check(...edit("private"))).rejects.toBeInstanceOf(WriteBlockedError);
    await expect(g.check("POST", `${UPLOAD}?uploadType=multipart`, json({ parents: ["root"] }))).rejects.toBeInstanceOf(WriteBlockedError);
  });

  it("refuses deleting even the quiz's own files", async () => {
    const { g } = guard();
    await expect(g.check("DELETE", `${DRIVE}/sheet`)).rejects.toBeInstanceOf(WriteBlockedError);
  });

  it("trashes and writes into only what it created itself", async () => {
    const { g } = guard([]);
    const trash = (id) => g.check("PATCH", `${DRIVE}/${id}`, json({ trashed: true }));
    await expect(trash("q")).rejects.toBeInstanceOf(WriteBlockedError);
    await expect(g.check("POST", `${DRIVE}?fields=id`, json({ name: "New quiz", mimeType: FOLDER, parents: ["root"] }))).resolves.toMatchObject({ kind: "folder" });
    g.created("newQuiz");
    await expect(g.check("POST", `${DRIVE}/template/copy`, json({ parents: ["newQuiz"] }))).resolves.toBeTruthy();
    await expect(trash("newQuiz")).resolves.toMatchObject({ kind: "trash" });
  });

  it("sees a quiz opened later", async () => {
    const quizzes = [];
    const { g } = guard(quizzes);
    await expect(g.check(...edit("sheet"))).rejects.toBeInstanceOf(WriteBlockedError);
    quizzes.push(QUIZ);
    await expect(g.check(...edit("sheet"))).resolves.toBeTruthy();
  });
});
