import { useState, useEffect, useMemo, useRef } from "react";
import { ArrowLeft, Play, Link2, RefreshCw, FileUp, Trash2, Music, Film, AlertTriangle, CheckCircle2, Loader2, ScanSearch, Calculator } from "lucide-react";
import { getQuiz, updateQuiz, listMedia, putMedia, deleteMedia } from "../quiz/storage";
import { parseQuestionsCsv, parseScoresCsv } from "../quiz/parse";
import { fetchSheetCsv, SheetError } from "../quiz/sheets";
import { referencedMedia, isMediaFile } from "../quiz/package";
import { mediaKey } from "../quiz/model";
import { formatBytes, fmt } from "../quiz/files";
import { quizHash } from "../hooks/useHashRoute";
import FileDrop from "./FileDrop";
import { Section, Status, inputCls, btnCls } from "./EditorParts";
import TeamsSection from "./TeamsSection";
import { teamImageName } from "../quiz/teams";

function sheetErrorText(err, t) {
  if (err instanceof SheetError && err.message.startsWith("http_")) return t.ed_err_http;
  if (err instanceof SheetError) return t.ed_err_network;
  return String(err.message || err);
}

export default function QuizEditor({ id, t }) {
  const [quiz, setQuiz] = useState(undefined); // undefined = loading, null = not found
  const [media, setMedia] = useState([]); // { name, blob, url }
  const [saveState, setSaveState] = useState("saved");
  const [sheetUrl, setSheetUrl] = useState("");
  const [questionsStatus, setQuestionsStatus] = useState(null);
  const [scoresStatus, setScoresStatus] = useState(null);
  const [busy, setBusy] = useState(null);
  const csvInput = useRef(null);
  const dirty = useRef(false);

  useEffect(() => {
    getQuiz(id).then((q) => {
      setQuiz(q ?? null);
      setSheetUrl(q?.questionsSheetUrl || "");
    });
  }, [id]);

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
    // Only the fields edited here; settings and scores are saved by other windows
    const { title, questionsCsv, questionsSheetUrl, scoresSheetUrl, teams } = quiz;
    const timer = setTimeout(() => updateQuiz(quiz.id, (q) => ({ ...q, title, questionsCsv, questionsSheetUrl, scoresSheetUrl, teams })).then(() => {
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

  const rounds = useMemo(() => (quiz ? parseQuestionsCsv(quiz.questionsCsv) : []), [quiz]);
  const questionCount = rounds.reduce((n, r) => n + r.questions.length, 0);
  const referenced = useMemo(() => (quiz ? referencedMedia(quiz) : []), [quiz]);
  const mediaKeys = useMemo(() => new Set(media.map((m) => mediaKey(m.name))), [media]);
  const missing = referenced.filter((name) => !mediaKeys.has(mediaKey(name)));
  const usedKeys = new Set(referenced.map(mediaKey));
  const mediaUrlByKey = useMemo(() => new Map(media.map((m) => [mediaKey(m.name), m.url])), [media]);

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

  const addMedia = async (files) => {
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
    const name = teamImageName(team, file);
    if (team.image && team.image.startsWith("team-") && mediaKey(team.image) !== mediaKey(name)) await deleteMedia(id, team.image);
    await putMedia(id, [{ name, blob: file }]);
    await reloadMedia();
    return name;
  };

  const removeMedia = async (name) => {
    await deleteMedia(id, name);
    await reloadMedia();
  };

  const testScores = async () => {
    setBusy("scores");
    try {
      const parsed = parseScoresCsv(await fetchSheetCsv(quiz.scoresSheetUrl));
      // The leaderboard matches Sheet rows to teams by name; unmatched rows get no photo or color
      const known = new Set(quiz.teams.map((team) => team.name.trim().toLowerCase()));
      const unmatched = parsed ? parsed.teams.map((row) => row.name).filter((name) => !known.has(name.trim().toLowerCase())) : [];
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

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <div className="max-w-4xl mx-auto px-6 py-10 space-y-6">
        <header className="flex flex-wrap items-center gap-4">
          <a href="#/" className="flex items-center gap-2 text-gray-400 hover:text-white">
            <ArrowLeft size={18} /> {t.settings_exit}
          </a>
          <span className="ml-auto text-xs text-gray-500">{saveState === "saving" ? t.ed_saving : t.ed_saved}</span>
          <a href={quizHash(quiz.id, "check")} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-white/10 hover:bg-white/20">
            <ScanSearch size={18} /> {t.chk_open}
          </a>
          <a href={quizHash(quiz.id)} className="flex items-center gap-2 px-5 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold">
            <Play size={18} /> {t.lib_open}
          </a>
        </header>

        <input value={quiz.title} onChange={(e) => update({ title: e.target.value })} placeholder={t.ed_untitled}
          className="w-full bg-transparent font-['League_Spartan'] text-4xl font-black tracking-wide border-b border-white/10 pb-3 focus:outline-none focus:border-yellow-500/60" />

        <Section title={t.ed_questions} help={t.ed_questions_help}>
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
          ) : (
            <p className="text-sm text-gray-500">{t.ed_no_questions}</p>
          )}
        </Section>

        <Section title={t.ed_media} help={t.ed_media_help}>
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
          ) : (
            <FileDrop compact onFiles={addMedia} accept="image/*,audio/*,video/*" label={t.ed_media_drop} t={t} />
          )}

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

        <TeamsSection
          teams={quiz.teams}
          onChange={(teams) => update({ teams })}
          mediaUrl={(name) => mediaUrlByKey.get(mediaKey(name))}
          hasMedia={(name) => mediaKeys.has(mediaKey(name))}
          onUploadImage={uploadTeamImage}
          t={t}
        />

        <Section title={t.ed_scores} help={t.ed_scores_help}>
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
          <Status status={scoresStatus} />
          {!quiz.scoresSheetUrl && (
            <a href={quizHash(quiz.id, "scores")} className={`${btnCls} w-fit`}>
              <Calculator size={16} /> {t.sc_open}
            </a>
          )}
        </Section>
      </div>
    </div>
  );
}
