import { googleJson, GoogleApiError } from "./google";
import { sheetTabs, tabRange } from "./sheets";
import { DriveError, toDriveError } from "./drive";
import { readSheet, COLUMNS } from "./sheetModel";

// Writing the question editor's plans (sheetModel.js) into the questions tab, as the signed-in user

const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";
const DRIVE_API = "https://www.googleapis.com/drive/v3";

// The question being saved changed in the Sheet since it was opened; data is the newer { tab, sheet }
export class ConflictError extends Error {}

const column = (i) => String.fromCharCode(65 + i);

// The questions tab of a Drive quiz: by the tab id in questionsSheetUrl, so renaming the tab is fine
export async function loadQuestionSheet(quiz) {
  try {
    const spreadsheetId = quiz.drive.spreadsheetId;
    const gid = (String(quiz.questionsSheetUrl).match(/[#&?]gid=(\d+)/) || [])[1];
    const tabs = await sheetTabs(spreadsheetId);
    const tab = tabs.find((t) => String(t.sheetId) === gid) ?? tabs.find((t) => t.title === quiz.drive.questionsTab) ?? tabs[0];
    const data = await googleJson(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(`${tabRange(tab.title)}!A:${column(COLUMNS - 1)}`)}`);
    return { tab: { spreadsheetId, sheetId: tab.sheetId, title: tab.title }, sheet: readSheet(data.values || []) };
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

async function applyPlan(tab, { start, deleteCount, insert, fixups }) {
  const id = tab.spreadsheetId;
  const name = tabRange(tab.title);

  // Rows first, so the values below land in the right place
  const diff = insert.length - deleteCount;
  const requests = [];
  if (diff > 0) {
    const at = start + deleteCount;
    requests.push({ insertDimension: { range: { sheetId: tab.sheetId, dimension: "ROWS", startIndex: at, endIndex: at + diff }, inheritFromBefore: at > 0 } });
  }
  if (diff < 0) {
    requests.push({ deleteDimension: { range: { sheetId: tab.sheetId, dimension: "ROWS", startIndex: start + insert.length, endIndex: start + deleteCount } } });
  }
  if (requests.length) await post(`${SHEETS_API}/${id}:batchUpdate`, { requests });

  // Text exactly as typed (no formulas, dates or number formats), but Q# as a number like the rest of the column
  const raw = [];
  const entered = [];
  if (insert.length) raw.push({ range: `${name}!A${start + 1}:${column(COLUMNS - 1)}${start + insert.length}`, values: insert });
  insert.forEach((row, i) => entered.push({ range: `${name}!B${start + i + 1}`, values: [[row[1]]] }));
  fixups.forEach((f) => (f.col === 1 ? entered : raw).push({ range: `${name}!${column(f.col)}${f.row + 1}`, values: [[f.value]] }));
  if (raw.length) await post(`${SHEETS_API}/${id}/values:batchUpdate`, { valueInputOption: "RAW", data: raw });
  if (entered.length) await post(`${SHEETS_API}/${id}/values:batchUpdate`, { valueInputOption: "USER_ENTERED", data: entered });
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
