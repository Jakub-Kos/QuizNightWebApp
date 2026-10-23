import { useState, useEffect, useMemo, useRef } from "react";
import { MotionConfig } from "framer-motion";
import { ArrowLeft, ArrowRight, CornerDownLeft, Play, X, Keyboard } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv, isRoundHidden } from "../quiz/parse";
import { resolveTeams } from "../quiz/model";
import { rulesFor, prizesFor } from "../quiz/intro";
import { previewStep } from "../quiz/validate";
import { fmt } from "../quiz/files";
import { useQuizBundle } from "../hooks/useQuizBundle";
import { useDisplaySettings } from "../hooks/useDisplay";
import { useScores } from "../hooks/useScores";
import { quizHash } from "../hooks/useHashRoute";
import { StageFrame } from "./Stage";
import Rich from "./RichText";
import ShowLauncher from "./ShowLauncher";
import WelcomeScreen from "./WelcomeScreen";
import RulesScreen from "./RulesScreen";
import PrizesScreen from "./PrizesScreen";
import Dashboard from "./Dashboard";
import QuestionScreen from "./QuestionScreen";
import Leaderboard from "./Leaderboard";
import PauseScreen from "./PauseScreen";

// Questions shown in full at the start of a round's pass; the rest are summed up before the last one
const SHOWN_QUESTIONS = 2;
const noop = () => {};

export default function QuizMap({ id, t }) {
  const state = useQuizBundle(id);
  if (state.status === "ready") return <MapView bundle={state.bundle} t={t} />;
  return (
    <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
      {state.status !== "loading" && (
        <>
          <p className="text-xl text-gray-300">{state.status === "missing" ? t.ed_not_found : state.message}</p>
          <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
        </>
      )}
    </div>
  );
}

// Every screen of the show as a graph: the screens before the quiz in a row, then the dashboard as the
// hub with a branch per round, the leaderboard and half-time. Each line says what moves the show on.
// The thumbnails are the real scenes with this quiz's content (static previews, nothing is synced).
function MapView({ bundle, t }) {
  const { quiz, resolveMedia } = bundle;
  const showT = TRANSLATIONS[quiz.settings.language] || TRANSLATIONS.en;
  const [display] = useDisplaySettings("main");
  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const teams = useMemo(() => resolveTeams(quiz.teams, resolveMedia), [quiz.teams, resolveMedia]);
  const context = useMemo(() => ({ quiz, rounds, teams, resolveMedia }), [quiz, rounds, teams, resolveMedia]);
  const scores = useScores(quiz, rounds, showT.round);
  const rules = rulesFor(quiz, showT);
  const prizes = prizesFor(quiz);
  const visible = rounds.filter((r) => !isRoundHidden(r));
  const [open, setOpen] = useState(null); // { label, node }
  const [launcher, setLauncher] = useState(false);

  const click = (button) => fmt(t.mp_click, { button });
  const thumb = (label, node) => <Thumb key={label} label={label} display={display} onOpen={() => setOpen({ label, node })}>{node}</Thumb>;

  const welcome = <WelcomeScreen onStart={noop} startTime={quiz.settings.startTime} showTime={quiz.settings.showTime} config={quiz.settings} t={showT} />;
  const dashboard = <Dashboard rounds={rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={showT} />;
  const question = (round, qIndex, step, mode) => (
    <QuestionScreen roundData={round} mode={mode} onBack={noop} isPresenter={false} t={showT} preview={{ qIndex, step }} />
  );
  const leaderboard = (step) => (
    <Leaderboard data={scores.teams} availableRounds={scores.rounds} onClose={noop} isPresenter={false} t={showT} preview={{ step }} />
  );

  // The screens before the dashboard, each with the button that leaves it
  const intro = [
    [t.mp_welcome, welcome, showT.start_show],
    rules && [t.mp_rules, <RulesScreen rules={rules} onNext={noop} t={showT} />, showT.continue],
    prizes && [t.mp_prizes, <PrizesScreen prizes={prizes} onNext={noop} t={showT} />, showT.continue_dash],
  ].filter(Boolean);

  // A pass through a round: the title card, the first questions, the count of the rest and the last one
  const pass = (round, mode) => {
    const count = round.questions.length;
    const screens = (qi) => {
      const q = round.questions[qi];
      const asked = [<Edge key={`e${qi}`} how={t.mp_edge_next} />,
        thumb(`${round.title} · ${fmt(t.mp_question, { n: q.id })}`, question(round, qi, previewStep(q.type, "question"), mode))];
      if (mode !== "with_answers") return asked;
      return [...asked, <Edge key={`a${qi}`} how={t.mp_edge_answer} />,
        thumb(`${round.title} · ${fmt(t.mp_answer, { n: q.id })}`, question(round, qi, previewStep(q.type, "answer"), mode))];
    };
    const first = round.questions.slice(0, SHOWN_QUESTIONS).map((_, qi) => screens(qi));
    return [
      thumb(`${round.title} · ${t.mp_round_card}`, question(round, -1, 0, mode)),
      ...first.flat(),
      count > SHOWN_QUESTIONS + 1 && <More key="more" n={count - SHOWN_QUESTIONS - 1} t={t} />,
      ...(count > SHOWN_QUESTIONS ? screens(count - 1) : []),
      <Edge key="end" how={t.mp_edge_last} />,
      <Return key="ret" t={t} />,
    ];
  };

  return (
    <QuizContext.Provider value={context}>
      <MotionConfig reducedMotion="always">
        <div className="min-h-screen bg-[#050505] text-white font-sans">
          <header className="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
            <div className="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
              <a href={quiz.builtin ? "#/" : quizHash(quiz.id, "edit")} className="flex items-center gap-2 text-gray-400 hover:text-white">
                <ArrowLeft size={18} /> {quiz.builtin ? t.settings_exit : t.lib_edit}
              </a>
              <h1 className="font-['League_Spartan'] text-2xl font-black truncate">{t.mp_title}: {quiz.title || t.ed_untitled}</h1>
              <div className="ml-auto flex flex-wrap items-center gap-3">
                <a href={quizHash(quiz.id, "check")} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">{t.chk_open}</a>
                <button onClick={() => setLauncher(true)} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
                  <Play size={16} /> {t.lib_open}
                </button>
              </div>
            </div>
          </header>

          <main className="max-w-[1800px] mx-auto px-6 py-8 space-y-10">
            <p className="text-gray-300 max-w-3xl">{t.mp_intro}</p>

            <section className="space-y-3">
              <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest text-gray-400">{t.mp_before}</h2>
              <div className="flex items-start overflow-x-auto pb-3">
                {intro.flatMap(([label, node, button], i) => [thumb(label, node), <Edge key={`i${i}`} how={click(button)} />])}
                {thumb(t.mp_dashboard, dashboard)}
              </div>
            </section>

            <section className="space-y-4">
              <div>
                <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest text-gray-400">{t.mp_hub}</h2>
                <p className="text-sm text-gray-400 mt-1">{t.mp_hub_text}</p>
              </div>
              <div className="grid lg:grid-cols-[260px_1fr] gap-6">
                <div className="space-y-4 lg:sticky lg:top-24 lg:self-start">
                  {thumb(t.mp_dashboard, dashboard)}
                  <div className="rounded-xl border border-white/10 bg-white/[0.03] p-4 space-y-2 text-sm text-gray-300">
                    <h3 className="flex items-center gap-2 font-bold text-white"><Keyboard size={16} /> {t.mp_keys_title}</h3>
                    {[t.mp_key_d, t.mp_key_esc, t.mp_key_z, t.mp_key_p, t.mp_key_c].map((line) => <p key={line}><Rich text={line} /></p>)}
                  </div>
                </div>
                <div className="min-w-0">
                  {visible.map((round) => (
                    <Branch key={round.id} title={`${showT.round} ${round.number} · ${round.title}`}>
                      <Lane label={t.mp_round_q}><Edge fromTrunk how={click(showT.btn_questions)} />{pass(round, "questions_only")}</Lane>
                      <Lane label={t.mp_round_a}><Edge fromTrunk how={click(showT.btn_answers)} />{pass(round, "with_answers")}</Lane>
                    </Branch>
                  ))}
                  <Branch title={t.mp_leaderboard}>
                    <Lane>
                      <Edge fromTrunk how={click(showT.leaderboard_btn)} />
                      {thumb(t.mp_lb_table, leaderboard(0))}
                      <Edge how={t.mp_edge_group} />
                      {thumb(t.mp_lb_card, leaderboard(1))}
                      <Edge how={t.mp_edge_points} />
                      {thumb(t.mp_lb_added, leaderboard(2))}
                      <More n={null} t={t} />
                      <Edge how={t.mp_edge_shuffle} />
                      {thumb(t.mp_lb_final, leaderboard(Number.MAX_SAFE_INTEGER))}
                      <Edge how={t.mp_back_lb} />
                      <Return t={t} />
                    </Lane>
                  </Branch>
                  <Branch title={t.mp_pause} last>
                    <Lane>
                      <Edge fromTrunk how={click(showT.half_time)} />
                      {thumb(t.mp_pause, <PauseScreen isPresenter={false} leaderboardData={scores.teams} availableRounds={scores.rounds} onBack={noop} t={showT} />)}
                      <Edge how={t.mp_back_pause} />
                      <Return t={t} />
                    </Lane>
                  </Branch>
                </div>
              </div>
            </section>
          </main>
        </div>

        {open && <Enlarged open={open} display={display} onClose={() => setOpen(null)} t={t} />}
      </MotionConfig>
      {launcher && <ShowLauncher quizId={quiz.id} onClose={() => setLauncher(false)} t={t} />}
    </QuizContext.Provider>
  );
}

// A screen thumbnail; the scene mounts only when it comes near the viewport, as a quiz has many
function Thumb({ label, display, onOpen, children }) {
  const ref = useRef(null);
  const [seen, setSeen] = useState(false);
  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => entry.isIntersecting && setSeen(true), { rootMargin: "600px" });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);
  return (
    // A div, not a button: the scenes inside have buttons of their own
    <div ref={ref} role="button" tabIndex={0} onClick={onOpen} onKeyDown={(e) => e.key === "Enter" && onOpen()} title={label}
      className="group shrink-0 w-56 text-left cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-yellow-400 rounded-lg">
      <StageFrame display={display} className="rounded-lg ring-1 ring-white/15 group-hover:ring-2 group-hover:ring-yellow-400 pointer-events-none">
        {seen && children}
      </StageFrame>
      <span className="block text-xs text-gray-300 mt-1.5 truncate group-hover:text-yellow-300">{label}</span>
    </div>
  );
}

// A line to the next screen, with what moves the show on (keys in `backticks`). Thumbnails are 126 px
// tall, so the line sits at their middle. fromTrunk: the first line of a branch, drawn from the trunk.
function Edge({ how, fromTrunk = false }) {
  return (
    <div className={`shrink-0 h-[126px] flex flex-col justify-center ${fromTrunk ? "w-[10.5rem] pr-2" : "w-32 px-2"}`}>
      <span className={`text-[11px] leading-tight text-gray-300 text-center mb-1.5 ${fromTrunk ? "pl-10" : ""}`}><Rich text={how} /></span>
      <div className="relative h-px bg-white/30">
        <ArrowRight size={14} className="absolute -right-1.5 -top-[7px] text-white/50" />
      </div>
    </div>
  );
}

// Back to the dashboard at the end of a branch
function Return({ how, t }) {
  return (
    <>
      {how && <Edge how={how} />}
      <div className="shrink-0 h-[126px] flex items-center">
        <span className="flex items-center gap-1.5 px-3 py-2 rounded-full border border-yellow-500/40 bg-yellow-500/10 text-yellow-200 text-xs font-bold">
          <CornerDownLeft size={14} /> {t.mp_dashboard}
        </span>
      </div>
    </>
  );
}

// The screens left out between two thumbnails
function More({ n, t }) {
  return (
    <div className="shrink-0 h-[126px] flex items-center px-1">
      <span className="px-3 py-1.5 rounded-full bg-white/5 border border-dashed border-white/20 text-xs text-gray-400 whitespace-nowrap">
        {n === null ? "…" : fmt(t.mp_more, { n })}
      </span>
    </div>
  );
}

// One branch from the dashboard: the trunk runs down the left edge (the last branch's ends at its first
// line) and each lane's first line starts at it
function Branch({ title, last = false, children }) {
  return (
    <div className="relative pb-6">
      <span className={`absolute left-0 top-0 w-px bg-white/25 ${last ? "h-[108px]" : "h-full"}`} />
      <h3 className="font-bold mb-2 pl-10">{title}</h3>
      <div className="space-y-2">{children}</div>
    </div>
  );
}

// A row of screens inside a branch (a round has one per pass)
function Lane({ label, children }) {
  return (
    <div>
      {label && <p className="text-xs uppercase tracking-widest text-gray-500 mb-1 pl-10">{label}</p>}
      <div className="flex items-start overflow-x-auto pb-2">{children}</div>
    </div>
  );
}

function Enlarged({ open, display, onClose, t }) {
  useEffect(() => {
    const onKey = (e) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
  return (
    <div className="fixed inset-0 z-50 bg-black/90 backdrop-blur-sm text-white font-sans flex flex-col items-center justify-center gap-3 p-4" onClick={onClose}>
      <div className="w-full flex flex-col gap-3" style={{ maxWidth: "min(1600px, calc((100vh - 140px) * 16 / 9))" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-3 text-sm">
          <span className="font-bold truncate">{open.label}</span>
          <button onClick={onClose} className="ml-auto p-2 rounded-full hover:bg-white/10" aria-label={t.chk_close}><X size={20} /></button>
        </div>
        <StageFrame display={display} className="rounded-xl ring-1 ring-white/20">{open.node}</StageFrame>
      </div>
    </div>
  );
}
