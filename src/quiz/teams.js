import Papa from "papaparse";

export function newTeam(fields = {}) {
  return { id: crypto.randomUUID(), name: "", quote: "", players: 0, image: null, color: null, icon: "Brain", isNew: true, ...fields };
}

const sameName = (a, b) => String(a || "").trim().toLowerCase() === String(b || "").trim().toLowerCase();

// First header whose name matches one of the patterns
function findColumn(headers, patterns) {
  return headers.find((h) => patterns.some((p) => p.test(h)));
}

const TEAM_COLUMNS = {
  name: [/název týmu/i, /team name/i, /^tým$/i, /^team$/i, /^název$/i, /^name$/i],
  players: [/počet hráčů/i, /players/i, /hráč/i],
  quote: [/motto/i, /moudro/i, /citát/i, /quote/i],
  image: [/profilovk/i, /fotk/i, /photo/i, /image/i, /obrázek/i, /logo/i],
};

// Registrations (e.g. a Google Form export) -> teams. Existing teams with the same name are updated.
// Returns { teams, added, updated } or throws when no team-name column is found.
export function importTeamsCsv(text, existing, hasMedia) {
  const { data, meta } = Papa.parse(text, { header: true, skipEmptyLines: true });
  const headers = meta.fields || [];
  const col = Object.fromEntries(Object.entries(TEAM_COLUMNS).map(([k, p]) => [k, findColumn(headers, p)]));
  if (!col.name) throw new Error("no_name_column");

  const teams = [...existing];
  let added = 0, updated = 0;
  for (const row of data) {
    const name = String(row[col.name] || "").trim();
    if (!name) continue;

    const fields = { name };
    if (col.players && row[col.players]) fields.players = parseInt(row[col.players]) || 0;
    if (col.quote && row[col.quote]) fields.quote = String(row[col.quote]).trim();
    // Only file names of uploaded media can be shown; links (e.g. Google Drive) cannot
    const image = col.image ? String(row[col.image] || "").trim() : "";
    if (image && !/^https?:/i.test(image) && hasMedia(image)) fields.image = image;

    const index = teams.findIndex((t) => sameName(t.name, name));
    if (index >= 0) { teams[index] = { ...teams[index], ...fields }; updated++; }
    else { teams.push(newTeam(fields)); added++; }
  }
  return { teams, added, updated };
}

// Team names from a scores Sheet (parseScoresCsv result) added after the existing teams.
// Teams already in the list (same name, any letter case) are kept as they are.
export function addTeamsFromScores(scores, existing) {
  const teams = [...existing];
  let added = 0;
  for (const { name } of scores.teams) {
    if (teams.some((t) => sameName(t.name, name))) continue;
    teams.push(newTeam({ name: name.trim() }));
    added++;
  }
  return { teams, added, existing: scores.teams.length - added };
}

// Past results: a team-name column, one rank column per past quiz (header containing "rank", "pořadí" or
// "umístění") and optionally an accuracy column. Fills the stats shown on the welcome screen.
export function importHistoryCsv(text, existing) {
  const { data, meta } = Papa.parse(text, { header: true, skipEmptyLines: true });
  const headers = meta.fields || [];
  const nameCol = findColumn(headers, [/master_team/i, ...TEAM_COLUMNS.name, /tým/i, /team/i]);
  const rankCols = headers.filter((h) => /rank|pořadí|umístění/i.test(h));
  const accuracyCol = findColumn(headers, [/accuracy/i, /úspěšnost/i, /přesnost/i]);
  if (!nameCol || rankCols.length === 0) throw new Error("no_history_columns");

  let matched = 0;
  const teams = existing.map((team) => {
    const row = data.find((r) => sameName(r[nameCol], team.name));
    if (!row) return team;
    matched++;
    const rankHistory = rankCols.map((c) => {
      const n = parseInt(row[c]);
      return isNaN(n) ? "?" : n;
    });
    const ranks = rankHistory.filter((r) => typeof r === "number");
    return {
      ...team,
      isNew: ranks.length === 0,
      rankHistory,
      bestRank: ranks.length ? Math.min(...ranks) : 0,
      podium: { first: ranks.filter((r) => r === 1).length, second: ranks.filter((r) => r === 2).length, third: ranks.filter((r) => r === 3).length },
      correctRate: accuracyCol ? parseFloat(String(row[accuracyCol] || "").replace("%", "").replace(",", ".")) || 0 : team.correctRate ?? 0,
    };
  });
  return { teams, matched, total: data.length };
}

// Team photos are stored as media named after the team id, so renaming a team keeps its photo
export const teamImageName = (team, file) => `team-${team.id}.${file.name.split(".").pop().toLowerCase()}`;
