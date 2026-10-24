import { useState, useEffect, useMemo, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeft, ArrowRight, ArrowUp, ArrowDown, Minus as MinusIcon, Trophy, Crown, CheckCircle, HelpCircle, Shuffle, Database, Flame, RefreshCw, AlertTriangle, Info, Loader2, LogIn } from "lucide-react";
import { useQuiz } from "../quiz/context";
import { isPresent, directionText, teamForScores } from "../quiz/teams";
import VenueMap from "./VenueMap";
import TeamAvatar from "./TeamAvatar";
import { fmt } from "../quiz/files";
import { SceneBackdrop } from "./Theme";
import Celebration from "./Celebration";
import { useTheme } from "../quiz/themes";

// The scores the reveal works on, without the poll time, so equal scores compare equal
const toSnapshot = (scores) => ({ teams: scores?.teams || [], rounds: scores?.rounds || [] });
const sameScores = (a, b) => JSON.stringify(a) === JSON.stringify(b);
// The latest round anyone has points in
const defaultRound = (snap) => [...snap.rounds].reverse().find(r => snap.teams.some(t => t.scores[r] > 0)) ?? snap.rounds[0] ?? "";

// preview: { step } renders one fixed frame of the reveal (the quiz map): no keys and no sync.
// celebrate: the TV window celebrates the final standings with the theme's celebration.
export default function Leaderboard({ isPresenter, data, availableRounds, lastSync, onReload, authNeeded, onSignIn, onClose, t, preview = null, celebrate = false }) {
  const tr = t;
  const { quiz, teams } = useQuiz();
  const role = isPresenter ? 'presenter' : 'main';
  const isPreview = preview !== null;
  const live = useMemo(() => toSnapshot({ teams: data, rounds: availableRounds }), [data, availableRounds]);

  // The reveal works on a snapshot of the scores, shared by both windows so they show the same groups.
  // Until the reveal starts it follows the live scores; later only Reload scores changes it.
  const [snapshot, setSnapshot] = useState(live);
  const [selectedRound, setSelectedRound] = useState(() => defaultRound(live));
  const [stepState, setRevealStep] = useState(0);
  const [roundChosen, setRoundChosen] = useState(false); // picked in the presenter window, so new scores keep it
  // Main window: whether the presenter window is open (it then shows the controls)
  const [presenterOpen, setPresenterOpen] = useState(false);
  const [reloading, setReloading] = useState(false);
  const [reloadFailed, setReloadFailed] = useState(false);

  const adopt = (next) => {
      setSnapshot(next);
      if (!roundChosen || !next.rounds.includes(selectedRound)) setSelectedRound(defaultRound(next));
  };

  // Before the reveal starts, new scores are taken in by one window (the presenter when it is open),
  // which shares them with the other
  const [seenLive, setSeenLive] = useState(live);
  if (live !== seenLive) {
      setSeenLive(live);
      if ((isPresenter || !presenterOpen) && stepState === 0 && !sameScores(live, snapshot)) adopt(live);
  }

  const reload = async () => {
      setReloading(true);
      const fresh = onReload ? await onReload().catch(() => null) : null;
      setReloadFailed(!fresh);
      adopt(fresh ? toSnapshot(fresh) : live);
      setRevealStep(0);
      setReloading(false);
  };

  // --- 2-WAY SYNC (see QuestionScreen) ---
  // Only local changes are broadcast; lastSyncedRef holds the state at mount or last received, so a
  // window joining mid-reveal adopts the other's state instead of resetting it.
  const channel = useMemo(() => (isPreview ? null : new BroadcastChannel('quiz-leaderboard-sync')), [isPreview]);
  const syncKey = JSON.stringify([snapshot, stepState, selectedRound]);
  const lastSyncedRef = useRef(syncKey);
  // The current state for replies to the other window (declared before the effects that read it)
  const stateRef = useRef({ snapshot, revealStep: stepState, selectedRound });
  useEffect(() => { stateRef.current = { snapshot, revealStep: stepState, selectedRound }; }, [snapshot, stepState, selectedRound]);

  useEffect(() => {
    if (!channel) return;
    channel.postMessage({ type: 'LB_REQUEST', sender: role });
    // Closing the presenter window hands the controls back to the main window
    const leave = () => channel.postMessage({ type: 'LB_LEAVE', sender: role });
    if (isPresenter) window.addEventListener('pagehide', leave);
    return () => window.removeEventListener('pagehide', leave);
  }, [channel, role, isPresenter]);

  useEffect(() => {
    if (!channel || syncKey === lastSyncedRef.current) return;
    lastSyncedRef.current = syncKey;
    channel.postMessage({ type: 'LB_UPDATE', sender: role, payload: stateRef.current });
  }, [syncKey, role, channel]);

  useEffect(() => {
    if (!channel) return;
    const apply = (payload) => {
        lastSyncedRef.current = JSON.stringify([payload.snapshot, payload.revealStep, payload.selectedRound]);
        setSnapshot(payload.snapshot);
        setRevealStep(payload.revealStep);
        setSelectedRound(payload.selectedRound);
    };
    const handleSync = (e) => {
        const { type, payload, sender } = e.data || {};
        if (!sender || sender === role) return;
        if (sender === 'presenter') setPresenterOpen(type !== 'LB_LEAVE');
        if (type === 'LB_UPDATE') apply(payload);
        else if (type === 'LB_REQUEST') channel.postMessage({ type: 'LB_STATE', sender: role, payload: stateRef.current });
        else if (type === 'LB_STATE') {
            // A reply to this window's request. The presenter keeps its own scores unless the main
            // window is already revealing; the main window always takes the presenter's.
            if (!isPresenter || payload.revealStep > 0) apply(payload);
            else channel.postMessage({ type: 'LB_UPDATE', sender: role, payload: stateRef.current });
        }
    };
    channel.addEventListener('message', handleSync);
    return () => channel.removeEventListener('message', handleSync);
  }, [channel, role, isPresenter]);

  const frozenData = snapshot.teams;
  const frozenRounds = snapshot.rounds;

  const { initialTeams, scoreGroups } = useMemo(() => {
      if (!selectedRound || !frozenRounds.includes(selectedRound)) return { initialTeams: [], scoreGroups: [] };

      const roundIndex = frozenRounds.indexOf(selectedRound);
      const includedRounds = frozenRounds.slice(0, roundIndex + 1);
      const previousRounds = frozenRounds.slice(0, roundIndex);
      const prevRoundName = roundIndex > 0 ? frozenRounds[roundIndex - 1] : null;

      const enriched = frozenData.map(team => {
          const teamMeta = teamForScores(teams, team) || {};

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
              initials: teamMeta.initials,
              quote: teamMeta.quote || ""
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
  }, [frozenData, frozenRounds, selectedRound, teams]);

  const maxStep = scoreGroups.length * 2 + 1;
  const revealStep = isPreview ? Math.min(preview.step, maxStep) : stepState;
  const isShuffled = revealStep === maxStep;
  // The theme's layout for the scores: the table, racing bars, or a podium for the final standings
  const layout = useTheme().scenes?.leaderboard?.layout || "table";

  // Reaching the final standings (stepping forward onto them, in this window or the other) celebrates
  // once on the TV; stepping back stops it
  const [celebrating, setCelebrating] = useState(false);
  const [seenStep, setSeenStep] = useState(stepState);
  if (seenStep !== stepState) {
      setSeenStep(stepState);
      if (stepState === maxStep && stepState > seenStep && celebrate && !isPresenter && !isPreview && frozenData.length > 0) setCelebrating(true);
      else if (stepState < maxStep) setCelebrating(false);
  }
  const endCelebration = useCallback(() => setCelebrating(false), []);
  const maxPointsThisRound = scoreGroups.length > 0 ? scoreGroups[scoreGroups.length - 1].points : 0;

  useEffect(() => {
    if (isPreview) return;
    const handleKeyDown = (e) => {
      // Either window can step; the change is shared with the other
      if (e.key === "ArrowRight") setRevealStep(prev => Math.min(prev + 1, maxStep));
      if (e.key === "ArrowLeft") setRevealStep(prev => Math.max(prev - 1, 0));
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [maxStep, onClose, isPreview]);

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

      // Shared rank for equal scores: a team ranks right after the last team with a higher score
      const shownScore = (team) => (isShuffled ? team.currentScore : team.previousTotal);
      const ranks = [];
      active.forEach((team, idx) => ranks.push(idx > 0 && shownScore(team) >= shownScore(active[idx - 1]) ? ranks[idx - 1] : idx + 1));

      return active.map((team, idx) => {
          const displayRank = ranks[idx];
          const rankChange = team.previousRank - displayRank;

          const deltaLeaderRaw = leaderScore - team.currentScore;
          const deltaLeader = displayRank === 1 ? t.lb_leader : `-${deltaLeaderRaw.toFixed(1).replace('.0', '')}`;
          const deltaNextRaw = displayRank === 1 ? 0 : active[idx - 1].currentScore - team.currentScore;
          const deltaNext = displayRank === 1 ? "" : `(-${deltaNextRaw.toFixed(1).replace('.0', '')})`;

          return { ...team, displayRank, rankChange, deltaLeader, deltaNext };
      });
  }, [initialTeams, revealStep, isShuffled, t.lb_leader]);

  // UI rendering remains exactly the same below...
  if (isPresenter) {
      const activeGroupIndex = Math.floor((revealStep) / 2);

      // Where to look: the group being revealed now, or the leader(s) once all groups are applied
      const metaFor = (row) => teamForScores(teams, row);
      const seated = teams.filter(team => team.seat && isPresent(team));
      const topTotal = Math.max(0, ...initialTeams.map(team => team.total));
      const focusRows = revealStep >= maxStep - 1
          ? initialTeams.filter(team => team.total === topTotal)
          : (scoreGroups[activeGroupIndex]?.teams || []);
      const focusIds = new Set(focusRows.map(row => metaFor(row)?.id).filter(Boolean));
      const isCardStep = revealStep % 2 === 1 && revealStep < maxStep;

      let nextActionText = tr.lb_start;
      if (revealStep === maxStep) nextActionText = tr.lb_done;
      else if (revealStep === maxStep - 1) nextActionText = tr.lb_shuffle;
      else if (isCardStep) nextActionText = fmt(tr.lb_apply, { points: scoreGroups[activeGroupIndex].points });
      else nextActionText = fmt(tr.lb_reveal_group, { points: scoreGroups[activeGroupIndex]?.points });

      const hasScores = frozenData.length > 0;
      const scoredCount = frozenData.filter(team => (team.scores[selectedRound] || 0) > 0).length;
      // New points arrived after the reveal started; they are only taken in by Reload scores
      const scoresChanged = revealStep > 0 && live.teams.length > 0 && !sameScores(live, snapshot);

      return (
          <div className="h-full w-full bg-panel text-white p-10 font-sans flex flex-col relative">
              <div className="flex justify-between items-start border-b border-white/10 pb-6 mb-8">
                  <div>
                      <h1 className="text-3xl font-black uppercase tracking-widest text-yellow-500 mb-2">{tr.lb_controls}</h1>
                      <div className="text-white/40 font-mono mb-4">
                          {tr.lb_step}: <span className="text-white">{revealStep} / {maxStep}</span>
                          {hasScores && revealStep < maxStep && <span className="ml-3 font-sans">{fmt(tr.lb_steps_left, { n: maxStep - revealStep })}</span>}
                      </div>

                      <div className="flex items-center gap-4 bg-primary-900/20 border border-primary-500/30 px-4 py-3 rounded-xl w-fit">
                          <Database size={24} className="text-primary-500"/>
                          <div className="flex flex-col">
                              <span className="text-primary-400 font-bold uppercase text-[10px] tracking-widest">{tr.lb_select_round}</span>
                              <select
                                  value={selectedRound}
                                  onChange={(e) => { setRoundChosen(true); setSelectedRound(e.target.value); setRevealStep(0); }}
                                  className="bg-transparent text-white font-black text-lg outline-none cursor-pointer"
                              >
                                  {frozenRounds.map(r => <option key={r} value={r} className="bg-gray-900">{r}</option>)}
                              </select>
                          </div>
                          <div className="h-8 w-px bg-white/10 mx-2" />
                          <div className="flex flex-col">
                              <span className="text-primary-400 font-bold uppercase text-[10px] tracking-widest">{tr.lb_last_sync}</span>
                              <span className="font-mono text-sm text-white/80">{lastSync || "—"}</span>
                          </div>
                      </div>
                  </div>
                  <div className="flex gap-3">
                      <button onClick={reload} disabled={reloading} title={tr.lb_reload_note}
                          className="flex items-center gap-2 px-5 py-3 bg-white/10 hover:bg-white/20 disabled:opacity-50 rounded-xl transition-all font-bold">
                          {reloading ? <Loader2 size={18} className="animate-spin" /> : <RefreshCw size={18} />} {tr.lb_reload}
                      </button>
                      <button onClick={onClose} className="px-6 py-3 bg-red-900/50 hover:bg-red-800/80 rounded-xl transition-all font-bold">{tr.lb_exit}</button>
                  </div>
              </div>

              {authNeeded && (
                  <div className="flex flex-wrap items-center gap-4 mb-6 -mt-2 p-4 rounded-xl bg-amber-500/10 border border-amber-500/40">
                      <p className="flex-1 min-w-[280px] flex items-start gap-2 text-amber-200"><AlertTriangle size={18} className="shrink-0 mt-0.5" /> {tr.lb_signin_needed}</p>
                      <button onClick={onSignIn} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-amber-500 hover:bg-amber-400 text-black font-bold">
                          <LogIn size={18} /> {tr.lb_signin}
                      </button>
                  </div>
              )}

              {(reloadFailed || scoresChanged || (hasScores && scoredCount < frozenData.length)) && (
                  <div className="space-y-2 mb-6 -mt-2">
                      {reloadFailed && (
                          <p className="flex items-start gap-2 text-red-300"><AlertTriangle size={18} className="shrink-0 mt-0.5" /> {tr.lb_reload_failed}</p>
                      )}
                      {scoresChanged && (
                          <p className="flex items-start gap-2 text-amber-300"><AlertTriangle size={18} className="shrink-0 mt-0.5" /> {tr.lb_scores_changed}</p>
                      )}
                      {hasScores && scoredCount < frozenData.length && (
                          <p className="flex items-start gap-2 text-yellow-300">
                              <AlertTriangle size={18} className="shrink-0 mt-0.5" /> {fmt(tr.lb_scored_count, { n: scoredCount, total: frozenData.length, round: selectedRound })}
                          </p>
                      )}
                  </div>
              )}

              {!hasScores ? (
                  <div className="flex-1 flex flex-col items-center justify-center gap-4 text-center bg-white/5 rounded-2xl border border-white/10 p-10">
                      <Trophy size={48} className="text-white/20" />
                      <h2 className="text-2xl font-black">{tr.lb_empty_title}</h2>
                      <p className="text-white/60 max-w-xl">{authNeeded ? tr.lb_signin_needed : quiz.scoresSheetUrl ? tr.lb_empty_sheet : tr.lb_empty_manual}</p>
                  </div>
              ) : (<>
              <div className="flex gap-6 mb-3">
                  <button onClick={() => setRevealStep(p => Math.max(p - 1, 0))} disabled={revealStep === 0} className="px-8 py-6 bg-white/5 hover:bg-white/10 disabled:opacity-20 rounded-2xl font-bold flex flex-col items-center justify-center transition-all border border-white/10">
                      <ArrowLeft size={32} className="mb-2"/> {tr.lb_prev_step}
                  </button>

                  <button onClick={() => setRevealStep(p => Math.min(p + 1, maxStep))} disabled={revealStep === maxStep} className={`flex-1 flex flex-col items-center justify-center py-6 rounded-2xl font-black text-2xl tracking-widest uppercase transition-all shadow-xl ${revealStep === maxStep ? "bg-white/5 text-white/20" : revealStep === maxStep - 1 ? "bg-green-600 hover:bg-green-500 text-white shadow-[0_0_40px_rgba(74,222,128,0.4)]" : "bg-primary-600 hover:bg-primary-500 text-white shadow-[0_0_30px_rgb(var(--glow-primary)/0.3)]"}`}>
                      {revealStep === maxStep - 1 && <Shuffle size={28} className="mb-2" />}
                      {nextActionText}
                  </button>
              </div>
              <p className="flex items-start gap-2 text-sm text-white/40 mb-8">
                  <Info size={16} className="shrink-0 mt-0.5" /> {isShuffled ? tr.lb_done_note : tr.lb_order_note}
              </p>

              <div className="flex-1 min-h-0 flex flex-col gap-6">
              {/* Where to look: the wide seating map across the full width, above the reveal order */}
              {seated.length > 0 && (
                  <div className="h-[24vh] min-h-[170px] shrink-0 bg-white/5 rounded-2xl border border-white/10 p-4 flex gap-4">
                      <div className="flex-1 min-w-0 flex flex-col">
                          <h3 className="text-sm font-bold text-white/30 uppercase tracking-widest">{tr.att_look}</h3>
                          <VenueMap arc={quiz.settings?.venueArc || 180} teams={seated} highlight={focusIds} youLabel={tr.att_you} fill markerScale={1.8} className="flex-1 min-h-0" />
                      </div>
                      <ul className="w-72 shrink-0 space-y-2 self-center overflow-y-auto max-h-full">
                          {focusRows.map(row => (
                              <li key={row.id || row.name} className="flex justify-between items-baseline gap-3">
                                  <span className="text-lg font-black truncate">{row.name}</span>
                                  <span className="text-lg font-bold text-yellow-300 whitespace-nowrap">{directionText(metaFor(row)?.seat, tr) || "—"}</span>
                              </li>
                          ))}
                      </ul>
                  </div>
              )}
              <div className="flex-1 min-h-0 bg-white/5 rounded-2xl border border-white/10 overflow-y-auto p-6 custom-scrollbar">
                  <h3 className="text-sm font-bold text-white/30 uppercase tracking-widest mb-6">{tr.lb_order}</h3>
                  {scoreGroups.map((group, gIdx) => (
                      <div key={gIdx} className={`mb-6 p-4 rounded-xl border ${revealStep >= (gIdx * 2) + 2 ? 'bg-primary-900/30 border-primary-500/30' : 'bg-black/40 border-white/10'}`}>
                          <h4 className="text-lg font-black text-primary-400 mb-3 flex items-center gap-2">
                              {revealStep >= (gIdx * 2) + 2 && <CheckCircle size={18} className="text-primary-500"/>}
                              {fmt(tr.lb_group, { points: group.points })}
                          </h4>
                          <div className="flex flex-wrap gap-3">
                              {group.teams.map(t => (
                                  <div key={t.name} className="flex items-center gap-1 bg-white/10 px-3 py-1.5 rounded text-sm font-bold border border-white/5">
                                      {t.name}
                                      {metaFor(t)?.seat && <span className="ml-1 text-xs font-normal text-yellow-300/80">{directionText(metaFor(t).seat, tr)}</span>}
                                      {t.isHighestRound && <Crown size={14} className="text-yellow-500 ml-1" />}
                                      {t.isOnFire && <Flame size={14} className="text-orange-500" fill="currentColor" />}
                                  </div>
                              ))}
                          </div>
                      </div>
                  ))}
                  <div className={`p-4 rounded-xl border ${isShuffled ? 'bg-green-900/30 border-green-500/50' : 'bg-black/40 border-white/10'}`}>
                      <h4 className={`text-lg font-black flex items-center gap-2 ${isShuffled ? 'text-green-400' : 'text-white/40'}`}>
                          <Shuffle size={18}/> {tr.lb_final_step}
                      </h4>
                  </div>
              </div>

              </div>
              </>)}
          </div>
      )
  }

  const isCardOverlayActive = revealStep % 2 === 1 && revealStep < maxStep;
  const activeGroupForOverlay = isCardOverlayActive ? scoreGroups[Math.floor(revealStep / 2)] : null;

  return (
    <div className="h-full w-full bg-ground relative overflow-hidden flex flex-col font-show selection:bg-primary-500/30">
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

        <SceneBackdrop quiet />
        {celebrating && <Celebration onDone={endCelebration} />}
        <div className="absolute top-0 right-0 w-[1200px] h-[1200px] bg-primary-900/10 blur-[250px] rounded-full pointer-events-none" />
        <div className="absolute bottom-0 left-0 w-[800px] h-[800px] bg-accent-900/5 blur-[200px] rounded-full pointer-events-none" />

        <div className="relative z-10 flex flex-col h-full w-full max-w-[1920px] mx-auto pt-[6cqh] pb-[6cqh] px-12">
            <div className="flex items-center justify-between border-b border-white/10 pb-6 mb-8 shrink-0">
                <div className="flex items-center gap-6">
                    <Trophy size={48} className="text-yellow-500" />
                    <div>
                        <h1 className="font-title text-5xl font-black uppercase tracking-[0.2em] text-white leading-none">{t.lb_title}</h1>
                        <h2 className="text-primary-400 font-bold tracking-[0.4em] text-sm uppercase mt-2">
                            {isShuffled ? t.lb_final : fmt(t.lb_round_reveal, { round: selectedRound })}
                        </h2>
                    </div>
                </div>

                <div className="flex flex-col items-end">
                    <span className="text-white/40 font-bold tracking-widest uppercase text-xs mb-2">{t.lb_progress}</span>
                    <div className="flex gap-1.5">
                        {scoreGroups.map((_, i) => (
                            <div key={i} className={`w-6 h-3 rounded-full transition-colors duration-500 ${revealStep >= (i * 2) + 2 ? 'bg-primary-500 shadow-[0_0_10px_rgb(var(--glow-primary)/0.5)]' : 'bg-white/10'}`} />
                        ))}
                        <div className={`w-8 h-3 rounded-full ml-2 transition-colors duration-500 ${isShuffled ? 'bg-green-500 shadow-[0_0_15px_rgba(74,222,128,0.8)]' : 'bg-white/10'}`} />
                    </div>
                </div>
            </div>

            <div className="flex-1 w-full overflow-hidden relative">
                {frozenData.length === 0 && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center gap-6 text-center">
                        <Trophy size={96} className="text-white/10" />
                        <h2 className="text-5xl font-black uppercase tracking-[0.2em] text-white/60">{t.lb_empty_title}</h2>
                        {authNeeded && (
                            <button onClick={onSignIn} className="flex items-center gap-3 px-8 py-4 rounded-2xl bg-amber-500 hover:bg-amber-400 text-black text-2xl font-bold font-sans">
                                <LogIn size={28} /> {t.lb_signin}
                            </button>
                        )}
                    </div>
                )}
                {layout === "race" ? <RaceView teams={activeTeamsList} isShuffled={isShuffled} t={t} />
                  : layout === "podium" && isShuffled ? <PodiumView teams={activeTeamsList} />
                  : (
                <div className="grid grid-cols-2 grid-flow-row gap-x-16 gap-y-5 w-full content-start">

                    {activeTeamsList.map((team) => {
                        const isWinner = isShuffled && team.displayRank === 1;
                        const showFire = team.isOnFire && team.isRevealed;

                        return (
                            <motion.div
                                layout
                                transition={{ type: "spring", bounce: 0.25, duration: 1.2 }}
                                key={team.name}
                                className={`relative flex items-stretch h-[80px] rounded-r-2xl overflow-hidden shadow-lg transition-colors duration-700 ${
                                    isWinner
                                        ? "bg-black/80 border border-yellow-500/80 shadow-[0_0_30px_rgba(234,179,8,0.2)]"
                                        : team.isRevealed
                                            ? "bg-primary-950/20 border border-primary-500/20"
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
                                    <TeamAvatar team={team} className="w-10 h-10 rounded-full border border-white/20" textClass="text-sm" />
                                    <span className={`font-black text-xl tracking-widest uppercase truncate ${isWinner ? 'text-yellow-400' : showFire ? 'text-orange-400 drop-shadow-[0_0_8px_rgba(249,115,22,0.8)]' : 'text-white'} ${!team.isRevealed && 'opacity-50'}`}>
                                        {team.name}
                                    </span>
                                    {showFire && <Flame size={20} className="text-orange-500 animate-pulse drop-shadow-[0_0_10px_rgba(249,115,22,1)] ml-1 shrink-0" fill="currentColor" />}
                                </div>

                                <div className={`w-[110px] flex items-center justify-center px-2 border-l border-white/5 shrink-0 relative z-20 ${team.isRevealed ? "bg-green-500/10 text-green-400" : "bg-white/[0.02] text-white/20"}`}>
                                    {team.isRevealed ? (
                                        <motion.span initial={{ scale: 0 }} animate={{ scale: 1 }} className="text-base font-bold tracking-widest relative">
                                            {team.isHighestRound && <Crown size={12} className="absolute -top-1 -right-4 text-yellow-500 rotate-12" />}
                                            +{team.roundPoints} <span className="text-[10px] text-green-500/50">{t.lb_round_short}</span>
                                        </motion.span>
                                    ) : (
                                        <HelpCircle size={16} className="opacity-30" />
                                    )}
                                </div>

                                <div className={`w-[100px] flex items-center justify-end px-4 border-l border-white/5 shrink-0 relative z-20 ${team.isRevealed && !isShuffled ? "bg-primary-900/20" : "bg-white/[0.02]"}`}>
                                    <span className={`font-mono font-black text-3xl transition-colors duration-500 ${isWinner ? "text-yellow-500" : team.isRevealed ? "text-primary-400" : "text-white/40"}`}>
                                        {team.currentScore}
                                    </span>
                                </div>

                                <div className="w-[120px] flex items-center justify-end px-4 bg-white/5 border-l border-white/10 shrink-0 relative z-20">
                                    {isShuffled ? (
                                        team.displayRank === 1 ? (
                                            <span className="font-black text-yellow-500 text-sm tracking-widest uppercase animate-[popIn_0.5s_ease-out]">{t.lb_leader}</span>
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
                )}
            </div>

            {/* Run from this window (no presenter window): how to step, until the reveal starts */}
            <AnimatePresence>
                {!isPreview && !presenterOpen && revealStep === 0 && frozenData.length > 0 && (
                    <motion.p initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                        className="absolute bottom-[3.5cqh] inset-x-0 text-center text-white/40 font-sans text-xl tracking-wide">
                        {t.lb_keys_hint}
                    </motion.p>
                )}
            </AnimatePresence>

            <AnimatePresence>
                {isCardOverlayActive && activeGroupForOverlay && (
                    <motion.div
                        initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.3 }}
                        className="absolute inset-0 z-50 bg-black/80 backdrop-blur-md flex flex-col items-center justify-center p-12"
                    >
                        <motion.h2
                            initial={{ scale: 0.8, y: 30 }} animate={{ scale: 1, y: 0 }} transition={{ type: "spring", bounce: 0.4 }}
                            className="font-title text-7xl font-black uppercase tracking-[0.2em] text-primary-400 drop-shadow-[0_0_40px_rgb(var(--glow-primary)/0.5)] mb-16"
                        >
                            {fmt(t.lb_teams_with, { points: activeGroupForOverlay.points })}
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

                                        <TeamAvatar team={team} className="w-32 h-32 rounded-full border-4 border-white/20 mb-6 relative z-10" textClass="text-5xl" />

                                        <h3 className="text-2xl font-black text-white uppercase tracking-wider text-center relative z-10 leading-tight mb-2">{team.name}</h3>
                                        {team.quote && <p className="text-sm text-white/50 italic text-center relative z-10">"{team.quote}"</p>}
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
// Racing bars: each team's bar grows with its points as the groups are revealed; the order changes
// only at the final step, like the table. Two columns when there are many teams.
function RaceView({ teams, isShuffled, t }) {
  // Bars start from a baseline a little below the lowest score, so close scores still differ visibly
  const top = Math.max(1, ...teams.map((team) => team.total));
  const low = Math.min(top, ...teams.map((team) => team.previousTotal));
  const base = Math.max(0, low - (top - low) * 0.25);
  const width = (score) => Math.max(3, ((score - base) / Math.max(1, top - base)) * 100);
  const columns = teams.length > 12 ? 2 : 1;
  const rows = Math.ceil(teams.length / columns) || 1;
  const height = Math.min(7.5, 68 / rows - 0.8);
  return (
    <div className="grid gap-x-12 gap-y-[0.8cqh] content-start" style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}>
      {teams.map((team) => {
        const isWinner = isShuffled && team.displayRank === 1;
        const nameCls = `truncate font-black uppercase tracking-wide ${isWinner ? "text-yellow-400" : "text-white"} ${team.isRevealed ? "" : "opacity-60"}`;
        const scoreCls = `shrink-0 text-right font-mono font-black ${isWinner ? "text-yellow-400" : team.isRevealed ? "text-primary-300" : "text-white/40"}`;
        const bar = (
          <div className={`relative rounded-full bg-white/5 overflow-hidden ${columns > 1 ? "h-[1.7cqh]" : "flex-1 h-[62%]"}`}>
            <div className={`absolute inset-y-0 left-0 rounded-full transition-[width] duration-1000 ease-out ${isWinner ? "bg-gradient-to-r from-yellow-600 to-yellow-300" : team.isRevealed ? "bg-gradient-to-r from-primary-700 to-primary-400" : "bg-white/15"}`}
              style={{ width: `${width(team.currentScore)}%` }} />
            {team.isRevealed && !isShuffled && columns === 1 && (
              <span className="absolute right-[1cqw] inset-y-0 flex items-center font-bold text-[1.6cqh] text-green-400">+{team.roundPoints} {t.lb_round_short}</span>
            )}
          </div>
        );
        return (
          <motion.div layout transition={{ type: "spring", bounce: 0.25, duration: 1.2 }} key={team.name}
            className="flex items-center gap-[1.2cqw]" style={{ height: `${height}cqh` }}>
            <span className={`w-[3cqw] shrink-0 text-center font-mono font-black text-[2.4cqh] ${isWinner ? "text-yellow-400" : "text-white/70"}`}>{team.displayRank}</span>
            <TeamAvatar team={team} className="h-[80%] aspect-square rounded-full border border-white/20 shrink-0" textClass="text-[1.4cqh]" />
            {columns > 1 ? (
              // Two columns: the name and score above a full-width bar
              <div className="flex-1 min-w-0 flex flex-col justify-center gap-[0.5cqh]">
                <div className="flex items-baseline gap-3">
                  <span className={`flex-1 text-[1.9cqh] ${nameCls}`}>{team.name}</span>
                  {team.isRevealed && !isShuffled && <span className="shrink-0 font-bold text-[1.5cqh] text-green-400">+{team.roundPoints}</span>}
                  <span className={`text-[2.3cqh] ${scoreCls}`}>{team.currentScore}</span>
                </div>
                {bar}
              </div>
            ) : (
              <>
                <span className={`w-[26%] shrink-0 text-[2.2cqh] ${nameCls}`}>{team.name}</span>
                {bar}
                <span className={`w-[6cqw] text-[2.6cqh] ${scoreCls}`}>{team.currentScore}</span>
              </>
            )}
          </motion.div>
        );
      })}
    </div>
  );
}

// The final standings on a podium: the top three rise onto gold, silver and bronze blocks (second,
// first, third from the left), everyone else in a compact list below
function PodiumView({ teams }) {
  const [first, second, third] = teams;
  const rest = teams.slice(3);
  const block = (team, height, color, delay) => team && (
    <motion.div key={team.name} initial={{ y: "35%", opacity: 0 }} animate={{ y: 0, opacity: 1 }} transition={{ delay, type: "spring", bounce: 0.3, duration: 1.1 }}
      className="flex flex-col items-center justify-end w-[22%] min-w-0">
      <TeamAvatar team={team} className="w-[11cqh] h-[11cqh] rounded-full border-4 border-white/30 mb-[1.2cqh] shadow-2xl" textClass="text-[3.5cqh]" />
      <div className="w-full text-center truncate font-black uppercase tracking-wider text-[2.6cqh] text-white">{team.name}</div>
      <div className="font-mono font-black text-[3.4cqh] text-primary-300 mb-[1cqh]">{team.currentScore}</div>
      <div className={`w-full rounded-t-2xl flex items-start justify-center pt-[1.2cqh] shadow-[0_0_40px_rgba(0,0,0,0.5)] ${color}`} style={{ height }}>
        <span className="font-title font-black text-[6cqh] leading-none text-black/60">{team.displayRank}</span>
      </div>
    </motion.div>
  );
  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 min-h-0 flex items-end justify-center gap-[2cqw]">
        {block(second, "18cqh", "bg-gradient-to-b from-slate-200 to-slate-400", 0.6)}
        {block(first, "25cqh", "bg-gradient-to-b from-yellow-300 to-yellow-500", 1.2)}
        {block(third, "12cqh", "bg-gradient-to-b from-orange-300 to-orange-600", 0)}
      </div>
      {rest.length > 0 && (
        <div className="grid grid-cols-3 gap-x-6 gap-y-[0.8cqh] mt-[2.5cqh] shrink-0">
          {rest.map((team, i) => (
            <motion.div key={team.name} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 1.6 + i * 0.04 }}
              className="flex items-center gap-3 h-[4cqh] px-3 rounded-lg bg-black/40 border border-white/10">
              <span className="w-[2.5cqw] font-mono font-black text-[1.8cqh] text-white/60">{team.displayRank}</span>
              <TeamAvatar team={team} className="h-[70%] aspect-square rounded-full border border-white/20 shrink-0" textClass="text-[1.1cqh]" />
              <span className="flex-1 truncate font-bold uppercase tracking-wider text-[1.7cqh] text-white">{team.name}</span>
              <span className="font-mono font-black text-[2cqh] text-primary-300">{team.currentScore}</span>
            </motion.div>
          ))}
        </div>
      )}
    </div>
  );
}
