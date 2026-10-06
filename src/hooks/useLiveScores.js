import { useState, useEffect } from "react";
import { fetchSheetCsv } from "../quiz/sheets";
import { parseScoresCsv } from "../quiz/parse";

const EMPTY = { teams: [], rounds: [], lastSync: null };

// Polls the quiz's scores Sheet every 10 s
export function useLiveScores(url) {
  const [scores, setScores] = useState(EMPTY);

  useEffect(() => {
    if (!url) return;
    let cancelled = false;
    const load = async () => {
      try {
        const parsed = parseScoresCsv(await fetchSheetCsv(url));
        if (parsed && !cancelled) setScores({ ...parsed, lastSync: new Date().toLocaleTimeString() });
      } catch (err) {
        console.error("Failed to fetch live leaderboard", err);
      }
    };
    load();
    const interval = setInterval(load, 10000);
    return () => { cancelled = true; clearInterval(interval); };
  }, [url]);

  return url ? scores : EMPTY;
}
