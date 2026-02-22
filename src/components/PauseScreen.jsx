import { useState, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Coffee, Play, Pause, Plus, Minus, ArrowLeft, Trophy, Medal, TrendingUp } from "lucide-react";
import { TEAMS_DATA } from "../data/teams";

const getRankText = (rank) => {
    if (rank === 1) return "1ST PLACE";
    if (rank === 2) return "2ND PLACE";
    if (rank === 3) return "3RD PLACE";
    return `${rank}TH PLACE`;
};

const RankGraph = ({ history, maxRank }) => {
    if (!history || history.length < 2) return (
        <div className="mt-8 text-white/30 text-xs tracking-widest uppercase italic flex items-center gap-2">
            <TrendingUp size={16}/> More rounds needed for trend graph
        </div>
    );

    const width = 350;
    const height = 70;
    const padding = 20;

    return (
        <div className="mt-8 flex flex-col items-start relative z-10">
            <span className="text-[10px] font-bold text-white/30 uppercase tracking-[0.3em] mb-6">Tournament Rank Trend</span>
            <svg width={width + padding*2} height={height + padding*2} className="overflow-visible -ml-[20px]">
                <polyline
                    points={history.map((r, i) => `${padding + (i / (history.length - 1)) * width},${padding + ((r - 1) / (maxRank - 1 || 1)) * height}`).join(" ")}
                    fill="none" stroke="rgba(255,255,255,0.2)" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"
                />

                {history.map((rank, i) => {
                    const x = padding + (i / (history.length - 1)) * width;
                    const y = padding + ((rank - 1) / (maxRank - 1 || 1)) * height;
                    const isLast = i === history.length - 1;
                    return (
                        <g key={i}>
                            <circle cx={x} cy={y} r={isLast ? "6" : "4"} fill={isLast ? "#60A5FA" : "#fff"} />
                            <text x={x} y={y - 14} fill={isLast ? "#60A5FA" : "rgba(255,255,255,0.6)"} fontSize="14" fontWeight="900" textAnchor="middle" className="drop-shadow-md">
                                {rank}.
                            </text>
                        </g>
                    )
                })}
            </svg>
            <div className="flex justify-between w-[350px] mt-2 text-[10px] text-white/30 font-bold tracking-widest uppercase">
                <span>Start</span>
                <span>Current</span>
            </div>
        </div>
    )
};

export default function PauseScreen({ isPresenter, leaderboardData, availableRounds, onBack, t }) {
  const [timeLeft, setTimeLeft] = useState(30 * 60);
  const [isActive, setIsActive] = useState(false);
  const [spotlightIndex, setSpotlightIndex] = useState(0);

  const sortedTeams = useMemo(() => {
      if (!leaderboardData || leaderboardData.length === 0) return [];

      const safeRounds = availableRounds || [];
      const activeRounds = safeRounds.filter(r => leaderboardData.some(t => t.scores[r] > 0));

      let cumulativeScores = leaderboardData.map(t => ({ name: t.name, score: 0 }));
      const historyByTeam = {};
      leaderboardData.forEach(t => historyByTeam[t.name] = []);

      activeRounds.forEach(round => {
          cumulativeScores.forEach(t => {
              const teamData = leaderboardData.find(x => x.name === t.name);
              t.score += (teamData.scores[round] || 0);
          });

          const sortedForRound = [...cumulativeScores].sort((a, b) => b.score - a.score);

          // NEW: Standard Competition Ranking for History Graph
          let roundRank = 1;
          sortedForRound.forEach((t, i) => {
              if (i > 0 && t.score < sortedForRound[i - 1].score) {
                  roundRank = i + 1;
              }
              t._tempRank = roundRank;
          });

          cumulativeScores.forEach(t => {
              const teamDataInSorted = sortedForRound.find(x => x.name === t.name);
              historyByTeam[t.name].push(teamDataInSorted._tempRank);
          });
      });

      const finalSorted = [...cumulativeScores].sort((a, b) => b.score - a.score);

      // NEW: Standard Competition Ranking for Current Rank display
      let finalRank = 1;

      return finalSorted.map((t, index) => {
          if (index > 0 && t.score < finalSorted[index - 1].score) {
              finalRank = index + 1;
          }

          const teamMeta = TEAMS_DATA.find(meta => meta.name.toLowerCase() === t.name.toLowerCase()) || {};
          return {
              name: t.name,
              total: t.score,
              rank: finalRank,
              rankHistory: historyByTeam[t.name],
              image: teamMeta.image,
              color: teamMeta.color || "from-blue-900 to-blue-950",
              quote: teamMeta.quote || "Taking a well-deserved break...",
              IconComponent: teamMeta.icon
          };
      });
  }, [leaderboardData, availableRounds]);

  const channel = useMemo(() => new BroadcastChannel('quiz-pause-sync'), []);

  useEffect(() => {
    if (isPresenter) {
      let interval;
      if (isActive && timeLeft > 0) {
        interval = setInterval(() => {
          setTimeLeft(prev => {
            const newTime = prev - 1;
            channel.postMessage({ type: 'TIMER_SYNC', timeLeft: newTime, isActive });
            return newTime;
          });
        }, 1000);
      } else if (!isActive) {
        channel.postMessage({ type: 'TIMER_SYNC', timeLeft, isActive });
      }
      return () => clearInterval(interval);
    }
  }, [isActive, timeLeft, isPresenter, channel]);

  useEffect(() => {
    if (!isPresenter) {
      const handleSync = (e) => {
        if (e.data.type === 'TIMER_SYNC') {
          setTimeLeft(e.data.timeLeft);
          setIsActive(e.data.isActive);
        }
      };
      channel.addEventListener('message', handleSync);
      return () => channel.removeEventListener('message', handleSync);
    }
  }, [isPresenter, channel]);

  useEffect(() => {
      if (!isPresenter && sortedTeams.length > 0) {
          const interval = setInterval(() => {
              setSpotlightIndex(prev => (prev + 1) % sortedTeams.length);
          }, 8000);
          return () => clearInterval(interval);
      }
  }, [isPresenter, sortedTeams.length]);

  const adjustTime = (minutes) => setTimeLeft(prev => Math.max(0, prev + minutes * 60));

  const formatTime = (seconds) => {
      const m = Math.floor(seconds / 60);
      const s = seconds % 60;
      return `${m}:${s.toString().padStart(2, '0')}`;
  };

  if (isPresenter) {
      return (
          <div className="h-screen w-full bg-[#0a0a0a] text-white p-10 font-sans flex flex-col items-center justify-center relative">
              <button onClick={onBack} className="absolute top-10 left-10 flex items-center gap-3 px-6 py-3 bg-white/10 hover:bg-white/20 rounded-xl transition-all font-bold">
                  <ArrowLeft size={20}/> Back to Dashboard
              </button>

              <Coffee size={80} className="text-blue-500 mb-6 opacity-50" />
              <h1 className="text-4xl font-black uppercase tracking-widest text-blue-400 mb-12">Break Timer Controls</h1>

              <div className="text-[120px] font-mono font-bold leading-none mb-12 drop-shadow-[0_0_30px_rgba(59,130,246,0.5)]">
                  {formatTime(timeLeft)}
              </div>

              <div className="flex gap-6 items-center bg-white/5 p-8 rounded-3xl border border-white/10">
                  <button onClick={() => adjustTime(-1)} className="p-5 bg-white/5 hover:bg-white/10 rounded-2xl transition-all"><Minus size={32}/></button>
                  <button onClick={() => setIsActive(!isActive)} className={`flex items-center gap-4 px-12 py-6 rounded-2xl font-black text-2xl tracking-widest uppercase transition-all shadow-xl ${isActive ? "bg-yellow-500 hover:bg-yellow-400 text-black shadow-yellow-500/20" : "bg-green-600 hover:bg-green-500 text-white shadow-green-500/20"}`}>
                      {isActive ? <Pause size={32} fill="currentColor"/> : <Play size={32} fill="currentColor"/>}
                      {isActive ? "PAUSE" : "START TIMER"}
                  </button>
                  <button onClick={() => adjustTime(1)} className="p-5 bg-white/5 hover:bg-white/10 rounded-2xl transition-all"><Plus size={32}/></button>
              </div>
          </div>
      )
  }

  const currentTeam = sortedTeams[spotlightIndex];
  const tickerItems = [...sortedTeams, ...sortedTeams];

  return (
    <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col font-['League_Spartan'] selection:bg-blue-500/30">

        <div className="absolute inset-0 bg-black z-0 pointer-events-none" />
        <AnimatePresence>
            {currentTeam && (
                <motion.div
                    key={`bg-${currentTeam.name}`}
                    initial={{ opacity: 0 }} animate={{ opacity: 0.15 }} exit={{ opacity: 0 }} transition={{ duration: 2 }}
                    className={`absolute inset-0 bg-gradient-to-br ${currentTeam.color} z-0 pointer-events-none blur-3xl`}
                />
            )}
        </AnimatePresence>

        <div className="relative z-10 flex flex-col h-full w-full">

            <div className="flex-none h-[22vh] flex flex-col items-center justify-center pt-8">
                <div className="flex items-center gap-4 text-blue-500 mb-2 animate-pulse">
                    <Coffee size={28} />
                    <h2 className="text-2xl font-black uppercase tracking-[0.4em]">Half-Time Break</h2>
                    <Coffee size={28} />
                </div>

                <div className={`text-[120px] font-mono font-black leading-none drop-shadow-[0_0_50px_rgba(59,130,246,0.3)] transition-colors duration-1000 ${timeLeft <= 60 ? 'text-red-500 drop-shadow-[0_0_50px_rgba(239,68,68,0.5)]' : 'text-white'}`}>
                    {formatTime(timeLeft)}
                </div>

                <div className="text-white/40 font-bold tracking-[0.4em] text-xs uppercase mt-4">
                    {timeLeft === 0 ? "WE ARE RESUMING SHORTLY!" : "UNTIL THE SECOND HALF BEGINS"}
                </div>
            </div>

            <div className="flex-1 flex items-center justify-center p-12 w-full max-w-[1700px] mx-auto">
                <AnimatePresence mode="wait">
                    {currentTeam && (
                        <motion.div
                            key={currentTeam.name}
                            initial={{ opacity: 0, y: 30, scale: 0.95 }}
                            animate={{ opacity: 1, y: 0, scale: 1 }}
                            exit={{ opacity: 0, y: -20, scale: 1.02 }}
                            transition={{ duration: 0.8, ease: "easeInOut" }}
                            className="w-full bg-black/40 backdrop-blur-2xl border border-white/10 rounded-[4rem] p-12 flex items-center gap-16 shadow-[0_0_100px_rgba(0,0,0,0.5)] relative overflow-hidden"
                        >
                            <div className={`absolute top-0 left-0 w-[500px] h-[500px] bg-gradient-to-br ${currentTeam.color} opacity-20 blur-[100px] pointer-events-none rounded-full -translate-x-1/2 -translate-y-1/2`} />

                            <div className="w-[400px] h-[400px] rounded-3xl overflow-hidden shrink-0 border border-white/10 shadow-2xl bg-white/5 flex items-center justify-center relative z-10">
                                {currentTeam.image ? (
                                    <img src={currentTeam.image} alt={currentTeam.name} className="w-full h-full object-cover" />
                                ) : currentTeam.IconComponent ? (
                                    <currentTeam.IconComponent size={150} className="text-white/50" />
                                ) : (
                                    <span className="text-[150px] font-black text-white/30">{currentTeam.name.charAt(0)}</span>
                                )}
                            </div>

                            <div className="flex-1 flex flex-col justify-center relative z-10 h-full py-2">

                                <div className="flex items-center gap-3 mb-4">
                                    {currentTeam.rank === 1 ? <Trophy size={32} className="text-yellow-500" /> : <Medal size={32} className="text-white/30" />}
                                    <span className={`text-xl font-bold tracking-[0.3em] uppercase ${currentTeam.rank === 1 ? 'text-yellow-500' : 'text-white/40'}`}>
                                        Currently in {getRankText(currentTeam.rank)}
                                    </span>
                                </div>

                                <h1 className="text-[70px] font-black text-white uppercase tracking-tight leading-none mb-6 drop-shadow-lg line-clamp-2">
                                    {currentTeam.name}
                                </h1>

                                <div className="flex justify-between items-start gap-12 w-full pr-10">
                                    <div className="flex-1">
                                        <div className="relative pl-6">
                                            <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-white/20 rounded-full" />
                                            <p className="text-2xl text-white/60 font-medium italic leading-relaxed line-clamp-2">
                                                "{currentTeam.quote}"
                                            </p>
                                        </div>

                                        <RankGraph history={currentTeam.rankHistory} maxRank={sortedTeams.length} />
                                    </div>

                                    <div className="flex flex-col items-end shrink-0 mt-auto">
                                        <span className="text-[110px] font-mono font-black text-blue-400 leading-none drop-shadow-[0_0_30px_rgba(59,130,246,0.3)]">
                                            {currentTeam.total}
                                        </span>
                                        <span className="text-2xl text-white/30 font-bold tracking-widest font-sans mt-2">
                                            TOTAL PTS
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            <div className="flex-none h-[10vh] bg-blue-950/20 border-t border-blue-500/20 flex items-center overflow-hidden backdrop-blur-md relative z-20">
                <div className="h-full bg-blue-600 px-8 flex items-center justify-center relative z-30 shadow-[10px_0_20px_rgba(0,0,0,0.5)]">
                    <span className="font-black text-white uppercase tracking-[0.3em] text-lg whitespace-nowrap">
                        Live Standings
                    </span>
                    <div className="absolute right-[-14px] top-1/2 -translate-y-1/2 w-0 h-0 border-t-[15px] border-t-transparent border-l-[15px] border-l-blue-600 border-b-[15px] border-b-transparent"></div>
                </div>

                <div className="flex-1 overflow-hidden h-full flex items-center relative mask-image-fade">
                    <motion.div
                        className="flex whitespace-nowrap items-center h-full"
                        animate={{ x: ["0%", "-50%"] }}
                        transition={{ ease: "linear", duration: sortedTeams.length * 3.5, repeat: Infinity }}
                    >
                        {tickerItems.map((team, i) => {
                            // Map the rank back to the displayed item
                            const realTeam = sortedTeams.find(t => t.name === team.name);
                            return (
                                <div key={`${team.name}-${i}`} className="flex items-center gap-6 px-12 border-r border-white/10 h-[50%]">
                                    <span className={`text-2xl font-black ${realTeam.rank === 1 ? 'text-yellow-500' : 'text-white/30'}`}>{realTeam.rank}.</span>
                                    <span className="text-3xl font-bold text-white uppercase tracking-wider">{realTeam.name}</span>
                                    <span className="text-3xl font-mono font-black text-blue-400">[{realTeam.total} PTS]</span>
                                </div>
                            )
                        })}
                    </motion.div>
                </div>
            </div>

        </div>

        <style dangerouslySetInnerHTML={{__html: `
            .mask-image-fade { mask-image: linear-gradient(to right, transparent, black 5%, black 95%, transparent); -webkit-mask-image: linear-gradient(to right, transparent, black 5%, black 95%, transparent); }
        `}} />
    </div>
  );
}