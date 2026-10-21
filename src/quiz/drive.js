import { googleJson, googleFetch, GoogleAuthError, GoogleApiError } from "./google";
import { sheetTabs, tabRange, rowsToCsv } from "./sheets";
import { parseQuestionsCsv, parseScoresCsv } from "./parse";
import { listMedia, putMedia, deleteMedia } from "./storage";
import { isMediaFile } from "./package";
import { mediaKey } from "./model";
import { fmt } from "./files";

// A quiz on Google Drive is a folder with the Sheet (scores tab + questions tab) and a media folder
// ("Zdroje", "source", …). Zdroj cells are paths inside the media folder, e.g. "kolo3/kocka.jpg".
// Syncing copies the questions and every media file into the browser, so the show runs offline.

const DRIVE_API = "https://www.googleapis.com/drive/v3";
const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";
const FOLDER = "application/vnd.google-apps.folder";
const SPREADSHEET = "application/vnd.google-apps.spreadsheet";
const MEDIA_FOLDER = /^(zdroje?|sources?|media)$/i;

// code: "link" (not a Drive link), "access" (not found or not shared with you), "no_sheet", "auth"
// (signed out or expired), "readonly" (cannot edit the Sheet), "upload_denied" (cannot add files),
// "no_folder" (opened from a Sheet whose folder is not shared), "api" (anything else; message from Google)
export class DriveError extends Error {
  constructor(code, message = code) {
    super(message);
    this.code = code;
  }
}

export const toDriveError = (err) => {
  if (err instanceof DriveError) return err;
  if (err instanceof GoogleAuthError) return new DriveError("auth");
  if (err instanceof GoogleApiError && (err.status === 404 || err.status === 403)) return new DriveError("access");
  return new DriveError("api", err.message);
};

export const folderUrl = (id) => `https://drive.google.com/drive/folders/${id}`;
export const sheetUrl = (id, gid) => `https://docs.google.com/spreadsheets/d/${id}/edit${gid != null ? `#gid=${gid}` : ""}`;

// The file or folder id in a Drive folder link, a Sheet link (not a "publish to web" one), or a bare id
export function parseDriveLink(link) {
  const s = String(link || "").trim();
  const match = s.match(/\/folders\/([\w-]{10,})/) || s.match(/\/d\/([\w-]{10,})/) || s.match(/[?&]id=([\w-]{10,})/);
  if (match) return match[1];
  return /^[\w-]{20,}$/.test(s) ? s : null;
}

const getFile = (id) =>
  googleJson(`${DRIVE_API}/files/${id}?fields=id,name,mimeType,parents&supportsAllDrives=true`);

async function listChildren(folderId) {
  const files = [];
  let pageToken = "";
  do {
    const params = new URLSearchParams({
      q: `'${folderId}' in parents and trashed = false`,
      fields: "nextPageToken,files(id,name,mimeType,modifiedTime,size)",
      pageSize: "1000",
      supportsAllDrives: "true",
      includeItemsFromAllDrives: "true",
    });
    if (pageToken) params.set("pageToken", pageToken);
    const data = await googleJson(`${DRIVE_API}/files?${params}`);
    files.push(...data.files);
    pageToken = data.nextPageToken;
  } while (pageToken);
  return files;
}

// Folder, Sheet and media folder for a folder or Sheet link
async function findQuiz(link) {
  const id = parseDriveLink(link);
  if (!id) throw new DriveError("link");
  const item = await getFile(id);

  let folder = null;
  let sheet = null;
  if (item.mimeType === FOLDER) folder = item;
  else if (item.mimeType === SPREADSHEET) {
    sheet = item;
    // The Sheet may be shared without its folder
    if (item.parents?.[0]) folder = await getFile(item.parents[0]).catch(() => null);
  } else throw new DriveError("link");

  const children = folder ? await listChildren(folder.id) : [];
  // Several Sheets in the folder: the most recently changed one
  sheet ??= children.filter((f) => f.mimeType === SPREADSHEET).sort((a, b) => b.modifiedTime.localeCompare(a.modifiedTime))[0];
  if (!sheet) throw new DriveError("no_sheet");
  const media = children.find((f) => f.mimeType === FOLDER && MEDIA_FOLDER.test(f.name.trim()));

  return {
    folderId: folder?.id ?? null,
    folderName: folder?.name ?? sheet.name,
    spreadsheetId: sheet.id,
    spreadsheetName: sheet.name,
    mediaFolderId: media?.id ?? null,
    mediaFolderName: media?.name ?? null,
  };
}

// Every tab with its contents as CSV
async function readTabs(spreadsheetId) {
  const tabs = await sheetTabs(spreadsheetId);
  const params = new URLSearchParams();
  tabs.forEach((tab) => params.append("ranges", tabRange(tab.title)));
  const data = await googleJson(`${SHEETS_API}/${spreadsheetId}/values:batchGet?${params}`);
  return tabs.map((tab, i) => ({ ...tab, csv: rowsToCsv(data.valueRanges[i]?.values) }));
}

// The quiz fields read from Drive: { drive, questionsCsv, questionsSheetUrl, scoresSheetUrl }.
// The scores tab is the one with a team name column, the questions tab one with rounds (usually the
// first and second tab).
export async function readDriveQuiz(link) {
  try {
    const found = await findQuiz(link);
    const tabs = await readTabs(found.spreadsheetId);
    const scores = tabs.find((tab) => parseScoresCsv(tab.csv));
    const others = tabs.filter((tab) => tab !== scores);
    const questions = others.find((tab) => parseQuestionsCsv(tab.csv).length) || others[0] || tabs[0];
    return {
      drive: { ...found, questionsTab: questions.title, scoresTab: scores?.title ?? null },
      questionsCsv: questions.csv,
      questionsSheetUrl: sheetUrl(found.spreadsheetId, questions.sheetId),
      scoresSheetUrl: scores ? sheetUrl(found.spreadsheetId, scores.sheetId) : "",
    };
  } catch (err) {
    throw toDriveError(err);
  }
}

// Media files below a folder, with paths relative to it
async function listMediaTree(folderId, prefix = "") {
  const files = [];
  for (const f of await listChildren(folderId)) {
    if (f.mimeType === FOLDER) files.push(...await listMediaTree(f.id, `${prefix}${f.name}/`));
    else if (isMediaFile(f.name)) files.push({ id: f.id, path: `${prefix}${f.name}`, modified: f.modifiedTime, size: Number(f.size) || 0 });
  }
  return files;
}

// Download, counting bytes as they arrive (a video can be 100 MB)
async function download(file, onBytes) {
  const res = await googleFetch(`${DRIVE_API}/files/${file.id}?alt=media&supportsAllDrives=true`);
  const type = res.headers.get("content-type") || "";
  if (!res.body) return res.blob();
  const reader = res.body.getReader();
  const chunks = [];
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    onBytes(value.length);
  }
  return new Blob(chunks, { type });
}

const UPLOAD_API = "https://www.googleapis.com/upload/drive/v3/files";
const quoted = (name) => `'${String(name).replace(/\\/g, "\\\\").replace(/'/g, "\\'")}'`;

async function findChild(parentId, name, isFolder) {
  const params = new URLSearchParams({
    q: `${quoted(parentId)} in parents and name = ${quoted(name)} and trashed = false and mimeType ${isFolder ? "=" : "!="} '${FOLDER}'`,
    fields: "files(id,name)",
    supportsAllDrives: "true",
    includeItemsFromAllDrives: "true",
  });
  return (await googleJson(`${DRIVE_API}/files?${params}`)).files[0] ?? null;
}

const createFolder = (parentId, name) => googleJson(`${DRIVE_API}/files?supportsAllDrives=true&fields=id,name`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ name, mimeType: FOLDER, parents: [parentId] }),
});

// The quiz's media folder, created as "Zdroje" next to the Sheet when there is none: { id, name }
export async function ensureMediaFolder(drive) {
  try {
    if (drive.mediaFolderId) return { id: drive.mediaFolderId, name: drive.mediaFolderName };
    if (!drive.folderId) throw new DriveError("no_folder");
    return (await findChild(drive.folderId, "Zdroje", true)) ?? (await createFolder(drive.folderId, "Zdroje"));
  } catch (err) {
    throw uploadError(err);
  }
}

const uploadError = (err) => (err instanceof GoogleApiError && err.status === 403 ? new DriveError("upload_denied") : toDriveError(err));

// The file body in one request; XMLHttpRequest because fetch cannot report upload progress
function sendFile(sessionUrl, file, type, onProgress) {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("PUT", sessionUrl);
    xhr.setRequestHeader("Content-Type", type);
    xhr.upload.onprogress = (e) => onProgress?.(e.loaded / (e.total || file.size || 1));
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) resolve(JSON.parse(xhr.responseText));
      else reject(new GoogleApiError(xhr.status, `HTTP ${xhr.status}`));
    };
    xhr.onerror = () => reject(new GoogleApiError(0, "network"));
    xhr.send(file);
  });
}

// Upload a file to `path` in the media folder (e.g. "kolo3/cat.jpg"), creating subfolders on the way,
// and keep a copy in the browser like a synced file. A file already at that path is replaced only when
// confirmReplace(path) says so; returns the path, or null when not replaced. onProgress(0..1).
export async function uploadMedia(quizId, mediaFolderId, path, file, { onProgress, confirmReplace } = {}) {
  try {
    const clean = path.split("/").map((s) => s.trim()).filter(Boolean).join("/");
    const segments = clean.split("/");
    const name = segments.pop();
    let parentId = mediaFolderId;
    for (const segment of segments) parentId = ((await findChild(parentId, segment, true)) ?? (await createFolder(parentId, segment))).id;

    const existing = await findChild(parentId, name, false);
    if (existing && !(await confirmReplace?.(clean))) return null;

    // A resumable session takes files of any size; replacing keeps the file id (and its sharing)
    const type = file.type || "application/octet-stream";
    const query = "uploadType=resumable&supportsAllDrives=true&fields=id,modifiedTime";
    const init = await googleFetch(existing ? `${UPLOAD_API}/${existing.id}?${query}` : `${UPLOAD_API}?${query}`, {
      method: existing ? "PATCH" : "POST",
      headers: { "Content-Type": "application/json; charset=UTF-8", "X-Upload-Content-Type": type, "X-Upload-Content-Length": String(file.size) },
      body: JSON.stringify(existing ? {} : { name, parents: [parentId] }),
    });
    const session = init.headers.get("Location");
    if (!session) throw new DriveError("api", "no upload session");
    const uploaded = await sendFile(session, file, type, onProgress);

    await putMedia(quizId, [{ name: clean, blob: file, driveId: uploaded.id, driveModified: uploaded.modifiedTime }]);
    return clean;
  } catch (err) {
    throw uploadError(err);
  }
}

// Bring the quiz's stored media in line with the Drive folder: download new and changed files, drop
// files that left Drive. Files added by hand (team photos) stay, unless Drive now has a file of that name.
// onProgress({ done, total, bytes, totalBytes }). Returns { downloaded, kept, removed, duplicates }.
export async function syncDriveMedia(quizId, mediaFolderId, onProgress) {
  try {
    const remote = mediaFolderId ? await listMediaTree(mediaFolderId) : [];
    const local = await listMedia(quizId);
    const remotePaths = new Set(remote.map((f) => f.path));
    const remoteKeys = new Set(remote.map((f) => mediaKey(f.path)));

    const stale = local.filter((m) => !remotePaths.has(m.name) && (m.driveId || remoteKeys.has(mediaKey(m.name))));
    for (const m of stale) await deleteMedia(quizId, m.name);

    const byName = new Map(local.map((m) => [m.name, m]));
    const todo = remote.filter((f) => byName.get(f.path)?.driveModified !== f.modified);
    const progress = { done: 0, total: todo.length, bytes: 0, totalBytes: todo.reduce((n, f) => n + f.size, 0) };
    onProgress?.({ ...progress });

    // A few downloads at a time; the first failure stops the rest
    const queue = [...todo];
    let failed = false;
    const worker = async () => {
      while (queue.length && !failed) {
        const file = queue.shift();
        try {
          const blob = await download(file, (n) => { progress.bytes += n; onProgress?.({ ...progress }); });
          await putMedia(quizId, [{ name: file.path, blob, driveId: file.id, driveModified: file.modified }]);
        } catch (err) {
          failed = true;
          throw err;
        }
        progress.done++;
        onProgress?.({ ...progress });
      }
    };
    await Promise.all([worker(), worker(), worker()]);

    // Zdroj cells are matched by file name, so equal names in different subfolders are ambiguous
    const counts = new Map();
    remote.forEach((f) => counts.set(mediaKey(f.path), (counts.get(mediaKey(f.path)) || 0) + 1));
    const duplicates = [...counts].filter(([, n]) => n > 1).map(([key]) => key);

    return { downloaded: todo.length, kept: remote.length - todo.length, removed: stale.length, duplicates };
  } catch (err) {
    throw toDriveError(err);
  }
}

// The quiz fields to save after readDriveQuiz; a scores link set by hand stays when no scores tab is found
export const mergeDrive = (quiz, fields, extra) => ({
  questionsCsv: fields.questionsCsv,
  questionsSheetUrl: fields.questionsSheetUrl,
  scoresSheetUrl: fields.scoresSheetUrl || quiz.scoresSheetUrl,
  drive: { ...quiz.drive, ...fields.drive, ...extra },
});

// The same quiz opened twice: same folder, or the same Sheet when its folder is not shared
export const sameDriveQuiz = (a, b) => Boolean(a && b) && (a.folderId ?? a.spreadsheetId) === (b.folderId ?? b.spreadsheetId);

// Message for a failed Drive call or sign-in; null when the user just closed the sign-in popup
export function driveErrorText(err, t, email) {
  if (err instanceof GoogleAuthError) {
    if (["popup_closed", "superseded"].includes(err.message)) return null;
    return err.message === "scope" ? t.gd_err_scope : t.gd_err_signin;
  }
  if (err?.code === "link") return t.gd_err_link;
  if (err?.code === "access") return fmt(t.gd_err_access, { email: email || "?" });
  if (err?.code === "no_sheet") return t.gd_err_no_sheet;
  if (err?.code === "auth") return t.gd_err_auth;
  if (err?.code === "readonly") return t.gd_err_readonly;
  if (err?.code === "upload_denied") return t.gd_err_upload;
  if (err?.code === "no_folder") return t.gd_err_no_folder;
  return fmt(t.gd_err_api, { message: err?.message || String(err) });
}
