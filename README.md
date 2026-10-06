<div align="center">

# Quiz Night OS

**Turn your pub quiz into a TV game show.**

Write the questions in a Google Sheet, drop in your pictures and teams, and run the whole night from your laptop:<br>
the TV shows the show, a second window gives the host the answers and the controls.

### [▶ Open the app](https://jakub-kos.github.io/QuizNightWebApp/)

Free, nothing to install, no account. Your quizzes stay in your own browser.

![React](https://img.shields.io/badge/React_19-20232A?style=flat-square&logo=react&logoColor=61DAFB)
![Vite](https://img.shields.io/badge/Vite-646CFF?style=flat-square&logo=vite&logoColor=white)
![Tailwind](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=flat-square&logo=tailwind-css&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

<br>

<img src="docs/screenshots/welcome.jpg" alt="Welcome screen introducing a team with its past results" width="100%">

</div>

## How a quiz night looks

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/team-wall.jpg" alt="Team wall"><br><b>Every team gets an introduction.</b> Teams with photos and history get big cards; teams known only by name get a living team wall.</td>
    <td width="50%"><img src="docs/screenshots/dashboard.jpg" alt="Rounds dashboard"><br><b>The rounds at a glance.</b> Finished rounds reveal their names, the bar shows how far the night is.</td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/question-abcd.jpg" alt="ABCD question with the answer revealed"><br><b>Questions, then answers.</b> Play a round with answers, or questions only with a timer and reveal them later.</td>
    <td><img src="docs/screenshots/question-sort.jpg" alt="Sort question with pictures"><br><b>Eleven question types</b>, from ABCD and Top 5 to pictures, audio, video and putting things in order.</td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/leaderboard.jpg" alt="Leaderboard during the reveal"><br><b>A real reveal.</b> The standings hold until the host reveals the round's points group by group, from the lowest score up.</td>
    <td><img src="docs/screenshots/pause.jpg" alt="Half-time screen"><br><b>Half-time</b> with a countdown, the teams in the spotlight and the live standings ticker.</td>
  </tr>
</table>

<details>
<summary><b>More screens:</b> rules, prizes, picture and Top 5 questions</summary>
<br>
<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/rules.jpg" alt="Rules screen"></td>
    <td width="50%"><img src="docs/screenshots/prizes.jpg" alt="Prizes screen"></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/question-image.jpg" alt="Picture question"></td>
    <td><img src="docs/screenshots/question-top5.jpg" alt="Top 5 question with the answers revealed"></td>
  </tr>
</table>
</details>

## What the host sees

Press `Shift + P` and a second window opens on the laptop. It runs the show: the TV follows every click, with no server in between.

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/host-question.jpg" alt="Host window during a question"><br><b>The answer before anyone else.</b> The current question, the correct answer and what the next click will do.</td>
    <td width="50%"><img src="docs/screenshots/host-leaderboard.jpg" alt="Host window during the leaderboard reveal"><br><b>Which way to look.</b> During the reveal the seating map shows where the announced teams sit.</td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/attendance.jpg" alt="Attendance page with the seating map"><br><b>Attendance.</b> Tick off who came, fix the number of players, and place each team on the map of the room.</td>
    <td><img src="docs/screenshots/check.jpg" alt="Quiz check page"><br><b>Check before the show.</b> Every question as the TV will show it, plus a list of what is missing or wrong.</td>
  </tr>
</table>

## Make your own quiz

<table>
  <tr>
    <td width="33%"><img src="docs/screenshots/library.jpg" alt="Quiz library"></td>
    <td width="33%"><img src="docs/screenshots/editor.jpg" alt="Quiz setup page"></td>
    <td width="33%"><img src="docs/screenshots/help.jpg" alt="Help page showing the sheet rows for a question type"></td>
  </tr>
</table>

1. **Open the [app](https://jakub-kos.github.io/QuizNightWebApp/)** and click **New quiz**, or start from the built-in demo.
2. **Questions:** copy the Google Sheet template (Help → Templates), fill it in, share it as *anyone with the link can view* and paste the link. Help → Question types shows the exact rows for every type and what they look like on screen.
3. **Media:** drag in your pictures, audio and video. A whole folder works.
4. **Scores:** paste the link of your scores Sheet, or type the points into the app during the show.
5. **Teams:** type them in, import a CSV, or load the names from the scores Sheet.
6. **Check**, fix what it finds, and **Start show**.

Export a quiz as one `.zip` to back it up or move it to the laptop you will use on the night.

## On the night

1. Connect the laptop to the TV, open the quiz with **Start show**, drag the window to the TV and press `F11`.
2. Press `Shift + P` and keep the host window on the laptop.
3. If the TV cuts off the edges, press `Shift + C` on the TV window to shrink the picture.

| Key | Does |
| --- | --- |
| `Shift + P` | Open the host window |
| `→` / `←` | Next / previous step |
| `Z` | Enlarge the question's picture |
| `Esc` | Close the picture, or leave the current screen |
| `Shift + D` | Jump back to the rounds dashboard |
| `Shift + C` | Adjust the picture size for the TV |

The app is in **English, Czech and Hungarian**. It works best in Chrome or Edge.

## For developers

React 19 + Vite, Tailwind CSS v4, Framer Motion, Lucide icons, PapaParse and fflate. There is no backend: quizzes and media live in IndexedDB, the two windows talk over `BroadcastChannel`, and Google Sheets are read as CSV.

```bash
git clone git@github.com:Jakub-Kos/QuizNightWebApp.git
cd QuizNightWebApp
npm install
npm run dev     # development server
npm run lint    # ESLint
npm run build   # production build in dist/
```

Every push to `main` deploys to GitHub Pages.

## License

MIT
