// "Hvězda II" -> "HI", "Honibrci" -> "HO"; emoji and accents are kept as whole characters
export function teamInitials(name) {
  const all = String(name || "").trim().split(/\s+/).filter(Boolean);
  // Words without letters or digits (emoji, "=", "–") are skipped unless that is all there is
  const words = all.filter((w) => /[\p{L}\p{N}]/u.test(w)).length ? all.filter((w) => /[\p{L}\p{N}]/u.test(w)) : all;
  if (words.length === 0) return "?";
  const chars = (word) => Array.from(word.replace(/^[^\p{L}\p{N}]+/u, "") || word);
  const letters = words.length === 1 ? chars(words[0]).slice(0, 2) : [chars(words[0])[0], chars(words[1])[0]];
  return letters.join("").toUpperCase();
}

// Initials for every team, distinct within the quiz: the first team keeps its plain initials, later
// teams sharing them get the next free variant ("Hvězda I" HI, "Hvězda II" HII; "Bandytové" BN;
// "bakalářstvo a magistr" BAM).
export function uniqueInitials(names) {
  const base = names.map(teamInitials);
  const counts = new Map();
  base.forEach((b) => counts.set(b, (counts.get(b) || 0) + 1));
  const used = new Set(base.filter((b) => counts.get(b) === 1));

  return names.map((name, i) => {
    if (counts.get(base[i]) === 1) return base[i];
    const pick = candidates(name, base[i]).find((c) => !used.has(c)) ?? `${base[i]}${i + 1}`;
    used.add(pick);
    return pick;
  });
}

function candidates(name, base) {
  const words = String(name).trim().split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w));
  const letters = (w) => Array.from(w).filter((c) => /[\p{L}\p{N}]/u.test(c));
  const list = [base];
  if (words.length >= 3) list.push(words.slice(0, 3).map((w) => letters(w)[0]).join(""));
  const last = words.length > 1 ? letters(words[words.length - 1]) : [];
  if (last.length > 0 && last.length <= 3) list.push(letters(words[0])[0] + last.join(""));
  // First letter followed by each later letter of the name in turn: Bandytové -> BA, BN, BD, ...
  const all = words.flatMap(letters);
  for (let k = 1; k < all.length; k++) list.push(all[0] + all[k]);
  return list.map((c) => c.toUpperCase());
}
