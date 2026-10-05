import { useState, useEffect, useMemo, useRef } from "react";
import { AnimatePresence } from "framer-motion";
import { Settings, ArrowLeft, Calculator, ClipboardCheck } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv } from "../quiz/parse";
import { resolveTeams } from "../quiz/model";
import { updateQuiz } from "../quiz/storage";
import { useScores } from "../hooks/useScores";
import { quizHash, navigate } from "../hooks/useHashRoute";

import WelcomeScreen from "./WelcomeScreen";
import Dashboard from "./Dashboard";
import QuestionScreen from "./QuestionScreen";
import Leaderboard from "./Leaderboard";
import SettingsModal from "./SettingsModal";
import RulesScreen from "./RulesScreen";
import PrizesScreen from "./PrizesScreen";
import PauseScreen from "./PauseScreen";
import Stage from "./Stage";
import { useDisplaySettings } from "../hooks/useDisplay";
import { CalibrationPattern, CalibrationControls } from "./CalibrationScreen";
import FullscreenHint from "./FullscreenHint";

const SCENES = {
  WELCOME: "welcome",
  RULES: "rules",
  PRIZES: "prizes",
  DASHBOARD: "dashboard",
  GAME: "game",
  LEADERBOARD: "leaderboard",
  PAUSE: "pause",
};

// bundle comes from loadQuizBundle()
export default function QuizShow({ bundle, isPresenter }) {
  const { quiz, resolveMedia } = bundle;

  const [scene, setScene] = useState(SCENES.WELCOME);
  const [activeRoundId, setActiveRoundId] = useState(null);
  const [playMode, setPlayMode] = useState("with_answers");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isCalibrating, setIsCalibrating] = useState(false);
  const [display, setDisplay] = useDisplaySettings(isPresenter ? "presenter" : "main");

  const [config, setConfig] = useState(quiz.settings);
  const updateConfig = (next) => {
    setConfig(next);
    // The built-in demo is read-only; its settings only last for this session
    if (!quiz.builtin) updateQuiz(quiz.id, (q) => ({ ...q, settings: next }));
  };

  const t = TRANSLATIONS[config.language] || TRANSLATIONS.en;
  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const liveLeaderboard = useScores(quiz, rounds);
  const teams = useMemo(() => resolveTeams(quiz.teams, resolveMedia), [quiz.teams, resolveMedia]);
  const quizContext = useMemo(() => ({ quiz, rounds, teams, resolveMedia }), [quiz, rounds, teams, resolveMedia]);

  const activeRound = useMemo(() => rounds.find(r => r.id === activeRoundId) || null, [rounds, activeRoundId]);

  // Only local changes are broadcast (see QuestionScreen): lastSyncedRef holds the state at mount or last received
  const appChannel = useMemo(() => new BroadcastChannel('quiz-app-sync'), []);
  const syncKey = JSON.stringify([scene, activeRoundId, playMode]);
  const lastSyncedRef = useRef(syncKey);

  useEffect(() => {
    const handleGlobalKey = (e) => {
       if (e.key.toLowerCase() === 'p' && e.shiftKey) window.open(window.location.origin + window.location.pathname + quizHash(quiz.id, 'presenter'), 'PresenterWindow', 'width=1200,height=800');
       if (e.key.toLowerCase() === 'c' && e.shiftKey) setIsCalibrating(c => !c);
       if (e.key.toLowerCase() === 'd' && e.shiftKey) setScene(SCENES.DASHBOARD);
    };
    window.addEventListener('keydown', handleGlobalKey);
    return () => window.removeEventListener('keydown', handleGlobalKey);
  }, [quiz.id]);

  useEffect(() => { appChannel.postMessage({ type: 'REQUEST_STATE', sender: isPresenter ? 'presenter' : 'main' }); }, [appChannel, isPresenter]);

  useEffect(() => {
    if (syncKey === lastSyncedRef.current) return;
    lastSyncedRef.current = syncKey;
    appChannel.postMessage({ type: 'STATE_UPDATE', sender: isPresenter ? 'presenter' : 'main', payload: { scene, activeRoundId, playMode } });
  }, [syncKey, scene, activeRoundId, playMode, isPresenter, appChannel]);

  useEffect(() => {
    const handleAppSync = (e) => {
      const { type, payload, sender } = e.data;
      const myRole = isPresenter ? 'presenter' : 'main';
      if (sender === myRole) return;

      if (type === 'STATE_UPDATE') {
          lastSyncedRef.current = JSON.stringify([payload.scene, payload.activeRoundId, payload.playMode]);
          setScene(payload.scene); setActiveRoundId(payload.activeRoundId); setPlayMode(payload.playMode);
      } else if (type === 'REQUEST_STATE') {
          appChannel.postMessage({ type: 'STATE_UPDATE', sender: myRole, payload: { scene, activeRoundId, playMode } });
      }
    };
    appChannel.addEventListener('message', handleAppSync);
    return () => appChannel.removeEventListener('message', handleAppSync);
  }, [scene, activeRoundId, playMode, isPresenter, appChannel]);

  const startRound = (round, mode) => { setActiveRoundId(round.id); setPlayMode(mode); setScene(SCENES.GAME); };

  return (
    <QuizContext.Provider value={quizContext}>
    {/* Presenter control panels use the full window; everything else renders on the scaled stage */}
    <Stage display={display} bypass={isPresenter && !isCalibrating && [SCENES.GAME, SCENES.LEADERBOARD, SCENES.PAUSE].includes(scene)}>
    <div className="bg-[#050505] h-full w-full font-['League_Spartan'] overflow-hidden relative selection:bg-yellow-500/30 text-white">
      {!isPresenter && (
          <div className="absolute top-0 w-full p-6 flex justify-between z-50 pointer-events-none">
            <div className="pointer-events-auto flex gap-2">
              {(scene === SCENES.GAME || scene === SCENES.LEADERBOARD || scene === SCENES.PAUSE) && (
                <button onClick={() => setScene(SCENES.DASHBOARD)} className="p-3 bg-white/10 border border-white/10 text-white rounded-full hover:bg-white hover:text-black transition-all"><ArrowLeft /></button>
              )}
            </div>
            <div className="pointer-events-auto flex gap-4">
              <button onClick={() => setIsSettingsOpen(true)} className="p-3 bg-white/10 border border-white/10 text-white rounded-full hover:bg-white hover:text-black hover:rotate-90 transition-all"><Settings /></button>
            </div>
          </div>
      )}

      <AnimatePresence mode="wait">
        {scene === SCENES.WELCOME && <WelcomeScreen key="welcome" onStart={() => setScene(SCENES.RULES)} startTime={config.startTime} showTime={config.showTime} config={config} t={t} />}
        {scene === SCENES.RULES && <RulesScreen key="rules" onNext={() => setScene(SCENES.PRIZES)} t={t} />}
        {scene === SCENES.PRIZES && <PrizesScreen key="prizes" onNext={() => setScene(SCENES.DASHBOARD)} t={t} />}

        {scene === SCENES.DASHBOARD && (
          <Dashboard key="dashboard" rounds={rounds} onSelectRound={startRound} onOpenLeaderboard={() => setScene(SCENES.LEADERBOARD)} onOpenPause={() => setScene(SCENES.PAUSE)} t={t} />
        )}

        {scene === SCENES.LEADERBOARD && (
          <Leaderboard
              key="leaderboard"
              data={liveLeaderboard.teams}
              availableRounds={liveLeaderboard.rounds}
              lastSync={liveLeaderboard.lastSync}
              isPresenter={isPresenter}
              onClose={() => setScene(SCENES.DASHBOARD)}
              t={t}
          />
        )}

        {scene === SCENES.PAUSE && (
          <PauseScreen key="pause" isPresenter={isPresenter} leaderboardData={liveLeaderboard.teams} availableRounds={liveLeaderboard.rounds} onBack={() => setScene(SCENES.DASHBOARD)} t={t} />
        )}

        {scene === SCENES.GAME && activeRound && (
          <QuestionScreen key="game" roundData={activeRound} mode={playMode} isPresenter={isPresenter} onBack={() => setScene(SCENES.DASHBOARD)} t={t} />
        )}
      </AnimatePresence>

      {!isPresenter && (
          <div className="absolute bottom-0 w-full h-10 bg-black/50 border-t border-white/5 flex items-center justify-between px-6 text-gray-500 text-xs z-40 pointer-events-none backdrop-blur-sm">
            <span className="tracking-widest">Quiz Night OS v2.0</span>
            {config.showTime && <span className="font-mono text-yellow-500/50">{t.start_show}: {config.startTime}</span>}
          </div>
      )}

      {isCalibrating && <CalibrationPattern t={t} />}
    </div>
    </Stage>

    {/* Operator overlays stay outside the stage so they are never scaled */}
    <AnimatePresence>
      {isSettingsOpen && (
        <SettingsModal
          config={config}
          onUpdate={updateConfig}
          onClose={() => setIsSettingsOpen(false)}
          onOpenCalibration={() => { setIsSettingsOpen(false); setIsCalibrating(true); }}
          onExit={() => navigate("#/")}
          t={t}
        />
      )}
    </AnimatePresence>
    {isCalibrating && <CalibrationControls display={display} onUpdate={setDisplay} onClose={() => setIsCalibrating(false)} t={t} />}
    {!isPresenter && !isCalibrating && <FullscreenHint t={t} />}
    {/* Host tools in the presenter window, each opened in its own window */}
    {isPresenter && !quiz.builtin && (
      <div className="fixed bottom-4 left-4 z-[150] flex gap-2 font-sans">
        <button
          onClick={() => window.open(window.location.origin + window.location.pathname + quizHash(quiz.id, 'attendance'), 'AttendanceWindow', 'width=1200,height=800')}
          className="flex items-center gap-2 px-4 py-3 rounded-xl bg-white/10 hover:bg-white/20 text-white font-bold shadow-xl backdrop-blur">
          <ClipboardCheck size={18} /> {t.att_title}
        </button>
        {!quiz.scoresSheetUrl && (
          <button
            onClick={() => window.open(window.location.origin + window.location.pathname + quizHash(quiz.id, 'scores'), 'ScoresWindow', 'width=1100,height=750')}
            className="flex items-center gap-2 px-4 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold shadow-xl">
            <Calculator size={18} /> {t.sc_title}
          </button>
        )}
      </div>
    )}
    </QuizContext.Provider>
  );
}
