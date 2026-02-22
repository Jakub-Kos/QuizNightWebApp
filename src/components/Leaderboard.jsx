import { useState, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeft, ArrowRight, ArrowUp, ArrowDown, Minus as MinusIcon, Trophy, Crown, CheckCircle, HelpCircle, Shuffle, Database, Flame } from "lucide-react";
import { TEAMS_DATA } from "../data/teams";

export default function Leaderboard({ isPresenter, data, availableRounds, latestRoundName, lastSync, onClose, t }) {
  const [frozenData] = useState(() => data || []);
  const [frozenRounds] = useState(() => availableRounds || []);

  const [selectedRound, setSelectedRound] = useState("");
  const [revealStep, setRevealStep] = useState(0);

  useEffect(() => {
      if (!selectedRound && frozenRounds.length > 0 && frozenData.length > 0) {
          let latest = frozenRounds[0];
          for (let i = frozenRounds.length - 1; i >= 0; i--) {
              const rName = frozenRounds[i];
              if (frozenData.some(t => t.scores[rName] > 0)) {
                  latest = rName;
                  break;
              }
          }
          setSelectedRound(latest);
      }
  }, [frozenRounds, frozenData, selectedRound]);

  const channel = useMemo(() => new BroadcastChannel('quiz-leaderboard-sync'), []);

  useEffect(() => {
    if (isPresenter) channel.postMessage({ type: 'SYNC_REVEAL', revealStep, selectedRound });
  }, [revealStep, selectedRound, isPresenter, channel]);

  useEffect(() => {
    if (!isPresenter) {
      const handleSync = (e) => {
          if (e.data.type === 'SYNC_REVEAL') {
              setRevealStep(e.data.revealStep);
              setSelectedRound(e.data.selectedRound);
          }
      };
      channel.addEventListener('message', handleSync);
      return () => channel.removeEventListener('message', handleSync);
    }
  }, [isPresenter, channel]);

  const { initialTeams, scoreGroups } = useMemo(() => {
      if (!selectedRound || !frozenRounds.includes(selectedRound)) return { initialTeams: [], scoreGroups: [] };

      const roundIndex = frozenRounds.indexOf(selectedRound);
      const includedRounds = frozenRounds.slice(0, roundIndex + 1);
      const previousRounds = frozenRounds.slice(0, roundIndex);
      const prevRoundName = roundIndex > 0 ? frozenRounds[roundIndex - 1] : null;

      const enriched = frozenData.map(team => {
          const teamMeta = TEAMS_DATA.find(t => t.name.toLowerCase() === team.name.toLowerCase()) || {};

          const total = includedRounds.reduce((sum, r) => sum + (team.scores[r] || 0), 0);
          const previousTotal = previousRounds.reduce((sum, r) => sum + (team.scores[r] || 0), 0);
          const roundPoints = team.scores[selectedRound] || 0;
          const prevRoundPoints = prevRoundName ? (team.scores[prevRoundName] || 0) : 0;

          return {
              ...team,
              total,
              previousTotal,
              roundPoints,
              prevRoundPoints,
              color: teamMeta.color || "from-gray-500 to-gray-700",
              image: teamMeta.image,
              IconComponent: teamMeta.icon,
              quote: teamMeta.quote || "Waiting for scores..."
          };
      });

      const maxRoundPts = Math.max(0, ...enriched.map(t => t.roundPoints));
      const maxPrevPts = Math.max(0, ...enriched.map(t => t.prevRoundPoints));

      const sortedPrev = [...enriched].sort((a, b) => b.previousTotal - a.previousTotal);

      // NEW: Standard Competition Ranking for PREVIOUS Rank
      let currentPrevRank = 1;
      const withPrevRank = sortedPrev.map((t, idx) => {
          if (idx > 0 && t.previousTotal < sortedPrev[idx - 1].previousTotal) {
              currentPrevRank = idx + 1;
          }

          const isHighestRound = t.roundPoints === maxRoundPts && maxRoundPts > 0;
          const wasHighestPrev = t.prevRoundPoints === maxPrevPts && maxPrevPts > 0;
          const isOnFire = isHighestRound && wasHighestPrev;

          return { ...t, previousRank: currentPrevRank, isHighestRound, isOnFire };
      });

      const uniqueScores = Array.from(new Set(withPrevRank.map(t => t.roundPoints))).sort((a, b) => a - b);
      const groups = uniqueScores.map(score => ({
          points: score,
          teams: withPrevRank.filter(t => t.roundPoints === score)
      }));

      const finalTeams = withPrevRank.map(t => ({ ...t, groupIndex: uniqueScores.indexOf(t.roundPoints) }));
      return { initialTeams: finalTeams, scoreGroups: groups };
  }, [frozenData, frozenRounds, selectedRound]);

  const maxStep = scoreGroups.length * 2 + 1;
  const isShuffled = revealStep === maxStep;
  const maxPointsThisRound = scoreGroups.length > 0 ? scoreGroups[scoreGroups.length - 1].points : 0;

  useEffect(() => {
    const handleKeyDown = (e) => {
      if (isPresenter || (!isPresenter && window.name !== 'PresenterWindow')) {
          if (e.key === "ArrowRight") setRevealStep(prev => Math.min(prev + 1, maxStep));
          if (e.key === "ArrowLeft") setRevealStep(prev => Math.max(prev - 1, 0));
          if (e.key === "Escape") onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [revealStep, maxStep, isPresenter, onClose]);

  // ==========================================
  // CALCULATE CURRENT TABLE STATE
  // ==========================================
  const activeTeamsList = useMemo(() => {
      const active = initialTeams.map(team => {
          const isRevealed = revealStep >= (team.groupIndex * 2) + 2;
          return { ...team, currentScore: isRevealed ? team.total : team.previousTotal, isRevealed };
      });

      if (isShuffled) active.sort((a, b) => b.currentScore - a.currentScore);
      else active.sort((a, b) => b.previousTotal - a.previousTotal);

      const leaderScore = active.length > 0 ? active[0].currentScore : 0;

      let currentDisplayRank = 1;

      return active.map((team, idx) => {
          if (idx > 0) {
              const prevScore = isShuffled ? active[idx - 1].currentScore : active[idx - 1].previousTotal;
              const myScore = isShuffled ? team.currentScore : team.previousTotal;
              if (myScore < prevScore) {
                  currentDisplayRank = idx + 1;
              }
          }

          const displayRank = currentDisplayRank;
          const rankChange = team.previousRank - displayRank;

          const deltaLeaderRaw = leaderScore - team.currentScore;
          const deltaLeader = displayRank === 1 ? "LEADER" : `-${deltaLeaderRaw.toFixed(1).replace('.0', '')}`;
          const deltaNextRaw = displayRank === 1 ? 0 : active[idx - 1].currentScore - team.currentScore;
          const deltaNext = displayRank === 1 ? "" : `(-${deltaNextRaw.toFixed(1).replace('.0', '')})`;

          return { ...team, displayRank, rankChange, deltaLeader, deltaNext };
      });
  }, [initialTeams, scoreGroups, revealStep, isShuffled]);

  // UI rendering remains exactly the same below...
  if (isPresenter) {
      const activeGroupIndex = Math.floor((revealStep) / 2);
      const isCardStep = revealStep % 2 === 1 && revealStep < maxStep;

      let nextActionText = "START REVEAL";
      if (revealStep === maxStep) nextActionText = "REVEAL FINISHED";
      else if (revealStep === maxStep - 1) nextActionText = "SHUFFLE STANDINGS!";
      else if (isCardStep) nextActionText = `APPLY ${scoreGroups[activeGroupIndex].points} PTS TO TABLE`;
      else nextActionText = `REVEAL TEAMS WITH ${scoreGroups[activeGroupIndex]?.points} PTS`;

      return (
          <div className="h-screen w-full bg-[#0a0a0a] text-white p-10 font-sans flex flex-col relative">
              <div className="flex justify-between items-start border-b border-white/10 pb-6 mb-8">
                  <div>
                      <h1 className="text-3xl font-black uppercase tracking-widest text-yellow-500 mb-2">Eurovision Reveal Controls</h1>
                      <div className="text-white/40 font-mono mb-4">Step: <span className="text-white">{revealStep} / {maxStep}</span></div>

                      <div className="flex items-center gap-4 bg-blue-900/20 border border-blue-500/30 px-4 py-3 rounded-xl w-fit">
                          <Database size={24} className="text-blue-500"/>
                          <div className="flex flex-col">
                              <span className="text-blue-400 font-bold uppercase text-[10px] tracking-widest">Select Target Round</span>
                              <select
                                  value={selectedRound}
                                  onChange={(e) => { setSelectedRound(e.target.value); setRevealStep(0); }}
                                  className="bg-transparent text-white font-black text-lg outline-none cursor-pointer"
                              >
                                  {frozenRounds.map(r => <option key={r} value={r} className="bg-gray-900">{r}</option>)}
                              </select>
                          </div>
                          <div className="h-8 w-px bg-white/10 mx-2" />
                          <div className="flex flex-col">
                              <span className="text-blue-400 font-bold uppercase text-[10px] tracking-widest">Last Synced</span>
                              <span className="font-mono text-sm text-white/80">{lastSync || "Unknown"}</span>
                          </div>
                      </div>
                  </div>
                  <button onClick={onClose} className="px-6 py-3 bg-red-900/50 hover:bg-red-800/80 rounded-xl transition-all font-bold">EXIT LEADERBOARD</button>
              </div>

              <div className="flex gap-6 mb-8">
                  <button onClick={() => setRevealStep(p => Math.max(p - 1, 0))} disabled={revealStep === 0} className="px-8 py-6 bg-white/5 hover:bg-white/10 disabled:opacity-20 rounded-2xl font-bold flex flex-col items-center justify-center transition-all border border-white/10">
                      <ArrowLeft size={32} className="mb-2"/> Previous Step
                  </button>

                  <button onClick={() => setRevealStep(p => Math.min(p + 1, maxStep))} disabled={revealStep === maxStep} className={`flex-1 flex flex-col items-center justify-center py-6 rounded-2xl font-black text-2xl tracking-widest uppercase transition-all shadow-xl ${revealStep === maxStep ? "bg-white/5 text-white/20" : revealStep === maxStep - 1 ? "bg-green-600 hover:bg-green-500 text-white shadow-[0_0_40px_rgba(74,222,128,0.4)]" : "bg-blue-600 hover:bg-blue-500 text-white shadow-[0_0_30px_rgba(59,130,246,0.3)]"}`}>
                      {revealStep === maxStep - 1 && <Shuffle size={28} className="mb-2" />}
                      {nextActionText}
                  </button>
              </div>

              <div className="flex-1 bg-white/5 rounded-2xl border border-white/10 overflow-y-auto p-6 custom-scrollbar">
                  <h3 className="text-sm font-bold text-white/30 uppercase tracking-widest mb-6">Reveal Order (Lowest to Highest)</h3>
                  {scoreGroups.map((group, gIdx) => (
                      <div key={gIdx} className={`mb-6 p-4 rounded-xl border ${revealStep >= (gIdx * 2) + 2 ? 'bg-blue-900/30 border-blue-500/30' : 'bg-black/40 border-white/10'}`}>
                          <h4 className="text-lg font-black text-blue-400 mb-3 flex items-center gap-2">
                              {revealStep >= (gIdx * 2) + 2 && <CheckCircle size={18} className="text-blue-500"/>}
                              Groups Scoring {group.points} Pts
                          </h4>
                          <div className="flex flex-wrap gap-3">
                              {group.teams.map(t => (
                                  <div key={t.name} className="flex items-center gap-1 bg-white/10 px-3 py-1.5 rounded text-sm font-bold border border-white/5">
                                      {t.name}
                                      {t.isHighestRound && <Crown size={14} className="text-yellow-500 ml-1" />}
                                      {t.isOnFire && <Flame size={14} className="text-orange-500" fill="currentColor" />}
                                  </div>
                              ))}
                          </div>
                      </div>
                  ))}
                  <div className={`p-4 rounded-xl border ${isShuffled ? 'bg-green-900/30 border-green-500/50' : 'bg-black/40 border-white/10'}`}>
                      <h4 className={`text-lg font-black flex items-center gap-2 ${isShuffled ? 'text-green-400' : 'text-white/40'}`}>
                          <Shuffle size={18}/> FINAL STEP: Shuffle Standings
                      </h4>
                  </div>
              </div>
          </div>
      )
  }

  const isCardOverlayActive = revealStep % 2 === 1 && revealStep < maxStep;
  const activeGroupForOverlay = isCardOverlayActive ? scoreGroups[Math.floor(revealStep / 2)] : null;

  return (
    <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col font-['League_Spartan'] selection:bg-blue-500/30">
        <style dangerouslySetInnerHTML={{__html: `
            @keyframes popIn { 0% { transform: scale(0); opacity: 0; } 80% { transform: scale(1.2); opacity: 1; } 100% { transform: scale(1); opacity: 1; } }
            @keyframes fireFlicker {
                0% { opacity: 0.5; transform: scaleY(1); }
                50% { opacity: 0.9; transform: scaleY(1.05); }
                100% { opacity: 0.6; transform: scaleY(1.02); }
            }
            .fire-overlay {
                background: linear-gradient(0deg, rgba(239,68,68,0.4) 0%, rgba(249,115,22,0.1) 60%, transparent 100%);
                animation: fireFlicker 1s infinite alternate ease-in-out;
            }
        `}} />

        <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-[#000] z-0 pointer-events-none" />
        <div className="absolute top-0 right-0 w-[1200px] h-[1200px] bg-blue-900/10 blur-[250px] rounded-full pointer-events-none" />
        <div className="absolute bottom-0 left-0 w-[800px] h-[800px] bg-yellow-900/5 blur-[200px] rounded-full pointer-events-none" />

        <div className="relative z-10 flex flex-col h-full w-full max-w-[1920px] mx-auto pt-[6vh] pb-[6vh] px-12">
            <div className="flex items-center justify-between border-b border-white/10 pb-6 mb-8 shrink-0">
                <div className="flex items-center gap-6">
                    <Trophy size={48} className="text-yellow-500" />
                    <div>
                        <h1 className="text-5xl font-black uppercase tracking-[0.2em] text-white leading-none">Global Standings</h1>
                        <h2 className="text-blue-400 font-bold tracking-[0.4em] text-sm uppercase mt-2">
                            {isShuffled ? "Official Final Results" : `${selectedRound} Reveal`}
                        </h2>
                    </div>
                </div>

                <div className="flex flex-col items-end">
                    <span className="text-white/40 font-bold tracking-widest uppercase text-xs mb-2">Reveal Progress</span>
                    <div className="flex gap-1.5">
                        {scoreGroups.map((_, i) => (
                            <div key={i} className={`w-6 h-3 rounded-full transition-colors duration-500 ${revealStep >= (i * 2) + 2 ? 'bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]' : 'bg-white/10'}`} />
                        ))}
                        <div className={`w-8 h-3 rounded-full ml-2 transition-colors duration-500 ${isShuffled ? 'bg-green-500 shadow-[0_0_15px_rgba(74,222,128,0.8)]' : 'bg-white/10'}`} />
                    </div>
                </div>
            </div>

            <div className="flex-1 w-full overflow-hidden relative">
                <div className="grid grid-cols-2 grid-flow-row gap-x-12 gap-y-2.5 w-full content-start">

                    {activeTeamsList.map((team) => {
                        const isWinner = isShuffled && team.displayRank === 1;
                        const showFire = team.isOnFire && team.isRevealed;

                        return (
                            <motion.div
                                layout
                                transition={{ type: "spring", bounce: 0.25, duration: 1.2 }}
                                key={team.name}
                                className={`relative flex items-stretch h-[60px] rounded-r-xl overflow-hidden shadow-lg transition-colors duration-700 ${
                                    isWinner
                                        ? "bg-black/80 border border-yellow-500/80 shadow-[0_0_30px_rgba(234,179,8,0.2)]"
                                        : team.isRevealed
                                            ? "bg-blue-950/20 border border-blue-500/20"
                                            : "bg-black/40 border border-white/10"
                                }`}
                            >
                                {isWinner && <div className="absolute inset-0 border-2 border-yellow-500/80 rounded-r-xl pointer-events-none z-20" />}
                                {showFire && <div className="absolute inset-0 fire-overlay border-b-2 border-orange-500 rounded-r-xl pointer-events-none z-10" />}

                                <div className={`w-[60px] flex items-center justify-center font-mono font-black text-2xl border-r border-white/5 shrink-0 transition-colors relative z-20 ${isWinner ? "bg-yellow-500 text-black" : "bg-white/10 text-white/80"}`}>
                                    {team.displayRank}
                                </div>

                                <div className="w-[50px] flex items-center justify-center bg-white/[0.02] border-r border-white/5 shrink-0 relative z-20">
                                    {isShuffled ? (
                                        team.rankChange > 0 ? (
                                            <div className="flex flex-col items-center text-green-500 animate-[popIn_0.5s_ease-out]"><ArrowUp size={16} strokeWidth={4} /><span className="text-[10px] font-black leading-none">{team.rankChange}</span></div>
                                        ) : team.rankChange < 0 ? (
                                            <div className="flex flex-col items-center text-red-500 animate-[popIn_0.5s_ease-out]"><ArrowDown size={16} strokeWidth={4} /><span className="text-[10px] font-black leading-none">{Math.abs(team.rankChange)}</span></div>
                                        ) : (
                                            <MinusIcon size={16} className="text-white/20 animate-[popIn_0.5s_ease-out]" strokeWidth={3} />
                                        )
                                    ) : (
                                        <div className="w-1.5 h-1.5 rounded-full bg-white/10 animate-pulse" />
                                    )}
                                </div>

                                <div className={`w-[6px] shrink-0 bg-gradient-to-b ${team.color} ${team.isRevealed ? "opacity-100" : "opacity-30"} relative z-20`} />

                                <div className="flex-1 flex items-center px-4 overflow-hidden gap-4 relative z-20">
                                    <div className="w-10 h-10 rounded-full overflow-hidden shrink-0 border border-white/20 bg-white/5 flex items-center justify-center">
                                        {team.image ? <img src={team.image} className="w-full h-full object-cover" /> : team.IconComponent ? <team.IconComponent size={20} className="text-white/50" /> : <span className="text-lg font-black text-white/50">{team.name.charAt(0)}</span>}
                                    </div>
                                    <span className={`font-black text-xl tracking-widest uppercase truncate ${isWinner ? 'text-yellow-400' : showFire ? 'text-orange-400 drop-shadow-[0_0_8px_rgba(249,115,22,0.8)]' : 'text-white'} ${!team.isRevealed && 'opacity-50'}`}>
                                        {team.name}
                                    </span>
                                    {showFire && <Flame size={20} className="text-orange-500 animate-pulse drop-shadow-[0_0_10px_rgba(249,115,22,1)] ml-1 shrink-0" fill="currentColor" />}
                                </div>

                                <div className={`w-[110px] flex items-center justify-center px-2 border-l border-white/5 shrink-0 relative z-20 ${team.isRevealed ? "bg-green-500/10 text-green-400" : "bg-white/[0.02] text-white/20"}`}>
                                    {team.isRevealed ? (
                                        <motion.span initial={{ scale: 0 }} animate={{ scale: 1 }} className="text-base font-bold tracking-widest relative">
                                            {team.isHighestRound && <Crown size={12} className="absolute -top-1 -right-4 text-yellow-500 rotate-12" />}
                                            +{team.roundPoints} <span className="text-[10px] text-green-500/50">RND</span>
                                        </motion.span>
                                    ) : (
                                        <HelpCircle size={16} className="opacity-30" />
                                    )}
                                </div>

                                <div className={`w-[100px] flex items-center justify-end px-4 border-l border-white/5 shrink-0 relative z-20 ${team.isRevealed && !isShuffled ? "bg-blue-900/20" : "bg-white/[0.02]"}`}>
                                    <span className={`font-mono font-black text-3xl transition-colors duration-500 ${isWinner ? "text-yellow-500" : team.isRevealed ? "text-blue-400" : "text-white/40"}`}>
                                        {team.currentScore}
                                    </span>
                                </div>

                                <div className="w-[120px] flex items-center justify-end px-4 bg-white/5 border-l border-white/10 shrink-0 relative z-20">
                                    {isShuffled ? (
                                        team.displayRank === 1 ? (
                                            <span className="font-black text-yellow-500 text-sm tracking-widest uppercase animate-[popIn_0.5s_ease-out]">LEADER</span>
                                        ) : (
                                            <div className="flex flex-col items-end justify-center animate-[popIn_0.5s_ease-out]">
                                                <span className="font-mono font-bold text-white/80 text-base leading-none tracking-tighter">{team.deltaLeader}</span>
                                                <span className="font-mono font-bold text-white/40 text-[10px] leading-none mt-1 tracking-tighter">{team.deltaNext}</span>
                                            </div>
                                        )
                                    ) : (
                                        <div className="w-8 h-1 bg-white/10 rounded-full" />
                                    )}
                                </div>
                            </motion.div>
                        )
                    })}
                </div>
            </div>

            <AnimatePresence>
                {isCardOverlayActive && activeGroupForOverlay && (
                    <motion.div
                        initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.3 }}
                        className="absolute inset-0 z-50 bg-black/80 backdrop-blur-md flex flex-col items-center justify-center p-12"
                    >
                        <motion.h2
                            initial={{ scale: 0.8, y: 30 }} animate={{ scale: 1, y: 0 }} transition={{ type: "spring", bounce: 0.4 }}
                            className="text-7xl font-black uppercase tracking-[0.2em] text-blue-400 drop-shadow-[0_0_40px_rgba(59,130,246,0.5)] mb-16"
                        >
                            Teams Earning {activeGroupForOverlay.points} Points
                        </motion.h2>

                        <div className="flex flex-wrap justify-center gap-8 max-w-[1600px]">
                            {activeGroupForOverlay.teams.map((team, idx) => {
                                const isAbsoluteMax = activeGroupForOverlay.points === maxPointsThisRound && maxPointsThisRound > 0;

                                return (
                                    <motion.div
                                        key={team.name}
                                        initial={{ opacity: 0, scale: 0.5, rotateY: 90 }}
                                        animate={{ opacity: 1, scale: 1, rotateY: 0 }}
                                        transition={{ delay: idx * 0.15, type: "spring", bounce: 0.4 }}
                                        className="bg-black border border-white/20 rounded-[3rem] p-10 flex flex-col items-center justify-center shadow-[0_0_50px_rgba(0,0,0,0.8)] relative w-[350px]"
                                    >
                                        <div className={`absolute inset-0 bg-gradient-to-b ${team.color} opacity-20 rounded-[3rem] pointer-events-none`} />

                                        {team.isOnFire && (
                                            <div className="absolute -inset-8 bg-orange-600/30 blur-3xl z-0 animate-pulse rounded-full pointer-events-none" />
                                        )}

                                        {isAbsoluteMax && (
                                            <Crown size={48} className={`absolute -top-6 text-yellow-500 drop-shadow-[0_0_20px_rgba(234,179,8,0.8)] z-20 ${team.isOnFire ? 'right-12' : ''}`} />
                                        )}

                                        {team.isOnFire && (
                                            <Flame size={56} className={`absolute -top-8 text-orange-500 drop-shadow-[0_0_30px_rgba(249,115,22,1)] z-20 animate-bounce ${isAbsoluteMax ? 'left-12' : ''}`} fill="currentColor" />
                                        )}

                                        <div className="w-32 h-32 rounded-full overflow-hidden bg-white/10 border-4 border-white/20 flex items-center justify-center mb-6 relative z-10">
                                            {team.image ? <img src={team.image} className="w-full h-full object-cover" /> : team.IconComponent ? <team.IconComponent size={60} className="text-white/80" /> : <span className="text-5xl font-black text-white/50">{team.name.charAt(0)}</span>}
                                        </div>

                                        <h3 className="text-2xl font-black text-white uppercase tracking-wider text-center relative z-10 leading-tight mb-2">{team.name}</h3>
                                        <p className="text-sm text-white/50 italic text-center relative z-10">"{team.quote}"</p>
                                    </motion.div>
                                )
                            })}
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    </div>
  );
}