import { useState, useEffect, useCallback, createContext, useContext } from "react";

// Virtual canvas at 100% content size. Matches the original look of a 1920x1080 screen at 80% browser zoom.
export const BASE_W = 2400;
export const BASE_H = 1350;

// size: content size multiplier (bigger = larger elements, smaller canvas); margin: edge inset in % of the window
export const DEFAULT_DISPLAY = { size: 1, margin: 0 };

// Per-device display calibration, stored separately for the TV and presenter windows
export function useDisplaySettings(role) {
  const key = `quiznight.display.${role}`;
  const [display, setDisplay] = useState(() => {
    try {
      const { size, margin } = JSON.parse(localStorage.getItem(key) || "{}");
      return { size: size ?? DEFAULT_DISPLAY.size, margin: margin ?? DEFAULT_DISPLAY.margin };
    } catch {
      return DEFAULT_DISPLAY;
    }
  });

  const update = useCallback((next) => {
    setDisplay(next);
    try { localStorage.setItem(key, JSON.stringify(next)); } catch { /* storage unavailable */ }
  }, [key]);

  return [display, update];
}

export function useViewport() {
  const [vp, setVp] = useState({ w: window.innerWidth, h: window.innerHeight });
  useEffect(() => {
    const onResize = () => setVp({ w: window.innerWidth, h: window.innerHeight });
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return vp;
}

export function getStageLayout(vp, display) {
  const w = Math.round(BASE_W / display.size);
  const h = Math.round(BASE_H / display.size);
  const m = display.margin / 100;
  const scale = Math.min((vp.w * (1 - 2 * m)) / w, (vp.h * (1 - 2 * m)) / h);
  return { w, h, scale };
}

// Canvas size in px, for animations that cannot use cqh units
export const StageSizeContext = createContext({ w: BASE_W, h: BASE_H });
export const useStageSize = () => useContext(StageSizeContext);
