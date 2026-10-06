import { useState, useEffect } from "react";

// F11 fullscreen is invisible to document.fullscreenElement, so measure the browser UI above the page instead.
// outer* is in screen pixels and inner* in CSS pixels; the width ratio cancels out page zoom.
export function detectFullscreen() {
  if (document.fullscreenElement) return true;
  if (!window.outerWidth || !window.innerWidth) return false;
  const zoom = window.outerWidth / window.innerWidth;
  const browserUiHeight = window.outerHeight - window.innerHeight * zoom;
  return browserUiHeight < 8;
}

export function useIsFullscreen() {
  const [isFullscreen, setIsFullscreen] = useState(detectFullscreen);
  useEffect(() => {
    const update = () => setIsFullscreen(detectFullscreen());
    window.addEventListener("resize", update);
    document.addEventListener("fullscreenchange", update);
    return () => {
      window.removeEventListener("resize", update);
      document.removeEventListener("fullscreenchange", update);
    };
  }, []);
  return isFullscreen;
}

export const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
export const FULLSCREEN_KEY = isMac ? "⌃⌘F" : "F11";

// API fullscreen normally exits on Esc, which the app uses for navigation.
// Keyboard Lock (Chrome/Edge) keeps Esc for the app; leaving then requires holding Esc.
export async function enterFullscreen() {
  try {
    await document.documentElement.requestFullscreen?.();
    await navigator.keyboard?.lock?.(["Escape"]);
  } catch { /* denied or unsupported: the F11 hint stays visible */ }
}

export function toggleFullscreen() {
  if (document.fullscreenElement) document.exitFullscreen();
  else enterFullscreen();
}
