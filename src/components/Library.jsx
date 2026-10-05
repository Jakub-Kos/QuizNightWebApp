import { useState, useEffect, useCallback } from "react";
import { Plus, Play, Pencil, Copy, Download, Trash2, FileDown, Loader2, AlertTriangle, ScanSearch, HelpCircle } from "lucide-react";
import { listQuizzes, saveQuiz, deleteQuiz, requestPersistence } from "../quiz/storage";
import { createQuiz, loadQuizBundle, DEMO_ID } from "../quiz/model";
import { parseQuestionsCsv } from "../quiz/parse";
import { readImport, saveImport, exportZip, duplicateQuiz, downloadBlob, safeFileName } from "../quiz/package";
import { quizHash, navigate } from "../hooks/useHashRoute";
import FileDrop from "./FileDrop";

const LANGUAGES = [["cs", "Čeština"], ["sk", "Slovenčina"], ["en", "English"]];

function summarize(quiz) {
  const rounds = parseQuestionsCsv(quiz.questionsCsv);
  return { rounds: rounds.length, questions: rounds.reduce((n, r) => n + r.questions.length, 0) };
}

export default function Library({ t, lang, onLanguage }) {
  const [quizzes, setQuizzes] = useState(null);
  const [demo, setDemo] = useState(null);
  const [busy, setBusy] = useState(null); // id of the card (or "import") with an action running
  const [confirmDelete, setConfirmDelete] = useState(null);
  const [error, setError] = useState(null);

  const refresh = useCallback(async () => {
    const list = await listQuizzes();
    setQuizzes(list.sort((a, b) => b.updatedAt - a.updatedAt));
  }, []);

  useEffect(() => {
    refresh().catch((e) => setError(String(e)));
    loadQuizBundle(DEMO_ID).then((b) => setDemo(b.quiz)).catch(() => {});
  }, [refresh]);

  const run = async (key, fn) => {
    setBusy(key);
    setError(null);
    try { await fn(); } catch (e) { setError(`${t.lib_action_failed} ${e.message || e}`); }
    setBusy(null);
  };

  const withBundle = async (id, fn) => {
    const bundle = await loadQuizBundle(id);
    try { return await fn(bundle); } finally { bundle.dispose(); }
  };

  const handleNew = () => run("new", async () => {
    const quiz = createQuiz({ title: t.ed_untitled, settings: { language: lang } });
    await saveQuiz(quiz);
    requestPersistence();
    navigate(quizHash(quiz.id, "edit"));
  });

  const handleImport = (files) => run("import", async () => {
    const id = await saveImport(await readImport(files));
    navigate(quizHash(id, "edit"));
  });

  const handleExport = (quiz) => run(quiz.id, () =>
    withBundle(quiz.id, async (b) => downloadBlob(await exportZip(b), `${safeFileName(quiz.title)}.zip`)));

  const handleCopy = (quiz) => run(quiz.id, async () => {
    await withBundle(quiz.id, (b) => duplicateQuiz(b, `${quiz.title} (${t.lib_copy_suffix})`));
    await refresh();
  });

  const handleDelete = (quiz) => run(quiz.id, async () => {
    await deleteQuiz(quiz.id);
    setConfirmDelete(null);
    await refresh();
  });

  const cards = [...(demo ? [demo] : []), ...(quizzes || [])];

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <div className="max-w-5xl mx-auto px-6 py-12 space-y-10">
        <header className="flex flex-wrap items-end justify-between gap-6">
          <div>
            <h1 className="font-['League_Spartan'] text-5xl font-black uppercase tracking-widest text-yellow-400">Quiz Night</h1>
            <p className="text-gray-400 mt-3 max-w-xl">{t.lib_subtitle}</p>
          </div>
          <div className="flex flex-wrap items-center gap-3">
          <a href="#/help" className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
            <HelpCircle size={16} /> {t.lib_help}
          </a>
          <div className="flex gap-1 bg-white/5 rounded-xl p-1">
            {LANGUAGES.map(([code, name]) => (
              <button key={code} onClick={() => onLanguage(code)}
                className={`px-3 py-1.5 rounded-lg text-sm ${lang === code ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>
                {name}
              </button>
            ))}
          </div>
          </div>
        </header>

        {error && (
          <div className="flex items-start gap-3 p-4 rounded-xl bg-red-500/10 border border-red-500/30 text-red-200">
            <AlertTriangle size={20} className="shrink-0 mt-0.5" /> <span>{error}</span>
          </div>
        )}

        <section className="grid md:grid-cols-[1fr_2fr] gap-4">
          <div className="flex flex-col gap-3">
            <button onClick={handleNew} disabled={busy === "new"}
              className="flex-1 flex flex-col items-center justify-center gap-3 p-8 rounded-2xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-lg transition-colors">
              <Plus size={36} /> {t.lib_new}
            </button>
            <button onClick={() => demo && handleExport(demo)} disabled={!demo || busy === DEMO_ID}
              className="flex items-center justify-center gap-2 p-3 rounded-xl bg-white/5 hover:bg-white/10 text-gray-300 text-sm">
              <FileDown size={16} /> {t.lib_template}
            </button>
          </div>
          {busy === "import" ? (
            <div className="rounded-2xl border-2 border-dashed border-white/15 flex items-center justify-center gap-3 text-gray-300 p-10">
              <Loader2 className="animate-spin" /> {t.lib_importing}
            </div>
          ) : (
            <FileDrop onFiles={handleImport} accept=".zip,.csv,.json,image/*,audio/*,video/*" label={t.lib_import_hint} t={t} />
          )}
        </section>

        <section className="space-y-4">
          <h2 className="font-['League_Spartan'] text-2xl font-bold uppercase tracking-widest text-gray-400">{t.lib_title}</h2>
          {quizzes && quizzes.length === 0 && <p className="text-gray-500">{t.lib_empty}</p>}
          <div className="grid md:grid-cols-2 gap-4">
            {cards.map((quiz) => {
              const { rounds, questions } = summarize(quiz);
              const isBusy = busy === quiz.id;
              return (
                <article key={quiz.id} className="rounded-2xl bg-white/[0.04] border border-white/10 p-6 flex flex-col gap-5">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <h3 className="text-xl font-bold truncate">{quiz.title || t.ed_untitled}</h3>
                      <p className="text-sm text-gray-400 mt-1">
                        {rounds} {t.lib_rounds} · {questions} {t.lib_questions} · {quiz.teams.length} {t.lib_teams}
                      </p>
                      {!quiz.builtin && (
                        <p className="text-xs text-gray-500 mt-1">{t.lib_updated} {new Date(quiz.updatedAt).toLocaleString()}</p>
                      )}
                    </div>
                    {quiz.builtin && <span className="shrink-0 text-xs font-bold uppercase tracking-wider px-2 py-1 rounded bg-blue-500/20 text-blue-300">{t.lib_demo_badge}</span>}
                    {isBusy && <Loader2 className="animate-spin shrink-0 text-gray-400" size={20} />}
                  </div>

                  {confirmDelete === quiz.id ? (
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-red-300 text-sm mr-auto">{t.lib_delete_confirm}</span>
                      <button onClick={() => setConfirmDelete(null)} className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">{t.lib_cancel}</button>
                      <button onClick={() => handleDelete(quiz)} className="px-4 py-2 rounded-xl bg-red-600 hover:bg-red-500 text-sm font-bold">{t.lib_delete}</button>
                    </div>
                  ) : (
                    <div className="flex flex-wrap gap-2">
                      <a href={quizHash(quiz.id)} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
                        <Play size={16} /> {t.lib_open}
                      </a>
                      {!quiz.builtin && (
                        <a href={quizHash(quiz.id, "edit")} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
                          <Pencil size={16} /> {t.lib_edit}
                        </a>
                      )}
                      <a href={quizHash(quiz.id, "check")} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
                        <ScanSearch size={16} /> {t.chk_open}
                      </a>
                      <button onClick={() => handleCopy(quiz)} disabled={isBusy} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
                        <Copy size={16} /> {t.lib_copy}
                      </button>
                      <button onClick={() => handleExport(quiz)} disabled={isBusy} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
                        <Download size={16} /> {t.lib_export}
                      </button>
                      {!quiz.builtin && (
                        <button onClick={() => setConfirmDelete(quiz.id)} className="flex items-center gap-2 px-4 py-2 rounded-xl text-red-300 hover:bg-red-500/20 text-sm ml-auto">
                          <Trash2 size={16} /> {t.lib_delete}
                        </button>
                      )}
                    </div>
                  )}
                </article>
              );
            })}
          </div>
        </section>
      </div>
    </div>
  );
}
