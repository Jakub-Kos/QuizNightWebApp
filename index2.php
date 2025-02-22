<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Quiz Night Výsledky</title>
    <link href="https://fonts.googleapis.com/css2?family=League+Spartan:wght@400;700&display=swap" rel="stylesheet">
    <style>
        body {
            font-family: 'League Spartan', sans-serif;
            margin: 0;
            background-color: #042335;
            color: white;
            display: flex;
            flex-direction: column;
            height: 100vh; /* Use full viewport height */
        }

        h1 {
            font-size: 3em;
            margin-bottom: 20px;
            text-align: center;
        }

        .podium {
            display: flex;
            justify-content: center;
            align-items: flex-end;
            gap: 20px;
            margin-bottom: 15px;
            flex: 1; /* Take up remaining space in the 1st 1/3rd section */
            justify-content: center;
        }

        .podium .first, .podium .second, .podium .third {
            text-align: center;
            padding: 10px;
            border-radius: 10px;
            flex-basis: 30%;
            color: #042335;
        }

        .podium .first {
            background-color: gold;
            font-weight: bold;
            order: 2;
        }

        .podium .second {
            background-color: silver;
            font-weight: bold;
            order: 1;
        }

        .podium .third {
            background-color: #cd7f32;
            font-weight: bold;
            order: 3;
        }

        .podium h2 {
            font-size: 2em;
        }

        .podium p {
            font-size: 1.1em;
        }

        /* Adjust table to take up remaining space */
        table {
            width: 100%;
            border-collapse: collapse;
            background-color: #dda434;
            color: #042335;
            font-size: 1.2em;
            flex: 2; /* Take up remaining space after podium */
            overflow: auto; /* Make sure it doesn't cause overflow */
            height: 100%; /* Table height takes up remaining space */
            margin-top: 20px;
        }

        th, td {
            border: 1px solid #042335;
            padding: 10px;
            padding-bottom: 2px;
        }

        th {
            background-color: #042335;
            color: white;
        }

        table td:nth-child(1), /* 1st column */
        table td:nth-child(2), /* 3rd column */
        table td:nth-child(3) /* 4th column */
        {
            background-color: #f8a90c; /* Column background color */
        }

        table th:nth-child(3) {
            width: 50px; /* Adjust this value for the desired width */
        }

        /* Highlight rows that changed position */
        .row-changed {
            animation: highlight 2s ease-in-out;
        }

        /* Define the animation */
        @keyframes highlight {
            0% {
                background-color: #f4e04d; /* Highlight color */
            }
            100% {
                background-color: transparent; /* Fade back to default */
            }
        }

        /* Animation for podium changes */
        .podium-changed {
            animation: bounce 1.5s ease-in-out;
        }

        /* Define the bounce animation */
        @keyframes bounce {
            0%, 20%, 50%, 80%, 100% {
                transform: translateY(0);
            }
            40% {
                transform: translateY(-20px);
            }
            60% {
                transform: translateY(-10px);
            }
        }

    </style>
</head>
<body>
    <h1>Quiz Night Výsledky</h1>
    
    <?php
    // Read the CSV file
    $csvFile = 'resultsU.csv'; // Ensure this file is in the same directory
    $csvUrl = "https://docs.google.com/spreadsheets/d/e/2PACX-1vTZRpz_6pN57XyFAtRSzn7Dvgdyp5AI972sr9fizOREi7P4-12Te_dIK75vfj5OeHb6Q_AasxqXZ9Aw/pub?gid=2040116638&single=true&output=csv";
    $data = array_map('str_getcsv', file($csvUrl));

    //$data = array_map('str_getcsv', file($csvFile));
    $headers = array_shift($data);

    // Sort data by "Počet bodů" (column 1) in descending order
    usort($data, function ($a, $b) {
        return $b[1] - $a[1];
    });

    // Display the podium
    echo '<div class="podium">';
    $podium_classes = ['first', 'second', 'third'];
    foreach ($podium_classes as $index => $class) {
        $team = $data[$index];
        echo "<div class='$class' data-team='" . htmlspecialchars($team[0]) . "'>";
        echo "<h2>" . htmlspecialchars($team[0]) . "</h2>";
        echo "<p>Points: " . htmlspecialchars($team[1]) . "</p>";
        echo "</div>";
    }
    echo '</div>';

    // Display the table
    echo '<table>';
    echo '<thead>';
    echo '<tr>';
    foreach ($headers as $header) {
        echo '<th>' . htmlspecialchars($header) . '</th>';
    }
    echo '</tr>';
    echo '</thead>';
    echo '<tbody>';
    foreach ($data as $row) {
        echo '<tr>';
        foreach ($row as $cell) {
            echo '<td>' . htmlspecialchars($cell) . '</td>';
        }
        echo '</tr>';
    }
    echo '</tbody>';
    echo '</table>';
    ?>
    
<script>
document.addEventListener("DOMContentLoaded", function () {
    // Get all rows from the table body
    const tableRows = Array.from(document.querySelectorAll("table tbody tr"));

    // Retrieve the previous order from localStorage
    const previousOrder = JSON.parse(localStorage.getItem("rowOrder")) || [];

    // Get the current order of rows (based on the first column, team names)
    const currentOrder = tableRows.map(row => row.cells[0].innerText);

    // Save the current order to localStorage for the next refresh
    localStorage.setItem("rowOrder", JSON.stringify(currentOrder));

    // Compare the current order with the previous order
    tableRows.forEach((row, index) => {
        const previousIndex = previousOrder.indexOf(currentOrder[index]);

        // If the row's position changed, add a highlight animation class
        if (previousIndex !== -1 && previousIndex !== index) {
            row.classList.add("row-changed");
        }
    });

    // Get the podium elements
    const podiums = document.querySelectorAll(".podium > div");

    // Retrieve the previous podium order from localStorage
    const previousPodium = JSON.parse(localStorage.getItem("podiumOrder")) || [];

    // Get the current podium order (team names in `data-team`)
    const currentPodium = Array.from(podiums).map(podium => podium.dataset.team);

    // Save the current podium order to localStorage
    localStorage.setItem("podiumOrder", JSON.stringify(currentPodium));

    // Compare previous and current podium orders to animate changes
    podiums.forEach((podium, index) => {
        const previousIndex = previousPodium.indexOf(currentPodium[index]);

        if (previousIndex !== -1 && previousIndex !== index) {
            // If the podium's position changed, add an animation class
            podium.classList.add("podium-changed");
        }
    });
});
</script>

</body>
</html>
