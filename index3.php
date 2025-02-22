<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Quiz Night</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div class="container">
        <h1>Quiz Night</h1>
        <div class="round-buttons">
            <?php for ($i = 1; $i <= 6; $i++): ?>
                <a href="round.php?round=<?= $i ?>" class="btn">Round <?= $i ?></a>
            <?php endfor; ?>
        </div>
        <a href="leaderboard.php" class="btn">Leaderboard</a>
    </div>
</body>
</html>