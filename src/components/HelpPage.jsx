import { useEffect } from "react";
import { ArrowLeft, Download, Copy, Info } from "lucide-react";
import { HELP } from "../data/help";
import { GOOGLE_SHEET_TEMPLATE, googleCopyUrl } from "../config";
import Rich from "./RichText";
import HelpTypes from "./HelpTypes";

const TEMPLATES = [
  ["questions", "questions-template.csv"],
  ["scores", "scores-template.csv"],
  ["teams", "teams-template.csv"],
];

// section: id of the section to scroll to (#/help/<section>)
export default function HelpPage({ section, lang, t }) {
  const help = HELP[lang] || HELP.en;

  useEffect(() => {
    const target = section && document.getElementById(`help-${section}`);
    if (target) target.scrollIntoView({ behavior: "smooth", block: "start" });
    else window.scrollTo(0, 0);
  }, [section]);

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans">
      <div className="max-w-6xl mx-auto px-6 py-10 grid md:grid-cols-[220px_1fr] gap-10">
        <nav className="md:sticky md:top-10 md:self-start space-y-1 text-sm">
          <a href="#/" className="flex items-center gap-2 text-gray-400 hover:text-white mb-6">
            <ArrowLeft size={18} /> {t.settings_exit}
          </a>
          {help.sections.map((s) => (
            <a key={s.id} href={`#/help/${s.id}`}
              className={`block px-3 py-2 rounded-lg ${section === s.id ? "bg-white/10 text-white" : "text-gray-400 hover:text-white hover:bg-white/5"}`}>
              {s.title}
            </a>
          ))}
        </nav>

        <main className="space-y-12 min-w-0">
          <header className="space-y-6">
            <h1 className="font-['League_Spartan'] text-4xl font-black uppercase tracking-widest text-yellow-400">{help.title}</h1>
            <div className="rounded-2xl bg-white/[0.04] border border-white/10 p-5 space-y-3">
              <h2 className="font-bold">{help.templates.title}</h2>
              <div className="flex flex-wrap gap-2">
                {GOOGLE_SHEET_TEMPLATE && (
                  <a href={googleCopyUrl(GOOGLE_SHEET_TEMPLATE)} target="_blank" rel="noreferrer"
                    className="flex items-center gap-2 px-4 py-2 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold text-sm">
                    <Copy size={16} /> {help.templates.google}
                  </a>
                )}
                {TEMPLATES.map(([key, file]) => (
                  <a key={key} href={`${import.meta.env.BASE_URL}templates/${file}`} download={file}
                    className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-sm">
                    <Download size={16} /> {help.templates[key]}
                  </a>
                ))}
              </div>
              <p className="text-xs text-gray-500">{help.templates.importHint}</p>
            </div>
          </header>

          {help.sections.map((s) => (
            <section key={s.id} id={`help-${s.id}`} className="scroll-mt-6 space-y-4">
              <h2 className="font-['League_Spartan'] text-2xl font-bold uppercase tracking-widest">{s.title}</h2>
              {s.intro && <p className="text-gray-300"><Rich text={s.intro} /></p>}

              {s.items && (() => {
                const List = s.ordered ? "ol" : "ul";
                return (
                  <List className={`space-y-2 text-gray-300 pl-6 ${s.ordered ? "list-decimal" : "list-disc"} marker:text-gray-600`}>
                    {s.items.map((item, i) => <li key={i} className="pl-1"><Rich text={item} /></li>)}
                  </List>
                );
              })()}

              {s.examples && <HelpTypes section={s} lang={lang} t={t} />}

              {s.table && !s.examples && (
                <div className="overflow-x-auto rounded-xl border border-white/10">
                  <table className="w-full text-sm">
                    <thead className="bg-white/5 text-gray-400 text-left">
                      <tr>{s.table.head.map((h) => <th key={h} className="px-4 py-2 font-bold">{h}</th>)}</tr>
                    </thead>
                    <tbody className="divide-y divide-white/5">
                      {s.table.rows.map((row, i) => (
                        <tr key={i}>
                          {row.map((cell, j) => (
                            <td key={j} className={`px-4 py-2 align-top ${j === 0 ? "whitespace-nowrap" : "text-gray-300"}`}><Rich text={cell} /></td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}

              {s.note && (
                <p className="flex items-start gap-2 text-sm text-blue-200 bg-blue-500/10 border border-blue-500/20 rounded-xl p-3">
                  <Info size={16} className="shrink-0 mt-0.5" /> <span><Rich text={s.note} /></span>
                </p>
              )}
            </section>
          ))}
        </main>
      </div>
    </div>
  );
}
