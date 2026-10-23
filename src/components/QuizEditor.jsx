import { useState, useEffect, useMemo, useRef } from "react";
import { ArrowLeft, Play, Link2, RefreshCw, FileUp, Trash2, Music, Film, AlertTriangle, CheckCircle2, Loader2, ScanSearch, Calculator, HelpCircle, HardDrive, FolderOpen, FileSpreadsheet, ListChecks, Copy, Check, Network } from "lucide-react";
import { getQuiz, updateQuiz, listMedia, putMedia, deleteMedia, onQuizSaved } from "../quiz/storage";
import { syncDriveConfig } from "../quiz/driveConfig";
import { parseQuestionsCsv, parseScoresCsv } from "../quiz/parse";
import { fetchSheetCsv, sheetErrorText } from "../quiz/sheets";
import { referencedMedia, isMediaFile } from "../quiz/package";
import { mediaKey, isUrl } from "../quiz/model";
import { validateQuiz } from "../quiz/validate";
import { formatBytes, fmt } from "../quiz/files";
import { quizHash, navigate } from "../hooks/useHashRoute";
import FileDrop from "./FileDrop";
import { Section, Status, inputCls, btnCls } from "./EditorParts";
import TeamsSection from "./TeamsSection";
import IntroSection from "./IntroSection";
import ThemeSection from "./ThemeSection";
import { themeById } from "../quiz/themes";
import { useUiLanguage } from "../hooks/useUiLanguage";
import { saveTeamImage } from "../quiz/teams";
import { useGoogleAccount } from "../hooks/useGoogleAccount";
import { readDriveQuiz, syncDriveMedia, mergeDrive, driveErrorText, folderUrl, sheetUrl as driveSheetUrl, ensureMediaFolder, uploadMedia, isMediaFolderName } from "../quiz/drive";
import { GoogleAccountBar, DriveProgress, BetaBadge, SaveStatus } from "./DriveParts";
import { useDriveSync } from "../hooks/useDriveSync";
import { teamForScores } from "../quiz/teams";
import DrivePeople from "./DrivePeople";
import ReadyChecklist from "./ReadyChecklist";
import QuestionsStart from "./QuestionsStart";
import { exampleQuestions } from "../quiz/examples";
import ShowLauncher from "./ShowLauncher";
import { openSection } from "./sectionEvents";

export default function QuizEditor({ id, t }) {
  const [quiz, setQuiz] = useState(undefined); // undefined = loading, null = not found
  const [media, setMedia] = useState([]); // { name, blob, url }
  const [saveState, setSaveState] = useState("saved");
  const [sheetUrl, setSheetUrl] = useState("");
  const [questionsStatus, setQuestionsStatus] = useState(null);
  const [scoresStatus, setScoresStatus] = useState(null);
  const [busy, setBusy] = useState(null);
  const [driveStatus, setDriveStatus] = useState(null);
  const [uploadStatus, setUploadStatus] = useState(null);
  const [driveProgress, setDriveProgress] = useState(null);
  const google = useGoogleAccount();
  const [uiLang] = useUiLanguage();
  const sync = useDriveSync(id);
  const [scoresPreview, setScoresPreview] = useState(null); // parseScoresCsv result of the last test
  const csvInput = useRef(null);
  const dirty = useRef(false);
  const [launcher, setLauncher] = useState(false);
  // Scores come from a Google Sheet, or are typed into the score table during the quiz
  const [scoresMode, setScoresMode] = useState("manual");
  const lastScoresUrl = useRef("");

  useEffect(() => {
    getQuiz(id).then((q) => {
      setQuiz(q ?? null);
      setSheetUrl(q?.questionsSheetUrl || "");
      setScoresMode(q?.scoresSheetUrl ? "sheet" : "manual");
    });
  }, [id]);

  // What a Drive sync brought in (setup and teams from the folder) replaces the copy shown here,
  // unless there are edits of our own still waiting to be saved
  useEffect(() => onQuizSaved((saved, source) => {
    if (source === "drive" && saved.id === id && !dirty.current) setQuiz(saved);
  }), [id]);

  // Object URLs for thumbnails; the ref lets them be revoked on reload and unmount
  const mediaUrls = useRef([]);
  const fetchMedia = async () => {
    const list = (await listMedia(id)).sort((a, b) => a.name.localeCompare(b.name));
    mediaUrls.current.forEach((url) => URL.revokeObjectURL(url));
    const withUrls = list.map((m) => ({ ...m, url: URL.createObjectURL(m.blob) }));
    mediaUrls.current = withUrls.map((m) => m.url);
    return withUrls;
  };
  const reloadMedia = async () => setMedia(await fetchMedia());
  useEffect(() => { fetchMedia().then(setMedia); }, [id]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => () => mediaUrls.current.forEach((url) => URL.revokeObjectURL(url)), []);

  // Autosave every change shortly after it happens
  useEffect(() => {
    if (!dirty.current) return;
    // Only the fields edited here; settings and scores are saved by other windows, the Drive link by syncs
    const { title, questionsCsv, questionsSheetUrl, scoresSheetUrl, teams, rules, prizes } = quiz;
    const timer = setTimeout(() => updateQuiz(quiz.id, (q) => ({ ...q, title, questionsCsv, questionsSheetUrl, scoresSheetUrl, teams, rules, prizes })).then(() => {
      dirty.current = false;
      setSaveState("saved");
    }), 400);
    return () => clearTimeout(timer);
  }, [quiz]);

  const update = (fields) => {
    dirty.current = true;
    setSaveState("saving");
    setQuiz((q) => ({ ...q, ...fields }));
  };

  // The theme is one of the show's settings, which the show window saves too: only that key is written
  const setTheme = (theme) => {
    setQuiz((q) => ({ ...q, settings: { ...q.settings, theme } }));
    updateQuiz(id, (q) => ({ ...q, settings: { ...q.settings, theme } }));
  };

  const rounds = useMemo(() => (quiz ? parseQuestionsCsv(quiz.questionsCsv) : []), [quiz]);
  const questionCount = rounds.reduce((n, r) => n + r.questions.length, 0);
  const referenced = useMemo(() => (quiz ? referencedMedia(quiz) : []), [quiz]);
  const mediaKeys = useMemo(() => new Set(media.map((m) => mediaKey(m.name))), [media]);
  const missing = referenced.filter((name) => !mediaKeys.has(mediaKey(name)));
  const usedKeys = new Set(referenced.map(mediaKey));
  const mediaUrlByKey = useMemo(() => new Map(media.map((m) => [mediaKey(m.name), m.url])), [media]);
  // The same issues the Check page lists, for the checklist
  const issues = useMemo(() => (quiz ? validateQuiz(quiz, rounds, (name) => isUrl(name) || mediaKeys.has(mediaKey(name))) : []), [quiz, rounds, mediaKeys]);

  const loadSheet = async () => {
    setBusy("sheet");
    try {
      const csv = await fetchSheetCsv(sheetUrl);
      const parsed = parseQuestionsCsv(csv);
      update({ questionsCsv: csv, questionsSheetUrl: sheetUrl.trim() });
      setQuestionsStatus(parsed.length
        ? { ok: true, text: `${t.ed_loaded}: ${parsed.length} ${t.lib_rounds}` }
        : { ok: false, text: t.ed_no_rounds });
    } catch (err) {
      setQuestionsStatus({ ok: false, text: sheetErrorText(err, t) });
    }
    setBusy(null);
  };

  // Questions, tabs and the media folder again from Drive, then the media files that changed
  const syncDrive = async () => {
    setBusy("drive");
    setDriveStatus(null);
    try {
      // Questions and tabs first (the rest needs them), then the setup and the media side by side
      const fields = await readDriveQuiz(quiz.drive.folderId || quiz.drive.spreadsheetId);
      await updateQuiz(id, (q) => ({ ...q, ...mergeDrive(q, fields) }), "drive");
      const [, result] = await Promise.all([syncDriveConfig(id), syncDriveMedia(id, fields.drive.mediaFolderId, setDriveProgress)]);
      await updateQuiz(id, (q) => ({ ...q, drive: { ...q.drive, syncedAt: Date.now(), syncedBy: google.user?.name || "" } }), "drive");
      await reloadMedia();
      const text = [
        fmt(t.gd_result, result),
        result.duplicates.length ? fmt(t.gd_duplicates, { names: result.duplicates.join(", ") }) : "",
      ].filter(Boolean).join(" ");
      setDriveStatus({ ok: !result.duplicates.length, text });
    } catch (err) {
      setDriveStatus({ ok: false, text: driveErrorText(err, t, google.user?.email) });
    }
    setDriveProgress(null);
    setBusy(null);
  };

  // Every visit starts from what is on Drive now (only changed media is downloaded)
  const autoSynced = useRef(false);
  useEffect(() => {
    if (!quiz?.drive || !google.signedIn || autoSynced.current) return;
    const timer = setTimeout(() => { autoSynced.current = true; syncDrive(); });
    return () => clearTimeout(timer);
  }, [quiz, google.signedIn]); // eslint-disable-line react-hooks/exhaustive-deps

  const uploadCsv = async (e) => {
    const file = e.target.files[0];
    e.target.value = "";
    if (!file) return;
    const csv = await file.text();
    const parsed = parseQuestionsCsv(csv);
    update({ questionsCsv: csv, questionsSheetUrl: "" });
    setSheetUrl("");
    setQuestionsStatus(parsed.length
      ? { ok: true, text: `${t.ed_loaded}: ${parsed.length} ${t.lib_rounds}` }
      : { ok: false, text: t.ed_no_rounds });
  };

  // A first round with an example of every question type, and the demo files those use
  const loadExamples = async () => {
    setBusy("examples");
    try {
      const { csv, media: names } = exampleQuestions(quiz.settings?.language || "cs", t.ed_example_round);
      const files = await Promise.all(names.map(async (name) => {
        const res = await fetch(`${import.meta.env.BASE_URL}source/${name}`);
        if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
        return { name, blob: await res.blob() };
      }));
      await putMedia(id, files);
      await reloadMedia();
      update({ questionsCsv: csv, questionsSheetUrl: "" });
      setSheetUrl("");
      setQuestionsStatus({ ok: true, text: t.qs_examples_done });
    } catch (err) {
      setQuestionsStatus({ ok: false, text: String(err.message || err) });
    }
    setBusy(null);
  };

  // Drive quizzes: dropped files go into the media folder on Drive (a dropped folder keeps its
  // subfolders; a dropped Zdroje folder itself is not repeated), one after another
  const uploadToDrive = async (files) => {
    setBusy("drive-upload");
    setUploadStatus(null);
    const progress = { done: 0, total: files.length, bytes: 0, totalBytes: files.reduce((n, f) => n + f.size, 0) };
    setDriveProgress({ ...progress });
    let replace = null; // asked once for the whole drop
    try {
      let folder = quiz.drive.mediaFolderId ? { id: quiz.drive.mediaFolderId } : null;
      if (!folder) {
        folder = await ensureMediaFolder(quiz.drive);
        await updateQuiz(id, (q) => ({ ...q, drive: { ...q.drive, mediaFolderId: folder.id, mediaFolderName: folder.name } }), "drive");
      }
      for (const file of files) {
        const parts = String(file.relativePath || file.webkitRelativePath || file.name).split("/").filter(Boolean);
        if (parts.length > 1 && isMediaFolderName(parts[0])) parts.shift();
        const before = progress.bytes;
        await uploadMedia(id, folder.id, parts.join("/"), file, {
          onProgress: (share) => { progress.bytes = before + share * file.size; setDriveProgress({ ...progress }); },
          confirmReplace: () => (replace ??= window.confirm(t.ed_drive_replace)),
        });
        progress.done++;
        progress.bytes = before + file.size;
        setDriveProgress({ ...progress });
      }
      setUploadStatus({ ok: true, text: fmt(t.ed_drive_uploaded, { count: files.length }) });
    } catch (err) {
      setUploadStatus({ ok: false, text: driveErrorText(err, t, google.user?.email) });
    }
    await reloadMedia();
    setDriveProgress(null);
    setBusy(null);
  };

  const addMedia = async (files) => {
    if (quiz.drive && google.signedIn) return uploadToDrive(files.filter((f) => isMediaFile(f.name)));
    setBusy("media");
    const accepted = files.filter((f) => isMediaFile(f.name)).map((f) => ({ name: f.name, blob: f }));
    // Replace files that differ only in letter case, so lookups stay unambiguous
    for (const m of media) {
      if (accepted.some((a) => mediaKey(a.name) === mediaKey(m.name) && a.name !== m.name)) await deleteMedia(id, m.name);
    }
    await putMedia(id, accepted);
    await reloadMedia();
    setBusy(null);
  };

  // Team photos are stored as team-<id>.<ext>; a new photo replaces the previous one
  const uploadTeamImage = async (team, file) => {
    const name = await saveTeamImage(id, team, file);
    await reloadMedia();
    return name;
  };

  const removeMedia = async (name) => {
    await deleteMedia(id, name);
    await reloadMedia();
  };

  const chooseScores = (mode) => {
    setScoresMode(mode);
    setScoresStatus(null);
    setScoresPreview(null);
    // Typing points in drops the link (the show would read the Sheet otherwise); it comes back on switching back
    if (mode === "manual" && quiz.scoresSheetUrl) {
      lastScoresUrl.current = quiz.scoresSheetUrl;
      update({ scoresSheetUrl: "" });
    }
    if (mode === "sheet" && !quiz.scoresSheetUrl && lastScoresUrl.current) update({ scoresSheetUrl: lastScoresUrl.current });
  };

  const testScores = async () => {
    setBusy("scores");
    try {
      const parsed = parseScoresCsv(await fetchSheetCsv(quiz.scoresSheetUrl));
      setScoresPreview(parsed);
      // The leaderboard matches Sheet rows to teams (by id, else by name); unmatched rows get no photo or color
      const unmatched = parsed ? parsed.teams.filter((row) => !teamForScores(quiz.teams, row)).map((row) => row.name) : [];
      setScoresStatus(!parsed
        ? { ok: false, text: t.ed_err_format }
        : unmatched.length && quiz.teams.length
          ? { ok: false, text: `${fmt(t.ed_scores_ok, { teams: parsed.teams.length, rounds: parsed.rounds.length })} ${fmt(t.ed_scores_unmatched, { names: unmatched.join(", ") })}` }
          : { ok: true, text: fmt(t.ed_scores_ok, { teams: parsed.teams.length, rounds: parsed.rounds.length }) });
    } catch (err) {
      setScoresStatus({ ok: false, text: sheetErrorText(err, t) });
    }
    setBusy(null);
  };

  // A Drive quiz shows its current scores straight away
  const scoresLoaded = useRef(false);
  useEffect(() => {
    if (!quiz?.drive?.scoresTab || !google.signedIn || scoresLoaded.current) return;
    const timer = setTimeout(() => { scoresLoaded.current = true; testScores(); });
    return () => clearTimeout(timer);
  }, [quiz, google.signedIn]); // eslint-disable-line react-hooks/exhaustive-deps

  if (quiz === undefined) return <div className="min-h-screen bg-[#050505]" />;
  if (quiz === null) {
    return (
      <div className="min-h-screen bg-[#050505] text-white flex flex-col items-center justify-center gap-6">
        <p className="text-xl text-gray-300">{t.ed_not_found}</p>
        <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 text-black font-bold">{t.settings_exit}</a>
      </div>
    );
  }

  const totalSize = media.reduce((n, m) => n + m.blob.size, 0);

  // What still needs doing, per section (outlined in yellow)
  const prizes = { enabled: true, ...quiz.prizes };
  const warnings = {
    drive: quiz.drive && !google.signedIn ? t.ed_warn_signin : null,
    questions: rounds.length === 0 ? t.ed_warn_questions : null,
    media: missing.length ? fmt(t.ed_warn_media, { count: missing.length }) : null,
    scores: scoresMode === "sheet" && !quiz.scoresSheetUrl ? t.ed_warn_scores : null,
    teams: !quiz.teams.some((team) => team.name.trim()) ? t.ed_warn_teams : null,
    intro: prizes.enabled && !["first", "second", "third", "extra"].some((k) => String(prizes[k] || "").trim()) ? t.ed_warn_prizes : null,
    // No theme until one is picked (Classic counts once chosen), so the look is a conscious step
    theme: !quiz.settings?.theme ? t.ed_warn_theme : null,
  };
  // Ready for the show: each step opens its section (the check its own page)
  const errors = issues.filter((i) => i.level === "error").length;
  const toLook = issues.filter((i) => i.level === "warning").length;
  const namedTeams = quiz.teams.filter((team) => team.name.trim()).length;
  const step = (key, label, todo, status, onClick = () => openSection(key)) => ({ key, label, state: todo ? "todo" : "done", status, onClick });
  const theme = quiz.settings?.theme ? themeById(quiz.settings.theme) : null;
  const steps = [
    step("theme", t.ed_theme, warnings.theme, theme ? theme.name[uiLang] || theme.name.en : t.rd_theme_todo),
    quiz.drive && step("drive", t.rd_google, !google.signedIn, google.signedIn ? t.rd_google_ok : t.rd_google_todo),
    step("questions", t.ed_questions, warnings.questions, rounds.length ? fmt(t.rd_questions_ok, { rounds: rounds.length, questions: questionCount }) : t.ed_warn_questions),
    step("media", t.ed_media, warnings.media, missing.length ? fmt(t.rd_media_todo, { count: missing.length })
      : referenced.length ? fmt(t.rd_media_ok, { count: referenced.length }) : t.rd_media_none),
    step("teams", t.tm_title, warnings.teams, namedTeams ? fmt(t.rd_teams_ok, { count: namedTeams }) : t.ed_warn_teams),
    step("scores", t.ed_scores, warnings.scores, warnings.scores ? t.rd_scores_todo : quiz.scoresSheetUrl ? t.rd_scores_sheet : t.rd_scores_manual),
    step("intro", t.ed_intro, warnings.intro, warnings.intro ? t.rd_intro_todo : t.rd_intro_ok),
    // Warnings (like a hidden TEST round) can be intended, so only errors keep the check from being done
    { key: "check", label: t.chk_open, state: errors ? "error" : "done", onClick: () => navigate(quizHash(quiz.id, "check")),
      status: errors ? fmt(t.rd_check_err, { count: errors }) : toLook ? fmt(t.rd_check_warn, { count: toLook }) : t.rd_check_ok },
  ].filter(Boolean);

  // Teams, rules and prizes: saved here, and for a Drive quiz in its folder
  const saving = saveState === "saving";
  const setupSection = (key) => ({
    warning: warnings[key],
    busy: Boolean(quiz.drive) && (saving || sync.phase === "pending" || sync.phase === "syncing"),
    status: <SaveStatus saving={saving} drive={Boolean(quiz.drive)} sync={sync} t={t}
      onRefresh={quiz.drive && google.signedIn ? () => syncDriveConfig(id).catch(() => {}) : null} />,
  });

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <div className="max-w-4xl mx-auto px-6 py-10 space-y-6">
        <header className="flex flex-wrap items-center gap-4">
          <a href="#/" className="flex items-center gap-2 text-gray-400 hover:text-white">
            <ArrowLeft size={18} /> {t.settings_exit}
          </a>
          <a href="#/help" className="flex items-center gap-1.5 text-gray-400 hover:text-white text-sm"><HelpCircle size={16} /> {t.lib_help}</a>
          <span className="ml-auto text-xs text-gray-500">{saveState === "saving" ? t.ed_saving : t.ed_saved}</span>
          <a href={quizHash(quiz.id, "map")} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-white/10 hover:bg-white/20">
            <Network size={18} /> {t.mp_open}
          </a>
          <a href={quizHash(quiz.id, "check")} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-white/10 hover:bg-white/20">
            <ScanSearch size={18} /> {t.chk_open}
          </a>
          <button onClick={() => setLauncher(true)} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold">
            <Play size={18} /> {t.lib_open}
          </button>
        </header>
        {launcher && <ShowLauncher quizId={quiz.id} onClose={() => setLauncher(false)} t={t} />}

        <input value={quiz.title} onChange={(e) => update({ title: e.target.value })} placeholder={t.ed_untitled}
          className="w-full bg-transparent font-['League_Spartan'] text-4xl font-black tracking-wide border-b border-white/10 pb-3 focus:outline-none focus:border-yellow-500/60" />

        <ReadyChecklist steps={steps} onStart={() => setLauncher(true)} t={t} />

        <ThemeSection quiz={quiz} onChange={setTheme} warning={warnings.theme} t={t} />

        {quiz.drive && (
          <Section id="drive" warning={warnings.drive} busy={busy === "drive"} title={<>{t.gd_title} <BetaBadge t={t} /></>} help={t.gd_section_help}>
            <GoogleAccountBar account={google} t={t} onError={(text) => setDriveStatus(text && { ok: false, text })} />
            <ul className="space-y-2 text-sm">
              {quiz.drive.folderId && (
                <li className="flex items-center gap-2"><HardDrive size={16} className="text-gray-500 shrink-0" />
                  <a href={folderUrl(quiz.drive.folderId)} target="_blank" rel="noreferrer" className="hover:text-yellow-400 truncate">{quiz.drive.folderName}</a>
                </li>
              )}
              <li className="flex items-center gap-2"><FileSpreadsheet size={16} className="text-gray-500 shrink-0" />
                <a href={driveSheetUrl(quiz.drive.spreadsheetId)} target="_blank" rel="noreferrer" className="hover:text-yellow-400 truncate">{quiz.drive.spreadsheetName}</a>
                <span className="text-gray-500 truncate">
                  · {fmt(t.gd_questions_tab, { tab: quiz.drive.questionsTab })}
                  {quiz.drive.scoresTab && ` · ${fmt(t.gd_scores_tab, { tab: quiz.drive.scoresTab })}`}
                </span>
              </li>
              <li className="flex items-center gap-2"><FolderOpen size={16} className="text-gray-500 shrink-0" />
                {quiz.drive.mediaFolderId ? (
                  <a href={folderUrl(quiz.drive.mediaFolderId)} target="_blank" rel="noreferrer" className="hover:text-yellow-400 truncate">{quiz.drive.mediaFolderName}</a>
                ) : <span className="text-yellow-300/90">{t.gd_no_media_folder}</span>}
              </li>
            </ul>
            {quiz.drive.folderId && <ShareLink folderId={quiz.drive.folderId} t={t} />}
            <DrivePeople fileId={quiz.drive.folderId || quiz.drive.spreadsheetId} account={google} t={t} />
            <div className="flex flex-wrap items-center gap-3">
              <a href={quizHash(quiz.id, "questions")} className={`${btnCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
                <ListChecks size={16} /> {t.qe_open}
              </a>
              <button onClick={syncDrive} disabled={!google.signedIn || busy === "drive"} className={btnCls}>
                {busy === "drive" ? <Loader2 size={16} className="animate-spin" /> : <RefreshCw size={16} />}
                {busy === "drive" ? t.gd_syncing : t.gd_sync}
              </button>
              <span className="text-xs text-gray-500">
                {quiz.drive.syncedAt
                  ? fmt(t.gd_synced, { time: new Date(quiz.drive.syncedAt).toLocaleString(), name: quiz.drive.syncedBy || "?" })
                  : t.gd_never}
              </span>
            </div>
            {driveProgress && <DriveProgress progress={driveProgress} t={t} />}
            <Status status={driveStatus} />
            <p className="text-xs text-gray-500">{t.gd_offline_note}</p>
          </Section>
        )}

        <Section id="questions" warning={warnings.questions} title={t.ed_questions} help={quiz.drive ? t.gd_questions_note : t.ed_questions_help} helpId="questions">
          {!quiz.drive && (<>
          <div className="flex flex-col sm:flex-row gap-2">
            <div className="relative flex-1">
              <Link2 size={16} className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" />
              <input value={sheetUrl} onChange={(e) => setSheetUrl(e.target.value)} placeholder={t.ed_sheet_link} className={`${inputCls} pl-10`} />
            </div>
            <button onClick={loadSheet} disabled={!sheetUrl.trim() || busy === "sheet"} className={btnCls}>
              {busy === "sheet" ? <Loader2 size={16} className="animate-spin" /> : <RefreshCw size={16} />}
              {quiz.questionsSheetUrl && quiz.questionsSheetUrl === sheetUrl.trim() ? t.ed_refresh : t.ed_load}
            </button>
            <button onClick={() => csvInput.current.click()} className={btnCls}>
              <FileUp size={16} /> {t.ed_upload_csv}
            </button>
            <input ref={csvInput} type="file" accept=".csv,text/csv" hidden onChange={uploadCsv} />
          </div>
          {quiz.questionsSheetUrl && <p className="text-xs text-gray-500">{t.ed_sheet_auto}</p>}
          <Status status={questionsStatus} />
          </>)}

          {rounds.length > 0 ? (
            <ul className="divide-y divide-white/5 rounded-xl bg-black/30">
              {rounds.map((r) => (
                <li key={r.id} className="flex justify-between gap-4 px-4 py-2 text-sm">
                  <span><span className="text-gray-500">{r.number}.</span> {r.title}</span>
                  <span className="text-gray-500 whitespace-nowrap">{r.questions.length} {t.lib_questions}</span>
                </li>
              ))}
              <li className="flex justify-between px-4 py-2 text-sm font-bold">
                <span>{rounds.length} {t.lib_rounds}</span><span>{questionCount} {t.lib_questions}</span>
              </li>
            </ul>
          ) : quiz.drive ? (
            <p className="text-sm text-gray-500">{t.ed_no_questions}</p>
          ) : (
            <QuestionsStart lang={quiz.settings?.language || "cs"} onExamples={loadExamples} loading={busy === "examples"} t={t} />
          )}
        </Section>

        <Section id="media" warning={warnings.media} title={t.ed_media} help={t.ed_media_help} helpId="media">
          {missing.length > 0 && (
            <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/30 space-y-2">
              <p className="flex items-center gap-2 text-red-200 font-bold text-sm"><AlertTriangle size={16} /> {t.ed_missing} ({missing.length})</p>
              <div className="flex flex-wrap gap-2">
                {missing.map((name) => <code key={name} className="px-2 py-1 rounded bg-black/40 text-red-200 text-xs">{name}</code>)}
              </div>
            </div>
          )}
          {referenced.length > 0 && missing.length === 0 && (
            <Status status={{ ok: true, text: t.ed_all_media }} />
          )}

          {busy === "media" ? (
            <div className="flex items-center justify-center gap-2 p-6 text-gray-300"><Loader2 className="animate-spin" /> {t.lib_importing}</div>
          ) : busy === "drive-upload" && driveProgress ? (
            <div className="p-4 rounded-xl bg-black/30"><DriveProgress progress={driveProgress} t={{ ...t, gd_downloading: t.ed_drive_uploading }} /></div>
          ) : (
            <FileDrop compact onFiles={addMedia} accept="image/*,audio/*,video/*" label={quiz.drive && google.signedIn ? t.ed_media_drop_drive : t.ed_media_drop} t={t} />
          )}
          {quiz.drive && <Status status={uploadStatus} />}

          {media.length > 0 && (
            <>
              <p className="text-xs text-gray-500">{media.length} · {formatBytes(totalSize)}</p>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
                {media.map((m) => {
                  const type = m.blob.type || "";
                  const unused = !usedKeys.has(mediaKey(m.name));
                  return (
                    <div key={m.name} className="group rounded-xl bg-black/40 border border-white/10 overflow-hidden">
                      <div className="aspect-video bg-black flex items-center justify-center text-gray-500">
                        {type.startsWith("audio") ? <Music size={32} /> : type.startsWith("video") ? <Film size={32} /> : (
                          <img src={m.url} alt={m.name} className="w-full h-full object-contain" />
                        )}
                      </div>
                      <div className="p-2 flex items-start gap-2">
                        <div className="min-w-0 flex-1">
                          <p className="text-xs truncate" title={m.name}>{m.name}</p>
                          <p className="text-[10px] text-gray-500">{formatBytes(m.blob.size)}{unused && <span className="text-yellow-500/80"> · {t.ed_unused}</span>}</p>
                        </div>
                        <button onClick={() => removeMedia(m.name)} className="p-1 rounded text-gray-500 hover:text-red-300 hover:bg-red-500/20" aria-label={t.lib_delete}>
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}
        </Section>

        <Section id="scores" warning={warnings.scores} title={t.ed_scores} help={quiz.drive?.scoresTab || scoresMode === "manual" ? null : t.ed_scores_help} helpId="scores">
          {quiz.drive?.scoresTab ? (
            <div className="flex flex-col sm:flex-row sm:items-center gap-3">
              <p className="flex-1 text-sm text-gray-300">{fmt(t.ed_scores_drive, { tab: quiz.drive.scoresTab })}</p>
              <button onClick={testScores} disabled={busy === "scores"} className={btnCls}>
                {busy === "scores" ? <Loader2 size={16} className="animate-spin" /> : <CheckCircle2 size={16} />} {t.ed_test}
              </button>
            </div>
          ) : (<>
          <div className="flex gap-1 bg-white/5 rounded-xl p-1 w-fit text-sm">
            {[["manual", t.ed_scores_mode_manual], ["sheet", t.ed_scores_mode_sheet]].map(([key, label]) => (
              <button key={key} onClick={() => chooseScores(key)}
                className={`px-3 py-1.5 rounded-lg ${scoresMode === key ? "bg-yellow-500 text-black font-bold" : "text-gray-400 hover:text-white"}`}>{label}</button>
            ))}
          </div>
          {scoresMode === "manual" ? <p className="text-sm text-gray-400">{t.ed_scores_manual_help}</p> : (
          <div className="flex flex-col sm:flex-row gap-2">
            <div className="relative flex-1">
              <Link2 size={16} className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" />
              <input value={quiz.scoresSheetUrl} onChange={(e) => { update({ scoresSheetUrl: e.target.value.trim() }); setScoresStatus(null); }}
                placeholder={t.ed_sheet_link} className={`${inputCls} pl-10`} />
            </div>
            <button onClick={testScores} disabled={!quiz.scoresSheetUrl || busy === "scores"} className={btnCls}>
              {busy === "scores" ? <Loader2 size={16} className="animate-spin" /> : <CheckCircle2 size={16} />} {t.ed_test}
            </button>
          </div>
          )}
          </>)}
          <Status status={scoresStatus} />
          {scoresPreview && <ScoresPreview scores={scoresPreview} teams={quiz.teams} t={t} />}
          {!quiz.scoresSheetUrl && scoresMode === "manual" && (
            <a href={quizHash(quiz.id, "scores")} className={`${btnCls} w-fit`}>
              <Calculator size={16} /> {t.sc_open}
            </a>
          )}
        </Section>

        <TeamsSection
          quizId={quiz.id}
          teams={quiz.teams}
          onChange={(teams) => update({ teams })}
          mediaUrl={(name) => mediaUrlByKey.get(mediaKey(name))}
          hasMedia={(name) => mediaKeys.has(mediaKey(name))}
          scoresSheetUrl={quiz.scoresSheetUrl}
          fromSheet={quiz.drive?.scoresTab || null}
          onUploadImage={uploadTeamImage}
          sectionProps={setupSection("teams")}
          t={t}
        />

        <IntroSection quiz={quiz} onChange={update} sectionProps={setupSection("intro")} t={t} />
      </div>
    </div>
  );
}

// The link that opens this quiz for anyone the folder is shared with
function ShareLink({ folderId, t }) {
  const [copied, setCopied] = useState(false);
  const link = `${window.location.origin}${window.location.pathname}#/drive/${folderId}`;
  const copy = async () => {
    try { await navigator.clipboard.writeText(link); } catch { /* the field can still be selected */ }
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <div className="space-y-1.5">
      <div className="flex gap-2">
        <input readOnly value={link} onFocus={(e) => e.target.select()} className={`${inputCls} py-2.5 text-sm text-gray-300`} />
        <button onClick={copy} className={btnCls}>{copied ? <Check size={16} /> : <Copy size={16} />} {copied ? t.gd_copied : t.gd_copy_link}</button>
      </div>
      <p className="text-xs text-gray-500">{t.gd_share_help}</p>
    </div>
  );
}

// The scores the last test read, compactly: one row per team, best first, points per round and the
// total. Rows that match no team (by id or name) are marked, as the show gives them no photo or colour.
function ScoresPreview({ scores, teams, t }) {
  if (!scores.teams.length) return <p className="text-sm text-gray-500">{t.tm_from_scores_none}</p>;
  const total = (row) => scores.rounds.reduce((n, r) => n + (row.scores[r] || 0), 0);
  const rows = [...scores.teams].sort((a, b) => total(b) - total(a));
  const short = (round) => round.match(/\d+/)?.[0] ?? round;
  const points = (n) => (n ? String(Math.round(n * 100) / 100).replace(".", ",") : "–");
  return (
    <div className="rounded-xl bg-black/30 border border-white/5 overflow-hidden">
      <div className="max-h-72 overflow-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-[#111] text-gray-500 text-xs">
            <tr>
              <th className="px-3 py-2 text-left font-normal">#</th>
              <th className="px-3 py-2 text-left font-normal w-full">{t.tm_title}</th>
              {scores.rounds.map((r) => <th key={r} title={r} className="px-2 py-2 text-right font-normal whitespace-nowrap">{short(r)}</th>)}
              <th className="px-3 py-2 text-right font-bold text-gray-300">{t.ed_scores_total}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/5">
            {rows.map((row, i) => {
              const matched = teamForScores(teams, row);
              return (
                <tr key={row.id || `${row.name}-${i}`}>
                  <td className="px-3 py-1.5 text-gray-500 font-mono">{i + 1}</td>
                  <td className="px-3 py-1.5 truncate max-w-0">
                    <span className={matched ? "" : "text-yellow-300"} title={matched ? "" : t.ed_scores_unmatched_one}>{row.name}</span>
                  </td>
                  {scores.rounds.map((r) => <td key={r} className="px-2 py-1.5 text-right text-gray-400 font-mono">{points(row.scores[r])}</td>)}
                  <td className="px-3 py-1.5 text-right font-bold font-mono">{points(total(row))}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
