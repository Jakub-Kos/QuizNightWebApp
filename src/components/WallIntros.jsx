import { motion } from "framer-motion";
import { Users } from "lucide-react";
import { useTheme } from "../quiz/themes";
import { teamHex } from "../quiz/model";
import TeamAvatar from "./TeamAvatar";

// How the team wall introduces its featured team, in the theme's style (theme.scenes.welcome.intro):
// flip (a card flips forward), broadcast (a TV name strap slides in), spotlight (the team steps into a
// beam of light) or polaroid (a photo is tossed onto the screen). Rendered inside TeamWall's
// AnimatePresence, so every part has its own exit.

// Layers that reach far past the wall in every direction: the welcome screen clips them at the screen
// edge, so they never show the outline of the wall's box.
export const FULL_SCREEN = "absolute -inset-x-[45%] -inset-y-[70%] pointer-events-none";

export default function WallIntro({ team, t }) {
  const intro = useTheme().scenes?.welcome?.intro || "flip";
  const View = { broadcast: BroadcastIntro, spotlight: SpotlightIntro, polaroid: PolaroidIntro }[intro] || FlipIntro;
  return <View team={team} t={t} />;
}

// Name size for a box of the given width (cqw of the nearest size container)
const nameSize = (team, max, width) => `min(${max}, ${(width / Math.max(6, Math.min(Array.from(team.name).length, 18))).toFixed(1)}cqw)`;

function Dim({ amount = "bg-black/55" }) {
  return <motion.div className={`${FULL_SCREEN} ${amount} z-20`} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.5 }} />;
}

function Players({ team, t, className = "" }) {
  if (!(team.players > 0)) return null;
  return (
    <span className={`self-start px-6 py-3 bg-white/10 rounded-full flex items-center gap-3 text-4xl font-bold uppercase tracking-wider border border-white/10 ${className}`}>
      <Users size={44} /> {team.players} {t.players}
    </span>
  );
}

// The featured team, flipped forward over the darkened screen
function FlipIntro({ team, t }) {
  return (
    <>
      <Dim />
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
              <h2 className="font-black text-white leading-[1.05] break-words" style={{ fontSize: nameSize(team, "6rem", 110) }}>{team.name}</h2>
              {team.quote && <p className="text-4xl text-white/60 italic leading-snug line-clamp-3">"{team.quote}"</p>}
              <Players team={team} t={t} />
            </div>
          </div>
        </motion.div>
      </div>
    </>
  );
}

// A TV name strap: a colour stripe wipes across, the team's band follows with the avatar popping out
// of its left end, and it all slides off to the right
function BroadcastIntro({ team, t }) {
  const color = teamHex(team, 0);
  return (
    <>
      <Dim amount="bg-black/70" />
      <motion.div className="absolute left-[-6%] right-[4%] top-[30%] z-30 pointer-events-none" style={{ containerType: "inline-size" }}
        exit={{ x: "110%", opacity: 0 }} transition={{ duration: 0.5, ease: "easeIn" }}>
        <motion.div className="absolute inset-x-0 -top-[1.2cqw] h-[0.8cqw] origin-left bg-accent-500" initial={{ scaleX: 0 }} animate={{ scaleX: 1 }} transition={{ duration: 0.45, ease: "easeOut" }} />
        <motion.div className="relative flex items-center -skew-x-[8deg] bg-panel/95 border-l-[2cqw] shadow-[0_2cqw_6cqw_rgba(0,0,0,0.6)]" style={{ borderColor: color }}
          initial={{ clipPath: "inset(0 100% 0 0)" }} animate={{ clipPath: "inset(0 0% 0 0)" }} transition={{ delay: 0.2, duration: 0.6, ease: [0.2, 0.8, 0.2, 1] }}>
          <div className="w-[24cqw] shrink-0" />
          <div className="skew-x-[8deg] flex-1 min-w-0 py-[2.2cqw] pr-[8cqw]">
            <motion.h2 className="font-title font-black text-white uppercase leading-[1.02] break-words" style={{ fontSize: nameSize(team, "7cqw", 80) }}
              initial={{ x: -60, opacity: 0 }} animate={{ x: 0, opacity: 1 }} transition={{ delay: 0.55, duration: 0.5 }}>
              {team.name}
            </motion.h2>
          </div>
        </motion.div>
        {(team.quote || team.players > 0) && (
          <motion.div className="relative ml-[24cqw] mr-[16cqw] mt-[1cqw] -skew-x-[8deg] px-[2cqw] py-[1cqw] flex items-center gap-[2cqw]" style={{ backgroundColor: color }}
            initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.9, duration: 0.4 }}>
            <div className="skew-x-[8deg] flex items-center gap-[2cqw] min-w-0 text-white text-[2.2cqw] font-bold drop-shadow">
              {team.players > 0 && <span className="flex items-center gap-[0.6cqw] uppercase tracking-wider shrink-0"><Users className="w-[2.4cqw] h-[2.4cqw]" /> {team.players} {t.players}</span>}
              {team.quote && <span className="italic truncate opacity-90">"{team.quote}"</span>}
            </div>
          </motion.div>
        )}
        <motion.div className="absolute left-[7cqw] top-1/2 -translate-y-1/2" style={{ marginTop: team.quote || team.players > 0 ? "-2.6cqw" : 0 }}
          initial={{ scale: 0, rotate: -20 }} animate={{ scale: 1, rotate: 0 }} transition={{ delay: 0.45, type: "spring", bounce: 0.45 }}>
          <TeamAvatar team={team} className="w-[15cqw] h-[15cqw] rounded-[2cqw] border-[0.5cqw] border-white shadow-2xl" textClass="text-[6cqw]" />
        </motion.div>
      </motion.div>
    </>
  );
}

// The room goes dark, a beam falls from above and the team appears in it
function SpotlightIntro({ team, t }) {
  return (
    <>
      <Dim amount="bg-black/90" />
      <div className="absolute inset-0 z-30 pointer-events-none flex flex-col items-center justify-end pb-[3%]" style={{ containerType: "size" }}>
        <motion.div className="absolute -top-[30%] left-1/2 -translate-x-1/2 w-[70cqw] h-[125%] origin-top"
          style={{ background: "linear-gradient(to bottom, rgba(255,250,220,0.5), rgba(255,250,220,0.08) 85%, transparent)", clipPath: "polygon(42% 0, 58% 0, 100% 100%, 0 100%)" }}
          initial={{ opacity: 0, scaleX: 0.3 }} animate={{ opacity: 1, scaleX: 1 }} exit={{ opacity: 0, scaleX: 0.3 }} transition={{ duration: 0.6, ease: "easeOut" }} />
        <motion.div className="absolute bottom-[2%] w-[60cqw] h-[16cqh] rounded-[50%] bg-[radial-gradient(ellipse,rgba(255,250,220,0.35),transparent_70%)]"
          initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ delay: 0.2, duration: 0.6 }} />
        <motion.div className="relative flex flex-col items-center gap-[2.5cqh] max-w-[80cqw]"
          initial={{ opacity: 0, y: 40, filter: "brightness(0)" }} animate={{ opacity: 1, y: 0, filter: "brightness(1)" }} exit={{ opacity: 0, filter: "brightness(0)" }}
          transition={{ delay: 0.35, duration: 0.8 }}>
          <TeamAvatar team={team} className="w-[36cqh] h-[36cqh] rounded-full border-[0.8cqh] border-white/80 shadow-[0_0_8cqh_rgba(255,250,220,0.45)]" textClass="text-[14cqh]" />
          <motion.h2 className="font-title font-black text-white text-center uppercase leading-none break-words [text-shadow:0_0_4cqh_rgba(255,250,220,0.6)]"
            style={{ fontSize: nameSize(team, "15cqh", 120) }}
            initial={{ letterSpacing: "0.5em", opacity: 0 }} animate={{ letterSpacing: "0.02em", opacity: 1 }} transition={{ delay: 0.6, duration: 0.9, ease: "easeOut" }}>
            {team.name}
          </motion.h2>
          {team.quote && <p className="text-[4.5cqh] text-white/70 italic text-center leading-snug line-clamp-2">"{team.quote}"</p>}
          <Players team={team} t={t} className="self-center" />
        </motion.div>
      </div>
    </>
  );
}

// An instant photo tossed onto the screen, the team's name written on its strip
function PolaroidIntro({ team, t }) {
  const tilt = (Array.from(team.name).reduce((n, c) => n + c.charCodeAt(0), 0) % 7) - 3; // a steady tilt per team
  return (
    <>
      <Dim amount="bg-black/80" />
      <div className="absolute inset-0 z-30 pointer-events-none flex items-center justify-center gap-[4%]" style={{ containerType: "size" }}>
        <motion.div className="relative bg-[#f7f3ea] p-[2.2cqh] pb-0 shadow-[0_3cqh_8cqh_rgba(0,0,0,0.7)] w-[min(84cqh,40cqw)]"
          initial={{ y: "-130%", x: "-30%", rotate: -35 }} animate={{ y: 0, x: 0, rotate: tilt }} exit={{ y: "130%", x: "30%", rotate: 25, opacity: 0 }}
          transition={{ type: "spring", bounce: 0.3, duration: 0.9 }}>
          {/* A strip of tape holding it */}
          <div className="absolute -top-[2.6cqh] left-1/2 -translate-x-1/2 rotate-[-3deg] w-[38%] h-[5cqh] bg-white/45 border border-white/40 backdrop-blur-sm" />
          <motion.div initial={{ filter: "brightness(2.2) saturate(0)" }} animate={{ filter: "brightness(1) saturate(1)" }} transition={{ delay: 0.5, duration: 1.4 }}>
            <TeamAvatar team={team} className="w-full aspect-square rounded-none" textClass="text-[min(22cqh,14cqw)]" />
          </motion.div>
          <div className="h-[16cqh] flex items-center justify-center px-[1cqh]" style={{ containerType: "inline-size" }}>
            <h2 className="font-title font-black text-[#1d1d1f] text-center leading-[1.05] line-clamp-2 break-words" style={{ fontSize: `min(8cqh, ${(140 / Math.max(6, Math.min(Array.from(team.name).length, 20))).toFixed(1)}cqi)` }}>{team.name}</h2>
          </div>
        </motion.div>
        {(team.quote || team.players > 0) && (
          <motion.div className="max-w-[34cqw] flex flex-col gap-[3cqh]" initial={{ opacity: 0, x: 40 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: 40 }} transition={{ delay: 0.6, duration: 0.5 }}>
            {team.quote && <p className="text-[7cqh] text-white/90 italic leading-snug line-clamp-4">"{team.quote}"</p>}
            <Players team={team} t={t} />
          </motion.div>
        )}
      </div>
    </>
  );
}
