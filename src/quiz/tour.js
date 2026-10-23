import { quizHash, navigate } from "../hooks/useHashRoute";

// The guided tour of a quiz (started on the demo): { quizId, phase } in localStorage, so the TV and the
// presenter window show the same step. Each phase ends when the show reaches a scene (see nextPhase).
const TOUR_KEY = "quiznight.tour";
export const TOUR_EVENT = "quiznight:tour";

export function readTour() {
  try { return JSON.parse(localStorage.getItem(TOUR_KEY)) || null; } catch { return null; }
}

export function writeTour(tour) {
  try {
    if (tour) localStorage.setItem(TOUR_KEY, JSON.stringify(tour));
    else localStorage.removeItem(TOUR_KEY);
  } catch { /* storage blocked: the tour cannot run */ }
  // The storage event only reaches other windows
  window.dispatchEvent(new Event(TOUR_EVENT));
}

export function startTour(quizId) {
  writeTour({ quizId, phase: "intro" });
  navigate(quizHash(quizId));
}

export const endTour = () => writeTour(null);

// The phase after this one once the show is in scene; presenter: a presenter window is open; visited:
// the scenes this window has shown during the phase. Coming back to the dashboard counts only after the
// round or leaderboard was shown here, as the other window can still be on the dashboard for a moment.
export function nextPhase(phase, scene, presenter, visited) {
  const backFrom = (from) => scene === "dashboard" && visited.includes(from);
  switch (phase) {
    case "intro": return scene === "dashboard" ? "questions" : phase;
    case "questions": return scene === "game" ? "questions_back" : phase;
    case "questions_back": return backFrom("game") ? (presenter ? "answers" : "presenter") : phase;
    case "presenter": return presenter ? "answers" : phase;
    case "answers": return scene === "game" ? "answers_back" : phase;
    case "answers_back": return backFrom("game") ? "leaderboard" : phase;
    case "leaderboard": return scene === "leaderboard" ? "leaderboard_back" : phase;
    case "leaderboard_back": return backFrom("leaderboard") ? "done" : phase;
    default: return phase;
  }
}

// The numbered steps shown in the bubble
export const TOUR_STEPS = 6;
export const tourStep = (phase) => ({
  intro: 1, questions: 2, questions_back: 2, presenter: 3, answers: 4, answers_back: 4, leaderboard: 5, leaderboard_back: 5, done: 6,
})[phase] || 1;
