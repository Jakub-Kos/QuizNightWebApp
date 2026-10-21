import { useRef, useState } from "react";
import { Plus, X, Shuffle, AlertTriangle, Upload, Loader2 } from "lucide-react";
import { withLetters } from "../quiz/sheetModel";
import QuestionTypePicker from "./QuestionTypePicker";
import { MAX_SORT_ITEMS } from "../quiz/sort";
import { isUrl } from "../quiz/model";
import { inputCls } from "./EditorParts";
import { fmt } from "../quiz/files";

const smallBtn = "flex items-center gap-1.5 px-3 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-xs disabled:opacity-40";
const MEDIA_TYPES = ["PImage", "PVideo", "Audio", "Image", "Video"];
const NO_TEXT_NEEDED = ["PImage", "PVideo", "Handout"];

const Label = ({ children, hint }) => (
  <span className="block text-xs font-bold uppercase tracking-wider text-gray-400 mb-1.5">
    {children}{hint && <span className="normal-case tracking-normal font-normal text-gray-500"> · {hint}</span>}
  </span>
);

// Zdroj path with suggestions from the synced media, and a note when the file is not there. A file
// picked or dropped here is uploaded to the media folder: into the subfolder already typed ("kolo3/"),
// otherwise its top level. upload(file, folder, onProgress) resolves to { path } or { error }, or null.
function MediaInput({ value, onChange, hasMedia, placeholder, accept, upload, t }) {
  const picker = useRef(null);
  const [progress, setProgress] = useState(null); // { name, share } while uploading
  const [error, setError] = useState(null);
  const [over, setOver] = useState(false);
  const missing = value.trim() && !isUrl(value) && !hasMedia(value.trim());

  const send = async (file) => {
    if (!file || !upload) return;
    const typed = value.trim();
    const folder = typed.includes("/") ? typed.slice(0, typed.lastIndexOf("/")) : "";
    setError(null);
    setProgress({ name: file.name, share: 0 });
    const result = await upload(file, folder, (share) => setProgress({ name: file.name, share }));
    setProgress(null);
    if (result?.path) onChange(result.path);
    if (result?.error) setError(result.error);
  };

  return (
    <div className="space-y-1">
      <div className={`relative rounded-xl ${over ? "ring-2 ring-yellow-500" : ""}`}
        onDragOver={(e) => { if (upload) { e.preventDefault(); setOver(true); } }}
        onDragLeave={() => setOver(false)}
        onDrop={(e) => { e.preventDefault(); setOver(false); send(e.dataTransfer.files[0]); }}>
        <input value={value} onChange={(e) => onChange(e.target.value)} list="qe-media" placeholder={placeholder || t.qe_media_hint}
          className={`${inputCls} ${upload ? "pr-12" : ""}`} />
        {upload && (
          <button type="button" onClick={() => picker.current.click()} disabled={Boolean(progress)} title={t.qe_upload}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10">
            {progress ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />}
          </button>
        )}
        <input ref={picker} type="file" accept={accept} hidden onChange={(e) => { const file = e.target.files[0]; e.target.value = ""; send(file); }} />
      </div>
      {progress && (
        <div className="space-y-1">
          <div className="h-1.5 rounded-full bg-white/10 overflow-hidden"><div className="h-full bg-yellow-500 transition-[width]" style={{ width: `${Math.round(progress.share * 100)}%` }} /></div>
          <p className="text-xs text-gray-400 truncate">{fmt(t.qe_uploading, { file: progress.name, percent: Math.round(progress.share * 100) })}</p>
        </div>
      )}
      {error && <p className="flex items-start gap-1.5 text-xs text-red-300"><AlertTriangle size={14} className="shrink-0 mt-px" /> {error}</p>}
      {missing && !progress && <p className="flex items-start gap-1.5 text-xs text-yellow-300/90"><AlertTriangle size={14} className="shrink-0 mt-px" /> {upload ? t.qe_media_missing_upload : t.qe_media_missing}</p>}
    </div>
  );
}

const acceptFor = (type) => (type === "Audio" ? "audio/*" : type.includes("Video") ? "video/*" : "image/*");

// The fields of one question, depending on its type. view / onView: the preview step shown next to
// the form, which the type picker's "how it plays" steps switch; isNew: a question being added
export default function QuestionForm({ draft, onChange, mediaNames, hasMedia, upload, view, onView, isNew, disabled, t }) {
  const { type } = draft;
  // Updates apply to the latest draft: an upload can finish after other fields changed
  const update = (fn) => onChange((d) => ({ ...d, ...fn(d) }));
  const set = (fields) => update(() => fields);

  const setAnswerAt = (i, value) => update((d) => ({ answers: d.answers.map((a, j) => (j === i ? value : a)) }));
  const setItem = (i, fields) => update((d) => ({ items: d.items.map((it, j) => (j === i ? { ...it, ...fields } : it)) }));
  // Adding or removing an item changes how many letters there are, so they are mixed again
  const resizeItems = (items) => set({ items: withLetters(items) });

  return (
    <fieldset disabled={disabled} className="space-y-5 min-w-0">
      <datalist id="qe-media">{mediaNames.map((name) => <option key={name} value={name} />)}</datalist>

      <div>
        <Label>{t.qe_type}</Label>
        <QuestionTypePicker type={type} onChange={(next) => set({ type: next })} view={view} onView={onView} open={isNew} t={t} />
      </div>

      <label className="block">
        <Label hint={NO_TEXT_NEEDED.includes(type) ? t.qe_optional : null}>{t.qe_text}</Label>
        <textarea value={draft.text} onChange={(e) => set({ text: e.target.value })} rows={3} className={`${inputCls} resize-y`} />
      </label>

      {type === "ABCD" && (
        <div className="space-y-2">
          <Label>{t.qe_options}</Label>
          {["A", "B", "C", "D"].map((o) => (
            <div key={o} className="flex items-center gap-2">
              <button type="button" onClick={() => set({ answer: o })} title={t.qe_correct}
                className={`w-10 h-10 shrink-0 rounded-lg font-black ${draft.answer.trim().toUpperCase() === o ? "bg-green-500 text-black" : "bg-white/10 text-gray-300 hover:bg-white/20"}`}>
                {o}
              </button>
              <input value={draft.options[o]} onChange={(e) => set({ options: { ...draft.options, [o]: e.target.value } })} className={inputCls} />
            </div>
          ))}
          <p className="text-xs text-gray-500">{t.qe_correct_hint}</p>
        </div>
      )}

      {type === "Yes/No" && (
        <div>
          <Label>{t.qe_answer}</Label>
          <div className="flex gap-2">
            {["Yes", "No"].map((v) => (
              <button key={v} type="button" onClick={() => set({ answer: v })}
                className={`px-5 py-2.5 rounded-lg font-bold ${draft.answer.trim().toLowerCase() === v.toLowerCase() ? "bg-green-500 text-black" : "bg-white/10 text-gray-300 hover:bg-white/20"}`}>
                {v === "Yes" ? t.qe_yes : t.qe_no}
              </button>
            ))}
          </div>
        </div>
      )}

      {type === "Top5" && (
        <div className="space-y-2">
          <Label hint={t.qe_answers_hint}>{t.qe_answers}</Label>
          {draft.answers.map((answer, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="w-6 text-right text-sm text-gray-500">{i + 1}.</span>
              <input value={answer} onChange={(e) => setAnswerAt(i, e.target.value)} className={inputCls} />
              <button type="button" onClick={() => set({ answers: draft.answers.filter((_, j) => j !== i) })} disabled={draft.answers.length <= 1}
                className="p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20 disabled:opacity-30" aria-label={t.qe_remove}><X size={16} /></button>
            </div>
          ))}
          <button type="button" onClick={() => set({ answers: [...draft.answers, ""] })} disabled={draft.answers.length >= 15} className={smallBtn}>
            <Plus size={14} /> {t.qe_add_answer}
          </button>
        </div>
      )}

      {type === "Sort" && (
        <div className="space-y-2">
          <Label hint={t.qe_items_hint}>{t.qe_items}</Label>
          {draft.items.map((item, i) => (
            <div key={i} className="flex items-start gap-2">
              <span className="w-6 pt-3 text-right text-sm text-gray-500">{i + 1}.</span>
              <span className="w-10 h-[50px] shrink-0 flex items-center justify-center rounded-lg bg-white/10 font-black text-yellow-400" title={t.qe_letter}>{item.letter}</span>
              <div className="flex-1 min-w-0 grid sm:grid-cols-[3fr_2fr] gap-2">
                <input value={item.text} onChange={(e) => setItem(i, { text: e.target.value })} placeholder={t.qe_item} className={inputCls} />
                <MediaInput value={item.media} onChange={(media) => setItem(i, { media })} hasMedia={hasMedia} placeholder={t.qe_item_media} accept="image/*" upload={upload} t={t} />
              </div>
              <button type="button" onClick={() => resizeItems(draft.items.filter((_, j) => j !== i))} disabled={draft.items.length <= 2}
                className="mt-1.5 p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20 disabled:opacity-30" aria-label={t.qe_remove}><X size={16} /></button>
            </div>
          ))}
          <div className="flex flex-wrap gap-2">
            <button type="button" onClick={() => resizeItems([...draft.items, { text: "", media: "" }])} disabled={draft.items.length >= MAX_SORT_ITEMS} className={smallBtn}>
              <Plus size={14} /> {t.qe_add_item}
            </button>
            <button type="button" onClick={() => resizeItems(draft.items)} className={smallBtn}>
              <Shuffle size={14} /> {t.qe_shuffle}
            </button>
          </div>
          <p className="text-xs text-gray-500">{t.qe_letters_hint}</p>
        </div>
      )}

      {type === "Handout" && (
        <>
          <div><Label>{t.qe_handout_picture}</Label><MediaInput value={draft.media} onChange={(media) => set({ media })} hasMedia={hasMedia} accept="image/*" upload={upload} t={t} /></div>
          <div><Label>{t.qe_handout_answer}</Label><MediaInput value={draft.answer} onChange={(answer) => set({ answer })} hasMedia={hasMedia} accept="image/*" upload={upload} t={t} /></div>
        </>
      )}

      {!["ABCD", "Yes/No", "Top5", "Sort", "Handout"].includes(type) && (
        <label className="block">
          <Label>{t.qe_answer}</Label>
          <input value={draft.answer} onChange={(e) => set({ answer: e.target.value })} className={inputCls} />
        </label>
      )}

      {!["Sort", "Handout"].includes(type) && (
        <div>
          <Label hint={MEDIA_TYPES.includes(type) ? null : t.qe_optional}>{t.qe_media}</Label>
          <MediaInput value={draft.media} onChange={(media) => set({ media })} hasMedia={hasMedia} accept={acceptFor(type)} upload={upload} t={t} />
        </div>
      )}
    </fieldset>
  );
}
