import { googleJson, googleFetch, currentToken, GoogleApiError } from "./google";
import { getQuiz, updateQuiz, onQuizSaved } from "./storage";
import { fetchSheetCsv, sheetTabs, tabRange } from "./sheets";
import { parseScoresCsv, TEAM_COLUMN } from "./parse";

// The setup of a Drive quiz lives in its folder, so everyone who opens it sees the same quiz:
//   quiznight.json   title, show settings, rules, prizes, and per-team details keyed by team name
//   the scores tab   the team names themselves (the leaderboard of the night), never copied elsewhere
// A team added in the app is a draft (team.draft) until its name is confirmed; it stays local until then.
// The browser's quiz record stays the working copy (the show runs offline). Syncing merges three ways:
// what changed here since the last sync (drive.configBase) wins over what is in the file, everything
// else comes from the file, so two people changing different things never undo each other.

const DRIVE_API = "https://www.googleapis.com/drive/v3";
const UPLOAD_API = "https://www.googleapis.com/upload/drive/v3/files";
const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";
export const CONFIG_NAME = "quiznight.json";

const isObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);
const stable = (v) => JSON.stringify(v, (_, x) => (isObject(x) ? Object.fromEntries(Object.keys(x).sort().map((k) => [k, x[k]])) : x));
const same = (a, b) => stable(a) === stable(b);
const lower = (name) => String(name).trim().toLowerCase();

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

export function configFromQuiz(quiz) {
  const teams = {};
  for (const { id, name, ...details } of quiz.teams) { // eslint-disable-line no-unused-vars
    if (String(name || "").trim() && !details.draft) teams[String(name).trim()] = details;
  }
  return { version: 1, title: quiz.title, settings: quiz.settings, rules: quiz.rules, prizes: quiz.prizes, teams };
}

// names: the scores tab's teams in order, or null when there is no scores tab (the file's teams then).
// unsaved: names that could not be written to the tab; those teams stay here as drafts.
export function applyConfig(quiz, config, names, unsaved = []) {
  const keep = new Set(unsaved.map(lower));
  const details = new Map(Object.entries(config.teams || {}).map(([name, d]) => [lower(name), d]));
  const known = new Map(quiz.teams.map((team) => [lower(team.name), team]));
  let nextId = Math.max(0, ...quiz.teams.map((team) => Number(team.id) || 0)) + 1;
  const list = names ?? Object.keys(config.teams || {});
  return {
    ...quiz,
    title: config.title ?? quiz.title,
    settings: { ...quiz.settings, ...config.settings },
    rules: config.rules ?? quiz.rules,
    prizes: config.prizes ?? quiz.prizes,
    teams: [
      ...list.map((name) => ({ ...(details.get(lower(name)) || {}), id: known.get(lower(name))?.id ?? nextId++, name })),
      ...quiz.teams.filter((team) => team.draft || keep.has(lower(team.name))).map((team) => ({ ...team, draft: true })),
    ],
  };
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

// Teams added in the app go into the scores tab: into the first empty name cells below the header
// (the template prepares rows with formulas), then as new rows
async function addTeamsToSheet(quiz, names) {
  const spreadsheetId = quiz.drive.spreadsheetId;
  const gid = (String(quiz.scoresSheetUrl).match(/[#&?]gid=(\d+)/) || [])[1];
  const tabs = await sheetTabs(spreadsheetId);
  const tab = tabs.find((t) => String(t.sheetId) === gid) ?? tabs[0];
  const name = tabRange(tab.title);
  const rows = (await googleJson(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(name)}`)).values || [];
  const header = rows.findIndex((row) => row.some((c) => TEAM_COLUMN.test(String(c))));
  if (header < 0) return;
  const col = rows[header].findIndex((c) => TEAM_COLUMN.test(String(c)));

  const cells = [];
  const extra = [];
  let r = header + 1;
  for (const team of names) {
    while (r < rows.length && String(rows[r]?.[col] ?? "").trim()) r++;
    if (r < rows.length) cells.push({ range: `${name}!${column(col)}${r + 1}`, values: [[team]] });
    else extra.push(Object.assign(Array(col + 1).fill(""), { [col]: team }));
    r++;
  }
  const post = (url, body) => googleJson(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
  if (cells.length) await post(`${SHEETS_API}/${spreadsheetId}/values:batchUpdate`, { valueInputOption: "RAW", data: cells });
  if (extra.length) await post(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(`${name}!A${header + 1}`)}:append?valueInputOption=RAW&insertDataOption=INSERT_ROWS`, { values: extra });
}

async function runSync(quizId) {
  const quiz = await getQuiz(quizId);
  const drive = quiz?.drive;
  if (!drive?.folderId || !currentToken()) return null;

  const fileId = await findConfigFile(drive);
  const remote = fileId ? await readConfig(fileId) : null;
  const local = configFromQuiz(quiz);
  const merged = !remote ? local : drive.configBase ? merge3(drive.configBase, local, remote) : overlay(local, remote);

  // Team names from the scores tab. Names added here since the last sync (not there, not seen there
  // before) are written into it; a first sync only adopts the tab, so old local teams are not pushed.
  let names = null;
  let unsaved = [];
  if (quiz.scoresSheetUrl) {
    const scores = parseScoresCsv(await fetchSheetCsv(quiz.scoresSheetUrl));
    if (scores) {
      names = scores.teams.map((team) => team.name);
      const seen = new Set([...(drive.knownTeams || []), ...names].map(lower));
      const added = drive.knownTeams
        ? quiz.teams.filter((team) => !team.draft).map((team) => String(team.name || "").trim()).filter((n) => n && !seen.has(lower(n)))
        : [];
      if (added.length) {
        try {
          await addTeamsToSheet(quiz, added);
          names = [...names, ...added];
        } catch (err) {
          // A viewer, or offline: the teams stay here as drafts rather than disappearing
          console.warn("Could not add teams to the scores tab", err);
          unsaved = added;
        }
      }
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
    const next = applyConfig(q, merged, names, unsaved);
    return { ...next, drive: { ...q.drive, configFileId: savedId, configBase: configFromQuiz(next), knownTeams: names ?? q.drive.knownTeams ?? null } };
  }, "drive");
}

// One sync at a time per quiz; resolves with the updated quiz record (null when not signed in)
const running = new Map();
export function syncDriveConfig(quizId) {
  const next = (running.get(quizId) || Promise.resolve()).catch(() => {}).then(() => runSync(quizId));
  running.set(quizId, next);
  return next;
}

// Changes made in this window are written to Drive shortly after they are saved
const timers = new Map();
onQuizSaved((quiz, source) => {
  if (source === "drive" || !quiz?.drive?.folderId || !currentToken()) return;
  if (quiz.drive.configBase && same(configFromQuiz(quiz), quiz.drive.configBase)) return;
  clearTimeout(timers.get(quiz.id));
  timers.set(quiz.id, setTimeout(() => syncDriveConfig(quiz.id).catch((err) => console.error("Drive sync failed", err)), 1500));
});
