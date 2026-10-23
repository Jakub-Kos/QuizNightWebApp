import { googleJson, GoogleApiError } from "./google";
import { sheetTabs, tabRange } from "./sheets";
import { DriveError, toDriveError } from "./drive";
import { readSheet, stampColumns, toModelRows, sheetWrites } from "./sheetModel";

// Writing the question editor's plans (sheetModel.js) into the questions tab, as the signed-in user

const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";
const DRIVE_API = "https://www.googleapis.com/drive/v3";

// The question being saved changed in the Sheet since it was opened; data is the newer { tab, sheet }
export class ConflictError extends Error {}

// The questions tab of a Drive quiz: by the tab id in questionsSheetUrl, so renaming the tab is fine.
// tab.stamps says where the author and time columns are in the Sheet (see stampColumns).
export async function loadQuestionSheet(quiz) {
  try {
    const spreadsheetId = quiz.drive.spreadsheetId;
    const gid = (String(quiz.questionsSheetUrl).match(/[#&?]gid=(\d+)/) || [])[1];
    const tabs = await sheetTabs(spreadsheetId);
    const tab = tabs.find((t) => String(t.sheetId) === gid) ?? tabs.find((t) => t.title === quiz.drive.questionsTab) ?? tabs[0];
    const data = await googleJson(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(tabRange(tab.title))}`);
    const values = data.values || [];
    const stamps = stampColumns(values);
    return { tab: { spreadsheetId, sheetId: tab.sheetId, title: tab.title, stamps }, sheet: readSheet(toModelRows(values, stamps)) };
  } catch (err) {
    throw toDriveError(err);
  }
}

// Whether the signed-in user may edit the Sheet (viewers and commenters may not)
export async function canEditSheet(spreadsheetId) {
  try {
    const file = await googleJson(`${DRIVE_API}/files/${spreadsheetId}?fields=capabilities(canEdit)&supportsAllDrives=true`);
    return Boolean(file.capabilities?.canEdit);
  } catch (err) {
    throw toDriveError(err);
  }
}

const post = (url, body) => googleJson(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });

async function applyPlan(tab, plan) {
  const { structure, raw, entered } = sheetWrites(tab, tabRange(tab.title), plan);
  // Rows first, so the values land in the right place
  if (structure.length) await post(`${SHEETS_API}/${tab.spreadsheetId}:batchUpdate`, { requests: structure });
  if (raw.length) await post(`${SHEETS_API}/${tab.spreadsheetId}/values:batchUpdate`, { valueInputOption: "RAW", data: raw });
  if (entered.length) await post(`${SHEETS_API}/${tab.spreadsheetId}/values:batchUpdate`, { valueInputOption: "USER_ENTERED", data: entered });
}

// Read the tab, plan the change on that fresh copy (makePlan(sheet) may throw ConflictError), write it,
// and return the tab as it is afterwards. Planning on a fresh copy keeps other people's edits elsewhere
// in the Sheet intact; only a change to the same question is a conflict.
export async function commitEdit(quiz, makePlan) {
  const data = await loadQuestionSheet(quiz);
  let plan;
  try {
    plan = makePlan(data.sheet);
  } catch (err) {
    if (err instanceof ConflictError) err.data = data;
    throw err;
  }
  try {
    await applyPlan(data.tab, plan);
  } catch (err) {
    if (err instanceof GoogleApiError && err.status === 403) throw new DriveError("readonly");
    throw toDriveError(err);
  }
  return loadQuestionSheet(quiz);
}

// Who else has the editor open, kept in the Sheet file's appProperties (not in the Sheet, so no cells or
// version history): one entry per person, "seconds|question|1 when typing|name", refreshed while open.
// Entries older than PRESENCE_TTL count as gone; ones much older are cleared by the next writer.
const PRESENCE_TTL = 60;
const PRESENCE_PREFIX = "qn_editing_";

export function presenceKey(email) {
  let h = 2166136261;
  for (const ch of String(email)) h = Math.imul(h ^ ch.codePointAt(0), 16777619);
  return `${PRESENCE_PREFIX}${(h >>> 0).toString(36)}`;
}

// [{ key, question, typing, name, at }] of entries still fresh
export async function readPresence(spreadsheetId) {
  const file = await googleJson(`${DRIVE_API}/files/${spreadsheetId}?fields=appProperties&supportsAllDrives=true`);
  const now = Date.now() / 1000;
  return Object.entries(file.appProperties || {})
    .filter(([key]) => key.startsWith(PRESENCE_PREFIX))
    .map(([key, value]) => {
      const [at, question, typing, ...name] = String(value).split("|");
      return { key, at: Number(at), question, typing: typing === "1", name: name.join("|") };
    })
    .filter((p) => now - p.at < PRESENCE_TTL);
}

// entry: { question, typing, name }, or null when leaving
export async function writePresence(spreadsheetId, key, entry, { keepalive = false } = {}) {
  const props = {};
  if (entry) {
    const value = `${Math.round(Date.now() / 1000)}|${entry.question}|${entry.typing ? 1 : 0}|${entry.name}`;
    props[key] = value.slice(0, 124 - key.length - 1); // Drive allows 124 bytes for key and value together
  } else {
    props[key] = null;
  }
  // Clear entries nobody refreshed for ten minutes
  if (entry) {
    try {
      const file = await googleJson(`${DRIVE_API}/files/${spreadsheetId}?fields=appProperties&supportsAllDrives=true`);
      const now = Date.now() / 1000;
      Object.entries(file.appProperties || {}).forEach(([k, v]) => {
        if (k.startsWith(PRESENCE_PREFIX) && k !== key && now - Number(String(v).split("|")[0]) > 600) props[k] = null;
      });
    } catch { /* clearing is optional */ }
  }
  await googleJson(`${DRIVE_API}/files/${spreadsheetId}?fields=id&supportsAllDrives=true`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ appProperties: props }),
    keepalive,
  });
}
