<?php

class LeaderBoardViewer {
    function render($questions){
        if ($questions === null) {
            http_response_code(404);
            echo "";
            return;
        }
    
        echo "
        <!DOCTYPE html>
        <html lang='en'>
        <head>
            <meta charset='UTF-8'>
            <title>Questions</title>
            <link rel='stylesheet' type='text/css' href='./views/leaderBoard.css'>
        </head> ";
        // Extract header
        $header = array_shift($questions);

        // Sort by last column (Počet bodů)
        usort($questions, function ($a, $b) {
            return (int)$b[count($b) - 1] - (int)$a[count($a) - 1];
        });

        // Display the podium
        echo '<div class="podium">';
        $podium_classes = ['first', 'second', 'third'];
        foreach ($podium_classes as $index => $class) {
            $team = $questions[$index];
            echo "<div class='$class'>";
            echo "<h2>" . htmlspecialchars($team[0]) . "</h2>";
            echo "<p>Points: " . htmlspecialchars($team[1]) . "</p>";
            echo "</div>";
        }
        echo '</div>';

        // Display the table
        echo '<table>';
        echo '<tr>';
        foreach ($header as $head) {
            echo '<th>' . htmlspecialchars($head) . '</th>';
        }
        echo '</tr>';
        foreach ($questions as $row) {
            echo '<tr>';
            foreach ($row as $cell) {
                echo '<td>' . htmlspecialchars($cell) . '</td>';
            }
            echo '</tr>';
        }
        echo '</table>';
    }
}