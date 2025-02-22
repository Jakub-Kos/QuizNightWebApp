<?php
session_start();
$round = $_GET['round'] ?? 1;
$question = $_GET['q'] ?? 1;
$questions = json_decode(file_get_contents("questions.json"), true);

if (!isset($questions[$round][$question])) {
    header("Location: index.php");
    exit;
}
$qData = $questions[$round][$question];
?>
<!DOCTYPE html>
<html>
<head>
    <title>Round <?= $round ?> - Question <?= $question ?></title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div class="container">
        <h2>Round <?= $round ?> - Question <?= $question ?></h2>
        <p><?= $qData['question'] ?></p>
        <form action="submit_answer.php" method="post">
            <input type="hidden" name="round" value="<?= $round ?>">
            <input type="hidden" name="question" value="<?= $question ?>">
            <?php if ($qData['type'] == 'ABCD'): ?>
                <?php foreach (["A", "B", "C", "D"] as $option): ?>
                    <label><input type="radio" name="answer" value="<?= $option ?>"> <?= $option ?>: <?= $qData[$option] ?></label><br>
                <?php endforeach; ?>
            <?php elseif ($qData['type'] == 'Yes/No'): ?>
                <label><input type="radio" name="answer" value="Yes"> Yes</label>
                <label><input type="radio" name="answer" value="No"> No</label>
            <?php else: ?>
                <input type="text" name="answer">
            <?php endif; ?>
            <br><br>
            <button type="submit">Submit</button>
        </form>
        <br>
        <?php if ($question < count($questions[$round])): ?>
            <a href="round.php?round=<?= $round ?>&q=<?= $question + 1 ?>" class="btn">Next Question</a>
        <?php else: ?>
            <a href="index.php" class="btn">Back to Round Selection</a>
        <?php endif; ?>
    </div>
</body>
</html>