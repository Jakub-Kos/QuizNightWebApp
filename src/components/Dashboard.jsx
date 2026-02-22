import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Play, Eye, Trophy, CheckCircle, Circle, Coffee } from "lucide-react"; // ADDED COFFEE ICON

export default function Dashboard({ rounds, onSelectRound, onOpenLeaderboard, onOpenPause, t }) { // ADDED onOpenPause
  const [hoveredRound, setHoveredRound] = useState(null);

  const [completedRounds, setCompletedRounds] = useState(() => {
      const saved = localStorage.getItem("quizCompletedRounds");
      return saved ? JSON.parse(saved) : [];
  });

  useEffect(() => {
      localStorage.setItem("quizCompletedRounds", JSON.stringify(completedRounds));
  }, [completedRounds]);

  const toggleCompleted = (e, roundTitle) => {
      e.stopPropagation();
      setCompletedRounds(prev =>
          prev.includes(roundTitle) ? prev.filter(title => title !== roundTitle) : [...prev, roundTitle]
      );
  };

  const visibleRounds = rounds.filter(r => {
      const title = (r.title || "").toUpperCase();
      return title !== "KOLO" && !title.includes("TEST") && !title.includes("KOLO X") && r.questions.length !== 0;
  });

  const getExpandedTitleClass = (text) => {
      if (!text) return "text-6xl";
      const len = text.length;
      if (len < 12) return "text-6xl";
      if (len < 18) return "text-[54px]";
      if (len < 24) return "text-5xl";
      if (len < 30) return "text-[42px]";
      return "text-4xl";
  };

  const getCollapsedTitleClass = (text) => {
      if (!text) return "text-5xl";
      const len = text.length;
      if (len < 12) return "text-5xl";
      if (len < 18) return "text-[42px]";
      if (len < 24) return "text-4xl";
      if (len < 30) return "text-[32px]";
      return "text-3xl";
  };

  return (
    <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col font-['League_Spartan'] selection:bg-yellow-500/30">

      <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0 pointer-events-none" />
      <div className="absolute top-0 right-0 w-[800px] h-[800px] bg-blue-900/10 blur-[150px] rounded-full pointer-events-none" />
      <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-yellow-600/5 blur-[120px] rounded-full pointer-events-none" />

      {/* --- HEADER --- */}
      <div className="h-[15vh] flex items-center justify-between px-16 relative z-20 border-b border-white/5 bg-black/40 backdrop-blur-md shrink-0">

          <div className="flex flex-col gap-3 w-[400px]">
              <div className="flex justify-between items-end">
                  <h1 className="text-2xl font-black text-white/50 uppercase tracking-[0.3em]">Live Progress</h1>
                  <span className="text-blue-400 font-mono font-bold tracking-widest text-xl">
                      {completedRounds.length} <span className="text-white/30">/ {visibleRounds.length}</span>
                  </span>
              </div>
              <div className="h-3 w-full bg-white/5 rounded-full overflow-hidden border border-white/10">
                  <motion.div
                      className="h-full bg-gradient-to-r from-blue-600 to-blue-400 shadow-[0_0_15px_rgba(59,130,246,0.6)] rounded-full"
                      initial={{ width: 0 }}
                      animate={{ width: `${(completedRounds.length / visibleRounds.length) * 100}%` }}
                      transition={{ duration: 0.8, type: "spring" }}
                  />
              </div>
          </div>

          <div className="flex gap-6">
              {/* NEW HALF-TIME BUTTON */}
              <button
                 onClick={onOpenPause}
                 className="group relative flex items-center gap-3 px-8 py-5 bg-gradient-to-br from-blue-900/50 to-blue-800/30 hover:from-blue-600 hover:to-blue-500 text-white rounded-full transition-all duration-300 shadow-[0_0_20px_rgba(59,130,246,0.1)] hover:shadow-[0_0_40px_rgba(59,130,246,0.4)] border border-blue-500/20"
              >
                 <Coffee size={28} className="relative z-10" />
                 <span className="font-black text-xl uppercase tracking-[0.15em] relative z-10">
                     HALF-TIME
                 </span>
              </button>

              <button
                 onClick={onOpenLeaderboard}
                 className="group relative flex items-center gap-4 px-10 py-5 bg-gradient-to-br from-yellow-600 to-yellow-400 hover:from-yellow-400 hover:to-yellow-300 text-black rounded-full transition-all duration-300 hover:scale-[1.03] shadow-[0_0_30px_rgba(234,179,8,0.2)] hover:shadow-[0_0_50px_rgba(234,179,8,0.5)] overflow-hidden"
              >
                 <Trophy size={32} className="relative z-10" />
                 <span className="font-black text-2xl uppercase tracking-[0.15em] relative z-10">
                     {t?.leaderboard_btn || "VIEW STANDINGS"}
                 </span>
                 <div className="absolute inset-0 bg-white/40 translate-x-[-150%] skew-x-12 group-hover:animate-[shimmer_1.5s_infinite]" />
              </button>
          </div>
      </div>

      {/* --- MAIN CONTENT (ACCORDION) --- */}
      <div className="flex-1 flex flex-col items-center justify-center p-10 relative z-10 w-full">
          {visibleRounds.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-white/30 space-y-4">
                <div className="w-16 h-16 border-4 border-t-blue-500 border-white/10 rounded-full animate-spin" />
                <p className="font-mono tracking-widest uppercase">Loading Modules...</p>
            </div>
          ) : (
            <div className="flex w-full max-w-[1600px] h-[70vh] gap-4" onMouseLeave={() => setHoveredRound(null)}>
                {visibleRounds.map((round, index) => {
                    const isHovered = hoveredRound === index;
                    const isCompleted = completedRounds.includes(round.title);

                    const parts = (round.title || "").split("-");
                    const themeName = parts.length > 1 ? parts.slice(1).join("-").trim() : round.title;
                    const defaultName = `${t?.round || "Round"} ${index + 1}`;
                    const displayTitle = isCompleted ? themeName : defaultName;

                    return (
                        <motion.div
                            key={round.id || index}
                            onMouseEnter={() => setHoveredRound(index)}
                            animate={{ flex: isHovered ? 4 : 1, opacity: hoveredRound !== null && !isHovered ? 0.4 : 1 }}
                            transition={{ type: "spring", bounce: 0.2, duration: 0.7 }}
                            className={`relative rounded-[2.5rem] overflow-hidden border cursor-pointer group flex flex-col items-center justify-end shadow-2xl transition-colors duration-500 ${
                                isCompleted ? "border-green-500/30 bg-[#051005]" : "border-white/10 bg-[#0a0a0a]"
                            }`}
                        >
                            <div className={`absolute inset-0 transition-all duration-700 ease-in-out pointer-events-none ${
                                isHovered
                                    ? isCompleted ? 'bg-gradient-to-b from-green-900/30 to-black opacity-100' : 'bg-gradient-to-b from-blue-900/30 to-black opacity-100'
                                    : 'bg-black/80 opacity-50'
                            }`} />

                            <motion.div
                                animate={{ top: isHovered ? "5%" : "10%", left: "50%", x: "-50%", scale: isHovered ? 1.5 : 1, opacity: isHovered ? 0.05 : 0.15 }}
                                transition={{ duration: 0.5 }}
                                className={`absolute text-[150px] font-black pointer-events-none leading-none origin-top ${isCompleted ? "text-green-500" : "text-white"}`}
                            >
                                {index + 1}
                            </motion.div>

                            <AnimatePresence>
                                {!isHovered && (
                                    <motion.div
                                        initial={{ opacity: 0 }} animate={{ opacity: 1, transition: { delay: 0.5, duration: 0.4 } }} exit={{ opacity: 0, transition: { duration: 0.1 } }}
                                        className="absolute inset-x-0 bottom-12 flex justify-center pointer-events-none"
                                    >
                                        <div className="w-10 h-10 flex items-end justify-center relative">
                                            <h2
                                                style={{ transformOrigin: "left bottom", transform: "rotate(-90deg) translateX(0) translateY(50%)" }}
                                                className={`absolute left-1/2 bottom-0 w-[55vh] text-left font-bold tracking-[0.2em] uppercase transition-colors leading-tight ${getCollapsedTitleClass(displayTitle)} ${
                                                    isCompleted ? "text-green-500/60 group-hover:text-green-400" : "text-white/40 group-hover:text-white/80"
                                                }`}
                                            >
                                                {displayTitle}
                                            </h2>
                                        </div>
                                    </motion.div>
                                )}
                            </AnimatePresence>

                            <AnimatePresence>
                                {isHovered && (
                                    <motion.div
                                        initial={{ opacity: 0, y: 30 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: 20, transition: { duration: 0.2 } }} transition={{ duration: 0.4, delay: 0.2 }}
                                        className="relative z-10 flex flex-col items-center text-center w-full max-w-[450px] mb-8 px-6"
                                    >
                                        <div className="flex gap-3 mb-6">
                                            <span className="px-5 py-2 bg-blue-500/10 text-blue-400 text-[11px] font-black tracking-widest uppercase rounded-full border border-blue-500/20">
                                                {round.questions.length} Questions
                                            </span>
                                            {isCompleted && (
                                                <span className="px-5 py-2 bg-green-500/20 text-green-400 text-[11px] font-black tracking-widest uppercase rounded-full border border-green-500/30 flex items-center gap-2">
                                                    <CheckCircle size={12}/> Finished
                                                </span>
                                            )}
                                        </div>

                                        <h2 className={`${getExpandedTitleClass(displayTitle)} font-black uppercase tracking-tight mb-8 drop-shadow-xl leading-tight ${isCompleted ? "text-white" : "text-white"}`}>
                                            {displayTitle}
                                        </h2>

                                        <div className="grid grid-cols-2 gap-4 w-full mb-6">
                                            <button onClick={() => onSelectRound(round, "questions_only")} className="py-5 bg-white/5 hover:bg-white/10 text-white rounded-2xl font-bold tracking-widest text-xs flex flex-col items-center justify-center gap-2 transition-all border border-white/10 hover:border-white/20">
                                                <Eye size={24} className="text-gray-400"/><span>Q ONLY</span>
                                            </button>
                                            <button onClick={() => onSelectRound(round, "with_answers")} className="py-5 bg-gradient-to-br from-yellow-400 to-yellow-600 hover:from-yellow-300 hover:to-yellow-500 text-black rounded-2xl font-black tracking-widest text-xs flex flex-col items-center justify-center gap-2 transition-all shadow-[0_0_20px_rgba(234,179,8,0.4)] hover:scale-[1.02]">
                                                <Play size={24} fill="currentColor"/><span>REVEAL</span>
                                            </button>
                                        </div>

                                        <button onClick={(e) => toggleCompleted(e, round.title)} className={`px-6 py-3 rounded-full border text-[10px] sm:text-xs font-bold tracking-widest flex items-center gap-2 transition-all ${isCompleted ? "bg-transparent text-white/30 border-white/10 hover:text-white/80 hover:border-white/30" : "bg-white/5 text-white/50 border-white/10 hover:bg-white/10 hover:text-white"}`}>
                                            {isCompleted ? <Circle size={16} /> : <CheckCircle size={16} />}
                                            {isCompleted ? "MARK AS UNFINISHED" : "MARK AS FINISHED"}
                                        </button>
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </motion.div>
                    )
                })}
            </div>
          )}
      </div>
    </div>
  );
}