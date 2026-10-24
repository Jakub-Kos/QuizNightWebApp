import { useEffect, useMemo, useRef, useState } from "react";
import Confetti from "react-confetti";
import { useTheme } from "../quiz/themes";
import { useStageSize } from "../hooks/useDisplay";
import { ThemeDecorations } from "./Theme";

// What bursts over the TV when the final standings appear (theme.celebration): confetti, fireworks, or
// a storm of the theme's own decoration (confetti when it has none). Runs once for `duration` ms,
// fades out and calls onDone. type overrides the theme's (the theme editor's preview).
const CLASSIC = { primary: "#3b82f6", accent: "#eab308" };

export default function Celebration({ type: forced, duration = 8000, onDone }) {
  const theme = useTheme();
  const { w, h } = useStageSize();
  const [fading, setFading] = useState(false);
  const type = forced ?? theme.celebration?.type;
  const colors = useMemo(() => [theme.colors?.accent || CLASSIC.accent, theme.colors?.primary || CLASSIC.primary, "#ffffff", "#facc15", "#ff5d8f", "#4ade80"], [theme.colors]);

  useEffect(() => {
    const fade = setTimeout(() => setFading(true), duration - 1200);
    const done = setTimeout(() => onDone?.(), duration);
    return () => { clearTimeout(fade); clearTimeout(done); };
  }, [duration, onDone]);

  if (!type) return null;
  const shown = type === "decoration" && !theme.decoration?.type ? "confetti" : type;
  return (
    <div className="absolute inset-0 z-[45] pointer-events-none transition-opacity duration-1000" style={{ opacity: fading ? 0 : 1 }}>
      {shown === "confetti" && <Confetti width={w} height={h} numberOfPieces={Math.round(w / 4)} recycle={false} gravity={0.18} colors={colors} tweenDuration={4000} />}
      {shown === "fireworks" && <Fireworks width={w} height={h} colors={colors} duration={duration - 1500} />}
      {shown === "decoration" && <ThemeDecorations density={3} />}
    </div>
  );
}

// Rockets rising from the bottom and bursting into sparks, on a canvas the size of the stage
function Fireworks({ width, height, colors, duration }) {
  const canvas = useRef(null);
  useEffect(() => {
    const ctx = canvas.current.getContext("2d");
    const rockets = [];
    const sparks = [];
    const start = performance.now();
    let last = start;
    let nextLaunch = start;
    let frame;
    const pick = () => colors[Math.floor(Math.random() * colors.length)];
    const tick = (now) => {
      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      if (now - start < duration && now >= nextLaunch) {
        rockets.push({ x: width * (0.12 + Math.random() * 0.76), y: height, vy: -height * (0.9 + Math.random() * 0.3), top: height * (0.12 + Math.random() * 0.3), color: pick() });
        nextLaunch = now + 250 + Math.random() * 450;
      }
      ctx.clearRect(0, 0, width, height);
      ctx.globalCompositeOperation = "lighter";
      for (let i = rockets.length - 1; i >= 0; i--) {
        const r = rockets[i];
        r.y += r.vy * dt;
        ctx.fillStyle = r.color;
        ctx.fillRect(r.x - 2, r.y, 4, 18);
        if (r.y <= r.top) {
          rockets.splice(i, 1);
          const count = 60 + Math.floor(Math.random() * 40);
          const speed = height * (0.25 + Math.random() * 0.2);
          for (let k = 0; k < count; k++) {
            const angle = (k / count) * Math.PI * 2;
            const v = speed * (0.6 + Math.random() * 0.4);
            sparks.push({ x: r.x, y: r.y, vx: Math.cos(angle) * v, vy: Math.sin(angle) * v, life: 1, color: Math.random() < 0.8 ? r.color : "#ffffff" });
          }
        }
      }
      for (let i = sparks.length - 1; i >= 0; i--) {
        const s = sparks[i];
        const px = s.x;
        const py = s.y;
        s.vx *= 0.985;
        s.vy = s.vy * 0.985 + height * 0.35 * dt;
        s.x += s.vx * dt;
        s.y += s.vy * dt;
        s.life -= dt / 1.6;
        if (s.life <= 0) {
          sparks.splice(i, 1);
          continue;
        }
        ctx.globalAlpha = s.life;
        ctx.strokeStyle = s.color;
        ctx.lineWidth = 4;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.lineTo(s.x, s.y);
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
      if (now - start < duration + 2500 || sparks.length) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [width, height, colors, duration]);
  return <canvas ref={canvas} width={width} height={height} className="absolute inset-0 w-full h-full" />;
}
