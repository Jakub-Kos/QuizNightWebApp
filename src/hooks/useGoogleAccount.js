import { useState, useEffect, useCallback } from "react";
import { googleEnabled, loadGoogle, subscribeGoogle, getSession, currentToken, signIn, signOut } from "../quiz/google";

// { enabled, user, signedIn, signIn, signOut }. user stays known after the token expires, so the page
// can say "sign in again as …"; signedIn means there is a usable token right now.
export function useGoogleAccount() {
  const [, setTick] = useState(0);
  const rerender = useCallback(() => setTick((n) => n + 1), []);

  useEffect(() => {
    if (!googleEnabled) return;
    loadGoogle().catch(() => {});
    return subscribeGoogle(rerender);
  }, [rerender]);

  // Re-render when the token runs out
  const session = getSession();
  const token = currentToken();
  useEffect(() => {
    if (!token) return;
    const timer = setTimeout(rerender, Math.max(0, session.expires - Date.now() - 60_000) + 500);
    return () => clearTimeout(timer);
  }, [token, session, rerender]);

  return { enabled: googleEnabled, user: session?.user ?? null, signedIn: Boolean(token), signIn, signOut };
}
