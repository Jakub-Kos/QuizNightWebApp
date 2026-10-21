import { useState, useEffect } from "react";
import { fetchSheetCsv } from "../quiz/sheets";
import { parseScoresCsv } from "../quiz/parse";
import { getSession, currentToken, subscribeGoogle, loadGoogle } from "../quiz/google";

const EMPTY = { teams: [], rounds: [], lastSync: null, authNeeded: false };

// A private Sheet read as the signed-in user: once the token runs out, polling fails until they sign in again
const isSheetLink = (url) => /docs\.google\.com\/spreadsheets\/d\/(?!e\/)/.test(url);
const signInExpired = (url) => isSheetLink(url) && Boolean(getSession()?.user) && !currentToken();

// Polls the quiz's scores Sheet every 10 s
export function useLiveScores(url) {
  const [scores, setScores] = useState(EMPTY);

  useEffect(() => {
    if (!url) return;
    let cancelled = false;
    // Ready for a "sign in again" click: the popup must open straight from it
    if (isSheetLink(url) && getSession()?.user) loadGoogle().catch(() => {});
    const load = async () => {
      try {
        const parsed = parseScoresCsv(await fetchSheetCsv(url));
        if (parsed && !cancelled) setScores({ ...parsed, lastSync: new Date().toLocaleTimeString(), authNeeded: false });
      } catch (err) {
        console.error("Failed to fetch live leaderboard", err);
        if (!cancelled && signInExpired(url)) setScores((s) => (s.authNeeded ? s : { ...s, authNeeded: true }));
      }
    };
    load();
    const interval = setInterval(load, 10000);
    // Signing in again (in this or another window) reloads at once
    const unsubscribe = subscribeGoogle(() => currentToken() && load());
    return () => { cancelled = true; clearInterval(interval); unsubscribe(); };
  }, [url]);

  return url ? scores : EMPTY;
}
