import { useEffect, useMemo, useId } from "react";
import { ThemeContext, THEMES, themeStyle, loadThemeFonts, useTheme } from "../quiz/themes";

// The theme's colours and fonts for everything inside (see quiz/themes.js). resolveAsset(name) gives
// a URL for the theme's files (stored with the quiz's media).
export function ThemeScope({ theme = THEMES[0], resolveAsset, className = "", children }) {
  useEffect(() => loadThemeFonts(theme, resolveAsset), [theme, resolveAsset]);
  const value = useMemo(() => ({ ...theme, imageUrl: theme.background?.image ? resolveAsset?.(theme.background.image) ?? null : null }), [theme, resolveAsset]);
  return (
    <ThemeContext.Provider value={value}>
      <div className={className} style={themeStyle(theme)}>{children}</div>
    </ThemeContext.Provider>
  );
}

// Small deterministic random numbers, so a decoration looks the same on every render
function seeded(seed) {
  let s = seed;
  return () => {
    s = (s * 16807) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

function Moon() {
  return (
    <div className="absolute right-[5cqw] top-[8cqh] w-[15cqh] h-[15cqh] rounded-full"
      style={{
        background: "radial-gradient(circle at 38% 36%, #fff6dc 0%, #ffd889 45%, #e6a14a 100%)",
        boxShadow: "0 0 10cqh rgba(255, 190, 100, 0.35), 0 0 30cqh rgba(255, 140, 40, 0.15)",
        opacity: 0.85,
      }}>
      <div className="absolute left-[22%] top-[52%] w-[18%] h-[18%] rounded-full bg-[#d8913f]/40" />
      <div className="absolute left-[58%] top-[30%] w-[12%] h-[12%] rounded-full bg-[#d8913f]/35" />
      <div className="absolute left-[55%] top-[64%] w-[9%] h-[9%] rounded-full bg-[#d8913f]/30" />
    </div>
  );
}

// Fog drifting slowly along the bottom edge, in two layers moving at different speeds
const FOG_BLOBS = "radial-gradient(ellipse 18% 60% at 10% 80%, rgba(150, 110, 200, 0.5), transparent 70%), radial-gradient(ellipse 22% 50% at 35% 90%, rgba(130, 90, 180, 0.45), transparent 70%), radial-gradient(ellipse 16% 55% at 62% 85%, rgba(160, 120, 210, 0.5), transparent 70%), radial-gradient(ellipse 20% 60% at 85% 88%, rgba(140, 100, 190, 0.45), transparent 70%)";
function Fog() {
  return (
    <>
      <div className="absolute inset-x-0 bottom-0 h-[30cqh]" style={{ background: "linear-gradient(to top, rgba(120, 80, 170, 0.22), transparent)" }} />
      {[{ time: 70, opacity: 0.35, delay: 0 }, { time: 110, opacity: 0.25, delay: -40 }].map((layer, i) => (
        <div key={i} className="theme-anim absolute bottom-0 left-0 w-[200%] h-[28cqh]"
          style={{ backgroundImage: FOG_BLOBS, backgroundSize: "50% 100%", opacity: layer.opacity, filter: "blur(2cqh)", animation: `theme-drift ${layer.time}s linear ${layer.delay}s infinite` }} />
      ))}
    </>
  );
}

// A jack-o'-lantern whose face flickers like a candle
function Pumpkin({ x, y, scale, delay, glowId }) {
  return (
    <g transform={`translate(${x} ${y}) scale(${scale})`}>
      <circle r="70" fill={`url(#${glowId})`} className="theme-anim" style={{ animation: `theme-flicker 3.1s ease-in-out ${delay}s infinite` }} />
      <rect x="-4" y="-38" width="8" height="14" rx="3" fill="#3f5a1f" transform="rotate(12)" />
      <ellipse cx="-15" cy="0" rx="20" ry="25" fill="#c4560a" />
      <ellipse cx="15" cy="0" rx="20" ry="25" fill="#c4560a" />
      <ellipse cx="0" cy="0" rx="21" ry="27" fill="#e0680f" />
      <path d="M-8 -24 Q-12 0 -8 25 M8 -24 Q12 0 8 25" stroke="#a9470a" strokeWidth="2" fill="none" />
      <g fill="#ffd257" className="theme-anim" style={{ animation: `theme-flicker 3.1s ease-in-out ${delay}s infinite` }}>
        <path d="M-17 -8 L-7 -8 L-12 -17 Z M7 -8 L17 -8 L12 -17 Z M-3 -1 L3 -1 L0 -6 Z" />
        <path d="M-19 6 L-13 13 L-8 8 L-3 14 L3 8 L8 14 L13 8 L19 6 Q0 24 -19 6 Z" />
      </g>
    </g>
  );
}

const SILHOUETTE = "#06020b";
// Hills with tombstones, a bare tree and glowing jack-o'-lanterns along the bottom edge
function Graveyard() {
  const glowId = useId();
  const stone = (x, y, w, h, tilt, cross) => (
    <g key={x} transform={`translate(${x} ${y}) rotate(${tilt})`} fill="#0e0718">
      {cross
        ? <path d={`M${-w / 6} ${-h} h${w / 3} v${h / 4} h${w / 3} v${w / 3} h${-w / 3} v${h * 0.75 - w / 3} h${-w / 3} v${-(h * 0.75 - w / 3)} h${-w / 3} v${-w / 3} h${w / 3} Z`} />
        : <path d={`M${-w / 2} 0 V${-h + w / 2} A${w / 2} ${w / 2} 0 0 1 ${w / 2} ${-h + w / 2} V0 Z`} />}
    </g>
  );
  return (
    <svg className="absolute inset-x-0 bottom-0 w-full h-[30cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice">
      <defs>
        <radialGradient id={glowId}>
          <stop offset="0%" stopColor="#ffb347" stopOpacity="0.55" />
          <stop offset="100%" stopColor="#ff7a1a" stopOpacity="0" />
        </radialGradient>
      </defs>
      <path d="M0 300 L0 222 Q180 182 400 210 Q620 238 820 205 Q1050 170 1260 208 Q1430 236 1600 196 L1600 300 Z" fill="#12091e" />
      {[stone(290, 214, 34, 50, -6), stone(352, 220, 30, 58, 4, true), stone(420, 216, 26, 38, 8), stone(1130, 196, 36, 54, 3), stone(1196, 204, 28, 62, -5, true), stone(1300, 218, 30, 42, -10)]}
      <path d="M0 300 L0 240 Q260 212 520 236 Q800 262 1080 230 Q1340 204 1600 242 L1600 300 Z" fill={SILHOUETTE} />
      <g stroke={SILHOUETTE} strokeLinecap="round" fill="none">
        <path d="M1462 250 C1458 214 1450 180 1438 120" strokeWidth="18" />
        <path d="M1446 170 C1420 150 1392 146 1366 120 M1442 140 C1470 118 1500 112 1526 86 M1438 124 C1430 104 1432 92 1420 78" strokeWidth="8" />
        <path d="M1366 120 L1344 112 M1380 134 L1366 150 M1526 86 L1548 84 M1500 108 L1512 126 M1420 78 L1408 70 M1420 78 L1428 66" strokeWidth="4" />
      </g>
      <Pumpkin x={205} y={232} scale={1} delay={0} glowId={glowId} />
      <Pumpkin x={640} y={244} scale={0.7} delay={-1.3} glowId={glowId} />
      <Pumpkin x={1010} y={230} scale={0.9} delay={-2.1} glowId={glowId} />
    </svg>
  );
}

// A cobweb in the top left corner with a spider bobbing on its thread
function Cobweb() {
  const spokes = [0, 18, 36, 54, 72, 90].map((deg) => (deg * Math.PI) / 180);
  const at = (r, a) => [r * Math.cos(a), r * Math.sin(a)];
  const rings = [45, 90, 135, 180].map((r) => spokes.slice(1).map((a, i) => {
    const [x0, y0] = at(r, spokes[i]);
    const [x1, y1] = at(r, a);
    const [cx, cy] = at(r * 0.82, (a + spokes[i]) / 2);
    return `${i ? "" : `M${x0} ${y0} `}Q${cx} ${cy} ${x1} ${y1}`;
  }).join(" "));
  return (
    <svg className="absolute left-0 top-0 w-[24cqh] h-[24cqh]" viewBox="0 0 200 200">
      <g stroke="rgba(230, 220, 255, 0.22)" strokeWidth="1.2" fill="none">
        {spokes.map((a, i) => <line key={i} x1="0" y1="0" x2={at(200, a)[0]} y2={at(200, a)[1]} />)}
        {rings.map((d, i) => <path key={i} d={d} />)}
      </g>
      <g className="theme-anim" style={{ animation: "theme-bob 4s ease-in-out infinite" }}>
        <line x1="120" y1="40" x2="120" y2="118" stroke="rgba(230, 220, 255, 0.3)" strokeWidth="1" />
        <g transform="translate(120 124)" stroke="#05020a" strokeWidth="2" fill="none">
          <path d="M-4 -2 L-14 -10 M-4 0 L-16 -2 M-4 2 L-15 7 M-3 4 L-12 13 M4 -2 L14 -10 M4 0 L16 -2 M4 2 L15 7 M3 4 L12 13" />
          <ellipse rx="6" ry="7" fill="#05020a" />
          <circle cy="-8" r="4" fill="#05020a" />
          <circle cx="-1.5" cy="-9" r="0.9" fill="#ff7a1a" stroke="none" />
          <circle cx="1.5" cy="-9" r="0.9" fill="#ff7a1a" stroke="none" />
        </g>
      </g>
    </svg>
  );
}

const BULBS = ["#ff3b3b", "#ffd447", "#39d353", "#4aa8ff"];
// A string of lights hanging along the top edge, twinkling
function Lights() {
  const segments = 9;
  const sag = 3.2; // cqh
  const bulbs = useMemo(() => {
    const list = [];
    for (let s = 0; s < segments; s++) {
      for (let k = 1; k <= 3; k++) {
        const t = k / 4;
        list.push({ x: ((s + t) / segments) * 100, y: 0.6 + sag * Math.sin(Math.PI * t), color: BULBS[list.length % BULBS.length], delay: (list.length * 0.37) % 2.4 });
      }
    }
    return list;
  }, []);
  const wire = Array.from({ length: segments }, (_, s) => {
    const x0 = (s / segments) * 1000;
    const x1 = ((s + 1) / segments) * 1000;
    return `${s ? "" : `M${x0},6 `}Q${(x0 + x1) / 2},${6 + sag * 20} ${x1},6`;
  }).join(" ");
  return (
    <div className="absolute inset-x-0 top-0 h-[7cqh]">
      <svg className="absolute inset-0 w-full h-full" viewBox="0 0 1000 70" preserveAspectRatio="none">
        <path d={wire} fill="none" stroke="#0d3322" strokeWidth="2" vectorEffect="non-scaling-stroke" />
      </svg>
      {bulbs.map((b, i) => (
        <span key={i} className="theme-anim absolute w-[0.9cqh] h-[1.4cqh] rounded-full -translate-x-1/2"
          style={{ left: `${b.x}%`, top: `${b.y}cqh`, background: b.color, boxShadow: `0 0 1.6cqh ${b.color}`, animation: `theme-twinkle 2.4s ease-in-out ${b.delay}s infinite` }} />
      ))}
    </div>
  );
}

// A city at night along the bottom edge: two rows of buildings, some windows lit, a few flickering
function Skyline() {
  const { back, front, windows } = useMemo(() => {
    const rand = seeded(41);
    const row = (minH, maxH) => {
      const list = [];
      for (let x = 0; x < 1600;) {
        const w = 40 + rand() * 70;
        list.push({ x, w, h: minH + rand() * (maxH - minH), spire: rand() < 0.15 });
        x += w + rand() * 6;
      }
      return list;
    };
    const back = row(90, 210);
    const front = row(50, 150);
    const windows = front.flatMap((b) => {
      const cols = Math.floor((b.w - 10) / 14);
      const rows = Math.floor((b.h - 16) / 18);
      const cells = [];
      for (let c = 0; c < cols; c++) for (let r = 0; r < rows; r++) {
        if (rand() < 0.28) cells.push({ x: b.x + 8 + c * 14, y: 300 - b.h + 10 + r * 18, flicker: rand() < 0.12, delay: -rand() * 6 });
      }
      return cells;
    });
    return { back, front, windows };
  }, []);
  const bar = (b) => `M${b.x} 300 V${300 - b.h} h${b.w} V300 Z${b.spire ? ` M${b.x + b.w / 2 - 2} ${300 - b.h} v-30 h4 v30 Z` : ""}`;
  return (
    <svg className="absolute inset-x-0 bottom-0 w-full h-[34cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice">
      <path d={back.map(bar).join(" ")} fill="#0e0b1a" />
      <path d={front.map(bar).join(" ")} fill="#05030a" />
      {windows.map((w, i) => (
        <rect key={i} x={w.x} y={w.y} width="6" height="9" fill="#ffd27a" opacity="0.75"
          className={w.flicker ? "theme-anim" : undefined} style={w.flicker ? { animation: `theme-twinkle 3.5s ease-in-out ${w.delay}s infinite` } : undefined} />
      ))}
    </svg>
  );
}

// Waves rolling along the bottom edge in the theme's main colour, three layers at different speeds
const WAVE = "M0 40 Q100 0 200 40 T400 40 T600 40 T800 40 T1000 40 T1200 40 T1400 40 T1600 40 V120 H0 Z";
function Waves() {
  return (
    <div className="absolute inset-x-0 bottom-0 h-[24cqh] overflow-hidden">
      {[{ y: 0, time: 26, opacity: 0.25, shade: 400 }, { y: 30, time: 18, opacity: 0.35, shade: 600 }, { y: 58, time: 12, opacity: 0.55, shade: 800 }].map((layer, i) => (
        <svg key={i} viewBox="0 0 1600 120" preserveAspectRatio="none" className="theme-anim absolute left-0 w-[200%] h-full"
          style={{ top: `${layer.y}%`, opacity: layer.opacity, animation: `theme-drift ${layer.time}s linear infinite` }}>
          <path d={WAVE} style={{ fill: `var(--color-primary-${layer.shade})` }} transform="scale(0.5 1)" />
          <path d={WAVE} style={{ fill: `var(--color-primary-${layer.shade})` }} transform="translate(800 0) scale(0.5 1)" />
        </svg>
      ))}
    </div>
  );
}

const FLOWERS = ["#f9a8d4", "#fde68a", "#ffffff", "#c4b5fd", "#fca5a5"];
// A meadow along the bottom edge: grass swaying in two layers, with flowers
function Meadow() {
  const { blades, flowers } = useMemo(() => {
    const rand = seeded(53);
    const blades = Array.from({ length: 160 }, (_, i) => ({ x: (i / 160) * 1600 + rand() * 10, h: 30 + rand() * 60, lean: (rand() - 0.5) * 30, back: rand() < 0.5 }));
    const flowers = Array.from({ length: 26 }, () => ({ x: 20 + rand() * 1560, y: 238 + rand() * 40, r: 5 + rand() * 5, color: FLOWERS[Math.floor(rand() * FLOWERS.length)] }));
    return { blades, flowers };
  }, []);
  const grass = (back) => blades.filter((b) => b.back === back).map((b) => `M${b.x - 4} 300 Q${b.x + b.lean / 2} ${300 - b.h / 2} ${b.x + b.lean} ${300 - b.h - (back ? 20 : 0)} Q${b.x + 2} ${300 - b.h / 2} ${b.x + 4} 300 Z`).join(" ");
  return (
    <svg className="absolute inset-x-0 bottom-0 w-full h-[26cqh]" viewBox="0 0 1600 300" preserveAspectRatio="xMidYMax slice">
      <path d="M0 300 L0 250 Q400 228 800 248 Q1200 266 1600 244 L1600 300 Z" fill="#071a0d" />
      <g className="theme-anim" style={{ transformOrigin: "50% 100%", animation: "theme-grass 7s ease-in-out infinite" }}><path d={grass(true)} fill="#0c2a15" /></g>
      {flowers.map((f, i) => (
        <g key={i} transform={`translate(${f.x} ${f.y})`}>
          {[0, 72, 144, 216, 288].map((a) => <ellipse key={a} cx="0" cy={-f.r} rx={f.r * 0.6} ry={f.r} fill={f.color} opacity="0.85" transform={`rotate(${a})`} />)}
          <circle r={f.r * 0.45} fill="#facc15" />
        </g>
      ))}
      <g className="theme-anim" style={{ transformOrigin: "50% 100%", animation: "theme-grass 5s ease-in-out -2s infinite" }}><path d={grass(false)} fill="#123d1f" /></g>
    </svg>
  );
}

// A scene's background: the theme's gradient and glows, and its ornament. Busy scenes (questions, the
// round overview, the leaderboard) pass quiet and leave the ornament out, so nothing competes with them.
export function SceneBackdrop({ quiet = false }) {
  const theme = useTheme();
  return (
    <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-b from-ground-top to-ground-bottom" />
      {theme.imageUrl && (
        // A busy scene gets the picture fainter, so text stays readable
        <div className="absolute inset-0 bg-cover bg-center"
          style={{ backgroundImage: `url("${theme.imageUrl}")`, opacity: theme.background.imageOpacity * (quiet ? 0.5 : 1), filter: theme.background.imageBlur ? `blur(${theme.background.imageBlur}px)` : undefined, transform: theme.background.imageBlur ? "scale(1.05)" : undefined }} />
      )}
      {theme.background.glows.map((g, i) => (
        <div key={i} className="absolute rounded-full -translate-x-1/2 -translate-y-1/2"
          style={{ left: `${g.x}%`, top: `${g.y}%`, width: `${g.size}cqw`, height: `${g.size}cqw`, opacity: g.opacity, background: `radial-gradient(circle, ${g.color} 0%, transparent 70%)` }} />
      ))}
      {!quiet && theme.background.ornament === "graveyard" && <><Moon /><Cobweb /><Fog /><Graveyard /></>}
      {!quiet && theme.background.ornament === "lights" && <Lights />}
      {!quiet && theme.background.ornament === "skyline" && <Skyline />}
      {!quiet && theme.background.ornament === "waves" && <Waves />}
      {!quiet && theme.background.ornament === "meadow" && <Meadow />}
    </div>
  );
}

function Snow({ density = 1 }) {
  const flakes = useMemo(() => {
    const rand = seeded(7);
    const storm = density > 1; // the celebration: bigger, brighter and faster
    return Array.from({ length: Math.round(70 * density) }, () => {
      const depth = storm ? 0.4 + rand() * 0.6 : rand();
      return {
        left: rand() * 100,
        size: (0.35 + depth * 0.9) * (storm ? 1.45 : 1),
        duration: (22 - depth * 12) * (storm ? 0.55 : 1),
        delay: -rand() * 22,
        sway: (rand() - 0.5) * 4,
        swayTime: 3 + rand() * 4,
        opacity: 0.35 + depth * 0.55,
        blur: depth < 0.3 ? 0.15 : 0,
      };
    });
  }, [density]);
  return flakes.map((f, i) => (
    <span key={i} className="theme-anim absolute top-0 will-change-transform"
      style={{ left: `${f.left}%`, "--drift": `${f.sway}cqw`, animation: `theme-fall ${f.duration}s linear ${f.delay}s infinite` }}>
      <span className="theme-anim block rounded-full bg-white"
        style={{ width: `${f.size}cqh`, height: `${f.size}cqh`, opacity: f.opacity, filter: f.blur ? `blur(${f.blur}cqh)` : undefined, "--sway": `${f.sway / 2}cqw`, animation: `theme-sway ${f.swayTime}s ease-in-out ${f.delay}s infinite` }} />
    </span>
  ));
}

const BAT = "M50 14 L52.5 8.5 L53.6 15 C56 14 57 16 58 18 C64 9 76 7 92 11 C83 15 81 21 83 27 C77 23 70 25 68 31 C64 27 58 27 56 31 C54 29 52 31 50 35 C48 31 46 29 44 31 C42 27 36 27 32 31 C30 25 23 23 17 27 C19 21 17 15 8 11 C24 7 36 9 42 18 C43 16 44 14 46.4 15 L47.5 8.5 Z";
function Bats({ density = 1 }) {
  const bats = useMemo(() => {
    const rand = seeded(13);
    return Array.from({ length: Math.round(7 * density) }, () => ({
      top: 6 + rand() * 40,
      scale: 0.55 + rand() * 0.7,
      duration: 16 + rand() * 14,
      delay: -rand() * 30,
      flap: 0.22 + rand() * 0.12,
    }));
  }, [density]);
  return bats.map((b, i) => (
    <span key={i} className="theme-anim absolute left-0 will-change-transform"
      style={{ top: `${b.top}cqh`, "--scale": b.scale, animation: `theme-fly ${b.duration}s ease-in-out ${b.delay}s infinite` }}>
      <svg viewBox="0 0 100 40" className="theme-anim block w-[6cqw]"
        style={{ animation: `theme-flap ${b.flap}s ease-in-out infinite`, filter: "drop-shadow(0 0 0.3cqh rgba(255, 140, 60, 0.8)) drop-shadow(0 0 1.2cqh rgba(160, 80, 255, 0.35))" }}>
        <path d={BAT} fill="#0c0512" />
      </svg>
    </span>
  ));
}

// Glowing embers rising from below and fading, a few of them ghostly green
function Embers({ density = 1 }) {
  const embers = useMemo(() => {
    const rand = seeded(29);
    return Array.from({ length: Math.round(16 * density) }, () => ({
      left: rand() * 100,
      size: 0.4 + rand() * 0.5,
      duration: 9 + rand() * 9,
      delay: -rand() * 18,
      drift: (rand() - 0.5) * 8,
      color: rand() < 0.2 ? "#7dffb8" : "#ffab45",
    }));
  }, [density]);
  return embers.map((e, i) => (
    <span key={i} className="theme-anim absolute bottom-0 rounded-full will-change-transform"
      style={{ left: `${e.left}%`, width: `${e.size}cqh`, height: `${e.size}cqh`, background: e.color, boxShadow: `0 0 1.2cqh ${e.color}`, "--drift": `${e.drift}cqw`, animation: `theme-rise ${e.duration}s ease-out ${e.delay}s infinite` }} />
  ));
}

// Things falling and tumbling: leaves, petals, confetti (shape(i) draws one; colors cycle)
function Falling({ count, seed, size, duration, colors, shape, scale = 1 }) {
  const items = useMemo(() => {
    const rand = seeded(seed);
    return Array.from({ length: count }, (_, i) => ({
      left: rand() * 100,
      size: (size[0] + rand() * (size[1] - size[0])) * scale,
      duration: duration[0] + rand() * (duration[1] - duration[0]),
      delay: -rand() * duration[1],
      drift: (rand() - 0.5) * 10,
      spin: 3 + rand() * 5,
      color: colors[i % colors.length],
    }));
  }, [count, seed, size, duration, colors, scale]);
  return items.map((it, i) => (
    <span key={i} className="theme-anim absolute top-0 will-change-transform"
      style={{ left: `${it.left}%`, "--drift": `${it.drift}cqw`, animation: `theme-fall ${it.duration}s linear ${it.delay}s infinite` }}>
      <span className="theme-anim block" style={{ width: `${it.size}cqh`, height: `${it.size}cqh`, animation: `theme-tumble ${it.spin}s linear ${it.delay}s infinite` }}>
        {shape(it.color, i)}
      </span>
    </span>
  ));
}

const LEAF = "M50 0 C70 20 95 35 90 60 C85 85 60 95 50 100 C40 95 15 85 10 60 C5 35 30 20 50 0 Z M50 10 V100";
const leaf = (color) => <svg viewBox="0 0 100 100" className="w-full h-full"><path d={LEAF} fill={color} stroke="rgba(0,0,0,0.25)" strokeWidth="3" /></svg>;
const petal = (color) => <span className="block w-full h-full" style={{ background: color, borderRadius: "80% 0 80% 0", opacity: 0.85 }} />;
const confetto = (color, i) => <span className="block w-full" style={{ height: i % 3 ? "45%" : "100%", background: color, borderRadius: i % 3 ? 1 : "50%" }} />;
const LEAF_COLORS = ["#d9541e", "#e8a33d", "#b5361c", "#c97b2b", "#8c5a2b"];
const PETAL_COLORS = ["#f9c6d8", "#fbe3ec", "#f5a9c4", "#ffffff"];
const CONFETTI_COLORS = ["var(--color-accent-400)", "var(--color-primary-400)", "#ffffff", "#ff5d8f", "#4ade80", "var(--color-accent-200)"];

// Things rising from below and fading: hearts and bubbles
function Rising({ count, seed, size, duration, render }) {
  const items = useMemo(() => {
    const rand = seeded(seed);
    return Array.from({ length: count }, () => ({
      left: rand() * 100,
      size: size[0] + rand() * (size[1] - size[0]),
      duration: duration[0] + rand() * (duration[1] - duration[0]),
      delay: -rand() * duration[1],
      sway: (rand() - 0.5) * 4,
      tint: rand(),
    }));
  }, [count, seed, size, duration]);
  return items.map((it, i) => (
    <span key={i} className="theme-anim absolute bottom-0 will-change-transform"
      style={{ left: `${it.left}%`, animation: `theme-float ${it.duration}s ease-in ${it.delay}s infinite` }}>
      <span className="theme-anim block" style={{ width: `${it.size}cqh`, height: `${it.size}cqh`, "--sway": `${it.sway}cqw`, animation: `theme-sway ${3 + it.tint * 3}s ease-in-out ${it.delay}s infinite` }}>
        {render(it)}
      </span>
    </span>
  ));
}

const HEART = "M50 88 C20 65 4 48 4 30 C4 16 15 6 28 6 C38 6 46 12 50 20 C54 12 62 6 72 6 C85 6 96 16 96 30 C96 48 80 65 50 88 Z";
const heart = (it) => <svg viewBox="0 0 100 100" className="w-full h-full"><path d={HEART} fill={it.tint < 0.5 ? "#ff4d6d" : "#ff8fab"} style={{ filter: "drop-shadow(0 0 6px rgba(255, 77, 109, 0.6))" }} /></svg>;
const bubble = () => (
  <span className="block w-full h-full rounded-full"
    style={{ border: "1px solid rgba(255, 255, 255, 0.35)", background: "radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.45), rgba(255, 255, 255, 0.05) 45%, transparent 70%)" }} />
);

// A starry sky over the top part of the screen, twinkling, with a shooting star now and then
function Stars({ density = 1 }) {
  const stars = useMemo(() => {
    const rand = seeded(67);
    return Array.from({ length: Math.round(80 * density) }, () => ({ left: rand() * 100, top: rand() * 70, size: 0.15 + rand() * 0.35, delay: -rand() * 4, time: 2 + rand() * 4 }));
  }, [density]);
  return (
    <>
      {stars.map((st, i) => (
        <span key={i} className="theme-anim absolute rounded-full bg-white"
          style={{ left: `${st.left}%`, top: `${st.top}%`, width: `${st.size}cqh`, height: `${st.size}cqh`, boxShadow: `0 0 ${st.size * 3}cqh rgba(255, 255, 255, 0.8)`, animation: `theme-twinkle ${st.time}s ease-in-out ${st.delay}s infinite` }} />
      ))}
      {[{ top: 8, delay: 0 }, { top: 22, delay: -7 }].map((sh, i) => (
        <span key={`s${i}`} className="theme-anim absolute left-0 h-[0.25cqh] w-[12cqw] rounded-full"
          style={{ top: `${sh.top}%`, background: "linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.9))", animation: `theme-shoot 14s ease-in ${sh.delay}s infinite` }} />
      ))}
    </>
  );
}

// The theme's decoration over every scene, nearly hidden (quiet) over questions and the leaderboard
// density: more of everything (the celebration's storm)
export function ThemeDecorations({ quiet = false, density = 1 }) {
  const theme = useTheme();
  const type = theme.decoration?.type;
  if (!type) return null;
  return (
    <div className="absolute inset-0 z-30 pointer-events-none overflow-hidden transition-opacity duration-1000" style={{ opacity: quiet ? 0.15 : 1 }}>
      {type === "snow" && <Snow density={density} />}
      {type === "bats" && <><Embers density={density} /><Bats density={density} /></>}
      {type === "leaves" && <Falling count={30 * density} seed={71} size={[2.4, 4.2]} duration={[14, 24]} colors={LEAF_COLORS} shape={leaf} scale={density > 1 ? 1.4 : 1} />}
      {type === "petals" && <Falling count={34 * density} seed={73} size={[0.8, 1.6]} duration={[12, 22]} colors={PETAL_COLORS} shape={petal} scale={density > 1 ? 1.4 : 1} />}
      {type === "confetti" && <Falling count={90 * density} seed={79} size={[1.1, 1.9]} duration={[8, 16]} colors={CONFETTI_COLORS} shape={confetto} scale={density > 1 ? 1.4 : 1} />}
      {type === "hearts" && <Rising count={16 * density} seed={83} size={[1.6, 3.4]} duration={[12, 22]} render={heart} />}
      {type === "bubbles" && <Rising count={22 * density} seed={89} size={[0.8, 3.2]} duration={[10, 20]} render={bubble} />}
      {type === "stars" && <Stars density={density} />}
    </div>
  );
}
