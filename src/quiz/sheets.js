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

export async function fetchSheetCsv(url) {
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
