import { isRoundHidden } from "./parse";
import { isUrl } from "./model";
import { MAX_SORT_ITEMS, sheetLetters, sortLayout } from "./sort";

// Types QuestionScreen renders. Image/Video (and any other type) show their media inline with a written answer;
// PImage/PVideo show the media full screen first.
const KNOWN_TYPES = ["Written", "Numeric", "ABCD", "Yes/No", "Top5", "Sort", "PImage", "PVideo", "Audio", "Image", "Video"];
const MEDIA_REQUIRED = ["PImage", "PVideo", "Audio", "Image", "Video"];

const IMAGE_EXT = ["jpg", "jpeg", "png", "gif", "webp", "avif", "svg", "bmp"];
const AUDIO_EXT = ["mp3", "wav", "ogg", "m4a", "aac", "flac"];
const VIDEO_EXT = ["mp4", "webm", "mov", "m4v"];

const ext = (name) => String(name).split(".").pop().toLowerCase();
const blank = (v) => v === undefined || v === null || String(v).trim() === "";

// What kind of file the screen will try to show for this type
function expectedMediaKind(type) {
  if (type === "Audio") return "audio";
  if (type.includes("Video")) return "video";
  return "image";
}

function mediaKind(name) {
  const e = ext(name);
  if (IMAGE_EXT.includes(e)) return "image";
  if (AUDIO_EXT.includes(e)) return "audio";
  if (VIDEO_EXT.includes(e)) return "video";
  return "other";
}

// Returns [{ level: "error" | "warning" | "info", code, params, roundIndex, qIndex }]
// qIndex -1 = the round itself, roundIndex -1 = the whole quiz. Messages live in translations as chk_<code>.
export function validateQuiz(quiz, rounds, hasMedia) {
  const issues = [];
  const add = (level, code, roundIndex = -1, qIndex = -1, params = {}) => issues.push({ level, code, params, roundIndex, qIndex });

  if (rounds.length === 0) add("error", "no_rounds");
  if (quiz.teams.length === 0) add("info", "no_teams");
  if (!quiz.scoresSheetUrl && !quiz.builtin) add("info", "no_scores");

  const names = quiz.teams.map((t) => String(t.name || "").trim().toLowerCase());
  quiz.teams.forEach((team, i) => {
    if (!names[i]) add("warning", "team_no_name");
    else if (names.indexOf(names[i]) !== i) add("warning", "team_duplicate", -1, -1, { team: team.name });
  });

  rounds.forEach((round, ri) => {
    if (isRoundHidden(round)) add("warning", "round_hidden", ri);

    const seen = new Set();
    round.questions.forEach((q, qi) => {
      const issue = (level, code, params) => add(level, code, ri, qi, params);
      const type = q.type;

      if (seen.has(q.id)) issue("warning", "duplicate_number", { number: q.id });
      seen.add(q.id);

      if (!KNOWN_TYPES.includes(type)) issue("warning", "type_unknown", { type });

      if (blank(q.text) && !["PImage", "PVideo"].includes(type)) issue("warning", "no_text");
      if (String(q.text || "").length > 250) issue("warning", "long_text", { length: String(q.text).length });

      if (type === "Top5") {
        const answers = q.answer.filter((a) => !blank(a));
        if (answers.length !== 5) issue("warning", "top5_count", { count: answers.length });
      } else if (type === "Sort") {
        const items = q.items || [];
        if (items.length < 2) issue("error", "sort_count_low", { count: items.length });
        if (items.length > MAX_SORT_ITEMS) issue("warning", "sort_count_high", { count: items.length, max: MAX_SORT_ITEMS });
        const letters = sheetLetters(items);
        if (items.length >= 2 && letters === "none") issue("warning", "sort_no_letters", { letters: sortLayout(q).correct.map((it) => it.letter).join(", ") });
        if (letters === "invalid") issue("error", "sort_bad_letters", { letters: items.map((it) => it.letter || "–").join(", "), max: String.fromCharCode(64 + items.length) });
        if (letters === "ok" && items.length > 1 && items.every((it, i) => it.letter === String.fromCharCode(65 + i))) issue("warning", "sort_in_order");
        const pictures = items.filter((it) => it.media);
        if (pictures.length && pictures.length < items.length) issue("warning", "sort_some_pictures", { count: pictures.length, total: items.length });
        pictures.forEach(({ media }) => {
          if (!isUrl(media) && !hasMedia(media)) issue("error", "media_missing", { file: media });
          else if (!["image", "other"].includes(mediaKind(media))) issue("warning", "media_kind", { file: media, type });
        });
      } else if (blank(q.answer)) {
        issue("warning", "no_answer");
      }

      if (type === "ABCD") {
        const missingOpts = ["A", "B", "C", "D"].filter((o) => blank(q.options[o]));
        if (missingOpts.length) issue("error", "abcd_options", { missing: missingOpts.join(", ") });
        const answer = String(q.answer || "").trim().toUpperCase();
        if (!blank(q.answer) && !["A", "B", "C", "D"].includes(answer)) issue("error", "abcd_answer", { answer: q.answer });
      }

      if (type === "Yes/No" && !blank(q.answer) && !["yes", "no"].includes(String(q.answer).trim().toLowerCase())) {
        issue("error", "yesno_answer", { answer: q.answer });
      }

      if (type === "Numeric" && !blank(q.answer) && !/\d/.test(String(q.answer))) {
        issue("warning", "numeric_answer", { answer: q.answer });
      }

      if (q.media) {
        if (!isUrl(q.media) && !hasMedia(q.media)) issue("error", "media_missing", { file: q.media });
        const kind = mediaKind(q.media);
        if (kind !== "other" && kind !== expectedMediaKind(type)) issue("warning", "media_kind", { file: q.media, type });
      } else if (MEDIA_REQUIRED.includes(type)) {
        issue("error", "media_required", { type });
      }
    });
  });

  quiz.teams.forEach((team) => {
    if (team.image && !isUrl(team.image) && !hasMedia(team.image)) add("error", "team_image_missing", -1, -1, { team: team.name, file: team.image });
  });

  return issues;
}

// Step that shows a question with its text and media, and the step that reveals the answer
export function previewStep(type, view) {
  const pre = type === "PImage" || type === "PVideo";
  if (view === "answer") return pre ? 3 : 2;
  return pre ? 2 : 1;
}
