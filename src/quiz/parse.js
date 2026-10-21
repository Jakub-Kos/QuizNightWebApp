import Papa from "papaparse";

// questions.csv columns (by position; row 1 is a header):
// 0 round marker ("Kolo N - Title" or "Round N - Title"), 1 Q#, 2 text, 3 type, 4-7 options A-D, 8 answer, 9 media file

// Word that starts a round marker, and that marks a round column in a scores sheet
export const ROUND_WORD = /^(kolo|round)\b/i;
const ROUND_COLUMN = /kolo|round/i;
export const TEAM_COLUMN = /název týmu|team name/i;
// Hidden column the app adds to the scores tab of a Drive quiz: a team's id, which survives renames
export const TEAM_ID_HEADER = "quiznight-id";
export function parseQuestionsCsv(text) {
  const rows = Papa.parse(text || "", { header: false, skipEmptyLines: false }).data;
  return groupRounds(rows).map((round) => ({ ...round, questions: buildQuestions(round.rows) }));
}

// "Kolo 3 - Title" in column 0; the header row ("Kolo,Q#,..." / "Round,Q#,...") also starts with the word,
// its Q# cell tells it apart
export const isRoundMarker = (col0, col1) => ROUND_WORD.test(col0) && col1.toLowerCase() !== "q#";

// "Kolo 3 - Title" -> { word: "Kolo", number: "3", title: "Title" }
export function splitMarker(marker) {
  const parts = marker.split("-");
  return {
    word: marker.match(ROUND_WORD)?.[0] ?? "",
    number: parts[0].replace(ROUND_WORD, "").trim(),
    title: parts.slice(1).join("-").trim(),
  };
}

// A numeric Q# keeps the header row ("Q#") out; rows without text, media or answer are skipped
// (an answer alone is a further Top5 or Sort row)
export function isQuestionRow(row) {
  const col1 = row[1] ? String(row[1]).trim() : "";
  if (!col1 || isNaN(col1)) return false;
  return [2, 9, 8].some((i) => row[i] && String(row[i]).trim() !== "");
}

function groupRounds(rows) {
  const rounds = [];
  let current = null;

  rows.forEach((row) => {
    const col0 = row[0] ? String(row[0]).trim() : "";
    const col1 = row[1] ? String(row[1]).trim() : "";

    if (isRoundMarker(col0, col1)) {
      if (current) rounds.push(current);
      const { number, title } = splitMarker(col0);
      current = { id: `round-${rounds.length + 1}`, number, title, rows: [] };
    }

    if (current && isQuestionRow(row)) current.rows.push(row);
  });

  if (current) rounds.push(current);
  return rounds;
}

// The type cell as the show knows it: "Top10", "Top 3"… are all the Top5 type (a top list of any
// length); the number written in them (not in "Top5" itself) is kept as question.declared for the check
const TOP_LIST = /^top\s*(\d+)$/i;
export const normalizeType = (cell) => {
  const type = String(cell ?? "").trim() || "Written";
  return TOP_LIST.test(type) ? "Top5" : type;
};

// Rows of one round -> its questions (Top5 and Sort take the rows that follow them)
export function buildQuestions(rows) {
  const qs = [];
  let i = 0;

  while (i < rows.length) {
    const row = rows[i];
    const type = normalizeType(row[3]);
    const media = row[9] ? String(row[9]).trim() : null;

    if (type === "Sort") {
      // Items in the correct order, from the question row and the following rows without text or type:
      // the item in column A (4), the letter it is shown under in the answer column (8), optionally a
      // picture in Zdroj. Without column A the answer column holds the item itself (letters then mixed automatically).
      const item = (r) => {
        const option = String(r[4] ?? "").trim();
        const answer = String(r[8] ?? "").trim();
        const isLetter = /^[a-z]$/i.test(answer);
        return {
          text: option || (isLetter ? "" : answer),
          letter: option || isLetter ? answer.toUpperCase() || null : null,
          media: r[9] ? String(r[9]).trim() : null,
        };
      };
      const items = [item(row)];
      let j = 1;
      while (i + j < rows.length) {
        const nextRow = rows[i + j];
        if (nextRow[2] || nextRow[3]) break;
        items.push(item(nextRow));
        j++;
      }
      qs.push({ id: row[1], text: row[2], type: "Sort", options: {}, answer: "", media: null, items: items.filter((it) => it.text || it.media) });
      i += j;
    } else if (type === "Top5") {
      // Following rows without text or type only add answers
      const answers = [row[8]];
      let j = 1;
      while (i + j < rows.length) {
        const nextRow = rows[i + j];
        if (nextRow[2] || nextRow[3]) break;
        answers.push(nextRow[8]);
        j++;
      }
      // "Top5" is the type's own name (any length); "Top10", "Top 3"… also say how long the list is
      const cell = String(row[3] ?? "").trim();
      const declared = cell === "Top5" ? null : Number(cell.match(TOP_LIST)?.[1]) || null;
      qs.push({ id: row[1], text: row[2], type: "Top5", options: {}, answer: answers, media, declared });
      i += j;
    } else {
      qs.push({
        id: row[1],
        text: row[2],
        type,
        options: { A: row[4], B: row[5], C: row[6], D: row[7] },
        answer: row[8],
        media,
      });
      i++;
    }
  }
  return qs;
}

// Rounds the Dashboard leaves out: placeholders, test rounds and empty rounds
export function isRoundHidden(round) {
  const title = (round.title || "").toUpperCase();
  return title === "KOLO" || title.includes("TEST") || title.includes("KOLO X") || round.questions.length === 0;
}

// Scores sheet: a "Název týmu" / "Team name" column and one column per round whose name contains "kolo" or "round"
export function parseScoresCsv(text) {
  const rows = Papa.parse(text || "", { header: false }).data.map((row) => row.map((c) => String(c ?? "").trim()));

  const headerIdx = rows.findIndex((row) => row.some((col) => TEAM_COLUMN.test(col)));
  if (headerIdx === -1) return null;

  const headers = rows[headerIdx];
  const teamNameIdx = headers.findIndex((h) => TEAM_COLUMN.test(h));
  const teamIdIdx = headers.indexOf(TEAM_ID_HEADER);
  const roundColumns = headers
    .map((name, index) => ({ name, index }))
    .filter((h) => ROUND_COLUMN.test(h.name));

  const teams = [];
  for (const row of rows.slice(headerIdx + 1)) {
    const name = row[teamNameIdx];
    if (!name) continue;
    const scores = {};
    roundColumns.forEach((rc) => {
      scores[rc.name] = parseFloat((row[rc.index] || "0").replace(",", ".")) || 0;
    });
    teams.push({ name, scores, ...(teamIdIdx >= 0 && row[teamIdIdx] && { id: row[teamIdIdx] }) });
  }

  return { teams, rounds: roundColumns.map((rc) => rc.name) };
}
