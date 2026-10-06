import { useState, useEffect, useRef, useCallback } from "react";
import { listMedia } from "../quiz/storage";
import { mediaKey } from "../quiz/model";

// Object URLs for a quiz's stored media: { urlFor(name), reload() }; URLs are freed on reload and unmount
export function useMediaUrls(quizId) {
  const [urls, setUrls] = useState(() => new Map());
  const current = useRef(new Map());

  const load = useCallback(async () => {
    const next = new Map((await listMedia(quizId)).map((m) => [mediaKey(m.name), URL.createObjectURL(m.blob)]));
    current.current.forEach((url) => URL.revokeObjectURL(url));
    current.current = next;
    return next;
  }, [quizId]);

  useEffect(() => {
    let cancelled = false;
    load().then((next) => !cancelled && setUrls(next));
    return () => { cancelled = true; };
  }, [load]);
  useEffect(() => () => current.current.forEach((url) => URL.revokeObjectURL(url)), []);

  const reload = useCallback(async () => setUrls(await load()), [load]);
  const urlFor = useCallback((name) => (name ? urls.get(mediaKey(name)) ?? (/^https?:\/\//i.test(name) ? name : null) : null), [urls]);
  return { urlFor, reload };
}
