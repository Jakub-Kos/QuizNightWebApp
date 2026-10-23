import { useState, useEffect, useCallback, useRef } from "react";
import { Plus, Play, Pencil, Copy, Download, Trash2, FileDown, Loader2, AlertTriangle, ScanSearch, HelpCircle, HardDrive, Link2, MailOpen, LogIn, RefreshCw, X, PackageOpen, Compass, Palette } from "lucide-react";
import { listQuizzes, saveQuiz, updateQuiz, deleteQuiz, requestPersistence } from "../quiz/storage";
import { createQuiz, loadQuizBundle, DEMO_ID } from "../quiz/model";
import { parseQuestionsCsv } from "../quiz/parse";
import { readImport, saveImport, exportZip, duplicateQuiz, downloadBlob, safeFileName } from "../quiz/package";
import { quizHash, navigate } from "../hooks/useHashRoute";
import FileDrop from "./FileDrop";
import ShowLauncher from "./ShowLauncher";
import { startTour } from "../quiz/tour";
import { useGoogleAccount } from "../hooks/useGoogleAccount";
import { readDriveQuiz, mergeDrive, sameDriveQuiz, driveErrorText, createDriveQuiz, folderUrl } from "../quiz/drive";
import { GOOGLE_SHEET_TEMPLATE } from "../config";
import { GoogleAccountBar, BetaBadge } from "./DriveParts";
import { hasDriveInvite, acceptDriveInvite } from "../quiz/google";
import { Status, inputCls, btnCls } from "./EditorParts";
import { fmt } from "../quiz/files";

const LANGUAGES = [["cs", "Čeština"], ["sk", "Slovenčina"], ["en", "English"]];
const WELCOME_KEY = "quiznight.welcomeDismissed";

function summarize(quiz) {
  const rounds = parseQuestionsCsv(quiz.questionsCsv);
  return { rounds: rounds.length, questions: rounds.reduce((n, r) => n + r.questions.length, 0) };
}

export default function Library({ t, lang, onLanguage, openDrive = false, driveFolder = null }) {
  const [quizzes, setQuizzes] = useState(null);
  const [demo, setDemo] = useState(null);
  const [busy, setBusy] = useState(null); // id of the card (or "import") with an action running
  const [confirmDelete, setConfirmDelete] = useState(null);
  const [launching, setLaunching] = useState(null); // id of the quiz whose show is being started
  const [welcomeDismissed, setWelcomeDismissed] = useState(() => {
    try { return localStorage.getItem(WELCOME_KEY) === "1"; } catch { return false; }
  });
  const dismissWelcome = () => {
    setWelcomeDismissed(true);
    try { localStorage.setItem(WELCOME_KEY, "1"); } catch { /* shown again next visit */ }
  };
  const [error, setError] = useState(null);
  const google = useGoogleAccount();
  const [driveInvited] = useState(() => {
    if (openDrive && google.enabled) acceptDriveInvite();
    return hasDriveInvite();
  });
  // A shared quiz link shows the invitation instead of the panel
  const [driveOpen, setDriveOpen] = useState(openDrive && driveInvited && !driveFolder);

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
          <a href="#/themes" className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
            <Palette size={16} /> {t.ws_title}
          </a>
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

        {quizzes?.length === 0 && !welcomeDismissed && (
          <Welcome onDemo={() => setLaunching(DEMO_ID)} onNew={handleNew} onClose={dismissWelcome} t={t} />
        )}

        <section className="grid md:grid-cols-[1fr_2fr] gap-4">
          <div className="flex flex-col gap-3">
            <button onClick={handleNew} disabled={busy === "new"}
              className="flex-1 flex flex-col items-center justify-center gap-3 p-8 rounded-2xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-lg transition-colors">
              <Plus size={36} /> {t.lib_new}
            </button>
            {driveInvited && (
              <button onClick={() => setDriveOpen((v) => !v)}
                className={`flex items-center justify-center gap-2 p-3 rounded-xl text-sm text-gray-300 ${driveOpen ? "bg-white/10" : "bg-white/5 hover:bg-white/10"}`}>
                <HardDrive size={16} /> {t.gd_open} <BetaBadge t={t} />
              </button>
            )}
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

        {launching && <ShowLauncher quizId={launching} onClose={() => setLaunching(null)} t={t} />}
        {driveOpen && <DriveOpen account={google} quizzes={quizzes} lang={lang} t={t} />}
        {driveFolder && google.enabled && <InviteScreen folder={driveFolder} account={google} quizzes={quizzes} lang={lang} t={t} />}

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
                    {quiz.drive && (
                      <span className="shrink-0 flex items-center gap-1 text-xs font-bold uppercase tracking-wider px-2 py-1 rounded bg-green-500/15 text-green-300">
                        <HardDrive size={12} /> {t.gd_badge}
                      </span>
                    )}
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
                      <button onClick={() => setLaunching(quiz.id)} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
                        <Play size={16} /> {t.lib_open}
                      </button>
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

// First visit (no quizzes of their own yet): the three ways in, until dismissed or a quiz exists
function Welcome({ onDemo, onNew, onClose, t }) {
  const card = (icon, title, text, action) => (
    <div className="rounded-xl bg-black/30 border border-white/10 p-5 flex flex-col gap-3">
      <div className="w-10 h-10 rounded-xl bg-yellow-500/15 text-yellow-400 flex items-center justify-center">{icon}</div>
      <h3 className="font-bold text-lg">{title}</h3>
      <p className="text-sm text-gray-400 flex-1">{text}</p>
      {action}
    </div>
  );
  return (
    <section className="relative rounded-2xl bg-gradient-to-br from-yellow-500/10 to-transparent border border-yellow-500/30 p-6 space-y-5">
      <button onClick={onClose} className="absolute top-4 right-4 p-2 rounded-full text-gray-400 hover:text-white hover:bg-white/10" aria-label={t.chk_close}><X size={18} /></button>
      <div className="space-y-2 pr-10">
        <h2 className="font-['League_Spartan'] text-2xl font-bold uppercase tracking-widest">{t.wl_title}</h2>
        <p className="text-gray-300 max-w-2xl">{t.wl_text}</p>
      </div>
      <div className="grid md:grid-cols-3 gap-3">
        {card(<Play size={20} />, t.wl_demo_title, t.wl_demo_text, (
          <div className="flex flex-col gap-2">
            <button onClick={() => startTour(DEMO_ID)} className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm"><Compass size={16} /> {t.tr_start}</button>
            <button onClick={onDemo} className="text-sm text-gray-400 hover:text-white">{t.wl_demo_button}</button>
          </div>
        ))}
        {card(<Plus size={20} />, t.wl_new_title, t.wl_new_text,
          <button onClick={onNew} className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-white/10 hover:bg-white/20 font-bold text-sm"><Plus size={16} /> {t.lib_new}</button>)}
        {card(<PackageOpen size={20} />, t.wl_import_title, t.wl_import_text, null)}
      </div>
      <a href="#/help/start" className="inline-flex items-center gap-1.5 text-sm text-gray-400 hover:text-yellow-400"><HelpCircle size={14} /> {t.wl_help}</a>
    </section>
  );
}

// Paste a Drive folder or Sheet link: the quiz is created in the library (or updated when it is already
// there) and opened in the setup page, which downloads the media
// Add a Drive quiz to the library (or update the copy already there) and open its setup page
async function openDriveQuiz(link, quizzes, lang) {
  const fields = await readDriveQuiz(link);
  const existing = quizzes.find((q) => sameDriveQuiz(q.drive, fields.drive));
  if (existing) {
    await updateQuiz(existing.id, (q) => ({ ...q, ...mergeDrive(q, fields) }));
    navigate(quizHash(existing.id, "edit"));
    return;
  }
  const quiz = createQuiz({ title: fields.drive.folderName, ...fields, settings: { language: lang } });
  await saveQuiz(quiz);
  requestPersistence();
  navigate(quizHash(quiz.id, "edit"));
}

// The page a shared quiz link (#/drive/<folder id>) opens: an invitation over everything that asks to
// sign in with Google, then opens the quiz by itself. A browser that already has the quiz goes straight to it.
function InviteScreen({ folder, account, quizzes, lang, t }) {
  const [phase, setPhase] = useState("signin"); // "signin" | "opening" | "error"
  const [error, setError] = useState(null);
  const [signingIn, setSigningIn] = useState(false);
  const started = useRef(false);

  const open = async () => {
    started.current = true;
    setPhase("opening");
    setError(null);
    try {
      await openDriveQuiz(folderUrl(folder), quizzes || [], lang);
    } catch (err) {
      setError(driveErrorText(err, t, account.user?.email));
      setPhase("error");
    }
  };

  useEffect(() => {
    if (!quizzes || started.current) return;
    const existing = quizzes.find((q) => q.drive?.folderId === folder);
    if (!existing && !account.signedIn) return;
    const timer = setTimeout(() => {
      started.current = true;
      if (existing) navigate(quizHash(existing.id, "edit"));
      else open();
    });
    return () => clearTimeout(timer);
  }, [quizzes, account.signedIn, folder]); // eslint-disable-line react-hooks/exhaustive-deps

  const signInWith = async (chooseAccount) => {
    setSigningIn(true);
    setError(null);
    try {
      await account.signIn({ chooseAccount });
      if (phase === "error") open(); // after picking another account; the first sign-in opens by itself
    } catch (err) {
      const text = driveErrorText(err, t, account.user?.email);
      if (text) { setError(text); setPhase("error"); }
    }
    setSigningIn(false);
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/85 backdrop-blur-sm flex items-center justify-center p-4 font-sans">
      <div className="w-full max-w-md rounded-3xl bg-[#0f0f0f] border border-white/10 shadow-2xl p-8 space-y-6 text-center">
        <div className="mx-auto w-16 h-16 rounded-2xl bg-yellow-500/15 text-yellow-400 flex items-center justify-center"><MailOpen size={32} /></div>
        <div className="space-y-2">
          <h2 className="font-['League_Spartan'] text-3xl font-black uppercase tracking-wide">{t.inv_title}</h2>
          <p className="text-gray-300">{phase === "opening" ? t.inv_opening_text : t.inv_text}</p>
        </div>

        {phase === "opening" && (
          <div className="flex items-center justify-center gap-3 text-gray-300"><Loader2 className="animate-spin" /> {t.inv_opening}</div>
        )}

        {phase === "signin" && (
          <button onClick={() => signInWith(false)} disabled={signingIn}
            className="w-full flex items-center justify-center gap-3 px-6 py-4 rounded-2xl bg-white hover:bg-gray-200 text-black text-lg font-bold disabled:opacity-60">
            {signingIn ? <Loader2 className="animate-spin" /> : <LogIn size={22} />} {t.gd_sign_in}
          </button>
        )}

        {phase === "error" && (
          <div className="space-y-4 text-left">
            <Status status={{ ok: false, text: error }} />
            {account.user && <p className="text-sm text-gray-400">{fmt(t.inv_signed_as, { email: account.user.email })}</p>}
            <div className="flex flex-col gap-2">
              <button onClick={() => signInWith(true)} disabled={signingIn}
                className="flex items-center justify-center gap-2 px-5 py-3 rounded-xl bg-white hover:bg-gray-200 text-black font-bold disabled:opacity-60">
                {signingIn ? <Loader2 size={18} className="animate-spin" /> : <LogIn size={18} />} {t.inv_other_account}
              </button>
              {account.signedIn && <button onClick={open} className={btnCls}><RefreshCw size={16} /> {t.inv_retry}</button>}
            </div>
          </div>
        )}

        <div className="space-y-3 pt-2 border-t border-white/5">
          <p className="text-xs text-gray-500">{t.inv_note}</p>
          <a href="#/" className="inline-block text-sm text-gray-400 hover:text-white">{t.inv_close}</a>
        </div>
      </div>
    </div>
  );
}

function DriveOpen({ account, quizzes: loaded, lang, t }) {
  const quizzes = loaded || [];
  const [mode, setMode] = useState("open"); // "open" an existing folder, or "create" a new one
  const [link, setLink] = useState("");
  const [title, setTitle] = useState("");
  const [parent, setParent] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(null);

  const open = async (folderLink = link) => {
    setBusy(true);
    setError(null);
    try {
      await openDriveQuiz(folderLink, quizzes, lang);
    } catch (err) {
      setError(driveErrorText(err, t, account.user?.email));
      setBusy(false);
    }
  };

  const create = async () => {
    setBusy(true);
    setError(null);
    try {
      const folderId = await createDriveQuiz({ title, parentLink: parent, templateUrl: GOOGLE_SHEET_TEMPLATE });
      await open(folderUrl(folderId));
    } catch (err) {
      setError(driveErrorText(err, t, account.user?.email));
      setBusy(false);
    }
  };

  return (
    <section className="rounded-2xl bg-white/[0.04] border border-white/10 p-6 space-y-4">
      <div>
        <h2 className="flex items-center gap-2 font-['League_Spartan'] text-xl font-bold uppercase tracking-widest">
          <HardDrive size={20} /> {t.gd_open} <BetaBadge t={t} />
        </h2>
        <p className="text-sm text-gray-400 mt-1">{t.gd_open_help}</p>
        <p className="text-xs text-gray-500 mt-2">{t.gd_beta_note}</p>
      </div>
      <GoogleAccountBar account={account} t={t} onError={setError} />
      {account.signedIn && GOOGLE_SHEET_TEMPLATE && (
        <div className="flex gap-1 bg-white/5 rounded-xl p-1 w-fit text-sm">
          {[["open", t.gd_mode_open], ["create", t.gd_mode_create]].map(([key, label]) => (
            <button key={key} onClick={() => { setMode(key); setError(null); }}
              className={`px-3 py-1.5 rounded-lg ${mode === key ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>{label}</button>
          ))}
        </div>
      )}
      {account.signedIn && mode === "create" && (
        <form onSubmit={(e) => { e.preventDefault(); create(); }} className="space-y-2">
          <div className="flex flex-col sm:flex-row gap-2">
            <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder={t.gd_new_title} className={inputCls} autoFocus required />
            <button type="submit" disabled={!title.trim() || busy} className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
              {busy ? <Loader2 size={16} className="animate-spin" /> : <Plus size={16} />} {t.gd_create}
            </button>
          </div>
          <div className="relative">
            <Link2 size={16} className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" />
            <input value={parent} onChange={(e) => setParent(e.target.value)} placeholder={t.gd_new_parent} className={`${inputCls} pl-10`} />
          </div>
          <p className="text-xs text-gray-500">{t.gd_create_help}</p>
        </form>
      )}
      {account.signedIn && mode === "open" && (
        <form onSubmit={(e) => { e.preventDefault(); open(); }} className="flex flex-col sm:flex-row gap-2">
          <div className="relative flex-1">
            <Link2 size={16} className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" />
            <input value={link} onChange={(e) => setLink(e.target.value)} placeholder={t.gd_link} className={`${inputCls} pl-10`} autoFocus />
          </div>
          <button type="submit" disabled={!link.trim() || busy} className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
            {busy ? <Loader2 size={16} className="animate-spin" /> : <HardDrive size={16} />} {t.gd_connect}
          </button>
        </form>
      )}
      {error && <Status status={{ ok: false, text: error }} />}
    </section>
  );
}
