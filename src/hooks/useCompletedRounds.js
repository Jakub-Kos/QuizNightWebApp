import { useState, useEffect, useCallback } from "react";

const readCompleted = (key) => {
  try { return JSON.parse(localStorage.getItem(key) || "[]"); } catch { return []; }
};

// Rounds marked as finished on the Dashboard: per quiz, by round id (titles can repeat), kept in
// localStorage and shared with the other window of the show through the storage event.
export function useCompletedRounds(quizId) {
  const key = `quiznight.completed.${quizId}`;
  const [completed, setCompleted] = useState(() => readCompleted(key));

  useEffect(() => {
    // Before quizzes had ids this was one list for every quiz, keyed by round title
    try { localStorage.removeItem("quizCompletedRounds"); } catch { /* storage unavailable */ }
    const onStorage = (e) => e.key === key && setCompleted(readCompleted(key));
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [key]);

  const update = useCallback((next) => {
    setCompleted(next);
    try { localStorage.setItem(key, JSON.stringify(next)); } catch { /* storage unavailable */ }
  }, [key]);

  return [completed, update];
}
