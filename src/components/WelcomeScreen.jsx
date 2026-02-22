import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { TEAMS_DATA } from "../data/teams";
import { Clock, Users, Trophy, Medal, TrendingUp, Target, Crown, Play, Timer, RotateCw } from "lucide-react";

// --- SUB-COMPONENT: RANK GRAPH (BAR CHART VERSION) ---
const RankGraph = ({ history }) => {
  if (!history || history.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center border border-white/5 bg-white/5 rounded-xl text-gray-500 italic text-sm">
        No matches played yet
      </div>
    );
  }

  const lowestRank = Math.max(...history, 16);

  return (
    <div className="w-full h-full flex items-end justify-between gap-2 pt-6 pb-2 px-2">
      {history.map((rank, i) => {
        // --- CUSTOM HEIGHT LOGIC ---
        let heightPercent;
        heightPercent = Math.max(15, ((lowestRank - rank) / lowestRank) * 80);

        // --- COLOR LOGIC ---
        let barClass = "bg-white/10 group-hover:bg-white/20"; // Default
        let textClass = "text-gray-500";

        if (rank === 1) { // GOLD
          barClass = "bg-gradient-to-t from-yellow-600 to-yellow-300 shadow-[0_0_15px_rgba(234,179,8,0.6)]";
          textClass = "text-yellow-400 scale-125";
        } else if (rank === 2) { // SILVER
          barClass = "bg-gradient-to-t from-slate-500 to-slate-300 shadow-[0_0_10px_rgba(203,213,225,0.4)]";
          textClass = "text-slate-300 scale-110";
        } else if (rank === 3) { // BRONZE
          barClass = "bg-gradient-to-t from-orange-800 to-orange-500 shadow-[0_0_10px_rgba(249,115,22,0.4)]";
          textClass = "text-orange-500 scale-105";
        }

        return (
          <div key={i} className="flex-1 flex flex-col items-center justify-end h-full group relative">

            {/* Rank Label */}
            <motion.span
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.1 + 0.5 }}
              className={`text-1xl font-bold mb-2 font-mono transition-transform ${textClass}`}
            >
              #{rank}
            </motion.span>

            {/* The Bar */}
            <motion.div
              initial={{ height: "0%" }}
              animate={{ height: `${heightPercent}%` }}
              transition={{ duration: 0.6, delay: i * 0.1, type: "spring", bounce: 0.2 }}
              className={`w-full max-w-[40px] rounded-t-lg relative overflow-hidden transition-all duration-300 ${barClass}`}
            >
              {/* Shine effect for Gold only */}
              {rank === 1 && (
                <div className="absolute inset-0 bg-white/30 translate-y-full animate-[shimmer_2s_infinite]" />
              )}
            </motion.div>

            {/* X-Axis Label */}
            <span className="text-1xl text-gray-600 mt-2 font-mono uppercase">Q{i + 1}</span>
          </div>
        );
      })}
    </div>
  );
};

// --- SUB-COMPONENT: ROTATING HEADER ITEM ---
const RotatingHeaderItem = ({ children }) => (
  <motion.div
    initial={{ rotateX: -90, y: 50, opacity: 0 }}
    animate={{ rotateX: 0, y: 0, opacity: 1 }}
    exit={{ rotateX: 90, y: -50, opacity: 0 }}
    transition={{ duration: 0.8, type: "spring", bounce: 0.3 }}
    className="absolute inset-0 flex flex-col items-center justify-center backface-hidden"
    style={{ transformOrigin: "50% 50% -20px" }}
  >
    {children}
  </motion.div>
);

// --- MAIN COMPONENT ---
export default function WelcomeScreen({ onStart, startTime, showTime, config, t }) {
  const uiScale = config?.uiScale || 1;
  const splitDelay = config?.splitDelay || 3000;
  const cycleDuration = config?.cycleDuration || 8000;
  const headerInterval = config?.headerInterval || 5000;

  const [activeIndex, setActiveIndex] = useState(0);
  const [viewState, setViewState] = useState("center");
  const [isExiting, setIsExiting] = useState(false);
  const [isPaused, setIsPaused] = useState(false);

  const [headerState, setHeaderState] = useState(0);
  const [countdownText, setCountdownText] = useState("");

  // --- HEADER CYCLE LOGIC ---
  useEffect(() => {
    if(!showTime) return;
    const interval = setInterval(() => {
      setHeaderState(prev => (prev + 1) % 3);
    }, headerInterval);
    return () => clearInterval(interval);
  }, [headerInterval, showTime]);

  // --- COUNTDOWN LOGIC ---
  useEffect(() => {
    const updateCountdown = () => {
      if (!startTime) return;
      const now = new Date();
      const [hours, minutes] = startTime.split(':').map(Number);
      const target = new Date();
      target.setHours(hours, minutes, 0, 0);
      const diff = target - now;

      if (diff <= 0) {
        setCountdownText(t.happening_now);
      } else {
        const h = Math.floor(diff / (1000 * 60 * 60));
        const m = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
        const s = Math.floor((diff % (1000 * 60)) / 1000);
        if (h > 0) setCountdownText(`${t.starts_in} ${h} ${t.hr} ${m} ${t.min}`);
        else setCountdownText(`${t.starts_in} ${m} ${t.min} ${s} ${t.sec}`);
      }
    };
    updateCountdown();
    const timer = setInterval(updateCountdown, 1000);
    return () => clearInterval(timer);
  }, [startTime, t]);


  // --- MAIN ANIMATION LOGIC ---
  useEffect(() => {
    if (isExiting) return;

    // Always reset to center when index changes
    setViewState("center");

    // Always unfold details after delay (even if paused)
    const splitTimer = setTimeout(() => setViewState("detail"), splitDelay);

    let nextTimer, foldTimer;

    // Only auto-cycle if NOT paused
    if (!isPaused) {
        foldTimer = setTimeout(() => setViewState("center"), cycleDuration - 600);
        nextTimer = setTimeout(() => {
            setActiveIndex((prev) => (prev + 1) % TEAMS_DATA.length);
        }, cycleDuration);
    }

    return () => { clearTimeout(splitTimer); clearTimeout(foldTimer); clearTimeout(nextTimer); };
  }, [activeIndex, splitDelay, cycleDuration, isExiting, isPaused]); // Added isPaused dependency

  // --- HANDLERS ---
  const handleStart = () => {
    setIsExiting(true);
    setTimeout(() => {
      onStart();
    }, 800);
  };

  const handleManualSelect = (index) => {
      setActiveIndex(index);
      setIsPaused(true); // Stop auto-rotation
  };

  const getIndex = (offset) => (activeIndex + offset) % TEAMS_DATA.length;
  const activeTeam = TEAMS_DATA[activeIndex];
  const ActiveIcon = activeTeam.icon;

  return (
    <motion.div
      initial={{ y: 0 }}
      animate={isExiting ? { y: "-100vh" } : { y: 0 }}
      transition={{ duration: 0.8, ease: "easeInOut" }}
      className="h-screen w-full relative overflow-hidden flex flex-col bg-[#050505] font-['League_Spartan']"
    >

      <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0" />
      <div className={`absolute inset-0 opacity-20 bg-gradient-to-r ${activeTeam.color} blur-[150px] transition-colors duration-1000`} />

      <div style={{ transform: `scale(${uiScale})` }} className="relative w-full h-full flex flex-col transition-transform duration-500 origin-center">

        {/* 1. ROTATING HEADER */}
        <div className="h-[20vh] w-full relative z-50 perspective-1000">
           <AnimatePresence mode="wait">
              {(headerState === 0 || !showTime) && (
                <RotatingHeaderItem key="title">
                   <h1 className="text-8xl font-black text-white uppercase tracking-[0.1em] drop-shadow-2xl">{t.title}</h1>
                </RotatingHeaderItem>
              )}
              {headerState === 1 && showTime && (
                 <RotatingHeaderItem key="time">
                    <div className="flex flex-col items-center">
                        <span className="text-3xl text-yellow-500 font-bold tracking-[0.3em] mb-2 uppercase">{t.event_schedule}</span>
                        <div className="flex items-center gap-4 text-white font-mono text-6xl font-bold bg-white/5 px-8 py-2 rounded-2xl border border-white/10 backdrop-blur-md">
                           <Clock size={48} className="text-yellow-500" />
                           <span>{startTime}</span>
                        </div>
                    </div>
                 </RotatingHeaderItem>
              )}
              {headerState === 2 && showTime && (
                 <RotatingHeaderItem key="countdown">
                    <div className="flex flex-col items-center">
                        <span className="text-4xl text-green-500 font-bold tracking-[0.3em] mb-2 uppercase">{t.get_ready}</span>
                        <div className="flex items-center gap-4 text-white font-mono text-6xl font-bold bg-white/5 px-8 py-2 rounded-2xl border border-white/10 backdrop-blur-md">
                           <Timer size={48} className="text-green-500" />
                           <span>{countdownText}</span>
                        </div>
                    </div>
                 </RotatingHeaderItem>
              )}
           </AnimatePresence>
        </div>

        {/* 2. MAIN CONTENT */}
        <div className="h-[65vh] w-full relative flex items-center pl-32 perspective-1000">

            <div className="relative w-[600px] h-[800px] z-20">
                <AnimatePresence mode="popLayout">

                    {[2, 1].map((offset) => {
                        const stackIdx = getIndex(offset);
                        const stackTeam = TEAMS_DATA[stackIdx];
                        return (
                            <motion.div
                                key={`stack-${stackTeam.id}`}
                                layoutId={`card-${stackTeam.id}`}
                                initial={{ opacity: 0, scale: 0.8, y: 50 }}
                                animate={{
                                    scale: 1 - (offset * 0.05),
                                    y: offset * -30,
                                    x: offset * 20,
                                    zIndex: 10 - offset,
                                    opacity: 0.4 - (offset * 0.1)
                                }}
                                className={`absolute inset-0 rounded-[3rem] bg-gradient-to-br ${stackTeam.color} border border-white/10 shadow-2xl`}
                            />
                        );
                    })}

                    <motion.div
                        key={activeTeam.id}
                        layoutId={`card-${activeTeam.id}`}
                        className="absolute inset-0 z-30"
                        initial={{ y: 0, scale: 0.95, opacity: 0 }}
                        animate={{ y: 0, x: 0, scale: 1, opacity: 1 }}
                        exit={{
                            y: -1200,
                            opacity: 0,
                            scale: 1,
                            rotate: -5,
                            zIndex: 100,
                            transition: { duration: 0.5, ease: "backIn" }
                        }}
                        transition={{ type: "spring", bounce: 0.2, duration: 0.8 }}
                    >
                        <div className={`w-full h-full rounded-[3rem] bg-gradient-to-br ${activeTeam.color} p-3 shadow-[0_0_80px_rgba(0,0,0,0.6)]`}>
                            <div className="h-full w-full bg-black/85 backdrop-blur-md rounded-[2.5rem] p-10 flex flex-col items-center text-white relative overflow-hidden border border-white/5">
                                <div className="absolute -bottom-32 -right-32 opacity-10 rotate-12 scale-[3] text-white">
                                    <ActiveIcon size={250} />
                                </div>
                                <div className="w-64 h-64 rounded-3xl bg-white/5 mb-8 flex items-center justify-center shadow-inner border border-white/10 relative z-10 overflow-hidden group">
                                    {activeTeam.image ? (
                                        <img src={activeTeam.image} alt={activeTeam.name} className="w-full h-full object-cover transition-all duration-700" />
                                    ) : (
                                        <ActiveIcon size={140} className="text-white/80 drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]" />
                                    )}
                                </div>
                                <h2 className="text-7xl font-black text-center leading-none mb-6 z-10 drop-shadow-lg tracking-tight">{activeTeam.name}</h2>
                                <p className="text-4xl text-gray-400 italic text-center z-10 px-4 leading-relaxed">"{activeTeam.quote}"</p>
                                <div className="mt-auto flex gap-4 z-10">
                                    <span className="px-6 py-3 bg-white/10 rounded-full flex items-center gap-3 text-4xl font-bold uppercase tracking-wider border border-white/10">
                                        <Users size={48} /> {activeTeam.players} {t.players}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </motion.div>

                    <AnimatePresence>
                    {viewState === "detail" && !activeTeam.isNew && (
                        <motion.div
                            key={`details-${activeTeam.id}`}
                            initial={{ opacity: 0, x: 0, zIndex: -1 }}
                            animate={{ opacity: 1, x: 620, zIndex: 10 }}
                            exit={{ opacity: 0, x: 200 }}
                            transition={{ duration: 0.8, type: "spring", bounce: 0.3 }}
                            className="absolute top-0 bottom-0 w-[850px] bg-[#0a0a0a] border border-white/20 rounded-r-[3rem] rounded-l-[2rem] p-10 flex flex-col shadow-2xl"
                        >
                            <div className="flex justify-between items-start border-b border-white/10 pb-6 mb-6">
                                <div>
                                    <h3 className="text-gray-400 uppercase tracking-[0.2em] text-4xl mb-1">{t.all_time_best}</h3>
                                    <span className="text-white font-black text-6xl flex items-center gap-3">
                                      <Crown size={65} className="text-yellow-500"/> {t.rank} #{activeTeam.bestRank}
                                    </span>
                                </div>
                                <div className="text-right">
                                    <h3 className="text-gray-400 uppercase tracking-[0.2em] text-4xl mb-1">{t.accuracy}</h3>
                                    <span className="text-white font-black text-6xl flex items-center justify-end gap-3">
                                       {activeTeam.correctRate}% <Target size={65} className="text-green-500"/>
                                    </span>
                                </div>
                            </div>

                            <div className="grid grid-cols-3 gap-4 mb-6">
                                {["first", "second", "third"].map((place, i) => {
                                    let label = t.place_1;
                                    if (i === 1) label = t.place_2;
                                    if (i === 2) label = t.place_3;
                                    return(
                                    <div key={place} className={`p-5 rounded-2xl flex flex-col items-center bg-white/5 border border-white/5 relative overflow-hidden`}>
                                        <div className={`absolute top-0 left-0 w-full h-1 ${i===0?"bg-yellow-500":i===1?"bg-gray-400":"bg-orange-700"}`}/>
                                        <Medal size={64} className={`mb-2 ${i===0?"text-yellow-400":i===1?"text-gray-400":"text-orange-600"}`} />
                                        <span className="text-6xl font-black text-white">{activeTeam.podium?.[place] || 0}</span>
                                        <span className="text-3xl uppercase tracking-widest text-gray-500 mt-1">{label}</span>
                                    </div>
                                    );
                                })}
                            </div>

                            <div className="mt-auto flex-1 flex flex-col">
                                 <h3 className="text-gray-400 uppercase tracking-widest text-2xl mb-3 flex items-center gap-2">
                                    <TrendingUp size={32}/> {t.season_history}
                                 </h3>
                                 <div className="flex-1">
                                    <RankGraph history={activeTeam.rankHistory} />
                                 </div>
                            </div>
                        </motion.div>
                    )}
                    </AnimatePresence>
                </AnimatePresence>
            </div>

            <div className="absolute right-24 top-10 bottom-32 w-[550px] z-10 flex flex-col justify-start">
               <div className="flex justify-between items-center mb-4 border-b border-white/10 pb-2">
                   <h3 className="text-gray-500 text-3xl uppercase tracking-widest">{t.roster} ({TEAMS_DATA.length})</h3>
                   {isPaused && (
                       <button onClick={() => setIsPaused(false)} className="flex items-center gap-2 text-1xl font-bold text-green-400 bg-green-400/10 px-3 py-1 rounded-full animate-pulse hover:bg-green-400/20">
                           <RotateCw size={12}/> {t.resume_autoplay}
                       </button>
                   )}
               </div>

               <div className="grid grid-cols-4 gap-3">
                   {TEAMS_DATA.map((team, i) => {
                       const isActive = i === activeIndex;
                       const isPast = i < activeIndex;
                       const TIcon = team.icon;

                       return (
                           <motion.div
                               key={team.id}
                               layout
                               onClick={() => handleManualSelect(i)} // MANUAL SELECTION
                               className={`aspect-square rounded-xl border relative overflow-hidden transition-all duration-300 group cursor-pointer
                                   ${isActive ? "border-yellow-500 scale-110 shadow-[0_0_15px_rgba(234,179,8,0.6)] z-20 bg-white/20" : "border-white/10 bg-white/5"}
                                   ${isPast ? "opacity-30 grayscale border-transparent hover:opacity-100 hover:grayscale-0" : "opacity-70 hover:opacity-100 hover:border-white/30"}
                               `}
                           >
                               {team.image ? (
                                   <img src={team.image} className="w-full h-full object-cover" />
                               ) : (
                                   <div className="w-full h-full flex items-center justify-center text-white/50 group-hover:text-white">
                                        <TIcon size={28} />
                                   </div>
                               )}
                               <div className={`absolute top-1 left-1 text-[15px] font-bold px-1.5 rounded
                                   ${isActive ? "bg-yellow-500 text-black" : "bg-black/50 text-white/50"}`}>
                                   {i + 1}
                               </div>
                               {isActive && (
                                 <motion.div
                                    className="absolute inset-0 border-2 border-yellow-500 rounded-xl"
                                    animate={{ opacity: [0.5, 1, 0.5] }}
                                    transition={{ duration: 1.5, repeat: Infinity }}
                                 />
                               )}
                           </motion.div>
                       )
                   })}
               </div>
            </div>

        </div>

        {/* 3. FOOTER */}
        <div className="h-[15vh] w-full flex items-center justify-center z-[100] relative">
            {!isExiting && (
                <button
                    onClick={handleStart}
                    className="group relative flex flex-col items-center justify-center p-10 outline-none"
                >
                    {/* The Main Text - Starts Ghosted, Glows on Hover */}
                    <span className="text-4xl font-black uppercase tracking-[0.2em] text-white/20 transition-all duration-500 group-hover:text-yellow-400 group-hover:tracking-[0.4em] group-hover:scale-110 group-hover:drop-shadow-[0_0_15px_rgba(234,179,8,0.8)]">
                        {t.start_show}
                    </span>

                    {/* The "Power Line" - Expands from center */}
                    <div className="h-[2px] bg-gradient-to-r from-transparent via-yellow-500 to-transparent w-0 group-hover:w-full transition-all duration-700 ease-out mt-4 opacity-0 group-hover:opacity-100" />

                    {/* Subtle Play Icon that appears */}
                    <motion.div
                        initial={{ opacity: 0, y: -10 }}
                        whileHover={{ opacity: 1, y: 5 }}
                        className="absolute bottom-2 text-yellow-500"
                    >
                         <Play size={16} fill="currentColor" />
                    </motion.div>
                </button>
            )}
        </div>

      </div>
    </motion.div>
  );
}