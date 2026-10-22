import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { fetchSheetCsv } from "../quiz/sheets";
import { parseScoresCsv } from "../quiz/parse";
import { getSession, currentToken, subscribeGoogle, loadGoogle, googleEnabled } from "../quiz/google";

const EMPTY = { teams: [], rounds: [], lastSync: null, authNeeded: false };

// A private Sheet read as the signed-in user: once the token runs out, polling fails until they sign in again.
// privateSheet: the Sheet of a Drive quiz, which needs the sign-in even in a browser that never signed in.
const isSheetLink = (url) => /docs\.google\.com\/spreadsheets\/d\/(?!e\/)/.test(url);
const usesGoogle = (url, privateSheet) => googleEnabled && isSheetLink(url) && (privateSheet || Boolean(getSession()?.user));
const signInNeeded = (url, privateSheet) => usesGoogle(url, privateSheet) && !currentToken();

// Polls the quiz's scores Sheet every 10 s. reload() reads it at once and resolves with the new
// scores, or null when the Sheet could not be read. authNeeded: the Sheet needs a Google sign-in;
// usesGoogle: it is read with the sign-in (see useSignInEnding).
export function useLiveScores(url, { privateSheet = false } = {}) {
  const [scores, setScores] = useState(EMPTY);
  const loadRef = useRef(null);

  useEffect(() => {
    if (!url) return;
    let cancelled = false;
    // Ready for a "sign in again" click: the popup must open straight from it
    if (usesGoogle(url, privateSheet)) loadGoogle().catch(() => {});
    const flagSignIn = () => {
      if (!cancelled && signInNeeded(url, privateSheet)) setScores((s) => (s.authNeeded ? s : { ...s, authNeeded: true }));
    };
    const load = async () => {
      try {
        const parsed = parseScoresCsv(await fetchSheetCsv(url));
        if (cancelled) return null;
        // Without the sign-in a private Sheet can come back as Google's sign-in page instead of an error
        if (!parsed) { flagSignIn(); return null; }
        const next = { ...parsed, lastSync: new Date().toLocaleTimeString(), authNeeded: false };
        setScores(next);
        return next;
      } catch (err) {
        console.error("Failed to fetch live leaderboard", err);
        flagSignIn();
        return null;
      }
    };
    loadRef.current = load;
    load();
    const interval = setInterval(load, 10000);
    // Signing in again (in this or another window) reloads at once
    const unsubscribe = subscribeGoogle(() => currentToken() && load());
    return () => { cancelled = true; loadRef.current = null; clearInterval(interval); unsubscribe(); };
  }, [url, privateSheet]);

  const reload = useCallback(() => (loadRef.current ? loadRef.current() : Promise.resolve(null)), []);
  const google = Boolean(url) && usesGoogle(url, privateSheet);
  return useMemo(() => ({ ...(url ? scores : EMPTY), usesGoogle: google, reload }), [url, scores, google, reload]);
}

// Minutes until the Google sign-in ends when that is 10 minutes or less, else null: the presenter window
// offers to renew it before the scores stop (a token lasts about an hour, shorter than most quizzes)
export function useSignInEnding(active) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return;
    const timer = setInterval(() => setNow(Date.now()), 30_000);
    const unsubscribe = subscribeGoogle(() => setNow(Date.now()));
    return () => { clearInterval(timer); unsubscribe(); };
  }, [active]);
  const expires = getSession()?.expires;
  if (!active || !expires || !currentToken()) return null;
  const minutes = Math.ceil((expires - now) / 60_000);
  return minutes <= 10 ? Math.max(1, minutes) : null;
}
