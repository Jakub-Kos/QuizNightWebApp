import { useState, useEffect, useRef } from "react";
import { getStageLayout, useViewport, StageSizeContext } from "../hooks/useDisplay";

// Scenes are laid out on a virtual canvas that is scaled to fit the window.
// The canvas is a size container, so scenes use cqh/cqw (percent of the canvas) where they would use vh/vw.
// bypass renders children full-window (used for presenter control panels).
// The element tree stays identical either way so toggling does not remount the scenes.
export default function Stage({ display, bypass = false, children }) {
  const vp = useViewport();
  const { w, h, scale } = getStageLayout(vp, display);

  const style = bypass
    ? { position: "absolute", inset: 0, containerType: "size" }
    : {
        position: "absolute",
        width: w,
        height: h,
        left: "50%",
        top: "50%",
        transform: `translate(-50%, -50%) scale(${scale})`,
        transformOrigin: "center",
        containerType: "size",
      };

  return (
    <StageSizeContext.Provider value={bypass ? vp : { w, h }}>
      <div className="fixed inset-0 overflow-hidden bg-black">
        <div style={style}>{children}</div>
      </div>
    </StageSizeContext.Provider>
  );
}

// The same virtual canvas scaled into a box of any width (quiz check thumbnails and the large preview).
// Uses the TV window's content size so previews match what the audience sees.
export function StageFrame({ display, className = "", children }) {
  const ref = useRef(null);
  const [width, setWidth] = useState(0);
  const { w, h } = getStageLayout({ w: 1, h: 1 }, display);

  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => setWidth(entry.contentRect.width));
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} className={`relative overflow-hidden bg-black ${className}`} style={{ aspectRatio: `${w} / ${h}` }}>
      {width > 0 && (
        <StageSizeContext.Provider value={{ w, h }}>
          <div style={{ position: "absolute", left: 0, top: 0, width: w, height: h, transform: `scale(${width / w})`, transformOrigin: "top left", containerType: "size" }}>
            {children}
          </div>
        </StageSizeContext.Provider>
      )}
    </div>
  );
}
