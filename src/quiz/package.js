import { zipSync, unzipSync, strToU8 } from "fflate";
import { createQuiz, mediaKey, isUrl } from "./model";
import { parseQuestionsCsv } from "./parse";
import { listMedia, saveQuiz, putMedia, requestPersistence } from "./storage";

// Package layout:
//   quiz.json       title, settings, teams, sheet links
//   questions.csv
//   media/<file>    question media and team images

const MIME = {
  jpg: "image/jpeg", jpeg: "image/jpeg", png: "image/png", gif: "image/gif", webp: "image/webp",
  avif: "image/avif", svg: "image/svg+xml", bmp: "image/bmp",
  mp3: "audio/mpeg", wav: "audio/wav", ogg: "audio/ogg", m4a: "audio/mp4", aac: "audio/aac", flac: "audio/flac",
  mp4: "video/mp4", webm: "video/webm", mov: "video/quicktime", m4v: "video/mp4",
};

const ext = (name) => name.split(".").pop().toLowerCase();
export const isMediaFile = (name) => ext(name) in MIME;
const baseName = (path) => path.split("/").pop();
const isJunk = (path) => path.startsWith("__MACOSX/") || path.includes("/__MACOSX/") || baseName(path).startsWith(".");

// Media file names a quiz refers to (question column 9 and team images); web addresses are not files
export function referencedMedia(quiz) {
  const names = new Set();
  parseQuestionsCsv(quiz.questionsCsv).forEach((r) => r.questions.forEach((q) => q.media && names.add(q.media)));
  quiz.teams.forEach((t) => t.image && names.add(t.image));
  return [...names].filter((name) => !isUrl(name));
}

// Turn dropped/selected files (a .zip, a folder, or loose files) into { quiz, media }
export async function readImport(files) {
  let entries = []; // { path, blob }

  for (const file of files) {
    const path = file.webkitRelativePath || file.relativePath || file.name;
    if (ext(file.name) === "zip") {
      const unzipped = unzipSync(new Uint8Array(await file.arrayBuffer()));
      for (const [p, data] of Object.entries(unzipped)) {
        if (!p.endsWith("/")) entries.push({ path: p, blob: new Blob([data], { type: MIME[ext(p)] || "" }) });
      }
    } else {
      entries.push({ path, blob: file });
    }
  }
  entries = entries.filter((e) => !isJunk(e.path));

  const find = (pred) => entries.find((e) => pred(baseName(e.path).toLowerCase()));
  const meta = find((n) => n === "quiz.json");
  const csv = find((n) => n === "questions.csv") || find((n) => n.endsWith(".csv"));

  const info = meta ? JSON.parse(await meta.blob.text()) : {};
  const quiz = createQuiz({
    title: info.title || "",
    questionsSheetUrl: info.questionsSheetUrl || "",
    scoresSheetUrl: info.scoresSheetUrl || "",
    teams: Array.isArray(info.teams) ? info.teams : [],
    ...(info.rules && { rules: info.rules }),
    ...(info.prizes && { prizes: info.prizes }),
    settings: info.settings,
    questionsCsv: csv ? await csv.blob.text() : "",
  });
  if (!quiz.title) quiz.title = csv ? baseName(csv.path).replace(/\.csv$/i, "") : "";

  // Last one wins when two folders contain the same file name
  const media = new Map();
  entries.filter((e) => isMediaFile(e.path)).forEach((e) => media.set(mediaKey(e.path), { name: baseName(e.path), blob: e.blob }));

  return { quiz, media: [...media.values()] };
}

// bundle from loadQuizBundle; demo media is fetched from public/source/
export async function exportZip(bundle) {
  const { quiz } = bundle;
  const files = {};

  const { id, createdAt, updatedAt, builtin, questionsCsv, ...meta } = quiz; // eslint-disable-line no-unused-vars
  files["quiz.json"] = strToU8(JSON.stringify(meta, null, 2));
  files["questions.csv"] = strToU8(questionsCsv);

  let media;
  if (quiz.builtin) {
    media = [];
    for (const name of referencedMedia(quiz)) {
      const res = await fetch(bundle.resolveMedia(name));
      // Missing files can come back as the HTML app page instead of a 404
      if (res.ok && !res.headers.get("content-type")?.includes("text/html")) media.push({ name, blob: await res.blob() });
    }
  } else {
    media = await listMedia(quiz.id);
  }
  for (const { name, blob } of media) files[`media/${name}`] = new Uint8Array(await blob.arrayBuffer());

  // Media is already compressed; level 0 just stores it
  return new Blob([zipSync(files, { level: 0 })], { type: "application/zip" });
}

export function downloadBlob(blob, fileName) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export const safeFileName = (title) => (title || "quiz").replace(/[\\/:*?"<>|]+/g, "").trim() || "quiz";

export async function saveImport({ quiz, media }) {
  await saveQuiz(quiz);
  await putMedia(quiz.id, media);
  requestPersistence();
  return quiz.id;
}

// Editable copy of any quiz (including the built-in demo), via the same path as export + import
export async function duplicateQuiz(bundle, title) {
  const zip = await exportZip(bundle);
  const imported = await readImport([new File([zip], "copy.zip")]);
  imported.quiz.title = title;
  return saveImport(imported);
}
