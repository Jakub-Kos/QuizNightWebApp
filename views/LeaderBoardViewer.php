<?php

class LeaderBoardViewer {
    public function render($questions) {
        if ($questions === null) {
            http_response_code(404);
            echo "";
            return;
        }

        // 1) Extract header & sort by total points descending
        $header = array_shift($questions);
        usort($questions, fn($a, $b) => (int)$b[count($b) - 1] - (int)$a[count($a) - 1]);

        // 2) Compute competition rankings (ties share same, next skips)
        $data = [];
        $prevPts = null;
        $prevPos = 0;
        foreach ($questions as $i => $row) {
            $pts = (int)$row[count($row) - 1];
            $pos = ($pts === $prevPts) ? $prevPos : ($i + 1);
            $data[] = [
                'position' => $pos,
                'name'     => $row[0],
                'points'   => $pts,
                'cells'    => $row
            ];
            $prevPts = $pts;
            $prevPos = $pos;
        }

        // 3) JSON-encode for JS
        $jsonData = json_encode($data, JSON_HEX_TAG);

        // 4) Add “Position” to header
        array_unshift($header, 'Pozice');

        // 5) Emit HTML + CSS + JS
        echo <<<HTML
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Leaderboard</title>
  <!-- Google Fonts -->
  <link href="https://fonts.googleapis.com/css2?family=League+Spartan:wght@400;700&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="../styles/leaderBoard_scifi.css">
</head>
<body>
  <div class="top-bar">
      <button id="reveal-btn">Zobrazit další tým</button>
      <div id="round-indicator"></div>
  </div>

  <div class="podium" id="podium">
    <div class="first"></div>
    <div class="second"></div>
    <div class="third"></div>
  </div>

  <div class="table-container">
    <table>
      <thead>
        <tr>
HTML;
        foreach ($header as $h) {
            echo "          <th>" . htmlspecialchars($h) . "</th>\n";
        }
        echo <<<HTML
        </tr>
      </thead>
      <tbody id="leader-body"></tbody>
    </table>
  </div>

  <script>
    document.addEventListener('DOMContentLoaded', function() {
      const teams = $jsonData;

      const numRounds = teams[0].cells.length - 2;               // skip “name” and “total”
      const maxScores = Array(numRounds).fill(0);
      teams.forEach(team => {
        for (let r = 1; r <= numRounds; r++) {
            const pts = parseInt(team.cells[r], 10);
            if (pts > maxScores[r - 1]) {
              maxScores[r - 1] = pts;
            }
        }
      });

      // — compute per‐round status (0=not started,1=in progress,2=finished)
      const roundStatus = [];
      const totalTeams  = teams.length;
      for (let r = 1; r <= numRounds; r++) {
        const filled = teams.filter(t => t.cells[r] !== '' && t.cells[r] != null).length;
        if      (filled === 0)          roundStatus.push(0);
        else if (filled < totalTeams)   roundStatus.push(1);
        else                             roundStatus.push(2);
      }

      // — build the indicator UI
      const indicator = document.getElementById('round-indicator');
      const textEl    = document.createElement('div');
      textEl.id       = 'round-text';
      indicator.appendChild(textEl);

      const boxesEl   = document.createElement('div');
      boxesEl.id      = 'round-boxes';
      indicator.appendChild(boxesEl);

      roundStatus.forEach(status => {
        const box = document.createElement('div');
        box.classList.add('round-rect',
          status === 2 ? 'finished'
        : status === 1 ? 'in-progress'
        : 'not-started'
        );
        boxesEl.appendChild(box);
      });

      // — helper to pick current round & label
      function updateRoundIndicator() {
        // count boxes that are either in-progress (1) or finished (2)
        const progressedCount = roundStatus.filter(s => s > 0).length;
        // any round currently in-progress?
        const inProg = roundStatus.includes(1);
        // build the text
        textEl.textContent = (progressedCount) + '/' + numRounds + ' kolo' + (inProg ? ' se opravuje...' : '');
      }
      updateRoundIndicator();

      const byPoints = teams.reduce((acc, t) => { (acc[t.points] = acc[t.points] || []).push(t); return acc; }, {});
      const sortedPts = Object.keys(byPoints).map(Number).sort((a,b) => a - b);

      let grpIdx = 0;
      const btn = document.getElementById('reveal-btn');
      const tbody = document.getElementById('leader-body');
      const podiumData = {1: [], 2: [], 3: []};
      const podiumEls = {
        1: document.querySelector('.first'),
        2: document.querySelector('.second'),
        3: document.querySelector('.third')
      };

      btn.addEventListener('click', function() {
        if (grpIdx >= sortedPts.length) {
          btn.disabled = true;
          btn.textContent = 'Všechny týmy zobrazeny';
          return;
        }
        const pts = sortedPts[grpIdx++];
        const group = byPoints[pts];

        group.forEach(function(team) {
          const tr = document.createElement('tr');
          tr.classList.add('new-row');

          tr.dataset.position = team.position;

          const tdP = document.createElement('td');
          tdP.textContent = team.position;
          tr.appendChild(tdP);
          team.cells.forEach(function(c, i) {
            const td = document.createElement('td');
            // append crown if this cell is a round and equals that round's max
            if (i > 0 && i <= numRounds && parseInt(c, 10) === maxScores[i - 1]) {
                td.textContent = c + ' 👑';
            } else {
                td.textContent = c;
            }
            tr.appendChild(td);
          });

          tbody.insertBefore(tr, tbody.firstChild);

          if (team.position <= 3) {
            podiumData[team.position].push(team);
            const inner = podiumData[team.position]
              .map(function(t) {
                return '<div class="medalist"><h2>' + t.name + '</h2></div>';
              })
              .join('') + '<p>Points: ' + podiumData[team.position][0].points + '</p>';
            podiumEls[team.position].innerHTML = inner;
            podiumEls[team.position].classList.add('visible');
          }

          tr.addEventListener('animationend', function() {
            tr.classList.remove('new-row');
          });
        });
      });
    });
  </script>

</body>
</html>
HTML;
    }
}
