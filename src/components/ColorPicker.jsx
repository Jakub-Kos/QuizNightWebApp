import { useState, useEffect, useRef } from "react";
import { HexColorPicker, HexColorInput } from "react-colorful";
import { Pipette, X } from "lucide-react";

// Colours that work as theme colours on a TV: festive and calm ones, all readable with the scenes' text
const SUGGESTIONS = [
  "#3b82f6", "#00b4d8", "#14b8a6", "#22c55e", "#84cc16", "#eab308", "#f5c542", "#ff7a1a",
  "#ef4444", "#e0303a", "#ec4899", "#a855f7", "#8b3dff", "#6366f1", "#94a3b8", "#ffffff",
];
const DARKS = ["#050505", "#111111", "#0b0614", "#1d0c30", "#06150e", "#0f2a1d", "#06101d", "#0b1d33", "#1a0a0a", "#2a1410"];

// A colour swatch button that opens a picker: a saturation field and hue slider, the hex code, an
// eyedropper (where the browser has one), the theme's other colours and suggestions.
// dark: suggest background colours instead of bright ones.
export default function ColorPicker({ value, onChange, label, themeColors = [], dark = false, t }) {
  const [open, setOpen] = useState(false);
  const box = useRef(null);

  useEffect(() => {
    if (!open) return;
    const close = (e) => { if (!box.current?.contains(e.target)) setOpen(false); };
    const esc = (e) => { if (e.key === "Escape") setOpen(false); };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", esc);
    return () => { document.removeEventListener("pointerdown", close); document.removeEventListener("keydown", esc); };
  }, [open]);

  const pickFromScreen = async () => {
    try {
      const { sRGBHex } = await new window.EyeDropper().open();
      onChange(sRGBHex.toLowerCase());
    } catch { /* cancelled */ }
  };
  const others = [...new Set(themeColors.filter((c) => c && c !== value))];
  const swatches = (colors) => (
    <div className="grid grid-cols-8 gap-1.5">
      {colors.map((c) => (
        <button key={c} type="button" onClick={() => onChange(c)} title={c} aria-label={c}
          className={`aspect-square rounded-md ring-1 ${c === value ? "ring-2 ring-yellow-400" : "ring-white/15 hover:ring-white/50"}`} style={{ background: c }} />
      ))}
    </div>
  );

  return (
    <div ref={box} className="relative shrink-0">
      <button type="button" onClick={() => setOpen((o) => !o)} aria-label={label} aria-expanded={open}
        className="w-11 h-11 rounded-lg ring-1 ring-white/20 hover:ring-white/60 shadow-inner" style={{ background: value }} />
      {open && (
        <div className="absolute left-0 top-full mt-2 z-50 w-64 p-3 rounded-2xl bg-[#16161a] border border-white/15 shadow-2xl space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-bold">{label}</span>
            <button type="button" onClick={() => setOpen(false)} aria-label={t.chk_close} className="p-1 rounded text-gray-500 hover:text-white"><X size={16} /></button>
          </div>
          <HexColorPicker color={value} onChange={onChange} style={{ width: "100%", height: 170 }} />
          <div className="flex items-center gap-2">
            <span className="text-gray-500 font-mono text-sm">#</span>
            <HexColorInput color={value} onChange={(c) => onChange(c.toLowerCase())}
              className="flex-1 min-w-0 bg-black/40 border border-white/10 rounded-lg px-2 py-1.5 text-sm font-mono uppercase text-white focus:outline-none focus:border-yellow-500/60" />
            {typeof window !== "undefined" && "EyeDropper" in window && (
              <button type="button" onClick={pickFromScreen} title={t.cp_eyedropper} aria-label={t.cp_eyedropper}
                className="p-2 rounded-lg bg-white/10 hover:bg-white/20"><Pipette size={16} /></button>
            )}
          </div>
          {others.length > 0 && (
            <div className="space-y-1.5">
              <span className="text-xs text-gray-500">{t.cp_theme}</span>
              {swatches(others.slice(0, 8))}
            </div>
          )}
          <div className="space-y-1.5">
            <span className="text-xs text-gray-500">{t.cp_suggested}</span>
            {swatches(dark ? DARKS : SUGGESTIONS)}
          </div>
        </div>
      )}
    </div>
  );
}
