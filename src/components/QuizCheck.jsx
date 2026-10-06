import { useState, useEffect, useMemo, useCallback } from "react";
import { MotionConfig } from "framer-motion";
import { ArrowLeft, ArrowRight, AlertTriangle, AlertCircle, Info, CheckCircle2, Play, X, EyeOff } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv, isRoundHidden } from "../quiz/parse";
import { resolveTeams } from "../quiz/model";
import { referencedMedia } from "../quiz/package";
import { validateQuiz, previewStep } from "../quiz/validate";
import { fmt } from "../quiz/files";
import { useQuizBundle } from "../hooks/useQuizBundle";
import { useDisplaySettings } from "../hooks/useDisplay";
import { quizHash } from "../hooks/useHashRoute";
import { StageFrame } from "./Stage";
import QuestionScreen from "./QuestionScreen";

const LEVELS = {
  error: { icon: AlertCircle, text: "text-red-300", ring: "ring-2 ring-red-500", badge: "bg-red-600" },
  warning: { icon: AlertTriangle, text: "text-yellow-300", ring: "ring-2 ring-yellow-500", badge: "bg-yellow-500 text-black" },
  info: { icon: Info, text: "text-blue-300", ring: "", badge: "bg-blue-600" },
};
const tileKey = (ri, qi) => `${ri}:${qi}`;

export default function QuizCheck({ id, t }) {
  const state = useQuizBundle(id);
  if (state.status === "ready") return <CheckView bundle={state.bundle} t={t} />;
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

// The built-in demo serves media as plain files, so ask the server which of them exist
function useDemoMediaPresence(quiz, resolveMedia) {
  const [present, setPresent] = useState(null);
  useEffect(() => {
    if (!quiz.builtin) return;
    let cancelled = false;
    Promise.all(referencedMedia(quiz).map(async (name) => {
      try {
        const res = await fetch(resolveMedia(name), { method: "HEAD" });
        // Missing files can come back as the HTML app page instead of a 404
        return res.ok && !res.headers.get("content-type")?.includes("text/html") ? name : null;
      } catch { return null; }
    })).then((names) => !cancelled && setPresent(new Set(names.filter(Boolean))));
    return () => { cancelled = true; };
  }, [quiz, resolveMedia]);
  return present;
}

function CheckView({ bundle, t }) {
  const { quiz, resolveMedia, hasMedia } = bundle;
  const showT = TRANSLATIONS[quiz.settings.language] || TRANSLATIONS.en;
  const [display] = useDisplaySettings("main");

  const rounds = useMemo(() => parseQuestionsCsv(quiz.questionsCsv), [quiz.questionsCsv]);
  const teams = useMemo(() => resolveTeams(quiz.teams, resolveMedia), [quiz.teams, resolveMedia]);
  const quizContext = useMemo(() => ({ quiz, rounds, teams, resolveMedia }), [quiz, rounds, teams, resolveMedia]);

  const demoPresent = useDemoMediaPresence(quiz, resolveMedia);
  const issues = useMemo(() => {
    const exists = quiz.builtin ? (name) => demoPresent?.has(name) ?? true : hasMedia;
    return validateQuiz(quiz, rounds, exists);
  }, [quiz, rounds, hasMedia, demoPresent]);

  const issuesByTile = useMemo(() => {
    const map = new Map();
    issues.forEach((i) => {
      if (i.roundIndex < 0) return;
      const k = tileKey(i.roundIndex, i.qIndex);
      map.set(k, [...(map.get(k) || []), i]);
    });
    return map;
  }, [issues]);

  // Every frame in show order: the round intro (qi -1) then each question
  const tiles = useMemo(() => rounds.flatMap((r, ri) => [{ ri, qi: -1 }, ...r.questions.map((_, qi) => ({ ri, qi }))]), [rounds]);

  const [view, setView] = useState("question");
  const [columns, setColumns] = useState(4);
  const [openIndex, setOpenIndex] = useState(null);

  const counts = { error: 0, warning: 0, info: 0 };
  issues.forEach((i) => counts[i.level]++);

  const message = (i) => fmt(t[`chk_${i.code}`] || i.code, i.params);
  const location = (ri, qi) => {
    if (ri < 0) return t.chk_quiz;
    const round = rounds[ri];
    const r = `${t.chk_round} ${round.number}`;
    return qi < 0 ? r : `${r} · ${t.chk_question} ${round.questions[qi].id}`;
  };
  const openTile = (ri, qi) => {
    const index = tiles.findIndex((tile) => tile.ri === ri && tile.qi === (qi < 0 ? -1 : qi));
    if (index >= 0) setOpenIndex(index);
  };

  const renderFrame = (ri, qi, frameView) => {
    const round = rounds[ri];
    const preview = qi < 0
      ? { qIndex: -1, step: 0 }
      : { qIndex: qi, step: previewStep(round.questions[qi].type, frameView) };
    return <QuestionScreen roundData={round} mode="with_answers" onBack={() => {}} isPresenter={false} t={showT} preview={preview} />;
  };

  const levelOrder = { error: 0, warning: 1, info: 2 };
  const sortedIssues = [...issues].sort((a, b) => levelOrder[a.level] - levelOrder[b.level]);

  return (
    <QuizContext.Provider value={quizContext}>
      {/* Thumbnails show the end state of each frame instead of animating dozens of screens */}
      <MotionConfig reducedMotion="always">
        <div className="min-h-screen bg-[#050505] text-white font-sans">
          <header className="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10" style={{ top: "env(safe-area-inset-top, 0px)" }}>
            <div className="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
              <a href={quiz.builtin ? "#/" : quizHash(quiz.id, "edit")} className="flex items-center gap-2 text-gray-400 hover:text-white">
                <ArrowLeft size={18} /> {quiz.builtin ? t.settings_exit : t.lib_edit}
              </a>
              <h1 className="font-['League_Spartan'] text-2xl font-black truncate">{t.chk_title}: {quiz.title || t.ed_untitled}</h1>
              <div className="ml-auto flex flex-wrap items-center gap-3">
                <div className="flex gap-1 bg-white/5 rounded-xl p-1 text-sm">
                  {["question", "answer"].map((v) => (
                    <button key={v} onClick={() => setView(v)} className={`px-3 py-1.5 rounded-lg ${view === v ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>
                      {v === "question" ? t.chk_view_question : t.chk_view_answer}
                    </button>
                  ))}
                </div>
                <label className="flex items-center gap-2 text-sm text-gray-400">
                  {t.chk_size}
                  <input type="range" min="2" max="8" value={10 - columns} onChange={(e) => setColumns(10 - Number(e.target.value))} className="w-24 accent-yellow-500" />
                </label>
                <a href={quizHash(quiz.id)} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
                  <Play size={16} /> {t.lib_open}
                </a>
              </div>
            </div>
          </header>

          <main className="max-w-[1800px] mx-auto px-6 py-8 space-y-10">
            <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-4">
              <div className="flex flex-wrap items-center gap-4">
                <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">{t.chk_problems}</h2>
                <span className="text-red-300 text-sm">{counts.error} {t.chk_errors}</span>
                <span className="text-yellow-300 text-sm">{counts.warning} {t.chk_warnings}</span>
                <span className="text-blue-300 text-sm">{counts.info} {t.chk_notes}</span>
                {quiz.builtin && demoPresent === null && <span className="text-xs text-gray-500">{t.chk_checking_media}</span>}
              </div>
              {counts.error + counts.warning === 0 && (
                <p className="flex items-center gap-2 text-green-300"><CheckCircle2 size={18} /> {t.chk_all_good}</p>
              )}
              <ul className="divide-y divide-white/5">
                {sortedIssues.map((i, n) => {
                  const L = LEVELS[i.level];
                  const clickable = i.roundIndex >= 0;
                  return (
                    <li key={n}>
                      <button disabled={!clickable} onClick={() => openTile(i.roundIndex, i.qIndex)}
                        className={`w-full flex items-start gap-3 py-2 text-left text-sm ${clickable ? "hover:bg-white/5" : "cursor-default"}`}>
                        <L.icon size={16} className={`${L.text} shrink-0 mt-0.5`} />
                        <span className="text-gray-500 w-40 shrink-0">{location(i.roundIndex, i.qIndex)}</span>
                        <span className="text-gray-200">{message(i)}</span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            </section>

            {rounds.map((round, ri) => (
              <section key={round.id} className="space-y-4">
                <h2 className="flex flex-wrap items-center gap-3 font-['League_Spartan'] text-2xl font-bold">
                  <span className="text-gray-500">{t.chk_round} {round.number}</span> {round.title}
                  {isRoundHidden(round) && (
                    <span className="flex items-center gap-1 text-xs font-sans font-bold uppercase tracking-wider px-2 py-1 rounded bg-white/10 text-gray-400">
                      <EyeOff size={12} /> {t.chk_hidden_badge}
                    </span>
                  )}
                </h2>
                <div className="grid gap-4" style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}>
                  {tiles.filter((tile) => tile.ri === ri).map(({ qi }) => {
                    const tileIssues = issuesByTile.get(tileKey(ri, qi)) || [];
                    const worst = tileIssues.find((i) => i.level === "error") ? "error" : tileIssues.length ? "warning" : null;
                    const q = qi >= 0 ? round.questions[qi] : null;
                    return (
                      <div key={qi} role="button" tabIndex={0} onClick={() => openTile(ri, qi)}
                        onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), openTile(ri, qi))}
                        className="group text-left space-y-1.5 cursor-pointer focus:outline-none">
                        <div className={`relative rounded-lg overflow-hidden ${worst ? LEVELS[worst].ring : "ring-1 ring-white/10"} group-hover:ring-2 group-hover:ring-white/60 group-focus-visible:ring-2 group-focus-visible:ring-white`}>
                          <StageFrame display={display} className="pointer-events-none">{renderFrame(ri, qi, view)}</StageFrame>
                          {tileIssues.length > 0 && (
                            <span className={`absolute top-2 right-2 min-w-6 h-6 px-1.5 rounded-full text-xs font-bold flex items-center justify-center ${LEVELS[worst].badge}`}>{tileIssues.length}</span>
                          )}
                        </div>
                        <p className="text-xs text-gray-400 truncate">
                          {q ? <><span className="text-gray-200 font-bold">{t.chk_question} {q.id}</span> · {q.type}{q.media ? ` · ${q.media}` : ""}</> : t.chk_intro}
                        </p>
                      </div>
                    );
                  })}
                </div>
              </section>
            ))}
          </main>
        </div>

        {openIndex !== null && (
          <PreviewModal
            tiles={tiles} index={openIndex} onIndex={setOpenIndex} onClose={() => setOpenIndex(null)}
            view={view} onView={setView} display={display} renderFrame={renderFrame}
            issuesFor={(ri, qi) => issuesByTile.get(tileKey(ri, qi)) || []} message={message} location={location} t={t}
          />
        )}
      </MotionConfig>
    </QuizContext.Provider>
  );
}

function PreviewModal({ tiles, index, onIndex, onClose, view, onView, display, renderFrame, issuesFor, message, location, t }) {
  const { ri, qi } = tiles[index];
  const go = useCallback((delta) => onIndex((i) => Math.min(tiles.length - 1, Math.max(0, i + delta))), [onIndex, tiles.length]);

  useEffect(() => {
    const onKey = (e) => {
      if (e.key === "Escape") onClose();
      else if (e.key === "ArrowRight") go(1);
      else if (e.key === "ArrowLeft") go(-1);
      else if (e.key === " ") { e.preventDefault(); onView((v) => (v === "question" ? "answer" : "question")); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [go, onClose, onView]);

  const tileIssues = issuesFor(ri, qi);

  return (
    <div className="fixed inset-0 z-50 bg-black/90 backdrop-blur-sm text-white font-sans flex flex-col items-center justify-center gap-4 p-4" onClick={onClose}>
      <div className="w-full flex flex-col gap-3" style={{ maxWidth: "min(1600px, calc((100vh - 220px) * 16 / 9))" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex flex-wrap items-center gap-3 text-sm">
          <span className="font-bold">{location(ri, qi)}</span>
          <span className="text-gray-500">{index + 1} / {tiles.length}</span>
          <div className="ml-auto flex gap-1 bg-white/5 rounded-xl p-1">
            {["question", "answer"].map((v) => (
              <button key={v} onClick={() => onView(v)} className={`px-3 py-1.5 rounded-lg ${view === v ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>
                {v === "question" ? t.chk_view_question : t.chk_view_answer}
              </button>
            ))}
          </div>
          <button onClick={onClose} className="p-2 rounded-full hover:bg-white/10" aria-label={t.chk_close}><X size={20} /></button>
        </div>

        <div className="relative">
          <StageFrame display={display} className="rounded-xl ring-1 ring-white/20">{renderFrame(ri, qi, view)}</StageFrame>
          <button onClick={() => go(-1)} disabled={index === 0} className="absolute left-2 top-1/2 -translate-y-1/2 p-3 rounded-full bg-black/60 hover:bg-black/90 disabled:opacity-20"><ArrowLeft /></button>
          <button onClick={() => go(1)} disabled={index === tiles.length - 1} className="absolute right-2 top-1/2 -translate-y-1/2 p-3 rounded-full bg-black/60 hover:bg-black/90 disabled:opacity-20"><ArrowRight /></button>
        </div>

        {tileIssues.length > 0 && (
          <ul className="space-y-1 text-sm">
            {tileIssues.map((i, n) => {
              const L = LEVELS[i.level];
              return <li key={n} className={`flex items-start gap-2 ${L.text}`}><L.icon size={16} className="shrink-0 mt-0.5" /> {message(i)}</li>;
            })}
          </ul>
        )}
        <p className="text-xs text-gray-500">{t.chk_keys}</p>
      </div>
    </div>
  );
}
