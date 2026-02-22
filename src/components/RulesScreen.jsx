import { motion } from "framer-motion";
import { Shield, Check, AlertTriangle, Gavel, ArrowRight } from "lucide-react";

export default function RulesScreen({ onNext, t }) {
  const rules = [
    { icon: <Shield size={32} className="text-blue-400" />, text: t.rule_1 },
    { icon: <Gavel size={32} className="text-red-400" />, text: t.rule_2 },
    { icon: <AlertTriangle size={32} className="text-yellow-400" />, text: t.rule_3 },
    { icon: <Check size={32} className="text-green-400" />, text: t.rule_4 },
  ];

  return (
    <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col items-center justify-center font-['League_Spartan']">

      {/* --- BACKGROUND AMBIENCE (Matching Welcome) --- */}
      <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0" />
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[500px] bg-blue-600/10 blur-[120px] rounded-full opacity-50 pointer-events-none" />
      <div className="absolute bottom-0 right-0 w-[600px] h-[600px] bg-purple-900/10 blur-[150px] rounded-full opacity-30 pointer-events-none" />

      {/* --- MAIN CONTENT CONTAINER --- */}
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.8, ease: "circOut" }}
        className="relative z-10 w-full max-w-5xl"
      >
        {/* HEADER */}
        <div className="text-center mb-16 relative">
             <motion.h1
                initial={{ y: -50, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.2, type: "spring" }}
                className="text-8xl font-black text-white uppercase tracking-[0.1em] drop-shadow-2xl"
             >
                {t.rules_title}
             </motion.h1>
             <motion.div
                initial={{ width: 0 }}
                animate={{ width: "200px" }}
                transition={{ delay: 0.5, duration: 0.8 }}
                className="h-1 bg-blue-500 mx-auto mt-4 rounded-full shadow-[0_0_15px_rgba(59,130,246,0.5)]"
             />
        </div>

        {/* RULES GRID */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 px-8">
            {rules.map((rule, i) => (
                <motion.div
                    key={i}
                    initial={{ opacity: 0, x: i % 2 === 0 ? -30 : 30, y: 20 }}
                    animate={{ opacity: 1, x: 0, y: 0 }}
                    transition={{ delay: 0.4 + (i * 0.1), type: "spring", bounce: 0.2 }}
                    className="group relative overflow-hidden bg-white/5 border border-white/10 rounded-3xl p-8 hover:bg-white/10 transition-colors duration-300"
                >
                    {/* Hover Glow Effect */}
                    <div className="absolute inset-0 bg-gradient-to-r from-blue-500/0 via-white/5 to-blue-500/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />

                    <div className="flex items-center gap-6 relative z-10">
                        <div className="flex-shrink-0 w-16 h-16 rounded-2xl bg-black/40 border border-white/10 flex items-center justify-center shadow-inner group-hover:scale-110 transition-transform duration-300">
                            {rule.icon}
                        </div>
                        <span className="text-2xl text-gray-200 font-bold leading-tight group-hover:text-white transition-colors">
                            {rule.text}
                        </span>
                    </div>
                </motion.div>
            ))}
        </div>

        {/* FOOTER ACTION */}
        <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 1 }}
            className="mt-16 flex justify-center"
        >
            <button
                onClick={onNext}
                className="group relative px-12 py-5 bg-white text-black font-black text-xl uppercase tracking-widest rounded-full overflow-hidden transition-all hover:scale-105 hover:shadow-[0_0_40px_rgba(255,255,255,0.3)]"
            >
                <div className="absolute inset-0 bg-gradient-to-r from-blue-400 to-cyan-300 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <span className="relative z-10 flex items-center gap-3">
                    {t.continue} <ArrowRight className="group-hover:translate-x-1 transition-transform" />
                </span>
            </button>
        </motion.div>

      </motion.div>
    </div>
  );
}