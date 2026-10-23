import { useState, useEffect, useMemo } from "react";
import { ArrowLeft, Search, X, Download, BookmarkPlus, Wand2, Loader2, Palette, Share2 } from "lucide-react";
import { QuizContext } from "../quiz/context";
import { parseQuestionsCsv, isRoundHidden } from "../quiz/parse";
import { resolveTeams, DEMO_ID } from "../quiz/model";
import { rulesFor, prizesFor } from "../quiz/intro";
import { previewStep } from "../quiz/validate";
import { fmt } from "../quiz/files";
import { downloadBlob } from "../quiz/package";
import { listQuizzes, putLibraryTheme } from "../quiz/storage";
import { sampleLeaderboard } from "../quiz/scores";
import { themeName } from "../quiz/themes";
import { exportThemePack, themePackName, themeForQuizMedia } from "../quiz/themePack";
import { storeQuizTheme } from "../quiz/themeStore";
import { loadWorkshop, workshopFileUrl, fetchWorkshopPack } from "../quiz/workshop";
import { driveErrorText } from "../quiz/drive";
import { quizHash } from "../hooks/useHashRoute";
import { useQuizBundle } from "../hooks/useQuizBundle";
import { useDisplaySettings } from "../hooks/useDisplay";
import { inputCls, btnCls } from "./EditorParts";
import { StageFrame } from "./Stage";
import { ThemeDecorations } from "./Theme";
import WelcomeScreen from "./WelcomeScreen";
import RulesScreen from "./RulesScreen";
import Dashboard from "./Dashboard";
import QuestionScreen from "./QuestionScreen";
import Leaderboard from "./Leaderboard";
import PrizesScreen from "./PrizesScreen";

const noop = () => {};

// #/themes: the theme workshop, themes shared by others (quiz/workshop.js), shown on the demo quiz
export default function Workshop({ lang, t }) {
  const demo = useQuizBundle(DEMO_ID);
  const [entries, setEntries] = useState(null); // null while loading
  const [error, setError] = useState(false);
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState(null);
  const [open, setOpen] = useState(null);

  useEffect(() => { loadWorkshop().then(setEntries, () => setError(true)); }, []);

  const tags = useMemo(() => [...new Set((entries || []).flatMap((e) => e.theme.tags))].sort(), [entries]);
  const shown = (entries || []).filter((e) => {
    const text = [themeName(e.theme, lang), e.theme.author, Object.values(e.theme.description).join(" "), e.theme.tags.join(" ")].join(" ").toLowerCase();
    return (!tag || e.theme.tags.includes(tag)) && text.includes(query.trim().toLowerCase());
  });

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <header className="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
        <div className="max-w-6xl mx-auto px-6 py-4 flex flex-wrap items-center gap-4">
          <a href="#/" className="flex items-center gap-2 text-gray-400 hover:text-white"><ArrowLeft size={18} /> {t.ws_back}</a>
          <h1 className="font-['League_Spartan'] text-2xl font-bold uppercase tracking-widest mr-auto flex items-center gap-3"><Palette size={22} className="text-yellow-400" /> {t.ws_title}</h1>
          <div className="relative w-full sm:w-72">
            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
            <input className={`${inputCls} pl-9 py-2`} value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t.ws_search} />
          </div>
        </div>
      </header>

      <main className="max-w-6xl mx-auto px-6 py-6 space-y-6">
        <p className="text-gray-400 max-w-3xl">{t.ws_intro}</p>
        {tags.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {[null, ...tags].map((tg) => (
              <button key={tg ?? "all"} type="button" onClick={() => setTag(tg)}
                className={`px-3 py-1.5 rounded-full text-sm ${tag === tg ? "bg-yellow-500 text-black font-bold" : "bg-white/5 text-gray-300 hover:bg-white/10"}`}>
                {tg ? `#${tg}` : t.ws_all}
              </button>
            ))}
          </div>
        )}

        {error && <p className="text-red-400">{t.ws_error}</p>}
        {!error && (entries === null || demo.status !== "ready") && <Loader2 className="animate-spin text-gray-500" />}
        {entries && demo.status === "ready" && (
          shown.length ? (
            <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
              {shown.map((entry) => <Card key={entry.path} entry={entry} demo={demo.bundle} lang={lang} t={t} onOpen={() => setOpen(entry)} />)}
            </div>
          ) : <p className="text-gray-500">{t.ws_none}</p>
        )}

        <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-5 flex flex-wrap items-center gap-4">
          <Share2 size={22} className="text-yellow-400 shrink-0" />
          <div className="flex-1 min-w-[240px]">
            <h2 className="font-bold">{t.ws_share_title}</h2>
            <p className="text-sm text-gray-400">{t.ws_share_text}</p>
          </div>
        </section>
      </main>

      {open && demo.status === "ready" && <Detail entry={open} demo={demo.bundle} lang={lang} t={t} onClose={() => setOpen(null)} />}
    </div>
  );
}

// The demo quiz as preview content, with the theme's files served from the workshop
function useDemoContext(entry, demo) {
  const { quiz, resolveMedia } = demo;
  return useMemo(() => {
    const rounds = parseQuestionsCsv(quiz.questionsCsv);
    const resolve = (name) => (name && name.startsWith("assets/") ? workshopFileUrl(entry, name) : resolveMedia(name));
    return { quiz, rounds, teams: resolveTeams(quiz.teams, resolveMedia), resolveMedia: resolve };
  }, [entry, quiz, resolveMedia]);
}

function Card({ entry, demo, lang, t, onOpen }) {
  const [display] = useDisplaySettings("main");
  const context = useDemoContext(entry, demo);
  const rules = rulesFor(context.quiz, t);
  const description = entry.theme.description[lang] || entry.theme.description.en || "";
  return (
    <div role="button" tabIndex={0} onClick={onOpen} onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onOpen(); } }}
      className="group cursor-pointer rounded-2xl border border-white/10 hover:border-white/30 bg-white/[0.03] p-2 space-y-2 transition-colors">
      <QuizContext.Provider value={context}>
        <StageFrame display={display} theme={entry.theme} className="rounded-xl pointer-events-none">
          {rules ? <RulesScreen rules={rules} onNext={noop} t={t} /> : <Dashboard rounds={context.rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={t} />}
          <ThemeDecorations />
        </StageFrame>
      </QuizContext.Provider>
      <div className="px-2 pb-1">
        <div className="flex items-baseline gap-2">
          <h3 className="font-bold truncate">{themeName(entry.theme, lang)}</h3>
          {entry.theme.author && <span className="text-xs text-gray-500 truncate">{fmt(t.thm_by, { author: entry.theme.author })}</span>}
        </div>
        {description && <p className="text-sm text-gray-400 line-clamp-2">{description}</p>}
        <div className="flex flex-wrap gap-1 mt-1.5">{entry.theme.tags.map((tg) => <span key={tg} className="text-xs text-gray-500">#{tg}</span>)}</div>
      </div>
    </div>
  );
}

function Detail({ entry, demo, lang, t, onClose }) {
  const [display] = useDisplaySettings("main");
  const context = useDemoContext(entry, demo);
  const [scene, setScene] = useState("welcome");
  const [quizzes, setQuizzes] = useState([]);
  const [target, setTarget] = useState("");
  const [busy, setBusy] = useState(null);
  const [notice, setNotice] = useState(null); // { ok, text, quizId }

  useEffect(() => {
    listQuizzes().then((list) => {
      const own = list.filter((q) => !q.builtin).sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0));
      setQuizzes(own);
      setTarget((cur) => cur || own[0]?.id || "");
    });
  }, []);
  useEffect(() => {
    const esc = (e) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", esc);
    return () => window.removeEventListener("keydown", esc);
  }, [onClose]);

  const { quiz, rounds, teams } = context;
  const name = themeName(entry.theme, lang);
  const firstRound = rounds.find((r) => !isRoundHidden(r) && r.questions.length);
  const rules = rulesFor(quiz, t);
  const prizes = prizesFor(quiz);
  const board = sampleLeaderboard(teams, [], t.round, []);
  const screens = [
    ["welcome", t.mp_welcome, <WelcomeScreen onStart={noop} startTime={quiz.settings.startTime} showTime={quiz.settings.showTime} config={quiz.settings} t={t} />],
    rules && ["rules", t.mp_rules, <RulesScreen rules={rules} onNext={noop} t={t} />],
    prizes && ["prizes", t.mp_prizes, <PrizesScreen prizes={prizes} onNext={noop} t={t} />],
    ["dashboard", t.te_s_dashboard, <Dashboard rounds={rounds} onSelectRound={noop} onOpenLeaderboard={noop} onOpenPause={noop} t={t} />],
    firstRound && ["round", t.te_s_round, <QuestionScreen roundData={firstRound} mode="with_answers" onBack={noop} isPresenter={false} t={t} preview={{ qIndex: -1, step: 0 }} />],
    firstRound && ["question", t.te_s_question, <QuestionScreen roundData={firstRound} mode="with_answers" onBack={noop} isPresenter={false} t={t} preview={{ qIndex: 0, step: previewStep(firstRound.questions[0].type, "question") }} />],
    ["leaderboard", t.te_s_leaderboard, <Leaderboard data={board.teams} availableRounds={board.rounds} onClose={noop} isPresenter={false} t={t} preview={{ step: 0 }} />],
  ].filter(Boolean);
  const shown = screens.find(([key]) => key === scene) || screens[0];

  const act = async (kind, fn) => {
    setBusy(kind);
    setNotice(null);
    try {
      setNotice(await fn(await fetchWorkshopPack(entry)));
    } catch (err) {
      setNotice({ ok: false, text: fmt(t.ws_failed, { message: driveErrorText(err, t) }) });
    }
    setBusy(null);
  };
  const applyToQuiz = () => {
    const chosen = quizzes.find((q) => q.id === target);
    if (!chosen) return;
    if (chosen.customTheme && !window.confirm(fmt(t.thm_new_confirm, { name: themeName(chosen.customTheme, lang) }))) return;
    act("quiz", async (pack) => {
      await storeQuizTheme(chosen.id, pack);
      return { ok: true, text: fmt(t.ws_used, { name, quiz: chosen.title || t.ed_untitled }), quizId: chosen.id };
    });
  };
  const keep = () => act("library", async (pack) => {
    const { theme, media } = themeForQuizMedia(pack);
    await putLibraryTheme({ id: crypto.randomUUID(), theme, files: media });
    return { ok: true, text: fmt(t.te_lib_saved, { name }) };
  });
  const download = () => act("download", async (pack) => {
    const blobs = new Map(pack.assets.map((a) => [a.name, a.blob]));
    downloadBlob(await exportThemePack(entry.theme, (n) => blobs.get(n) ?? null), themePackName(entry.theme));
    return null;
  });

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-start justify-center overflow-y-auto p-4 sm:p-8" onClick={onClose}>
      <div className="w-full max-w-5xl rounded-2xl bg-[#0d0d10] border border-white/15 p-5 space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-start gap-4">
          <div className="flex-1 min-w-0">
            <h2 className="text-2xl font-bold">{name}</h2>
            {entry.theme.author && <p className="text-sm text-gray-500">{fmt(t.thm_by, { author: entry.theme.author })}</p>}
            {(entry.theme.description[lang] || entry.theme.description.en) && <p className="text-gray-300 mt-2">{entry.theme.description[lang] || entry.theme.description.en}</p>}
          </div>
          <button type="button" onClick={onClose} aria-label={t.chk_close} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10"><X size={20} /></button>
        </div>
        <div className="flex flex-wrap gap-1.5">
          {screens.map(([key, label]) => (
            <button key={key} type="button" onClick={() => setScene(key)}
              className={`px-3 py-1.5 rounded-lg text-sm ${shown[0] === key ? "bg-yellow-500 text-black font-bold" : "bg-white/5 text-gray-300 hover:bg-white/10"}`}>{label}</button>
          ))}
        </div>
        <QuizContext.Provider value={context}>
          <StageFrame display={display} theme={entry.theme} className="rounded-xl ring-1 ring-white/15">
            <div key={shown[0]} className="contents">{shown[2]}</div>
            <ThemeDecorations quiet={["question", "leaderboard"].includes(shown[0])} />
          </StageFrame>
        </QuizContext.Provider>
        <div className="flex flex-wrap items-center gap-2">
          {quizzes.length > 0 ? (
            <>
              <select className={`${inputCls} py-2.5 w-auto max-w-xs`} value={target} onChange={(e) => setTarget(e.target.value)} aria-label={t.ws_pick_quiz}>
                {quizzes.map((q) => <option key={q.id} value={q.id}>{q.title || t.ed_untitled}</option>)}
              </select>
              <button type="button" className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`} onClick={applyToQuiz} disabled={Boolean(busy)}>
                {busy === "quiz" ? <Loader2 size={16} className="animate-spin" /> : <Wand2 size={16} />} {t.ws_use}
              </button>
            </>
          ) : <span className="text-sm text-gray-500">{t.ws_no_quizzes}</span>}
          <button type="button" className={btnCls} onClick={keep} disabled={Boolean(busy)}>
            {busy === "library" ? <Loader2 size={16} className="animate-spin" /> : <BookmarkPlus size={16} />} {t.te_lib_save}
          </button>
          <button type="button" className={btnCls} onClick={download} disabled={Boolean(busy)}>
            {busy === "download" ? <Loader2 size={16} className="animate-spin" /> : <Download size={16} />} {t.ws_download}
          </button>
        </div>
        {notice && (
          <p className={`text-sm ${notice.ok ? "text-green-400" : "text-red-400"}`}>
            {notice.text}{" "}
            {notice.quizId && <a href={quizHash(notice.quizId, "edit")} className="underline hover:text-white">{t.ws_open_setup}</a>}
          </p>
        )}
      </div>
    </div>
  );
}
