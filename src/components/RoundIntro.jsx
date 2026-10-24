import { motion } from "framer-motion";
import { useTheme } from "../quiz/themes";
import { SceneBackdrop } from "./Theme";

// A round's title card (QuestionScreen at qIndex -1), in the theme's layout (theme.scenes.round.layout):
// classic, curtain (stage curtains open on the title), slam (the title crashes in and shakes the
// screen) or marquee (a cinema sign with chasing bulbs). label is "Round 3", title the round's name.
export default function RoundIntro({ label, title, onClick }) {
  const layout = useTheme().scenes?.round?.layout || "classic";
  const View = { curtain: CurtainIntro, slam: SlamIntro, marquee: MarqueeIntro }[layout] || ClassicIntro;
  return (
    <div onClick={onClick} className="h-full w-full bg-ground flex flex-col items-center justify-center font-show cursor-pointer relative overflow-hidden">
      <SceneBackdrop />
      <View label={label} title={title} />
    </div>
  );
}

// Title size that keeps long names on a few lines
const titleSize = (title, max) => `${Math.min(max, Math.max(5, 150 / Math.max(8, Array.from(title || "").length) * 3.2)).toFixed(1)}cqh`;

function ClassicIntro({ label, title }) {
  return (
    <motion.div initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }} className="text-center z-10 flex flex-col items-center max-w-6xl px-8">
      <h2 className="text-primary-500 font-bold tracking-[0.5em] text-4xl mb-6 uppercase drop-shadow-lg">{label}</h2>
      <div className="h-2 w-40 bg-primary-500 mx-auto mb-12 rounded-full shadow-[0_0_20px_rgb(var(--glow-primary)/0.8)]" />
      <h1 className="font-title text-[120px] font-black text-white uppercase tracking-tight drop-shadow-2xl leading-none text-center">{title}</h1>
    </motion.div>
  );
}

// Folds of heavy cloth in the theme's main colour
const CLOTH = {
  backgroundColor: "var(--color-primary-800)",
  backgroundImage: "repeating-linear-gradient(90deg, rgba(0,0,0,0.45) 0, rgba(255,255,255,0.08) 3.5%, rgba(0,0,0,0.45) 7%)",
};

function CurtainIntro({ label, title }) {
  const open = { duration: 1.6, ease: [0.65, 0, 0.35, 1], delay: 0.35 };
  return (
    <>
      {/* The stage light the curtains open on */}
      <motion.div className="absolute inset-0 pointer-events-none" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.8, duration: 1.2 }}
        style={{ background: "radial-gradient(ellipse 45% 60% at 50% 45%, rgb(var(--glow-accent) / 0.22), transparent 70%)" }} />
      <motion.div initial={{ opacity: 0, y: 30, scale: 0.92 }} animate={{ opacity: 1, y: 0, scale: 1 }} transition={{ delay: 1.1, duration: 0.9, type: "spring", bounce: 0.3 }}
        className="relative z-10 text-center flex flex-col items-center max-w-[70cqw] px-8">
        <h2 className="text-accent-400 font-bold tracking-[0.5em] text-[3cqh] mb-[3cqh] uppercase">{label}</h2>
        <h1 className="font-title font-black text-white uppercase leading-none drop-shadow-2xl" style={{ fontSize: titleSize(title, 10) }}>{title}</h1>
        {/* The stage floor */}
        <div className="mt-[6cqh] h-[1.2cqh] w-[60cqw] rounded-full bg-[radial-gradient(ellipse,rgb(var(--glow-accent)/0.5),transparent_70%)]" />
      </motion.div>
      {/* The two halves, gathered to the sides as they open */}
      {[-1, 1].map((side) => (
        <motion.div key={side} className="absolute top-0 bottom-0 w-[51%] z-20 shadow-[0_0_80px_rgba(0,0,0,0.8)]"
          style={{ ...CLOTH, [side < 0 ? "left" : "right"]: 0, transformOrigin: side < 0 ? "left" : "right" }}
          initial={{ scaleX: 1 }} animate={{ scaleX: 0.16 }} transition={open} />
      ))}
      {/* The valance across the top, with a scalloped edge */}
      <div className="absolute top-0 inset-x-0 z-30">
        <div className="h-[6cqh] border-b-[0.6cqh] border-accent-500/70" style={{ ...CLOTH, backgroundColor: "var(--color-primary-900)" }} />
        <div className="h-[3cqh]" style={{ backgroundImage: "radial-gradient(circle at 50% 0, var(--color-primary-900) 3cqh, transparent calc(3cqh + 1px))", backgroundSize: "6cqh 3cqh" }} />
      </div>
    </>
  );
}

function SlamIntro({ label, title }) {
  const hit = 0.45; // when the title lands
  return (
    <motion.div className="absolute inset-0 flex items-center justify-center"
      animate={{ x: [0, -24, 20, -12, 8, 0], y: [0, 14, -10, 6, -3, 0] }} transition={{ delay: hit, duration: 0.5 }}>
      {/* The round number, huge and hollow behind everything */}
      <motion.span initial={{ opacity: 0, scale: 1.4 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.6 }}
        className="absolute font-title font-black leading-none select-none text-transparent text-[80cqh]" style={{ WebkitTextStroke: "0.4cqh rgb(var(--glow-primary) / 0.25)" }}>
        {label.replace(/\D+/g, "") || "?"}
      </motion.span>
      {/* Shock rings and streaks from where it lands */}
      {[0, 1, 2].map((i) => (
        <motion.div key={i} className="absolute rounded-full border-accent-400" style={{ borderWidth: "0.8cqh", width: "30cqh", height: "30cqh" }}
          initial={{ scale: 0.3, opacity: 0 }} animate={{ scale: [0.3, 5 + i * 1.5], opacity: [0.9, 0] }} transition={{ delay: hit + i * 0.12, duration: 1.1, ease: "easeOut" }} />
      ))}
      {Array.from({ length: 16 }, (_, i) => (
        <motion.div key={`s${i}`} className="absolute h-[0.6cqh] w-[14cqw] origin-left bg-gradient-to-r from-white to-transparent" style={{ rotate: `${i * 22.5}deg`, left: "50%", top: "50%" }}
          initial={{ scaleX: 0, opacity: 0, x: "4cqw" }} animate={{ scaleX: [0, 1, 1], opacity: [0, 1, 0], x: ["4cqw", "8cqw", "22cqw"] }} transition={{ delay: hit, duration: 0.7, ease: "easeOut" }} />
      ))}
      <motion.div className="absolute inset-0 bg-white pointer-events-none" initial={{ opacity: 0 }} animate={{ opacity: [0, 0.5, 0] }} transition={{ delay: hit, duration: 0.35 }} />
      <div className="relative z-10 text-center flex flex-col items-center max-w-[80cqw] px-8">
        <motion.h2 initial={{ opacity: 0, y: -40 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: hit + 0.25, type: "spring", bounce: 0.5 }}
          className="mb-[3cqh] px-[2cqh] py-[0.8cqh] -skew-x-12 bg-accent-500 text-black font-black tracking-[0.3em] text-[3.4cqh] uppercase shadow-[0_0_30px_rgb(var(--glow-accent)/0.5)]">
          {label}
        </motion.h2>
        <motion.h1 initial={{ scale: 3.5, opacity: 0, rotate: -6 }} animate={{ scale: 1, opacity: 1, rotate: -2 }} transition={{ duration: hit, ease: [0.5, 0, 0.9, 0.4] }}
          className="font-title font-black text-white uppercase leading-[0.95] drop-shadow-[0_1cqh_0_rgb(var(--glow-primary-deep)/0.9)]" style={{ fontSize: titleSize(title, 13) }}>
          {title}
        </motion.h1>
      </div>
    </motion.div>
  );
}

// Bulbs around the sign: positions along its edge, 0–100 % of the sign's box
function bulbRing(across, down) {
  const spots = [];
  for (let i = 0; i < across; i++) spots.push([((i + 0.5) / across) * 100, 0], [((i + 0.5) / across) * 100, 100]);
  for (let i = 0; i < down; i++) spots.push([0, ((i + 0.5) / down) * 100], [100, ((i + 0.5) / down) * 100]);
  return spots;
}
const BULBS = bulbRing(22, 8);

function MarqueeIntro({ label, title }) {
  return (
    <motion.div initial={{ opacity: 0, y: -60 }} animate={{ opacity: 1, y: 0 }} transition={{ type: "spring", bounce: 0.35, duration: 1 }}
      className="relative z-10 w-[72cqw] flex flex-col items-center">
      {/* The pointed top with the round number */}
      <div className="relative -mb-[1cqh] px-[5cqw] pt-[2.4cqh] pb-[3cqh] bg-accent-500 text-black [clip-path:polygon(8%_100%,0_0,100%_0,92%_100%)]">
        <span className="font-title font-black uppercase tracking-[0.35em] text-[4cqh]">{label}</span>
      </div>
      <div className="relative w-full rounded-[3cqh] bg-panel border-[1.4cqh] border-accent-600 shadow-[0_0_80px_rgb(var(--glow-accent)/0.35)] px-[6cqw] py-[9cqh]">
        {BULBS.map(([x, y], i) => (
          <motion.span key={i} className="absolute w-[1.8cqh] h-[1.8cqh] -ml-[0.9cqh] -mt-[0.9cqh] rounded-full bg-[#fff6d5]"
            style={{ left: `${x}%`, top: `${y}%`, boxShadow: "0 0 1.4cqh 0.4cqh rgb(var(--glow-accent) / 0.9)" }}
            animate={{ opacity: [1, 0.25, 1] }} transition={{ duration: 0.9, repeat: Infinity, delay: (i % 3) * 0.3 }} />
        ))}
        <motion.h1 initial={{ opacity: 0, letterSpacing: "0.4em" }} animate={{ opacity: [0, 1, 0.4, 1], letterSpacing: "0.02em" }} transition={{ delay: 0.5, duration: 1.2 }}
          className="font-title font-black text-white uppercase leading-none text-center [text-shadow:0_0_2cqh_rgb(var(--glow-primary)/0.9),0_0_6cqh_rgb(var(--glow-primary)/0.6)]"
          style={{ fontSize: titleSize(title, 10) }}>
          {title}
        </motion.h1>
      </div>
      {/* The posts it stands on */}
      <div className="flex justify-between w-[50%]">
        {[0, 1].map((i) => <div key={i} className="w-[1.6cqw] h-[10cqh] bg-gradient-to-b from-accent-700 to-transparent" />)}
      </div>
    </motion.div>
  );
}
