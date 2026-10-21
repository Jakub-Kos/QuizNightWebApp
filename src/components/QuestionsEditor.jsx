import { Component, useState, useEffect, useMemo, useCallback, useRef } from "react";
import { MotionConfig } from "framer-motion";
import { ArrowLeft, Plus, Pencil, Trash2, Save, Loader2, RefreshCw, FileSpreadsheet, AlertTriangle, CheckCircle2, X, Lock } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { HELP } from "../data/help";
import { QuizContext } from "../quiz/context";
import { listMedia, updateQuiz } from "../quiz/storage";
import { rowsToCsv } from "../quiz/sheets";
import { driveErrorText, sheetUrl, ensureMediaFolder, uploadMedia } from "../quiz/drive";
import { mediaKey } from "../quiz/model";
import { loadQuestionSheet, canEditSheet, commitEdit, ConflictError } from "../quiz/sheetEdit";
import {
  draftFrom, emptyDraft, previewQuestion, planSave, planAdd, planDelete, planRename, planAddRound,
  findBlock, findRound, stampNow, parseStamp,
} from "../quiz/sheetModel";
import { validateQuiz, previewStep } from "../quiz/validate";
import { fmt } from "../quiz/files";
import { useQuizBundle } from "../hooks/useQuizBundle";
import { useGoogleAccount } from "../hooks/useGoogleAccount";
import { useDisplaySettings } from "../hooks/useDisplay";
import { quizHash } from "../hooks/useHashRoute";
import { StageFrame } from "./Stage";
import QuestionScreen from "./QuestionScreen";
import QuestionForm from "./QuestionForm";
import { GoogleAccountBar, BetaBadge } from "./DriveParts";
import { btnCls, inputCls } from "./EditorParts";

const POLL_MS = 15000;

// Question editor for quizzes opened from Google Drive: every save goes straight into the Sheet
export default function QuestionsEditor({ id, lang, t }) {
  const state = useQuizBundle(id);
  if (state.status === "ready") {
    if (!state.bundle.quiz.drive) return <Message text={t.qe_not_drive} back={quizHash(id, "edit")} t={t} />;
    return <EditorView bundle={state.bundle} lang={lang} t={t} />;
  }
  if (state.status === "loading") return <div className="min-h-screen bg-[#050505]" />;
  return <Message text={state.status === "missing" ? t.ed_not_found : state.message} back="#/" t={t} />;
}

const Message = ({ text, back, t }) => (
  <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6 px-6 text-center">
    <p className="text-xl text-gray-300">{text}</p>
    <a href={back} className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
  </div>
);

// A broken draft (e.g. half-typed) must not take the whole page down with the preview
class PreviewBoundary extends Component {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidUpdate(prev) { if (prev.resetKey !== this.props.resetKey && this.state.failed) this.setState({ failed: false }); }
  render() { return this.state.failed ? <div className="absolute inset-0 flex items-center justify-center text-gray-500">—</div> : this.props.children; }
}

// "5 minutes ago", or the date for older changes
function ago(stamp, lang) {
  const date = parseStamp(stamp);
  if (!date) return stamp;
  const locale = lang === "sk" ? "hu" : lang; // the sk texts are Hungarian
  const minutes = Math.round((Date.now() - date) / 60000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  if (minutes < 60) return rtf.format(-Math.max(0, minutes), "minute");
  if (minutes < 24 * 60) return rtf.format(-Math.round(minutes / 60), "hour");
  return date.toLocaleDateString(locale);
}

function EditorView({ bundle, lang, t }) {
  const { quiz, resolveMedia, hasMedia } = bundle;
  const google = useGoogleAccount();
  const [display] = useDisplaySettings("main");
  const showT = TRANSLATIONS[quiz.settings.language] || TRANSLATIONS.en;

  const [data, setData] = useState(null); // { tab, sheet } from loadQuestionSheet
  const [loadedAt, setLoadedAt] = useState(null);
  const [loadError, setLoadError] = useState(null);
  const [canEdit, setCanEdit] = useState(true);
  const [mediaNames, setMediaNames] = useState([]);
  const [uploaded, setUploaded] = useState(() => new Map()); // mediaKey -> object URL of files uploaded here
  const mediaFolder = useRef(quiz.drive.mediaFolderId ? { id: quiz.drive.mediaFolderId, name: quiz.drive.mediaFolderName } : null);

  // selection: { kind: "edit", ri, qi, rows } | { kind: "add", ri, marker } | { kind: "round" }
  const [selection, setSelection] = useState(null);
  const [draft, setDraft] = useState(null);
  const [roundTitle, setRoundTitle] = useState("");
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(null); // "save" | "delete" | "rename"
  const [notice, setNotice] = useState(null); // { ok, text }
  const [theirs, setTheirs] = useState(null); // newer version of the open question, saved by someone else
  const [renaming, setRenaming] = useState(null); // { marker, title }
  const [view, setView] = useState("question");

  // The poll compares against the latest selection without restarting
  const live = useRef({});
  useEffect(() => { live.current = { selection, dirty, busy }; }, [selection, dirty, busy]);
  const lastCsv = useRef(quiz.questionsCsv);

  const errorText = useCallback((err) => driveErrorText(err, t, google.user?.email), [t, google.user]);

  // Keep the show's copy of the questions in step with the Sheet
  const adopt = useCallback((fresh) => {
    setData(fresh);
    setLoadedAt(new Date());
    setLoadError(null);
    const csv = rowsToCsv(fresh.sheet.rows);
    if (csv !== lastCsv.current) {
      lastCsv.current = csv;
      updateQuiz(quiz.id, (q) => ({ ...q, questionsCsv: csv }));
    }
  }, [quiz.id]);

  const load = useCallback(async () => {
    try {
      const fresh = await loadQuestionSheet(quiz);
      adopt(fresh);
      // Someone else saved the question that is open here
      const { selection: sel } = live.current;
      if (sel?.kind === "edit") {
        const found = findBlock(fresh.sheet, sel.rows, sel);
        if (found) setSelection((s) => (s === sel ? { ...sel, ...found } : s));
        else setTheirs(fresh.sheet.rounds[sel.ri]?.questions[sel.qi] ?? null);
      }
    } catch (err) {
      setLoadError(errorText(err));
    }
  }, [quiz, adopt, errorText]);

  useEffect(() => {
    if (!google.signedIn) return;
    let cancelled = false;
    const first = setTimeout(load);
    canEditSheet(quiz.drive.spreadsheetId).then((v) => !cancelled && setCanEdit(v)).catch(() => {});
    const timer = setInterval(() => { if (!live.current.busy) load(); }, POLL_MS);
    return () => { cancelled = true; clearTimeout(first); clearInterval(timer); };
  }, [google.signedIn, load, quiz.drive.spreadsheetId]);

  useEffect(() => {
    listMedia(quiz.id).then((list) => setMediaNames(list.map((m) => m.name).sort()));
  }, [quiz.id]);
  const uploadedRef = useRef(uploaded);
  useEffect(() => { uploadedRef.current = uploaded; }, [uploaded]);
  useEffect(() => () => uploadedRef.current.forEach((url) => URL.revokeObjectURL(url)), []);

  // Files uploaded on this page count as present, and the preview shows them, without reloading
  const hasMediaLive = useCallback((name) => uploaded.has(mediaKey(name)) || hasMedia(name), [uploaded, hasMedia]);
  const resolveLive = useCallback((name) => (name && uploaded.get(mediaKey(name))) || resolveMedia(name), [uploaded, resolveMedia]);

  // Into the media folder (created as Zdroje when the quiz has none); see MediaInput
  const upload = async (file, folder, onProgress) => {
    try {
      if (!mediaFolder.current) {
        mediaFolder.current = await ensureMediaFolder(quiz.drive);
        const { id: mediaFolderId, name: mediaFolderName } = mediaFolder.current;
        updateQuiz(quiz.id, (q) => ({ ...q, drive: { ...q.drive, mediaFolderId, mediaFolderName } }));
      }
      const path = await uploadMedia(quiz.id, mediaFolder.current.id, folder ? `${folder}/${file.name}` : file.name, file, {
        onProgress,
        confirmReplace: (p) => window.confirm(fmt(t.qe_upload_replace, { file: p })),
      });
      if (!path) return null;
      setUploaded((m) => {
        const next = new Map(m);
        if (next.has(mediaKey(path))) URL.revokeObjectURL(next.get(mediaKey(path)));
        next.set(mediaKey(path), URL.createObjectURL(file));
        return next;
      });
      setMediaNames((names) => [...new Set([...names, path])].sort());
      return { path };
    } catch (err) {
      return { error: errorText(err) };
    }
  };

  // What each type looks like, from the help page's type table
  const typeHelp = useMemo(() => {
    const table = (HELP[lang] || HELP.en).sections.find((s) => s.id === "types")?.table;
    return Object.fromEntries((table?.rows || []).map(([type, onScreen]) => [type.replace(/`/g, ""), onScreen.replace(/`/g, "")]));
  }, [lang]);

  const rounds = useMemo(() => data?.sheet.rounds ?? [], [data]);

  const confirmDiscard = () => !dirty || window.confirm(t.qe_discard);
  const choose = (sel, nextDraft) => {
    setSelection(sel);
    setDraft(nextDraft);
    setDirty(false);
    setNotice(null);
    setTheirs(null);
  };
  const openQuestion = (ri, qi) => {
    if (!confirmDiscard()) return;
    const block = rounds[ri].questions[qi];
    choose({ kind: "edit", ri, qi, rows: block.rows }, draftFrom(block.question));
  };
  const openAdd = (ri) => confirmDiscard() && choose({ kind: "add", ri, marker: rounds[ri].marker }, emptyDraft());
  const openAddRound = () => {
    if (!confirmDiscard()) return;
    setRoundTitle("");
    choose({ kind: "round" }, emptyDraft());
  };
  const close = () => confirmDiscard() && choose(null, null);
  const change = (next) => { setDraft(next); setDirty(true); };

  const meta = () => ({
    author: google.user?.name || google.user?.email || "",
    changed: stampNow(),
    headers: { author: t.qe_col_author, changed: t.qe_col_changed },
  });

  const save = async () => {
    const sel = selection;
    setBusy("save");
    setNotice(null);
    try {
      let target;
      const fresh = await commitEdit(quiz, (sheet) => {
        if (sel.kind === "edit") {
          target = findBlock(sheet, sel.rows, sel);
          if (!target) throw new ConflictError();
          return planSave(sheet, target.ri, target.qi, draft, meta());
        }
        if (sel.kind === "add") {
          const ri = findRound(sheet, sel.marker);
          if (ri < 0) throw new ConflictError();
          target = { ri, qi: sheet.rounds[ri].questions.length };
          return planAdd(sheet, ri, draft, meta());
        }
        target = { ri: sheet.rounds.length, qi: 0 };
        return planAddRound(sheet, roundTitle, draft, meta());
      });
      adopt(fresh);
      const block = fresh.sheet.rounds[target.ri]?.questions[target.qi];
      if (block) choose({ kind: "edit", ...target, rows: block.rows }, draftFrom(block.question));
      setNotice({ ok: true, text: t.qe_saved });
    } catch (err) {
      if (err instanceof ConflictError && err.data) {
        // Keep the draft; aim the next save at the newer version, which replaces it
        adopt(err.data);
        const block = sel.kind === "edit" ? err.data.sheet.rounds[sel.ri]?.questions[sel.qi] : null;
        if (block) setSelection({ ...sel, rows: block.rows });
        setTheirs(block);
        setNotice({ ok: false, text: sel.kind === "edit" ? t.qe_conflict : t.qe_conflict_round });
      } else {
        setNotice({ ok: false, text: errorText(err) });
      }
    }
    setBusy(null);
  };

  const remove = async () => {
    if (!window.confirm(t.qe_delete_confirm)) return;
    const sel = selection;
    setBusy("delete");
    try {
      const fresh = await commitEdit(quiz, (sheet) => {
        const found = findBlock(sheet, sel.rows, sel);
        if (!found) throw new ConflictError();
        return planDelete(sheet, found.ri, found.qi, meta());
      });
      adopt(fresh);
      choose(null, null);
    } catch (err) {
      if (err instanceof ConflictError && err.data) adopt(err.data);
      setNotice({ ok: false, text: err instanceof ConflictError ? t.qe_conflict : errorText(err) });
    }
    setBusy(null);
  };

  const rename = async () => {
    const { marker, title } = renaming;
    setBusy("rename");
    try {
      const fresh = await commitEdit(quiz, (sheet) => {
        const ri = findRound(sheet, marker);
        if (ri < 0) throw new ConflictError();
        return planRename(sheet, ri, title, meta());
      });
      adopt(fresh);
      setRenaming(null);
    } catch (err) {
      if (err instanceof ConflictError && err.data) adopt(err.data);
      setNotice({ ok: false, text: err instanceof ConflictError ? t.qe_conflict_round : errorText(err) });
      setRenaming(null);
    }
    setBusy(null);
  };

  // Ctrl+S saves
  const saveRef = useRef(null);
  useEffect(() => { saveRef.current = draft && dirty && canEdit && !busy ? save : null; });
  useEffect(() => {
    const onKey = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        saveRef.current?.();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Leaving with unsaved changes
  useEffect(() => {
    if (!dirty) return;
    const onLeave = (e) => { e.preventDefault(); e.returnValue = ""; };
    window.addEventListener("beforeunload", onLeave);
    return () => window.removeEventListener("beforeunload", onLeave);
  }, [dirty]);

  // The draft as the show will render it, and what the check would say about it
  const selectedRound = selection?.kind === "round" ? null : rounds[selection?.ri];
  const previewRound = useMemo(() => {
    if (!draft) return null;
    let question;
    try { question = previewQuestion(draft); } catch { return null; }
    if (!question) return null;
    // A new question gets the number after the round's last row
    const last = selectedRound?.questions.at(-1);
    const number = selection?.kind === "edit"
      ? rounds[selection.ri]?.questions[selection.qi]?.question.id
      : last ? String(Number(last.rows.at(-1)[1]) + 1) : null;
    return { id: "editor", number: selectedRound?.number ?? "", title: selection?.kind === "round" ? roundTitle : selectedRound?.title ?? "", questions: [{ ...question, id: number ?? question.id }] };
  }, [draft, selection, rounds, selectedRound, roundTitle]);
  const issues = useMemo(() => (previewRound
    ? validateQuiz({ teams: [], scoresSheetUrl: "-" }, [previewRound], hasMediaLive).filter((i) => i.qIndex === 0 && i.code !== "duplicate_number")
    : []), [previewRound, hasMediaLive]);
  const context = useMemo(() => ({ quiz, rounds: previewRound ? [previewRound] : [], teams: [], resolveMedia: resolveLive }), [quiz, previewRound, resolveLive]);

  // PImage and PVideo show their media alone first
  const hasMediaStep = draft?.type === "PImage" || draft?.type === "PVideo";
  const shownView = view === "media" && !hasMediaStep ? "question" : view;

  const answerText = (q) => (q.type === "Sort" ? q.items.map((it) => it.letter).join(" ") : [q.answer].flat().filter(Boolean).join(", "));
  const label = (block) => block.question.text?.trim() || block.question.media || block.question.items?.map((it) => it.text).filter(Boolean).join(", ") || "—";

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <header className="sticky top-0 z-30 bg-[#050505]/95 backdrop-blur border-b border-white/10">
        <div className="max-w-[1800px] mx-auto px-6 py-4 flex flex-wrap items-center gap-x-4 gap-y-2">
          <a href={quizHash(quiz.id, "edit")} onClick={(e) => !confirmDiscard() && e.preventDefault()} className="flex items-center gap-2 text-gray-400 hover:text-white">
            <ArrowLeft size={18} /> {t.qe_back}
          </a>
          <h1 className="font-['League_Spartan'] text-2xl font-black truncate">{t.qe_title}: {quiz.title || t.ed_untitled}</h1>
          <BetaBadge t={t} />
          <div className="ml-auto flex flex-wrap items-center gap-4 text-sm">
            <a href={sheetUrl(quiz.drive.spreadsheetId)} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 text-gray-400 hover:text-white">
              <FileSpreadsheet size={16} /> {quiz.drive.spreadsheetName}
            </a>
            {loadedAt && (
              <button onClick={load} className="flex items-center gap-1.5 text-gray-500 hover:text-white" title={t.qe_reload}>
                <RefreshCw size={14} /> {fmt(t.qe_loaded, { time: loadedAt.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) })}
              </button>
            )}
          </div>
        </div>
      </header>

      <div className="max-w-[1800px] mx-auto px-6 py-6 space-y-4">
        {!google.signedIn && (
          <div className="rounded-2xl bg-white/[0.04] border border-white/10 p-5 space-y-3">
            <p className="text-gray-300">{t.qe_signin}</p>
            <GoogleAccountBar account={google} t={t} onError={(text) => text && setLoadError(text)} />
          </div>
        )}
        {loadError && <Banner tone="error" text={loadError} />}
        {!canEdit && <Banner tone="info" icon={Lock} text={t.qe_readonly} />}

        {data && (
          <div className="grid lg:grid-cols-[minmax(280px,380px)_1fr] gap-6 items-start">
            <nav className="space-y-4 lg:sticky lg:top-24 lg:max-h-[calc(100vh-7rem)] lg:overflow-y-auto pr-1">
              {rounds.map((round, ri) => (
                <section key={`${round.marker}-${ri}`} className="rounded-2xl bg-white/[0.04] border border-white/10 overflow-hidden">
                  <div className="flex items-center gap-2 px-4 py-3 border-b border-white/5">
                    {renaming?.marker === round.marker ? (
                      <form className="flex-1 flex gap-2" onSubmit={(e) => { e.preventDefault(); rename(); }}>
                        <span className="py-2 text-sm text-gray-500 whitespace-nowrap">{round.word} {round.number}</span>
                        <input value={renaming.title} onChange={(e) => setRenaming({ ...renaming, title: e.target.value })} autoFocus
                          onKeyDown={(e) => e.key === "Escape" && setRenaming(null)} className={`${inputCls} py-2 text-sm`} />
                        <button type="submit" disabled={busy === "rename"} className="p-2 rounded-lg bg-yellow-500 text-black" aria-label={t.qe_save}>
                          {busy === "rename" ? <Loader2 size={16} className="animate-spin" /> : <CheckCircle2 size={16} />}
                        </button>
                      </form>
                    ) : (
                      <>
                        <h2 className="flex-1 min-w-0 font-bold truncate"><span className="text-gray-500">{round.number}.</span> {round.title || "—"}</h2>
                        {canEdit && (
                          <button onClick={() => setRenaming({ marker: round.marker, title: round.title })} className="p-1.5 rounded-lg text-gray-500 hover:text-white hover:bg-white/10" aria-label={t.qe_rename}>
                            <Pencil size={14} />
                          </button>
                        )}
                      </>
                    )}
                  </div>
                  <ul>
                    {round.questions.map((block, qi) => {
                      const active = selection?.kind === "edit" && selection.ri === ri && selection.qi === qi;
                      return (
                        <li key={block.start}>
                          <button onClick={() => openQuestion(ri, qi)}
                            className={`w-full text-left px-4 py-2.5 flex gap-3 border-l-2 ${active ? "bg-yellow-500/10 border-yellow-500" : "border-transparent hover:bg-white/5"}`}>
                            <span className="w-6 shrink-0 text-right font-mono text-sm text-gray-500">{block.question.id}</span>
                            <span className="min-w-0 flex-1">
                              <span className="block truncate text-sm">{label(block)}</span>
                              <span className="block text-xs text-gray-500 truncate">
                                {block.question.type}
                                {block.author && ` · ${block.author}`}
                                {block.changed && ` · ${ago(block.changed, lang)}`}
                              </span>
                            </span>
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                  {canEdit && (
                    <button onClick={() => openAdd(ri)}
                      className={`w-full flex items-center gap-2 px-4 py-2.5 text-sm border-t border-white/5 ${selection?.kind === "add" && selection.ri === ri ? "text-yellow-400 bg-yellow-500/10" : "text-gray-400 hover:text-white hover:bg-white/5"}`}>
                      <Plus size={16} /> {t.qe_add_question}
                    </button>
                  )}
                </section>
              ))}
              {canEdit && (
                <button onClick={openAddRound} className={`${btnCls} w-full ${selection?.kind === "round" ? "ring-1 ring-yellow-500" : ""}`}>
                  <Plus size={16} /> {t.qe_add_round}
                </button>
              )}
            </nav>

            <main className="min-w-0">
              {!draft ? (
                <div className="rounded-2xl border-2 border-dashed border-white/10 p-12 text-center text-gray-500">{t.qe_pick}</div>
              ) : (
                <div className="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-5">
                  <div className="flex flex-wrap items-center gap-3">
                    <h2 className="font-['League_Spartan'] text-xl font-bold uppercase tracking-widest mr-auto">
                      {selection.kind === "edit"
                        ? `${selectedRound?.number}. ${selectedRound?.title || ""} · ${fmt(t.qe_number, { n: previewRound?.questions[0].id ?? "" })}`
                        : selection.kind === "add" ? `${selectedRound?.number}. ${selectedRound?.title || ""} · ${t.qe_new_question}` : t.qe_new_round}
                    </h2>
                    {selection.kind === "edit" && (() => {
                      const block = rounds[selection.ri]?.questions[selection.qi];
                      return block?.author ? <span className="text-xs text-gray-500">{fmt(t.qe_by, { name: block.author, time: ago(block.changed, lang) })}</span> : null;
                    })()}
                    <button onClick={close} className="p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label={t.qe_cancel}><X size={18} /></button>
                  </div>

                  {theirs && (
                    <div className="rounded-xl bg-yellow-500/10 border border-yellow-500/30 p-4 space-y-2 text-sm">
                      <p className="flex items-start gap-2 text-yellow-200">
                        <AlertTriangle size={16} className="shrink-0 mt-0.5" />
                        {fmt(t.qe_changed_by, { name: theirs.author || t.qe_someone, time: theirs.changed ? ago(theirs.changed, lang) : "" })}
                      </p>
                      <p className="text-gray-300 pl-6 truncate">“{label(theirs)}”</p>
                      {answerText(theirs.question) && <p className="text-gray-400 pl-6 truncate">{t.qe_answer}: {answerText(theirs.question)}</p>}
                      <div className="flex flex-wrap gap-2 pl-6">
                        <button onClick={() => {
                          const ri = selection.ri;
                          const qi = selection.qi;
                          choose({ kind: "edit", ri, qi, rows: theirs.rows }, draftFrom(theirs.question));
                        }} className="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20">{t.qe_load_theirs}</button>
                        <span className="py-1.5 text-gray-400">{t.qe_keep_mine}</span>
                      </div>
                    </div>
                  )}

                  <div className="grid 2xl:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-6 items-start">
                    <div className="space-y-5 min-w-0">
                      {selection.kind === "round" && (
                        <label className="block">
                          <span className="block text-xs font-bold uppercase tracking-wider text-gray-400 mb-1.5">{t.qe_round_title}</span>
                          <input value={roundTitle} onChange={(e) => { setRoundTitle(e.target.value); setDirty(true); }} className={inputCls} autoFocus />
                        </label>
                      )}
                      <QuestionForm draft={draft} onChange={change} mediaNames={mediaNames} hasMedia={hasMediaLive} upload={canEdit ? upload : null} typeHelp={typeHelp} disabled={!canEdit || Boolean(busy)} t={t} />
                    </div>

                    <div className="space-y-3 min-w-0 2xl:sticky 2xl:top-24">
                      <div className="flex items-center gap-3">
                        <span className="text-xs font-bold uppercase tracking-wider text-gray-400 mr-auto">{t.qe_preview}</span>
                        <div className="flex gap-1 bg-white/5 rounded-xl p-1 text-sm">
                          {(hasMediaStep ? ["media", "question", "answer"] : ["question", "answer"]).map((v) => (
                            <button key={v} onClick={() => setView(v)} className={`px-3 py-1 rounded-lg ${shownView === v ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>
                              {v === "media" ? t.qe_view_media : v === "question" ? t.chk_view_question : t.chk_view_answer}
                            </button>
                          ))}
                        </div>
                      </div>
                      <QuizContext.Provider value={context}>
                        <MotionConfig reducedMotion="always">
                          <StageFrame display={display} className="rounded-xl ring-1 ring-white/15 pointer-events-none">
                            <PreviewBoundary resetKey={JSON.stringify(draft)}>
                              {previewRound && (
                                <QuestionScreen key={`${draft.type}-${shownView}`} roundData={previewRound} mode="with_answers" onBack={() => {}} isPresenter={false} t={showT}
                                  preview={{ qIndex: 0, step: shownView === "media" ? 1 : previewStep(draft.type, shownView) }} />
                              )}
                            </PreviewBoundary>
                          </StageFrame>
                        </MotionConfig>
                      </QuizContext.Provider>
                      {issues.length > 0 && (
                        <ul className="space-y-1 text-sm">
                          {issues.map((i, n) => (
                            <li key={n} className={`flex items-start gap-2 ${i.level === "error" ? "text-red-300" : "text-yellow-300"}`}>
                              <AlertTriangle size={14} className="shrink-0 mt-0.5" /> {fmt(t[`chk_${i.code}`] || i.code, i.params)}
                            </li>
                          ))}
                        </ul>
                      )}
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center gap-3 pt-2 border-t border-white/5">
                    <button onClick={save} disabled={!canEdit || Boolean(busy) || (!dirty && selection.kind === "edit")}
                      className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
                      {busy === "save" ? <Loader2 size={16} className="animate-spin" /> : <Save size={16} />}
                      {busy === "save" ? t.qe_saving : t.qe_save}
                    </button>
                    <button onClick={close} className={btnCls}>{t.qe_cancel}</button>
                    {notice && (
                      <span className={`flex items-center gap-1.5 text-sm ${notice.ok ? "text-green-300" : "text-red-300"}`}>
                        {notice.ok ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />} {notice.text}
                      </span>
                    )}
                    <span className="text-xs text-gray-600 hidden md:inline">{t.qe_keys}</span>
                    {selection.kind === "edit" && canEdit && (
                      <button onClick={remove} disabled={Boolean(busy)} className="ml-auto flex items-center gap-2 px-4 py-3 rounded-xl text-red-300 hover:bg-red-500/20 text-sm">
                        {busy === "delete" ? <Loader2 size={16} className="animate-spin" /> : <Trash2 size={16} />} {t.qe_delete}
                      </button>
                    )}
                  </div>
                </div>
              )}
            </main>
          </div>
        )}

        {google.signedIn && !data && !loadError && (
          <div className="flex items-center justify-center gap-3 p-16 text-gray-400"><Loader2 className="animate-spin" /> {t.qe_loading}</div>
        )}
      </div>
    </div>
  );
}

function Banner({ tone, text, icon }) {
  const Icon = icon || AlertTriangle;
  return (
  <p className={`flex items-start gap-2 p-4 rounded-xl text-sm ${tone === "error" ? "bg-red-500/10 border border-red-500/30 text-red-200" : "bg-blue-500/10 border border-blue-500/20 text-blue-200"}`}>
    <Icon size={16} className="shrink-0 mt-0.5" /> {text}
  </p>
  );
}
