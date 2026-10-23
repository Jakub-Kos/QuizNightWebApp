import { getQuiz, updateQuiz, putMedia, deleteMedia, listMedia, putLibraryTheme } from "./storage";
import { ensureMediaFolder, uploadMedia } from "./drive";
import { currentToken } from "./google";
import { mediaKey } from "./model";
import { themeFiles, slug, IMAGE_TYPES, FONT_TYPES, LIMITS } from "./themeFormat";
import { themeForQuizMedia } from "./themePack";
import { CUSTOM_THEME_ID } from "./themes";

// A quiz's own theme and its files. The files are stored like team photos: as quiz media named
// theme-<file>, and for a Drive quiz also in the media folder's theme/ subfolder, so everyone who
// opens the quiz gets them with the next sync.

const isDrive = (quiz) => Boolean(quiz?.drive?.folderId && currentToken());

async function mediaFolder(quizId, quiz) {
  const folder = await ensureMediaFolder(quiz.drive);
  if (!quiz.drive.mediaFolderId) await updateQuiz(quizId, (q) => ({ ...q, drive: { ...q.drive, mediaFolderId: folder.id, mediaFolderName: folder.name } }), "drive");
  return folder;
}

async function putThemeMedia(quizId, quiz, media) {
  if (isDrive(quiz)) {
    const folder = await mediaFolder(quizId, quiz);
    for (const m of media) await uploadMedia(quizId, folder.id, `theme/${m.name}`, new File([m.blob], m.name, { type: m.blob.type }), { confirmReplace: () => true });
  } else {
    await putMedia(quizId, media);
  }
}

// Make a theme pack (readThemePack, or an entry of My themes) the quiz's own theme and choose it.
// Returns the stored theme.
export async function storeQuizTheme(quizId, pack) {
  const { theme, media } = themeForQuizMedia(pack);
  const quiz = await getQuiz(quizId);
  await putThemeMedia(quizId, quiz, media);
  await updateQuiz(quizId, (q) => ({ ...q, customTheme: theme, settings: { ...q.settings, theme: CUSTOM_THEME_ID } }));
  await pruneThemeFiles(quizId, theme);
  return theme;
}

// FileError(code, params): "file_type" ({ types }) or "file_size" ({ size } in MB)
export class FileError extends Error {
  constructor(code, params) {
    super(code);
    this.params = params;
  }
}

// One picture or font file for the theme being edited; returns its media name
export async function storeThemeFile(quizId, file, kind) {
  const types = kind === "font" ? FONT_TYPES : IMAGE_TYPES;
  const limit = kind === "font" ? LIMITS.font : LIMITS.image;
  const ext = file.name.split(".").pop().toLowerCase();
  if (!types.includes(ext)) throw new FileError("file_type", { types: types.join(", ") });
  if (file.size > limit) throw new FileError("file_size", { size: limit / 1024 / 1024 });
  const name = `theme-${slug(file.name.replace(/\.[^.]+$/, "")) || kind}.${ext}`;
  await putThemeMedia(quizId, await getQuiz(quizId), [{ name, blob: file }]);
  return name;
}

// Drop theme files the theme no longer uses (a replaced picture, an earlier theme). Only for quizzes
// kept in this browser: a Drive quiz's files stay in its folder, where people manage them themselves.
export async function pruneThemeFiles(quizId, theme) {
  const quiz = await getQuiz(quizId);
  if (quiz?.drive) return;
  const used = new Set(themeFiles(theme).map(mediaKey));
  for (const m of await listMedia(quizId)) {
    const base = m.name.split("/").pop();
    if (base.startsWith("theme-") && !used.has(mediaKey(m.name))) await deleteMedia(quizId, m.name);
  }
}

// Keep a theme in My themes with its files (id: the entry to replace, else a new one); returns the id
export async function saveToLibrary(quizId, theme, id = crypto.randomUUID()) {
  const byKey = new Map((await listMedia(quizId)).map((m) => [mediaKey(m.name), m.blob]));
  const files = themeFiles(theme).map((name) => ({ name, blob: byKey.get(mediaKey(name)) })).filter((f) => f.blob);
  await putLibraryTheme({ id, theme, files });
  return id;
}

// An entry of My themes as the quiz's own theme
export const applyLibraryTheme = (quizId, entry) => storeQuizTheme(quizId, { theme: entry.theme, assets: entry.files });
