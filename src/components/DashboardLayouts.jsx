import { motion } from "framer-motion";
import { Play, Eye, CheckCircle, Circle, Check } from "lucide-react";

// Other layouts for the round overview (theme.scenes.dashboard.layout), next to Dashboard's columns.
// Like the columns, a round shows its title only once it is finished, and the first round's buttons
// carry the guided tour's data-tour markers. props: { rounds (visible ones), completed (ids),
// onToggle(event, id), onSelectRound(round, mode), t }

const titleOf = (round, index, done, t) => (done && round.title ? round.title : `${t.round} ${index + 1}`);
const nextIndex = (rounds, completed) => rounds.findIndex((r) => !completed.includes(r.id));

function Buttons({ round, index, onSelectRound, t, compact = false }) {
  const size = compact ? "py-[1.2cqh] text-[1.3cqh]" : "py-[1.8cqh] text-[1.5cqh]";
  return (
    <div className="grid grid-cols-2 gap-[1cqh] w-full">
      <button data-tour={index === 0 ? "questions" : undefined} onClick={() => onSelectRound(round, "questions_only")}
        className={`${size} rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 font-bold tracking-widest uppercase flex items-center justify-center gap-2 transition-colors`}>
        <Eye size={compact ? 16 : 20} className="text-gray-400" /> {t.btn_questions}
      </button>
      <button data-tour={index === 0 ? "answers" : undefined} onClick={() => onSelectRound(round, "with_answers")}
        className={`${size} rounded-xl bg-gradient-to-br from-accent-400 to-accent-600 hover:from-accent-300 hover:to-accent-500 text-black font-black tracking-widest uppercase flex items-center justify-center gap-2 shadow-[0_0_20px_rgb(var(--glow-accent)/0.3)] transition-colors`}>
        <Play size={compact ? 16 : 20} fill="currentColor" /> {t.btn_answers}
      </button>
    </div>
  );
}

function FinishToggle({ done, onClick, t }) {
  return (
    <button onClick={onClick} title={done ? t.mark_unfinished : t.mark_finished} aria-label={done ? t.mark_unfinished : t.mark_finished}
      className={`p-[0.8cqh] rounded-full border transition-colors ${done ? "border-green-500/40 text-green-400 bg-green-500/10" : "border-white/15 text-white/40 hover:text-white hover:border-white/40"}`}>
      {done ? <CheckCircle size={20} /> : <Circle size={20} />}
    </button>
  );
}

// A grid of round cards with their buttons always visible; the next round to play stands out
export function TilesView({ rounds, completed, onToggle, onSelectRound, t }) {
  const next = nextIndex(rounds, completed);
  const columns = rounds.length <= 4 ? rounds.length : rounds.length <= 6 ? 3 : 4;
  const rows = Math.ceil(rounds.length / columns);
  return (
    <div className="grid w-full max-w-[1700px] gap-[2cqh]"
      style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`, gridTemplateRows: `repeat(${rows}, minmax(0, 1fr))`, height: `${Math.min(70, rows * 34)}cqh` }}>
      {rounds.map((round, index) => {
        const done = completed.includes(round.id);
        const isNext = index === next;
        return (
          <motion.div key={round.id || index} data-tour={index === 0 ? "round" : undefined}
            initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.05 }}
            className={`relative min-h-0 rounded-[2rem] border overflow-hidden flex flex-col p-[2.4cqh] gap-[1.2cqh] ${
              done ? "border-green-500/30 bg-[#051005]" : isNext ? "border-accent-400/70 bg-panel shadow-[0_0_40px_rgb(var(--glow-accent)/0.25)]" : "border-white/10 bg-panel"}`}>
            <span className={`font-title absolute -right-[1cqh] -top-[3cqh] text-[18cqh] font-black leading-none pointer-events-none ${done ? "text-green-500/10" : "text-white/[0.06]"}`}>{index + 1}</span>
            <div className="flex items-center gap-2 relative">
              <span className="px-3 py-1 rounded-full bg-primary-500/10 border border-primary-500/20 text-primary-400 text-[1.2cqh] font-black tracking-widest uppercase">{round.questions.length} {t.questions_count}</span>
              {isNext && <span className="px-3 py-1 rounded-full bg-accent-500 text-black text-[1.2cqh] font-black tracking-widest uppercase">{t.db_next}</span>}
              <span className="ml-auto"><FinishToggle done={done} onClick={(e) => onToggle(e, round.id)} t={t} /></span>
            </div>
            <h2 className="font-title relative flex-1 min-h-0 flex items-center text-[3.6cqh] font-black uppercase leading-tight text-white line-clamp-3">{titleOf(round, index, done, t)}</h2>
            <div className="relative"><Buttons round={round} index={index} onSelectRound={onSelectRound} t={t} compact={rows > 1} /></div>
          </motion.div>
        );
      })}
    </div>
  );
}

// The evening as a path: a node per round along a line, finished ones checked, the next one large
// with its buttons; the others have small buttons under their titles
export function TimelineView({ rounds, completed, onToggle, onSelectRound, t }) {
  const next = nextIndex(rounds, completed);
  const doneShare = rounds.length > 1 ? Math.max(0, (next === -1 ? rounds.length - 1 : next) / (rounds.length - 1)) : 0;
  return (
    <div className="relative w-full max-w-[1800px] h-[64cqh] flex items-start">
      <div className="absolute left-[5%] right-[5%] top-[14cqh] h-[0.8cqh] rounded-full bg-white/10" />
      <motion.div className="absolute left-[5%] top-[14cqh] h-[0.8cqh] rounded-full bg-gradient-to-r from-green-500 to-accent-400"
        initial={{ width: 0 }} animate={{ width: `${doneShare * 90}%` }} transition={{ duration: 1, type: "spring" }} />
      <div className="relative w-full grid" style={{ gridTemplateColumns: `repeat(${rounds.length}, minmax(0, 1fr))` }}>
        {rounds.map((round, index) => {
          const done = completed.includes(round.id);
          const isNext = index === next;
          return (
            <div key={round.id || index} data-tour={index === 0 ? "round" : undefined} className="flex flex-col items-center text-center px-[0.8cqw] gap-[1.4cqh]">
              <div className="h-[28cqh] flex items-center justify-center">
                <motion.div animate={isNext ? { scale: [1, 1.06, 1] } : { scale: 1 }} transition={isNext ? { repeat: Infinity, duration: 2.4 } : {}}
                  className={`rounded-full flex items-center justify-center font-title font-black border-4 ${
                    isNext ? "w-[16cqh] h-[16cqh] text-[7cqh] bg-accent-500 text-black border-accent-200 shadow-[0_0_60px_rgb(var(--glow-accent)/0.6)]"
                      : done ? "w-[9cqh] h-[9cqh] text-[4cqh] bg-green-600 text-white border-green-300/50"
                      : "w-[9cqh] h-[9cqh] text-[4cqh] bg-panel text-white/70 border-white/15"}`}>
                  {done ? <Check size={40} strokeWidth={3} /> : index + 1}
                </motion.div>
              </div>
              <h2 className={`font-title font-black uppercase leading-tight line-clamp-2 ${isNext ? "text-[3cqh] text-white" : "text-[2cqh] text-white/70"}`}>{titleOf(round, index, done, t)}</h2>
              <span className="text-[1.3cqh] font-bold tracking-widest uppercase text-primary-400">{round.questions.length} {t.questions_count}</span>
              <div className={isNext ? "w-[125%] max-w-[20cqw]" : "w-full"}><Buttons round={round} index={index} onSelectRound={onSelectRound} t={t} compact={!isNext} /></div>
              <FinishToggle done={done} onClick={(e) => onToggle(e, round.id)} t={t} />
            </div>
          );
        })}
      </div>
    </div>
  );
}
