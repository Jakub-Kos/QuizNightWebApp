import { Brain, Rocket, Zap, Crown, Flame, Shield, Beer, Glasses, Gavel, Microscope } from "lucide-react";
import { DEMO_TEAMS } from "./demoTeams";
import { getQuiz, listMedia } from "./storage";
import { fetchSheetCsv } from "./sheets";

// Teams store the icon by name so they can live in IndexedDB and zip files
export const TEAM_ICONS = { Brain, Rocket, Zap, Crown, Flame, Shield, Beer, Glasses, Gavel, Microscope };

export const TEAM_COLORS = [
  "from-purple-600 to-indigo-900", "from-red-600 to-pink-900",
  "from-blue-600 to-cyan-800", "from-emerald-600 to-teal-900",
  "from-orange-500 to-red-600", "from-yellow-500 to-orange-700",
  "from-sky-500 to-blue-700", "from-stone-700 to-stone-900",
];

export const DEFAULT_SETTINGS = {
  showTime: true,
  startTime: "20:00",
  splitDelay: 3000,
  cycleDuration: 8000,
  headerInterval: 5000,
  language: "cs",
};

export const DEMO_ID = "demo";

export function createQuiz(fields = {}) {
  const now = Date.now();
  return {
    id: crypto.randomUUID(),
    title: "",
    createdAt: now,
    updatedAt: now,
    questionsCsv: "",
    questionsSheetUrl: "",
    scoresSheetUrl: "",
    teams: [],
    ...fields,
    settings: { ...DEFAULT_SETTINGS, ...fields.settings },
  };
}

// The quiz shipped in public/: questions.csv + public/source/ media
async function loadDemo() {
  const res = await fetch(`${import.meta.env.BASE_URL}questions.csv`, { cache: "no-store" });
  return createQuiz({
    id: DEMO_ID,
    title: "Quiz Night (demo)",
    builtin: true,
    questionsCsv: await res.text(),
    scoresSheetUrl: "https://docs.google.com/spreadsheets/d/e/2PACX-1vTc7xeSY_i7arNqlYwJKIukSjZS-cpBVAjsol62xtGE_bxJLHCrdpAi_pMfpijJ3y7zTJzmYnl7CWiL/pub?gid=728177247&single=true&output=csv",
    teams: DEMO_TEAMS,
  });
}

// CSV cells may say "photo.JPG" or "source/photo.jpg"; match on the lowercase base name
export const mediaKey = (name) => String(name || "").trim().split(/[\\/]/).pop().toLowerCase();

// Quiz + a resolver from media file names to URLs. Call bundle.dispose() to free object URLs.
export async function loadQuizBundle(id) {
  if (id === DEMO_ID) {
    const quiz = await loadDemo();
    return {
      quiz,
      resolveMedia: (name) => (name ? `${import.meta.env.BASE_URL}source/${String(name).trim()}` : null),
      hasMedia: () => true,
      dispose: () => {},
    };
  }

  const quiz = await getQuiz(id);
  if (!quiz) return null;

  // Pull the latest questions from a linked Sheet; fall back to the cached copy when offline
  if (quiz.questionsSheetUrl) {
    try { quiz.questionsCsv = await fetchSheetCsv(quiz.questionsSheetUrl); } catch { /* keep cache */ }
  }

  const urls = new Map();
  for (const { name, blob } of await listMedia(id)) urls.set(mediaKey(name), URL.createObjectURL(blob));

  return {
    quiz,
    resolveMedia: (name) => (name ? urls.get(mediaKey(name)) ?? null : null),
    hasMedia: (name) => urls.has(mediaKey(name)),
    dispose: () => urls.forEach((url) => URL.revokeObjectURL(url)),
  };
}
