import { useState, useEffect } from "react";

// Hash routes, because GitHub Pages cannot serve deep links:
//   #/                       library
//   #/quiz/<id>              show (TV window)
//   #/quiz/<id>/presenter    show (host window)
//   #/quiz/<id>/edit         quiz setup
function parse(hash) {
  const parts = hash.replace(/^#\/?/, "").split("/").filter(Boolean).map(decodeURIComponent);
  if (parts[0] === "quiz" && parts[1]) {
    const view = parts[2] === "presenter" ? "presenter" : parts[2] === "edit" ? "edit" : "show";
    return { name: view, id: parts[1] };
  }
  return { name: "library" };
}

export function useHashRoute() {
  const [route, setRoute] = useState(() => parse(window.location.hash));
  useEffect(() => {
    const onChange = () => setRoute(parse(window.location.hash));
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return route;
}

export const quizHash = (id, view = "") => `#/quiz/${encodeURIComponent(id)}${view ? `/${view}` : ""}`;
export const navigate = (hash) => { window.location.hash = hash; };
