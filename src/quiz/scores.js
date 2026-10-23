import { isRoundHidden } from "./parse";
import { isPresent } from "./teams";

// Windows showing a quiz reload its manual scores when this channel announces a change
export const SCORES_CHANNEL = "quiz-scores";
export const announceScores = (quizId) => {
  const channel = new BroadcastChannel(SCORES_CHANNEL);
  channel.postMessage({ quizId });
  channel.close();
};

// Rounds that get scores: the ones the Dashboard shows. Keyed by round number, which stays
// stable when rounds are reordered in the questions sheet.
export const scoredRounds = (rounds) => rounds.filter((r) => !isRoundHidden(r));
// "Kolo 3" / "Round 3": word is the show language's t.round (any capitalisation)
export const roundLabel = (round, word = "Kolo") => `${word.charAt(0).toUpperCase()}${word.slice(1).toLowerCase()} ${round.number}`;

// quiz.manualScores = { [teamId]: { [roundNumber]: points } } -> the shape the leaderboard reads
// (same as a scores Sheet: { teams: [{ name, scores: { "Kolo 1": 10 } }], rounds: ["Kolo 1", ...] })
export function manualLeaderboard(teams, rounds, manualScores = {}, roundWord) {
  const scored = scoredRounds(rounds);
  return {
    rounds: scored.map((r) => roundLabel(r, roundWord)),
    teams: teams.filter((team) => team.name.trim() && isPresent(team)).map((team) => ({
      name: team.name.trim(),
      scores: Object.fromEntries(scored.map((r) => [roundLabel(r, roundWord), Number(manualScores[team.id]?.[r.number]) || 0])),
    })),
  };
}

// Made-up points for previews when a quiz has no scores yet (the theme editor's leaderboard): the
// quiz's teams, or the demo's, over its rounds (four when it has none), the same on every render
export function sampleLeaderboard(teams, rounds, roundWord, fallbackNames) {
  const names = teams.map((team) => String(team.name || "").trim()).filter(Boolean);
  const labels = rounds.length ? rounds : [1, 2, 3, 4].map((n) => `${roundWord || "Kolo"} ${n}`);
  let seed = 11;
  const next = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  return {
    rounds: labels,
    teams: (names.length ? names : fallbackNames).map((name) => ({
      name,
      scores: Object.fromEntries(labels.map((label) => [label, Math.round(next() * 20) / 2])),
    })),
  };
}

// Whether a leaderboard has any points at all
export const hasPoints = (board) => board.teams.some((team) => Object.values(team.scores).some((v) => Number(v) > 0));
