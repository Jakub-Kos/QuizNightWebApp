import { isRoundHidden } from "./parse";

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
export const roundLabel = (round) => `Kolo ${round.number}`;

// quiz.manualScores = { [teamId]: { [roundNumber]: points } } -> the shape the leaderboard reads
// (same as a scores Sheet: { teams: [{ name, scores: { "Kolo 1": 10 } }], rounds: ["Kolo 1", ...] })
export function manualLeaderboard(teams, rounds, manualScores = {}) {
  const scored = scoredRounds(rounds);
  return {
    rounds: scored.map(roundLabel),
    teams: teams.filter((team) => team.name.trim()).map((team) => ({
      name: team.name.trim(),
      scores: Object.fromEntries(scored.map((r) => [roundLabel(r), Number(manualScores[team.id]?.[r.number]) || 0])),
    })),
  };
}
