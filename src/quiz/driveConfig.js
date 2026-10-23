import { googleJson, googleFetch, currentToken, GoogleApiError } from "./google";
import { getQuiz, updateQuiz, onQuizSaved } from "./storage";
import { sheetTabs, tabRange } from "./sheets";
import { TEAM_COLUMN, TEAM_ID_HEADER } from "./parse";

// The setup of a Drive quiz lives in its folder, so everyone who opens it sees the same quiz:
//   quiznight.json   title, show settings, rules, prizes, and the teams' details keyed by team id
//   the scores tab   the teams themselves: one row each, with the team id in a hidden column
// Ids never come from names, so a rename (in the Sheet or here) keeps a team's details, and no team
// name can collide with anything. A team whose row is deleted (or whose name is cleared) is archived
// ({ removed: true }) with its details; typing its name into a row again reconnects it. Removing a team
// here only clears its name and id cells, so its row and points stay in the Sheet. A team added in the app is a draft
// (team.draft) until its name is confirmed.
// The browser's quiz record stays the working copy (the show runs offline). Syncing merges three ways:
// what changed here since the last sync (drive.configBase) wins over the file, the rest comes from it.

const DRIVE_API = "https://www.googleapis.com/drive/v3";
const UPLOAD_API = "https://www.googleapis.com/upload/drive/v3/files";
const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";
export const CONFIG_NAME = "quiznight.json";
const VERSION = 2;

const isObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
const stable = (v) => JSON.stringify(v, (_, x) => (isObject(x) ? Object.fromEntries(Object.keys(x).sort().map((k) => [k, x[k]])) : x));
const same = (a, b) => stable(a) === stable(b);
const lower = (name) => String(name ?? "").trim().toLowerCase();
export const newTeamId = () => `t${Math.random().toString(36).slice(2, 10)}`;

// Objects merge key by key; anything else is taken whole from the side that changed (here wins a tie)
export function merge3(base, local, remote) {
  if (isObject(local) && isObject(remote)) {
    const out = {};
    for (const key of new Set([...Object.keys(local), ...Object.keys(remote)])) {
      const value = merge3(isObject(base) ? base[key] : undefined, local[key], remote[key]);
      if (value !== undefined) out[key] = value;
    }
    return out;
  }
  if (same(local, base)) return remote;
  if (same(remote, base)) return local;
  return local;
}

// The file wins, but keeps what it does not have (a first sync on a new device)
function overlay(local, remote) {
  if (!isObject(local) || !isObject(remote)) return remote === undefined ? local : remote;
  const out = { ...local };
  for (const [key, value] of Object.entries(remote)) out[key] = overlay(local[key], value);
  return out;
}

// The first version kept details by team name; give them placeholder ids, which the scores tab's
// rows then reconnect to by name
function upgrade(config) {
  if (!config || config.version === VERSION) return config;
  const teams = {};
  Object.entries(config.teams || {}).forEach(([name, details]) => { teams[`name:${lower(name)}`] = { ...details, name }; });
  return { ...config, version: VERSION, teams };
}

export function configFromQuiz(quiz) {
  const teams = {};
  Object.entries(quiz.teamArchive || {}).forEach(([id, entry]) => { teams[id] = { ...entry, removed: true }; });
  quiz.teams.forEach(({ id, ...team }) => {
    if (!team.draft && String(team.name || "").trim()) teams[id] = { ...team, name: String(team.name).trim() };
  });
  return { version: VERSION, title: quiz.title, settings: quiz.settings, rules: quiz.rules, prizes: quiz.prizes, teams };
}

// The quiz record for a config; order: the ids shown, in order (the rest of the teams is archived)
export function applyConfig(quiz, config, order) {
  const teams = config.teams || {};
  const shown = (order ?? Object.keys(teams).filter((id) => !teams[id].removed)).filter((id) => teams[id]);
  const visible = new Set(shown);
  const strip = ({ removed, ...entry }) => entry; // eslint-disable-line no-unused-vars
  return {
    ...quiz,
    title: config.title ?? quiz.title,
    settings: { ...quiz.settings, ...config.settings },
    rules: config.rules ?? quiz.rules,
    prizes: config.prizes ?? quiz.prizes,
    teams: [...shown.map((id) => ({ ...strip(teams[id]), id })), ...quiz.teams.filter((team) => team.draft)],
    teamArchive: Object.fromEntries(Object.entries(teams).filter(([id]) => !visible.has(id)).map(([id, entry]) => [id, strip(entry)])),
  };
}

// What to do with the scores tab and the team entries. rows: [{ row, name, id }] with a name, from
// the tab; entries: the merged config's teams; base and local: the teams at the last sync and here.
// Here wins over the Sheet: a team renamed here since the last sync is renamed in the Sheet, one removed
// here has its name and id cleared, one added here is appended. Everything else follows the Sheet. With
// canWrite false (a viewer) nothing is written and the Sheet is followed as it is.
// Returns { entries, order, ops: { ids: [{ row, id }], names: [{ row, name }], clears: [row], appends: [{ name, id }] } }.
export function reconcile({ rows, entries, base, local, canWrite }) {
  entries = Object.fromEntries(Object.entries(entries).map(([id, e]) => [id, { ...e }]));
  const ops = { ids: [], names: [], clears: [], appends: [] };
  const active = (set, id) => Boolean(set?.[id]) && !set[id].removed;
  const deletedHere = new Set(base ? Object.keys(base).filter((id) => active(base, id) && !local[id]) : []);
  const renamedHere = new Map(base ? Object.keys(local).filter((id) => active(local, id) && active(base, id) && local[id].name !== base[id].name).map((id) => [id, local[id].name]) : []);
  const addedHere = base ? Object.keys(local).filter((id) => active(local, id) && !base[id]) : [];

  // Every row gets an id: its own, or (a new row, or a copy of another row) the entry of the same
  // name that no row has, so a team typed back in gets its details again, or a new one
  const rowIds = new Set(rows.map((r) => r.id).filter(Boolean));
  const used = new Set();
  const free = (name) => {
    const matches = Object.keys(entries).filter((id) => !used.has(id) && !rowIds.has(id) && !addedHere.includes(id) && lower(entries[id].name) === lower(name));
    return matches.find((id) => entries[id].removed) ?? matches[0];
  };
  const order = [];
  for (const r of rows) {
    let id = r.id && !used.has(r.id) ? r.id : null;
    if (!id) {
      id = free(r.name) ?? (canWrite ? newTeamId() : `name:${lower(r.name)}`);
      // Placeholder ids (old files, viewers) are made from names: never write one into the Sheet
      if (canWrite && id.startsWith("name:")) {
        const real = newTeamId();
        if (entries[id]) entries[real] = entries[id];
        delete entries[id];
        id = real;
      }
      if (canWrite) ops.ids.push({ row: r.row, id });
    }
    used.add(id);
    if (canWrite && deletedHere.has(id)) {
      ops.clears.push(r.row);
      entries[id] = { ...(entries[id] || base[id]), removed: true };
      continue;
    }
    const entry = entries[id] || {};
    const name = canWrite && renamedHere.has(id) ? renamedHere.get(id) : r.name;
    if (name !== r.name) ops.names.push({ row: r.row, name });
    entries[id] = { ...entry, name };
    delete entries[id].removed;
    order.push(id);
  }

  // Teams added here: appended, reconnecting to an archived team of the same name
  if (canWrite) {
    for (const id of addedHere) {
      if (used.has(id)) continue; // already in the Sheet (written by an earlier sync)
      const name = local[id].name;
      const archived = Object.keys(entries).find((other) => other !== id && !used.has(other) && entries[other].removed && lower(entries[other].name) === lower(name));
      const target = archived ?? id;
      if (archived) {
        const { removed, ...details } = entries[archived]; // eslint-disable-line no-unused-vars
        entries[archived] = { ...local[id], ...details, name };
        delete entries[id];
      }
      ops.appends.push({ name, id: target });
      used.add(target);
      order.push(target);
    }
  }

  // Not in the Sheet any more: archived with their details
  for (const id of Object.keys(entries)) if (!used.has(id)) entries[id] = { ...entries[id], removed: true };
  return { entries, order, ops };
}

async function findConfigFile(drive) {
  if (drive.configFileId) {
    try {
      const file = await googleJson(`${DRIVE_API}/files/${drive.configFileId}?fields=id,trashed&supportsAllDrives=true`);
      if (!file.trashed) return file.id;
    } catch { /* deleted: look it up by name */ }
  }
  const params = new URLSearchParams({
    q: `'${drive.folderId}' in parents and name = '${CONFIG_NAME}' and trashed = false`,
    fields: "files(id)",
    orderBy: "modifiedTime desc",
    supportsAllDrives: "true",
    includeItemsFromAllDrives: "true",
  });
  return (await googleJson(`${DRIVE_API}/files?${params}`)).files[0]?.id ?? null;
}

async function readConfig(fileId) {
  const text = await (await googleFetch(`${DRIVE_API}/files/${fileId}?alt=media&supportsAllDrives=true`)).text();
  try { return JSON.parse(text); } catch { return null; } // edited by hand into something else
}

async function writeConfig(folderId, fileId, config) {
  const body = JSON.stringify(config, null, 2);
  if (fileId) {
    await googleFetch(`${UPLOAD_API}/${fileId}?uploadType=media&supportsAllDrives=true`, { method: "PATCH", headers: { "Content-Type": "application/json" }, body });
    return fileId;
  }
  const boundary = `quiznight${Math.random().toString(36).slice(2)}`;
  const meta = JSON.stringify({ name: CONFIG_NAME, parents: [folderId], mimeType: "application/json" });
  const multipart = `--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n${meta}\r\n--${boundary}\r\nContent-Type: application/json\r\n\r\n${body}\r\n--${boundary}--`;
  const created = await googleJson(`${UPLOAD_API}?uploadType=multipart&supportsAllDrives=true&fields=id`, {
    method: "POST",
    headers: { "Content-Type": `multipart/related; boundary=${boundary}` },
    body: multipart,
  });
  return created.id;
}

const column = (i) => (i < 26 ? String.fromCharCode(65 + i) : String.fromCharCode(64 + Math.floor(i / 26)) + String.fromCharCode(65 + (i % 26)));
const post = (url, body) => googleJson(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });

// The scores tab: its team rows [{ row, name, id }] and empty name rows below the header, or null
// when it has no team name column
async function readScoresTab(quiz) {
  const spreadsheetId = quiz.drive.spreadsheetId;
  const gid = (String(quiz.scoresSheetUrl).match(/[#&?]gid=(\d+)/) || [])[1];
  const tabs = await sheetTabs(spreadsheetId);
  const tab = tabs.find((t) => String(t.sheetId) === gid) ?? tabs[0];
  const values = (await googleJson(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(tabRange(tab.title))}`)).values || [];
  const table = scoresTable(values);
  return table && { spreadsheetId, tab, ...table };
}

// The team rows of the scores tab's values (see readScoresTab), or null
export function scoresTable(values) {
  const header = values.findIndex((row) => row.some((c) => TEAM_COLUMN.test(String(c))));
  if (header < 0) return null;
  const nameCol = values[header].findIndex((c) => TEAM_COLUMN.test(String(c)));
  const idCol = values[header].indexOf(TEAM_ID_HEADER);
  const cell = (r, c) => String(values[r]?.[c] ?? "").trim();
  const rows = [];
  const empty = [];
  for (let r = header + 1; r < values.length; r++) {
    if (cell(r, nameCol)) rows.push({ row: r, name: cell(r, nameCol), id: idCol >= 0 ? cell(r, idCol) || null : null });
    else empty.push(r);
  }
  return { header, nameCol, idCol, width: Math.max(0, ...values.map((v) => v.length)), end: values.length, rows, empty };
}

// Ids, renames, cleared names and new teams as cell values. New teams go into the first empty name
// cells below the header (the template prepares rows with formulas), then into rows inserted right
// below the table. The id column is added, hidden, right of the table's last column the first time.
export async function writeScoresTab(sheet, ops) {
  const { structure, cells } = scoresWrites(sheet, tabRange(sheet.tab.title), ops);
  // Columns and rows first, so the values land in them
  if (structure.length) await post(`${SHEETS_API}/${sheet.spreadsheetId}:batchUpdate`, { requests: structure });
  if (cells.length) await post(`${SHEETS_API}/${sheet.spreadsheetId}/values:batchUpdate`, { valueInputOption: "RAW", data: cells });
}

// The requests writeScoresTab sends: { structure } for :batchUpdate, { cells } as RAW values.
// name: the tab's name quoted for a range.
export function scoresWrites(sheet, name, ops) {
  const { tab, header, nameCol } = sheet;
  let idCol = sheet.idCol;
  const cells = [];
  const structure = [];
  if (idCol < 0) {
    idCol = sheet.width;
    cells.push({ range: `${name}!${column(idCol)}${header + 1}`, values: [[TEAM_ID_HEADER]] });
    structure.push({ updateDimensionProperties: { range: { sheetId: tab.sheetId, dimension: "COLUMNS", startIndex: idCol, endIndex: idCol + 1 }, properties: { hiddenByUser: true }, fields: "hiddenByUser" } });
  }
  const set = (row, col, value) => cells.push({ range: `${name}!${column(col)}${row + 1}`, values: [[value]] });
  ops.ids.forEach(({ row, id }) => set(row, idCol, id));
  ops.names.forEach(({ row, name: value }) => set(row, nameCol, value));
  ops.clears.forEach((row) => { set(row, nameCol, ""); set(row, idCol, ""); });
  const empty = [...sheet.empty];
  let next = sheet.end; // first row below the table
  const inserted = ops.appends.length - empty.length;
  if (inserted > 0) {
    structure.push({ insertDimension: { range: { sheetId: tab.sheetId, dimension: "ROWS", startIndex: next, endIndex: next + inserted }, inheritFromBefore: next > header + 1 } });
  }
  ops.appends.forEach(({ name: value, id }) => {
    const row = empty.length ? empty.shift() : next++;
    set(row, nameCol, value);
    set(row, idCol, id);
  });
  return { structure, cells };
}

// Edit access rarely changes during a session; asked again after a few minutes
const editAccess = new Map(); // fileId -> { at, value }
const canEdit = async (fileId) => {
  const cached = editAccess.get(fileId);
  if (cached && Date.now() - cached.at < 5 * 60_000) return cached.value;
  let value = false;
  try {
    value = Boolean((await googleJson(`${DRIVE_API}/files/${fileId}?fields=capabilities(canEdit)&supportsAllDrives=true`)).capabilities?.canEdit);
  } catch { /* treated as a viewer */ }
  editAccess.set(fileId, { at: Date.now(), value });
  return value;
};

async function runSync(quizId) {
  const quiz = await getQuiz(quizId);
  const drive = quiz?.drive;
  if (!drive?.folderId || !currentToken()) return null;

  // The file, the scores tab and edit access do not depend on each other: fetched at the same time
  const [{ fileId, stored }, sheet, writable] = await Promise.all([
    findConfigFile(drive).then(async (id) => ({ fileId: id, stored: id ? await readConfig(id) : null })),
    quiz.scoresSheetUrl ? readScoresTab(quiz) : null,
    quiz.scoresSheetUrl ? canEdit(drive.spreadsheetId) : false,
  ]);
  const remote = upgrade(stored);
  const local = configFromQuiz(quiz);
  const base = drive.configBase?.version === VERSION ? drive.configBase : null;
  // A first sync here takes the folder's teams as they are (old local teams are not pushed)
  let merged = !remote ? local : base ? merge3(base, local, remote) : { ...overlay(local, remote), teams: remote.teams || {} };

  let order = null;
  if (sheet) {
    const result = reconcile({ rows: sheet.rows, entries: merged.teams || {}, base: base?.teams ?? null, local: local.teams, canWrite: writable });
    const { ids, names, clears, appends } = result.ops;
    let written = true;
    if (ids.length || names.length || clears.length || appends.length) {
      try {
        await writeScoresTab(sheet, result.ops);
      } catch (err) {
        console.warn("Could not update the scores tab", err);
        written = false;
      }
    }
    if (written) {
      merged = { ...merged, teams: result.entries };
      order = result.order;
    } else {
      // The Sheet did not take the changes (no edit access, offline): follow it as it is, so teams
      // renamed or deleted here come back rather than drifting apart from the Sheet
      const asIs = reconcile({ rows: sheet.rows, entries: merged.teams || {}, base: null, local: {}, canWrite: false });
      merged = { ...merged, teams: asIs.entries };
      order = asIs.order;
    }
  }

  let savedId = fileId;
  if (!same(merged, remote)) {
    try {
      savedId = await writeConfig(drive.folderId, fileId, merged);
    } catch (err) {
      if (!(err instanceof GoogleApiError && err.status === 403)) throw err; // viewers keep the shared setup
    }
  }

  return updateQuiz(quizId, (q) => {
    const next = applyConfig(q, merged, order);
    return { ...next, drive: { ...q.drive, configFileId: savedId, configBase: configFromQuiz(next) } };
  }, "drive");
}

// Where each quiz's sync is, for the setup page: { phase: "idle" | "pending" (a change waits to be
// written) | "syncing" | "done" | "error", at (last success, ms), error }
const syncStates = new Map();
const stateListeners = new Set();
function setSyncState(quizId, patch) {
  const next = { ...(syncStates.get(quizId) || { phase: "idle" }), ...patch };
  syncStates.set(quizId, next);
  stateListeners.forEach((fn) => fn(quizId, next));
}
export const getSyncState = (quizId) => syncStates.get(quizId) || { phase: "idle" };
export function onSyncState(fn) {
  stateListeners.add(fn);
  return () => stateListeners.delete(fn);
}

// One sync at a time per quiz; resolves with the updated quiz record (null when not signed in)
const running = new Map();
export function syncDriveConfig(quizId) {
  const next = (running.get(quizId) || Promise.resolve()).catch(() => {}).then(async () => {
    setSyncState(quizId, { phase: "syncing", error: null });
    try {
      const result = await runSync(quizId);
      setSyncState(quizId, result ? { phase: "done", at: Date.now() } : { phase: "idle" });
      return result;
    } catch (err) {
      setSyncState(quizId, { phase: "error", error: err });
      throw err;
    }
  });
  running.set(quizId, next);
  return next;
}

// Changes made in this window are written to Drive shortly after they are saved
const timers = new Map();
onQuizSaved((quiz, source) => {
  if (source === "drive" || !quiz?.drive?.folderId || !currentToken()) return;
  if (quiz.drive.configBase && same(configFromQuiz(quiz), quiz.drive.configBase)) return;
  clearTimeout(timers.get(quiz.id));
  setSyncState(quiz.id, { phase: "pending" });
  timers.set(quiz.id, setTimeout(() => syncDriveConfig(quiz.id).catch((err) => console.error("Drive sync failed", err)), 1500));
});
