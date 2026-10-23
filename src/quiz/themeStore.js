import { getQuiz, updateQuiz, putMedia, deleteMedia } from "./storage";
import { ensureMediaFolder, uploadMedia } from "./drive";
import { currentToken } from "./google";
import { mediaKey } from "./model";
import { themeFiles } from "./themeFormat";
import { themeForQuizMedia } from "./themePack";
import { CUSTOM_THEME_ID } from "./themes";

// Make a theme pack (readThemePack) the quiz's own theme and choose it. Its files are stored like
// team photos: as quiz media, and for a Drive quiz in the media folder's theme/ subfolder, so everyone
// who opens the quiz gets them with the next sync. Returns the stored theme.
export async function storeQuizTheme(quizId, pack) {
  const { theme, media } = themeForQuizMedia(pack);
  const quiz = await getQuiz(quizId);
  if (quiz?.drive?.folderId && currentToken()) {
    const folder = await ensureMediaFolder(quiz.drive);
    if (!quiz.drive.mediaFolderId) await updateQuiz(quizId, (q) => ({ ...q, drive: { ...q.drive, mediaFolderId: folder.id, mediaFolderName: folder.name } }), "drive");
    for (const m of media) await uploadMedia(quizId, folder.id, `theme/${m.name}`, new File([m.blob], m.name, { type: m.blob.type }), { confirmReplace: () => true });
  } else {
    // The previous theme's files go, unless the new theme uses a file of the same name
    const keep = new Set(media.map((m) => mediaKey(m.name)));
    for (const name of themeFiles(quiz?.customTheme)) if (!keep.has(mediaKey(name))) await deleteMedia(quizId, name);
    await putMedia(quizId, media);
  }
  await updateQuiz(quizId, (q) => ({ ...q, customTheme: theme, settings: { ...q.settings, theme: CUSTOM_THEME_ID } }));
  return theme;
}
