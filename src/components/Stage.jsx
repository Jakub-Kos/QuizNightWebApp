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
