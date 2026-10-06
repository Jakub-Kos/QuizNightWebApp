import { useState, useEffect } from "react";
import { loadQuizBundle } from "../quiz/model";

// { status: "loading" | "ready" | "missing" | "error", bundle?, message? }; frees media URLs on unmount
export function useQuizBundle(id) {
  const [state, setState] = useState({ status: "loading" });

  useEffect(() => {
    let bundle = null;
    let cancelled = false;
    loadQuizBundle(id)
      .then((b) => {
        bundle = b;
        if (cancelled) b?.dispose();
        else setState(b ? { status: "ready", bundle: b } : { status: "missing" });
      })
      .catch((err) => !cancelled && setState({ status: "error", message: String(err) }));
    return () => { cancelled = true; bundle?.dispose(); };
  }, [id]);

  return state;
}
