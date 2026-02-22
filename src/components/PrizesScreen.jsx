import { motion } from "framer-motion";
import { Trophy, Medal, Crown } from "lucide-react";

export default function PrizesScreen({ onNext, t }) {
  const prizes = [
    {
        rank: 2,
        label: t.prize_silver,
        icon: <Medal size={64} />,
        color: "from-slate-400 to-slate-600",
        glow: "shadow-slate-400/30",
        border: "border-slate-400/30",
        height: "h-[500px]",
        delay: 0.2
    },
    {
        rank: 1,
        label: t.prize_gold,
        icon: <Trophy size={80} />,
        color: "from-yellow-300 to-yellow-600",
        glow: "shadow-yellow-500/50",
        border: "border-yellow-400/50",
        height: "h-[580px]",
        delay: 0
    },
    {
        rank: 3,
        label: t.prize_bronze,
        icon: <Medal size={56} />,
        color: "from-orange-400 to-orange-700",
        glow: "shadow-orange-500/30",
        border: "border-orange-500/30",
        height: "h-[450px]",
        delay: 0.4
    },
  ];

  return (
    <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col items-center justify-center font-['League_Spartan']">

      {/* --- BACKGROUND AMBIENCE --- */}
      <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0" />
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-[500px] bg-yellow-600/10 blur-[130px] rounded-full pointer-events-none" />

      {/* --- TITLE --- */}
      <motion.div
        initial={{ opacity: 0, y: -50 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative z-10 text-center mb-12"
      >
          <h1 className="text-8xl font-black text-white uppercase tracking-[0.1em] drop-shadow-2xl mb-2">{t.prizes_title}</h1>
      </motion.div>

      {/* --- CARDS CONTAINER --- */}
      <div className="flex items-end justify-center gap-8 relative z-10 px-10 pb-8">
          {prizes.map((p) => (
              <motion.div
                key={p.rank}
                initial={{ opacity: 0, y: 200, scale: 0.8 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                transition={{
                    delay: p.delay,
                    duration: 0.8,
                    type: "spring",
                    bounce: 0.3
                }}
                className={`relative w-[320px] ${p.height} rounded-[3rem] p-[2px] bg-gradient-to-b ${p.color} ${p.glow} shadow-2xl flex-shrink-0 group hover:-translate-y-4 transition-transform duration-500`}
              >
                  {/* INNER GLASS CARD */}
                  <div className="w-full h-full bg-[#0a0a0a] rounded-[2.9rem] flex flex-col items-center p-8 relative overflow-hidden">

                      {/* Inner Shine/Gradient */}
                      <div className={`absolute inset-0 bg-gradient-to-b ${p.color} opacity-10 group-hover:opacity-20 transition-opacity duration-500`} />

                      {/* Rank Circle */}
                      <div className={`w-20 h-20 rounded-full bg-gradient-to-br ${p.color} flex items-center justify-center text-black font-black text-4xl mb-10 shadow-lg z-10 ring-4 ring-black/50`}>
                          #{p.rank}
                      </div>

                      {/* Icon with 3D float effect */}
                      <div className={`text-white mb-8 drop-shadow-[0_0_25px_rgba(255,255,255,0.4)] transform group-hover:scale-110 group-hover:rotate-3 transition-all duration-500`}>
                          {p.icon}
                      </div>

                      {/* Text Content */}
                      <h2 className={`text-4xl text-center font-black text-transparent bg-clip-text bg-gradient-to-br ${p.color} uppercase tracking-wider mb-4`}>
                          {p.label}
                      </h2>

                      <div className="w-12 h-1 bg-white/10 rounded-full my-6" />

                      {/* 1st Place Special Shimmer */}
                      {p.rank === 1 && (
                         <div className="absolute inset-0 bg-gradient-to-tr from-transparent via-white/20 to-transparent translate-y-full group-hover:animate-[shimmer_2s_infinite]" />
                      )}
                  </div>
              </motion.div>
          ))}
      </div>

      {/* --- FOOTER BUTTON --- */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.2 }}
        className="mt-8 relative z-10"
      >
         <button
            onClick={onNext}
            className="px-12 py-4 bg-white/10 hover:bg-yellow-500 text-white hover:text-black border border-white/20 hover:border-yellow-500 font-bold text-lg uppercase tracking-[0.2em] rounded-xl transition-all hover:scale-105 backdrop-blur-md"
        >
            {t.continue_dash}
        </button>
      </motion.div>

    </div>
  );
}