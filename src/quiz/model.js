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

// One solid color per TEAM_COLORS gradient, for places that cannot use Tailwind classes (SVG)
const TEAM_HEX = ["#9333ea", "#dc2626", "#2563eb", "#059669", "#f97316", "#eab308", "#0ea5e9", "#57534e"];
export const teamHex = (team, index) => {
  const i = TEAM_COLORS.indexOf(team.color);
  return TEAM_HEX[i >= 0 ? i : index % TEAM_HEX.length];
};

// Stored teams -> what the scenes render: image URL, icon component, a color for every team
export const resolveTeams = (teams, resolveMedia) => teams.map((team, i) => ({
  ...team,
  id: team.id ?? i + 1,
  image: resolveMedia(team.image),
  icon: TEAM_ICONS[team.icon] || TEAM_ICONS.Brain,
  color: team.color || TEAM_COLORS[i % TEAM_COLORS.length],
}));

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

// A web address instead of an uploaded file name (e.g. a team photo hosted elsewhere); used as given
export const isUrl = (name) => /^https?:\/\//i.test(String(name || "").trim());

// CSV cells may say "photo.JPG" or "source/photo.jpg"; match on the lowercase base name
export const mediaKey = (name) => String(name || "").trim().split(/[\\/]/).pop().toLowerCase();

// Quiz + a resolver from media file names to URLs. Call bundle.dispose() to free object URLs.
export async function loadQuizBundle(id) {
  if (id === DEMO_ID) {
    const quiz = await loadDemo();
    return {
      quiz,
      resolveMedia: (name) => (!name ? null : isUrl(name) ? String(name).trim() : `${import.meta.env.BASE_URL}source/${String(name).trim()}`),
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
    resolveMedia: (name) => (!name ? null : isUrl(name) ? String(name).trim() : urls.get(mediaKey(name)) ?? null),
    hasMedia: (name) => isUrl(name) || urls.has(mediaKey(name)),
    dispose: () => urls.forEach((url) => URL.revokeObjectURL(url)),
  };
}
