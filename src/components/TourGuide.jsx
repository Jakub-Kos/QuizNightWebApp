import { useState, useEffect } from "react";
import { X, Compass, Plus } from "lucide-react";
import { readTour, writeTour, endTour, nextPhase, tourStep, TOUR_STEPS, TOUR_EVENT } from "../quiz/tour";
import { previewStep } from "../quiz/validate";
import { navigate } from "../hooks/useHashRoute";
import { fmt } from "../quiz/files";
import Rich from "./RichText";

// The guided tour's bubble: says what to do next and rings the button to press. It follows the show
// through the scene and the messages the scenes already send to the other window (the question and the
// leaderboard step), so the scenes know nothing about it. Rendered outside the stage, in both windows.
export default function TourGuide({ quizId, scene, activeRound, isPresenter, t }) {
  const [tour, setTour] = useState(readTour);
  const active = tour?.quizId === quizId;
  const [question, setQuestion] = useState({ qIndex: -1, step: 0 });
  const [lbStep, setLbStep] = useState(0);
  const [presenterSeen, setPresenterSeen] = useState(false);
  const presenter = isPresenter || presenterSeen;

  useEffect(() => {
    const sync = () => setTour(readTour());
    window.addEventListener("storage", sync);
    window.addEventListener(TOUR_EVENT, sync);
    return () => { window.removeEventListener("storage", sync); window.removeEventListener(TOUR_EVENT, sync); };
  }, []);

  // Each scene starts from its beginning
  const [seenScene, setSeenScene] = useState(scene);
  if (scene !== seenScene) {
    setSeenScene(scene);
    setQuestion({ qIndex: -1, step: 0 });
    setLbStep(0);
  }

  useEffect(() => {
    if (!active) return;
    const q = new BroadcastChannel("quiz-question-sync");
    const lb = new BroadcastChannel("quiz-leaderboard-sync");
    const app = new BroadcastChannel("quiz-app-sync");
    q.onmessage = (e) => e.data?.type === "Q_UPDATE" && setQuestion({ qIndex: e.data.payload.qIndex, step: e.data.payload.step });
    lb.onmessage = (e) => ["LB_UPDATE", "LB_STATE"].includes(e.data?.type) && setLbStep(e.data.payload.revealStep);
    app.onmessage = (e) => e.data?.sender === "presenter" && setPresenterSeen(true);
    return () => { q.close(); lb.close(); app.close(); };
  }, [active]);

  // The scenes this window has shown since the phase began
  const [visited, setVisited] = useState({ phase: null, scenes: [] });
  if (active && (visited.phase !== tour.phase || !visited.scenes.includes(scene))) {
    setVisited({ phase: tour.phase, scenes: visited.phase === tour.phase ? [...visited.scenes, scene] : [scene] });
  }

  // Moving on once the show reaches the phase's scene (both windows write the same phase)
  useEffect(() => {
    if (!active || visited.phase !== tour.phase) return;
    const next = nextPhase(tour.phase, scene, presenter, visited.scenes);
    if (next !== tour.phase) writeTour({ ...tour, phase: next });
  }, [active, tour, scene, presenter, visited]);

  const { text, target, actions } = active ? message(tour.phase, scene, question, lbStep, activeRound, t) : {};
  const ring = useTargetRect(active ? target : null);

  if (!active) return null;
  const step = tourStep(tour.phase);
  const skipPresenter = () => writeTour({ ...tour, phase: "answers" });

  return (
    <>
      {ring && (
        <div className="fixed z-[290] pointer-events-none rounded-2xl ring-4 ring-yellow-400 animate-pulse shadow-[0_0_40px_rgba(250,204,21,0.6)]"
          style={{ left: ring.left - 8, top: ring.top - 8, width: ring.width + 16, height: ring.height + 16 }} />
      )}
      <div className="fixed z-[300] right-4 bottom-14 w-[min(380px,calc(100vw-32px))] rounded-2xl bg-[#0f0f0f]/95 backdrop-blur border border-yellow-500/50 shadow-2xl p-5 space-y-3 font-sans text-white">
        <div className="flex items-center gap-2">
          <Compass size={18} className="text-yellow-400" />
          <span className="text-xs font-bold uppercase tracking-widest text-yellow-400">{t.tr_title}</span>
          <span className="text-xs text-gray-400">{fmt(t.tr_step, { n: step, total: TOUR_STEPS })}</span>
          <button onClick={endTour} className="ml-auto p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label={t.tr_end} title={t.tr_end}><X size={16} /></button>
        </div>
        <div className="flex gap-1">
          {Array.from({ length: TOUR_STEPS }, (_, i) => <span key={i} className={`h-1 flex-1 rounded-full ${i < step ? "bg-yellow-400" : "bg-white/10"}`} />)}
        </div>
        <p className="text-sm leading-relaxed text-gray-200"><Rich text={text} /></p>
        {actions === "presenter" && (
          <button onClick={skipPresenter} className="text-sm text-gray-400 hover:text-white underline underline-offset-4">{t.tr_presenter_skip}</button>
        )}
        {actions === "done" && (
          <div className="flex flex-wrap gap-2">
            <button onClick={() => { endTour(); navigate("#/"); }} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
              <Plus size={16} /> {t.tr_done_new}
            </button>
            <button onClick={endTour} className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">{t.tr_end}</button>
          </div>
        )}
      </div>
    </>
  );
}

// What the bubble says in this phase and scene, which button it rings ([data-tour] in the scenes) and
// which buttons of its own it shows
function message(phase, scene, question, lbStep, round, t) {
  const back = { text: t.tr_back_to_dashboard };
  if (phase === "done") return { text: t.tr_done, actions: "done" };
  if (phase === "intro") {
    if (scene === "welcome") return { text: t.tr_welcome, target: ["start"] };
    if (scene === "rules") return { text: t.tr_rules, target: ["next"] };
    if (scene === "prizes") return { text: t.tr_prizes, target: ["next"] };
    return back;
  }
  if (phase === "presenter") return scene === "dashboard" ? { text: t.tr_presenter, actions: "presenter" } : back;
  if (["questions", "answers", "leaderboard"].includes(phase)) {
    if (scene !== "dashboard") return back;
    // A round's buttons show once it is pointed at, so until then its card is ringed
    return {
      questions: { text: t.tr_dashboard, target: ["questions", "round"] },
      answers: { text: t.tr_answers, target: ["answers", "round"] },
      leaderboard: { text: t.tr_leaderboard, target: ["leaderboard"] },
    }[phase];
  }
  if (phase === "questions_back" || phase === "answers_back") {
    if (scene !== "game") return back;
    if (question.qIndex < 0) return { text: t.tr_round_card };
    if (phase === "questions_back") return { text: t.tr_questions };
    const q = round?.questions[question.qIndex];
    return { text: q && question.step >= previewStep(q.type, "answer") ? t.tr_answer_shown : t.tr_answer_reveal };
  }
  if (phase === "leaderboard_back") {
    if (scene !== "leaderboard") return back;
    if (lbStep === 0) return { text: t.tr_lb_start };
    return { text: lbStep % 2 ? t.tr_lb_card : t.tr_lb_points };
  }
  return back;
}

// Where the ringed button is on screen (the first of targets found); followed while the scenes animate
// and the stage scales
function useTargetRect(targets) {
  const [rect, setRect] = useState(null);
  const target = targets?.join(",") || null;
  useEffect(() => {
    if (!target) return;
    const measure = () => {
      const el = target.split(",").map((name) => document.querySelector(`[data-tour="${name}"]`)).find(Boolean);
      const r = el?.getBoundingClientRect();
      setRect(r && r.width ? { left: r.left, top: r.top, width: r.width, height: r.height } : null);
    };
    const first = setTimeout(measure);
    const timer = setInterval(measure, 300);
    return () => { clearTimeout(first); clearInterval(timer); };
  }, [target]);
  return target ? rect : null;
}
