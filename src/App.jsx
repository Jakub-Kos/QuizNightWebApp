import { useState, useEffect, useMemo, useRef } from "react";
import { useQuizData } from "./hooks/useQuizData";
import { AnimatePresence } from "framer-motion";
import { Settings, ArrowLeft } from "lucide-react";
import { TRANSLATIONS } from "./data/translations";

import WelcomeScreen from "./components/WelcomeScreen";
import Dashboard from "./components/Dashboard";
import QuestionScreen from "./components/QuestionScreen";
import Leaderboard from "./components/Leaderboard";
import SettingsModal from "./components/SettingsModal";
import RulesScreen from "./components/RulesScreen";
import PrizesScreen from "./components/PrizesScreen";
import PauseScreen from "./components/PauseScreen";

const SCENES = {
  WELCOME: "welcome",
  RULES: "rules",
  PRIZES: "prizes",
  DASHBOARD: "dashboard",
  GAME: "game",
  LEADERBOARD: "leaderboard",
  PAUSE: "pause",
};

// Robust CSV Row Parser
function parseCSVRow(str) {
    const result = [];
    let cur = '';
    let inQuote = false;
    for (let i = 0; i < str.length; i++) {
        if (str[i] === '"') {
            inQuote = !inQuote;
        } else if (str[i] === ',' && !inQuote) {
            result.push(cur.trim());
            cur = '';
        } else {
            cur += str[i];
        }
    }
    result.push(cur.trim());
    return result.map(s => s.replace(/^"|"$/g, '').trim());
}

export default function App() {
  const { data: rawData, loading } = useQuizData("/questions.csv");

  const urlParams = new URLSearchParams(window.location.search);
  const isPresenter = urlParams.get("presenter") === "true";

  const [scene, setScene] = useState(SCENES.WELCOME);
  const [activeRoundId, setActiveRoundId] = useState(null);
  const [playMode, setPlayMode] = useState("with_answers");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const [liveLeaderboard, setLiveLeaderboard] = useState({
      teams: [],
      rounds: [],
      lastSync: null
  });

  const [config, setConfig] = useState({
    showTime: true,
    startTime: "20:00",
    splitDelay: 3000,
    cycleDuration: 8000,
    uiScale: 1.0,
    headerInterval: 5000,
    language: "cs"
  });

  const t = TRANSLATIONS[config.language] || TRANSLATIONS.en;

  // --- GOOGLE SHEETS LIVE FETCHING ---
  useEffect(() => {
      const fetchLeaderboard = async () => {
          try {
              // CACHE BUSTER: Prevents Google Sheets from serving deleted 5-minute-old data
              const timestamp = new Date().getTime();
              const res = await fetch(`https://docs.google.com/spreadsheets/d/e/2PACX-1vTPi2K8Mi8kFMlROt4F5V0TtPY2f1RLRJvgjN1TGiQelOLja0bFBu6zk_lOzlQk6K3QvOr-PLRQZvkn/pub?gid=728177247&single=true&output=csv&_cb=${timestamp}`, { cache: "no-store" });
              const text = await res.text();

              const rows = text.split('\n').map(row => parseCSVRow(row));
              if (rows.length < 2) return;

              let headerIdx = -1;
              for(let i=0; i < rows.length; i++) {
                  if (rows[i].some(col => col.includes("Název týmu"))) {
                      headerIdx = i; break;
                  }
              }
              if (headerIdx === -1) return;

              const headers = rows[headerIdx];
              const teamNameIdx = headers.findIndex(h => h.includes("Název týmu"));

              const roundColumns = [];
              headers.forEach((h, i) => {
                  if (h.toLowerCase().includes("kolo")) roundColumns.push({ name: h, index: i });
              });

              const parsedLeaderboard = [];
              for (let i = headerIdx + 1; i < rows.length; i++) {
                  const row = rows[i];
                  const name = row[teamNameIdx];
                  if (!name) continue; // Your "to be announced" team will now show correctly!

                  const scores = {};
                  roundColumns.forEach(rc => {
                      const valStr = row[rc.index] ? row[rc.index].replace(',', '.') : "0";
                      scores[rc.name] = parseFloat(valStr) || 0;
                  });

                  parsedLeaderboard.push({ name, scores });
              }

              setLiveLeaderboard({
                  teams: parsedLeaderboard,
                  rounds: roundColumns.map(rc => rc.name),
                  lastSync: new Date().toLocaleTimeString()
              });
          } catch (err) {
              console.error("Failed to fetch live leaderboard", err);
          }
      };

      fetchLeaderboard();
      const interval = setInterval(fetchLeaderboard, 10000);
      return () => clearInterval(interval);
  }, []);

  const rounds = useMemo(() => {
    if (!rawData || rawData.length === 0) return [];
    const parsedRounds = [];
    let currentRound = null;

    rawData.forEach((row) => {
      const col0 = row[0] ? String(row[0]).trim() : "";
      const col1 = row[1] ? String(row[1]).trim() : "";

      if (col0.toLowerCase().startsWith("kolo") && !col0.toLowerCase().includes("q#")) {
        if (currentRound) parsedRounds.push(currentRound);
        const parts = col0.split("-");
        currentRound = { id: `round-${parsedRounds.length + 1}`, number: parts[0].replace(/kolo/i, "").trim(), title: parts.slice(1).join("-").trim(), questions: [] };
      }

      if (currentRound && col1 && !isNaN(col1)) {
        const hasText = row[2] && String(row[2]).trim() !== "";
        const hasMedia = row[9] && String(row[9]).trim() !== "";
        if (hasText || hasMedia) currentRound.questions.push(row);
      }
    });
    if (currentRound) parsedRounds.push(currentRound);
    return parsedRounds;
  }, [rawData]);

  const activeRound = useMemo(() => rounds.find(r => r.id === activeRoundId) || null, [rounds, activeRoundId]);

  const appChannel = useMemo(() => new BroadcastChannel('quiz-app-sync'), []);
  const isReceivingRef = useRef(false);
  const isInitialMount = useRef(true);

  useEffect(() => {
    const handleGlobalKey = (e) => {
       if (e.key.toLowerCase() === 'p' && e.shiftKey) window.open(window.location.origin + '?presenter=true', 'PresenterWindow', 'width=1200,height=800');
       if (e.key.toLowerCase() === 'd' && e.shiftKey) setScene(SCENES.DASHBOARD);
    };
    window.addEventListener('keydown', handleGlobalKey);
    return () => window.removeEventListener('keydown', handleGlobalKey);
  }, []);

  useEffect(() => { appChannel.postMessage({ type: 'REQUEST_STATE', sender: isPresenter ? 'presenter' : 'main' }); }, [appChannel, isPresenter]);

  useEffect(() => {
    if (isInitialMount.current) { isInitialMount.current = false; return; }
    if (isReceivingRef.current) { isReceivingRef.current = false; return; }
    appChannel.postMessage({ type: 'STATE_UPDATE', sender: isPresenter ? 'presenter' : 'main', payload: { scene, activeRoundId, playMode } });
  }, [scene, activeRoundId, playMode, isPresenter, appChannel]);

  useEffect(() => {
    const handleAppSync = (e) => {
      const { type, payload, sender } = e.data;
      const myRole = isPresenter ? 'presenter' : 'main';
      if (sender === myRole) return;

      if (type === 'STATE_UPDATE') {
          isReceivingRef.current = true;
          setScene(payload.scene); setActiveRoundId(payload.activeRoundId); setPlayMode(payload.playMode);
      } else if (type === 'REQUEST_STATE') {
          appChannel.postMessage({ type: 'STATE_UPDATE', sender: myRole, payload: { scene, activeRoundId, playMode } });
      }
    };
    appChannel.addEventListener('message', handleAppSync);
    return () => appChannel.removeEventListener('message', handleAppSync);
  }, [scene, activeRoundId, playMode, isPresenter, appChannel]);

  const startRound = (round, mode) => { setActiveRoundId(round.id); setPlayMode(mode); setScene(SCENES.GAME); };

  if (loading) return (
    <div className="bg-[#050505] text-white h-screen flex flex-col items-center justify-center font-['League_Spartan']">
        <div className="w-16 h-16 border-4 border-t-blue-500 border-white/10 rounded-full animate-spin mb-4" />
        <span className="tracking-[0.2em] uppercase text-sm opacity-50">Loading System...</span>
    </div>
  );

  return (
    <div className="bg-[#050505] min-h-screen font-['League_Spartan'] overflow-hidden relative selection:bg-yellow-500/30 text-white">
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

      <AnimatePresence>
        {isSettingsOpen && <SettingsModal config={config} onUpdate={setConfig} onClose={() => setIsSettingsOpen(false)} t={t} />}
      </AnimatePresence>

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
    </div>
  );
}