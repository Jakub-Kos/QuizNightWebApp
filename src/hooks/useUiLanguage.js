import { useState, useCallback } from "react";

const KEY = "quiznight.lang";

// Language of the library and editor; each quiz has its own show language
export function useUiLanguage() {
  const [lang, setLang] = useState(() => {
    try { return localStorage.getItem(KEY) || "cs"; } catch { return "cs"; }
  });
  const update = useCallback((next) => {
    setLang(next);
    try { localStorage.setItem(KEY, next); } catch { /* storage unavailable */ }
  }, []);
  return [lang, update];
}
