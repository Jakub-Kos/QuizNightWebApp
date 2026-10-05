// Rules and prizes screens shown between the welcome screen and the dashboard.
// quiz.rules  = { enabled, items }  items: array of strings, or null for the default rules in the show language
// quiz.prizes = { enabled, first, second, third, extra }  texts may be empty (only the place is shown)
export const DEFAULT_RULE_KEYS = ["rule_1", "rule_2", "rule_3", "rule_4"];
export const MAX_RULES = 8;

export const defaultRules = (t) => DEFAULT_RULE_KEYS.map((key) => t[key]).filter(Boolean);

// Rules to show, or null when the screen is turned off or there are none
export function rulesFor(quiz, t) {
  const rules = quiz.rules || {};
  if (rules.enabled === false) return null;
  const items = (rules.items ?? defaultRules(t)).map((r) => String(r || "").trim()).filter(Boolean);
  return items.length ? items : null;
}

// Prize texts to show, or null when the screen is turned off
export function prizesFor(quiz) {
  const prizes = quiz.prizes || {};
  if (prizes.enabled === false) return null;
  const text = (v) => String(v || "").trim();
  return { first: text(prizes.first), second: text(prizes.second), third: text(prizes.third), extra: text(prizes.extra) };
}
