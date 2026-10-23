import { Copy, Sparkles, Download, Loader2, HelpCircle } from "lucide-react";
import { GOOGLE_SHEET_TEMPLATE, googleCopyUrl } from "../config";
import { exampleQuestions } from "../quiz/examples";
import { SheetRows } from "./HelpTypes";

const cardCls = "rounded-xl border border-white/10 bg-black/30 p-4 space-y-3 flex flex-col";
const linkCls = "flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl text-sm whitespace-nowrap mt-auto";

// Shown in an empty questions section: the ways to get started, and what the questions sheet looks like.
// lang: the quiz language, for the example questions; onExamples loads them (with their media).
export default function QuestionsStart({ lang, onExamples, loading, t }) {
  const preview = exampleQuestions(lang, t.ed_example_round).rows.slice(0, 4);
  return (
    <div className="space-y-4">
      <h3 className="font-bold">{t.qs_title}</h3>
      <div className="grid sm:grid-cols-3 gap-3">
        {GOOGLE_SHEET_TEMPLATE && (
          <div className={cardCls}>
            <h4 className="font-bold text-sm">{t.qs_sheet_title}</h4>
            <p className="text-sm text-gray-400">{t.qs_sheet_text}</p>
            <a href={googleCopyUrl(GOOGLE_SHEET_TEMPLATE)} target="_blank" rel="noreferrer" className={`${linkCls} bg-yellow-500 hover:bg-yellow-400 text-black font-bold`}>
              <Copy size={16} /> {t.qs_sheet_copy}
            </a>
          </div>
        )}
        <div className={cardCls}>
          <h4 className="font-bold text-sm">{t.qs_examples_title}</h4>
          <p className="text-sm text-gray-400">{t.qs_examples_text}</p>
          <button onClick={onExamples} disabled={loading} className={`${linkCls} bg-white/10 hover:bg-white/20 disabled:opacity-50`}>
            {loading ? <Loader2 size={16} className="animate-spin" /> : <Sparkles size={16} />} {t.qs_examples_load}
          </button>
        </div>
        <div className={cardCls}>
          <h4 className="font-bold text-sm">{t.qs_csv_title}</h4>
          <p className="text-sm text-gray-400">{t.qs_csv_text}</p>
          <a href={`${import.meta.env.BASE_URL}templates/questions-template.csv`} download="questions-template.csv" className={`${linkCls} bg-white/10 hover:bg-white/20`}>
            <Download size={16} /> {t.qs_csv_download}
          </a>
        </div>
      </div>
      <SheetRows rows={preview} label={t.qs_preview} />
      <a href="#/help/questions" className="inline-flex items-center gap-1.5 text-sm text-gray-400 hover:text-yellow-400">
        <HelpCircle size={14} /> {t.qs_help}
      </a>
    </div>
  );
}
