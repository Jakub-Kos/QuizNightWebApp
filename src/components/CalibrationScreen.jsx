import { useEffect } from "react";
import { Maximize, RotateCcw, Check } from "lucide-react";
import { DEFAULT_DISPLAY, getStageLayout, useViewport } from "../hooks/useDisplay";
import { toggleFullscreen } from "../hooks/useFullscreen";

const clamp = (v, min, max) => Math.min(max, Math.max(min, v));
const round2 = (v) => Math.round(v * 100) / 100;

const TEXT_SAMPLES = ["text-2xl", "text-4xl", "text-6xl", "text-8xl"];

// Rendered inside the Stage: shows exactly where the 1920x1080 canvas lands on the screen
export function CalibrationPattern({ t }) {
  const corner = "absolute w-24 h-24 border-yellow-400";
  return (
    <div className="absolute inset-0 z-[200] bg-[#050505] text-white font-['League_Spartan'] overflow-hidden">
      {/* 10% grid */}
      <div
        className="absolute inset-0 opacity-20"
        style={{
          backgroundImage: "linear-gradient(to right, #fff 1px, transparent 1px), linear-gradient(to bottom, #fff 1px, transparent 1px)",
          backgroundSize: "10% 10%",
        }}
      />

      {/* Outer edge + corners: all four must be fully visible */}
      <div className="absolute inset-0 border-4 border-yellow-400" />
      <div className={`${corner} top-0 left-0 border-t-[12px] border-l-[12px]`} />
      <div className={`${corner} top-0 right-0 border-t-[12px] border-r-[12px]`} />
      <div className={`${corner} bottom-0 left-0 border-b-[12px] border-l-[12px]`} />
      <div className={`${corner} bottom-0 right-0 border-b-[12px] border-r-[12px]`} />

      {/* Safe area (5%) */}
      <div className="absolute inset-[5%] border-2 border-dashed border-white/40" />

      {/* Center circle: must look round, not oval */}
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full border-4 border-blue-400" />
      <div className="absolute left-1/2 top-0 bottom-0 w-px bg-blue-400/50" />
      <div className="absolute top-1/2 left-0 right-0 h-px bg-blue-400/50" />

      <div className="absolute top-[8%] left-0 right-0 text-center">
        <h1 className="text-7xl font-black uppercase tracking-widest text-yellow-400">{t.calib_title}</h1>
        <p className="text-3xl text-gray-300 mt-4">{t.calib_corners}</p>
        <p className="text-3xl text-gray-300 mt-2">{t.calib_circle}</p>
      </div>

      <div className="absolute bottom-[8%] left-[7%] space-y-2 text-left">
        {TEXT_SAMPLES.map((cls) => (
          <div key={cls} className={`${cls} font-bold`}>{t.calib_text_sample} <span className="text-gray-500 text-xl font-mono">{cls}</span></div>
        ))}
      </div>
    </div>
  );
}

// Rendered outside the Stage so it stays usable at any scale
export function CalibrationControls({ display, onUpdate, onClose, t }) {
  const vp = useViewport();
  const stage = getStageLayout(vp, display);

  const setSize = (v) => onUpdate({ ...display, size: round2(clamp(v, 0.6, 1.5)) });
  const setMargin = (v) => onUpdate({ ...display, margin: clamp(v, 0, 10) });

  // Capture phase + stopPropagation keeps the scenes underneath from reacting to these keys
  useEffect(() => {
    const onKey = (e) => {
      e.stopPropagation();
      if (e.key === "Escape" || e.key === "Enter") onClose();
      else if (e.key === "+" || e.key === "=" || e.key === "ArrowUp") setSize(display.size + 0.01);
      else if (e.key === "-" || e.key === "ArrowDown") setSize(display.size - 0.01);
      else if (e.key === "]" || e.key === "ArrowRight") setMargin(display.margin + 0.5);
      else if (e.key === "[" || e.key === "ArrowLeft") setMargin(display.margin - 0.5);
      else if (e.key === "0") onUpdate(DEFAULT_DISPLAY);
      else if (e.key.toLowerCase() === "f") toggleFullscreen();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  return (
    <div className="fixed inset-0 z-[300] flex items-center justify-center pointer-events-none">
      <div className="pointer-events-auto w-[420px] max-w-[90vw] bg-[#053049]/95 backdrop-blur-md border border-gray-700 rounded-2xl shadow-2xl p-6 space-y-5 text-white font-sans text-sm">
        <div className="flex justify-between font-mono text-xs text-gray-400">
          <span>{t.calib_window}: {vp.w}×{vp.h} @{window.devicePixelRatio}x</span>
          <span>{t.calib_stage}: {stage.w}×{stage.h}</span>
        </div>

        <label className="block">
          <div className="flex justify-between mb-2">
            <span>{t.calib_scale}</span>
            <span className="font-mono text-yellow-400">{Math.round(display.size * 100)}%</span>
          </div>
          <input type="range" min="60" max="150" value={Math.round(display.size * 100)}
            onChange={(e) => setSize(Number(e.target.value) / 100)}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-yellow-500" />
        </label>

        <label className="block">
          <div className="flex justify-between mb-2">
            <span>{t.calib_margin}</span>
            <span className="font-mono text-blue-400">{display.margin}%</span>
          </div>
          <input type="range" min="0" max="10" step="0.5" value={display.margin}
            onChange={(e) => setMargin(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500" />
        </label>

        <p className="text-xs text-gray-400 leading-relaxed">{t.calib_keys}</p>

        <div className="flex gap-2">
          <button onClick={() => onUpdate(DEFAULT_DISPLAY)} className="flex-1 py-3 rounded-xl bg-white/10 hover:bg-white/20 flex items-center justify-center gap-2">
            <RotateCcw size={16} /> {t.calib_reset}
          </button>
          <button onClick={toggleFullscreen} className="flex-1 py-3 rounded-xl bg-white/10 hover:bg-white/20 flex items-center justify-center gap-2">
            <Maximize size={16} /> {t.calib_fullscreen}
          </button>
          <button onClick={onClose} className="flex-1 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold flex items-center justify-center gap-2">
            <Check size={16} /> {t.calib_done}
          </button>
        </div>
      </div>
    </div>
  );
}
