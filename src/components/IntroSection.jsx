import { Plus, Trash2, RotateCcw, Trophy, Medal, Gift } from "lucide-react";
import { TRANSLATIONS } from "../data/translations";
import { defaultRules, MAX_RULES } from "../quiz/intro";
import { Section, btnCls } from "./EditorParts";

const fieldCls = "w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-yellow-500/60";

function Toggle({ on, onChange, label }) {
  return (
    <label className="flex items-center justify-between gap-4 cursor-pointer">
      <span className="text-gray-200 font-medium">{label}</span>
      <button type="button" onClick={() => onChange(!on)}
        className={`w-14 h-8 rounded-full p-1 transition-colors ${on ? "bg-green-500" : "bg-gray-700"}`}>
        <div className={`w-6 h-6 bg-white rounded-full shadow transition-transform ${on ? "translate-x-6" : ""}`} />
      </button>
    </label>
  );
}

// Rules and prizes screens of the show (quiz/intro.js). While the quiz has no own rules, the default
// rules in the show language are shown here, and editing any of them saves them as the quiz's own.
export default function IntroSection({ quiz, onChange, t }) {
  const rules = { enabled: true, items: null, ...quiz.rules };
  const prizes = { enabled: true, first: "", second: "", third: "", extra: "", ...quiz.prizes };
  const showT = TRANSLATIONS[quiz.settings?.language] || TRANSLATIONS.en;
  const items = rules.items ?? defaultRules(showT);

  const setRules = (fields) => onChange({ rules: { ...rules, ...fields } });
  const setItems = (next) => setRules({ items: next });
  const setPrizes = (fields) => onChange({ prizes: { ...prizes, ...fields } });

  const prizeFields = [
    ["first", t.ed_prize_1, <Trophy key="1" size={18} className="text-yellow-400" />],
    ["second", t.ed_prize_2, <Medal key="2" size={18} className="text-slate-300" />],
    ["third", t.ed_prize_3, <Medal key="3" size={18} className="text-orange-400" />],
    ["extra", t.ed_prize_extra, <Gift key="4" size={18} className="text-pink-400" />],
  ];

  return (
    <Section title={t.ed_intro} help={t.ed_intro_help}>
      <div className="grid lg:grid-cols-2 gap-6">
        <div className="space-y-3 rounded-xl bg-black/30 border border-white/5 p-4">
          <Toggle on={rules.enabled} onChange={(enabled) => setRules({ enabled })} label={t.ed_rules_show} />
          {rules.enabled && (
            <>
              <ol className="space-y-2">
                {items.map((rule, i) => (
                  <li key={i} className="flex items-center gap-2">
                    <span className="w-5 text-right text-xs text-gray-500">{i + 1}.</span>
                    <input value={rule} placeholder={t.ed_rule_placeholder} className={fieldCls}
                      onChange={(e) => setItems(items.map((r, k) => (k === i ? e.target.value : r)))} />
                    <button onClick={() => setItems(items.filter((_, k) => k !== i))} title={t.lib_delete}
                      className="p-2 rounded-lg text-gray-500 hover:text-red-300 hover:bg-red-500/20"><Trash2 size={16} /></button>
                  </li>
                ))}
              </ol>
              <div className="flex flex-wrap gap-2">
                <button onClick={() => setItems([...items, ""])} disabled={items.length >= MAX_RULES} className={btnCls}>
                  <Plus size={16} /> {t.ed_rule_add}
                </button>
                {rules.items !== null && (
                  <button onClick={() => setItems(null)} className={btnCls}>
                    <RotateCcw size={16} /> {t.ed_rules_default}
                  </button>
                )}
              </div>
            </>
          )}
        </div>

        <div className="space-y-3 rounded-xl bg-black/30 border border-white/5 p-4">
          <Toggle on={prizes.enabled} onChange={(enabled) => setPrizes({ enabled })} label={t.ed_prizes_show} />
          {prizes.enabled && prizeFields.map(([key, label, icon]) => (
            <label key={key} className="block space-y-1">
              <span className="flex items-center gap-2 text-xs text-gray-400">{icon} {label}</span>
              <input value={prizes[key]} placeholder={t.ed_prize_placeholder} className={fieldCls}
                onChange={(e) => setPrizes({ [key]: e.target.value })} />
            </label>
          ))}
        </div>
      </div>
    </Section>
  );
}
