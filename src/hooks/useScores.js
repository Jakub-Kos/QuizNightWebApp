import { useState, useEffect, useMemo } from "react";
import { getQuiz } from "../quiz/storage";
import { SCORES_CHANNEL, manualLeaderboard } from "../quiz/scores";
import { useLiveScores } from "./useLiveScores";

// Scores for the leaderboard: from the linked Sheet, or entered by hand on the score entry page
export function useScores(quiz, rounds) {
  const sheet = useLiveScores(quiz.scoresSheetUrl);
  const usesManual = !quiz.scoresSheetUrl && !quiz.builtin;
  const [manual, setManual] = useState(quiz.manualScores || {});

  useEffect(() => {
    if (!usesManual) return;
    const reload = () => getQuiz(quiz.id).then((q) => q && setManual(q.manualScores || {}));
    const channel = new BroadcastChannel(SCORES_CHANNEL);
    channel.onmessage = (e) => e.data?.quizId === quiz.id && reload();
    const interval = setInterval(reload, 15000); // fallback if a message is missed
    return () => { channel.close(); clearInterval(interval); };
  }, [quiz.id, usesManual]);

  return useMemo(
    () => (usesManual ? { ...manualLeaderboard(quiz.teams, rounds, manual), lastSync: null } : sheet),
    [usesManual, quiz.teams, rounds, manual, sheet],
  );
}
