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

    // Use justify-around so a single item centers itself instead of sticking to the left
    const containerAlignment = history.length <= 2 ? "justify-around" : "justify-between";

    return (
        <div className={`w-full h-full flex items-end ${containerAlignment} gap-2 pt-6 pb-2 px-2`}>
            {history.map((rank, i) => {
                // --- CUSTOM HEIGHT LOGIC ---
                // Rank 1 gets 95% height. Scales down by 4.5% per rank. Minimum height is 30%.
                const heightPercent = Math.max(30, 95 - ((rank - 1) * 4.5));

                // --- COLOR LOGIC ---
                // Default (4th place and below) - Using the readable "Deep Tech Blue"
                let barClass = "bg-gradient-to-t from-blue-950 to-blue-800 border border-blue-500/20 group-hover:from-blue-900 group-hover:to-blue-700 shadow-[0_0_10px_rgba(30,58,138,0.2)]";
                let textClass = "text-blue-400/70 group-hover:text-blue-300 transition-colors";

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

                // Make the bar much wider if there's only 1 match in the history
                const barWidth = history.length === 1 ? "max-w-[80px]" : "max-w-[45px]";

                return (
                    <div key={i} className="flex-1 flex flex-col items-center justify-end h-full group relative">

                        {/* Rank Label (Fixed text-1xl to text-xl) */}
                        <motion.span
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ delay: i * 0.1 + 0.5 }}
                            className={`text-xl font-bold mb-2 font-mono transition-transform ${textClass}`}
                        >
                            #{rank}
                        </motion.span>

                        {/* The Bar */}
                        <motion.div
                            initial={{ height: "0%" }}
                            animate={{ height: `${heightPercent}%` }}
                            transition={{ duration: 0.6, delay: i * 0.1, type: "spring", bounce: 0.2 }}
                            className={`w-full ${barWidth} rounded-t-lg relative overflow-hidden transition-all duration-300 ${barClass}`}
                        >
                            {/* Shine effect for Gold only */}
                            {rank === 1 && (
                                <div className="absolute inset-0 bg-white/30 translate-y-full animate-[shimmer_2s_infinite]" />
                            )}
                        </motion.div>

                        {/* X-Axis Label (Fixed text-1xl to text-xl) */}
                        <span className="text-xl text-gray-600 mt-2 font-mono uppercase">Q{i + 1}</span>
                    </div>
                );
            })}
        </div>
    );
};
// --- SUB-COMPONENT: SLOT MACHINE NUMBER ---
const SlotMachineRank = ({ rank, delay }) => {
    // 1. Create a "strip" of fake numbers to spin through, ending with the real rank
    const fakeSpins = [12, 5, 16, 8, 2, 14, 7, 11, 4, 9, 15, 3];
    const allSpins = [...fakeSpins, rank];

    // 2. Format the numbers
    const formatRank = (r, isFinal) => {
        if (!isFinal) return `#${r}`;
        if (r === 1) return "1st";
        if (r === 2) return "2nd";
        if (r === 3) return "3rd";
        return `#${r}`;
    };

    const itemHeight = 40;

    return (
        // FIX: Removed absolute positioning. Used items-start to pin the strip to the top.
        // ADDED: w-[60px] to ensure the container never collapses to 0 width.
        <div className="h-[40px] w-[60px] overflow-hidden flex justify-center items-start leading-none">
            <motion.div
                className="flex flex-col items-center"
                initial={{ y: 0 }}
                animate={{ y: -(allSpins.length - 1) * itemHeight }}
                transition={{
                    duration: 2.5,
                    delay: delay,
                    ease: [0.15, 0.85, 0.2, 1], // The "grinding to a halt" slot machine friction
                }}
            >
                {allSpins.map((val, idx) => {
                    const isFinal = idx === allSpins.length - 1;
                    return (
                        <span
                            key={idx}
                            className={`h-[40px] flex items-center justify-center font-black ${
                                isFinal && val === 1 ? 'text-2xl' : 'text-3xl'
                            }`}
                        >
              {formatRank(val, isFinal)}
            </span>
                    );
                })}
            </motion.div>
        </div>
    );
};
// --- SUB-COMPONENT: RECENT FORM BADGES (Sports Style) ---
const RecentFormBadges = ({ history }) => {
    if (!history || history.length === 0) {
        return (
            <div className="flex-1 flex items-center justify-center border border-white/5 bg-white/5 rounded-xl text-gray-500 italic text-sm">
                No matches played yet
            </div>
        );
    }

    return (
        <div className="w-full h-full flex items-center justify-center gap-4 pt-4 pb-2 px-2 overflow-x-auto">
            {history.map((rank, i) => {

                // --- EMPTY SLOT (Did not play this volume) ---
                if (rank === "?") {
                    return (
                        <div
                            key={i}
                            className="flex-shrink-0 w-28 h-28 rounded-2xl border-2 border-dashed border-white/10 bg-black/20 flex flex-col items-center justify-center relative"
                        >
              <span className="font-mono text-[10px] absolute top-1.5 uppercase tracking-widest text-white/30 font-bold">
                Vol.{i + 1}
              </span>
                            <div className="mt-3 w-3 h-1 rounded-full bg-white/10"></div>
                        </div>
                    );
                }

                // --- BADGE COLOR & ICON LOGIC ---
                // Default (4th place and below)
                let badgeClass = "bg-blue-950/80 border-blue-800 text-blue-300 shadow-[0_2px_10px_rgba(30,58,138,0.2)]";

                // Use our new Slot Machine component! Stagger the spin delay based on index.
                let rankContent = <SlotMachineRank rank={rank} delay={0.2 + (i * 0.15)} />;

                if (rank === 1) { // GOLD
                    badgeClass = "bg-gradient-to-br from-yellow-400 to-yellow-600 border-yellow-300 text-yellow-950 shadow-[0_0_20px_rgba(234,179,8,0.5)] scale-110 z-10";
                    rankContent = (
                        <div className="flex flex-col items-center">
                            <motion.div
                                initial={{ opacity: 0, scale: 0 }}
                                animate={{ opacity: 1, scale: 1 }}
                                transition={{ delay: 2.2 + (i * 0.15), type: "spring" }} // Crown pops in right as spin finishes
                            >
                                <Crown size={18} className="mb-0.5" strokeWidth={3} />
                            </motion.div>
                            <SlotMachineRank rank={rank} delay={0.2 + (i * 0.15)} />
                        </div>
                    );
                } else if (rank === 2) { // SILVER
                    badgeClass = "bg-gradient-to-br from-slate-300 to-slate-400 border-slate-200 text-slate-900 shadow-[0_0_15px_rgba(203,213,225,0.4)] scale-105 z-10";
                } else if (rank === 3) { // BRONZE
                    badgeClass = "bg-gradient-to-br from-orange-400 to-orange-600 border-orange-300 text-orange-950 shadow-[0_0_15px_rgba(249,115,22,0.4)] scale-105 z-10";
                }

                return (
                    <motion.div
                        key={i}
                        // Removed the aggressive popup animation, replaced with a smooth, fast fade-in
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        transition={{ duration: 0.4 }}
                        className={`flex-shrink-0 w-28 h-28 rounded-2xl border-2 flex flex-col items-center justify-center relative overflow-hidden ${badgeClass}`}
                    >
                        {/* Shine effect for Gold only */}
                        {rank === 1 && (
                            <div className="absolute inset-0 bg-white/40 -translate-x-full animate-[shimmer_2s_infinite]" />
                        )}

                        {/* Volume / Match Label */}
                        <span className="font-mono text-[10px] absolute top-1.5 uppercase tracking-widest opacity-60 font-bold">
              Vol.{i + 1}
            </span>

                        {/* Main Rank Display (The Slot Machine) */}
                        <div className="mt-3">
                            {rankContent}
                        </div>
                    </motion.div>
                );
            })}
        </div>
    );
};

// --- SUB-COMPONENT: ACHIEVEMENT SLOTS (Replaces RankGraph) ---
const AchievementSlots = ({ history }) => {
    // We expect exactly 5 quizzes total until now.
    const TOTAL_QUIZZES = 5;
    const safeHistory = history || [];

    return (
        <div className="w-full h-full flex items-center justify-between gap-3 pt-4 pb-2 px-2">
            {Array.from({ length: TOTAL_QUIZZES }).map((_, i) => {
                const rank = safeHistory[i];
                const hasPlayed = rank !== undefined;

                // --- EMPTY SLOT (Did not play this volume) ---
                if (!hasPlayed) {
                    return (
                        <div key={i} className="flex-1 aspect-square max-w-[85px] rounded-2xl border-2 border-dashed border-white/10 bg-black/20 flex flex-col items-center justify-center relative">
                            <span className="text-white/20 font-mono text-xs absolute top-2 uppercase tracking-widest">Vol.{i + 1}</span>
                            <div className="w-3 h-3 rounded-full bg-white/5 mt-3"></div>
                        </div>
                    );
                }

                // --- FILLED SLOT COLOR LOGIC ---
                // Default (4th place and below) - Deep Blue
                let slotClass = "bg-gradient-to-br from-blue-950 to-slate-900 border-blue-500/30 shadow-[0_0_15px_rgba(30,58,138,0.3)]";
                let textClass = "text-blue-300";

                if (rank === 1) { // GOLD
                    slotClass = "bg-gradient-to-br from-yellow-400 to-yellow-600 border-yellow-300 shadow-[0_0_20px_rgba(234,179,8,0.6)]";
                    textClass = "text-yellow-950 scale-110";
                } else if (rank === 2) { // SILVER
                    slotClass = "bg-gradient-to-br from-slate-300 to-slate-500 border-slate-200 shadow-[0_0_15px_rgba(203,213,225,0.4)]";
                    textClass = "text-slate-900 scale-105";
                } else if (rank === 3) { // BRONZE
                    slotClass = "bg-gradient-to-br from-orange-400 to-orange-700 border-orange-300 shadow-[0_0_15px_rgba(249,115,22,0.4)]";
                    textClass = "text-orange-950 scale-105";
                }

                // --- FILLED SLOT RENDER ---
                return (
                    <motion.div
                        key={i}
                        initial={{ opacity: 0, scale: 0.5 }}
                        animate={{ opacity: 1, scale: 1 }}
                        transition={{ delay: i * 0.1, type: "spring", bounce: 0.4 }}
                        className={`flex-1 aspect-square max-w-[85px] rounded-2xl border flex flex-col items-center justify-center relative overflow-hidden ${slotClass}`}
                    >
                        {/* Shine effect for Gold only */}
                        {rank === 1 && (
                            <div className="absolute inset-0 bg-white/40 -translate-x-full animate-[shimmer_2s_infinite]" />
                        )}

                        {/* Volume Label */}
                        <span className={`font-mono text-xs absolute top-2 uppercase tracking-widest ${rank <= 3 ? 'text-black/50' : 'text-white/40'}`}>
              Vol.{i + 1}
            </span>

                        {/* Rank Number */}
                        <span className={`text-3xl font-black mt-3 ${textClass}`}>
              #{rank}
            </span>
                    </motion.div>
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

  // 1. Determine the team name size
  const nameSize = activeTeam.name.length > 15 ? 'text-5xl' : 'text-7xl';

  // 2. Base quote size (one Tailwind size smaller than the name)
  let quoteSize = activeTeam.name.length > 15 ? 'text-4xl' : 'text-5xl';

  // 3. Apply the quote's own rules to shrink it further if it's too long
  if (activeTeam.quote?.length > 120) {
      quoteSize = 'text-3xl';
  } else if (activeTeam.quote?.length > 80) {
      quoteSize = 'text-4xl';
  } else if (activeTeam.quote?.length > 50) {
      quoteSize = 'text-4xl';
  }
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
                                <div className="w-75 h-75 rounded-3xl bg-white/5 mb-8 flex items-center justify-center shadow-inner border border-white/10 relative z-10 overflow-hidden group">
                                    {activeTeam.image ? (
                                        <img src={activeTeam.image} alt={activeTeam.name} className="w-full h-full object-cover transition-all duration-700" />
                                    ) : (
                                        <ActiveIcon size={140} className="text-white/80 drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]" />
                                    )}
                                </div>
                                <h2 className={`font-black text-center leading-none mb-6 z-10 drop-shadow-lg tracking-tight ${nameSize}`}>
                                    {activeTeam.name}
                                </h2>
                                <p className={`text-gray-400 italic text-center z-10 px-4 leading-relaxed ${
                                    !activeTeam.quote ? 'hidden' : quoteSize
                                }`}>
                                    "{activeTeam.quote}"
                                </p>
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
                                    <RecentFormBadges history={activeTeam.rankHistory} />
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