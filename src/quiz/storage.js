// IndexedDB persistence: localStorage is too small for images and media.
// "quizzes": quiz records (keyPath id). "media": { key: `${quizId}/${name}`, quizId, name, blob }, plus
// driveId and driveModified for files synced from Google Drive (name is then the path in the media folder).
const DB_NAME = "quiznight";
const DB_VERSION = 1;

let dbPromise = null;

function openDb() {
  if (!dbPromise) {
    dbPromise = new Promise((resolve, reject) => {
      const req = indexedDB.open(DB_NAME, DB_VERSION);
      req.onupgradeneeded = () => {
        const db = req.result;
        db.createObjectStore("quizzes", { keyPath: "id" });
        const media = db.createObjectStore("media", { keyPath: "key" });
        media.createIndex("quizId", "quizId");
      };
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  }
  return dbPromise;
}

const promisify = (req) => new Promise((resolve, reject) => {
  req.onsuccess = () => resolve(req.result);
  req.onerror = () => reject(req.error);
});

async function tx(stores, mode, fn) {
  const db = await openDb();
  const t = db.transaction(stores, mode);
  const done = new Promise((resolve, reject) => {
    t.oncomplete = resolve;
    t.onerror = () => reject(t.error);
    t.onabort = () => reject(t.error);
  });
  const result = await fn(t);
  await done;
  return result;
}

// Ask the browser not to evict our data under storage pressure (best effort)
export async function requestPersistence() {
  try { return await navigator.storage?.persist?.(); } catch { return false; }
}

export const listQuizzes = () =>
  tx(["quizzes"], "readonly", (t) => promisify(t.objectStore("quizzes").getAll()));

export const getQuiz = (id) =>
  tx(["quizzes"], "readonly", (t) => promisify(t.objectStore("quizzes").get(id)));

// fn(quiz, source) after every save in this window; source is what the caller passed (e.g. "drive"
// for changes pulled from Google Drive), so a listener can tell its own writes apart
const saveListeners = new Set();
export function onQuizSaved(fn) {
  saveListeners.add(fn);
  return () => saveListeners.delete(fn);
}
const notify = (quiz, source) => saveListeners.forEach((fn) => fn(quiz, source));

export const saveQuiz = async (quiz, source) => {
  const next = { ...quiz, updatedAt: Date.now() };
  await tx(["quizzes"], "readwrite", (t) => promisify(t.objectStore("quizzes").put(next)));
  notify(next, source);
};

// Read-modify-write in one transaction, so windows editing different fields of the same quiz
// (show settings, setup page, score entry) do not overwrite each other's changes
export const updateQuiz = async (id, change, source) => {
  const next = await tx(["quizzes"], "readwrite", async (t) => {
    const store = t.objectStore("quizzes");
    const quiz = await promisify(store.get(id));
    if (!quiz) return null;
    const changed = { ...change(quiz), updatedAt: Date.now() };
    store.put(changed);
    return changed;
  });
  if (next) notify(next, source);
  return next;
};

export const deleteQuiz = (id) =>
  tx(["quizzes", "media"], "readwrite", async (t) => {
    t.objectStore("quizzes").delete(id);
    const keys = await promisify(t.objectStore("media").index("quizId").getAllKeys(id));
    keys.forEach((key) => t.objectStore("media").delete(key));
  });

export const listMedia = (quizId) =>
  tx(["media"], "readonly", (t) => promisify(t.objectStore("media").index("quizId").getAll(quizId)));

// files: [{ name, blob, ...extra fields }]
export const putMedia = (quizId, files) =>
  tx(["media"], "readwrite", (t) => {
    files.forEach(({ name, blob, ...extra }) => t.objectStore("media").put({ ...extra, key: `${quizId}/${name}`, quizId, name, blob }));
  });

export const deleteMedia = (quizId, name) =>
  tx(["media"], "readwrite", (t) => promisify(t.objectStore("media").delete(`${quizId}/${name}`)));
