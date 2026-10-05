import { useState, useEffect, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { User, Users } from "lucide-react";
import { fmt } from "../quiz/files";
import { teamHex } from "../quiz/model";
import TeamAvatar from "./TeamAvatar";

// Welcome screen for teams known only by name and number of players: every team at once in a grid
// sized to the screen. The tiles float over a drifting colour aurora with a passing shimmer, and every
// few seconds one team flips forward into a big intro card (shown for introMs, then a pause of pauseMs).
export default function TeamWall({ teams, introMs = 4700, pauseMs = 2600, t }) {
  const count = teams.length;
  // Balanced grid for the wide area: about 1.8 columns per row (6 -> 3x2, 15 -> 5x3, 25 -> 7x4);
  // an incomplete last row is centred
  const rows = Math.max(1, Math.round(Math.sqrt(count / 1.8)));
  const cols = Math.ceil(count / rows);
  const size = rows <= 2 ? { maxName: 3, avatar: "w-36 h-36", initials: "text-6xl", icon: 32, gap: "gap-4" }
    : rows === 3 ? { maxName: 2.25, avatar: "w-20 h-20", initials: "text-3xl", icon: 22, gap: "gap-2" }
    : rows === 4 ? { maxName: 1.875, avatar: "w-14 h-14", initials: "text-2xl", icon: 18, gap: "gap-1.5" }
    : { maxName: 1.5, avatar: "w-12 h-12", initials: "text-xl", icon: 16, gap: "gap-1" };
  const players = teams.reduce((n, team) => n + (Number(team.players) || 0), 0);
  const entranceDelay = (i) => 0.15 + i * 0.07;
  const tileStyle = { width: `calc((100% - ${cols - 1} * 1.5rem) / ${cols})`, height: `calc((100% - ${rows - 1} * 1.5rem) / ${rows})`, containerType: "size" };

  // Name size that fits the tile (a size container): limited by the tile width for the longest line and
  // by the tile height for one or two lines
  const nameFont = (name) => {
    const text = name.trim();
    const length = Math.max(4, Array.from(text).length);
    const longestWord = Math.max(4, ...text.split(/\s+/).map((w) => Array.from(w).length));
    const lines = length > 12 && /\s/.test(text) ? 2 : 1;
    const perLine = lines === 2 ? Math.max(longestWord, Math.ceil(length / 2) + 2) : length;
    return `min(${size.maxName}rem, ${(150 / perLine).toFixed(1)}cqi, ${(28 / lines).toFixed(1)}cqh)`;
  };

  const [featured, onIntroGone] = useTakeover(count, introMs, pauseMs);

  return (
    <div className="w-full h-full flex flex-col px-24 pb-2">
      <p className="relative text-center text-3xl font-bold uppercase tracking-[0.3em] text-white/40 mb-8">
        {players ? fmt(t.wall_summary, { teams: count, players }) : fmt(t.wall_summary_teams, { teams: count })}
      </p>

      <div className="relative flex-1 min-h-0">
        <Aurora teams={teams} />

        <div className="absolute inset-0 flex flex-wrap justify-center content-center gap-6">
          {teams.map((team, i) => (
            <motion.div
              key={team.id}
              style={tileStyle}
              initial={{ opacity: 0, y: 40, scale: 0.9 }}
              animate={{ opacity: featured === i ? 0.15 : 1, y: 0, scale: featured === i ? 0.92 : 1 }}
              transition={{ opacity: { delay: featured === null ? entranceDelay(i) : 0, duration: 0.5 }, y: { delay: entranceDelay(i), type: "spring", bounce: 0.3 }, scale: { type: "spring", bounce: 0.3 } }}
            >
              {/* Each tile floats on its own slow rhythm */}
              <motion.div className="w-full h-full" animate={{ y: [0, -7, 0, 5, 0] }}
                transition={{ duration: 6 + (i % 5) * 0.8, repeat: Infinity, ease: "easeInOut", delay: i * 0.37 }}>
                <Tile team={team} size={size} font={nameFont(team.name)} />
              </motion.div>
            </motion.div>
          ))}
        </div>

        <Shimmer />
        <AnimatePresence onExitComplete={onIntroGone}>
          {featured !== null && teams[featured] && <IntroCard key={teams[featured].id} team={teams[featured]} t={t} />}
        </AnimatePresence>
      </div>
    </div>
  );
}

// Which tile is shown as the big intro card (null during the pause); cycles through every team.
// The pause starts once the card has finished flipping away (onIntroGone), so pauseMs is the time the
// wall is seen without a card.
function useTakeover(count, introMs, pauseMs) {
  const [featured, setFeatured] = useState(null);
  const timer = useRef(null);
  const next = useRef(0);

  const show = useCallback(() => {
    setFeatured(next.current % count);
    next.current++;
    timer.current = setTimeout(() => setFeatured(null), introMs);
  }, [count, introMs]);

  const onIntroGone = useCallback(() => {
    clearTimeout(timer.current);
    timer.current = setTimeout(show, pauseMs);
  }, [show, pauseMs]);

  useEffect(() => {
    if (count === 0) return;
    timer.current = setTimeout(show, 1500 + count * 70); // after the tiles have appeared
    return () => clearTimeout(timer.current);
  }, [count, show]);

  return [featured, onIntroGone];
}

function PlayerIcons({ players, size }) {
  if (!(players > 0)) return null;
  return (
    <div className="relative flex items-center gap-1 text-white/60">
      {players <= 8
        ? Array.from({ length: players }, (_, k) => <User key={k} size={size} strokeWidth={2.5} />)
        : <><Users size={size} strokeWidth={2.5} /><span className="font-bold" style={{ fontSize: size }}>{players}</span></>}
    </div>
  );
}

function Tile({ team, size, font }) {
  return (
    <div className={`relative w-full h-full rounded-3xl border border-white/10 overflow-hidden flex flex-col items-center justify-center ${size.gap} p-4`}>
      <div className={`absolute inset-0 bg-gradient-to-br ${team.color} opacity-[0.18]`} />
      <div className="absolute inset-0 bg-black/40" />
      <TeamAvatar team={team} className={`${size.avatar} rounded-2xl border border-white/20 relative shadow-xl`} textClass={size.initials} />
      <h3 className="relative font-black text-white text-center leading-tight line-clamp-2 break-words max-w-full px-2" style={{ fontSize: font }}>{team.name}</h3>
      <PlayerIcons players={team.players} size={size.icon} />
    </div>
  );
}

// Layers below reach far past the wall in every direction: the welcome screen clips them at the
// screen edge, so they never show the outline of the wall's box.
const FULL_SCREEN = "absolute -inset-x-[45%] -inset-y-[70%] pointer-events-none";

const AURORA_PATHS = [
  { x: ["10%", "40%", "25%", "10%"], y: ["15%", "35%", "55%", "15%"] },
  { x: ["55%", "30%", "60%", "55%"], y: ["45%", "20%", "30%", "45%"] },
  { x: ["35%", "60%", "20%", "35%"], y: ["55%", "40%", "25%", "55%"] },
  { x: ["65%", "40%", "50%", "65%"], y: ["20%", "50%", "40%", "20%"] },
];

// Slowly drifting blobs in the first teams' colours behind the tiles
function Aurora({ teams }) {
  return (
    <div className={FULL_SCREEN}>
      {teams.slice(0, 4).map((team, i) => (
        <motion.div key={i} className="absolute left-0 top-0 w-[28%] h-[32%] rounded-full blur-[140px] opacity-25"
          style={{ backgroundColor: teamHex(team, i) }}
          animate={AURORA_PATHS[i]} transition={{ duration: 22 + i * 4, repeat: Infinity, ease: "easeInOut" }} />
      ))}
    </div>
  );
}

// A diagonal band of light sweeping across the screen every few seconds. The layer is much wider than
// the screen (FULL_SCREEN), so each sweep starts and ends well outside it and waits there: the restart
// is never visible.
function Shimmer() {
  return (
    <div className={`${FULL_SCREEN} overflow-hidden z-10`}>
      <motion.div className="absolute inset-y-0 w-[10%] -skew-x-[20deg] bg-gradient-to-r from-transparent via-white/[0.07] to-transparent"
        initial={{ left: "-15%" }} animate={{ left: "105%" }}
        transition={{ duration: 5.6, repeat: Infinity, repeatDelay: 5, ease: "linear", delay: 2 }} />
    </div>
  );
}

// The featured team, flipped forward over the darkened screen
function IntroCard({ team, t }) {
  const nameLength = Math.max(6, Math.min(Array.from(team.name).length, 18));
  return (
    <>
      <motion.div className={`${FULL_SCREEN} bg-black/55 z-20`} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.5 }} />
      <div className="absolute inset-0 flex items-center justify-center z-30 pointer-events-none" style={{ perspective: "2000px" }}>
        <motion.div
          initial={{ rotateY: -95, scale: 0.55, opacity: 0 }}
          animate={{ rotateY: 0, scale: 1, opacity: 1 }}
          exit={{ rotateY: 95, scale: 0.55, opacity: 0 }}
          transition={{ type: "spring", bounce: 0.25, duration: 0.9 }}
          className={`w-[58%] h-[86%] rounded-[3rem] bg-gradient-to-br ${team.color} p-3 shadow-[0_0_120px_rgba(0,0,0,0.7)]`}
        >
          <div className="w-full h-full rounded-[2.5rem] bg-black/80 backdrop-blur-md flex items-center gap-12 px-14 relative overflow-hidden" style={{ containerType: "size" }}>
            <TeamAvatar team={team} className="w-[min(34cqw,80cqh)] h-[min(34cqw,80cqh)] rounded-[2rem] border border-white/20 shadow-2xl" textClass="text-[min(12cqw,28cqh)]" />
            <div className="flex-1 min-w-0 flex flex-col gap-6">
              <h2 className="font-black text-white leading-[1.05] break-words" style={{ fontSize: `min(6rem, ${(110 / nameLength).toFixed(1)}cqw)` }}>{team.name}</h2>
              {team.quote && <p className="text-4xl text-white/60 italic leading-snug line-clamp-3">"{team.quote}"</p>}
              {team.players > 0 && (
                <span className="self-start px-6 py-3 bg-white/10 rounded-full flex items-center gap-3 text-4xl font-bold uppercase tracking-wider border border-white/10">
                  <Users size={44} /> {team.players} {t.players}
                </span>
              )}
            </div>
          </div>
        </motion.div>
      </div>
    </>
  );
}
