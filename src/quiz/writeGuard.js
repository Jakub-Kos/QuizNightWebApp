// A last check on every request that changes something on Google Drive or in a Sheet. The sign-in
// grants the whole Drive, so a bug elsewhere in the app could otherwise touch any file the user has.
// A write is let through only when it targets a quiz this browser has opened (its folder, Sheet,
// media folder or quiznight.json), something inside such a folder, or a file the app created in
// this window. Files are never deleted, and only files created in this window may be trashed.
// Creating an empty folder is allowed anywhere (a new quiz), since it cannot change existing data.

// The request was refused before it left the browser
export class WriteBlockedError extends Error {}

const SHEETS = /^https:\/\/sheets\.googleapis\.com\/v4\/spreadsheets\/([\w-]+)(?:[/:?]|$)/;
const FILE = /^https:\/\/www\.googleapis\.com\/(upload\/)?drive\/v3\/files(?:\/([\w-]+))?(\/permissions(?:\/[\w-]+)?|\/copy)?(?:[?#]|$)/;
const FOLDER = "application/vnd.google-apps.folder";

const text = (body) => (typeof body === "string" ? body : "");

// The parents named in a JSON body, or in the metadata part of a multipart upload
export function bodyParents(body) {
  const list = text(body).match(/"parents"\s*:\s*\[([^\]]*)\]/);
  return list ? [...list[1].matchAll(/"([\w-]+)"/g)].map((m) => m[1]) : [];
}

// What a request would change: null for reads, else { kind, ids } where kind is "edit" (the files in
// ids), "create" (inside the folders in ids), "folder" (a new folder, allowed anywhere) or "trash";
// { refuse } for writes the app never makes
export function writeTargets(method = "GET", url = "", body) {
  method = method.toUpperCase();
  if (method === "GET" || method === "HEAD") return null;

  const sheet = url.match(SHEETS);
  if (sheet) return method === "POST" || method === "PUT" ? { kind: "edit", ids: [sheet[1]] } : { refuse: "sheet_method" };

  const file = url.match(FILE);
  if (!file) return { refuse: "unknown_url" };
  const [, upload, id, sub] = file;
  if (method === "DELETE" && !sub?.startsWith("/permissions")) return { refuse: "delete" };

  if (!id) {
    if (method !== "POST") return { refuse: "unknown_method" };
    const parents = bodyParents(body);
    const isFolder = !upload && text(body).includes(`"${FOLDER}"`);
    if (isFolder && parents.length <= 1) return { kind: "folder", ids: parents };
    return parents.length ? { kind: "create", ids: parents } : { refuse: "no_parent" };
  }
  if (sub === "/copy") {
    const parents = bodyParents(body);
    return parents.length ? { kind: "create", ids: parents } : { refuse: "no_parent" };
  }
  if (sub) return { kind: "edit", ids: [id] }; // sharing the quiz folder
  if (method !== "PATCH") return { refuse: "unknown_method" };
  if (/"trashed"\s*:\s*true/.test(text(body))) return { kind: "trash", ids: [id] };
  return { kind: "edit", ids: [id] };
}

// The ids of a quiz's own files, from its record
export const quizFileIds = (quiz) => {
  const d = quiz?.drive;
  return d ? [d.folderId, d.spreadsheetId, d.mediaFolderId, d.configFileId].filter(Boolean) : [];
};

// A guard over one set of quiz files. roots(): the ids of the quizzes' own files (read on every
// check, so a quiz opened in another window counts); parentsOf(id): its parent folder ids;
// rootFolder(): the id of My Drive, which never counts as a quiz folder even when a Sheet lies in it.
export function createWriteGuard({ roots, parentsOf, rootFolder }) {
  const created = new Set();
  const inside = new Set(); // found below a quiz folder
  const known = new Set(); // quiz files seen so far: no lookup needed (a write on closing the window)
  let myDrive;

  async function allowed(id, rootIds) {
    if (created.has(id) || inside.has(id)) return true;
    myDrive ??= await rootFolder().catch(() => null);
    if (id === myDrive) return false;
    if (rootIds.has(id) || known.has(id)) return true;
    // Walk up the folders; a quiz folder above means the file is part of that quiz
    let level = [id];
    const seen = new Set(level);
    for (let depth = 0; depth < 8 && level.length; depth++) {
      const parents = (await Promise.all(level.map((p) => parentsOf(p).catch(() => [])))).flat();
      if (parents.some((p) => p !== myDrive && (rootIds.has(p) || created.has(p) || inside.has(p)))) {
        inside.add(id);
        return true;
      }
      level = parents.filter((p) => p !== myDrive && !seen.has(p));
      level.forEach((p) => seen.add(p));
    }
    return false;
  }

  return {
    // Throws WriteBlockedError when the request may not be sent
    async check(method, url, body) {
      const target = writeTargets(method, url, body);
      if (!target) return null;
      const refuse = (reason) => {
        console.error("Blocked a Google write outside the quiz:", reason, method, url);
        throw new WriteBlockedError(reason);
      };
      if (target.refuse) refuse(target.refuse);
      if (target.kind === "folder") return target;
      if (target.kind === "trash") return created.has(target.ids[0]) ? target : refuse("trash");
      let rootIds = known;
      if (target.ids.some((id) => !known.has(id))) {
        rootIds = new Set(await roots());
        rootIds.forEach((id) => known.add(id));
      }
      for (const id of target.ids) if (!(await allowed(id, rootIds))) refuse(`outside:${id}`);
      return target;
    },
    // A file the app just created; it may be written and trashed from now on
    created(id) {
      if (id) created.add(id);
    },
  };
}
