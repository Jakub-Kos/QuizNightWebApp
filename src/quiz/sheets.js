import Papa from "papaparse";
import { currentToken, googleJson } from "./google";

const SHEETS_API = "https://sheets.googleapis.com/v4/spreadsheets";

// Accepts a Google Sheets share link ("anyone with the link"), a "publish to web" link, or a direct CSV URL
export function toCsvUrl(url) {
  const u = String(url || "").trim();
  if (/output=csv|format=csv|tqx=out:csv/.test(u)) return u;

  const gid = (u.match(/[#&?]gid=(\d+)/) || [])[1];

  const published = u.match(/docs\.google\.com\/spreadsheets\/d\/e\/([\w-]+)/);
  if (published) return `https://docs.google.com/spreadsheets/d/e/${published[1]}/pub?output=csv${gid ? `&single=true&gid=${gid}` : ""}`;

  const shared = u.match(/docs\.google\.com\/spreadsheets\/d\/([\w-]+)/);
  // Without a tab id Google exports the first tab (tab ids are random, so there may be no tab 0)
  if (shared) return `https://docs.google.com/spreadsheets/d/${shared[1]}/export?format=csv${gid ? `&gid=${gid}` : ""}`;

  return u;
}

export class SheetError extends Error {}

// Message for a failed fetchSheetCsv, from the UI translations
export function sheetErrorText(err, t) {
  if (err instanceof SheetError && err.message.startsWith("http_")) return t.ed_err_http;
  if (err instanceof SheetError) return t.ed_err_network;
  return String(err.message || err);
}

// A tab as a range: 'Tab name' (quotes doubled inside)
export const tabRange = (title) => `'${String(title).replace(/'/g, "''")}'`;

// Sheets API values (rows of strings, trailing empty cells left out) -> the CSV the parsers read
export const rowsToCsv = (rows) => Papa.unparse(rows || []);

// Tab ids and names, kept a few minutes so the 10 s score polling makes one call instead of two
const tabCache = new Map(); // spreadsheetId -> { at, tabs }
export async function sheetTabs(spreadsheetId) {
  const cached = tabCache.get(spreadsheetId);
  if (cached && Date.now() - cached.at < 5 * 60_000) return cached.tabs;
  const meta = await googleJson(`${SHEETS_API}/${spreadsheetId}?fields=sheets.properties(sheetId,title,index)`);
  const tabs = meta.sheets.map((s) => s.properties).sort((a, b) => a.index - b.index);
  tabCache.set(spreadsheetId, { at: Date.now(), tabs });
  return tabs;
}

// One tab through the Sheets API as the signed-in user, for Sheets that are not shared by link
async function fetchTabCsv(spreadsheetId, gid) {
  const tabs = await sheetTabs(spreadsheetId);
  const tab = gid == null ? tabs[0] : tabs.find((t) => String(t.sheetId) === String(gid));
  if (!tab) throw new SheetError("http_404");
  const data = await googleJson(`${SHEETS_API}/${spreadsheetId}/values/${encodeURIComponent(tabRange(tab.title))}`);
  return rowsToCsv(data.values);
}

export async function fetchSheetCsv(url) {
  // Signed in to Google: read private Sheets too; on any failure try the public link as before
  const shared = String(url || "").match(/docs\.google\.com\/spreadsheets\/d\/([\w-]{10,})/);
  if (shared && currentToken()) {
    try {
      return await fetchTabCsv(shared[1], (String(url).match(/[#&?]gid=(\d+)/) || [])[1]);
    } catch {
      tabCache.delete(shared[1]); // a renamed or deleted tab
    }
  }

  const csvUrl = toCsvUrl(url);
  // Cache buster: Google otherwise serves copies up to 5 minutes old
  const sep = csvUrl.includes("?") ? "&" : "?";
  let res;
  try {
    res = await fetch(`${csvUrl}${sep}_cb=${Date.now()}`, { cache: "no-store" });
  } catch {
    throw new SheetError("network");
  }
  if (!res.ok) throw new SheetError(`http_${res.status}`);
  const text = await res.text();
  // A private sheet answers with a sign-in page instead of CSV
  if (/^\s*</.test(text)) throw new SheetError("not_shared");
  return text;
}
