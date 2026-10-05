import { motion } from "framer-motion";
import { ArrowRight, ArrowDownUp } from "lucide-react";

// Sort question on the TV. layout comes from sortLayout (quiz/sort.js), its items carry `source` (resolved
// picture URL or null). Before the reveal the items are shown mixed up with letters; on the reveal they
// appear one by one in the correct order, numbered, together with the answer as a row of letters.
export default function SortItems({ layout, revealed, t }) {
  const items = revealed ? layout.correct : layout.shown;
  const withPictures = items.some((it) => it.source);

  return (
    <div className="w-full flex flex-col items-center gap-[2.5cqh]">
      {withPictures ? <PictureGrid items={items} revealed={revealed} /> : <TextList items={items} revealed={revealed} />}

      {revealed ? (
        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.25 * items.length + 0.2 }}
          className="flex items-center gap-5 px-10 py-4 rounded-full bg-gradient-to-br from-green-500 to-green-600 text-black shadow-[0_0_60px_rgba(34,197,94,0.5)]">
          <span className="text-2xl font-bold uppercase tracking-widest">{t.sort_correct_order}</span>
          <span className="flex items-center gap-3 text-5xl font-black">
            {layout.correct.map((it, i) => (
              <span key={it.letter} className="flex items-center gap-3">
                {i > 0 && <ArrowRight size={28} strokeWidth={3} className="opacity-60" />}{it.letter}
              </span>
            ))}
          </span>
        </motion.div>
      ) : (
        <div className="flex items-center gap-5 px-12 py-4 rounded-full bg-white/5 border border-white/10 text-white/50 animate-pulse">
          <ArrowDownUp size={32} />
          <span className="font-black text-3xl tracking-[0.15em] uppercase">{t.sort_hint}</span>
        </div>
      )}
    </div>
  );
}

// Fade in one after another on the reveal (no layout animation: the stage is a scaled transform)
const appear = (revealed, i) => revealed
  ? { initial: { opacity: 0, x: -30 }, animate: { opacity: 1, x: 0 }, transition: { delay: 0.25 * i } }
  : { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, transition: { delay: 0.06 * i } };

function Badge({ item, revealed, className }) {
  return (
    <div className={`flex items-center gap-2 shrink-0 ${className}`}>
      {revealed && <span className="w-14 h-14 rounded-xl bg-green-500 text-black flex items-center justify-center font-black text-3xl">{item.place}.</span>}
      <span className={`w-14 h-14 rounded-full flex items-center justify-center font-black text-2xl border ${revealed ? "bg-black/60 border-green-400/60 text-green-300" : "bg-black/40 border-white/15 text-white"}`}>{item.letter}</span>
    </div>
  );
}

function TextList({ items, revealed }) {
  const twoColumns = items.length > 5;
  // Two columns are filled top to bottom, so the order reads down the left column first
  const rows = twoColumns ? Math.ceil(items.length / 2) : items.length;
  return (
    <div className={`grid gap-x-8 ${twoColumns ? "grid-cols-2 w-full max-w-[1800px] gap-y-3" : "w-full max-w-5xl gap-y-4"}`}
      style={{ gridTemplateRows: `repeat(${rows}, auto)`, gridAutoFlow: "column" }}>
      {items.map((item, i) => (
        <motion.div key={`${revealed}-${item.letter}`} {...appear(revealed, i)}
          className={`flex items-center gap-6 rounded-2xl border ${twoColumns ? "px-6 py-3" : "px-8 py-4"} ${revealed
            ? "bg-gradient-to-r from-green-900/50 to-green-800/20 border-green-500/60"
            : "bg-white/5 border-white/10"}`}>
          <Badge item={item} revealed={revealed} />
          <span className={`font-bold text-white text-left leading-tight ${twoColumns ? "text-4xl" : "text-5xl"}`}>{item.text}</span>
        </motion.div>
      ))}
    </div>
  );
}

function PictureGrid({ items, revealed }) {
  const n = items.length;
  const cols = n <= 4 ? n : n <= 6 ? 3 : Math.ceil(n / 2);
  const rows = Math.ceil(n / cols);
  const height = rows === 1 ? 44 : 23; // cqh, leaves room for the question and the answer line
  return (
    <div className="w-full flex flex-wrap justify-center gap-5 max-w-[2100px]">
      {items.map((item, i) => (
        <motion.div key={`${revealed}-${item.letter}`} {...appear(revealed, i)}
          style={{ width: `calc((100% - ${cols - 1} * 1.25rem) / ${cols})`, height: `${height}cqh` }}
          className={`relative rounded-3xl overflow-hidden border flex flex-col bg-black/50 ${revealed ? "border-green-500/70 shadow-[0_0_30px_rgba(34,197,94,0.25)]" : "border-white/10"}`}>
          <div className="flex-1 min-h-0 flex items-center justify-center bg-black/40">
            {item.source
              ? <img src={item.source} alt="" className="w-full h-full object-contain" />
              : <span className="text-8xl font-black text-white/10">{item.letter}</span>}
          </div>
          {item.text && (
            <div className={`shrink-0 px-4 py-3 text-center font-bold text-3xl leading-tight line-clamp-2 ${revealed ? "bg-green-900/60 text-white" : "bg-white/5 text-white"}`}>{item.text}</div>
          )}
          <Badge item={item} revealed={revealed} className="absolute top-3 left-3 drop-shadow-xl" />
        </motion.div>
      ))}
    </div>
  );
}
